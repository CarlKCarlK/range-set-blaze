//! Internal type to abstract a floating point value,
//! providing the necessary functionality for the `Finite` types to `impl Integer`.
//!
//! The public capability trait is doc-hidden and sealed; use `Finite` instead.

use core::{
    cmp::Ordering,
    fmt::{Debug, Display},
    hash::{Hash, Hasher},
    ops::{AddAssign, SubAssign},
};

#[cfg(feature = "float_nightly_experimental")]
use super::total_float::{from_ordered_16, from_ordered_128, to_ordered_16, to_ordered_128};
use super::total_float::{from_ordered_32, from_ordered_64, to_ordered_32, to_ordered_64};
use num_traits::ops::wrapping::{WrappingAdd, WrappingSub};
use num_traits::{One, Zero};

mod private {
    pub trait Sealed {}
}

/// Public capability required by the generic [`Finite`](super::finite::Finite) APIs.
///
/// This trait is sealed because it is an implementation detail of the supported primitive
/// floating-point wrappers. Use [`Finite`](super::finite::Finite) rather than implementing it.
#[doc(hidden)]
pub trait FiniteFloat:
    private::Sealed + Default + Copy + Clone + Debug + Send + Sync + 'static
{
    /// The minimum finite value.
    const MIN: Self;
    /// The maximum finite value.
    const MAX: Self;
    /// The maximum number of values in a finite range.
    const MAX_SIZE: Self::SafeLen;

    /// Integral type for holding the size of any finite floating-point range.
    type SafeLen: Send
        + Sync
        + Debug
        + Display
        + Hash
        + Copy
        + PartialEq
        + PartialOrd
        + num_traits::Zero
        + num_traits::One
        + AddAssign
        + SubAssign;

    fn hash<H: Hasher>(x: Self, state: &mut H);
    fn total_cmp(x: Self, y: Self) -> Ordering;
    fn is_finite(x: Self) -> bool;
    fn normalize(x: Self) -> Self;
    fn after(x: Self) -> Self;
    fn before(x: Self) -> Self;
    fn is_neg_zero(x: Self) -> bool;
    fn prim_safe_len(start: Self, end: Self) -> Self::SafeLen;
    fn safe_len_to_f64_lossy(len: Self::SafeLen) -> f64;
    fn f64_to_safe_len_lossy(f: f64) -> Self::SafeLen;
    fn inclusive_end_from_start(a: Self, b: Self::SafeLen) -> Self;
    fn start_from_inclusive_end(a: Self, b: Self::SafeLen) -> Self;
}

/// Crate-private encoding and arithmetic machinery for finite floats.
pub(super) trait FiniteFloatImpl:
    FiniteFloat + Default + Copy + Clone + Debug + Send + Sync + 'static
{
    /// The result of `to_bits()` on the wrapped type, e.g. u64
    type Bits: Copy + Eq + Hash + Send + Sync + Debug;
    /// The intermediate type used for `total_cmp` comparison, e.g. i64
    type Ordered: WrappingAdd
        + WrappingSub
        + One
        + PartialEq
        + Copy
        + Send
        + Sync
        + Debug
        + Display
        + PartialOrd;
    /// The minimum value available, in the usual floating point sense
    const MIN: Self;
    /// The maximum value available, in the usual floating point sense
    const MAX: Self;

    /// `MIN` converted to the Ordered type
    const MIN_ORDERED: Self::Ordered;
    /// `MAX` converted to the Ordered type
    const MAX_ORDERED: Self::Ordered;

    /// The maximum possible size of a range, i.e. the maximum value possible from `safe_len()`
    const MAX_SIZE: Self::SafeLen;

    /// The bit pattern for negative zero
    const NEG_ZERO_BITS: Self::Bits;

    /// The ordered value of negative zero
    const NEG_ZERO_ORDERED: Self::Ordered;

    /// Transform a float value into Ordered, to allow comparison and addition
    fn to_ordered(x: Self) -> Self::Ordered;
    /// Transform Ordered back to a float value
    fn from_ordered(x: Self::Ordered) -> Self;
    /// Transform a float value into a type with more concrete semantics, e.g. `f64::to_bits()`
    fn to_bits(x: Self) -> Self::Bits;
    /// Return the size of an inclusive ordered range from `start` to `end`.
    ///
    /// # Precondition
    /// `start <= end`, and both endpoints are within the finite ordered domain. The range must
    /// therefore be non-empty and already validated by the caller. This is checked with
    /// `debug_assert!` and is not checked in release builds. Public set/map APIs accept inverted
    /// ranges as empty; they must not pass such ranges to this internal helper.
    fn safe_len(start: Self::Ordered, end: Self::Ordered) -> Self::SafeLen;
    /// Converts [`FiniteFloat::SafeLen`] to `f64`, potentially losing precision for large values.
    fn safe_len_to_f64_lossy(len: Self::SafeLen) -> f64;
    /// Converts a `f64` to [`FiniteFloat::SafeLen`] using the formula `f as Self::SafeLen`. For large integer types, this will result in a loss of precision.
    fn f64_to_safe_len_lossy(f: f64) -> Self::SafeLen;
    /// Returns `(x - 1)` as `Self::Ordered`.
    ///
    /// # Precondition
    /// `x` must not be zero, or `x - 1` underflows. This is checked with
    /// `debug_assert!` and is *not* checked in release builds, where violating it
    /// silently wraps to a nonsense (but not unsafe) result.
    fn safe_as_ordered(x: Self::SafeLen) -> Self::Ordered;
    /// Returns the ordering between `x` and `y`, as per the standard library's `f64::total_cmp`.
    fn total_cmp(x: Self, y: Self) -> Ordering;

    /// Computes `self + (b - 1)` where `b` is of type [`FiniteFloat::SafeLen`].
    fn inclusive_end_from_start(a: Self, b: Self::SafeLen) -> Self {
        #[cfg(debug_assertions)]
        {
            let max_len =
                <Self as FiniteFloatImpl>::prim_safe_len(a, <Self as FiniteFloatImpl>::MAX);
            assert!(
                Self::SafeLen::zero() < b && b <= max_len,
                "b must be in range 1..=max_len (b = {b}, max_len = {max_len})"
            );
        }
        // `Ordered` is a signed integer whose wrapping distance matches the total float order.
        // Because the debug precondition bounds `b` by the remaining finite domain, modular
        // addition lands on the correct endpoint even when the signed representation overflows.
        // The following correction removes the excluded `-0.0` slot from that distance.
        let start = Self::to_ordered(a);
        let mut end = start.wrapping_add(&Self::safe_as_ordered(b));
        if Self::crosses_neg_zero(start, end) {
            end = end.wrapping_add(&Self::Ordered::one());
        }
        Self::from_ordered(end)
    }
    /// Computes `self - (b - 1)` where `b` is of type [`FiniteFloat::SafeLen`].
    fn start_from_inclusive_end(a: Self, b: Self::SafeLen) -> Self {
        #[cfg(debug_assertions)]
        {
            let max_len =
                <Self as FiniteFloatImpl>::prim_safe_len(<Self as FiniteFloatImpl>::MIN, a);
            assert!(
                Self::SafeLen::zero() < b && b <= max_len,
                "b must be in range 1..=max_len (b = {b}, max_len = {max_len})"
            );
        }
        // `Ordered` is a signed integer whose wrapping distance matches the total float order.
        // Because the debug precondition bounds `b` by the distance from `MIN`, modular
        // subtraction lands on the correct endpoint even when the signed representation underflows.
        // The following correction removes the excluded `-0.0` slot from that distance.
        let end = Self::to_ordered(a);
        let mut start = end.wrapping_sub(&Self::safe_as_ordered(b));
        if Self::crosses_neg_zero(start, end) {
            start = start.wrapping_sub(&Self::Ordered::one());
        }
        Self::from_ordered(start)
    }
    /// Return the size of the inclusive range from start to end.
    fn prim_safe_len(start: Self, end: Self) -> Self::SafeLen {
        Self::safe_len(Self::to_ordered(start), Self::to_ordered(end))
    }
    /// Return true if the float is finite.
    fn is_finite(x: Self) -> bool;
    /// Turn negative zero into positive zero, leave other numbers unchanged.
    fn normalize(x: Self) -> Self;
    /// Returns the least float strictly greater than `x` (`x.next_up()`).
    fn after(x: Self) -> Self;
    /// Returns the greatest float strictly less than `x` (`x.next_down()`).
    fn before(x: Self) -> Self;
    /// Returns true if `x`'s bit pattern is that of negative zero.
    fn is_neg_zero(x: Self) -> bool;

    /// Returns whether an ordered inclusive interval contains the excluded `-0.0` slot.
    #[must_use]
    fn crosses_neg_zero(start: Self::Ordered, end: Self::Ordered) -> bool {
        (start..=end).contains(&Self::NEG_ZERO_ORDERED)
    }
}

macro_rules! impl_finite_ops {
    ($to_ordered:ident) => {
        const MIN: Self = Self::MIN;
        const MAX: Self = Self::MAX;
        const MIN_ORDERED: Self::Ordered = $to_ordered(Self::MIN);
        const MAX_ORDERED: Self::Ordered = $to_ordered(Self::MAX);
        const NEG_ZERO_BITS: Self::Bits = Self::to_bits(-0.0);
        const NEG_ZERO_ORDERED: Self::Ordered = $to_ordered(-0.0);

        fn to_ordered(x: Self) -> Self::Ordered {
            $to_ordered(x)
        }

        fn to_bits(x: Self) -> Self::Bits {
            x.to_bits()
        }
        #[expect(clippy::cast_sign_loss)]
        fn safe_len(start: Self::Ordered, end: Self::Ordered) -> Self::SafeLen {
            // 1️⃣ Contract: caller promises start ≤ end  (checked only in debug builds)
            debug_assert!(start <= end, "start ≤ end required");
            debug_assert!(start >= Self::MIN_ORDERED, "start >= MIN required");
            debug_assert!(end <= Self::MAX_ORDERED, "end <= MAX required");

            if Self::crosses_neg_zero(start, end) {
                end.wrapping_sub(start) as Self::SafeLen
            } else {
                end.wrapping_sub(start).wrapping_add(1) as Self::SafeLen
            }
        }

        #[allow(clippy::cast_precision_loss)]
        #[allow(clippy::use_self, reason = "f64 is not really Self")]
        #[allow(clippy::cast_lossless)]
        fn safe_len_to_f64_lossy(len: Self::SafeLen) -> f64 {
            len as f64
        }

        #[expect(clippy::cast_possible_truncation)]
        #[expect(clippy::cast_sign_loss)]
        fn f64_to_safe_len_lossy(f: f64) -> Self::SafeLen {
            f as Self::SafeLen
        }

        #[expect(clippy::cast_possible_wrap)]
        fn safe_as_ordered(x: Self::SafeLen) -> Self::Ordered {
            debug_assert!(!x.is_zero(), "x must not be zero");
            (x - 1) as Self::Ordered
        }

        fn total_cmp(x: Self, y: Self) -> Ordering {
            x.total_cmp(&y)
        }

        fn is_finite(x: Self) -> bool {
            x.is_finite()
        }

        fn normalize(x: Self) -> Self {
            if x.to_bits() == Self::NEG_ZERO_BITS {
                0.0
            } else {
                x
            }
        }

        fn after(x: Self) -> Self {
            x.next_up()
        }

        fn before(x: Self) -> Self {
            x.next_down()
        }

        fn is_neg_zero(x: Self) -> bool {
            x.to_bits() == Self::NEG_ZERO_BITS
        }
    };
}

impl private::Sealed for f64 {}
impl private::Sealed for f32 {}
#[cfg(feature = "float_nightly_experimental")]
impl private::Sealed for f16 {}
#[cfg(feature = "float_nightly_experimental")]
impl private::Sealed for f128 {}

macro_rules! impl_finite_capability {
    ($primitive:ty, $safe_len:ty) => {
        impl FiniteFloat for $primitive {
            type SafeLen = $safe_len;

            const MIN: Self = <Self as FiniteFloatImpl>::MIN;
            const MAX: Self = <Self as FiniteFloatImpl>::MAX;
            const MAX_SIZE: Self::SafeLen = <Self as FiniteFloatImpl>::MAX_SIZE;

            fn hash<H: Hasher>(x: Self, state: &mut H) {
                <Self as FiniteFloatImpl>::to_bits(x).hash(state);
            }

            fn total_cmp(x: Self, y: Self) -> Ordering {
                <Self as FiniteFloatImpl>::total_cmp(x, y)
            }

            fn is_finite(x: Self) -> bool {
                <Self as FiniteFloatImpl>::is_finite(x)
            }

            fn normalize(x: Self) -> Self {
                <Self as FiniteFloatImpl>::normalize(x)
            }

            fn after(x: Self) -> Self {
                <Self as FiniteFloatImpl>::after(x)
            }

            fn before(x: Self) -> Self {
                <Self as FiniteFloatImpl>::before(x)
            }

            fn is_neg_zero(x: Self) -> bool {
                <Self as FiniteFloatImpl>::is_neg_zero(x)
            }

            fn prim_safe_len(start: Self, end: Self) -> Self::SafeLen {
                <Self as FiniteFloatImpl>::prim_safe_len(start, end)
            }

            fn safe_len_to_f64_lossy(len: Self::SafeLen) -> f64 {
                <Self as FiniteFloatImpl>::safe_len_to_f64_lossy(len)
            }

            fn f64_to_safe_len_lossy(f: f64) -> Self::SafeLen {
                <Self as FiniteFloatImpl>::f64_to_safe_len_lossy(f)
            }

            fn inclusive_end_from_start(a: Self, b: Self::SafeLen) -> Self {
                <Self as FiniteFloatImpl>::inclusive_end_from_start(a, b)
            }

            fn start_from_inclusive_end(a: Self, b: Self::SafeLen) -> Self {
                <Self as FiniteFloatImpl>::start_from_inclusive_end(a, b)
            }
        }
    };
}

impl_finite_capability!(f64, u64);
impl_finite_capability!(f32, u32);
#[cfg(feature = "float_nightly_experimental")]
impl_finite_capability!(f16, u16);
#[cfg(feature = "float_nightly_experimental")]
impl_finite_capability!(f128, u128);

impl FiniteFloatImpl for f64 {
    type Bits = u64;
    type Ordered = i64;

    // Finite bit patterns, with the -0.0 slot collapsed into +0.0.
    const MAX_SIZE: Self::SafeLen = 0xFFE0_0000_0000_0000_u64 - 1;

    fn from_ordered(bits: Self::Ordered) -> Self {
        from_ordered_64(bits)
    }

    impl_finite_ops!(to_ordered_64);
}

impl FiniteFloatImpl for f32 {
    type Bits = u32;
    type Ordered = i32;

    // Finite bit patterns, with the -0.0 slot collapsed into +0.0.
    const MAX_SIZE: Self::SafeLen = 0xFF00_0000_u32 - 1;

    impl_finite_ops!(to_ordered_32);

    fn from_ordered(bits: Self::Ordered) -> Self {
        from_ordered_32(bits)
    }
}

#[cfg(feature = "float_nightly_experimental")]
impl FiniteFloatImpl for f16 {
    type Bits = u16;
    type Ordered = i16;

    // Finite bit patterns, with the -0.0 slot collapsed into +0.0.
    const MAX_SIZE: Self::SafeLen = 0xF800u16 - 1;

    impl_finite_ops!(to_ordered_16);

    fn from_ordered(bits: Self::Ordered) -> Self {
        from_ordered_16(bits)
    }
}

#[cfg(feature = "float_nightly_experimental")]
impl FiniteFloatImpl for f128 {
    type Bits = u128;
    type Ordered = i128;

    // Finite bit patterns, with the -0.0 slot collapsed into +0.0.
    const MAX_SIZE: Self::SafeLen = 0xFFFE_0000_0000_0000_0000_0000_0000_0000_u128 - 1;

    impl_finite_ops!(to_ordered_128);

    fn from_ordered(bits: Self::Ordered) -> Self {
        from_ordered_128(bits)
    }
}
