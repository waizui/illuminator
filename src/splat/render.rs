use crate::{
    core::{matrix::Matrix, tensor::Mat1x3f, vec::Vector},
    prelude::{BVH, Vec3f},
    raycast::Ray,
    splat::{gaussian::Gaussian, io::read_ply},
};
use anyhow::Result;
use num_traits::{One, Zero};
use rayon::prelude::*;

pub struct SplatsRenderer {
    pub bvh: BVH,
}

impl SplatsRenderer {
    pub const CHUNK_SIZE: usize = 64;
    pub const BVH_NODE_SIZE: usize = 64;

    pub fn from_ply(path: &str) -> Result<Self> {
        let input_gs = read_ply(path)?;
        let splats: Vec<Gaussian> = input_gs.par_iter().map(Gaussian::from_input).collect();

        let mut bvh = BVH::new(splats.len());

        (0..splats.len()).for_each(|i| {
            bvh.push(splats[i]);
        });

        bvh.build(Self::BVH_NODE_SIZE + 1, true);

        Ok(SplatsRenderer { bvh })
    }

    pub fn trace(&self, ray: &Ray) -> Vec3f {
        const T_MIN: f32 = 1e-5;
        const ALPHA_MIN: f32 = 4e-2;

        let mut col = Vec3f::zero();
        let mut tsm = 1.; // transmittance

        let mut hit_count = 0;
        let mut buf = [(0, 0.); Self::CHUNK_SIZE];

        self.bvh.mult_raycast(ray, |_, hit, i| {
            buf[hit_count] = (i, hit.t);
            hit_count += 1;
            if hit_count == Self::CHUNK_SIZE {
                let (chunk_col, chunk_tsm) = self.chunk_color(&mut buf, ray, tsm, T_MIN, ALPHA_MIN);
                col = col + chunk_col;
                tsm = chunk_tsm;

                // restore chunk state
                hit_count = 0;

                if tsm < T_MIN {
                    return true;
                }
            }
            // continue tracing
            false
        });

        if hit_count < Self::CHUNK_SIZE {
            let (chunk_col, _) = self.chunk_color(&mut buf, ray, tsm, T_MIN, ALPHA_MIN);
            col = col + chunk_col;
        }

        col
    }

    fn chunk_color(
        &self,
        buf: &mut [(usize, f32)],
        ray: &Ray,
        mut tsm: f32,
        t_min: f32,
        a_min: f32,
    ) -> (Vec3f, f32) {
        buf.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());

        let mut col = Vec3f::zero();
        for &(i, _) in buf.iter() {
            let splat = self.get_gaussian(i);
            let alpha = (self.process_hit(splat, ray) * splat.opacity).min(0.99);
            if alpha < a_min {
                continue;
            }

            let rgb = splat.sh_color(2, ray.dir);
            tsm *= 1. - alpha;
            if tsm < t_min {
                break;
            }
            col = col + rgb * (tsm * alpha);
        }

        (col, tsm)
    }

    fn process_hit(&self, splat: &Gaussian, ray: &Ray) -> f32 {
        let rot = splat.rot.to_matrix();
        let inv_scl = (Vec3f::one() / splat.scale).reshape(&[1, 3]);

        // S^-1[T^-1 P]R = p1
        let trans_pos: Mat1x3f = (ray.org - splat.pos).reshape(&[1, 3]).matmul(rot);
        let ray_pos: Vec3f = (inv_scl * trans_pos).reshape(&[3]);

        // S^-1[D]R = D1
        let rot_dir: Mat1x3f = ray.dir.reshaped(&[1, 3]).matmul(rot);
        let ray_dir: Vec3f = (inv_scl * rot_dir).reshape(&[3]).normalize();

        let cp = ray_pos.cross(ray_dir);
        let graydist = cp.dot(cp);

        (-0.5 * graydist).exp()
    }

    pub fn get_gaussian(&self, i: usize) -> &Gaussian {
        let prim = &self.bvh.primitives[i];
        prim.as_any().downcast_ref::<Gaussian>().unwrap()
    }
}

#[test]
fn test_trace_splats() {
    use crate::img::*;
    use crate::{prelude::Vec3f, raycast::Ray, splat::render::SplatsRenderer};
    use image::{Rgb, RgbImage};
    use rayon::prelude::*;
    use std::path::Path;

    // let path = "./target/bicycle.ply";
    // let path = "./target/obj_011.ply";
    let path = "./target/background.ply";
    let gs = SplatsRenderer::from_ply(path).unwrap();

    let (w, h) = (32, 32);
    // let (w, h) = (128, 128);
    // let (w, h) = (512, 512);

    println!("test tace {w}x{h}");

    let mut img: Image<Rgb<u8>> = Image::new(w, h);

    img.data_mut()
        .par_iter_mut()
        .with_min_len(w)
        .enumerate()
        .for_each(|(i, pix)| {
            let (iw, ih) = (i % w, i / w);
            // let (lw, ly) = (3., 3.);
            let (lw, ly) = (2., 2.);
            let (x, y) = (
                iw as f32 * lw / (w - 1) as f32,
                (h - ih) as f32 * ly / (h - 1) as f32,
            );

            let org = Vec3f::vec([-5., 0.5 + x - lw / 2., 0.5 + y - ly / 2.]);
            let dir = Vec3f::vec([1., 0., 0.]);
            let ray = Ray::new(org, dir);
            let col = gs.trace(&ray);
            let col = col * 255.;
            *pix = Rgb([col[0] as u8, col[1] as u8, col[2] as u8]);
        });

    let fname = Path::new(path).file_stem().unwrap().to_str().unwrap();
    let rgbimg = RgbImage::from(img);
    rgbimg
        .save(format!("./target/{fname}_trace.png"))
        .expect("Failed to save BVH example image");
}
