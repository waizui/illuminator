use crate::core::{quaternion::Quat, tensor::Vec3f};

#[derive(Debug, Clone, Copy)]
pub struct Gaussian {
    pub rot: Quat,
    pub pos: Vec3f,
    pub scale: Vec3f,
    pub opacity: f32,
    pub dc0: Vec3f,
    pub sh: [Vec3f; 15],
}
