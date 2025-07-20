use crate::raycast::{bounds::Bounds3f, primitive::Primitive, *};

#[derive(Debug, Default)]
pub struct LinearBVHNode {
    pub bounds: Bounds3f,
    pub axis: usize,
    pub nprimitives: usize,
    /// primitive offset or second child offset
    pub offset: usize,
}

impl LinearBVHNode {
    pub fn is_leaf(&self) -> bool {
        self.nprimitives > 0
    }
}

pub struct BVH {
    pub node_prims_limit: usize, // max primitives a node can include
    pub primitives: Vec<Box<dyn Primitive>>,
    pub nodes: Vec<LinearBVHNode>,
}

impl BVH {
    pub fn new(capacity: usize) -> BVH {
        BVH {
            node_prims_limit: 65,
            primitives: Vec::with_capacity(capacity),
            nodes: Vec::new(),
        }
    }

    pub fn push(&mut self, prim: impl Primitive + 'static) {
        self.primitives.push(Box::new(prim));
    }

    pub fn bounds(&self) -> Bounds3f {
        if !self.nodes.is_empty() {
            self.nodes[0].bounds
        } else {
            Bounds3f::default()
        }
    }
}

impl Raycast for BVH {
    fn raycast(&self, ray: &Ray) -> Option<Hit> {
        let mut hit: Option<Hit> = None;

        let mut cur_node_i = 0;
        let mut to_visit_i = 0;
        let mut nodes_to_visit = [0; 64];

        let ray = &mut ray.clone();

        loop {
            let node = &self.nodes[cur_node_i];
            if node.bounds.raycast(ray).is_some() {
                if node.is_leaf() {
                    // cast ray with primitives
                    for i in 0..node.nprimitives {
                        if let Some(hit_p) = self.primitives[node.offset + i].raycast(ray) {
                            //update t_max to find nearest primitive
                            ray.t_max = hit_p.t;
                            hit = Some(hit_p);
                        }
                    }
                    if to_visit_i == 0 {
                        break;
                    }

                    cur_node_i = nodes_to_visit[to_visit_i - 1];
                    to_visit_i -= 1;
                } else {
                    // not leaf, put far BVH node on nodes_to_visit stack, advance to near node
                    if ray.dir[node.axis] < 0. {
                        // far node is left
                        nodes_to_visit[to_visit_i] = cur_node_i + 1;
                        cur_node_i = node.offset;
                    } else {
                        // far node is right
                        nodes_to_visit[to_visit_i] = node.offset;
                        cur_node_i += 1;
                    }
                    to_visit_i += 1;
                }
            } else {
                // not hit
                if to_visit_i == 0 {
                    break;
                }
                cur_node_i = nodes_to_visit[to_visit_i - 1];
                to_visit_i -= 1;
            }
        }

        hit
    }
}

#[test]
fn test_bvh_order() {
    use crate::raycast::sphere::Sphere;
    use rand::seq::SliceRandom;
    /*
     if n > 1024, test might fail since more than one points can map to same code,
    radix_sort use a 30bit 10bit-per axis setting, meaning only 1024 sperated points
    for each axis.
    */
    let n = 1024;
    let node_limit = 17;

    let mut bvh = BVH::new(n);
    let mut arr: Vec<usize> = (0..n).collect();
    let mut rng = rand::rng();
    arr.shuffle(&mut rng);
    for &i in arr.iter() {
        let sph = Sphere::new(Float3::vec(&[i as f32 + 0.5; 3]), 0.5);
        bvh.push(sph);
    }

    // sequential build , all primitives are sequentially ordered
    bvh.build(node_limit, false);

    for i in 1..n {
        let b1 = bvh.primitives[i].bounds();
        let b0 = bvh.primitives[i - 1].bounds();
        if b1.min[0] < b0.min[0] {
            panic!("b1:{} < b0:{}", i, i - 1);
        }
        assert!(b1.min[0] >= b0.min[0])
    }
}

#[test]
fn test_bvh_nodes() {
    use crate::raycast::sphere::Sphere;
    use rand::seq::SliceRandom;

    let n = 1024;
    let node_limit = 17;

    let mut bvh = BVH::new(n);
    let mut arr: Vec<usize> = (0..n).collect();
    let mut rng = rand::rng();
    arr.shuffle(&mut rng);
    for &i in arr.iter() {
        let sph = Sphere::new(Float3::vec(&[i as f32 + 0.5; 3]), 0.5);
        bvh.push(sph);
    }

    // parallel build , primitives are sequentially ordered inside segment
    bvh.build(node_limit, true);

    let mut start = 0;
    while start < bvh.nodes.len() - 1 {
        let c = &bvh.nodes[start];
        let mut b = Bounds3f::default();
        if c.is_leaf() {
            for i in 0..c.nprimitives {
                let cb = bvh.primitives[c.offset + i].bounds();
                b = cb.union(b);
            }
            assert_eq!(b, c.bounds);
        } else {
            if start + 1 < bvh.nodes.len() {
                let c0 = &bvh.nodes[start + 1];
                b = c0.bounds.union(b);
            }
            if c.offset < bvh.nodes.len() {
                let c1 = &bvh.nodes[c.offset];
                b = c1.bounds.union(b);
            }
            assert_eq!(b, c.bounds)
        }

        start += 1;
    }
}

#[test]
fn test_bvh_cast() {
    use crate::raycast::sphere::Sphere;
    use rand::seq::SliceRandom;
    use std::time::Instant;

    let n = 1024;
    let node_limit = 17;

    let mut bvh = BVH::new(n);
    let mut arr: Vec<usize> = (0..n).collect();
    let mut rng = rand::rng();
    arr.shuffle(&mut rng);
    let mut rays: Vec<(usize, Ray)> = Vec::new();
    for &i in arr.iter() {
        let cnt = Float3::vec(&[i as f32 + 0.5; 3]);
        let sph = Sphere::new(cnt, 0.5);
        bvh.push(sph);

        let org = Float3::vec(&[i as f32 + 0.5, i as f32 + 0.5, 1025.]);
        let dir = Float3::vec(&[0., 0., -1.]);
        rays.push((i, Ray::new(org, dir)));
    }
    bvh.build(node_limit, true);

    let sw = Instant::now();

    rays.iter().for_each(|(i, ray)| {
        let hit = bvh.raycast(ray);
        assert!(hit.is_some());
        assert_eq!(hit.unwrap().t, 1024. - *i as f32);
    });

    println!(
        "raycast {} prims bvh, {}ms",
        bvh.primitives.len(),
        sw.elapsed().as_millis()
    );
}

#[test]
fn test_bvh_perf() {
    use crate::raycast::sphere::Sphere;
    use rand::seq::SliceRandom;
    use std::time::Instant;

    for n in [1024, 2048, 4096] {
        let node_limit = 65;

        let mut bvh = BVH::new(n);
        let mut arr: Vec<usize> = (0..n).collect();
        let mut rng = rand::rng();
        arr.shuffle(&mut rng);
        let mut rays: Vec<(usize, Ray)> = Vec::new();
        for &i in arr.iter() {
            let cnt = Float3::vec(&[i as f32 + 0.5; 3]);
            let sph = Sphere::new(cnt, 0.5);
            bvh.push(sph);

            let org = Float3::vec(&[i as f32 + 0.5, i as f32 + 0.5, n as f32 + 1.]);
            let dir = Float3::vec(&[0., 0., -1.]);
            rays.push((i, Ray::new(org, dir)));
        }
        bvh.build(node_limit, true);

        let sw = Instant::now();

        rays.iter().take(1024).for_each(|(_, ray)| {
            let hit = bvh.raycast(ray);
            assert!(hit.is_some());
        });

        println!(
            "raycast 1024 times, {} prims bvh, {}ms",
            bvh.primitives.len(),
            sw.elapsed().as_millis()
        );
    }
}
