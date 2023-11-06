use std::sync::Arc;

use crate::{
    custom_cursor::BadCursor,
    platform_impl::{wayland::WaylandCustomCursor, x11::util::X11CustomCursor},
};

#[derive(Debug, Clone)]
pub struct LinuxCustomCursor {
    #[cfg(wayland_platform)]
    pub(crate) wayland: Arc<WaylandCustomCursor>,
    #[cfg(x11_platform)]
    /// Option because x11 may not be available
    pub(crate) x11: Option<Arc<X11CustomCursor>>,
}

impl LinuxCustomCursor {
    pub fn from_rgba(
        mut rgba: Vec<u8>,
        width: u32,
        height: u32,
        hotspot_x: u32,
        hotspot_y: u32,
    ) -> Result<Self, BadCursor> {
        // We have to create cursors for both backends, since we don't know which one
        // will be used in the event loop.

        // Swap rgba to bgra
        rgba.chunks_exact_mut(4).for_each(|chunk| chunk.swap(0, 2));

        let x11 = match X11CustomCursor::new(&rgba, width, height, hotspot_x, hotspot_y) {
            Ok(x11) => Some(Arc::new(x11)),
            // If we failed to create a x11 cursor because x11 is not available. We should still
            // support wayland, so we don't return an error. The user cannot create a x11 event loop
            // either so it should not be a problem.
            Err(_) => None,
        };

        #[cfg(wayland_platform)]
        let wayland = Arc::new(WaylandCustomCursor {
            bgra_bytes: rgba,
            w: width as i32,
            h: height as i32,
            hot_x: hotspot_x as i32,
            hot_y: hotspot_y as i32,
        });

        Ok(Self {
            #[cfg(wayland_platform)]
            wayland,
            #[cfg(x11_platform)]
            x11,
        })
    }
}
