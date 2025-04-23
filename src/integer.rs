use crate::UIntPlusOne;
#[cfg(feature = "from_slice")]
use crate::{RangeSetBlaze, from_slice::FromSliceIter};
use core::hash::Hash;
use core::net::{Ipv4Addr, Ipv6Addr};
use core::ops::{AddAssign, SubAssign};
use core::panic;
use core::{fmt, ops::RangeInclusive};
use num_traits::ops::overflowing::OverflowingSub;

#[cfg(feature = "from_slice")]
#[allow(clippy::redundant_pub_crate)]
pub(crate) const LANES: usize = 16;

/// Represents elements that can be used within [`RangeSetBlaze`] and as keys in [`RangeMapBlaze`].
///
/// This includes integer types from `u8` to `u128` (including `usize`), `i8` to `i128` (including `isize`),
/// as well as `char`, `Ipv4Addr`, and `Ipv6Addr`.
///
/// [`RangeSetBlaze`]: crate::RangeSetBlaze
/// [`RangeMapBlaze`]: crate::RangeMapBlaze
pub trait Integer: Copy + PartialEq + PartialOrd + Ord + fmt::Debug + Send + Sync {
    /// Attempts to add one to the current value, returning `None` if the operation would overflow.
    fn checked_add_one(self) -> Option<Self>;

    /// Adds one to the current value, panicking in debug mode if the operation overflows.
    ///
    /// # Examples
    /// ```
    /// use range_set_blaze::Integer;
    ///
    /// assert_eq!(5u8.add_one(), 6);
    /// ```
    #[must_use]
    fn add_one(self) -> Self;

    /// Subtracts one from the current value, panicking in debug mode if the operation underflows.
    ///
    /// # Examples
    /// ```
    /// use range_set_blaze::Integer;
    ///
    /// assert_eq!(5u8.sub_one(), 4);
    /// ```
    #[must_use]
    fn sub_one(self) -> Self;

    /// Subtracts one from the current value and assigns it back to `self`.
    fn assign_sub_one(&mut self);

    /// Returns an exhausted range, which is a range that starts from the maximum value and ends at the minimum value.
    /// This results in an empty range.
    ///
    /// # Examples
    /// ```
    /// use range_set_blaze::Integer;
    ///
    /// let range = u8::exhausted_range();
    /// assert!(range.is_empty());
    /// ```
    #[must_use]
    fn exhausted_range() -> RangeInclusive<Self> {
        Self::max_value()..=Self::min_value()
    }

    /// Advances the iterator for the given range by one step, returning the next value or `None` if the range is exhausted.
    ///
    /// This method needs to be defined on each type of interest because the `core::Step` trait is not stable yet.
    fn range_next(range: &mut RangeInclusive<Self>) -> Option<Self>;

    /// Advances the iterator for the given range in reverse by one step, returning the previous value or `None` if the range is exhausted.
    ///
    /// This method needs to be defined on each type of interest because the `core::Step` trait is not stable yet.
    fn range_next_back(range: &mut RangeInclusive<Self>) -> Option<Self>;

    /// Returns the minimum value that can be represented by the type.
    ///
    /// # Examples
    /// ```
    /// use range_set_blaze::Integer;
    ///
    /// assert_eq!(u8::min_value(), 0);
    /// ```
    #[must_use]
    fn min_value() -> Self;

    /// Returns the maximum value that can be represented by the type.
    ///
    /// # Examples
    /// ```
    /// use range_set_blaze::Integer;
    ///
    /// assert_eq!(u8::max_value(), 255);
    /// ```
    #[must_use]
    fn max_value() -> Self;

    #[cfg(feature = "from_slice")]
    /// Creates a [`RangeSetBlaze`] from a slice, specific to the integer type.
    ///
    /// [`RangeSetBlaze`]: crate::RangeSetBlaze
    fn from_slice(slice: impl AsRef<[Self]>) -> RangeSetBlaze<Self>;

    /// The type representing the safe length for a [`RangeSetBlaze`]. For example, the length of a `RangeSetBlaze<u8>` is `u16` to handle ranges up to 256 elements.
    /// For larger types like `u128`, this is represented by a custom type `UIntPlusOne<u128>`.
    ///
    /// [`RangeSetBlaze`]: crate::RangeSetBlaze
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

    /// Calculates the length of a range without overflow.
    ///
    /// # Examples
    /// ```
    /// use range_set_blaze::Integer;
    ///
    /// assert_eq!(<u8 as Integer>::safe_len(&(0..=255)), 256);
    /// ```
    fn safe_len(range: &RangeInclusive<Self>) -> <Self as Integer>::SafeLen;

    // FUTURE define .len() SortedDisjoint

    /// Converts a `f64` to [`Integer::SafeLen`] using the formula `f as Self::SafeLen`. For large integer types, this will result in a loss of precision.
    fn f64_to_safe_len_lossy(f: f64) -> Self::SafeLen;

    /// Converts [`Integer::SafeLen`] to `f64`, potentially losing precision for large values.
    fn safe_len_to_f64_lossy(len: Self::SafeLen) -> f64;

    /// Computes `self + (b - 1)` where `b` is of type [`Integer::SafeLen`].
    #[must_use]
    fn inclusive_end_from_start(self, b: Self::SafeLen) -> Self;

    /// Computes `self - (b - 1)` where `b` is of type [`Integer::SafeLen`].
    #[must_use]
    fn start_from_inclusive_end(self, b: Self::SafeLen) -> Self;
}

/// Macro to implement the `Integer` trait for specific integer types.
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
            // 1️⃣ Contract: caller promises start ≤ end  (checked only in debug builds)
            debug_assert!(r.start() <= r.end(), "start ≤ end required");

            // 2️⃣ Compute distance in `Self` then reinterpret‑cast to the first
            //     widening type `$type2` (loss‑free under the invariant above).
            let delta_wide: $type2 = r.end().overflowing_sub(r.start()).0 as $type2;

            // 3️⃣ Final widening to `SafeLen`.
            //    `try_from` is infallible here, so LLVM removes the check in release.
            <<Self as Integer>::SafeLen>::try_from(delta_wide)
                .expect("$type2 ⊆ SafeLen; optimizer drops this in release")
                + 1 // 4️⃣ Inclusive length = distance + 1
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
                let max_len = Self::safe_len(&(self..=Self::max_value()));
                assert!(
                    b > 0 && b <= max_len,
                    "b must be in range 1..=max_len (b = {b}, max_len = {max_len})"
                );
            }
            // If b is in range, two’s-complement wrap-around yields the correct inclusive end even if the add overflows
            self.wrapping_add((b - 1) as Self)
        }

        #[allow(clippy::cast_possible_truncation)]
        #[allow(clippy::cast_possible_wrap)]
        fn start_from_inclusive_end(self, b: Self::SafeLen) -> Self {
            #[cfg(debug_assertions)]
            {
                let max_len = Self::safe_len(&(Self::min_value()..=self));
                assert!(
                    b > 0 && b <= max_len,
                    "b must be in range 1..=max_len (b = {b}, max_len = {max_len})"
                );
            }
            // If b is in range, two’s-complement wrap-around yields the correct start even if the sub overflows
            self.wrapping_sub((b - 1) as Self)
        }
    };
}

impl Integer for i8 {
    type SafeLen = u16;
    impl_integer_ops!(i8, u8);
}

impl Integer for u8 {
    type SafeLen = u16;
    impl_integer_ops!(u8, Self);
}

impl Integer for i32 {
    type SafeLen = u64;

    impl_integer_ops!(i32, u32);
}

impl Integer for u32 {
    type SafeLen = u64;

    impl_integer_ops!(u32, Self);
}

impl Integer for i64 {
    type SafeLen = u128;

    impl_integer_ops!(i64, u64);
}

impl Integer for u64 {
    type SafeLen = u128;

    impl_integer_ops!(u64, Self);
}

impl Integer for i128 {
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
    fn from_slice(slice: impl AsRef<[Self]>) -> RangeSetBlaze<Self> {
        RangeSetBlaze::from_iter(slice.as_ref())
    }

    #[allow(clippy::cast_sign_loss)]
    fn safe_len(r: &RangeInclusive<Self>) -> <Self as Integer>::SafeLen {
        debug_assert!(r.start() <= r.end());
        // Signed sub may overflow, but casting preserves correct unsigned distance
        let less1 = r.end().overflowing_sub(r.start()).0 as u128;
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

    /// Adds `b - 1` to `self`.
    ///
    /// # Panics
    /// It is an error to call this method with `b` equal to 0 or a value that is too large to add to `i128`.
    /// This will always trigger a panic in debug mode; in release mode, the behavior is undefined.
    fn inclusive_end_from_start(self, b: Self::SafeLen) -> Self {
        let UIntPlusOne::UInt(b) = b else {
            if self == Self::MIN {
                return Self::MAX;
            }
            panic!("Too large to add to i128");
        };
        debug_assert!(b > 0);
        // cmk00000 need to check for overflow here when in debug mode
        self + (b - 1) as Self
    }

    /// Subtract `b - 1` from `self`.
    ///
    /// # Panics
    /// It is an error to call this method with `b` equal to 0 or a value that is too large to subtract from `i128`.
    /// This will always trigger a panic in debug mode; in release mode, the behavior is undefined.
    #[allow(clippy::cast_possible_wrap)]
    fn start_from_inclusive_end(self, b: Self::SafeLen) -> Self {
        let UIntPlusOne::UInt(b) = b else {
            if self == Self::MAX {
                return Self::MIN;
            }
            panic!("Too large to subtract from i128");
        };
        debug_assert!(b > 0);
        self - (b - 1) as Self
    }
}

impl Integer for u128 {
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
    fn from_slice(slice: impl AsRef<[Self]>) -> RangeSetBlaze<Self> {
        RangeSetBlaze::from_iter(slice.as_ref())
    }

    fn safe_len(r: &RangeInclusive<Self>) -> <Self as Integer>::SafeLen {
        debug_assert!(r.start() <= r.end());
        UIntPlusOne::UInt(r.end() - r.start()) + UIntPlusOne::UInt(1)
    }

    #[allow(clippy::cast_precision_loss)]
    fn safe_len_to_f64_lossy(len: Self::SafeLen) -> f64 {
        match len {
            UIntPlusOne::UInt(len) => len as f64,
            UIntPlusOne::MaxPlusOne => UIntPlusOne::<Self>::max_plus_one_as_f64(),
        }
    }

    #[allow(clippy::cast_sign_loss)]
    #[allow(clippy::cast_possible_truncation)]
    fn f64_to_safe_len_lossy(f: f64) -> Self::SafeLen {
        if f >= UIntPlusOne::<Self>::max_plus_one_as_f64() {
            UIntPlusOne::MaxPlusOne
        } else {
            UIntPlusOne::UInt(f as Self)
        }
    }

    /// Adds `b - 1` to `self`.
    ///
    /// # Panics
    /// It is an error to call this method with `b` equal to 0 or a value that is too large to add to `u128`.
    /// This will always trigger a panic in debug mode; in release mode, the behavior is undefined.
    #[allow(clippy::cast_possible_wrap)]
    fn inclusive_end_from_start(self, b: Self::SafeLen) -> Self {
        let UIntPlusOne::UInt(b) = b else {
            if self == Self::MIN {
                return Self::MAX;
            }
            panic!("Too large to add to u128");
        };
        debug_assert!(b > 0);
        // cmk00000 need to check for overflow here when in debug mode
        self + (b - 1) as Self
    }

    /// Subtract `b - 1` from `self`.
    ///
    /// # Panics
    /// It is an error to call this method with `b` equal to 0 or a value that is too large to subtract from `u128`.
    /// This will always trigger a panic in debug mode; in release mode, the behavior is undefined.
    #[allow(clippy::cast_possible_wrap)]
    fn start_from_inclusive_end(self, b: Self::SafeLen) -> Self {
        let UIntPlusOne::UInt(b) = b else {
            if self == Self::MAX {
                return Self::MIN;
            }
            panic!("Too large to subtract from u128");
        };
        debug_assert!(b > 0);
        self - (b - 1) as Self
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
    type SafeLen = u32;

    impl_integer_ops!(i16, u16);
}

impl Integer for u16 {
    type SafeLen = u32;

    impl_integer_ops!(u16, Self);
}

impl Integer for Ipv4Addr {
    type SafeLen = u64;

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
        RangeSetBlaze::from_iter(slice.as_ref())
    }

    #[allow(clippy::cast_lossless)]
    fn safe_len(r: &RangeInclusive<Self>) -> <Self as Integer>::SafeLen {
        let start_num = u32::from(*r.start());
        let end_num = u32::from(*r.end());
        debug_assert!(start_num <= end_num);
        // Signed sub may overflow, but casting preserves correct unsigned distance
        end_num.overflowing_sub(start_num).0 as <Self as Integer>::SafeLen + 1
    }

    #[allow(clippy::cast_precision_loss)]
    fn safe_len_to_f64_lossy(len: Self::SafeLen) -> f64 {
        len as f64
    }

    #[allow(clippy::cast_possible_truncation)]
    #[allow(clippy::cast_sign_loss)]
    fn f64_to_safe_len_lossy(f: f64) -> Self::SafeLen {
        f as Self::SafeLen
    }

    #[allow(clippy::cast_possible_truncation)]
    fn inclusive_end_from_start(self, b: Self::SafeLen) -> Self {
        debug_assert!(b > 0);
        // cmk00000 need to check for overflow here when in debug mode
        Self::from(u32::from(self) + (b - 1) as u32)
    }

    #[allow(clippy::cast_possible_truncation)]
    fn start_from_inclusive_end(self, b: Self::SafeLen) -> Self {
        Self::from(u32::from(self) - (b - 1) as u32)
    }
}

impl Integer for Ipv6Addr {
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
        RangeSetBlaze::from_iter(slice.as_ref())
    }

    fn safe_len(r: &RangeInclusive<Self>) -> <Self as Integer>::SafeLen {
        let start_num = u128::from(*r.start());
        let end_num = u128::from(*r.end());

        debug_assert!(start_num <= end_num);
        UIntPlusOne::UInt(end_num - start_num) + UIntPlusOne::UInt(1)
    }

    #[allow(clippy::cast_precision_loss)]
    fn safe_len_to_f64_lossy(len: Self::SafeLen) -> f64 {
        match len {
            UIntPlusOne::UInt(len) => len as f64,
            UIntPlusOne::MaxPlusOne => UIntPlusOne::<u128>::max_plus_one_as_f64(),
        }
    }

    #[allow(clippy::cast_possible_truncation)]
    #[allow(clippy::cast_sign_loss)]
    fn f64_to_safe_len_lossy(f: f64) -> Self::SafeLen {
        if f >= UIntPlusOne::<u128>::max_plus_one_as_f64() {
            UIntPlusOne::MaxPlusOne
        } else {
            UIntPlusOne::UInt(f as u128)
        }
    }

    /// Adds `b - 1` to `self`.
    ///
    /// # Panics
    /// It is an error to call this method with `b` equal to 0 or a value that is too large to add to `Ipv6Addr`.
    /// This will always trigger a panic in debug mode; in release mode, the behavior is undefined.
    fn inclusive_end_from_start(self, b: Self::SafeLen) -> Self {
        let UIntPlusOne::UInt(b) = b else {
            if self == Self::min_value() {
                return Self::max_value();
            }
            panic!("Too large to add to Ipv6Addr");
        };
        // cmk00000 need to check for overflow here when in debug mode
        debug_assert!(b > 0);
        Self::from(u128::from(self) + (b - 1))
    }

    /// Subtract `b - 1` from `self`.
    ///
    /// # Panics
    /// It is an error to call this method with `b` equal to 0 or a value that is too large to subtract from `Ipv6Addr`.
    /// This will always trigger a panic in debug mode; in release mode, the behavior is undefined.
    fn start_from_inclusive_end(self, b: Self::SafeLen) -> Self {
        let UIntPlusOne::UInt(b) = b else {
            if self == Self::max_value() {
                return Self::min_value();
            }
            panic!("Too large to subtract from Ipv6Addr");
        };
        debug_assert!(b > 0);
        Self::from(u128::from(self) - (b - 1))
    }
}

// all inclusive
const SURROGATE_START: u32 = 0xD800;
const SURROGATE_END: u32 = 0xDFFF;

impl Integer for char {
    // in general, the length of a 32-bit inclusive range does not fit in a u32,
    // but unicode doesn't use the full range, so it does fit
    type SafeLen = u32;

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
                panic!("char overflow"); // Panics in both debug and release modes
            },
            |next| next,
        )
    }

    #[inline]
    fn sub_one(self) -> Self {
        let mut num = u32::from(self) - 1; // by design, debug will panic if underflow
        // skip over the surrogate range
        if num == SURROGATE_END {
            num = SURROGATE_START - 1;
        }
        Self::from_u32(num)
            .expect("Real Assert: Can never underflow here because of the range of char.")
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
        RangeSetBlaze::from_iter(slice.as_ref())
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

    #[allow(clippy::cast_precision_loss, clippy::cast_lossless)]
    fn safe_len_to_f64_lossy(len: Self::SafeLen) -> f64 {
        len as f64
    }

    #[allow(clippy::cast_possible_truncation)]
    #[allow(clippy::cast_sign_loss)]
    fn f64_to_safe_len_lossy(f: f64) -> Self::SafeLen {
        f as Self::SafeLen
    }

    fn inclusive_end_from_start(self, b: Self::SafeLen) -> Self {
        if let Some(b_less_one) = b.checked_sub(1) {
            let a = u32::from(self);
            if let Some(mut num) = a.checked_add(b_less_one) {
                // skip over the surrogate range
                if a < SURROGATE_START && SURROGATE_START <= num {
                    num += SURROGATE_END - SURROGATE_START + 1;
                }
                if let Some(c) = Self::from_u32(num) {
                    return c;
                }
            }
        }
        // cmk00000 need to check for overflow here when in debug mode
        panic!("char overflow");
    }

    fn start_from_inclusive_end(self, b: Self::SafeLen) -> Self {
        if let Some(b_less_one) = b.checked_sub(1) {
            let a = u32::from(self);
            if let Some(mut num) = a.checked_sub(b_less_one) {
                // Skip over the surrogate range
                if num <= SURROGATE_END && SURROGATE_END < a {
                    num -= SURROGATE_END - SURROGATE_START + 1;
                }
                return Self::from_u32(num).expect(
                    "Real Assert: Impossible for this to fail because char goes down to 0",
                );
            }
        }
        panic!("char underflow");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::prelude::*;
    use num_traits::{One, Zero};
    use syntactic_for::syntactic_for;

    use wasm_bindgen_test::*;
    wasm_bindgen_test_configure!(run_in_browser);

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    fn coverage_integer() {
        let mut a = 0u8..=0u8;
        assert_eq!(u8::range_next_back(&mut a), Some(0));
        assert_eq!(u8::range_next(&mut a), None);

        let mut b = 0i128..=0i128;
        assert_eq!(i128::range_next_back(&mut b), Some(0));
        assert_eq!(i128::range_next(&mut b), None);

        let mut b = 0i128;
        i128::assign_sub_one(&mut b);
        assert_eq!(b, -1);

        // convert  UIntPlusOne::MaxPlusOne to f64 and back
        let f = i128::safe_len_to_f64_lossy(UIntPlusOne::MaxPlusOne);
        let i = i128::f64_to_safe_len_lossy(f);
        assert_eq!(i, UIntPlusOne::MaxPlusOne);

        let mut b = 0u128..=0u128;
        assert_eq!(u128::range_next_back(&mut b), Some(0));
        assert_eq!(u128::range_next(&mut b), None);

        let mut b = 1u128;
        u128::assign_sub_one(&mut b);
        assert_eq!(b, 0);

        // convert  UIntPlusOne::MaxPlusOne to f64 and back
        let f = u128::safe_len_to_f64_lossy(UIntPlusOne::MaxPlusOne);
        let i = u128::f64_to_safe_len_lossy(f);
        assert_eq!(i, UIntPlusOne::MaxPlusOne);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    #[should_panic(expected = "1")]
    #[cfg(debug_assertions)] // Only run this test in debug mode
    fn test_add_len_less_one_with_max_plus_one() {
        let value: i128 = 100;
        let len = UIntPlusOne::MaxPlusOne;
        let _ = value.inclusive_end_from_start(len); // This should panic in debug mode
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    #[should_panic(expected = "2")]
    #[cfg(debug_assertions)] // Only run this test in debug mode
    fn test_sub_len_less_one_with_max_plus_one() {
        let value: i128 = 100;
        let len = UIntPlusOne::MaxPlusOne;
        let _ = value.start_from_inclusive_end(len); // This should panic in debug mode
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    #[allow(clippy::cognitive_complexity, clippy::legacy_numeric_constants)]
    fn test_ip4_and_ip6_etc() {
        syntactic_for! { ty in [char, Ipv6Addr, u128, i128, Ipv4Addr] {
            $(
            // Test the minimum value for the type
            let a = <$ty>::min_value();
            let b = a.checked_add_one();
            assert_eq!(b, Some(<$ty>::min_value().add_one()));

            // Show overflow behavior
            let a = <$ty>::max_value();
            let b = a.checked_add_one();
            assert_eq!(b, None);

            let a = <$ty>::min_value();
            let mut b = a.add_one();
            assert_eq!(b, <$ty>::min_value().add_one());

            let c = b.sub_one();
            assert_eq!(c, a);

            b.assign_sub_one();
            assert_eq!(b, a);

            let mut a = <$ty>::min_value()..=<$ty>::min_value();
            let b = <$ty>::range_next(&mut a);
            assert_eq!(b, Some(<$ty>::min_value()));
            let b = <$ty>::range_next(&mut a);
            assert_eq!(b, None);

            let mut a = <$ty>::min_value()..=<$ty>::max_value();
            let b = <$ty>::range_next_back(&mut a);
            assert_eq!(b, Some(<$ty>::max_value()));

            assert_eq!(<$ty>::min_value(), <$ty>::min_value());

            let universe = <$ty>::min_value()..=<$ty>::max_value();
            let len = <$ty>::safe_len(&universe);
            assert_eq!(len, <$ty>::safe_len(&(<$ty>::min_value()..=<$ty>::max_value())));

            let len_via_f64 = <$ty>::f64_to_safe_len_lossy(<$ty>::safe_len_to_f64_lossy(len));
            assert_eq!(len, len_via_f64);

            let short = <$ty>::min_value()..=<$ty>::min_value();
            let len = <$ty>::safe_len(&short);
            let len_via_f64 = <$ty>::f64_to_safe_len_lossy(<$ty>::safe_len_to_f64_lossy(len));
            assert_eq!(len, len_via_f64);

            let len = <$ty>::safe_len(&universe);
            let b = <$ty>::min_value().inclusive_end_from_start(len);
            assert_eq!(b, <$ty>::max_value());

            let c = b.start_from_inclusive_end(len);
            assert_eq!(c, <$ty>::min_value());

            let range = <$ty>::min_value()..=<$ty>::min_value().add_one();
            let len2 = <$ty>::safe_len(&range);
            let b = <$ty>::min_value().inclusive_end_from_start(len2);
            assert_eq!(b, <$ty>::min_value().add_one());

            let b = <$ty>::max_value().start_from_inclusive_end(len2);
            assert_eq!(b, <$ty>::max_value().sub_one());

            #[cfg(feature = "from_slice")]
            {
                let range_set_blaze = <$ty>::from_slice(&[<$ty>::min_value()]);
                assert_eq!(range_set_blaze, RangeSetBlaze::from_iter([<$ty>::min_value()]));
            }
            )*
        }}
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    #[should_panic(expected = "Too large to add to i128")]
    #[allow(clippy::legacy_numeric_constants)]
    fn test_i128_overflow() {
        let value: i128 = i128::max_value();
        let _ = value.inclusive_end_from_start(UIntPlusOne::MaxPlusOne);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    #[should_panic(expected = "Too large to subtract from i128")]
    #[allow(clippy::legacy_numeric_constants)]
    fn test_i128_underflow() {
        let value: i128 = i128::min_value();
        let _ = value.start_from_inclusive_end(UIntPlusOne::MaxPlusOne);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    #[should_panic(expected = "Too large to add to u128")]
    #[allow(clippy::legacy_numeric_constants)]
    fn test_u128_overflow() {
        let value: u128 = u128::max_value();
        let _ = value.inclusive_end_from_start(UIntPlusOne::MaxPlusOne);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    #[should_panic(expected = "Too large to subtract from u128")]
    #[allow(clippy::legacy_numeric_constants)]
    fn test_u128_underflow() {
        let value: u128 = u128::min_value();
        let _ = value.start_from_inclusive_end(UIntPlusOne::MaxPlusOne);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    #[should_panic(expected = "Too large to add to Ipv6Addr")]
    #[allow(clippy::legacy_numeric_constants)]
    fn test_ipv6_overflow() {
        let value: Ipv6Addr = Ipv6Addr::max_value();
        let _ = value.inclusive_end_from_start(UIntPlusOne::MaxPlusOne);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    #[should_panic(expected = "char overflow")]
    #[allow(clippy::legacy_numeric_constants)]
    fn test_char0_overflow() {
        let value: char = char::max_value();
        let _ = value.add_one();
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    #[should_panic(expected = "char overflow")]
    #[allow(clippy::legacy_numeric_constants)]
    fn test_char1_overflow() {
        let value: char = char::max_value();
        let len2 = char::safe_len(&(char::min_value()..=char::min_value().add_one()));
        let _ = value.inclusive_end_from_start(len2);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    #[should_panic(expected = "char underflow")]
    #[allow(clippy::legacy_numeric_constants)]
    fn test_char1_underflow() {
        let value: char = char::min_value();
        let len2 = char::safe_len(&(char::min_value()..=char::min_value().add_one()));
        let _ = value.start_from_inclusive_end(len2);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    #[should_panic(expected = "Too large to subtract from Ipv6Addr")]
    fn test_ipv6_underflow() {
        let value: Ipv6Addr = Ipv6Addr::min_value();
        let _ = value.start_from_inclusive_end(UIntPlusOne::MaxPlusOne);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    #[allow(clippy::cognitive_complexity)]
    fn test_char() {
        // This loops over 1 million characters, but it seems fast enough.
        // Define the universe as the complement of an empty RangeSetBlaze
        let universe = !RangeSetBlaze::<char>::default();

        // Check add_one and sub_one behavior
        let max_value = <char as Integer>::max_value();
        assert_eq!(max_value.checked_add_one(), None);

        let mut prev = None;
        let mut len = <char as Integer>::SafeLen::zero();
        for item in <char as Integer>::min_value()..=max_value {
            let len2b = <char as Integer>::safe_len(&(item..=max_value));
            let mut expected = universe.len();
            expected -= len;
            assert_eq!(len2b, expected);

            let item2 = max_value.start_from_inclusive_end(len2b);
            assert_eq!(item2, item);

            let item3 = item2.inclusive_end_from_start(len2b);
            assert_eq!(item3, max_value);

            len += <char as Integer>::SafeLen::one();
            let len2 = <char as Integer>::safe_len(&(<char as Integer>::min_value()..=item));
            assert_eq!(len, len2);
            assert_eq!(
                len2,
                <char as Integer>::f64_to_safe_len_lossy(<char as Integer>::safe_len_to_f64_lossy(
                    len2
                ))
            );

            let item2 = <char as Integer>::min_value().inclusive_end_from_start(len);
            assert_eq!(item2, item);

            let item3 = item.start_from_inclusive_end(len);
            assert_eq!(item3, <char as Integer>::min_value());

            if let Some(prev) = prev {
                assert!(universe.contains(prev));
                assert!(universe.contains(item));
                assert!(universe.is_superset(&RangeSetBlaze::from_iter([prev..=item])));

                assert_eq!(prev.checked_add_one(), Some(item));
                assert_eq!(prev.add_one(), item);

                assert_eq!(item.sub_one(), prev);
                let mut item2 = item;
                item2.assign_sub_one();
                assert_eq!(item2, prev);
            }

            prev = Some(item);
        }
        assert_eq!(universe.len(), len);

        // Additional checks can be added here if needed for coverage
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    #[should_panic(expected = "char overflow")]
    fn test_add_len_less_one_panic_conditions1() {
        // Case 1: `b.checked_sub(1)` returns `None`
        let character = 'A';
        let b = 0;
        _ = character.inclusive_end_from_start(b); // This should panic due to overflow
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    #[should_panic(expected = "char overflow")]
    fn test_add_len_less_one_panic_conditions2() {
        // Case 2: `self.checked_add(b_less_one)` returns `None`
        let character = char::MAX;
        let b = 3;
        _ = character.inclusive_end_from_start(b); // This should panic due to overflow
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    #[should_panic(expected = "char overflow")]
    fn test_add_len_less_one_panic_conditions3() {
        // Case 3: overflow when adding `b - 1` to `self`
        let character = 'A';
        let b = u32::MAX;
        _ = character.inclusive_end_from_start(b); // This should panic due to overflow
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    #[should_panic(expected = "char underflow")]
    fn test_sub_len_less_one_panic_conditions1() {
        // Case 1: `b.checked_sub(1)` fails, causing an immediate panic.
        let character = 'A';
        let b = 0;
        _ = character.start_from_inclusive_end(b); // This should panic due to underflow
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    #[should_panic(expected = "char underflow")]
    fn test_sub_len_less_one_panic_conditions2() {
        // Case 2: `a.checked_sub(b_less_one)` fails, causing underflow.
        let character = 'A';
        let b = u32::MAX;
        _ = character.start_from_inclusive_end(b); // This should panic due to underflow
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    fn test_use_of_as() {
        syntactic_for! { ty in [char, i8, i16, i32, i64, i128, isize, Ipv4Addr, Ipv6Addr, u8, u16, u32, u64, u128, usize] {
            $(
        let a = <$ty>::min_value();
        let b = <$ty>::max_value();
        let len = <$ty>::safe_len(&(a..=b));
        assert_eq!(<$ty>::inclusive_end_from_start(a, len), b);
        assert_eq!(<$ty>::start_from_inclusive_end(b, len), a);
            )*
        }}
    }

    #[cfg(debug_assertions)]
    #[test]
    #[should_panic(expected = "b must be in range 1..=max_len (b = 0, max_len = 1)")]
    fn test_use_of_as_1() {
        let _ = 127i8.inclusive_end_from_start(0);
    }

    // cmk00000 should we run all (some) tests again in release mode?
    #[cfg(not(debug_assertions))]
    #[test]
    fn test_use_of_as_2() {
        assert_eq!(127i8.inclusive_end_from_start(0), 126);
        assert_eq!(127i8.start_from_inclusive_end(0), -128);
        assert_eq!(127i8.inclusive_end_from_start(2), -128);
        assert_eq!((-126i8).start_from_inclusive_end(4), 127);
    }

    #[cfg(debug_assertions)]
    #[test]
    #[should_panic(expected = "b must be in range 1..=max_len (b = 0, max_len = 256)")]
    fn test_use_of_as_3() {
        let _ = 127i8.start_from_inclusive_end(0);
    }

    #[cfg(debug_assertions)]
    #[test]
    #[should_panic(expected = "b must be in range 1..=max_len (b = 2, max_len = 1)")]
    fn test_use_of_as_4() {
        let _ = 127i8.inclusive_end_from_start(2);
    }

    #[cfg(debug_assertions)]
    #[test]
    #[should_panic(expected = "b must be in range 1..=max_len (b = 4, max_len = 3)")]
    fn test_use_of_as_5() {
        let _ = (-126i8).start_from_inclusive_end(4);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    fn test_use_of_as_6() {
        for a in (-128i8)..=127i8 {
            let b = i8::safe_len(&(a..=127i8));
            assert_eq!(a.inclusive_end_from_start(b), 127i8);
            let b = i8::safe_len(&(i8::MIN..=a));
            assert_eq!(a.start_from_inclusive_end(b), -128i8);
        }
    }
}
