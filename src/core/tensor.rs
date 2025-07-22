use num_traits::Num;

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
    pub fn oned(size: usize) -> Self {
        //hight 8 bits for dim 1st
        TensorShape {
            raw_shape: size << 24,
        }
    }

    pub fn twod(size: [usize; 2]) -> Self {
        //hight 8 bits for dim 1st
        TensorShape {
            raw_shape: (size[0] << 24) | (size[1] << 16),
        }
    }

    /// get size at dim
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

    pub fn size(&self) -> usize {
        for i in 0..MAX_DIM {
            if self.get(i) == 0 {
                return i;
            }
        }

        MAX_DIM
    }

    fn stride(&self, dim: usize) -> usize {
        let mut acc = 1;
        for i in dim..self.size() - 1 {
            acc *= self.get(i);
        }
        acc
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

/// simple stack-alloc tensor
/// sadlly 3 floats array will use ptr instead of registers: https://mcyoung.xyz/2024/04/17/calling-convention/
#[derive(Clone, Copy, Debug)]
pub struct Tensor<T: TensorNum, const N: usize> {
    pub raw: [T; N],
    pub shape: TensorShape, // 1 byte 1 dim
}

impl<T: TensorNum, const N: usize> Tensor<T, N> {
    pub fn new(arr: [T; N], shape: &[usize]) -> Self {
        let count: usize = shape.iter().fold(1, |acc, &x| {
            assert!(x < 0xFF, "Dimension limit is 0-255, now:{x}");
            acc * x
        });
        assert!(count <= N, "Elements count:{count} must less than {N}.");

        let shape = TensorShape::from(shape);
        Tensor { raw: arr, shape }
    }

    pub fn vec(arr: [T; N]) -> Self {
        Tensor {
            raw: arr,
            shape: TensorShape::oned(N),
        }
    }

    pub fn mat(arr: [T; N], shape: [usize; 2]) -> Self {
        Tensor {
            raw: arr,
            shape: TensorShape::twod(shape),
        }
    }

    pub fn dot<const RN: usize>(&self, rhs: Tensor<T, RN>) -> T {
        assert!(
            self.shape.size() == 1 || rhs.shape.size() == 1,
            "Dot Only for 1d tensors"
        );

        assert!(N == RN, "Dot operand's length not equal");
        let mut acc = T::zero();
        for i in 0..N {
            acc = acc + self.raw[i] * rhs.raw[i];
        }
        acc
    }

    pub fn min(&self, other: Self) -> Self {
        let raw: [T; N] = std::array::from_fn(|i| {
            if self.raw[i] < other.raw[i] {
                return self.raw[i];
            }

            other.raw[i]
        });

        Tensor {
            raw,
            shape: self.shape,
        }
    }

    pub fn max(&self, other: Self) -> Self {
        let raw: [T; N] = std::array::from_fn(|i| {
            if self.raw[i] < other.raw[i] {
                return other.raw[i];
            }

            self.raw[i]
        });

        Tensor {
            raw,
            shape: self.shape,
        }
    }
}

pub type Vec3f = Tensor<f32, 3>;
pub type Vec3x3f = Tensor<f32, 9>;

pub type Vec4f = Tensor<f32, 4>;
pub type Vec4x4f = Tensor<f32, 16>;

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
