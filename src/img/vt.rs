use anyhow::{Ok, Result, anyhow};
use image::{DynamicImage, Rgb, RgbImage, imageops::FilterType};

use crate::img::*;

pub struct VirtualTexture {
    pub mips: Vec<RawImage<Rgb<u8>>>,
}

impl VirtualTexture {
    pub fn new(mip_level: usize, img: &RawImage<Rgb<u8>>) -> Result<Self> {
        const MIN_WH: usize = 64;

        let mut vt = VirtualTexture { mips: Vec::new() };
        if !img.w.is_multiple_of(2) || !img.h.is_multiple_of(2) {
            return Err(anyhow!(
                "err: Can not create VirtualTexture with non-power-of-2 image"
            ));
        }

        vt.gen_mipmaps(mip_level, img, MIN_WH);

        Ok(vt)
    }

    fn gen_mipmaps(&mut self, level: usize, img: &RawImage<Rgb<u8>>, min_wh: usize) {
        let mut w = img.w;
        let mut h = img.h;

        let mut cur_lever = level;

        while w > min_wh && h > min_wh && cur_lever > 0 {
            let resized = self.resize(w, h, img);
            self.mips.push(resized);

            w /= 2;
            h /= 2;
            cur_lever -= 1;
        }
    }

    fn resize(&self, w: usize, h: usize, img: &RawImage<Rgb<u8>>) -> RawImage<Rgb<u8>> {
        let dynimg = {
            let mut cloned = RgbImage::new(img.w as u32, img.h as u32);
            cloned
                .pixels_mut()
                .enumerate()
                .for_each(|(i, p)| *p = img.raw[i]);
            DynamicImage::ImageRgb8(cloned)
        };

        let rgb = dynimg
            .resize(w as u32, h as u32, FilterType::Lanczos3)
            .to_rgb8();
        RawImage::from(rgb)
    }
}

#[test]
fn testvt() -> Result<()> {
    use image::RgbImage;

    let test_img: RawImage<Rgb<u8>> = RawImage::checkerboard(512, 512, 32, &[0.; 3], &[1.; 3]);
    let vt = VirtualTexture::new(3, &test_img)?;

    for (i, mip) in vt.mips.into_iter().enumerate() {
        let rgb = RgbImage::from(mip);
        rgb.save(format!("./target/vt_text_{i}.png"))
            .expect("test failed of virtual texture");
    }

    Ok(())
}
