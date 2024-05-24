use crate::UIntPlusOne;
#[cfg(feature = "from_slice")]
use crate::{from_slice::FromSliceIter, RangeSetBlaze};
use core::hash::Hash;
use core::net::{Ipv4Addr, Ipv6Addr};
use core::ops::{AddAssign, SubAssign};
use core::{fmt, ops::RangeInclusive};
use num_traits::ops::overflowing::OverflowingSub;

#[cfg(feature = "from_slice")]
const LANES: usize = 16;

/// Elements of [`RangeSetBlaze`] and the keys of [`RangeMapBlaze`], specifically `u8` to `u128` (including `usize`), `i8` to `i128`
/// (including `isize`), `char`, `Ipv4Addr`, and `Ipv6Addr`.
///
/// [`RangeSetBlaze`]: crate::RangeSetBlaze
/// [`RangeMapBlaze`]: crate::RangeMapBlaze
pub trait Integer: Copy + PartialEq + PartialOrd + Ord + fmt::Debug + Send + Sync {
    /// cmk doc
    fn checked_add_one(self) -> Option<Self>;
    /// cmk doc
    #[must_use]
    fn add_one(self) -> Self;
    /// cmk doc
    #[must_use]
    fn sub_one(self) -> Self;
    /// cmk doc
    fn assign_sub_one(&mut self);

    /// cmk doc
    #[must_use]
    fn exhausted_range() -> RangeInclusive<Self> {
        Self::max_value()..=Self::min_value()
    }

    /// cmk doc
    fn range_next(range: &mut RangeInclusive<Self>) -> Option<Self>;

    /// cmk doc
    fn range_next_back(range: &mut RangeInclusive<Self>) -> Option<Self>;

    /// cmk doc
    #[must_use]
    fn min_value() -> Self;
    /// cmk doc
    #[must_use]
    fn max_value() -> Self;

    #[cfg(feature = "from_slice")]
    /// A definition of [`RangeSetBlaze::from_slice()`] specific to this integer type.
    fn from_slice(slice: impl AsRef<[Self]>) -> RangeSetBlaze<Self>;

    /// The type of the length of a [`RangeSetBlaze`]. For example, the length of a `RangeSetBlaze<u8>` is `usize`. Note
    /// that it can't be `u8` because the length ranges from 0 to 256, which is one too large for `u8`.
    ///
    /// In general, `SafeLen` will be `usize` if `usize` is always large enough. If not, `SafeLen` will be the smallest unsigned integer
    /// type that is always large enough. However, for `u128` and `i128`, nothing is always large enough so
    ///  `SafeLen` will be `u128` and we prohibit the largest value from being used in [`Integer`].
    ///
    /// # Examples
    /// ```
    /// use range_set_blaze::{RangeSetBlaze, Integer};
    ///
    /// let len: <u8 as Integer>::SafeLen = RangeSetBlaze::from_iter([0u8..=255]).len();
    /// assert_eq!(len, 256);
    /// ```
    type SafeLen: Hash
        + Copy
        + PartialEq
        + PartialOrd
        + num_traits::Zero
        + num_traits::One
        + AddAssign
        + SubAssign;

    /// Returns the length of a range without any overflow.
    ///
    /// # Example
    /// ```
    /// use range_set_blaze::Integer;
    ///
    /// assert_eq!(<u8 as Integer>::safe_len(&(0..=255)), 256);
    /// ```
    fn safe_len(range: &RangeInclusive<Self>) -> <Self as Integer>::SafeLen;

    // FUTURE define .len() SortedDisjoint

    /// Converts a `f64` to [`Integer::SafeLen`] using the formula `f as Self::SafeLen`. For large integer types, this will result in a loss of precision.
    fn f64_to_safe_len(f: f64) -> Self::SafeLen;

    /// Converts [`Integer::SafeLen`] to `f64` using the formula `len as f64`. For large integer types, this will result in a loss of precision.
    fn safe_len_to_f64(len: Self::SafeLen) -> f64;

    /// Computes `a + (b - 1) as Self`
    #[must_use]
    fn add_len_less_one(self, b: Self::SafeLen) -> Self;

    /// Computes `a - (b - 1) as Self`
    #[must_use]
    fn sub_len_less_one(self, b: Self::SafeLen) -> Self;
}

/// Define the Integer trait operations for a given integer type.
macro_rules! impl_integer_ops {
    ($type:ty, $type2:ty) => {
        #[inline]
        fn checked_add_one(self) -> Option<Self> {
            self.checked_add(1)
        }

        #[inline]
        fn add_one(self) -> Self {
            self + 1
        }

        #[inline]
        fn sub_one(self) -> Self {
            self - 1
        }

        #[inline]
        fn assign_sub_one(&mut self) {
            *self -= 1;
        }

        #[inline]
        fn range_next(range: &mut RangeInclusive<Self>) -> Option<Self> {
            range.next()
        }

        #[inline]
        fn range_next_back(range: &mut RangeInclusive<Self>) -> Option<Self> {
            range.next_back()
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
            FromSliceIter::<Self, LANES>::new(slice.as_ref()).collect()
        }

        #[allow(clippy::cast_sign_loss)]
        fn safe_len(r: &RangeInclusive<Self>) -> <Self as Integer>::SafeLen {
            r.end().overflowing_sub(r.start()).0 as $type2 as <Self as Integer>::SafeLen + 1
        }

        #[allow(clippy::cast_precision_loss)]
        fn safe_len_to_f64(len: Self::SafeLen) -> f64 {
            len as f64
        }

        #[allow(clippy::cast_sign_loss)]
        #[allow(clippy::cast_precision_loss)]
        #[allow(clippy::cast_possible_truncation)]
        fn f64_to_safe_len(f: f64) -> Self::SafeLen {
            f as Self::SafeLen
        }

        #[allow(clippy::cast_possible_truncation)]
        #[allow(clippy::cast_possible_wrap)]
        fn add_len_less_one(self, b: Self::SafeLen) -> Self {
            debug_assert!(b > 0);
            self + (b - 1) as Self
        }

        #[allow(clippy::cast_possible_truncation)]
        #[allow(clippy::cast_possible_wrap)]
        fn sub_len_less_one(self, b: Self::SafeLen) -> Self {
            self - (b - 1) as Self
        }
    };
}

impl Integer for i8 {
    #[cfg(target_pointer_width = "32")]
    type SafeLen = usize;
    #[cfg(target_pointer_width = "64")]
    type SafeLen = usize;

    impl_integer_ops!(i8, u8);
}

impl Integer for u8 {
    #[cfg(target_pointer_width = "32")]
    type SafeLen = usize;
    #[cfg(target_pointer_width = "64")]
    type SafeLen = usize;

    impl_integer_ops!(u8, Self);
}

impl Integer for i32 {
    #[cfg(target_pointer_width = "32")]
    type SafeLen = u64;
    #[cfg(target_pointer_width = "64")]
    type SafeLen = usize;

    impl_integer_ops!(i32, u32);
}

impl Integer for u32 {
    #[cfg(target_pointer_width = "32")]
    type SafeLen = u64;
    #[cfg(target_pointer_width = "64")]
    type SafeLen = usize;

    impl_integer_ops!(u32, Self);
}

impl Integer for i64 {
    #[cfg(target_pointer_width = "32")]
    type SafeLen = u128;
    #[cfg(target_pointer_width = "64")]
    type SafeLen = u128;

    impl_integer_ops!(i64, u64);
}

impl Integer for u64 {
    #[cfg(target_pointer_width = "32")]
    type SafeLen = u128;
    #[cfg(target_pointer_width = "64")]
    type SafeLen = u128;

    impl_integer_ops!(u64, Self);
}

impl Integer for i128 {
    #[cfg(target_pointer_width = "32")]
    type SafeLen = UIntPlusOne<u128>;
    #[cfg(target_pointer_width = "64")]
    type SafeLen = UIntPlusOne<u128>;

    #[inline]
    fn checked_add_one(self) -> Option<Self> {
        self.checked_add(1)
    }

    #[inline]
    fn add_one(self) -> Self {
        self + 1
    }

    #[inline]
    fn sub_one(self) -> Self {
        self - 1
    }

    #[inline]
    fn assign_sub_one(&mut self) {
        *self -= 1;
    }

    #[inline]
    fn range_next(range: &mut RangeInclusive<Self>) -> Option<Self> {
        range.next()
    }

    #[inline]
    fn range_next_back(range: &mut RangeInclusive<Self>) -> Option<Self> {
        range.next_back()
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
    fn from_slice(slice: impl AsRef<[Self]>) -> crate::RangeSetBlaze<Self> {
        slice.as_ref().iter().collect()
    }

    #[allow(clippy::cast_sign_loss)]
    fn safe_len(r: &RangeInclusive<Self>) -> <Self as Integer>::SafeLen {
        debug_assert!(r.start() <= r.end());
        let less1 = r.end().overflowing_sub(r.start()).0 as u128;
        let less1 = UIntPlusOne::UInt(less1);
        less1 + UIntPlusOne::UInt(1)
    }

    #[allow(clippy::cast_precision_loss)]
    #[allow(clippy::cast_possible_truncation)]
    fn safe_len_to_f64(len: Self::SafeLen) -> f64 {
        match len {
            UIntPlusOne::UInt(v) => v as f64,
            UIntPlusOne::MaxPlusOne => UIntPlusOne::<u128>::max_plus_one_as_f64(),
        }
    }

    #[allow(clippy::cast_sign_loss)]
    #[allow(clippy::cast_possible_truncation)]
    fn f64_to_safe_len(f: f64) -> Self::SafeLen {
        if f >= UIntPlusOne::<u128>::max_plus_one_as_f64() {
            UIntPlusOne::MaxPlusOne
        } else {
            UIntPlusOne::UInt(f as u128)
        }
    }

    #[allow(clippy::cast_possible_wrap)]
    fn add_len_less_one(self, b: Self::SafeLen) -> Self {
        let UIntPlusOne::UInt(b) = b else {
            debug_assert!(false, "Too large to add to i128");
            return Self::MAX;
        };
        debug_assert!(b > 0);
        self + (b - 1) as Self
    }

    #[allow(clippy::cast_possible_wrap)]
    fn sub_len_less_one(self, b: Self::SafeLen) -> Self {
        // a - (b - 1) as Self
        let UIntPlusOne::UInt(b) = b else {
            debug_assert!(false, "Too large to subtract from i128");
            return Self::MIN;
        };
        debug_assert!(b > 0);
        self - (b - 1) as Self
    }
}

impl Integer for u128 {
    #[cfg(target_pointer_width = "32")]
    type SafeLen = UIntPlusOne<Self>;
    #[cfg(target_pointer_width = "64")]
    type SafeLen = UIntPlusOne<Self>;

    #[inline]
    fn checked_add_one(self) -> Option<Self> {
        self.checked_add(1)
    }

    #[inline]
    fn add_one(self) -> Self {
        self + 1
    }

    #[inline]
    fn sub_one(self) -> Self {
        self - 1
    }

    #[inline]
    fn assign_sub_one(&mut self) {
        *self -= 1;
    }

    #[inline]
    fn range_next(range: &mut RangeInclusive<Self>) -> Option<Self> {
        range.next()
    }

    #[inline]
    fn range_next_back(range: &mut RangeInclusive<Self>) -> Option<Self> {
        range.next_back()
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
    fn from_slice(slice: impl AsRef<[Self]>) -> crate::RangeSetBlaze<Self> {
        slice.as_ref().iter().collect()
    }

    fn safe_len(r: &RangeInclusive<Self>) -> <Self as Integer>::SafeLen {
        debug_assert!(r.start() <= r.end());
        UIntPlusOne::UInt(r.end() - r.start()) + UIntPlusOne::UInt(1)
    }

    #[allow(clippy::cast_precision_loss)]
    fn safe_len_to_f64(len: Self::SafeLen) -> f64 {
        match len {
            UIntPlusOne::UInt(len) => len as f64,
            UIntPlusOne::MaxPlusOne => UIntPlusOne::<Self>::max_plus_one_as_f64(),
        }
    }

    #[allow(clippy::cast_sign_loss)]
    #[allow(clippy::cast_possible_truncation)]
    fn f64_to_safe_len(f: f64) -> Self::SafeLen {
        if f >= UIntPlusOne::<Self>::max_plus_one_as_f64() {
            UIntPlusOne::MaxPlusOne
        } else {
            UIntPlusOne::UInt(f as Self)
        }
    }

    fn add_len_less_one(self, b: Self::SafeLen) -> Self {
        // a + (b - 1) as Self
        match b {
            UIntPlusOne::UInt(b) => {
                debug_assert!(b > 0);
                self + (b - 1)
            }
            UIntPlusOne::MaxPlusOne => self + Self::MAX,
        }
    }
    fn sub_len_less_one(self, b: Self::SafeLen) -> Self {
        // a - (b - 1) as Self
        match b {
            UIntPlusOne::UInt(v) => {
                debug_assert!(v > 0);
                self - (v - 1)
            }
            UIntPlusOne::MaxPlusOne => self - Self::MAX,
        }
    }
}

impl Integer for isize {
    #[cfg(target_pointer_width = "32")]
    type SafeLen = u64;
    #[cfg(target_pointer_width = "64")]
    type SafeLen = u128;

    impl_integer_ops!(isize, usize);
}

impl Integer for usize {
    #[cfg(target_pointer_width = "32")]
    type SafeLen = u64;
    #[cfg(target_pointer_width = "64")]
    type SafeLen = u128;

    impl_integer_ops!(usize, Self);
}

impl Integer for i16 {
    #[cfg(target_pointer_width = "32")]
    type SafeLen = usize;
    #[cfg(target_pointer_width = "64")]
    type SafeLen = usize;

    impl_integer_ops!(i16, u16);
}

impl Integer for u16 {
    #[cfg(target_pointer_width = "32")]
    type SafeLen = usize;
    #[cfg(target_pointer_width = "64")]
    type SafeLen = usize;

    impl_integer_ops!(u16, Self);
}

impl Integer for Ipv4Addr {
    #[cfg(target_pointer_width = "32")]
    type SafeLen = u64;
    #[cfg(target_pointer_width = "64")]
    type SafeLen = usize;

    #[inline]
    fn checked_add_one(self) -> Option<Self> {
        let num = u32::from(self);
        num.checked_add(1).map(Self::from)
    }

    #[inline]
    fn add_one(self) -> Self {
        let num = u32::from(self);
        Self::from(num + 1)
    }

    #[inline]
    fn sub_one(self) -> Self {
        let num = u32::from(self);
        Self::from(num - 1)
    }

    #[inline]
    fn assign_sub_one(&mut self) {
        let num = u32::from(*self);
        *self = Self::from(num - 1);
    }

    #[inline]
    fn range_next(range: &mut RangeInclusive<Self>) -> Option<Self> {
        range.next()
    }

    #[inline]
    fn range_next_back(range: &mut RangeInclusive<Self>) -> Option<Self> {
        range.next_back()
    }

    #[inline]
    fn min_value() -> Self {
        Self::new(0, 0, 0, 0)
    }

    #[inline]
    fn max_value() -> Self {
        Self::new(255, 255, 255, 255)
    }

    #[cfg(feature = "from_slice")]
    #[inline]
    fn from_slice(slice: impl AsRef<[Self]>) -> RangeSetBlaze<Self> {
        slice.as_ref().iter().collect()
    }

    fn safe_len(r: &RangeInclusive<Self>) -> <Self as Integer>::SafeLen {
        let start_num = u32::from(*r.start());
        let end_num = u32::from(*r.end());
        end_num.overflowing_sub(start_num).0 as <Self as Integer>::SafeLen + 1
    }

    #[allow(clippy::cast_precision_loss)]
    fn safe_len_to_f64(len: Self::SafeLen) -> f64 {
        len as f64
    }

    #[allow(clippy::cast_possible_truncation)]
    #[allow(clippy::cast_sign_loss)]
    fn f64_to_safe_len(f: f64) -> Self::SafeLen {
        f as Self::SafeLen
    }

    #[allow(clippy::cast_possible_truncation)]
    fn add_len_less_one(self, b: Self::SafeLen) -> Self {
        debug_assert!(b > 0);
        Self::from(u32::from(self) + (b - 1) as u32)
    }

    #[allow(clippy::cast_possible_truncation)]
    fn sub_len_less_one(self, b: Self::SafeLen) -> Self {
        Self::from(u32::from(self) - (b + 1) as u32)
    }
}

impl Integer for Ipv6Addr {
    #[cfg(target_pointer_width = "32")]
    type SafeLen = UIntPlusOne<u128>;
    #[cfg(target_pointer_width = "64")]
    type SafeLen = UIntPlusOne<u128>;

    #[inline]
    fn checked_add_one(self) -> Option<Self> {
        let num = u128::from(self);
        num.checked_add(1).map(Self::from)
    }

    #[inline]
    fn add_one(self) -> Self {
        let num = u128::from(self);
        Self::from(num + 1)
    }

    #[inline]
    fn sub_one(self) -> Self {
        let num = u128::from(self);
        Self::from(num - 1)
    }

    #[inline]
    fn assign_sub_one(&mut self) {
        let num = u128::from(*self);
        *self = Self::from(num - 1);
    }

    #[inline]
    fn range_next(range: &mut RangeInclusive<Self>) -> Option<Self> {
        range.next()
    }

    #[inline]
    fn range_next_back(range: &mut RangeInclusive<Self>) -> Option<Self> {
        range.next_back()
    }

    #[inline]
    fn min_value() -> Self {
        Self::new(0, 0, 0, 0, 0, 0, 0, 0)
    }

    #[inline]
    fn max_value() -> Self {
        Self::from(u128::MAX)
    }

    #[cfg(feature = "from_slice")]
    #[inline]
    fn from_slice(slice: impl AsRef<[Self]>) -> RangeSetBlaze<Self> {
        slice.as_ref().iter().collect()
    }

    fn safe_len(r: &RangeInclusive<Self>) -> <Self as Integer>::SafeLen {
        let start_num = u128::from(*r.start());
        let end_num = u128::from(*r.end());

        debug_assert!(start_num <= end_num);
        UIntPlusOne::UInt(end_num - start_num) + UIntPlusOne::UInt(1)
    }

    #[allow(clippy::cast_precision_loss)]
    fn safe_len_to_f64(len: Self::SafeLen) -> f64 {
        match len {
            UIntPlusOne::UInt(len) => len as f64,
            UIntPlusOne::MaxPlusOne => UIntPlusOne::<u128>::max_plus_one_as_f64(),
        }
    }

    #[allow(clippy::cast_possible_truncation)]
    #[allow(clippy::cast_sign_loss)]
    fn f64_to_safe_len(f: f64) -> Self::SafeLen {
        if f >= UIntPlusOne::<u128>::max_plus_one_as_f64() {
            UIntPlusOne::MaxPlusOne
        } else {
            UIntPlusOne::UInt(f as u128)
        }
    }
    fn add_len_less_one(self, b: Self::SafeLen) -> Self {
        let UIntPlusOne::UInt(b) = b else {
            debug_assert!(false, "Too large to add to Ipv6Addr");
            return Self::from(u128::MAX);
        };
        debug_assert!(b > 0);
        Self::from(u128::from(self) + (b - 1))
    }
    fn sub_len_less_one(self, b: Self::SafeLen) -> Self {
        match b {
            UIntPlusOne::UInt(v) => {
                debug_assert!(v > 0);
                Self::from(u128::from(self) - (v - 1))
            }
            UIntPlusOne::MaxPlusOne => Self::from(u128::from(self) - u128::MAX),
        }
    }
}

// all inclusive
const SURROGATE_START: u32 = 0xD800;
const SURROGATE_END: u32 = 0xDFFF;

impl Integer for char {
    #[cfg(target_pointer_width = "32")]
    // in general, the length of a 32-bit inclusive range does not fit in a u32,
    // but unicode doesn't use the full range, so it does fit
    type SafeLen = usize;
    #[cfg(target_pointer_width = "64")]
    type SafeLen = usize;

    #[inline]
    fn checked_add_one(self) -> Option<Self> {
        // Can't overflow u64 because of the range of char
        let mut num = u32::from(self) + 1;
        // skip over the surrogate range
        if num == SURROGATE_START {
            num = SURROGATE_END + 1;
        }
        // Will report char overflow as None
        Self::from_u32(num)
    }

    #[inline]
    fn add_one(self) -> Self {
        self.checked_add_one().map_or_else(
            || {
                #[cfg(debug_assertions)]
                panic!("char overflow");
                #[cfg(not(debug_assertions))]
                Self::max_value()
            },
            |c| c,
        )
    }

    #[inline]
    fn sub_one(self) -> Self {
        let mut num = u32::from(self) - 1; // by design, debug will panic if underflow
                                           // skip over the surrogate range
        if num == SURROGATE_END {
            num = SURROGATE_START - 1;
        }
        // can never underflow here because of the range of char
        Self::from_u32(num).unwrap()
    }

    #[inline]
    fn assign_sub_one(&mut self) {
        *self = self.sub_one();
    }

    #[inline]
    fn range_next(range: &mut RangeInclusive<Self>) -> Option<Self> {
        range.next()
    }

    #[inline]
    fn range_next_back(range: &mut RangeInclusive<Self>) -> Option<Self> {
        range.next_back()
    }

    #[inline]
    fn min_value() -> Self {
        '\u{0}'
    }

    #[inline]
    fn max_value() -> Self {
        '\u{10FFFF}'
    }

    #[cfg(feature = "from_slice")]
    #[inline]
    fn from_slice(slice: impl AsRef<[Self]>) -> RangeSetBlaze<Self> {
        slice.as_ref().iter().collect()
    }

    fn safe_len(r: &RangeInclusive<Self>) -> <Self as Integer>::SafeLen {
        // assume valid, non-empty range
        let start_num = u32::from(*r.start());
        let end_num = u32::from(*r.end());
        let mut len = (end_num - start_num) as <Self as Integer>::SafeLen + 1;
        if start_num < SURROGATE_START && SURROGATE_END < end_num {
            len -= (SURROGATE_END - SURROGATE_START + 1) as <Self as Integer>::SafeLen;
        }
        len
    }

    #[allow(clippy::cast_precision_loss)]
    fn safe_len_to_f64(len: Self::SafeLen) -> f64 {
        len as f64
    }

    #[allow(clippy::cast_possible_truncation)]
    #[allow(clippy::cast_sign_loss)]
    fn f64_to_safe_len(f: f64) -> Self::SafeLen {
        f as Self::SafeLen
    }

    #[allow(clippy::cast_possible_truncation)]
    fn add_len_less_one(self, b: Self::SafeLen) -> Self {
        let a = u32::from(self);
        debug_assert!(b > 0);
        let mut num = a + (b - 1) as u32;
        // skip over the surrogate range
        if a < SURROGATE_START && SURROGATE_START <= num {
            num += SURROGATE_END - SURROGATE_START + 1;
        }

        Self::from_u32(num).map_or_else(
            || {
                #[cfg(debug_assertions)]
                panic!("char overflow");
                #[cfg(not(debug_assertions))]
                Self::max_value()
            },
            |c| c,
        )
    }

    #[allow(clippy::cast_possible_truncation)]
    fn sub_len_less_one(self, b: Self::SafeLen) -> Self {
        let a = u32::from(self);
        let mut num = a - (b - 1) as u32;
        // skip over the surrogate range
        if num <= SURROGATE_END && SURROGATE_END < a {
            num -= SURROGATE_END - SURROGATE_START + 1;
        }

        Self::from_u32(num).map_or_else(
            || {
                #[cfg(debug_assertions)]
                panic!("char underflow");
                #[cfg(not(debug_assertions))]
                Self::min_value()
            },
            |c| c,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::prelude::*;

    #[test]
    fn test_unicode() {
        // cmk define 'universe'
        let universe = !RangeSetBlaze::<char>::default();
        assert_eq!('\u{10FFFF}'.checked_add_one(), None);
        // cmk test that add_one throws exception
        // cmk test that sub_one throws exception
        let mut prev = None;
        let mut len = 0;
        for c in '\u{0}'..='\u{10FFFF}' {
            let len2b = char::safe_len(&(c..='\u{10FFFF}'));
            assert_eq!(len2b, universe.len() - len);
            let c2 = '\u{10FFFF}'.sub_len_less_one(len2b);
            assert_eq!(c2, c);
            let c3 = c2.add_len_less_one(len2b);
            assert_eq!(c3, '\u{10FFFF}');
            len += 1;
            let len2 = char::safe_len(&('\u{0}'..=c));
            assert_eq!(len, len2);
            assert_eq!(len2, char::f64_to_safe_len(char::safe_len_to_f64(len2)));
            let c2 = '\u{0}'.add_len_less_one(len);
            assert_eq!(c2, c);
            let c3 = c.sub_len_less_one(len);

            assert_eq!(c3, '\u{0}');
            if let Some(prev) = prev {
                assert!(universe.contains(prev));
                assert!(universe.contains(c));
                assert!(universe.is_superset(&RangeSetBlaze::from_iter([prev..=c])));

                assert_eq!(prev.checked_add_one(), Some(c));
                assert_eq!(prev.add_one(), c);

                assert_eq!(c.sub_one(), prev);
                let mut c2 = c;
                c2.assign_sub_one();
                assert_eq!(c2, prev);
            }

            prev = Some(c);
        }
        assert_eq!(universe.len(), len);
        // cmk need more methods for coverage
    }

    // should have similar tests for ip4 and ip6
}
