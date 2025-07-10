use crate::raycast::*;

#[derive(Debug)]
pub struct BVH<T> {
    raw: Vec<T>,
}

impl<T> BVH<T> {}

impl<T> Raycast for BVH<T> {
    fn raycast(&self, ray: Ray) -> Option<Hit> {
        todo!()
    }
}
