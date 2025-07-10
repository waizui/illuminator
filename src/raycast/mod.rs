use crate::core::tensor::Float3;

pub mod bvh;
pub mod sphere;

pub struct Ray {
    pub org: Float3,
    pub dir: Float3,
}

impl Ray {
    pub fn new(org: Float3, dir: Float3) -> Ray {
        Ray { org, dir }
    }
}

pub struct Hit {
    pub ray: Ray,
    pub t: f32,
}

impl Hit {
    pub fn position(&self) -> Float3 {
        self.ray.org + self.ray.dir * self.t
    }
}

pub trait Raycast {
    fn raycast(&self, ray: Ray) -> Hit;
}

