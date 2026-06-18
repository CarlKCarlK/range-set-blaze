//! Ordered `f64` support.

use core::cmp::Ordering;
use core::hash::{Hash, Hasher};
use core::ops::{Add, AddAssign, Mul, RangeInclusive, SubAssign};

use num_traits::{One, Zero};

/// A transparent wrapper around [`f64`] with total ordering.
///
/// Comparison, equality, and hashing all agree with [`f64::total_cmp`].
#[repr(transparent)]
#[derive(Copy, Clone, Default, Debug)]
pub struct F64(pub f64);

impl F64 {
    /// The minimum value in [`f64::total_cmp`] order.
    pub const MIN: Self = Self(f64::from_bits(u64::MAX));

    /// The maximum value in [`f64::total_cmp`] order.
    pub const MAX: Self = Self(f64::from_bits(0x7fff_ffff_ffff_ffff));

    /// The most negative finite [`f64`] value.
    pub const MIN_VALUE: Self = Self(f64::MIN);

    /// The most positive finite [`f64`] value.
    pub const MAX_VALUE: Self = Self(f64::MAX);

    /// A [`RangeInclusive`] including the finite values of the type.
    /// This is what the complement of an empty set ought to be, but tragically cannot.
    pub const ALL_VALUES: core::ops::RangeInclusive<Self> = Self::MIN_VALUE..=Self::MAX_VALUE;

    /// new zero value
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns the wrapped [`f64`] value.
    #[must_use]
    pub const fn into_inner(self) -> f64 {
        self.0
    }

    /// Transforms the float bits into the monotonically ordered `i64` space used by `total_cmp`.
    pub(crate) const fn to_ordered_i64(self) -> i64 {
        let mut bits = self.0.to_bits().cast_signed();
        bits ^= ((bits >> 63).cast_unsigned() >> 1).cast_signed();
        bits
    }

    /// Transforms the ordered `i64` space back into standard float bits.
    pub(crate) const fn from_ordered_i64(mut bits: i64) -> Self {
        // Reversing the XOR transformation
        bits ^= ((bits >> 63).cast_unsigned() >> 1).cast_signed();
        Self(f64::from_bits(bits.cast_unsigned()))
    }

    /// Returns the next float in total order.
    ///
    /// Panics on overflow if `self` is the maximum value in total order.
    #[must_use]
    pub const fn next(self) -> Self {
        let ordered = self.to_ordered_i64();
        Self::from_ordered_i64(ordered + 1)
    }

    /// Returns the previous float in total order.
    ///
    /// Panics on overflow if `self` is the minimum value in total order.
    #[must_use]
    pub const fn prev(self) -> Self {
        let ordered = self.to_ordered_i64();
        Self::from_ordered_i64(ordered - 1)
    }

    /// Returns the next float in total order.
    ///
    /// Returns [`None`] if `self` is the maximum value in total order.
    #[must_use]
    pub const fn checked_next(self) -> Option<Self> {
        let ordered = self.to_ordered_i64();
        match ordered.checked_add(1) {
            Some(next_ordered) => Some(Self::from_ordered_i64(next_ordered)),
            None => None,
        }
    }

    /// Returns the previous float in total order.
    ///
    /// Returns [`None`] if `self` is the minimum value in total order.
    #[must_use]
    pub const fn checked_prev(self) -> Option<Self> {
        let ordered = self.to_ordered_i64();
        match ordered.checked_sub(1) {
            Some(prev_ordered) => Some(Self::from_ordered_i64(prev_ordered)),
            None => None,
        }
    }

    /// Converts an inclusive primitive [`f64`] range into an inclusive [`F64`] range.
    #[must_use]
    pub fn range(range: RangeInclusive<f64>) -> RangeInclusive<Self> {
        let (start, end) = range.into_inner();
        Self(start)..=Self(end)
    }

    /// Converts inclusive primitive [`f64`] ranges into inclusive [`F64`] ranges.
    pub fn ranges<I>(ranges: I) -> impl Iterator<Item = RangeInclusive<Self>>
    where
        I: IntoIterator<Item = RangeInclusive<f64>>,
    {
        ranges.into_iter().map(Self::range)
    }

    /// Converts primitive [`f64`] values into ordered [`F64`] values.
    pub fn values<I>(values: I) -> impl Iterator<Item = Self>
    where
        I: IntoIterator<Item = f64>,
    {
        values.into_iter().map(F64)
    }

    /// Views primitive [`f64`] values as ordered [`F64`] values.
    ///
    /// This runs in `O(1)` and does not allocate.
    #[must_use]
    pub const fn slice(values: &[f64]) -> &[Self] {
        // SAFETY: F64 is #[repr(transparent)] over f64, making `&[f64]`
        // and `&[F64]` entirely interchangeable in layout and lifetimes.
        unsafe { core::mem::transmute::<&[f64], &[Self]>(values) }
    }
}

impl From<f64> for F64 {
    fn from(value: f64) -> Self {
        Self(value)
    }
}

impl From<F64> for f64 {
    fn from(value: F64) -> Self {
        value.0
    }
}

impl Add for F64 {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}

impl AddAssign for F64 {
    fn add_assign(&mut self, rhs: Self) {
        self.0 += rhs.0;
    }
}

impl Mul for F64 {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        Self(self.0 * rhs.0)
    }
}

impl SubAssign for F64 {
    fn sub_assign(&mut self, rhs: Self) {
        self.0 -= rhs.0;
    }
}

impl Zero for F64 {
    fn zero() -> Self {
        Self(0.0)
    }

    fn is_zero(&self) -> bool {
        self == &Self::zero()
    }
}

impl One for F64 {
    fn one() -> Self {
        Self(1.0)
    }
}

impl PartialEq for F64 {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other).is_eq()
    }
}

impl Eq for F64 {}

impl PartialOrd for F64 {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for F64 {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.total_cmp(&other.0)
    }
}

impl Hash for F64 {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.to_bits().hash(state);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::hash_map::DefaultHasher;
    use std::vec;
    use std::vec::Vec;

    #[test]
    fn ordering_agrees_with_total_cmp() {
        let values = [
            f64::NEG_INFINITY,
            -f64::MAX,
            -1.0,
            -0.0,
            0.0,
            1.0,
            f64::MAX,
            f64::INFINITY,
            f64::NAN,
            f64::from_bits(0x7ff8_0000_0000_0001),
            f64::from_bits(0xfff8_0000_0000_0001),
        ];

        for left in values {
            for right in values {
                assert_eq!(F64(left).cmp(&F64(right)), left.total_cmp(&right));
            }
        }
    }

    #[test]
    fn equality_agrees_with_total_cmp() {
        assert_ne!(F64(-0.0), F64(0.0));
        assert_eq!(F64(f64::NAN), F64(f64::NAN));
    }

    #[test]
    fn equal_values_hash_equally() {
        let left = hash(F64(f64::NAN));
        let right = hash(F64(f64::NAN));

        assert_eq!(left, right);
    }

    #[test]
    fn converts_ranges() {
        assert_eq!(F64::range(10.0..=20.0), F64(10.0)..=F64(20.0));
        assert_eq!(
            F64::ranges([10.0..=20.0, 30.0..=40.0]).collect::<Vec<_>>(),
            vec![F64(10.0)..=F64(20.0), F64(30.0)..=F64(40.0)]
        );
    }

    #[test]
    fn add_assign_adds_inner_values() {
        let mut value = F64(1.5);

        value += F64(2.25);

        assert_eq!(value, F64(3.75));
    }

    #[test]
    fn sub_assign_subtracts_inner_values() {
        let mut value = F64(1.5);

        value -= F64(2.25);

        assert_eq!(value, F64(-0.75));
    }

    #[test]
    fn zero_is_additive_identity() {
        let value = F64(3.5);

        assert_eq!(F64::zero(), F64(0.0));
        assert!(F64::zero().is_zero());
        assert!(!F64(-0.0).is_zero());
        assert_eq!(value + F64::zero(), value);
        assert_eq!(F64::zero() + value, value);
    }

    #[test]
    fn one_is_multiplicative_identity() {
        let value = F64(3.5);

        assert_eq!(F64::one(), F64(1.0));
        assert_eq!(value * F64::one(), value);
        assert_eq!(F64::one() * value, value);
    }

    #[test]
    fn next_and_prev_step_through_zero_in_total_order() {
        assert_eq!(F64(-0.0).next(), F64(0.0));
        assert_eq!(F64(0.0).prev(), F64(-0.0));
        assert_eq!(F64(0.0).next(), F64(f64::from_bits(1)));
        assert_eq!(F64(-0.0).prev(), F64(f64::from_bits(0x8000_0000_0000_0001)));
    }

    #[test]
    fn next_and_prev_step_around_infinities() {
        assert_eq!(F64(f64::MAX).next(), F64(f64::INFINITY));
        assert_eq!(F64(f64::INFINITY).prev(), F64(f64::MAX));
        assert_eq!(F64(f64::NEG_INFINITY).next(), F64(-f64::MAX));
        assert_eq!(F64(-f64::MAX).prev(), F64(f64::NEG_INFINITY));
    }

    #[test]
    fn checked_next_and_prev_stop_at_total_order_boundaries() {
        assert_eq!(F64::MIN.checked_prev(), None);
        assert_eq!(F64::MAX.checked_next(), None);
        assert_eq!(F64::MIN.checked_next(), Some(F64::MIN.next()));
        assert_eq!(F64::MAX.checked_prev(), Some(F64::MAX.prev()));
    }

    #[test]
    fn min_and_max_are_total_order_boundaries() {
        let values = [
            F64(f64::NEG_INFINITY),
            F64(-f64::MAX),
            F64(-1.0),
            F64(-0.0),
            F64(0.0),
            F64(1.0),
            F64(f64::MAX),
            F64(f64::INFINITY),
            F64(f64::NAN),
            F64(f64::from_bits(0x7ff8_0000_0000_0001)),
            F64(f64::from_bits(0xfff8_0000_0000_0001)),
        ];

        for value in values {
            assert!(F64::MIN <= value);
            assert!(value <= F64::MAX);
        }
    }

    #[test]
    fn next_and_prev_are_neighbors_in_total_order() {
        let values = [
            F64(f64::NEG_INFINITY),
            F64(-f64::MAX),
            F64(-1.0),
            F64(-0.0),
            F64(0.0),
            F64(1.0),
            F64(f64::MAX),
            F64(f64::INFINITY),
            F64(f64::NAN),
            F64(f64::from_bits(0x7ff8_0000_0000_0001)),
            F64(f64::from_bits(0xfff8_0000_0000_0001)),
        ];

        for value in values {
            assert_eq!(value.next().prev(), value);
            assert_eq!(value.prev().next(), value);
        }
    }

    fn hash(value: F64) -> u64 {
        let mut hasher = DefaultHasher::new();
        value.hash(&mut hasher);
        hasher.finish()
    }
}
