use anyhow::Result;
use num_traits::Num;

const MAX_DIM: usize = 4;

#[derive(Clone, Copy, Debug)]
pub struct Tensor<T: Num + Copy, const N: usize> {
    pub raw: [T; N],
    pub shape: usize, // 1 byte 1 dim
}

impl<T: Num + Copy, const N: usize> Tensor<T, N> {
    pub fn new(arr: &[T], shape: &[usize]) -> Self {
        if shape.len() > MAX_DIM {
            panic!("Only {} dimension tensor supported.", MAX_DIM);
        }

        let count: usize = shape.iter().fold(1, |acc, x| {
            if *x > 0xFF {
                panic!("Dimension limit is 0-255, now:{}", x);
            }
            acc * x
        });

        if count > N {
            panic!("Elements count:{} must less than {}.", count, N);
        }

        let raw = std::array::from_fn(|i| arr[i]);
        let shape = Self::encode_shape(shape).unwrap();
        Tensor { raw, shape }
    }

    fn encode_shape(shape: &[usize]) -> Result<usize> {
        let mut code = 0;
        for (i, &s) in shape.iter().enumerate() {
            if s == 0 {
                break;
            }
            code |= (s & 0xFF) << (i * 8);
        }
        Ok(code)
    }
}

#[test]
fn test_shape() {
    use crate::tensor;
    let t = tensor!(1.;2,2);
    assert_eq!(t.shape, 256 * 2 + 2);
}
