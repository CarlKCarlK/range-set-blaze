//! impl Integer for `TotalF64`

#[cfg(feature = "from_slice")]
use crate::RangeSetBlaze;
use crate::total_f64::TotalF64;

use core::ops::RangeInclusive;

///```
/// use range_set_blaze::{RangeSetBlaze, TotalF64};
/// let set = RangeSetBlaze::from_iter([TotalF64(3.0)..=TotalF64(5.0)]);
/// assert!(set.contains(TotalF64(3.1)));
/// assert!(!set.contains(TotalF64(2.9)));
///
/// let set = RangeSetBlaze::from(TotalF64::range(3.0..=5.0));
/// assert!(set.contains(TotalF64(4.9)));
/// assert!(!set.contains(TotalF64(5.1)));
///
/// let set = RangeSetBlaze::from_iter(TotalF64::ranges([3.0..=5.0, 7.0..=9.0]));
/// assert!(set.contains(TotalF64(4.0)));
/// assert!(!set.contains(TotalF64(6.0)));
///```
impl crate::Integer for TotalF64 {
    type SafeLen = i128;

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

    // Ideally, we would `std::iter::Step for TotalF64` and just call Range::next(), but that's still experimental.
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

    #[allow(clippy::cast_sign_loss)]
    fn safe_len(r: &RangeInclusive<Self>) -> Self::SafeLen {
        // 1️⃣ Contract: caller promises start ≤ end  (checked only in debug builds)
        debug_assert!(r.start() <= r.end(), "start ≤ end required");

        // 2️⃣ Compute distance in `Self` then reinterpret‑cast to the first
        Self::SafeLen::from(r.end().to_ordered_i64())
            - Self::SafeLen::from(r.start().to_ordered_i64())
            + 1
    }

    #[allow(clippy::cast_precision_loss, clippy::cast_lossless)]
    fn safe_len_to_f64_lossy(len: Self::SafeLen) -> f64 {
        len as f64
    }

    #[allow(clippy::cast_sign_loss)]
    #[allow(clippy::cast_precision_loss)]
    #[allow(clippy::cast_possible_truncation)]
    fn f64_to_safe_len_lossy(f: f64) -> Self::SafeLen {
        f as Self::SafeLen
    }

    #[allow(clippy::cast_possible_truncation)]
    #[allow(clippy::cast_possible_wrap)]
    fn inclusive_end_from_start(self, b: Self::SafeLen) -> Self {
        #[cfg(debug_assertions)]
        {
            let max_len = Self::safe_len(&(self..=Self::MAX));
            assert!(
                b > 0 && b <= max_len,
                "b must be in range 1..=max_len (b = {b}, max_len = {max_len})"
            );
        }
        // If b is in range, two’s-complement wrap-around yields the correct inclusive end even if the add overflows
        Self::from_ordered_i64(self.to_ordered_i64().wrapping_add((b - 1) as i64))
    }

    #[allow(clippy::cast_possible_truncation)]
    #[allow(clippy::cast_possible_wrap)]
    fn start_from_inclusive_end(self, b: Self::SafeLen) -> Self {
        #[cfg(debug_assertions)]
        {
            let max_len = Self::safe_len(&(Self::MIN..=self));
            assert!(
                0 < b && b <= max_len,
                "b must be in range 1..=max_len (b = {b}, max_len = {max_len})"
            );
        }
        // If b is in range, two’s-complement wrap-around yields the correct start even if the sub overflows
        Self::from_ordered_i64(self.to_ordered_i64().wrapping_sub((b - 1) as i64))
    }
}
