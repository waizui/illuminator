const SH_C0: f32 = 0.2820948;

use crate::{
    core::{math, quaternion::Quat, spherical::sh_reconstruct_one},
    prelude::Vec3f,
    raycast::{Hit, Ray, Raycast, bounds::Bounds3f, primitive::Primitive},
    splat::io::InputGaussian,
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
}

impl Gaussian {
    pub fn from_input(input: &InputGaussian) -> Self {
        let col = Vec3f::vec(input.dc0) * SH_C0 + 0.5;

        let sh = std::array::from_fn(|i| {
            let reodered_sh = [input.sh[i], input.sh[i + 15], input.sh[i + 30]];
            Vec3f::vec(reodered_sh)
        });

        Gaussian {
            pos: Vec3f::vec(input.pos),
            nor: Vec3f::vec(input.nor),
            col,
            sh,
            opacity: math::sigmoid(input.opacity),
            scale: Vec3f::vec(input.scale),
            rot: Quat::new(input.rot),
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
        //TODO: icosahedron
        self.bounds().raycast(ray)
    }
}

impl Primitive for Gaussian {
    fn bounds(&self) -> Bounds3f {
        let one = Vec3f::vec([0.01; 3]);
        Bounds3f {
            min: self.pos - one,
            max: self.pos + one,
        }
    }

    fn clone_as_box(&self) -> Box<dyn Primitive> {
        Box::new(*self)
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
