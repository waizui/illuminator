use num_traits::Zero;

use crate::prelude::*;

pub struct Camera {
    pub pos: Vec3f,
    pub forward: Vec3f,
}

impl Camera {
    pub fn new() -> Self {
        Self::default()
    }

}

impl Default for Camera {
    fn default() -> Self {
        let forward = Vec3f::vec([0., 0., 1.]);
        Camera {
            pos: Vec3f::zero(),
            forward,
        }
    }
}
