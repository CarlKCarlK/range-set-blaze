//! Ordered `f128` support.

use core::cmp::Ordering;
use core::hash::{Hash, Hasher};
use core::ops::{Add, AddAssign, Mul, RangeInclusive, SubAssign};

use num_traits::{One, Zero};

/// A transparent wrapper around [`f128`] with total ordering.
///
/// Comparison, equality, and hashing all agree with [`f128::total_cmp`].
#[repr(transparent)]
#[derive(Copy, Clone, Default, Debug)]
pub struct TotalF128(pub f128);

impl TotalF128 {
    /// The minimum value in [`f128::total_cmp`] order.
    pub const MIN: Self = Self(f128::from_bits(u128::MAX));

    /// The maximum value in [`f128::total_cmp`] order.
    pub const MAX: Self = Self(f128::from_bits(0x7fff_ffff_ffff_ffff_ffff_ffff_ffff_ffff));

    /// The most negative finite [`f128`] value.
    pub const MIN_VALUE: Self = Self(f128::MIN);

    /// The most positive finite [`f128`] value.
    pub const MAX_VALUE: Self = Self(f128::MAX);

    /// Returns the wrapped [`f128`] value.
    #[must_use]
    pub const fn into_inner(self) -> f128 {
        self.0
    }

    /// Views primitive [`f128`] values as ordered [`TotalF128`] values.
    ///
    /// This runs in `O(1)` and does not allocate.
    #[must_use]
    pub const fn values(values: &[f128]) -> &[Self] {
        // SAFETY: TotalF128 is #[repr(transparent)] over f128, making `&[f128]`
        // and `&[TotalF128]` entirely interchangeable in layout and lifetimes.
        unsafe { core::mem::transmute::<&[f128], &[Self]>(values) }
    }

    /// Transforms the float bits into the monotonically ordered `i128` space used by `total_cmp`.
    pub(crate) const fn to_ordered_i128(self) -> i128 {
        let mut bits = self.0.to_bits().cast_signed();
        bits ^= ((bits >> 127).cast_unsigned() >> 1).cast_signed();
        bits
    }

    /// Transforms the ordered `i128` space back into standard float bits.
    pub(crate) const fn from_ordered_i128(mut bits: i128) -> Self {
        // Reversing the XOR transformation
        bits ^= ((bits >> 127).cast_unsigned() >> 1).cast_signed();
        Self(f128::from_bits(bits.cast_unsigned()))
    }

    /// Returns the next float in total order.
    ///
    /// Panics on overflow if `self` is the maximum value in total order.
    #[must_use]
    pub const fn next(self) -> Self {
        let ordered = self.to_ordered_i128();
        Self::from_ordered_i128(ordered + 1)
    }

    /// Returns the previous float in total order.
    ///
    /// Panics on overflow if `self` is the minimum value in total order.
    #[must_use]
    pub const fn prev(self) -> Self {
        let ordered = self.to_ordered_i128();
        Self::from_ordered_i128(ordered - 1)
    }

    /// Returns the next float in total order.
    ///
    /// Returns [`None`] if `self` is the maximum value in total order.
    #[must_use]
    pub const fn checked_next(self) -> Option<Self> {
        let ordered = self.to_ordered_i128();
        match ordered.checked_add(1) {
            Some(next_ordered) => Some(Self::from_ordered_i128(next_ordered)),
            None => None,
        }
    }

    /// Returns the previous float in total order.
    ///
    /// Returns [`None`] if `self` is the minimum value in total order.
    #[must_use]
    pub const fn checked_prev(self) -> Option<Self> {
        let ordered = self.to_ordered_i128();
        match ordered.checked_sub(1) {
            Some(prev_ordered) => Some(Self::from_ordered_i128(prev_ordered)),
            None => None,
        }
    }
}

impl From<f128> for TotalF128 {
    fn from(value: f128) -> Self {
        Self(value)
    }
}

impl From<TotalF128> for f128 {
    fn from(value: TotalF128) -> Self {
        value.0
    }
}

impl Add for TotalF128 {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}

impl AddAssign for TotalF128 {
    fn add_assign(&mut self, rhs: Self) {
        self.0 += rhs.0;
    }
}

impl Mul for TotalF128 {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        Self(self.0 * rhs.0)
    }
}

impl SubAssign for TotalF128 {
    fn sub_assign(&mut self, rhs: Self) {
        self.0 -= rhs.0;
    }
}

impl Zero for TotalF128 {
    fn zero() -> Self {
        Self(0.0)
    }

    fn is_zero(&self) -> bool {
        self == &Self::zero()
    }
}

impl One for TotalF128 {
    fn one() -> Self {
        Self(1.0)
    }
}

impl PartialEq for TotalF128 {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other).is_eq()
    }
}

impl Eq for TotalF128 {}

impl PartialOrd for TotalF128 {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for TotalF128 {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.total_cmp(&other.0)
    }
}

impl Hash for TotalF128 {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.to_bits().hash(state);
    }
}

/// Wraps a primitive [`f128`] as an [`TotalF128`].
#[must_use]
pub const fn value(value: f128) -> TotalF128 {
    TotalF128(value)
}

/// Converts primitive [`f128`] values into ordered [`TotalF128`] values.
pub fn values<I>(values: I) -> impl Iterator<Item = TotalF128>
where
    I: IntoIterator<Item = f128>,
{
    values.into_iter().map(TotalF128)
}

/// Converts an inclusive primitive [`f128`] range into an inclusive [`TotalF128`] range.
#[must_use]
pub fn range(range: RangeInclusive<f128>) -> RangeInclusive<TotalF128> {
    let (start, end) = range.into_inner();
    TotalF128(start)..=TotalF128(end)
}

/// Converts inclusive primitive [`f128`] ranges into inclusive [`TotalF128`] ranges.
pub fn ranges<I>(ranges: I) -> impl Iterator<Item = RangeInclusive<TotalF128>>
where
    I: IntoIterator<Item = RangeInclusive<f128>>,
{
    ranges.into_iter().map(range)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::hash_map::DefaultHasher;

    #[test]
    fn ordering_agrees_with_total_cmp() {
        let values = [
            f128::NEG_INFINITY,
            -f128::MAX,
            -1.0,
            -0.0,
            0.0,
            1.0,
            f128::MAX,
            f128::INFINITY,
            f128::NAN,
            f128::from_bits(0x7fff_8000_0000_0000_0000_0000_0000_0001),
            f128::from_bits(0xffff_8000_0000_0000_0000_0000_0000_0001),
        ];

        for left in values {
            for right in values {
                assert_eq!(TotalF128(left).cmp(&TotalF128(right)), left.total_cmp(&right));
            }
        }
    }

    #[test]
    fn equality_agrees_with_total_cmp() {
        assert_ne!(TotalF128(-0.0), TotalF128(0.0));
        assert_eq!(TotalF128(f128::NAN), TotalF128(f128::NAN));
    }

    #[test]
    fn equal_values_hash_equally() {
        let left = hash(TotalF128(f128::NAN));
        let right = hash(TotalF128(f128::NAN));

        assert_eq!(left, right);
    }

    #[test]
    fn values_views_f128_slice_as_f128_wrapper_slice() {
        let values = [3.0, 2.0, -0.0, f128::NAN];
        let wrapped = TotalF128::values(&values);

        assert_eq!(wrapped, [TotalF128(3.0), TotalF128(2.0), TotalF128(-0.0), TotalF128(f128::NAN)]);
        assert_eq!(wrapped.as_ptr().cast::<f128>(), values.as_ptr());
    }

    #[test]
    fn add_assign_adds_inner_values() {
        let mut value = TotalF128(1.5);

        value += TotalF128(2.25);

        assert_eq!(value, TotalF128(3.75));
    }

    #[test]
    fn sub_assign_subtracts_inner_values() {
        let mut value = TotalF128(1.5);

        value -= TotalF128(2.25);

        assert_eq!(value, TotalF128(-0.75));
    }

    #[test]
    fn zero_is_additive_identity() {
        let value = TotalF128(3.5);

        assert_eq!(TotalF128::zero(), TotalF128(0.0));
        assert!(TotalF128::zero().is_zero());
        assert!(!TotalF128(-0.0).is_zero());
        assert_eq!(value + TotalF128::zero(), value);
        assert_eq!(TotalF128::zero() + value, value);
    }

    #[test]
    fn one_is_multiplicative_identity() {
        let value = TotalF128(3.5);

        assert_eq!(TotalF128::one(), TotalF128(1.0));
        assert_eq!(value * TotalF128::one(), value);
        assert_eq!(TotalF128::one() * value, value);
    }

    #[test]
    fn next_and_prev_step_through_zero_in_total_order() {
        assert_eq!(TotalF128(-0.0).next(), TotalF128(0.0));
        assert_eq!(TotalF128(0.0).prev(), TotalF128(-0.0));
        assert_eq!(TotalF128(0.0).next(), TotalF128(f128::from_bits(1)));
        assert_eq!(
            TotalF128(-0.0).prev(),
            TotalF128(f128::from_bits(0x8000_0000_0000_0000_0000_0000_0000_0001))
        );
    }

    #[test]
    fn next_and_prev_step_around_infinities() {
        assert_eq!(TotalF128(f128::MAX).next(), TotalF128(f128::INFINITY));
        assert_eq!(TotalF128(f128::INFINITY).prev(), TotalF128(f128::MAX));
        assert_eq!(TotalF128(f128::NEG_INFINITY).next(), TotalF128(-f128::MAX));
        assert_eq!(TotalF128(-f128::MAX).prev(), TotalF128(f128::NEG_INFINITY));
    }

    #[test]
    fn checked_next_and_prev_stop_at_total_order_boundaries() {
        assert_eq!(TotalF128::MIN.checked_prev(), None);
        assert_eq!(TotalF128::MAX.checked_next(), None);
        assert_eq!(TotalF128::MIN.checked_next(), Some(TotalF128::MIN.next()));
        assert_eq!(TotalF128::MAX.checked_prev(), Some(TotalF128::MAX.prev()));
    }

    #[test]
    fn min_and_max_are_total_order_boundaries() {
        let values = [
            TotalF128(f128::NEG_INFINITY),
            TotalF128(-f128::MAX),
            TotalF128(-1.0),
            TotalF128(-0.0),
            TotalF128(0.0),
            TotalF128(1.0),
            TotalF128(f128::MAX),
            TotalF128(f128::INFINITY),
            TotalF128(f128::NAN),
            TotalF128(f128::from_bits(0x7fff_8000_0000_0000_0000_0000_0000_0001)),
            TotalF128(f128::from_bits(0xffff_8000_0000_0000_0000_0000_0000_0001)),
        ];

        for value in values {
            assert!(TotalF128::MIN <= value);
            assert!(value <= TotalF128::MAX);
        }
    }

    #[test]
    fn min_value_and_max_value_bound_the_finite_values() {
        let finite_values = [
            TotalF128(f128::MIN),
            TotalF128(-1.0),
            TotalF128(-0.0),
            TotalF128(0.0),
            TotalF128(1.0),
            TotalF128(f128::MAX),
        ];

        for value in finite_values {
            assert!(TotalF128::MIN_VALUE <= value);
            assert!(value <= TotalF128::MAX_VALUE);
        }

        let non_finite_values = [
            TotalF128::MIN,
            TotalF128(f128::NEG_INFINITY),
            TotalF128(f128::INFINITY),
            TotalF128(f128::NAN),
            TotalF128(f128::from_bits(0x7fff_8000_0000_0000_0000_0000_0000_0001)),
            TotalF128::MAX,
        ];

        for value in non_finite_values {
            assert!(value < TotalF128::MIN_VALUE || TotalF128::MAX_VALUE < value);
        }
    }

    #[test]
    fn next_and_prev_are_neighbors_in_total_order() {
        let values = [
            TotalF128(f128::NEG_INFINITY),
            TotalF128(-f128::MAX),
            TotalF128(-1.0),
            TotalF128(-0.0),
            TotalF128(0.0),
            TotalF128(1.0),
            TotalF128(f128::MAX),
            TotalF128(f128::INFINITY),
            TotalF128(f128::NAN),
            TotalF128(f128::from_bits(0x7fff_8000_0000_0000_0000_0000_0000_0001)),
            TotalF128(f128::from_bits(0xffff_8000_0000_0000_0000_0000_0000_0001)),
        ];

        for value in values {
            assert_eq!(value.next().prev(), value);
            assert_eq!(value.prev().next(), value);
        }
    }

    fn hash(value: TotalF128) -> u64 {
        let mut hasher = DefaultHasher::new();
        value.hash(&mut hasher);
        hasher.finish()
    }
}
