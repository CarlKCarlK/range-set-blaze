use crate::intersection_iter_map::IntersectionIterMap;
use crate::merge_map::MergeMap;
use crate::range_values::{
    AdjustPriorityMap, RangeValuesFromBTree, RangeValuesIter, RangesFromMapIter,
};
use alloc::borrow::ToOwned;
#[cfg(feature = "std")]
use alloc::sync::Arc;
use alloc::vec;
use alloc::vec::Vec;
// use crate::range_values::RangeValuesIter;
use crate::range_values::NonZeroEnumerateExt;
use crate::sorted_disjoint_map::{DebugToString, RangeValue};
use crate::sorted_disjoint_map::{SortedDisjointMap, SortedStartsMap};
use crate::union_iter_map::UnionIterMap;
use crate::unsorted_disjoint_map::{AssumeSortedStartsMap, SortedDisjointWithLenSoFarMap};
use crate::{Integer, NotIter, RangeSetBlaze, SortedDisjoint};
use alloc::collections::{btree_map, BTreeMap};
use alloc::rc::Rc;
use core::borrow::Borrow;
use core::fmt;
use core::iter::FusedIterator;
use core::marker::PhantomData;
use core::ops::BitOr;
use core::{cmp::max, convert::From, ops::RangeInclusive};
use gen_ops::gen_ops_ex;
use num_traits::One;
use num_traits::Zero;

// cmk fix name aka 'Clone'
pub trait ValueOwned: PartialEq + Clone {}

// pub trait ValueRef: PartialEq + ToOwned {
//     type Owned: ValueOwned<Ref = Self>;
// }

impl<T> ValueOwned for T where T: PartialEq + Clone {}

pub trait CloneBorrow<V: ?Sized + ValueOwned>: Borrow<V>
where
    Self: Sized,
{
    fn clone_borrow(&self) -> Self;

    // If you intend to consume `Self`, the method signature should indeed take `self`
    fn borrow_clone(self) -> V {
        // Since `self` is consumed, you can do anything with it, including dropping
        self.borrow().clone()
    }
}

impl<V: ?Sized + ValueOwned> CloneBorrow<V> for &V {
    fn clone_borrow(&self) -> Self {
        self
    }
}

impl<V: ?Sized + ValueOwned> CloneBorrow<V> for Rc<V> {
    fn clone_borrow(&self) -> Self {
        Rc::clone(self)
    }
}

#[cfg(feature = "std")]
impl<V: ?Sized + ValueOwned> CloneBorrow<V> for Arc<V> {
    fn clone_borrow(&self) -> Self {
        Arc::clone(self)
    }
}

#[derive(Clone, Hash, Default, PartialEq)]
pub(crate) struct EndValue<T: Integer, V: ValueOwned>
where
    <V as ToOwned>::Owned: PartialEq,
{
    pub(crate) end: T,
    pub(crate) value: V,
}

#[derive(Clone, Hash, Default, PartialEq)]
/// A set of integers stored as sorted & disjoint ranges.
///
/// Internally, it stores the ranges in a cache-efficient [`BTreeMap`].
///
/// # Table of Contents
/// * [`RangeMapBlaze` Constructors](#RangeMapBlaze-constructors)
///    * [Performance](#constructor-performance)
///    * [Examples](struct.RangeMapBlaze.html#constructor-examples)
/// * [`RangeMapBlaze` Set Operations](#RangeMapBlaze-set-operations)
///    * [Performance](struct.RangeMapBlaze.html#set-operation-performance)
///    * [Examples](struct.RangeMapBlaze.html#set-operation-examples)
///  * [`RangeMapBlaze` Comparisons](#RangeMapBlaze-comparisons)
///  * [Additional Examples](#additional-examples)
///
/// # `RangeMapBlaze` Constructors
///
/// You can also create `RangeMapBlaze`'s from unsorted and overlapping integers (or ranges).
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
/// | [`from_sorted_disjoint`][3]/[`into_range_set_blaze`][3] | [`SortedDisjointMap`] iterator |               |
/// | [`from`][4] /[`into`][4]                    | array of integers            |                          |
///
///
/// [`BTreeMap`]: alloc::collections::BTreeMap
/// [`new`]: RangeMapBlaze::new
/// [`default`]: RangeMapBlaze::default
/// [1]: struct.RangeMapBlaze.html#impl-FromIterator<T, V, VR>-for-RangeMapBlaze<T, V>
/// [2]: struct.RangeMapBlaze.html#impl-FromIterator<RangeInclusive<T, V, VR>>-for-RangeMapBlaze<T, V>
/// [3]: RangeMapBlaze::from_sorted_disjoint
/// [4]: RangeMapBlaze::from
/// [5]: RangeMapBlaze::from_slice()
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
/// On a representative benchmark, the speed up was 7×.
/// The method works by scanning the input for blocks of consecutive integers, and then using `from_iter` on the results.
/// Where available, it uses SIMD instructions. It is nightly only and enabled by the `from_slice` feature.
///
/// ## Constructor Examples
///
/// ```
/// use range_set_blaze::prelude::*;
///
/// // Create an empty set with 'new' or 'default'.
/// let a0 = RangeMapBlaze::<i32>::new();
/// let a1 = RangeMapBlaze::<i32>::default();
/// assert!(a0 == a1 && a0.is_empty());
///
/// // 'from_iter'/'collect': From an iterator of integers.
/// // Duplicates and out-of-order elements are fine.
/// let a0 = RangeMapBlaze::from_iter([3, 2, 1, 100, 1]);
/// let a1: RangeMapBlaze<i32> = [3, 2, 1, 100, 1].into_iter().collect();
/// assert!(a0 == a1 && a0.to_string() == "1..=3, 100..=100");
///
/// // 'from_iter'/'collect': From an iterator of inclusive ranges, start..=end.
/// // Overlapping, out-of-order, and empty ranges are fine.
/// #[allow(clippy::reversed_empty_ranges)]
/// let a0 = RangeMapBlaze::from_iter([1..=2, 2..=2, -10..=-5, 1..=0]);
/// #[allow(clippy::reversed_empty_ranges)]
/// let a1: RangeMapBlaze<i32> = [1..=2, 2..=2, -10..=-5, 1..=0].into_iter().collect();
/// assert!(a0 == a1 && a0.to_string() == "-10..=-5, 1..=2");
///
/// // 'from_slice': From any array-like collection of integers.
/// // Nightly-only, but faster than 'from_iter'/'collect' on integers.
/// #[cfg(feature = "from_slice")]
/// let a0 = RangeMapBlaze::from_slice(vec![3, 2, 1, 100, 1]);
/// #[cfg(feature = "from_slice")]
/// assert!(a0.to_string() == "1..=3, 100..=100");
///
/// // If we know the ranges are already sorted and disjoint,
/// // we can avoid work and use 'from'/'into'.
/// let a0 = RangeMapBlaze::from_sorted_disjoint(CheckSortedDisjoint::from([-10..=-5, 1..=2]));
/// let a1: RangeMapBlaze<i32> = CheckSortedDisjoint::from([-10..=-5, 1..=2]).into_range_set_blaze();
/// assert!(a0 == a1 && a0.to_string() == "-10..=-5, 1..=2");
///
/// // For compatibility with `BTreeSet`, we also support
/// // 'from'/'into' from arrays of integers.
/// let a0 = RangeMapBlaze::from([3, 2, 1, 100, 1]);
/// let a1: RangeMapBlaze<i32> = [3, 2, 1, 100, 1].into();
/// assert!(a0 == a1 && a0.to_string() == "1..=3, 100..=100");
/// ```
///
/// # `RangeMapBlaze` Set Operations
///
/// You can perform set operations on `RangeMapBlaze`s using operators.
///
/// | Set Operation           | Operator                   |  Multiway Method |
/// |-------------------|-------------------------|-------------------------|
/// | union       |  `a` &#124; `b`                     | `[a, b, c].`[`union`]`()` |
/// | intersection       |  `a & b`                     | `[a, b, c].`[`intersection`]`()` |
/// | difference       |  `a - b`                     | *n/a* |
/// | symmetric difference       |  `a ^ b`                     | *n/a* |
/// | complement       |  `!a`                     | *n/a* |
///
/// `RangeMapBlaze` also implements many other methods, such as [`insert`], [`pop_first`] and [`split_off`]. Many of
/// these methods match those of `BTreeSet`.
///
/// [`union`]: trait.MultiwayRangeMapBlaze.html#method.union
/// [`intersection`]: trait.MultiwayRangeMapBlaze.html#method.intersection
/// [`insert`]: RangeMapBlaze::insert
/// [`pop_first`]: RangeMapBlaze::pop_first
/// [`split_off`]: RangeMapBlaze::split_off
///
///
/// ## Set Operation Performance
///
/// Every operation is implemented as
/// 1. a single pass over the sorted & disjoint ranges
/// 2. the construction of a new `RangeMapBlaze`
///
/// Thus, applying multiple operators creates intermediate
/// `RangeMapBlaze`'s. If you wish, you can avoid these intermediate
/// `RangeMapBlaze`'s by switching to the [`SortedDisjointMap`] API. The last example below
/// demonstrates this.
///
/// ## Set Operation Examples
///
/// ```
/// use range_set_blaze::prelude::*;
///
/// let a = RangeMapBlaze::from_iter([1..=2, 5..=100]);
/// let b = RangeMapBlaze::from_iter([2..=6]);
///
/// // Union of two 'RangeMapBlaze's.
/// let result = &a | &b;
/// // Alternatively, we can take ownership via 'a | b'.
/// assert_eq!(result.to_string(), "1..=100");
///
/// // Intersection of two 'RangeMapBlaze's.
/// let result = &a & &b; // Alternatively, 'a & b'.
/// assert_eq!(result.to_string(), "2..=2, 5..=6");
///
/// // Set difference of two 'RangeMapBlaze's.
/// let result = &a - &b; // Alternatively, 'a - b'.
/// assert_eq!(result.to_string(), "1..=1, 7..=100");
///
/// // Symmetric difference of two 'RangeMapBlaze's.
/// let result = &a ^ &b; // Alternatively, 'a ^ b'.
/// assert_eq!(result.to_string(), "1..=1, 3..=4, 7..=100");
///
/// // complement of a 'RangeMapBlaze'.
/// let result = !&a; // Alternatively, '!a'.
/// assert_eq!(
///     result.to_string(),
///     "-2147483648..=0, 3..=4, 101..=2147483647"
/// );
///
/// // Multiway union of 'RangeMapBlaze's.
/// let c = RangeMapBlaze::from_iter([2..=2, 6..=200]);
/// let result = [&a, &b, &c].union();
/// assert_eq!(result.to_string(), "1..=200");
///
/// // Multiway intersection of 'RangeMapBlaze's.
/// let result = [&a, &b, &c].intersection();
/// assert_eq!(result.to_string(), "2..=2, 6..=6");
///
/// // Applying multiple operators
/// let result0 = &a - (&b | &c); // Creates an intermediate 'RangeMapBlaze'.
/// // Alternatively, we can use the 'SortedDisjointMap' API and avoid the intermediate 'RangeMapBlaze'.
/// let result1 = RangeMapBlaze::from_sorted_disjoint(a.ranges() - (b.ranges() | c.ranges()));
/// assert!(result0 == result1 && result0.to_string() == "1..=1");
/// ```
/// # `RangeMapBlaze` Comparisons
///
/// We can compare `RangeMapBlaze`s using the following operators:
/// `<`, `<=`, `>`, `>=`.  Following the convention of `BTreeSet`,
/// these comparisons are lexicographic. See [`cmp`] for more examples.
///
/// Use the [`is_subset`] and [`is_superset`] methods to check if one `RangeMapBlaze` is a subset
/// or superset of another.
///
/// Use `==`, `!=` to check if two `RangeMapBlaze`s are equal or not.
///
/// [`BTreeSet`]: alloc::collections::BTreeSet
/// [`is_subset`]: RangeMapBlaze::is_subset
/// [`is_superset`]: RangeMapBlaze::is_superset
/// [`cmp`]: RangeMapBlaze::cmp
///
/// # Additional Examples
///
/// See the [module-level documentation] for additional examples.
///
/// [module-level documentation]: index.html
pub struct RangeMapBlaze<T: Integer, V: ValueOwned> {
    len: <T as Integer>::SafeLen,
    pub(crate) btree_map: BTreeMap<T, EndValue<T, V>>,
}

impl<T: Integer, V: ValueOwned + fmt::Debug> fmt::Debug for RangeMapBlaze<T, V> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.range_values().to_string())
    }
}

impl<T: Integer, V: ValueOwned + fmt::Debug> fmt::Display for RangeMapBlaze<T, V> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.range_values().to_string())
    }
}

impl<T: Integer, V: ValueOwned> RangeMapBlaze<T, V> {
    /// Gets an (double-ended) iterator that visits the integer elements in the [`RangeMapBlaze`] in
    /// ascending and/or descending order.
    ///
    /// Also see the [`RangeMapBlaze::ranges`] method.
    ///
    /// # Examples
    ///
    /// ```
    /// use range_set_blaze::RangeMapBlaze;
    ///
    /// let set = RangeMapBlaze::from_iter([1..=3]);
    /// let mut set_iter = set.iter();
    /// assert_eq!(set_iter.next(), Some(1));
    /// assert_eq!(set_iter.next(), Some(2));
    /// assert_eq!(set_iter.next(), Some(3));
    /// assert_eq!(set_iter.next(), None);
    /// ```
    ///
    /// Values returned by `.next()` are in ascending order.
    /// Values returned by `.next_back()` are in descending order.
    ///
    /// ```
    /// use range_set_blaze::RangeMapBlaze;
    ///
    /// let set = RangeMapBlaze::from_iter([3, 1, 2]);
    /// let mut set_iter = set.iter();
    /// assert_eq!(set_iter.next(), Some(1));
    /// assert_eq!(set_iter.next_back(), Some(3));
    /// assert_eq!(set_iter.next(), Some(2));
    /// assert_eq!(set_iter.next_back(), None);
    /// ```
    // cmk00 implement this
    pub fn iter(&self) -> IterMap<'_, T, V, &V, RangeValuesIter<'_, T, V>> {
        // If the user asks for an iter, we give them a RangesIter iterator
        // and we iterate that one integer at a time.
        IterMap {
            option_range_value_front: None,
            option_range_value_back: None,
            iter: self.range_values(),
            phantom0: PhantomData,
            phantom1: PhantomData,
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
    /// use range_set_blaze::RangeMapBlaze;
    ///
    /// let mut set = RangeMapBlaze::new();
    /// assert_eq!(set.first(), None);
    /// set.insert(1);
    /// assert_eq!(set.first(), Some(1));
    /// set.insert(2);
    /// assert_eq!(set.first(), Some(1));
    /// ```
    #[must_use]
    pub fn first_key_value(&self) -> Option<(T, &V)> {
        self.btree_map
            .first_key_value()
            .map(|(k, end_value)| (*k, &end_value.value))
    }

    /// Returns the element in the set, if any, that is equal to
    /// the value.
    ///
    /// # Examples
    ///
    /// ```
    /// use range_set_blaze::RangeMapBlaze;
    ///
    /// let set = RangeMapBlaze::from_iter([1, 2, 3]);
    /// assert_eq!(set.get(2), Some(2));
    /// assert_eq!(set.get(4), None);
    /// ```
    pub fn get(&self, key: T) -> Option<&V> {
        assert!(
            key <= T::safe_max_value(),
            "key must be <= T::safe_max_value()"
        );
        self.btree_map
            .range(..=key)
            .next_back()
            .and_then(|(_, end_value)| {
                if key <= end_value.end {
                    Some(&end_value.value)
                } else {
                    None
                }
            })
    }

    /// Returns the last element in the set, if any.
    /// This element is always the maximum of all elements in the set.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// use range_set_blaze::RangeMapBlaze;
    ///
    /// let mut set = RangeMapBlaze::new();
    /// assert_eq!(set.last(), None);
    /// set.insert(1);
    /// assert_eq!(set.last(), Some(1));
    /// set.insert(2);
    /// assert_eq!(set.last(), Some(2));
    /// ```
    #[must_use]
    pub fn last_key_value(&self) -> Option<(T, &V)> {
        self.btree_map
            .last_key_value()
            .map(|(_, end_value)| (end_value.end, &end_value.value))
    }

    // cmk look at HashMap, etc for last related methods to see if when return the value.

    /// Create a [`RangeMapBlaze`] from a [`SortedDisjointMap`] iterator.
    ///
    /// *For more about constructors and performance, see [`RangeMapBlaze` Constructors](struct.RangeMapBlaze.html#RangeMapBlaze-constructors).*
    ///
    /// # Examples
    ///
    /// ```
    /// use range_set_blaze::prelude::*;
    ///
    /// let a0 = RangeMapBlaze::from_sorted_disjoint(CheckSortedDisjoint::from([-10..=-5, 1..=2]));
    /// let a1: RangeMapBlaze<i32> = CheckSortedDisjoint::from([-10..=-5, 1..=2]).into_range_set_blaze();
    /// assert!(a0 == a1 && a0.to_string() == "-10..=-5, 1..=2");
    /// ```
    pub fn from_sorted_disjoint_map<'a, VR, I>(iter: I) -> Self
    where
        T: 'a,
        V: 'a,
        VR: CloneBorrow<V> + 'a,
        I: SortedDisjointMap<'a, T, V, VR>,
    {
        let mut iter_with_len = SortedDisjointWithLenSoFarMap::from(iter);
        let btree_map = BTreeMap::from_iter(&mut iter_with_len);
        RangeMapBlaze {
            btree_map,
            len: iter_with_len.len_so_far(),
        }
    }

    /// Creates a [`RangeMapBlaze`] from a collection of integers. It is typically many
    /// times faster than [`from_iter`][1]/[`collect`][1].
    /// On a representative benchmark, the speed up was 7×.
    ///
    /// **Warning: Requires the nightly compiler. Also, you must enable the `from_slice`
    /// feature in your `Cargo.toml`. For example, with the command:**
    /// ```bash
    ///  cargo add range-set-blaze --features "from_slice"
    /// ```
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
    /// *For more about constructors and performance, see [`RangeMapBlaze` Constructors](struct.RangeMapBlaze.html#RangeMapBlaze-constructors).*
    ///
    /// # Examples
    ///
    /// ```
    /// use range_set_blaze::RangeMapBlaze;
    ///
    /// let a0 = RangeMapBlaze::from_slice(&[3, 2, 1, 100, 1]); // reference to a slice
    /// let a1 = RangeMapBlaze::from_slice([3, 2, 1, 100, 1]);   // array
    /// let a2 = RangeMapBlaze::from_slice(vec![3, 2, 1, 100, 1]); // vector
    /// assert!(a0 == a1 && a1 == a2 && a0.to_string() == "1..=3, 100..=100");
    /// ```
    /// [1]: struct.RangeMapBlaze.html#impl-FromIterator<T, V, VR>-for-RangeMapBlaze<T, V>
    // cmk
    // #[cfg(feature = "from_slice")]
    // #[inline]
    // pub fn from_slice(slice: impl AsRef<[T]>) -> Self {
    //     T::from_slice(slice)
    // }

    fn _len_slow(&self) -> <T as Integer>::SafeLen {
        RangeMapBlaze::btree_map_len(&self.btree_map)
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
    /// use range_set_blaze::RangeMapBlaze;
    ///
    /// let mut a = RangeMapBlaze::from_iter([1..=3]);
    /// let mut b = RangeMapBlaze::from_iter([3..=5]);
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
        for range_values in other.range_values() {
            let range = range_values.range;
            let v = range_values.value.borrow_clone();
            self.internal_add(range, v);
        }
        other.clear();
    }

    /// Clears the set, removing all integer elements.
    ///
    /// # Examples
    ///
    /// ```
    /// use range_set_blaze::RangeMapBlaze;
    ///
    /// let mut v = RangeMapBlaze::new();
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
    /// use range_set_blaze::RangeMapBlaze;
    ///
    /// let mut v = RangeMapBlaze::new();
    /// assert!(v.is_empty());
    /// v.insert(1);
    /// assert!(!v.is_empty());
    /// ```
    #[must_use]
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.range_values_len() == 0
    }

    /// Returns `true` if the set is a subset of another,
    /// i.e., `other` contains at least all the elements in `self`.
    ///
    /// # Examples
    ///
    /// ```
    /// use range_set_blaze::RangeMapBlaze;
    ///
    /// let sup = RangeMapBlaze::from_iter([1..=3]);
    /// let mut set = RangeMapBlaze::new();
    ///
    /// assert_eq!(set.is_subset(&sup), true);
    /// set.insert(2);
    /// assert_eq!(set.is_subset(&sup), true);
    /// set.insert(4);
    /// assert_eq!(set.is_subset(&sup), false);
    /// ```
    // cmk
    // #[must_use]
    // #[inline]
    // pub fn is_subset(&self, other: &RangeMapBlaze<T, V>) -> bool {
    //     // Add a fast path
    //     if self.len() > other.len() {
    //         return false;
    //     }
    //     self.ranges().is_subset(other.ranges())
    // }

    /// Returns `true` if the set is a superset of another,
    /// i.e., `self` contains at least all the elements in `other`.
    ///
    /// # Examples
    ///
    /// ```
    /// use range_set_blaze::RangeMapBlaze;
    ///
    /// let sub = RangeMapBlaze::from_iter([1, 2]);
    /// let mut set = RangeMapBlaze::new();
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
    // cmk
    // #[must_use]
    // pub fn is_superset(&self, other: &RangeMapBlaze<T, V>) -> bool {
    //     other.is_subset(self)
    // }

    /// Returns `true` if the set contains an element equal to the value.
    ///
    /// # Examples
    ///
    /// ```
    /// use range_set_blaze::RangeMapBlaze;
    ///
    /// let set = RangeMapBlaze::from_iter([1, 2, 3]);
    /// assert_eq!(set.contains(1), true);
    /// assert_eq!(set.contains(4), false);
    /// ```
    pub fn contains_key(&self, key: T) -> bool {
        assert!(
            key <= T::safe_max_value(),
            "value must be <= T::safe_max_value()"
        );
        self.btree_map
            .range(..=key)
            .next_back()
            .map_or(false, |(_, end_value)| key <= end_value.end)
    }

    /// Returns `true` if `self` has no elements in common with `other`.
    /// This is equivalent to checking for an empty intersection.
    ///
    /// # Examples
    ///
    /// ```
    /// use range_set_blaze::RangeMapBlaze;
    ///
    /// let a = RangeMapBlaze::from_iter([1..=3]);
    /// let mut b = RangeMapBlaze::new();
    ///
    /// assert_eq!(a.is_disjoint(&b), true);
    /// b.insert(4);
    /// assert_eq!(a.is_disjoint(&b), true);
    /// b.insert(1);
    /// assert_eq!(a.is_disjoint(&b), false);
    /// ```
    // cmk
    // #[must_use]
    // #[inline]
    // pub fn is_disjoint(&self, other: &RangeMapBlaze<T, V>) -> bool {
    //     self.ranges().is_disjoint(other.ranges())
    // }

    // cmk might be able to shorten code by combining cases
    fn delete_extra(&mut self, internal_range: &RangeInclusive<T>) {
        let (start, end) = internal_range.clone().into_inner();
        let mut after = self.btree_map.range_mut(start..);
        let (start_after, end_value_after) = after.next().unwrap(); // there will always be a next
        debug_assert!(start == *start_after && end == end_value_after.end);

        let mut end_new = end;
        let mut end_new_same_val = end;
        let delete_list = after
            .map_while(|(start_delete, end_value_delete)| {
                // same values
                if end_value_after.value == end_value_delete.value {
                    // must check this in two parts to avoid overflow
                    if *start_delete <= end || *start_delete <= end + T::one() {
                        end_new_same_val = max(end_new_same_val, end_value_delete.end);
                        end_new = max(end_new, end_value_delete.end);
                        self.len -= T::safe_len(&(*start_delete..=end_value_delete.end));
                        Some(*start_delete)
                    } else {
                        None
                    }
                // different values
                } else if *start_delete <= end {
                    end_new = max(end_new, end_value_delete.end);
                    self.len -= T::safe_len(&(*start_delete..=end_value_delete.end));
                    Some(*start_delete)
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();
        if end >= end_new {
            for start in delete_list {
                self.btree_map.remove(&start);
            }
        } else if end_new_same_val > end {
            // last item is the same as the new and extends beyond the new
            self.len += T::safe_len(&(end..=end_new - T::one()));
            debug_assert!(*start_after <= end_new);
            end_value_after.end = end_new;
            for start in delete_list {
                self.btree_map.remove(&start);
            }
        } else {
            // last item extends beyond the new but has a different value.
            for &start in delete_list[0..delete_list.len() - 1].iter() {
                self.btree_map.remove(&start);
                // take the last one
            }
            let last = self
                .btree_map
                .remove(&delete_list[delete_list.len() - 1])
                .unwrap(); // there will always be a last
            let last_end = last.end;
            debug_assert!(end + T::one() <= last.end); // real assert
            self.btree_map.insert(end + T::one(), last);
            self.len += T::safe_len(&(end + T::one()..=last_end));
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
    /// use range_set_blaze::RangeMapBlaze;
    ///
    /// let mut set = RangeMapBlaze::new();
    ///
    /// assert_eq!(set.insert(2), true);
    /// assert_eq!(set.insert(2), false);
    /// assert_eq!(set.len(), 1usize);
    /// ```
    pub fn insert(&mut self, key: T, value: V) -> bool {
        let len_before = self.len;
        self.internal_add(key..=key, value);
        self.len != len_before
    }

    // cmk also define insert_under

    /// Constructs an iterator over a sub-range of elements in the set.
    ///
    /// Not to be confused with [`RangeMapBlaze::ranges`], which returns an iterator over the ranges in the set.
    ///
    /// The simplest way is to use the range syntax `min..max`, thus `range(min..max)` will
    /// yield elements from min (inclusive) to max (exclusive).
    /// The range may also be entered as `(Bound<T, V, VR>, Bound<T, V, VR>)`, so for example
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
    /// use range_set_blaze::RangeMapBlaze;
    /// use core::ops::Bound::Included;
    ///
    /// let mut set = RangeMapBlaze::new();
    /// set.insert(3);
    /// set.insert(5);
    /// set.insert(8);
    /// for elem in set.range((Included(4), Included(8))) {
    ///     println!("{elem}");
    /// }
    /// assert_eq!(Some(5), set.range(4..).next());
    /// ```
    // cmk
    // pub fn range<R>(&self, range: R) -> IntoIter<T, V, VR>
    // where
    //     R: RangeBounds<T, V, VR>,
    // {
    //     let start = match range.start_bound() {
    //         Bound::Included(n) => *n,
    //         Bound::Excluded(n) => *n + T::one(),
    //         Bound::Unbounded => T::min_value(),
    //     };
    //     let end = match range.end_bound() {
    //         Bound::Included(n) => *n,
    //         Bound::Excluded(n) => *n - T::one(),
    //         Bound::Unbounded => T::safe_max_value(),
    //     };
    //     assert!(start <= end);

    //     let bounds = CheckSortedDisjoint::from([start..=end]);
    //     RangeMapBlaze::from_sorted_disjoint(self.ranges() & bounds).into_iter()
    // }

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
    /// use range_set_blaze::RangeMapBlaze;
    ///
    /// let mut set = RangeMapBlaze::new();
    ///
    /// assert_eq!(set.ranges_insert(2..=5), true);
    /// assert_eq!(set.ranges_insert(5..=6), true);
    /// assert_eq!(set.ranges_insert(3..=4), false);
    /// assert_eq!(set.len(), 5usize);
    /// ```
    pub fn ranges_insert(&mut self, range: RangeInclusive<T>, value: V) -> bool {
        let len_before = self.len;
        self.internal_add(range, value);
        self.len != len_before
    }

    /// If the set contains an element equal to the value, removes it from the
    /// set and drops it. Returns whether such an element was present.
    ///
    /// # Examples
    ///
    /// ```
    /// use range_set_blaze::RangeMapBlaze;
    ///
    /// let mut set = RangeMapBlaze::new();
    ///
    /// set.insert(2);
    /// assert!(set.remove(2));
    /// assert!(!set.remove(2));
    /// ```
    pub fn remove(&mut self, key: T) -> Option<V> {
        assert!(
            key <= T::safe_max_value(),
            "value must be <= T::safe_max_value()"
        );

        // The code can have only one mutable reference to self.btree_map.
        let Some((start_ref, end_value_mut)) = self.btree_map.range_mut(..=key).next_back() else {
            return None;
        };

        if end_value_mut.end < key {
            return None;
        }
        let start = *start_ref;
        let end = end_value_mut.end;
        let value = end_value_mut.value.clone();
        if start < key {
            end_value_mut.end = key - T::one();
            // special, special case if value == end
            if key == end {
                self.len -= <T::SafeLen>::one();
                return Some(value);
            }
        }

        self.len -= <T::SafeLen>::one();
        if start == key {
            // unwrap is safe
            self.btree_map.remove(&start);
            // cmk should recycle this value
        };

        if key < end {
            let end_value = EndValue {
                end,
                value: value.clone(),
            };
            self.btree_map.insert(key + T::one(), end_value);
        }
        Some(value)
    }

    // /// Splits the collection into two at the value. Returns a new collection
    // /// with all elements greater than or equal to the value.
    // ///
    // /// # Examples
    // ///
    // /// Basic usage:
    // ///
    // /// ```
    // /// use range_set_blaze::RangeMapBlaze;
    // ///
    // /// let mut a = RangeMapBlaze::new();
    // /// a.insert(1);
    // /// a.insert(2);
    // /// a.insert(3);
    // /// a.insert(17);
    // /// a.insert(41);
    // ///
    // /// let b = a.split_off(3);
    // ///
    // /// assert_eq!(a, RangeMapBlaze::from_iter([1, 2]));
    // /// assert_eq!(b, RangeMapBlaze::from_iter([3, 17, 41]));
    // /// ```
    // cmk
    // pub fn split_off(&mut self, value: T) -> Self {
    //     assert!(
    //         value <= T::safe_max_value(),
    //         "value must be <= T::safe_max_value()"
    //     );
    //     let old_len = self.len;
    //     let mut b = self.btree_map.split_off(&value);
    //     if let Some(mut last_entry) = self.btree_map.last_entry() {
    //         // Can assume start strictly less than value
    //         let end_value_ref = last_entry.get_mut();
    //         if value <= end_value_ref.end {
    //             b.insert(value, *end_value_ref);
    //             end_value_ref.end = value - T::one();
    //         }
    //     }

    //     // Find the length of the smaller map and then length of self & b.
    //     let b_len = if self.btree_map.len() < b.len() {
    //         self.len = RangeMapBlaze::btree_map_len(&self.btree_map);
    //         old_len - self.len
    //     } else {
    //         let b_len = RangeMapBlaze::btree_map_len(&b);
    //         self.len = old_len - b_len;
    //         b_len
    //     };
    //     RangeMapBlaze {
    //         btree_map: b,
    //         len: b_len,
    //     }
    // }

    #[allow(dead_code)] // cmk
    fn btree_map_len(btree_map: &BTreeMap<T, EndValue<T, V>>) -> T::SafeLen {
        btree_map.iter().fold(
            <T as Integer>::SafeLen::zero(),
            |acc, (start, end_value)| acc + T::safe_len(&(*start..=end_value.end)),
        )
    }

    // /// Removes and returns the element in the set, if any, that is equal to
    // /// the value.
    // ///
    // /// # Examples
    // ///
    // /// ```
    // /// use range_set_blaze::RangeMapBlaze;
    // ///
    // /// let mut set = RangeMapBlaze::from_iter([1, 2, 3]);
    // /// assert_eq!(set.take(2), Some(2));
    // /// assert_eq!(set.take(2), None);
    // /// ```
    // cmk
    // pub fn take(&mut self, value: T) -> Option<T, V, VR> {
    //     if self.remove(value) {
    //         Some(value)
    //     } else {
    //         None
    //     }
    // }

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
    /// use range_set_blaze::RangeMapBlaze;
    ///
    /// let mut set = RangeMapBlaze::new();
    /// assert!(set.replace(5).is_none());
    /// assert!(set.replace(5).is_some());
    /// ```
    // cmk
    //pub fn replace(&mut self, value: T) -> Option<T> {
    //     if self.insert(value) {
    //         None
    //     } else {
    //         Some(value)
    //     }
    // }

    // fn internal_add_chatgpt(&mut self, range: RangeInclusive<T, V, VR>) {
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

    #[inline]
    fn has_gap(end_before: T, start: T) -> bool {
        match end_before.checked_add(&T::one()) {
            Some(end_before_succ) => end_before_succ < start,
            None => false,
        }
    }

    // https://stackoverflow.com/questions/49599833/how-to-find-next-smaller-key-in-btreemap-btreeset
    // https://stackoverflow.com/questions/35663342/how-to-modify-partially-remove-a-range-from-a-btreemap
    // cmk2 might be able to shorten code by combining cases
    // FUTURE: would be nice of BTreeMap to have a partition_point function that returns two iterators
    pub(crate) fn internal_add(&mut self, range: RangeInclusive<T>, value: V) {
        let (start, end) = range.clone().into_inner();
        assert!(
            end <= T::safe_max_value(),
            "end must be <= T::safe_max_value()"
        );

        // === case: empty
        if end < start {
            return;
        }
        let mut before_iter = self.btree_map.range_mut(..=start).rev();

        // === case: no before
        let Some((start_before, end_value_before)) = before_iter.next() else {
            // no before, so must be first
            self.internal_add2(&range, value);
            // You must return or break out of the current block after handling the failure case
            return;
        };

        let start_before = *start_before;
        let end_before = end_value_before.end;

        // === case: gap between before and new
        if Self::has_gap(end_before, start) {
            // there is a gap between the before and the new
            // ??? aa...
            self.internal_add2(&range, value);
            return;
        }

        let before_contains_new = end_before >= end;
        let same_value = value == end_value_before.value;

        // === case: same value and before contains new
        if before_contains_new && same_value {
            // same value, so do nothing
            // AAAAA
            //  aaa
            return;
        };

        // === case: same value and new extends beyond before
        if !before_contains_new && same_value {
            // same value, so just extend the before
            // AAA
            //  aaaa...
            self.len += T::safe_len(&(end_before..=end - T::one()));
            debug_assert!(start_before <= end); // real assert
            end_value_before.end = end;
            self.delete_extra(&(start_before..=end));
            return;
        }

        // Thus, the values are different

        let same_start = start == start_before;

        // === case: new goes beyond before and different values
        if !before_contains_new && !same_value && same_start {
            // Thus, values are different, before contains new, and they start together

            let interesting_before_before = match before_iter.next() {
                Some(bb) if bb.1.end + T::one() == start && bb.1.value == value => Some(bb),
                _ => None,
            };

            // === case: values are different, new extends beyond before, and they start together and an interesting before-before
            // an interesting before-before: something before before, touching and with the same value as new
            if let Some(bb) = interesting_before_before {
                debug_assert!(!before_contains_new && !same_value && same_start);

                // AABBBB
                //   aaaaaaa
                // AAAAAAAAA
                self.len += T::safe_len(&(bb.1.end + T::one()..=end));
                let bb_start = *bb.0;
                debug_assert!(bb_start <= end); // real assert
                bb.1.end = end;
                self.delete_extra(&(bb_start..=end));
                return;
            }

            // === case: values are different, they start together but new ends later and no interesting before-before
            debug_assert!(!same_value && same_start && interesting_before_before.is_none());

            // ^BBBB
            //  aaaaaaa
            // ^AAAAAAA
            debug_assert!(end_before < end); // real assert
            self.len += T::safe_len(&(end_before + T::one()..=end));
            end_value_before.end = end;
            end_value_before.value = value;
            self.delete_extra(&range);
            return;
        }
        if !before_contains_new && !same_value && !same_start {
            // different value, so must trim the before and then insert the new
            // BBB
            //  aaaa...
            if end_before >= start {
                self.len -= T::safe_len(&(start..=end_before));
                debug_assert!(start_before <= start - T::one()); // real assert
                end_value_before.end = start - T::one(); // cmk overflow danger?
            }
            self.internal_add2(&range, value);
            return;
        }

        // Thus, the values are different and before contains new
        debug_assert!(before_contains_new && !same_value);

        let same_end = end == end_before;

        // === case: values are different and new is surrounded by before
        if !same_start && !same_end {
            debug_assert!(before_contains_new && !same_value);
            debug_assert!(start_before < start && end < end_before);
            // Different values still ...
            // The new starts later and ends before,
            // BBBBBB
            //   aaa
            // BBAAAB
            //  so trim the before and then insert two
            debug_assert!(start_before <= start - T::one()); // real assert
            end_value_before.end = start - T::one();
            let before_value = end_value_before.value.clone();
            debug_assert!(start <= end); // real assert
            self.btree_map.insert(start, EndValue { end, value });
            debug_assert!(end + T::one() <= end_before); // real assert
            self.btree_map.insert(
                end + T::one(),
                EndValue {
                    end: end_before,
                    value: before_value,
                },
            );
            return;
        }

        // === case: values are different, new instead of before and they end together
        if !same_start && same_end {
            debug_assert!(before_contains_new && !same_value);
            debug_assert!(start_before < start && end == end_before);
            // Different values still ...
            // The new starts later but they end together,
            // BBBBB???
            //   aaa
            // BBAAA???
            //  so trim the before and then insert the new.
            debug_assert!(start_before <= start - T::one()); // real assert
            end_value_before.end = start - T::one();
            debug_assert!(start <= end); // real assert
            self.btree_map.insert(start, EndValue { end, value });
            self.delete_extra(&(start..=end));
            return;
        }

        // Thus, values are different, before contains new, and they start together

        let interesting_before_before = match before_iter.next() {
            Some(bb) if bb.1.end + T::one() == start && bb.1.value == value => Some(bb),
            _ => None,
        };

        // === case: values are different, before contains new, and they start together and an interesting before-before
        // an interesting before-before: something before before, touching and with the same value as new
        if let Some(bb) = interesting_before_before {
            debug_assert!(before_contains_new && !same_value && same_start);

            // AABBBB???
            //   aaaa
            // AAAAAA???
            self.len += T::safe_len(&(bb.1.end + T::one()..=end));
            let bb_start = *bb.0;
            debug_assert!(bb_start <= end); // real assert
            bb.1.end = end;
            self.delete_extra(&(bb_start..=end));
            return;
        }

        // === case: values are different, they start and end together and no interesting before-before
        if same_end {
            debug_assert!(!same_value && same_start && interesting_before_before.is_none());

            // ^BBBB???
            //  aaaa
            // ^AAAA???
            end_value_before.value = value;
            self.delete_extra(&(start_before..=end));
            return;
        }

        // === case: values are different, they start together, new ends first, and no interesting before-before
        {
            debug_assert!(
                !same_value
                    && same_start
                    && end < end_before
                    && interesting_before_before.is_none()
            );

            // ^BBBB
            //  aaa
            // ^AAAB
            let value_before = core::mem::replace(&mut end_value_before.value, value);
            debug_assert!(start_before <= end); // real assert
            end_value_before.end = end;
            debug_assert!(end + T::one() <= end_before); // real assert
            self.btree_map.insert(
                end + T::one(),
                EndValue {
                    end: end_before,
                    value: value_before,
                },
            );
        }
    }

    fn internal_add2(&mut self, internal_range: &RangeInclusive<T>, value: V) {
        let (start, end) = internal_range.clone().into_inner();
        let end_value = EndValue { end, value };
        debug_assert!(start <= end_value.end); // real assert
        let was_there = self.btree_map.insert(start, end_value);
        debug_assert!(was_there.is_none()); // no range with the same start should be there
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
    /// use range_set_blaze::RangeMapBlaze;
    ///
    /// let mut v = RangeMapBlaze::new();
    /// assert_eq!(v.len(), 0usize);
    /// v.insert(1);
    /// assert_eq!(v.len(), 1usize);
    ///
    /// let v = RangeMapBlaze::from_iter([
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

    /// Makes a new, empty [`RangeMapBlaze`].
    ///
    /// # Examples
    ///
    /// ```
    /// # #![allow(unused_mut)]
    /// use range_set_blaze::RangeMapBlaze;
    ///
    /// let mut set: RangeMapBlaze<i32> = RangeMapBlaze::new();
    /// ```
    #[must_use]
    pub fn new() -> Self {
        RangeMapBlaze {
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
    /// use range_set_blaze::RangeMapBlaze;
    ///
    /// let mut set = RangeMapBlaze::new();
    ///
    /// set.insert(1);
    /// while let Some(n) = set.pop_first() {
    ///     assert_eq!(n, 1);
    /// }
    /// assert!(set.is_empty());
    /// ```
    // cmk doc that often must clone
    pub fn pop_first(&mut self) -> Option<(T, V)>
    where
        V: Clone,
    {
        if let Some(entry) = self.btree_map.first_entry() {
            let (start, end_value) = entry.remove_entry();
            self.len -= T::safe_len(&(start..=end_value.end));
            if start != end_value.end {
                let start = start + T::one();
                self.len += T::safe_len(&(start..=end_value.end));
                let value = end_value.value.clone();
                debug_assert!(start <= end_value.end); // real assert
                self.btree_map.insert(start, end_value);
                Some((start, value))
            } else {
                Some((start, end_value.value))
            }
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
    /// use range_set_blaze::RangeMapBlaze;
    ///
    /// let mut set = RangeMapBlaze::new();
    ///
    /// set.insert(1);
    /// while let Some(n) = set.pop_last() {
    ///     assert_eq!(n, 1);
    /// }
    /// assert!(set.is_empty());
    /// ```
    // cmk
    // pub fn pop_last(&mut self) -> Option<T, V, VR> {
    //     let Some(mut entry) = self.btree_map.last_entry() else {
    //         return None;
    //     };

    //     let start = *entry.key();
    //     let end_value = entry.get_mut();
    //     let result = *end_value;
    //     self.len -= T::safe_len(&(start..=end_value.end));
    //     if start == end_value.end {
    //         entry.remove_entry();
    //     } else {
    //         end_value.end -= T::one();
    //         self.len += T::safe_len(&(start..=end_value.end));
    //     }
    //     Some(result.end)
    // }

    /// An iterator that visits the ranges in the [`RangeMapBlaze`],
    /// i.e., the integers as sorted & disjoint ranges.
    ///
    /// Also see [`RangeMapBlaze::iter`] and [`RangeMapBlaze::into_ranges`].
    ///
    /// # Examples
    ///
    /// ```
    /// use range_set_blaze::RangeMapBlaze;
    ///
    /// let set = RangeMapBlaze::from_iter([10..=20, 15..=25, 30..=40]);
    /// let mut ranges = set.ranges();
    /// assert_eq!(ranges.next(), Some(10..=25));
    /// assert_eq!(ranges.next(), Some(30..=40));
    /// assert_eq!(ranges.next(), None);
    /// ```
    ///
    /// Values returned by the iterator are returned in ascending order:
    ///
    /// ```
    /// use range_set_blaze::RangeMapBlaze;
    ///
    /// let set = RangeMapBlaze::from_iter([30..=40, 15..=25, 10..=20]);
    /// let mut ranges = set.ranges();
    /// assert_eq!(ranges.next(), Some(10..=25));
    /// assert_eq!(ranges.next(), Some(30..=40));
    /// assert_eq!(ranges.next(), None);
    /// ```
    pub fn range_values(&self) -> RangeValuesIter<'_, T, V> {
        RangeValuesIter {
            iter: self.btree_map.iter(),
        }
    }

    /// cmk
    pub fn ranges(&self) -> RangesFromMapIter<T, V, &V, RangeValuesFromBTree<T, V>> {
        let iter = RangeValuesFromBTree {
            iter: self.btree_map.iter(),
            phantom: PhantomData,
        };
        RangesFromMapIter::new(iter)
    }

    /// cmk
    /// cmk it's going to clone the value
    pub fn complement_with(&self, value: V) -> RangeMapBlaze<T, V> {
        self.ranges()
            .complement()
            .map(|r| (r, value.borrow_clone()))
            .collect()
    }

    /// An iterator that moves out the ranges in the [`RangeMapBlaze`],
    /// i.e., the integers as sorted & disjoint ranges.
    ///
    /// Also see [`RangeMapBlaze::into_iter`] and [`RangeMapBlaze::ranges`].
    ///
    /// # Examples
    ///
    /// ```
    /// use range_set_blaze::RangeMapBlaze;
    ///
    /// let mut ranges = RangeMapBlaze::from_iter([10..=20, 15..=25, 30..=40]).into_ranges();
    /// assert_eq!(ranges.next(), Some(10..=25));
    /// assert_eq!(ranges.next(), Some(30..=40));
    /// assert_eq!(ranges.next(), None);
    /// ```
    ///
    /// Values returned by the iterator are returned in ascending order:
    ///
    /// ```
    /// use range_set_blaze::RangeMapBlaze;
    ///
    /// let mut ranges = RangeMapBlaze::from_iter([30..=40, 15..=25, 10..=20]).into_ranges();
    /// assert_eq!(ranges.next(), Some(10..=25));
    /// assert_eq!(ranges.next(), Some(30..=40));
    /// assert_eq!(ranges.next(), None);
    /// ```
    // cmk
    // pub fn into_ranges(self) -> IntoRangesIter<T, V, VR> {
    //     IntoRangesIter {
    //         iter: self.btree_map.into_iter(),
    //     }
    // }

    // FUTURE BTreeSet some of these as 'const' but it uses unstable. When stable, add them here and elsewhere.

    /// Returns the number of sorted & disjoint ranges in the set.
    ///
    /// # Example
    ///
    /// ```
    /// use range_set_blaze::RangeMapBlaze;
    ///
    /// // We put in three ranges, but they are not sorted & disjoint.
    /// let set = RangeMapBlaze::from_iter([10..=20, 15..=25, 30..=40]);
    /// // After RangeMapBlaze sorts & 'disjoint's them, we see two ranges.
    /// assert_eq!(set.ranges_len(), 2);
    /// assert_eq!(set.to_string(), "10..=25, 30..=40");
    /// ```
    #[must_use]
    pub fn range_values_len(&self) -> usize {
        self.btree_map.len()
    }

    // / Retains only the elements specified by the predicate.
    // /
    // / In other words, remove all integers `e` for which `f(&e)` returns `false`.
    // / The integer elements are visited in ascending order.
    // /
    // / # Examples
    // /
    // / ```
    // / use range_set_blaze::RangeMapBlaze;
    // /
    // / let mut set = RangeMapBlaze::from_iter([1..=6]);
    // / // Keep only the even numbers.
    // / set.retain(|k| k % 2 == 0);
    // / assert_eq!(set, RangeMapBlaze::from_iter([2, 4, 6]));
    // / ```
    // cmk
    // pub fn retain<F>(&mut self, mut f: F)
    // where
    //     F: FnMut(&T) -> bool,
    // {
    //     *self = self.iter().filter(|v| f(v)).collect();
    // }
}

// We create a RangeMapBlaze from an iterator of integers or integer ranges by
// 1. turning them into a UnionIterMap (internally, it collects into intervals and sorts by start).
// 2. Turning the SortedDisjointMap into a BTreeMap.
impl<'a, T, V> FromIterator<(T, &'a V)> for RangeMapBlaze<T, V>
where
    T: Integer,
    V: ValueOwned + 'a,
{
    /// Create a [`RangeMapBlaze`] from an iterator of integers. Duplicates and out-of-order elements are fine.
    ///
    /// *For more about constructors and performance, see [`RangeMapBlaze` Constructors](struct.RangeMapBlaze.html#RangeMapBlaze-constructors).*
    ///
    /// # Examples
    ///
    /// ```
    /// use range_set_blaze::RangeMapBlaze;
    ///
    /// let a0 = RangeMapBlaze::from_iter([3, 2, 1, 100, 1]);
    /// let a1: RangeMapBlaze<i32> = [3, 2, 1, 100, 1].into_iter().collect();
    /// assert!(a0 == a1 && a0.to_string() == "1..=3, 100..=100");
    /// ```
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = (T, &'a V)>,
    {
        let iter = iter.into_iter().map(|(x, r)| (x..=x, r));
        RangeMapBlaze::from_iter(iter)
    }
}

impl<'a, T, V> FromIterator<(RangeInclusive<T>, &'a V)> for RangeMapBlaze<T, V>
where
    T: Integer + 'a,
    V: ValueOwned + 'a,
{
    /// Create a [`RangeMapBlaze`] from an iterator of inclusive ranges, `start..=end`.
    /// Overlapping, out-of-order, and empty ranges are fine.
    ///
    /// *For more about constructors and performance, see [`RangeMapBlaze` Constructors](struct.RangeMapBlaze.html#RangeMapBlaze-constructors).*
    ///
    /// # Examples
    ///
    /// ```
    /// use range_set_blaze::RangeMapBlaze;
    ///
    /// #[allow(clippy::reversed_empty_ranges)]
    /// let a0 = RangeMapBlaze::from_iter([1..=2, 2..=2, -10..=-5, 1..=0]);
    /// #[allow(clippy::reversed_empty_ranges)]
    /// let a1: RangeMapBlaze<i32> = [1..=2, 2..=2, -10..=-5, 1..=0].into_iter().collect();
    /// assert!(a0 == a1 && a0.to_string() == "-10..=-5, 1..=2");
    /// ```
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = (RangeInclusive<T>, &'a V)>,
    {
        let iter = iter.into_iter();
        let union_iter_map = UnionIterMap::<'a, T, V, &V, _>::from_iter(iter);
        RangeMapBlaze::from_sorted_disjoint_map(union_iter_map)
    }
}

impl<T: Integer, V: ValueOwned> FromIterator<(RangeInclusive<T>, V)> for RangeMapBlaze<T, V> {
    /// Create a [`RangeMapBlaze`] from an iterator of inclusive ranges, `start..=end`.
    /// Overlapping, out-of-order, and empty ranges are fine.
    ///
    /// *For more about constructors and performance, see [`RangeMapBlaze` Constructors](struct.RangeMapBlaze.html#RangeMapBlaze-constructors).*
    ///
    /// # Examples
    ///
    /// ```
    /// use range_set_blaze::RangeMapBlaze;
    ///
    /// #[allow(clippy::reversed_empty_ranges)]
    /// let vec_range = vec![1..=2, 2..=2, -10..=-5, 1..=0];
    /// let a0 = RangeMapBlaze::from_iter(vec_range.iter());
    /// let a1: RangeMapBlaze<i32> = vec_range.iter().collect();
    /// assert!(a0 == a1 && a0.to_string() == "-10..=-5, 1..=2");
    /// ```
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = (RangeInclusive<T>, V)>,
    {
        let iter = iter
            .into_iter()
            .non_zero_enumerate()
            .map(|(priority, (r, v))| {
                RangeValue::new(r.clone(), UniqueValue { value: Some(v) }, Some(priority))
            });
        // let _n: RangeValue<T, V, UniqueValue<V>> = iter.next().unwrap();
        let union_iter_map = UnionIterMap::<T, V, UniqueValue<V>, _>::from_iter(iter);
        RangeMapBlaze::from_sorted_disjoint_map(union_iter_map)
    }
}

impl<T: Integer, V: ValueOwned> FromIterator<(T, V)> for RangeMapBlaze<T, V> {
    /// Create a [`RangeMapBlaze`] from an iterator of inclusive ranges, `start..=end`.
    /// Overlapping, out-of-order, and empty ranges are fine.
    ///
    /// *For more about constructors and performance, see [`RangeMapBlaze` Constructors](struct.RangeMapBlaze.html#RangeMapBlaze-constructors).*
    ///
    /// # Examples
    ///
    /// ```
    /// use range_set_blaze::RangeMapBlaze;
    ///
    /// #[allow(clippy::reversed_empty_ranges)]
    /// let vec_range = vec![1..=2, 2..=2, -10..=-5, 1..=0];
    /// let a0 = RangeMapBlaze::from_iter(vec_range.iter());
    /// let a1: RangeMapBlaze<i32> = vec_range.iter().collect();
    /// assert!(a0 == a1 && a0.to_string() == "-10..=-5, 1..=2");
    /// ```
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = (T, V)>,
    {
        let iter = iter.into_iter().map(|(k, v)| (k..=k, v));
        RangeMapBlaze::from_iter(iter)
    }
}

#[doc(hidden)]
pub type MergeMapAdjusted<'a, T, V, VR, L, R> =
    MergeMap<'a, T, V, VR, AdjustPriorityMap<'a, T, V, VR, L>, AdjustPriorityMap<'a, T, V, VR, R>>;

#[doc(hidden)]
pub type BitOrMergeMap<'a, T, V, VR, L, R> =
    UnionIterMap<'a, T, V, VR, MergeMapAdjusted<'a, T, V, VR, L, R>>;

#[doc(hidden)]
pub type BitAndRangesMap<'a, T, V, VR, L, R> = IntersectionIterMap<'a, T, V, VR, L, R>;
#[doc(hidden)]
pub type BitSubRangesMap<'a, T, V, VR, L, R> = IntersectionIterMap<'a, T, V, VR, L, NotIter<T, R>>;

#[doc(hidden)]
pub type SortedStartsInVecMap<'a, T, V, VR> =
    AssumeSortedStartsMap<'a, T, V, VR, vec::IntoIter<RangeValue<'a, T, V, VR>>>;
// pub type BitXOrTeeMap<'a, T, V, VR, L, R> = BitOrMergeMap<
//     'a,
//     T,
//     V,
//     VR,
//     BitSubRangesMap<'a, T, V, VR, Tee<L>, Tee<R>>,
//     BitSubRangesMap<'a, T, V, VR, Tee<R>, Tee<L>>,
// >;
// #[doc(hidden)]
// pub type BitOrKMergeMap<T, V, I> = UnionIterMap<T, V, KMergeMap<T, V, I>>;
// #[doc(hidden)]
// pub type BitAndMergeMap<T, V, L, R> = NotIterMap<T, V, BitNandMergeMap<T, V, L, R>>;
// #[doc(hidden)]
// pub type BitAndKMergeMap<T, V, I> = NotIterMap<T, V, BitNandKMergeMap<T, V, I>>;
// #[doc(hidden)]
// pub type BitNandMergeMap<T, V, L, R> =
//     BitOrMergeMap<T, V, NotIterMap<T, V, L>, NotIterMap<T, V, R>>;
// #[doc(hidden)]
// pub type BitNandKMergeMap<T, V, I> = BitOrKMergeMap<T, V, NotIterMap<T, V, I>>;
// #[doc(hidden)]
// pub type BitNorMergeMap<T, V, L, R> = NotIterMap<T, V, BitOrMergeMap<T, V, L, R>>;
// #[doc(hidden)]
// pub type BitSubMergeMap<T, V, L, R> = NotIterMap<T, V, BitOrMergeMap<T, V, NotIterMap<T, V, L>, R>>;
// #[doc(hidden)]
// pub type BitXOrTeeMap<T, V, L, R> =
//     BitOrMergeMap<T, V, BitSubMergeMap<T, V, Tee<L>, Tee<R>>, BitSubMergeMap<T, V, Tee<R>, Tee<L>>>;
// #[doc(hidden)]
// pub type BitXOrMap<T, V, L, R> =
//     BitOrMergeMap<T, V, BitSubMergeMap<T, V, L, Tee<R>>, BitSubMergeMap<T, V, Tee<R>, L>>;
// #[doc(hidden)]
// pub type BitEqMap<T, V, L, R> = BitOrMergeMap<
//     T,
//     V,
//     NotIterMap<T, V, BitOrMergeMap<T, V, NotIterMap<T, V, Tee<L>>, NotIterMap<T, V, Tee<R>>>>,
//     NotIterMap<T, V, BitOrMergeMap<T, V, Tee<L>, Tee<R>>>,
// >;

// If the inputs have sorted starts, then so does the output.
impl<'a, T: Integer + 'a, V: ValueOwned + 'a, VR, I: SortedStartsMap<'a, T, V, VR>>
    SortedStartsMap<'a, T, V, VR> for UnionIterMap<'a, T, V, VR, I>
where
    VR: CloneBorrow<V> + 'a,
{
}

// If the inputs have sorted starts, the output is sorted and disjoint.
impl<'a, T: Integer + 'a, V: ValueOwned + 'a, VR, I: SortedStartsMap<'a, T, V, VR>>
    SortedDisjointMap<'a, T, V, VR> for UnionIterMap<'a, T, V, VR, I>
where
    VR: CloneBorrow<V> + 'a,
{
}

impl<T: Integer, V: ValueOwned> BitOr<RangeMapBlaze<T, V>> for RangeMapBlaze<T, V> {
    /// Unions the contents of two [`RangeMapBlaze`]'s.
    ///
    /// Passing ownership rather than borrow sometimes allows a many-times
    /// faster speed up.
    ///
    /// # Examples
    /// ```
    /// use range_set_blaze::RangeMapBlaze;
    /// let a = RangeMapBlaze::from_iter([1..=4]);
    /// let b = RangeMapBlaze::from_iter([0..=0, 3..=5, 10..=10]);
    /// let union = a | b;
    /// assert_eq!(union, RangeMapBlaze::from_iter([0..=5, 10..=10]));
    /// ```
    type Output = RangeMapBlaze<T, V>;
    fn bitor(self, other: Self) -> RangeMapBlaze<T, V> {
        // cmk
        // self |= other;
        // self
        // cmk00 use |
        self.range_values()
            .union(other.range_values())
            .into_range_map_blaze()
    }
}

impl<T: Integer, V: ValueOwned> BitOr<&RangeMapBlaze<T, V>> for RangeMapBlaze<T, V> {
    /// Unions the contents of two [`RangeMapBlaze`]'s.
    ///
    /// Passing ownership rather than borrow sometimes allows a many-times
    /// faster speed up.
    ///
    /// # Examples
    /// ```
    /// use range_set_blaze::RangeMapBlaze;
    /// let mut a = RangeMapBlaze::from_iter([1..=4]);
    /// let mut b = RangeMapBlaze::from_iter([0..=0,3..=5,10..=10]);
    /// let union = a | &b;
    /// assert_eq!(union, RangeMapBlaze::from_iter([0..=5, 10..=10]));
    /// ```
    type Output = RangeMapBlaze<T, V>;
    fn bitor(self, other: &Self) -> RangeMapBlaze<T, V> {
        // self |= other;
        // self
        // cmk00 use |
        self.range_values()
            .union(other.range_values())
            .into_range_map_blaze()
    }
}

impl<T: Integer, V: ValueOwned> BitOr<RangeMapBlaze<T, V>> for &RangeMapBlaze<T, V> {
    type Output = RangeMapBlaze<T, V>;
    /// Unions the contents of two [`RangeMapBlaze`]'s.
    ///
    /// Passing ownership rather than borrow sometimes allows a many-times
    /// faster speed up.
    ///
    /// # Examples
    /// ```
    /// use range_set_blaze::RangeMapBlaze;
    /// let mut a = RangeMapBlaze::from_iter([1..=4]);
    /// let mut b = RangeMapBlaze::from_iter([0..=0,3..=5,10..=10]);
    /// let union = &a | b;
    /// assert_eq!(union, RangeMapBlaze::from_iter([0..=5, 10..=10]));
    /// ```
    fn bitor(self, other: RangeMapBlaze<T, V>) -> RangeMapBlaze<T, V> {
        // cmk
        // other |= self;
        // other
        // cmk00 use |
        self.range_values()
            .union(other.range_values())
            .into_range_map_blaze()
    }
}

impl<T: Integer, V: ValueOwned> BitOr<&RangeMapBlaze<T, V>> for &RangeMapBlaze<T, V> {
    type Output = RangeMapBlaze<T, V>;
    /// Unions the contents of two [`RangeMapBlaze`]'s.
    ///
    /// Passing ownership rather than borrow sometimes allows a many-times
    /// faster speed up.
    ///
    /// # Examples
    /// ```
    /// use range_set_blaze::RangeMapBlaze;
    /// let mut a = RangeMapBlaze::from_iter([1..=4]);
    /// let mut b = RangeMapBlaze::from_iter([0..=0,3..=5,10..=10]);
    /// let union = &a | &b;
    /// assert_eq!(union, RangeMapBlaze::from_iter([0..=5, 10..=10]));
    /// ```
    fn bitor(self, other: &RangeMapBlaze<T, V>) -> RangeMapBlaze<T, V> {
        let left = self.range_values();
        let right = other.range_values();
        // cmk00 use |
        left.union(right).into_range_map_blaze()
    }
}

pub struct UniqueValue<V>
where
    V: ValueOwned,
{
    value: Option<V>,
}

impl<V> CloneBorrow<V> for UniqueValue<V>
where
    V: ValueOwned,
{
    fn clone_borrow(&self) -> Self {
        UniqueValue {
            value: self.value.clone(),
        }
    }

    fn borrow_clone(mut self) -> V {
        // cmk will panic if None
        self.value.take().unwrap()
    }
}

impl<V> Borrow<V> for UniqueValue<V>
where
    V: ValueOwned,
{
    fn borrow(&self) -> &V {
        // cmk will panic if None
        self.value.as_ref().unwrap()
        // &self.value
    }
}

gen_ops_ex!(
    <T, V>;
    types ref RangeMapBlaze<T,V>, ref RangeMapBlaze<T,V> => RangeMapBlaze<T,V>;

    /// Intersects the contents of two [`RangeMapBlaze`]'s.
    ///
    /// Either, neither, or both inputs may be borrowed.
    ///
    /// # Examples
    /// ```
    /// use range_set_blaze::prelude::*;
    ///
    /// let a = RangeMapBlaze::from_iter([1..=2, 5..=100]);
    /// let b = RangeMapBlaze::from_iter([2..=6]);
    /// let result = &a & &b; // Alternatively, 'a & b'.
    /// assert_eq!(result.to_string(), "2..=2, 5..=6");
    /// ```
    for & call |a: &RangeMapBlaze<T, V>, b: &RangeMapBlaze<T, V>| {
        // cmk use & ???
        a.range_values().intersection(b.ranges()).into_range_map_blaze()
    };
/// Symmetric difference the contents of two [`RangeMapBlaze`]'s.
///
/// Either, neither, or both inputs may be borrowed.
///
/// # Examples
/// ```
/// use range_set_blaze::prelude::*;
///
/// let a = RangeMapBlaze::from_iter([1..=2, 5..=100]);
/// let b = RangeMapBlaze::from_iter([2..=6]);
/// let result = &a ^ &b; // Alternatively, 'a ^ b'.
/// assert_eq!(result.to_string(), "1..=1, 3..=4, 7..=100");
/// ```
for ^ call |a: &RangeMapBlaze<T, V>, b: &RangeMapBlaze<T, V>| {
    // We optimize this by using ranges() twice per input, rather than tee()
    let lhs0 = a.range_values();
    let lhs1 = a.ranges();
    let rhs0 = b.ranges();
    let rhs1 = b.range_values();
    (lhs0.difference(rhs0).union(rhs1.difference(lhs1))).into_range_map_blaze()
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

for - call |a: &RangeMapBlaze<T, V>, b: &RangeMapBlaze<T, V>| {
    a.range_values().difference(b.ranges()).into_range_map_blaze()
};
where T: Integer, V: ValueOwned
);

gen_ops_ex!(
    <T, V>;
    types ref RangeMapBlaze<T,V>, ref RangeSetBlaze<T> => RangeMapBlaze<T,V>;


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
/// cmk
for - call |a: &RangeMapBlaze<T, V>, b: &RangeSetBlaze<T>| {
    a.range_values().difference(b.ranges()).into_range_map_blaze()
};

/// cmk
for & call |a: &RangeMapBlaze<T, V>, b: &RangeSetBlaze<T>| {
    a.range_values().intersection(b.ranges()).into_range_map_blaze()
};

where T: Integer, V: ValueOwned
);

gen_ops_ex!(
    <T, V>;
    types ref RangeMapBlaze<T,V> => RangeSetBlaze<T>;


/// cmk
for ! call |a: &RangeMapBlaze<T, V>| {
    a.ranges().complement().into_range_set_blaze()
};
where T: Integer, V: ValueOwned
);

/// A (double-ended) iterator over the integer elements of a [`RangeMapBlaze`].
///
/// This `struct` is created by the [`iter`] method on [`RangeMapBlaze`]. See its
/// documentation for more.
///
/// [`iter`]: RangeMapBlaze::iter
#[must_use = "iterators are lazy and do nothing unless consumed"]
#[derive(Clone, Debug)]
pub struct IterMap<'a, T, V, VR, I>
where
    T: Integer + 'a,
    V: ValueOwned + 'a,
    VR: CloneBorrow<V> + 'a,
    I: SortedDisjointMap<'a, T, V, VR>,
{
    iter: I,
    option_range_value_front: Option<RangeValue<'a, T, V, VR>>,
    option_range_value_back: Option<RangeValue<'a, T, V, VR>>,
    phantom0: PhantomData<&'a V>,
    phantom1: PhantomData<VR>,
}

impl<'a, T, V, VR, I> FusedIterator for IterMap<'a, T, V, VR, I>
where
    T: Integer + 'a,
    V: ValueOwned + 'a,
    VR: CloneBorrow<V> + 'a,
    I: SortedDisjointMap<'a, T, V, VR> + FusedIterator,
{
}

impl<'a, T, V, VR, I> Iterator for IterMap<'a, T, V, VR, I>
where
    T: Integer + 'a,
    V: ValueOwned + 'a,
    VR: CloneBorrow<V> + 'a,
    I: SortedDisjointMap<'a, T, V, VR>,
{
    type Item = (T, VR);

    fn next(&mut self) -> Option<Self::Item> {
        let mut range_value = self
            .option_range_value_front
            .take()
            .or_else(|| self.iter.next())
            .or_else(|| self.option_range_value_back.take())?;

        let (start, end) = range_value.range.into_inner();
        debug_assert!(start <= end && end <= T::safe_max_value());
        let value = range_value.value.clone_borrow();
        if start < end {
            range_value.range = start + T::one()..=end;
            self.option_range_value_front = Some(range_value);
        }
        Some((start, value))
    }

    // We'll have at least as many integers as intervals. There could be more that usize MAX
    // The option_range field could increase the number of integers, but we can ignore that.
    fn size_hint(&self) -> (usize, Option<usize>) {
        let (low, _high) = self.iter.size_hint();
        (low, None)
    }
}

impl<'a, T, V, VR, I> DoubleEndedIterator for IterMap<'a, T, V, VR, I>
where
    T: Integer + 'a,
    V: ValueOwned + 'a,
    VR: CloneBorrow<V> + 'a,
    I: SortedDisjointMap<'a, T, V, VR> + DoubleEndedIterator,
{
    fn next_back(&mut self) -> Option<Self::Item> {
        let mut range_value = self
            .option_range_value_back
            .take()
            .or_else(|| self.iter.next_back())
            .or_else(|| self.option_range_value_front.take())?;
        let (start, end) = range_value.range.into_inner();
        debug_assert!(start <= end && end <= T::safe_max_value());
        let value = range_value.value.clone_borrow();
        if start < end {
            range_value.range = start..=end - T::one();
            self.option_range_value_back = Some(range_value);
        }

        Some((end, value))
    }
}

#[must_use = "iterators are lazy and do nothing unless consumed"]
/// A (double-ended) iterator over the integer elements of a [`RangeMapBlaze`].
///
/// This `struct` is created by the [`into_iter`] method on [`RangeMapBlaze`]. See its
/// documentation for more.
///
/// [`into_iter`]: RangeMapBlaze::into_iter
pub struct IntoIterMap<'a, T, V>
where
    T: Integer + 'a,
    V: ValueOwned + 'a,
{
    option_start_end_value_front: Option<(T, EndValue<T, V>)>,
    option_start_end_value_back: Option<(T, EndValue<T, V>)>,
    into_iter: btree_map::IntoIter<T, EndValue<T, V>>,
    phantom: PhantomData<&'a V>, // cmk00 needed?
}

impl<'a, T, V> FusedIterator for IntoIterMap<'a, T, V>
where
    T: Integer + 'a,
    V: ValueOwned + 'a,
{
}

impl<'a, T, V> Iterator for IntoIterMap<'a, T, V>
where
    T: Integer + 'a,
    V: ValueOwned + 'a,
{
    type Item = (T, V);

    fn next(&mut self) -> Option<Self::Item> {
        let start_end_value = self
            .option_start_end_value_front
            .take()
            .or_else(|| self.into_iter.next())
            .or_else(|| self.option_start_end_value_back.take())?;

        let start = start_end_value.0;
        let end = start_end_value.1.end;
        let value = start_end_value.1.value.borrow_clone();
        debug_assert!(start <= end && end <= T::safe_max_value());
        if start < end {
            let end_value = start_end_value.1;
            let start_end_value = (start + T::one(), end_value);
            self.option_start_end_value_front = Some(start_end_value);
        }
        Some((start, value))
    }

    // We'll have at least as many integers as intervals. There could be more that usize MAX
    // the option_range field could increase the number of integers, but we can ignore that.
    fn size_hint(&self) -> (usize, Option<usize>) {
        let (low, _high) = self.into_iter.size_hint();
        (low, None)
    }
}

impl<'a, T, V> DoubleEndedIterator for IntoIterMap<'a, T, V>
where
    T: Integer + 'a,
    V: ValueOwned + 'a,
{
    fn next_back(&mut self) -> Option<Self::Item> {
        let start_end_value = self
            .option_start_end_value_back
            .take()
            .or_else(|| self.into_iter.next())
            .or_else(|| self.option_start_end_value_front.take())?;

        let start = start_end_value.0;
        let end = start_end_value.1.end;
        let value = start_end_value.1.value.borrow_clone();
        debug_assert!(start <= end && end <= T::safe_max_value());

        if start < end {
            let mut end_value = start_end_value.1;
            end_value.end -= T::one();
            let start_end_value = (start, end_value);
            self.option_start_end_value_back = Some(start_end_value);
        }

        Some((end, value))
    }
}
