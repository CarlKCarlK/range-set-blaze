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

impl<T> Default for UIntPlusOne<T>
where
    T: UInt,
{
    fn default() -> Self {
        UIntPlusOne::UInt(T::zero())
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
                    debug_assert!(wrapped_less1 == zero, "overflow");
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
            (UIntPlusOne::UInt(a), UIntPlusOne::UInt(b)) => {
                debug_assert!(a >= b, "UIntPlusOne::UInt - UIntPlusOne::UInt");
                UIntPlusOne::UInt(a - b)
            }
            (UIntPlusOne::UInt(_), UIntPlusOne::MaxPlusOne) => {
                debug_assert!(false, "UIntPlusOne::UInt - UIntPlusOne::Max");
                UIntPlusOne::UInt(zero)
            }
            (UIntPlusOne::MaxPlusOne, UIntPlusOne::MaxPlusOne) => UIntPlusOne::UInt(zero),
            (UIntPlusOne::MaxPlusOne, UIntPlusOne::UInt(v)) => UIntPlusOne::UInt(max - (v - one)),
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
                if a.overflowing_mul(&(b - one)).1 {
                    debug_assert!(false, "overflow");
                    UIntPlusOne::MaxPlusOne
                } else {
                    UIntPlusOne::UInt(a * (b - one) + a) + rhs
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
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match (self, other) {
            (UIntPlusOne::MaxPlusOne, UIntPlusOne::MaxPlusOne) => Some(std::cmp::Ordering::Equal),
            (UIntPlusOne::MaxPlusOne, _) => Some(std::cmp::Ordering::Greater),
            (_, UIntPlusOne::MaxPlusOne) => Some(std::cmp::Ordering::Less),
            (UIntPlusOne::UInt(a), UIntPlusOne::UInt(b)) => a.partial_cmp(b),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use quickcheck::Gen;
    use quickcheck::{Arbitrary, QuickCheck};
    use quickcheck_macros::quickcheck;
    use rand::Rng;
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

    #[derive(Clone, Copy, Debug)]
    struct SmallU16(u16);

    impl Arbitrary for SmallU16 {
        fn arbitrary(g: &mut Gen) -> SmallU16 {
            let value = *g.choose(&(0u16..=256).collect::<Vec<_>>()).unwrap();
            SmallU16(value)
        }
    }

    fn add_em(a: u16, b: u16) -> bool {
        let a_p1 = u16_to_p1(a);
        let b_p1 = u16_to_p1(b);

        let sum = panic::catch_unwind(AssertUnwindSafe(|| {
            let sum = a + b;
            assert!(sum <= 256, "overflow");
            sum
        }));
        let sum_actual = panic::catch_unwind(AssertUnwindSafe(|| a_p1 + b_p1));
        println!("{:?}, {:?}", sum, sum_actual);

        match (sum, sum_actual) {
            (Ok(sum), Ok(sum_p1)) => u16_to_p1(sum) == sum_p1,
            (Err(_), Err(_)) => true,
            _ => false,
        }
    }

    #[test]
    fn cmk_remove() {
        assert!(add_em(110, 179));
    }

    #[quickcheck]
    fn test_addition_equivalence(a: SmallU16, b: SmallU16) -> bool {
        add_em(a.0, b.0)
    }
}
