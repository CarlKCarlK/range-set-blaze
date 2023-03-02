use std::ops::RangeInclusive;

use crate::Integer;

impl Integer for i8 {
    #[cfg(target_pointer_width = "16")]
    type Output = usize;
    #[cfg(target_pointer_width = "32")]
    type Output = usize;
    #[cfg(target_pointer_width = "64")]
    type Output = usize;
    fn safe_subtract_inclusive(range_inclusive: RangeInclusive<Self>) -> <Self as Integer>::Output {
        let (b, a) = range_inclusive.into_inner(); // !!!cmk00 rename to end, start
        a.overflowing_sub(b).0 as u8 as <Self as Integer>::Output + 1
    }
    fn max_value2() -> Self {
        Self::max_value()
    }
}

impl Integer for u8 {
    #[cfg(target_pointer_width = "16")]
    type Output = usize;
    #[cfg(target_pointer_width = "32")]
    type Output = usize;
    #[cfg(target_pointer_width = "64")]
    type Output = usize;
    fn safe_subtract_inclusive(range_inclusive: RangeInclusive<Self>) -> <Self as Integer>::Output {
        let (b, a) = range_inclusive.into_inner();
        a.overflowing_sub(b).0 as <Self as Integer>::Output + 1
    }
    fn max_value2() -> Self {
        Self::max_value()
    }
}

impl Integer for i32 {
    #[cfg(target_pointer_width = "16")]
    type Output = u64;
    #[cfg(target_pointer_width = "32")]
    type Output = u64;
    #[cfg(target_pointer_width = "64")]
    type Output = usize;
    fn safe_subtract_inclusive(range_inclusive: RangeInclusive<Self>) -> <Self as Integer>::Output {
        let (b, a) = range_inclusive.into_inner();
        a.overflowing_sub(b).0 as u32 as <Self as Integer>::Output + 1
    }
    fn max_value2() -> Self {
        Self::max_value()
    }
}

impl Integer for u32 {
    #[cfg(target_pointer_width = "16")]
    type Output = u64;
    #[cfg(target_pointer_width = "32")]
    type Output = u64;
    #[cfg(target_pointer_width = "64")]
    type Output = usize;
    fn safe_subtract_inclusive(range_inclusive: RangeInclusive<Self>) -> <Self as Integer>::Output {
        let (b, a) = range_inclusive.into_inner();
        a.overflowing_sub(b).0 as <Self as Integer>::Output + 1
    }
    fn max_value2() -> Self {
        Self::max_value()
    }
}

impl Integer for i64 {
    #[cfg(target_pointer_width = "16")]
    type Output = u128;
    #[cfg(target_pointer_width = "32")]
    type Output = u128;
    #[cfg(target_pointer_width = "64")]
    type Output = u128;
    fn safe_subtract_inclusive(range_inclusive: RangeInclusive<Self>) -> <Self as Integer>::Output {
        let (b, a) = range_inclusive.into_inner();
        a.overflowing_sub(b).0 as u64 as <Self as Integer>::Output + 1
    }
    fn max_value2() -> Self {
        Self::max_value()
    }
}

impl Integer for u64 {
    #[cfg(target_pointer_width = "16")]
    type Output = u128;
    #[cfg(target_pointer_width = "32")]
    type Output = u128;
    #[cfg(target_pointer_width = "64")]
    type Output = u128;
    fn safe_subtract_inclusive(range_inclusive: RangeInclusive<Self>) -> <Self as Integer>::Output {
        let (b, a) = range_inclusive.into_inner();
        a.overflowing_sub(b).0 as <Self as Integer>::Output + 1
    }
    fn max_value2() -> Self {
        Self::max_value()
    }
}

impl Integer for i128 {
    #[cfg(target_pointer_width = "16")]
    type Output = u128;
    #[cfg(target_pointer_width = "32")]
    type Output = u128;
    #[cfg(target_pointer_width = "64")]
    type Output = u128;
    fn safe_subtract_inclusive(range_inclusive: RangeInclusive<Self>) -> <Self as Integer>::Output {
        let (b, a) = range_inclusive.into_inner();
        a.overflowing_sub(b).0 as u128 as <Self as Integer>::Output + 1
    }
    fn max_value2() -> Self {
        Self::max_value() - 1
    }
}

impl Integer for u128 {
    #[cfg(target_pointer_width = "16")]
    type Output = u128;
    #[cfg(target_pointer_width = "32")]
    type Output = u128;
    #[cfg(target_pointer_width = "64")]
    type Output = u128;
    fn safe_subtract_inclusive(range_inclusive: RangeInclusive<Self>) -> <Self as Integer>::Output {
        let (b, a) = range_inclusive.into_inner();
        a.overflowing_sub(b).0 as <Self as Integer>::Output + 1
    }
    fn max_value2() -> Self {
        Self::max_value() - 1
    }
}

impl Integer for isize {
    #[cfg(target_pointer_width = "16")]
    type Output = u32;
    #[cfg(target_pointer_width = "32")]
    type Output = u64;
    #[cfg(target_pointer_width = "64")]
    type Output = u128;
    fn safe_subtract_inclusive(range_inclusive: RangeInclusive<Self>) -> <Self as Integer>::Output {
        let (b, a) = range_inclusive.into_inner();
        a.overflowing_sub(b).0 as usize as <Self as Integer>::Output + 1
    }
    fn max_value2() -> Self {
        Self::max_value()
    }
}

impl Integer for usize {
    #[cfg(target_pointer_width = "16")]
    type Output = u32;
    #[cfg(target_pointer_width = "32")]
    type Output = u64;
    #[cfg(target_pointer_width = "64")]
    type Output = u128;
    fn safe_subtract_inclusive(range_inclusive: RangeInclusive<Self>) -> <Self as Integer>::Output {
        let (b, a) = range_inclusive.into_inner();
        a.overflowing_sub(b).0 as <Self as Integer>::Output + 1
    }
    fn max_value2() -> Self {
        Self::max_value()
    }
}

impl Integer for i16 {
    #[cfg(target_pointer_width = "16")]
    type Output = u32;
    #[cfg(target_pointer_width = "32")]
    type Output = usize;
    #[cfg(target_pointer_width = "64")]
    type Output = usize;
    fn safe_subtract_inclusive(range_inclusive: RangeInclusive<Self>) -> <Self as Integer>::Output {
        let (b, a) = range_inclusive.into_inner();
        a.overflowing_sub(b).0 as u16 as <Self as Integer>::Output + 1
    }
    fn max_value2() -> Self {
        Self::max_value()
    }
}

impl Integer for u16 {
    #[cfg(target_pointer_width = "16")]
    type Output = u32;
    #[cfg(target_pointer_width = "32")]
    type Output = usize;
    #[cfg(target_pointer_width = "64")]
    type Output = usize;
    fn safe_subtract_inclusive(range_inclusive: RangeInclusive<Self>) -> <Self as Integer>::Output {
        let (b, a) = range_inclusive.into_inner();
        a.overflowing_sub(b).0 as <Self as Integer>::Output + 1
    }
    fn max_value2() -> Self {
        Self::max_value()
    }
}
