use num_traits::clamp;

//TODO:64bit support
pub const MACHINE_EPSILON32: f32 = f32::EPSILON * 0.5;
pub const ONE_MINUS_EPSILON: f32 = 1.0 - f32::EPSILON;

pub fn gamma(n: i32) -> f32 {
    (n as f32 * MACHINE_EPSILON32) / (1. - n as f32 * MACHINE_EPSILON32)
}

/// returned value no large than size-2
/// returens 0 if pred all false, size - 2 if all true
/// eg: for pred of elements =2: [2,2,3]->1 ,[2,2,2]-> 1  
pub fn split_index<F>(size: usize, pred: F) -> usize
where
    F: Fn(usize) -> bool,
{
    let (mut sz, mut first) = (size.saturating_sub(2), 1);

    while sz > 0 {
        let half = sz >> 1;
        let mid = first + half;
        let res = pred(mid);
        first = if res { mid + 1 } else { first };
        sz = if res {
            sz.saturating_sub(half + 1)
        } else {
            half
        };
    }

    clamp(first - 1, 0, size.saturating_sub(2))
}

pub fn factorial(x: i32) -> f32 {
    if x == 0 {
        return 1f32;
    }
    (1..x + 1).fold(1., |acc, x| acc * x as f32)
}

#[test]
fn test_split() {
    let arr = [2, 2, 3];
    let i = split_index(arr.len(), |i| arr[i] == 2);
    assert_eq!(i, 1);

    let arr = [2, 2, 2];
    let i = split_index(arr.len(), |i| arr[i] == 2);
    assert_eq!(i, 1);

    let arr = [2, 2, 3];
    let i = split_index(arr.len(), |i| arr[i] == 1);
    assert_eq!(i, 0);
}
