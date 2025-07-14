use crate::{core::tensor::Float3, raycast::*};

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
        self.min * 0.5 + self.max * 0.5
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
        todo!()
    }
}

impl Raycast for Bounds3f {
    fn raycast(&self, ray: Ray) -> Option<Hit> {
        todo!()
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
