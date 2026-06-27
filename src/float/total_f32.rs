//! Ordered `f32` support.

use core::cmp::Ordering;
use core::hash::{Hash, Hasher};
use core::ops::{Add, AddAssign, Mul, RangeInclusive, SubAssign};

use num_traits::{One, Zero};

/// Experimental: A transparent wrapper around [`f32`] with total ordering.
///
/// Comparison, equality, and hashing all agree with [`f32::total_cmp`].
///
/// # Enabling
///
/// This type is experimental and must be enabled with the `total_float_experimental` feature.
/// ```bash
/// cargo add range-set-blaze --features "total_float_experimental"
/// ```
#[repr(transparent)]
#[derive(Copy, Clone, Default, Debug)]
pub struct TotalF32(pub f32);

impl TotalF32 {
    /// The minimum value in [`f32::total_cmp`] order.
    pub const MIN: Self = Self(f32::from_bits(u32::MAX));

    /// The maximum value in [`f32::total_cmp`] order.
    pub const MAX: Self = Self(f32::from_bits(0x7fff_ffff));

    /// The most negative finite [`f32`] value.
    pub const MIN_VALUE: Self = Self(f32::MIN);

    /// The most positive finite [`f32`] value.
    pub const MAX_VALUE: Self = Self(f32::MAX);

    /// A [`RangeInclusive`] including the finite values of the type.
    /// This is what the complement of an empty set ought to be, but tragically cannot.
    pub const ALL_VALUES: core::ops::RangeInclusive<Self> = Self::MIN_VALUE..=Self::MAX_VALUE;

    /// new zero value
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns the wrapped [`f32`] value.
    #[must_use]
    pub const fn into_inner(self) -> f32 {
        self.0
    }

    /// Transforms the float bits into the monotonically ordered `i32` space used by `total_cmp`.
    pub(crate) const fn to_ordered_i32(self) -> i32 {
        let mut bits = self.0.to_bits().cast_signed();
        bits ^= ((bits >> 31).cast_unsigned() >> 1).cast_signed();
        bits
    }

    /// Transforms the ordered `i32` space back into standard float bits.
    pub(crate) const fn from_ordered_i32(mut bits: i32) -> Self {
        bits ^= ((bits >> 31).cast_unsigned() >> 1).cast_signed();
        Self(f32::from_bits(bits.cast_unsigned()))
    }

    /// Returns the next float in total order.
    ///
    /// Panics on overflow if `self` is the maximum value in total order.
    #[must_use]
    pub const fn next(self) -> Self {
        let ordered = self.to_ordered_i32();
        Self::from_ordered_i32(ordered + 1)
    }

    /// Returns the previous float in total order.
    ///
    /// Panics on overflow if `self` is the minimum value in total order.
    #[must_use]
    pub const fn prev(self) -> Self {
        let ordered = self.to_ordered_i32();
        Self::from_ordered_i32(ordered - 1)
    }

    /// Returns the next float in total order.
    ///
    /// Returns [`None`] if `self` is the maximum value in total order.
    #[must_use]
    pub const fn checked_next(self) -> Option<Self> {
        let ordered = self.to_ordered_i32();
        match ordered.checked_add(1) {
            Some(next_ordered) => Some(Self::from_ordered_i32(next_ordered)),
            None => None,
        }
    }

    /// Returns the previous float in total order.
    ///
    /// Returns [`None`] if `self` is the minimum value in total order.
    #[must_use]
    pub const fn checked_prev(self) -> Option<Self> {
        let ordered = self.to_ordered_i32();
        match ordered.checked_sub(1) {
            Some(prev_ordered) => Some(Self::from_ordered_i32(prev_ordered)),
            None => None,
        }
    }

    /// Converts an inclusive primitive [`f32`] range into an inclusive [`TotalF32`] range.
    #[must_use]
    pub fn range(range: RangeInclusive<f32>) -> RangeInclusive<Self> {
        let (start, end) = range.into_inner();
        Self(start)..=Self(end)
    }

    /// Converts inclusive primitive [`f32`] ranges into inclusive [`TotalF32`] ranges.
    pub fn ranges<I>(ranges: I) -> impl Iterator<Item = RangeInclusive<Self>>
    where
        I: IntoIterator<Item = RangeInclusive<f32>>,
    {
        ranges.into_iter().map(Self::range)
    }

    /// Converts primitive [`f32`] values into ordered [`TotalF32`] values.
    pub fn values<I>(values: I) -> impl Iterator<Item = Self>
    where
        I: IntoIterator<Item = f32>,
    {
        values.into_iter().map(TotalF32)
    }

    /// Views primitive [`f32`] values as ordered [`TotalF32`] values.
    ///
    /// This runs in `O(1)` and does not allocate.
    #[must_use]
    pub const fn slice(values: &[f32]) -> &[Self] {
        // SAFETY: TotalF32 is #[repr(transparent)] over f32, making `&[f32]`
        // and `&[TotalF32]` entirely interchangeable in layout and lifetimes.
        unsafe { core::mem::transmute::<&[f32], &[Self]>(values) }
    }
}

/// Views  [`TotalF32`] values as primitive [`f32`] values.
///
/// This runs in `O(1)` and does not allocate.
#[must_use]
pub const fn primitive_slice(values: &[TotalF32]) -> &[f32] {
    // SAFETY: TotalF32 is #[repr(transparent)] over f32, making `&[f32]`
    // and `&[TotalF32]` entirely interchangeable in layout and lifetimes.
    unsafe { core::mem::transmute::<&[TotalF32], &[f32]>(values) }
}

impl From<f32> for TotalF32 {
    fn from(value: f32) -> Self {
        Self(value)
    }
}

impl From<TotalF32> for f32 {
    fn from(value: TotalF32) -> Self {
        value.0
    }
}

impl Add for TotalF32 {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}

impl AddAssign for TotalF32 {
    fn add_assign(&mut self, rhs: Self) {
        self.0 += rhs.0;
    }
}

impl Mul for TotalF32 {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        Self(self.0 * rhs.0)
    }
}

impl SubAssign for TotalF32 {
    fn sub_assign(&mut self, rhs: Self) {
        self.0 -= rhs.0;
    }
}

impl Zero for TotalF32 {
    fn zero() -> Self {
        Self(0.0)
    }

    fn is_zero(&self) -> bool {
        self == &Self::zero()
    }
}

impl One for TotalF32 {
    fn one() -> Self {
        Self(1.0)
    }
}

impl PartialEq for TotalF32 {
    fn eq(&self, other: &Self) -> bool {
        self.0.to_bits() == other.0.to_bits()
    }
}

impl Eq for TotalF32 {}

impl PartialOrd for TotalF32 {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for TotalF32 {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.total_cmp(&other.0)
    }
}

impl Hash for TotalF32 {
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
            f32::NEG_INFINITY,
            -f32::MAX,
            -1.0,
            -0.0,
            0.0,
            1.0,
            f32::MAX,
            f32::INFINITY,
            f32::NAN,
            f32::from_bits(0x7fc0_0001),
            f32::from_bits(0xffc0_0001),
        ];

        for left in values {
            for right in values {
                assert_eq!(TotalF32(left).cmp(&TotalF32(right)), left.total_cmp(&right));
            }
        }
    }

    #[test]
    fn equality_agrees_with_total_cmp() {
        assert_ne!(TotalF32(-0.0), TotalF32(0.0));
        assert_eq!(TotalF32(f32::NAN), TotalF32(f32::NAN));
    }

    #[test]
    fn equal_values_hash_equally() {
        let left = hash(TotalF32(f32::NAN));
        let right = hash(TotalF32(f32::NAN));

        assert_eq!(left, right);
    }

    #[test]
    fn converts_ranges() {
        assert_eq!(
            TotalF32::range(10.0..=20.0),
            TotalF32(10.0)..=TotalF32(20.0)
        );
        assert_eq!(
            TotalF32::ranges([10.0..=20.0, 30.0..=40.0]).collect::<Vec<_>>(),
            vec![
                TotalF32(10.0)..=TotalF32(20.0),
                TotalF32(30.0)..=TotalF32(40.0)
            ]
        );
    }

    #[test]
    fn add_assign_adds_inner_values() {
        let mut value = TotalF32(1.5);

        value += TotalF32(2.25);

        assert_eq!(value, TotalF32(3.75));
    }

    #[test]
    fn sub_assign_subtracts_inner_values() {
        let mut value = TotalF32(1.5);

        value -= TotalF32(2.25);

        assert_eq!(value, TotalF32(-0.75));
    }

    #[test]
    fn zero_is_additive_identity() {
        let value = TotalF32(3.5);

        assert_eq!(TotalF32::zero(), TotalF32(0.0));
        assert!(TotalF32::zero().is_zero());
        assert!(!TotalF32(-0.0).is_zero());
        assert_eq!(value + TotalF32::zero(), value);
        assert_eq!(TotalF32::zero() + value, value);
    }

    #[test]
    fn one_is_multiplicative_identity() {
        let value = TotalF32(3.5);

        assert_eq!(TotalF32::one(), TotalF32(1.0));
        assert_eq!(value * TotalF32::one(), value);
        assert_eq!(TotalF32::one() * value, value);
    }

    #[test]
    fn next_and_prev_step_through_zero_in_total_order() {
        assert_eq!(TotalF32(-0.0).next(), TotalF32(0.0));
        assert_eq!(TotalF32(0.0).prev(), TotalF32(-0.0));
        assert_eq!(TotalF32(0.0).next(), TotalF32(f32::from_bits(1)));
        assert_eq!(TotalF32(-0.0).prev(), TotalF32(f32::from_bits(0x8000_0001)));
    }

    #[test]
    fn next_and_prev_step_around_infinities() {
        assert_eq!(TotalF32(f32::MAX).next(), TotalF32(f32::INFINITY));
        assert_eq!(TotalF32(f32::INFINITY).prev(), TotalF32(f32::MAX));
        assert_eq!(TotalF32(f32::NEG_INFINITY).next(), TotalF32(-f32::MAX));
        assert_eq!(TotalF32(-f32::MAX).prev(), TotalF32(f32::NEG_INFINITY));
    }

    #[test]
    fn checked_next_and_prev_stop_at_total_order_boundaries() {
        assert_eq!(TotalF32::MIN.checked_prev(), None);
        assert_eq!(TotalF32::MAX.checked_next(), None);
        assert_eq!(TotalF32::MIN.checked_next(), Some(TotalF32::MIN.next()));
        assert_eq!(TotalF32::MAX.checked_prev(), Some(TotalF32::MAX.prev()));
    }

    #[test]
    fn min_and_max_are_total_order_boundaries() {
        let values = [
            TotalF32(f32::NEG_INFINITY),
            TotalF32(-f32::MAX),
            TotalF32(-1.0),
            TotalF32(-0.0),
            TotalF32(0.0),
            TotalF32(1.0),
            TotalF32(f32::MAX),
            TotalF32(f32::INFINITY),
            TotalF32(f32::NAN),
            TotalF32(f32::from_bits(0x7fc0_0001)),
            TotalF32(f32::from_bits(0xffc0_0001)),
        ];

        for value in values {
            assert!(TotalF32::MIN <= value);
            assert!(value <= TotalF32::MAX);
        }
    }

    #[test]
    fn next_and_prev_are_neighbors_in_total_order() {
        let values = [
            TotalF32(f32::NEG_INFINITY),
            TotalF32(-f32::MAX),
            TotalF32(-1.0),
            TotalF32(-0.0),
            TotalF32(0.0),
            TotalF32(1.0),
            TotalF32(f32::MAX),
            TotalF32(f32::INFINITY),
            TotalF32(f32::NAN),
            TotalF32(f32::from_bits(0x7fc0_0001)),
            TotalF32(f32::from_bits(0xffc0_0001)),
        ];

        for value in values {
            assert_eq!(value.next().prev(), value);
            assert_eq!(value.prev().next(), value);
        }
    }

    fn hash(value: TotalF32) -> u64 {
        let mut hasher = DefaultHasher::new();
        value.hash(&mut hasher);
        hasher.finish()
    }
}
