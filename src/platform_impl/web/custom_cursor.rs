use std::sync::Arc;

use wasm_bindgen::{Clamped, JsCast};
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement};

use crate::custom_cursor::BadCursor;

#[derive(Debug, Clone)]
pub struct WebCustomCursor {
    pub(crate) inner: Arc<str>,
}

impl WebCustomCursor {
    pub fn from_rgba(
        rgba: Vec<u8>,
        width: u32,
        height: u32,
        hotspot_x: u32,
        hotspot_y: u32,
    ) -> Result<Self, BadCursor> {
        #[allow(clippy::disallowed_methods)]
        let cursor_icon_canvas = web_sys::window()
            .unwrap()
            .document()
            .unwrap()
            .create_element("canvas")
            .unwrap()
            .dyn_into::<HtmlCanvasElement>()
            .unwrap();

        #[allow(clippy::disallowed_methods)]
        cursor_icon_canvas.set_width(width);
        #[allow(clippy::disallowed_methods)]
        cursor_icon_canvas.set_height(height);

        let context = cursor_icon_canvas
            .get_context("2d")
            .unwrap()
            .unwrap()
            .dyn_into::<CanvasRenderingContext2d>()
            .unwrap();

        let image_data =
            web_sys::ImageData::new_with_u8_clamped_array_and_sh(Clamped(&rgba), width, height);

        context
            .put_image_data(&image_data.unwrap(), 0.0, 0.0)
            .unwrap();

        let data_url = cursor_icon_canvas.to_data_url().unwrap();

        Ok(Self {
            inner: format!("url({data_url}) {hotspot_x} {hotspot_y}, auto").into(),
        })
    }
}
