use crate::raycast::bvh::{BVH, LinearBVHNode};
use crate::{
    core::math::split_index,
    raycast::{
        bounds::Bounds3f,
        morton::{MortonCode, encode_morton3, radix_sort},
        primitive::Primitive,
    },
};
use rayon::prelude::*;
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicUsize, Ordering::Relaxed},
};

///how many splits in sah building, efficient setting
const N_BUCKETS: usize = 12;

#[derive(Default)]
struct BVHBuildNode {
    bounds: Bounds3f,
    axis: usize,
    prim_offset: usize,
    nprimitives: usize,
    c0: Option<Arc<BVHBuildNode>>,
    c1: Option<Arc<BVHBuildNode>>,
}

impl BVHBuildNode {
    pub fn bounds(&self) -> Bounds3f {
        self.bounds
    }

    pub fn init_leaf(&mut self, first: usize, n: usize, b: Bounds3f) {
        self.nprimitives += n;
        self.prim_offset = first;
        self.bounds = b;
        self.c0 = None;
        self.c1 = None;
    }

    pub fn init_interior(&mut self, axis: usize, c0: Arc<BVHBuildNode>, c1: Arc<BVHBuildNode>) {
        self.nprimitives = 0;
        self.axis = axis;
        self.bounds = c0.bounds.union(c1.bounds);
        self.c0 = Some(c0);
        self.c1 = Some(c1);
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

// imple linear bvh build
impl BVH {
    pub fn build(&mut self, node_prims_limit: usize, par_build: bool) {
        self.node_prims_limit = node_prims_limit;
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
                // use hight 12 bits to cluster treelets, clustering inside 16x16x16 grid
                let mask = 0b00111111111111000000000000000000;
                if (end == prims_size)
                    || (morton_prims[start].morton_code & mask)
                        != (morton_prims[end].morton_code & mask)
                {
                    let nprimitives = end - start;
                    // for n primitives max nodes less than n^2 -1
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

        let build_task = |tr: &mut Treelet| {
            // i-th treelet
            // 30 bits morton code , 12 bits used for building treelet clusters
            let first_bit_index: i32 = 29 - 12;
            let ordered_prims = ordered_prims.clone();
            let ordered_prims_offset = ordered_prims_offset.clone();
            let (root, nodes_created) = self.emit_lbvh(
                &mut tr.nodes[0..],
                &morton_prims[tr.start_index..tr.start_index + tr.nprimitives],
                tr.nprimitives,
                ordered_prims,
                ordered_prims_offset,
                first_bit_index,
            );

            tr.root = Some(root);
            let mut guard = total_nodes.lock().unwrap();
            *guard += nodes_created;
        };

        if par_build {
            treelets.par_iter_mut().for_each(build_task);
        } else {
            treelets.iter_mut().for_each(build_task);
        }

        let mut treelet_roots: Vec<Arc<BVHBuildNode>> = Vec::with_capacity(treelets.len());
        treelets.iter().for_each(|tr| {
            if let Some(root) = &tr.root {
                treelet_roots.push(root.clone());
            }
        });

        let (root, sah_created_nodes) = self.build_sah(&treelet_roots);
        assert!(sah_created_nodes < treelet_roots.len() * 2);
        let mut total_nodes = total_nodes.lock().unwrap();
        *total_nodes += sah_created_nodes;


        // swap ordered primitives and original primitives
        let ordered_prims = match Arc::try_unwrap(ordered_prims) {
            Ok(p_vec) => p_vec,
            Err(arc_p_vec) => {
                panic!(
                    "ordered_prims is still shared (count: {}).",
                    Arc::strong_count(&arc_p_vec)
                );
            }
        };

        let _ = std::mem::replace(&mut self.primitives, ordered_prims);
        self.nodes.resize_with(*total_nodes, LinearBVHNode::default);
        let mut offset = Box::new(0);
        self.flatten_bvh(&root, &mut offset);
    }

    /// returns root node of sub tree and created nodes num
    fn emit_lbvh(
        &self,
        build_nodes: &mut [Arc<BVHBuildNode>],
        morton_prims: &[MortonPrim],
        nprimitives: usize,
        ordered_prims: Arc<Vec<Box<dyn Primitive>>>,
        ordered_prims_offset: Arc<AtomicUsize>,
        bit_index: i32,
    ) -> (Arc<BVHBuildNode>, usize) {
        if bit_index == -1 || nprimitives < self.node_prims_limit {
            let first_prim_offset = ordered_prims_offset.fetch_add(nprimitives, Relaxed);
            let node = build_nodes[0].clone();
            let mut bounds = Bounds3f::zero();

            unsafe {
                let vec_ptr = Arc::as_ptr(&ordered_prims) as *mut Vec<Box<dyn Primitive>>;
                let buffer_ptr = (*vec_ptr).as_mut_ptr();

                for (i, morton_prim) in morton_prims.iter().take(nprimitives).enumerate() {
                    let org_prim_index = morton_prim.prim_index;
                    let prim_box_ptr = self.primitives[org_prim_index].clone_as_box();
                    bounds = bounds.union(prim_box_ptr.bounds());
                    let cur_prim_index = first_prim_offset + i;
                    std::ptr::write(buffer_ptr.add(cur_prim_index), prim_box_ptr);
                }

                let node_ptr = Arc::as_ptr(&node) as *mut BVHBuildNode;
                (*node_ptr).init_leaf(first_prim_offset, nprimitives, bounds);
            }

            (node, 1)
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
                    bit_index - 1,
                );
            }

            let mut split_offset = split_index(nprimitives, |i| {
                // split index of current bit index
                (first_morton & mask) == (morton_prims[i].morton_code & mask)
            });

            split_offset += 1; //split to build subtree
            assert!(split_offset < nprimitives);
            assert_ne!(
                morton_prims[split_offset - 1].morton_code & mask,
                morton_prims[split_offset].morton_code & mask
            );

            let (c0, c0_created_nodes) = self.emit_lbvh(
                &mut build_nodes[1..], // [0] as current node
                morton_prims,
                split_offset,
                Arc::clone(&ordered_prims),
                Arc::clone(&ordered_prims_offset),
                bit_index - 1,
            );

            let (c1, c1_created_nodes) = self.emit_lbvh(
                &mut build_nodes[1 + c0_created_nodes..],
                &morton_prims[split_offset..],
                nprimitives - split_offset,
                ordered_prims,
                ordered_prims_offset,
                bit_index - 1,
            );

            let node = build_nodes[0].clone();
            let axis = (bit_index % 3) as usize;
            unsafe {
                let node_ptr = Arc::as_ptr(&node) as *mut BVHBuildNode;
                (*node_ptr).init_interior(axis, c0, c1);
            }

            (node, c0_created_nodes + c1_created_nodes + 1)
        }
    }

    /// build treelets node use Surface Area Heuristic
    #[allow(clippy::only_used_in_recursion)]
    fn build_sah(&self, treelet_roots: &[Arc<BVHBuildNode>]) -> (Arc<BVHBuildNode>, usize) {
        if treelet_roots.len() == 1 {
            return (treelet_roots[0].clone(), 0);
        }

        let centroid_bounds = treelet_roots.iter().fold(Bounds3f::zero(), |acc, node| {
            acc.enlarge(node.bounds().centroid())
        });
        let dim = centroid_bounds.max_dim();
        // need handle when this hits
        assert_ne!(centroid_bounds.min[dim], centroid_bounds.max[dim]);

        #[derive(Default, Clone, Copy)]
        struct BVHSplitBucket {
            count: usize,
            bounds: Bounds3f,
        }

        let mut buckets = [BVHSplitBucket::default(); N_BUCKETS];

        // init partition buckets alone max dimension
        treelet_roots.iter().for_each(|node| {
            let centroid = node.bounds().centroid()[dim];
            let centroid_offset = (centroid - centroid_bounds.min[dim])
                / (centroid_bounds.max[dim] - centroid_bounds.min[dim]);
            let mut b = ((centroid_offset) * N_BUCKETS as f32) as usize;
            if b == N_BUCKETS {
                b = N_BUCKETS - 1;
            }

            buckets[b].count += 1;
            buckets[b].bounds = buckets[b].bounds.union(node.bounds());
        });

        let bounds = treelet_roots
            .iter()
            .fold(Bounds3f::zero(), |acc, node| node.bounds().union(acc));

        // compute costs for splitting after each bucket
        let mut cost = [0.; N_BUCKETS - 1];
        cost.iter_mut().enumerate().for_each(|(i, c)| {
            let (b0, c0) = buckets
                .iter()
                .take(i + 1)
                .fold((Bounds3f::zero(), 0), |(b, c), bk| {
                    (b.union(bk.bounds), c + bk.count)
                });

            let (b1, c1) = buckets
                .iter()
                .take(N_BUCKETS)
                .skip(i + 1)
                .fold((Bounds3f::zero(), 0), |(b, c), bk| {
                    (b.union(bk.bounds), c + bk.count)
                });

            *c = 0.125 + (c0 as f32 * b0.area() + c1 as f32 * b1.area()) / bounds.area();
        });

        // find bucket to split at that minimizes SAH metric
        let (min_cost_index, _) = cost.iter().enumerate().take(N_BUCKETS - 1).skip(1).fold(
            (0, cost[0]),
            |(im, m), (i, &cost)| {
                if cost < m { (i, cost) } else { (im, m) }
            },
        );

        // return how many elements satisfy the predicate
        let (start, end): (Vec<_>, Vec<_>) = treelet_roots.iter().partition(|node| {
            let centroid = node.bounds().centroid()[dim];
            let centroid_offset = (centroid - centroid_bounds.min[dim])
                / (centroid_bounds.max[dim] - centroid_bounds.min[dim]);

            let mut b = ((centroid_offset) * N_BUCKETS as f32) as usize;
            if b == N_BUCKETS {
                b = N_BUCKETS - 1;
            }

            b <= min_cost_index
        });

        // handle corner cases, eg. all centroids located same place
        // forcing split by fisrt element
        let (left, right) = if start.is_empty() {
            let left = vec![end[0].clone()];
            let right = end[1..].iter().map(|&x| x.clone()).collect();
            (left, right)
        } else if end.is_empty() {
            let left = start[..start.len() - 1]
                .iter()
                .map(|&x| x.clone())
                .collect();
            let right = vec![start[start.len() - 1].clone()];
            (left, right)
        } else {
            (
                start.iter().map(|&x| x.clone()).collect(),
                end.iter().map(|&x| x.clone()).collect(),
            )
        };

        let (c0, c0_created_nodes) = self.build_sah(&left);
        let (c1, c1_created_nodes) = self.build_sah(&right);

        let node = Arc::new(BVHBuildNode::default());
        unsafe {
            let node_ptr = Arc::as_ptr(&node) as *mut BVHBuildNode;
            (*node_ptr).init_interior(dim, c0, c1);
        }

        (node, c0_created_nodes + c1_created_nodes + 1)
    }

    //compact memory
    fn flatten_bvh(&mut self, root: &BVHBuildNode, offset: &mut Box<usize>) -> usize {
        let node_offset = **offset;
        **offset += 1;
        let lnode = &mut self.nodes[node_offset];
        lnode.bounds = root.bounds;
        // leaf
        if root.nprimitives > 0 {
            lnode.offset = root.prim_offset;
            lnode.nprimitives = root.nprimitives;
        } else {
            // interior
            lnode.axis = root.axis;
            lnode.nprimitives = 0;

            if let Some(c0) = &root.c0 {
                self.flatten_bvh(c0, offset);
            }

            if let Some(c1) = &root.c1 {
                let i = self.flatten_bvh(c1, offset);
                // put there since borrow checker
                let lnode = &mut self.nodes[node_offset];
                lnode.offset = i;
            }
        }

        node_offset
    }
}
