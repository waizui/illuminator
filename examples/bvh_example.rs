use illuminator::{
    core::tensor::Float3,
    img::*,
    raycast::{Ray, Raycast, bvh::BVH, sphere::Sphere},
};
use image::{Rgb, RgbImage};
use rayon::iter::{IndexedParallelIterator, IntoParallelRefMutIterator, ParallelIterator};

fn main() {
    let n = 1024;
    let node_limit = 17;

    let mut bvh = BVH::new(n);
    let arr: Vec<usize> = (0..n).collect();
    for &i in arr.iter().step_by(100).take(8) {
        let cnt = Float3::vec(&[i as f32 + 0.5; 3]);
        let sph = Sphere::new(cnt, 100.5);
        bvh.push(sph);
    }
    bvh.build(node_limit, true);

    let (w, h) = (256, 256);
    let mut img: Image<Rgb<u8>> = Image::new(w, h);

    img.data_mut()
        .par_iter_mut()
        .enumerate()
        .for_each(|(i, pix)| {
            let (iw, ih) = (i % w, i / w);
            let (u, v) = (
                iw as f32 * n as f32 / (w - 1) as f32,
                ih as f32 * n as f32 / (h - 1) as f32,
            );

            let org = Float3::vec(&[u - 0.5, v - 0.5, 1025.]);
            let dir = Float3::vec(&[0., 0., -1.]);
            let ray = Ray::new(org, dir);
            if let Some(hit) = bvh.raycast(&ray) {
                let t = (hit.t * 255. / 800.) as u8;
                *pix = Rgb([t; 3]);
            }
        });

    let rgbimg = RgbImage::from(img);
    rgbimg
        .save("./target/bvh_examples.png")
        .expect("save error");
}
