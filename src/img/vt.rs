use std::path::PathBuf;

use crate::img::*;
use anyhow::{Result, anyhow};
use image::{DynamicImage, ImageReader, Rgb, RgbImage, imageops::FilterType};
use itertools::Itertools;

pub fn ensure_pot(x: usize) -> Result<()> {
    if !x.is_multiple_of(2) {
        return Err(anyhow!(
            "err: Can not create VirtualTexture with non-power-of-2 image"
        ));
    }

    Ok(())
}

pub trait MipmapStreamer {
    fn fetch(&self, mip: usize, page: usize) -> impl Future<Output = Result<RawImage<Rgb<u8>>>>;
}

pub struct DiskMipmapStreamer {
    path: PathBuf,
}

impl DiskMipmapStreamer {
    pub fn new(dir: &str) -> Self {
        let path = PathBuf::from(dir);
        DiskMipmapStreamer { path }
    }
}

impl MipmapStreamer for DiskMipmapStreamer {
    async fn fetch(&self, mip: usize, page: usize) -> Result<RawImage<Rgb<u8>>> {
        let path = self.path.join(format!("{mip}_{page}.png"));
        let img = tokio::task::spawn_blocking(move || -> Result<RawImage<Rgb<u8>>> {
            Ok(RawImage::from(ImageReader::open(path)?.decode()?.to_rgb8()))
        })
        .await??;
        Ok(img)
    }
}

pub struct VirtualTexture<S: MipmapStreamer> {
    page_size: usize,
    virtual_page_axis: usize,
    physical_page_axis: usize,
    page_table: Vec<usize>,
    physcial_pages: Vec<RawImage<Rgb<u8>>>,
    streamer: S,
}

impl<S: MipmapStreamer> VirtualTexture<S> {
    pub fn new(
        page_size: usize,
        virtual_page_axis: usize,
        physical_page_axis: usize,
        streamer: S,
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
            streamer,
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
    use std::{
        fs::{create_dir_all, remove_dir_all},
        path::Path,
    };

    let save_path = "./target/vt_test/";
    if VirtualTexture::new(1, 3, 5, DiskMipmapStreamer::new(save_path)).is_ok() {
        return Err(anyhow!("test failed"));
    };

    if Path::new(save_path).exists() {
        remove_dir_all(save_path)?;
    }
    create_dir_all(save_path)?;

    let page_size = 64;
    let virtual_size = 256;

    let test_img: RawImage<Rgb<u8>> =
        RawImage::checkerboard(virtual_size, virtual_size, 16, &[0.; 3], &[1.; 3]);

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
            rgb.save(format!("{save_path}{i}_{j}.png"))
                .expect("test failed of virtual texture");
        }
    }

    let rt = tokio::runtime::Builder::new_current_thread().build()?;
    let imgs: Result<Vec<RawImage<Rgb<u8>>>> = rt.block_on(async {
        let streamer = DiskMipmapStreamer::new(save_path);
        let mut fetched = Vec::new();
        for i in 0..3 {
            fetched.push(streamer.fetch(i, 0).await?);
        }
        Ok(fetched)
    });

    assert_eq!(imgs?.len(), 3);

    // let vt = VirtualTexture::new(page_size, virtual_size / page_size, 4, streamer);

    Ok(())
}
