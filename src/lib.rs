#![cfg_attr(feature = "from_slice", feature(portable_simd))]
// #cmk Rule
#![doc = include_str!("../README.md")]
#![warn(missing_docs)]
#![cfg_attr(not(feature = "std"), no_std)]

// #[cfg(all(feature = "std", feature = "alloc"))]
// compile_error!("feature \"std\" and feature \"alloc\" cannot be enabled at the same time");
// #[cfg(feature = "std")]
// compile_error!("The 'std' feature is active");
// #[cfg(feature = "alloc")]
// compile_error!("The 'alloc' feature is active");
extern crate alloc;

// FUTURE: Support serde via optional feature
mod dyn_sorted_disjoint;
mod from_slice;
mod integer;
mod merge;
mod not_iter;
pub mod prelude;
mod ranges;
#[cfg(feature = "rog-experimental")]
mod rog;
mod sorted_disjoint;
mod tests;
mod union_iter;
mod unsorted_disjoint;
pub use crate::ranges::{IntoRangesIter, RangesIter};
use alloc::{collections::BTreeMap, vec::Vec};
use core::{
    cmp::{max, Ordering},
    convert::From,
    fmt,
    iter::FusedIterator,
    ops::{BitOr, BitOrAssign, Bound, RangeBounds, RangeInclusive},
    str::FromStr,
};
pub use dyn_sorted_disjoint::DynSortedDisjoint;
// use from_slice_iter::FromSliceIter;
use gen_ops::gen_ops_ex;
use itertools::Tee;
pub use merge::{KMerge, Merge};
pub use not_iter::NotIter;
use num_traits::{ops::overflowing::OverflowingSub, CheckedAdd, One, WrappingSub, Zero};
#[cfg(feature = "rog-experimental")]
pub use rog::{Rog, RogsIter};
pub use sorted_disjoint::{CheckSortedDisjoint, SortedDisjoint, SortedStarts};
pub use union_iter::UnionIter;
pub use unsorted_disjoint::AssumeSortedStarts;
use unsorted_disjoint::SortedDisjointWithLenSoFar;
use unsorted_disjoint::UnsortedDisjoint;

/// The element trait of the [`RangeSetBlaze`] and [`SortedDisjoint`], specifically `u8` to `u128` (including `usize`) and `i8` to `i128` (including `isize`).
pub trait Integer:
    num_integer::Integer
    + FromStr
    + Copy
    + fmt::Display
    + fmt::Debug
    + core::iter::Sum
    + num_traits::NumAssignOps
    + num_traits::Bounded
    + num_traits::NumCast
    + Send
    + Sync
    + OverflowingSub
    + CheckedAdd
    + WrappingSub
{
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
    type SafeLen: core::hash::Hash
        + num_integer::Integer
        + num_traits::NumAssignOps
        + num_traits::Bounded
        + num_traits::NumCast
        + num_traits::One
        + core::ops::AddAssign
        + core::ops::SubAssign
        + Copy
        + PartialEq
        + Eq
        + PartialOrd
        + Ord
        + Send
        + Default
        + fmt::Debug
        + fmt::Display;

    /// Returns the length of a range without any overflow.
    ///
    /// # Example
    /// ```
    /// use range_set_blaze::Integer;
    ///
    /// assert_eq!(<u8 as Integer>::safe_len(&(0..=255)), 256);
    /// ```
    fn safe_len(range: &RangeInclusive<Self>) -> <Self as Integer>::SafeLen;

    /// For a given `Integer` type, returns the largest value that can be used. For all types other than `u128` and `i128`,
    /// this is the same as `Self::MAX`. For `u128` and `i128`, this is one less than `Self::MAX`.
    ///
    /// # Example
    /// ```
    /// use range_set_blaze::{Integer, RangeSetBlaze};
    ///
    /// // for i8, we can use up to 127
    /// let a = RangeSetBlaze::from_iter([i8::MAX]);
    /// // for i128, we can use up to 170141183460469231731687303715884105726
    /// let a = RangeSetBlaze::from_iter([<i128 as Integer>::safe_max_value()]);
    /// ```
    /// # Panics
    /// ```should_panic
    /// use range_set_blaze::{Integer, RangeSetBlaze};
    ///
    /// // for i128, using 170141183460469231731687303715884105727 throws a panic.
    /// let a = RangeSetBlaze::from_iter([i128::MAX]);
    /// ```
    fn safe_max_value() -> Self {
        Self::max_value()
    }

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

#[derive(Clone, Hash, Default, PartialEq)]

/// A set of integers stored as sorted & disjoint ranges.
///
/// Internally, it stores the ranges in a cache-efficient [`BTreeMap`].
///
/// # Table of Contents
/// * [`RangeSetBlaze` Constructors](#rangesetblaze-constructors)
///    * [Performance](#constructor-performance)
///    * [Examples](struct.RangeSetBlaze.html#constructor-examples)
/// * [`RangeSetBlaze` Set Operations](#rangesetblaze-set-operations)
///    * [Performance](struct.RangeSetBlaze.html#set-operation-performance)
///    * [Examples](struct.RangeSetBlaze.html#set-operation-examples)
///  * [`RangeSetBlaze` Comparisons](#rangesetblaze-comparisons)
///  * [Additional Examples](#additional-examples)
///
/// # `RangeSetBlaze` Constructors
///
/// You can also create `RangeSetBlaze`'s from unsorted and overlapping integers (or ranges).
/// However, if you know that your input is sorted and disjoint, you can speed up construction.
///
/// Here are the constructors, followed by a
/// description of the performance, and then some examples.
///
/// | Methods                                     | Input                        | Notes                    |
/// |---------------------------------------------|------------------------------|--------------------------|
/// | [`new`]/[`default`]                         |                              |                          |
/// | [`from_iter`][1]/[`collect`][1]             | integer iterator             |                          |
/// | [`from_iter`][2]/[`collect`][2]             | ranges iterator              |                          |
/// | [`from_slice`][5]                           | slice of integers            | Fast, but nightly-only  |
/// | [`from_sorted_disjoint`][3]/[`into_range_set_blaze`][3] | [`SortedDisjoint`] iterator |               |
/// | [`from`][4] /[`into`][4]                    | array of integers            |                          |
///
///
/// [`BTreeMap`]: alloc::collections::BTreeMap
/// [`new`]: RangeSetBlaze::new
/// [`default`]: RangeSetBlaze::default
/// [1]: struct.RangeSetBlaze.html#impl-FromIterator<T>-for-RangeSetBlaze<T>
/// [2]: struct.RangeSetBlaze.html#impl-FromIterator<RangeInclusive<T>>-for-RangeSetBlaze<T>
/// [3]: RangeSetBlaze::from_sorted_disjoint
/// [4]: RangeSetBlaze::from
/// [5]: RangeSetBlaze::from_slice()
///
/// # Constructor Performance
///
/// The [`from_iter`][1]/[`collect`][1] constructors are designed to work fast on 'clumpy' data.
/// By 'clumpy', we mean that the number of ranges needed to represent the data is
/// small compared to the number of input integers. To understand this, consider the internals
/// of the constructors:
///
///  Internally, the `from_iter`/`collect` constructors take these steps:
/// * collect adjacent integers/ranges into disjoint ranges, O(*n₁*)
/// * sort the disjoint ranges by their `start`, O(*n₂* log *n₂*)
/// * merge adjacent ranges, O(*n₂*)
/// * create a `BTreeMap` from the now sorted & disjoint ranges, O(*n₃* log *n₃*)
///
/// where *n₁* is the number of input integers/ranges, *n₂* is the number of disjoint & unsorted ranges,
/// and *n₃* is the final number of sorted & disjoint ranges.
///
/// For example, an input of
///  *  `3, 2, 1, 4, 5, 6, 7, 0, 8, 8, 8, 100, 1`, becomes
///  * `0..=8, 100..=100, 1..=1`, and then
///  * `0..=8, 1..=1, 100..=100`, and finally
///  * `0..=8, 100..=100`.
///
/// What is the effect of clumpy data?
/// Notice that if *n₂* ≈ sqrt(*n₁*), then construction is O(*n₁*).
/// (Indeed, as long as *n₂* ≤ *n₁*/ln(*n₁*), then construction is O(*n₁*).)
/// Moreover, we'll see that set operations are O(*n₃*). Thus, if *n₃* ≈ sqrt(*n₁*) then set operations are O(sqrt(*n₁*)),
/// a quadratic improvement an O(*n₁*) implementation that ignores the clumps.
///
/// The [`from_slice`][5] constructor typically provides a constant-time speed up for array-like collections of clumpy integers.
/// On a representative benchmark, the speed up was 6×.
/// The method works by scanning the input for blocks of consecutive integers, and then using `from_iter` on the results.
/// Where available, it uses SIMD instructions. It is nightly only and enabled by the `from_slice` feature.
///
/// ## Constructor Examples
///
/// ```
/// use range_set_blaze::prelude::*;
///
/// // Create an empty set with 'new' or 'default'.
/// let a0 = RangeSetBlaze::<i32>::new();
/// let a1 = RangeSetBlaze::<i32>::default();
/// assert!(a0 == a1 && a0.is_empty());
///
/// // 'from_iter'/'collect': From an iterator of integers.
/// // Duplicates and out-of-order elements are fine.
/// let a0 = RangeSetBlaze::from_iter([3, 2, 1, 100, 1]);
/// let a1: RangeSetBlaze<i32> = [3, 2, 1, 100, 1].into_iter().collect();
/// assert!(a0 == a1 && a0.to_string() == "1..=3, 100..=100");
///
/// // 'from_iter'/'collect': From an iterator of inclusive ranges, start..=end.
/// // Overlapping, out-of-order, and empty ranges are fine.
/// #[allow(clippy::reversed_empty_ranges)]
/// let a0 = RangeSetBlaze::from_iter([1..=2, 2..=2, -10..=-5, 1..=0]);
/// #[allow(clippy::reversed_empty_ranges)]
/// let a1: RangeSetBlaze<i32> = [1..=2, 2..=2, -10..=-5, 1..=0].into_iter().collect();
/// assert!(a0 == a1 && a0.to_string() == "-10..=-5, 1..=2");
///
/// // 'from_slice': From any array-like collection of integers.
/// // Nightly-only, but faster than 'from_iter'/'collect' on integers.
/// #[cfg(feature = "from_slice")]
/// let a0 = RangeSetBlaze::from_slice(vec![3, 2, 1, 100, 1]);
/// #[cfg(feature = "from_slice")]
/// assert!(a0.to_string() == "1..=3, 100..=100");
///
/// // If we know the ranges are already sorted and disjoint,
/// // we can avoid work and use 'from'/'into'.
/// let a0 = RangeSetBlaze::from_sorted_disjoint(CheckSortedDisjoint::from([-10..=-5, 1..=2]));
/// let a1: RangeSetBlaze<i32> = CheckSortedDisjoint::from([-10..=-5, 1..=2]).into_range_set_blaze();
/// assert!(a0 == a1 && a0.to_string() == "-10..=-5, 1..=2");
///
/// // For compatibility with `BTreeSet`, we also support
/// // 'from'/'into' from arrays of integers.
/// let a0 = RangeSetBlaze::from([3, 2, 1, 100, 1]);
/// let a1: RangeSetBlaze<i32> = [3, 2, 1, 100, 1].into();
/// assert!(a0 == a1 && a0.to_string() == "1..=3, 100..=100");
/// ```
///
/// # `RangeSetBlaze` Set Operations
///
/// You can perform set operations on `RangeSetBlaze`s using operators.
///
/// | Set Operation           | Operator                   |  Multiway Method |
/// |-------------------|-------------------------|-------------------------|
/// | union       |  `a` &#124; `b`                     | `[a, b, c].`[`union`]`()` |
/// | intersection       |  `a & b`                     | `[a, b, c].`[`intersection`]`()` |
/// | difference       |  `a - b`                     | *n/a* |
/// | symmetric difference       |  `a ^ b`                     | *n/a* |
/// | complement       |  `!a`                     | *n/a* |
///
/// `RangeSetBlaze` also implements many other methods, such as [`insert`], [`pop_first`] and [`split_off`]. Many of
/// these methods match those of `BTreeSet`.
///
/// [`union`]: trait.MultiwayRangeSetBlaze.html#method.union
/// [`intersection`]: trait.MultiwayRangeSetBlaze.html#method.intersection
/// [`insert`]: RangeSetBlaze::insert
/// [`pop_first`]: RangeSetBlaze::pop_first
/// [`split_off`]: RangeSetBlaze::split_off
///
///
/// ## Set Operation Performance
///
/// Every operation is implemented as
/// 1. a single pass over the sorted & disjoint ranges
/// 2. the construction of a new `RangeSetBlaze`
///
/// Thus, applying multiple operators creates intermediate
/// `RangeSetBlaze`'s. If you wish, you can avoid these intermediate
/// `RangeSetBlaze`'s by switching to the [`SortedDisjoint`] API. The last example below
/// demonstrates this.
///
/// ## Set Operation Examples
///
/// ```
/// use range_set_blaze::prelude::*;
///
/// let a = RangeSetBlaze::from_iter([1..=2, 5..=100]);
/// let b = RangeSetBlaze::from_iter([2..=6]);
///
/// // Union of two 'RangeSetBlaze's.
/// let result = &a | &b;
/// // Alternatively, we can take ownership via 'a | b'.
/// assert_eq!(result.to_string(), "1..=100");
///
/// // Intersection of two 'RangeSetBlaze's.
/// let result = &a & &b; // Alternatively, 'a & b'.
/// assert_eq!(result.to_string(), "2..=2, 5..=6");
///
/// // Set difference of two 'RangeSetBlaze's.
/// let result = &a - &b; // Alternatively, 'a - b'.
/// assert_eq!(result.to_string(), "1..=1, 7..=100");
///
/// // Symmetric difference of two 'RangeSetBlaze's.
/// let result = &a ^ &b; // Alternatively, 'a ^ b'.
/// assert_eq!(result.to_string(), "1..=1, 3..=4, 7..=100");
///
/// // complement of a 'RangeSetBlaze'.
/// let result = !&a; // Alternatively, '!a'.
/// assert_eq!(
///     result.to_string(),
///     "-2147483648..=0, 3..=4, 101..=2147483647"
/// );
///
/// // Multiway union of 'RangeSetBlaze's.
/// let c = RangeSetBlaze::from_iter([2..=2, 6..=200]);
/// let result = [&a, &b, &c].union();
/// assert_eq!(result.to_string(), "1..=200");
///
/// // Multiway intersection of 'RangeSetBlaze's.
/// let result = [&a, &b, &c].intersection();
/// assert_eq!(result.to_string(), "2..=2, 6..=6");
///
/// // Applying multiple operators
/// let result0 = &a - (&b | &c); // Creates an intermediate 'RangeSetBlaze'.
/// // Alternatively, we can use the 'SortedDisjoint' API and avoid the intermediate 'RangeSetBlaze'.
/// let result1 = RangeSetBlaze::from_sorted_disjoint(a.ranges() - (b.ranges() | c.ranges()));
/// assert!(result0 == result1 && result0.to_string() == "1..=1");
/// ```
/// # `RangeSetBlaze` Comparisons
///
/// We can compare `RangeSetBlaze`s using the following operators:
/// `<`, `<=`, `>`, `>=`.  Following the convention of `BTreeSet`,
/// these comparisons are lexicographic. See [`cmp`] for more examples.
///
/// Use the [`is_subset`] and [`is_superset`] methods to check if one `RangeSetBlaze` is a subset
/// or superset of another.
///
/// Use `==`, `!=` to check if two `RangeSetBlaze`s are equal or not.
///
/// [`BTreeSet`]: alloc::collections::BTreeSet
/// [`is_subset`]: RangeSetBlaze::is_subset
/// [`is_superset`]: RangeSetBlaze::is_superset
/// [`cmp`]: RangeSetBlaze::cmp
///
/// # Additional Examples
///
/// See the [module-level documentation] for additional examples.
///
/// [module-level documentation]: index.html
pub struct RangeSetBlaze<T: Integer> {
    len: <T as Integer>::SafeLen,
    btree_map: BTreeMap<T, T>,
}

// FUTURE: Make all RangeSetBlaze iterators DoubleEndedIterator and ExactSizeIterator.
impl<T: Integer> fmt::Debug for RangeSetBlaze<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.ranges().to_string())
    }
}

impl<T: Integer> fmt::Display for RangeSetBlaze<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.ranges().to_string())
    }
}

impl<T: Integer> RangeSetBlaze<T> {
    /// Gets an iterator that visits the integer elements in the [`RangeSetBlaze`] in ascending
    /// order.
    ///
    /// Also see the [`RangeSetBlaze::ranges`] method.
    ///
    /// # Examples
    ///
    /// ```
    /// use range_set_blaze::RangeSetBlaze;
    ///
    /// let set = RangeSetBlaze::from_iter([1..=3]);
    /// let mut set_iter = set.iter();
    /// assert_eq!(set_iter.next(), Some(1));
    /// assert_eq!(set_iter.next(), Some(2));
    /// assert_eq!(set_iter.next(), Some(3));
    /// assert_eq!(set_iter.next(), None);
    /// ```
    ///
    /// Values returned by the iterator are returned in ascending order:
    ///
    /// ```
    /// use range_set_blaze::RangeSetBlaze;
    ///
    /// let set = RangeSetBlaze::from_iter([3, 1, 2]);
    /// let mut set_iter = set.iter();
    /// assert_eq!(set_iter.next(), Some(1));
    /// assert_eq!(set_iter.next(), Some(2));
    /// assert_eq!(set_iter.next(), Some(3));
    /// assert_eq!(set_iter.next(), None);
    /// ```
    pub fn iter(&self) -> Iter<T, RangesIter<T>> {
        // If the user asks for an iter, we give them a RangesIter iterator
        // and we iterate that one integer at a time.
        Iter {
            option_range: None,
            iter: self.ranges(),
        }
    }

    /// Returns the first element in the set, if any.
    /// This element is always the minimum of all integer elements in the set.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// use range_set_blaze::RangeSetBlaze;
    ///
    /// let mut set = RangeSetBlaze::new();
    /// assert_eq!(set.first(), None);
    /// set.insert(1);
    /// assert_eq!(set.first(), Some(1));
    /// set.insert(2);
    /// assert_eq!(set.first(), Some(1));
    /// ```
    #[must_use]
    pub fn first(&self) -> Option<T> {
        self.btree_map.iter().next().map(|(x, _)| *x)
    }

    /// Returns the element in the set, if any, that is equal to
    /// the value.
    ///
    /// # Examples
    ///
    /// ```
    /// use range_set_blaze::RangeSetBlaze;
    ///
    /// let set = RangeSetBlaze::from_iter([1, 2, 3]);
    /// assert_eq!(set.get(2), Some(2));
    /// assert_eq!(set.get(4), None);
    /// ```
    pub fn get(&self, value: T) -> Option<T> {
        if self.contains(value) {
            Some(value)
        } else {
            None
        }
    }

    /// Returns the last element in the set, if any.
    /// This element is always the maximum of all elements in the set.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// use range_set_blaze::RangeSetBlaze;
    ///
    /// let mut set = RangeSetBlaze::new();
    /// assert_eq!(set.last(), None);
    /// set.insert(1);
    /// assert_eq!(set.last(), Some(1));
    /// set.insert(2);
    /// assert_eq!(set.last(), Some(2));
    /// ```
    #[must_use]
    pub fn last(&self) -> Option<T> {
        self.btree_map.iter().next_back().map(|(_, x)| *x)
    }

    /// Create a [`RangeSetBlaze`] from a [`SortedDisjoint`] iterator.
    ///
    /// *For more about constructors and performance, see [`RangeSetBlaze` Constructors](struct.RangeSetBlaze.html#rangesetblaze-constructors).*
    ///
    /// # Examples
    ///
    /// ```
    /// use range_set_blaze::prelude::*;
    ///
    /// let a0 = RangeSetBlaze::from_sorted_disjoint(CheckSortedDisjoint::from([-10..=-5, 1..=2]));
    /// let a1: RangeSetBlaze<i32> = CheckSortedDisjoint::from([-10..=-5, 1..=2]).into_range_set_blaze();
    /// assert!(a0 == a1 && a0.to_string() == "-10..=-5, 1..=2");
    /// ```
    pub fn from_sorted_disjoint<I>(iter: I) -> Self
    where
        I: SortedDisjoint<T>,
    {
        let mut iter_with_len = SortedDisjointWithLenSoFar::from(iter);
        let btree_map = BTreeMap::from_iter(&mut iter_with_len);
        RangeSetBlaze {
            btree_map,
            len: iter_with_len.len_so_far(),
        }
    }

    // cmk Rule be sure ints don't wrap in a way that could be bad.
    // cmk Rule handle alignment at the start and end.

    /// Creates a [`RangeSetBlaze`] from a collection of integers. It is typically many
    /// times faster than [`from_iter`][1]/[`collect`][1].
    /// On a representative benchmark, the speed up was 6×.
    ///
    /// **Warning: Requires the nightly compiler. Also, you must enable the `from_slice`
    /// feature in your `Cargo.toml`. For example:**
    /// ```toml
    /// [dependencies]
    /// range-set-blaze = { features = ["std", "from_slice"], version=VERSION }
    /// ```
    /// *Replace VERSION with the current version.*
    ///
    /// The function accepts any type that can be referenced as a slice of integers,
    /// including slices, arrays, and vectors. Duplicates and out-of-order elements are fine.
    ///
    /// Where available, this function leverages SIMD (Single Instruction, Multiple Data) instructions
    /// for performance optimization. To enable SIMD optimizations, compile with the Rust compiler
    /// (rustc) flag `-C target-cpu=native`. This instructs rustc to use the native instruction set
    /// of the CPU on the machine compiling the code, potentially enabling more SIMD optimizations.
    ///
    /// **Caution**: Compiling with `-C target-cpu=native` optimizes the binary for your current CPU architecture,
    /// which may lead to compatibility issues on other machines with different architectures.
    /// This is particularly important for distributing the binary or running it in varied environments.
    ///
    /// *For more about constructors and performance, see [`RangeSetBlaze` Constructors](struct.RangeSetBlaze.html#rangesetblaze-constructors).*
    ///
    /// # Examples
    ///
    /// ```
    /// use range_set_blaze::RangeSetBlaze;
    ///
    /// let a0 = RangeSetBlaze::from_slice(&[3, 2, 1, 100, 1]); // reference to a slice
    /// let a1 = RangeSetBlaze::from_slice([3, 2, 1, 100, 1]);   // array
    /// let a2 = RangeSetBlaze::from_slice(vec![3, 2, 1, 100, 1]); // vector
    /// assert!(a0 == a1 && a1 == a2 && a0.to_string() == "1..=3, 100..=100");
    /// ```
    /// [1]: struct.RangeSetBlaze.html#impl-FromIterator<T>-for-RangeSetBlaze<T>
    #[cfg(feature = "from_slice")]
    #[inline]
    pub fn from_slice(slice: impl AsRef<[T]>) -> Self {
        T::from_slice(slice)
    }

    fn _len_slow(&self) -> <T as Integer>::SafeLen {
        RangeSetBlaze::btree_map_len(&self.btree_map)
    }

    /// Moves all elements from `other` into `self`, leaving `other` empty.
    ///
    /// # Performance
    /// It adds the integers in `other` to `self` in O(n log m) time, where n is the number of ranges in `other`
    /// and m is the number of ranges in `self`.
    /// When n is large, consider using `|` which is O(n+m) time.
    ///
    /// # Examples
    ///
    /// ```
    /// use range_set_blaze::RangeSetBlaze;
    ///
    /// let mut a = RangeSetBlaze::from_iter([1..=3]);
    /// let mut b = RangeSetBlaze::from_iter([3..=5]);
    ///
    /// a.append(&mut b);
    ///
    /// assert_eq!(a.len(), 5usize);
    /// assert_eq!(b.len(), 0usize);
    ///
    /// assert!(a.contains(1));
    /// assert!(a.contains(2));
    /// assert!(a.contains(3));
    /// assert!(a.contains(4));
    /// assert!(a.contains(5));
    ///
    /// ```
    pub fn append(&mut self, other: &mut Self) {
        for range in other.ranges() {
            self.internal_add(range);
        }
        other.clear();
    }

    /// Clears the set, removing all integer elements.
    ///
    /// # Examples
    ///
    /// ```
    /// use range_set_blaze::RangeSetBlaze;
    ///
    /// let mut v = RangeSetBlaze::new();
    /// v.insert(1);
    /// v.clear();
    /// assert!(v.is_empty());
    /// ```
    pub fn clear(&mut self) {
        self.btree_map.clear();
        self.len = <T as Integer>::SafeLen::zero();
    }

    /// Returns `true` if the set contains no elements.
    ///
    /// # Examples
    ///
    /// ```
    /// use range_set_blaze::RangeSetBlaze;
    ///
    /// let mut v = RangeSetBlaze::new();
    /// assert!(v.is_empty());
    /// v.insert(1);
    /// assert!(!v.is_empty());
    /// ```
    #[must_use]
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.ranges_len() == 0
    }

    /// Returns `true` if the set is a subset of another,
    /// i.e., `other` contains at least all the elements in `self`.
    ///
    /// # Examples
    ///
    /// ```
    /// use range_set_blaze::RangeSetBlaze;
    ///
    /// let sup = RangeSetBlaze::from_iter([1..=3]);
    /// let mut set = RangeSetBlaze::new();
    ///
    /// assert_eq!(set.is_subset(&sup), true);
    /// set.insert(2);
    /// assert_eq!(set.is_subset(&sup), true);
    /// set.insert(4);
    /// assert_eq!(set.is_subset(&sup), false);
    /// ```
    #[must_use]
    #[inline]
    pub fn is_subset(&self, other: &RangeSetBlaze<T>) -> bool {
        // Add a fast path
        if self.len() > other.len() {
            return false;
        }
        self.ranges().is_subset(other.ranges())
    }

    /// Returns `true` if the set is a superset of another,
    /// i.e., `self` contains at least all the elements in `other`.
    ///
    /// # Examples
    ///
    /// ```
    /// use range_set_blaze::RangeSetBlaze;
    ///
    /// let sub = RangeSetBlaze::from_iter([1, 2]);
    /// let mut set = RangeSetBlaze::new();
    ///
    /// assert_eq!(set.is_superset(&sub), false);
    ///
    /// set.insert(0);
    /// set.insert(1);
    /// assert_eq!(set.is_superset(&sub), false);
    ///
    /// set.insert(2);
    /// assert_eq!(set.is_superset(&sub), true);
    /// ```
    #[must_use]
    pub fn is_superset(&self, other: &RangeSetBlaze<T>) -> bool {
        other.is_subset(self)
    }

    /// Returns `true` if the set contains an element equal to the value.
    ///
    /// # Examples
    ///
    /// ```
    /// use range_set_blaze::RangeSetBlaze;
    ///
    /// let set = RangeSetBlaze::from_iter([1, 2, 3]);
    /// assert_eq!(set.contains(1), true);
    /// assert_eq!(set.contains(4), false);
    /// ```
    pub fn contains(&self, value: T) -> bool {
        assert!(
            value <= T::safe_max_value(),
            "value must be <= T::safe_max_value()"
        );
        self.btree_map
            .range(..=value)
            .next_back()
            .map_or(false, |(_, end)| value <= *end)
    }

    /// Returns `true` if `self` has no elements in common with `other`.
    /// This is equivalent to checking for an empty intersection.
    ///
    /// # Examples
    ///
    /// ```
    /// use range_set_blaze::RangeSetBlaze;
    ///
    /// let a = RangeSetBlaze::from_iter([1..=3]);
    /// let mut b = RangeSetBlaze::new();
    ///
    /// assert_eq!(a.is_disjoint(&b), true);
    /// b.insert(4);
    /// assert_eq!(a.is_disjoint(&b), true);
    /// b.insert(1);
    /// assert_eq!(a.is_disjoint(&b), false);
    /// ```
    #[must_use]
    #[inline]
    pub fn is_disjoint(&self, other: &RangeSetBlaze<T>) -> bool {
        self.ranges().is_disjoint(other.ranges())
    }

    fn delete_extra(&mut self, internal_range: &RangeInclusive<T>) {
        let (start, end) = internal_range.clone().into_inner();
        let mut after = self.btree_map.range_mut(start..);
        let (start_after, end_after) = after.next().unwrap(); // there will always be a next
        debug_assert!(start == *start_after && end == *end_after);

        let mut end_new = end;
        let delete_list = after
            .map_while(|(start_delete, end_delete)| {
                // must check this in two parts to avoid overflow
                if *start_delete <= end || *start_delete <= end + T::one() {
                    end_new = max(end_new, *end_delete);
                    self.len -= T::safe_len(&(*start_delete..=*end_delete));
                    Some(*start_delete)
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();
        if end_new > end {
            self.len += T::safe_len(&(end..=end_new - T::one()));
            *end_after = end_new;
        }
        for start in delete_list {
            self.btree_map.remove(&start);
        }
    }

    /// Adds a value to the set.
    ///
    /// Returns whether the value was newly inserted. That is:
    ///
    /// - If the set did not previously contain an equal value, `true` is
    ///   returned.
    /// - If the set already contained an equal value, `false` is returned, and
    ///   the entry is not updated.
    ///
    /// # Performance
    /// Inserting n items will take in O(n log m) time, where n is the number of inserted items and m is the number of ranges in `self`.
    /// When n is large, consider using `|` which is O(n+m) time.
    ///
    /// # Examples
    ///
    /// ```
    /// use range_set_blaze::RangeSetBlaze;
    ///
    /// let mut set = RangeSetBlaze::new();
    ///
    /// assert_eq!(set.insert(2), true);
    /// assert_eq!(set.insert(2), false);
    /// assert_eq!(set.len(), 1usize);
    /// ```
    pub fn insert(&mut self, value: T) -> bool {
        let len_before = self.len;
        self.internal_add(value..=value);
        self.len != len_before
    }

    /// Constructs an iterator over a sub-range of elements in the set.
    ///
    /// Not to be confused with [`RangeSetBlaze::ranges`], which returns an iterator over the ranges in the set.
    ///
    /// The simplest way is to use the range syntax `min..max`, thus `range(min..max)` will
    /// yield elements from min (inclusive) to max (exclusive).
    /// The range may also be entered as `(Bound<T>, Bound<T>)`, so for example
    /// `range((Excluded(4), Included(10)))` will yield a left-exclusive, right-inclusive
    /// range from 4 to 10.
    ///
    /// # Panics
    ///
    /// Panics if range `start > end`.
    /// Panics if range `start == end` and both bounds are `Excluded`.
    ///
    /// # Performance
    ///
    /// Although this could be written to run in time O(ln(n)) in the number of ranges, it is currently O(n) in the number of ranges.
    ///
    /// # Examples
    ///
    /// ```
    /// use range_set_blaze::RangeSetBlaze;
    /// use core::ops::Bound::Included;
    ///
    /// let mut set = RangeSetBlaze::new();
    /// set.insert(3);
    /// set.insert(5);
    /// set.insert(8);
    /// for elem in set.range((Included(4), Included(8))) {
    ///     println!("{elem}");
    /// }
    /// assert_eq!(Some(5), set.range(4..).next());
    /// ```
    pub fn range<R>(&self, range: R) -> IntoIter<T>
    where
        R: RangeBounds<T>,
    {
        let start = match range.start_bound() {
            Bound::Included(n) => *n,
            Bound::Excluded(n) => *n + T::one(),
            Bound::Unbounded => T::min_value(),
        };
        let end = match range.end_bound() {
            Bound::Included(n) => *n,
            Bound::Excluded(n) => *n - T::one(),
            Bound::Unbounded => T::safe_max_value(),
        };
        assert!(start <= end);

        let bounds = CheckSortedDisjoint::from([start..=end]);
        RangeSetBlaze::from_sorted_disjoint(self.ranges() & bounds).into_iter()
    }

    /// Adds a range to the set.
    ///
    /// Returns whether any values where newly inserted. That is:
    ///
    /// - If the set did not previously contain some value in the range, `true` is
    ///   returned.
    /// - If the set already contained every value in the range, `false` is returned, and
    ///   the entry is not updated.
    ///
    /// # Performance
    /// Inserting n items will take in O(n log m) time, where n is the number of inserted items and m is the number of ranges in `self`.
    /// When n is large, consider using `|` which is O(n+m) time.
    ///
    /// # Examples
    ///
    /// ```
    /// use range_set_blaze::RangeSetBlaze;
    ///
    /// let mut set = RangeSetBlaze::new();
    ///
    /// assert_eq!(set.ranges_insert(2..=5), true);
    /// assert_eq!(set.ranges_insert(5..=6), true);
    /// assert_eq!(set.ranges_insert(3..=4), false);
    /// assert_eq!(set.len(), 5usize);
    /// ```
    pub fn ranges_insert(&mut self, range: RangeInclusive<T>) -> bool {
        let len_before = self.len;
        self.internal_add(range);
        self.len != len_before
    }

    /// If the set contains an element equal to the value, removes it from the
    /// set and drops it. Returns whether such an element was present.
    ///
    /// # Examples
    ///
    /// ```
    /// use range_set_blaze::RangeSetBlaze;
    ///
    /// let mut set = RangeSetBlaze::new();
    ///
    /// set.insert(2);
    /// assert!(set.remove(2));
    /// assert!(!set.remove(2));
    /// ```
    pub fn remove(&mut self, value: T) -> bool {
        assert!(
            value <= T::safe_max_value(),
            "value must be <= T::safe_max_value()"
        );

        // The code can have only one mutable reference to self.btree_map.
        let start;
        let end;
        if let Some((start_ref, end_ref)) = self.btree_map.range_mut(..=value).next_back() {
            end = *end_ref;
            if end < value {
                return false;
            }
            start = *start_ref;
            // special case if in range and start strictly less than value
            if start < value {
                *end_ref = value - T::one();
                // special, special case if value == end
                if value == end {
                    self.len -= <T::SafeLen>::one();
                    return true;
                }
            }
        } else {
            return false;
        };
        self.len -= <T::SafeLen>::one();
        if start == value {
            self.btree_map.remove(&start);
        };
        if value < end {
            self.btree_map.insert(value + T::one(), end);
        }
        true
    }

    /// Splits the collection into two at the value. Returns a new collection
    /// with all elements greater than or equal to the value.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// use range_set_blaze::RangeSetBlaze;
    ///
    /// let mut a = RangeSetBlaze::new();
    /// a.insert(1);
    /// a.insert(2);
    /// a.insert(3);
    /// a.insert(17);
    /// a.insert(41);
    ///
    /// let b = a.split_off(3);
    ///
    /// assert_eq!(a, RangeSetBlaze::from_iter([1, 2]));
    /// assert_eq!(b, RangeSetBlaze::from_iter([3, 17, 41]));
    /// ```
    pub fn split_off(&mut self, value: T) -> Self {
        assert!(
            value <= T::safe_max_value(),
            "value must be <= T::safe_max_value()"
        );
        let old_len = self.len;
        let mut b = self.btree_map.split_off(&value);
        if let Some(mut last_entry) = self.btree_map.last_entry() {
            // Can assume start strictly less than value
            let end_ref = last_entry.get_mut();
            if value <= *end_ref {
                b.insert(value, *end_ref);
                *end_ref = value - T::one();
            }
        }

        // Find the length of the smaller map and then length of self & b.
        let b_len = if self.btree_map.len() < b.len() {
            self.len = RangeSetBlaze::btree_map_len(&self.btree_map);
            old_len - self.len
        } else {
            let b_len = RangeSetBlaze::btree_map_len(&b);
            self.len = old_len - b_len;
            b_len
        };
        RangeSetBlaze {
            btree_map: b,
            len: b_len,
        }
    }

    fn btree_map_len(btree_map: &BTreeMap<T, T>) -> T::SafeLen {
        btree_map
            .iter()
            .fold(<T as Integer>::SafeLen::zero(), |acc, (start, end)| {
                acc + T::safe_len(&(*start..=*end))
            })
    }

    /// Removes and returns the element in the set, if any, that is equal to
    /// the value.
    ///
    /// # Examples
    ///
    /// ```
    /// use range_set_blaze::RangeSetBlaze;
    ///
    /// let mut set = RangeSetBlaze::from_iter([1, 2, 3]);
    /// assert_eq!(set.take(2), Some(2));
    /// assert_eq!(set.take(2), None);
    /// ```
    pub fn take(&mut self, value: T) -> Option<T> {
        if self.remove(value) {
            Some(value)
        } else {
            None
        }
    }

    /// Adds a value to the set, replacing the existing element, if any, that is
    /// equal to the value. Returns the replaced element.
    ///
    /// Note: This is very similar to `insert`. It is included for consistency with [`BTreeSet`].
    ///
    /// [`BTreeSet`]: alloc::collections::BTreeSet
    ///
    /// # Examples
    ///
    /// ```
    /// use range_set_blaze::RangeSetBlaze;
    ///
    /// let mut set = RangeSetBlaze::new();
    /// assert!(set.replace(5).is_none());
    /// assert!(set.replace(5).is_some());
    /// ```
    pub fn replace(&mut self, value: T) -> Option<T> {
        if self.insert(value) {
            None
        } else {
            Some(value)
        }
    }

    // fn internal_add_chatgpt(&mut self, range: RangeInclusive<T>) {
    //     let (start, end) = range.into_inner();

    //     // Find the first overlapping range or the nearest one before it
    //     let mut next = self.btree_map.range(..=start).next_back();

    //     // Find all overlapping ranges
    //     while let Some((&start_key, &end_value)) = next {
    //         // If this range doesn't overlap, we're done
    //         if end_value < start {
    //             break;
    //         }

    //         // If this range overlaps or is adjacent, merge it
    //         if end_value >= start - T::one() {
    //             let new_end = end.max(end_value);
    //             let new_start = start.min(start_key);

    //             self.btree_map.remove(&start_key);
    //             self.btree_map.insert(new_start, new_end);

    //             // Restart from the beginning
    //             next = self.btree_map.range(..=new_start).next_back();
    //         } else {
    //             next = self.btree_map.range(..start_key).next_back();
    //         }
    //     }
    // }

    // https://stackoverflow.com/questions/49599833/how-to-find-next-smaller-key-in-btreemap-btreeset
    // https://stackoverflow.com/questions/35663342/how-to-modify-partially-remove-a-range-from-a-btreemap
    fn internal_add(&mut self, range: RangeInclusive<T>) {
        let (start, end) = range.clone().into_inner();
        assert!(
            end <= T::safe_max_value(),
            "end must be <= T::safe_max_value()"
        );
        if end < start {
            return;
        }
        // FUTURE: would be nice of BTreeMap to have a partition_point function that returns two iterators
        let mut before = self.btree_map.range_mut(..=start).rev();
        if let Some((start_before, end_before)) = before.next() {
            // Must check this in two parts to avoid overflow
            if match (*end_before).checked_add(&T::one()) {
                Some(end_before_succ) => end_before_succ < start,
                None => false,
            } {
                self.internal_add2(&range);
            } else if *end_before < end {
                self.len += T::safe_len(&(*end_before..=end - T::one()));
                *end_before = end;
                let start_before = *start_before;
                self.delete_extra(&(start_before..=end));
            } else {
                // completely contained, so do nothing
            }
        } else {
            self.internal_add2(&range);
        }
    }

    fn internal_add2(&mut self, internal_range: &RangeInclusive<T>) {
        let (start, end) = internal_range.clone().into_inner();
        let was_there = self.btree_map.insert(start, end);
        debug_assert!(was_there.is_none()); // real assert
        self.delete_extra(internal_range);
        self.len += T::safe_len(internal_range);
    }

    /// Returns the number of elements in the set.
    ///
    /// The number is allowed to be very, very large.
    ///
    /// # Examples
    ///
    /// ```
    /// use range_set_blaze::RangeSetBlaze;
    ///
    /// let mut v = RangeSetBlaze::new();
    /// assert_eq!(v.len(), 0usize);
    /// v.insert(1);
    /// assert_eq!(v.len(), 1usize);
    ///
    /// let v = RangeSetBlaze::from_iter([
    ///     -170_141_183_460_469_231_731_687_303_715_884_105_728i128..=10,
    ///     -10..=170_141_183_460_469_231_731_687_303_715_884_105_726,
    /// ]);
    /// assert_eq!(
    ///     v.len(),
    ///     340_282_366_920_938_463_463_374_607_431_768_211_455u128
    /// );
    /// ```
    #[must_use]
    pub const fn len(&self) -> <T as Integer>::SafeLen {
        self.len
    }

    /// Makes a new, empty [`RangeSetBlaze`].
    ///
    /// # Examples
    ///
    /// ```
    /// # #![allow(unused_mut)]
    /// use range_set_blaze::RangeSetBlaze;
    ///
    /// let mut set: RangeSetBlaze<i32> = RangeSetBlaze::new();
    /// ```
    #[must_use]
    pub fn new() -> Self {
        RangeSetBlaze {
            btree_map: BTreeMap::new(),
            len: <T as Integer>::SafeLen::zero(),
        }
    }

    /// Removes the first element from the set and returns it, if any.
    /// The first element is always the minimum element in the set.
    ///
    /// # Examples
    ///
    /// ```
    /// use range_set_blaze::RangeSetBlaze;
    ///
    /// let mut set = RangeSetBlaze::new();
    ///
    /// set.insert(1);
    /// while let Some(n) = set.pop_first() {
    ///     assert_eq!(n, 1);
    /// }
    /// assert!(set.is_empty());
    /// ```
    pub fn pop_first(&mut self) -> Option<T> {
        if let Some(entry) = self.btree_map.first_entry() {
            let (start, end) = entry.remove_entry();
            self.len -= T::safe_len(&(start..=end));
            if start != end {
                let start = start + T::one();
                self.btree_map.insert(start, end);
                self.len += T::safe_len(&(start..=end));
            }
            Some(start)
        } else {
            None
        }
    }

    /// Removes the last value from the set and returns it, if any.
    /// The last value is always the maximum value in the set.
    ///
    /// # Examples
    ///
    /// ```
    /// use range_set_blaze::RangeSetBlaze;
    ///
    /// let mut set = RangeSetBlaze::new();
    ///
    /// set.insert(1);
    /// while let Some(n) = set.pop_last() {
    ///     assert_eq!(n, 1);
    /// }
    /// assert!(set.is_empty());
    /// ```
    pub fn pop_last(&mut self) -> Option<T> {
        if let Some(mut entry) = self.btree_map.last_entry() {
            let start = *entry.key();
            let end = entry.get_mut();
            let result = *end;
            self.len -= T::safe_len(&(start..=*end));
            if start == *end {
                entry.remove_entry();
            } else {
                *end -= T::one();
                self.len += T::safe_len(&(start..=*end));
            }
            Some(result)
        } else {
            None
        }
    }

    /// An iterator that visits the ranges in the [`RangeSetBlaze`],
    /// i.e., the integers as sorted & disjoint ranges.
    ///
    /// Also see [`RangeSetBlaze::iter`] and [`RangeSetBlaze::into_ranges`].
    ///
    /// # Examples
    ///
    /// ```
    /// use range_set_blaze::RangeSetBlaze;
    ///
    /// let set = RangeSetBlaze::from_iter([10..=20, 15..=25, 30..=40]);
    /// let mut ranges = set.ranges();
    /// assert_eq!(ranges.next(), Some(10..=25));
    /// assert_eq!(ranges.next(), Some(30..=40));
    /// assert_eq!(ranges.next(), None);
    /// ```
    ///
    /// Values returned by the iterator are returned in ascending order:
    ///
    /// ```
    /// use range_set_blaze::RangeSetBlaze;
    ///
    /// let set = RangeSetBlaze::from_iter([30..=40, 15..=25, 10..=20]);
    /// let mut ranges = set.ranges();
    /// assert_eq!(ranges.next(), Some(10..=25));
    /// assert_eq!(ranges.next(), Some(30..=40));
    /// assert_eq!(ranges.next(), None);
    /// ```
    pub fn ranges(&self) -> RangesIter<'_, T> {
        RangesIter {
            iter: self.btree_map.iter(),
        }
    }

    /// An iterator that moves out the ranges in the [`RangeSetBlaze`],
    /// i.e., the integers as sorted & disjoint ranges.
    ///
    /// Also see [`RangeSetBlaze::into_iter`] and [`RangeSetBlaze::ranges`].
    ///
    /// # Examples
    ///
    /// ```
    /// use range_set_blaze::RangeSetBlaze;
    ///
    /// let mut ranges = RangeSetBlaze::from_iter([10..=20, 15..=25, 30..=40]).into_ranges();
    /// assert_eq!(ranges.next(), Some(10..=25));
    /// assert_eq!(ranges.next(), Some(30..=40));
    /// assert_eq!(ranges.next(), None);
    /// ```
    ///
    /// Values returned by the iterator are returned in ascending order:
    ///
    /// ```
    /// use range_set_blaze::RangeSetBlaze;
    ///
    /// let mut ranges = RangeSetBlaze::from_iter([30..=40, 15..=25, 10..=20]).into_ranges();
    /// assert_eq!(ranges.next(), Some(10..=25));
    /// assert_eq!(ranges.next(), Some(30..=40));
    /// assert_eq!(ranges.next(), None);
    /// ```
    pub fn into_ranges(self) -> IntoRangesIter<T> {
        IntoRangesIter {
            iter: self.btree_map.into_iter(),
        }
    }

    // FUTURE BTreeSet some of these as 'const' but it uses unstable. When stable, add them here and elsewhere.

    /// Returns the number of sorted & disjoint ranges in the set.
    ///
    /// # Example
    ///
    /// ```
    /// use range_set_blaze::RangeSetBlaze;
    ///
    /// // We put in three ranges, but they are not sorted & disjoint.
    /// let set = RangeSetBlaze::from_iter([10..=20, 15..=25, 30..=40]);
    /// // After RangeSetBlaze sorts & 'disjoint's them, we see two ranges.
    /// assert_eq!(set.ranges_len(), 2);
    /// assert_eq!(set.to_string(), "10..=25, 30..=40");
    /// ```
    #[must_use]
    pub fn ranges_len(&self) -> usize {
        self.btree_map.len()
    }

    /// Retains only the elements specified by the predicate.
    ///
    /// In other words, remove all integers `e` for which `f(&e)` returns `false`.
    /// The integer elements are visited in ascending order.
    ///
    /// # Examples
    ///
    /// ```
    /// use range_set_blaze::RangeSetBlaze;
    ///
    /// let mut set = RangeSetBlaze::from_iter([1..=6]);
    /// // Keep only the even numbers.
    /// set.retain(|k| k % 2 == 0);
    /// assert_eq!(set, RangeSetBlaze::from_iter([2, 4, 6]));
    /// ```
    pub fn retain<F>(&mut self, mut f: F)
    where
        F: FnMut(&T) -> bool,
    {
        *self = self.iter().filter(|v| f(v)).collect();
    }
}

// We create a RangeSetBlaze from an iterator of integers or integer ranges by
// 1. turning them into a UnionIter (internally, it collects into intervals and sorts by start).
// 2. Turning the SortedDisjoint into a BTreeMap.
impl<T: Integer> FromIterator<T> for RangeSetBlaze<T> {
    /// Create a [`RangeSetBlaze`] from an iterator of integers. Duplicates and out-of-order elements are fine.
    ///
    /// *For more about constructors and performance, see [`RangeSetBlaze` Constructors](struct.RangeSetBlaze.html#rangesetblaze-constructors).*
    ///
    /// # Examples
    ///
    /// ```
    /// use range_set_blaze::RangeSetBlaze;
    ///
    /// let a0 = RangeSetBlaze::from_iter([3, 2, 1, 100, 1]);
    /// let a1: RangeSetBlaze<i32> = [3, 2, 1, 100, 1].into_iter().collect();
    /// assert!(a0 == a1 && a0.to_string() == "1..=3, 100..=100");
    /// ```
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = T>,
    {
        iter.into_iter().map(|x| x..=x).collect()
    }
}

impl<'a, T: Integer> FromIterator<&'a T> for RangeSetBlaze<T> {
    /// Create a [`RangeSetBlaze`] from an iterator of integers references. Duplicates and out-of-order elements are fine.
    ///
    /// *For more about constructors and performance, see [`RangeSetBlaze` Constructors](struct.RangeSetBlaze.html#rangesetblaze-constructors).*
    ///
    /// # Examples
    ///
    /// ```
    /// use range_set_blaze::RangeSetBlaze;
    ///
    /// let a0 = RangeSetBlaze::from_iter(vec![3, 2, 1, 100, 1]);
    /// let a1: RangeSetBlaze<i32> = vec![3, 2, 1, 100, 1].into_iter().collect();
    /// assert!(a0 == a1 && a0.to_string() == "1..=3, 100..=100");
    /// ```
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = &'a T>,
    {
        iter.into_iter().map(|x| *x..=*x).collect()
    }
}

impl<T: Integer> FromIterator<RangeInclusive<T>> for RangeSetBlaze<T> {
    /// Create a [`RangeSetBlaze`] from an iterator of inclusive ranges, `start..=end`.
    /// Overlapping, out-of-order, and empty ranges are fine.
    ///
    /// *For more about constructors and performance, see [`RangeSetBlaze` Constructors](struct.RangeSetBlaze.html#rangesetblaze-constructors).*
    ///
    /// # Examples
    ///
    /// ```
    /// use range_set_blaze::RangeSetBlaze;
    ///
    /// #[allow(clippy::reversed_empty_ranges)]
    /// let a0 = RangeSetBlaze::from_iter([1..=2, 2..=2, -10..=-5, 1..=0]);
    /// #[allow(clippy::reversed_empty_ranges)]
    /// let a1: RangeSetBlaze<i32> = [1..=2, 2..=2, -10..=-5, 1..=0].into_iter().collect();
    /// assert!(a0 == a1 && a0.to_string() == "-10..=-5, 1..=2");
    /// ```
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = RangeInclusive<T>>,
    {
        let union_iter: UnionIter<T, _> = iter.into_iter().collect();
        RangeSetBlaze::from_sorted_disjoint(union_iter)
    }
}

impl<'a, T: Integer + 'a> FromIterator<&'a RangeInclusive<T>> for RangeSetBlaze<T> {
    /// Create a [`RangeSetBlaze`] from an iterator of inclusive ranges, `start..=end`.
    /// Overlapping, out-of-order, and empty ranges are fine.
    ///
    /// *For more about constructors and performance, see [`RangeSetBlaze` Constructors](struct.RangeSetBlaze.html#rangesetblaze-constructors).*
    ///
    /// # Examples
    ///
    /// ```
    /// use range_set_blaze::RangeSetBlaze;
    ///
    /// #[allow(clippy::reversed_empty_ranges)]
    /// let vec_range = vec![1..=2, 2..=2, -10..=-5, 1..=0];
    /// let a0 = RangeSetBlaze::from_iter(vec_range.iter());
    /// let a1: RangeSetBlaze<i32> = vec_range.iter().collect();
    /// assert!(a0 == a1 && a0.to_string() == "-10..=-5, 1..=2");
    /// ```
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = &'a RangeInclusive<T>>,
    {
        let union_iter: UnionIter<T, _> = iter.into_iter().cloned().collect();
        RangeSetBlaze::from_sorted_disjoint(union_iter)
    }
}
impl<T: Integer, const N: usize> From<[T; N]> for RangeSetBlaze<T> {
    /// For compatibility with [`BTreeSet`] you may create a [`RangeSetBlaze`] from an array of integers.
    ///
    /// *For more about constructors and performance, see [`RangeSetBlaze` Constructors](struct.RangeSetBlaze.html#rangesetblaze-constructors).*
    ///
    /// [`BTreeSet`]: alloc::collections::BTreeSet
    ///
    /// # Examples
    ///
    /// ```
    /// use range_set_blaze::RangeSetBlaze;
    ///
    /// let a0 = RangeSetBlaze::from([3, 2, 1, 100, 1]);
    /// let a1: RangeSetBlaze<i32> = [3, 2, 1, 100, 1].into();
    /// assert!(a0 == a1 && a0.to_string() == "1..=3, 100..=100")
    /// ```
    fn from(arr: [T; N]) -> Self {
        arr.into_iter().collect()
    }
}

#[doc(hidden)]
pub type BitOrMerge<T, L, R> = UnionIter<T, Merge<T, L, R>>;
#[doc(hidden)]
pub type BitOrKMerge<T, I> = UnionIter<T, KMerge<T, I>>;
#[doc(hidden)]
pub type BitAndMerge<T, L, R> = NotIter<T, BitNandMerge<T, L, R>>;
#[doc(hidden)]
pub type BitAndKMerge<T, I> = NotIter<T, BitNandKMerge<T, I>>;
#[doc(hidden)]
pub type BitNandMerge<T, L, R> = BitOrMerge<T, NotIter<T, L>, NotIter<T, R>>;
#[doc(hidden)]
pub type BitNandKMerge<T, I> = BitOrKMerge<T, NotIter<T, I>>;
#[doc(hidden)]
pub type BitNorMerge<T, L, R> = NotIter<T, BitOrMerge<T, L, R>>;
#[doc(hidden)]
pub type BitSubMerge<T, L, R> = NotIter<T, BitOrMerge<T, NotIter<T, L>, R>>;
#[doc(hidden)]
pub type BitXOrTee<T, L, R> =
    BitOrMerge<T, BitSubMerge<T, Tee<L>, Tee<R>>, BitSubMerge<T, Tee<R>, Tee<L>>>;
#[doc(hidden)]
pub type BitXOr<T, L, R> = BitOrMerge<T, BitSubMerge<T, L, Tee<R>>, BitSubMerge<T, Tee<R>, L>>;
#[doc(hidden)]
pub type BitEq<T, L, R> = BitOrMerge<
    T,
    NotIter<T, BitOrMerge<T, NotIter<T, Tee<L>>, NotIter<T, Tee<R>>>>,
    NotIter<T, BitOrMerge<T, Tee<L>, Tee<R>>>,
>;

impl<T, I> MultiwayRangeSetBlazeRef<T> for I
where
    T: Integer,
    I: IntoIterator<Item = RangeSetBlaze<T>>,
{
}

/// The trait used to provide methods on multiple [`RangeSetBlaze`] references,
/// specifically [`union`] and [`intersection`].
///
/// Also see [`MultiwayRangeSetBlaze`].
///
/// [`union`]: MultiwayRangeSetBlazeRef::union
/// [`intersection`]: MultiwayRangeSetBlazeRef::intersection
pub trait MultiwayRangeSetBlazeRef<T: Integer>:
    IntoIterator<Item = RangeSetBlaze<T>> + Sized
{
    /// Unions the given [`RangeSetBlaze`] references, creating a new [`RangeSetBlaze`].
    /// Any number of input can be given.
    ///
    /// For exactly two inputs, you can also use the '|' operator.
    /// Also see [`MultiwayRangeSetBlaze::union`].
    ///
    /// # Performance
    ///
    ///  All work is done on demand, in one pass through the inputs. Minimal memory is used.
    ///
    /// # Example
    ///
    /// Find the integers that appear in any of the [`RangeSetBlaze`]'s.
    ///
    /// ```
    /// use range_set_blaze::prelude::*;
    ///
    /// let a = RangeSetBlaze::from_iter([1..=6, 8..=9, 11..=15]);
    /// let b = RangeSetBlaze::from_iter([5..=13, 18..=29]);
    /// let c = RangeSetBlaze::from_iter([25..=100]);
    ///
    /// let union = vec![a, b, c].into_iter().union();
    ///
    /// assert_eq!(union, RangeSetBlaze::from_iter([1..=15, 18..=100]));
    /// ```
    fn union(self) -> RangeSetBlaze<T> {
        RangeSetBlaze::from_sorted_disjoint(self.into_iter().map(|x| x.into_ranges()).union())
    }

    /// Intersects the given [`RangeSetBlaze`] references, creating a new [`RangeSetBlaze`].
    /// Any number of input can be given.
    ///
    /// For exactly two inputs, you can also use the '&' operator.
    /// Also see [`MultiwayRangeSetBlaze::intersection`].
    ///
    /// # Performance
    ///
    ///  All work is done on demand, in one pass through the inputs. Minimal memory is used.
    ///
    /// # Example
    ///
    /// Find the integers that appear in all the [`RangeSetBlaze`]'s.
    ///
    /// ```
    /// use range_set_blaze::prelude::*;
    ///
    /// let a = RangeSetBlaze::from_iter([1..=6, 8..=9, 11..=15]);
    /// let b = RangeSetBlaze::from_iter([5..=13, 18..=29]);
    /// let c = RangeSetBlaze::from_iter([-100..=100]);
    ///
    /// let intersection = vec![a, b, c].into_iter().intersection();
    ///
    /// assert_eq!(intersection, RangeSetBlaze::from_iter([5..=6, 8..=9, 11..=13]));
    /// ```
    fn intersection(self) -> RangeSetBlaze<T> {
        self.into_iter()
            .map(RangeSetBlaze::into_ranges)
            .intersection()
            .into_range_set_blaze()
    }
}
impl<'a, T, I> MultiwayRangeSetBlaze<'a, T> for I
where
    T: Integer + 'a,
    I: IntoIterator<Item = &'a RangeSetBlaze<T>>,
{
}
/// The trait used to provide methods on multiple [`RangeSetBlaze`]'s,
/// specifically [`union`] and [`intersection`].
///
/// Also see [`MultiwayRangeSetBlazeRef`].
///
/// [`union`]: MultiwayRangeSetBlaze::union
/// [`intersection`]: MultiwayRangeSetBlaze::intersection
pub trait MultiwayRangeSetBlaze<'a, T: Integer + 'a>:
    IntoIterator<Item = &'a RangeSetBlaze<T>> + Sized
{
    /// Unions the given [`RangeSetBlaze`]'s, creating a new [`RangeSetBlaze`].
    /// Any number of input can be given.
    ///
    /// For exactly two inputs, you can also use the '|' operator.
    /// Also see [`MultiwayRangeSetBlazeRef::union`].
    ///
    /// # Performance
    ///
    ///  All work is done on demand, in one pass through the inputs. Minimal memory is used.
    ///
    /// # Example
    ///
    /// Find the integers that appear in any of the [`RangeSetBlaze`]'s.
    ///
    /// ```
    /// use range_set_blaze::prelude::*;
    ///
    /// let a = RangeSetBlaze::from_iter([1..=6, 8..=9, 11..=15]);
    /// let b = RangeSetBlaze::from_iter([5..=13, 18..=29]);
    /// let c = RangeSetBlaze::from_iter([25..=100]);
    ///
    /// let union = [a, b, c].union();
    ///
    /// assert_eq!(union, RangeSetBlaze::from_iter([1..=15, 18..=100]));
    /// ```
    fn union(self) -> RangeSetBlaze<T> {
        self.into_iter()
            .map(RangeSetBlaze::ranges)
            .union()
            .into_range_set_blaze()
    }

    /// Intersects the given [`RangeSetBlaze`]'s, creating a new [`RangeSetBlaze`].
    /// Any number of input can be given.
    ///
    /// For exactly two inputs, you can also use the '&' operator.
    /// Also see [`MultiwayRangeSetBlazeRef::intersection`].
    ///
    /// # Performance
    ///
    ///  All work is done on demand, in one pass through the inputs. Minimal memory is used.
    ///
    /// # Example
    ///
    /// Find the integers that appear in all the [`RangeSetBlaze`]'s.
    ///
    /// ```
    /// use range_set_blaze::prelude::*;
    ///
    /// let a = RangeSetBlaze::from_iter([1..=6, 8..=9, 11..=15]);
    /// let b = RangeSetBlaze::from_iter([5..=13, 18..=29]);
    /// let c = RangeSetBlaze::from_iter([-100..=100]);
    ///
    /// let intersection = [a, b, c].intersection();
    ///
    /// assert_eq!(intersection, RangeSetBlaze::from_iter([5..=6, 8..=9, 11..=13]));
    /// ```
    fn intersection(self) -> RangeSetBlaze<T> {
        self.into_iter()
            .map(RangeSetBlaze::ranges)
            .intersection()
            .into_range_set_blaze()
    }
}

impl<T, II, I> MultiwaySortedDisjoint<T, I> for II
where
    T: Integer,
    I: SortedDisjoint<T>,
    II: IntoIterator<Item = I>,
{
}

/// The trait used to define methods on multiple [`SortedDisjoint`] iterators,
/// specifically [`union`] and [`intersection`].
///
/// [`union`]: crate::MultiwaySortedDisjoint::union
/// [`intersection`]: crate::MultiwaySortedDisjoint::intersection
pub trait MultiwaySortedDisjoint<T: Integer, I>: IntoIterator<Item = I> + Sized
where
    I: SortedDisjoint<T>,
{
    /// Unions the given [`SortedDisjoint`] iterators, creating a new [`SortedDisjoint`] iterator.
    /// The input iterators must be of the same type. Any number of input iterators can be given.
    ///
    /// For input iterators of different types, use the [`union_dyn`] macro.
    ///
    /// # Performance
    ///
    ///  All work is done on demand, in one pass through the input iterators. Minimal memory is used.
    ///
    /// # Example
    ///
    /// Find the integers that appear in any of the [`SortedDisjoint`] iterators.
    ///
    /// ```
    /// use range_set_blaze::prelude::*;
    ///
    /// let a = RangeSetBlaze::from_iter([1..=6, 8..=9, 11..=15]).into_ranges();
    /// let b = RangeSetBlaze::from_iter([5..=13, 18..=29]).into_ranges();
    /// let c = RangeSetBlaze::from_iter([25..=100]).into_ranges();
    ///
    /// let union = [a, b, c].union();
    ///
    /// assert_eq!(union.to_string(), "1..=15, 18..=100");
    /// ```
    fn union(self) -> BitOrKMerge<T, I> {
        UnionIter::new(KMerge::new(self))
    }

    /// Intersects the given [`SortedDisjoint`] iterators, creating a new [`SortedDisjoint`] iterator.
    /// The input iterators must be of the same type. Any number of input iterators can be given.
    ///
    /// For input iterators of different types, use the [`intersection_dyn`] macro.
    ///
    /// # Performance
    ///
    ///  All work is done on demand, in one pass through the input iterators. Minimal memory is used.
    ///
    /// # Example
    ///
    /// Find the integers that appear in all the [`SortedDisjoint`] iterators.
    ///
    /// ```
    /// use range_set_blaze::prelude::*;
    ///
    /// let a = RangeSetBlaze::from_iter([1..=6, 8..=9, 11..=15]).into_ranges();
    /// let b = RangeSetBlaze::from_iter([5..=13, 18..=29]).into_ranges();
    /// let c = RangeSetBlaze::from_iter([-100..=100]).into_ranges();
    ///
    /// let intersection = [a, b, c].intersection();
    ///
    /// assert_eq!(intersection.to_string(), "5..=6, 8..=9, 11..=13");
    /// ```
    fn intersection(self) -> BitAndKMerge<T, I> {
        self.into_iter()
            .map(|seq| seq.into_iter().complement())
            .union()
            .complement()
    }
}

gen_ops_ex!(
    <T>;
    types ref RangeSetBlaze<T>, ref RangeSetBlaze<T> => RangeSetBlaze<T>;


    /// Intersects the contents of two [`RangeSetBlaze`]'s.
    ///
    /// Either, neither, or both inputs may be borrowed.
    ///
    /// # Examples
    /// ```
    /// use range_set_blaze::prelude::*;
    ///
    /// let a = RangeSetBlaze::from_iter([1..=2, 5..=100]);
    /// let b = RangeSetBlaze::from_iter([2..=6]);
    /// let result = &a & &b; // Alternatively, 'a & b'.
    /// assert_eq!(result.to_string(), "2..=2, 5..=6");
    /// ```
    for & call |a: &RangeSetBlaze<T>, b: &RangeSetBlaze<T>| {
        (a.ranges() & b.ranges()).into_range_set_blaze()
    };

    /// Symmetric difference the contents of two [`RangeSetBlaze`]'s.
    ///
    /// Either, neither, or both inputs may be borrowed.
    ///
    /// # Examples
    /// ```
    /// use range_set_blaze::prelude::*;
    ///
    /// let a = RangeSetBlaze::from_iter([1..=2, 5..=100]);
    /// let b = RangeSetBlaze::from_iter([2..=6]);
    /// let result = &a ^ &b; // Alternatively, 'a ^ b'.
    /// assert_eq!(result.to_string(), "1..=1, 3..=4, 7..=100");
    /// ```
    for ^ call |a: &RangeSetBlaze<T>, b: &RangeSetBlaze<T>| {
        // We optimize this by using ranges() twice per input, rather than tee()
        let lhs0 = a.ranges();
        let lhs1 = a.ranges();
        let rhs0 = b.ranges();
        let rhs1 = b.ranges();
        ((lhs0 - rhs0) | (rhs1 - lhs1)).into_range_set_blaze()
    };

    /// Difference the contents of two [`RangeSetBlaze`]'s.
    ///
    /// Either, neither, or both inputs may be borrowed.
    ///
    /// # Examples
    /// ```
    /// use range_set_blaze::prelude::*;
    ///
    /// let a = RangeSetBlaze::from_iter([1..=2, 5..=100]);
    /// let b = RangeSetBlaze::from_iter([2..=6]);
    /// let result = &a - &b; // Alternatively, 'a - b'.
    /// assert_eq!(result.to_string(), "1..=1, 7..=100");
    /// ```
    for - call |a: &RangeSetBlaze<T>, b: &RangeSetBlaze<T>| {
        (a.ranges() - b.ranges()).into_range_set_blaze()
    };
    where T: Integer //Where clause for all impl's
);

gen_ops_ex!(
    <T>;
    types ref RangeSetBlaze<T> => RangeSetBlaze<T>;

    /// Complement the contents of a [`RangeSetBlaze`].
    ///
    /// The input may be borrowed or not.
    ///
    /// # Examples
    /// ```
    /// use range_set_blaze::prelude::*;
    ///
    /// let a = RangeSetBlaze::from_iter([1..=2, 5..=100]);
    /// let result = !&a; // Alternatively, '!a'.
    /// assert_eq!(
    ///     result.to_string(),
    ///     "-2147483648..=0, 3..=4, 101..=2147483647"
    /// );
    /// ```
    for ! call |a: &RangeSetBlaze<T>| {
        (!a.ranges()).into_range_set_blaze()
    };

    where T: Integer //Where clause for all impl's
);

impl<T: Integer> IntoIterator for RangeSetBlaze<T> {
    type Item = T;
    type IntoIter = IntoIter<T>;

    /// Gets an iterator for moving out the [`RangeSetBlaze`]'s integer contents.
    ///
    /// # Examples
    ///
    /// ```
    /// use range_set_blaze::RangeSetBlaze;
    ///
    /// let set = RangeSetBlaze::from_iter([1, 2, 3, 4]);
    ///
    /// let v: Vec<_> = set.into_iter().collect();
    /// assert_eq!(v, [1, 2, 3, 4]);
    /// ```
    fn into_iter(self) -> IntoIter<T> {
        IntoIter {
            option_range: None,
            into_iter: self.btree_map.into_iter(),
        }
    }
}

/// An iterator over the integer elements of a [`RangeSetBlaze`].
///
/// This `struct` is created by the [`iter`] method on [`RangeSetBlaze`]. See its
/// documentation for more.
///
/// [`iter`]: RangeSetBlaze::iter
#[must_use = "iterators are lazy and do nothing unless consumed"]
#[derive(Clone, Debug)]
pub struct Iter<T, I>
where
    T: Integer,
    I: SortedDisjoint<T>,
{
    iter: I,
    option_range: Option<RangeInclusive<T>>,
}

impl<T: Integer, I> FusedIterator for Iter<T, I> where I: SortedDisjoint<T> + FusedIterator {}

impl<T: Integer, I> Iterator for Iter<T, I>
where
    I: SortedDisjoint<T>,
{
    type Item = T;
    fn next(&mut self) -> Option<T> {
        let range = self.option_range.take().or_else(|| self.iter.next())?;
        let (start, end) = range.into_inner();
        debug_assert!(start <= end && end <= T::safe_max_value());
        if start < end {
            self.option_range = Some(start + T::one()..=end);
        }
        Some(start)
    }

    // We'll have at least as many integers as intervals. There could be more that usize MAX
    // The option_range field could increase the number of integers, but we can ignore that.
    fn size_hint(&self) -> (usize, Option<usize>) {
        let (low, _high) = self.iter.size_hint();
        (low, None)
    }
}

#[must_use = "iterators are lazy and do nothing unless consumed"]
#[derive(Debug)]
/// An iterator over the integer elements of a [`RangeSetBlaze`].
///
/// This `struct` is created by the [`into_iter`] method on [`RangeSetBlaze`]. See its
/// documentation for more.
///
/// [`into_iter`]: RangeSetBlaze::into_iter
pub struct IntoIter<T: Integer> {
    option_range: Option<RangeInclusive<T>>,
    into_iter: alloc::collections::btree_map::IntoIter<T, T>,
}

impl<T: Integer> FusedIterator for IntoIter<T> {}

impl<T: Integer> Iterator for IntoIter<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        let range = self
            .option_range
            .take()
            .or_else(|| self.into_iter.next().map(|(start, end)| start..=end))?;
        let (start, end) = range.into_inner();
        debug_assert!(start <= end && end <= T::safe_max_value());
        if start < end {
            self.option_range = Some(start + T::one()..=end);
        }
        Some(start)
    }

    // We'll have at least as many integers as intervals. There could be more that usize MAX
    // the option_range field could increase the number of integers, but we can ignore that.
    fn size_hint(&self) -> (usize, Option<usize>) {
        let (low, _high) = self.into_iter.size_hint();
        (low, None)
    }
}

impl<T: Integer> Extend<T> for RangeSetBlaze<T> {
    /// Extends the [`RangeSetBlaze`] with the contents of an Integer iterator.
    ///
    /// Integers are added one-by-one. There is also a version
    /// that takes a range iterator.
    ///
    /// The [`|=`](RangeSetBlaze::bitor_assign) operator extends a [`RangeSetBlaze`]
    /// from another [`RangeSetBlaze`]. It is never slower
    /// than  [`RangeSetBlaze::extend`] and often several times faster.
    ///
    /// # Examples
    /// ```
    /// use range_set_blaze::RangeSetBlaze;
    /// let mut a = RangeSetBlaze::from_iter([1..=4]);
    /// a.extend([5, 0, 0, 3, 4, 10]);
    /// assert_eq!(a, RangeSetBlaze::from_iter([0..=5, 10..=10]));
    ///
    /// let mut a = RangeSetBlaze::from_iter([1..=4]);
    /// let mut b = RangeSetBlaze::from_iter([5, 0, 0, 3, 4, 10]);
    /// a |= b;
    /// assert_eq!(a, RangeSetBlaze::from_iter([0..=5, 10..=10]));
    /// ```
    fn extend<I>(&mut self, iter: I)
    where
        I: IntoIterator<Item = T>,
    {
        let iter = iter.into_iter();
        for range in UnsortedDisjoint::from(iter.map(|x| x..=x)) {
            self.internal_add(range);
        }
    }
}

impl<T: Integer> BitOrAssign<&RangeSetBlaze<T>> for RangeSetBlaze<T> {
    /// Adds the contents of another [`RangeSetBlaze`] to this one.
    ///
    /// Passing the right-hand side by ownership rather than borrow
    /// will allow a many-times faster speed up when the
    /// right-hand side is much larger than the left-hand side.
    ///
    /// Also, this operation is never slower than [`RangeSetBlaze::extend`] and
    /// can often be many times faster.
    ///
    /// # Examples
    /// ```
    /// use range_set_blaze::RangeSetBlaze;
    /// let mut a = RangeSetBlaze::from_iter([1..=4]);
    /// let mut b = RangeSetBlaze::from_iter([0..=0,3..=5,10..=10]);
    /// a |= &b;
    /// assert_eq!(a, RangeSetBlaze::from_iter([0..=5, 10..=10]));
    /// ```
    fn bitor_assign(&mut self, other: &Self) {
        let a_len = self.ranges_len();
        if a_len == 0 {
            *self = other.clone();
            return;
        }
        let b_len = other.ranges_len();
        if b_len * (a_len.ilog2() as usize + 1) < a_len + b_len {
            self.extend(other.ranges());
        } else {
            *self = (self.ranges() | other.ranges()).into_range_set_blaze();
        }
    }
}

impl<T: Integer> BitOrAssign<RangeSetBlaze<T>> for RangeSetBlaze<T> {
    /// Adds the contents of another [`RangeSetBlaze`] to this one.
    ///
    /// Passing the right-hand side by ownership rather than borrow
    /// will allow a many-times faster speed up when the
    /// right-hand side is much larger than the left-hand side.
    ///
    /// Also, this operation is never slower than [`RangeSetBlaze::extend`] and
    /// can often be many times faster.
    ///
    ///
    /// # Examples
    /// ```
    /// use range_set_blaze::RangeSetBlaze;
    /// let mut a = RangeSetBlaze::from_iter([1..=4]);
    /// let mut b = RangeSetBlaze::from_iter([0..=0,3..=5,10..=10]);
    /// a |= b;
    /// assert_eq!(a, RangeSetBlaze::from_iter([0..=5, 10..=10]));
    /// ```
    fn bitor_assign(&mut self, mut other: Self) {
        let a_len = self.ranges_len();
        let b_len = other.ranges_len();
        if b_len <= a_len {
            *self |= &other;
        } else {
            other |= &*self;
            *self = other;
        }
    }
}

impl<T: Integer> BitOr<RangeSetBlaze<T>> for RangeSetBlaze<T> {
    /// Unions the contents of two [`RangeSetBlaze`]'s.
    ///
    /// Passing ownership rather than borrow sometimes allows a many-times
    /// faster speed up.
    ///
    /// # Examples
    /// ```
    /// use range_set_blaze::RangeSetBlaze;
    /// let a = RangeSetBlaze::from_iter([1..=4]);
    /// let b = RangeSetBlaze::from_iter([0..=0, 3..=5, 10..=10]);
    /// let union = a | b;
    /// assert_eq!(union, RangeSetBlaze::from_iter([0..=5, 10..=10]));
    /// ```
    type Output = RangeSetBlaze<T>;
    fn bitor(mut self, other: Self) -> RangeSetBlaze<T> {
        self |= other;
        self
    }
}

impl<T: Integer> BitOr<&RangeSetBlaze<T>> for RangeSetBlaze<T> {
    /// Unions the contents of two [`RangeSetBlaze`]'s.
    ///
    /// Passing ownership rather than borrow sometimes allows a many-times
    /// faster speed up.
    ///
    /// # Examples
    /// ```
    /// use range_set_blaze::RangeSetBlaze;
    /// let mut a = RangeSetBlaze::from_iter([1..=4]);
    /// let mut b = RangeSetBlaze::from_iter([0..=0,3..=5,10..=10]);
    /// let union = a | &b;
    /// assert_eq!(union, RangeSetBlaze::from_iter([0..=5, 10..=10]));
    /// ```
    type Output = RangeSetBlaze<T>;
    fn bitor(mut self, other: &Self) -> RangeSetBlaze<T> {
        self |= other;
        self
    }
}

impl<T: Integer> BitOr<RangeSetBlaze<T>> for &RangeSetBlaze<T> {
    type Output = RangeSetBlaze<T>;
    /// Unions the contents of two [`RangeSetBlaze`]'s.
    ///
    /// Passing ownership rather than borrow sometimes allows a many-times
    /// faster speed up.
    ///
    /// # Examples
    /// ```
    /// use range_set_blaze::RangeSetBlaze;
    /// let mut a = RangeSetBlaze::from_iter([1..=4]);
    /// let mut b = RangeSetBlaze::from_iter([0..=0,3..=5,10..=10]);
    /// let union = &a | b;
    /// assert_eq!(union, RangeSetBlaze::from_iter([0..=5, 10..=10]));
    /// ```
    fn bitor(self, mut other: RangeSetBlaze<T>) -> RangeSetBlaze<T> {
        other |= self;
        other
    }
}

impl<T: Integer> BitOr<&RangeSetBlaze<T>> for &RangeSetBlaze<T> {
    type Output = RangeSetBlaze<T>;
    /// Unions the contents of two [`RangeSetBlaze`]'s.
    ///
    /// Passing ownership rather than borrow sometimes allows a many-times
    /// faster speed up.
    ///
    /// # Examples
    /// ```
    /// use range_set_blaze::RangeSetBlaze;
    /// let mut a = RangeSetBlaze::from_iter([1..=4]);
    /// let mut b = RangeSetBlaze::from_iter([0..=0,3..=5,10..=10]);
    /// let union = &a | &b;
    /// assert_eq!(union, RangeSetBlaze::from_iter([0..=5, 10..=10]));
    /// ```
    fn bitor(self, other: &RangeSetBlaze<T>) -> RangeSetBlaze<T> {
        (self.ranges() | other.ranges()).into_range_set_blaze()
    }
}

impl<T: Integer> Extend<RangeInclusive<T>> for RangeSetBlaze<T> {
    /// Extends the [`RangeSetBlaze`] with the contents of a
    /// range iterator.

    /// Elements are added one-by-one. There is also a version
    /// that takes an integer iterator.
    ///
    /// The [`|=`](RangeSetBlaze::bitor_assign) operator extends a [`RangeSetBlaze`]
    /// from another [`RangeSetBlaze`]. It is never slower
    ///  than  [`RangeSetBlaze::extend`] and often several times faster.
    ///
    /// # Examples
    /// ```
    /// use range_set_blaze::RangeSetBlaze;
    /// let mut a = RangeSetBlaze::from_iter([1..=4]);
    /// a.extend([5..=5, 0..=0, 0..=0, 3..=4, 10..=10]);
    /// assert_eq!(a, RangeSetBlaze::from_iter([0..=5, 10..=10]));
    ///
    /// let mut a = RangeSetBlaze::from_iter([1..=4]);
    /// let mut b = RangeSetBlaze::from_iter([5..=5, 0..=0, 0..=0, 3..=4, 10..=10]);
    /// a |= b;
    /// assert_eq!(a, RangeSetBlaze::from_iter([0..=5, 10..=10]));
    /// ```
    fn extend<I>(&mut self, iter: I)
    where
        I: IntoIterator<Item = RangeInclusive<T>>,
    {
        let iter = iter.into_iter();
        for range in iter {
            self.internal_add(range);
        }
    }
}

impl<T: Integer> Ord for RangeSetBlaze<T> {
    /// We define a total ordering on RangeSetBlaze. Following the convention of
    /// [`BTreeSet`], the ordering is lexicographic, *not* by subset/superset.
    ///
    /// [`BTreeSet`]: alloc::collections::BTreeSet
    ///
    /// # Examples
    /// ```
    /// use range_set_blaze::RangeSetBlaze;
    ///
    /// let a = RangeSetBlaze::from_iter([1..=3, 5..=7]);
    /// let b = RangeSetBlaze::from_iter([2..=2]);
    /// assert!(a < b); // Lexicographic comparison
    /// assert!(b.is_subset(&a)); // Subset comparison
    /// // More lexicographic comparisons
    /// assert!(a <= b);
    /// assert!(b > a);
    /// assert!(b >= a);
    /// assert!(a != b);
    /// assert!(a == a);
    /// use core::cmp::Ordering;
    /// assert_eq!(a.cmp(&b), Ordering::Less);
    /// assert_eq!(a.partial_cmp(&b), Some(Ordering::Less));
    /// ```

    #[inline]
    fn cmp(&self, other: &RangeSetBlaze<T>) -> Ordering {
        // slow one by one: return self.iter().cmp(other.iter());

        // fast by ranges:
        let mut a = self.ranges();
        let mut b = other.ranges();
        let mut a_rx = a.next();
        let mut b_rx = b.next();
        loop {
            match (a_rx.clone(), b_rx.clone()) {
                (Some(a_r), Some(b_r)) => {
                    let cmp_start = a_r.start().cmp(b_r.start());
                    if cmp_start != Ordering::Equal {
                        return cmp_start;
                    }
                    let cmp_end = a_r.end().cmp(b_r.end());
                    match cmp_end {
                        Ordering::Equal => {
                            a_rx = a.next();
                            b_rx = b.next();
                        }
                        Ordering::Less => {
                            a_rx = a.next();
                            b_rx = Some(*a_r.end() + T::one()..=*b_r.end());
                        }
                        Ordering::Greater => {
                            a_rx = Some(*b_r.end() + T::one()..=*a_r.end());
                            b_rx = b.next();
                        }
                    }
                }
                (Some(_), None) => return Ordering::Greater,
                (None, Some(_)) => return Ordering::Less,
                (None, None) => return Ordering::Equal,
            }
        }
    }
}

impl<T: Integer> PartialOrd for RangeSetBlaze<T> {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<T: Integer> Eq for RangeSetBlaze<T> {}

// If the iterator inside a BitOrIter is SortedStart, the output will be SortedDisjoint
impl<T: Integer, I: SortedStarts<T>> SortedStarts<T> for UnionIter<T, I> {}
impl<T: Integer, I: SortedStarts<T>> SortedDisjoint<T> for UnionIter<T, I> {}
// If the iterator inside NotIter is SortedDisjoint, the output will be SortedDisjoint
impl<T: Integer, I: SortedDisjoint<T>> SortedStarts<T> for NotIter<T, I> {}
impl<T: Integer, I: SortedDisjoint<T>> SortedDisjoint<T> for NotIter<T, I> {}
// If the iterator inside Tee is SortedDisjoint, the output will be SortedDisjoint
impl<T: Integer, I: SortedDisjoint<T>> SortedStarts<T> for Tee<I> {}
impl<T: Integer, I: SortedDisjoint<T>> SortedDisjoint<T> for Tee<I> {}

#[cfg(feature = "std")]
use std::{
    fs::File,
    io::{self, BufRead, BufReader},
    path::Path,
};

#[cfg(feature = "std")]
#[allow(missing_docs)]
pub fn demo_read_ranges_from_file<P, T>(path: P) -> io::Result<RangeSetBlaze<T>>
where
    P: AsRef<Path>,
    T: FromStr + Integer,
{
    let lines = BufReader::new(File::open(&path)?).lines();

    let mut set = RangeSetBlaze::new();
    for line in lines {
        let line = line?;
        let mut split = line.split('\t');
        let start = split
            .next()
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "Missing start of range"))?
            .parse::<T>()
            .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "Invalid start of range"))?;
        let end = split
            .next()
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "Missing end of range"))?
            .parse::<T>()
            .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "Invalid end of range"))?;
        set.ranges_insert(start..=end);
    }

    Ok(set)
}

// FUTURE: use fn range to implement one-at-a-time intersection, difference, etc. and then add more inplace ops.
// cmk Rule: Explain there is a great .as_simd method that we are not using because we want to code
// cmk Rule: to also work without SIMD. Also, getting generic constants to work is a pain/impossible.
// cmk Rule: Can you use SIMD Rust methods without SIMD?
// cmk Rule: Create a .cargo/config.toml, but think about what you check in.
// cmk Rule: spot test and benchmark
// cmk Rule: Look at assembly. Use AI tools to understand it (but they may not be perfect)
// cmk Thanks to B3NNY (or name) at the Seattle Rust Meetup for pointing me in the right direction on SIMD.")
// cmk Understand Benny's use of godbolt: https://godbolt.org/z/3P3jGej3h
// cmk Rule: Use Godbolt.
