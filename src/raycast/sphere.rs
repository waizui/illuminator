use crate::{core::tensor::Float3, raycast::{Hit, Ray, Raycast}};

pub struct Sphere {
    pub cnt: Float3,
    pub r: f32,
}


impl Raycast for Sphere {
    fn raycast(&self, ray: Ray) -> Hit {
        todo!()
    }
}
