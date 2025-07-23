use std::any::Any;
use std::fmt::Debug;

use crate::raycast::{Raycast, bounds::Bounds3f};

pub trait Primitive: Raycast + Sync + Send + Debug + Any {
    fn bounds(&self) -> Bounds3f;

    fn clone_as_box(&self) -> Box<dyn Primitive>;

    fn as_any(&self) -> &dyn Any;
}
