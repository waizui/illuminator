use crate::raycast::{Ray, Raycast};

#[derive(Debug)]
pub struct BVH<T> {
    raw: Vec<T>,
}

impl<T> BVH<T> {}

impl<T> Raycast for BVH<T> {
    fn raycast(&self, ray: Ray) -> super::Hit {
        todo!()
    }
}
