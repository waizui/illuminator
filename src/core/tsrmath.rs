use num_traits::Float;

use crate::core::tensor::{Tensor, TensorNum};

pub trait TensorMath<T: TensorNum + Float, const N: usize> {
    fn exp(&self) -> Self;

    fn min(&self, other: Self) -> Self;

    fn max(&self, other: Self) -> Self;
}

impl<T, const N: usize> TensorMath<T, N> for Tensor<T, N>
where
    T: TensorNum + Float,
{
    fn exp(&self) -> Self {
        let raw: [T; N] = std::array::from_fn(|i| self.raw[i].exp());

        Tensor {
            raw,
            shape: self.shape,
        }
    }

    fn min(&self, other: Self) -> Self {
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

    fn max(&self, other: Self) -> Self {
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
