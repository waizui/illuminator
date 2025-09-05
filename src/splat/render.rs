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
    pub const CHUNK_SIZE: usize = 16; // same as node size for k-closest finding
    pub const BVH_NODE_SIZE: usize = 32;

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
    pub fn trace(&self, in_ray: &Ray) -> Vec3f {
        const T_MIN: f32 = 1e-5;
        const ALPHA_MIN: f32 = 4e-2;

        let mut col = Vec3f::zero();
        let mut tsm = 1.; // transmittance
        let mut buf;

        let mut ray = in_ray.clone();

        loop {
            let mut end_trace = false;
            buf = [(0, f32::INFINITY); Self::CHUNK_SIZE];

            self.bvh.any_raycast(&ray, |_, hit, prim_i| {
                let mut cur_i = prim_i;
                let mut cur_t = hit.t;

                if hit.t < buf[Self::CHUNK_SIZE - 1].1 {
                    for k in 0..Self::CHUNK_SIZE {
                        if cur_t < buf[k].1 {
                            let (tmp_i, tmp_t) = buf[k];
                            buf[k] = (cur_i, cur_t);
                            cur_i = tmp_i;
                            cur_t = tmp_t;
                        }
                    }
                    // skip hit except farthest one
                    if hit.t < buf[Self::CHUNK_SIZE - 1].1 {
                        return true;
                    }
                }

                false
            });

            // process chunk hits
            let mut max_t = 0f32;
            for (_, t) in buf.iter() {
                max_t = max_t.max(*t);
                if *t == f32::INFINITY {
                    end_trace = true;
                    break;
                }

                let (chunk_col, chunk_tsm) = self.chunk_color(&buf, &ray, tsm, T_MIN, ALPHA_MIN);
                col = col + chunk_col;
                tsm = chunk_tsm;

                if tsm < T_MIN {
                    end_trace = true;
                    break;
                }
            }

            if end_trace {
                break;
            }

            ray.marching(max_t);
        }

        col
    }

    fn chunk_color(
        &self,
        buf: &[(usize, f32)],
        ray: &Ray,
        mut tsm: f32,
        t_min: f32,
        a_min: f32,
    ) -> (Vec3f, f32) {
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
fn test_trace_splats() -> Result<()> {
    use crate::{prelude::*, splat::render::SplatsRenderer};
    use std::path::Path;

    let ply_path = "./target/bicycle.ply";
    let rdr = SplatsRenderer::from_ply(ply_path)?;

    let mut cam = Camera::default();
    cam.pos = Vec3f::vec([-3., 0., 0.]);
    cam.look_at(Vec3f::zero());

    let (w, h) = (256, 256);
    let img = rdr.render(&cam, (w, h));

    let png_path = Path::new(ply_path)
        .with_extension("png")
        .to_string_lossy()
        .into_owned();

    let rgbimg = RgbImage::from(img);
    rgbimg.save(png_path).expect("Failed to save trace image");
    Ok(())
}
