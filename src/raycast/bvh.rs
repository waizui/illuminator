use crate::raycast::*;

#[derive(Debug)]
pub struct BVHNode {}

#[derive(Debug)]
pub struct BVH {
    root: BVHNode,
}

impl BVH {
    pub fn push(&self) {
        todo!()
    }

    pub fn build(&self) {
        todo!()
    }
}

impl Raycast for BVH {
    fn raycast(&self, ray: Ray) -> Option<Hit> {
        todo!()
    }
}
