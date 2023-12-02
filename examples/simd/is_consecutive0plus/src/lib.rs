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
#[allow(unused_macros)]
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

// cmk see https://godbolt.org/z/69dY1fvGj and see that it compiles well.

trait IsConsecutive {
    fn is_consecutive<const N: usize>(chunk: Simd<Self, N>) -> bool
    where
        Self: SimdElement,
        Simd<Self, N>: Sub<Simd<Self, N>, Output = Simd<Self, N>>,
        LaneCount<N>: SupportedLaneCount;
}

macro_rules! impl_is_consecutive {
    ($type:ty) => {
        // Repeat for each integer type (i8, i16, i32, i64, isize, u8, u16, u32, u64, usize)
        impl IsConsecutive for $type {
            #[inline] // cmk important
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

#[cfg(test)]
use std::hint::black_box;

#[test]
fn test_is_consecutive() {
    use std::array;

    // Works on i32 and 16 lanes
    let a: Simd<i32, 16> = black_box(Simd::from_array(array::from_fn(|i| 100 + i as i32)));
    let ninety_nines: Simd<i32, 16> = black_box(Simd::from_array([99; 16]));

    assert!(IsConsecutive::is_consecutive(a));
    assert!(!IsConsecutive::is_consecutive(ninety_nines));

    // Works on i8 and 64 lanes
    let a: Simd<i8, 64> = black_box(Simd::from_array(array::from_fn(|i| 100 + i as i8)));
    let ninety_nines: Simd<i8, 64> = black_box(Simd::from_array([99; 64]));

    assert!(IsConsecutive::is_consecutive(a));
    assert!(!IsConsecutive::is_consecutive(ninety_nines));
}
