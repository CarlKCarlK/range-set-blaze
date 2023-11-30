#![feature(portable_simd)]
#![feature(array_chunks)]

use core::cmp::PartialEq;
use core::ops::{Add, Sub};
use core::simd::{prelude::*, LaneCount, SimdElement, SupportedLaneCount};
use num_traits::{CheckedAdd, One};

#[inline]
pub fn is_consecutive_regular<T, const N: usize>(chunk: &[T; N]) -> bool
where
    T: SimdElement + Add + PartialEq,
    T: std::ops::Add<Output = T>,
    T: CheckedAdd + One,
{
    for i in 1..N {
        if chunk[i - 1].checked_add(&T::one()) != Some(chunk[i]) {
            return false;
        }
    }
    true
}

// cmk check the asm
#[inline]
pub fn is_consecutive_regular_i64_32(chunk: &[i64; 32]) -> bool {
    is_consecutive_regular::<i64, 32>(chunk)
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

#[macro_export]
macro_rules! reference_splat0 {
    ($function:ident, $type:ty) => {
        pub const fn $function<const N: usize>() -> Simd<$type, N>
        where
            LaneCount<N>: SupportedLaneCount,
        {
            let mut arr: [$type; N] = [0; N];
            let mut i = 0;
            while i < N {
                arr[i] = (N - 1 - i) as $type;
                i += 1;
            }
            Simd::from_array(arr)
        }
    };
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
pub fn is_consecutive_splat0<T, const N: usize>(
    chunk: Simd<T, N>,
    reference: Simd<T, N>,
    n_less_1: T,
) -> bool
where
    T: SimdElement + CheckedAdd + PartialEq,
    LaneCount<N>: SupportedLaneCount,
    Simd<T, N>: Add<Output = Simd<T, N>>,
    Simd<T, N>: PartialEq<Simd<T, N>>,
{
    if chunk[0].checked_add(&n_less_1) != Some(chunk[N - 1]) {
        return false;
    }
    let added = chunk + reference;
    Simd::<T, N>::splat(added[0]) == added
}

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

reference_splat0!(reference_splat0_i64_32, i64);
#[inline]
pub fn is_consecutive_splat0_i64_32(chunk: Simd<i64, 32>) -> bool {
    is_consecutive_splat0::<i64, 32>(chunk, reference_splat0_i64_32(), 31)
}
reference_splat!(reference_splat1_i64_32, i64);
// reference_splat!(reference_splat_integer, Integer);
#[inline]
pub fn is_consecutive_splat1_i64_32(chunk: Simd<i64, 32>) -> bool {
    is_consecutive_splat1::<i64, 32>(chunk, reference_splat1_i64_32())
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

#[macro_export]
macro_rules! reference_rotate {
    ($function:ident, $type:ty) => {
        pub const fn $function<const N: usize>() -> Simd<$type, N>
        where
            LaneCount<N>: SupportedLaneCount,
        {
            let mut arr: [$type; N] = [1; N];
            arr[0] = (1 as $type).wrapping_sub(N as $type);
            Simd::from_array(arr)
        }
    };
}

#[inline]
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

pub type Integer = i16;

#[test]
fn test_is_consecutive() {
    reference_splat0!(reference_splat0_integer, Integer);
    reference_splat!(reference_splat_integer, Integer);
    reference_rotate!(reference_rotate_integer, Integer);
    const LANES: usize = 64;

    //    syntactic_for! {LANES in [2, 4, 8, 16, 32, 64]  {$(

    let a: Vec<Integer> = (100..100 + (LANES as Integer)).collect();
    let ninety_nines: Vec<Integer> = vec![99; LANES];
    let a = Simd::<Integer, LANES>::from_slice(&a);
    let ninety_nines = Simd::<Integer, LANES>::from_slice(ninety_nines.as_slice());

    assert!(is_consecutive_regular(a.as_array()));
    assert!(!is_consecutive_regular(ninety_nines.as_array()));

    assert!(is_consecutive_splat0(
        a,
        reference_splat0_integer(),
        (LANES - 1) as Integer
    ));
    assert!(!is_consecutive_splat0(
        ninety_nines,
        reference_splat0_integer(),
        (LANES - 1) as Integer
    ));

    assert!(is_consecutive_splat1(a, reference_splat_integer()));
    assert!(!is_consecutive_splat1(
        ninety_nines,
        reference_splat_integer()
    ));

    assert!(is_consecutive_splat2(a, reference_splat_integer()));
    assert!(!is_consecutive_splat2(
        ninety_nines,
        reference_splat_integer()
    ));

    assert!(is_consecutive_rotate(a, reference_rotate_integer()));
    assert!(!is_consecutive_rotate(
        ninety_nines,
        reference_rotate_integer()
    ));
}
