use crate::core::tensor::Vec3f;

pub mod bounds;
pub mod bvh;
pub mod bvhbuild;
pub mod camera;
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
    
    /// move ray alone direction by scaling factor t
    pub fn marching(&mut self, t: f32) {
        self.org = self.org + self.dir * t;
    }
}

//TODO: hit multiple
#[derive(Debug)]
pub struct Hit {
    pub t: f32,
}

impl Hit {
    pub fn position(&self, ray: &Ray) -> Vec3f {
        ray.org + ray.dir * self.t
    }
}

pub trait Raycast {
    /// ray direction not always a unit vector
    fn raycast(&self, ray: &Ray) -> Option<Hit>;
}
