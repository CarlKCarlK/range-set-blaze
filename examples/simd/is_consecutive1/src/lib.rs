#![feature(portable_simd)]

use std::{
    ops::Add,
    ops::Sub,
    simd::{prelude::*, LaneCount, SimdElement, SupportedLaneCount},
};

#[macro_export]
macro_rules! define_is_consecutive_regular {
    ($function:ident, $type:ty) => {
        #[inline]
        pub fn $function<const N: usize>(chunk: &[$type; N]) -> bool
        where
            LaneCount<N>: SupportedLaneCount,
        {
            for i in 1..N {
                if chunk[i - 1].checked_add(1) != Some(chunk[i]) {
                    return false;
                }
            }
            true
        }
    };
}

#[macro_export]
macro_rules! define_is_consecutive_splat0 {
    ($function:ident, $type:ty) => {
        #[inline]
        pub fn $function<const N: usize>(chunk: Simd<$type, N>) -> bool
        where
            $type: SimdElement,
            Simd<$type, N>: Add<Simd<$type, N>, Output = Simd<$type, N>>,
            LaneCount<N>: SupportedLaneCount,
        {
            define_reference_splat0!(reference_splat0, $type);

            if chunk[0].overflowing_add(N as $type - 1) != (chunk[N - 1], false) {
                return false;
            }
            let added = chunk + reference_splat0();
            Simd::splat(added[0]) == added
        }
    };
}
#[macro_export]
macro_rules! define_reference_splat0 {
    ($function:ident, $type:ty) => {
        pub const fn $function<const N: usize>() -> Simd<$type, N>
        where
            LaneCount<N>: SupportedLaneCount,
        {
            let mut arr: [$type; N] = [0; N];
            let mut i = 0;
            while i < N {
                arr[i] = (N - i - 1) as $type;
                i += 1;
            }
            Simd::from_array(arr)
        }
    };
}

#[macro_export]
macro_rules! define_is_consecutive_splat1 {
    ($function:ident, $type:ty) => {
        #[inline]
        pub fn $function<const N: usize>(chunk: Simd<$type, N>) -> bool
        where
            LaneCount<N>: SupportedLaneCount,
        {
            define_reference_splat!(reference_splat, $type);

            let subtracted = chunk - reference_splat();
            Simd::splat(chunk[0]) == subtracted
        }
    };
}

#[macro_export]
macro_rules! define_reference_splat {
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

#[macro_export]
macro_rules! define_is_consecutive_splat2 {
    ($function:ident, $type:ty) => {
        #[inline]
        pub fn $function<const N: usize>(chunk: Simd<$type, N>) -> bool
        where
            LaneCount<N>: SupportedLaneCount,
        {
            define_reference_splat!(reference_splat, $type);

            let subtracted = chunk - reference_splat();
            Simd::splat(subtracted[0]) == subtracted
        }
    };
}

#[test]
fn test_regular() {
    const LANES: usize = 16;
    let a: Vec<i64> = (100..100 + LANES as i64).collect();
    let ninety_nines: Vec<i64> = vec![99; LANES];
    let a = Simd::<i64, LANES>::from_slice(&a);
    let ninety_nines = Simd::<i64, LANES>::from_slice(ninety_nines.as_slice());

    assert!(IsConsecutive::is_consecutive_regular(a.as_array()));
    assert!(!IsConsecutive::is_consecutive_regular(
        ninety_nines.as_array()
    ));
}

#[macro_export]
macro_rules! define_is_consecutive_rotate {
    ($function:ident, $type:ty) => {
        #[inline]
        pub fn $function<const N: usize>(chunk: Simd<$type, N>) -> bool
        where
            $type: SimdElement,
            LaneCount<N>: SupportedLaneCount,
        {
            define_reference_rotate!(reference, $type);

            let rotated = chunk.rotate_lanes_right::<1>();
            chunk - rotated == reference()
        }
    };
}

#[macro_export]
macro_rules! define_reference_rotate {
    ($function:ident, $type:ty) => {
        #[inline]
        pub const fn $function<const N: usize>() -> Simd<$type, N>
        where
            $type: SimdElement,
            LaneCount<N>: SupportedLaneCount,
        {
            let mut arr: [$type; N] = [1; N];
            arr[0] = (1 as $type).wrapping_sub(N as $type);
            Simd::from_array(arr)
        }
    };
}

#[test]
fn test_is_consecutive() {
    use core::simd::Simd;

    pub type Integer = i16;
    const LANES: usize = 4;

    let a: Vec<Integer> = (100..100 + LANES as Integer).collect();
    let ninety_nines: Vec<Integer> = vec![99; LANES];
    let a = Simd::<Integer, LANES>::from_slice(&a);
    let ninety_nines = Simd::<Integer, LANES>::from_slice(ninety_nines.as_slice());

    assert!(IsConsecutive::is_consecutive_regular(a.as_array()));
    assert!(!IsConsecutive::is_consecutive_regular(
        ninety_nines.as_array()
    ));

    assert!(IsConsecutive::is_consecutive_splat0(a));
    assert!(!IsConsecutive::is_consecutive_splat0(ninety_nines));

    assert!(IsConsecutive::is_consecutive_splat1(a));
    assert!(!IsConsecutive::is_consecutive_splat1(ninety_nines));

    assert!(IsConsecutive::is_consecutive_splat2(a));
    assert!(!IsConsecutive::is_consecutive_splat2(ninety_nines));

    assert!(IsConsecutive::is_consecutive_rotate(a));
    assert!(!IsConsecutive::is_consecutive_rotate(ninety_nines));
}

pub trait IsConsecutive {
    fn is_consecutive_regular<const N: usize>(chunk: &[Self; N]) -> bool
    where
        Self: Sized,
        LaneCount<N>: SupportedLaneCount;

    fn is_consecutive_splat0<const N: usize>(chunk: Simd<Self, N>) -> bool
    where
        Self: SimdElement,
        LaneCount<N>: SupportedLaneCount;

    fn is_consecutive_splat1<const N: usize>(chunk: Simd<Self, N>) -> bool
    where
        Self: SimdElement,
        LaneCount<N>: SupportedLaneCount;

    fn is_consecutive_splat2<const N: usize>(chunk: Simd<Self, N>) -> bool
    where
        Self: SimdElement,
        LaneCount<N>: SupportedLaneCount;

    fn is_consecutive_rotate<const N: usize>(chunk: Simd<Self, N>) -> bool
    where
        Self: SimdElement,
        LaneCount<N>: SupportedLaneCount;
}

macro_rules! impl_is_consecutive {
    ($type:ty) => {
        impl IsConsecutive for $type {
            #[inline]
            fn is_consecutive_regular<const N: usize>(chunk: &[$type; N]) -> bool
            where
                LaneCount<N>: SupportedLaneCount,
            {
                define_is_consecutive_regular!(is_consecutive_regular, $type);
                is_consecutive_regular(chunk)
            }

            #[inline]
            fn is_consecutive_splat0<const N: usize>(chunk: Simd<Self, N>) -> bool
            where
                Self: SimdElement,
                Simd<Self, N>: Sub<Simd<Self, N>, Output = Simd<Self, N>>,
                LaneCount<N>: SupportedLaneCount,
            {
                define_is_consecutive_splat0!(is_consecutive_splat0, $type);
                is_consecutive_splat0(chunk)
            }

            #[inline]
            fn is_consecutive_splat1<const N: usize>(chunk: Simd<Self, N>) -> bool
            where
                Self: SimdElement,
                Simd<Self, N>: Sub<Simd<Self, N>, Output = Simd<Self, N>>,
                LaneCount<N>: SupportedLaneCount,
            {
                define_is_consecutive_splat1!(is_consecutive_splat1, $type);
                is_consecutive_splat1(chunk)
            }

            #[inline]
            fn is_consecutive_splat2<const N: usize>(chunk: Simd<Self, N>) -> bool
            where
                Self: SimdElement,
                Simd<Self, N>: Add<Simd<Self, N>, Output = Simd<Self, N>>,
                LaneCount<N>: SupportedLaneCount,
            {
                define_is_consecutive_splat2!(is_consecutive_splat2, $type);
                is_consecutive_splat2(chunk)
            }

            #[inline]
            fn is_consecutive_rotate<const N: usize>(chunk: Simd<Self, N>) -> bool
            where
                Self: SimdElement,
                Simd<Self, N>: Add<Simd<Self, N>, Output = Simd<Self, N>>,
                LaneCount<N>: SupportedLaneCount,
            {
                define_is_consecutive_rotate!(is_consecutive_rotate, $type);
                is_consecutive_rotate(chunk)
            }
        }
    };
}

impl_is_consecutive!(i8);
impl_is_consecutive!(i16);
impl_is_consecutive!(i32);
impl_is_consecutive!(i64);
impl_is_consecutive!(isize);
impl_is_consecutive!(u8);
impl_is_consecutive!(u16);
impl_is_consecutive!(u32);
impl_is_consecutive!(u64);
impl_is_consecutive!(usize);
