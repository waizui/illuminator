use crate::{core::vec::Vector, prelude::Vec3f};
use num_traits::clamp;
use std::f32::consts::PI;

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

pub fn factorial(x: i32) -> f32 {
    if x == 0 {
        return 1f32;
    }
    (1..x + 1).fold(1., |acc, x| acc * x as f32)
}

/// evaluate Associated Legendre Polynomial P(l,m) at x
pub fn sh_legendre(l: i32, m: i32, x: f32) -> f32 {
    assert!(m >= 0);
    let mut pmm = 1f32;
    // evaluate  P(m,m) from P(0,0)
    if m > 0 {
        let sqrtfactor = ((1f32 - x) * (1f32 + x)).sqrt();
        let mut fact = 1f32;
        for _ in 0..m {
            pmm *= (-fact) * sqrtfactor;
            fact += 2f32;
        }
    }
    if l == m {
        return pmm;
    }

    let mut pmm1 = x * (2f32 * m as f32 + 1f32) * pmm;
    if l == m + 1 {
        return pmm1;
    }

    let mut pll = 0f32;
    for ll in m + 2..l + 1 {
        pll = (x * (2 * ll - 1) as f32 * pmm1 - (ll + m - 1) as f32 * pmm) / (ll - m) as f32;
        pmm = pmm1;
        pmm1 = pll;
    }

    pll
}

/// renormalisation constant for SH function
pub fn sh_k(l: i32, m: i32) -> f32 {
    let fac0 = (2f32 * l as f32 + 1f32) / (4f32 * PI);
    let fac1 = factorial(l - m) / factorial(l + m);
    let res = fac0 * fac1;
    res.sqrt()
}

/// l [0,N], m [-l,l]
/// evaluate real part of spherical harmonics
pub fn sh_eval(l: i32, m: i32, theta: f32, phi: f32) -> f32 {
    if m == 0 {
        return sh_k(l, m) * sh_legendre(l, m, theta.cos());
    }

    let sqrt2 = 2f32.sqrt();

    if m > 0 {
        return sqrt2 * sh_k(l, m) * (m as f32 * phi).cos() * sh_legendre(l, m, theta.cos());
    }

    let m = -m;
    sqrt2 * sh_k(l, m) * (m as f32 * phi).sin() * sh_legendre(l, m, theta.cos())
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
