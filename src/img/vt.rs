use anyhow::{Result, anyhow};
use image::{DynamicImage, Rgb, RgbImage, imageops::FilterType};

use crate::img::*;

pub fn ensure_pot(x: usize) -> Result<()> {
    if !x.is_multiple_of(2) {
        return Err(anyhow!(
            "err: Can not create VirtualTexture with non-power-of-2 image"
        ));
    }

    Ok(())
}

pub struct VirtualTexture {
    page_size: usize,
    virtual_page_axis: usize,
    physical_page_axis: usize,
    page_table: Vec<usize>,
    physcial_pages: Vec<RawImage<Rgb<u8>>>,
}

impl VirtualTexture {
    pub fn new(
        page_size: usize,
        virtual_page_axis: usize,
        physical_page_axis: usize,
    ) -> Result<Self> {
        ensure_pot(page_size)?;
        ensure_pot(virtual_page_axis)?;
        ensure_pot(physical_page_axis)?;

        let vt = VirtualTexture {
            page_size,
            virtual_page_axis,
            physical_page_axis,
            page_table: vec![virtual_page_axis.pow(2)],
            physcial_pages: vec![RawImage::new(page_size, page_size); physical_page_axis.pow(2)],
        };

        Ok(vt)
    }

    pub fn sample(&self, uv: &[f32; 2]) -> Rgb<u8> {
        todo!()
    }
}

fn resize(w: usize, h: usize, img: &RawImage<Rgb<u8>>) -> RawImage<Rgb<u8>> {
    let dynimg = {
        let cloned = RgbImage::from(img.clone());
        DynamicImage::ImageRgb8(cloned)
    };

    let rgb = dynimg
        .resize(w as u32, h as u32, FilterType::Lanczos3)
        .to_rgb8();
    RawImage::from(rgb)
}

fn make_pages(img: &RawImage<Rgb<u8>>, page_size: usize) -> Vec<RawImage<Rgb<u8>>> {
    todo!()
}

pub fn gen_mipmaps(img: &RawImage<Rgb<u8>>, page_size: usize) -> Result<Vec<RawImage<Rgb<u8>>>> {
    ensure_pot(img.w)?;
    ensure_pot(img.h)?;

    let mut w = img.w;
    let mut h = img.h;

    let mut mips = Vec::new();
    while w > page_size && h > page_size {
        let resized = resize(w, h, img);
        let pages = make_pages(&resized, page_size);
        mips.extend(pages);

        w /= 2;
        h /= 2;
    }

    Ok(mips)
}

#[test]
fn testvt() -> Result<()> {
    use image::RgbImage;

    if VirtualTexture::new(1, 3, 5).is_ok() {
        return Err(anyhow!("test failed"));
    };

    let page_size = 64;

    let test_img: RawImage<Rgb<u8>> = RawImage::checkerboard(512, 512, 32, &[0.; 3], &[1.; 3]);

    let mipmaps = gen_mipmaps(&test_img, page_size)?;

    for (i, mip) in mipmaps.into_iter().enumerate() {
        let rgb = RgbImage::from(mip);
        rgb.save(format!("./target/vt_test/vt_text_{i}.png"))
            .expect("test failed of virtual texture");
    }

    Ok(())
}
