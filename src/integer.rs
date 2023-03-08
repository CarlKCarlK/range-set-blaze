use std::ops::RangeInclusive;

use crate::Integer;

impl Integer for i8 {
    #[cfg(target_pointer_width = "16")]
    type Output = usize;
    #[cfg(target_pointer_width = "32")]
    type Output = usize;
    #[cfg(target_pointer_width = "64")]
    type SafeLen = usize;
    fn safe_inclusive_len(r: &RangeInclusive<Self>) -> <Self as Integer>::SafeLen {
        r.end().overflowing_sub(*r.start()).0 as u8 as <Self as Integer>::SafeLen + 1
    }
    fn from_f64(f: f64) -> Self {
        f as i8
    }
    fn into_f64(len: Self::SafeLen) -> f64 {
        len as f64
    }
}

impl Integer for u8 {
    #[cfg(target_pointer_width = "16")]
    type Output = usize;
    #[cfg(target_pointer_width = "32")]
    type Output = usize;
    #[cfg(target_pointer_width = "64")]
    type SafeLen = usize;
    fn safe_inclusive_len(r: &RangeInclusive<Self>) -> <Self as Integer>::SafeLen {
        r.end().overflowing_sub(*r.start()).0 as <Self as Integer>::SafeLen + 1
    }
    fn from_f64(f: f64) -> Self {
        f as u8
    }
    fn into_f64(len: Self::SafeLen) -> f64 {
        len as f64
    }
}

impl Integer for i32 {
    #[cfg(target_pointer_width = "16")]
    type Output = u64;
    #[cfg(target_pointer_width = "32")]
    type Output = u64;
    #[cfg(target_pointer_width = "64")]
    type SafeLen = usize;
    fn safe_inclusive_len(r: &RangeInclusive<Self>) -> <Self as Integer>::SafeLen {
        r.end().overflowing_sub(*r.start()).0 as u32 as <Self as Integer>::SafeLen + 1
    }
    fn from_f64(f: f64) -> Self {
        f as i32
    }
    fn into_f64(len: Self::SafeLen) -> f64 {
        len as f64
    }
}

impl Integer for u32 {
    #[cfg(target_pointer_width = "16")]
    type Output = u64;
    #[cfg(target_pointer_width = "32")]
    type Output = u64;
    #[cfg(target_pointer_width = "64")]
    type SafeLen = usize;
    fn safe_inclusive_len(r: &RangeInclusive<Self>) -> <Self as Integer>::SafeLen {
        r.end().overflowing_sub(*r.start()).0 as <Self as Integer>::SafeLen + 1
    }
    fn from_f64(f: f64) -> Self {
        f as u32
    }
    fn into_f64(len: Self::SafeLen) -> f64 {
        len as f64
    }
}

impl Integer for i64 {
    #[cfg(target_pointer_width = "16")]
    type Output = u128;
    #[cfg(target_pointer_width = "32")]
    type Output = u128;
    #[cfg(target_pointer_width = "64")]
    type SafeLen = u128;
    fn safe_inclusive_len(r: &RangeInclusive<Self>) -> <Self as Integer>::SafeLen {
        r.end().overflowing_sub(*r.start()).0 as u64 as <Self as Integer>::SafeLen + 1
    }
    fn from_f64(f: f64) -> Self {
        f as i64
    }
    fn into_f64(len: Self::SafeLen) -> f64 {
        len as f64
    }
}

impl Integer for u64 {
    #[cfg(target_pointer_width = "16")]
    type Output = u128;
    #[cfg(target_pointer_width = "32")]
    type Output = u128;
    #[cfg(target_pointer_width = "64")]
    type SafeLen = u128;
    fn safe_inclusive_len(r: &RangeInclusive<Self>) -> <Self as Integer>::SafeLen {
        r.end().overflowing_sub(*r.start()).0 as <Self as Integer>::SafeLen + 1
    }
    fn from_f64(f: f64) -> Self {
        f as u64
    }
    fn into_f64(len: Self::SafeLen) -> f64 {
        len as f64
    }
}

impl Integer for i128 {
    #[cfg(target_pointer_width = "16")]
    type Output = u128;
    #[cfg(target_pointer_width = "32")]
    type Output = u128;
    #[cfg(target_pointer_width = "64")]
    type SafeLen = u128;
    fn safe_inclusive_len(r: &RangeInclusive<Self>) -> <Self as Integer>::SafeLen {
        r.end().overflowing_sub(*r.start()).0 as u128 as <Self as Integer>::SafeLen + 1
    }
    fn max_value2() -> Self {
        Self::max_value() - 1
    }
    fn from_f64(f: f64) -> Self {
        f as i128
    }
    fn into_f64(len: Self::SafeLen) -> f64 {
        len as f64
    }
}

impl Integer for u128 {
    #[cfg(target_pointer_width = "16")]
    type Output = u128;
    #[cfg(target_pointer_width = "32")]
    type Output = u128;
    #[cfg(target_pointer_width = "64")]
    type SafeLen = u128;
    fn safe_inclusive_len(r: &RangeInclusive<Self>) -> <Self as Integer>::SafeLen {
        r.end().overflowing_sub(*r.start()).0 as <Self as Integer>::SafeLen + 1
    }
    fn max_value2() -> Self {
        Self::max_value() - 1
    }
    fn from_f64(f: f64) -> Self {
        f as u128
    }
    fn into_f64(len: Self::SafeLen) -> f64 {
        len as f64
    }
}

impl Integer for isize {
    #[cfg(target_pointer_width = "16")]
    type Output = u32;
    #[cfg(target_pointer_width = "32")]
    type Output = u64;
    #[cfg(target_pointer_width = "64")]
    type SafeLen = u128;
    fn safe_inclusive_len(r: &RangeInclusive<Self>) -> <Self as Integer>::SafeLen {
        r.end().overflowing_sub(*r.start()).0 as usize as <Self as Integer>::SafeLen + 1
    }
    fn from_f64(f: f64) -> Self {
        f as isize
    }
    fn into_f64(len: Self::SafeLen) -> f64 {
        len as f64
    }
}

impl Integer for usize {
    #[cfg(target_pointer_width = "16")]
    type Output = u32;
    #[cfg(target_pointer_width = "32")]
    type Output = u64;
    #[cfg(target_pointer_width = "64")]
    type SafeLen = u128;
    fn safe_inclusive_len(r: &RangeInclusive<Self>) -> <Self as Integer>::SafeLen {
        r.end().overflowing_sub(*r.start()).0 as <Self as Integer>::SafeLen + 1
    }
    fn from_f64(f: f64) -> Self {
        f as usize
    }
    fn into_f64(len: Self::SafeLen) -> f64 {
        len as f64
    }
}

impl Integer for i16 {
    #[cfg(target_pointer_width = "16")]
    type Output = u32;
    #[cfg(target_pointer_width = "32")]
    type Output = usize;
    #[cfg(target_pointer_width = "64")]
    type SafeLen = usize;
    fn safe_inclusive_len(r: &RangeInclusive<Self>) -> <Self as Integer>::SafeLen {
        r.end().overflowing_sub(*r.start()).0 as u16 as <Self as Integer>::SafeLen + 1
    }
    fn from_f64(f: f64) -> Self {
        f as i16
    }
    fn into_f64(len: Self::SafeLen) -> f64 {
        len as f64
    }
}

impl Integer for u16 {
    #[cfg(target_pointer_width = "16")]
    type Output = u32;
    #[cfg(target_pointer_width = "32")]
    type Output = usize;
    #[cfg(target_pointer_width = "64")]
    type SafeLen = usize;
    fn safe_inclusive_len(r: &RangeInclusive<Self>) -> <Self as Integer>::SafeLen {
        r.end().overflowing_sub(*r.start()).0 as <Self as Integer>::SafeLen + 1
    }
    fn from_f64(f: f64) -> Self {
        f as u16
    }
    fn into_f64(len: Self::SafeLen) -> f64 {
        len as f64
    }
}
