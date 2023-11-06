use icrate::Foundation::{NSPoint, NSSize};
use objc2::rc::Id;

use crate::custom_cursor::BadCursor;

use super::appkit::{NSBitmapImageRep, NSCursor, NSImage};

#[derive(Debug, Clone)]
pub struct MacosCustomCursor {
    pub(crate) inner: Id<NSCursor>,
}

impl MacosCustomCursor {
    pub fn from_rgba(
        rgba: Vec<u8>,
        width: u32,
        height: u32,
        hotspot_x: u32,
        hotspot_y: u32,
    ) -> Result<Self, BadCursor> {
        let bitmap = NSBitmapImageRep::init_rgba(width as isize, height as isize);
        let bitmap_data = unsafe {
            std::slice::from_raw_parts_mut(bitmap.bitmap_data(), (width * height * 4) as usize)
        };
        bitmap_data.copy_from_slice(&rgba);

        let image = NSImage::init_with_size(NSSize::new(width.into(), height.into()));
        image.add_representation(&bitmap);

        let hotspot = NSPoint::new(hotspot_x as f64, hotspot_y as f64);

        Ok(Self {
            inner: NSCursor::new(&image, hotspot),
        })
    }
}
