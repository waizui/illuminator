use num_traits::Num;
use std::{iter::zip, ops::Index};

const MAX_DIM: usize = 4;

#[derive(Clone, Copy, Debug)]
pub struct TensorShape {
    raw_shape: usize,
}

impl From<usize> for TensorShape {
    fn from(value: usize) -> Self {
        TensorShape { raw_shape: value }
    }
}

impl From<&[usize]> for TensorShape {
    fn from(value: &[usize]) -> Self {
        if value.len() > MAX_DIM {
            panic!("Only {MAX_DIM} dimension tensor supported.");
        }

        let mut raw = 0;
        for (i, &s) in value.iter().enumerate() {
            if s == 0 {
                break;
            }
            raw |= (s & 0xFF) << ((MAX_DIM - 1 - i) * 8);
        }
        TensorShape { raw_shape: raw }
    }
}

impl TensorShape {
    pub fn get(&self, dim: usize) -> usize {
        let shift = (MAX_DIM - 1 - dim) * 8;
        (self.raw_shape & (0xFF << shift)) >> shift
    }

    /// to flatten index
    pub fn to_index(&self, index: &[usize]) -> usize {
        let mut real_i = 0usize;
        for (dim, &i) in index.iter().enumerate() {
            real_i += i * self.stride(dim);
        }
        real_i
    }

    fn stride(&self, dim: usize) -> usize {
        let mut acc = 1;
        for i in dim..MAX_DIM - 1 {
            let next = self.get(i + 1);
            if next == 0 {
                break;
            }

            acc *= self.get(i);
        }
        acc
    }
}

/// simple stack-alloc tensor
#[derive(Clone, Copy, Debug)]
pub struct Tensor<T: Num + Copy, const N: usize> {
    pub raw: [T; N],
    pub shape: TensorShape, // 1 byte 1 dim
}

impl<T: Num + Copy, const N: usize> Tensor<T, N> {
    pub fn new(arr: &[T], shape: &[usize]) -> Self {
        let count: usize = shape.iter().fold(1, |acc, x| {
            if *x > 0xFF {
                panic!("Dimension limit is 0-255, now:{x}");
            }
            acc * x
        });

        if count > N {
            panic!("Elements count:{count} must less than {N}.");
        }

        let raw = std::array::from_fn(|i| arr[i]);
        let shape = TensorShape::from(shape);
        Tensor { raw, shape }
    }
}

impl<T, const N: usize> Index<&[usize]> for Tensor<T, N>
where
    T: Num + Copy,
{
    type Output = T;
    fn index(&self, index: &[usize]) -> &Self::Output {
        if index.len() > MAX_DIM {
            panic!("Only {MAX_DIM} dimension tensor supported.");
        }

        let real_i = self.shape.to_index(index);
        &self.raw[real_i]
    }
}

#[test]
fn test_shape() {
    use crate::tensor;
    let t = tensor!(1.;1,2,3,4);

    for i in 0..4 {
        assert_eq!(t.shape.get(i), i + 1);
    }

    let t = tensor!([1,2,3,4];2,1,2);
    assert_eq!(t.shape.get(0), 2);
    assert_eq!(t.shape.get(1), 1);
    assert_eq!(t.shape.get(2), 2);

    let t = tensor!([1,2,3,4,5,6,7,8,9];3,3);
    assert_eq!(t.shape.get(0), 3);
    assert_eq!(t.shape.get(1), 3);
}

#[test]
fn test_index() {
    use crate::tensor;
    let t = tensor!(1.;1,2,3,4);

    let i = &[0; 4];
    assert_eq!(t[i], 1.);

    let i = &[0, 0, 0, 3];
    assert_eq!(t[i], 1.);

    let t = tensor!([1,2,3,4,5,6,7,8,9];3,3);
    for (i, j) in zip(0..2usize, 0..2usize) {
        assert_eq!(t[&[i, j]], i * 3 + j + 1);
    }
}
