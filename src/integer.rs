use core::mem::align_of;
use core::mem::size_of;
use core::ops::RangeInclusive;
#[cfg(target_feature = "avx512f")]
use core::simd::i32x16;
#[cfg(all(target_feature = "sse2", not(target_feature = "avx2")))]
use core::simd::i32x4;
#[cfg(all(target_feature = "avx2", not(target_feature = "avx512f")))]
use core::simd::i32x8;
#[cfg(target_feature = "avx512f")]
use core::simd::u32x16;
// cmk may want to turn this off because it is slower than the non-simd version
#[cfg(all(target_feature = "sse2", not(target_feature = "avx2")))]
use core::simd::u32x4;
#[cfg(all(target_feature = "avx2", not(target_feature = "avx512f")))]
use core::simd::u32x8;
use std::simd::prelude::*; // cmk use? when?

use crate::Integer;

macro_rules! is_consecutive_etc {
    ($scalar:ty, $simd:ty, $decrease:expr) => {
        fn is_consecutive(chunk: &[$scalar]) -> bool {
            debug_assert!(chunk.len() == <$simd>::LANES, "Chunk is wrong length");
            debug_assert!(
                // cmk is there a more built in way to do this?
                chunk.as_ptr() as usize % align_of::<$simd>() == 0,
                "Chunk is not aligned"
            );

            // const LAST_INDEX: usize = <$simd>::LANES - 1;
            // let (expected, overflowed) = chunk[0].overflowing_add(LAST_INDEX as $scalar);
            // if overflowed || expected != chunk[LAST_INDEX] {
            //     return false;
            // }

            // cmk should do with from_slice_uncheck unsafe????
            let a = <$simd>::from_slice(chunk) + $decrease;
            // cmk is a[0] the best way to extract an element?
            // cmk is a[0] and then splat the best way to create a simd from one element?
            // cmk is eq the best way to compare two simds?
            let b = <$simd>::splat(a[0]); // cmk seems same as simd_swizzle!
                                          // cmk let b = simd_swizzle!(a, [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
            a == b
        }
        fn bit_size_and_offset(slice: &[Self]) -> (usize, usize) {
            {
                let alignment = align_of::<$simd>();
                let misalignment = (slice.as_ptr() as usize) % alignment;

                // return bit_size, offset
                (
                    size_of::<$simd>() * 8,
                    (alignment - misalignment) / size_of::<$scalar>(),
                )
            }
        }
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

// cmk should it be called $simd or $simd_type? etc

// #[allow(unused_macros)]
// macro_rules! init_decrease_simd {
//     ($name:ident, $el_type:ty, $simd_type:ty) => {
//         lazy_static! {
//             static ref $name: $simd_type = {
//                 // cmk rename "temp"
//                 let mut temp = <$simd_type>::splat(0); // cmk use simd_zero! or new???
//                 for i in 0..<$simd_type>::LANES {
//                     temp[i] = (!(i as $el_type)).wrapping_add(1);
//                 }
//                 temp
//             };
//         }
//     };
// }

// cmk this allows us to add the negative which seems faster
// cmk is there a way to extract $el_type from $simd_type?

#[cfg(target_feature = "avx512f")]
// init_decrease_simd!(DECREASE_I32, i32, i32x16);
const DECREASE_I32: i32x16 = unsafe {
    std::mem::transmute([
        0i32, -1, -2, -3, -4, -5, -6, -7, -8, -9, -10, -11, -12, -13, -14, -15,
    ])
};

#[cfg(target_feature = "avx512f")]
// init_decrease_simd!(DECREASE_U32, u32, u32x16);
const DECREASE_U32: u32x16 = unsafe {
    std::mem::transmute([
        0i32, -1, -2, -3, -4, -5, -6, -7, -8, -9, -10, -11, -12, -13, -14, -15,
    ])
};

#[cfg(all(target_feature = "avx2", not(target_feature = "avx512f")))]
init_decrease_simd!(DECREASE_I32, i32, i32x8);

#[cfg(all(target_feature = "avx2", not(target_feature = "avx512f")))]
init_decrease_simd!(DECREASE_U32, u32, u32x8);

#[cfg(all(target_feature = "sse2", not(target_feature = "avx2")))]
init_decrease_simd!(DECREASE_I32, i32, i32x4);

#[cfg(all(target_feature = "sse2", not(target_feature = "avx2")))]
init_decrease_simd!(DECREASE_U32, u32, u32x4);

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

    #[cfg(target_feature = "avx512f")]
    is_consecutive_etc!(i32, i32x16, DECREASE_I32);

    #[cfg(all(target_feature = "avx2", not(target_feature = "avx512f")))]
    is_consecutive_etc!(i32, i32x8, *DECREASE_I32);

    #[cfg(all(target_feature = "sse2", not(target_feature = "avx2")))]
    is_consecutive_etc!(i32, i32x4, *DECREASE_I32);
}

impl Integer for u32 {
    #[cfg(target_pointer_width = "32")]
    type SafeLen = u64;
    #[cfg(target_pointer_width = "64")]
    type SafeLen = usize;

    #[cfg(target_feature = "avx512f")]
    is_consecutive_etc!(u32, u32x16, DECREASE_U32);

    #[cfg(all(target_feature = "avx2", not(target_feature = "avx512f")))]
    is_consecutive_etc!(u32, u32x8, *DECREASE_U32);

    #[cfg(all(target_feature = "sse2", not(target_feature = "avx2")))]
    is_consecutive_etc!(u32, u32x4, *DECREASE_U32);

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
// cmk  Tigher clippy, etc.
// cmk look at Rust meet up photos, including way to get alignment
// cmk Rule: Expect operations to wrap. Unlike scalar it is the default.
// cmk Rule: Use #[inline] on functions that take a SIMD input and return a SIMD output (see docs)
// cmk Rule: It's generally OK to use the read "unaligned" on aligned. There is no penalty. (cmk test this)
// cmk Rule: Useful: https://github.com/rust-lang/portable-simd/blob/master/beginners-guide.md (talks about reduce_and, etc)
// cmk Rule: Do const values like ... https://rust-lang.zulipchat.com/#narrow/stream/122651-general/topic/const.20SIMD.20values
