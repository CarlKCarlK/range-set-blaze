use core::fmt::Display;
use core::ops::{Add, AddAssign, Mul, SubAssign};

use num_traits::One;
pub(crate) const TWO_POW_128: f64 = 340_282_366_920_938_463_463_374_607_431_768_211_456.0;

// cmk
#[derive(Debug, Clone, Copy, PartialEq, Eq, Ord, Hash)]
pub enum U128PlusOne {
    U128(u128),
    MaxPlusOne,
}

impl Default for U128PlusOne {
    fn default() -> Self {
        U128PlusOne::U128(0)
    }
}

impl Display for U128PlusOne {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            U128PlusOne::U128(v) => write!(f, "{}", v),
            U128PlusOne::MaxPlusOne => write!(f, "(u128::MAX + 1"),
        }
    }
}

impl num_traits::Zero for U128PlusOne {
    fn zero() -> Self {
        U128PlusOne::U128(0)
    }

    fn is_zero(&self) -> bool {
        matches!(self, U128PlusOne::U128(0))
    }
}

impl Add for U128PlusOne {
    type Output = Self;

    fn add(self, rhs: Self) -> Self {
        match (self, rhs) {
            (U128PlusOne::U128(0), b) | (b, U128PlusOne::U128(0)) => b,
            (U128PlusOne::U128(a), U128PlusOne::U128(b)) => {
                let less1 = a - 1 + b;
                if less1 == u128::MAX {
                    U128PlusOne::MaxPlusOne
                } else {
                    U128PlusOne::U128(less1 + 1)
                }
            }
            (U128PlusOne::MaxPlusOne, U128PlusOne::U128(v))
            | (U128PlusOne::U128(v), U128PlusOne::MaxPlusOne) => {
                debug_assert!(v == 0);
                U128PlusOne::MaxPlusOne
            }
            (U128PlusOne::MaxPlusOne, U128PlusOne::MaxPlusOne) => {
                debug_assert!(false, "U128PlusOne::Max + U128PlusOne::Max");
                U128PlusOne::MaxPlusOne
            }
        }
    }
}

impl SubAssign for U128PlusOne {
    // cmk000000 check for overflow
    fn sub_assign(&mut self, rhs: Self) {
        *self = match (*self, rhs) {
            (U128PlusOne::U128(_), U128PlusOne::MaxPlusOne) => {
                debug_assert!(false, "U128PlusOne::U128 - U128PlusOne::Max");
                U128PlusOne::U128(0)
            }
            (U128PlusOne::U128(a), U128PlusOne::U128(b)) => {
                debug_assert!(a >= b, "U128PlusOne::U128 - U128PlusOne::U128");
                U128PlusOne::U128(a - b)
            }
            (U128PlusOne::MaxPlusOne, U128PlusOne::MaxPlusOne) => U128PlusOne::U128(0),
            (U128PlusOne::MaxPlusOne, U128PlusOne::U128(0)) => U128PlusOne::MaxPlusOne,
            (U128PlusOne::MaxPlusOne, U128PlusOne::U128(v)) => {
                U128PlusOne::U128(u128::MAX - (v - 1))
            }
        }
    }
}

impl AddAssign for U128PlusOne {
    fn add_assign(&mut self, rhs: Self) {
        *self = self.add(rhs);
    }
}

impl One for U128PlusOne {
    fn one() -> Self {
        U128PlusOne::U128(1)
    }
}

impl Mul for U128PlusOne {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self {
        let left_f64 = match self {
            U128PlusOne::U128(v) => v as f64,
            U128PlusOne::MaxPlusOne => TWO_POW_128,
        };
        let right_f64 = match rhs {
            U128PlusOne::U128(v) => v as f64,
            U128PlusOne::MaxPlusOne => TWO_POW_128,
        };
        let product = left_f64 * right_f64;
        if product >= TWO_POW_128 {
            U128PlusOne::MaxPlusOne
        } else {
            U128PlusOne::U128(product as u128)
        }
    }
}

impl PartialOrd for U128PlusOne {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match (self, other) {
            (U128PlusOne::MaxPlusOne, U128PlusOne::MaxPlusOne) => Some(std::cmp::Ordering::Equal),
            (U128PlusOne::MaxPlusOne, _) => Some(std::cmp::Ordering::Greater),
            (_, U128PlusOne::MaxPlusOne) => Some(std::cmp::Ordering::Less),
            (U128PlusOne::U128(a), U128PlusOne::U128(b)) => a.partial_cmp(b),
        }
    }
}
