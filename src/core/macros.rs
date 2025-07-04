#[macro_export]
macro_rules! tensor {
    ($val:expr ; $($dim:expr),+) => {{
        const N: usize = $crate::tensor!(@multiply $($dim),+);
        let arr = [$val; N];
        let shape = [$($dim),+];
        Tensor::<_, N>::new(&arr, &shape)
    }};

    (@multiply $dim:expr) => {
        $dim
    };

    (@multiply $dim:expr, $($rest:expr),+) => {
        $dim * $crate::tensor!(@multiply $($rest),+)
    };
}
