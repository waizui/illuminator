use std::{
    fs::{create_dir_all, remove_dir_all},
    path::{self, Path},
};

use crate::img::*;
use anyhow::{Result, anyhow};
use image::{DynamicImage, Rgb, RgbImage, imageops::FilterType};
use itertools::Itertools;

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
    let page_num = img.w / page_size;
    let mut pages = Vec::new();
    for (i, j) in (0..page_num).cartesian_product(0..page_num) {
        let mut page = RawImage::new(page_size, page_size);
        page.par_iter_pixel(|(k, pix)| {
            let w = k % page_size + i * page_size;
            let h = k / page_size + j * page_size;
            *pix = img[(w, h)];
        });
        pages.push(page);
    }

    pages
}

pub fn gen_mipmaps(
    img: &RawImage<Rgb<u8>>,
    page_size: usize,
) -> Result<Vec<Vec<RawImage<Rgb<u8>>>>> {
    ensure_pot(img.w)?;
    ensure_pot(img.h)?;

    let mut w = img.w;
    let mut h = img.h;

    let mut mips = Vec::new();
    while w >= page_size && h >= page_size {
        let resized = resize(w, h, img);
        let pages = make_pages(&resized, page_size);
        mips.push(pages);

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

    let save_path = "./target/vt_test/";
    if Path::new(save_path).exists() {
        remove_dir_all(save_path)?;
    }
    create_dir_all(save_path)?;

    let page_size = 64;

    let test_img: RawImage<Rgb<u8>> = RawImage::checkerboard(256, 256, 16, &[0.; 3], &[1.; 3]);

    let mipmaps = gen_mipmaps(&test_img, page_size)?;

    let cols = [[0, 0, 1], [0, 1, 0], [1, 0, 0]];

    for (i, mips) in mipmaps.into_iter().enumerate() {
        for (j, mut mip) in mips.into_iter().enumerate() {
            mip.par_iter_pixel(|(_, pix)| {
                let r = pix.0[0] * cols[i][0];
                let g = pix.0[1] * cols[i][1];
                let b = pix.0[2] * cols[i][2];
                *pix = Rgb([r, g, b]);
            });
            let rgb = RgbImage::from(mip);
            rgb.save(format!("{save_path}vt_text_{i}_{j}.png"))
                .expect("test failed of virtual texture");
        }
    }

    Ok(())
}
