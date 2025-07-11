use crate::raycast::{bounds::Bounds3f, primitive::Primitive, *};
use rayon::prelude::*;

pub struct BVHNode {}

pub struct BVH {
    root: BVHNode,
    primitives: Vec<Box<dyn Primitive>>,
}

impl BVH {
    pub fn push(&self) {
        todo!()
    }

    pub fn build(&self) {
        struct MortonPrim {
            morton_code: usize,
            prim_index: usize,
        }

        // bound of whole bvh
        let bounds = self
            .primitives
            .iter()
            .fold(Bounds3f::zero(), |acc, b| acc.union(b.bounds()));

        let mut morton_prims: Vec<MortonPrim> = Vec::with_capacity(self.primitives.len());
        morton_prims
            .par_iter_mut()
            .enumerate()
            .for_each(|(i, morton_prim)| {
                let morton_bits: usize = 10;
                let morton_scale = 1 << morton_bits;
                morton_prim.prim_index = i;
                let cnt_offset = bounds.offset(self.primitives[i].bounds().centroid());
                let offset = cnt_offset * morton_scale as f32;
                morton_prim.morton_code = encode_morton(offset);
            });
    }
}

fn encode_morton(p: Float3) -> usize {
    todo!()
}

impl Raycast for BVH {
    fn raycast(&self, ray: Ray) -> Option<Hit> {
        todo!()
    }
}
