use std::mem;

use crate::{
    core::{math::gamma, tensor::Float3},
    raycast::*,
};

#[derive(Debug, Clone, Copy)]
pub struct Bounds3f {
    pub min: Float3,
    pub max: Float3,
}

impl Bounds3f {
    pub fn new(min: Float3, max: Float3) -> Bounds3f {
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

    pub fn centroid(&self) -> Float3 {
        (self.min + self.max) * 0.5
    }

    pub fn union(&self, other: Bounds3f) -> Bounds3f {
        Bounds3f {
            min: self.min.min(other.min),
            max: self.max.max(other.max),
        }
    }

    pub fn enlarge(&self, p: Float3) -> Bounds3f {
        Bounds3f {
            min: self.min.min(p),
            max: self.max.max(p),
        }
    }

    pub fn offset(&self, p: Float3) -> Float3 {
        let o = p - self.min;
        let d = self.diagonal();
        Float3::vec(&[
            if d.get(0) > 0. {
                o.get(0) / d.get(0)
            } else {
                o.get(0)
            },
            if d.get(1) > 0. {
                o.get(1) / d.get(1)
            } else {
                o.get(1)
            },
            if d.get(2) > 0. {
                o.get(2) / d.get(2)
            } else {
                o.get(2)
            },
        ])
    }

    pub fn diagonal(&self) -> Float3 {
        self.max - self.min
    }

    pub fn max_dim(&self) -> usize {
        let d = self.diagonal();
        let x = d.get(0);
        let y = d.get(1);
        let z = d.get(2);
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
        let x = d.get(0);
        let y = d.get(1);
        let z = d.get(2);
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
        let (mut t0, mut t1) = (0f32, ray.t_max);
        for i in 0..3 {
            let inv_dir = 1. / ray.dir.get(i);
            // inside axis aligned plane x = x0, t = (x0-org_x)/dir_x
            let mut tnear = (self.min.get(i) - ray.org.get(i)) * inv_dir;
            let mut tfar = (self.max.get(i) - ray.org.get(i)) * inv_dir;
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
    let b = Bounds3f::new(Float3::vec(&[-1.; 3]), Float3::vec(&[1.; 3]));
    let b1 = Bounds3f::new(Float3::vec(&[-1.; 3]), Float3::vec(&[2.; 3]));
    let b3 = b.union(b1);
    assert_eq!(b3.min[&[0]], -1.);
    assert_eq!(b3.max[&[0]], 2.);
    assert_eq!(b.centroid()[&[0]], 0.);
}

#[test]
fn test_hit_bounds() {
    let e = 1e-4;
    let b = Bounds3f::new(Float3::vec(&[-1.; 3]), Float3::vec(&[1.; 3]));

    // on left
    let org = Float3::vec(&[-1. - e, 0., 0.]);
    let dir = Float3::vec(&[1., 0., 0.]);
    let ray = Ray::new(org, dir);

    let h = b.raycast(&ray);
    assert!(h.is_some());
    assert!((h.unwrap().t - e).abs() < e);

    // on x0
    let org = Float3::vec(&[-1., 0., 0.]);
    let dir = Float3::vec(&[1., 0., 0.]);
    let ray = Ray::new(org, dir);

    let h = b.raycast(&ray);
    assert!(h.is_some());
    assert!((h.unwrap().t - 2.) < e);

    // inside
    let org = Float3::vec(&[-1. + e, 0., 0.]);
    let ray = Ray::new(org, dir);
    let h = b.raycast(&ray);
    assert!(h.is_some());
    assert!((h.unwrap().t - 2. + e) < e);

    let org = Float3::vec(&[1. + e, 0., 0.]);
    let ray = Ray::new(org, dir);
    let h = b.raycast(&ray);
    assert!(h.is_none());
}
