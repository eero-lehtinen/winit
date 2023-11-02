use core::slice;
use std::{ffi::CString, io::Cursor};

use x11rb::connection::Connection;

use crate::window::CursorIcon;

use super::*;

impl XConnection {
    pub fn set_cursor_icon(&self, window: xproto::Window, cursor: Option<CursorIcon>) {
        let cursor = *self
            .cursor_cache
            .lock()
            .unwrap()
            .entry(cursor)
            .or_insert_with(|| self.get_cursor(cursor));

        self.update_cursor(window, cursor)
            .expect("Failed to set cursor");
    }

    pub fn set_custom_cursor_icon(&self, window: xproto::Window, key: u64) {
        let cursor = *self
            .custom_cursors
            .lock()
            .unwrap()
            .entry(key)
            .or_insert_with(|| self.create_empty_cursor());

        self.update_cursor(window, cursor)
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
        key: u64,
        png_bytes: Vec<u8>,
        hot_x: u32,
        hot_y: u32,
    ) {
        let decoder = png::Decoder::new(Cursor::new(png_bytes));

        let mut reader = decoder.read_info().unwrap();
        let info = reader.info();

        if info.color_type != png::ColorType::Rgba || info.bit_depth != png::BitDepth::Eight {
            panic!("Invalid png (8bit rgba required)");
        }

        let (w, h) = info.size();

        unsafe {
            let image = (self.xcursor.XcursorImageCreate)(w as i32, h as i32);
            if image.is_null() {
                panic!("failed to allocate cursor image");
            }
            (*image).xhot = hot_x;
            (*image).yhot = hot_y;
            (*image).delay = 0;

            let dst = slice::from_raw_parts_mut((*image).pixels, (w * h) as usize);
            let mut i = 0;
            while let Ok(Some(row)) = reader.next_row() {
                for chunk in row.data().chunks_exact(4) {
                    // "Each pixel in the cursor is a 32-bit value containing ARGB with A in the high byte"
                    // So it basically wants BGRA and we have RGBA.
                    let mut chunk: [u8; 4] = chunk.try_into().unwrap();
                    chunk.swap(0, 2);
                    dst[i] = std::mem::transmute(chunk);
                    i += 1;
                }
            }

            let cursor = (self.xcursor.XcursorImageLoadCursor)(self.display, image);
            (self.xcursor.XcursorImageDestroy)(image);

            let mut cursors = self.custom_cursors.lock().unwrap();
            cursors.insert(key, cursor);
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
