#![doc = include_str!("../README.md")]
#![warn(missing_docs)]

// !!!cmk Implement default and other traits mentioned in the video.
// !!!cmk0000 For RangeSetInt make table of constructor and set operations.
// !!!cmk0000 For SortedDisjoint make table of constructor and set operations.
// !!!cmk0doc give a link to RangeSetInt struct at top of the docs.

// https://docs.rs/range_bounds_map/latest/range_bounds_map/range_bounds_set/struct.RangeBoundsSet.html
// Here are some relevant crates I found whilst searching around the topic area:

// https://crates.io/crates/sorted-iter
//    cmk0 Look at sorted-iter's note about exporting.
//    cmk0 Look at sorted-iter's note about their testing tool.
// https://docs.rs/rangemap Very similar to this crate but can only use RangesIter and RangeInclusives as keys in it's map and set structs (separately).
// https://docs.rs/btree-range-map
// https://docs.rs/ranges Cool library for fully-generic ranges (unlike std::ops ranges), along with a RangesIter data structure for storing them (Vec-based unfortunately)
// https://docs.rs/intervaltree Allows overlapping intervals but is immutable unfortunately
// https://docs.rs/nonoverlapping_interval_tree Very similar to rangemap except without a gaps() function and only for RangesIter and not RangeInclusives. And also no fancy coalescing functions.
// https://docs.rs/unbounded-interval-tree A data structure based off of a 2007 published paper! It supports any RangeBounds as keys too, except it is implemented with a non-balancing Box<Node> based tree, however it also supports overlapping RangeBounds which my library does not.
// https://docs.rs/rangetree I'm not entirely sure what this library is or isn't, but it looks like a custom red-black tree/BTree implementation used specifically for a Range Tree. Interesting but also quite old (5 years) and uses unsafe.
// https://docs.rs/btree-range-map/latest/btree_range_map/
// Related: https://lib.rs/crates/iset
// https://lib.rs/crates/interval_tree
// https://lib.rs/crates/range-set
// https://lib.rs/crates/rangemap
// https://lib.rs/crates/ranges
// https://lib.rs/crates/nonoverlapping_interval_tree
// https://stackoverflow.com/questions/30540766/how-can-i-add-new-methods-to-iterator
// !!!cmk0 how could you write your own subtraction that subtracted many sets from one set via iterators?
// cmk rules: When should use Iterator and when IntoIterator?
// cmk rules: When should use: from_iter, from, new from_something?
// !!! cmk rule: Don't have a function and a method. Pick one (method)
// !!!cmk rule: Follow the rules of good API design including accepting almost any type of input
// cmk rule: don't create an assign method if it is not more efficient
// cmk rule: generate HTML: criterion = { version = "0.4", features = ["html_reports"] }
// cmk rule: pick another similar data structure and implement everything that makes sense (copy docs as much as possible)
// cmk000 write a straight-forward README.md by merging the Python range_int_set and Rust anytype
// cmk000 finish constructor list
// cmk0doc add documentation
// cmk000 finish the benchmark story
// cmk0doc in docs, be sure `bitor` is a live link to the bitor method
// cmk rule: define and understand PartialOrd, Ord, Eq, etc.
// cmk0doc document that we implement most methods of BTreeSet -- exceptions 'range', drain_filter & new_in (nightly-only). Also, it's iter is a double-ended iterator. and ours is not.
// cmk implement "ranges" by using log n search and then SortedDisjoint intersection.
// cmk000 is 'ranges' a good name for the function or confusing with btreeset's 'range'?

// FUTURE: Support serde via optional feature
mod integer;
mod merge;
mod not_iter;
mod ranges;
mod sorted_disjoint;
mod sorted_disjoint_iterator;
mod tests;
mod union_iter;
mod unsorted_disjoint;

use gen_ops::gen_ops_ex;
use itertools::Tee;
pub use merge::KMerge;
pub use merge::Merge;
pub use not_iter::NotIter;
use num_traits::ops::overflowing::OverflowingSub;
use num_traits::One;
use num_traits::Zero;
use rand::distributions::uniform::SampleUniform;
pub use ranges::IntoRangesIter;
pub use ranges::RangesIter;
pub use sorted_disjoint::{CheckSortedDisjoint, DynSortedDisjoint};
pub use sorted_disjoint_iterator::SortedDisjointIterator;
use std::cmp::max;
use std::cmp::Ordering;
use std::collections::BTreeMap;
use std::convert::From;
use std::fmt;
use std::ops;
use std::ops::RangeInclusive;
use std::str::FromStr;
pub use union_iter::UnionIter;
pub use unsorted_disjoint::AssumeSortedStarts;
use unsorted_disjoint::SortedDisjointWithLenSoFar;
use unsorted_disjoint::UnsortedDisjoint;

// cmk rule: Support Send and Sync (what about Clone (Copy?) and ExactSizeIterator?)
// cmk rule: Test Send and Sync with a test (see example)

// cmk rule: Define your element type
/// The element trait of the [`RangeSetInt`] and [`SortedDisjoint`], specifically `u8` to `u128` (including `usize`) and `i8` to `i128` (including `isize`).
pub trait Integer:
    num_integer::Integer
    + FromStr
    + fmt::Display
    + fmt::Debug
    + std::iter::Sum
    + num_traits::NumAssignOps
    + FromStr
    + Copy
    + num_traits::Bounded
    + num_traits::NumCast
    + Send
    + Sync
    + OverflowingSub
    + SampleUniform
{
    /// The type of the length of a [`RangeSetInt`]. For example, the length of a `RangeSetInt<u8>` is `usize`. Note
    /// that it can't be `u8` because the length ranges from 0 to 256, which is one too large for `u8`.
    ///
    /// In general, `SafeLen` will be `usize` if `usize` is always large enough. If not, `SafeLen` will be the smallest unsigned integer
    /// type that is always large enough. However, for `u128` and `i128`, nothing is always large enough so
    ///  `SafeLen` will be `u128` and we prohibit the largest value from being used in [`Integer`].
    ///
    /// # Examples
    /// ```
    /// use range_set_int::{RangeSetInt, Integer};
    ///
    /// let len: <u8 as Integer>::SafeLen = RangeSetInt::from_iter([0u8..=255]).len();
    /// assert_eq!(len, 256);
    /// ```
    type SafeLen: std::hash::Hash
        + num_integer::Integer
        + num_traits::NumAssignOps
        + num_traits::Bounded
        + num_traits::NumCast
        + num_traits::One
        + std::ops::AddAssign
        + std::ops::SubAssign
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
    /// #Example
    /// ```
    /// use range_set_int::Integer;
    ///
    /// assert_eq!(<u8 as Integer>::safe_len(&(0..=255)), 256);
    /// ```
    fn safe_len(range: &RangeInclusive<Self>) -> <Self as Integer>::SafeLen;

    /// For a given `Integer` type, returns the largest value that can be used. For all types other than `u128` and `i128`,
    /// this is the same as `Self::MAX`. For `u128` and `i128`, this is one less than `Self::MAX`.
    ///
    /// # Example
    /// ```
    /// use range_set_int::{Integer, RangeSetInt};
    ///
    /// // for i8, we can use up to 127
    /// let a = RangeSetInt::from_iter([i8::MAX]);
    /// // for i128, we can use up to 170141183460469231731687303715884105726
    /// let a = RangeSetInt::from_iter([<i128 as Integer>::safe_max_value()]);
    /// ```
    /// # Panics
    /// ```should_panic
    /// use range_set_int::{Integer, RangeSetInt};
    ///
    /// // for i128, using 170141183460469231731687303715884105727 throws a panic.
    /// let a = RangeSetInt::from_iter([i128::MAX]);
    /// ```
    fn safe_max_value() -> Self {
        Self::max_value()
    }

    // !!!cmk we should define .len() SortedDisjoint

    /// Converts a `f64` to [`Integer`] using the formula `f as Self`. For large integer types, this will result in a loss of precision.
    fn f64_to_t(f: f64) -> Self;

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
/// Internally, it uses a cache-efficient [`BTreeMap`] to store the ranges.
///
/// # Table of Contents
/// * [Constructors](#constructors)
///    * [Examples](struct.RangeSetInt.html#constructor-examples)
/// * [Set Operations](#set-operations)
///    * [Performance](struct.RangeSetInt.html#set-operation-performance)
///    * [Examples](struct.RangeSetInt.html#set-operation-examples)
///
/// # Constructors
///
/// | Methods           | Input                   |
/// |-------------------|-------------------------|
/// | [`new`]/[`default`]       |                         |
/// | [`from_iter`][1]/[`collect`][1] | integer iterator        |
/// | [`from_iter`][2]/[`collect`][2] | ranges iterator         |
/// | [`from`][3] /[`into`][3]         | [`SortedDisjoint`] iterator |
/// | [`from`][4] /[`into`][4]         | array of integers       |
///
/// [`BTreeMap`]: std::collections::BTreeMap
/// [`new`]: RangeSetInt::new
/// [`default`]: RangeSetInt::default
/// [1]: struct.RangeSetInt.html#impl-FromIterator<T>-for-RangeSetInt<T>
/// [2]: struct.RangeSetInt.html#impl-FromIterator<RangeInclusive<T>>-for-RangeSetInt<T>
/// [3]: struct.RangeSetInt.html#impl-From<I>-for-RangeSetInt<T>
/// [4]: struct.RangeSetInt.html#impl-From<%5BT%3B%20N%5D>-for-RangeSetInt<T>
///
/// ## Constructor Examples
///
/// ```
/// use range_set_int::{RangeSetInt, CheckSortedDisjoint};
///
/// // Create an empty set with 'new' or 'default'.
/// let a0 = RangeSetInt::<i32>::new();
/// let a1 = RangeSetInt::<i32>::default();
/// assert!(a0 == a1 && a0.is_empty());
///
/// // 'from_iter'/'collect': From an iterator of integers.
/// // Duplicates and out-of-order elements are fine.
/// let a0 = RangeSetInt::from_iter([3, 2, 1, 100, 1]);
/// let a1: RangeSetInt<i32> = [3, 2, 1, 100, 1].into_iter().collect();
/// assert!(a0 == a1 && a0.to_string() == "1..=3, 100..=100");
///
/// // 'from_iter'/'collect': From an iterator of inclusive ranges, start..=end.
/// // Overlapping, out-of-order, and empty ranges are fine.
/// #[allow(clippy::reversed_empty_ranges)]
/// let a0 = RangeSetInt::from_iter([1..=2, 2..=2, -10..=-5, 1..=0]);
/// #[allow(clippy::reversed_empty_ranges)]
/// let a1: RangeSetInt<i32> = [1..=2, 2..=2, -10..=-5, 1..=0].into_iter().collect();
/// assert!(a0 == a1 && a0.to_string() == "-10..=-5, 1..=2");
///
/// // If we know the ranges are sorted and disjoint, we can use 'from'/'into'.
/// let a0 = RangeSetInt::from(CheckSortedDisjoint::new([-10..=-5, 1..=2].into_iter()));
/// let a1: RangeSetInt<i32> = CheckSortedDisjoint::new([-10..=-5, 1..=2].into_iter()).into();
/// assert!(a0 == a1 && a0.to_string() == "-10..=-5, 1..=2");
///
/// // For compatibility with `BTreeSet`, we also support
/// // 'from'/'into' from arrays of integers.
/// let a0 = RangeSetInt::from([3, 2, 1, 100, 1]);
/// let a1: RangeSetInt<i32> = [3, 2, 1, 100, 1].into();
/// assert!(a0 == a1 && a0.to_string() == "1..=3, 100..=100");
/// ```
///
/// # Set Operations
///
/// | Set Operation           | Operator                   |  Multiway Method |
/// |-------------------|-------------------------|-------------------------|
/// | union       |  `a` &#124; `b`                     | `[a, b, c].`[`union`]`()` |
/// | intersection       |  `a & b`                     | `[a, b, c].`[`intersection`]`()` |
/// | difference       |  `a - b`                     | *n/a* |
/// | symmetric difference       |  `a ^ b`                     | *n/a* |
/// | complement       |  `!a`                     | *n/a* |
///
///
/// [`union`]: trait.MultiwayRangeSetInt.html#method.union
/// [`intersection`]: trait.MultiwayRangeSetInt.html#method.intersection
///
/// ## Set Operation Performance
///
/// Every operation is implemented as
/// 1. a single pass over the sorted & disjoint ranges
/// 2. the construction of a new [`RangeSetInt`]
///
/// Thus, applying multiple operators creates intermediate
/// [`RangeSetInt`]'s. You can avoid these intermediate
/// [`RangeSetInt`]'s by switching to the [`SortedDisjoint`] API. The last example below
/// demonstrates this.
///
/// ## Set Operation Examples
///
/// ```
/// use range_set_int::{RangeSetInt, MultiwayRangeSetInt};
///
/// let a = RangeSetInt::from_iter([1..=2, 5..=100].into_iter());
/// let b = RangeSetInt::from_iter([2..=6].into_iter());
///
/// // Union of two 'RangeSetInt's.
/// let result = &a | &b;
/// // Alternatively, we can take ownership via 'a | b'.
/// assert_eq!(result.to_string(), "1..=100");
///
/// // Intersection of two 'RangeSetInt's.
/// let result = &a & &b; // Alternatively, 'a & b'.
/// assert_eq!(result.to_string(), "2..=2, 5..=6");
///
/// // Set difference of two 'RangeSetInt's.
/// let result = &a - &b; // Alternatively, 'a - b'.
/// assert_eq!(result.to_string(), "1..=1, 7..=100");
///
/// // Symmetric difference of two 'RangeSetInt's.
/// let result = &a ^ &b; // Alternatively, 'a ^ b'.
/// assert_eq!(result.to_string(), "1..=1, 3..=4, 7..=100");
///
/// // Negation of a 'RangeSetInt'.
/// let result = !&a; // Alternatively, '!a'.
/// assert_eq!(
///     result.to_string(),
///     "-2147483648..=0, 3..=4, 101..=2147483647"
/// );
///
/// // Multiway union of 'RangeSetInt's.
/// let c = RangeSetInt::from_iter([2..=2, 6..=200].into_iter());
/// let result = [&a, &b, &c].union();
/// assert_eq!(result.to_string(), "1..=200");
///
/// // Multiway intersection of 'RangeSetInt's.
/// let result = [&a, &b, &c].intersection();
/// assert_eq!(result.to_string(), "2..=2, 6..=6");
///
/// // Applying multiple operators
/// let result0 = &a - (&b | &c); // Creates an intermediate 'RangeSetInt'.
/// // Alternatively, we can use the 'SortedDisjoint' API and avoid the intermediate 'RangeSetInt'.
/// let result1 = RangeSetInt::from(a.ranges() - (b.ranges() | c.ranges()));
/// assert!(result0 == result1 && result0.to_string() == "1..=1");
/// ```
/// cmk00000
///
/// See the [module-level documentation] for additional examples.
///
/// [module-level documentation]: index.html
pub struct RangeSetInt<T: Integer> {
    len: <T as Integer>::SafeLen,
    btree_map: BTreeMap<T, T>,
}

impl<T: Integer> fmt::Debug for RangeSetInt<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.ranges().to_string())
    }
}

impl<T: Integer> fmt::Display for RangeSetInt<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.ranges().to_string())
    }
}

impl<T: Integer> RangeSetInt<T> {
    /// Gets an iterator that visits the integer elements in the [`RangeSetInt`] in ascending
    /// order.
    ///
    /// Also see the [`RangeSetInt::ranges`] method.
    ///
    /// # Examples
    ///
    /// ```
    /// use range_set_int::RangeSetInt;
    ///
    /// let set = RangeSetInt::from_iter([1..=3]);
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
    /// use range_set_int::RangeSetInt;
    ///
    /// let set = RangeSetInt::from_iter([3, 1, 2]);
    /// let mut set_iter = set.iter();
    /// assert_eq!(set_iter.next(), Some(1));
    /// assert_eq!(set_iter.next(), Some(2));
    /// assert_eq!(set_iter.next(), Some(3));
    /// assert_eq!(set_iter.next(), None);
    /// ```
    pub fn iter(&self) -> Iter<T, impl Iterator<Item = RangeInclusive<T>> + SortedDisjoint + '_> {
        // If the user asks for an iter, we give them a borrow to a RangesIter iterator
        // and we iterate that one integer at a time.
        Iter {
            current: T::zero(),
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
    /// use range_set_int::RangeSetInt;
    ///
    /// let mut set = RangeSetInt::new();
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
    /// use range_set_int::RangeSetInt;
    ///
    /// let set = RangeSetInt::from_iter([1, 2, 3]);
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
    /// use range_set_int::RangeSetInt;
    ///
    /// let mut set = RangeSetInt::new();
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
}

// cmk0000 link regular union with multiway union, etc etc etc
impl<T: Integer> RangeSetInt<T> {
    /// !!! cmk understand the 'where for'
    /// !!! cmk understand the operator 'Sub'
    fn _len_slow(&self) -> <T as Integer>::SafeLen
    where
        for<'a> &'a T: ops::Sub<&'a T, Output = T>,
    {
        RangeSetInt::btree_map_len(&self.btree_map)
    }

    /// Moves all elements from `other` into `self`, leaving `other` empty.
    ///
    /// # Performance
    /// It adds the integers in `other` to `self` in O(n log m) time, where n is the number of ranges in `other`
    /// and m is the number of ranges in `self`.
    /// When n is large, consider using `bitor` which is O(n+m) time. cmk000 advise
    ///
    /// # Examples
    ///
    /// ```
    /// use range_set_int::RangeSetInt;
    ///
    /// let mut a = RangeSetInt::from_iter([1..=3]);
    /// let mut b = RangeSetInt::from_iter([3..=5]);
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
    /// use range_set_int::RangeSetInt;
    ///
    /// let mut v = RangeSetInt::new();
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
    /// use range_set_int::RangeSetInt;
    ///
    /// let mut v = RangeSetInt::new();
    /// assert!(v.is_empty());
    /// v.insert(1);
    /// assert!(!v.is_empty());
    /// ```
    pub fn is_empty(&self) -> bool {
        self.ranges_len() == 0
    }

    /// Returns `true` if the set is a subset of another,
    /// i.e., `other` contains at least all the elements in `self`.
    ///
    /// # Examples
    ///
    /// ```
    /// use range_set_int::RangeSetInt;
    ///
    /// let sup = RangeSetInt::from_iter([1..=3]);
    /// let mut set = RangeSetInt::new();
    ///
    /// assert_eq!(set.is_subset(&sup), true);
    /// set.insert(2);
    /// assert_eq!(set.is_subset(&sup), true);
    /// set.insert(4);
    /// assert_eq!(set.is_subset(&sup), false);
    /// ```
    #[must_use]
    pub fn is_subset(&self, other: &RangeSetInt<T>) -> bool {
        // Add a fast path
        if self.len() > other.len() {
            return false;
        }
        (self.ranges() - other.ranges()).next().is_none()
    }

    /// Returns `true` if the set is a superset of another,
    /// i.e., `self` contains at least all the elements in `other`.
    ///
    /// # Examples
    ///
    /// ```
    /// use range_set_int::RangeSetInt;
    ///
    /// let sub = RangeSetInt::from_iter([1, 2]);
    /// let mut set = RangeSetInt::new();
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
    pub fn is_superset(&self, other: &RangeSetInt<T>) -> bool {
        other.is_subset(self)
    }

    /// Returns `true` if the set contains an element equal to the value.
    ///
    /// # Examples
    ///
    /// ```
    /// use range_set_int::RangeSetInt;
    ///
    /// let set = RangeSetInt::from_iter([1, 2, 3]);
    /// assert_eq!(set.contains(1), true);
    /// assert_eq!(set.contains(4), false);
    /// ```
    pub fn contains(&self, value: T) -> bool {
        assert!(value <= T::safe_max_value()); //cmk0 panic
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
    /// use range_set_int::RangeSetInt;
    ///
    /// let a = RangeSetInt::from_iter([1..=3]);
    /// let mut b = RangeSetInt::new();
    ///
    /// assert_eq!(a.is_disjoint(&b), true);
    /// b.insert(4);
    /// assert_eq!(a.is_disjoint(&b), true);
    /// b.insert(1);
    /// assert_eq!(a.is_disjoint(&b), false);
    /// ```
    /// cmk rule which functions should be must_use? iterator, constructor, predicates, first, last,
    #[must_use]
    pub fn is_disjoint(&self, other: &RangeSetInt<T>) -> bool {
        (self.ranges() & other.ranges()).next().is_none()
    }

    fn delete_extra(&mut self, internal_range: &RangeInclusive<T>) {
        let (start, end) = internal_range.clone().into_inner();
        let mut after = self.btree_map.range_mut(start..);
        let (start_after, end_after) = after.next().unwrap(); // there will always be a next
        debug_assert!(start == *start_after && end == *end_after); // real assert
                                                                   // !!!cmk would be nice to have a delete_range function
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
    /// When n is large, consider using `cmk0doc` which is O(n+m) time.
    ///
    /// # Examples
    ///
    /// ```
    /// use range_set_int::RangeSetInt;
    ///
    /// let mut set = RangeSetInt::new();
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
    /// When n is large, consider using `cmk0doc` which is O(n+m) time.
    ///
    /// # Examples
    ///
    /// ```
    /// use range_set_int::RangeSetInt;
    ///
    /// let mut set = RangeSetInt::new();
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
    /// use range_set_int::RangeSetInt;
    ///
    /// let mut set = RangeSetInt::new();
    ///
    /// set.insert(2);
    /// assert!(set.remove(2));
    /// assert!(!set.remove(2));
    /// ```
    pub fn remove(&mut self, value: T) -> bool {
        assert!(value <= T::safe_max_value()); //cmk0 panic

        // The code can have only one mutable reference to self.btree_map.
        let start;
        let end;
        if let Some((start_ref, end_ref)) = self.btree_map.range_mut(..=value).rev().next() {
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
    /// use range_set_int::RangeSetInt;
    ///
    /// let mut a = RangeSetInt::new();
    /// a.insert(1);
    /// a.insert(2);
    /// a.insert(3);
    /// a.insert(17);
    /// a.insert(41);
    ///
    /// let b = a.split_off(3);
    ///
    /// assert_eq!(a, RangeSetInt::from_iter([1, 2]));
    /// assert_eq!(b, RangeSetInt::from_iter([3, 17, 41]));
    /// ```
    pub fn split_off(&mut self, value: T) -> Self {
        assert!(value <= T::safe_max_value()); //cmk0 panic

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
            self.len = RangeSetInt::btree_map_len(&self.btree_map);
            old_len - self.len
        } else {
            let b_len = RangeSetInt::btree_map_len(&b);
            self.len = old_len - b_len;
            b_len
        };
        RangeSetInt {
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
    /// use range_set_int::RangeSetInt;
    ///
    /// let mut set = RangeSetInt::from_iter([1, 2, 3]);
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
    /// [`BTreeSet`]: std::collections::BTreeSet
    ///
    /// # Examples
    ///
    /// ```
    /// use range_set_int::RangeSetInt;
    ///
    /// let mut set = RangeSetInt::new();
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

    // https://stackoverflow.com/questions/49599833/how-to-find-next-smaller-key-in-btreemap-btreeset
    // https://stackoverflow.com/questions/35663342/how-to-modify-partially-remove-a-range-from-a-btreemap
    fn internal_add(&mut self, range: RangeInclusive<T>) {
        let (start, end) = range.clone().into_inner();
        assert!(end <= T::safe_max_value()); //cmk0 panic
        if end < start {
            return;
        }
        // !!! cmk would be nice to have a partition_point function that returns two iterators
        let mut before = self.btree_map.range_mut(..=start).rev();
        if let Some((start_before, end_before)) = before.next() {
            // Must check this in two parts to avoid overflow
            if *end_before < start && *end_before + T::one() < start {
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
    /// use range_set_int::RangeSetInt;
    ///
    /// let mut v = RangeSetInt::new();
    /// assert_eq!(v.len(), 0usize);
    /// v.insert(1);
    /// assert_eq!(v.len(), 1usize);
    ///
    /// let v = RangeSetInt::from_iter([
    ///     -170_141_183_460_469_231_731_687_303_715_884_105_728i128..=10,
    ///     -10..=170_141_183_460_469_231_731_687_303_715_884_105_726,
    /// ]);
    /// assert_eq!(
    ///     v.len(),
    ///     340_282_366_920_938_463_463_374_607_431_768_211_455u128
    /// );
    /// ```
    #[must_use]
    pub fn len(&self) -> <T as Integer>::SafeLen {
        self.len
    }

    /// Makes a new, empty [`RangeSetInt`].
    ///
    /// # Examples
    ///
    /// ```
    /// # #![allow(unused_mut)]
    /// use range_set_int::RangeSetInt;
    ///
    /// let mut set: RangeSetInt<i32> = RangeSetInt::new();
    /// ```
    #[must_use]
    pub fn new() -> RangeSetInt<T> {
        RangeSetInt {
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
    /// use range_set_int::RangeSetInt;
    ///
    /// let mut set = RangeSetInt::new();
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
    /// use range_set_int::RangeSetInt;
    ///
    /// let mut set = RangeSetInt::new();
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

    /// An iterator that visits the ranges in the [`RangeSetInt`],
    /// i.e., the integers as sorted & disjoint ranges.
    ///
    /// Also see [`RangeSetInt::iter`] and [`RangeSetInt::into_ranges`].
    ///
    /// # Examples
    ///
    /// ```
    /// use range_set_int::RangeSetInt;
    ///
    /// let set = RangeSetInt::from_iter([10..=20, 15..=25, 30..=40]);
    /// let mut ranges = set.ranges();
    /// assert_eq!(ranges.next(), Some(10..=25));
    /// assert_eq!(ranges.next(), Some(30..=40));
    /// assert_eq!(ranges.next(), None);
    /// ```
    ///
    /// Values returned by the iterator are returned in ascending order:
    ///
    /// ```
    /// use range_set_int::RangeSetInt;
    ///
    /// let set = RangeSetInt::from_iter([30..=40, 15..=25, 10..=20]);
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

    /// An iterator that visits the ranges in the [`RangeSetInt`],
    /// i.e., the integers as sorted & disjoint ranges.
    ///
    /// Also see [`RangeSetInt::into_iter`] and [`RangeSetInt::ranges`].
    ///
    /// # Examples
    ///
    /// ```
    /// use range_set_int::RangeSetInt;
    ///
    /// let mut ranges = RangeSetInt::from_iter([10..=20, 15..=25, 30..=40]).into_ranges();
    /// assert_eq!(ranges.next(), Some(10..=25));
    /// assert_eq!(ranges.next(), Some(30..=40));
    /// assert_eq!(ranges.next(), None);
    /// ```
    ///
    /// Values returned by the iterator are returned in ascending order:
    ///
    /// ```
    /// use range_set_int::RangeSetInt;
    ///
    /// let mut ranges = RangeSetInt::from_iter([30..=40, 15..=25, 10..=20]).into_ranges();
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
    /// use range_set_int::RangeSetInt;
    ///
    /// // We put in three ranges, but they are not sorted & disjoint.
    /// let set = RangeSetInt::from_iter([10..=20, 15..=25, 30..=40]);
    /// // After RangeSetInt sorts & 'disjoint's them, we see two ranges.
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
    /// use range_set_int::RangeSetInt;
    ///
    /// let mut set = RangeSetInt::from_iter([1..=6]);
    /// // Keep only the even numbers.
    /// set.retain(|k| k % 2 == 0);
    /// assert_eq!(set, RangeSetInt::from_iter([2, 4, 6]));
    /// ```
    pub fn retain<F>(&mut self, mut f: F)
    where
        F: FnMut(&T) -> bool,
    {
        *self = self.iter().filter(|v| f(v)).collect();
    }
}

// We create a RangeSetInt from an iterator of integers or integer ranges by
// 1. turning them into a UnionIter (internally, it collects into intervals and sorts by start).
// 2. Turning the SortedDisjoint into a BTreeMap.
impl<T: Integer> FromIterator<T> for RangeSetInt<T> {
    /// Create a [`RangeSetInt`] from an iterator of integers. Duplicates and out-of-order elements are fine.
    ///
    /// *For more about constructors, see [`RangeSetInt` Constructors](struct.RangeSetInt.html#constructors).*
    ///
    /// # Performance
    ///
    /// This constructor works fast on clumpy data. Internally, it
    ///  * collects adjacent integers into ranges (linear)
    ///  * sorts the ranges by start (log * linear)
    ///  * merges adjacent and overlapping ranges (linear)
    ///  * creates a BTreeMap from the now sorted & adjacent ranges (log * linear)
    ///
    /// # Examples
    ///
    /// ```
    /// use range_set_int::RangeSetInt;
    ///
    /// let a0 = RangeSetInt::from_iter([3, 2, 1, 100, 1]);
    /// let a1: RangeSetInt<i32> = [3, 2, 1, 100, 1].into_iter().collect();
    /// assert!(a0 == a1 && a0.to_string() == "1..=3, 100..=100");
    /// ```
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = T>,
    {
        iter.into_iter().map(|x| x..=x).collect()
    }
}

// cmk rules: Follow Rust conventions. For example this as empty let cmk = 1..=-1; we do the same
impl<T: Integer> FromIterator<RangeInclusive<T>> for RangeSetInt<T> {
    /// Create a [`RangeSetInt`] from an iterator of inclusive ranges, `start..=end`.
    /// Overlapping, out-of-order, and empty ranges are fine.
    ///
    /// *For more about constructors, see [`RangeSetInt` Constructors](struct.RangeSetInt.html#constructors).*
    ///
    /// # Performance
    ///
    /// This constructor works fast on clumpy data. Internally, it
    ///  * sorts the ranges by start (log * linear)
    ///  * merges adjacent and overlapping ranges (linear)
    ///  * creates a BTreeMap from the now sorted & adjacent ranges (log * linear)
    ///
    /// # Examples
    ///
    /// ```
    /// use range_set_int::RangeSetInt;
    ///
    /// #[allow(clippy::reversed_empty_ranges)]
    /// let a0 = RangeSetInt::from_iter([1..=2, 2..=2, -10..=-5, 1..=0]);
    /// #[allow(clippy::reversed_empty_ranges)]
    /// let a1: RangeSetInt<i32> = [1..=2, 2..=2, -10..=-5, 1..=0].into_iter().collect();
    /// assert!(a0 == a1 && a0.to_string() == "-10..=-5, 1..=2");
    /// ```
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = RangeInclusive<T>>,
    {
        let union_iter: UnionIter<T, _> = iter.into_iter().collect();
        union_iter.into()
    }
}

impl<T: Integer, const N: usize> From<[T; N]> for RangeSetInt<T> {
    /// For compatibility with [`BTreeSet`] you may create a [`RangeSetInt`] from an array of integers.
    ///
    /// *For more about constructors, see [`RangeSetInt` Constructors](struct.RangeSetInt.html#constructors).*
    ///
    /// # Performance
    ///
    /// This constructor works fast on clumpy data. Internally, it
    ///  * collects adjacent integers into ranges (linear)
    ///  * sorts the ranges by start (log * linear)
    ///  * merges adjacent and overlapping ranges (linear)
    ///  * creates a BTreeMap from the now sorted & adjacent ranges (log * linear)
    ///
    /// [`BTreeSet`]: std::collections::BTreeSet
    ///
    /// # Examples
    ///
    /// ```
    /// use range_set_int::RangeSetInt;
    ///
    /// let a0 = RangeSetInt::from([3, 2, 1, 100, 1]);
    /// let a1: RangeSetInt<i32> = [3, 2, 1, 100, 1].into();
    /// assert!(a0 == a1 && a0.to_string() == "1..=3, 100..=100")
    /// ```
    fn from(arr: [T; N]) -> Self {
        arr.into_iter().collect()
    }
}

impl<T, I> From<I> for RangeSetInt<T>
where
    T: Integer,
    // !!!cmk what does IntoIterator's ' IntoIter = I::IntoIter' mean?
    I: Iterator<Item = RangeInclusive<T>> + SortedDisjoint,
    // cmk0 understand why this can't be  I: IntoIterator<Item = RangeInclusive<T>>, <I as IntoIterator>::IntoIter: SortedDisjoint, some conflict with from[]
{
    /// Create a [`RangeSetInt`] from a [`SortedDisjoint`] iterator.
    ///
    /// *For more about constructors, see [`RangeSetInt` Constructors](struct.RangeSetInt.html#constructors).*
    ///
    /// # Performance
    ///
    /// This constructor works fast on clumpy data. Internally, it
    ///  * creates a BTreeMap from the sorted & adjacent ranges (log * linear)
    ///
    /// # Examples
    ///
    /// ```
    /// use range_set_int::{RangeSetInt, CheckSortedDisjoint};
    ///
    /// let a0 = RangeSetInt::from(CheckSortedDisjoint::new([-10..=-5, 1..=2].into_iter()));
    /// let a1: RangeSetInt<i32> = CheckSortedDisjoint::new([-10..=-5, 1..=2].into_iter()).into();
    /// assert!(a0 == a1 && a0.to_string() == "-10..=-5, 1..=2");    
    /// ```
    fn from(iter: I) -> Self {
        let mut iter_with_len = SortedDisjointWithLenSoFar::from(iter);
        let btree_map = BTreeMap::from_iter(&mut iter_with_len);
        RangeSetInt {
            btree_map,
            len: iter_with_len.len_so_far(),
        }
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

// cmk0 explain why this is needed
impl<'a, T, I> MultiwayRangeSetInt<'a, T> for I
where
    T: Integer + 'a,
    I: IntoIterator<Item = &'a RangeSetInt<T>>,
{
}

/// The trait used to provide methods on multiple [`RangeSetInt`]'s,
/// specifically [`union`] and [`intersection`].
///
/// [`union`]: MultiwayRangeSetInt::union
/// [`intersection`]: MultiwayRangeSetInt::intersection
pub trait MultiwayRangeSetInt<'a, T: Integer + 'a>:
    IntoIterator<Item = &'a RangeSetInt<T>> + Sized
{
    /// Unions the given [`RangeSetInt`]'s, creating a new [`RangeSetInt`].
    /// Any number of input can be given.
    ///
    /// For exactly two inputs, you can also use the '|' operator.
    ///
    /// # Performance
    ///
    ///  All work is done on demand, in one pass through the inputs. Minimal memory is used.
    ///
    /// # Example
    ///
    /// Find the integers that appear in any of the [`RangeSetInt`]'s.
    ///
    /// ```
    /// use range_set_int::{MultiwayRangeSetInt, RangeSetInt};
    ///
    /// let a = RangeSetInt::from_iter([1..=6, 8..=9, 11..=15]);
    /// let b = RangeSetInt::from_iter([5..=13, 18..=29]);
    /// let c = RangeSetInt::from_iter([25..=100]);
    ///
    /// let union = [a, b, c].union();
    ///
    /// assert_eq!(union, RangeSetInt::from_iter([1..=15, 18..=100]));
    /// ```
    fn union(self) -> RangeSetInt<T> {
        self.into_iter().map(RangeSetInt::ranges).union().into()
    }

    /// Intersects the given [`RangeSetInt`]'s, creating a new [`RangeSetInt`].
    /// Any number of input can be given.
    ///
    /// For exactly two inputs, you can also use the '&' operator.
    ///
    /// # Performance
    ///
    ///  All work is done on demand, in one pass through the inputs. Minimal memory is used.
    ///
    /// # Example
    ///
    /// Find the integers that appear in all the [`RangeSetInt`]'s.
    ///
    /// ```
    /// use range_set_int::{MultiwayRangeSetInt, RangeSetInt};
    ///
    /// let a = RangeSetInt::from_iter([1..=6, 8..=9, 11..=15]);
    /// let b = RangeSetInt::from_iter([5..=13, 18..=29]);
    /// let c = RangeSetInt::from_iter([-100..=100]);
    ///
    /// let intersection = [a, b, c].intersection();
    ///
    /// assert_eq!(intersection, RangeSetInt::from_iter([5..=6, 8..=9, 11..=13]));
    /// ```
    fn intersection(self) -> RangeSetInt<T> {
        self.into_iter()
            .map(RangeSetInt::ranges)
            .intersection()
            .into()
    }
}

// cmk0 explain why this is needed
impl<T, II, I> MultiwaySortedDisjoint<T, I> for II
where
    T: Integer,
    I: SortedDisjointIterator<T>,
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
    I: SortedDisjointIterator<T>,
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
    /// use range_set_int::{MultiwaySortedDisjoint, RangeSetInt, SortedDisjointIterator};
    ///
    /// let a = RangeSetInt::from_iter([1..=6, 8..=9, 11..=15]).into_ranges();
    /// let b = RangeSetInt::from_iter([5..=13, 18..=29]).into_ranges();
    /// let c = RangeSetInt::from_iter([25..=100]).into_ranges();
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
    /// use range_set_int::{MultiwaySortedDisjoint, RangeSetInt, SortedDisjointIterator};
    ///
    /// let a = RangeSetInt::from_iter([1..=6, 8..=9, 11..=15]).into_ranges();
    /// let b = RangeSetInt::from_iter([5..=13, 18..=29]).into_ranges();
    /// let c = RangeSetInt::from_iter([-100..=100]).into_ranges();
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

// cmk rule: don't forget these '+ SortedDisjoint'. They are easy to forget and hard to test, but must be tested (via "UI")

// cmk0000 not used
// Returns the union of `self` and `rhs` as a new [`RangeSetInt`].
//
// # Examples
//
// ```
// use range_set_int::RangeSetInt;
//
// let a = RangeSetInt::from_iter([1, 2, 3]);
// let b = RangeSetInt::from_iter([3, 4, 5]);
//
// let result = &a | &b;
// assert_eq!(result, RangeSetInt::from_iter([1, 2, 3, 4, 5]));
// let result = a | b;
// assert_eq!(result, RangeSetInt::from_iter([1, 2, 3, 4, 5]));
// ```
gen_ops_ex!(
    <T>;
    types ref RangeSetInt<T>, ref RangeSetInt<T> => RangeSetInt<T>;
    for | call |a: &RangeSetInt<T>, b: &RangeSetInt<T>| {
        (a.ranges()|b.ranges()).into()
    };
    for & call |a: &RangeSetInt<T>, b: &RangeSetInt<T>| {
        (a.ranges() & b.ranges()).into()
    };
    for ^ call |a: &RangeSetInt<T>, b: &RangeSetInt<T>| {
        // We optimize this by using ranges() twice per input, rather than tee()
        let lhs0 = a.ranges();
        let lhs1 = a.ranges();
        let rhs0 = b.ranges();
        let rhs1 = b.ranges();
        ((lhs0 - rhs0) | (rhs1 - lhs1)).into()
    };
    for - call |a: &RangeSetInt<T>, b: &RangeSetInt<T>| {
        (a.ranges() - b.ranges()).into()
    };
    // cmk0 must/should we support both operators and methods?

    where T: Integer //Where clause for all impl's
);

gen_ops_ex!(
    <T>;
    types ref RangeSetInt<T> => RangeSetInt<T>;
    for ! call |a: &RangeSetInt<T>| {
        (!a.ranges()).into()
    };

    where T: Integer //Where clause for all impl's
);

impl<T: Integer> IntoIterator for RangeSetInt<T> {
    type Item = T;
    type IntoIter = IntoIter<T>;

    /// Gets an iterator for moving out the [`RangeSetInt`]'s contents.
    ///
    /// # Examples
    ///
    /// ```
    /// use range_set_int::RangeSetInt;
    ///
    /// let set = RangeSetInt::from_iter([1, 2, 3, 4]);
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

/// An iterator over the integer elements of a [`RangeSetInt`].
///
/// This `struct` is created by the [`iter`] method on [`RangeSetInt`]. See its
/// documentation for more.
///
/// [`iter`]: RangeSetInt::iter
#[must_use = "iterators are lazy and do nothing unless consumed"]
#[derive(Clone)]
pub struct Iter<T, I>
where
    T: Integer,
    I: Iterator<Item = RangeInclusive<T>> + SortedDisjoint,
{
    iter: I,
    current: T, // !!!cmk can't we write this without current? (likewise IntoIter)
    option_range: Option<RangeInclusive<T>>,
}

impl<T: Integer, I> Iterator for Iter<T, I>
where
    I: Iterator<Item = RangeInclusive<T>> + SortedDisjoint,
{
    type Item = T;
    fn next(&mut self) -> Option<T> {
        loop {
            if let Some(range) = self.option_range.clone() {
                let (start, end) = range.into_inner();
                debug_assert!(start <= end && end <= T::safe_max_value());
                self.current = start;
                if start < end {
                    self.option_range = Some(start + T::one()..=end);
                } else {
                    self.option_range = None;
                }
                return Some(self.current);
            } else if let Some(range) = self.iter.next() {
                self.option_range = Some(range);
                continue;
            } else {
                return None;
            }
        }
    }

    // We'll have at least as many integers as intervals. There could be more that usize MAX
    // The option_range field could increase the number of integers, but we can ignore that.
    fn size_hint(&self) -> (usize, Option<usize>) {
        let (low, _high) = self.iter.size_hint();
        (low, None)
    }
}

#[must_use = "iterators are lazy and do nothing unless consumed"]
/// An iterator over the integer elements of a [`RangeSetInt`].
///
/// This `struct` is created by the [`into_iter`] method on [`RangeSetInt`]. See its
/// documentation for more.
///
/// [`into_iter`]: RangeSetInt::into_iter
pub struct IntoIter<T: Integer> {
    option_range: Option<RangeInclusive<T>>, // cmk000 replace option_range: with option_range or range
    into_iter: std::collections::btree_map::IntoIter<T, T>,
}

impl<T: Integer> Iterator for IntoIter<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(range) = self.option_range.clone() {
            let (start, end) = range.into_inner();
            debug_assert!(start <= end && end <= T::safe_max_value());
            if start < end {
                self.option_range = Some(start + T::one()..=end);
            } else {
                self.option_range = None;
            }
            Some(start)
        } else if let Some((start, end)) = self.into_iter.next() {
            self.option_range = Some(start..=end);
            self.next() // will recurse at most once
        } else {
            None
        }
    }

    // We'll have at least as many integers as intervals. There could be more that usize MAX
    // the option_range field could increase the number of integers, but we can ignore that.
    fn size_hint(&self) -> (usize, Option<usize>) {
        let (low, _high) = self.into_iter.size_hint();
        (low, None)
    }
}

/// cmk warn that adds one-by-one
impl<T: Integer> Extend<T> for RangeSetInt<T> {
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

impl<T: Integer> Extend<RangeInclusive<T>> for RangeSetInt<T> {
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

impl<'a, T: 'a + Integer> Extend<&'a T> for RangeSetInt<T> {
    fn extend<I: IntoIterator<Item = &'a T>>(&mut self, iter: I) {
        self.extend(iter.into_iter().cloned());
    }
}

impl<'a, T: 'a + Integer> Extend<&'a RangeInclusive<T>> for RangeSetInt<T> {
    fn extend<I: IntoIterator<Item = &'a RangeInclusive<T>>>(&mut self, iter: I) {
        self.extend(iter.into_iter().cloned());
    }
}

// !!!cmk support =, and single numbers
// !!!cmk error to use -
// !!!cmk are the unwraps OK?
// !!!cmk what about bad input?

// cmk This code from sorted-iter shows how to define clone when possible
// impl<I: Iterator + Clone, J: Iterator + Clone> Clone for Union<I, J>
// where
//     I::Item: Clone,
//     J::Item: Clone,
// {
//     fn clone(&self) -> Self {
//         Self {
//             a: self.a.clone(),
//             b: self.b.clone(),
//         }
//     }
// }

// cmk sort-iter uses peekable. Is that better?

impl<T: Integer> Ord for RangeSetInt<T> {
    /// cmk0doc clarify that this is lexicographic order not subset/superset
    #[inline]
    fn cmp(&self, other: &RangeSetInt<T>) -> Ordering {
        // slow return self.iter().cmp(other.iter());

        let mut a = self.ranges();
        let mut b = other.ranges();
        let mut a_rx = a.next();
        let mut b_rx = b.next();
        loop {
            match (a_rx.clone(), b_rx.clone()) {
                (Some(a_r), Some(b_r)) => {
                    let cmp = a_r.start().cmp(b_r.start());
                    if cmp != Ordering::Equal {
                        return cmp;
                    }
                    let cmp = a_r.end().cmp(b_r.end());
                    match cmp {
                        Ordering::Equal => {}
                        Ordering::Less => {
                            a_rx = a.next();
                            b_rx = Some(*a_r.end() + T::one()..=*b_r.end());
                            continue;
                        }
                        Ordering::Greater => {
                            b_rx = b.next();
                            a_rx = Some(*b_r.end() + T::one()..=*a_r.end());
                            continue;
                        }
                    }
                    if cmp != Ordering::Equal {
                        return cmp;
                    }
                }
                (Some(_), None) => return Ordering::Greater,
                (None, Some(_)) => return Ordering::Less,
                (None, None) => return Ordering::Equal,
            }
            a_rx = a.next();
            b_rx = b.next();
        }
    }
}

impl<T: Integer> PartialOrd for RangeSetInt<T> {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<T: Integer> Eq for RangeSetInt<T> {}

/// The trait used to mark iterators that provide ranges that are sorted by start and that do not overlap.
///
/// # Constructors
///
/// | Input type | Method |
/// |------------|--------|
/// | [`RangeSetInt`] | [`ranges`] |
/// | [`RangeSetInt`] | [`into_ranges`] |
/// | [`RangeSetInt`]'s [`RangesIter`] | [`clone`] |
/// | sorted & disjoint ranges | [`CheckSortedDisjoint::new`] |
/// | `SortedDisjoint` iterator | [itertools `tee`] |
/// | `SortedDisjoint` iterator | [`DynSortedDisjoint::new`] |
/// |  *your iterator type* | *[How to mark your type as `SortedDisjoint`][1]* |
///
/// [`ranges`]: RangeSetInt::ranges
/// [`into_ranges`]: RangeSetInt::into_ranges
/// [`clone`]: RangesIter::clone
/// [itertools `tee`]: https://docs.rs/itertools/latest/itertools/trait.Itertools.html#method.tee
/// [1]: #how-to-mark-your-type-as-sorteddisjoint
///
/// ## Constructor Examples
///
/// ```
/// use range_set_int::{RangeSetInt, CheckSortedDisjoint, DynSortedDisjoint};
/// use range_set_int::SortedDisjointIterator;
/// use itertools::Itertools;
///
/// // RangeSetInt's .ranges(), .range().clone() and .into_ranges()
/// let r = RangeSetInt::from_iter([3, 2, 1, 100, 1]);
/// let a = r.ranges();
/// let b = a.clone();
/// assert!(a.to_string() == "1..=3, 100..=100");
/// assert!(b.to_string() == "1..=3, 100..=100");
/// //    'into_ranges' takes ownership of the 'RangeSetInt'
/// let a = RangeSetInt::from_iter([3, 2, 1, 100, 1]).into_ranges();
/// assert!(a.to_string() == "1..=3, 100..=100");
///
/// // CheckSortedDisjoint -- unsorted or overlapping input ranges will cause a panic.
/// let a = CheckSortedDisjoint::new([1..=3, 100..=100].into_iter());
/// assert!(a.to_string() == "1..=3, 100..=100");
///
/// // tee of a SortedDisjoint iterator
/// let a = CheckSortedDisjoint::new([1..=3, 100..=100].into_iter());
/// let (a, b) = a.tee();
/// assert!(a.to_string() == "1..=3, 100..=100");
/// assert!(b.to_string() == "1..=3, 100..=100");
///
/// // DynamicSortedDisjoint of a SortedDisjoint iterator
/// let a = CheckSortedDisjoint::new([1..=3, 100..=100].into_iter());
/// let b = DynSortedDisjoint::new(a);
/// assert!(b.to_string() == "1..=3, 100..=100");
/// ```
///
///
/// # Set and Other Operations
///
/// ## Performance
/// ## Examples
///
/// # How to mark your type as `SortedDisjoint`
///
/// To mark your iterator type as `SortedDisjoint`, you implement the `SortedStarts` and [`SortedDisjoint`] traits.
/// This is your promise to the compiler that your iterator will provide inclusive ranges that are sorted by start and that do not overlap.
///
/// When you do this, your iterator will get access to the
/// efficient set operations methods, such as [`intersection`] and [`complement`]. The example below shows this.
///
/// > For access to operators such as `&` and `!`, you must also implement the [`BitAnd`], [`Not`], etc. traits.
///
/// [`BitAnd`]: https://doc.rust-lang.org/std/ops/trait.BitAnd.html
/// [`Not`]: https://doc.rust-lang.org/std/ops/trait.Not.html
/// [`intersection`]: SortedDisjointIterator::intersection
/// [`complement`]: SortedDisjointIterator::complement
///
/// ## Example -- Find the ordinal weekdays in September 2023
/// ```
/// use std::ops::RangeInclusive;
/// use range_set_int::{SortedDisjoint, SortedStarts};
///
/// // Ordinal dates count January 1 as day 1, February 1 as day 32, etc.
/// struct OrdinalWeekends2023 {
///     next_range: RangeInclusive<i32>,
/// }
///
/// // We promise the compiler that our iterator will provide
/// // ranges that are sorted and disjoint.
/// impl SortedStarts for OrdinalWeekends2023 {}
/// impl SortedDisjoint for OrdinalWeekends2023 {}
///
/// impl OrdinalWeekends2023 {
///     fn new() -> Self {
///         Self { next_range: 0..=1 }
///     }
/// }
/// impl Iterator for OrdinalWeekends2023 {
///     type Item = RangeInclusive<i32>;
///     fn next(&mut self) -> Option<Self::Item> {
///         let (start, end) = self.next_range.clone().into_inner();
///         if start > 365 {
///             None
///         } else {
///             self.next_range = (start + 7)..=(end + 7);
///             Some(start.max(1)..=end.min(365))
///         }
///     }
/// }
///
/// use range_set_int::{SortedDisjointIterator, CheckSortedDisjoint};
///
/// let weekends = OrdinalWeekends2023::new();
/// let september = CheckSortedDisjoint::new([244..=273].into_iter());
/// let september_weekdays = september.intersection(weekends.complement());
/// assert_eq!(
///     september_weekdays.to_string(),
///     "244..=244, 247..=251, 254..=258, 261..=265, 268..=272"
/// );
/// ```
pub trait SortedDisjoint: SortedStarts {}

/// Internally, a trait used to mark iterators that provide ranges sorted by start, but not necessarily by end,
/// and may overlap.
#[doc(hidden)]
pub trait SortedStarts {}

// cmk rule add must_use to every iter and other places ala https://doc.rust-lang.org/src/alloc/collections/btree/map.rs.html#1259-1261

// If the iterator inside a BitOrIter is SortedStart, the output will be SortedDisjoint
impl<T: Integer, I: Iterator<Item = RangeInclusive<T>> + SortedStarts> SortedStarts
    for UnionIter<T, I>
{
}
impl<T: Integer, I: Iterator<Item = RangeInclusive<T>> + SortedStarts> SortedDisjoint
    for UnionIter<T, I>
{
}
// If the iterator inside NotIter is SortedDisjoint, the output will be SortedDisjoint
impl<T: Integer, I: Iterator<Item = RangeInclusive<T>> + SortedDisjoint> SortedStarts
    for NotIter<T, I>
{
}
impl<T: Integer, I: Iterator<Item = RangeInclusive<T>> + SortedDisjoint> SortedDisjoint
    for NotIter<T, I>
{
}
// If the iterator inside Tee is SortedDisjoint, the output will be SortedDisjoint
impl<T: Integer, I: Iterator<Item = RangeInclusive<T>> + SortedDisjoint> SortedStarts for Tee<I> {}
impl<T: Integer, I: Iterator<Item = RangeInclusive<T>> + SortedDisjoint> SortedDisjoint for Tee<I> {}
