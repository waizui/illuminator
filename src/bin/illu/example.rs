use illuminator::prelude::*;
use rayon::prelude::*;

pub fn bvh_example(save_path: Option<&str>) {
    println!("Running BVH example...");

    let n = 1024;
    let node_limit = 65;

    let mut bvh = BVH::new(n);
    for i in (0..n).step_by(150).skip(1).take(6) {
        let cnt = Vec3f::vec(&[i as f32 + 0.5; 3]);
        bvh.push(Sphere::new(cnt, 100.));
    }

    bvh.build(node_limit, true);

    let (w, h) = (512, 512);
    let mut img: Image<Rgb<u8>> = Image::new(w, h);

    img.data_mut()
        .par_iter_mut()
        .enumerate()
        .for_each(|(i, pix)| {
            let (iw, ih) = (i % w, i / w);
            let (x, y) = (
                iw as f32 * n as f32 / (w - 1) as f32,
                (h - ih) as f32 * n as f32 / (h - 1) as f32,
            );

            let org = Vec3f::vec(&[x - 0.5, y - 0.5, 1025.]);
            let dir = Vec3f::vec(&[0., 0., -1.]);
            let ray = Ray::new(org, dir);

            if let Some(hit) = bvh.raycast(&ray) {
                let t = (hit.t * 255. / 1024.) as u8;
                *pix = Rgb([t; 3]);
            }
        });

    let default_path = if std::path::Path::new("Cargo.toml").exists() {
        "./target/bvh_examples.png"
    } else {
        "./bvh_examples.png"
    };

    let save_path = save_path.unwrap_or(default_path);

    let rgbimg = RgbImage::from(img);
    rgbimg
        .save(save_path)
        .expect("Failed to save BVH example image");

    println!("BVH example completed! Output saved to {save_path}");
}
