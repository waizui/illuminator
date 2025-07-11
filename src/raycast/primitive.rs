use crate::raycast::{Raycast, bounds::Bounds3f};

pub trait Primitive: Raycast {
    fn bounds() -> Bounds3f;
}
