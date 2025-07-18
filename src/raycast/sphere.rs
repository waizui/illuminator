use std::fmt::Debug;

use crate::{
    core::tensor::Float3,
    raycast::{bounds::Bounds3f, primitive::Primitive, *},
};

#[derive(Clone)]
pub struct Sphere {
    pub cnt: Float3,
    pub r: f32,
}

impl Sphere {
    pub fn new(cnt: Float3, r: f32) -> Sphere {
        Sphere { cnt, r }
    }

    pub fn intersect(&self, ray_src: Float3, ray_dir: Float3) -> Option<f32> {
        // Solve t^2*d.d + 2*t*(o-p).d + (o-p).(o-p)-R^2 = 0
        let op = ray_src - self.cnt;
        let a = ray_dir.dot(ray_dir);
        let b = op.dot(ray_dir);
        let c = op.dot(op) - self.r * self.r;
        let det = b * b - c * a;
        if det < 0. {
            None
        } else {
            let det = det.sqrt();
            if -b - det >= 0. {
                return Some((-b - det) / a);
            } else if -b + det >= 0. {
                return Some((-b + det) / a);
            }
            None
        }
    }
}

impl Raycast for Sphere {
    fn raycast(&self, ray: &Ray) -> Option<Hit> {
        if let Some(t) = self.intersect(ray.org, ray.dir) {
            return Some(Hit {
                ray: ray.clone(),
                t,
            });
        }
        None
    }
}

impl Debug for Sphere {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", self.cnt, self.r)
    }
}

impl Primitive for Sphere {
    fn bounds(&self) -> bounds::Bounds3f {
        let r = Float3::vec(&[self.r; 3]);
        let min = self.cnt - r;
        let max = self.cnt + r;
        Bounds3f { min, max }
    }

    fn clone_as_box(&self) -> Box<dyn Primitive> {
        Box::new(self.clone())
    }
}

#[test]
fn test_sphere() {
    let s = Sphere::new(Float3::vec(&[0.; 3]), 1.);

    for i in 0..11 {
        let y = i as f32 / 10.;
        let x = (1. - y * y).sqrt();
        let z = 0.;

        let org = Float3::vec(&[x, y, z]) * 2.;
        let dir = org * -1.;
        let ray = Ray::new(org, dir);

        let hit = s.raycast(&ray).unwrap();
        assert_eq!(hit.position()[0], x);
        assert_eq!(hit.position()[1], y);
    }

    let b = s.bounds();
    assert_eq!(b.min[0], -1.);
    assert_eq!(b.min[1], -1.);
    assert_eq!(b.min[2], -1.);
    assert_eq!(b.max[0], 1.);
    assert_eq!(b.max[1], 1.);
    assert_eq!(b.max[2], 1.);
}
