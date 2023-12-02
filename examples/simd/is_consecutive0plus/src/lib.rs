#![feature(portable_simd)]

use std::{
    ops::Sub,
    simd::{prelude::*, LaneCount, SimdElement, SupportedLaneCount},
};

// We can make our main function generic both for type and # of lanes:
#[inline]
pub fn is_consecutive_splat1_gen<T, const N: usize>(
    chunk: Simd<T, N>,
    reference: Simd<T, N>,
) -> bool
where
    T: SimdElement + PartialEq,
    Simd<T, N>: Sub<Simd<T, N>, Output = Simd<T, N>>,
    LaneCount<N>: SupportedLaneCount,
{
    let subtracted = chunk - reference;
    Simd::splat(chunk[0]) == subtracted
}

// // But can we safely make the const reference_splat generic for type and # of lanes?
// // Not curretly, because we need From or One to be const.
// use std::ops::AddAssign;

// pub const fn reference_splat_gen<T, const N: usize>() -> Simd<T, N>
// where
//     T: SimdElement + Default + From<usize> + AddAssign,
//     LaneCount<N>: SupportedLaneCount,
// {
//     let mut arr: [T; N] = [T::from(0usize); N];
//     let mut i_usize = 0;
//     while i_usize < N {
//         arr[i_usize] = T::from(i_usize);
//         i_usize += 1;
//     }
//     Simd::from_array(arr)
// }

// We can use a macro for the type and leave lane count a const generic.
// However, in my case, I need to run on different types, but I need only one lane count at a time.
// So, we use a macro from the type and a const for lane count.

#[macro_export]
macro_rules! define_is_consecutive_splat1 {
    ($function:ident, $type:ty) => {
        #[inline]
        pub fn $function(chunk: Simd<$type, LANES>) -> bool {
            define_reference_splat!(reference_splat, $type);

            let subtracted = chunk - reference_splat();
            Simd::splat(chunk[0]) == subtracted
        }
    };
}
#[allow(unused_macros)]
macro_rules! define_reference_splat {
    ($function:ident, $type:ty) => {
        pub const fn $function() -> Simd<$type, LANES> {
            let mut arr: [$type; LANES] = [0; LANES];
            let mut i = 0;
            while i < LANES {
                arr[i] = i as $type;
                i += 1;
            }
            Simd::from_array(arr)
        }
    };
}

pub const LANES: usize = 16;

#[cfg(test)]
use std::hint::black_box;

#[test]
fn test_is_consecutive() {
    use std::array;

    let a: [i8; LANES] = array::from_fn(|i| 100 + i as i8);
    let ninety_nines: [i8; LANES] = [99; LANES];
    let a = black_box(Simd::from_array(a));
    let ninety_nines = black_box(Simd::from_array(ninety_nines));

    define_is_consecutive_splat1!(is_consecutive_splat1_i8, i8);
    assert!(is_consecutive_splat1_i8(a));
    assert!(!is_consecutive_splat1_i8(ninety_nines));
}

// cmk see https://godbolt.org/z/69dY1fvGj and see that it compiles well.
