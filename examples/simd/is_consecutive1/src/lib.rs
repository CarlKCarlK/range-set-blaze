#![feature(portable_simd)]

use std::{
    ops::Sub,
    simd::{prelude::*, LaneCount, SimdElement, SupportedLaneCount},
};

// We can make our main function generic both for type and # of lanes:
#[inline]
pub fn is_consecutive_splat1_gen<T, const N: usize>(
    chunk: Simd<T, N>,
    comparison_value: Simd<T, N>,
) -> bool
where
    T: SimdElement + PartialEq,
    Simd<T, N>: Sub<Simd<T, N>, Output = Simd<T, N>>,
    LaneCount<N>: SupportedLaneCount,
{
    let subtracted = chunk - comparison_value;
    Simd::splat(chunk[0]) == subtracted
}

// // But can we safely make the const comparison_value_splat generic for type and # of lanes?
// // Not currently, because we need From or One to be const.
// use std::ops::AddAssign;

// pub const fn comparison_value_splat_gen<T, const N: usize>() -> Simd<T, N>
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

// So, we'll next use a macro over the type and make LANES const generic.

#[macro_export]
macro_rules! define_is_consecutive_splat1 {
    ($function:ident, $type:ty) => {
        #[inline]
        pub fn $function<const N: usize>(chunk: Simd<$type, N>) -> bool
        where
            LaneCount<N>: SupportedLaneCount,
        {
            define_comparison_value_splat!(comparison_value_splat, $type);

            let subtracted = chunk - comparison_value_splat();
            Simd::splat(chunk[0]) == subtracted
        }
    };
}
#[macro_export]
macro_rules! define_comparison_value_splat {
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

#[test]
fn test_is_consecutive_macros() {
    use std::array;
    use std::hint::black_box;

    // Works on i32 and 16 lanes
    define_is_consecutive_splat1!(is_consecutive_splat1_i32, i32);

    let a: Simd<i32, 16> = black_box(Simd::from_array(array::from_fn(|i| 100 + i as i32)));
    let ninety_nines: Simd<i32, 16> = black_box(Simd::from_array([99; 16]));
    assert!(is_consecutive_splat1_i32(a));
    assert!(!is_consecutive_splat1_i32(ninety_nines));

    // Works on i8 and 64 lanes
    define_is_consecutive_splat1!(is_consecutive_splat1_i8, i8);

    let a: Simd<i8, 64> = black_box(Simd::from_array(array::from_fn(|i| 10 + i as i8)));
    let ninety_nines: Simd<i8, 64> = black_box(Simd::from_array([99; 64]));
    assert!(is_consecutive_splat1_i8(a));
    assert!(!is_consecutive_splat1_i8(ninety_nines));
}

// We can also make a trait for this, allowing type to also be generic.

pub trait IsConsecutive {
    fn is_consecutive<const N: usize>(chunk: Simd<Self, N>) -> bool
    where
        Self: SimdElement,
        Simd<Self, N>: Sub<Simd<Self, N>, Output = Simd<Self, N>>,
        LaneCount<N>: SupportedLaneCount;
}

macro_rules! impl_is_consecutive {
    ($type:ty) => {
        impl IsConsecutive for $type {
            #[inline] // very important
            fn is_consecutive<const N: usize>(chunk: Simd<Self, N>) -> bool
            where
                Self: SimdElement,
                Simd<Self, N>: Sub<Simd<Self, N>, Output = Simd<Self, N>>,
                LaneCount<N>: SupportedLaneCount,
            {
                define_is_consecutive_splat1!(is_consecutive_splat1, $type);
                is_consecutive_splat1(chunk)
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

#[test]
fn test_is_consecutive_trait() {
    use std::array;
    use std::hint::black_box;

    // Works on i32 and 16 lanes
    let a: Simd<i32, 16> = black_box(Simd::from_array(array::from_fn(|i| 100 + i as i32)));
    let ninety_nines: Simd<i32, 16> = black_box(Simd::from_array([99; 16]));

    assert!(IsConsecutive::is_consecutive(a));
    assert!(!IsConsecutive::is_consecutive(ninety_nines));

    // Works on i8 and 64 lanes
    let a: Simd<i8, 64> = black_box(Simd::from_array(array::from_fn(|i| 10 + i as i8)));
    let ninety_nines: Simd<i8, 64> = black_box(Simd::from_array([99; 64]));

    assert!(IsConsecutive::is_consecutive(a));
    assert!(!IsConsecutive::is_consecutive(ninety_nines));
}
