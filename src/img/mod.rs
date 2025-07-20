pub mod ops;
pub mod rgbimage;

pub use crate::img::ops::ImgOp;

pub trait PixelType: Copy + Send + Sync {
    fn from(c: &[u8; 3]) -> Self;
}

///image type provides unified operation interface
pub struct Image<P: PixelType> {
    /// orginal image
    raw: Vec<P>,
    w: usize,
    h: usize,
}

impl<P: PixelType> Image<P> {
    pub fn new(w: usize, h: usize) -> Self {
        let data = vec![P::from(&[0; 3]); w * h];
        Image { raw: data, w, h }
    }

    pub fn shape(&self) -> (usize, usize) {
        (self.w, self.h)
    }

    pub fn data(&self) -> &[P] {
        &self.raw
    }
    pub fn data_mut(&mut self) -> &mut [P] {
        &mut self.raw
    }
}
