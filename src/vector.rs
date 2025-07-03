use num_traits::Num;

#[derive(Clone, Copy)]
pub struct Vector<T: Num + Copy, const N: usize, const D: usize> {
    pub raw: [T; N],
    pub shape: [usize; D],
}

impl<T: Num + Copy, const N: usize, const D: usize> Vector<T, N, D> {
    pub fn new(arr: &[T], shape: &[usize]) -> Self {
        let raw = std::array::from_fn(|i| arr[i]);
        let shape = std::array::from_fn(|i| shape[i]);
        Vector { raw, shape }
    }
}
