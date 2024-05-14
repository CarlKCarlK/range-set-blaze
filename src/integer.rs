// cmk use crate::u128plus_one::TWO_POW_128;
use crate::UIntPlusOne;
#[cfg(feature = "from_slice")]
use crate::{from_slice::FromSliceIter, RangeSetBlaze};
use core::hash::Hash;
use core::net::Ipv4Addr;
use core::ops::{AddAssign, SubAssign};
use core::{fmt, ops::RangeInclusive};
use num_traits::ops::overflowing::OverflowingSub;

#[cfg(feature = "from_slice")]
const LANES: usize = 16;

/// The element trait of the [`RangeSetBlaze`] and [`SortedDisjoint`], specifically `u8` to `u128` (including `usize`) and `i8` to `i128` (including `isize`).
pub trait Integer:
    Copy
    + PartialEq
    + PartialOrd
    + Ord
    + fmt::Display // cmk0000 make these conditional
    + fmt::Debug // cmk0000 make these conditional
    + Send
    + Sync
{
    /// cmk doc
    fn checked_add_one(self) -> Option<Self>;
    fn add_one(self) -> Self;
    fn sub_one(self) -> Self;
    fn assign_sub_one(&mut self);
    fn min_value2() -> Self;
    fn max_value2() -> Self;

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
    fn add_len_less_one(a: Self, b: Self::SafeLen) -> Self;

    /// Computes `a - (b - 1) as Self`
    fn sub_len_less_one(a: Self, b: Self::SafeLen) -> Self;
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
        fn min_value2() -> Self {
            Self::min_value()
        }

        #[inline]
        fn max_value2() -> Self {
            Self::max_value()
        }

        #[cfg(feature = "from_slice")]
        #[inline]
        fn from_slice(slice: impl AsRef<[$type]>) -> RangeSetBlaze<$type> {
            FromSliceIter::<$type, LANES>::new(slice.as_ref()).collect()
        }

        fn safe_len(r: &RangeInclusive<$type>) -> <$type as Integer>::SafeLen {
            r.end().overflowing_sub(r.start()).0 as $type2 as <$type as Integer>::SafeLen + 1
        }

        fn safe_len_to_f64(len: Self::SafeLen) -> f64 {
            len as f64
        }

        fn f64_to_safe_len(f: f64) -> Self::SafeLen {
            f as Self::SafeLen
        }

        fn add_len_less_one(a: $type, b: Self::SafeLen) -> $type {
            a + (b - 1) as $type
        }

        fn sub_len_less_one(a: $type, b: Self::SafeLen) -> $type {
            a - (b - 1) as $type
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

    impl_integer_ops!(u8, u8);
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

    impl_integer_ops!(u32, u32);
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

    impl_integer_ops!(u64, u64);
}

impl Integer for i128 {
    #[cfg(target_pointer_width = "32")]
    type SafeLen = u128;
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
    fn min_value2() -> Self {
        Self::min_value()
    }

    #[inline]
    fn max_value2() -> Self {
        Self::max_value()
    }

    #[cfg(feature = "from_slice")]
    #[inline]
    fn from_slice(slice: impl AsRef<[Self]>) -> crate::RangeSetBlaze<Self> {
        slice.as_ref().iter().collect()
    }

    fn safe_len(r: &RangeInclusive<Self>) -> <Self as Integer>::SafeLen {
        debug_assert!(r.start() <= r.end());
        let less1 = r.end().overflowing_sub(r.start()).0 as u128;
        if less1 == u128::MAX {
            UIntPlusOne::MaxPlusOne
        } else {
            UIntPlusOne::UInt(less1 + 1)
        }
    }

    fn safe_len_to_f64(len: Self::SafeLen) -> f64 {
        match len {
            UIntPlusOne::UInt(v) => v as f64,
            UIntPlusOne::MaxPlusOne => UIntPlusOne::<u128>::max_plus_one_as_f64(),
        }
    }
    fn f64_to_safe_len(f: f64) -> Self::SafeLen {
        if f >= UIntPlusOne::<u128>::max_plus_one_as_f64() {
            UIntPlusOne::MaxPlusOne
        } else {
            UIntPlusOne::UInt(f as u128)
        }
    }
    fn add_len_less_one(a: Self, b: Self::SafeLen) -> Self {
        let UIntPlusOne::UInt(v) = b else {
            debug_assert!(false, "Too large to add to i128");
            return i128::MAX;
        };
        a + (v - 1) as Self
    }
    fn sub_len_less_one(a: Self, b: Self::SafeLen) -> Self {
        // a - (b - 1) as Self
        let UIntPlusOne::UInt(v) = b else {
            debug_assert!(false, "Too large to subtract from i128");
            return i128::MIN;
        };
        a - (v - 1) as Self
    }
}

impl Integer for u128 {
    #[cfg(target_pointer_width = "32")]
    type SafeLen = u128;
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
    fn min_value2() -> Self {
        Self::min_value()
    }

    #[inline]
    fn max_value2() -> Self {
        Self::max_value()
    }

    #[cfg(feature = "from_slice")]
    #[inline]
    fn from_slice(slice: impl AsRef<[Self]>) -> crate::RangeSetBlaze<Self> {
        slice.as_ref().iter().collect()
    }

    fn safe_len(r: &RangeInclusive<Self>) -> <Self as Integer>::SafeLen {
        debug_assert!(r.start() <= r.end());
        let less1 = r.end().overflowing_sub(r.start()).0 as u128;
        if less1 == u128::MAX {
            UIntPlusOne::MaxPlusOne
        } else {
            UIntPlusOne::UInt(less1 + 1)
        }
    }

    fn safe_len_to_f64(len: Self::SafeLen) -> f64 {
        match len {
            UIntPlusOne::UInt(v) => v as f64,
            UIntPlusOne::MaxPlusOne => UIntPlusOne::<u128>::max_plus_one_as_f64(),
        }
    }
    fn f64_to_safe_len(f: f64) -> Self::SafeLen {
        if f >= UIntPlusOne::<u128>::max_plus_one_as_f64() {
            UIntPlusOne::MaxPlusOne
        } else {
            UIntPlusOne::UInt(f as u128)
        }
    }

    fn add_len_less_one(a: Self, b: Self::SafeLen) -> Self {
        // a + (b - 1) as Self
        match b {
            UIntPlusOne::UInt(v) => {
                debug_assert!(v > 0);
                a + (v - 1)
            }
            UIntPlusOne::MaxPlusOne => a + Self::max_value(),
        }
    }
    fn sub_len_less_one(a: Self, b: Self::SafeLen) -> Self {
        // a - (b - 1) as Self
        match b {
            UIntPlusOne::UInt(v) => {
                debug_assert!(v > 0);
                a - (v - 1)
            }
            UIntPlusOne::MaxPlusOne => a - Self::max_value(),
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

    impl_integer_ops!(usize, usize);
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

    impl_integer_ops!(u16, u16);
}

impl Integer for Ipv4Addr {
    #[cfg(target_pointer_width = "32")]
    type SafeLen = u64;
    #[cfg(target_pointer_width = "64")]
    type SafeLen = usize;

    #[inline]
    fn checked_add_one(self) -> Option<Self> {
        let num = u32::from(self);
        num.checked_add(1).map(Ipv4Addr::from)
    }

    #[inline]
    fn add_one(self) -> Self {
        let num = u32::from(self);
        Ipv4Addr::from(num + 1)
    }

    #[inline]
    fn sub_one(self) -> Self {
        let num = u32::from(self);
        Ipv4Addr::from(num - 1)
    }

    #[inline]
    fn assign_sub_one(&mut self) {
        let num = u32::from(*self);
        *self = Ipv4Addr::from(num - 1);
    }

    #[inline]
    fn min_value2() -> Self {
        Ipv4Addr::new(0, 0, 0, 0)
    }

    #[inline]
    fn max_value2() -> Self {
        Ipv4Addr::new(255, 255, 255, 255)
    }

    #[cfg(feature = "from_slice")]
    #[inline]
    fn from_slice(slice: impl AsRef<[Self]>) -> RangeSetBlaze<Self> {
        slice.as_ref().iter().collect()
    }

    fn safe_len(r: &RangeInclusive<Self>) -> <Self as Integer>::SafeLen {
        let (start, end) = r.clone().into_inner(); // cmk0000 remove clone
        let start_num = u32::from(start);
        let end_num = u32::from(end);

        end_num.overflowing_sub(start_num).0 as <Self as Integer>::SafeLen + 1
    }

    fn safe_len_to_f64(len: Self::SafeLen) -> f64 {
        len as f64
    }
    fn f64_to_safe_len(f: f64) -> Self::SafeLen {
        f as Self::SafeLen
    }
    fn add_len_less_one(a: Self, b: Self::SafeLen) -> Self {
        Ipv4Addr::from(u32::from(a) + b as u32 - 1)
    }
    fn sub_len_less_one(a: Self, b: Self::SafeLen) -> Self {
        Ipv4Addr::from(u32::from(a) - b as u32 + 1)
    }
}
