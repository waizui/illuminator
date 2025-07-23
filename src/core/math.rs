use num_traits::clamp;

use crate::{core::vec::Vector, prelude::Vec3f};

pub const MACHINE_EPSILON32: f32 = f32::EPSILON * 0.5;

pub fn gamma(n: i32) -> f32 {
    (n as f32 * MACHINE_EPSILON32) / (1. - n as f32 * MACHINE_EPSILON32)
}

/// returned value no large than size-2
/// returens 0 if pred all false, size - 2 if all true
/// eg: for pred of elements =2: [2,2,3]->1 ,[2,2,2]-> 1  
pub fn split_index<F>(size: usize, pred: F) -> usize
where
    F: Fn(usize) -> bool,
{
    let (mut sz, mut first) = (size.saturating_sub(2), 1);

    while sz > 0 {
        let half = sz >> 1;
        let mid = first + half;
        let res = pred(mid);
        first = if res { mid + 1 } else { first };
        sz = if res {
            sz.saturating_sub(half + 1)
        } else {
            half
        };
    }

    clamp(first - 1, 0, size.saturating_sub(2))
}

/// y-up, z-forward, cartesian to spherical coordinates
pub fn xyz2spherical(xyz: Vec3f) -> Vec3f {
    let r = xyz.magnitude();
    assert!(r > 0f32);
    let u = xyz.normalize();
    let theta = u[1].acos();
    let phi = u[0].atan2(u[2]);
    Vec3f::vec([r, theta, phi])
}


#[test]
fn test_split() {
    let arr = [2, 2, 3];
    let i = split_index(arr.len(), |i| arr[i] == 2);
    assert_eq!(i, 1);

    let arr = [2, 2, 2];
    let i = split_index(arr.len(), |i| arr[i] == 2);
    assert_eq!(i, 1);

    let arr = [2, 2, 3];
    let i = split_index(arr.len(), |i| arr[i] == 1);
    assert_eq!(i, 0);
}
