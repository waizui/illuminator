use crate::img::*;
use image::{Rgb, RgbImage};

impl PixelType for Rgb<u8> {
    fn from(c: &[f32; 3]) -> Self {
        let r = (c[0] * 255.).clamp(0., 255.) as u8;
        let g = (c[1] * 255.).clamp(0., 255.) as u8;
        let b = (c[2] * 255.).clamp(0., 255.) as u8;
        Rgb([r, g, b])
    }
}

impl From<RgbImage> for Image<Rgb<u8>> {
    fn from(value: RgbImage) -> Self {
        let (w, h) = (value.width() as usize, value.height() as usize);
        let size = w * h;
        let mut data: Vec<Rgb<u8>> = Vec::with_capacity(size);
        for p in value.pixels() {
            data.push(*p);
        }

        Image { raw: data, w, h }
    }
}

impl From<Image<Rgb<u8>>> for RgbImage {
    fn from(value: Image<Rgb<u8>>) -> Self {
        let mut img = RgbImage::new(value.w as u32, value.h as u32);
        img.pixels_mut()
            .enumerate()
            .for_each(|(i, p)| *p = value.raw[i]);
        img
    }
}

#[test]
fn test_image() {
    use image::Pixel;
    use image::RgbImage;

    let mut img = Image::from(RgbImage::new(100, 100));
    assert_eq!(img[(49, 49)].0, [0; 3]);

    img[(49, 49)] = *Rgb::from_slice(&[1; 3]);
    assert_eq!(img[(49, 49)].0, [1; 3]);

    assert_eq!(img[(49, 49)].0, [1; 3]);

    let img2 = Image::from(RgbImage::new(100, 100));
    let img = img.stitch_hor(&img2);

    assert_eq!((img.w, img.h), (200, 100));
    assert_eq!(img[(49, 49)].0, [1; 3]);
    assert_eq!(img[(100, 99)].0, [0; 3]);
}
