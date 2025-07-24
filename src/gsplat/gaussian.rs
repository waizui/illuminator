use crate::{
    core::quaternion::Quat,
    gsplat::io::InputSplat,
    prelude::Vec3f,
    raycast::{Hit, Ray, Raycast, bounds::Bounds3f, primitive::Primitive},
};

#[derive(Debug, Clone, Copy)]
pub struct Splat {
    pub pos: Vec3f,
    pub nor: Vec3f,
    pub col: Vec3f,
    pub sh: [Vec3f; 15],
    pub opacity: f32,
    pub scale: Vec3f,
    pub rot: Quat,
}

impl Splat {
    pub fn from_input(input: &InputSplat) -> Self {
        const SH_C0: f32 = 0.2820948;
        // let col = Vec3f::vec(input.dc0) * SH_C0 + 0.5;
        let col = Vec3f::vec(input.dc0);

        let sh = std::array::from_fn(|i| {
            let reodered_sh = [input.sh[i], input.sh[i + 15], input.sh[i + 30]];
            Vec3f::vec(reodered_sh)
        });

        Splat {
            pos: Vec3f::vec(input.pos),
            nor: Vec3f::vec(input.nor),
            col,
            sh,
            opacity: input.opacity,
            scale: Vec3f::vec(input.scale),
            rot: Quat::from_array(input.rot),
        }
    }
}

impl Raycast for Splat {
    fn raycast(&self, ray: &Ray) -> Option<Hit> {
        self.bounds().raycast(ray)
    }
}

impl Primitive for Splat {
    fn bounds(&self) -> Bounds3f {
        let one = Vec3f::vec([0.01; 3]);
        Bounds3f {
            min: self.pos - one,
            max: self.pos + one,
        }
    }

    fn clone_as_box(&self) -> Box<dyn Primitive> {
        Box::new(*self)
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
