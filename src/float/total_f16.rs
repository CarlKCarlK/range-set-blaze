//! Ordered `f16` support.

use core::cmp::Ordering;
use core::hash::{Hash, Hasher};
use core::ops::{Add, AddAssign, Mul, RangeInclusive, SubAssign};

use num_traits::{One, Zero};

/// A transparent wrapper around [`f16`] with total ordering.
///
/// Comparison, equality, and hashing all agree with [`f16::total_cmp`].
#[repr(transparent)]
#[derive(Copy, Clone, Default, Debug)]
pub struct TotalF16(pub f16);

impl TotalF16 {
    /// The minimum value in [`f16::total_cmp`] order.
    pub const MIN: Self = Self(f16::from_bits(u16::MAX));

    /// The maximum value in [`f16::total_cmp`] order.
    pub const MAX: Self = Self(f16::from_bits(0x7fff));

    /// The most negative finite [`f16`] value.
    pub const MIN_VALUE: Self = Self(f16::MIN);

    /// The most positive finite [`f16`] value.
    pub const MAX_VALUE: Self = Self(f16::MAX);

    /// A [`RangeInclusive`] including the finite values of the type.
    /// This is what the complement of an empty set ought to be, but tragically cannot.
    pub const ALL_VALUES: core::ops::RangeInclusive<Self> = Self::MIN_VALUE..=Self::MAX_VALUE;

    /// new zero value
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns the wrapped [`f16`] value.
    #[must_use]
    pub const fn into_inner(self) -> f16 {
        self.0
    }

    /// Transforms the float bits into the monotonically ordered `i16` space used by `total_cmp`.
    pub(crate) const fn to_ordered_i16(self) -> i16 {
        let mut bits = self.0.to_bits().cast_signed();
        bits ^= ((bits >> 15).cast_unsigned() >> 1).cast_signed();
        bits
    }

    /// Transforms the ordered `i16` space back into standard float bits.
    pub(crate) const fn from_ordered_i16(mut bits: i16) -> Self {
        bits ^= ((bits >> 15).cast_unsigned() >> 1).cast_signed();
        Self(f16::from_bits(bits.cast_unsigned()))
    }

    /// Returns the next float in total order.
    ///
    /// Panics on overflow if `self` is the maximum value in total order.
    #[must_use]
    pub const fn next(self) -> Self {
        let ordered = self.to_ordered_i16();
        Self::from_ordered_i16(ordered + 1)
    }

    /// Returns the previous float in total order.
    ///
    /// Panics on overflow if `self` is the minimum value in total order.
    #[must_use]
    pub const fn prev(self) -> Self {
        let ordered = self.to_ordered_i16();
        Self::from_ordered_i16(ordered - 1)
    }

    /// Returns the next float in total order.
    ///
    /// Returns [`None`] if `self` is the maximum value in total order.
    #[must_use]
    pub const fn checked_next(self) -> Option<Self> {
        let ordered = self.to_ordered_i16();
        match ordered.checked_add(1) {
            Some(next_ordered) => Some(Self::from_ordered_i16(next_ordered)),
            None => None,
        }
    }

    /// Returns the previous float in total order.
    ///
    /// Returns [`None`] if `self` is the minimum value in total order.
    #[must_use]
    pub const fn checked_prev(self) -> Option<Self> {
        let ordered = self.to_ordered_i16();
        match ordered.checked_sub(1) {
            Some(prev_ordered) => Some(Self::from_ordered_i16(prev_ordered)),
            None => None,
        }
    }

    /// Converts an inclusive primitive [`f16`] range into an inclusive [`TotalF16`] range.
    #[must_use]
    pub fn range(range: RangeInclusive<f16>) -> RangeInclusive<Self> {
        let (start, end) = range.into_inner();
        Self(start)..=Self(end)
    }

    /// Converts inclusive primitive [`f16`] ranges into inclusive [`TotalF16`] ranges.
    pub fn ranges<I>(ranges: I) -> impl Iterator<Item = RangeInclusive<Self>>
    where
        I: IntoIterator<Item = RangeInclusive<f16>>,
    {
        ranges.into_iter().map(Self::range)
    }

    /// Converts primitive [`f16`] values into ordered [`TotalF16`] values.
    pub fn values<I>(values: I) -> impl Iterator<Item = Self>
    where
        I: IntoIterator<Item = f16>,
    {
        values.into_iter().map(TotalF16)
    }

    /// Views primitive [`f16`] values as ordered [`TotalF16`] values.
    ///
    /// This runs in `O(1)` and does not allocate.
    #[must_use]
    pub const fn slice(values: &[f16]) -> &[Self] {
        // SAFETY: TotalF16 is #[repr(transparent)] over f16, making `&[f16]`
        // and `&[TotalF16]` entirely interchangeable in layout and lifetimes.
        unsafe { core::mem::transmute::<&[f16], &[Self]>(values) }
    }
}

/// Views  [`TotalF16`] values as primitive [`f16`] values.
///
/// This runs in `O(1)` and does not allocate.
#[must_use]
pub const fn primitive_slice(values: &[TotalF16]) -> &[f16] {
    // SAFETY: TotalF16 is #[repr(transparent)] over f16, making `&[f16]`
    // and `&[TotalF16]` entirely interchangeable in layout and lifetimes.
    unsafe { core::mem::transmute::<&[TotalF16], &[f16]>(values) }
}

impl From<f16> for TotalF16 {
    fn from(value: f16) -> Self {
        Self(value)
    }
}

impl From<TotalF16> for f16 {
    fn from(value: TotalF16) -> Self {
        value.0
    }
}

impl Add for TotalF16 {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}

impl AddAssign for TotalF16 {
    fn add_assign(&mut self, rhs: Self) {
        self.0 += rhs.0;
    }
}

impl Mul for TotalF16 {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        Self(self.0 * rhs.0)
    }
}

impl SubAssign for TotalF16 {
    fn sub_assign(&mut self, rhs: Self) {
        self.0 -= rhs.0;
    }
}

impl Zero for TotalF16 {
    fn zero() -> Self {
        Self(0.0)
    }

    fn is_zero(&self) -> bool {
        self == &Self::zero()
    }
}

impl One for TotalF16 {
    fn one() -> Self {
        Self(1.0)
    }
}

impl PartialEq for TotalF16 {
    fn eq(&self, other: &Self) -> bool {
        self.0.to_bits() == other.0.to_bits()
    }
}

impl Eq for TotalF16 {}

impl PartialOrd for TotalF16 {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for TotalF16 {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.total_cmp(&other.0)
    }
}

impl Hash for TotalF16 {
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
            f16::NEG_INFINITY,
            -f16::MAX,
            -1.0,
            -0.0,
            0.0,
            1.0,
            f16::MAX,
            f16::INFINITY,
            f16::NAN,
            f16::from_bits(0x7fc1),
            f16::from_bits(0xffc1),
        ];

        for left in values {
            for right in values {
                assert_eq!(TotalF16(left).cmp(&TotalF16(right)), left.total_cmp(&right));
            }
        }
    }

    #[test]
    fn equality_agrees_with_total_cmp() {
        assert_ne!(TotalF16(-0.0), TotalF16(0.0));
        assert_eq!(TotalF16(f16::NAN), TotalF16(f16::NAN));
    }

    #[test]
    fn equal_values_hash_equally() {
        let left = hash(TotalF16(f16::NAN));
        let right = hash(TotalF16(f16::NAN));

        assert_eq!(left, right);
    }

    #[test]
    fn converts_ranges() {
        assert_eq!(
            TotalF16::range(10.0..=20.0),
            TotalF16(10.0)..=TotalF16(20.0)
        );
        assert_eq!(
            TotalF16::ranges([10.0..=20.0, 30.0..=40.0]).collect::<Vec<_>>(),
            vec![
                TotalF16(10.0)..=TotalF16(20.0),
                TotalF16(30.0)..=TotalF16(40.0)
            ]
        );
    }

    #[test]
    fn add_assign_adds_inner_values() {
        let mut value = TotalF16(1.5);

        value += TotalF16(2.25);

        assert_eq!(value, TotalF16(3.75));
    }

    #[test]
    fn sub_assign_subtracts_inner_values() {
        let mut value = TotalF16(1.5);

        value -= TotalF16(2.25);

        assert_eq!(value, TotalF16(-0.75));
    }

    #[test]
    fn zero_is_additive_identity() {
        let value = TotalF16(3.5);

        assert_eq!(TotalF16::zero(), TotalF16(0.0));
        assert!(TotalF16::zero().is_zero());
        assert!(!TotalF16(-0.0).is_zero());
        assert_eq!(value + TotalF16::zero(), value);
        assert_eq!(TotalF16::zero() + value, value);
    }

    #[test]
    fn one_is_multiplicative_identity() {
        let value = TotalF16(3.5);

        assert_eq!(TotalF16::one(), TotalF16(1.0));
        assert_eq!(value * TotalF16::one(), value);
        assert_eq!(TotalF16::one() * value, value);
    }

    #[test]
    fn next_and_prev_step_through_zero_in_total_order() {
        assert_eq!(TotalF16(-0.0).next(), TotalF16(0.0));
        assert_eq!(TotalF16(0.0).prev(), TotalF16(-0.0));
        assert_eq!(TotalF16(0.0).next(), TotalF16(f16::from_bits(1)));
        assert_eq!(TotalF16(-0.0).prev(), TotalF16(f16::from_bits(0x8001)));
    }

    #[test]
    fn next_and_prev_step_around_infinities() {
        assert_eq!(TotalF16(f16::MAX).next(), TotalF16(f16::INFINITY));
        assert_eq!(TotalF16(f16::INFINITY).prev(), TotalF16(f16::MAX));
        assert_eq!(TotalF16(f16::NEG_INFINITY).next(), TotalF16(-f16::MAX));
        assert_eq!(TotalF16(-f16::MAX).prev(), TotalF16(f16::NEG_INFINITY));
    }

    #[test]
    fn checked_next_and_prev_stop_at_total_order_boundaries() {
        assert_eq!(TotalF16::MIN.checked_prev(), None);
        assert_eq!(TotalF16::MAX.checked_next(), None);
        assert_eq!(TotalF16::MIN.checked_next(), Some(TotalF16::MIN.next()));
        assert_eq!(TotalF16::MAX.checked_prev(), Some(TotalF16::MAX.prev()));
    }

    #[test]
    fn min_and_max_are_total_order_boundaries() {
        let values = [
            TotalF16(f16::NEG_INFINITY),
            TotalF16(-f16::MAX),
            TotalF16(-1.0),
            TotalF16(-0.0),
            TotalF16(0.0),
            TotalF16(1.0),
            TotalF16(f16::MAX),
            TotalF16(f16::INFINITY),
            TotalF16(f16::NAN),
            TotalF16(f16::from_bits(0x7fc1)),
            TotalF16(f16::from_bits(0xffc1)),
        ];

        for value in values {
            assert!(TotalF16::MIN <= value);
            assert!(value <= TotalF16::MAX);
        }
    }

    #[test]
    fn next_and_prev_are_neighbors_in_total_order() {
        let values = [
            TotalF16(f16::NEG_INFINITY),
            TotalF16(-f16::MAX),
            TotalF16(-1.0),
            TotalF16(-0.0),
            TotalF16(0.0),
            TotalF16(1.0),
            TotalF16(f16::MAX),
            TotalF16(f16::INFINITY),
            TotalF16(f16::NAN),
            TotalF16(f16::from_bits(0x7fc1)),
            TotalF16(f16::from_bits(0xffc1)),
        ];

        for value in values {
            assert_eq!(value.next().prev(), value);
            assert_eq!(value.prev().next(), value);
        }
    }

    fn hash(value: TotalF16) -> u64 {
        let mut hasher = DefaultHasher::new();
        value.hash(&mut hasher);
        hasher.finish()
    }
}
