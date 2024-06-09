use crate::intersection_iter_map::IntersectionIterMap;
use crate::iter_map::IntoIterMap;
use crate::iter_map::{IterMap, KeysMap};
use crate::range_values::{
    IntoRangeValuesIter, MapIntoRangesIter, MapRangesIter, RangeValuesIter, RangeValuesToRangesIter,
};
use crate::sorted_disjoint_map::SortedDisjointMap;
use crate::sorted_disjoint_map::{IntoString, Priority};
use crate::sym_diff_iter_map::SymDiffIterMap;
use crate::unsorted_disjoint_map::{
    AssumePrioritySortedStartsMap, SortedDisjointMapWithLenSoFar, UnsortedPriorityDisjointMap,
};
#[cfg(feature = "rog-experimental")]
#[allow(deprecated)]
use crate::SomeOrGap;
use crate::{
    AssumeSortedStarts, CheckSortedDisjoint, Integer, NotIter, RangeSetBlaze, SortedDisjoint,
};
use alloc::collections::BTreeMap;
use alloc::rc::Rc;
#[cfg(feature = "std")]
use alloc::sync::Arc;
use alloc::vec;
use alloc::vec::Vec;
use core::borrow::Borrow;
use core::cmp::Ordering;
use core::fmt;
use core::ops::{BitOr, Bound, Index, RangeBounds};
use core::{cmp::max, convert::From, ops::RangeInclusive};
use gen_ops::gen_ops_ex;
use num_traits::One;
use num_traits::Zero;

/// cmk doc
pub trait EqClone: Eq + Clone {}

impl<T> EqClone for T where T: Eq + Clone {}

/// Trait for references that can be cloned to return a new reference to an equal value.
/// The associated value type must implement `EqClone`
pub trait ValueRef: Borrow<Self::Value> {
    /// The associated value type.
    type Value: EqClone;

    /// Clones the reference, returning a new reference to a value that is eq with the original value.
    /// (It may or may not be the original value.)
    #[must_use]
    fn clone_ref(&self) -> Self;
}

// Implementations for references and smart pointers
impl<V> ValueRef for &V
where
    V: EqClone,
{
    type Value = V;

    fn clone_ref(&self) -> Self {
        self
    }
}

impl<V> ValueRef for Rc<V>
where
    V: EqClone,
{
    type Value = V;

    fn clone_ref(&self) -> Self {
        Self::clone(self)
    }
}

#[cfg(feature = "std")]
impl<V> ValueRef for Arc<V>
where
    V: EqClone,
{
    type Value = V;

    fn clone_ref(&self) -> Self {
        Self::clone(self)
    }
}

#[derive(Clone, Hash, Default, PartialEq, Eq)]
pub struct EndValue<T, V>
where
    T: Integer,
    V: EqClone,
{
    pub(crate) end: T,
    pub(crate) value: V,
}

/// A map from integers to values stored as a map of sorted & disjoint ranges to values.
///
/// Internally, it stores the ranges in a cache-efficient [`BTreeMap`].
///
/// # Table of Contents
/// * [`RangeMapBlaze` Constructors](#rangemapblaze-constructors)
///    * [Performance](#constructor-performance)
///    * [Examples](struct.RangeMapBlaze.html#constructor-examples)
/// * [`RangeMapBlaze` Set Operations](#rangemapblaze-set-operations)
///    * [Performance](struct.RangeMapBlaze.html#set-operation-performance)
///    * [Examples](struct.RangeMapBlaze.html#set-operation-examples)
///  * [`RangeMapBlaze` Comparisons](#rangemapblaze-comparisons)
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
///
/// | Methods                                     | Input                        | Notes                    |
/// |---------------------------------------------|------------------------------|--------------------------|
/// | [`new`]/[`default`]                         |                              |                          |
/// | [`from_iter`][1]/[`collect`][1]           | iterator of `(integer, value)` | '`&`' allowed before `value` or the pair |
/// | [`from_iter`][2]/[`collect`][2]           | iterator of `(range, value)` | '`&`' allowed before `value` or the pair |
/// | [`from_sorted_disjoint_map`][3]/<br>[`into_range_set_blaze`][3b] | [`SortedDisjointMap`] iterator |               |
/// | [`from`][4] /[`into`][4]                    | array of `(integer, value)`  |                          |
///
///
/// [`BTreeMap`]: alloc::collections::BTreeMap
/// [`new`]: RangeMapBlaze::new
/// [`default`]: RangeMapBlaze::default
/// [1]: struct.RangeMapBlaze.html#impl-FromIterator<(T,+V)>-for-RangeMapBlaze<T,+V>
/// [2]: struct.RangeMapBlaze.html#impl-FromIterator<(RangeInclusive<T>,+V)>-for-RangeMapBlaze<T,+V>
/// [3]: `RangeMapBlaze::from_sorted_disjoint_map`
/// [3b]: `SortedDisjointMap::into_range_map_blaze
/// [`SortedDisjointMap`]: trait.SortedDisjointMap.html#table-of-contents
/// [4]: `RangeMapBlaze::from`
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
/// ## Constructor Examples
///
/// ```
/// use range_set_blaze::prelude::*;
///
/// // Create an empty set with 'new' or 'default'.
/// let a0 = RangeMapBlaze::<i32, &str>::new();
/// let a1 = RangeMapBlaze::<i32, &str>::default();
/// assert!(a0 == a1 && a0.is_empty());
///
/// // 'from_iter'/'collect': From an iterator of integers.
/// // Duplicates and out-of-order elements are fine.
/// // Values have left-to-right precedence.
/// let a0 = RangeMapBlaze::from_iter([(3, "a"), (2, "a"), (1, "a"), (100, "b"), (1, "c")]);
/// let a1: RangeMapBlaze<i32, &str> = [(3, "a"), (2, "a"), (1, "a"), (100, "b"), (1, "c")].into_iter().collect();
/// assert!(a0 == a1 && a0.to_string() == r#"(1..=3, "a"), (100..=100, "b")"#);
///
/// // 'from_iter'/'collect': From an iterator of inclusive ranges, start..=end.
/// // Overlapping, out-of-order, and empty ranges are fine.
/// // Values have left-to-right precedence.
/// #[allow(clippy::reversed_empty_ranges)]
/// let a0 = RangeMapBlaze::from_iter([(1..=2, "a"), (2..=2, "b"), (-10..=-5, "c"), (1..=0, "d")]);
/// #[allow(clippy::reversed_empty_ranges)]
/// let a1: RangeMapBlaze<i32, &str> = [(1..=2, "a"), (2..=2, "b"), (-10..=-5, "c"), (1..=0, "d")].into_iter().collect();
/// assert!(a0 == a1 && a0.to_string() == r#"(-10..=-5, "c"), (1..=2, "a")"#);
///
/// // If we know the ranges are already sorted and disjoint,
/// // we can avoid work and use 'from_sorted_disjoint_map'/'into'.
/// let a0 = RangeMapBlaze::from_sorted_disjoint_map(CheckSortedDisjointMap::new([(-10..=-5, &"c"), (1..=2, &"a")]));
/// let a1: RangeMapBlaze<i32, &str> = CheckSortedDisjointMap::new([(-10..=-5, &"c"), (1..=2, &"a")]).into_range_map_blaze();
/// assert!(a0 == a1 && a0.to_string() == r#"(-10..=-5, "c"), (1..=2, "a")"#);
///
/// // For compatibility with `BTreeSet`, we also support
/// // 'from'/'into' from arrays of integers.
/// let a0 = RangeMapBlaze::from([(3, "a"), (2, "a"), (1, "a"), (100, "b"), (1, "c")]);
/// let a1: RangeMapBlaze<i32, &str> = [(3, "a"), (2, "a"), (1, "a"), (100, "b"), (1, "c")].into();
/// assert!(a0 == a1 && a0.to_string() == r#"(1..=3, "a"), (100..=100, "b")"#);
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
/// [`union`]: trait.MultiwayRangeMapBlazeRef.html#method.union
/// [`intersection`]: trait.MultiwayRangeMapBlazeRef.html#method.intersection
/// [`insert`]: RangeMapBlaze::insert
/// [`pop_first`]: RangeMapBlaze::pop_first
/// [`split_off`]: RangeMapBlaze::split_off
/// [`SortedDisjointMap`]: trait.SortedDisjointMap.html#table-of-contents
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
/// let a = RangeMapBlaze::from_iter([(1..=2, "a"), (5..=100, "a")]);
/// let b = RangeMapBlaze::from_iter([(2..=6, "b")]);
///
/// // Union of two 'RangeMapBlaze's. Alternatively, we can take ownership via 'a | b'.
/// // Values have left-to-right precedence.
/// let result = &a | &b;
/// assert_eq!(result.to_string(), r#"(1..=2, "a"), (3..=4, "b"), (5..=100, "a")"#);
///
/// // Intersection of two 'RangeMapBlaze's.
/// let result = &a & &b; // Alternatively, 'a & b'.
/// assert_eq!(result.to_string(), r#"(2..=2, "a"), (5..=6, "a")"#);
///
/// // Set difference of two 'RangeMapBlaze's.
/// let result = &a - &b; // Alternatively, 'a - b'.
/// assert_eq!(result.to_string(), r#"(1..=1, "a"), (7..=100, "a")"#);
///
/// // Symmetric difference of two 'RangeMapBlaze's.
/// let result = &a ^ &b; // Alternatively, 'a ^ b'.
/// assert_eq!(result.to_string(), r#"(1..=1, "a"), (3..=4, "b"), (7..=100, "a")"#);
///
/// // complement of a 'RangeMapBlaze' is a `RangeSetBlaze`.
/// let result = !&a; // Alternatively, '!a'.
/// assert_eq!(result.to_string(), "-2147483648..=0, 3..=4, 101..=2147483647"
/// );
/// // use `complement_with` to create a 'RangeMapBlaze'.
/// let result = a.complement_with(&"z");
/// assert_eq!(result.to_string(), r#"(-2147483648..=0, "z"), (3..=4, "z"), (101..=2147483647, "z")"#);
///
/// // Multiway union of 'RangeMapBlaze's.
/// let c = RangeMapBlaze::from_iter([(2..=2, "c"), (6..=200, "c")]);
/// let result = [&a, &b, &c].union();
/// assert_eq!(result.to_string(), r#"(1..=2, "a"), (3..=4, "b"), (5..=100, "a"), (101..=200, "c")"# );
///
/// // Multiway intersection of 'RangeMapBlaze's.
/// let result = [&a, &b, &c].intersection();
/// assert_eq!(result.to_string(), r#"(2..=2, "a"), (6..=6, "a")"#);
///
/// // Applying multiple operators
/// let result0 = &a - (&b | &c); // Creates an intermediate 'RangeMapBlaze'.
/// // Alternatively, we can use the 'SortedDisjointMap' API and avoid the intermediate 'RangeMapBlaze'.
/// let result1 = RangeMapBlaze::from_sorted_disjoint_map(
///          a.range_values() - (b.range_values() | c.range_values()));
/// assert!(result0 == result1 && result0.to_string() == r#"(1..=1, "a")"#);
/// ```
/// # `RangeMapBlaze` Comparisons
///
/// `RangeMapBlaze` supports comparisons for equality and lexicographic order:
///
/// - **Equality**: Use `==` and `!=` to check if two `RangeMapBlaze` instances
///   are equal. Two `RangeMapBlaze` instances are considered equal if they
///   contain the same ranges and associated values.
/// - **Ordering**: If the values implement `Ord`, you can use `<`, `<=`, `>`, and `>=`
///   to compare two `RangeMapBlaze` instances. These comparisons are lexicographic,
///   similar to `BTreeSet`, meaning they compare the ranges and their values in sequence.
/// - **Partial Ordering**: If the values implement `PartialOrd` but not `Ord`, you can use
///   the [`partial_cmp`] method to compare two `RangeMapBlaze` instances. This method returns
///   an `Option<Ordering>` that indicates the relative order of the instances or `None` if the
///   values are not comparable.
///
/// See [`partial_cmp`] and [`cmp`] for more examples.
///
///
/// [`BTreeSet`]: alloc::collections::BTreeSet
/// [`partial_cmp`]: RangeMapBlaze::partial_cmp
/// [`cmp`]: RangeMapBlaze::cmp
///
/// # Additional Examples
///
/// See the [module-level documentation] for additional examples.
///
/// [module-level documentation]: index.html
#[derive(Clone, Hash, PartialEq)]
pub struct RangeMapBlaze<T: Integer, V: EqClone> {
    pub(crate) len: <T as Integer>::SafeLen,
    pub(crate) btree_map: BTreeMap<T, EndValue<T, V>>,
}

/// Creates a new, empty `RangeMapBlaze`.
///
/// # Examples
///
/// ```
/// use range_set_blaze::RangeMapBlaze;
///
/// let a = RangeMapBlaze::<i32, &str>::default();
/// assert!(a.is_empty());
/// ```
impl<T: Integer, V: EqClone> Default for RangeMapBlaze<T, V> {
    fn default() -> Self {
        Self {
            len: <T as Integer>::SafeLen::zero(),
            btree_map: BTreeMap::new(),
        }
    }
}

impl<T: Integer, V: EqClone + fmt::Debug> fmt::Debug for RangeMapBlaze<T, V> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.range_values().into_string())
    }
}

impl<T: Integer, V: EqClone + fmt::Debug> fmt::Display for RangeMapBlaze<T, V> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.range_values().into_string())
    }
}

impl<T: Integer, V: EqClone> RangeMapBlaze<T, V> {
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
    /// let map = RangeMapBlaze::from_iter([(1..=3,"a")]);
    /// let mut map_iter = map.iter();
    /// assert_eq!(map_iter.next(), Some((1, &"a")));
    /// assert_eq!(map_iter.next(), Some((2, &"a")));
    /// assert_eq!(map_iter.next(), Some((3, &"a")));
    /// assert_eq!(map_iter.next(), None);
    /// ```
    ///
    /// Values returned by `.next()` are in ascending order.
    /// Values returned by `.next_back()` are in descending order.
    ///
    /// ```
    /// use range_set_blaze::RangeMapBlaze;
    ///
    /// let map = RangeMapBlaze::from_iter([(3,"c"), (1,"a"), (2,"b")]);
    /// let mut map_iter = map.iter();
    /// assert_eq!(map_iter.next(), Some((1, &"a")));
    /// assert_eq!(map_iter.next_back(), Some((3, &"c")));
    /// assert_eq!(map_iter.next(), Some((2, &"b")));
    /// assert_eq!(map_iter.next_back(), None);
    /// ```
    #[inline] // cmk should RangeSETBlazes iter be inlined? (look at BTreeSet)
    pub fn iter(&self) -> IterMap<T, &V, RangeValuesIter<'_, T, V>> {
        // If the user asks for an iter, we give them a RangesIter iterator
        // and we iterate that one integer at a time.
        IterMap::new(self.range_values())
    }

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
    /// let map = RangeMapBlaze::from_iter([(1..=3,"a")]);
    /// let mut keys_iter = map.keys();
    /// assert_eq!(keys_iter.next(), Some(1));
    /// assert_eq!(keys_iter.next(), Some(2));
    /// assert_eq!(keys_iter.next(), Some(3));
    /// assert_eq!(keys_iter.next(), None);
    /// ```
    ///
    /// Values returned by `.next()` are in ascending order.
    /// Values returned by `.next_back()` are in descending order.
    ///
    /// ```
    /// use range_set_blaze::RangeMapBlaze;
    ///
    /// let map = RangeMapBlaze::from_iter([(3,"c"), (1,"a"), (2,"b")]);
    /// let mut keys_iter = map.keys();
    /// assert_eq!(keys_iter.next(), Some(1));
    /// assert_eq!(keys_iter.next_back(), Some(3));
    /// assert_eq!(keys_iter.next(), Some(2));
    /// assert_eq!(keys_iter.next_back(), None);
    /// ```
    pub fn keys(&self) -> KeysMap<T, &V, RangeValuesIter<'_, T, V>> {
        // If the user asks for an iter, we give them a RangesIter iterator
        // and we iterate that one integer at a time.
        KeysMap::new(self.range_values())
    }

    // cmk BTreeMap also has 'into_keys'

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
    /// let mut map = RangeMapBlaze::new();
    /// assert_eq!(map.first_key_value(), None);
    /// map.insert(1,"a");
    /// assert_eq!(map.first_key_value(), Some((1, &"a")));
    /// map.insert(2,"b");
    /// assert_eq!(map.first_key_value(), Some((1, &"a")));
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
    /// let map = RangeMapBlaze::from_iter([(3,"c"), (1,"a"), (2,"b")]);
    /// assert_eq!(map.get(2), Some(&"b"));
    /// assert_eq!(map.get(4), None);
    /// ```
    pub fn get(&self, key: T) -> Option<&V> {
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

    /// cmk doc
    #[cfg(feature = "rog-experimental")]
    #[allow(deprecated)]
    pub fn get_range_value(&self, key: T) -> SomeOrGap<(RangeInclusive<T>, &V), T> {
        let one_back = self.btree_map.range(..=key).next_back();
        let Some((start, end_value)) = one_back else {
            // nothing before, find any after
            if let Some((start, _)) = self.btree_map.range(key..).next() {
                debug_assert!(&key < &start);
                return SomeOrGap::Gap(T::min_value()..=start.sub_one());
            };
            return SomeOrGap::Gap(T::min_value()..=T::max_value());
        };
        if key <= end_value.end {
            SomeOrGap::Some((start.clone()..=end_value.end, &end_value.value))
        } else if key == T::max_value() {
            SomeOrGap::Gap(end_value.end.add_one()..=key)
        } else {
            let next = self.btree_map.range(key..).next();
            if let Some((next_start, _)) = next {
                SomeOrGap::Gap(end_value.end.add_one()..=next_start.sub_one())
            } else {
                SomeOrGap::Gap(end_value.end.add_one()..=T::max_value())
            }
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
    /// use range_set_blaze::RangeMapBlaze;
    ///
    /// let mut map = RangeMapBlaze::new();
    /// assert_eq!(map.last_key_value(), None);
    /// map.insert(1, "a");
    /// assert_eq!(map.last_key_value(), Some((1, &"a")));
    /// map.insert(2, "b");
    /// assert_eq!(map.last_key_value(), Some((2, &"b")));
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
    /// [`SortedDisjointMap`]: trait.SortedDisjointMap.html#table-of-contents
    ///
    /// # Examples
    ///
    /// ```
    /// use range_set_blaze::prelude::*;
    ///
    /// let a0 = RangeMapBlaze::from_sorted_disjoint_map(CheckSortedDisjointMap::new([(-10..=-5, &"a"), (1..=2, &"b")]));
    /// let a1: RangeMapBlaze<i32,_> = CheckSortedDisjointMap::new([(-10..=-5, &"a"), (1..=2, &"b")]).into_range_map_blaze();
    /// assert!(a0 == a1 && a0.to_string() == r#"(-10..=-5, "a"), (1..=2, "b")"#);
    /// ```
    pub fn from_sorted_disjoint_map<VR, I>(iter: I) -> Self
    where
        VR: ValueRef<Value = V>,
        I: SortedDisjointMap<T, VR>,
    {
        let mut iter_with_len = SortedDisjointMapWithLenSoFar::from(iter);
        let btree_map: BTreeMap<T, EndValue<T, VR::Value>> = (&mut iter_with_len).collect();
        Self {
            btree_map,
            len: iter_with_len.len_so_far(),
        }
    }

    #[allow(dead_code)]
    pub(crate) fn len_slow(&self) -> <T as Integer>::SafeLen {
        Self::btree_map_len(&self.btree_map)
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
    /// let mut a = RangeMapBlaze::from_iter([(1..=3,"a")]);
    /// let mut b = RangeMapBlaze::from_iter([(3..=5,"b")]);
    ///
    /// a.append(&mut b);
    ///
    /// assert_eq!(a.len(), 5);
    /// assert_eq!(b.len(), 0);
    ///
    /// assert_eq!(a[1], "a");
    /// assert_eq!(a[2], "a");
    /// assert_eq!(a[3], "b");
    /// assert_eq!(a[4], "b");
    /// assert_eq!(a[5], "b");
    /// ```
    pub fn append(&mut self, other: &mut Self) {
        for (range, value) in other.range_values() {
            let value = value.clone();
            self.internal_add(range, value);
        }
        other.clear();
    }

    /// Clears the map, removing all elements.
    ///
    /// # Examples
    ///
    /// ```
    /// use range_set_blaze::RangeMapBlaze;
    ///
    /// let mut a = RangeMapBlaze::new();
    /// a.insert(1, "a");
    /// a.clear();
    /// assert!(a.is_empty());
    /// ```
    pub fn clear(&mut self) {
        self.btree_map.clear();
        self.len = <T as Integer>::SafeLen::zero();
    }

    /// Returns `true` if the map contains no elements.
    ///
    /// # Examples
    ///
    /// ```
    /// use range_set_blaze::RangeMapBlaze;
    ///
    /// let mut a = RangeMapBlaze::new();
    /// assert!(a.is_empty());
    /// a.insert(1, "a");
    /// assert!(!a.is_empty());
    /// ```
    #[must_use]
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.btree_map.is_empty()
    }

    /// Returns `true` if the set contains an element equal to the value.
    ///
    /// # Examples
    ///
    /// ```
    /// use range_set_blaze::RangeMapBlaze;
    ///
    /// let map = RangeMapBlaze::from_iter([(3,"c"), (1,"a"), (2,"b")]);
    /// assert_eq!(map.contains_key(1), true);
    /// assert_eq!(map.contains_key(4), false);
    /// ```
    pub fn contains_key(&self, key: T) -> bool {
        self.btree_map
            .range(..=key)
            .next_back()
            .map_or(false, |(_, end_value)| key <= end_value.end)
    }

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
                    if *start_delete <= end || *start_delete <= end.add_one() {
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
            self.len += T::safe_len(&(end..=end_new.sub_one()));
            debug_assert!(*start_after <= end_new);
            end_value_after.end = end_new;
            for start in delete_list {
                self.btree_map.remove(&start);
            }
        } else {
            // last item extends beyond the new but has a different value.
            for &start in &delete_list[0..delete_list.len() - 1] {
                self.btree_map.remove(&start);
                // take the last one
            }
            let last = self
                .btree_map
                .remove(&delete_list[delete_list.len() - 1])
                .unwrap(); // there will always be a last
            let last_end = last.end;
            debug_assert!(end.add_one() <= last.end); // real assert
            self.btree_map.insert(end.add_one(), last);
            self.len += T::safe_len(&(end.add_one()..=last_end));
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
    /// let mut map = RangeMapBlaze::new();
    /// assert_eq!(map.insert(37, "a"), None);
    /// assert_eq!(map.is_empty(), false);
    ///
    /// map.insert(37, "b");
    /// assert_eq!(map.insert(37, "c"), Some("b"));
    /// assert_eq!(map[37], "c");
    /// ```
    pub fn insert(&mut self, key: T, value: V) -> Option<V> {
        let old = self.get(key).cloned();
        self.internal_add(key..=key, value);
        old
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
    /// let mut map = RangeMapBlaze::new();
    /// map.insert(3, "a");
    /// map.insert(5, "b");
    /// map.insert(8, "c");
    /// for (key, value) in map.range((Included(4), Included(8))) {
    ///     println!("{key}: {value}");
    /// } // prints "5: b" and "8: c"
    /// assert_eq!(Some((5, "b")), map.range(4..).next());
    /// ```
    pub fn range<R>(&self, range: R) -> IntoIterMap<T, V>
    where
        R: RangeBounds<T>,
    {
        // cmk 'range' should be made more efficient (it currently creates a RangeMapBlaze for no good reason)
        let start = match range.start_bound() {
            Bound::Included(n) => *n,
            Bound::Excluded(n) => (*n).add_one(),
            Bound::Unbounded => T::min_value(),
        };
        let end = match range.end_bound() {
            Bound::Included(n) => *n,
            Bound::Excluded(n) => (*n).sub_one(),
            Bound::Unbounded => T::max_value(),
        };
        assert!(start <= end);

        let bounds = CheckSortedDisjoint::new([start..=end]);
        Self::from_sorted_disjoint_map(self.range_values().intersection_with_set(bounds))
            .into_iter()
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
    /// use range_set_blaze::RangeMapBlaze;
    ///
    /// let mut map = RangeMapBlaze::new();
    /// assert_eq!(map.ranges_insert(2..=5, "a"), true);
    /// assert_eq!(map.ranges_insert(5..=6, "b"), true);
    /// assert_eq!(map.ranges_insert(3..=4, "c"), false);
    /// assert_eq!(map.len(), 5usize);
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
    /// let mut map = RangeMapBlaze::new();
    /// map.insert(1, "a");
    /// assert_eq!(map.remove(1), Some("a"));
    /// assert_eq!(map.remove(1), None);
    /// ```
    pub fn remove(&mut self, key: T) -> Option<V> {
        // The code can have only one mutable reference to self.btree_map.
        let (start_ref, end_value_mut) = self.btree_map.range_mut(..=key).next_back()?;
        if end_value_mut.end < key {
            return None;
        }
        let start = *start_ref;
        let end = end_value_mut.end;
        let value = end_value_mut.value.clone();
        if start < key {
            end_value_mut.end = key.sub_one();
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
            self.btree_map.insert(key.add_one(), end_value);
        }
        Some(value)
    }

    /// Splits the collection into two at the value. Returns a new collection
    /// with all elements greater than or equal to the value.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// use range_set_blaze::RangeMapBlaze;
    ///
    /// let mut a = RangeMapBlaze::new();
    /// a.insert(1, "a");
    /// a.insert(2, "b");
    /// a.insert(3, "c");
    /// a.insert(17, "d");
    /// a.insert(41, "e");
    ///
    /// let b = a.split_off(3);
    ///
    /// assert_eq!(a, RangeMapBlaze::from_iter([(1..=1, "a"), (2..=2, "b")]));
    /// assert_eq!(b, RangeMapBlaze::from_iter([(3..=3, "c"), (17..=17, "d"), (41..=41, "e")]));
    /// ```
    #[must_use]
    pub fn split_off(&mut self, key: T) -> Self {
        let old_len = self.len;
        let old_btree_len = self.btree_map.len();
        let mut new_btree = self.btree_map.split_off(&key);
        let Some(last_entry) = self.btree_map.last_entry() else {
            // Left is empty
            self.len = T::SafeLen::zero();
            return Self {
                btree_map: new_btree,
                len: old_len,
            };
        };

        let end_value = last_entry.get();
        let end = end_value.end;
        if end < key {
            // The split is clean
            let (a_len, b_len) = self.two_element_lengths(old_btree_len, &new_btree, old_len);
            self.len = a_len;
            return Self {
                btree_map: new_btree,
                len: b_len,
            };
        }

        // The split is not clean, so we must move some keys from the end of self to the start of b.
        let value = end_value.value.clone();
        last_entry.into_mut().end = key.sub_one();
        new_btree.insert(key, EndValue { end, value });
        let (a_len, b_len) = self.two_element_lengths(old_btree_len, &new_btree, old_len);
        self.len = a_len;
        Self {
            btree_map: new_btree,
            len: b_len,
        }
    }

    // Find the len of the smaller btree_map and then the element len of self & b.
    fn two_element_lengths(
        &self,
        old_btree_len: usize,
        new_btree: &BTreeMap<T, EndValue<T, V>>,
        mut old_len: <T as Integer>::SafeLen,
    ) -> (<T as Integer>::SafeLen, <T as Integer>::SafeLen) {
        if old_btree_len / 2 < new_btree.len() {
            let a_len = Self::btree_map_len(&self.btree_map);
            old_len -= a_len;
            (a_len, old_len)
        } else {
            let b_len = Self::btree_map_len(new_btree);
            old_len -= b_len;
            (old_len, b_len)
        }
    }

    #[allow(dead_code)] // cmk
    fn btree_map_len(btree_map: &BTreeMap<T, EndValue<T, V>>) -> T::SafeLen {
        btree_map.iter().fold(
            <T as Integer>::SafeLen::zero(),
            |acc, (start, end_value)| acc + T::safe_len(&(*start..=end_value.end)),
        )
    }

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
    //         if end_value >= start.sub_one() {
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
        end_before
            .checked_add_one()
            .map_or(false, |end_before_succ| end_before_succ < start)
    }

    // https://stackoverflow.com/questions/49599833/how-to-find-next-smaller-key-in-btreemap-btreeset
    // https://stackoverflow.com/questions/35663342/how-to-modify-partially-remove-a-range-from-a-btreemap
    // cmk2 might be able to shorten code by combining cases
    // FUTURE: would be nice of BTreeMap to have a partition_point function that returns two iterators
    #[allow(clippy::too_many_lines)]
    #[allow(clippy::cognitive_complexity)]
    pub(crate) fn internal_add(&mut self, range: RangeInclusive<T>, value: V) {
        let (start, end) = range.clone().into_inner();

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
            self.len += T::safe_len(&(end_before..=end.sub_one()));
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
                Some(bb) if bb.1.end.add_one() == start && bb.1.value == value => Some(bb),
                _ => None,
            };

            // === case: values are different, new extends beyond before, and they start together and an interesting before-before
            // an interesting before-before: something before before, touching and with the same value as new
            if let Some(bb) = interesting_before_before {
                debug_assert!(!before_contains_new && !same_value && same_start);

                // AABBBB
                //   aaaaaaa
                // AAAAAAAAA
                self.len += T::safe_len(&(bb.1.end.add_one()..=end));
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
            self.len += T::safe_len(&(end_before.add_one()..=end));
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
                debug_assert!(start_before <= start.sub_one()); // real assert
                end_value_before.end = start.sub_one(); // cmk overflow danger?
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
            debug_assert!(start_before <= start.sub_one()); // real assert
            end_value_before.end = start.sub_one();
            let before_value = end_value_before.value.clone();
            debug_assert!(start <= end); // real assert
            self.btree_map.insert(start, EndValue { end, value });
            debug_assert!(end.add_one() <= end_before); // real assert
            self.btree_map.insert(
                end.add_one(),
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
            debug_assert!(start_before <= start.sub_one()); // real assert
            end_value_before.end = start.sub_one();
            debug_assert!(start <= end); // real assert
            self.btree_map.insert(start, EndValue { end, value });
            self.delete_extra(&(start..=end));
            return;
        }

        // Thus, values are different, before contains new, and they start together

        let interesting_before_before = match before_iter.next() {
            Some(bb) if bb.1.end.add_one() == start && bb.1.value == value => Some(bb),
            _ => None,
        };

        // === case: values are different, before contains new, and they start together and an interesting before-before
        // an interesting before-before: something before before, touching and with the same value as new
        if let Some(bb) = interesting_before_before {
            debug_assert!(before_contains_new && !same_value && same_start);

            // AABBBB???
            //   aaaa
            // AAAAAA???
            self.len += T::safe_len(&(bb.1.end.add_one()..=end));
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
            debug_assert!(end.add_one() <= end_before); // real assert
            self.btree_map.insert(
                end.add_one(),
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
    /// use range_set_blaze::prelude::*;
    ///
    /// let mut a = RangeMapBlaze::new();
    /// assert_eq!(a.len(), 0usize);
    /// a.insert(1, "a");
    /// assert_eq!(a.len(), 1usize);
    ///
    /// let a = RangeMapBlaze::from_iter([
    ///     (-170_141_183_460_469_231_731_687_303_715_884_105_728_i128..=10, "a"),
    ///     (-10..=170_141_183_460_469_231_731_687_303_715_884_105_726, "a")]);
    /// assert_eq!(
    ///     a.len(),
    ///     UIntPlusOne::UInt(340282366920938463463374607431768211455)
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
    /// let mut map = RangeMapBlaze::new();
    ///
    /// // entries can now be inserted into the empty map
    /// map.insert(1, "a");
    /// assert_eq!(map[1], "a");
    /// ```
    #[must_use]
    pub fn new() -> Self {
        Self {
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
    /// let mut map = RangeMapBlaze::new();
    ///
    /// map.insert(1, "a");
    /// map.insert(2, "b");
    /// while let Some((key, _val)) = map.pop_first() {
    ///     assert!(map.iter().all(|(k, _v)| k > key));
    /// }
    /// assert!(map.is_empty());
    /// ```
    // cmk doc that often must clone
    pub fn pop_first(&mut self) -> Option<(T, V)>
    where
        V: Clone,
    {
        let entry = self.btree_map.first_entry()?;
        // We must remove the entry because the key will change
        let (start, end_value) = entry.remove_entry();

        self.len -= T::SafeLen::one();
        if start == end_value.end {
            Some((start, end_value.value))
        } else {
            let value = end_value.value.clone();
            self.btree_map.insert(start.add_one(), end_value);
            Some((start, value))
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
    /// let mut map = RangeMapBlaze::new();
    /// map.insert(1, "a");
    /// map.insert(2, "b");
    /// while let Some((key, _val)) = map.pop_last() {
    ///      assert!(map.iter().all(|(k, _v)| k < key));
    /// }
    /// assert!(map.is_empty());
    /// ```
    pub fn pop_last(&mut self) -> Option<(T, V)> {
        let mut entry = self.btree_map.last_entry()?;
        let start = *entry.key();
        self.len -= T::SafeLen::one();
        let end = entry.get().end;
        if start == end {
            let value = entry.remove_entry().1.value;
            Some((end, value))
        } else {
            let value = entry.get().value.clone();
            entry.get_mut().end.assign_sub_one();
            Some((end, value))
        }
    }

    /// An iterator that visits the ranges in the [`RangeMapBlaze`],
    /// i.e., the integers as sorted & disjoint ranges.
    ///
    /// Also see [`RangeMapBlaze::iter`] and [`RangeMapBlaze::into_range_values`].
    ///
    /// # Examples
    ///
    /// ```
    /// use range_set_blaze::RangeMapBlaze;
    ///
    /// let map = RangeMapBlaze::from_iter([(10..=20, "a"), (15..=25, "b"), (30..=40, "c")]);
    /// let mut range_values = map.range_values();
    /// assert_eq!(range_values.next(), Some((10..=20, &"a")));
    /// assert_eq!(range_values.next(), Some((21..=25, &"b")));
    /// assert_eq!(range_values.next(), Some((30..=40, &"c")));
    /// assert_eq!(range_values.next(), None);
    /// ```
    ///
    /// Values returned by the iterator are returned in ascending order
    /// with left-to-right precedence.
    ///
    /// ```
    /// use range_set_blaze::RangeMapBlaze;
    ///
    /// let map = RangeMapBlaze::from_iter([(30..=40, "c"), (15..=25, "b"), (10..=20, "a")]);
    /// let mut range_values = map.range_values();
    /// assert_eq!(range_values.next(), Some((10..=14, &"a")));
    /// assert_eq!(range_values.next(), Some((15..=25, &"b")));
    /// assert_eq!(range_values.next(), Some((30..=40, &"c")));
    /// assert_eq!(range_values.next(), None);
    /// ```
    pub fn range_values(&self) -> RangeValuesIter<'_, T, V> {
        RangeValuesIter {
            iter: self.btree_map.iter(),
        }
    }

    /// An iterator that visits the ranges in the [`RangeMapBlaze`],
    /// i.e., the integers as sorted & disjoint ranges.
    ///
    /// Also see [`RangeMapBlaze::iter`] and [`RangeMapBlaze::into_range_values`].
    ///
    /// # Examples
    ///
    /// ```
    /// extern crate alloc;
    /// use alloc::rc::Rc;
    /// use range_set_blaze::RangeMapBlaze;
    ///
    /// let map = RangeMapBlaze::from_iter([(10..=20, "a"), (15..=25, "b"), (30..=40, "c")]);
    /// let mut range_values = map.into_range_values();
    /// assert_eq!(range_values.next(), Some((10..=20, Rc::new("a"))));
    /// assert_eq!(range_values.next(), Some((21..=25, Rc::new("b"))));
    /// assert_eq!(range_values.next(), Some((30..=40, Rc::new("c"))));
    /// assert_eq!(range_values.next(), None);
    /// ```
    ///
    /// Values returned by the iterator are returned in ascending order
    /// with left-to-right precedence.
    ///
    /// ```
    /// extern crate alloc;
    /// use alloc::rc::Rc;
    /// use range_set_blaze::RangeMapBlaze;
    ///
    /// let map = RangeMapBlaze::from_iter([(30..=40, "c"), (15..=25, "b"), (10..=20, "a")]);
    /// let mut range_values = map.into_range_values();
    /// assert_eq!(range_values.next(), Some((10..=14, Rc::new("a"))));
    /// assert_eq!(range_values.next(), Some((15..=25, Rc::new("b"))));
    /// assert_eq!(range_values.next(), Some((30..=40, Rc::new("c"))));
    /// assert_eq!(range_values.next(), None);
    /// ```
    pub fn into_range_values(self) -> IntoRangeValuesIter<T, V> {
        IntoRangeValuesIter {
            iter: self.btree_map.into_iter(),
        }
    }

    /// An iterator that visits the ranges in the [`RangeMapBlaze`],
    /// i.e., the integers as sorted & disjoint ranges.
    ///
    /// Also see [`RangeMapBlaze::iter`] and [`RangeMapBlaze::into_range_values`].
    ///
    /// # Examples
    ///
    /// ```
    /// use range_set_blaze::RangeMapBlaze;
    ///
    /// let map = RangeMapBlaze::from_iter([(10..=20, "a"), (15..=25, "b"), (30..=40, "c")]);
    /// let mut ranges = map.ranges();
    /// assert_eq!(ranges.next(), Some(10..=25));
    /// assert_eq!(ranges.next(), Some(30..=40));
    /// assert_eq!(ranges.next(), None);
    /// ```
    ///
    /// Values returned by the iterator are returned in ascending order
    /// with left-to-right precedence.
    ///
    /// ```
    /// use range_set_blaze::RangeMapBlaze;
    ///
    /// let map = RangeMapBlaze::from_iter([(30..=40, "c"), (15..=25, "b"), (10..=20, "a")]);
    /// let mut ranges = map.ranges();
    /// assert_eq!(ranges.next(), Some(10..=25));
    /// assert_eq!(ranges.next(), Some(30..=40));
    /// assert_eq!(ranges.next(), None);
    /// ```
    pub fn ranges(&self) -> MapRangesIter<T, V> {
        MapRangesIter::new(self.btree_map.iter())
    }

    /// An iterator that visits the ranges in the [`RangeMapBlaze`],
    /// i.e., the integers as sorted & disjoint ranges.
    ///
    /// Also see [`RangeMapBlaze::iter`] and [`RangeMapBlaze::into_range_values`].
    ///
    /// # Examples
    ///
    /// ```
    /// use range_set_blaze::RangeMapBlaze;
    ///
    /// let map = RangeMapBlaze::from_iter([(10..=20, "a"), (15..=25, "b"), (30..=40, "c")]);
    /// let mut ranges = map.into_ranges();
    /// assert_eq!(ranges.next(), Some(10..=25));
    /// assert_eq!(ranges.next(), Some(30..=40));
    /// assert_eq!(ranges.next(), None);
    /// ```
    ///
    /// Values returned by the iterator are returned in ascending order
    /// with left-to-right precedence.
    ///
    /// ```
    /// use range_set_blaze::RangeMapBlaze;
    ///
    /// let map = RangeMapBlaze::from_iter([(30..=40, "c"), (15..=25, "b"), (10..=20, "a")]);
    /// let mut ranges = map.into_ranges();
    /// assert_eq!(ranges.next(), Some(10..=25));
    /// assert_eq!(ranges.next(), Some(30..=40));
    /// assert_eq!(ranges.next(), None);
    /// ```
    pub fn into_ranges(self) -> MapIntoRangesIter<T, V> {
        MapIntoRangesIter::new(self.btree_map.into_iter())
    }

    /// ```
    /// use range_set_blaze::RangeMapBlaze;
    ///
    /// let map = RangeMapBlaze::from_iter([(10u16..=20, "a"), (15..=25, "b"), (30..=40, "c")]);
    /// let complement = map.complement_with(&"z");
    /// assert_eq!(complement.to_string(), r#"(0..=9, "z"), (26..=29, "z"), (41..=65535, "z")"#);
    /// ```
    #[must_use]
    pub fn complement_with(&self, value: &V) -> Self {
        self.ranges()
            .complement()
            .map(|r| (r, value.clone()))
            .collect()
    }

    // FUTURE BTreeSet some of these as 'const' but it uses unstable. When stable, add them here and elsewhere.

    // cmk why is this must_use but not the others?
    /// Returns the number of sorted & disjoint ranges in the set.
    ///
    /// # Example
    ///
    /// ```
    /// use range_set_blaze::RangeMapBlaze;
    ///
    /// // We put in four ranges, but they are not sorted & disjoint.
    /// let map = RangeMapBlaze::from_iter([(10..=20, "a"), (15..=25, "b"), (30..=40, "c"), (28..=35, "c")]);
    /// // After RangeMapBlaze sorts & 'disjoint's them, we see three ranges.
    /// assert_eq!(map.range_values_len(), 3);
    /// assert_eq!(map.to_string(), r#"(10..=20, "a"), (21..=25, "b"), (28..=40, "c")"#);
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

    // cmk00 is this really the only place we need this?
    // cmk00 is this tested enough?
    /// cmk doc
    #[must_use]
    pub fn intersection_with_set(&self, other: &RangeSetBlaze<T>) -> Self {
        self.range_values()
            .intersection_with_set(other.ranges())
            .into_range_map_blaze()
    }
}

impl<T, V> IntoIterator for RangeMapBlaze<T, V>
where
    T: Integer,
    V: EqClone,
{
    type Item = (T, V);
    type IntoIter = IntoIterMap<T, V>;

    /// Gets a (double-ended) iterator for moving out the [`RangeSetBlaze`]'s integer contents.
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
    ///
    /// let set = RangeSetBlaze::from_iter([1, 2, 3, 4]);
    /// let v: Vec<_> = set.into_iter().rev().collect();
    /// assert_eq!(v, [4, 3, 2, 1]);
    /// ```
    fn into_iter(self) -> IntoIterMap<T, V> {
        IntoIterMap::new(self.btree_map.into_iter())
    }
}

// Implementing `IntoIterator` for `&RangeMapBlaze<T, V>`
impl<'a, T: Integer, V: EqClone> IntoIterator for &'a RangeMapBlaze<T, V> {
    type IntoIter = IterMap<T, &'a V, RangeValuesIter<'a, T, V>>;
    type Item = (T, &'a V);

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

// cmk remove
// #[doc(hidden)]
// pub type MergeMapAdjusted<'a, T, V, VR, L, R> =
//     MergeMap<'a, T, V, VR, AdjustPriorityMap<'a, T, V, VR, L>, AdjustPriorityMap<'a, T, V, VR, R>>;
// #[doc(hidden)]
// pub type BitOrMergeMap<'a, T, V, VR, L, R> =
//     UnionIterMap<'a, T, V, VR, MergeMapAdjusted<'a, T, V, VR, L, R>>;

#[doc(hidden)]
#[allow(clippy::module_name_repetitions)]
pub type BitAndRangesMap<T, VR, L, R> = IntersectionIterMap<T, VR, L, R>;
#[doc(hidden)]
pub type BitAndRangesMap2<T, VR, L, R> =
    BitAndRangesMap<T, VR, L, RangeValuesToRangesIter<T, VR, R>>;

#[doc(hidden)]
#[allow(clippy::module_name_repetitions)]
pub type BitSubRangesMap<T, VR, L, R> = IntersectionIterMap<T, VR, L, NotIter<T, R>>;
#[doc(hidden)]
pub type BitSubRangesMap2<T, VR, L, R> =
    BitSubRangesMap<T, VR, L, RangeValuesToRangesIter<T, VR, R>>;

#[doc(hidden)]
#[allow(clippy::module_name_repetitions)]
pub type SortedStartsInVecMap<T, VR> =
    AssumePrioritySortedStartsMap<T, VR, vec::IntoIter<Priority<T, VR>>>;

#[doc(hidden)]
pub type SortedStartsInVec<T> = AssumeSortedStarts<T, vec::IntoIter<RangeInclusive<T>>>;

impl<T: Integer, V: EqClone> BitOr<Self> for RangeMapBlaze<T, V> {
    /// Unions the contents of two [`RangeMapBlaze`]'s.
    ///
    /// Passing ownership rather than borrow sometimes allows a many-times
    /// faster speed up.
    ///
    /// # Examples
    /// ```
    /// use range_set_blaze::RangeMapBlaze;
    /// let a = RangeMapBlaze::from_iter([(1..=2, "a"), (5..=100, "a")]);
    /// let b = RangeMapBlaze::from_iter([(2..=6, "b")]);
    /// let union = a | b;
    /// assert_eq!(union, RangeMapBlaze::from_iter([(1..=2, "a"), (3..=4, "b"), (5..=100, "a")]));
    /// ```
    type Output = Self;
    fn bitor(self, other: Self) -> Self {
        // cmk
        // self |= other;
        // self
        (self.range_values() | other.range_values()).into_range_map_blaze()
    }
}

impl<T: Integer, V: EqClone> BitOr<&Self> for RangeMapBlaze<T, V> {
    /// Unions the contents of two [`RangeMapBlaze`]'s.
    ///
    /// Passing ownership rather than borrow sometimes allows a many-times
    /// faster speed up.
    ///
    /// # Examples
    /// ```
    /// use range_set_blaze::RangeMapBlaze;
    /// let a = RangeMapBlaze::from_iter([(1..=2, "a"), (5..=100, "a")]);
    /// let b = RangeMapBlaze::from_iter([(2..=6, "b")]);
    /// let union = a | &b;
    /// assert_eq!(union, RangeMapBlaze::from_iter([(1..=2, "a"), (3..=4, "b"), (5..=100, "a")]));
    /// ```
    type Output = Self;
    fn bitor(self, other: &Self) -> Self {
        // self |= other;
        // self
        (self.range_values() | other.range_values()).into_range_map_blaze()
    }
}

impl<T: Integer, V: EqClone> BitOr<RangeMapBlaze<T, V>> for &RangeMapBlaze<T, V> {
    type Output = RangeMapBlaze<T, V>;
    /// Unions the contents of two [`RangeMapBlaze`]'s.
    ///
    /// Passing ownership rather than borrow sometimes allows a many-times
    /// faster speed up.
    ///
    /// # Examples
    /// ```
    /// use range_set_blaze::RangeMapBlaze;
    /// let a = RangeMapBlaze::from_iter([(1..=2, "a"), (5..=100, "a")]);
    /// let b = RangeMapBlaze::from_iter([(2..=6, "b")]);
    /// let union = &a | b;
    /// assert_eq!(union, RangeMapBlaze::from_iter([(1..=2, "a"), (3..=4, "b"), (5..=100, "a")]));
    /// ```
    fn bitor(self, other: RangeMapBlaze<T, V>) -> RangeMapBlaze<T, V> {
        // cmk
        // other |= self;
        // other
        (self.range_values() | other.range_values()).into_range_map_blaze()
    }
}

impl<T: Integer, V: EqClone> BitOr<&RangeMapBlaze<T, V>> for &RangeMapBlaze<T, V> {
    type Output = RangeMapBlaze<T, V>;
    /// Unions the contents of two [`RangeMapBlaze`]'s.
    ///
    /// Passing ownership rather than borrow sometimes allows a many-times
    /// faster speed up.
    ///
    /// # Examples
    /// ```
    /// use range_set_blaze::RangeMapBlaze;
    /// let a = RangeMapBlaze::from_iter([(1..=2, "a"), (5..=100, "a")]);
    /// let b = RangeMapBlaze::from_iter([(2..=6, "b")]);
    /// let union = &a | &b;
    /// assert_eq!(union, RangeMapBlaze::from_iter([(1..=2, "a"), (3..=4, "b"), (5..=100, "a")]));
    /// ```
    fn bitor(self, other: &RangeMapBlaze<T, V>) -> RangeMapBlaze<T, V> {
        (self.range_values() | other.range_values()).into_range_map_blaze()
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
    /// let a = RangeMapBlaze::from_iter([(1..=2, "a"), (5..=100, "a")]);
    /// let b = RangeMapBlaze::from_iter([(2..=6, "b")]);
    /// let result = &a & &b; // Alternatively, 'a & b'.
    /// assert_eq!(result.to_string(), r#"(2..=2, "a"), (5..=6, "a")"#);
    /// ```
    for & call |a: &RangeMapBlaze<T, V>, b: &RangeMapBlaze<T, V>| {
        a.range_values().intersection_with_set(b.ranges()).into_range_map_blaze()
    };
/// Symmetric difference the contents of two [`RangeMapBlaze`]'s.
///
/// Either, neither, or both inputs may be borrowed.
///
/// # Examples
/// ```
/// use range_set_blaze::prelude::*;
///
/// let a = RangeMapBlaze::from_iter([(1..=2, "a"), (5..=100, "a")]);
/// let b = RangeMapBlaze::from_iter([(2..=6, "b")]);
/// let result = &a ^ &b; // Alternatively, 'a ^ b'.
/// assert_eq!(result.to_string(), r#"(1..=1, "a"), (3..=4, "b"), (7..=100, "a")"#);
/// ```
for ^ call |a: &RangeMapBlaze<T, V>, b: &RangeMapBlaze<T, V>| {
    SymDiffIterMap::new2(a.range_values(), b.range_values()).into_range_map_blaze()
};
/// Difference the contents of two [`RangeSetBlaze`]'s.
///
/// Either, neither, or both inputs may be borrowed.
///
/// # Examples
/// ```
/// use range_set_blaze::prelude::*;
///
/// let a = RangeMapBlaze::from_iter([(1..=2, "a"), (5..=100, "a")]);
/// let b = RangeMapBlaze::from_iter([(2..=6, "b")]);
/// let result = &a - &b; // Alternatively, 'a - b'.
/// assert_eq!(result.to_string(), r#"(1..=1, "a"), (7..=100, "a")"#);
/// ```

for - call |a: &RangeMapBlaze<T, V>, b: &RangeMapBlaze<T, V>| {
    a.range_values().difference_with_set(b.ranges()).into_range_map_blaze()
};
where T: Integer, V: EqClone
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
    a.range_values().difference_with_set(b.ranges()).into_range_map_blaze()
};

/// cmk
for & call |a: &RangeMapBlaze<T, V>, b: &RangeSetBlaze<T>| {
    a.range_values().intersection_with_set(b.ranges()).into_range_map_blaze()
};

where T: Integer, V: EqClone
);

gen_ops_ex!(
    <T, V>;
    types ref RangeMapBlaze<T,V> => RangeSetBlaze<T>;


/// cmk
for ! call |a: &RangeMapBlaze<T, V>| {
    a.ranges().complement().into_range_set_blaze()
};
where T: Integer, V: EqClone
);

impl<T, V> Extend<(T, V)> for RangeMapBlaze<T, V>
where
    T: Integer,
    V: EqClone,
{
    /// Extends the [`RangeSetBlaze`] with the contents of a
    /// range iterator. cmk this has right-to-left priority -- like `BTreeMap`, but unlike most other `RangeSetBlaze` methods.

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
    #[inline]
    fn extend<I>(&mut self, iter: I)
    where
        I: IntoIterator<Item = (T, V)>,
    {
        let iter = iter.into_iter();

        // We gather adjacent values into ranges via UnsortedPriorityDisjointMap, but ignore the priority.
        for priority in UnsortedPriorityDisjointMap::new(iter.map(|(r, v)| (r..=r, Rc::new(v)))) {
            let (range, value) = priority.into_range_value();
            let value = Rc::try_unwrap(value).unwrap_or_else(|_| panic!("Failed to unwrap Rc"));
            self.internal_add(range, value);
        }
    }

    // cmk define extend_one and make it inline
}

// cmk also from (RangeInclusive<T>, V(r))???
impl<T, V, const N: usize> From<[(T, V); N]> for RangeMapBlaze<T, V>
where
    T: Integer,
    V: EqClone,
{
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
    fn from(arr: [(T, V); N]) -> Self {
        arr.into_iter().collect()
    }
}

// implement Index trait
impl<T: Integer, V: EqClone> Index<T> for RangeMapBlaze<T, V> {
    type Output = V;

    /// Returns a reference to the value corresponding to the supplied key.
    ///
    /// # Panics
    ///
    /// Panics if the key is not present in the `BTreeMap`.
    #[inline]
    fn index(&self, index: T) -> &Self::Output {
        self.get(index).expect("no entry found for key")
    }
}

// cmk missing methods
// cmk retain -- inline
// cmk into_keys -- inline
// cmk into_values -- inline
// cmk trait PartialOrd with inline partial_cmp and cmp
// cmk look at other BTreeMap methods and traits

// cmk missing values and values per range

// cmk add difference_with_set??? is there a complement with set? sub_assign

impl<T, V> PartialOrd for RangeMapBlaze<T, V>
where
    T: Integer,
    V: EqClone + Ord,
{
    /// We define a partial ordering on `RangeMapBlaze`. Following the convention of
    /// [`BTreeMap`], the ordering is lexicographic, *not* by subset/superset.
    ///
    /// [`BTreeMap`]: alloc::collections::BTreeMap
    ///
    /// # Examples
    /// ```
    /// use range_set_blaze::prelude::*;
    ///
    /// let a = RangeMapBlaze::from_iter([(1..=3, "a"), (5..=100, "a")]);
    /// let b = RangeMapBlaze::from_iter([(2..=2, "b")] );
    /// assert!(a < b); // Lexicographic comparison
    /// // More lexicographic comparisons
    /// assert!(a <= b);
    /// assert!(b > a);
    /// assert!(b >= a);
    /// assert!(a != b);
    /// assert!(a == a);
    /// use core::cmp::Ordering;
    /// assert_eq!(a.cmp(&b), Ordering::Less);
    ///
    /// // Floats aren't comparable, but we can convert them to comparable bits.
    /// let a = RangeMapBlaze::from_iter([(2..=3, 1.0f32.to_bits()), (5..=100, 2.0f32.to_bits())]);
    /// let b = RangeMapBlaze::from_iter([(2..=2, f32::NAN.to_bits())] );
    /// assert_eq!(a.partial_cmp(&b), Some(Ordering::Less));
    /// ```
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<T, V> Ord for RangeMapBlaze<T, V>
where
    T: Integer,
    V: EqClone + Ord,
{
    /// We define an ordering on `RangeMapBlaze`. Following the convention of
    /// [`BTreeMap`], the ordering is lexicographic, *not* by subset/superset.
    ///
    /// [`BTreeMap`]: alloc::collections::BTreeMap
    ///
    /// # Examples
    /// ```
    /// use range_set_blaze::prelude::*;
    ///
    /// let a = RangeMapBlaze::from_iter([(1..=3, "a"), (5..=100, "a")]);
    /// let b = RangeMapBlaze::from_iter([(2..=2, "b")] );
    /// assert!(a < b); // Lexicographic comparison
    /// // More lexicographic comparisons
    /// assert!(a <= b);
    /// assert!(b > a);
    /// assert!(b >= a);
    /// assert!(a != b);
    /// assert!(a == a);
    /// use core::cmp::Ordering;
    /// assert_eq!(a.cmp(&b), Ordering::Less);
    ///
    /// // Floats aren't comparable, but we can convert them to comparable bits.
    /// let a = RangeMapBlaze::from_iter([(2..=3, 1.0f32.to_bits()), (5..=100, 2.0f32.to_bits())]);
    /// let b = RangeMapBlaze::from_iter([(2..=2, f32::NAN.to_bits())] );
    /// assert_eq!(a.partial_cmp(&b), Some(Ordering::Less));
    /// ```
    #[inline]
    fn cmp(&self, other: &Self) -> Ordering {
        // fast by ranges:
        let mut a = self.range_values();
        let mut b = other.range_values();
        let mut a_rx = a.next();
        let mut b_rx = b.next();
        loop {
            // compare Some/None
            match (a_rx, &b_rx) {
                (Some(_), None) => return Ordering::Greater,
                (None, Some(_)) => return Ordering::Less,
                (None, None) => return Ordering::Equal,
                (Some((a_r, a_v)), Some((b_r, b_v))) => {
                    // if tie, compare starts
                    match a_r.start().cmp(b_r.start()) {
                        Ordering::Greater => return Ordering::Greater,
                        Ordering::Less => return Ordering::Less,
                        Ordering::Equal => { /* keep going */ }
                    }

                    // if tie, compare values
                    match a_v.cmp(b_v) {
                        Ordering::Less => return Ordering::Less,
                        Ordering::Greater => return Ordering::Greater,
                        Ordering::Equal => { /* keep going */ }
                    }

                    // if tie, compare ends
                    match a_r.end().cmp(b_r.end()) {
                        Ordering::Less => {
                            a_rx = a.next();
                            b_rx = Some(((*a_r.end()).add_one()..=*b_r.end(), b_v));
                        }
                        Ordering::Greater => {
                            a_rx = Some(((*b_r.end()).add_one()..=*a_r.end(), a_v));
                            b_rx = b.next();
                        }
                        Ordering::Equal => {
                            a_rx = a.next();
                            b_rx = b.next();
                        }
                    }
                }
            }
        }
    }
}

impl<T: Integer, V: EqClone> Eq for RangeMapBlaze<T, V> {}

#[cfg(feature = "std")]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cmp() {
        let test_cases = vec![
            (
                vec![(2, 1.0), (11, 1.0), (12, 1.0)],
                vec![(3, 2.0), (11, 1.0), (12, f64::NAN)],
                Ordering::Less,
            ),
            // Mixed case
            (
                vec![(0, 1.0), (1, 2.0), (2, 1.0), (3, 3.0), (4, 2.0)],
                vec![(0, 1.0), (1, 1.0), (2, 1.0), (3, 2.0), (4, 2.0)],
                Ordering::Greater,
            ),
            // Equal elements
            (
                vec![(0, 1.0), (1, 1.0), (2, 1.0), (3, 2.0), (4, 2.0), (5, 2.0)],
                vec![(0, 1.0), (1, 1.0), (2, 1.0), (3, 2.0), (4, 2.0), (5, 2.0)],
                Ordering::Equal,
            ),
            // Different values
            (
                vec![(0, 1.0), (1, 1.0), (2, 1.0), (3, 2.0), (4, 2.0), (5, 2.0)],
                vec![(0, 1.0), (1, 1.0), (2, 1.0), (3, 2.0), (4, 2.0), (5, 3.0)],
                Ordering::Less,
            ),
            (
                vec![(0, 1.0), (1, 1.0), (2, 1.0), (3, 2.0), (4, 2.0), (5, 3.0)],
                vec![(0, 1.0), (1, 1.0), (2, 1.0), (3, 2.0), (4, 2.0), (5, 2.0)],
                Ordering::Greater,
            ),
            // Different keys
            (
                vec![(0, 1.0), (1, 1.0), (2, 1.0), (4, 2.0), (5, 2.0)],
                vec![(0, 1.0), (1, 1.0), (2, 1.0), (3, 2.0), (4, 2.0), (5, 2.0)],
                Ordering::Greater,
            ),
            (
                vec![(0, 1.0), (1, 1.0), (2, 1.0)],
                vec![(0, 1.0), (1, 1.0), (2, 1.0), (3, 2.0), (4, 2.0), (5, 2.0)],
                Ordering::Less,
            ),
            (
                vec![(0, 1.0), (1, 1.0), (2, 1.0), (3, 2.0), (4, 2.0), (5, 2.0)],
                vec![(0, 1.0), (1, 1.0), (2, 1.0)],
                Ordering::Greater,
            ),
            // To apply .to_bits() so that NANs are compared, too.
            (
                vec![(0, 1.0), (1, 1.0), (2, f64::NAN)],
                vec![(0, 1.0), (1, 1.0), (2, 1.0)],
                Ordering::Greater,
            ),
            (
                vec![(0, 1.0), (1, 1.0), (2, 1.0)],
                vec![(0, 1.0), (1, 1.0), (2, f64::NAN)],
                Ordering::Less,
            ),
        ];

        fn to_bits(vv_pair: Vec<(u32, f64)>) -> Vec<(u32, u64)> {
            vv_pair.into_iter().map(|(k, v)| (k, v.to_bits())).collect()
        }
        let test_cases = test_cases
            .into_iter()
            .map(|(a, b, expected)| (to_bits(a), to_bits(b), expected));

        for (a_data, b_data, expected) in test_cases {
            println!("expected = {expected:?}");
            let a_btree = BTreeMap::from_iter(a_data.clone());
            let b_btree = BTreeMap::from_iter(b_data.clone());
            assert_eq!(a_btree.cmp(&b_btree), expected);

            let a_range_set = RangeMapBlaze::from_iter(a_data);
            let b_range_set = RangeMapBlaze::from_iter(b_data);
            assert_eq!(a_range_set.cmp(&b_range_set), expected);
        }
    }
}
