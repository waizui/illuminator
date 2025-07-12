use crate::core::tensor::Float3;

/// https://pbr-book.org/4ed/Utilities/Mathematical_Infrastructure#EncodeMorton3
pub fn encode_morton3(p: Float3) -> usize {
    let x = p.get(0) as usize;
    let y = p.get(1) as usize;
    let z = p.get(2) as usize;
    (left_shift3(z) << 2) | (left_shift3(y) << 1) | left_shift3(x)
}

pub fn left_shift3(mut x: usize) -> usize {
    assert!(x < 1 << 10);
    if x == (1 << 10) {
        x -= 1
    }

    x = (x | (x << 16)) & 0b00000011000000000000000011111111;
    // x = ---- --98 ---- ---- ---- ---- 7654 3210
    x = (x | (x << 8)) & 0b00000011000000001111000000001111;
    // x = ---- --98 ---- ---- 7654 ---- ---- 3210
    x = (x | (x << 4)) & 0b00000011000011000011000011000011;
    // x = ---- --98 ---- 76-- --54 ---- 32-- --10
    x = (x | (x << 2)) & 0b00001001001001001001001001001001;
    // x = ---- 9--8 --7- -6-- 5--4 --3- -2-- 1--0
    x
}

pub trait MortonCode {
    fn get_morton_code(&self) -> usize;
}

pub fn radix_sort(v: &mut [impl MortonCode]) {
    todo!()
}

