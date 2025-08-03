use std::any::Any;
use std::fmt::Debug;

use crate::raycast::{Raycast, bounds::Bounds3f};

pub trait Primitive: Raycast + Sync + Send + Debug + Any + Clone {
    fn bounds(&self) -> Bounds3f;
}
