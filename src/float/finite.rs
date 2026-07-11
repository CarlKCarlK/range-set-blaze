//! Finite is a floating point type, suitable for use in ranges. Only finite values are valid.
//!
//! Ordering and other semantics are as per normal floating point comparisons.
//!
//! Enable with `float_experimental` (stable, `FiniteF32`/`FiniteF64`) and
//! `float_nightly_experimental` (nightly, adds `FiniteF16`/`FiniteF128`).

use core::{
    cmp::Ordering,
    fmt::Debug,
    hash::{Hash, Hasher},
    mem,
    ops::RangeInclusive,
    slice::from_raw_parts,
};

use super::finite_float::FiniteFloat;

use crate::Integer;
#[cfg(feature = "from_slice")]
use crate::RangeSetBlaze;
use num_traits::Zero;

/// Total ordered f64, with `-0.0` normalized to `+0.0`, and excluding NaN and infinities.
pub type FiniteF64 = Finite<f64>;
/// Total ordered f32, with `-0.0` normalized to `+0.0`, and excluding NaN and infinities.
pub type FiniteF32 = Finite<f32>;
/// Total ordered f16, with `-0.0` normalized to `+0.0`, and excluding NaN and infinities.
#[cfg(feature = "float_nightly_experimental")]
pub type FiniteF16 = Finite<f16>;
/// Total ordered f128, with `-0.0` normalized to `+0.0`, and excluding NaN and infinities.
#[cfg(feature = "float_nightly_experimental")]
pub type FiniteF128 = Finite<f128>;

/// Construct a [`FiniteF64`] from an `f64`. Shorthand for [`FiniteF64::new`]
#[must_use]
pub const fn ff64(x: f64) -> FiniteF64 {
    finite_f64(x)
}

/// Construct a [`FiniteF32`] from an `f32`. Shorthand for [`FiniteF32::new`]
#[must_use]
pub const fn ff32(x: f32) -> FiniteF32 {
    finite_f32(x)
}

/// Construct a [`FiniteF16`] from an `f16`. Shorthand for [`FiniteF16::new`]
#[cfg(feature = "float_nightly_experimental")]
#[must_use]
pub const fn ff16(x: f16) -> FiniteF16 {
    finite_f16(x)
}

/// Construct a [`FiniteF128`] from an `f128`. Shorthand for [`FiniteF128::new`]
#[cfg(feature = "float_nightly_experimental")]
#[must_use]
pub const fn ff128(x: f128) -> FiniteF128 {
    finite_f128(x)
}

// TODO When const trait methods are stable, make the generic Finite constructors and other
// eligible methods const, then have these shorthands call Finite::new directly. That will also
// let their negative-zero normalization share `FiniteFloat::normalize` with runtime paths.
macro_rules! finite_const_constructor {
    ($name:ident, $primitive:ty, $finite:ty) => {
        const fn $name(x: $primitive) -> $finite {
            assert!(x.is_finite(), "Finite type requires a finite value");
            let normalized = if x == 0.0 && x.is_sign_negative() {
                0.0
            } else {
                x
            };
            Finite(normalized)
        }
    };
}

finite_const_constructor!(finite_f64, f64, FiniteF64);
finite_const_constructor!(finite_f32, f32, FiniteF32);
#[cfg(feature = "float_nightly_experimental")]
finite_const_constructor!(finite_f16, f16, FiniteF16);
#[cfg(feature = "float_nightly_experimental")]
finite_const_constructor!(finite_f128, f128, FiniteF128);

/// Experimental: A transparent wrapper around [`f64`] and friends with total ordering.
///
/// Comparison, equality, and hashing all agree with `total_cmp` after zero normalization.
///
/// # Basic Usage
/// ```
/// use range_set_blaze::{RangeSetBlaze, FiniteF64, FiniteF32};
/// let set = RangeSetBlaze::from_iter([FiniteF64::new(3.0)..=FiniteF64::new(5.0)]);
/// assert!(set.contains(FiniteF64::new(3.1)));
/// assert!(!set.contains(FiniteF64::new(2.9)));
///
/// let set = RangeSetBlaze::from(FiniteF64::from_primitive_range(3.0..=5.0));
/// assert!(set.contains(FiniteF64::new(4.9)));
/// assert!(!set.contains(FiniteF64::new(5.1)));
///
/// let set = RangeSetBlaze::from_iter(FiniteF32::from_primitive_ranges([3.0..=5.0, 7.0..=9.0]));
/// assert!(set.contains(FiniteF32::new(4.0)));
/// assert!(!set.contains(FiniteF32::new(6.0)));
/// ```
///
/// # Enabling
///
/// This type is experimental and must be enabled with the `float_experimental` feature.
/// ```bash
/// cargo add range-set-blaze --features "float_experimental"
/// ```
/// That provides the `FiniteF32` and `FiniteF64` types.
///
/// If you're building with nightly, you can instead use the `float_nightly_experimental` feature.
/// ```bash
/// cargo add range-set-blaze --features "float_nightly_experimental"
/// ```
/// To also use the `FiniteF16` and `FiniteF128` types.
#[repr(transparent)]
#[derive(Copy, Clone, Default, Debug)]
pub struct Finite<T: FiniteFloat>(T);

impl<T: FiniteFloat> Finite<T> {
    /// The minimum value that can be represented by the type.\
    /// Maps directly to `crate::Integer::min_value()`
    ///
    /// # Examples
    /// ```
    /// use range_set_blaze::FiniteF64;
    ///
    /// assert_eq!(FiniteF64::MIN, FiniteF64::new(f64::MIN));
    /// ```
    pub const MIN: Self = Self(T::MIN);

    /// The maximum value that can be represented by the type.\
    /// Maps directly to [`crate::Integer::max_value()`]
    ///
    /// # Examples
    /// ```
    /// use range_set_blaze::FiniteF64;
    ///
    /// assert_eq!(FiniteF64::MAX, FiniteF64::new(f64::MAX));
    /// ```
    pub const MAX: Self = Self(T::MAX);

    /// The maximum possible size of a range, i.e. the size if `[MIN..=MAX]`
    /// For `Finite` types, this is unusual because NaN and infinity values are excluded, and
    /// `-0.0` and `+0.0` share one slot after normalization.
    ///
    /// # Examples
    /// ```
    /// use range_set_blaze::FiniteF32;
    ///
    /// assert_eq!(FiniteF32::MAX_SIZE, 0xFF00_0000_u32 - 1);
    /// ```
    pub const MAX_SIZE: T::SafeLen = T::MAX_SIZE;

    /// Creates a new [`Finite`] from a primitive float.
    /// Only finite values are legal
    ///
    /// # Examples
    /// ```
    /// use range_set_blaze::FiniteF64;
    ///
    /// let _ = FiniteF64::new(1.0);
    /// ```
    /// # Panics
    ///
    /// Panics if `x.is_finite()` returns false
    #[must_use]
    pub fn new(x: T) -> Self {
        Self::try_new(x).expect("Finite type requires a finite value")
    }

    /// Creates a new [`Finite`] from a primitive float.
    ///
    /// Returns `None` if the float is not finite (NaN or infinity).
    ///
    /// # Examples
    /// ```
    /// use range_set_blaze::FiniteF64;
    ///
    /// assert_eq!(FiniteF64::try_new(1.0), Some(FiniteF64::new(1.0)));
    /// assert_eq!(FiniteF64::try_new(f64::NAN), None);
    /// ```
    #[must_use]
    pub fn try_new(x: T) -> Option<Self> {
        // SAFETY: `T::is_finite` rules out NaN/infinity, and `T::normalize` canonicalizes -0.0.
        T::is_finite(x).then(|| unsafe { Self::new_unchecked(T::normalize(x)) })
    }

    /// Creates a new [`Finite`] from a primitive float without validating it.
    ///
    /// This is the unchecked building block every validating constructor in this module
    /// (`new`, `try_new`, `from_primitive_range`, `values`,
    /// `from_primitive_slice`, ...) is defined in terms
    /// of. Prefer those; only reach for this when you have already independently established
    /// the safety precondition below and need to skip the redundant check.
    ///
    /// # Safety
    ///
    /// The caller must guarantee that:
    /// - `x` is finite: not NaN, not `+/-infinity`.
    /// - `x` is not `-0.0`: zero must already be canonicalized to `+0.0`.
    ///
    /// [`Finite`] has a public type invariant ("only finite values, with zero canonicalized to
    /// `+0.0`, are legal"). Even though today's implementation would only produce incorrect
    /// results (wrong `MAX_SIZE`, a duplicated zero slot, `after`/`before` landing somewhere
    /// unexpected) rather than immediate undefined behavior if this precondition is violated,
    /// safe code must never be able to construct a value that breaks it. This preserves the
    /// option for this crate, and downstream code, to rely on the invariant in future
    /// (potentially unsafe) abstractions without an audit of every safe caller.
    #[must_use]
    pub const unsafe fn new_unchecked(x: T) -> Self {
        Self(x)
    }

    /// Computes `self + (b - 1)` where `b` is of type `SafeLen`.
    ///
    /// # Panics
    /// Panics if `b` is not small enough that the result stays within range for `T`
    /// (checked unconditionally, in both debug and release builds, so safe code can
    /// never construct a `Finite` value that breaks its invariant this way).
    #[must_use]
    pub fn inclusive_end_from_start(self, b: T::SafeLen) -> Self {
        let max_len = T::prim_safe_len(self.0, T::MAX);
        assert!(
            !b.is_zero() && b <= max_len,
            "b must be in range 1..=max_len"
        );
        Self(T::inclusive_end_from_start(self.0, b))
    }

    /// Computes `self - (b - 1)` where `b` is of type `SafeLen`.
    ///
    /// # Panics
    /// Panics if `b` is not small enough that the result stays within range for `T`
    /// (checked unconditionally, in both debug and release builds, so safe code can
    /// never construct a `Finite` value that breaks its invariant this way).
    #[must_use]
    pub fn start_from_inclusive_end(self, b: T::SafeLen) -> Self {
        let max_len = T::prim_safe_len(T::MIN, self.0);
        assert!(
            !b.is_zero() && b <= max_len,
            "b must be in range 1..=max_len"
        );
        Self(T::start_from_inclusive_end(self.0, b))
    }

    /// Returns the wrapped value.
    ///
    /// # Examples
    /// ```
    /// use range_set_blaze::FiniteF64;
    ///
    /// assert_eq!(FiniteF64::new(42.0).into_inner(), 42.0);
    /// ```
    #[must_use]
    pub const fn into_inner(self) -> T {
        self.0
    }

    /// Returns the next float, in total order.
    ///
    /// # Examples
    /// ```
    /// use range_set_blaze::FiniteF64;
    ///
    /// assert_eq!(FiniteF64::new(42.0).after().before().into_inner(), 42.0);
    /// ```
    ///
    /// # Panics
    ///
    /// Panics if `self` is the maximum value (checked unconditionally, in both debug
    /// and release builds, so safe code can never construct a `Finite` value that
    /// breaks its invariant this way).
    #[must_use]
    pub fn after(self) -> Self {
        assert!(self != Self::MAX, "after() called on maximum value");
        Self(T::normalize(T::after(self.0)))
    }

    /// Returns the previous float, in total order.
    ///
    /// # Examples
    /// ```
    /// use range_set_blaze::FiniteF64;
    ///
    /// assert_eq!(FiniteF64::new(42.0).before().after().into_inner(), 42.0);
    /// ```
    ///
    /// # Panics
    ///
    /// Panics if `self` is the minimum value (checked unconditionally, in both debug
    /// and release builds, so safe code can never construct a `Finite` value that
    /// breaks its invariant this way).
    #[must_use]
    pub fn before(self) -> Self {
        assert!(self != Self::MIN, "before() called on minimum value");
        Self(T::normalize(T::before(self.0)))
    }

    /// Returns the next float, in total order.
    ///
    /// Returns [`None`] if `self` is the maximum value.
    ///
    /// # Examples
    /// ```
    /// use range_set_blaze::FiniteF64;
    ///
    /// let value = FiniteF64::new(42.0);
    /// assert_eq!(value.checked_after(), Some(value.after()));
    /// let value = FiniteF64::MAX;
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

    /// Returns the previous float, in total order.
    ///
    /// Returns [`None`] if `self` is the minimum value.
    ///
    /// # Examples
    /// ```
    /// use range_set_blaze::FiniteF64;
    ///
    /// let value = FiniteF64::new(42.0);
    /// assert_eq!(value.checked_before(), Some(value.before()));
    /// let value = FiniteF64::MIN;
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

    /// Converts an inclusive primitive range into an inclusive [`Finite`] range.
    ///
    /// "Primitive" here means Rust's built-in float type (e.g. `f64`).
    ///
    ///
    /// # Examples
    /// ```
    /// use range_set_blaze::{RangeSetBlaze, FiniteF64};
    ///
    /// let short = RangeSetBlaze::from(FiniteF64::from_primitive_range(3.0..=5.0));
    /// let long = RangeSetBlaze::from(FiniteF64::new(3.0)..=FiniteF64::new(5.0));
    /// assert_eq!(short, long);
    /// ```
    /// # Panics
    ///
    /// Panics if `start` or `end` is not finite.
    #[must_use]
    pub fn from_primitive_range(range: RangeInclusive<T>) -> RangeInclusive<Self> {
        let (start, end) = range.into_inner();
        Self::new(start)..=Self::new(end)
    }

    /// Converts inclusive primitive ranges into inclusive [`Finite`] ranges.
    ///
    /// "Primitive" here means Rust's built-in float type (e.g. `f64`).
    ///
    ///
    /// # Examples
    /// ```
    /// use range_set_blaze::{RangeSetBlaze, FiniteF64};
    ///
    /// let short = RangeSetBlaze::from_iter(FiniteF64::from_primitive_ranges([1.0..=2.0, 3.0..=4.0]));
    /// let long = RangeSetBlaze::from_iter([FiniteF64::new(1.0)..=FiniteF64::new(2.0), FiniteF64::new(3.0)..=FiniteF64::new(4.0)]);
    /// assert_eq!(short, long);
    /// ```
    /// # Panics
    ///
    /// Panics when the returned iterator is consumed if any range endpoint is not finite.
    pub fn from_primitive_ranges<I>(ranges: I) -> impl Iterator<Item = RangeInclusive<Self>>
    where
        I: IntoIterator<Item = RangeInclusive<T>>,
    {
        ranges.into_iter().map(Self::from_primitive_range)
    }

    /// Convenience method to convert primitive values into ordered [`Finite`] values.
    /// # Examples
    /// ```
    /// use range_set_blaze::{RangeSetBlaze, FiniteF64};
    ///
    /// let short = RangeSetBlaze::from_iter(FiniteF64::values([1.0, 2.0, 3.0, 4.0]));
    /// let long = RangeSetBlaze::from_iter([FiniteF64::new(1.0), FiniteF64::new(2.0), FiniteF64::new(3.0), FiniteF64::new(4.0)]);
    /// assert_eq!(short, long);
    /// ```
    ///
    /// # Panics
    ///
    /// Panics (when iterated) if any value is not finite.
    pub fn values<I>(values: I) -> impl Iterator<Item = Self>
    where
        I: IntoIterator<Item = T>,
    {
        values.into_iter().map(Self::new)
    }

    /// Views primitive values as ordered [`Finite`] values, validating as it goes.
    ///
    /// "Primitive" here means Rust's built-in float type (e.g. `f64`).
    ///
    ///
    /// This runs in `O(n)` (to validate every element) and does not allocate.
    /// # Examples
    /// ```
    /// use range_set_blaze::{RangeSetBlaze, FiniteF64};
    ///
    /// let short = RangeSetBlaze::from_iter(FiniteF64::from_primitive_slice(&[1.0, 2.0, 3.0, 4.0]));
    /// let long = RangeSetBlaze::from_iter([FiniteF64::new(1.0), FiniteF64::new(2.0), FiniteF64::new(3.0), FiniteF64::new(4.0)]);
    /// assert_eq!(short, long);
    /// ```
    ///
    /// # Panics
    ///
    /// Panics if any element is not finite, or is `-0.0` (which can't be normalized to `+0.0`
    /// without copying — see [`Finite::from_primitive_slice_unchecked`] if you need a true
    /// zero-copy view and can guarantee your data already satisfies [`Finite`]'s invariant).
    #[must_use]
    pub fn from_primitive_slice(values: &[T]) -> &[Self] {
        assert!(
            values
                .iter()
                .all(|&v| T::is_finite(v) && !T::is_neg_zero(v)),
            "Finite type requires finite, non-negative-zero values"
        );
        // SAFETY: just validated every element is finite and not -0.0.
        unsafe { Self::from_primitive_slice_unchecked(values) }
    }

    /// Views primitive values as ordered [`Finite`] values, without validating them.
    ///
    /// "Primitive" here means Rust's built-in float type (e.g. `f64`).
    ///
    ///
    /// This runs in `O(1)` and does not allocate.
    ///
    /// # Safety
    ///
    /// The caller must guarantee that every element of `values` is finite (not NaN, not
    /// `+/-infinity`) and not `-0.0` (zero must already be canonicalized to `+0.0`). Because
    /// the returned slice is a live view over the same memory (not a copy), there is no
    /// opportunity to normalize `-0.0` even if the caller wanted to; the data must already be
    /// clean.
    ///
    /// [`Finite`] has a public type invariant that safe code must never be able to break, even
    /// though violating it today would only produce incorrect results (see
    /// [`Finite::new_unchecked`] for the full rationale).
    #[must_use]
    pub const unsafe fn from_primitive_slice_unchecked(values: &[T]) -> &[Self] {
        // SAFETY: Finite is #[repr(transparent)] over T, making `&[T]`
        // and `&[Finite]` entirely interchangeable in layout and lifetimes; the caller is
        // responsible for the value-level invariant per the safety doc above.
        unsafe { mem::transmute::<&[T], &[Self]>(values) }
    }
}

/// Extension trait for viewing a slice of [`Finite`] values as primitive values.
pub trait FiniteSliceExt<T: FiniteFloat> {
    /// Views [`Finite`] values as primitive values.
    ///
    /// "Primitive" here means Rust's built-in float type (e.g. `f64`).
    ///
    ///
    /// This runs in `O(1)` and does not allocate.
    /// # Examples
    /// ```
    /// use range_set_blaze::FiniteF64;
    /// use range_set_blaze::finite::FiniteSliceExt;
    ///
    /// let finites = [FiniteF64::new(1.0), FiniteF64::new(2.0), FiniteF64::new(3.0)];
    /// assert_eq!(&[1.0, 2.0, 3.0], finites.as_primitive_slice());
    /// ```
    fn as_primitive_slice(&self) -> &[T];
}

impl<T: FiniteFloat> FiniteSliceExt<T> for [Finite<T>] {
    fn as_primitive_slice(&self) -> &[T] {
        // SAFETY: Finite<T> is #[repr(transparent)] over T, making `&[T]`
        // and `&[Finite<T>]` entirely interchangeable in layout and lifetimes.
        unsafe { from_raw_parts(self.as_ptr().cast::<T>(), self.len()) }
    }
}

/// Extension trait for converting an inclusive [`Finite`] range into an inclusive primitive
/// range (or a `(start, end)` primitive tuple).
pub trait FiniteRangeExt<T: FiniteFloat> {
    /// Converts an inclusive [`Finite`] range into an inclusive primitive range.
    ///
    /// "Primitive" here means Rust's built-in float type (e.g. `f64`).
    ///
    ///
    /// This is the reverse of [`Finite::from_primitive_range`].
    ///
    /// # Examples
    /// ```
    /// use range_set_blaze::FiniteF64;
    /// use range_set_blaze::finite::FiniteRangeExt;
    ///
    /// let range = FiniteF64::new(3.0)..=FiniteF64::new(5.0);
    /// assert_eq!(range.into_primitive_range(), 3.0..=5.0);
    /// ```
    #[must_use]
    fn into_primitive_range(self) -> RangeInclusive<T>;

    /// Converts an inclusive [`Finite`] range into a `(start, end)` tuple of primitive values.
    ///
    /// "Primitive" here means Rust's built-in float type (e.g. `f64`).
    ///
    ///
    /// Mirrors [`RangeInclusive::into_inner`] from the standard library, which unwraps a
    /// range into its `(start, end)` tuple; this additionally converts each endpoint to its
    /// primitive type.
    ///
    /// # Examples
    /// ```
    /// use range_set_blaze::FiniteF64;
    /// use range_set_blaze::finite::FiniteRangeExt;
    ///
    /// let range = FiniteF64::new(3.0)..=FiniteF64::new(5.0);
    /// assert_eq!(range.into_primitive_inner(), (3.0, 5.0));
    /// ```
    #[must_use]
    fn into_primitive_inner(self) -> (T, T);
}

impl<T: FiniteFloat> FiniteRangeExt<T> for RangeInclusive<Finite<T>> {
    fn into_primitive_range(self) -> RangeInclusive<T> {
        let (start, end) = self.into_primitive_inner();
        start..=end
    }

    fn into_primitive_inner(self) -> (T, T) {
        let (start, end) = self.into_inner();
        (start.into_inner(), end.into_inner())
    }
}

impl<T: FiniteFloat> PartialEq for Finite<T> {
    fn eq(&self, other: &Self) -> bool {
        T::total_cmp(self.0, other.0) == Ordering::Equal
    }
}

impl<T: FiniteFloat> Eq for Finite<T> {}

impl<T: FiniteFloat> PartialOrd for Finite<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<T: FiniteFloat> Ord for Finite<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        T::total_cmp(self.0, other.0)
    }
}

impl<T: FiniteFloat> Hash for Finite<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        T::hash(self.0, state);
    }
}

impl<T: FiniteFloat> Integer for Finite<T> {
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

    // Ideally, we would `impl std::iter::Step for FiniteF64` and just call Range::next(), but that's still experimental.
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
    #[cfg(not(target_arch = "wasm32"))]
    use std::hint::black_box;
    #[cfg(not(target_arch = "wasm32"))]
    use std::panic::{AssertUnwindSafe, catch_unwind};
    use std::vec;
    use std::vec::Vec;

    #[cfg(not(target_arch = "wasm32"))]
    fn panics(f: impl FnOnce()) -> bool {
        catch_unwind(AssertUnwindSafe(f)).is_err()
    }

    // WASM targets currently abort instead of unwinding, so `catch_unwind`
    // cannot observe the expected constructor panics there.
    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn safe_constructors_preserve_finite_invariant() {
        assert_eq!(ff32(-0.0).into_inner().to_bits(), 0);
        assert_eq!(ff64(-0.0).into_inner().to_bits(), 0);
        assert_eq!(FiniteF64::new(-0.0), ff64(0.0));
        assert_eq!(FiniteF64::try_new(-0.0), Some(ff64(0.0)));

        for invalid in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            assert!(panics(|| {
                black_box(FiniteF64::new(invalid));
            }));
            assert_eq!(FiniteF64::try_new(invalid), None);
            assert!(panics(|| drop(FiniteF64::from_primitive_range(
                invalid..=1.0
            ))));
            assert!(panics(|| {
                FiniteF64::values([invalid]).count();
            }));
            assert!(panics(|| {
                black_box(FiniteF64::from_primitive_slice(&[invalid]));
            }));
        }

        assert!(panics(|| {
            black_box(FiniteF64::from_primitive_slice(&[-0.0]));
        }));
        assert!(panics(|| {
            black_box(FiniteF64::from_primitive_slice(&[f64::INFINITY]));
        }));
        assert!(panics(|| {
            black_box(FiniteF64::from_primitive_slice(&[f64::NAN]));
        }));

        let values = [1.0, 2.0, 3.0];
        let finites = FiniteF64::from_primitive_slice(&values);
        assert_eq!(finites.as_primitive_slice(), &values);
        assert_eq!(
            FiniteF64::values(values).collect::<Vec<_>>(),
            vec![ff64(1.0), ff64(2.0), ff64(3.0)]
        );
        assert_eq!(
            FiniteF64::from_primitive_ranges([1.0..=2.0]).collect::<Vec<_>>(),
            vec![ff64(1.0)..=ff64(2.0)]
        );
    }

    #[test]
    fn ordering_agrees_with_total_cmp() {
        let values = [-f64::MAX, -1.0, 0.0, 1.0, f64::MAX];

        for left in values {
            for right in values {
                assert_eq!(ff64(left).cmp(&ff64(right)), left.total_cmp(&right));
            }
        }
        assert_ne!(ff64(0.0).cmp(&ff64(-0.0)), 0.0_f64.total_cmp(&-0.0));
    }

    #[test]
    fn converts_ranges() {
        assert_eq!(
            FiniteF64::from_primitive_range(10.0..=20.0),
            ff64(10.0)..=ff64(20.0)
        );
        assert_eq!(
            FiniteF64::from_primitive_ranges([10.0..=20.0, 30.0..=40.0]).collect::<Vec<_>>(),
            vec![ff64(10.0)..=ff64(20.0), ff64(30.0)..=ff64(40.0)]
        );
    }

    #[test]
    fn after_and_before_step_through_zero_in_total_order() {
        assert_eq!(ff64(-0.0), ff64(0.0));
        assert_ne!(ff64(0.0).before(), ff64(-0.0));
        assert_eq!(ff64(0.0).after(), ff64(f64::from_bits(1)));
        assert_eq!(
            ff64(0.0).before(),
            ff64(f64::from_bits(0x8000_0000_0000_0001))
        );
    }

    #[test]
    fn after_and_before_panic_at_boundaries_in_all_build_modes() {
        assert_eq!(FiniteF64::MAX.checked_after(), None);
        assert_eq!(FiniteF64::MIN.checked_before(), None);
    }

    #[test]
    #[should_panic(expected = "b must be in range 1..=max_len")]
    fn finite_endpoint_offset_cannot_leave_domain() {
        let _ = FiniteF32::MAX.inclusive_end_from_start(2);
    }

    #[test]
    #[should_panic(expected = "after() called on maximum value")]
    fn after_panics_at_max() {
        let _ = FiniteF64::MAX.after();
    }

    #[test]
    #[should_panic(expected = "before() called on minimum value")]
    fn before_panics_at_min() {
        let _ = FiniteF64::MIN.before();
    }

    #[test]
    fn checked_after_and_before_stop_at_total_order_boundaries() {
        assert_eq!(FiniteF64::MIN.checked_before(), None);
        assert_eq!(FiniteF64::MAX.checked_after(), None);
        assert_eq!(FiniteF64::MIN.checked_after(), Some(FiniteF64::MIN.after()));
        assert_eq!(
            FiniteF64::MAX.checked_before(),
            Some(FiniteF64::MAX.before())
        );
    }

    #[test]
    fn min_and_max_are_total_order_boundaries() {
        let values = [
            ff64(-f64::MAX),
            ff64(-1.0),
            ff64(-0.0),
            ff64(0.0),
            ff64(1.0),
            ff64(f64::MAX),
        ];

        for value in values {
            assert!(FiniteF64::MIN <= value);
            assert!(value <= FiniteF64::MAX);
        }
    }

    #[test]
    fn after_and_before_are_neighbors_in_total_order() {
        let values = [
            ff64(f64::MIN),
            ff64(-f64::MAX),
            ff64(-1.0),
            ff64(-0.0),
            ff64(0.0),
            ff64(1.0),
            ff64(f64::MAX),
        ];

        for value in values {
            if value != ff64(f64::MAX) {
                assert_eq!(value.after().before(), value);
            }
            if value != ff64(f64::MIN) {
                assert_eq!(value.before().after(), value);
            }
        }
    }

    #[test]
    fn adjacency_laws_cover_f32_and_f64_edges() {
        macro_rules! check {
            ($name:ident, $zero:expr, $negative_subnormal:expr, $positive_subnormal:expr, $min:expr, $max:expr) => {{
                let values = [
                    ff32($zero),
                    ff32($negative_subnormal),
                    ff32($positive_subnormal),
                    ff32(-1.0),
                    ff32(1.0),
                    ff32($min),
                    ff32($max),
                ];
                for value in values {
                    if value != FiniteF32::MAX {
                        assert_eq!(value.after().before(), value);
                    }
                    if value != FiniteF32::MIN {
                        assert_eq!(value.before().after(), value);
                    }
                }
                assert_eq!(FiniteF32::MIN.checked_before(), None);
                assert_eq!(FiniteF32::MAX.checked_after(), None);
                assert_eq!(ff32($negative_subnormal).after(), ff32($zero));
                assert_eq!(ff32($zero).after(), ff32($positive_subnormal));
                let _ = stringify!($name);
            }};
        }

        check!(
            f32_edges,
            0.0_f32,
            -f32::from_bits(1),
            f32::from_bits(1),
            f32::MIN,
            f32::MAX
        );

        let values = [
            ff64(-f64::from_bits(1)),
            ff64(0.0),
            ff64(f64::from_bits(1)),
            ff64(-1.0),
            ff64(1.0),
            FiniteF64::MIN,
            FiniteF64::MAX,
        ];
        for value in values {
            if value != FiniteF64::MAX {
                assert_eq!(value.after().before(), value);
            }
            if value != FiniteF64::MIN {
                assert_eq!(value.before().after(), value);
            }
        }
        assert_eq!(FiniteF64::MIN.checked_before(), None);
        assert_eq!(FiniteF64::MAX.checked_after(), None);
        assert_eq!(ff64(-f64::from_bits(1)).after(), ff64(0.0));
        assert_eq!(ff64(0.0).after(), ff64(f64::from_bits(1)));
    }

    #[test]
    fn range_length_laws_cover_f32_and_f64() {
        let start = ff32(-f32::from_bits(1));
        let end = ff32(f32::from_bits(1));
        assert_eq!(FiniteF32::safe_len(&(start..=start)), 1);
        assert_eq!(FiniteF32::safe_len(&(start..=start.after())), 2);
        assert_eq!(FiniteF32::safe_len(&(start..=end)), 3);
        assert_eq!(
            FiniteF32::MAX_SIZE,
            FiniteF32::safe_len(&(FiniteF32::MIN..=FiniteF32::MAX))
        );
        let length = 17;
        let endpoint = start.inclusive_end_from_start(length);
        assert_eq!(endpoint.start_from_inclusive_end(length), start);
        assert_eq!(start.inclusive_end_from_start(length), endpoint);

        let start = ff64(-f64::from_bits(1));
        let end = ff64(f64::from_bits(1));
        assert_eq!(FiniteF64::safe_len(&(start..=start)), 1);
        assert_eq!(FiniteF64::safe_len(&(start..=start.after())), 2);
        assert_eq!(FiniteF64::safe_len(&(start..=end)), 3);
        assert_eq!(
            FiniteF64::MAX_SIZE,
            FiniteF64::safe_len(&(FiniteF64::MIN..=FiniteF64::MAX))
        );
        let length = 17;
        let endpoint = start.inclusive_end_from_start(length);
        assert_eq!(endpoint.start_from_inclusive_end(length), start);
        assert_eq!(start.inclusive_end_from_start(length), endpoint);
    }

    #[cfg(feature = "float_nightly_experimental")]
    #[test]
    fn f16_finite_adjacency_and_lengths_are_exhaustive() {
        for bits in 0..=u16::MAX {
            let value = f16::from_bits(bits);
            let Some(value) = FiniteF16::try_new(value) else {
                continue;
            };
            if value != FiniteF16::MIN {
                assert_eq!(value.before().after(), value);
            }
            if value != FiniteF16::MAX {
                assert_eq!(value.after().before(), value);
            }
            assert_eq!(FiniteF16::safe_len(&(value..=value)), 1);
        }
        assert_eq!(
            FiniteF16::MAX_SIZE,
            FiniteF16::safe_len(&(FiniteF16::MIN..=FiniteF16::MAX))
        );
    }
}
