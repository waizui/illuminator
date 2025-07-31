use num_traits::{Num, Zero};

pub const MAX_DIM: usize = 4;

pub trait TensorNum: Num + Copy + PartialOrd {}
impl TensorNum for f32 {}
impl TensorNum for i32 {}
impl TensorNum for usize {}

#[derive(Clone, Copy, Debug)]
pub struct TensorShape {
    pub raw_shape: usize,
}

impl TensorShape {
    /// get size at dim
    pub fn get(&self, dim: usize) -> usize {
        let shift = (MAX_DIM - 1 - dim) * 8;
        (self.raw_shape & (0xFF << shift)) >> shift
    }

    /// to flatten index
    pub fn to_index(&self, index: &[usize]) -> usize {
        index
            .iter()
            .enumerate()
            .fold(0, |acc, (dim, &i)| acc + i * self.stride(dim))
    }

    /// return how many dimensions it has
    pub fn size(&self) -> usize {
        for i in 0..MAX_DIM {
            if self.get(i) == 0 {
                return i;
            }
        }

        MAX_DIM
    }

    fn stride(&self, dim: usize) -> usize {
        (dim + 1..self.size()).fold(1, |acc, i| acc * self.get(i))
    }
}

impl From<usize> for TensorShape {
    fn from(value: usize) -> Self {
        TensorShape { raw_shape: value }
    }
}

impl From<&[usize]> for TensorShape {
    fn from(value: &[usize]) -> Self {
        assert!(
            value.len() <= MAX_DIM,
            "Only {MAX_DIM} dimension tensor supported."
        );

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

/// small stack-alloc tensor
// sadlly 3 floats array will use ptr instead of registers: https://mcyoung.xyz/2024/04/17/calling-convention/
#[derive(Clone, Copy, Debug)]
pub struct Tensor<T: TensorNum, const N: usize> {
    pub raw: [T; N],
    pub shape: TensorShape, // 1 byte 1 dim
}

impl<T: TensorNum, const N: usize> Tensor<T, N> {
    pub fn new(shape: &[usize], arr: [T; N]) -> Self {
        #[cfg(debug_assertions)]
        {
            let count: usize = shape.iter().product();
            debug_assert!(count <= N, "Elements count:{count} must less than {N}.");
        }

        let shape = TensorShape::from(shape);
        Self { raw: arr, shape }
    }

    pub fn vec(arr: [T; N]) -> Self {
        Self::new(&[N], arr)
    }

    /// row major matrix
    pub fn mat(shape: [usize; 2], arr: [T; N]) -> Self {
        Self::new(&shape, arr)
    }

    pub fn reshape(&mut self, shape: &[usize]) {
        //TODO:check shape
        self.shape = TensorShape::from(shape);
    }

    pub fn reshaped(&self, shape: &[usize]) -> Self {
        Self {
            raw: self.raw,
            shape: TensorShape::from(shape),
        }
    }
}

impl<T, const N: usize> Default for Tensor<T, N>
where
    T: TensorNum,
{
    fn default() -> Self {
        let t = T::zero();
        Self::vec([t; N])
    }
}

impl<T, const N: usize> Zero for Tensor<T, N>
where
    T: TensorNum,
{
    fn zero() -> Self {
        Self::default()
    }

    fn is_zero(&self) -> bool {
        self.raw.iter().all(|x| x.is_zero())
    }

    fn set_zero(&mut self) {
        self.raw.iter_mut().for_each(|x| *x = T::zero());
    }
}

pub type Vec3f = Tensor<f32, 3>;
pub type Mat3x3f = Tensor<f32, 9>;

pub type Vec4f = Tensor<f32, 4>;
pub type Mat4x4f = Tensor<f32, 16>;

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
