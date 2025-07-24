use crate::{
    gsplat::{gaussian::Splat, io::read_ply},
    prelude::BVH,
    raycast::primitive::Primitive,
};
use anyhow::Result;
use rayon::prelude::*;

pub struct SplatsRenderer {
    pub bvh: BVH,
}

impl SplatsRenderer {
    pub fn from_ply(path: &str) -> Result<Self> {
        let input_gs = read_ply(path)?;
        let splats: Vec<Splat> = input_gs.par_iter().map(Splat::from_input).collect();

        let mut bvh = BVH::new(splats.len());

        (0..splats.len()).for_each(|i| {
            bvh.push(splats[i]);
        });

        bvh.build(129, true);

        Ok(SplatsRenderer { bvh })
    }
}

#[test]
fn test_splats_bvh() {
    use crate::{gsplat::gaussian::Splat, prelude::Vec3f, raycast::Ray};
    use rayon::prelude::*;

    let path = "./target/obj_011.ply";

    let input_gs = read_ply(path).unwrap();
    let splats: Vec<Splat> = input_gs.par_iter().map(Splat::from_input).collect();
    let mut bvh0 = BVH::new(splats.len());
    (0..splats.len()).for_each(|i| {
        bvh0.push(splats[i]);
    });
    bvh0.build(129, true);

    let input_gs = read_ply(path).unwrap();
    let splats: Vec<Splat> = input_gs.par_iter().map(Splat::from_input).collect();
    let mut bvh1 = BVH::new(splats.len());
    (0..splats.len()).for_each(|i| {
        bvh1.push(splats[i]);
    });
    bvh1.build(129, true);

    let (w, h) = (128, 128);

    (0..w * h).into_par_iter().for_each(|i| {
        let (iw, ih) = (i % w, i / w);
        let (lw, ly) = (8., 8.);
        let (x, y) = (
            iw as f32 * lw / (w - 1) as f32,
            (h - ih) as f32 * ly / (h - 1) as f32,
        );

        let org = Vec3f::vec([x - lw / 2., y - ly / 2., 1025.]);
        let dir = Vec3f::vec([0., 0., -1.]);
        let ray = Ray::new(org, dir);

        let hit0 = bvh0.raycast_node(&ray);
        let hit1 = bvh1.raycast_node(&ray);

        assert_eq!(hit0.is_some(), hit1.is_some());

        if hit0.is_some() {
            let (hit0, i0) = hit0.unwrap();
            let (hit1, i1) = hit1.unwrap();
            assert_eq!(hit0.t, hit1.t);

            let splat0 = &bvh0.primitives[i0]
                .as_any()
                .downcast_ref::<Splat>()
                .unwrap();

            let splat1 = &bvh1.primitives[i1]
                .as_any()
                .downcast_ref::<Splat>()
                .unwrap();

            assert_eq!(splat0.bounds(), splat1.bounds());
            assert_eq!(splat0.sh, splat1.sh);
        }
    });
}

#[test]
fn test_splats_render() {
    use crate::img::*;
    use crate::{
        core::spherical::sh_reconstruct_one,
        gsplat::{gaussian::Splat, render::SplatsRenderer},
        prelude::Vec3f,
        raycast::Ray,
    };
    use image::{Rgb, RgbImage};
    use rayon::prelude::*;

    let path = "./target/obj_011.ply";
    let gs = SplatsRenderer::from_ply(path).unwrap();

    // let (w, h) = (32, 32);
    let (w, h) = (512, 512);
    let mut img: Image<Rgb<u8>> = Image::new(w, h);

    img.data_mut()
        .par_iter_mut()
        .enumerate()
        .for_each(|(i, pix)| {
            let (iw, ih) = (i % w, i / w);
            let (lw, ly) = (8., 8.);
            let (x, y) = (
                iw as f32 * lw / (w - 1) as f32,
                (h - ih) as f32 * ly / (h - 1) as f32,
            );

            let org = Vec3f::vec([x - lw / 2., y - ly / 2., 1025.]);
            let dir = Vec3f::vec([0., 0., -1.]);
            let ray = Ray::new(org, dir);

            if let Some((_, i)) = gs.bvh.raycast_node(&ray) {
                let prim = &gs.bvh.primitives[i];
                let splat = prim.as_any().downcast_ref::<Splat>().unwrap();
                let coeffs = [
                    splat.col,
                    splat.sh[0],
                    splat.sh[1],
                    splat.sh[2],
                    splat.sh[3],
                    splat.sh[4],
                    splat.sh[5],
                    splat.sh[6],
                    splat.sh[7],
                ];

                let mut rgb: Vec3f = sh_reconstruct_one(&coeffs, 2, dir);
                rgb = rgb + 0.5 ;

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
