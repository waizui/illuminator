use crate::raycast::{Raycast, bounds::Bounds3f, sphere::Sphere};

pub trait Primitive: Raycast + Sync {
    fn bounds(&self) -> Bounds3f;

}
