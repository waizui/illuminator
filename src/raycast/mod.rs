use crate::core::tensor::Vec3f;

pub mod bounds;
pub mod bvh;
pub mod bvhbuild;
pub mod gaussian;
pub mod morton;
pub mod primitive;
pub mod sphere;

#[derive(Debug, Clone)]
pub struct Ray {
    pub org: Vec3f,
    pub dir: Vec3f,
    pub t_max: f32,
}

impl Ray {
    pub fn new(org: Vec3f, dir: Vec3f) -> Ray {
        Ray {
            org,
            dir,
            t_max: f32::MAX,
        }
    }

    pub fn segment(org: Vec3f, dir: Vec3f, t_max: f32) -> Ray {
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
    pub fn position(&self) -> Vec3f {
        self.ray.org + self.ray.dir * self.t
    }
}

pub trait Raycast {
    /// ray direction not always a unit vector
    fn raycast(&self, ray: &Ray) -> Option<Hit>;
}
