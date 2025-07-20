use rayon::prelude::*;
use std::ops::{Index, IndexMut};

use crate::img::*;

fn uv2index<P>(img: &Image<P>, u: f32, v: f32) -> (usize, usize)
where
    P: PixelType,
{
    let iw = u * img.w as f32 - 0.5;
    let ih = v * img.h as f32 - 0.5;

    (iw as usize, ih as usize)
}

impl<P> Index<(f32, f32)> for Image<P>
where
    P: PixelType,
{
    type Output = P;
    fn index(&self, index: (f32, f32)) -> &Self::Output {
        let (iw, ih) = uv2index(self, index.0, index.1);
        &self.raw[self.w * ih + iw]
    }
}

impl<P> IndexMut<(f32, f32)> for Image<P>
where
    P: PixelType,
{
    fn index_mut(&mut self, index: (f32, f32)) -> &mut Self::Output {
        let (iw, ih) = uv2index(self, index.0, index.1);
        &mut self.raw[self.w * ih + iw]
    }
}

impl<P> Index<(usize, usize)> for Image<P>
where
    P: PixelType,
{
    type Output = P;
    fn index(&self, index: (usize, usize)) -> &Self::Output {
        &self.raw[self.w * index.1 + index.0]
    }
}

impl<P> IndexMut<(usize, usize)> for Image<P>
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

impl<P> ImgOp<P> for Image<P>
where
    P: PixelType,
{
    fn stitch_hor(&self, other: &Self) -> Self {
        let (w0, h0) = self.shape();
        let (w1, h1) = other.shape();
        let (w, h) = (w0 + w1, h0.max(h1));

        let mut img = Image::new(w, h);

        img.raw.par_iter_mut().enumerate().for_each(|(ipix, pix)| {
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
