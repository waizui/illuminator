use crate::core::tensor::Float3;
use std::mem;

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

pub trait MortonCode: Default {
    fn morton_code(&self) -> usize;
}

pub fn radix_sort(v: &mut [impl MortonCode]) {
    let mut orgv: Vec<(usize, usize)> = v
        .iter()
        .enumerate()
        .map(|(i, x)| (i, x.morton_code()))
        .collect();

    let mut tempv: Vec<(usize, usize)> = (0..v.len()).enumerate().collect();

    let pass_bits = 6;
    let nbits = 30;
    assert_eq!(nbits % pass_bits, 0);
    let npasses = nbits / pass_bits;

    for pass in 0..npasses {
        // perform one pass of radix sort, sorting _bitsPerPass_ bits
        let lowbit = pass * pass_bits;
        // set in and out vector references for radix sort pass
        let (invec, outvec) = {
            if pass & 1 == 1 {
                (&mut tempv, &mut orgv)
            } else {
                (&mut orgv, &mut tempv)
            }
        };

        // count number of zero bits in array for current radix sort bit
        let nbuckets = 1 << pass_bits;
        let mut buckets_count: Vec<usize> = vec![0; nbuckets];
        let bit_mask = (1 << pass_bits) - 1;
        for &mp in invec.iter() {
            let code = mp.1;
            let bucket = (code >> lowbit) & bit_mask;
            buckets_count[bucket] += 1;
        }

        // compute starting index in output array for each bucket
        let mut out_index: Vec<usize> = vec![0; nbuckets];
        out_index[0] = 0;
        for i in 1..nbuckets {
            out_index[i] = out_index[i - 1] + buckets_count[i - 1];
        }

        for &mp in invec.iter() {
            let code = mp.1;
            let bucket = (code >> lowbit) & bit_mask;
            outvec[out_index[bucket]] = mp;
            out_index[bucket] += 1;
        }
    }

    // make orgv always sorted one
    if npasses & 1 == 1 {
        mem::swap(&mut orgv, &mut tempv);
    }

    // mapping proxy array order to orginal array
    let mut map: Vec<usize> = orgv.iter().map(|(org_i, _)| *org_i).collect();
    inplace_map(&mut map, v);
}

/// new[i] = org[map[i]]
pub fn inplace_map<T>(map: &mut [usize], org: &mut [T])
where
    T: Default,
{
    assert_eq!(map.len(), org.len());
    for i in 0..org.len() {
        let x = mem::take(&mut org[i]);
        let mut j = i;
        loop {
            let k = map[j];
            map[j] = j;
            if i == k {
                break;
            }
            org.swap(k, j);
            j = k;
        }

        org[j] = x;
    }
}

/// new[map[i]] = org[i]
pub fn inplace_map_rev<T>(map: &mut [usize], org: &mut [T]) {
    let n = org.len();
    for i in 0..n {
        while map[i] != i {
            let k = map[i];
            org.swap(i, k);
            map.swap(i, k);
        }
    }
}

#[test]
fn test_inplace_map() {
    let mut map = [2, 0, 1, 3];
    let mut chars = ["A", "B", "C", "D"];
    inplace_map_rev(&mut map, &mut chars);
    assert_eq!(chars, ["B", "C", "A", "D"]);

    let mut map = [2, 0, 1, 3];
    let mut chars = ["A", "B", "C", "D"];
    inplace_map(&mut map, &mut chars);
    assert_eq!(chars, ["C", "A", "B", "D"]);
}

#[test]
fn test_encode_morton() {
    let p = Float3::vec(&[1.; 3]);
    let m = encode_morton3(p);
    assert_eq!(m, 7);

    let p = Float3::vec(&[1., 2., 3.]);
    let m = encode_morton3(p);
    assert_eq!(m, 0b110101);

    let p = Float3::vec(&[1023.; 3]);
    let m = encode_morton3(p);
    assert_eq!(m, 0b111111111111111111111111111111);
}

#[test]
fn test_radix_sort() {
    #[derive(Default)]
    struct TestMorton {
        morton_code: usize,
        org_index: usize,
    }

    impl MortonCode for TestMorton {
        fn morton_code(&self) -> usize {
            self.morton_code
        }
    }

    let nm = 256;
    let mut ms: Vec<TestMorton> = Vec::with_capacity(nm);
    for i in 0..nm {
        let x = (i as f32 / nm as f32) * 1024.;
        let m = TestMorton {
            morton_code: encode_morton3(Float3::vec(&[x; 3])),
            org_index: i,
        };
        ms.push(m);
    }

    for i in (0..nm).step_by(2) {
        ms.swap(i, i + 1);
    }
    ms.reverse();
    radix_sort(&mut ms);
    for (i, m) in ms.iter().enumerate() {
        assert_eq!(m.org_index, i);
    }
}
