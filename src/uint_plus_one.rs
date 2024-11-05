use alloc::fmt;
use core::cmp::Ordering;
use core::fmt::Display;
use core::mem;
use core::ops::{Add, AddAssign, Mul, Sub, SubAssign};
#[cfg(not(feature = "std"))]
#[allow(unused_imports)] // cmk
use num_traits::float::FloatCore;
use num_traits::ops::overflowing::{OverflowingAdd, OverflowingMul};

pub trait UInt:
    num_traits::Zero
    + num_traits::One
    + num_traits::Unsigned
    + OverflowingAdd
    + num_traits::Bounded
    + Sub<Output = Self>
    + PartialOrd
    + Copy
    + Sized
    + OverflowingMul
{
}

// u128 and u8 are UInt.
// We define u8 for testing purposes.
impl UInt for u128 {}
impl UInt for u8 {}

/// Represents values from `0` to `u128::MAX + 1` (inclusive).
///
/// Needed to represent every possible length of a `RangeInclusive<i128>` and `RangeInclusive<u128>`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum UIntPlusOne<T>
where
    T: UInt,
{
    /// cmk
    UInt(T),
    /// cmk
    MaxPlusOne,
}

impl<T> UIntPlusOne<T>
where
    T: UInt,
{
    /// cmk
    #[allow(clippy::cast_possible_truncation)]
    #[allow(clippy::cast_possible_wrap)]
    #[must_use]
    pub fn max_plus_one_as_f64() -> f64 {
        2.0f64.powi((mem::size_of::<T>() * 8) as i32)
    }
}

impl<T> Display for UIntPlusOne<T>
where
    T: UInt + Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UInt(v) => write!(f, "{v}"),
            Self::MaxPlusOne => write!(f, "(u128::MAX + 1"),
        }
    }
}

impl<T> num_traits::Zero for UIntPlusOne<T>
where
    T: UInt,
{
    fn zero() -> Self {
        Self::UInt(T::zero())
    }

    fn is_zero(&self) -> bool {
        matches!(self, Self::UInt(v) if v.is_zero())
    }
}

impl<T> Add for UIntPlusOne<T>
where
    T: UInt,
{
    type Output = Self;

    fn add(self, rhs: Self) -> Self {
        let zero = T::zero();
        let one: T = T::one();
        let max: T = T::max_value();

        match (self, rhs) {
            (Self::UInt(z), b) | (b, Self::UInt(z)) if z == zero => b,
            (Self::UInt(a), Self::UInt(b)) => {
                let (wrapped_less1, overflow) = a.overflowing_add(&(b - one));
                if overflow {
                    debug_assert!(false, "overflow");
                    Self::MaxPlusOne
                } else if wrapped_less1 == max {
                    Self::MaxPlusOne
                } else {
                    Self::UInt(wrapped_less1 + T::one())
                }
            }
            (Self::MaxPlusOne, _) | (_, Self::MaxPlusOne) => {
                debug_assert!(false, "UIntPlusOne::Max + something more than 1");
                Self::MaxPlusOne
            }
        }
    }
}

impl<T> SubAssign for UIntPlusOne<T>
where
    T: UInt,
{
    fn sub_assign(&mut self, rhs: Self) {
        let zero = T::zero();
        let one: T = T::one();
        let max: T = T::max_value();

        *self = match (*self, rhs) {
            (Self::UInt(a), Self::UInt(b)) => Self::UInt(a - b),
            (Self::MaxPlusOne, Self::UInt(z)) if z == zero => Self::MaxPlusOne,
            (Self::MaxPlusOne, Self::UInt(v)) => Self::UInt(max - (v - one)),
            (Self::MaxPlusOne, Self::MaxPlusOne) => Self::UInt(zero),
            (Self::UInt(_), Self::MaxPlusOne) => {
                debug_assert!(false, "UIntPlusOne::UInt - UIntPlusOne::Max");
                Self::UInt(zero)
            }
        }
    }
}

impl<T> AddAssign for UIntPlusOne<T>
where
    T: UInt,
{
    fn add_assign(&mut self, rhs: Self) {
        *self = self.add(rhs);
    }
}

impl<T> num_traits::One for UIntPlusOne<T>
where
    T: UInt,
{
    fn one() -> Self {
        Self::UInt(T::one())
    }
}

impl<T> Mul for UIntPlusOne<T>
where
    T: UInt,
{
    type Output = Self;

    fn mul(self, rhs: Self) -> Self {
        let zero = T::zero();
        let one: T = T::one();

        match (self, rhs) {
            (Self::UInt(o1), b) | (b, Self::UInt(o1)) if o1 == one => b,
            (Self::UInt(z), _) | (_, Self::UInt(z)) if z == zero => Self::UInt(zero),
            (Self::UInt(a), Self::UInt(b)) => {
                let (a_times_b_less1, overflow) = a.overflowing_mul(&(b - one));
                if overflow {
                    debug_assert!(false, "overflow");
                    Self::MaxPlusOne
                } else {
                    Self::UInt(a_times_b_less1) + self
                }
            }
            (Self::MaxPlusOne, _) | (_, Self::MaxPlusOne) => {
                debug_assert!(false, "UIntPlusOne::Max * something more than 1");
                Self::MaxPlusOne
            }
        }
    }
}

impl<T> PartialOrd for UIntPlusOne<T>
where
    T: UInt,
{
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match (self, other) {
            (Self::MaxPlusOne, Self::MaxPlusOne) => Some(Ordering::Equal),
            (Self::MaxPlusOne, _) => Some(Ordering::Greater),
            (_, Self::MaxPlusOne) => Some(Ordering::Less),
            (Self::UInt(a), Self::UInt(b)) => a.partial_cmp(b),
        }
    }
}

#[cfg(feature = "std")]
#[cfg(test)]
mod tests {
    use super::*;
    use std::panic;
    use std::panic::AssertUnwindSafe;
    use std::prelude::v1::*;
    use std::println;

    #[allow(clippy::cast_possible_truncation)]
    const fn u16_to_p1(v: u16) -> UIntPlusOne<u8> {
        if v == 256 {
            UIntPlusOne::MaxPlusOne
        } else {
            UIntPlusOne::UInt(v as u8)
        }
    }

    #[cfg(not(target_arch = "wasm32"))] // This tests panics, so it's not suitable for wasm32.
    fn add_em(a: u16, b: u16) -> bool {
        let a_p1 = u16_to_p1(a);
        let b_p1 = u16_to_p1(b);

        let c = panic::catch_unwind(AssertUnwindSafe(|| {
            let c = a + b;
            assert!(c <= 256, "overflow");
            c
        }));
        let c_actual = panic::catch_unwind(AssertUnwindSafe(|| a_p1 + b_p1));
        println!("cmk {c:?}, {c_actual:?}");

        match (c, c_actual) {
            (Ok(c), Ok(c_p1)) => u16_to_p1(c) == c_p1,
            (Err(_), Err(_)) => true,
            _ => false,
        }
    }

    #[cfg(not(target_arch = "wasm32"))] // This tests panics, so it's not suitable for wasm32.
    fn mul_em(a: u16, b: u16) -> bool {
        let a_p1 = u16_to_p1(a);
        let b_p1 = u16_to_p1(b);

        let c = panic::catch_unwind(AssertUnwindSafe(|| {
            let c = a * b;
            assert!(c <= 256, "overflow");
            c
        }));
        let c_actual = panic::catch_unwind(AssertUnwindSafe(|| a_p1 * b_p1));
        println!("cmk {c:?}, {c_actual:?}");

        match (c, c_actual) {
            (Ok(c), Ok(c_p1)) => u16_to_p1(c) == c_p1,
            (Err(_), Err(_)) => true,
            _ => false,
        }
    }

    #[cfg(not(target_arch = "wasm32"))] // This tests panics, so it's not suitable for wasm32.
    fn sub_em(a: u16, b: u16) -> bool {
        let a_p1 = u16_to_p1(a);
        let b_p1 = u16_to_p1(b);

        let c = panic::catch_unwind(AssertUnwindSafe(|| {
            let mut c = a;
            c -= b;
            assert!(c <= 256, "overflow");
            c
        }));
        let c_actual = panic::catch_unwind(AssertUnwindSafe(|| {
            let mut c_actual = a_p1;
            c_actual -= b_p1;
            c_actual
        }));
        println!("cmk {c:?}, {c_actual:?}");

        match (c, c_actual) {
            (Ok(c), Ok(c_p1)) => u16_to_p1(c) == c_p1,
            (Err(_), Err(_)) => true,
            _ => false,
        }
    }

    #[cfg(not(target_arch = "wasm32"))] // This tests panics, so it's not suitable for wasm32.
    fn compare_em(a: u16, b: u16) -> bool {
        let a_p1 = u16_to_p1(a);
        let b_p1 = u16_to_p1(b);

        let c = panic::catch_unwind(AssertUnwindSafe(|| a.partial_cmp(&b)));
        let c_actual = panic::catch_unwind(AssertUnwindSafe(|| a_p1.partial_cmp(&b_p1)));
        println!("cmk {c:?}, {c_actual:?}");

        match (c, c_actual) {
            (Ok(Some(c)), Ok(Some(c_p1))) => c == c_p1,
            _ => panic!("never happens"),
        }
    }

    #[cfg(not(target_arch = "wasm32"))] // This tests panics, so it's not suitable for wasm32.
    #[test]
    fn test_add_equivalence() {
        for a in 0..=256 {
            for b in 0..=256 {
                assert!(add_em(a, b), "a: {a}, b: {b}");
            }
        }
    }

    #[cfg(not(target_arch = "wasm32"))] // This tests panics, so it's not suitable for wasm32.
    #[test]
    fn test_mul_equivalence() {
        for a in 0..=256 {
            for b in 0..=256 {
                assert!(mul_em(a, b), "a: {a}, b: {b}");
            }
        }
    }

    #[cfg(not(target_arch = "wasm32"))] // This tests panics, so it's not suitable for wasm32.
    #[test]
    fn test_sub_equivalence() {
        for a in 0..=256 {
            for b in 0..=256 {
                assert!(sub_em(a, b), "a: {a}, b: {b}");
            }
        }
    }

    #[cfg(not(target_arch = "wasm32"))] // This tests panics, so it's not suitable for wasm32.
    #[test]
    fn test_compare_equivalence() {
        for a in 0..=256 {
            for b in 0..=256 {
                assert!(compare_em(a, b), "a: {a}, b: {b}");
            }
        }
    }

    #[cfg(not(target_arch = "wasm32"))] // This tests panics, so it's not suitable for wasm32.
    #[test]
    fn test_add_assign() {
        let mut a = UIntPlusOne::<u128>::UInt(1);
        a += UIntPlusOne::UInt(1);
        assert_eq!(a, UIntPlusOne::UInt(2));
    }
}
