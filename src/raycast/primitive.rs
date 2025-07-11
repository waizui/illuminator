use crate::raycast::{Raycast, bounds::Bounds};

pub trait Primitive: Raycast {
    fn bounds() -> Bounds;
}
