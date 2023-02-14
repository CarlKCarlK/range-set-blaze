use crate::SafeSubtract;

impl SafeSubtract for i8 {
    #[cfg(target_pointer_width = "16")]
    type Output = usize;
    #[cfg(target_pointer_width = "32")]
    type Output = usize;
    #[cfg(target_pointer_width = "64")]
    type Output = usize;
    fn safe_subtract(end: Self, start: Self) -> <Self as SafeSubtract>::Output {
        end.overflowing_sub(start).0 as u8 as <Self as SafeSubtract>::Output
    }
    fn safe_subtract_inclusive(a: Self, b: Self) -> <Self as SafeSubtract>::Output {
        a.overflowing_sub(b).0 as u8 as <Self as SafeSubtract>::Output + 1
    }
    fn max_value2() -> Self {
        Self::max_value()
    }
}

impl SafeSubtract for u8 {
    #[cfg(target_pointer_width = "16")]
    type Output = usize;
    #[cfg(target_pointer_width = "32")]
    type Output = usize;
    #[cfg(target_pointer_width = "64")]
    type Output = usize;
    fn safe_subtract(end: Self, start: Self) -> <Self as SafeSubtract>::Output {
        end.overflowing_sub(start).0 as <Self as SafeSubtract>::Output
    }
    fn safe_subtract_inclusive(a: Self, b: Self) -> <Self as SafeSubtract>::Output {
        a.overflowing_sub(b).0 as <Self as SafeSubtract>::Output + 1
    }
    fn max_value2() -> Self {
        Self::max_value()
    }
}

impl SafeSubtract for i32 {
    #[cfg(target_pointer_width = "16")]
    type Output = u64;
    #[cfg(target_pointer_width = "32")]
    type Output = u64;
    #[cfg(target_pointer_width = "64")]
    type Output = usize;
    fn safe_subtract(end: Self, start: Self) -> <Self as SafeSubtract>::Output {
        end.overflowing_sub(start).0 as u32 as <Self as SafeSubtract>::Output
    }
    fn safe_subtract_inclusive(a: Self, b: Self) -> <Self as SafeSubtract>::Output {
        a.overflowing_sub(b).0 as u32 as <Self as SafeSubtract>::Output + 1
    }
    fn max_value2() -> Self {
        Self::max_value()
    }
}

impl SafeSubtract for u32 {
    #[cfg(target_pointer_width = "16")]
    type Output = u64;
    #[cfg(target_pointer_width = "32")]
    type Output = u64;
    #[cfg(target_pointer_width = "64")]
    type Output = usize;
    fn safe_subtract(end: Self, start: Self) -> <Self as SafeSubtract>::Output {
        end.overflowing_sub(start).0 as <Self as SafeSubtract>::Output
    }
    fn safe_subtract_inclusive(a: Self, b: Self) -> <Self as SafeSubtract>::Output {
        a.overflowing_sub(b).0 as <Self as SafeSubtract>::Output + 1
    }
    fn max_value2() -> Self {
        Self::max_value()
    }
}

impl SafeSubtract for i64 {
    #[cfg(target_pointer_width = "16")]
    type Output = u128;
    #[cfg(target_pointer_width = "32")]
    type Output = u128;
    #[cfg(target_pointer_width = "64")]
    type Output = u128;
    fn safe_subtract(end: Self, start: Self) -> <Self as SafeSubtract>::Output {
        end.overflowing_sub(start).0 as u64 as <Self as SafeSubtract>::Output
    }
    fn safe_subtract_inclusive(a: Self, b: Self) -> <Self as SafeSubtract>::Output {
        a.overflowing_sub(b).0 as u64 as <Self as SafeSubtract>::Output + 1
    }
    fn max_value2() -> Self {
        Self::max_value()
    }
}

impl SafeSubtract for u64 {
    #[cfg(target_pointer_width = "16")]
    type Output = u128;
    #[cfg(target_pointer_width = "32")]
    type Output = u128;
    #[cfg(target_pointer_width = "64")]
    type Output = u128;
    fn safe_subtract(end: Self, start: Self) -> <Self as SafeSubtract>::Output {
        end.overflowing_sub(start).0 as <Self as SafeSubtract>::Output
    }
    fn safe_subtract_inclusive(a: Self, b: Self) -> <Self as SafeSubtract>::Output {
        a.overflowing_sub(b).0 as <Self as SafeSubtract>::Output + 1
    }
    fn max_value2() -> Self {
        Self::max_value()
    }
}

impl SafeSubtract for i128 {
    #[cfg(target_pointer_width = "16")]
    type Output = u128;
    #[cfg(target_pointer_width = "32")]
    type Output = u128;
    #[cfg(target_pointer_width = "64")]
    type Output = u128;
    fn safe_subtract(end: Self, start: Self) -> <Self as SafeSubtract>::Output {
        end.overflowing_sub(start).0 as u128 as <Self as SafeSubtract>::Output
    }
    fn safe_subtract_inclusive(a: Self, b: Self) -> <Self as SafeSubtract>::Output {
        a.overflowing_sub(b).0 as u128 as <Self as SafeSubtract>::Output + 1
    }
    fn max_value2() -> Self {
        Self::max_value() - 1
    }
}

impl SafeSubtract for u128 {
    #[cfg(target_pointer_width = "16")]
    type Output = u128;
    #[cfg(target_pointer_width = "32")]
    type Output = u128;
    #[cfg(target_pointer_width = "64")]
    type Output = u128;
    fn safe_subtract(end: Self, start: Self) -> <Self as SafeSubtract>::Output {
        end.overflowing_sub(start).0 as <Self as SafeSubtract>::Output
    }
    fn safe_subtract_inclusive(a: Self, b: Self) -> <Self as SafeSubtract>::Output {
        a.overflowing_sub(b).0 as <Self as SafeSubtract>::Output + 1
    }
    fn max_value2() -> Self {
        Self::max_value() - 1
    }
}

impl SafeSubtract for isize {
    #[cfg(target_pointer_width = "16")]
    type Output = u32;
    #[cfg(target_pointer_width = "32")]
    type Output = u64;
    #[cfg(target_pointer_width = "64")]
    type Output = u128;
    fn safe_subtract(end: Self, start: Self) -> <Self as SafeSubtract>::Output {
        end.overflowing_sub(start).0 as usize as <Self as SafeSubtract>::Output
    }
    fn safe_subtract_inclusive(a: Self, b: Self) -> <Self as SafeSubtract>::Output {
        a.overflowing_sub(b).0 as usize as <Self as SafeSubtract>::Output + 1
    }
    fn max_value2() -> Self {
        Self::max_value()
    }
}

impl SafeSubtract for usize {
    #[cfg(target_pointer_width = "16")]
    type Output = u32;
    #[cfg(target_pointer_width = "32")]
    type Output = u64;
    #[cfg(target_pointer_width = "64")]
    type Output = u128;
    fn safe_subtract(end: Self, start: Self) -> <Self as SafeSubtract>::Output {
        end.overflowing_sub(start).0 as <Self as SafeSubtract>::Output
    }
    fn safe_subtract_inclusive(a: Self, b: Self) -> <Self as SafeSubtract>::Output {
        a.overflowing_sub(b).0 as <Self as SafeSubtract>::Output + 1
    }
    fn max_value2() -> Self {
        Self::max_value()
    }
}

impl SafeSubtract for i16 {
    #[cfg(target_pointer_width = "16")]
    type Output = u32;
    #[cfg(target_pointer_width = "32")]
    type Output = usize;
    #[cfg(target_pointer_width = "64")]
    type Output = usize;
    fn safe_subtract(end: Self, start: Self) -> <Self as SafeSubtract>::Output {
        end.overflowing_sub(start).0 as u16 as <Self as SafeSubtract>::Output
    }
    fn safe_subtract_inclusive(a: Self, b: Self) -> <Self as SafeSubtract>::Output {
        a.overflowing_sub(b).0 as u16 as <Self as SafeSubtract>::Output + 1
    }
    fn max_value2() -> Self {
        Self::max_value()
    }
}

impl SafeSubtract for u16 {
    #[cfg(target_pointer_width = "16")]
    type Output = u32;
    #[cfg(target_pointer_width = "32")]
    type Output = usize;
    #[cfg(target_pointer_width = "64")]
    type Output = usize;
    fn safe_subtract(end: Self, start: Self) -> <Self as SafeSubtract>::Output {
        end.overflowing_sub(start).0 as <Self as SafeSubtract>::Output
    }
    fn safe_subtract_inclusive(a: Self, b: Self) -> <Self as SafeSubtract>::Output {
        a.overflowing_sub(b).0 as <Self as SafeSubtract>::Output + 1
    }
    fn max_value2() -> Self {
        Self::max_value()
    }
}
