const SH_C0: f32 = 0.2820948;

use crate::{
    core::{
        math, matrix::Matrix, quaternion::Quat, spherical::sh_reconstruct_one, tsrmath::TensorMath,
    },
    prelude::Vec3f,
    raycast::{Hit, Ray, Raycast, bounds::Bounds3f, primitive::Primitive},
    splat::io::RawGaussian,
};

#[derive(Debug, Clone, Copy)]
pub struct Gaussian {
    pub pos: Vec3f,
    pub nor: Vec3f,
    pub col: Vec3f,
    pub sh: [Vec3f; 15],
    pub opacity: f32,
    pub scale: Vec3f,
    pub rot: Quat,
    bounds: Bounds3f,
}

impl Gaussian {
    pub fn from_input(input: &RawGaussian) -> Self {
        let col = Vec3f::vec(input.dc0) * SH_C0 + 0.5;

        let sh = std::array::from_fn(|i| {
            let reodered_sh = [input.sh[i], input.sh[i + 15], input.sh[i + 30]];
            Vec3f::vec(reodered_sh)
        });

        let pos = Vec3f::vec(input.pos);
        let scale = Vec3f::vec(input.scale).exp();
        let rot = Quat::new(input.rot);

        Gaussian {
            pos,
            nor: Vec3f::vec(input.nor),
            col,
            sh,
            opacity: math::sigmoid(input.opacity),
            scale,
            rot,
            bounds: calc_bounds(pos, scale, rot),
        }
    }

    /// l: sh degree, max is 3
    pub fn sh_color(&self, l: i32, dir: Vec3f) -> Vec3f {
        assert!(l <= 3);
        let n = ((l + 1) * (l + 1)) as usize;
        let mut coeffs: Vec<Vec3f> = Vec::with_capacity(n);
        let c0 = (self.col - 0.5) / SH_C0;
        coeffs.push(c0);
        (0..n - 1).for_each(|i| coeffs.push(self.sh[i]));

        let rgb: Vec3f = sh_reconstruct_one(&coeffs, l, dir);
        rgb + 0.5
    }
}

impl Raycast for Gaussian {
    fn raycast(&self, ray: &Ray) -> Option<Hit> {
        self.bounds.raycast(ray)
    }
}

fn calc_bounds(pos: Vec3f, scl: Vec3f, rot: Quat) -> Bounds3f {
    calc_bound_aabb(pos, scl, rot)
}

fn calc_bound_aabb(pos: Vec3f, scl: Vec3f, quat: Quat) -> Bounds3f {
    let aabb_vrt = [
        Vec3f::vec([-1., -1., -1.]),
        Vec3f::vec([-1., -1., 1.]),
        Vec3f::vec([-1., 1., -1.]),
        Vec3f::vec([-1., 1., 1.]),
        Vec3f::vec([1., -1., -1.]),
        Vec3f::vec([1., -1., 1.]),
        Vec3f::vec([1., 1., -1.]),
        Vec3f::vec([1., 1., 1.]),
    ];

    let mut min = Vec3f::vec([f32::INFINITY; 3]);
    let mut max = Vec3f::vec([f32::NEG_INFINITY; 3]);

    let rot = quat.to_matrix();

    aabb_vrt.iter().for_each(|&vrt| {
        let vrt = rot.matmulvec(vrt * scl) + pos;
        min = min.min(vrt);
        max = max.max(vrt);
    });

    Bounds3f { min, max }
}

impl Primitive for Gaussian {
    fn bounds(&self) -> Bounds3f {
        self.bounds
    }

    fn clone_as_box(&self) -> Box<dyn Primitive> {
        Box::new(*self)
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
