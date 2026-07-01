//! Ordered `f64` support.

use core::cmp::Ordering;
use core::hash::{Hash, Hasher};
use core::ops::RangeInclusive;

use num_traits::ops::checked::{CheckedAdd, CheckedSub};
use num_traits::ops::wrapping::{WrappingAdd, WrappingSub};
use num_traits::{Num, NumAssign, One, Zero};
use std::fmt::{Debug, Display};

#[cfg(feature = "from_slice")]
use crate::RangeSetBlaze;

/// Minimum scaffolding necessary to implement Total
pub trait Float: Default + Copy + Clone + Debug {
    /// The wrapped type, e.g. f64
    type Primitive: Default + Float + Copy + Send + Sync + Debug + Display;
    /// The result of `to_bits()` on the wrapped type, e.g. u64
    type Bits: Num + Copy + std::hash::Hash + Send + Sync + Debug;
    /// The intermediate type used for comparison, e.g. i64
    type Signed: CheckedAdd
        + CheckedSub
        + WrappingAdd
        + WrappingSub
        + One
        + Copy
        + Send
        + Sync
        + Debug
        + Display;
    /// Integral type for holding size of any range. Must hold at least one more value than `Bits`.
    type SafeLen: CheckedAdd
        + CheckedSub
        + WrappingAdd
        + WrappingSub
        + NumAssign
        + PartialOrd
        + Hash
        + One
        + Copy
        + Send
        + Sync
        + Debug
        + Display;

    /// The minimum value available, in the `total_cmp`, range-set sense
    const MIN: Self::Primitive;
    /// The maximum value available, in the `total_cmp`, range-set sense
    const MAX: Self::Primitive;

    /// The minimum value available, in the usual floating point sense
    const MIN_FINITE: Self::Primitive;
    /// The maximum value available, in the usual floating point sense
    const MAX_FINITE: Self::Primitive;

    /// Transform Primitive into Signed, to allow comparison and addition
    fn to_ordered(x: Self::Primitive) -> Self::Signed;
    /// Transform Signed back to Primitive, presumably after some addition
    fn from_ordered(x: Self::Signed) -> Self::Primitive;
    /// Transform Primitive into a type with more concrete semantics
    fn to_bits(x: Self::Primitive) -> Self::Bits;
    /// Return the size of the inclusive range from start to end
    fn safe_len(start: Self::Signed, end: Self::Signed) -> Self::SafeLen;
    /// Converts [`SafeLen`] to `f64`, potentially losing precision for large values.
    fn safe_len_to_f64_lossy(len: Self::SafeLen) -> f64;
    /// Converts a `f64` to [`SafeLen`] using the formula `f as Self::SafeLen`. For large integer types, this will result in a loss of precision.
    fn f64_to_safe_len_lossy(f: f64) -> Self::SafeLen;
    /// return (x - 1) as `Self::Signed`
    fn safe_as_signed(x: Self::SafeLen) -> Self::Signed;
    /// Returns the ordering between `x` and `y`, as per the standard library's `f64::total_cmp`.
    /// Needed because f16 is not supported in `num_traits`.
    fn total_cmp(x: Self::Primitive, y: Self::Primitive) -> Ordering;

    /// Computes `self + (b - 1)` where `b` is of type [`SafeLen`].
    fn inclusive_end_from_start(a: Self::Primitive, b: Self::SafeLen) -> Self::Primitive {
        #[cfg(debug_assertions)]
        {
            let max_len = Self::prim_safe_len(a, Self::MAX);
            assert!(
                Self::SafeLen::zero() < b && b <= max_len,
                "b must be in range 1..=max_len (b = {b}, max_len = {max_len})"
            );
        }
        // If b is in range, two’s-complement wrap-around yields the correct inclusive end even if the add overflows
        Self::from_ordered(Self::to_ordered(a).wrapping_add(&Self::safe_as_signed(b)))
    }
    /// Computes `self - (b - 1)` where `b` is of type [`Integer::SafeLen`].
    fn start_from_inclusive_end(a: Self::Primitive, b: Self::SafeLen) -> Self::Primitive {
        #[cfg(debug_assertions)]
        {
            let max_len = Self::prim_safe_len(Self::MIN, a);
            assert!(
                Self::SafeLen::zero() < b && b <= max_len,
                "b must be in range 1..=max_len (b = {b}, max_len = {max_len})"
            );
        }
        // If b is in range, two’s-complement wrap-around yields the correct start even if the sub overflows
        Self::from_ordered(Self::to_ordered(a).wrapping_sub(&Self::safe_as_signed(b)))
    }
    /// Return the size of the inclusive range from start to end.
    fn prim_safe_len(start: Self::Primitive, end: Self::Primitive) -> Self::SafeLen {
        Self::safe_len(Self::to_ordered(start), Self::to_ordered(end))
    }
    /// Return true if the float is finite.
    fn is_finite(x: Self::Primitive) -> bool;
    /// Turn negative zero into positive zero, leave other numbers unchanged.
    fn normalize(x: Self::Primitive) -> Self::Primitive;
}

/// Total ordered f64, all values valid, including NaN, -0.0, +0.0, and infinities.
pub type TotalF64 = Total<f64>;
/// Total ordered f32, all values valid, including NaN, -0.0, +0.0, and infinities.
pub type TotalF32 = Total<f32>;
/// Total ordered f16, all values valid, including NaN, -0.0, +0.0, and infinities.
#[cfg(feature = "total_float_nightly_experimental")]
pub type TotalF16 = Total<f16>;

/// Construct a [`TotalF64`] from an `f64`.
#[must_use]
pub const fn tf64(x: f64) -> TotalF64 {
    Total::<f64>::new(x)
}

/// Construct a [`TotalF32`] from an `f32`.
#[must_use]
pub const fn tf32(x: f32) -> TotalF32 {
    Total::<f32>::new(x)
}

/// Construct a [`TotalF16`] from an `f16`.
#[cfg(feature = "total_float_nightly_experimental")]
#[must_use]
pub const fn tf16(x: f16) -> TotalF16 {
    Total::<f16>::new(x)
}

impl Float for f64 {
    type Primitive = Self;
    type Bits = u64;
    type Signed = i64;
    type SafeLen = i128;

    const MIN: Self = Self::from_bits(u64::MAX);
    const MAX: Self = Self::from_bits(0x7fff_ffff_ffff_ffff);
    const MIN_FINITE: Self = Self::MIN;
    const MAX_FINITE: Self = Self::MAX;

    fn to_bits(x: Self::Primitive) -> Self::Bits {
        x.to_bits()
    }

    /// Transforms the float bits into the monotonically ordered `i64` space used by `total_cmp`.
    fn to_ordered(x: Self::Primitive) -> Self::Signed {
        let mut bits = x.to_bits().cast_signed();
        bits ^= ((bits >> 63).cast_unsigned() >> 1).cast_signed();
        bits
    }

    /// Transforms the ordered `i64` space back into standard float bits.
    fn from_ordered(mut bits: Self::Signed) -> Self::Primitive {
        // Reversing the XOR transformation
        bits ^= ((bits >> 63).cast_unsigned() >> 1).cast_signed();
        Self::from_bits(bits.cast_unsigned())
    }

    fn safe_len(start: Self::Signed, end: Self::Signed) -> Self::SafeLen {
        // 1️⃣ Contract: caller promises start ≤ end  (checked only in debug builds)
        debug_assert!(start <= end, "start ≤ end required");

        // 2️⃣ Compute distance in `Self` then reinterpret‑cast to the first
        Self::SafeLen::from(end) - Self::SafeLen::from(start) + 1
    }

    #[expect(clippy::cast_precision_loss)]
    #[expect(clippy::use_self, reason = "f64 is not really Self")]
    fn safe_len_to_f64_lossy(len: Self::SafeLen) -> f64 {
        len as f64
    }

    #[expect(clippy::cast_possible_truncation)]
    fn f64_to_safe_len_lossy(f: f64) -> Self::SafeLen {
        f as Self::SafeLen
    }

    #[expect(clippy::cast_possible_truncation)]
    fn safe_as_signed(x: Self::SafeLen) -> Self::Signed {
        (x - 1) as Self::Signed
    }
    fn total_cmp(x: Self::Primitive, y: Self::Primitive) -> Ordering {
        x.total_cmp(&y)
    }
    fn is_finite(x: Self::Primitive) -> bool {
        x.is_finite() 
    }
    fn normalize(x: Self::Primitive) -> Self::Primitive {
        const NEG_ZERO: u64 = f64::to_bits(-0.0);
        if x.to_bits() == NEG_ZERO {
            0.0
        } else {
            x
        }
    }

}

impl Float for f32 {
    type Primitive = Self;
    type Bits = u32;
    type Signed = i32;
    type SafeLen = i64;

    const MIN: Self = Self::from_bits(u32::MAX);
    const MAX: Self = Self::from_bits(0x7fff_ffff);
    const MIN_FINITE: Self = Self::MIN;
    const MAX_FINITE: Self = Self::MAX;

    fn to_bits(x: Self::Primitive) -> Self::Bits {
        x.to_bits()
    }

    /// Transforms the float bits into the monotonically ordered `i64` space used by `total_cmp`.
    fn to_ordered(x: Self::Primitive) -> Self::Signed {
        let mut bits = x.to_bits().cast_signed();
        bits ^= ((bits >> 31).cast_unsigned() >> 1).cast_signed();
        bits
    }

    /// Transforms the ordered `i64` space back into standard float bits.
    fn from_ordered(mut bits: Self::Signed) -> Self::Primitive {
        // Reversing the XOR transformation
        bits ^= ((bits >> 31).cast_unsigned() >> 1).cast_signed();
        Self::from_bits(bits.cast_unsigned())
    }

    fn safe_len(start: Self::Signed, end: Self::Signed) -> Self::SafeLen {
        // 1️⃣ Contract: caller promises start ≤ end  (checked only in debug builds)
        debug_assert!(start <= end, "start ≤ end required");

        // 2️⃣ Compute distance in `Self` then reinterpret‑cast to the first
        Self::SafeLen::from(end) - Self::SafeLen::from(start) + 1
    }

    #[allow(clippy::cast_precision_loss)]
    fn safe_len_to_f64_lossy(len: Self::SafeLen) -> f64 {
        len as f64
    }

    #[expect(clippy::cast_possible_truncation)]
    fn f64_to_safe_len_lossy(f: f64) -> Self::SafeLen {
        f as Self::SafeLen
    }

    #[expect(clippy::cast_possible_truncation)]
    fn safe_as_signed(x: Self::SafeLen) -> Self::Signed {
        (x - 1) as Self::Signed
    }
    fn total_cmp(x: Self::Primitive, y: Self::Primitive) -> Ordering {
        x.total_cmp(&y)
    }
    fn is_finite(x: Self::Primitive) -> bool {
        x.is_finite() 
    }
    fn normalize(x: Self::Primitive) -> Self::Primitive {
        const NEG_ZERO: u32 = f32::to_bits(-0.0);
        if x.to_bits() == NEG_ZERO {
            0.0
        } else {
            x
        }
    }
}

#[cfg(feature = "total_float_nightly_experimental")]
impl Float for f16 {
    type Primitive = Self;
    type Bits = u16;
    type Signed = i16;
    type SafeLen = i32;

    const MIN: Self = Self::from_bits(u16::MAX);
    const MAX: Self = Self::from_bits(0x7fff);
    const MIN_FINITE: Self = Self::MIN;
    const MAX_FINITE: Self = Self::MAX;

    fn to_bits(x: Self::Primitive) -> Self::Bits {
        x.to_bits()
    }

    /// Transforms the float bits into the monotonically ordered `i64` space used by `total_cmp`.
    fn to_ordered(x: Self::Primitive) -> Self::Signed {
        let mut bits = x.to_bits().cast_signed();
        bits ^= ((bits >> 15).cast_unsigned() >> 1).cast_signed();
        bits
    }

    /// Transforms the ordered `i64` space back into standard float bits.
    fn from_ordered(mut bits: Self::Signed) -> Self::Primitive {
        // Reversing the XOR transformation
        bits ^= ((bits >> 15).cast_unsigned() >> 1).cast_signed();
        Self::from_bits(bits.cast_unsigned())
    }

    fn safe_len(start: Self::Signed, end: Self::Signed) -> Self::SafeLen {
        // 1️⃣ Contract: caller promises start ≤ end  (checked only in debug builds)
        debug_assert!(start <= end, "start ≤ end required");

        // 2️⃣ Compute distance in `Self` then reinterpret‑cast to the first
        Self::SafeLen::from(end) - Self::SafeLen::from(start) + 1
    }

    #[allow(clippy::cast_precision_loss)]
    fn safe_len_to_f64_lossy(len: Self::SafeLen) -> f64 {
        f64::from(len)
    }

    #[expect(clippy::cast_possible_truncation)]
    fn f64_to_safe_len_lossy(f: f64) -> Self::SafeLen {
        f as Self::SafeLen
    }

    #[expect(clippy::cast_possible_truncation)]
    fn safe_as_signed(x: Self::SafeLen) -> Self::Signed {
        (x - 1) as Self::Signed
    }
    fn total_cmp(x: Self::Primitive, y: Self::Primitive) -> Ordering {
        x.total_cmp(&y)
    }
    fn is_finite(x: Self::Primitive) -> bool {
        x.is_finite() 
    }
    fn normalize(x: Self::Primitive) -> Self::Primitive {
        const NEG_ZERO: u16 = f16::to_bits(-0.0);
        if x.to_bits() == NEG_ZERO {
            0.0
        } else {
            x
        }
    }
}

/// Experimental: A transparent wrapper around [`f64`] with total ordering.
///
/// Comparison, equality, and hashing all agree with [`f64::total_cmp`].
///
/// # Enabling
///
/// This type is experimental and must be enabled with the `total_float_experimental` feature.
/// ```bash
/// cargo add range-set-blaze --features "total_float_experimental"
/// ```
#[repr(transparent)]
#[derive(Copy, Clone, Default, Debug)]
pub struct Total<T: Float>(pub T::Primitive);

impl<T: Float> Total<T> {
    /// The minimum value in [`f64::total_cmp`] order.
    pub const MIN: Self = Self(T::MIN);

    /// The maximum value in [`f64::total_cmp`] order.
    pub const MAX: Self = Self(T::MAX);

    /// Creates a new [`Total`] from a primitive float.
    #[must_use]
    pub const fn new(x: T::Primitive) -> Self {
        Self(x)
    }

    /// Computes `self + (b - 1)` where `b` is of type [`SafeLen`].
    #[must_use]
    pub fn inclusive_end_from_start(self, b: T::SafeLen) -> Self {
        Self(T::inclusive_end_from_start(self.0, b))
    }

    /// Computes `self - (b - 1)` where `b` is of type [`SafeLen`].
    #[must_use]
    pub fn start_from_inclusive_end(self, b: T::SafeLen) -> Self {
        Self(T::start_from_inclusive_end(self.0, b))
    }

    /// Returns the wrapped [`f64`] value.
    #[must_use]
    pub const fn into_inner(self) -> T::Primitive {
        self.0
    }

    /// Transforms the float bits into the monotonically ordered Signed space used by `total_cmp`.
    pub fn to_ordered(self) -> T::Signed {
        T::to_ordered(self.0)
    }

    /// Transforms the ordered Signed space back into standard float bits.
    pub fn from_ordered(x: T::Signed) -> Self {
        Self(T::from_ordered(x))
    }

    /// Returns the next float in total order.
    ///
    /// Panics on overflow if `self` is the maximum value in total order.
    #[must_use]
    pub fn next(self) -> Self {
        let ordered = self.to_ordered();
        Self::from_ordered(ordered.wrapping_add(&T::Signed::one()))
    }

    /// Returns the previous float in total order.
    ///
    /// Panics on overflow if `self` is the minimum value in total order.
    #[must_use]
    pub fn prev(self) -> Self {
        let ordered = self.to_ordered();
        Self::from_ordered(ordered.wrapping_sub(&T::Signed::one()))
    }

    /// Returns the next float in total order.
    ///
    /// Returns [`None`] if `self` is the maximum value in total order.
    #[must_use]
    pub fn checked_next(self) -> Option<Self> {
        // let ordered = self.to_ordered();
        self.to_ordered()
            .checked_add(&T::Signed::one())
            .map(Self::from_ordered)
    }

    /// Returns the previous float in total order.
    ///
    /// Returns [`None`] if `self` is the minimum value in total order.
    #[must_use]
    pub fn checked_prev(self) -> Option<Self> {
        self.to_ordered()
            .checked_sub(&T::Signed::one())
            .map(Self::from_ordered)
    }

    /// Converts an inclusive primitive range into an inclusive [`Total`] range.
    #[must_use]
    pub fn range(range: RangeInclusive<T::Primitive>) -> RangeInclusive<Self> {
        let (start, end) = range.into_inner();
        Self(start)..=Self(end)
    }

    /// Converts inclusive primitive ranges into inclusive [`Total`] ranges.
    pub fn ranges<I>(ranges: I) -> impl Iterator<Item = RangeInclusive<Self>>
    where
        I: IntoIterator<Item = RangeInclusive<T::Primitive>>,
    {
        ranges.into_iter().map(Self::range)
    }

    /// Converts primitive values into ordered [`Total`] values.
    pub fn values<I>(values: I) -> impl Iterator<Item = Self>
    where
        I: IntoIterator<Item = T::Primitive>,
    {
        values.into_iter().map(Self)
    }

    /// Views primitive [`f64`] values as ordered [`Total64`] values.
    ///
    /// This runs in `O(1)` and does not allocate.
    #[must_use]
    pub const fn slice(values: &[T::Primitive]) -> &[Self] {
        // SAFETY: Total is #[repr(transparent)] over T::Primitive, making `&[T::Primitive]`
        // and `&[Total]` entirely interchangeable in layout and lifetimes.
        unsafe { core::mem::transmute::<&[T::Primitive], &[Self]>(values) }
    }
}

/// Views  [`Total`] values as primitive values.
///
/// This runs in `O(1)` and does not allocate.
#[must_use]
pub const fn primitive_slice<T: Float>(values: &[T]) -> &[T::Primitive] {
    // SAFETY: Float is #[repr(transparent)] over T::Primitive, making `&[T::Primitive]`
    // and `&[Float]` entirely interchangeable in layout and lifetimes.
    unsafe { core::mem::transmute::<&[T], &[T::Primitive]>(values) }
}

impl<T: Float> PartialEq for Total<T> {
    fn eq(&self, other: &Self) -> bool {
        T::to_bits(self.0) == T::to_bits(other.0)
    }
}

impl<T: Float> Eq for Total<T> {}

impl<T: Float> PartialOrd for Total<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<T: Float> Ord for Total<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        T::total_cmp(self.0, other.0)
    }
}

impl<T: Float> Hash for Total<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        T::to_bits(self.0).hash(state);
    }
}

///```
/// use range_set_blaze::{RangeSetBlaze, TotalF64};
/// let set = RangeSetBlaze::from_iter([TotalF64::new(3.0)..=TotalF64::new(5.0)]);
/// assert!(set.contains(TotalF64::new(3.1)));
/// assert!(!set.contains(TotalF64::new(2.9)));
///
/// let set = RangeSetBlaze::from(TotalF64::range(3.0..=5.0));
/// assert!(set.contains(TotalF64::new(4.9)));
/// assert!(!set.contains(TotalF64::new(5.1)));
///
/// let set = RangeSetBlaze::from_iter(TotalF64::ranges([3.0..=5.0, 7.0..=9.0]));
/// assert!(set.contains(TotalF64::new(4.0)));
/// assert!(!set.contains(TotalF64::new(6.0)));
///```
impl<T: Float> crate::Integer for Total<T> {
    type SafeLen = T::SafeLen;

    #[inline]
    fn checked_add_one(self) -> Option<Self> {
        self.checked_next()
    }

    // This moves to the next representable float in total_cmp order, not a numeric + 1.0.
    #[inline]
    fn add_one(self) -> Self {
        self.next()
    }

    #[inline]
    // This moves to the previous representable float in total_cmp order, not a numeric - 1.0.
    fn sub_one(self) -> Self {
        self.prev()
    }

    #[inline]
    fn assign_sub_one(&mut self) {
        *self = self.prev();
    }

    // Ideally, we would `impl std::iter::Step for TotalF64` and just call Range::next(), but that's still experimental.
    #[inline]
    fn range_next(range: &mut RangeInclusive<Self>) -> Option<Self> {
        if range.is_empty() {
            None
        } else if range.start() == range.end() && *range.start() == Self::MAX {
            // This is cheating, but I think it still fulfills the contract
            let next = *range.start();
            *range = next..=range.end().prev();
            Some(next)
        } else {
            let next = *range.start();
            *range = (next.next())..=*range.end();
            Some(next)
        }
    }

    #[inline]
    fn range_next_back(range: &mut RangeInclusive<Self>) -> Option<Self> {
        if range.is_empty() {
            None
        } else if range.start() == range.end() && *range.start() == Self::MIN {
            // This is cheating, but I think it still fulfills the contract
            let last = *range.end();
            *range = last.next()..=last;
            Some(last)
        } else {
            let last = *range.end();
            *range = *range.start()..=last.prev();
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
        // no way to do the fancy thing
        RangeSetBlaze::from_iter(slice.as_ref())
    }

    fn safe_len(r: &RangeInclusive<Self>) -> Self::SafeLen {
        T::prim_safe_len(r.start().into_inner(), r.end().into_inner())
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
        assert_eq!(TotalF64::range(10.0..=20.0), tf64(10.0)..=tf64(20.0));
        assert_eq!(
            TotalF64::ranges([10.0..=20.0, 30.0..=40.0]).collect::<Vec<_>>(),
            vec![tf64(10.0)..=tf64(20.0), tf64(30.0)..=tf64(40.0)]
        );
    }

    #[test]
    fn next_and_prev_step_through_zero_in_total_order() {
        assert_eq!(tf64(-0.0).next(), tf64(0.0));
        assert_eq!(tf64(0.0).prev(), tf64(-0.0));
        assert_eq!(tf64(0.0).next(), tf64(f64::from_bits(1)));
        assert_eq!(
            tf64(-0.0).prev(),
            tf64(f64::from_bits(0x8000_0000_0000_0001))
        );
    }

    #[test]
    fn next_and_prev_wrap() {
        // These should be true in release mode, but panic in debug as expected
        // assert_eq!(TotalF64::MAX.next(), TotalF64::MIN);
        // assert_eq!(TotalF64::MIN.prev(), TotalF64::MAX);
        assert_eq!(TotalF64::MAX.checked_next(), None);
        assert_eq!(TotalF64::MIN.checked_prev(), None);
    }

    #[test]
    fn next_and_prev_step_around_infinities() {
        assert_eq!(tf64(f64::MAX).next(), tf64(f64::INFINITY));
        assert_eq!(tf64(f64::INFINITY).prev(), tf64(f64::MAX));
        assert_eq!(tf64(f64::NEG_INFINITY).next(), tf64(-f64::MAX));
        assert_eq!(tf64(-f64::MAX).prev(), tf64(f64::NEG_INFINITY));
    }

    #[test]
    fn checked_next_and_prev_stop_at_total_order_boundaries() {
        assert_eq!(TotalF64::MIN.checked_prev(), None);
        assert_eq!(TotalF64::MAX.checked_next(), None);
        assert_eq!(TotalF64::MIN.checked_next(), Some(TotalF64::MIN.next()));
        assert_eq!(TotalF64::MAX.checked_prev(), Some(TotalF64::MAX.prev()));
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
    fn next_and_prev_are_neighbors_in_total_order() {
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
            assert_eq!(value.next().prev(), value);
            assert_eq!(value.prev().next(), value);
        }
    }

    fn hash(value: TotalF64) -> u64 {
        let mut hasher = DefaultHasher::new();
        value.hash(&mut hasher);
        hasher.finish()
    }
}
