pub mod ops;
pub mod rgb;
pub mod vt;

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

impl<P: PixelType> RawImage<P> {
    pub fn new(w: usize, h: usize) -> Self {
        let data = vec![P::from(&[0.; 3]); w * h];
        RawImage { raw: data, w, h }
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

    pub fn checkerboard(w: usize, h: usize, squre_size: usize) -> Self {
        use rayon::prelude::*;
        let mut img: RawImage<P> = RawImage::new(w, h);
        img.data_mut()
            .par_iter_mut()
            .enumerate()
            .for_each(|(i, pix)| {
                let r = i / w;
                let c = i - r * w;
                if (r / squre_size + c / squre_size).is_multiple_of(2) {
                    *pix = P::from(&[0.; 3]);
                } else {
                    *pix = P::from(&[1.; 3]);
                }
            });

        img
    }
}
