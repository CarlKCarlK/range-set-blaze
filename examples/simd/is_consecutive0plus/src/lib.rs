#![feature(portable_simd)]

use std::{
    ops::Sub,
    simd::{prelude::*, SimdElement},
};

pub const LANES: usize = 16;

// pub fn is_consecutive_regular(chunk: &[T; LANES]) -> bool {
//     for i in 1..LANES {
//         if chunk[i - 1] == T::MAX || chunk[i - 1] + 1 != chunk[i] {
//             return false;
//         }
//     }
//     true
// }

// const REFERENCE_SPLAT1: Simd<T, LANES> =
//     Simd::from_array([0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15]);

pub fn is_consecutive_splat1<T>(chunk: Simd<T, LANES>, reference: Simd<T, LANES>) -> bool
where
    T: SimdElement + PartialEq + Sub<Simd<T, LANES>, Output = Simd<T, LANES>>,
{
    let subtracted = chunk - reference;
    Simd::<T, LANES>::splat(chunk[0]) == subtracted
}

#[test]
fn test_is_consecutive() {
    use std::array;

    let a: [T; LANES] = array::from_fn(|i| 100 + i as T);
    let ninety_nines: [T; LANES] = [99; LANES];
    // assert!(is_consecutive_regular(&a));
    // assert!(!is_consecutive_regular(&ninety_nines));

    let a = Simd::from_array(a);
    let ninety_nines = Simd::from_array(ninety_nines);

    assert!(is_consecutive_splat1(a));
    assert!(!is_consecutive_splat1(ninety_nines));
}
