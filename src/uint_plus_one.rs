use core::cmp::Ordering;
use core::fmt::Display;
use core::mem;
use core::ops::{Add, AddAssign, Mul, Sub, SubAssign};

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

// u128 and u8 are UInt
impl UInt for u128 {}
impl UInt for u8 {}

/// cmk
#[derive(Debug, Clone, Copy, PartialEq, Eq, Ord, Hash)]
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
    pub fn max_plus_one_as_f64() -> f64 {
        2.0f64.powi((mem::size_of::<T>() * 8) as i32)
    }
}

impl<T> Display for UIntPlusOne<T>
where
    T: UInt + Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UIntPlusOne::UInt(v) => write!(f, "{}", v),
            UIntPlusOne::MaxPlusOne => write!(f, "(u128::MAX + 1"),
        }
    }
}

impl<T> num_traits::Zero for UIntPlusOne<T>
where
    T: UInt,
{
    fn zero() -> Self {
        UIntPlusOne::UInt(T::zero())
    }

    fn is_zero(&self) -> bool {
        matches!(self, UIntPlusOne::UInt(v) if v.is_zero())
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
            (UIntPlusOne::UInt(z), b) | (b, UIntPlusOne::UInt(z)) if z == zero => b,
            (UIntPlusOne::UInt(a), UIntPlusOne::UInt(b)) => {
                let (wrapped_less1, overflow) = a.overflowing_add(&(b - one));
                if overflow {
                    debug_assert!(false, "overflow");
                    UIntPlusOne::MaxPlusOne
                } else if wrapped_less1 == max {
                    UIntPlusOne::MaxPlusOne
                } else {
                    UIntPlusOne::UInt(wrapped_less1 + T::one())
                }
            }
            (UIntPlusOne::MaxPlusOne, _) | (_, UIntPlusOne::MaxPlusOne) => {
                debug_assert!(false, "UIntPlusOne::Max + something more than 1");
                UIntPlusOne::MaxPlusOne
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
            (UIntPlusOne::UInt(a), UIntPlusOne::UInt(b)) => UIntPlusOne::UInt(a - b),
            (UIntPlusOne::MaxPlusOne, UIntPlusOne::UInt(z)) if z == zero => UIntPlusOne::MaxPlusOne,
            (UIntPlusOne::MaxPlusOne, UIntPlusOne::UInt(v)) => UIntPlusOne::UInt(max - (v - one)),
            (UIntPlusOne::MaxPlusOne, UIntPlusOne::MaxPlusOne) => UIntPlusOne::UInt(zero),
            (UIntPlusOne::UInt(_), UIntPlusOne::MaxPlusOne) => {
                debug_assert!(false, "UIntPlusOne::UInt - UIntPlusOne::Max");
                UIntPlusOne::UInt(zero)
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
        UIntPlusOne::UInt(T::one())
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
            (UIntPlusOne::UInt(o1), b) | (b, UIntPlusOne::UInt(o1)) if o1 == one => b,
            (UIntPlusOne::UInt(z), _) | (_, UIntPlusOne::UInt(z)) if z == zero => {
                UIntPlusOne::UInt(zero)
            }
            (UIntPlusOne::UInt(a), UIntPlusOne::UInt(b)) => {
                let (a_times_b_less1, overflow) = a.overflowing_mul(&(b - one));
                if overflow {
                    debug_assert!(false, "overflow");
                    UIntPlusOne::MaxPlusOne
                } else {
                    UIntPlusOne::UInt(a_times_b_less1) + self
                }
            }
            (UIntPlusOne::MaxPlusOne, _) | (_, UIntPlusOne::MaxPlusOne) => {
                debug_assert!(false, "UIntPlusOne::Max * something more than 1");
                UIntPlusOne::MaxPlusOne
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
            (UIntPlusOne::MaxPlusOne, UIntPlusOne::MaxPlusOne) => Some(Ordering::Equal),
            (UIntPlusOne::MaxPlusOne, _) => Some(Ordering::Greater),
            (_, UIntPlusOne::MaxPlusOne) => Some(Ordering::Less),
            (UIntPlusOne::UInt(a), UIntPlusOne::UInt(b)) => a.partial_cmp(b),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::panic;
    use std::panic::catch_unwind;
    use std::panic::AssertUnwindSafe;

    fn u16_to_p1(v: u16) -> UIntPlusOne<u8> {
        if v == 256 {
            UIntPlusOne::MaxPlusOne
        } else {
            UIntPlusOne::UInt(v as u8)
        }
    }

    fn add_em(a: u16, b: u16) -> bool {
        let a_p1 = u16_to_p1(a);
        let b_p1 = u16_to_p1(b);

        let c = panic::catch_unwind(AssertUnwindSafe(|| {
            let c = a + b;
            assert!(c <= 256, "overflow");
            c
        }));
        let c_actual = panic::catch_unwind(AssertUnwindSafe(|| a_p1 + b_p1));
        println!("cmk {:?}, {:?}", c, c_actual);

        match (c, c_actual) {
            (Ok(c), Ok(c_p1)) => u16_to_p1(c) == c_p1,
            (Err(_), Err(_)) => true,
            _ => false,
        }
    }

    fn mul_em(a: u16, b: u16) -> bool {
        let a_p1 = u16_to_p1(a);
        let b_p1 = u16_to_p1(b);

        let c = panic::catch_unwind(AssertUnwindSafe(|| {
            let c = a * b;
            assert!(c <= 256, "overflow");
            c
        }));
        let c_actual = panic::catch_unwind(AssertUnwindSafe(|| a_p1 * b_p1));
        println!("cmk {:?}, {:?}", c, c_actual);

        match (c, c_actual) {
            (Ok(c), Ok(c_p1)) => u16_to_p1(c) == c_p1,
            (Err(_), Err(_)) => true,
            _ => false,
        }
    }

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
        println!("cmk {:?}, {:?}", c, c_actual);

        match (c, c_actual) {
            (Ok(c), Ok(c_p1)) => u16_to_p1(c) == c_p1,
            (Err(_), Err(_)) => true,
            _ => false,
        }
    }

    fn compare_em(a: u16, b: u16) -> bool {
        let a_p1 = u16_to_p1(a);
        let b_p1 = u16_to_p1(b);

        let c = panic::catch_unwind(AssertUnwindSafe(|| a.partial_cmp(&b)));
        let c_actual = panic::catch_unwind(AssertUnwindSafe(|| a_p1.partial_cmp(&b_p1)));
        println!("cmk {:?}, {:?}", c, c_actual);

        match (c, c_actual) {
            (Ok(Some(c)), Ok(Some(c_p1))) => c == c_p1,
            _ => panic!("never happens"),
        }
    }

    #[test]
    fn cmk_remove() {
        assert!(sub_em(256, 0));
    }

    #[test]
    fn test_add_equivalence() {
        for a in 0..=256 {
            for b in 0..=256 {
                assert!(add_em(a, b), "a: {}, b: {}", a, b);
            }
        }
    }

    #[test]
    fn test_mul_equivalence() {
        for a in 0..=256 {
            for b in 0..=256 {
                assert!(mul_em(a, b), "a: {}, b: {}", a, b);
            }
        }
    }

    #[test]
    fn test_sub_equivalence() {
        for a in 0..=256 {
            for b in 0..=256 {
                assert!(sub_em(a, b), "a: {}, b: {}", a, b);
            }
        }
    }

    #[test]
    fn test_compare_equivalence() {
        for a in 0..=256 {
            for b in 0..=256 {
                assert!(compare_em(a, b), "a: {}, b: {}", a, b);
            }
        }
    }

    #[test]
    fn test_add_assign() {
        let mut a = UIntPlusOne::<u128>::UInt(1);
        a += UIntPlusOne::UInt(1);
        assert_eq!(a, UIntPlusOne::UInt(2));
    }
}
