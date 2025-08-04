use illuminator::prelude::*;
use num_traits::Zero;
use rayon::prelude::*;

pub fn bvh_example(save_path: Option<&str>) {
    println!("Running BVH example...");

    let n = 1024;
    let node_limit = 65;

    let mut bvh = BVH::new(n);
    for i in (0..n).step_by(150).skip(1).take(6) {
        let cnt = Vec3f::vec([i as f32 + 0.5; 3]);
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

            let org = Vec3f::vec([x - 0.5, y - 0.5, 1025.]);
            let dir = Vec3f::vec([0., 0., -1.]);
            let ray = Ray::new(org, dir);

            if let Some(hit) = bvh.raycast(&ray) {
                let t = (hit.t * 255. / 1024.) as u8;
                *pix = Rgb([t; 3]);
            }
        });

    let save_path = &path_or_default(save_path, "bvh_examples.png");

    let rgbimg = RgbImage::from(img);
    rgbimg
        .save(save_path)
        .expect("Failed to save BVH example image");

    println!("BVH example completed! Output saved to {save_path}");
}

pub fn gaussian_splatting_example(ply_path: Option<&str>, (w, h): (usize, usize)) {
    use illuminator::{prelude::*, splat::render::SplatsRenderer};
    use image::{Rgb, RgbImage};
    use rayon::prelude::*;
    use std::path::Path;
    use std::time::Instant;

    println!("Running BVH example...");

    let read_path = &path_or_default(ply_path, "point_cloud.ply");
    let gs = SplatsRenderer::from_ply(read_path);
    if gs.is_err() {
        println!("Read file at {read_path} Error.");
        return;
    }

    let gs = gs.unwrap();

    let high_res = w > 256 || h > 256;
    if high_res {
        print!("Resolution is larger than 256x256. This may take a while. Continue? (y/n): ");
        std::io::Write::flush(&mut std::io::stdout()).unwrap();
        let mut input = String::new();
        std::io::stdin().read_line(&mut input).unwrap();

        if !input.trim().to_lowercase().starts_with('y') {
            println!("Gaussian splatting example cancelled.");
            return;
        }
    }

    println!("Trace resolution: {w}x{h}");

    let mut img: Image<Rgb<u8>> = Image::new(w, h);
    let cam_pos = Vec3f::vec([5., 0., 0.]);
    let forward = Vec3f::zero() - cam_pos;
    let cam = Camera::new(cam_pos, forward, 30., 0.25, 4.);

    let total_pixels = w * h;
    let progress_counter = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let progress_counter_clone = progress_counter.clone();

    let progress_handle = std::thread::spawn(move || {
        loop {
            std::thread::sleep(std::time::Duration::from_millis(1000));
            let current = progress_counter_clone.load(std::sync::atomic::Ordering::Relaxed);
            if current >= total_pixels {
                println!("\rProgress: ({total_pixels}/{total_pixels})");
                break;
            }

            print!("\rProgress: ({current}/{total_pixels})");
            std::io::Write::flush(&mut std::io::stdout()).unwrap();
        }
    });

    let start = Instant::now();

    img.data_mut()
        .par_iter_mut()
        .enumerate()
        .for_each(|(i, pix)| {
            let (iw, ih) = (i % w, i / w);
            // let ray = cam.gen_ray_orthogonal((iw, ih), (0., 0.), (w, h), 1.5);
            let ray = cam.gen_ray((iw, ih), (0., 0.), (w, h));
            let col = gs.trace(&ray);
            let col = col * 255.;
            *pix = Rgb([col[0] as u8, col[1] as u8, col[2] as u8]);

            progress_counter.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        });

    progress_handle.join().unwrap();

    println!("Rendering used {:.2}s", start.elapsed().as_secs_f32());

    let fname = Path::new(read_path)
        .with_extension("png")
        .to_string_lossy()
        .into_owned();
    let rgbimg = RgbImage::from(img);
    rgbimg
        .save(fname)
        .expect("Failed to save Gaussian Splatting example image");
}

fn path_or_default(path: Option<&str>, default: &str) -> String {
    let default_path = if std::path::Path::new("Cargo.toml").exists() {
        format!("./target/{default}")
    } else {
        format!("./{default}")
    };

    path.unwrap_or(&default_path).to_string()
}
