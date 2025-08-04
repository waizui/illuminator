use num_traits::Zero;

use crate::{
    core::{math::orthogonalization, quaternion::Quat},
    prelude::*,
};

//TODO: camera types

pub struct Camera {
    pub pos: Vec3f,
    pub rot: Quat,
    pub fov: f32, // degree
    pub near: f32,
    pub far: f32,

    forward: Vec3f,
    up: Vec3f,
    right: Vec3f,
}

impl Camera {
    pub fn new(pos: Vec3f, forward: Vec3f, fov: f32, near: f32, far: f32) -> Self {
        // y up , right handed
        let up = Vec3f::vec([0., 1., 0.]);
        let (forward, up, right) = orthogonalization(forward, up);

        Camera {
            pos,
            forward,
            up,
            right,
            rot: Quat::identity(),
            fov,
            near,
            far,
        }
    }

    pub fn look_at(&mut self, target: Vec3f) {
        let up = Vec3f::vec([0., 1., 0.]);
        let dir = target - self.pos;
        let (forward, up, right) = orthogonalization(dir, up);
        self.forward = forward;
        self.up = up;
        self.right = right;
    }

    /// return ray with unnormalized dir
    pub fn gen_ray(
        &self,
        (ix, iy): (usize, usize),
        (dx, dy): (f32, f32),
        (res_w, res_h): (usize, usize), //resolution
    ) -> Ray {
        assert!(ix < res_w && iy < res_h);

        let aspect = res_w as f32 / res_h as f32;
        let focal = 0.5 / (self.fov.to_radians() * 0.5).tan();

        // to NDC [-0.5,0.5]
        let x = ((ix as f32 + 0.5 + dx) / res_w as f32 - 0.5) * aspect;
        let y = 0.5 - (iy as f32 + 0.5 + dy) / res_h as f32;

        let dir = self.right * x + self.up * y + self.forward * focal;
        Ray::new(self.pos, dir)
    }

    pub fn gen_ray_orthogonal(
        &self,
        (ix, iy): (usize, usize),
        (dx, dy): (f32, f32),
        (res_w, res_h): (usize, usize),
        size: f32, // half height
    ) -> Ray {
        assert!(ix < res_w && iy < res_h);

        let aspect = res_w as f32 / res_h as f32;

        let size = 2.0 * size;

        let x = ((ix as f32 + 0.5 + dx) / res_w as f32 - 0.5) * size * aspect;
        let y = (0.5 - (iy as f32 + 0.5 + dy) / res_h as f32) * size;

        let origin = self.pos + self.right * x + self.up * y;
        let dir = self.forward;
        Ray::new(origin, dir)
    }
}

impl Default for Camera {
    fn default() -> Self {
        let forward = Vec3f::vec([0., 0., -1.]);
        Self::new(Vec3f::zero(), forward, 60., 0.25, 4.)
    }
}

#[test]
fn test_camera() {
    let forward = Vec3f::vec([0., 0., -1.]);
    let cam = Camera::new(Vec3f::zero(), forward, 90., 0.25, 4.);

    let (w, h) = (2, 2);

    (0..w * h).for_each(|i| {
        let (iw, ih) = (i % w, i / w);
        let ray = cam.gen_ray((iw, ih), (0., 0.), (w, h));

        assert_eq!(iw as f32 + 0.5, (ray.dir[0] + 0.5) * w as f32);
        assert_eq!((h - ih - 1) as f32 + 0.5, (ray.dir[1] + 0.5) * h as f32);
    });
}
