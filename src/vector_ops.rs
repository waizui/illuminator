use crate::vector::Vector;
use num_traits::Num;
use std::ops::{Add, Div, Mul, Sub};

impl<T: Num + Copy, const N: usize, const D: usize> Sub for Vector<T, N, D> {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        let re: [T; N] = std::array::from_fn(|i| self.raw[i] - rhs.raw[i]);
        Vector::new(&re, &self.shape)
    }
}

impl<T: Num + Copy, const N: usize, const D: usize> Sub<T> for Vector<T, N, D> {
    type Output = Self;
    fn sub(self, rhs: T) -> Self::Output {
        let re: [T; N] = std::array::from_fn(|i| self.raw[i] - rhs);
        Vector::new(&re, &self.shape)
    }
}

impl<T: Num + Copy, const N: usize, const D: usize> Add for Vector<T, N, D> {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        let re: [T; N] = std::array::from_fn(|i| self.raw[i] + rhs.raw[i]);
        Vector::new(&re, &self.shape)
    }
}

impl<T: Num + Copy, const N: usize, const D: usize> Add<T> for Vector<T, N, D> {
    type Output = Self;
    fn add(self, rhs: T) -> Self::Output {
        let re: [T; N] = std::array::from_fn(|i| self.raw[i] + rhs);
        Vector::new(&re, &self.shape)
    }
}

impl<T: Num + Copy, const N: usize, const D: usize> Mul for Vector<T, N, D> {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self::Output {
        let re: [T; N] = std::array::from_fn(|i| self.raw[i] * rhs.raw[i]);
        Vector::new(&re, &self.shape)
    }
}

impl<T: Num + Copy, const N: usize, const D: usize> Mul<T> for Vector<T, N, D> {
    type Output = Self;
    fn mul(self, rhs: T) -> Self::Output {
        let re: [T; N] = std::array::from_fn(|i| self.raw[i] * rhs);
        Vector::new(&re, &self.shape)
    }
}

impl<T: Num + Copy, const N: usize, const D: usize> Div for Vector<T, N, D> {
    type Output = Self;
    fn div(self, rhs: Self) -> Self::Output {
        let re: [T; N] = std::array::from_fn(|i| self.raw[i] / rhs.raw[i]);
        Vector::new(&re, &self.shape)
    }
}

impl<T: Num + Copy, const N: usize, const D: usize> Div<T> for Vector<T, N, D> {
    type Output = Self;
    fn div(self, rhs: T) -> Self::Output {
        let re: [T; N] = std::array::from_fn(|i| self.raw[i] / rhs);
        Vector::new(&re, &self.shape)
    }
}

#[test]
fn test_add() {
    let fv: Vector<f32, 3, 1> = Vector::new(&[1.; 3], &[3]);
    let fv2 = fv + fv;
    assert_eq!(fv2.raw, [2.; 3]);

    let fv3 = fv + 1.;
    assert_eq!(fv3.raw, [2.; 3]);

    let iv: Vector<i32, 3, 1> = Vector::new(&[1; 3], &[3]);
    let iv2 = iv + iv;
    assert_eq!(iv2.raw, [2; 3]);

    let iv3 = iv + 1;
    assert_eq!(iv3.raw, [2; 3]);
}

#[test]
fn test_sub() {
    let fv: Vector<f32, 3, 1> = Vector::new(&[1.; 3], &[3]);
    let fv2 = fv - fv;
    assert_eq!(fv2.raw, [0.; 3]);

    let fv3 = fv - 1.;
    assert_eq!(fv3.raw, [0.; 3]);

    let iv: Vector<i32, 3, 1> = Vector::new(&[1; 3], &[3]);
    let iv2 = iv - iv;
    assert_eq!(iv2.raw, [0; 3]);

    let iv3 = iv - 1;
    assert_eq!(iv3.raw, [0; 3]);
}

#[test]
fn test_mul() {
    let fv: Vector<f32, 3, 1> = Vector::new(&[1.; 3], &[3]);
    let fv2 = fv * fv;
    assert_eq!(fv2.raw, [1.; 3]);

    let fv3 = fv * 1.;
    assert_eq!(fv3.raw, [1.; 3]);

    let iv: Vector<i32, 3, 1> = Vector::new(&[1; 3], &[3]);
    let iv2 = iv * iv;
    assert_eq!(iv2.raw, [1; 3]);

    let iv3 = iv * 1;
    assert_eq!(iv3.raw, [1; 3]);
}

#[test]
fn test_div() {
    let fv: Vector<f32, 3, 1> = Vector::new(&[2.; 3], &[3]);
    let fv2 = fv / fv;
    assert_eq!(fv2.raw, [1.; 3]);

    let fv3 = fv / 2.;
    assert_eq!(fv3.raw, [1.; 3]);

    let iv: Vector<i32, 3, 1> = Vector::new(&[2; 3], &[3]);
    let iv2 = iv / iv;
    assert_eq!(iv2.raw, [1; 3]);

    let iv3 = iv / 2;
    assert_eq!(iv3.raw, [1; 3]);
}
