//! `NotNan` is a floating point type, suitable for use in ranges. Every value except NaN is
//! valid, including `+infinity` and `-infinity`.
//!
//! Ordering and other semantics are as per normal floating point comparisons.
//!
//! The `NotNanF32`/`NotNanF64` wrappers are available by default. Enable
//! `float_nightly_experimental` on nightly to add `NotNanF16`/`NotNanF128`.

use core::{
    cmp::Ordering,
    fmt::Debug,
    hash::{Hash, Hasher},
    mem,
    ops::RangeInclusive,
    slice::from_raw_parts,
};

use super::not_nan_float::NotNanFloat;

use crate::Integer;
use crate::RangeSetBlaze;
use num_traits::Zero;

/// Total ordered f64, with `-0.0` normalized to `+0.0`, and excluding NaN.
pub type NotNanF64 = NotNan<f64>;
/// Total ordered f32, with `-0.0` normalized to `+0.0`, and excluding NaN.
pub type NotNanF32 = NotNan<f32>;
/// Total ordered f16, with `-0.0` normalized to `+0.0`, and excluding NaN.
#[cfg(feature = "float_nightly_experimental")]
pub type NotNanF16 = NotNan<f16>;
/// Total ordered f128, with `-0.0` normalized to `+0.0`, and excluding NaN.
#[cfg(feature = "float_nightly_experimental")]
pub type NotNanF128 = NotNan<f128>;

/// Construct a [`NotNanF64`] from an `f64`. Shorthand for [`NotNanF64::new`]
#[must_use]
pub const fn nnf64(x: f64) -> NotNanF64 {
    not_nan_f64(x)
}

/// Construct a [`NotNanF32`] from an `f32`. Shorthand for [`NotNanF32::new`]
#[must_use]
pub const fn nnf32(x: f32) -> NotNanF32 {
    not_nan_f32(x)
}

/// Construct a [`NotNanF16`] from an `f16`. Shorthand for [`NotNanF16::new`]
#[cfg(feature = "float_nightly_experimental")]
#[must_use]
pub const fn nnf16(x: f16) -> NotNanF16 {
    not_nan_f16(x)
}

/// Construct a [`NotNanF128`] from an `f128`. Shorthand for [`NotNanF128::new`]
#[cfg(feature = "float_nightly_experimental")]
#[must_use]
pub const fn nnf128(x: f128) -> NotNanF128 {
    not_nan_f128(x)
}

// TODO When const trait methods are stable, make the generic NotNan constructors and other
// eligible methods const, then have these shorthands call NotNan::new directly. That will also
// let their negative-zero normalization share `NotNanFloat::normalize` with runtime paths.
macro_rules! not_nan_const_constructor {
    ($name:ident, $primitive:ty, $not_nan:ty) => {
        const fn $name(x: $primitive) -> $not_nan {
            assert!(!x.is_nan(), "NotNan type requires a non-NaN value");
            let normalized = if x == 0.0 && x.is_sign_negative() {
                0.0
            } else {
                x
            };
            NotNan(normalized)
        }
    };
}

not_nan_const_constructor!(not_nan_f64, f64, NotNanF64);
not_nan_const_constructor!(not_nan_f32, f32, NotNanF32);
#[cfg(feature = "float_nightly_experimental")]
not_nan_const_constructor!(not_nan_f16, f16, NotNanF16);
#[cfg(feature = "float_nightly_experimental")]
not_nan_const_constructor!(not_nan_f128, f128, NotNanF128);

/// A transparent wrapper around [`f64`] and friends with total ordering.
///
/// Comparison, equality, and hashing all agree with `total_cmp` after zero normalization.
/// Every value except NaN is legal, including `+infinity` and `-infinity`.
///
/// # Basic Usage
/// ```
/// use range_set_blaze::{RangeSetBlaze, NotNanF64, NotNanF32};
/// let set = RangeSetBlaze::from_iter([NotNanF64::new(3.0)..=NotNanF64::new(5.0)]);
/// assert!(set.contains(NotNanF64::new(3.1)));
/// assert!(!set.contains(NotNanF64::new(2.9)));
///
/// let set = RangeSetBlaze::from(NotNanF64::from_primitive_range(3.0..=5.0));
/// assert!(set.contains(NotNanF64::new(4.9)));
/// assert!(!set.contains(NotNanF64::new(5.1)));
///
/// let set = RangeSetBlaze::from_iter(NotNanF32::from_primitive_ranges([3.0..=5.0, 7.0..=9.0]));
/// assert!(set.contains(NotNanF32::new(4.0)));
/// assert!(!set.contains(NotNanF32::new(6.0)));
/// ```
///
/// # The Full Non-NaN Domain
///
/// The primitive `-∞..=+∞` range converts to `NotNanF64::MIN..=NotNanF64::MAX`,
/// the complete ordered domain of legal values: every non-NaN `f64`, including
/// both infinities, is in the range.
///
/// ```
/// use range_set_blaze::{NotNanF64, RangeSetBlaze};
///
/// assert_eq!(NotNanF64::MIN, NotNanF64::new(f64::NEG_INFINITY));
/// assert_eq!(NotNanF64::MAX, NotNanF64::new(f64::INFINITY));
///
/// let primitive_domain =
///     NotNanF64::from_primitive_range(f64::NEG_INFINITY..=f64::INFINITY);
/// let full_domain = NotNanF64::MIN..=NotNanF64::MAX;
/// assert_eq!(primitive_domain, full_domain);
///
/// let full_domain = RangeSetBlaze::from(full_domain);
/// assert!(full_domain.contains(NotNanF64::new(f64::NEG_INFINITY)));
/// assert!(full_domain.contains(NotNanF64::new(-42.0)));
/// assert!(full_domain.contains(NotNanF64::new(0.0)));
/// assert!(full_domain.contains(NotNanF64::new(42.0)));
/// assert!(full_domain.contains(NotNanF64::new(f64::INFINITY)));
/// ```
///
/// The stable `NotNanF32` and `NotNanF64` types are available by default.
/// On nightly, enable `float_nightly_experimental` to also use the
/// `NotNanF16` and `NotNanF128` types.
#[repr(transparent)]
#[derive(Copy, Clone, Default, Debug)]
pub struct NotNan<T: NotNanFloat>(T);

impl<T: NotNanFloat> NotNan<T> {
    /// The minimum value that can be represented by the type: negative infinity.\
    /// Maps directly to `crate::Integer::min_value()`
    ///
    /// # Examples
    /// ```
    /// use range_set_blaze::NotNanF64;
    ///
    /// assert_eq!(NotNanF64::MIN, NotNanF64::new(f64::NEG_INFINITY));
    /// ```
    pub const MIN: Self = Self(T::MIN);

    /// The maximum value that can be represented by the type: positive infinity.\
    /// Maps directly to [`crate::Integer::max_value()`]
    ///
    /// # Examples
    /// ```
    /// use range_set_blaze::NotNanF64;
    ///
    /// assert_eq!(NotNanF64::MAX, NotNanF64::new(f64::INFINITY));
    /// ```
    pub const MAX: Self = Self(T::MAX);

    /// The maximum possible size of a range, i.e. the size if `[MIN..=MAX]`
    /// For `NotNan` types, this is unusual because NaN values are excluded, and
    /// `-0.0` and `+0.0` share one slot after normalization.
    ///
    /// # Examples
    /// ```
    /// use range_set_blaze::NotNanF32;
    ///
    /// assert_eq!(NotNanF32::MAX_SIZE, 0xFF00_0000_u32 + 1);
    /// ```
    pub const MAX_SIZE: T::SafeLen = T::MAX_SIZE;

    /// Creates a new [`NotNan`] from a primitive float.
    /// Any value except NaN is legal, including the infinities.
    ///
    /// # Examples
    /// ```
    /// use range_set_blaze::NotNanF64;
    ///
    /// let _ = NotNanF64::new(1.0);
    /// let _ = NotNanF64::new(f64::INFINITY);
    /// ```
    /// # Panics
    ///
    /// Panics if `x` is NaN.
    #[must_use]
    pub fn new(x: T) -> Self {
        Self::try_new(x).expect("NotNan type requires a non-NaN value")
    }

    /// Creates a new [`NotNan`] from a primitive float.
    ///
    /// Returns `None` if the float is NaN.
    ///
    /// # Examples
    /// ```
    /// use range_set_blaze::NotNanF64;
    ///
    /// assert_eq!(NotNanF64::try_new(1.0), Some(NotNanF64::new(1.0)));
    /// assert_eq!(NotNanF64::try_new(f64::INFINITY), Some(NotNanF64::new(f64::INFINITY)));
    /// assert_eq!(NotNanF64::try_new(f64::NAN), None);
    /// ```
    #[must_use]
    pub fn try_new(x: T) -> Option<Self> {
        // SAFETY: `!T::is_nan` rules out NaN, and `T::normalize` canonicalizes -0.0.
        (!T::is_nan(x)).then(|| unsafe { Self::new_unchecked(T::normalize(x)) })
    }

    /// Creates a new [`NotNan`] from a primitive float without validating it.
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
    /// - `x` is not NaN.
    /// - `x` is not `-0.0`: zero must already be canonicalized to `+0.0`.
    ///
    /// [`NotNan`] has a public type invariant ("only non-NaN values, with zero canonicalized to
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
    /// never construct a `NotNan` value that breaks its invariant this way).
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
    /// never construct a `NotNan` value that breaks its invariant this way).
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
    /// use range_set_blaze::NotNanF64;
    ///
    /// assert_eq!(NotNanF64::new(42.0).into_inner(), 42.0);
    /// ```
    #[must_use]
    pub const fn into_inner(self) -> T {
        self.0
    }

    /// Returns the next float, in total order.
    ///
    /// # Examples
    /// ```
    /// use range_set_blaze::NotNanF64;
    ///
    /// assert_eq!(NotNanF64::new(42.0).after().before().into_inner(), 42.0);
    /// ```
    ///
    /// # Panics
    ///
    /// Panics if `self` is the maximum value (checked unconditionally, in both debug
    /// and release builds, so safe code can never construct a `NotNan` value that
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
    /// use range_set_blaze::NotNanF64;
    ///
    /// assert_eq!(NotNanF64::new(42.0).before().after().into_inner(), 42.0);
    /// ```
    ///
    /// # Panics
    ///
    /// Panics if `self` is the minimum value (checked unconditionally, in both debug
    /// and release builds, so safe code can never construct a `NotNan` value that
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
    /// use range_set_blaze::NotNanF64;
    ///
    /// let value = NotNanF64::new(42.0);
    /// assert_eq!(value.checked_after(), Some(value.after()));
    /// let value = NotNanF64::MAX;
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
    /// use range_set_blaze::NotNanF64;
    ///
    /// let value = NotNanF64::new(42.0);
    /// assert_eq!(value.checked_before(), Some(value.before()));
    /// let value = NotNanF64::MIN;
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

    /// Converts an inclusive primitive range into an inclusive [`NotNan`] range.
    ///
    /// "Primitive" here means Rust's built-in float type (e.g. `f64`).
    ///
    ///
    /// # Examples
    /// ```
    /// use range_set_blaze::{RangeSetBlaze, NotNanF64};
    ///
    /// let short = RangeSetBlaze::from(NotNanF64::from_primitive_range(3.0..=5.0));
    /// let long = RangeSetBlaze::from(NotNanF64::new(3.0)..=NotNanF64::new(5.0));
    /// assert_eq!(short, long);
    /// ```
    /// # Panics
    ///
    /// Panics if `start` or `end` is NaN.
    #[must_use]
    pub fn from_primitive_range(range: RangeInclusive<T>) -> RangeInclusive<Self> {
        let (start, end) = range.into_inner();
        Self::new(start)..=Self::new(end)
    }

    /// Converts inclusive primitive ranges into inclusive [`NotNan`] ranges.
    ///
    /// "Primitive" here means Rust's built-in float type (e.g. `f64`).
    ///
    ///
    /// # Examples
    /// ```
    /// use range_set_blaze::{RangeSetBlaze, NotNanF64};
    ///
    /// let short = RangeSetBlaze::from_iter(NotNanF64::from_primitive_ranges([1.0..=2.0, 3.0..=4.0]));
    /// let long = RangeSetBlaze::from_iter([NotNanF64::new(1.0)..=NotNanF64::new(2.0), NotNanF64::new(3.0)..=NotNanF64::new(4.0)]);
    /// assert_eq!(short, long);
    /// ```
    /// # Panics
    ///
    /// Panics when the returned iterator is consumed if any range endpoint is NaN.
    pub fn from_primitive_ranges<I>(ranges: I) -> impl Iterator<Item = RangeInclusive<Self>>
    where
        I: IntoIterator<Item = RangeInclusive<T>>,
    {
        ranges.into_iter().map(Self::from_primitive_range)
    }

    /// Convenience method to convert primitive values into ordered [`NotNan`] values.
    /// # Examples
    /// ```
    /// use range_set_blaze::{RangeSetBlaze, NotNanF64};
    ///
    /// let short = RangeSetBlaze::from_iter(NotNanF64::values([1.0, 2.0, 3.0, 4.0]));
    /// let long = RangeSetBlaze::from_iter([NotNanF64::new(1.0), NotNanF64::new(2.0), NotNanF64::new(3.0), NotNanF64::new(4.0)]);
    /// assert_eq!(short, long);
    /// ```
    ///
    /// # Panics
    ///
    /// Panics (when iterated) if any value is NaN.
    pub fn values<I>(values: I) -> impl Iterator<Item = Self>
    where
        I: IntoIterator<Item = T>,
    {
        values.into_iter().map(Self::new)
    }

    /// Views primitive values as ordered [`NotNan`] values, validating as it goes.
    ///
    /// "Primitive" here means Rust's built-in float type (e.g. `f64`).
    ///
    ///
    /// This runs in `O(n)` (to validate every element) and does not allocate.
    /// # Examples
    /// ```
    /// use range_set_blaze::{RangeSetBlaze, NotNanF64};
    ///
    /// let short = RangeSetBlaze::from_iter(NotNanF64::from_primitive_slice(&[1.0, 2.0, 3.0, 4.0]));
    /// let long = RangeSetBlaze::from_iter([NotNanF64::new(1.0), NotNanF64::new(2.0), NotNanF64::new(3.0), NotNanF64::new(4.0)]);
    /// assert_eq!(short, long);
    /// ```
    ///
    /// # Panics
    ///
    /// Panics if any element is NaN, or is `-0.0` (which can't be normalized to `+0.0`
    /// without copying — see [`NotNan::from_primitive_slice_unchecked`] if you need a true
    /// zero-copy view and can guarantee your data already satisfies [`NotNan`]'s invariant).
    #[must_use]
    pub fn from_primitive_slice(values: &[T]) -> &[Self] {
        assert!(
            values.iter().all(|&v| !T::is_nan(v) && !T::is_neg_zero(v)),
            "NotNan type requires non-NaN, non-negative-zero values"
        );
        // SAFETY: just validated every element is not NaN and not -0.0.
        unsafe { Self::from_primitive_slice_unchecked(values) }
    }

    /// Views primitive values as ordered [`NotNan`] values, without validating them.
    ///
    /// "Primitive" here means Rust's built-in float type (e.g. `f64`).
    ///
    ///
    /// This runs in `O(1)` and does not allocate.
    ///
    /// # Safety
    ///
    /// The caller must guarantee that every element of `values` is not NaN and not `-0.0`
    /// (zero must already be canonicalized to `+0.0`). Because the returned slice is a live
    /// view over the same memory (not a copy), there is no opportunity to normalize `-0.0`
    /// even if the caller wanted to; the data must already be clean.
    ///
    /// [`NotNan`] has a public type invariant that safe code must never be able to break, even
    /// though violating it today would only produce incorrect results (see
    /// [`NotNan::new_unchecked`] for the full rationale).
    #[must_use]
    pub const unsafe fn from_primitive_slice_unchecked(values: &[T]) -> &[Self] {
        // SAFETY: NotNan is #[repr(transparent)] over T, making `&[T]`
        // and `&[NotNan]` entirely interchangeable in layout and lifetimes; the caller is
        // responsible for the value-level invariant per the safety doc above.
        unsafe { mem::transmute::<&[T], &[Self]>(values) }
    }
}

/// Extension trait for viewing a slice of [`NotNan`] values as primitive values.
pub trait NotNanSliceExt<T: NotNanFloat> {
    /// Views [`NotNan`] values as primitive values.
    ///
    /// "Primitive" here means Rust's built-in float type (e.g. `f64`).
    ///
    ///
    /// This runs in `O(1)` and does not allocate.
    /// # Examples
    /// ```
    /// use range_set_blaze::NotNanF64;
    /// use range_set_blaze::not_nan::NotNanSliceExt;
    ///
    /// let not_nans = [NotNanF64::new(1.0), NotNanF64::new(2.0), NotNanF64::new(3.0)];
    /// assert_eq!(&[1.0, 2.0, 3.0], not_nans.as_primitive_slice());
    /// ```
    fn as_primitive_slice(&self) -> &[T];
}

impl<T: NotNanFloat> NotNanSliceExt<T> for [NotNan<T>] {
    fn as_primitive_slice(&self) -> &[T] {
        // SAFETY: NotNan<T> is #[repr(transparent)] over T, making `&[T]`
        // and `&[NotNan<T>]` entirely interchangeable in layout and lifetimes.
        unsafe { from_raw_parts(self.as_ptr().cast::<T>(), self.len()) }
    }
}

/// Extension trait for converting an inclusive [`NotNan`] range into an inclusive primitive
/// range (or a `(start, end)` primitive tuple).
pub trait NotNanRangeExt<T: NotNanFloat> {
    /// Converts an inclusive [`NotNan`] range into an inclusive primitive range.
    ///
    /// "Primitive" here means Rust's built-in float type (e.g. `f64`).
    ///
    ///
    /// This is the reverse of [`NotNan::from_primitive_range`].
    ///
    /// # Examples
    /// ```
    /// use range_set_blaze::NotNanF64;
    /// use range_set_blaze::not_nan::NotNanRangeExt;
    ///
    /// let range = NotNanF64::new(3.0)..=NotNanF64::new(5.0);
    /// assert_eq!(range.into_primitive_range(), 3.0..=5.0);
    /// ```
    #[must_use]
    fn into_primitive_range(self) -> RangeInclusive<T>;

    /// Converts an inclusive [`NotNan`] range into a `(start, end)` tuple of primitive values.
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
    /// use range_set_blaze::NotNanF64;
    /// use range_set_blaze::not_nan::NotNanRangeExt;
    ///
    /// let range = NotNanF64::new(3.0)..=NotNanF64::new(5.0);
    /// assert_eq!(range.into_primitive_inner(), (3.0, 5.0));
    /// ```
    #[must_use]
    fn into_primitive_inner(self) -> (T, T);
}

impl<T: NotNanFloat> NotNanRangeExt<T> for RangeInclusive<NotNan<T>> {
    fn into_primitive_range(self) -> RangeInclusive<T> {
        let (start, end) = self.into_primitive_inner();
        start..=end
    }

    fn into_primitive_inner(self) -> (T, T) {
        let (start, end) = self.into_inner();
        (start.into_inner(), end.into_inner())
    }
}

impl<T: NotNanFloat> PartialEq for NotNan<T> {
    fn eq(&self, other: &Self) -> bool {
        T::total_cmp(self.0, other.0) == Ordering::Equal
    }
}

impl<T: NotNanFloat> Eq for NotNan<T> {}

impl<T: NotNanFloat> PartialOrd for NotNan<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<T: NotNanFloat> Ord for NotNan<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        T::total_cmp(self.0, other.0)
    }
}

impl<T: NotNanFloat> Hash for NotNan<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        T::hash(self.0, state);
    }
}

impl<T: NotNanFloat> Integer for NotNan<T> {
    type SafeLen = T::SafeLen;

    #[inline]
    fn checked_add_one(self) -> Option<Self> {
        self.checked_after()
    }

    // This moves to the next representable float in total_cmp order, not a numeric + 1.0.
    #[inline]
    fn add_one(self) -> Self {
        self.after()
    }

    #[inline]
    // This moves to the previous representable float in total_cmp order, not a numeric - 1.0.
    fn sub_one(self) -> Self {
        self.before()
    }

    #[inline]
    fn assign_sub_one(&mut self) {
        *self = self.before();
    }

    // Ideally, we would `impl std::iter::Step for NotNanF64` and just call Range::next(), but that's still experimental.
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
    #[allow(clippy::float_cmp)]
    fn safe_constructors_preserve_not_nan_invariant() {
        assert_eq!(nnf32(-0.0).into_inner().to_bits(), 0);
        assert_eq!(nnf64(-0.0).into_inner().to_bits(), 0);
        assert_eq!(NotNanF64::new(-0.0), nnf64(0.0));
        assert_eq!(NotNanF64::try_new(-0.0), Some(nnf64(0.0)));

        // Infinities are legal values, not rejected.
        for value in [f64::INFINITY, f64::NEG_INFINITY] {
            assert_eq!(NotNanF64::new(value).into_inner(), value);
            assert_eq!(NotNanF64::try_new(value), Some(NotNanF64::new(value)));
        }

        for invalid in [f64::NAN, -f64::NAN] {
            assert!(panics(|| {
                black_box(NotNanF64::new(invalid));
            }));
            assert_eq!(NotNanF64::try_new(invalid), None);
            assert!(panics(|| drop(NotNanF64::from_primitive_range(
                invalid..=1.0
            ))));
            assert!(panics(|| {
                NotNanF64::values([invalid]).count();
            }));
            assert!(panics(|| {
                black_box(NotNanF64::from_primitive_slice(&[invalid]));
            }));
        }

        assert!(panics(|| {
            black_box(NotNanF64::from_primitive_slice(&[-0.0]));
        }));
        assert!(panics(|| {
            black_box(NotNanF64::from_primitive_slice(&[f64::NAN]));
        }));

        let values = [1.0, 2.0, 3.0];
        let not_nans = NotNanF64::from_primitive_slice(&values);
        assert_eq!(not_nans.as_primitive_slice(), &values);
        assert_eq!(
            NotNanF64::values(values).collect::<Vec<_>>(),
            vec![nnf64(1.0), nnf64(2.0), nnf64(3.0)]
        );
        assert_eq!(
            NotNanF64::from_primitive_ranges([1.0..=2.0]).collect::<Vec<_>>(),
            vec![nnf64(1.0)..=nnf64(2.0)]
        );
    }

    #[test]
    fn ordering_agrees_with_total_cmp() {
        let values = [
            f64::NEG_INFINITY,
            -f64::MAX,
            -1.0,
            0.0,
            1.0,
            f64::MAX,
            f64::INFINITY,
        ];

        for left in values {
            for right in values {
                assert_eq!(nnf64(left).cmp(&nnf64(right)), left.total_cmp(&right));
            }
        }
        assert_ne!(nnf64(0.0).cmp(&nnf64(-0.0)), 0.0_f64.total_cmp(&-0.0));
    }

    #[test]
    fn converts_ranges() {
        assert_eq!(
            NotNanF64::from_primitive_range(10.0..=20.0),
            nnf64(10.0)..=nnf64(20.0)
        );
        assert_eq!(
            NotNanF64::from_primitive_ranges([10.0..=20.0, 30.0..=40.0]).collect::<Vec<_>>(),
            vec![nnf64(10.0)..=nnf64(20.0), nnf64(30.0)..=nnf64(40.0)]
        );
    }

    #[test]
    fn after_and_before_step_through_zero_in_total_order() {
        assert_eq!(nnf64(-0.0), nnf64(0.0));
        assert_ne!(nnf64(0.0).before(), nnf64(-0.0));
        assert_eq!(nnf64(0.0).after(), nnf64(f64::from_bits(1)));
        assert_eq!(
            nnf64(0.0).before(),
            nnf64(f64::from_bits(0x8000_0000_0000_0001))
        );
    }

    #[test]
    fn after_and_before_panic_at_boundaries_in_all_build_modes() {
        assert_eq!(NotNanF64::MAX.checked_after(), None);
        assert_eq!(NotNanF64::MIN.checked_before(), None);
    }

    #[test]
    #[should_panic(expected = "b must be in range 1..=max_len")]
    fn not_nan_endpoint_offset_cannot_leave_domain() {
        let _ = NotNanF32::MAX.inclusive_end_from_start(2);
    }

    #[test]
    #[should_panic(expected = "after() called on maximum value")]
    fn after_panics_at_max() {
        let _ = NotNanF64::MAX.after();
    }

    #[test]
    #[should_panic(expected = "before() called on minimum value")]
    fn before_panics_at_min() {
        let _ = NotNanF64::MIN.before();
    }

    #[test]
    fn checked_after_and_before_stop_at_total_order_boundaries() {
        assert_eq!(NotNanF64::MIN.checked_before(), None);
        assert_eq!(NotNanF64::MAX.checked_after(), None);
        assert_eq!(NotNanF64::MIN.checked_after(), Some(NotNanF64::MIN.after()));
        assert_eq!(
            NotNanF64::MAX.checked_before(),
            Some(NotNanF64::MAX.before())
        );
    }

    #[test]
    fn min_and_max_are_total_order_boundaries() {
        let values = [
            nnf64(-f64::MAX),
            nnf64(-1.0),
            nnf64(-0.0),
            nnf64(0.0),
            nnf64(1.0),
            nnf64(f64::MAX),
        ];

        for value in values {
            assert!(NotNanF64::MIN <= value);
            assert!(value <= NotNanF64::MAX);
        }
    }

    /// `MIN`/`MAX` are the infinities, and they sit directly adjacent (in total order) to the
    /// largest-magnitude finite values -- exactly the values reached by stepping `.before()`/
    /// `.after()` once, and nowhere else.
    #[test]
    fn infinities_are_adjacent_to_finite_extremes() {
        assert_eq!(NotNanF64::MIN, nnf64(f64::NEG_INFINITY));
        assert_eq!(NotNanF64::MAX, nnf64(f64::INFINITY));
        assert_eq!(NotNanF64::MIN.after(), nnf64(f64::MIN));
        assert_eq!(NotNanF64::MAX.before(), nnf64(f64::MAX));
        assert_eq!(nnf64(f64::MIN).before(), NotNanF64::MIN);
        assert_eq!(nnf64(f64::MAX).after(), NotNanF64::MAX);

        assert_eq!(NotNanF32::MIN, nnf32(f32::NEG_INFINITY));
        assert_eq!(NotNanF32::MAX, nnf32(f32::INFINITY));
        assert_eq!(NotNanF32::MIN.after(), nnf32(f32::MIN));
        assert_eq!(NotNanF32::MAX.before(), nnf32(f32::MAX));
        assert_eq!(nnf32(f32::MIN).before(), NotNanF32::MIN);
        assert_eq!(nnf32(f32::MAX).after(), NotNanF32::MAX);
    }

    #[test]
    fn infinities_are_valid_range_endpoints() {
        use crate::RangeSetBlaze;

        let set = RangeSetBlaze::from_iter([nnf64(f64::NEG_INFINITY)..=nnf64(0.0)]);
        assert!(set.contains(NotNanF64::MIN));
        assert!(set.contains(nnf64(f64::MIN)));
        assert!(set.contains(nnf64(0.0)));
        assert!(!set.contains(nnf64(0.0).after()));
        assert!(!set.contains(NotNanF64::MAX));

        let full = !RangeSetBlaze::<NotNanF64>::new();
        assert!(full.contains(NotNanF64::MIN));
        assert!(full.contains(NotNanF64::MAX));
        assert_eq!(full.len(), NotNanF64::MAX_SIZE);
    }

    #[test]
    fn after_and_before_are_neighbors_in_total_order() {
        let values = [
            NotNanF64::MIN,
            nnf64(f64::MIN),
            nnf64(-f64::MAX),
            nnf64(-1.0),
            nnf64(-0.0),
            nnf64(0.0),
            nnf64(1.0),
            nnf64(f64::MAX),
            NotNanF64::MAX,
        ];

        for value in values {
            if value != NotNanF64::MAX {
                assert_eq!(value.after().before(), value);
            }
            if value != NotNanF64::MIN {
                assert_eq!(value.before().after(), value);
            }
        }
    }

    #[test]
    fn adjacency_laws_cover_f32_and_f64_edges() {
        macro_rules! check {
            ($name:ident, $zero:expr, $negative_subnormal:expr, $positive_subnormal:expr, $min:expr, $max:expr) => {{
                let values = [
                    nnf32($zero),
                    nnf32($negative_subnormal),
                    nnf32($positive_subnormal),
                    nnf32(-1.0),
                    nnf32(1.0),
                    nnf32($min),
                    nnf32($max),
                ];
                for value in values {
                    if value != NotNanF32::MAX {
                        assert_eq!(value.after().before(), value);
                    }
                    if value != NotNanF32::MIN {
                        assert_eq!(value.before().after(), value);
                    }
                }
                assert_eq!(NotNanF32::MIN.checked_before(), None);
                assert_eq!(NotNanF32::MAX.checked_after(), None);
                assert_eq!(nnf32($negative_subnormal).after(), nnf32($zero));
                assert_eq!(nnf32($zero).after(), nnf32($positive_subnormal));
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
            nnf64(-f64::from_bits(1)),
            nnf64(0.0),
            nnf64(f64::from_bits(1)),
            nnf64(-1.0),
            nnf64(1.0),
            NotNanF64::MIN,
            NotNanF64::MAX,
        ];
        for value in values {
            if value != NotNanF64::MAX {
                assert_eq!(value.after().before(), value);
            }
            if value != NotNanF64::MIN {
                assert_eq!(value.before().after(), value);
            }
        }
        assert_eq!(NotNanF64::MIN.checked_before(), None);
        assert_eq!(NotNanF64::MAX.checked_after(), None);
        assert_eq!(nnf64(-f64::from_bits(1)).after(), nnf64(0.0));
        assert_eq!(nnf64(0.0).after(), nnf64(f64::from_bits(1)));
    }

    #[test]
    fn range_length_laws_cover_f32_and_f64() {
        let start = nnf32(-f32::from_bits(1));
        let end = nnf32(f32::from_bits(1));
        assert_eq!(NotNanF32::safe_len(&(start..=start)), 1);
        assert_eq!(NotNanF32::safe_len(&(start..=start.after())), 2);
        assert_eq!(NotNanF32::safe_len(&(start..=end)), 3);
        assert_eq!(
            NotNanF32::MAX_SIZE,
            NotNanF32::safe_len(&(NotNanF32::MIN..=NotNanF32::MAX))
        );
        let length = 17;
        let endpoint = start.inclusive_end_from_start(length);
        assert_eq!(endpoint.start_from_inclusive_end(length), start);
        assert_eq!(start.inclusive_end_from_start(length), endpoint);

        let start = nnf64(-f64::from_bits(1));
        let end = nnf64(f64::from_bits(1));
        assert_eq!(NotNanF64::safe_len(&(start..=start)), 1);
        assert_eq!(NotNanF64::safe_len(&(start..=start.after())), 2);
        assert_eq!(NotNanF64::safe_len(&(start..=end)), 3);
        assert_eq!(
            NotNanF64::MAX_SIZE,
            NotNanF64::safe_len(&(NotNanF64::MIN..=NotNanF64::MAX))
        );
        let length = 17;
        let endpoint = start.inclusive_end_from_start(length);
        assert_eq!(endpoint.start_from_inclusive_end(length), start);
        assert_eq!(start.inclusive_end_from_start(length), endpoint);
    }

    #[cfg(feature = "float_nightly_experimental")]
    #[test]
    fn f16_not_nan_adjacency_and_lengths_are_exhaustive() {
        for bits in 0..=u16::MAX {
            let value = f16::from_bits(bits);
            let Some(value) = NotNanF16::try_new(value) else {
                continue;
            };
            if value != NotNanF16::MIN {
                assert_eq!(value.before().after(), value);
            }
            if value != NotNanF16::MAX {
                assert_eq!(value.after().before(), value);
            }
            assert_eq!(NotNanF16::safe_len(&(value..=value)), 1);
        }
        assert_eq!(
            NotNanF16::MAX_SIZE,
            NotNanF16::safe_len(&(NotNanF16::MIN..=NotNanF16::MAX))
        );
    }
}
