use core::fmt;
use std::{error::Error, io};

#[derive(Debug)]
/// An error produced when using [`CustomCursor::from_rgba`] with invalid arguments.
pub enum BadCursor {
    /// Produced when the length of the `rgba` argument isn't divisible by 4, thus `rgba` can't be
    /// safely interpreted as 32bpp RGBA pixels.
    ByteCountNotDivisibleBy4 { byte_count: usize },
    /// Produced when the number of pixels (`rgba.len() / 4`) isn't equal to `width * height`.
    /// At least one of your arguments is incorrect.
    DimensionsVsPixelCount {
        width: u32,
        height: u32,
        width_x_height: usize,
        pixel_count: usize,
    },
    /// Produced when the hotspot is outside the image bounds.
    HotspotOutOfBounds {
        width: u32,
        height: u32,
        hotspot_x: u32,
        hotspot_y: u32,
    },
    /// Produced when underlying OS functionality failed to create the image.
    OsError(io::Error),
    /// TODO
    OtherError,
}

impl fmt::Display for BadCursor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BadCursor::ByteCountNotDivisibleBy4 { byte_count } => write!(f,
                "The length of the `rgba` argument ({byte_count:?}) isn't divisible by 4, making it impossible to interpret as 32bpp RGBA pixels.",
            ),
            BadCursor::DimensionsVsPixelCount {
                width,
                height,
                width_x_height,
                pixel_count,
            } => write!(f,
                "The specified dimensions ({width:?}x{height:?}) don't match the number of pixels supplied by the `rgba` argument ({pixel_count:?}). For those dimensions, the expected pixel count is {width_x_height:?}.",
            ),
            BadCursor::HotspotOutOfBounds {
                width,
                height,
                hotspot_x,
                hotspot_y,
            } => write!(f,
                "The specified hotspot ({hotspot_x:?}, {hotspot_y:?}) is outside the image bounds ({width:?}x{height:?}).",
            ),
            BadCursor::OsError(e) => write!(f, "OS error when instantiating the image: {e:?}"),
            BadCursor::OtherError => write!(f, "Other error"),
        }
    }
}

impl Error for BadCursor {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(self)
    }
}

use crate::{icon::PIXEL_SIZE, platform_impl::PlatformCustomCursor};

fn validate_rgba_cursor(
    rgba: &[u8],
    width: u32,
    height: u32,
    hotspot_x: u32,
    hotspot_y: u32,
) -> Result<(), BadCursor> {
    if rgba.len() % PIXEL_SIZE != 0 {
        return Err(BadCursor::ByteCountNotDivisibleBy4 {
            byte_count: rgba.len(),
        });
    }
    let pixel_count = rgba.len() / PIXEL_SIZE;
    if pixel_count != (width * height) as usize {
        return Err(BadCursor::DimensionsVsPixelCount {
            width,
            height,
            width_x_height: (width * height) as usize,
            pixel_count,
        });
    }

    if hotspot_x >= width || hotspot_y >= height {
        return Err(BadCursor::HotspotOutOfBounds {
            width,
            height,
            hotspot_x,
            hotspot_y,
        });
    }

    Ok(())
}

#[derive(Debug, Clone)]
pub struct CustomCursor {
    pub(crate) inner: PlatformCustomCursor,
}

impl CustomCursor {
    pub fn from_rgba(
        rgba: Vec<u8>,
        width: u32,
        height: u32,
        hotspot_x: u32,
        hotspot_y: u32,
    ) -> Result<Self, BadCursor> {
        validate_rgba_cursor(&rgba, width, height, hotspot_x, hotspot_y)?;

        Ok(Self {
            inner: PlatformCustomCursor::from_rgba(rgba, width, height, hotspot_x, hotspot_y)?,
        })
    }
}

#[derive(Debug, Clone)]
pub(crate) struct NoCustomCursor;

impl NoCustomCursor {
    pub fn from_rgba(
        rgba: Vec<u8>,
        width: u32,
        height: u32,
        hotspot_x: u32,
        hotspot_y: u32,
    ) -> Result<Self, BadCursor> {
        validate_rgba_cursor(&rgba, width, height, hotspot_x, hotspot_y)?;
        Ok(Self)
    }
}
