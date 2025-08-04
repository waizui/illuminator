use crate::{
    core::{matrix::Matrix, tensor::Mat1x3f, vec::Vector},
    img::{Image, PixelType},
    prelude::*,
    splat::{gaussian::Gaussian, io::read_ply},
};

use anyhow::Result;
use num_traits::{One, Zero};
use rayon::prelude::*;

pub struct SplatsRenderer {
    pub bvh: BVH<Gaussian>,
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

    pub fn render<P: PixelType>(&self, cam: &Camera, (w, h): (usize, usize)) -> Image<P> {
        let total_pixs = w * h;
        let finished_pixs = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let arc_finished_pixs = finished_pixs.clone();

        let inspector = std::thread::spawn(move || {
            loop {
                std::thread::sleep(std::time::Duration::from_millis(1000));
                let current = arc_finished_pixs.load(std::sync::atomic::Ordering::Relaxed);
                if current >= total_pixs {
                    println!("\rProgress: ({total_pixs}/{total_pixs})");
                    break;
                }

                print!("\rProgress: ({current}/{total_pixs})");
                std::io::Write::flush(&mut std::io::stdout()).unwrap();
            }
        });

        let mut img: Image<P> = Image::new(w, h);
        img.data_mut()
            .par_iter_mut()
            .enumerate()
            .for_each(|(i, pix)| {
                let (iw, ih) = (i % w, i / w);
                // let ray = cam.gen_ray_orthogonal((iw, ih), (0., 0.), (w, h), 1.5);
                let ray = cam.gen_ray((iw, ih), (0., 0.), (w, h));
                let col = self.trace(&ray);
                *pix = P::from(&[col[0], col[1], col[2]]);
                finished_pixs.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            });

        inspector.join().unwrap();
        img
    }

    ///TODO: clip for rendering
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

                if tsm < T_MIN {
                    return true;
                }

                // restore chunk state
                hit_count = 0;
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
        &self.bvh.primitives[i]
    }
}

#[test]
fn test_trace_splats() {
    use crate::render::camera::Camera;
    use crate::{prelude::Vec3f, splat::render::SplatsRenderer};
    use std::path::Path;

    // let path = "./target/bicycle.ply";
    let path = "./target/obj_011.ply";
    // let path = "./target/background.ply";
    let rdr = SplatsRenderer::from_ply(path).unwrap();

    // let (w, h) = (32, 32);
    let (w, h) = (128, 128);
    // let (w, h) = (512, 512);

    println!("test tace {w}x{h}");

    let cam_pos = Vec3f::vec([5., 0., 0.]);
    let forward = Vec3f::zero() - cam_pos;
    let cam = Camera::new(cam_pos, forward, 30., 0.25, 4.);

    let img = rdr.render(&cam, (w, h));

    let fname = Path::new(path).file_stem().unwrap().to_str().unwrap();
    let rgbimg = RgbImage::from(img);
    rgbimg
        .save(format!("./target/{fname}_trace.png"))
        .expect("Failed to save BVH example image");
}
