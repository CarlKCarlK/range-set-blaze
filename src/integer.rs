use core::ops::RangeInclusive;
use core::simd::{LaneCount, SupportedLaneCount};
use std::simd::prelude::*; // cmk use? when?

// cmk may want to skip sse2 (128) because it is slower than the non-simd version

use crate::Integer;

// macro_rules! create_const_array {
//     ($element:ty, $create_constant_id:ident) => {
//         #[repr(align(64))] // cmk 64 is a guess
//         struct AlignedArray<T, const N: usize> {
//             data: [T; N],
//         }

//         // cmk better name?
//         #[allow(dead_code)]
//         const fn $create_constant_id<const N: usize>() -> [$element; N] {
//             let mut arr = AlignedArray { data: [1; N] };
//             // cmk000 make this right
//             arr.data[0] = !(N as $element) + 1;
//             arr.data
//         }
//     };
// }

// cmk is 'chunk' the best name?
#[allow(unused_macros)] // cmk
macro_rules! is_consecutive {
    ($element:ty, $N:tt, $expected:ident) => {
        // cmk better name?

        // fn is_consecutive<const N: usize>(chunk: &Simd<Self, N>) -> bool
        // where
        //     LaneCount<N>: SupportedLaneCount,
        // {
        //     debug_assert!(N == $N, "Chunk is wrong length");
        //     let expected: &Simd<$element, N> = unsafe { std::mem::transmute(&EXPECTED_I32X16) };
        //     let b = chunk.rotate_lanes_right::<1>();
        //     chunk - b == *expected
        // }

        fn is_consecutive<const N: usize>(chunk: &Simd<Self, N>) -> bool
        where
            LaneCount<N>: SupportedLaneCount,
        {
            assert!(N == 16, "Chunk is wrong length");
            if N == 16 {
                let b = chunk.rotate_lanes_right::<1>();
                let expected: &Simd<$element, N> = unsafe { std::mem::transmute(&$expected) };
                chunk - b == *expected
            } else {
                // Fallback or generic logic
                false
            }
        }

        //     // // cmk0 should do with from_slice_uncheck unsafe????
        //     // // cmk0 is a[0] the best way to extract an element?
        //     // // cmk0 is a[0] and then splat the best way to create a simd from one element?
        //     // // cmk0 is eq the best way to compare two simds?
        //     // let b = <$simd>::splat(a[0]); // cmk0 seems same as simd_swizzle!
        //     //                               // cmk0 let b = simd_swizzle!(a, [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
        //     // let a = <$simd>::from_slice(chunk) + $decrease; // decrease is 0, -1, -2 ...
        //     // a == <$simd>::splat(a[0])

        //     // This needlessly check's length and fixes the alignment, but that doesn't to slow things.
        //     // let a = <$simd>::from_slice(chunk);
        //     let b = chunk.rotate_lanes_right::<1>();
        //     chunk - b == $expected
        // }
    };
}

impl Integer for i8 {
    #[cfg(target_pointer_width = "32")]
    type SafeLen = usize;
    #[cfg(target_pointer_width = "64")]
    type SafeLen = usize;

    fn safe_len(r: &RangeInclusive<Self>) -> <Self as Integer>::SafeLen {
        r.end().overflowing_sub(*r.start()).0 as u8 as <Self as Integer>::SafeLen + 1
    }
    fn safe_len_to_f64(len: Self::SafeLen) -> f64 {
        len as f64
    }
    fn f64_to_safe_len(f: f64) -> Self::SafeLen {
        f as Self::SafeLen
    }
    fn add_len_less_one(a: Self, b: Self::SafeLen) -> Self {
        a + (b - 1) as Self
    }
    fn sub_len_less_one(a: Self, b: Self::SafeLen) -> Self {
        a - (b - 1) as Self
    }
}

impl Integer for u8 {
    #[cfg(target_pointer_width = "32")]
    type SafeLen = usize;
    #[cfg(target_pointer_width = "64")]
    type SafeLen = usize;

    fn safe_len(r: &RangeInclusive<Self>) -> <Self as Integer>::SafeLen {
        r.end().overflowing_sub(*r.start()).0 as <Self as Integer>::SafeLen + 1
    }

    fn safe_len_to_f64(len: Self::SafeLen) -> f64 {
        len as f64
    }
    fn f64_to_safe_len(f: f64) -> Self::SafeLen {
        f as Self::SafeLen
    }
    fn add_len_less_one(a: Self, b: Self::SafeLen) -> Self {
        a + (b - 1) as Self
    }
    fn sub_len_less_one(a: Self, b: Self::SafeLen) -> Self {
        a - (b - 1) as Self
    }
}

// create_const_array!(i32, create_constant_i32);
// // create_const_array!(u32, create_constant_u32);

// // #[allow(dead_code)] // cmk
// const EXPECTED_I32X16: i32x16 = unsafe { std::mem::transmute(create_constant_i32::<16>()) };

// // #[allow(dead_code)] // cmk
// // const EXPECTED_U32X16: u32x16 = unsafe { std::mem::transmute(create_constant_u32::<16>()) };
const EXPECTED_I32X16: Simd<i32, 16> =
    unsafe { std::mem::transmute([-15, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]) };

const EXPECTED_U32X16: u32x16 =
    unsafe { std::mem::transmute([-15, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]) };

// #[allow(dead_code)] // cmk
// const EXPECTED_I32X8: i32x8 = unsafe { std::mem::transmute([-7, 1, 1, 1, 1, 1, 1, 1]) };

// #[allow(dead_code)] // cmk
// const EXPECTED_U32X8: u32x8 = unsafe { std::mem::transmute([-7, 1, 1, 1, 1, 1, 1, 1]) };

// #[allow(dead_code)] // cmk
// const EXPECTED_I32X4: Simd<i32, 4> = unsafe { std::mem::transmute([-3, 1, 1, 1]) };

// #[allow(dead_code)] // cmk
// const EXPECTED_U32X4: u32x4 = unsafe { std::mem::transmute([-3, 1, 1, 1]) };

impl Integer for i32 {
    #[cfg(target_pointer_width = "32")]
    type SafeLen = u64;
    #[cfg(target_pointer_width = "64")]
    type SafeLen = usize;

    fn safe_len(r: &RangeInclusive<Self>) -> <Self as Integer>::SafeLen {
        r.end().overflowing_sub(*r.start()).0 as u32 as <Self as Integer>::SafeLen + 1
    }

    fn safe_len_to_f64(len: Self::SafeLen) -> f64 {
        len as f64
    }
    fn f64_to_safe_len(f: f64) -> Self::SafeLen {
        f as Self::SafeLen
    }
    fn add_len_less_one(a: Self, b: Self::SafeLen) -> Self {
        a + (b - 1) as Self
    }
    fn sub_len_less_one(a: Self, b: Self::SafeLen) -> Self {
        a - (b - 1) as Self
    }

    // avx512f or scalar
    #[cfg(any(
        target_feature = "avx512f",
        all(not(target_feature = "sse2"), not(target_feature = "avx2"))
    ))]
    is_consecutive!(i32, 16, EXPECTED_I32X16);

    // avx2 (256 bits)
    #[cfg(all(target_feature = "avx2", not(target_feature = "avx512f")))]
    is_consecutive!(i32x8, EXPECTED_I32X8);

    // sse2 (128 bits)
    #[cfg(all(target_feature = "sse2", not(target_feature = "avx2")))]
    is_consecutive!(i32x4, EXPECTED_I32X4);
}

impl Integer for u32 {
    #[cfg(target_pointer_width = "32")]
    type SafeLen = u64;
    #[cfg(target_pointer_width = "64")]
    type SafeLen = usize;

    // avx512f or scalar
    #[cfg(any(
        target_feature = "avx512f",
        all(not(target_feature = "sse2"), not(target_feature = "avx2"))
    ))]
    is_consecutive!(u32, 16, EXPECTED_U32X16);

    // avx2 (256 bits)
    #[cfg(all(target_feature = "avx2", not(target_feature = "avx512f")))]
    is_consecutive!(u32x8, EXPECTED_U32X8);

    // sse2 (128 bits)
    #[cfg(all(target_feature = "sse2", not(target_feature = "avx2")))]
    is_consecutive!(u32x4, EXPECTED_U32X4);

    fn safe_len(r: &RangeInclusive<Self>) -> <Self as Integer>::SafeLen {
        r.end().overflowing_sub(*r.start()).0 as <Self as Integer>::SafeLen + 1
    }

    fn safe_len_to_f64(len: Self::SafeLen) -> f64 {
        len as f64
    }
    fn f64_to_safe_len(f: f64) -> Self::SafeLen {
        f as Self::SafeLen
    }
    fn add_len_less_one(a: Self, b: Self::SafeLen) -> Self {
        a + (b - 1) as Self
    }
    fn sub_len_less_one(a: Self, b: Self::SafeLen) -> Self {
        a - (b - 1) as Self
    }
}

impl Integer for i64 {
    #[cfg(target_pointer_width = "32")]
    type SafeLen = u128;
    #[cfg(target_pointer_width = "64")]
    type SafeLen = u128;

    fn safe_len(r: &RangeInclusive<Self>) -> <Self as Integer>::SafeLen {
        r.end().overflowing_sub(*r.start()).0 as u64 as <Self as Integer>::SafeLen + 1
    }
    fn safe_len_to_f64(len: Self::SafeLen) -> f64 {
        len as f64
    }
    fn f64_to_safe_len(f: f64) -> Self::SafeLen {
        f as Self::SafeLen
    }
    fn add_len_less_one(a: Self, b: Self::SafeLen) -> Self {
        a + (b - 1) as Self
    }
    fn sub_len_less_one(a: Self, b: Self::SafeLen) -> Self {
        a - (b - 1) as Self
    }
}

impl Integer for u64 {
    #[cfg(target_pointer_width = "32")]
    type SafeLen = u128;
    #[cfg(target_pointer_width = "64")]
    type SafeLen = u128;

    fn safe_len(r: &RangeInclusive<Self>) -> <Self as Integer>::SafeLen {
        r.end().overflowing_sub(*r.start()).0 as <Self as Integer>::SafeLen + 1
    }
    fn safe_len_to_f64(len: Self::SafeLen) -> f64 {
        len as f64
    }
    fn f64_to_safe_len(f: f64) -> Self::SafeLen {
        f as Self::SafeLen
    }
    fn add_len_less_one(a: Self, b: Self::SafeLen) -> Self {
        a + (b - 1) as Self
    }
    fn sub_len_less_one(a: Self, b: Self::SafeLen) -> Self {
        a - (b - 1) as Self
    }
}

impl Integer for i128 {
    #[cfg(target_pointer_width = "32")]
    type SafeLen = u128;
    #[cfg(target_pointer_width = "64")]
    type SafeLen = u128;

    fn safe_len(r: &RangeInclusive<Self>) -> <Self as Integer>::SafeLen {
        r.end().overflowing_sub(*r.start()).0 as u128 as <Self as Integer>::SafeLen + 1
    }
    fn safe_max_value() -> Self {
        Self::max_value() - 1
    }

    fn safe_len_to_f64(len: Self::SafeLen) -> f64 {
        len as f64
    }
    fn f64_to_safe_len(f: f64) -> Self::SafeLen {
        f as Self::SafeLen
    }
    fn add_len_less_one(a: Self, b: Self::SafeLen) -> Self {
        a + (b - 1) as Self
    }
    fn sub_len_less_one(a: Self, b: Self::SafeLen) -> Self {
        a - (b - 1) as Self
    }
}

impl Integer for u128 {
    #[cfg(target_pointer_width = "32")]
    type SafeLen = u128;
    #[cfg(target_pointer_width = "64")]
    type SafeLen = u128;

    fn safe_len(r: &RangeInclusive<Self>) -> <Self as Integer>::SafeLen {
        r.end().overflowing_sub(*r.start()).0 as <Self as Integer>::SafeLen + 1
    }
    fn safe_max_value() -> Self {
        Self::max_value() - 1
    }
    fn safe_len_to_f64(len: Self::SafeLen) -> f64 {
        len as f64
    }
    fn f64_to_safe_len(f: f64) -> Self::SafeLen {
        f as Self::SafeLen
    }
    fn add_len_less_one(a: Self, b: Self::SafeLen) -> Self {
        a + (b - 1) as Self
    }
    fn sub_len_less_one(a: Self, b: Self::SafeLen) -> Self {
        a - (b - 1) as Self
    }
}

impl Integer for isize {
    #[cfg(target_pointer_width = "32")]
    type SafeLen = u64;
    #[cfg(target_pointer_width = "64")]
    type SafeLen = u128;

    fn safe_len(r: &RangeInclusive<Self>) -> <Self as Integer>::SafeLen {
        r.end().overflowing_sub(*r.start()).0 as usize as <Self as Integer>::SafeLen + 1
    }

    fn safe_len_to_f64(len: Self::SafeLen) -> f64 {
        len as f64
    }
    fn f64_to_safe_len(f: f64) -> Self::SafeLen {
        f as Self::SafeLen
    }
    fn add_len_less_one(a: Self, b: Self::SafeLen) -> Self {
        a + (b - 1) as Self
    }
    fn sub_len_less_one(a: Self, b: Self::SafeLen) -> Self {
        a - (b - 1) as Self
    }
}

impl Integer for usize {
    #[cfg(target_pointer_width = "32")]
    type SafeLen = u64;
    #[cfg(target_pointer_width = "64")]
    type SafeLen = u128;

    fn safe_len(r: &RangeInclusive<Self>) -> <Self as Integer>::SafeLen {
        r.end().overflowing_sub(*r.start()).0 as <Self as Integer>::SafeLen + 1
    }

    fn safe_len_to_f64(len: Self::SafeLen) -> f64 {
        len as f64
    }
    fn f64_to_safe_len(f: f64) -> Self::SafeLen {
        f as Self::SafeLen
    }
    fn add_len_less_one(a: Self, b: Self::SafeLen) -> Self {
        a + (b - 1) as Self
    }
    fn sub_len_less_one(a: Self, b: Self::SafeLen) -> Self {
        a - (b - 1) as Self
    }
}

impl Integer for i16 {
    #[cfg(target_pointer_width = "32")]
    type SafeLen = usize;
    #[cfg(target_pointer_width = "64")]
    type SafeLen = usize;

    fn safe_len(r: &RangeInclusive<Self>) -> <Self as Integer>::SafeLen {
        r.end().overflowing_sub(*r.start()).0 as u16 as <Self as Integer>::SafeLen + 1
    }

    fn safe_len_to_f64(len: Self::SafeLen) -> f64 {
        len as f64
    }
    fn f64_to_safe_len(f: f64) -> Self::SafeLen {
        f as Self::SafeLen
    }
    fn add_len_less_one(a: Self, b: Self::SafeLen) -> Self {
        a + (b - 1) as Self
    }
    fn sub_len_less_one(a: Self, b: Self::SafeLen) -> Self {
        a - (b - 1) as Self
    }
}

impl Integer for u16 {
    #[cfg(target_pointer_width = "32")]
    type SafeLen = usize;
    #[cfg(target_pointer_width = "64")]
    type SafeLen = usize;

    fn safe_len(r: &RangeInclusive<Self>) -> <Self as Integer>::SafeLen {
        r.end().overflowing_sub(*r.start()).0 as <Self as Integer>::SafeLen + 1
    }

    fn safe_len_to_f64(len: Self::SafeLen) -> f64 {
        len as f64
    }
    fn f64_to_safe_len(f: f64) -> Self::SafeLen {
        f as Self::SafeLen
    }
    fn add_len_less_one(a: Self, b: Self::SafeLen) -> Self {
        a + (b - 1) as Self
    }
    fn sub_len_less_one(a: Self, b: Self::SafeLen) -> Self {
        a - (b - 1) as Self
    }
}

// cmk: Rule: Look at the docs in a way that lets you see every useful command (how?)
// cmk: Rule: You have to use nightly, so not usefull. (how to turn on for just one project)
// cmk: Rule: As soon as you think about SIMD algorithms, you'll likely make non-faster
// cmk: Rule: Set up for multiple levels of support
// cmk  Rule: AMD 512 might be slower than Intel (but maybe not for permutations)
// cmk  Rule: Docs: https://doc.rust-lang.org/nightly/std/simd/index.html
// cmk: Rule: Docs: more https://doc.rust-lang.org/nightly/std/simd/struct.Simd.html
// cmk  Tighter clippy, etc.
// cmk look at Rust meet up photos, including way to get alignment
// cmk Rule: Expect operations to wrap. Unlike scalar it is the default.
// cmk Rule: Use #[inline] on functions that take a SIMD input and return a SIMD output (see docs)
// cmk0 Rule: It's generally OK to use the read "unaligned" on aligned. There is no penalty. (cmk test this)
// cmk Rule: Useful: https://github.com/rust-lang/portable-simd/blob/master/beginners-guide.md (talks about reduce_and, etc)
// cmk Rule: Do const values like ... https://rust-lang.zulipchat.com/#narrow/stream/122651-general/topic/const.20SIMD.20values
// cmk Rule: Use SIMD rust command even without SIMD.
// cmk Rule: Use unsafe where you need to.
