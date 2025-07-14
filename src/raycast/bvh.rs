use std::sync::{Arc, Mutex, atomic::AtomicUsize, atomic::Ordering::SeqCst};

use crate::raycast::{
    bounds::Bounds3f,
    morton::{MortonCode, encode_morton3, radix_sort},
    primitive::Primitive,
    *,
};
use rayon::prelude::*;

#[derive(Default)]
pub struct BVHBuildNode {}

#[derive(Default, Clone)]
struct MortonPrim {
    morton_code: usize,
    prim_index: usize,
}

impl MortonCode for MortonPrim {
    fn morton_code(&self) -> usize {
        self.morton_code
    }
}

#[derive(Default)]
struct Treelet {
    start_index: usize,
    nprimitives: usize,
    nodes: Vec<BVHBuildNode>, // root node of treelet
}

pub struct BVH {
    root: BVHBuildNode,
    node_prims_limit: usize,
    primitives: Vec<Box<dyn Primitive>>,
}

impl BVH {
    pub fn new(capacity: usize) -> BVH {
        BVH {
            node_prims_limit: 256,
            root: BVHBuildNode {},
            primitives: Vec::with_capacity(capacity),
        }
    }

    pub fn push(&mut self, prim: impl Primitive + 'static) {
        self.primitives.push(Box::new(prim));
    }

    pub fn build(&mut self, prims_limit: usize) {
        self.node_prims_limit = prims_limit;
        // bounds of whole bvh
        let bounds = self
            .primitives
            .iter()
            .fold(Bounds3f::zero(), |acc, b| acc.union(b.bounds()));

        let mut morton_prims: Vec<MortonPrim> = vec![MortonPrim::default(); self.primitives.len()];
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

        // create LBVH treelets at bottom of BVH
        // find intervals of primitives for each treelet
        let mut treelets: Vec<Treelet> = Vec::new();
        let mut start = 0;
        let mut end = 1;
        let prims_size = morton_prims.len();
        while end <= prims_size {
            // use hight 12 bits to cluster treelets
            let mask = 0b00111111111111000000000000000000;
            if (end == prims_size)
                || ((morton_prims[start].morton_code & mask)
                    != (morton_prims[end].morton_code & mask))
            {
                let nprimitives = end - start;
                let max_nodes = 2 * nprimitives - 1;
                let nodes = Vec::with_capacity(max_nodes);
                treelets.push(Treelet {
                    start_index: start,
                    nprimitives,
                    nodes,
                });
                start = end;
            }

            end += 1;
        }

        let mut p: Vec<Box<dyn Primitive>> = Vec::with_capacity(self.primitives.len());
        let ordered_prims = {
            unsafe {
                p.set_len(self.primitives.len());
            };
            Arc::new(p)
        };
        let ordered_prims_offset = Arc::new(AtomicUsize::new(0));
        let total_nodes = Arc::new(Mutex::new(0usize));

        treelets.par_iter_mut().for_each(|tr| {
            // i-th treelet
            // 30 bits morton code , 12 bits used for building treelet clusters
            let first_bit_index: i32 = 29 - 12;
            let ordered_prims = Arc::clone(&ordered_prims);
            let ordered_prims_offset = Arc::clone(&ordered_prims_offset);
            let mut nodes_created = 0;
            self.emit_lbvh(
                &mut tr.nodes,
                &morton_prims[tr.start_index..tr.nprimitives],
                tr.nprimitives,
                ordered_prims,
                ordered_prims_offset,
                &mut nodes_created,
                first_bit_index,
            );
            let total_nodes = Arc::clone(&total_nodes);
            let mut guard = total_nodes.lock().unwrap();
            *guard += nodes_created;
        });

        //TODO: replace ordered prims
    }

    #[allow(clippy::too_many_arguments)]
    fn emit_lbvh(
        &self,
        build_nodes: &mut [BVHBuildNode],
        morton_prims: &[MortonPrim],
        nprimitives: usize,
        ordered_prims: Arc<Vec<Box<dyn Primitive>>>,
        ordered_prims_offset: Arc<AtomicUsize>,
        created_nodes: &mut usize,
        bit_index: i32,
    ) {
        if bit_index == -1 || nprimitives < self.node_prims_limit {
            let prim_offset = ordered_prims_offset.fetch_add(nprimitives, SeqCst);
            unsafe {

                let mut bounds = Bounds3f::zero();
                let ordered_prims_ptr = ordered_prims.as_ptr() as *mut Vec<Box<dyn Primitive>>;
                let ordered_prims_ref = &mut *ordered_prims_ptr;
                for (i, morton_prim) in morton_prims.iter().enumerate() {
                    let org_prim_index = morton_prim.prim_index;
                    let prim_box = self.primitives[org_prim_index].clone_as_box();
                    bounds = bounds.union(prim_box.bounds());
                    let cur_prim_index = prim_offset + i;
                    ordered_prims_ref[cur_prim_index] = prim_box;
                }
                //TODO build node
            }
        } else {
            todo!()
        }
    }
}

impl Raycast for BVH {
    fn raycast(&self, ray: Ray) -> Option<Hit> {
        todo!()
    }
}

#[test]
fn test_bvh() {
    use crate::raycast::sphere::Sphere;
    let n = 8;
    let mut bvh = BVH::new(n);
    for i in 0..n {
        let sph = Sphere::new(Float3::vec(&[i as f32; 3]), 1.);
        bvh.push(sph);
    }

    bvh.build(64);
}
