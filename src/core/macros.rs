#[macro_export]
macro_rules! tensor {

    ([$($val:expr),*] ; $($dim:expr),+) => {{
        const N: usize = $crate::tensor!(@size $($dim),+);
        let arr = [$($val),*];
        let shape = [$($dim),+];
        Tensor::<_, N>::new(arr, &shape)
    }};

    ($val:expr ; $($dim:expr),+) => {{
        const N: usize = $crate::tensor!(@size $($dim),+);
        let arr = [$val; N];
        let shape = [$($dim),+];
        Tensor::<_, N>::new(arr, &shape)
    }};

    (@size $dim:expr) => {
        $dim
    };

    (@size $dim:expr, $($rest:expr),+) => {
        $dim * $crate::tensor!(@size $($rest),+)
    };

}
