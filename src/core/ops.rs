use crate::core::tensor::{Tensor, TensorNum, TensorShape};
use std::{
    fmt::Display,
    ops::{Add, Div, Index, Mul, Sub},
};

impl<T: TensorNum, const N: usize> Sub for Tensor<T, N> {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        let raw: [T; N] = std::array::from_fn(|i| self.raw[i] - rhs.raw[i]);
        Tensor {
            raw,
            shape: self.shape,
        }
    }
}

impl<T: TensorNum, const N: usize> Sub<T> for Tensor<T, N> {
    type Output = Self;
    fn sub(self, rhs: T) -> Self::Output {
        let raw: [T; N] = std::array::from_fn(|i| self.raw[i] - rhs);
        Tensor {
            raw,
            shape: self.shape,
        }
    }
}

impl<T: TensorNum, const N: usize> Add for Tensor<T, N> {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        let raw: [T; N] = std::array::from_fn(|i| self.raw[i] + rhs.raw[i]);
        Tensor {
            raw,
            shape: self.shape,
        }
    }
}

impl<T: TensorNum, const N: usize> Add<T> for Tensor<T, N> {
    type Output = Self;
    fn add(self, rhs: T) -> Self::Output {
        let raw: [T; N] = std::array::from_fn(|i| self.raw[i] + rhs);
        Tensor {
            raw,
            shape: self.shape,
        }
    }
}

impl<T: TensorNum, const N: usize> Mul for Tensor<T, N> {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self::Output {
        let raw: [T; N] = std::array::from_fn(|i| self.raw[i] * rhs.raw[i]);
        Tensor {
            raw,
            shape: self.shape,
        }
    }
}

impl<T: TensorNum, const N: usize> Mul<T> for Tensor<T, N> {
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
    T: TensorNum,
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

impl<T: TensorNum, const N: usize> Div<T> for Tensor<T, N> {
    type Output = Self;
    fn div(self, rhs: T) -> Self::Output {
        let raw: [T; N] = std::array::from_fn(|i| self.raw[i] / rhs);
        Tensor {
            raw,
            shape: self.shape,
        }
    }
}

impl PartialEq<TensorShape> for TensorShape {
    fn eq(&self, other: &TensorShape) -> bool {
        self.raw_shape == other.raw_shape
    }
}

impl<T: TensorNum, const N: usize> PartialEq<Tensor<T, N>> for Tensor<T, N> {
    fn eq(&self, other: &Tensor<T, N>) -> bool {
        self.raw == other.raw && self.shape == other.shape
    }
}

impl<T, const N: usize> Index<usize> for Tensor<T, N>
where
    T: TensorNum,
{
    type Output = T;
    fn index(&self, index: usize) -> &Self::Output {
        let real_i = self.shape.to_index(&[index]);
        &self.raw[real_i]
    }
}

impl<T, const N: usize> Index<(usize, usize)> for Tensor<T, N>
where
    T: TensorNum,
{
    type Output = T;
    fn index(&self, index: (usize, usize)) -> &Self::Output {
        let real_i = self.shape.to_index(&[index.0, index.1]);
        &self.raw[real_i]
    }
}

impl<T, const N: usize> Index<(usize, usize, usize)> for Tensor<T, N>
where
    T: TensorNum,
{
    type Output = T;
    fn index(&self, index: (usize, usize, usize)) -> &Self::Output {
        let real_i = self.shape.to_index(&[index.0, index.1, index.2]);
        &self.raw[real_i]
    }
}

impl<T, const N: usize> Index<(usize, usize, usize, usize)> for Tensor<T, N>
where
    T: TensorNum,
{
    type Output = T;
    fn index(&self, index: (usize, usize, usize, usize)) -> &Self::Output {
        let real_i = self.shape.to_index(&[index.0, index.1, index.2, index.3]);
        &self.raw[real_i]
    }
}


impl<T, const N: usize> Display for Tensor<T, N>
where
    T: TensorNum,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "todo:impl tensor fmt")
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

#[test]
fn test_index() {
    use crate::tensor;
    use std::iter::zip;
    let t = tensor!(1.;1,2,3,4);

    assert_eq!(t[(0, 0, 0, 0)], 1.);

    assert_eq!(t[(0, 0, 0, 3)], 1.);

    let t = tensor!([1,2,3,4,5,6,7,8,9];3,3);
    for (i, j) in zip(0..2usize, 0..2usize) {
        assert_eq!(t[(i, j)], i * 3 + j + 1);
    }
}
