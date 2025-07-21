use std::mem;

use crate::{
    core::{math::gamma, tensor::Vec3f},
    raycast::*,
};

#[derive(Debug, Clone, Copy)]
pub struct Bounds3f {
    pub min: Vec3f,
    pub max: Vec3f,
}

impl Bounds3f {
    pub fn new(min: Vec3f, max: Vec3f) -> Bounds3f {
        Bounds3f { min, max }
    }

    pub fn zero() -> Bounds3f {
        use crate::core::tensor::Tensor;
        use crate::tensor;
        Bounds3f {
            min: tensor!(0.;3),
            max: tensor!(0.;3),
        }
    }

    pub fn centroid(&self) -> Vec3f {
        (self.min + self.max) * 0.5
    }

    pub fn union(&self, other: Bounds3f) -> Bounds3f {
        Bounds3f {
            min: self.min.min(other.min),
            max: self.max.max(other.max),
        }
    }

    pub fn enlarge(&self, p: Vec3f) -> Bounds3f {
        Bounds3f {
            min: self.min.min(p),
            max: self.max.max(p),
        }
    }

    pub fn offset(&self, p: Vec3f) -> Vec3f {
        let o = p - self.min;
        let d = self.diagonal();
        Vec3f::vec(&[
            if d[0] > 0. { o[0] / d[0] } else { o[0] },
            if d[1] > 0. { o[1] / d[1] } else { o[1] },
            if d[2] > 0. { o[2] / d[2] } else { o[2] },
        ])
    }

    pub fn diagonal(&self) -> Vec3f {
        self.max - self.min
    }

    pub fn max_dim(&self) -> usize {
        let d = self.diagonal();
        let x = d[0];
        let y = d[1];
        let z = d[2];
        if x > y && x > z {
            0
        } else if y > z {
            1
        } else {
            2
        }
    }

    pub fn area(&self) -> f32 {
        let d = self.diagonal();
        let x = d[0];
        let y = d[1];
        let z = d[2];
        2. * (x * y + x * z + y * z)
    }
}

impl Default for Bounds3f {
    fn default() -> Self {
        Bounds3f::zero()
    }
}

impl Raycast for Bounds3f {
    fn raycast(&self, ray: &Ray) -> Option<Hit> {
        //TODO: branch free testing
        let (mut t0, mut t1) = (0f32, ray.t_max);
        for i in 0..3 {
            let inv_dir = 1. / ray.dir[i];
            // inside axis aligned plane x = x0, t = (x0-org_x)/dir_x
            let mut tnear = (self.min[i] - ray.org[i]) * inv_dir;
            let mut tfar = (self.max[i] - ray.org[i]) * inv_dir;
            if tnear > tfar {
                mem::swap(&mut tnear, &mut tfar);
            }

            // robust intersect
            // TODO: explain
            tfar *= 1. + 2. * gamma(3);

            //NaN still works
            t0 = if tnear > t0 { tnear } else { t0 };
            t1 = if tfar < t1 { tfar } else { t1 };
            if t0 > t1 {
                return None;
            }
        }
        // if org_x = x0, not intersect at x0
        Some(Hit {
            ray: ray.clone(),
            t: if t0 > 0. { t0 } else { t1 },
        })
    }
}

impl PartialEq<Bounds3f> for Bounds3f {
    fn eq(&self, other: &Bounds3f) -> bool {
        self.min == other.min && self.max == other.max
    }
}

#[test]
fn test_bounds() {
    let b = Bounds3f::new(Vec3f::vec(&[-1.; 3]), Vec3f::vec(&[1.; 3]));
    let b1 = Bounds3f::new(Vec3f::vec(&[-1.; 3]), Vec3f::vec(&[2.; 3]));
    let b3 = b.union(b1);
    assert_eq!(b3.min[0], -1.);
    assert_eq!(b3.max[0], 2.);
    assert_eq!(b.centroid()[0], 0.);
}

#[test]
fn test_hit_bounds() {
    let e = 1e-4;
    let b = Bounds3f::new(Vec3f::vec(&[-1.; 3]), Vec3f::vec(&[1.; 3]));

    // on left
    let org = Vec3f::vec(&[-1. - e, 0., 0.]);
    let dir = Vec3f::vec(&[1., 0., 0.]);
    let ray = Ray::new(org, dir);

    let h = b.raycast(&ray);
    assert!(h.is_some());
    assert!((h.unwrap().t - e).abs() < e);

    // on x0
    let org = Vec3f::vec(&[-1., 0., 0.]);
    let dir = Vec3f::vec(&[1., 0., 0.]);
    let ray = Ray::new(org, dir);

    let h = b.raycast(&ray);
    assert!(h.is_some());
    assert!((h.unwrap().t - 2.) < e);

    // inside
    let org = Vec3f::vec(&[-1. + e, 0., 0.]);
    let ray = Ray::new(org, dir);
    let h = b.raycast(&ray);
    assert!(h.is_some());
    assert!((h.unwrap().t - 2. + e) < e);

    let org = Vec3f::vec(&[1. + e, 0., 0.]);
    let ray = Ray::new(org, dir);
    let h = b.raycast(&ray);
    assert!(h.is_none());
}
