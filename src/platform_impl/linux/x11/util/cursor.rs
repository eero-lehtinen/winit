use core::slice;
use std::ffi::CString;

use x11rb::connection::Connection;

use crate::{cursor_image::CursorImage, window::CursorIcon};

use super::*;

#[derive(Debug, Clone, Copy)]
pub struct CustomCursor(ffi::Cursor);

impl CustomCursor {
    unsafe fn new(
        xcursor: &ffi::Xcursor,
        display: *mut ffi::Display,
        bgra_bytes: &[u8],
        w: u32,
        h: u32,
        hot_x: u32,
        hot_y: u32,
    ) -> Self {
        unsafe {
            let image = (xcursor.XcursorImageCreate)(w as i32, h as i32);
            if image.is_null() {
                panic!("failed to allocate cursor image");
            }
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
            CustomCursor(cursor)
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SelectedCursor {
    BuiltIn(CursorIcon),
    Custom(u64),
}

impl Default for SelectedCursor {
    fn default() -> Self {
        SelectedCursor::BuiltIn(Default::default())
    }
}

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

    pub fn set_custom_cursor_icon(&self, window: xproto::Window, key: u64) {
        let Some(cursor) = self.custom_cursors.lock().unwrap().get(&key).copied() else {
            return;
        };

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

    pub fn register_custom_cursor_icon(
        &self,
        window: xproto::Window,
        key: u64,
        mut image: CursorImage,
    ) {
        // Swap to bgra
        image
            .rgba
            .chunks_exact_mut(4)
            .for_each(|chunk| chunk.swap(0, 2));

        let new_cursor = unsafe {
            CustomCursor::new(
                &self.xcursor,
                self.display,
                &image.rgba,
                image.width,
                image.height,
                image.hotspot_x,
                image.hotspot_y,
            )
        };
        let mut cursors = self.custom_cursors.lock().unwrap();
        if let Some(cursor) = cursors.get(&key) {
            if *self.selected_cursor.lock().unwrap() == SelectedCursor::Custom(key) {
                self.update_cursor(window, new_cursor.0).unwrap();
            }
            unsafe {
                (self.xlib.XFreeCursor)(self.display, cursor.0);
            }
        }
        cursors.insert(key, new_cursor);
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
