use crate::img::*;
use std::ops::{Index, IndexMut};

impl<P> Index<(usize, usize)> for RawImage<P>
where
    P: PixelType,
{
    type Output = P;
    fn index(&self, index: (usize, usize)) -> &Self::Output {
        &self.raw[self.w * index.1 + index.0]
    }
}

impl<P> IndexMut<(usize, usize)> for RawImage<P>
where
    P: PixelType,
{
    fn index_mut(&mut self, index: (usize, usize)) -> &mut Self::Output {
        &mut self.raw[self.w * index.1 + index.0]
    }
}

pub trait ImgOp<P>
where
    P: PixelType,
{
    fn stitch_hor(&self, other: &Self) -> Self;
}

impl<P> ImgOp<P> for RawImage<P>
where
    P: PixelType,
{
    fn stitch_hor(&self, other: &Self) -> Self {
        let (w0, h0) = self.shape();
        let (w1, h1) = other.shape();
        let (w, h) = (w0 + w1, h0.max(h1));

        let mut img = RawImage::new(w, h);

        img.par_iter_pixel(|(ipix, pix)| {
            let x = ipix % w;
            let y = ipix / w;

            if x < w0 && y < h0 {
                // pixel belongs to the first image
                *pix = self[(x, y)];
            } else if x >= w0 && y < h1 {
                let x1 = x - w0;
                *pix = other[(x1, y)];
            }
            // else leave as default value from T::new()
        });

        img
    }
}
