//! Total is a floating point type, suitable for use in ranges. All values are valid.
//!
//! Ordering and other semantics are as per `total_cmp`.\
//! Every distinct bit pattern is a separate valid value, even though quite a few of them are NaN.\
//! For example, in a `TotalF32` all 16 million different NaN values are distinct from each other.
//!
//! Enable with `float_experimental` (stable, `TotalF32`/`TotalF64`) and
//! `float_nightly_experimental` (nightly, adds `TotalF16`/`TotalF128`).
//! ```
//! use range_set_blaze::{RangeSetBlaze, TotalF64, TotalF32};
//! let set = RangeSetBlaze::from_iter([TotalF64::new(3.0)..=TotalF64::new(5.0)]);
//! assert!(set.contains(TotalF64::new(3.1)));
//! assert!(!set.contains(TotalF64::new(2.9)));
//!
//! let set = RangeSetBlaze::from(TotalF64::from_primitive_range(3.0..=5.0));
//! assert!(set.contains(TotalF64::new(4.9)));
//! assert!(!set.contains(TotalF64::new(5.1)));
//!
//! let set = RangeSetBlaze::from_iter(TotalF32::from_primitive_ranges([3.0..=5.0, 7.0..=9.0]));
//! assert!(set.contains(TotalF32::new(4.0)));
//! assert!(!set.contains(TotalF32::new(6.0)));
//! ```

use super::total_float::TotalFloat;
use crate::Integer;
#[cfg(feature = "from_slice")]
use crate::RangeSetBlaze;
use core::{
    cmp::Ordering,
    fmt::Debug,
    hash::{Hash, Hasher},
    mem,
    ops::RangeInclusive,
    slice::from_raw_parts,
};
/// Total ordered f64, all values valid, including NaN, -0.0, +0.0, and infinities.
pub type TotalF64 = Total<f64>;
/// Total ordered f32, all values valid, including NaN, -0.0, +0.0, and infinities.
pub type TotalF32 = Total<f32>;
/// Total ordered f16, all values valid, including NaN, -0.0, +0.0, and infinities.
#[cfg(feature = "float_nightly_experimental")]
pub type TotalF16 = Total<f16>;
/// Total ordered f128, all values valid, including NaN, -0.0, +0.0, and infinities.
#[cfg(feature = "float_nightly_experimental")]
pub type TotalF128 = Total<f128>;

/// Construct a [`TotalF64`] from an `f64`. Shorthand for [`TotalF64::new`]
#[must_use]
pub const fn tf64(x: f64) -> TotalF64 {
    TotalF64::new(x)
}

/// Construct a [`TotalF32`] from an `f32`. Shorthand for [`TotalF32::new`]
#[must_use]
pub const fn tf32(x: f32) -> TotalF32 {
    TotalF32::new(x)
}

/// Construct a [`TotalF16`] from an `f16`. Shorthand for [`TotalF16::new`]
#[cfg(feature = "float_nightly_experimental")]
#[must_use]
pub const fn tf16(x: f16) -> TotalF16 {
    TotalF16::new(x)
}

/// Construct a [`TotalF128`] from an `f128`. Shorthand for [`TotalF128::new`]
#[cfg(feature = "float_nightly_experimental")]
#[must_use]
pub const fn tf128(x: f128) -> TotalF128 {
    TotalF128::new(x)
}

/// Experimental: A transparent wrapper around floating point values with total ordering.
///
/// Comparison, equality, and hashing all agree with `total_cmp`.
///
/// # Enabling
///
/// This type is experimental and must be enabled with the `float_experimental` feature.
/// ```bash
/// cargo add range-set-blaze --features "float_experimental"
/// ```
/// That provides the `TotalF32` and `TotalF64` types.
///
/// If you're building with nightly, you can instead use the `float_nightly_experimental` feature.
/// ```bash
/// cargo add range-set-blaze --features "float_nightly_experimental"
/// ```
/// To also use the `TotalF16` and `TotalF128` types.
#[repr(transparent)]
#[derive(Copy, Clone, Default, Debug)]
pub struct Total<T: TotalFloat>(T);

impl<T: TotalFloat> Total<T> {
    /// The minimum value that can be represented by the type.
    /// I.e., the smallest possible value according to `total_cmp`\
    /// Maps directly to [`crate::Integer::min_value()`]
    ///
    /// # Examples
    /// ```
    /// use range_set_blaze::TotalF64;
    ///
    /// assert_eq!(TotalF64::MIN, TotalF64::new(f64::from_bits(u64::MAX)));
    /// ```
    pub const MIN: Self = Self(T::MIN);

    /// The maximum value that can be represented by the type.
    /// I.e., the largest possible value according to `total_cmp`\
    /// Maps directly to [`crate::Integer::max_value()`]
    ///
    /// # Examples
    /// ```
    /// use range_set_blaze::TotalF64;
    ///
    /// assert_eq!(TotalF64::MAX, TotalF64::new(f64::from_bits(0x7fff_ffff_ffff_ffff)));
    /// ```
    pub const MAX: Self = Self(T::MAX);

    /// The maximum possible size of a range, i.e. the size if `[MIN..=MAX]`
    ///
    /// # Examples
    /// ```
    /// use range_set_blaze::TotalF32;
    ///
    /// assert_eq!(TotalF32::MAX_SIZE, u32::MAX as i64 + 1);
    /// ```
    pub const MAX_SIZE: T::SafeLen = T::MAX_SIZE;

    /// Creates a new [`Total`] from a primitive float.
    /// All values are legal.
    ///
    /// # Examples
    /// ```
    /// use range_set_blaze::TotalF64;
    ///
    /// let _ = TotalF64::new(f64::INFINITY);
    /// ```
    #[must_use]
    pub const fn new(x: T) -> Self {
        Self(x)
    }

    /// Computes `self + (b - 1)` where `b` is of type `SafeLen`.
    ///
    /// # Precondition
    /// `b` must be small enough that the result stays within range for `T`. This is
    /// checked with `debug_assert!` and is *not* checked in release builds, where
    /// violating it produces an unspecified (nonsense, but not unsafe) result rather
    /// than a panic. Callers are expected to only ever pass a `b` that satisfies this.
    #[must_use]
    pub fn inclusive_end_from_start(self, b: T::SafeLen) -> Self {
        Self(T::inclusive_end_from_start(self.0, b))
    }

    /// Computes `self - (b - 1)` where `b` is of type `SafeLen`.
    ///
    /// # Precondition
    /// `b` must be small enough that the result stays within range for `T`. This is
    /// checked with `debug_assert!` and is *not* checked in release builds, where
    /// violating it produces an unspecified (nonsense, but not unsafe) result rather
    /// than a panic. Callers are expected to only ever pass a `b` that satisfies this.
    #[must_use]
    pub fn start_from_inclusive_end(self, b: T::SafeLen) -> Self {
        Self(T::start_from_inclusive_end(self.0, b))
    }

    /// Returns the wrapped value.
    ///
    /// # Examples
    /// ```
    /// use range_set_blaze::TotalF64;
    ///
    /// assert_eq!(TotalF64::new(42.0).into_inner(), 42.0);
    /// ```
    #[must_use]
    pub const fn into_inner(self) -> T {
        self.0
    }

    /// Returns the next float in total order.
    ///
    /// # Examples
    /// ```
    /// use range_set_blaze::TotalF64;
    ///
    /// assert_eq!(TotalF64::new(42.0).after().before().into_inner(), 42.0);
    /// ```
    ///
    /// # Panics
    ///
    /// In debug builds, panics if `self` is the maximum value. In release
    /// builds, wraps around to the minimum value instead.
    #[must_use]
    pub fn after(self) -> Self {
        debug_assert!(self != Self::MAX, "after() called on maximum value");
        Self(T::after(self.0))
    }

    /// Returns the previous float in total order.
    ///
    /// # Examples
    /// ```
    /// use range_set_blaze::TotalF64;
    ///
    /// assert_eq!(TotalF64::new(42.0).before().after().into_inner(), 42.0);
    /// ```
    ///
    /// # Panics
    ///
    /// In debug builds, panics if `self` is the minimum value. In release
    /// builds, wraps around to the maximum value instead.
    #[must_use]
    pub fn before(self) -> Self {
        debug_assert!(self != Self::MIN, "before() called on minimum value");
        Self(T::before(self.0))
    }

    /// Returns the next float.
    ///
    /// Returns [`None`] if `self` is the maximum value.
    ///
    /// # Examples
    /// ```
    /// use range_set_blaze::TotalF64;
    ///
    /// let value = TotalF64::new(42.0);
    /// assert_eq!(value.checked_after(), Some(value.after()));
    /// let value = TotalF64::MAX;
    /// assert_eq!(value.checked_after(), None);
    /// ```
    #[must_use]
    pub fn checked_after(self) -> Option<Self> {
        if self == Self::MAX {
            None
        } else {
            Some(self.after())
        }
    }

    /// Returns the previous float.
    ///
    /// Returns [`None`] if `self` is the minimum value.
    ///
    /// # Examples
    /// ```
    /// use range_set_blaze::TotalF64;
    ///
    /// let value = TotalF64::new(42.0);
    /// assert_eq!(value.checked_before(), Some(value.before()));
    /// let value = TotalF64::MIN;
    /// assert_eq!(value.checked_before(), None);
    /// ```
    #[must_use]
    pub fn checked_before(self) -> Option<Self> {
        if self == Self::MIN {
            None
        } else {
            Some(self.before())
        }
    }

    /// Converts an inclusive primitive range into an inclusive [`Total`] range.
    ///
    /// "Primitive" here means Rust's built-in float type (e.g. `f64`).
    ///
    /// # Examples
    /// ```
    /// use range_set_blaze::{RangeSetBlaze, TotalF64};
    ///
    /// let short = RangeSetBlaze::from(TotalF64::from_primitive_range(3.0..=5.0));
    /// let long = RangeSetBlaze::from(TotalF64::new(3.0)..=TotalF64::new(5.0));
    /// assert_eq!(short, long);
    /// ```
    #[must_use]
    pub fn from_primitive_range(range: RangeInclusive<T>) -> RangeInclusive<Self> {
        let (start, end) = range.into_inner();
        Self(start)..=Self(end)
    }

    /// Converts inclusive primitive ranges into inclusive [`Total`] ranges.
    ///
    /// "Primitive" here means Rust's built-in float type (e.g. `f64`).
    ///
    /// # Examples
    /// ```
    /// use range_set_blaze::{RangeSetBlaze, TotalF64};
    ///
    /// let short = RangeSetBlaze::from_iter(TotalF64::from_primitive_ranges([1.0..=2.0, 3.0..=4.0]));
    /// let long = RangeSetBlaze::from_iter([TotalF64::new(1.0)..=TotalF64::new(2.0), TotalF64::new(3.0)..=TotalF64::new(4.0)]);
    /// assert_eq!(short, long);
    /// ```
    pub fn from_primitive_ranges<I>(ranges: I) -> impl Iterator<Item = RangeInclusive<Self>>
    where
        I: IntoIterator<Item = RangeInclusive<T>>,
    {
        ranges.into_iter().map(Self::from_primitive_range)
    }

    /// Convenience method to convert primitive values into ordered [`Total`] values.
    /// # Examples
    /// ```
    /// use range_set_blaze::{RangeSetBlaze, TotalF64};
    ///
    /// let short = RangeSetBlaze::from_iter(TotalF64::values([1.0, 2.0, 3.0, 4.0]));
    /// let long = RangeSetBlaze::from_iter([TotalF64::new(1.0), TotalF64::new(2.0), TotalF64::new(3.0), TotalF64::new(4.0)]);
    /// assert_eq!(short, long);
    /// ```
    pub fn values<I>(values: I) -> impl Iterator<Item = Self>
    where
        I: IntoIterator<Item = T>,
    {
        values.into_iter().map(Self)
    }

    /// Views primitive values as ordered [`Total`] values.
    ///
    /// "Primitive" here means Rust's built-in float type (e.g. `f64`).
    ///
    /// This runs in `O(1)` and does not allocate.
    /// # Examples
    /// ```
    /// use range_set_blaze::{RangeSetBlaze, TotalF64};
    ///
    /// let short = RangeSetBlaze::from_iter(TotalF64::from_primitive_slice(&[1.0, 2.0, 3.0, 4.0]));
    /// let long = RangeSetBlaze::from_iter([TotalF64::new(1.0), TotalF64::new(2.0), TotalF64::new(3.0), TotalF64::new(4.0)]);
    /// assert_eq!(short, long);
    /// ```
    #[must_use]
    pub const fn from_primitive_slice(values: &[T]) -> &[Self] {
        // SAFETY: Total is #[repr(transparent)] over T, making `&[T]`
        // and `&[Total]` entirely interchangeable in layout and lifetimes.
        unsafe { mem::transmute::<&[T], &[Self]>(values) }
    }
}

/// Extension trait for viewing a slice of [`Total`] values as primitive values.
pub trait TotalSliceExt<T: TotalFloat> {
    /// Views [`Total`] values as primitive values.
    ///
    /// "Primitive" here means Rust's built-in float type (e.g. `f64`).
    ///
    /// This runs in `O(1)` and does not allocate.
    /// # Examples
    /// ```
    /// use range_set_blaze::TotalF64;
    /// use range_set_blaze::total::TotalSliceExt;
    ///
    /// let totals = [TotalF64::new(1.0), TotalF64::new(2.0), TotalF64::new(3.0)];
    /// assert_eq!(&[1.0, 2.0, 3.0], totals.as_primitive_slice());
    /// ```
    fn as_primitive_slice(&self) -> &[T];
}

impl<T: TotalFloat> TotalSliceExt<T> for [Total<T>] {
    fn as_primitive_slice(&self) -> &[T] {
        // SAFETY: Total<T> is #[repr(transparent)] over T, making `&[T]`
        // and `&[Total<T>]` entirely interchangeable in layout and lifetimes.
        unsafe { from_raw_parts(self.as_ptr().cast::<T>(), self.len()) }
    }
}

/// Extension trait for converting an inclusive [`Total`] range into an inclusive primitive
/// range (or a `(start, end)` primitive tuple).
pub trait TotalRangeExt<T: TotalFloat> {
    /// Converts an inclusive [`Total`] range into an inclusive primitive range.
    ///
    /// "Primitive" here means Rust's built-in float type (e.g. `f64`).
    ///
    /// This is the reverse of [`Total::from_primitive_range`].
    ///
    /// # Examples
    /// ```
    /// use range_set_blaze::TotalF64;
    /// use range_set_blaze::total::TotalRangeExt;
    ///
    /// let range = TotalF64::new(3.0)..=TotalF64::new(5.0);
    /// assert_eq!(range.into_primitive_range(), 3.0..=5.0);
    /// ```
    #[must_use]
    fn into_primitive_range(self) -> RangeInclusive<T>;

    /// Converts an inclusive [`Total`] range into a `(start, end)` tuple of primitive values.
    ///
    /// "Primitive" here means Rust's built-in float type (e.g. `f64`).
    ///
    /// Mirrors [`RangeInclusive::into_inner`] from the standard library, which unwraps a
    /// range into its `(start, end)` tuple; this additionally converts each endpoint to its
    /// primitive type.
    ///
    /// # Examples
    /// ```
    /// use range_set_blaze::TotalF64;
    /// use range_set_blaze::total::TotalRangeExt;
    ///
    /// let range = TotalF64::new(3.0)..=TotalF64::new(5.0);
    /// assert_eq!(range.into_primitive_inner(), (3.0, 5.0));
    /// ```
    #[must_use]
    fn into_primitive_inner(self) -> (T, T);
}

impl<T: TotalFloat> TotalRangeExt<T> for RangeInclusive<Total<T>> {
    fn into_primitive_range(self) -> RangeInclusive<T> {
        let (start, end) = self.into_primitive_inner();
        start..=end
    }

    fn into_primitive_inner(self) -> (T, T) {
        let (start, end) = self.into_inner();
        (start.into_inner(), end.into_inner())
    }
}

impl<T: TotalFloat> PartialEq for Total<T> {
    fn eq(&self, other: &Self) -> bool {
        T::total_cmp(self.0, other.0) == Ordering::Equal
    }
}

impl<T: TotalFloat> Eq for Total<T> {}

impl<T: TotalFloat> PartialOrd for Total<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<T: TotalFloat> Ord for Total<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        T::total_cmp(self.0, other.0)
    }
}

impl<T: TotalFloat> Hash for Total<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        T::hash(self.0, state);
    }
}

impl<T: TotalFloat> Integer for Total<T> {
    type SafeLen = T::SafeLen;

    #[inline]
    fn checked_add_one(self) -> Option<Self> {
        self.checked_after()
    }

    // This moves to the next representable float in total_cmp order, not a numeric + 1.0.
    #[inline]
    fn add_one(self) -> Self {
        self.after()
    }

    #[inline]
    // This moves to the previous representable float in total_cmp order, not a numeric - 1.0.
    fn sub_one(self) -> Self {
        self.before()
    }

    #[inline]
    fn assign_sub_one(&mut self) {
        *self = self.before();
    }

    // Ideally, we would `impl std::iter::Step for TotalF64` and just call Range::next(), but that's still experimental.
    #[inline]
    fn range_next(range: &mut RangeInclusive<Self>) -> Option<Self> {
        if range.is_empty() {
            None
        } else if range.start() == range.end() && *range.start() == Self::MAX {
            // Preserve the exhausted range sentinel without calling `after()` on MAX.
            let next = *range.start();
            *range = next..=range.end().before();
            Some(next)
        } else {
            let next = *range.start();
            *range = (next.after())..=*range.end();
            Some(next)
        }
    }

    #[inline]
    fn range_next_back(range: &mut RangeInclusive<Self>) -> Option<Self> {
        if range.is_empty() {
            None
        } else if range.start() == range.end() && *range.start() == Self::MIN {
            // Preserve the exhausted range sentinel without calling `before()` on MIN.
            let last = *range.end();
            *range = last.after()..=last;
            Some(last)
        } else {
            let last = *range.end();
            *range = *range.start()..=last.before();
            Some(last)
        }
    }

    #[inline]
    fn min_value() -> Self {
        Self::MIN
    }

    #[inline]
    fn max_value() -> Self {
        Self::MAX
    }

    #[cfg(feature = "from_slice")]
    #[inline]
    fn from_slice(slice: impl AsRef<[Self]>) -> RangeSetBlaze<Self> {
        // TODO Investigate applying the ordered float transform in SIMD chunks here.
        // no way to do the fancy thing
        RangeSetBlaze::from_iter(slice.as_ref())
    }

    fn safe_len(r: &RangeInclusive<Self>) -> Self::SafeLen {
        let (start, end) = r.clone().into_primitive_inner();
        T::prim_safe_len(start, end)
    }

    fn safe_len_to_f64_lossy(len: Self::SafeLen) -> f64 {
        T::safe_len_to_f64_lossy(len)
    }

    fn f64_to_safe_len_lossy(f: f64) -> Self::SafeLen {
        T::f64_to_safe_len_lossy(f)
    }

    fn inclusive_end_from_start(self, b: Self::SafeLen) -> Self {
        self.inclusive_end_from_start(b)
    }

    fn start_from_inclusive_end(self, b: Self::SafeLen) -> Self {
        self.start_from_inclusive_end(b)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Integer;
    use crate::float::total_float::{
        from_ordered_32, from_ordered_64, to_ordered_32, to_ordered_64,
    };
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
                assert_eq!(tf64(left).cmp(&tf64(right)), left.total_cmp(&right));
            }
        }
    }

    #[test]
    fn equality_agrees_with_total_cmp() {
        assert_ne!(tf64(-0.0), tf64(0.0));
        assert_eq!(tf64(f64::NAN), tf64(f64::NAN));
    }

    #[test]
    fn equal_values_hash_equally() {
        let left = hash(tf64(f64::NAN));
        let right = hash(tf64(f64::NAN));

        assert_eq!(left, right);
    }

    #[test]
    fn converts_ranges() {
        assert_eq!(
            TotalF64::from_primitive_range(10.0..=20.0),
            tf64(10.0)..=tf64(20.0)
        );
        assert_eq!(
            TotalF64::from_primitive_ranges([10.0..=20.0, 30.0..=40.0]).collect::<Vec<_>>(),
            vec![tf64(10.0)..=tf64(20.0), tf64(30.0)..=tf64(40.0)]
        );
    }

    #[test]
    fn after_and_before_step_through_zero_in_total_order() {
        assert_eq!(tf64(-0.0).after(), tf64(0.0));
        assert_eq!(tf64(0.0).before(), tf64(-0.0));
        assert_eq!(tf64(0.0).after(), tf64(f64::from_bits(1)));
        assert_eq!(
            tf64(-0.0).before(),
            tf64(f64::from_bits(0x8000_0000_0000_0001))
        );
    }

    #[test]
    fn checked_after_and_before_are_not_wrapping() {
        assert_eq!(TotalF64::MAX.checked_after(), None);
        assert_eq!(TotalF64::MIN.checked_before(), None);
    }

    #[test]
    #[cfg(debug_assertions)]
    #[should_panic(expected = "after() called on maximum value")]
    fn total_after_panics_at_max_in_debug() {
        let _ = TotalF64::MAX.after();
    }

    #[test]
    #[cfg(not(debug_assertions))]
    fn total_after_wraps_at_max_in_release() {
        assert_eq!(TotalF64::MAX.after(), TotalF64::MIN);
    }

    #[test]
    #[cfg(debug_assertions)]
    #[should_panic(expected = "before() called on minimum value")]
    fn total_before_panics_at_min_in_debug() {
        let _ = TotalF64::MIN.before();
    }

    #[test]
    #[cfg(not(debug_assertions))]
    fn total_before_wraps_at_min_in_release() {
        assert_eq!(TotalF64::MIN.before(), TotalF64::MAX);
    }

    #[test]
    fn stable_ordered_round_trips() {
        let edge_f64 = [
            0,
            1,
            u64::MAX,
            0x7ff0_0000_0000_0000,
            0xfff0_0000_0000_0000,
            0x7ff8_0000_0000_0001,
            0xfff8_0000_0000_0001,
        ];
        for bits in edge_f64 {
            let value = f64::from_bits(bits);
            assert_eq!(from_ordered_64(to_ordered_64(value)).to_bits(), bits);
        }

        let edge_f32 = [
            0,
            1,
            u32::MAX,
            0x7f80_0000,
            0xff80_0000,
            0x7fc0_0001,
            0xffc0_0001,
        ];
        for bits in edge_f32 {
            let value = f32::from_bits(bits);
            assert_eq!(from_ordered_32(to_ordered_32(value)).to_bits(), bits);
        }

        let mut state = 0x9e37_79b9_u64;
        for _ in 0..10_000 {
            state = state
                .wrapping_mul(6_364_136_223_846_793_005)
                .wrapping_add(1);
            let value = f64::from_bits(state);
            assert_eq!(from_ordered_64(to_ordered_64(value)).to_bits(), state);
            let bytes = state.to_le_bytes();
            let bits = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
            let value = f32::from_bits(bits);
            assert_eq!(from_ordered_32(to_ordered_32(value)).to_bits(), bits);
        }
    }

    #[test]
    fn after_and_before_step_around_infinities() {
        assert_eq!(tf64(f64::MAX).after(), tf64(f64::INFINITY));
        assert_eq!(tf64(f64::INFINITY).before(), tf64(f64::MAX));
        assert_eq!(tf64(f64::NEG_INFINITY).after(), tf64(-f64::MAX));
        assert_eq!(tf64(-f64::MAX).before(), tf64(f64::NEG_INFINITY));
    }

    #[test]
    fn checked_after_and_before_stop_at_total_order_boundaries() {
        assert_eq!(TotalF64::MIN.checked_before(), None);
        assert_eq!(TotalF64::MAX.checked_after(), None);
        assert_eq!(TotalF64::MIN.checked_after(), Some(TotalF64::MIN.after()));
        assert_eq!(TotalF64::MAX.checked_before(), Some(TotalF64::MAX.before()));
    }

    #[test]
    fn min_and_max_are_total_order_boundaries() {
        let values = [
            tf64(f64::NEG_INFINITY),
            tf64(-f64::MAX),
            tf64(-1.0),
            tf64(-0.0),
            tf64(0.0),
            tf64(1.0),
            tf64(f64::MAX),
            tf64(f64::INFINITY),
            tf64(f64::NAN),
            tf64(f64::from_bits(0x7ff8_0000_0000_0001)),
            tf64(f64::from_bits(0xfff8_0000_0000_0001)),
        ];

        for value in values {
            assert!(TotalF64::MIN <= value);
            assert!(value <= TotalF64::MAX);
        }
    }

    #[test]
    fn after_and_before_are_neighbors_in_total_order() {
        let values = [
            tf64(f64::NEG_INFINITY),
            tf64(-f64::MAX),
            tf64(-1.0),
            tf64(-0.0),
            tf64(0.0),
            tf64(1.0),
            tf64(f64::MAX),
            tf64(f64::INFINITY),
            tf64(f64::NAN),
            tf64(f64::from_bits(0x7ff8_0000_0000_0001)),
            tf64(f64::from_bits(0xfff8_0000_0000_0001)),
        ];

        for value in values {
            assert_eq!(value.after().before(), value);
            assert_eq!(value.before().after(), value);
        }
    }

    #[test]
    fn adjacency_laws_cover_f32_and_f64_edges() {
        macro_rules! check {
            ($wrapper:ident, $constructor:ident, $zero:expr, $negative_subnormal:expr, $positive_subnormal:expr, $min:expr, $max:expr) => {
                let values = [
                    $constructor($zero),
                    $constructor($negative_subnormal),
                    $constructor($positive_subnormal),
                    $constructor(-1.0),
                    $constructor(1.0),
                    $constructor($min),
                    $constructor($max),
                    $constructor(f32::INFINITY),
                    $constructor(f32::NAN),
                ];
                for value in values {
                    assert_eq!(value.after().before(), value);
                    assert_eq!(value.before().after(), value);
                }
                assert_eq!($wrapper::MIN.checked_before(), None);
                assert_eq!($wrapper::MAX.checked_after(), None);
            };
        }
        check!(
            TotalF32,
            tf32,
            0.0_f32,
            -f32::from_bits(1),
            f32::from_bits(1),
            f32::MIN,
            f32::MAX
        );

        let values = [
            tf64(-0.0),
            tf64(0.0),
            tf64(-f64::from_bits(1)),
            tf64(f64::from_bits(1)),
            tf64(-f64::MAX),
            tf64(f64::MAX),
            tf64(f64::NEG_INFINITY),
            tf64(f64::INFINITY),
            tf64(f64::from_bits(0x7ff8_0000_0000_0001)),
        ];
        for value in values {
            assert_eq!(value.after().before(), value);
            assert_eq!(value.before().after(), value);
        }
        assert_eq!(TotalF64::MIN.checked_before(), None);
        assert_eq!(TotalF64::MAX.checked_after(), None);
    }

    #[test]
    fn range_length_laws_cover_f32_and_f64() {
        let start = tf32(-f32::from_bits(1));
        assert_eq!(TotalF32::safe_len(&(start..=start)), 1);
        assert_eq!(TotalF32::safe_len(&(start..=start.after())), 2);
        assert_eq!(
            TotalF32::MAX_SIZE,
            TotalF32::safe_len(&(TotalF32::MIN..=TotalF32::MAX))
        );
        let length = 17;
        let end = start.inclusive_end_from_start(length);
        assert_eq!(end.start_from_inclusive_end(length), start);

        let start = tf64(-f64::from_bits(1));
        assert_eq!(TotalF64::safe_len(&(start..=start)), 1);
        assert_eq!(TotalF64::safe_len(&(start..=start.after())), 2);
        assert_eq!(
            TotalF64::MAX_SIZE,
            TotalF64::safe_len(&(TotalF64::MIN..=TotalF64::MAX))
        );
        let length = 17;
        let end = start.inclusive_end_from_start(length);
        assert_eq!(end.start_from_inclusive_end(length), start);
    }

    #[cfg(feature = "float_nightly_experimental")]
    #[test]
    fn f16_total_adjacency_and_lengths_are_exhaustive() {
        for bits in 0..=u16::MAX {
            let value = TotalF16::new(f16::from_bits(bits));
            if value != TotalF16::MAX {
                assert_eq!(value.after().before(), value);
            }
            if value != TotalF16::MIN {
                assert_eq!(value.before().after(), value);
            }
            assert_eq!(TotalF16::safe_len(&(value..=value)), 1);
        }
        assert_eq!(
            TotalF16::MAX_SIZE,
            TotalF16::safe_len(&(TotalF16::MIN..=TotalF16::MAX))
        );
    }

    fn hash(value: TotalF64) -> u64 {
        let mut hasher = DefaultHasher::new();
        value.hash(&mut hasher);
        hasher.finish()
    }
    #[test]
    #[cfg(feature = "float_nightly_experimental")]
    fn ordered_round_trip() {
        use crate::float::total_float::from_ordered_16;
        use crate::float::total_float::to_ordered_16;
        for x in i16::MIN..=i16::MAX {
            assert_eq!(to_ordered_16(from_ordered_16(x)), x);
        }
    }
}
