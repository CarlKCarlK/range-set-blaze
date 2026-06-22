//! impl Integer for `TotalF128`

#[cfg(feature = "from_slice")]
use crate::RangeSetBlaze;
use crate::total_f128::TotalF128;
use crate::UIntPlusOne;
use crate::Integer;
use std::ops::RangeInclusive;
use num_traits::Zero;

///```
/// use range_set_blaze::{RangeSetBlaze, TotalF128};
/// let set = RangeSetBlaze::from_iter([TotalF128(3.0)..=TotalF128(5.0)]);
/// assert!(set.contains(TotalF128(3.1)));
/// assert!(!set.contains(TotalF128(2.9)));
///
/// let set = RangeSetBlaze::from(TotalF128::range(3.0..=5.0));
/// assert!(set.contains(TotalF128(4.9)));
/// assert!(!set.contains(TotalF128(5.1)));
///
/// let set = RangeSetBlaze::from_iter(TotalF128::ranges([3.0..=5.0, 7.0..=9.0]));
/// assert!(set.contains(TotalF128(4.0)));
/// assert!(!set.contains(TotalF128(6.0)));
///```
impl Integer for TotalF128 {
    type SafeLen = UIntPlusOne<u128>;

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

    #[inline]
    fn range_next(range: &mut RangeInclusive<Self>) -> Option<Self> {
        if range.is_empty() {
            None
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

    #[allow(clippy::cast_sign_loss)]
    fn safe_len(r: &RangeInclusive<Self>) -> <Self as Integer>::SafeLen {
        debug_assert!(r.start() <= r.end());
        let start = r.start().to_ordered_i128();
        let end = r.end().to_ordered_i128();
        // Signed sub may overflow, but casting preserves correct unsigned distance
        let less1 = end.overflowing_sub(start).0 as u128;
        let less1 = UIntPlusOne::UInt(less1);
        less1 + UIntPlusOne::UInt(1)
    }
    #[allow(clippy::cast_precision_loss)]
    #[allow(clippy::cast_possible_truncation)]
    fn safe_len_to_f64_lossy(len: Self::SafeLen) -> f64 {
        match len {
            UIntPlusOne::UInt(v) => v as f64,
            UIntPlusOne::MaxPlusOne => UIntPlusOne::<u128>::max_plus_one_as_f64(),
        }
    }

    #[allow(clippy::cast_sign_loss)]
    #[allow(clippy::cast_possible_truncation)]
    fn f64_to_safe_len_lossy(f: f64) -> Self::SafeLen {
        if f >= UIntPlusOne::<u128>::max_plus_one_as_f64() {
            UIntPlusOne::MaxPlusOne
        } else {
            UIntPlusOne::UInt(f as u128)
        }
    }

    /// Computes the inclusive end of a range starting at `self` with length `b`,
    /// by returning `self + (b - 1)`.
    ///
    /// # Panics
    /// In debug builds, panics if `b` is zero or too large to compute a valid result.
    /// In release builds, this will either panic or wrap on overflow; the result may be meaningless,
    /// but it is always defined and safe (never causes undefined behavior)
    #[allow(clippy::cast_possible_wrap)]
    fn inclusive_end_from_start(self, b: Self::SafeLen) -> Self {
        #[cfg(debug_assertions)]
        {
            let max_len = Self::safe_len(&(self..=Self::MAX));
            assert!(
                UIntPlusOne::zero() < b && b <= max_len,
                "b must be in range 1..=max_len (b = {b}, max_len = {max_len})"
            );
        }

        let UIntPlusOne::UInt(b) = b else {
            if self == Self::MIN {
                return Self::MAX;
            }
            // This is only reached in release builds.
            let max_len = Self::safe_len(&(self..=Self::MAX));
            panic!("b must be in range 1..=max_len (b = {b}, max_len = {max_len})");
        };
        // If b is in range, two’s-complement wrap-around yields the correct inclusive end even if the add overflows
        Self::from_ordered_i128(self.to_ordered_i128().wrapping_add((b - 1) as i128))
    }

    /// Computes the inclusive start of a range ending at `self` with length `b`,
    /// by returning `self - (b - 1)`.
    ///
    /// # Panics
    /// In debug builds, panics if `b` is zero or too large to compute a valid result.
    /// In release builds, this will either panic or wrap on overflow; the result may be meaningless,
    /// but it is always defined and safe (never causes undefined behavior).
    #[allow(clippy::cast_possible_wrap)]
    fn start_from_inclusive_end(self, b: Self::SafeLen) -> Self {
        #[cfg(debug_assertions)]
        {
            let max_len = Self::safe_len(&(Self::MIN..=self));
            assert!(
                UIntPlusOne::zero() < b && b <= max_len,
                "b must be in range 1..=max_len (b = {b}, max_len = {max_len})"
            );
        }

        let UIntPlusOne::UInt(b) = b else {
            if self == Self::MAX {
                return Self::MIN;
            }
            // This is only reached in release builds.
            let max_len = Self::safe_len(&(Self::MIN..=self));
            panic!("b must be in range 1..=max_len (b = {b}, max_len = {max_len})");
        };

        // If b is in range, two’s-complement wrap-around yields the correct inclusive start even if the subtraction overflows
        Self::from_ordered_i128(self.to_ordered_i128().wrapping_sub((b - 1) as i128))
    }
}
