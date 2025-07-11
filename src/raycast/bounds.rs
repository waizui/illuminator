use crate::{core::tensor::Float3, raycast::*};

#[derive(Debug, Clone, Copy)]
pub struct Bounds {
    pub min: Float3,
    pub max: Float3,
}

impl Bounds {
    pub fn centroid(&self) -> Float3 {
        self.min * 0.5 + self.max * 0.5
    }

    pub fn union(&self, other: &Bounds) -> Bounds {
        todo!()
    }

    pub fn encap(&self, p: Float3) -> Bounds {
        todo!()
    }
}

impl Raycast for Bounds {
    fn raycast(&self, ray: Ray) -> Option<Hit> {
        todo!()
    }
}
