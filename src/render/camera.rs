use num_traits::Zero;

use crate::{
    core::{quaternion::Quat, vec::Vector},
    prelude::*,
};

pub struct Camera {
    pub pos: Vec3f,
    pub forward: Vec3f,
    pub rot: Quat,
    pub fov: f32, // degree
    pub near: f32,
    pub far: f32,
    pub up: Vec3f,
}

impl Camera {
    pub fn new(pos: Vec3f, forward: Vec3f, up: Vec3f, fov: f32, near: f32, far: f32) -> Self {
        // y up , right handed
        Camera {
            pos,
            forward,
            rot: Quat::identity(),
            fov,
            near,
            far,
            up,
        }
    }

    /// return ray with unnormalized dir
    pub fn gen_ray(
        &self,
        (ix, iy): (usize, usize),
        (dx, dy): (f32, f32),
        (w, h): (usize, usize),
    ) -> Ray {
        assert!(ix < w && iy < h);

        let aspect = w as f32 / h as f32;
        let focal = 0.5 / (self.fov.to_radians() * 0.5).tan();

        // to NDC [-0.5,0.5]
        let x = ((ix as f32 + 0.5 + dx) / w as f32 - 0.5) * aspect;
        let y = 0.5 - (iy as f32 + 0.5 + dy) / h as f32;

        let up = self.up;
        let forward = self.forward;
        let right = forward.cross(up).normalize();
        let dir = right * x + up * y + forward * focal;
        Ray::new(self.pos, dir)
    }
}

impl Default for Camera {
    fn default() -> Self {
        let forward = Vec3f::vec([0., 0., -1.]);
        let up = Vec3f::vec([0., 1., 0.]);
        Self::new(Vec3f::zero(), forward, up, 60., 0.25, 4.)
    }
}

#[test]
fn test_camera() {
    let forward = Vec3f::vec([0., 0., -1.]);
    let up = Vec3f::vec([0., 1., 0.]);
    let cam = Camera::new(Vec3f::zero(), forward, up, 90., 0.25, 4.);

    let (w, h) = (32, 32);

    (0..w * h).for_each(|i| {
        let (iw, ih) = (i % w, i / w);
        let ray = cam.gen_ray((iw, ih), (0., 0.), (w, h));

        assert_eq!(iw as f32 + 0.5, (ray.dir[0] + 0.5) * w as f32);
        assert_eq!((h - ih - 1) as f32 + 0.5, (ray.dir[1] + 0.5) * h as f32);
    });
}
