use crate::raycast::{
    bounds::Bounds3f,
    morton::{MortonCode, encode_morton3, radix_sort},
    primitive::Primitive,
    *,
};
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
        #[derive(Default)]
        struct MortonPrim {
            morton_code: usize,
            prim_index: usize,
        }

        impl MortonCode for MortonPrim {
            fn morton_code(&self) -> usize {
                self.morton_code
            }
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
                morton_prim.morton_code = encode_morton3(offset);
            });

        radix_sort(&mut morton_prims);
    }
}

impl Raycast for BVH {
    fn raycast(&self, ray: Ray) -> Option<Hit> {
        todo!()
    }
}
