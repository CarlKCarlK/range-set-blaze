//! Ordered `f64` support.

use core::cmp::Ordering;
use core::hash::{Hash, Hasher};
use core::ops::RangeInclusive;

use crate::float::total::Float;
use num_traits::One;
use num_traits::ops::checked::{CheckedAdd, CheckedSub};
use num_traits::ops::wrapping::{WrappingAdd, WrappingSub};
use std::fmt::Debug;

#[cfg(feature = "from_slice")]
use crate::RangeSetBlaze;

/// Error type, only used for [`TryFrom`] implementations.
pub enum Error {
    /// The float is not finite (NaN or infinity).
    FloatIsNotFinite,
}

/// Total ordered f64, excluding NaN, -0.0, +0.0, and infinities.
pub type FiniteF64 = Finite<f64>;
/// Total ordered f32, excluding NaN, -0.0, +0.0, and infinities.
pub type FiniteF32 = Finite<f32>;
/// Total ordered f16, excluding NaN, -0.0, +0.0, and infinities.
#[cfg(feature = "total_float_nightly_experimental")]
pub type FiniteF16 = Finite<f16>;

/// Construct a [`FiniteF64`] from an `f64`.
#[must_use]
pub fn ff64(x: f64) -> FiniteF64 {
    Finite::<f64>::new(x)
}

/// Construct a [`FiniteF32`] from an `f32`.
#[must_use]
pub fn ff32(x: f32) -> FiniteF32 {
    Finite::<f32>::new(x)
}

/// Construct a [`FiniteF16`] from an `f16`.
#[cfg(feature = "total_float_nightly_experimental")]
#[must_use]
pub fn ff16(x: f16) -> FiniteF16 {
    Finite::<f16>::new(x)
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
pub struct Finite<T: Float>(pub T::Primitive);

impl<T: Float> Finite<T> {
    /// The minimum value in [`f64::total_cmp`] order.
    pub const MIN: Self = Self(T::MIN_FINITE);

    /// The maximum value in [`f64::total_cmp`] order.
    pub const MAX: Self = Self(T::MAX_FINITE);

    /// Creates a new [`Finite`] from a primitive float.
    /// # Panics
    ///
    /// Panics if start (inclusive) is greater than end (inclusive).
    #[must_use]
    pub fn new(x: T::Primitive) -> Self {
        assert!(T::is_finite(x), "Finite type requires a finite value");
        Self(T::normalize(x))
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

    /// Converts an inclusive primitive range into an inclusive [`Finite`] range.
    #[must_use]
    pub fn range(range: RangeInclusive<T::Primitive>) -> RangeInclusive<Self> {
        let (start, end) = range.into_inner();
        Self(start)..=Self(end)
    }

    /// Converts inclusive primitive ranges into inclusive [`Finite`] ranges.
    pub fn ranges<I>(ranges: I) -> impl Iterator<Item = RangeInclusive<Self>>
    where
        I: IntoIterator<Item = RangeInclusive<T::Primitive>>,
    {
        ranges.into_iter().map(Self::range)
    }

    /// Converts primitive values into ordered [`Finite`] values.
    pub fn values<I>(values: I) -> impl Iterator<Item = Self>
    where
        I: IntoIterator<Item = T::Primitive>,
    {
        values.into_iter().map(Self)
    }

    /// Views primitive [`f64`] values as ordered [`Finite64`] values.
    ///
    /// This runs in `O(1)` and does not allocate.
    #[must_use]
    pub const fn slice(values: &[T::Primitive]) -> &[Self] {
        // SAFETY: Finite is #[repr(transparent)] over T::Primitive, making `&[T::Primitive]`
        // and `&[Finite]` entirely interchangeable in layout and lifetimes.
        unsafe { core::mem::transmute::<&[T::Primitive], &[Self]>(values) }
    }
}

/// Views  [`Finite`] values as primitive values.
///
/// This runs in `O(1)` and does not allocate.
#[must_use]
pub const fn primitive_slice<T: Float>(values: &[T]) -> &[T::Primitive] {
    // SAFETY: Float is #[repr(transparent)] over T::Primitive, making `&[T::Primitive]`
    // and `&[Float]` entirely interchangeable in layout and lifetimes.
    unsafe { core::mem::transmute::<&[T], &[T::Primitive]>(values) }
}

impl<T: Float> PartialEq for Finite<T> {
    fn eq(&self, other: &Self) -> bool {
        T::to_bits(self.0) == T::to_bits(other.0)
    }
}

impl<T: Float> Eq for Finite<T> {}

impl<T: Float> PartialOrd for Finite<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<T: Float> Ord for Finite<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        T::total_cmp(self.0, other.0)
    }
}

impl<T: Float> Hash for Finite<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        T::to_bits(self.0).hash(state);
    }
}

// impl<T: Float> TryFrom<T::Primitive> for Finite<T> {
//     type Error = Error;

//     fn try_from(value: T::Primitive) -> Result<Self, Error> {
//         if T::is_finite(value) {
//             Ok(Self::new(T::normalize(value)))
//         } else {
//             Err(Error::FloatIsNotFinite)
//         }
//     }
// }

///```
/// use range_set_blaze::{RangeSetBlaze, FiniteF64};
/// let set = RangeSetBlaze::from_iter([FiniteF64::new(3.0)..=FiniteF64::new(5.0)]);
/// assert!(set.contains(FiniteF64::new(3.1)));
/// assert!(!set.contains(FiniteF64::new(2.9)));
///
/// let set = RangeSetBlaze::from(FiniteF64::range(3.0..=5.0));
/// assert!(set.contains(FiniteF64::new(4.9)));
/// assert!(!set.contains(FiniteF64::new(5.1)));
///
/// let set = RangeSetBlaze::from_iter(FiniteF64::ranges([3.0..=5.0, 7.0..=9.0]));
/// assert!(set.contains(FiniteF64::new(4.0)));
/// assert!(!set.contains(FiniteF64::new(6.0)));
///```
impl<T: Float> crate::Integer for Finite<T> {
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

    // Ideally, we would `impl std::iter::Step for FiniteF64` and just call Range::next(), but that's still experimental.
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
        assert_eq!(FiniteF64::range(10.0..=20.0), tf64(10.0)..=tf64(20.0));
        assert_eq!(
            FiniteF64::ranges([10.0..=20.0, 30.0..=40.0]).collect::<Vec<_>>(),
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
        // assert_eq!(FiniteF64::MAX.next(), FiniteF64::MIN);
        // assert_eq!(FiniteF64::MIN.prev(), FiniteF64::MAX);
        assert_eq!(FiniteF64::MAX.checked_next(), None);
        assert_eq!(FiniteF64::MIN.checked_prev(), None);
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
        assert_eq!(FiniteF64::MIN.checked_prev(), None);
        assert_eq!(FiniteF64::MAX.checked_next(), None);
        assert_eq!(FiniteF64::MIN.checked_next(), Some(FiniteF64::MIN.next()));
        assert_eq!(FiniteF64::MAX.checked_prev(), Some(FiniteF64::MAX.prev()));
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
            assert!(FiniteF64::MIN <= value);
            assert!(value <= FiniteF64::MAX);
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

    fn hash(value: FiniteF64) -> u64 {
        let mut hasher = DefaultHasher::new();
        value.hash(&mut hasher);
        hasher.finish()
    }
}
