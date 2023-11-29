#![feature(portable_simd)]

use core::cmp::PartialEq;
use core::ops::{Add, Sub};
use core::simd::{prelude::*, LaneCount, SimdElement, SupportedLaneCount};

#[inline]
pub fn is_consecutive_regular<T, const N: usize>(chunk: &[T; N], one: T, max: T) -> bool
where
    T: SimdElement + Add + PartialEq,
    T: std::ops::Add<Output = T>,
{
    for i in 1..N {
        if chunk[i - 1] == max || chunk[i - 1] + one != chunk[i] {
            return false;
        }
    }
    true
}

#[inline]
pub fn is_consecutive_regular_i64_32(chunk: &[i64; 32]) -> bool {
    is_consecutive_regular::<i64, 32>(chunk, 1, i64::MAX)
}

#[test]
fn test_regular() {
    let a: Vec<i64> = (100..132).collect();
    let ninety_nines: Vec<i64> = vec![99; 32];
    let a = Simd::<i64, 32>::from_slice(&a);
    let ninety_nines = Simd::<i64, 32>::from_slice(ninety_nines.as_slice());

    assert!(is_consecutive_regular_i64_32(a.as_array()));
    assert!(!is_consecutive_regular_i64_32(ninety_nines.as_array()));
}

// const REFERENCE_SPLAT0: Simd<T, N> =
//     Simd::from_array([15, 14, 13, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1, 0]);

// cmk00
// pub fn is_consecutive_splat0<T, const N: usize>(chunk: Simd<T, N>, reference: Simd<T, N>) -> bool
// where
//     T: SimdElement,
//     LaneCount<N>: SupportedLaneCount,
//     Simd<T, N>: Sub<Output = Simd<T, N>>,
//     Simd<T, N>: PartialEq<Simd<T, N>>,
// {
//     if chunk[0].overflowing_add(N as T - 1) != (chunk[N - 1], false) {
//         return false;
//     }
//     let added = chunk + reference;
//     Simd::<T, N>::splat(added[0]) == added
// }

#[inline]
pub fn is_consecutive_splat1<T, const N: usize>(chunk: Simd<T, N>, reference: Simd<T, N>) -> bool
where
    T: SimdElement,
    LaneCount<N>: SupportedLaneCount,
    Simd<T, N>: Sub<Output = Simd<T, N>>,
    Simd<T, N>: PartialEq<Simd<T, N>>,
{
    let subtracted = chunk - reference;
    Simd::<T, N>::splat(chunk[0]) == subtracted
}

#[macro_export]
macro_rules! reference_splat {
    ($function:ident, $type:ty) => {
        pub const fn $function<const N: usize>() -> Simd<$type, N>
        where
            LaneCount<N>: SupportedLaneCount,
        {
            let mut arr: [$type; N] = [0; N];
            let mut i = 0;
            while i < N {
                arr[i] = i as $type;
                i += 1;
            }
            Simd::from_array(arr)
        }
    };
}

#[inline]
pub fn is_consecutive_splat2<T, const N: usize>(chunk: Simd<T, N>, reference: Simd<T, N>) -> bool
where
    T: SimdElement,
    LaneCount<N>: SupportedLaneCount,
    Simd<T, N>: Sub<Output = Simd<T, N>>,
    Simd<T, N>: PartialEq<Simd<T, N>>,
{
    let subtracted = chunk - reference;
    Simd::<T, N>::splat(subtracted[0]) == subtracted
}

// cmk00
// pub fn is_consecutive_sizzle<T, const N: usize>(chunk: Simd<T, N>, reference: Simd<T, N>) -> bool
// where
//     T: SimdElement,
//     LaneCount<N>: SupportedLaneCount,
//     Simd<T, N>: Sub<Output = Simd<T, N>>,
//     Simd<T, N>: PartialEq<Simd<T, N>>,
// {
//     let subtracted = chunk - reference;
//     simd_swizzle!(subtracted, [0; N]) == subtracted
// }

// const REFERENCE_ROTATE: Simd<T, N> =
//     Simd::from_array([4294967281, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]);

pub fn is_consecutive_rotate<T, const N: usize>(chunk: Simd<T, N>, reference: Simd<T, N>) -> bool
where
    T: SimdElement,
    LaneCount<N>: SupportedLaneCount,
    Simd<T, N>: Sub<Output = Simd<T, N>>,
    Simd<T, N>: PartialEq<Simd<T, N>>,
{
    let rotated = chunk.rotate_lanes_right::<1>();
    chunk - rotated == reference
}

pub type Integer = i64;

#[cfg(test)]
use syntactic_for::syntactic_for;

#[test]
fn test_is_consecutive() {
    reference_splat!(reference_splat_integer, Integer);

    syntactic_for! {LANES in [2, 4, 8, 16, 32, 64]  {$(

        let a: Vec<Integer> = (100 as Integer..100 + $LANES).collect();
        let ninety_nines: Vec<Integer> = vec![99; $LANES];
        let a = Simd::<Integer, $LANES>::from_slice(&a);
        let ninety_nines = Simd::<Integer, $LANES>::from_slice(ninety_nines.as_slice());

        assert!(is_consecutive_regular(a.as_array(), 1, Integer::MAX));
        assert!(!is_consecutive_regular(ninety_nines.as_array(), 1, Integer::MAX));


        // assert!(is_consecutive_splat0(a));
        // assert!(!is_consecutive_splat0(ninety_nines));

        assert!(is_consecutive_splat1(a, reference_splat_integer()));
        assert!(!is_consecutive_splat1(ninety_nines, reference_splat_integer()));

        assert!(is_consecutive_splat2(a, reference_splat_integer()));
        assert!(!is_consecutive_splat2(ninety_nines, reference_splat_integer()));

        // assert!(is_consecutive_sizzle(a));
        // assert!(!is_consecutive_sizzle(ninety_nines));

        // assert!(is_consecutive_rotate(a));
        // assert!(!is_consecutive_rotate(ninety_nines));
    )*}}
}

// #[test]
// fn test_nested() {
//     syntactic_for! {INTEGER in [i16, u32, u64]  {
//         syntactic_for! {LANES in [2, 4, 8, 16, 32, 64]  {
//         $(
//         println!("INTEGER: {}, LANES: {}", stringify!($INTEGER), $LANES);
//         )
//         }}
//     *}}
// }
