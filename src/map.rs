use crate::{
    CheckSortedDisjoint, Integer, IntoKeys, Keys, RangeSetBlaze, SortedDisjoint,
    iter_map::{IntoIterMap, IterMap},
    map_op, map_unary_op,
    range_values::{IntoRangeValuesIter, MapIntoRangesIter, MapRangesIter, RangeValuesIter},
    set::extract_range,
    sorted_disjoint_map::{IntoString, SortedDisjointMap},
    sym_diff_iter_map::SymDiffIterMap,
    unsorted_priority_map::{SortedDisjointMapWithLenSoFar, UnsortedPriorityMap},
    values::{IntoValues, Values},
};
#[cfg(feature = "std")]
use alloc::sync::Arc;
use alloc::vec::Vec;
use alloc::{collections::BTreeMap, rc::Rc};
use core::{
    borrow::Borrow,
    cmp::{Ordering, max},
    convert::From,
    fmt, mem,
    ops::{BitOr, BitOrAssign, Index, RangeBounds, RangeInclusive},
    panic,
};
use num_traits::{One, Zero};

const STREAM_OVERHEAD: usize = 10;

/// A trait for references to `Eq + Clone` values, used by the [`SortedDisjointMap`] trait.
///
/// `ValueRef` enables [`SortedDisjointMap`] to map sorted, disjoint ranges of integers
/// to values of type `V: Eq + Clone`. It supports both references (`&V`) and shared ownership types
/// (`Rc<V>` and `Arc<V>`), avoiding unnecessary cloning of the underlying values while allowing
/// ownership when needed.
///
/// References must also implement `Clone`. Standard reference types like `&V`, `Rc<V>`, and `Arc<V>`
/// implement `Clone` efficiently, as cloning typically involves copying a pointer.
///
/// # Motivation
///
/// Iterating over `(range, value)` pairs, such as with [`RangeMapBlaze::ranges`], benefits from
/// references to values, which are cheap to clone. However, iterators like [`RangeMapBlaze::into_ranges`]
/// require ownership. By supporting `Rc<Eq + Clone>` and `Arc<Eq + Clone>`, `ValueRef` allows shared ownership,
/// freeing memory when the reference count drops to zero.
///
// # Examples
///
/// The following demonstrates the [`SortedDisjointMap::intersection`] operation working with
/// iterators of both `(RangeInclusive<Integer>, &Eq + Clone)` and `(RangeInclusive<Integer>, Rc<Eq + Clone>)`.
/// (However, types cannot be mixed in the same operation due to Rust's strong type system.)
///
/// ```rust
/// use range_set_blaze::prelude::*;
/// use std::rc::Rc;
///
/// let a = RangeMapBlaze::from_iter([(2..=3, "a".to_string()), (5..=100, "a".to_string())]);
/// let b = RangeMapBlaze::from_iter([(3..=10, "b".to_string())]);
///
/// let mut c = a.range_values() & b.range_values();
/// assert_eq!(c.next(), Some((3..=3, &"a".to_string())));
/// assert_eq!(c.next(), Some((5..=10, &"a".to_string())));
/// assert_eq!(c.next(), None);
///
/// let mut c = a.into_range_values() & b.into_range_values();
/// assert_eq!(c.next(), Some((3..=3, Rc::new("a".to_string()))));
/// assert_eq!(c.next(), Some((5..=10, Rc::new("a".to_string()))));
/// assert_eq!(c.next(), None);
/// ```
pub trait ValueRef: Borrow<Self::Target> + Clone {
    /// The `Eq + Clone` value type to which the reference points.
    type Target: Eq + Clone;

    /// Converts a reference or shared pointer to an owned value of type `Self::Target`.
    ///
    /// This method allows values of type `Self` (e.g., `&V`, `Rc<V>`, or `Arc<V>`) to be turned
    /// into a fully owned `V`, which is required when consuming or storing values independently.
    ///
    /// - For plain references (`&V`), this clones the referenced value.
    /// - For `Rc<V>` and `Arc<V>`, this attempts to unwrap the value if uniquely owned;
    ///   otherwise, it clones the inner value.
    ///
    /// This method is typically used when converting a stream of `(range, value)` pairs
    /// into owned data, such as when building a new `RangeMapBlaze` from an iterator.
    ///
    /// # Example
    /// ```
    /// use std::rc::Rc;
    /// use range_set_blaze::ValueRef;
    ///
    /// let rc = Rc::new("hello".to_string());
    /// let owned: String = rc.to_owned(); // avoids cloning if ref count is 1
    /// ```
    fn to_owned(self) -> Self::Target;
}

// Implementations for references and smart pointers
impl<V> ValueRef for &V
where
    V: Eq + Clone,
{
    type Target = V;

    #[inline]
    fn to_owned(self) -> Self::Target {
        self.clone()
    }
}

impl<V> ValueRef for Rc<V>
where
    V: Eq + Clone,
{
    type Target = V;

    #[inline]
    fn to_owned(self) -> Self::Target {
        Self::try_unwrap(self).unwrap_or_else(|rc| (*rc).clone())
    }
}

#[cfg(feature = "std")]
impl<V> ValueRef for Arc<V>
where
    V: Eq + Clone,
{
    type Target = V;

    #[inline]
    fn to_owned(self) -> Self::Target {
        Self::try_unwrap(self).unwrap_or_else(|arc| (*arc).clone())
    }
}

#[expect(clippy::redundant_pub_crate)]
#[derive(Clone, Hash, Default, PartialEq, Eq, Debug)]
pub(crate) struct EndValue<T, V>
where
    T: Integer,
    V: Eq + Clone,
{
    pub(crate) end: T,
    pub(crate) value: V,
}

/// A map from integers to values stored as a map of sorted & disjoint ranges to values.
///
/// Internally, the map stores the
/// ranges and values in a cache-efficient [`BTreeMap`].
///
/// # Table of Contents
/// * [`RangeMapBlaze` Constructors](#rangemapblaze-constructors)
///    * [Performance](#constructor-performance)
///    * [Examples](struct.RangeMapBlaze.html#constructor-examples)
/// * [`RangeMapBlaze` Set Operations](#rangemapblaze-set-operations)
///    * [Performance](struct.RangeMapBlaze.html#set-operation-performance)
///    * [Examples](struct.RangeMapBlaze.html#set-operation-examples)
/// * [`RangeMapBlaze` Union- and Extend-like Methods](#rangemapblaze-union--and-extend-like-methods)
/// * [`RangeMapBlaze` Comparisons](#rangemapblaze-comparisons)
/// * [Additional Examples](#additional-examples)
///
/// # `RangeMapBlaze` Constructors
///
/// You can create `RangeMapBlaze`'s from unsorted and overlapping integers (or ranges), along with values.
/// However, if you know that your input is sorted and disjoint, you can speed up construction.
///
/// Here are the constructors, followed by a
/// description of the performance, and then some examples.
///
///
/// | Methods                                     | Input                        | Notes                    |
/// |---------------------------------------------|------------------------------|--------------------------|
/// | [`new`]/[`default`]                         |                              |                          |
/// | [`from_iter`][1]/[`collect`][1]           | iterator of `(integer, value)` | References to the pair or value is OK. |
/// | [`from_iter`][2]/[`collect`][2]           | iterator of `(range, value)`   | References to the pair or value is OK. |
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
/// * collect adjacent integers/ranges with equal values into disjoint ranges, O(*n₁*)
/// * sort the disjoint ranges by their `start`, O(*n₂* ln *n₂*)
/// * merge ranges giving precedence to the original left-most values, O(*n₂*)
/// * create a `BTreeMap` from the now sorted & disjoint ranges, O(*n₃* ln *n₃*)
///
/// where *n₁* is the number of input integers/ranges, *n₂* is the number of disjoint & unsorted ranges,
/// and *n₃* is the final number of sorted & disjoint ranges with equal values.
///
/// For example, an input of
///  * `(3, "a"), (2, "a"), (1, "a"), (4, "a"), (100, "c"), (5, "b"), (4, "b"), (5, "b")`, becomes
///  * `(1..=4, "a"), (100..=100, "c"), (4..=5, "b")`, and then
///  * `(1..=4, "a"), (4..=5, "b"), (100..=100, "c")`, and finally
///  * `(1..=4, "a"), (5..=5, "b"), (100..=100, "c")`.
///
/// What is the effect of clumpy data?
/// Notice that if *n₂* ≈ sqrt(*n₁*), then construction is O(*n₁*).
/// Indeed, as long as *n₂* ≤ *n₁*/ln(*n₁*), then construction is O(*n₁*).
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
/// let a0 = RangeMapBlaze::from_iter([ (100, "b"), (1, "c"),(3, "a"), (2, "a"), (1, "a")]);
/// let a1: RangeMapBlaze<i32, &str> = [(100, "b"), (1, "c"), (3, "a"), (2, "a"), (1, "a")].into_iter().collect();
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
/// // we can avoid work and use 'from_sorted_disjoint_map'/'into_sorted_disjoint_map'.
/// let a0 = RangeMapBlaze::from_sorted_disjoint_map(CheckSortedDisjointMap::new([(-10..=-5, &"c"), (1..=2, &"a")]));
/// let a1: RangeMapBlaze<i32, &str> = CheckSortedDisjointMap::new([(-10..=-5, &"c"), (1..=2, &"a")]).into_range_map_blaze();
/// assert!(a0 == a1 && a0.to_string() == r#"(-10..=-5, "c"), (1..=2, "a")"#);
///
/// // For compatibility with `BTreeMap`, we also support
/// // 'from'/'into' from arrays of integers.
/// let a0 = RangeMapBlaze::from([(3, "a"), (2, "a"), (1, "a"), (100, "b"), (1, "c")]);
/// let a1: RangeMapBlaze<i32, &str> = [(3, "a"), (2, "a"), (1, "a"), (100, "b"), (1, "c")].into();
/// assert!(a0 == a1 && a0.to_string() == r#"(1..=3, "a"), (100..=100, "b")"#);
/// ```
///
/// # `RangeMapBlaze` Set Operations
///
/// You can perform set operations on `RangeMapBlaze`s
/// and `RangeSetBlaze`s using operators. In the table below, `a`, `b`, and `c` are `RangeMapBlaze`s and `s` is a `RangeSetBlaze`.
///
/// | Set Operation           | Operator                           | Multiway Method                        |
/// |--------------------------|------------------------------------|----------------------------------------|
/// | union                    | [`a` &#124; `b`]                   | `[a, b, c]`.[`union`]\(\)              |
/// | intersection             | [`a & b`]                          | `[a, b, c]`.[`intersection`]\(\)              |
/// | intersection             | [`a & s`]                          | *n/a*                                  |
/// | difference               | [`a - b`]                          | *n/a*                                  |
/// | difference               | [`a - s`]                          | *n/a*                                  |
/// | symmetric difference     | [`a ^ b`]                          | `[a, b, c]`.[`symmetric_difference`]\(\) |
/// | complement (to set)      | [`!a`]                             | *n/a*                                  |
/// | complement (to map)      | [`a.complement_with(&value)`]      | *n/a*                                  |
///
/// The result of all operations is a new `RangeMapBlaze` except for `!a`, which returns a `RangeSetBlaze`.
///
/// The union of any number of maps is defined such that, for any overlapping keys,
/// the values from the left-most input take precedence. This approach ensures
/// that the data from the left-most inputs remains dominant when merging with
/// later inputs. Likewise, for symmetric difference of three or more maps.
///
/// `RangeMapBlaze` also implements many other methods, such as [`insert`], [`pop_first`] and [`split_off`]. Many of
/// these methods match those of `BTreeMap`.
///
/// [`a` &#124; `b`]: struct.RangeMapBlaze.html#impl-BitOr-for-RangeMapBlaze<T,+V>
/// [`a & b`]: struct.RangeMapBlaze.html#impl-BitAnd-for-RangeMapBlaze<T,+V>
/// [`a & s`]: struct.RangeMapBlaze.html#impl-BitAnd<%26RangeSetBlaze<T>>-for-%26RangeMapBlaze<T,+V>
/// [`a - b`]: struct.RangeMapBlaze.html#impl-Sub-for-RangeMapBlaze<T,+V>
/// [`a - s`]: struct.RangeMapBlaze.html#impl-Sub<%26RangeSetBlaze<T>>-for-%26RangeMapBlaze<T,+V>
/// [`a ^ b`]: struct.RangeMapBlaze.html#impl-BitXor-for-RangeMapBlaze<T,+V>
/// [`!a`]: struct.RangeMapBlaze.html#impl-Not-for-%26RangeMapBlaze<T,+V>
/// [`a.complement_with(&value)`]: struct.RangeMapBlaze.html#method.complement_with
/// [`union`]: trait.MultiwayRangeMapBlazeRef.html#method.union
/// [`intersection`]: trait.MultiwayRangeMapBlazeRef.html#method.intersection
/// [`symmetric_difference`]: trait.MultiwayRangeMapBlazeRef.html#method.symmetric_difference
/// [`insert`]: RangeMapBlaze::insert
/// [`pop_first`]: RangeMapBlaze::pop_first
/// [`split_off`]: RangeMapBlaze::split_off
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
/// Several union-related operators — such as [`|`] (union) and [`|=`] (union append) — include performance
/// optimizations for common cases, including when one operand is much smaller than the other.
/// These optimizations reduce allocations and merging overhead.
/// **See:** [Summary of Union and Extend-like Methods](#rangemapblaze-union--and-extend-like-methods).
///
/// [`SortedDisjointMap`]: trait.SortedDisjointMap.html#table-of-contents
/// [`|`]: struct.RangeMapBlaze.html#impl-BitOr-for-RangeMapBlaze%3CT,+V%3E
/// [`|=`]: struct.RangeMapBlaze.html#impl-BitOrAssign-for-RangeMapBlaze%3CT,+V%3E
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
///
/// # `RangeMapBlaze` Union- and Extend-like Methods
///
/// | Operation & Syntax                  | Input Type           | Precedence     | Pre-merge Touching | Cases Optimized |
/// |-------------------------------------|----------------------|----------------|---------------------|------------------|
/// | [`a` &#124;= `b`]                   | `RangeMapBlaze`      | Left-to-right  | -                   | 3                |
/// | [`a` &#124;= `&b`]                  | `&RangeMapBlaze`     | Left-to-right  | -                   | 3                |
/// | [`a` &#124; `b`]                    | `RangeMapBlaze`      | Left-to-right  | -                   | 3                |
/// | [`a` &#124; `&b`]                   | `&RangeMapBlaze`     | Left-to-right  | -                   | 3                |
/// | [`&a` &#124; `b`]                   | `RangeMapBlaze`      | Left-to-right  | -                   | 3                |
/// | [`&a` &#124; `&b`]                  | `&RangeMapBlaze`     | Left-to-right  | -                   | 3                |
/// | [`a.extend([(r, v)])`][extend_rv]   | iter `(range, value)`     | Right-to-left  | Yes                 | 1                |
/// | [`a.extend([(i, v)])`][extend_iv]   | iter `(integer, value)`   | Right-to-left  | Yes                 | 1                |
/// | [`a.extend_simple(...)`][extend_simple] | iter `(range, value)` | Right-to-left  | No                  | 1                |
/// | [`a.extend_with(&b)`][extend_with]  | `&RangeMapBlaze`     | Right-to-left  | -                   | 1                |
/// | [`a.extend_from(b)`][extend_from]   | `RangeMapBlaze`      | Right-to-left  | -                   | 1                |
/// | [`b.append(&mut a)`][append]        | `&mut RangeMapBlaze` | Right-to-left  | -                   | 1                |
///
/// Notes:
///
/// - **Pre-merge Touching** means adjacent or overlapping ranges with the same value are combined into a single range before insertions.
/// - **Cases Optimized** indicates how many usage scenarios have dedicated performance paths:
///     - `3` = optimized for small-left, small-right, and similar-sized inputs
///     - `1` = optimized for small-right inputs only
///
/// [`a` &#124;= `b`]: struct.RangeMapBlaze.html#impl-BitOrAssign-for-RangeMapBlaze%3CT,+V%3E
/// [`a` &#124;= `&b`]: struct.RangeMapBlaze.html#impl-BitOrAssign%3C%26RangeMapBlaze%3CT,+V%3E%3E-for-RangeMapBlaze%3CT,+V%3E
/// [`a` &#124; `b`]: struct.RangeMapBlaze.html#impl-BitOr-for-RangeMapBlaze%3CT,+V%3E
/// [`a` &#124; `&b`]: struct.RangeMapBlaze.html#impl-BitOr%3C%26RangeMapBlaze%3CT,+V%3E%3E-for-RangeMapBlaze%3CT,+V%3E
/// [`&a` &#124; `b`]: struct.RangeMapBlaze.html#impl-BitOr%3CRangeMapBlaze%3CT,+V%3E%3E-for-%26RangeMapBlaze%3CT,+V%3E
/// [`&a` &#124; `&b`]: struct.RangeMapBlaze.html#impl-BitOr%3C%26RangeMapBlaze%3CT,+V%3E%3E-for-%26RangeMapBlaze%3CT,+V%3E
/// [extend_rv]: struct.RangeMapBlaze.html#impl-Extend%3C(RangeInclusive%3CT%3E,+V)%3E-for-RangeMapBlaze%3CT,+V%3E
/// [extend_iv]: struct.RangeMapBlaze.html#impl-Extend%3C(T,+V)%3E-for-RangeMapBlaze%3CT,+V%3E
/// [extend_simple]: struct.RangeMapBlaze.html#method.extend_simple
/// [extend_with]: struct.RangeMapBlaze.html#method.extend_with
/// [extend_from]: struct.RangeMapBlaze.html#method.extend_from
/// [append]: struct.RangeMapBlaze.html#method.append
///
/// # `RangeMapBlaze` Comparisons
///
/// `RangeMapBlaze` supports comparisons for equality and lexicographic order:
///
/// - **Equality**: Use `==` and `!=` to check if two `RangeMapBlaze` instances
///   are equal. Two `RangeMapBlaze` instances are considered equal if they
///   contain the same ranges and associated values.
/// - **Ordering**: If the values implement `Ord`, you can use `<`, `<=`, `>`, and `>=`
///   to compare two `RangeMapBlaze` instances. These comparisons are lexicographic,
///   similar to `BTreeMap`, meaning they compare the ranges and their values in sequence.
/// - **Partial Ordering**: If the values implement `PartialOrd` but not `Ord`, you can use
///   the [`partial_cmp`] method to compare two `RangeMapBlaze` instances. This method returns
///   an `Option<Ordering>` that indicates the relative order of the instances or `None` if the
///   values are not comparable.
///
/// See [`partial_cmp`] and [`cmp`] for more examples.
///
///
/// [`BTreeMap`]: alloc::collections::BTreeMap
/// [`partial_cmp`]: RangeMapBlaze::partial_cmp
/// [`cmp`]: RangeMapBlaze::cmp
///
/// # Additional Examples
///
/// See the [module-level documentation] for additional examples.
///
/// [module-level documentation]: index.html
#[derive(Clone, Hash, PartialEq)]
pub struct RangeMapBlaze<T: Integer, V: Eq + Clone> {
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
impl<T: Integer, V: Eq + Clone> Default for RangeMapBlaze<T, V> {
    fn default() -> Self {
        Self {
            len: <T as Integer>::SafeLen::zero(),
            btree_map: BTreeMap::new(),
        }
    }
}

impl<T: Integer, V: Eq + Clone + fmt::Debug> fmt::Debug for RangeMapBlaze<T, V> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.range_values().into_string())
    }
}

impl<T: Integer, V: Eq + Clone + fmt::Debug> fmt::Display for RangeMapBlaze<T, V> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.range_values().into_string())
    }
}

impl<T: Integer, V: Eq + Clone> RangeMapBlaze<T, V> {
    /// Gets an iterator that visits the integer elements in the [`RangeMapBlaze`] in
    /// ascending and/or descending order. Double-ended.
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
    pub fn iter(&self) -> IterMap<T, &V, RangeValuesIter<'_, T, V>> {
        // If the user asks for an iter, we give them a RangesIter iterator
        // and we iterate that one integer at a time.
        IterMap::new(self.range_values())
    }

    /// Gets an iterator that visits the integer elements in the [`RangeMapBlaze`] in
    /// ascending and/or descending order. Double-ended.
    ///
    /// For a consuming version, see the [`RangeMapBlaze::into_keys`] method.
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
    /// Keys returned by `.next()` are in ascending order.
    /// Keys returned by `.next_back()` are in descending order.
    ///
    /// ```
    /// # use range_set_blaze::RangeMapBlaze;
    /// let map = RangeMapBlaze::from_iter([(3,"c"), (1,"a"), (2,"b")]);
    /// let mut keys_iter = map.keys();
    /// assert_eq!(keys_iter.next(), Some(1));
    /// assert_eq!(keys_iter.next_back(), Some(3));
    /// assert_eq!(keys_iter.next(), Some(2));
    /// assert_eq!(keys_iter.next_back(), None);
    /// ```
    pub fn keys(&self) -> Keys<T, &V, RangeValuesIter<'_, T, V>> {
        Keys::new(self.range_values())
    }

    /// Gets an iterator that visits the integer elements in the [`RangeMapBlaze`] in
    /// ascending and/or descending order. Double-ended.
    ///
    /// The iterator consumes the [`RangeMapBlaze`], yielding one integer at a time from its ranges.
    /// For a non-consuming version, see the [`RangeMapBlaze::keys`] method.
    ///
    /// # Examples
    ///
    /// Iterating in ascending order:
    ///
    /// ```
    /// use range_set_blaze::RangeMapBlaze;
    ///
    /// let map = RangeMapBlaze::from_iter([(1..=3, "a")]);
    /// let mut into_keys_iter = map.into_keys();
    /// assert_eq!(into_keys_iter.next(), Some(1));
    /// assert_eq!(into_keys_iter.next(), Some(2));
    /// assert_eq!(into_keys_iter.next(), Some(3));
    /// assert_eq!(into_keys_iter.next(), None);
    /// ```
    ///
    /// Iterating in both ascending and descending order:
    ///
    /// ```
    /// # use range_set_blaze::RangeMapBlaze;
    /// let map = RangeMapBlaze::from_iter([(1..=3, "a"), (5..=5, "b")]);
    /// let mut into_keys_iter = map.into_keys();
    /// assert_eq!(into_keys_iter.next(), Some(1));
    /// assert_eq!(into_keys_iter.next_back(), Some(5));
    /// assert_eq!(into_keys_iter.next(), Some(2));
    /// assert_eq!(into_keys_iter.next_back(), Some(3));
    /// assert_eq!(into_keys_iter.next(), None);
    /// ```
    ///
    /// Keys returned by `.next()` are in ascending order.
    /// Keys returned by `.next_back()` are in descending order.
    #[inline]
    pub fn into_keys(self) -> IntoKeys<T, V> {
        IntoKeys::new(self.btree_map.into_iter())
    }

    /// Gets an iterator that visits the values in the [`RangeMapBlaze`] in
    /// the order corresponding to the integer elements. Double-ended.
    ///
    /// For a consuming version, see the [`RangeMapBlaze::into_values`] method.
    ///
    /// # Examples
    ///
    /// Iterating over values:
    ///
    /// ```rust
    /// use range_set_blaze::RangeMapBlaze;
    ///
    /// let map = RangeMapBlaze::from_iter([(3, "c"), (1, "a"), (2, "b")]);
    /// let mut values_iter = map.values();
    /// assert_eq!(values_iter.next(), Some(&"a"));
    /// assert_eq!(values_iter.next(), Some(&"b"));
    /// assert_eq!(values_iter.next(), Some(&"c"));
    /// assert_eq!(values_iter.next(), None);
    /// ```
    ///
    /// Values returned by `.next()` are in the order of corresponding integer elements.
    /// Values returned by `.next_back()` correspond to elements in descending integer order.
    ///
    /// ```rust
    /// # use range_set_blaze::RangeMapBlaze;
    /// let map = RangeMapBlaze::from_iter([(3, "c"), (1, "a"), (2, "b")]);
    /// let mut values_iter = map.values();
    /// assert_eq!(values_iter.next(), Some(&"a"));
    /// assert_eq!(values_iter.next_back(), Some(&"c"));
    /// assert_eq!(values_iter.next(), Some(&"b"));
    /// assert_eq!(values_iter.next_back(), None);
    /// ```
    pub fn values(&self) -> Values<T, &V, RangeValuesIter<'_, T, V>> {
        Values::new(self.range_values())
    }

    /// Gets an iterator that visits the values in the [`RangeMapBlaze`] in
    /// the order corresponding to the integer elements. Double-ended.
    ///
    /// The iterator consumes the [`RangeMapBlaze`], yielding one value at a time for
    /// each integer in its ranges. For a non-consuming version, see the [`RangeMapBlaze::values`] method.
    ///
    /// # Examples
    ///
    /// Iterating over values in ascending order:
    ///
    /// ```rust
    /// use range_set_blaze::RangeMapBlaze;
    ///
    /// let map = RangeMapBlaze::from_iter([(3, "c"), (1, "a"), (2, "b")]);
    /// let mut into_values_iter = map.into_values();
    /// assert_eq!(into_values_iter.next(), Some("a"));
    /// assert_eq!(into_values_iter.next(), Some("b"));
    /// assert_eq!(into_values_iter.next(), Some("c"));
    /// assert_eq!(into_values_iter.next(), None);
    /// ```
    ///
    /// Iterating over values in both ascending and descending order:
    ///
    /// ```rust
    /// use range_set_blaze::RangeMapBlaze;
    ///
    /// let map = RangeMapBlaze::from_iter([(1..=3, "a"), (5..=5, "b")]);
    /// let mut into_values_iter = map.into_values();
    /// assert_eq!(into_values_iter.next(), Some("a"));
    /// assert_eq!(into_values_iter.next_back(), Some("b"));
    /// assert_eq!(into_values_iter.next(), Some("a"));
    /// assert_eq!(into_values_iter.next_back(), Some("a"));
    /// assert_eq!(into_values_iter.next(), None);
    /// ```
    ///
    /// Values returned by `.next()` correspond to elements in ascending integer order.
    /// Values returned by `.next_back()` correspond to elements in descending integer order.
    #[inline]
    pub fn into_values(self) -> IntoValues<T, V> {
        IntoValues::new(self.btree_map.into_iter())
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
        self.get_key_value(key).map(|(_, value)| value)
    }

    /// Returns the key and value in the map, if any, that contains the given key.
    ///
    /// # Examples
    ///
    /// ```
    /// use range_set_blaze::RangeMapBlaze;
    ///
    /// let map = RangeMapBlaze::from_iter([(3..=5, "c"), (1..=2, "a")]);
    /// assert_eq!(map.get_key_value(2), Some((2, &"a")));
    /// assert_eq!(map.get_key_value(4), Some((4, &"c")));
    /// assert_eq!(map.get_key_value(6), None);
    /// ```
    pub fn get_key_value(&self, key: T) -> Option<(T, &V)> {
        self.btree_map
            .range(..=key)
            .next_back()
            .and_then(|(_start, end_value)| {
                if key <= end_value.end {
                    Some((key, &end_value.value))
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

    /// Create a [`RangeMapBlaze`] from a [`SortedDisjointMap`] iterator.
    ///
    /// *For more about constructors and performance, see [`RangeMapBlaze` Constructors](struct.RangeMapBlaze.html#rangemapblaze-constructors).*
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
        VR: ValueRef<Target = V>,
        I: SortedDisjointMap<T, VR>,
    {
        let mut iter_with_len = SortedDisjointMapWithLenSoFar::new(iter);
        let btree_map: BTreeMap<T, EndValue<T, VR::Target>> = (&mut iter_with_len).collect();
        Self {
            btree_map,
            len: iter_with_len.len_so_far(),
        }
    }

    #[allow(dead_code)]
    #[must_use]
    pub(crate) fn len_slow(&self) -> <T as Integer>::SafeLen {
        Self::btree_map_len(&self.btree_map)
    }

    /// Moves all elements from `other` into `self`, leaving `other` empty.
    ///
    /// This method has *right-to-left precedence*: if any ranges overlap, values in `other`
    /// will overwrite those in `self`.
    ///
    /// # Performance
    ///
    /// This method inserts each range from `other` into `self` one-by-one, with overall time
    /// complexity `O(n log m)`, where `n` is the number of ranges in `other` and `m` is the number
    /// of ranges in `self`.
    ///
    /// For large `n`, consider using the `|` operator, which performs a sorted merge and runs in `O(n + m)` time.
    ///
    /// **See:** [Summary of Union and Extend-like Methods](#rangemapblaze-union--and-extend-like-methods).
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
    /// assert_eq!(a.len(), 5u64);
    /// assert_eq!(b.len(), 0u64);
    ///
    /// assert_eq!(a[1], "a");
    /// assert_eq!(a[2], "a");
    /// assert_eq!(a[3], "b");
    /// assert_eq!(a[4], "b");
    /// assert_eq!(a[5], "b");
    /// ```
    pub fn append(&mut self, other: &mut Self) {
        let original_other_btree_map = core::mem::take(&mut other.btree_map);
        other.len = <T as Integer>::SafeLen::zero();

        for (start, end_value) in original_other_btree_map {
            self.internal_add(start..=end_value.end, end_value.value);
        }
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
            .is_some_and(|(_, end_value)| key <= end_value.end)
    }

    // LATER: might be able to shorten code by combining cases
    fn delete_extra(&mut self, internal_range: &RangeInclusive<T>) {
        let (start, end) = internal_range.clone().into_inner();
        let mut after = self.btree_map.range_mut(start..);
        let (start_after, end_value_after) = after
            .next()
            .expect("Real Assert: There will always be a next");
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
                .expect("Real Assert: There will always be a last");
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

    // LATER: Think about an entry API with or_insert and or_insert_with

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
    /// Panics if start (inclusive) is greater than end (inclusive).
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
    #[allow(clippy::manual_assert)] // We use "if...panic!" for coverage auditing.
    pub fn range<R>(&self, range: R) -> IntoIterMap<T, V>
    where
        R: RangeBounds<T>,
    {
        // LATER 'range' could be made more efficient (it currently creates a RangeMapBlaze for no good reason)
        let (start, end) = extract_range(range);
        assert!(
            start <= end,
            "start (inclusive) must be less than or equal to end (inclusive)"
        );

        let bounds = CheckSortedDisjoint::new([start..=end]);
        let range_map_blaze = self
            .range_values()
            .map_and_set_intersection(bounds)
            .into_range_map_blaze();
        range_map_blaze.into_iter()
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
    /// assert_eq!(map.len(), 5u64);
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
    #[allow(clippy::missing_panics_doc)]
    pub fn remove(&mut self, key: T) -> Option<V> {
        // The code can have only one mutable reference to self.btree_map.

        // Find that range that might contain the key
        let (start_ref, end_value_mut) = self.btree_map.range_mut(..=key).next_back()?;
        let end = end_value_mut.end;

        // If the key is not in the range, we're done
        if end < key {
            return None;
        }
        let start = *start_ref;
        debug_assert!(start <= key, "Real Assert: start <= key");

        // It's in the range.
        self.len -= <T::SafeLen>::one();

        let value = if start == key {
            self.btree_map
                .remove(&start)
                .expect("Real Assert: There will always be a start")
                .value
        } else {
            debug_assert!(start < key, "Real Assert: start < key");
            // This range will now end at key-1.
            end_value_mut.end = key.sub_one();
            end_value_mut.value.clone()
        };

        // If needed, add a new range after key
        if key < end {
            self.btree_map.insert(
                key.add_one(),
                EndValue {
                    end,
                    value: value.clone(),
                },
            );
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

    fn btree_map_len(btree_map: &BTreeMap<T, EndValue<T, V>>) -> T::SafeLen {
        btree_map.iter().fold(
            <T as Integer>::SafeLen::zero(),
            |acc, (start, end_value)| acc + T::safe_len(&(*start..=end_value.end)),
        )
    }

    #[inline]
    fn has_gap(end_before: T, start: T) -> bool {
        end_before
            .checked_add_one()
            .is_some_and(|end_before_succ| end_before_succ < start)
    }

    #[cfg(never)]
    // TODO: Look at other TODOs before enabling this.
    // #![cfg_attr(feature = "cursor", feature(btree_cursors, new_range_api))]
    // #[cfg(feature = "cursor")]
    //  use core::{cmp::min, range::Bound};
    #[inline]
    fn adjust_touching_for_insert(
        &mut self,
        stored_start: T,
        stored_end_value: EndValue<T, V>,
        range: &mut RangeInclusive<T>,
        value: &V,
    ) {
        let stored_value = &stored_end_value.value;
        let stored_end = stored_end_value.end;

        // ── 1. Same value → coalesce completely ──────────────────────────────
        if stored_value == value {
            let new_start = min(*range.start(), stored_start);
            let new_end = max(*range.end(), stored_end);
            *range = new_start..=new_end;

            self.len -= T::safe_len(&(stored_start..=stored_end));
            self.btree_map.remove(&stored_start);
            return;
        }

        // ── 2. Different value → may need to split ───────────────────────────
        let overlaps = stored_start <= *range.end() && stored_end >= *range.start();

        if overlaps {
            // Remove the overlapping range first.
            self.len -= T::safe_len(&(stored_start..=stored_end));
            self.btree_map.remove(&stored_start);

            // Left residual slice
            if stored_start < *range.start() {
                let left_end = range.start().sub_one(); // TODO are we sure this won't underflow?
                self.len += T::safe_len(&(stored_start..=left_end));
                self.btree_map.insert(
                    stored_start,
                    EndValue {
                        end: left_end,
                        value: stored_value.clone(),
                    },
                );
            }

            // Right residual slice
            if stored_end > *range.end() {
                let right_start = range.end().add_one();
                self.len += T::safe_len(&(right_start..=stored_end));
                self.btree_map.insert(
                    right_start,
                    EndValue {
                        end: stored_end,
                        value: stored_end_value.value, // already owned
                    },
                );
            }
        }
        // Otherwise: no overlap → keep ranges as they are.
    }

    #[cfg(never)]
    // For benchmarking, based on https://github.com/jeffparsons/rangemap's `insert` method.
    pub(crate) fn internal_add(&mut self, mut range: RangeInclusive<T>, value: V) {
        use core::ops::Bound::{Included, Unbounded}; // TODO: Move to the top

        let start = *range.start();
        let end = *range.end();

        // === case: empty
        if end < start {
            return;
        }

        // Walk *backwards* from the first stored range whose start ≤ `start`.
        //      Take the nearest two so we can look at “before” and “before-before”.
        let mut candidates = self
            .btree_map
            .range::<T, _>((Unbounded, Included(&start))) // ..= start
            .rev()
            .take(2)
            .filter(|(_stored_start, stored_end_value)| {
                // TODO use saturation arithmetic to avoid underflow
                let end = stored_end_value.end;
                end >= start || (start != T::min_value() && end >= start.sub_one())
            });

        if let Some(mut candidate) = candidates.next() {
            // Or the one before it if both cases described above exist.
            if let Some(another_candidate) = candidates.next() {
                candidate = another_candidate;
            }

            let stored_start: T = *candidate.0;
            let stored_end_value: EndValue<T, V> = candidate.1.clone();
            self.adjust_touching_for_insert(
                stored_start,
                stored_end_value,
                &mut range, // `end` is the current (possibly growing) tail
                &value,
            );
        }

        // let range = &mut range; // &mut RangeInclusive<T>

        loop {
            // first range whose start ≥ new_range.start()
            let next_entry = self
                .btree_map
                .range::<T, _>((Included(range.start()), Unbounded))
                .next();

            let Some((&stored_start, stored_end_value)) = next_entry else {
                break; // nothing more
            };

            let second_last_possible_start = *range.end();
            let maybe_latest_start = if second_last_possible_start == T::max_value() {
                None
            } else {
                Some(second_last_possible_start.add_one())
            };

            if maybe_latest_start.map_or(false, |latest| stored_start > latest) {
                break; // beyond end + 1
            }
            if let Some(latest) = maybe_latest_start {
                if stored_start == latest && stored_end_value.value != value {
                    break; // touches but diff value
                }
            }

            // clone so we can mutate the map in the helper
            let end_value_clone = stored_end_value.clone();

            self.adjust_touching_for_insert(stored_start, end_value_clone, &mut range, &value);

            // loop again; `new_range` might have grown on the right
        }

        let start_key = *range.start();
        let end_key = *range.end();
        // self.len += T::safe_len(&(start_key..=end_key));
        self.btree_map.insert(
            start_key,
            EndValue {
                end: end_key,
                value,
            },
        );

        debug_assert!(self.len == self.len_slow());
    }

    #[cfg(never)]
    // #[cfg(feature = "cursor")]
    pub(crate) fn internal_add(&mut self, mut range: RangeInclusive<T>, value: V) {
        // Based on https://github.com/jeffparsons/rangemap's `insert` method but with cursor's added
        use core::ops::Bound::{Included, Unbounded};
        use std::collections::btree_map::CursorMut;

        let start = *range.start();
        let end = *range.end();

        // === case: empty
        if end < start {
            return;
        }

        // Walk *backwards* from the first stored range whose start ≤ `start`.
        //      Take the nearest two so we can look at “before” and “before-before”.
        let mut candidates = self
            .btree_map
            .range::<T, _>((Unbounded, Included(&start))) // ..= start
            .rev()
            .take(2)
            .filter(|(_stored_start, stored_end_value)| {
                let end = stored_end_value.end;
                end >= start || (start != T::min_value() && end >= start.sub_one())
            });

        if let Some(mut candidate) = candidates.next() {
            // Or the one before it if both cases described above exist.
            if let Some(another_candidate) = candidates.next() {
                candidate = another_candidate;
            }

            let stored_start: T = *candidate.0;
            let stored_end_value: EndValue<T, V> = candidate.1.clone();
            self.adjust_touching_for_insert(
                stored_start,
                stored_end_value,
                &mut range, // `end` is the current (possibly growing) tail
                &value,
            );
        }
        // keep looping until we hit the stop window
        loop {
            // create a fresh cursor _for this iteration only_
            let mut cur: CursorMut<'_, T, EndValue<T, V>> = self
                .btree_map
                .lower_bound_mut(Bound::Included(range.start()));

            let Some(peeked) = cur.peek_next() else {
                break;
            };
            let (&stored_start, maybe_end_value) = peeked;

            let last_ok = *range.end();
            if last_ok == T::max_value() {
                if stored_start > last_ok {
                    break;
                }
            } else {
                let latest = last_ok.add_one();
                if stored_start > latest {
                    break;
                }
                if stored_start == latest && maybe_end_value.value != value {
                    break;
                }
            }

            // now clone and remove
            let cloned = maybe_end_value.clone();
            cur.remove_next();

            // now we can call any &mut-self helper safely
            self.adjust_touching_for_insert(stored_start, cloned, &mut range, &value);
        }

        let start_key = *range.start();
        let end_key = *range.end();

        self.btree_map.insert(
            start_key,
            EndValue {
                end: end_key,
                value, // moves `value`
            },
        );
        self.len += T::safe_len(&(start_key..=end_key));

        debug_assert!(self.len == self.len_slow());
    }

    // https://stackoverflow.com/questions/49599833/how-to-find-next-smaller-key-in-btreemap-btreeset
    // https://stackoverflow.com/questions/35663342/how-to-modify-partially-remove-a-range-from-a-btreemap
    // LATER might be able to shorten code by combining cases
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
        }

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
            if start <= end_before {
                self.len -= T::safe_len(&(start..=end_before));
                debug_assert!(start_before <= start.sub_one()); // real assert
                end_value_before.end = start.sub_one(); // safe because !same_start
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

    #[inline]
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
    /// assert_eq!(a.len(), 0u64);
    /// a.insert(1, "a");
    /// assert_eq!(a.len(), 1u64);
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
    #[inline]
    #[must_use]
    pub fn new() -> Self {
        Self {
            btree_map: BTreeMap::new(),
            len: <T as Integer>::SafeLen::zero(),
        }
    }

    /// Extends the [`RangeMapBlaze`] with an iterator of `(range, value)` pairs without pre-merging.
    ///
    /// Unlike [`RangeMapBlaze::extend`], this method does **not** merge adjacent or overlapping ranges
    /// before inserting. Each `(range, value)` pair is added as-is, making it faster when the input
    /// is already well-structured or disjoint.
    ///
    /// This method has *right-to-left precedence*: later ranges in the iterator overwrite earlier ones.
    ///
    /// **See:** [Summary of Union and Extend-like Methods](#rangemapblaze-union--and-extend-like-methods).
    ///
    /// # Examples
    /// ```
    /// use range_set_blaze::RangeMapBlaze;
    /// let mut a = RangeMapBlaze::from_iter([(1..=4, "a")]);
    /// a.extend_simple([(3..=5, "b"), (5..=5, "c")]);
    /// assert_eq!(a, RangeMapBlaze::from_iter([(1..=2, "a"), (3..=4, "b"), (5..=5, "c")]));
    ///
    /// let mut a = RangeMapBlaze::from_iter([(1..=4, "a")]);
    /// a.extend([(3..=5, "b"), (5..=5, "c")]);
    /// assert_eq!(a, RangeMapBlaze::from_iter([(1..=2, "a"), (3..=4, "b"), (5..=5, "c")]));
    ///
    /// let mut a = RangeMapBlaze::from_iter([(1..=4, "a")]);
    /// let mut b = RangeMapBlaze::from_iter([(3..=5, "b"), (5..=5, "c")]);
    /// a |= b;
    /// assert_eq!(a, RangeMapBlaze::from_iter([(1..=4, "a"), (5..=5, "b")]));
    /// ```
    pub fn extend_simple<I>(&mut self, iter: I)
    where
        I: IntoIterator<Item = (RangeInclusive<T>, V)>,
    {
        let iter = iter.into_iter();

        for (range, value) in iter {
            self.internal_add(range, value);
        }
    }

    /// Extends the [`RangeMapBlaze`] with the contents of an owned [`RangeMapBlaze`].
    ///
    /// This method has *right-to-left precedence* — like `BTreeMap`, but unlike most
    /// other `RangeMapBlaze` methods. If the maps contain overlapping ranges,
    /// values from `other` will overwrite those in `self`.
    ///
    /// Compared to [`RangeMapBlaze::extend_with`], this method can be more efficient because it can
    /// consume the internal data structures of `other` directly, avoiding some cloning.
    ///
    /// **See:** [Summary of Union and Extend-like Methods](#rangemapblaze-union--and-extend-like-methods).
    ///
    /// # Examples
    ///
    /// ```
    /// use range_set_blaze::RangeMapBlaze;
    /// let mut a = RangeMapBlaze::from_iter([(1..=4, "a")]);
    /// let mut b = RangeMapBlaze::from_iter([(3..=4, "b"), (5..=5, "c")]);
    /// a.extend_from(b);
    /// assert_eq!(a, RangeMapBlaze::from_iter([(1..=2, "a"), (3..=4, "b"), (5..=5, "c")]));
    /// ```
    #[inline]
    pub fn extend_from(&mut self, other: Self) {
        for (start, end_value) in other.btree_map {
            let range = start..=end_value.end;
            self.internal_add(range, end_value.value);
        }
    }

    /// Extends the [`RangeMapBlaze`] with the contents of a borrowed [`RangeMapBlaze`].
    ///
    /// This method has *right-to-left precedence* — like `BTreeMap`, but unlike most
    /// other `RangeMapBlaze` methods. If the maps contain overlapping ranges,
    /// values from `other` will overwrite those in `self`.
    ///
    /// This method is simple and predictable but not the most efficient option when
    /// the right-hand side is larger. For better performance when ownership is available,
    /// consider using [`RangeMapBlaze::extend_from`] or the `|=` operator.
    ///
    /// **See:** [Summary of Union and Extend-like Methods](#rangemapblaze-union--and-extend-like-methods).
    ///
    /// # Examples
    ///
    /// ```
    /// use range_set_blaze::RangeMapBlaze;
    /// let mut a = RangeMapBlaze::from_iter([(1..=4, "a")]);
    /// let mut b = RangeMapBlaze::from_iter([(3..=4, "b"), (5..=5, "c")]);
    /// a.extend_from(b);
    /// assert_eq!(a, RangeMapBlaze::from_iter([(1..=2, "a"), (3..=4, "b"), (5..=5, "c")]));
    /// ```
    #[inline]
    pub fn extend_with(&mut self, other: &Self) {
        for (start, end_value) in &other.btree_map {
            let range = *start..=end_value.end;
            self.internal_add(range, end_value.value.clone());
        }
    }

    /// Removes the first element from the set and returns it, if any.
    /// The first element is always the minimum element in the set.
    ///
    /// Often, internally, the value must be cloned.
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
    /// Often, internally, the value must be cloned.
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

    /// An iterator that visits the ranges and values in the [`RangeMapBlaze`],
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
        RangeValuesIter::new(&self.btree_map)
    }

    /// An iterator that visits the ranges and values in the [`RangeMapBlaze`]. Double-ended.
    ///
    /// Also see [`RangeMapBlaze::iter`] and [`RangeMapBlaze::range_values`].
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
    /// # extern crate alloc;
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
        IntoRangeValuesIter::new(self.btree_map)
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
    pub fn ranges(&self) -> MapRangesIter<'_, T, V> {
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

    /// Returns the number of sorted & disjoint ranges in the set.
    ///
    /// # Example
    ///
    /// ```
    /// use range_set_blaze::RangeMapBlaze;
    ///
    /// // We put in three ranges, but they are not sorted & disjoint.
    /// let map = RangeMapBlaze::from_iter([(10..=20,"a"), (15..=25,"a"), (30..=40,"b")]);
    /// // After RangeMapBlaze sorts & 'disjoint's them, we see two ranges.
    /// assert_eq!(map.ranges_len(), 2);
    /// assert_eq!(map.to_string(), r#"(10..=25, "a"), (30..=40, "b")"#);
    /// ```
    #[must_use]
    pub fn ranges_len(&self) -> usize {
        self.btree_map.len()
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

    /// Returns the number of sorted & disjoint ranges i
    /// n the set.
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

    /// Retains only the elements specified by the predicate.
    ///
    /// In other words, remove all pairs `(k, v)` for which `f(&k, &mut v)` returns `false`.
    /// The elements are visited in ascending key order.
    ///
    /// Because if visits every element in every range, it is expensive compared to
    /// [`RangeMapBlaze::ranges_retain`].
    ///
    /// # Examples
    ///
    /// ```
    /// use range_set_blaze::RangeMapBlaze;
    ///
    /// let mut map: RangeMapBlaze<i32, i32> = (0..8).map(|x| (x, x*10)).collect();
    /// // Keep only the elements with even-numbered keys.
    /// map.retain(|&k, _| k % 2 == 0);
    /// assert!(map.into_iter().eq(vec![(0, 0), (2, 20), (4, 40), (6, 60)]));
    /// ```
    pub fn retain<F>(&mut self, f: F)
    where
        F: Fn(&T, &V) -> bool,
    {
        *self = self.iter().filter(|(k, v)| f(k, v)).collect();
    }

    /// Retains only the `(range, value)` pairs specified by the predicate.
    ///
    /// In other words, removes all `(range, value)` pairs for which `f(&range, &value)`
    /// returns `false`. The `(range, value)` pairs are visited in ascending range order.
    ///
    /// # Examples
    ///
    /// ```
    /// use range_set_blaze::RangeMapBlaze;
    ///
    /// let mut map: RangeMapBlaze<i32, &str> = RangeMapBlaze::from_iter([(0..=3, "low"), (4..=7, "high")]);
    /// // Keep only the ranges with a specific value.
    /// map.ranges_retain(|range, &value| value == "low");
    /// assert_eq!(map, RangeMapBlaze::from_iter([(0..=3, "low")]));
    /// ```
    pub fn ranges_retain<F>(&mut self, mut f: F)
    where
        F: FnMut(&RangeInclusive<T>, &V) -> bool,
    {
        self.btree_map.retain(|start, end_value| {
            let range = *start..=end_value.end;
            if f(&range, &end_value.value) {
                true
            } else {
                self.len -= T::safe_len(&range);
                false
            }
        });
    }
}

impl<T, V> IntoIterator for RangeMapBlaze<T, V>
where
    T: Integer,
    V: Eq + Clone,
{
    type Item = (T, V);
    type IntoIter = IntoIterMap<T, V>;

    /// Gets an iterator for moving out the [`RangeSetBlaze`]'s integer contents.
    /// Double-ended.
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

// Implementing `IntoIterator` for `&RangeMapBlaze<T, V>` because BTreeMap does.
impl<'a, T: Integer, V: Eq + Clone> IntoIterator for &'a RangeMapBlaze<T, V> {
    type IntoIter = IterMap<T, &'a V, RangeValuesIter<'a, T, V>>;
    type Item = (T, &'a V);

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<T: Integer, V: Eq + Clone> BitOr<Self> for RangeMapBlaze<T, V> {
    /// Unions the contents of two [`RangeMapBlaze`]'s.
    ///    /// This operator has *right precedence*: when overlapping ranges are present,
    /// values on the right-hand side take priority over those self.
    ///
    /// This method is optimized for three usage scenarios:
    /// when the left-hand side is much smaller, when the right-hand side is much smaller,
    /// and when both sides are of similar size.
    ///
    /// **Also See:** [Summary of Union and Extend-like Methods](#rangemapblaze-union--and-extend-like-methods).
    ///
    /// # Examples
    /// ```
    /// use range_set_blaze::RangeMapBlaze;
    /// let a = RangeMapBlaze::from_iter([(1..=2, "a"), (5..=100, "a")]);
    /// let b = RangeMapBlaze::from_iter([(2..=6, "b")]);
    /// let union = a | b;  // Alternatively, '&a | &b', etc.
    /// // cmk000
    /// assert_eq!(union, RangeMapBlaze::from_iter([(1..=1, "a"), (2..=6, "b"), (7..=100, "a")]));
    /// ```
    type Output = Self;
    fn bitor(self, other: Self) -> Self {
        let b_len = other.ranges_len();
        if b_len == 0 {
            return self;
        }
        let a_len = self.ranges_len();
        if a_len == 0 {
            return other;
        } // cmk000
        if much_greater_than(a_len, b_len) {
            return small_b_over_a(self, other);
        }
        if much_greater_than(b_len, a_len) {
            return small_a_under_b(self, other);
        }
        // Sizes are comparable, use the iterator union
        (self.into_range_values() | other.into_range_values()).into_range_map_blaze()
    }
}

impl<T: Integer, V: Eq + Clone> BitOr<&Self> for RangeMapBlaze<T, V> {
    /// Unions the contents of two [`RangeMapBlaze`]'s.
    ///
    /// This operator has *right precedence*: when overlapping ranges are present,
    /// values on the right-hand side take priority over those self.
    ///
    /// This method is optimized for three usage scenarios:
    /// when the left-hand side is much smaller, when the right-hand side is much smaller,
    /// and when both sides are of similar size.
    ///
    /// **Also See:** [Summary of Union and Extend-like Methods](#rangemapblaze-union--and-extend-like-methods).
    ///
    /// # Examples
    /// ```
    /// use range_set_blaze::RangeMapBlaze;
    /// let a = RangeMapBlaze::from_iter([(1..=2, "a"), (5..=100, "a")]);
    /// let b = RangeMapBlaze::from_iter([(2..=6, "b")]);    /// let union = a | &b; // Alternatively, 'a | b', etc.
    /// // cmk000
    /// assert_eq!(union, RangeMapBlaze::from_iter([(1..=1, "a"), (2..=6, "b"), (7..=100, "a")]));
    /// ```
    type Output = Self;
    fn bitor(self, other: &Self) -> Self {
        let b_len = other.ranges_len();
        if b_len == 0 {
            return self;
        }
        let a_len = self.ranges_len();
        if a_len == 0 {
            return other.clone();
        }
        // cmk000
        if much_greater_than(a_len, b_len) {
            return small_b_over_a(self, other.clone());
        }
        if much_greater_than(b_len, a_len) {
            return small_a_under_b(self, other.clone());
        }

        // Sizes are comparable, use the iterator union
        (self.range_values() | other.range_values()).into_range_map_blaze()
    }
}

impl<T: Integer, V: Eq + Clone> BitOr<RangeMapBlaze<T, V>> for &RangeMapBlaze<T, V> {
    type Output = RangeMapBlaze<T, V>;
    /// Unions the contents of two [`RangeMapBlaze`]'s.
    ///
    /// This operator has *right precedence*: when overlapping ranges are present,
    /// values on the right-hand side take priority over those self.
    ///
    /// This method is optimized for three usage scenarios:
    /// when the left-hand side is much smaller, when the right-hand side is much smaller,
    /// and when both sides are of similar size.
    ///
    /// **Also See:** [Summary of Union and Extend-like Methods](#rangemapblaze-union--and-extend-like-methods).
    ///
    /// # Examples
    /// ```
    /// use range_set_blaze::RangeMapBlaze;
    /// let a = RangeMapBlaze::from_iter([(1..=2, "a"), (5..=100, "a")]);
    /// let b = RangeMapBlaze::from_iter([(2..=6, "b")]);
    /// let union = &a | b;  // Alternatively, 'a | b', etc.
    /// // cmk000
    /// assert_eq!(union, RangeMapBlaze::from_iter([(1..=1, "a"), (2..=6, "b"), (7..=100, "a")]));
    /// ```
    fn bitor(self, other: RangeMapBlaze<T, V>) -> RangeMapBlaze<T, V> {
        let a_len = self.ranges_len();
        if a_len == 0 {
            return other;
        }
        let b_len = other.ranges_len();
        if b_len == 0 {
            return self.clone();
        } // cmk000
        if much_greater_than(b_len, a_len) {
            return small_a_under_b(self.clone(), other);
        }
        if much_greater_than(a_len, b_len) {
            return small_b_over_a(self.clone(), other);
        }
        // Sizes are comparable, use the iterator union
        (self.range_values() | other.range_values()).into_range_map_blaze()
    }
}

impl<T: Integer, V: Eq + Clone> BitOr<&RangeMapBlaze<T, V>> for &RangeMapBlaze<T, V> {
    type Output = RangeMapBlaze<T, V>;
    /// Unions the contents of two [`RangeMapBlaze`]'s.
    ///
    /// This operator has *right precedence*: when overlapping ranges are present,
    /// values on the right-hand side take priority over those self.
    ///
    /// This method is optimized for three usage scenarios:
    /// when the left-hand side is much smaller, when the right-hand side is much smaller,
    /// and when both sides are of similar size.
    ///
    /// **Also See:** [Summary of Union and Extend-like Methods](#rangemapblaze-union--and-extend-like-methods).
    ///
    /// # Examples
    /// ```
    /// use range_set_blaze::RangeMapBlaze;
    /// let a = RangeMapBlaze::from_iter([(1..=2, "a"), (5..=100, "a")]);
    /// let b = RangeMapBlaze::from_iter([(2..=6, "b")]);
    /// let union = &a | &b; // Alternatively, 'a | b', etc.
    /// // cmk000
    /// assert_eq!(union, RangeMapBlaze::from_iter([(1..=1, "a"), (2..=6, "b"), (7..=100, "a")]));
    /// ```
    fn bitor(self, other: &RangeMapBlaze<T, V>) -> RangeMapBlaze<T, V> {
        let a_len = self.ranges_len();
        if a_len == 0 {
            return other.clone();
        }
        let b_len = other.ranges_len();
        if b_len == 0 {
            return self.clone();
        } // cmk000
        if much_greater_than(a_len, b_len) {
            return small_b_over_a(self.clone(), other.clone());
        }
        if much_greater_than(b_len, a_len) {
            return small_a_under_b(self.clone(), other.clone());
        }
        // Sizes are comparable, use the iterator union
        (self.range_values() | other.range_values()).into_range_map_blaze()
    }
}

map_op!(
     BitAnd bitand,
    RangeMapBlaze<T, V>,          // RHS concrete type

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
    "placeholder",

    // owned ∩ owned
    |a, b| {
        b.into_range_values()
         .map_and_set_intersection(a.into_ranges())
         .into_range_map_blaze()
    },

    // owned ∩ &borrowed
    |a, &b| {
        b.range_values()
         .map_and_set_intersection(a.into_ranges())
         .into_range_map_blaze()
    },

    // &borrowed ∩ owned
    |&a, b| {
        b.into_range_values()
         .map_and_set_intersection(a.ranges())
         .into_range_map_blaze()
    },

    // &borrowed ∩ &borrowed
    |&a, &b| {
        b.range_values()
         .map_and_set_intersection(a.ranges())
         .into_range_map_blaze()
    },
);

map_op!(
     BitAnd bitand,
    RangeSetBlaze<T>,          // RHS concrete type

/// Find the intersection between a [`RangeMapBlaze`] and a [`RangeSetBlaze`]. The result is a new [`RangeMapBlaze`].
///
/// Either, neither, or both inputs may be borrowed.
///
/// # Examples
/// ```
/// use range_set_blaze::prelude::*;
///
/// let a = RangeMapBlaze::from_iter([(1..=100, "a")]);
/// let b = RangeSetBlaze::from_iter([2..=6]);
/// let result = &a & &b; // Alternatively, 'a & b'.
/// assert_eq!(result.to_string(), r#"(2..=6, "a")"#);
/// ```
    "placeholder",

    // owned ∩ owned
    |a, b| {
        a.into_range_values()
         .map_and_set_intersection(b.into_ranges())
         .into_range_map_blaze()
    },

    // owned ∩ &borrowed
    |a, &b| {
        a.into_range_values()
         .map_and_set_intersection(b.ranges())
         .into_range_map_blaze()
    },

    // &borrowed ∩ owned
    |&a, b| {
        a.range_values()
         .map_and_set_intersection(b.into_ranges())
         .into_range_map_blaze()
    },

    // &borrowed ∩ &borrowed
    |&a, &b| {
        a.range_values()
         .map_and_set_intersection(b.ranges())
         .into_range_map_blaze()
    },
);

map_op!(
    BitXor bitxor,                           // trait + method name
    RangeMapBlaze<T, V>,                     // RHS concrete type

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
    "placeholder",

    // ── owned ^ owned ────────────────────────────────────────────
    |a, b| {
        SymDiffIterMap::new2(
            a.into_range_values(),
            b.into_range_values(),
        )
        .into_range_map_blaze()
    },

    // ── owned ^ &borrowed ────────────────────────────────────────
    |a, &b| {
        SymDiffIterMap::new2(
            a.range_values(),
            b.range_values(),
        )
        .into_range_map_blaze()
    },

    // ── &borrowed ^ owned ────────────────────────────────────────
    |&a, b| {
        SymDiffIterMap::new2(
            a.range_values(),
            b.range_values(),
        )
        .into_range_map_blaze()
    },

    // ── &borrowed ^ &borrowed ────────────────────────────────────
    |&a, &b| {
        SymDiffIterMap::new2(
            a.range_values(),
            b.range_values(),
        )
        .into_range_map_blaze()
    },
);

map_op!(
    Sub sub,                                 // trait + method name
    RangeMapBlaze<T, V>,         // RHS concrete type

    /// **Difference** of two [`RangeMapBlaze`] values (`a - b`).
    ///
    /// Either, neither, or both inputs may be borrowed.
    ///
    /// # Example
    /// ```
    /// use range_set_blaze::prelude::*;
    ///
    /// let a = RangeMapBlaze::from_iter([(1..=2, "a"), (5..=100, "a")]);
    /// let b = RangeMapBlaze::from_iter([(2..=6, "b")]);
    /// let result = &a - &b;                // or `a - b`
    /// assert_eq!(result.to_string(),
    ///            r#"(1..=1, "a"), (7..=100, "a")"#);
    /// ```
    "placeholder",

    // ── owned − owned ────────────────────────────────────────────
    |a, b| {
        a.into_range_values()
         .map_and_set_difference(b.ranges())
         .into_range_map_blaze()
    },

    // ── owned − &borrowed ────────────────────────────────────────
    |a, &b| {
        a.into_range_values()
         .map_and_set_difference(b.ranges())
         .into_range_map_blaze()
    },

    // ── &borrowed − owned ────────────────────────────────────────
    |&a, b| {
        a.range_values()
         .map_and_set_difference(b.into_ranges())
         .into_range_map_blaze()
    },

    // ── &borrowed − &borrowed ────────────────────────────────────
    |&a, &b| {
        a.range_values()
         .map_and_set_difference(b.ranges())
         .into_range_map_blaze()
    },
);

map_op!(
    Sub sub,                                 // trait + method name
    RangeSetBlaze<T>,         // RHS concrete type

/// Find the difference between a [`RangeMapBlaze`] and a [`RangeSetBlaze`]. The result is a new [`RangeMapBlaze`].
///
/// Either, neither, or both inputs may be borrowed.
///
/// # Examples
/// ```
/// use range_set_blaze::prelude::*;
///
/// let a = RangeMapBlaze::from_iter([(1..=100, "a")]);
/// let b = RangeSetBlaze::from_iter([2..=6]);
/// let result = &a - &b; // Alternatively, 'a - b'.
/// assert_eq!(result.to_string(), r#"(1..=1, "a"), (7..=100, "a")"#);
/// ```
    "placeholder",

    // ── owned − owned ────────────────────────────────────────────
    |a, b| {
        a.into_range_values()
         .map_and_set_difference(b.ranges())
         .into_range_map_blaze()
    },

    // ── owned − &borrowed ────────────────────────────────────────
    |a, &b| {
        a.into_range_values()
         .map_and_set_difference(b.ranges())
         .into_range_map_blaze()
    },

    // ── &borrowed − owned ────────────────────────────────────────
    |&a, b| {
        a.range_values()
         .map_and_set_difference(b.into_ranges())
         .into_range_map_blaze()
    },

    // ── &borrowed − &borrowed ────────────────────────────────────
    |&a, &b| {
        a.range_values()
         .map_and_set_difference(b.ranges())
         .into_range_map_blaze()
    },
);

map_unary_op!(
    Not not,                                   // trait + method
    crate::set::RangeSetBlaze<T>,              // output type

    /// Takes the complement of a [`RangeMapBlaze`].
    ///
    /// Produces a [`RangeSetBlaze`] containing all integers *not* present
    /// in the map’s key ranges.
    ///
    /// # Example
    /// ```
    /// use range_set_blaze::prelude::*;
    /// let map =
    ///     RangeMapBlaze::from_iter([(10u8..=20, "a"), (15..=25, "b"),
    ///                               (30..=40, "c")]);
    /// let complement = !&map;                // or `!map`
    /// assert_eq!(complement.to_string(), "0..=9, 26..=29, 41..=255");
    /// ```
    "placeholder",

    // body for &map
    |&m| {
        m.ranges()
         .complement()
         .into_range_set_blaze()
    }
);

impl<T, V> Extend<(T, V)> for RangeMapBlaze<T, V>
where
    T: Integer,
    V: Eq + Clone,
{
    /// Extends the [`RangeMapBlaze`] with the contents of an iterator of integer-value pairs.
    ///
    /// This method has *right-to-left precedence*: later values in the iterator take priority
    /// over earlier ones, matching the behavior of standard `BTreeMap::extend`.
    ///
    /// Each integer is treated as a singleton range. Adjacent integers with the same value
    /// are merged before insertion. For alternatives that skip merging or accept full ranges,
    /// see [`RangeMapBlaze::extend_simple`] and [`RangeMapBlaze::extend`].
    ///
    /// // cmk000
    /// **See:** [Summary of Union and Extend-like Methods](#rangemapblaze-union--and-extend-like-methods).
    ///
    /// # Examples
    /// ```
    /// use range_set_blaze::RangeMapBlaze;
    /// let mut a = RangeMapBlaze::from_iter([(3, "a"), (4, "e"), (5, "f"), (5, "g")]);
    /// a.extend([(1..=4, "b")]);
    /// assert_eq!(a, RangeMapBlaze::from_iter([(1..=4, "b"), (5..=5, "f")]));
    ///
    /// let mut a = RangeMapBlaze::from_iter([(3, "a"), (4, "e"), (5, "f"), (5, "g")]);
    /// let mut b = RangeMapBlaze::from_iter([(1..=4, "b")]);
    /// a |= b;
    /// // cmk000
    /// assert_eq!(a, RangeMapBlaze::from_iter([(1..=4, "b"), (5..=5, "f")]));
    /// ```
    #[inline]
    fn extend<I>(&mut self, iter: I)
    where
        I: IntoIterator<Item = (T, V)>,
    {
        let iter = iter.into_iter();

        // We gather adjacent values into ranges via UnsortedPriorityMap, but ignore the priority.
        for priority in UnsortedPriorityMap::new(iter.map(|(r, v)| (r..=r, Rc::new(v)))) {
            let (range, value) = priority.into_range_value();
            let value: V = Rc::try_unwrap(value).unwrap_or_else(|_| unreachable!());
            self.internal_add(range, value);
        }
    }
}

impl<T, V> Extend<(RangeInclusive<T>, V)> for RangeMapBlaze<T, V>
where
    T: Integer,
    V: Eq + Clone,
{
    /// Extends the [`RangeMapBlaze`] with the contents of an iterator of range-value pairs.
    ///
    /// This method has *right-to-left precedence* — like `BTreeMap`, but unlike most other
    /// `RangeMapBlaze` methods.
    ///
    /// It first merges any adjacent or overlapping ranges with the same value, then adds them one by one.
    /// For alternatives that skip merging or that accept integer-value pairs, see
    /// [`RangeMapBlaze::extend_simple`] and the `(integer, value)` overload.
    ///
    /// // cmk000
    /// For *left-to-right* precedence, use the union-related methods.
    ///
    /// **See:** [Summary of Union and Extend-like Methods](#rangemapblaze-union--and-extend-like-methods).
    /// # Examples
    /// ```
    /// use range_set_blaze::RangeMapBlaze;
    /// let mut a = RangeMapBlaze::from_iter([(1..=4, "a")]);
    /// a.extend([(3..=5, "b"), (5..=5, "c")]);
    /// assert_eq!(a, RangeMapBlaze::from_iter([(1..=2, "a"), (3..=4, "b"), (5..=5, "c")]));
    ///
    /// // `extend_simple` is a more efficient for the case where the ranges a likely disjoint.
    /// let mut a = RangeMapBlaze::from_iter([(1..=4, "a")]);
    /// a.extend_simple([(3..=5, "b"), (5..=5, "c")]);
    /// assert_eq!(a, RangeMapBlaze::from_iter([(1..=2, "a"), (3..=4, "b"), (5..=5, "c")]));
    ///
    /// let mut a = RangeMapBlaze::from_iter([(1..=4, "a")]);
    /// let mut b = RangeMapBlaze::from_iter([(3..=5, "b"), (5..=5, "c")]);
    /// a |= b;
    /// // cmk000
    /// assert_eq!(a, RangeMapBlaze::from_iter([(1..=4, "a"), (5..=5, "b")]));
    /// ```
    #[inline]
    fn extend<I>(&mut self, iter: I)
    where
        I: IntoIterator<Item = (RangeInclusive<T>, V)>,
    {
        let iter = iter.into_iter();

        // We gather adjacent values into ranges via UnsortedPriorityMap, but ignore the priority.
        for priority in UnsortedPriorityMap::new(iter.map(|(r, v)| (r, Rc::new(v)))) {
            let (range, value) = priority.into_range_value();
            let value = Rc::try_unwrap(value).unwrap_or_else(|_| unreachable!());
            self.internal_add(range, value);
        }
    }
}

impl<T, V, const N: usize> From<[(T, V); N]> for RangeMapBlaze<T, V>
where
    T: Integer,
    V: Eq + Clone,
{
    /// For compatibility with [`BTreeMap`] you may create a [`RangeSetBlaze`] from an array of integers.
    ///
    /// *For more about constructors and performance, see [`RangeSetBlaze` Constructors](struct.RangeSetBlaze.html#rangesetblaze-constructors).*
    ///
    /// [`BTreeMap`]: alloc::collections::BTreeMap
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
impl<T: Integer, V: Eq + Clone> Index<T> for RangeMapBlaze<T, V> {
    type Output = V;

    /// Returns a reference to the value corresponding to the supplied key.
    ///
    /// # Panics
    ///
    /// Panics if the key is not present in the `BTreeMap`.
    #[inline]
    #[allow(clippy::manual_assert)] // We use "if...panic!" for coverage auditing.
    fn index(&self, index: T) -> &Self::Output {
        self.get(index).unwrap_or_else(|| {
            panic!("no entry found for key");
        })
    }
}

// LATER define value_per_range and into_value_per_range

impl<T, V> PartialOrd for RangeMapBlaze<T, V>
where
    T: Integer,
    V: Eq + Clone + Ord,
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
    V: Eq + Clone + Ord,
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

impl<T: Integer, V: Eq + Clone> Eq for RangeMapBlaze<T, V> {}

impl<T: Integer, V: Eq + Clone> BitOrAssign<&Self> for RangeMapBlaze<T, V> {
    /// Adds the contents of another [`RangeMapBlaze`] to this one.
    ///
    /// This operator has *right precedence*: when overlapping ranges are present,
    /// values on the right-hand side take priority over those self.
    /// //cmk000
    /// To get *right precedence*, swap the operands or use
    /// [`RangeMapBlaze::extend_with`].
    ///
    /// This method is optimized for three usage scenarios:
    /// when the left-hand side is much smaller, when the right-hand side is much smaller,
    /// and when both sides are of similar size
    ///
    /// // cmk000
    /// Even greater efficiency is possible when the right-hand side is passed by value,
    /// allowing its internal data structures to be reused.
    ///
    /// **Also See:** [Summary of Union and Extend-like Methods](#rangemapblaze-union--and-extend-like-methods).
    ///
    /// # Examples
    /// ```
    /// use range_set_blaze::RangeMapBlaze;
    /// let mut a = RangeMapBlaze::from_iter([(3, "a"), (4, "e"), (5, "f"), (5, "g")]);
    /// let mut b = RangeMapBlaze::from_iter([(1..=4, "b")]);
    /// // cmk000
    /// a |= &b;
    /// assert_eq!(a, RangeMapBlaze::from_iter([(1..=4, "b"), (5..=5, "f")]));
    /// ```
    fn bitor_assign(&mut self, other: &Self) {
        let original_self = mem::take(self); // Take ownership of self
        *self = original_self | other; // Use the union operator to combine
    }
}

impl<T: Integer, V: Eq + Clone> BitOrAssign<Self> for RangeMapBlaze<T, V> {
    /// Adds the contents of another [`RangeMapBlaze`] to this one.
    ///
    /// This operator has *right precedence*: when overlapping ranges are present,
    /// values on the right-hand side take priority over those self.
    /// // cmk000
    /// To get *right precedence*, swap the operands or use
    /// [`RangeMapBlaze::extend_with`].
    ///
    /// This method is optimized for three usage scenarios:
    /// when the left-hand side is much smaller, when the right-hand side is much smaller,
    /// and when both sides are of similar size
    ///
    /// **Also See:** [Summary of Union and Extend-like Methods](#rangemapblaze-union--and-extend-like-methods).
    ///
    /// # Examples
    /// ```
    /// use range_set_blaze::RangeMapBlaze;
    /// let mut a = RangeMapBlaze::from_iter([(1..=4, "a")]);
    /// let mut b = RangeMapBlaze::from_iter([(3, "b"), (4, "e"), (5, "f"), (5, "g")]);
    /// a |= &b;
    /// // cmk000
    /// assert_eq!(a, RangeMapBlaze::from_iter([(1..=4, "a"), (5..=5, "f")]));
    /// ```
    ///
    /// # Examples
    /// ```
    /// use range_set_blaze::RangeMapBlaze;
    /// let mut a = RangeMapBlaze::from_iter([(1..=4, "a")]);
    /// let mut b = RangeMapBlaze::from_iter([(3, "b"), (4, "e"), (5, "f"), (5, "g")]);
    /// // cmk000
    /// a |= b;
    /// assert_eq!(a, RangeMapBlaze::from_iter([(1..=4, "a"), (5..=5, "f")]));
    /// ```
    fn bitor_assign(&mut self, other: Self) {
        *self = mem::take(self) | other;
    }
}

#[inline]
fn much_greater_than(a_len: usize, b_len: usize) -> bool {
    let a_len_log2_plus_one: usize = a_len
        .checked_ilog2()
        .map_or(0, |log| log.try_into().expect("log2 fits usize"))
        + 1;
    b_len * a_len_log2_plus_one < STREAM_OVERHEAD * a_len + b_len
}

#[inline]
fn small_b_over_a<T: Integer, V: Eq + Clone>(
    mut a: RangeMapBlaze<T, V>,
    b: RangeMapBlaze<T, V>,
) -> RangeMapBlaze<T, V> {
    debug_assert!(much_greater_than(a.ranges_len(), b.ranges_len()));
    for (start, end_value) in b.btree_map {
        a.internal_add(start..=(end_value.end), end_value.value);
    }
    a
}

#[inline]
fn small_a_under_b<T: Integer, V: Eq + Clone>(
    a: RangeMapBlaze<T, V>,
    mut b: RangeMapBlaze<T, V>,
) -> RangeMapBlaze<T, V> {
    debug_assert!(much_greater_than(b.ranges_len(), a.ranges_len()));
    let difference = a - &b;
    b.extend_simple(
        difference
            .btree_map
            .into_iter()
            .map(|(start, v)| (start..=v.end, v.value)),
    );
    b
}
