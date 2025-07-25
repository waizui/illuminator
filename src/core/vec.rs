use num_traits::Float;
use crate::core::tensor::{Tensor, TensorNum};

pub trait Vector<T: TensorNum, const N: usize> {
    fn sqrnorm(&self) -> T;

    fn norm(&self) -> T;

    fn normalize(&self) -> Tensor<T, N>;

    fn dot(&self, rhs: Tensor<T, N>) -> T;

    fn cross(&self, rhs: Tensor<T, N>) -> Self;
}

impl<T: TensorNum + Float, const N: usize> Vector<T, N> for Tensor<T, N> {
    fn norm(&self) -> T {
        self.sqrnorm().sqrt()
    }

    fn sqrnorm(&self) -> T {
        (0..N).fold(T::zero(), |acc, i| self[i] * self[i] + acc)
    }

    fn normalize(&self) -> Tensor<T, N> {
        *self * (T::one() / self.norm())
    }

    fn dot(&self, rhs: Tensor<T, N>) -> T {
        assert!(
            self.shape.size() == 1 && rhs.shape.size() == 1,
            "Dot Only for 1d tensors"
        );

        (0..N).fold(T::zero(), |acc, i| acc + self.raw[i] * rhs.raw[i])
    }

    fn cross(&self, rhs: Tensor<T, N>) -> Self {
        assert!(
            self.shape.size() == 1 && rhs.shape.size() == 1 && N > 2,
            "Cross Only for 1d tensors"
        );

        let u0 = self[0];
        let u1 = self[1];
        let u2 = self[2];
        let v0 = rhs[0];
        let v1 = rhs[1];
        let v2 = rhs[2];
        let mut arr = [T::zero(); N];
        arr[0] = u1 * v2 - v1 * u2;
        arr[1] = u2 * v0 - v2 * u0;
        arr[2] = u0 * v1 - v0 * u1;
        Self::vec(arr)
    }
}
