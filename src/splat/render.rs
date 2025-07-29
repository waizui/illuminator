use crate::{
    prelude::{BVH, Vec3f},
    raycast::Ray,
    splat::{gaussian::Gaussian, io::read_ply},
};
use anyhow::Result;
use num_traits::Zero;
use rayon::prelude::*;

pub struct SplatsRenderer {
    pub bvh: BVH,
}

impl SplatsRenderer {
    pub fn from_ply(path: &str) -> Result<Self> {
        let input_gs = read_ply(path)?;
        let splats: Vec<Gaussian> = input_gs.par_iter().map(Gaussian::from_input).collect();

        let mut bvh = BVH::new(splats.len());

        (0..splats.len()).for_each(|i| {
            bvh.push(splats[i]);
        });

        bvh.build(65, true);

        Ok(SplatsRenderer { bvh })
    }

    pub fn trace(&self, ray: &Ray) -> Vec3f {
        const CHUNK_SIZE: usize = 32;
        let mut ray = ray.clone();
        let mut col = Vec3f::zero();
        let mut tsm = 1.;

        loop {
            let mut hit_count = 0;
            while let Some((hit, i)) = self.bvh.raycast_node(&ray) {
                hit_count += 1;

                let splat = self.get_gaussian(i);
                let rgb = splat.sh_color(3, ray.dir);

                let w = tsm * (1. - splat.opacity);

                col = col + rgb * (tsm * splat.opacity);

                if w < 1e-4f32 {
                    break;
                }

                tsm = w;

                if hit_count == CHUNK_SIZE {
                    break;
                }
                ray.marching(hit.t + f32::EPSILON);
            }

            if hit_count < CHUNK_SIZE {
                break;
            }
        }
        col
    }

    pub fn get_gaussian(&self, i: usize) -> &Gaussian {
        let prim = &self.bvh.primitives[i];
        let splat = prim.as_any().downcast_ref::<Gaussian>().unwrap();
        splat
    }
}

#[test]
fn test_trace_splats() {
    use crate::img::*;
    use crate::{prelude::Vec3f, raycast::Ray, splat::render::SplatsRenderer};
    use image::{Rgb, RgbImage};
    use rayon::prelude::*;

    let path = "./target/obj_011.ply";
    let gs = SplatsRenderer::from_ply(path).unwrap();

    let (w, h) = (1024, 1024);

    let mut img: Image<Rgb<u8>> = Image::new(w, h);

    img.data_mut()
        .par_iter_mut()
        .with_min_len(w)
        .enumerate()
        .for_each(|(i, pix)| {
            let (iw, ih) = (i % w, i / w);
            let (lw, ly) = (3., 3.);
            let (x, y) = (
                iw as f32 * lw / (w - 1) as f32,
                (h - ih) as f32 * ly / (h - 1) as f32,
            );

            let org = Vec3f::vec([1025., x - lw / 2., y - ly / 2.]);
            let dir = Vec3f::vec([-1., 0., 0.]);
            let ray = Ray::new(org, dir);

            let col = gs.trace(&ray);
            let col = col * 300.;
            *pix = Rgb([col[0] as u8, col[1] as u8, col[2] as u8]);
        });

    let rgbimg = RgbImage::from(img);
    rgbimg
        .save("./target/trace_gs_example.png")
        .expect("Failed to save BVH example image");
}

#[test]
fn test_splats_render() {
    use crate::img::*;
    use crate::{
        prelude::Vec3f,
        raycast::Ray,
        splat::{gaussian::Gaussian, render::SplatsRenderer},
    };
    use image::{Rgb, RgbImage};
    use rayon::prelude::*;

    let path = "./target/obj_011.ply";
    let gs = SplatsRenderer::from_ply(path).unwrap();

    let (w, h) = (32, 32);
    // let (w, h) = (1024, 1024);
    let mut img: Image<Rgb<u8>> = Image::new(w, h);

    img.data_mut()
        .par_iter_mut()
        .with_min_len(w)
        .enumerate()
        .for_each(|(i, pix)| {
            let (iw, ih) = (i % w, i / w);
            let (lw, ly) = (6., 6.);
            let (x, y) = (
                iw as f32 * lw / (w - 1) as f32,
                (h - ih) as f32 * ly / (h - 1) as f32,
            );

            let org = Vec3f::vec([1025., x - lw / 2., y - ly / 2.]);
            let dir = Vec3f::vec([-1., 0., 0.]);
            let ray = Ray::new(org, dir);

            if let Some((_, i)) = gs.bvh.raycast_node(&ray) {
                let prim = &gs.bvh.primitives[i];
                let splat = prim.as_any().downcast_ref::<Gaussian>().unwrap();
                let rgb = splat.sh_color(3, dir);

                let r = (rgb[0] * 255.) as u8;
                let g = (rgb[1] * 255.) as u8;
                let b = (rgb[2] * 255.) as u8;
                *pix = Rgb([r, g, b]);
            }
        });

    let rgbimg = RgbImage::from(img);
    rgbimg
        .save("./target/gs_example.png")
        .expect("Failed to save BVH example image");
}
