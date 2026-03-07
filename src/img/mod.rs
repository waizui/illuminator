pub mod ops;
pub mod rgb;
pub mod vt;
pub mod img_impl;

pub use crate::img::ops::ImgOp;

pub trait PixelType: Copy + Send + Sync {
    fn from(c: &[f32; 3]) -> Self;
}

///image type provides unified operation interface
pub struct RawImage<P: PixelType> {
    /// orginal image
    raw: Vec<P>,
    w: usize,
    h: usize,
}

