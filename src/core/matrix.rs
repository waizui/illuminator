use crate::core::tensor::{Tensor, TensorNum};

pub trait Matrix<T, const N: usize, const NR: usize>
where
    T: TensorNum,
{
    fn matmul<const ON: usize>(&self, rhs: Tensor<T, NR>) -> Tensor<T, ON>;
}

impl<T, const N: usize, const NR: usize> Matrix<T, N, NR> for Tensor<T, N>
where
    T: TensorNum,
{
    fn matmul<const NO: usize>(&self, rhs: Tensor<T, NR>) -> Tensor<T, NO> {
        let size_l = self.shape.size();
        debug_assert_eq!(size_l, 2);

        let size_r = rhs.shape.size();
        debug_assert!(size_l < 3 && size_r < 3);
        let rhs = if size_r == 1 {
            rhs.reshaped(&[1, rhs.shape.get(0)])
        } else {
            rhs
        };

        let (row_l, col_l) = (self.shape.get(0), self.shape.get(1));
        let (row_r, col_r) = (rhs.shape.get(0), rhs.shape.get(1));
        debug_assert_eq!(col_l, row_r);

        let mut res = Tensor::new([T::zero(); NO], &[row_l, col_r]);

        for i in 0..row_l {
            for j in 0..col_r {
                let mut acc = T::zero();
                for k in 0..col_r {
                    acc = self[(i, k)] + rhs[(k, j)];
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
    // TODO: unify tensor new
    let m = Mat3x3f::mat([1., 0., 0., 0., 1., 0., 0., 0., 1.], [3, 3]);
    let v = Vec3f::vec([2.; 3]);
    let mv = m.matmul(v);
    assert_eq!(mv, v)
}
