//! Internal type to abstract a floating point value,
//! providing the necessary functionality for the `Total` types to `impl Integer`.
//!
//! The public capability trait is doc-hidden and sealed; use `Total` instead.

use core::{
    cmp::Ordering,
    fmt::{Debug, Display},
    hash::{Hash, Hasher},
    ops::{AddAssign, SubAssign},
};
use num_traits::ops::wrapping::{WrappingAdd, WrappingSub};
use num_traits::{One, Zero};

#[cfg(feature = "float_nightly_experimental")]
use crate::UIntPlusOne;

mod private {
    pub trait Sealed {}
}

/// Public capability required by the generic [`Total`](super::total::Total) APIs.
///
/// This trait is sealed because it is an implementation detail of the supported primitive
/// floating-point wrappers. Use [`Total`](super::total::Total) rather than implementing it.
#[doc(hidden)]
pub trait TotalFloat:
    private::Sealed + Default + Copy + Clone + Debug + Send + Sync + 'static
{
    /// The minimum value in total order.
    const MIN: Self;
    /// The maximum value in total order.
    const MAX: Self;
    /// The maximum number of values in a total-order range.
    const MAX_SIZE: Self::SafeLen;

    /// Integral type for holding the size of any total-order floating-point range.
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
    fn after(x: Self) -> Self;
    fn before(x: Self) -> Self;
    fn prim_safe_len(start: Self, end: Self) -> Self::SafeLen;
    fn safe_len_to_f64_lossy(len: Self::SafeLen) -> f64;
    fn f64_to_safe_len_lossy(f: f64) -> Self::SafeLen;
    fn inclusive_end_from_start(a: Self, b: Self::SafeLen) -> Self;
    fn start_from_inclusive_end(a: Self, b: Self::SafeLen) -> Self;
}

/// Crate-private encoding and arithmetic machinery for total-order floats.
pub(super) trait TotalFloatImpl:
    TotalFloat + Default + Copy + Clone + Debug + Send + Sync + 'static
{
    /// The result of `to_bits()` on the wrapped type, e.g. u64
    type Bits: Copy + Eq + Hash + Send + Sync + Debug;
    /// The intermediate type used for comparison, e.g. i64
    type Ordered: WrappingAdd + WrappingSub + One + PartialEq + Copy + Send + Sync + Debug + Display;
    /// The minimum value available, in the `total_cmp`  sense
    const MIN: Self;
    /// The maximum value available, in the `total_cmp`  sense
    const MAX: Self;

    /// The maximum possible size of a range, i.e. the maximum value possible from `safe_len()`
    const MAX_SIZE: Self::SafeLen;

    /// Transform a float value into Ordered, to allow comparison and addition
    fn to_ordered(x: Self) -> Self::Ordered;
    /// Transform Ordered back to a float value, presumably after some addition
    fn from_ordered(x: Self::Ordered) -> Self;
    /// Transform a float value into a type with more concrete semantics, e.g. `f64::to_bits()`
    fn to_bits(x: Self) -> Self::Bits;
    /// Return the size of an inclusive ordered range from `start` to `end`.
    ///
    /// # Precondition
    /// `start <= end`. The range must be non-empty and already expressed in ordered space. This
    /// is checked with `debug_assert!` and is not checked in release builds. Public set/map APIs
    /// accept inverted ranges as empty; they must not pass such ranges to this internal helper.
    fn safe_len(start: Self::Ordered, end: Self::Ordered) -> Self::SafeLen;
    /// Converts [`TotalFloat::SafeLen`] to `f64`, potentially losing precision for large values.
    fn safe_len_to_f64_lossy(len: Self::SafeLen) -> f64;
    /// Converts a `f64` to [`TotalFloat::SafeLen`] using the formula `f as Self::SafeLen`. For large integer types, this will result in a loss of precision.
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

    /// Computes `self + (b - 1)` where `b` is of type [`TotalFloat::SafeLen`].
    fn inclusive_end_from_start(a: Self, b: Self::SafeLen) -> Self {
        #[cfg(debug_assertions)]
        {
            let max_len = <Self as TotalFloatImpl>::prim_safe_len(a, <Self as TotalFloatImpl>::MAX);
            assert!(
                Self::SafeLen::zero() < b && b <= max_len,
                "b must be in range 1..=max_len (b = {b}, max_len = {max_len})"
            );
        }
        // `Ordered` is a signed integer whose wrapping distance matches the total float order.
        // Because the debug precondition bounds `b` by the remaining domain, modular addition
        // lands on the correct endpoint even when the signed representation overflows.
        Self::from_ordered(Self::to_ordered(a).wrapping_add(&Self::safe_as_ordered(b)))
    }
    /// Computes `self - (b - 1)` where `b` is of type [`TotalFloat::SafeLen`].
    fn start_from_inclusive_end(a: Self, b: Self::SafeLen) -> Self {
        #[cfg(debug_assertions)]
        {
            let max_len = <Self as TotalFloatImpl>::prim_safe_len(<Self as TotalFloatImpl>::MIN, a);
            assert!(
                Self::SafeLen::zero() < b && b <= max_len,
                "b must be in range 1..=max_len (b = {b}, max_len = {max_len})"
            );
        }
        // `Ordered` is a signed integer whose wrapping distance matches the total float order.
        // Because the debug precondition bounds `b` by the distance from `MIN`, modular
        // subtraction lands on the correct endpoint even when the signed representation underflows.
        Self::from_ordered(Self::to_ordered(a).wrapping_sub(&Self::safe_as_ordered(b)))
    }
    /// Return the size of the inclusive range from start to end.
    fn prim_safe_len(start: Self, end: Self) -> Self::SafeLen {
        Self::safe_len(Self::to_ordered(start), Self::to_ordered(end))
    }
}

// The to/from functions are pulled out as free functions for two reasons
// 1) Share code with FiniteFloat
// 2) Allow use from in a const context, which is not possible with trait methods.

macro_rules! impl_ordered_transform {
    ($primitive:ty, $ordered:ty, $to_ordered:ident, $from_ordered:ident, $sign_shift:literal) => {
        pub(super) const fn $to_ordered(x: $primitive) -> $ordered {
            let mut bits = x.to_bits().cast_signed();
            bits ^= ((bits >> $sign_shift).cast_unsigned() >> 1).cast_signed();
            bits
        }

        pub(super) const fn $from_ordered(mut bits: $ordered) -> $primitive {
            // Reversing the XOR transformation
            bits ^= ((bits >> $sign_shift).cast_unsigned() >> 1).cast_signed();
            <$primitive>::from_bits(bits.cast_unsigned())
        }
    };
}

impl_ordered_transform!(f64, i64, to_ordered_64, from_ordered_64, 63);
impl_ordered_transform!(f32, i32, to_ordered_32, from_ordered_32, 31);
#[cfg(feature = "float_nightly_experimental")]
impl_ordered_transform!(f16, i16, to_ordered_16, from_ordered_16, 15);
#[cfg(feature = "float_nightly_experimental")]
impl_ordered_transform!(f128, i128, to_ordered_128, from_ordered_128, 127);

macro_rules! impl_total_ops {
    ($to_ordered:ident) => {
        fn to_bits(x: Self) -> Self::Bits {
            x.to_bits()
        }

        fn to_ordered(x: Self) -> Self::Ordered {
            $to_ordered(x)
        }

        fn total_cmp(x: Self, y: Self) -> Ordering {
            x.total_cmp(&y)
        }
    };
}

// Used by everything but f128, which has a more complex SafeLen type
macro_rules! impl_total_ops_safelen {
    () => {
        fn safe_len(start: Self::Ordered, end: Self::Ordered) -> Self::SafeLen {
            // 1️⃣ Contract: caller promises start ≤ end  (checked only in debug builds)
            debug_assert!(start <= end, "start ≤ end required");

            // 2️⃣ Compute distance in `Self` then reinterpret‑cast to the first
            Self::SafeLen::from(end) - Self::SafeLen::from(start) + 1
        }

        #[allow(clippy::cast_precision_loss)]
        #[allow(clippy::use_self, reason = "f64 is not really Self")]
        #[allow(clippy::cast_lossless)]
        fn safe_len_to_f64_lossy(len: Self::SafeLen) -> f64 {
            len as f64
        }

        #[expect(clippy::cast_possible_truncation)]
        fn f64_to_safe_len_lossy(f: f64) -> Self::SafeLen {
            f as Self::SafeLen
        }

        #[expect(clippy::cast_possible_truncation)]
        fn safe_as_ordered(x: Self::SafeLen) -> Self::Ordered {
            debug_assert!(!x.is_zero(), "x must not be zero");
            (x - 1) as Self::Ordered
        }
    };
}

impl private::Sealed for f64 {}
impl private::Sealed for f32 {}
#[cfg(feature = "float_nightly_experimental")]
impl private::Sealed for f16 {}
#[cfg(feature = "float_nightly_experimental")]
impl private::Sealed for f128 {}

macro_rules! impl_total_capability {
    ($primitive:ty, $safe_len:ty) => {
        impl TotalFloat for $primitive {
            type SafeLen = $safe_len;

            const MIN: Self = <Self as TotalFloatImpl>::MIN;
            const MAX: Self = <Self as TotalFloatImpl>::MAX;
            const MAX_SIZE: Self::SafeLen = <Self as TotalFloatImpl>::MAX_SIZE;

            fn hash<H: Hasher>(x: Self, state: &mut H) {
                <Self as TotalFloatImpl>::to_bits(x).hash(state);
            }

            fn total_cmp(x: Self, y: Self) -> Ordering {
                <Self as TotalFloatImpl>::total_cmp(x, y)
            }

            fn after(x: Self) -> Self {
                <Self as TotalFloatImpl>::from_ordered(
                    <Self as TotalFloatImpl>::to_ordered(x)
                        .wrapping_add(<<Self as TotalFloatImpl>::Ordered as One>::one()),
                )
            }

            fn before(x: Self) -> Self {
                <Self as TotalFloatImpl>::from_ordered(
                    <Self as TotalFloatImpl>::to_ordered(x)
                        .wrapping_sub(<<Self as TotalFloatImpl>::Ordered as One>::one()),
                )
            }

            fn prim_safe_len(start: Self, end: Self) -> Self::SafeLen {
                <Self as TotalFloatImpl>::prim_safe_len(start, end)
            }

            fn safe_len_to_f64_lossy(len: Self::SafeLen) -> f64 {
                <Self as TotalFloatImpl>::safe_len_to_f64_lossy(len)
            }

            fn f64_to_safe_len_lossy(f: f64) -> Self::SafeLen {
                <Self as TotalFloatImpl>::f64_to_safe_len_lossy(f)
            }

            fn inclusive_end_from_start(a: Self, b: Self::SafeLen) -> Self {
                <Self as TotalFloatImpl>::inclusive_end_from_start(a, b)
            }

            fn start_from_inclusive_end(a: Self, b: Self::SafeLen) -> Self {
                <Self as TotalFloatImpl>::start_from_inclusive_end(a, b)
            }
        }
    };
}

impl_total_capability!(f64, i128);
impl_total_capability!(f32, i64);
#[cfg(feature = "float_nightly_experimental")]
impl_total_capability!(f16, i32);
#[cfg(feature = "float_nightly_experimental")]
impl_total_capability!(f128, UIntPlusOne<u128>);

impl TotalFloatImpl for f64 {
    type Bits = u64;
    type Ordered = i64;

    const MIN: Self = Self::from_bits(u64::MAX);
    const MAX: Self = Self::from_bits(0x7fff_ffff_ffff_ffff);
    // Every 64-bit pattern is a valid total-order value, including NaNs and signed zero.
    const MAX_SIZE: Self::SafeLen = u64::MAX as Self::SafeLen + 1;

    impl_total_ops!(to_ordered_64);
    impl_total_ops_safelen!();

    fn from_ordered(bits: Self::Ordered) -> Self {
        from_ordered_64(bits)
    }
}

impl TotalFloatImpl for f32 {
    type Bits = u32;
    type Ordered = i32;

    const MIN: Self = Self::from_bits(u32::MAX);
    const MAX: Self = Self::from_bits(0x7fff_ffff);
    // Every 32-bit pattern is a valid total-order value, including NaNs and signed zero.
    const MAX_SIZE: Self::SafeLen = u32::MAX as Self::SafeLen + 1;

    impl_total_ops!(to_ordered_32);
    impl_total_ops_safelen!();

    fn from_ordered(bits: Self::Ordered) -> Self {
        from_ordered_32(bits)
    }
}

#[cfg(feature = "float_nightly_experimental")]
impl TotalFloatImpl for f16 {
    type Bits = u16;
    type Ordered = i16;

    const MIN: Self = Self::from_bits(u16::MAX);
    const MAX: Self = Self::from_bits(0x7fff);
    // Every 16-bit pattern is a valid total-order value, including NaNs and signed zero.
    const MAX_SIZE: Self::SafeLen = u16::MAX as Self::SafeLen + 1;

    impl_total_ops!(to_ordered_16);
    impl_total_ops_safelen!();
    fn from_ordered(bits: Self::Ordered) -> Self {
        from_ordered_16(bits)
    }
}

#[cfg(feature = "float_nightly_experimental")]
impl TotalFloatImpl for f128 {
    type Bits = u128;
    type Ordered = i128;

    const MIN: Self = Self::from_bits(u128::MAX);
    const MAX: Self = Self::from_bits(0x7fff_ffff_ffff_ffff_ffff_ffff_ffff_ffff);
    // Every 128-bit pattern is a valid total-order value, including NaNs and signed zero.
    const MAX_SIZE: Self::SafeLen = UIntPlusOne::<u128>::MaxPlusOne;

    impl_total_ops!(to_ordered_128);

    fn from_ordered(bits: Self::Ordered) -> Self {
        from_ordered_128(bits)
    }

    #[expect(clippy::cast_sign_loss)]
    fn safe_len(start: Self::Ordered, end: Self::Ordered) -> Self::SafeLen {
        // 1️⃣ Contract: caller promises start ≤ end  (checked only in debug builds)
        debug_assert!(start <= end, "start ≤ end required");

        // 2️⃣ Compute distance in `Self` then reinterpret‑cast to the first
        let less1 = end.overflowing_sub(start).0 as u128;
        let less1 = UIntPlusOne::UInt(less1);
        less1 + UIntPlusOne::UInt(1)
    }

    #[allow(clippy::cast_precision_loss)]
    fn safe_len_to_f64_lossy(len: Self::SafeLen) -> f64 {
        match len {
            UIntPlusOne::UInt(len) => len as f64,
            UIntPlusOne::MaxPlusOne => UIntPlusOne::<u128>::max_plus_one_as_f64(),
        }
    }

    #[expect(clippy::cast_possible_truncation)]
    #[expect(clippy::cast_sign_loss)]
    fn f64_to_safe_len_lossy(f: f64) -> Self::SafeLen {
        if f >= UIntPlusOne::<u128>::max_plus_one_as_f64() {
            UIntPlusOne::MaxPlusOne
        } else {
            UIntPlusOne::UInt(f as u128)
        }
    }

    #[expect(clippy::cast_possible_wrap)]
    fn safe_as_ordered(x: Self::SafeLen) -> Self::Ordered {
        match x {
            UIntPlusOne::UInt(x) => {
                debug_assert!(x != 0, "x must not be zero");
                (x - 1) as Self::Ordered
            }
            UIntPlusOne::MaxPlusOne => u128::MAX as Self::Ordered,
        }
    }
}
