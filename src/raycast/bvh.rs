use std::sync::{
    Arc, Mutex,
    atomic::{AtomicUsize, Ordering::SeqCst},
};

use crate::{
    core::math::split_index,
    raycast::{
        bounds::Bounds3f,
        morton::{MortonCode, encode_morton3, radix_sort},
        primitive::Primitive,
        *,
    },
};
use rayon::prelude::*;

pub struct BVHBuildNode {
    bounds: Bounds3f,
    axis: usize,
    prim_offset: usize,
    prim_count: usize,
    c0: Option<Arc<BVHBuildNode>>,
    c1: Option<Arc<BVHBuildNode>>,
}

impl BVHBuildNode {
    pub fn bounds(&self) -> Bounds3f {
        self.bounds
    }

    pub fn init_leaf(&mut self, first: usize, n: usize, b: Bounds3f) {
        self.prim_count += n;
        self.prim_offset = first;
        self.bounds = b;
        self.c0 = None;
        self.c1 = None;
    }

    pub fn init_inerior(&mut self, axis: usize, c0: Arc<BVHBuildNode>, c1: Arc<BVHBuildNode>) {
        self.prim_count = 0;
        self.axis = axis;
        self.bounds = c0.bounds.union(c1.bounds);
        self.c0 = Some(c0);
        self.c1 = Some(c1);
    }
}

impl Default for BVHBuildNode {
    fn default() -> Self {
        BVHBuildNode {
            bounds: Bounds3f::zero(),
            prim_offset: 0,
            prim_count: 0,
            axis: 0,
            c0: None,
            c1: None,
        }
    }
}

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
    root: Option<Arc<BVHBuildNode>>,
    nodes: Vec<Arc<BVHBuildNode>>, // root node of treelet
}

pub struct BVH {
    node_prims_limit: usize,
    primitives: Vec<Box<dyn Primitive>>,
}

impl BVH {
    pub fn new(capacity: usize) -> BVH {
        BVH {
            node_prims_limit: 256,
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

        {
            let mut start = 0;
            let mut end = 1;
            let prims_size = morton_prims.len();
            while end <= prims_size {
                // use hight 12 bits to cluster treelets
                let mask = 0b00111111111111000000000000000000;
                if (end == prims_size)
                    || (morton_prims[start].morton_code & mask)
                        != (morton_prims[end].morton_code & mask)
                {
                    let nprimitives = end - start;
                    let max_nodes = 2 * nprimitives - 1;
                    let mut nodes = Vec::with_capacity(max_nodes);
                    for _ in 0..max_nodes {
                        nodes.push(Arc::new(BVHBuildNode::default()));
                    }
                    treelets.push(Treelet {
                        start_index: start,
                        nprimitives,
                        root: None,
                        nodes,
                    });
                    start = end;
                }

                end += 1;
            }
        };

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
            let ordered_prims = ordered_prims.clone();
            let ordered_prims_offset = ordered_prims_offset.clone();
            let mut nodes_created = 0;
            let root = self.emit_lbvh(
                &mut tr.nodes[0..],
                &morton_prims[tr.start_index..tr.start_index + tr.nprimitives],
                tr.nprimitives,
                ordered_prims,
                ordered_prims_offset,
                &mut nodes_created,
                first_bit_index,
            );

            tr.root = Some(root);
            let total_nodes = total_nodes.clone();
            let mut guard = total_nodes.lock().unwrap();
            *guard += nodes_created;
        });

        let mut treelet_roots: Vec<Arc<BVHBuildNode>> = Vec::with_capacity(treelets.len());
        treelets.iter().for_each(|tr| {
            if let Some(root) = &tr.root {
                treelet_roots.push(root.clone());
            }
        });
        self.build_sah(&treelet_roots, 0, treelets.len(), &total_nodes);

        // swap ordered primitives and original primitives
        self.primitives = Arc::try_unwrap(ordered_prims).unwrap();
    }

    #[allow(clippy::too_many_arguments)]
    fn emit_lbvh(
        &self,
        build_nodes: &mut [Arc<BVHBuildNode>],
        morton_prims: &[MortonPrim],
        nprimitives: usize,
        ordered_prims: Arc<Vec<Box<dyn Primitive>>>,
        ordered_prims_offset: Arc<AtomicUsize>,
        created_nodes: &mut usize,
        bit_index: i32,
    ) -> Arc<BVHBuildNode> {
        if bit_index == -1 || nprimitives < self.node_prims_limit {
            let first_prim_offset = ordered_prims_offset.fetch_add(nprimitives, SeqCst);
            *created_nodes += 1;
            let node = build_nodes[0].clone();
            let mut bounds = Bounds3f::zero();
            unsafe {
                let base_ptr = ordered_prims.as_ptr() as *mut Box<dyn Primitive>;
                for (i, morton_prim) in morton_prims.iter().enumerate() {
                    let org_prim_index = morton_prim.prim_index;
                    let prim_box = self.primitives[org_prim_index].clone_as_box();
                    bounds = bounds.union(prim_box.bounds());
                    // it is thread safe since first_prim_offset is unique per thread
                    let cur_prim_index = first_prim_offset + i;
                    std::ptr::write(base_ptr.add(cur_prim_index), prim_box);
                }

                let node_ptr = Arc::as_ptr(&node) as *mut BVHBuildNode;
                (*node_ptr).init_leaf(first_prim_offset, nprimitives, bounds);
            }
            node
        } else {
            let mask = 1 << bit_index;
            let first_morton = morton_prims[0].morton_code;
            // advance to next subtree level if there is no LBVH split for this bit
            if (first_morton & mask) == (morton_prims[nprimitives - 1].morton_code & mask) {
                return self.emit_lbvh(
                    build_nodes,
                    morton_prims,
                    nprimitives,
                    ordered_prims,
                    ordered_prims_offset,
                    created_nodes,
                    bit_index - 1,
                );
            }

            let mut splite_offset = split_index(nprimitives, |i| {
                (first_morton & mask) == (morton_prims[i].morton_code & mask)
            });

            splite_offset += 1;
            *created_nodes += 1;

            let c0 = self.emit_lbvh(
                &mut build_nodes[1..],
                morton_prims,
                splite_offset,
                Arc::clone(&ordered_prims),
                Arc::clone(&ordered_prims_offset),
                created_nodes,
                bit_index - 1,
            );

            let c1 = self.emit_lbvh(
                &mut build_nodes[1..],
                morton_prims,
                nprimitives - splite_offset,
                ordered_prims,
                ordered_prims_offset,
                created_nodes,
                bit_index - 1,
            );

            let node = build_nodes[0].clone();
            let axis = (bit_index % 3) as usize;
            unsafe {
                let node_ptr = Arc::as_ptr(&node) as *mut BVHBuildNode;
                (*node_ptr).init_inerior(axis, c0, c1);
            }
            node
        }
    }

    /// build treelets node use Surface Area Heuristic
    fn build_sah(
        &mut self,
        treelet_roots: &[Arc<BVHBuildNode>],
        start: usize,
        end: usize,
        total_nodes: &Arc<Mutex<usize>>,
    ) -> Arc<BVHBuildNode> {
        let nnodes = end - start;
        if nnodes == 1 {
            return treelet_roots[start].clone();
        }

        let mut total_nodes = total_nodes.lock().unwrap();
        (*total_nodes) += 1;

        let mut node = Arc::new(BVHBuildNode::default());

        //TODO: impl
        node
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
    use rand::seq::SliceRandom;
    let n = 8;
    let mut bvh = BVH::new(n);
    let mut arr: Vec<usize> = (0..n).collect();
    let mut rng = rand::rng();
    arr.shuffle(&mut rng);
    for &i in arr.iter() {
        let sph = Sphere::new(Float3::vec(&[i as f32; 3]), 1.);
        bvh.push(sph);
    }

    bvh.push(Sphere::new(Float3::vec(&[1022f32; 3]), 1.));
    bvh.build(64);

    for i in 1..n {
        let b1 = bvh.primitives[i].bounds();
        let b0 = bvh.primitives[i - 1].bounds();
        assert!(b1.min.get(0) > b0.min.get(0));
    }
}
