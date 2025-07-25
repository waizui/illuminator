use crate::{
    core::{math::factorial, vec::Vector},
    prelude::Vec3f,
};
use num_traits::{NumOps, Zero};
use std::{
    f32::consts::PI,
    ops::{Add, Mul},
};

/// y-up, z-forward, cartesian to spherical coordinates [r, theta,phi]
pub fn xyz2spherical(xyz: Vec3f) -> Vec3f {
    let r = xyz.norm();
    assert!(r > 0f32);
    let u = xyz.normalize();
    let theta = u[1].acos();
    let phi = u[0].atan2(u[2]);
    Vec3f::vec([r, theta, phi])
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
    // https://waizui.github.io/posts/spherical_harmonics/spherical_harmonics.html
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

#[derive(Clone, Debug)]
pub struct SHSample {
    // sampling direction
    pub xyz: Vec3f,
    // sh coefficients
    pub coeff: Vec<f32>,
}

/// nsamples: specify how many samples will be generated
/// degrees of sh eg: 0 for fist degree
/// return random sh samples  across sphere surface
pub fn sh_samples(nsamples: usize, l: i32) -> Vec<SHSample> {
    use crate::core::sampling::{radical_inverse, square2unitsphere};
    use rayon::prelude::*;
    assert!(l >= 0);

    let mut samples = vec![
        SHSample {
            xyz: Vec3f::vec([0.; 3]),
            coeff: Vec::new()
        };
        nsamples
    ];

    let task = |isample: usize, sample: &mut SHSample| {
        // quasi-random samples
        let rx: f32 = radical_inverse(isample, 2);
        let ry: f32 = radical_inverse(isample, 3);
        let xyz = square2unitsphere([rx, ry]);
        let spherial = xyz2spherical(Vec3f::vec(xyz));
        let theta = spherial[1];
        let phi = spherial[2];

        sample.xyz = Vec3f::vec(xyz);

        for il in 0..l + 1 {
            for im in -il..il + 1 {
                let sh = sh_eval(il, im, theta, phi);
                sample.coeff.push(sh);
            }
        }
    };

    samples
        .par_iter_mut()
        .enumerate()
        .for_each(|(isample, sample)| task(isample, sample));

    samples
}

/// project spherical function f to sh basis
pub fn sh_project_fn<F, T>(l: i32, nsamples: usize, f: F) -> Vec<T>
where
    T: Mul<f32, Output = T> + NumOps + Zero + Send + Sync + Clone,
    F: Fn(Vec3f) -> T + Sync,
{
    use rayon::prelude::*;
    let sh_samples = sh_samples(nsamples, l);
    let zero = T::zero();
    let mut coeffs = vec![zero.clone(); ((l + 1) * (l + 1)) as usize];

    // calculate coefficient ci
    let ci_task = |ic: usize, coeff: &mut T| {
        let mut acc = zero.clone();
        for sh_sample in sh_samples.iter().take(nsamples) {
            let coeff = sh_sample.coeff[ic];
            let val = f(sh_sample.xyz);
            acc = acc + val * coeff;
        }

        // Monte Carlo method, need to divide sample count
        // and probability density function(pdf), which is 1/(4*pi) of sampling a sphere
        acc = acc * (4f32 * PI / nsamples as f32);
        *coeff = acc;
    };

    coeffs
        .par_iter_mut()
        .enumerate()
        .for_each(|(ic, coeff)| ci_task(ic, coeff));

    coeffs
}

/// project  spherical function f to sh basis
pub fn sh_project_one<T>(val: T, l: i32, dir: Vec3f) -> T
where
    T: Mul<f32, Output = T> + NumOps + Zero + Clone,
{
    let sph = xyz2spherical(dir);
    let theta = sph[1];
    let phi = sph[2];

    let mut res = T::zero();

    for il in 0..l + 1 {
        for im in -il..il + 1 {
            let sh = sh_eval(il, im, theta, phi);
            res = res + val.clone() * sh;
        }
    }

    res
}

/// return a reconstructed spherical functon f
pub fn sh_reconstruct_fn<T>(coeffs: &[T], l: i32) -> impl Fn(Vec3f) -> T
where
    T: Add<f32, Output = T> + Mul<f32, Output = T> + NumOps + Zero + Clone,
{
    move |dir: Vec3f| sh_reconstruct_one(coeffs, l, dir)
}

/// reconstruc one value
pub fn sh_reconstruct_one<T>(coeffs: &[T], l: i32, dir: Vec3f) -> T
where
    T: Add<T, Output = T> + Mul<f32, Output = T> + Zero + Clone,
{
    let sph = xyz2spherical(dir);
    let theta = sph[1];
    let phi = sph[2];

    let mut res = T::zero();

    for il in 0..l + 1 {
        for im in -il..il + 1 {
            let sh = sh_eval(il, im, theta, phi);
            let ic = (il * (il + 1) + im) as usize;
            let coeff = coeffs[ic].clone();
            // sum all products of projected coefficient multipled by respective SH basis
            res = res + coeff * sh;
        }
    }

    res
}
