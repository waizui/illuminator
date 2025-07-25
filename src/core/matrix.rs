use crate::core::tensor::{Tensor, TensorNum};

pub trait Matrix<T, const N: usize, const NR: usize>
where
    T: TensorNum,
{
    fn matmul<const ON: usize>(&self, rhs: Tensor<T, NR>) -> Tensor<T, ON>;
    /// matmul for 1d vector
    fn matmulvec<const ON: usize>(&self, rhs: Tensor<T, NR>) -> Tensor<T, ON> {
        self.matmul(rhs.reshaped(&[NR, 1])).reshaped(&[ON])
    }
}

impl<T, const N: usize, const NR: usize> Matrix<T, N, NR> for Tensor<T, N>
where
    T: TensorNum,
{
    fn matmul<const NO: usize>(&self, rhs: Tensor<T, NR>) -> Tensor<T, NO> {
        let size_l = self.shape.size();
        let size_r = rhs.shape.size();
        debug_assert_eq!(size_l, 2);
        debug_assert_eq!(size_r, 2);

        let (row_l, col_l) = (self.shape.get(0), self.shape.get(1));
        let (row_r, col_r) = (rhs.shape.get(0), rhs.shape.get(1));
        debug_assert_eq!(col_l, row_r);

        let mut res = Tensor::new(&[row_l, col_r], [T::zero(); NO]);

        for i in 0..row_l {
            for j in 0..col_r {
                let mut acc = T::zero();
                for k in 0..col_l {
                    acc = acc + self[(i, k)] * rhs[(k, j)];
                }
                res[(i, j)] = acc;
            }
        }

        res
    }
}

#[test]
fn test_matrix() {
    use crate::prelude::{Mat3x3f, Vec3f};

    let m = Mat3x3f::mat([3, 3], [1., 0., 0., 0., 1., 0., 0., 0., 1.]);

    let rhs = Vec3f::vec([2.; 3]).reshaped(&[3, 1]);
    let re = m.matmul(rhs);
    assert_eq!(re, rhs);

    let v = Vec3f::vec([2.; 3]);
    let re = m.matmulvec(v);
    assert_eq!(re, v);
}
