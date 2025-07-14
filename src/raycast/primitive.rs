use crate::raycast::{bounds::Bounds3f, Raycast};

pub trait Primitive: Raycast + Sync + Send  {
    fn bounds(&self) -> Bounds3f;

    fn clone_as_box(&self)->Box<dyn Primitive>;
}
