use core::slice;
use std::{ffi::CString, sync::Arc};

use x11rb::connection::Connection;

use crate::{
    platform_impl::{XNotSupported, X11_BACKEND},
    window::CursorIcon,
};

use super::*;

#[derive(Debug, PartialEq, Eq)]
pub struct X11CustomCursor(ffi::Cursor);

impl X11CustomCursor {
    pub fn new(
        bgra_bytes: &[u8],
        w: u32,
        h: u32,
        hot_x: u32,
        hot_y: u32,
    ) -> Result<Self, XNotSupported> {
        let xconn_lock = X11_BACKEND.lock().unwrap();
        let xconn = (*xconn_lock).clone()?;

        let xcursor = &xconn.xcursor;
        let display = xconn.display;

        unsafe {
            let image = (xcursor.XcursorImageCreate)(w as i32, h as i32);
            (*image).xhot = hot_x;
            (*image).yhot = hot_y;
            (*image).delay = 0;

            let dst = slice::from_raw_parts_mut((*image).pixels, (w * h) as usize);
            for (i, chunk) in bgra_bytes.chunks_exact(4).enumerate() {
                dst[i] = (chunk[0] as u32)
                    | (chunk[1] as u32) << 8
                    | (chunk[2] as u32) << 16
                    | (chunk[3] as u32) << 24;
            }

            let cursor = (xcursor.XcursorImageLoadCursor)(display, image);
            (xcursor.XcursorImageDestroy)(image);
            Ok(Self(cursor))
        }
    }
}

impl Drop for X11CustomCursor {
    fn drop(&mut self) {
        let xconn_lock = X11_BACKEND.lock().unwrap();
        if let Ok(xconn) = &(*xconn_lock) {
            let xlib = &xconn.xlib;
            let display = xconn.display;
            unsafe {
                (xlib.XFreeCursor)(display, self.0);
            }
        }
    }
}

#[derive(Debug, Clone)]
pub enum SelectedCursor {
    BuiltIn(CursorIcon),
    Custom(Arc<X11CustomCursor>),
}

impl Default for SelectedCursor {
    fn default() -> Self {
        SelectedCursor::BuiltIn(Default::default())
    }
}

impl PartialEq for SelectedCursor {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::BuiltIn(a), Self::BuiltIn(b)) => a == b,
            (Self::Custom(a), Self::Custom(b)) => Arc::ptr_eq(a, b),
            _ => false,
        }
    }
}

impl Eq for SelectedCursor {}

impl XConnection {
    pub fn set_cursor_icon(&self, window: xproto::Window, cursor_icon: Option<CursorIcon>) {
        let cursor = *self
            .cursor_cache
            .lock()
            .unwrap()
            .entry(cursor_icon)
            .or_insert_with(|| self.get_cursor(cursor_icon));

        self.update_cursor(window, cursor)
            .expect("Failed to set cursor");
    }

    pub fn set_custom_cursor(&self, window: xproto::Window, cursor: &X11CustomCursor) {
        self.update_cursor(window, cursor.0)
            .expect("Failed to set cursor");
    }

    fn create_empty_cursor(&self) -> ffi::Cursor {
        let data = 0;
        let pixmap = unsafe {
            let screen = (self.xlib.XDefaultScreen)(self.display);
            let window = (self.xlib.XRootWindow)(self.display, screen);
            (self.xlib.XCreateBitmapFromData)(self.display, window, &data, 1, 1)
        };

        if pixmap == 0 {
            panic!("failed to allocate pixmap for cursor");
        }

        unsafe {
            // We don't care about this color, since it only fills bytes
            // in the pixmap which are not 0 in the mask.
            let mut dummy_color = MaybeUninit::uninit();
            let cursor = (self.xlib.XCreatePixmapCursor)(
                self.display,
                pixmap,
                pixmap,
                dummy_color.as_mut_ptr(),
                dummy_color.as_mut_ptr(),
                0,
                0,
            );
            (self.xlib.XFreePixmap)(self.display, pixmap);

            cursor
        }
    }

    fn get_cursor(&self, cursor: Option<CursorIcon>) -> ffi::Cursor {
        let cursor = match cursor {
            Some(cursor) => cursor,
            None => return self.create_empty_cursor(),
        };

        let name = CString::new(cursor.name()).unwrap();
        unsafe {
            (self.xcursor.XcursorLibraryLoadCursor)(self.display, name.as_ptr() as *const c_char)
        }
    }

    fn update_cursor(&self, window: xproto::Window, cursor: ffi::Cursor) -> Result<(), X11Error> {
        self.xcb_connection()
            .change_window_attributes(
                window,
                &xproto::ChangeWindowAttributesAux::new().cursor(cursor as xproto::Cursor),
            )?
            .ignore_error();

        self.xcb_connection().flush()?;
        Ok(())
    }
}
