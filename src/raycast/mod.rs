use crate::core::tensor::Float3;

pub mod bounds;
pub mod bvh;
pub mod bvhbuild;
pub mod morton;
pub mod primitive;
pub mod sphere;

#[derive(Debug, Clone)]
pub struct Ray {
    pub org: Float3,
    pub dir: Float3,
    pub t_max: f32,
}

impl Ray {
    pub fn new(org: Float3, dir: Float3) -> Ray {
        Ray {
            org,
            dir,
            t_max: f32::MAX,
        }
    }

    pub fn segment(org: Float3, dir: Float3, t_max: f32) -> Ray {
        Ray { org, dir, t_max }
    }
}

//TODO: hit multiple
#[derive(Debug)]
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
    /// ray direction not always a unit vector
    fn raycast(&self, ray: &Ray) -> Option<Hit>;
}
