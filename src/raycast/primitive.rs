use std::fmt::Debug;

use crate::raycast::{Raycast, bounds::Bounds3f};

pub trait Primitive: Raycast + Sync + Send + Debug {
    fn bounds(&self) -> Bounds3f;

    fn clone_as_box(&self) -> Box<dyn Primitive>;
}
