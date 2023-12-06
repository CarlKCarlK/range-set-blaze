#![feature(portable_simd)]

use std::simd::prelude::*;

pub const LANES: usize = 16;
pub fn is_consecutive_regular(chunk: &[u32; LANES]) -> bool {
    for i in 1..LANES {
        if chunk[i - 1].checked_add(1) != Some(chunk[i]) {
            return false;
        }
    }
    true
}

const REFERENCE_SPLAT0: Simd<u32, LANES> =
    Simd::from_array([15, 14, 13, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1, 0]);

pub fn is_consecutive_splat0(chunk: Simd<u32, LANES>) -> bool {
    if chunk[0].overflowing_add(LANES as u32 - 1) != (chunk[LANES - 1], false) {
        return false;
    }
    let added = chunk + REFERENCE_SPLAT0;
    Simd::splat(added[0]) == added
}

const REFERENCE_SPLAT1: Simd<u32, LANES> =
    Simd::from_array([0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15]);

pub fn is_consecutive_splat1(chunk: Simd<u32, LANES>) -> bool {
    let subtracted = chunk - REFERENCE_SPLAT1;
    Simd::splat(chunk[0]) == subtracted
}

pub fn is_consecutive_splat2(chunk: Simd<u32, LANES>) -> bool {
    let subtracted = chunk - REFERENCE_SPLAT1;
    Simd::splat(subtracted[0]) == subtracted
}

pub fn is_consecutive_sizzle(chunk: Simd<u32, LANES>) -> bool {
    let subtracted = chunk - REFERENCE_SPLAT1;
    simd_swizzle!(subtracted, [0; LANES]) == subtracted
}

const REFERENCE_ROTATE: Simd<u32, LANES> =
    Simd::from_array([4294967281, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]);

pub fn is_consecutive_rotate(chunk: Simd<u32, LANES>) -> bool {
    let rotated = chunk.rotate_elements_right::<1>();
    chunk - rotated == REFERENCE_ROTATE
}

#[test]
fn test_is_consecutive() {
    use std::array;

    let a: [u32; LANES] = array::from_fn(|i| 100 + i as u32);
    let ninety_nines: [u32; LANES] = [99; LANES];
    assert!(is_consecutive_regular(&a));
    assert!(!is_consecutive_regular(&ninety_nines));

    let a = Simd::from_array(a);
    let ninety_nines = Simd::from_array(ninety_nines);

    assert!(is_consecutive_splat0(a));
    assert!(!is_consecutive_splat0(ninety_nines));

    assert!(is_consecutive_splat1(a));
    assert!(!is_consecutive_splat1(ninety_nines));

    assert!(is_consecutive_splat2(a));
    assert!(!is_consecutive_splat2(ninety_nines));

    assert!(is_consecutive_sizzle(a));
    assert!(!is_consecutive_sizzle(ninety_nines));

    assert!(is_consecutive_rotate(a));
    assert!(!is_consecutive_rotate(ninety_nines));
}
