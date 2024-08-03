use core::ops::RangeInclusive;

use crate::Integer;

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
}

impl Integer for u32 {
    #[cfg(target_pointer_width = "32")]
    type SafeLen = u64;
    #[cfg(target_pointer_width = "64")]
    type SafeLen = usize;

    fn safe_len(r: &RangeInclusive<Self>) -> <Self as Integer>::SafeLen {
        r.end().overflowing_sub(*r.start()).0 as <Self as Integer>::SafeLen + 1
    }
    // fn f64_to_t(f: f64) -> Self {
    //     f as Self
    // }
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
        Self::MAX - 1
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
        Self::MAX - 1
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
