use image::{DynamicImage, Rgb, RgbImage, imageops::FilterType};

use crate::img::*;

pub struct VirtualTexture {
    pub mips: Vec<RawImage<Rgb<u8>>>,
}

impl VirtualTexture {
    pub fn new(mip_level: usize, img: &RawImage<Rgb<u8>>, min_wh: usize) -> Self {
        let mut vt = VirtualTexture { mips: Vec::new() };
        vt.gen_mipmaps(mip_level, img, min_wh);
        vt
    }

    fn gen_mipmaps(&mut self, level: usize, img: &RawImage<Rgb<u8>>, min_wh: usize) {
        let mut w = img.w;
        let mut h = img.h;

        let mut cur_lever = level;

        while w > min_wh && h > min_wh && cur_lever > 0 {
            w = (w / 2).max(1);
            h = (h / 2).max(1);
            cur_lever -= 1;
            let resized = self.resize(w, h, img);
            self.mips.push(resized);
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
fn testvt() {
    use image::RgbImage;

    let test_img: RawImage<Rgb<u8>> = RawImage::checkerboard(512, 512, 32);

    let vt = VirtualTexture::new(3, &test_img, 64);

    let mut mip_level = 1;
    for mip in vt.mips {
        let rgb = RgbImage::from(mip);
        rgb.save(format!("./target/vt_text_{mip_level}.png"))
            .expect("test failed of virtual texture");
        mip_level += 1;
    }
}
