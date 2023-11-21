use crate::{from_slice_iter::FromSliceIter, RangeSetBlaze};

use core::ops::RangeInclusive;
use core::simd::prelude::*;
use core::simd::{LaneCount, SupportedLaneCount};
use std::mem::size_of;

// cmk Rule may want to skip sse2 (128) because it is slower than the non-simd version

use crate::Integer;

macro_rules! from_slice_etc {
    ($expected:ident) => {
        // cmk Rule: const expressions are handy.
        // Note: Does the right thing for isize, usize
        // cmk5 Look for other uses of const expressions
        // cmk Rule: Making this inline reduced time from 146 to 92

        // avx512 (512 bits) or scalar
        #[cfg(any(target_feature = "avx512f", not(target_feature = "avx2")))]
        #[inline]
        fn from_slice(slice: &[Self]) -> RangeSetBlaze<Self> {
            FromSliceIter::<Self, { 64 / size_of::<Self>() }>::new(slice, &$expected).collect()
        }
        // avx2 (256 bits)
        // cmk0 shouldn't this be 32 and have a transmute?
        #[cfg(all(target_feature = "avx2", not(target_feature = "avx512f")))]
        #[inline]
        fn from_slice(slice: &[Self]) -> RangeSetBlaze<Self> {
            FromSliceIter::<Self, { 32 / size_of::<Self>() }>::new(slice, &$expected).collect()
        }
    };
}

macro_rules! expected_simd {
    ($const:ident, $function:ident, $type:ty, $n512:tt, $n128:tt) => {
        #[inline]
        const fn $function<const N: usize>() -> Simd<$type, N>
        where
            LaneCount<N>: SupportedLaneCount,
        {
            let mut arr: [$type; N] = [1; N];
            arr[0] = !(N as $type) + 2;
            Simd::from_array(arr)
        }

        // avx512 (512 bits) or scalar
        #[cfg(any(target_feature = "avx512f", not(target_feature = "avx2")))]
        const $const: Simd<$type, $n512> = $function::<$n512>();
        // avx2 (256 bits)
        #[cfg(all(target_feature = "avx2", not(target_feature = "avx512f")))]
        const $const: Simd<$type, $n128> = $function::<$n128>();
    };
}

expected_simd!(EXPECTED_I8, expected_i8, i8, 64, 32);
expected_simd!(EXPECTED_U8, expected_u8, u8, 64, 32);
expected_simd!(EXPECTED_I16, expected_i16, i16, 32, 16);
expected_simd!(EXPECTED_U16, expected_u16, u16, 32, 16);
expected_simd!(EXPECTED_I32, expected_i32, i32, 16, 8);
expected_simd!(EXPECTED_U32, expected_u32, u32, 16, 8);
expected_simd!(EXPECTED_I64, expected_i64, i64, 8, 4);
expected_simd!(EXPECTED_U64, expected_u64, u64, 8, 4);
#[cfg(target_pointer_width = "64")]
expected_simd!(EXPECTED_ISIZE, expected_isize, isize, 8, 4);
#[cfg(target_pointer_width = "64")]
expected_simd!(EXPECTED_USIZE, expected_usize, usize, 8, 4);
#[cfg(target_pointer_width = "32")]
expected_simd!(EXPECTED_ISIZE, expected_isize, isize, 4, 2);
#[cfg(target_pointer_width = "32")]
expected_simd!(EXPECTED_USIZE, expected_usize, usize, 4, 2);

impl Integer for i8 {
    #[cfg(target_pointer_width = "32")]
    type SafeLen = usize;
    #[cfg(target_pointer_width = "64")]
    type SafeLen = usize;

    from_slice_etc!(EXPECTED_I8);

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

    from_slice_etc!(EXPECTED_U8);

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

    from_slice_etc!(EXPECTED_I32);
}

impl Integer for u32 {
    #[cfg(target_pointer_width = "32")]
    type SafeLen = u64;
    #[cfg(target_pointer_width = "64")]
    type SafeLen = usize;

    from_slice_etc!(EXPECTED_U32);

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

    from_slice_etc!(EXPECTED_I64);

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

    from_slice_etc!(EXPECTED_U64);

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

    #[inline]
    fn from_slice(slice: &[Self]) -> crate::RangeSetBlaze<Self> {
        return slice.iter().collect();
    }

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

    #[inline]
    fn from_slice(slice: &[Self]) -> crate::RangeSetBlaze<Self> {
        return slice.iter().collect();
    }

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

    from_slice_etc!(EXPECTED_ISIZE);

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

    from_slice_etc!(EXPECTED_USIZE);

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

    from_slice_etc!(EXPECTED_I16);

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

    from_slice_etc!(EXPECTED_U16);

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

// cmk Rule: Look at the docs in a way that lets you see every useful command (how?)
// cmk Rule: You have to use nightly, so not usefull. (how to turn on for just one project)
// cmk Rule: As soon as you think about SIMD algorithms, you'll likely make non-faster
// cmk Rule: Set up for multiple levels of support
// cmk Rule: AMD 512 might be slower than Intel (but maybe not for permutations)
// cmk Rule: Docs: https://doc.rust-lang.org/nightly/std/simd/index.html
// cmk Rule: Docs: more https://doc.rust-lang.org/nightly/std/simd/struct.Simd.html
// cmk5 Tighter clippy, etc.
// cmk Rule: Expect operations to wrap. Unlike scalar it is the default.
// cmk Rule: Use #[inline] on functions that take a SIMD input and return a SIMD output (see docs)
// cmk Rule: It's generally OK to use the read "unaligned" on aligned. There is no penalty. (see https://doc.rust-lang.org/std/simd/struct.Simd.html#safe-simd-with-unsafe-rust)
// cmk Rule: Useful: https://github.com/rust-lang/portable-simd/blob/master/beginners-guide.md (talks about reduce_and, etc)
// cmk Rule: Do const values like ... https://rust-lang.zulipchat.com/#narrow/stream/122651-general/topic/const.20SIMD.20values
// cmk Rule: Use SIMD rust command even without SIMD.
// cmk Rule: Use unsafe where you need to.

#[allow(unused_macros)]
macro_rules! check_simd {
    ($simd:expr) => {{
        let length = $simd.lanes();
        let t_bytes = std::mem::size_of_val(&$simd) / length;
        // cmk0 what about 256bit SIMD?
        assert_eq!(length, 64 / t_bytes);
        assert_eq!($simd[0] as i32, -((length as i32) - 1));
        for &val in $simd.as_array().iter().skip(1) {
            assert_eq!(val, 1);
        }
    }};
}

// cmk Rule: Test your constants
#[test]
fn check_simd_constants() {
    check_simd!(EXPECTED_I8);
    // check_simd!(EXPECTED_U8);
    check_simd!(EXPECTED_I16);
    // check_simd!(EXPECTED_U16);
    check_simd!(EXPECTED_I32);
    // check_simd!(EXPECTED_U32);
    check_simd!(EXPECTED_I64);
    // check_simd!(EXPECTED_U64);
    check_simd!(EXPECTED_ISIZE);
    // check_simd!(EXPECTED_USIZE);
}
