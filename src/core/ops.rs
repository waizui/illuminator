use crate::core::tensor::Tensor;
use num_traits::Num;
use std::ops::{Add, Div, Mul, Sub};

impl<T: Num + Copy, const N: usize> Sub for Tensor<T, N> {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        let raw: [T; N] = std::array::from_fn(|i| self.raw[i] - rhs.raw[i]);
        Tensor {
            raw,
            shape: self.shape,
        }
    }
}

impl<T: Num + Copy, const N: usize> Sub<T> for Tensor<T, N> {
    type Output = Self;
    fn sub(self, rhs: T) -> Self::Output {
        let raw: [T; N] = std::array::from_fn(|i| self.raw[i] - rhs);
        Tensor {
            raw,
            shape: self.shape,
        }
    }
}

impl<T: Num + Copy, const N: usize> Add for Tensor<T, N> {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        let raw: [T; N] = std::array::from_fn(|i| self.raw[i] + rhs.raw[i]);
        Tensor {
            raw,
            shape: self.shape,
        }
    }
}

impl<T: Num + Copy, const N: usize> Add<T> for Tensor<T, N> {
    type Output = Self;
    fn add(self, rhs: T) -> Self::Output {
        let raw: [T; N] = std::array::from_fn(|i| self.raw[i] + rhs);
        Tensor {
            raw,
            shape: self.shape,
        }
    }
}

impl<T: Num + Copy, const N: usize> Mul for Tensor<T, N> {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self::Output {
        let raw: [T; N] = std::array::from_fn(|i| self.raw[i] * rhs.raw[i]);
        Tensor {
            raw,
            shape: self.shape,
        }
    }
}

impl<T: Num + Copy, const N: usize> Mul<T> for Tensor<T, N> {
    type Output = Self;
    fn mul(self, rhs: T) -> Self::Output {
        let raw: [T; N] = std::array::from_fn(|i| self.raw[i] * rhs);
        Tensor {
            raw,
            shape: self.shape,
        }
    }
}

impl<T, const N: usize> Div for Tensor<T, N>
where
    T: Num + Copy,
{
    type Output = Self;
    fn div(self, rhs: Self) -> Self::Output {
        let raw: [T; N] = std::array::from_fn(|i| self.raw[i] / rhs.raw[i]);
        Tensor {
            raw,
            shape: self.shape,
        }
    }
}

impl<T: Num + Copy, const N: usize> Div<T> for Tensor<T, N> {
    type Output = Self;
    fn div(self, rhs: T) -> Self::Output {
        let raw: [T; N] = std::array::from_fn(|i| self.raw[i] / rhs);
        Tensor {
            raw,
            shape: self.shape,
        }
    }
}

#[test]
fn test_add() {
    use crate::tensor;
    let fv = tensor!(1.;3);
    let fv2 = fv + fv;
    assert_eq!(fv2.raw, [2.; 3]);

    let fv3 = fv + 1.;
    assert_eq!(fv3.raw, [2.; 3]);

    let iv = tensor!(1; 3);
    let iv2 = iv + iv;
    assert_eq!(iv2.raw, [2; 3]);

    let iv3 = iv + 1;

    assert_eq!(iv3.raw, [2; 3]);
}

#[test]
fn test_sub() {
    use crate::tensor;
    let fv = tensor!(1.; 3);
    let fv2 = fv - fv;
    assert_eq!(fv2.raw, [0.; 3]);

    let fv3 = fv - 1.;
    assert_eq!(fv3.raw, [0.; 3]);

    let iv = tensor!(1; 3);
    let iv2 = iv - iv;
    assert_eq!(iv2.raw, [0; 3]);

    let iv3 = iv - 1;
    assert_eq!(iv3.raw, [0; 3]);
}

#[test]
fn test_mul() {
    use crate::tensor;
    let fv = tensor!(1.; 3);
    let fv2 = fv * fv;
    assert_eq!(fv2.raw, [1.; 3]);

    let fv3 = fv * 1.;
    assert_eq!(fv3.raw, [1.; 3]);

    let iv = tensor!(1; 3);
    let iv2 = iv * iv;
    assert_eq!(iv2.raw, [1; 3]);

    let iv3 = iv * 1;
    assert_eq!(iv3.raw, [1; 3]);
}

#[test]
fn test_div() {
    use crate::tensor;
    let fv = tensor!(2.; 3);
    let fv2 = fv / fv;
    assert_eq!(fv2.raw, [1.; 3]);

    let fv3 = fv / 2.;
    assert_eq!(fv3.raw, [1.; 3]);

    let iv = tensor!(2; 3);
    let iv2 = iv / iv;
    assert_eq!(iv2.raw, [1; 3]);

    let iv3 = iv / 2;
    assert_eq!(iv3.raw, [1; 3]);
}
