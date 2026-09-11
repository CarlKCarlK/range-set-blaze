use crate::DifferenceMap;
use crate::DifferenceMapInternal;
use crate::DynSortedDisjointMap;
use crate::FillGapsIter;
use crate::FillGapsIterMap;
use crate::IntersectionMap;
use crate::IntoRangeValuesIter;
use crate::NotIter;
use crate::NotMap;
use crate::SymDiffMergeMap;
use crate::UnionMergeMap;
use crate::intersection_iter_map::IntersectionIterMap;
use crate::map::ValueCarrier;
use crate::range_values::RangeValuesIter;
use crate::range_values::RangeValuesToRangesIter;
use crate::sorted_disjoint::SortedDisjoint;
use crate::sym_diff_iter_map::SymDiffIterMap;
use crate::{Integer, RangeMapBlaze, union_iter_map::UnionIterMap};
use alloc::format;
use alloc::rc::Rc;
use alloc::string::String;
use alloc::vec::Vec;
use core::{
    cmp::Ordering,
    fmt::Debug,
    iter::{
        Empty, Filter, FlatMap, Flatten, Fuse, FusedIterator, Once, Peekable, Skip, SkipWhile,
        Take, TakeWhile,
    },
    marker::PhantomData,
    ops,
    ops::RangeInclusive,
    option,
};

/// Used internally. Marks iterators that provide `(range, value)` pairs that are sorted by the range's start, but
/// that are not necessarily disjoint.
pub trait SortedStartsMap<T, VC>: Iterator<Item = (RangeInclusive<T>, VC)> + FusedIterator
where
    T: Integer,
    VC: ValueCarrier,
{
}

impl<T, VC, I, P> SortedStartsMap<T, VC> for Filter<I, P>
where
    T: Integer,
    VC: ValueCarrier,
    I: SortedStartsMap<T, VC>,
    P: FnMut(&I::Item) -> bool,
{
}

impl<T, VC, I, P> SortedDisjointMap<T, VC> for Filter<I, P>
where
    T: Integer,
    VC: ValueCarrier,
    I: SortedDisjointMap<T, VC>,
    P: FnMut(&I::Item) -> bool,
{
}

impl<T, VC, I, P> SortedStartsMap<T, VC> for TakeWhile<I, P>
where
    T: Integer,
    VC: ValueCarrier,
    I: SortedStartsMap<T, VC>,
    P: FnMut(&I::Item) -> bool,
{
}

impl<T, VC, I, P> SortedDisjointMap<T, VC> for TakeWhile<I, P>
where
    T: Integer,
    VC: ValueCarrier,
    I: SortedDisjointMap<T, VC>,
    P: FnMut(&I::Item) -> bool,
{
}

impl<T, VC, I, P> SortedStartsMap<T, VC> for SkipWhile<I, P>
where
    T: Integer,
    VC: ValueCarrier,
    I: SortedStartsMap<T, VC>,
    P: FnMut(&I::Item) -> bool,
{
}

impl<T, VC, I, P> SortedDisjointMap<T, VC> for SkipWhile<I, P>
where
    T: Integer,
    VC: ValueCarrier,
    I: SortedDisjointMap<T, VC>,
    P: FnMut(&I::Item) -> bool,
{
}

impl<T, VC, I> SortedStartsMap<T, VC> for Fuse<I>
where
    T: Integer,
    VC: ValueCarrier,
    I: SortedStartsMap<T, VC>,
{
}

impl<T, VC, I> SortedDisjointMap<T, VC> for Fuse<I>
where
    T: Integer,
    VC: ValueCarrier,
    I: SortedDisjointMap<T, VC>,
{
}

impl<T, VC, I> SortedStartsMap<T, VC> for Skip<I>
where
    T: Integer,
    VC: ValueCarrier,
    I: SortedStartsMap<T, VC>,
{
}

impl<T, VC, I> SortedDisjointMap<T, VC> for Skip<I>
where
    T: Integer,
    VC: ValueCarrier,
    I: SortedDisjointMap<T, VC>,
{
}

impl<T, VC, I> SortedStartsMap<T, VC> for Take<I>
where
    T: Integer,
    VC: ValueCarrier,
    I: SortedStartsMap<T, VC>,
{
}

impl<T, VC, I> SortedDisjointMap<T, VC> for Take<I>
where
    T: Integer,
    VC: ValueCarrier,
    I: SortedDisjointMap<T, VC>,
{
}

impl<T, VC, I> SortedStartsMap<T, VC> for Peekable<I>
where
    T: Integer,
    VC: ValueCarrier,
    I: SortedStartsMap<T, VC>,
{
}

impl<T, VC, I> SortedDisjointMap<T, VC> for Peekable<I>
where
    T: Integer,
    VC: ValueCarrier,
    I: SortedDisjointMap<T, VC>,
{
}

impl<T, VC> SortedStartsMap<T, VC> for Empty<(RangeInclusive<T>, VC)>
where
    T: Integer,
    VC: ValueCarrier,
{
}

impl<T, VC> SortedDisjointMap<T, VC> for Empty<(RangeInclusive<T>, VC)>
where
    T: Integer,
    VC: ValueCarrier,
{
}

impl<T, VC> SortedStartsMap<T, VC> for Once<(RangeInclusive<T>, VC)>
where
    T: Integer,
    VC: ValueCarrier,
{
}

impl<T, VC> SortedDisjointMap<T, VC> for Once<(RangeInclusive<T>, VC)>
where
    T: Integer,
    VC: ValueCarrier,
{
}

impl<T, VC, I, IInner, TMap> SortedStartsMap<T, VC> for FlatMap<option::IntoIter<I>, IInner, TMap>
where
    T: Integer,
    VC: ValueCarrier,
    IInner: SortedStartsMap<T, VC>,
    I: SortedStartsMap<T, VC>,
    TMap: FnMut(I) -> IInner,
{
}

impl<T, VC, I> SortedStartsMap<T, VC> for Flatten<option::IntoIter<I>>
where
    T: Integer,
    VC: ValueCarrier,
    I: SortedStartsMap<T, VC>,
{
}

impl<T, VC, I> SortedDisjointMap<T, VC> for Flatten<option::IntoIter<I>>
where
    T: Integer,
    VC: ValueCarrier,
    I: SortedDisjointMap<T, VC>,
{
}

impl<T, VC, I, IInner, TMap> SortedDisjointMap<T, VC> for FlatMap<option::IntoIter<I>, IInner, TMap>
where
    T: Integer,
    VC: ValueCarrier,
    IInner: SortedDisjointMap<T, VC>,
    I: SortedDisjointMap<T, VC>,
    TMap: FnMut(I) -> IInner,
{
}
/// Used internally by [`UnionIterMap`] and [`SymDiffIterMap`].
pub trait PrioritySortedStartsMap<T, VC>: Iterator<Item = Priority<T, VC>> + FusedIterator
where
    T: Integer,
    VC: ValueCarrier,
{
}

/// Marks iterators that provide `(range, value)` pairs that are sorted and disjoint. Set operations on
/// iterators that implement this trait can be performed in linear time.
///
/// # Table of Contents
/// * [`SortedDisjointMap` Constructors](#sorteddisjointmap-constructors)
///   * [Examples](#constructor-examples)
/// * [`SortedDisjointMap` Set Operations](#sorteddisjointmap-set-operations)
///   * [Performance](#performance)
///   * [Examples](#examples)
/// * [How to mark your type as `SortedDisjointMap`](#how-to-mark-your-type-as-sorteddisjointmap)
///
/// # `SortedDisjointMap` Constructors
///
/// You'll usually construct a `SortedDisjointMap` iterator from a [`RangeMapBlaze`] or a [`CheckSortedDisjointMap`].
/// Here is a summary table, followed by [examples](#constructor-examples).  You can also [define your own
/// `SortedDisjointMap`](#how-to-mark-your-type-as-sorteddisjointmap).
///
/// | Input type | Method |
/// |------------|--------|
/// | [`RangeMapBlaze`] | [`range_values`] |
/// | [`RangeMapBlaze`] | [`into_range_values`] |
/// | sorted & disjoint ranges and values | [`CheckSortedDisjointMap::new`] |
/// |  *your iterator type* | *[How to mark your type as `SortedDisjointMap`][1]* |
///
/// [`SortedDisjointMap`]: trait.SortedDisjointMap.html#table-of-contents
/// [`range_values`]: RangeMapBlaze::range_values
/// [`into_range_values`]: RangeMapBlaze::into_range_values
/// [1]: #how-to-mark-your-type-as-sorteddisjointmap
/// [`RangesIter`]: crate::RangesIter
/// [`BitAnd`]: core::ops::BitAnd
/// [`Not`]: core::ops::Not
///
/// ## Constructor Examples
/// ```
/// use range_set_blaze::prelude::*;
///
/// // RangeMapBlaze's .range_values(), and .into_range_values()
/// let r = RangeMapBlaze::from_iter([ (100, "b"), (1, "c"), (3, "a"), (2, "a"), (1, "a")]);
/// let a = r.range_values();
/// assert_eq!(a.into_string(), r#"(1..=3, "a"), (100..=100, "b")"#);
/// // 'into_range_values' takes ownership of the 'RangeMapBlaze'
/// let a = r.into_range_values();
/// assert_eq!(a.into_string(), r#"(1..=3, "a"), (100..=100, "b")"#);
///
/// // CheckSortedDisjointMap -- unsorted or overlapping input ranges will cause a panic.
/// let a = CheckSortedDisjointMap::new([(1..=3, &"a"), (100..=100, &"b")]);
/// assert_eq!(a.into_string(), r#"(1..=3, "a"), (100..=100, "b")"#);
/// ```
///
/// # `SortedDisjointMap` Set Operations
///
/// You can perform set operations on `SortedDisjointMap`s and `SortedDisjoint` sets using operators.
/// In the table below, `a`, `b`, and `c` are `SortedDisjointMap` and `s` is a `SortedDisjoint` set.
///
/// | Set Operator               | Operator                      | Multiway (same type)                                      | Multiway (different types)                     |
/// |----------------------------|-------------------------------|-----------------------------------------------------------|-----------------------------------------------|
/// | [`union`]                  | [`a` &#124; `b`]              | <code>[a, b, c].[union][multiway_union]() </code>                | [`union_map_dyn!`](a, b, c)                    |
/// | [`intersection`]           | [`a & b`]                     | <code>[a, b, c].[intersection][multiway_intersection]() </code>  | [`intersection_map_dyn!`](a, b, c)             |
/// | `intersection`             | [`a.map_and_set_intersection(s)`] | *n/a*                                                     | *n/a*                                          |
/// | [`difference`]             | [`a - b`]                     | *n/a*                                                     | *n/a*                                          |
/// | `difference`               | [`a.map_and_set_difference(s)`] | *n/a*                                                     | *n/a*                                          |
/// | [`symmetric_difference`]   | [`a ^ b`]                     | <code>[a, b, c].[symmetric_difference][multiway_symmetric_difference]() </code> | [`symmetric_difference_map_dyn!`](a, b, c) |
/// | [`complement`] (to set)    | [`!a`]                        | *n/a*                                                     | *n/a*                                          |
/// | `complement` (to map)      | [`a.complement_with(&value)`] | *n/a*                                                     | *n/a*                                          |
///
/// [`union`]: trait.SortedDisjointMap.html#method.union
/// [`intersection`]: trait.SortedDisjointMap.html#method.intersection
/// [`difference`]: trait.SortedDisjointMap.html#method.difference
/// [`symmetric_difference`]: trait.SortedDisjointMap.html#method.symmetric_difference
/// [`complement`]: trait.SortedDisjointMap.html#method.complement
/// [`a` &#124; `b`]: trait.SortedDisjointMap.html#method.union
/// [`a & b`]: trait.SortedDisjointMap.html#method.intersection
/// [`a.map_and_set_intersection(s)`]: trait.SortedDisjointMap.html#method.map_and_set_intersection
/// [`a - b`]: trait.SortedDisjointMap.html#method.difference
/// [`a.map_and_set_difference(s)`]: trait.SortedDisjointMap.html#method.map_and_set_difference
/// [`a ^ b`]: trait.SortedDisjointMap.html#method.symmetric_difference
/// [`!a`]: trait.SortedDisjointMap.html#method.complement
/// [`a.complement_with(&value)`]: trait.SortedDisjointMap.html#method.complement_with
/// [multiway_union]: trait.MultiwaySortedDisjointMap.html#method.union
/// [multiway_intersection]: trait.MultiwaySortedDisjointMap.html#method.intersection
/// [multiway_symmetric_difference]: trait.MultiwaySortedDisjointMap.html#method.symmetric_difference
/// [`union_map_dyn!`]: macro.union_map_dyn.html
/// [`intersection_map_dyn!`]: macro.intersection_map_dyn.html
/// [`symmetric_difference_map_dyn!`]: macro.symmetric_difference_map_dyn.html
///
/// The union of any number of maps is defined such that, for any overlapping keys,
/// the values from the right-most input take precedence. This approach ensures
/// that the data from the right-most inputs remains dominant when merging with
/// later inputs. Likewise, for symmetric difference of three or more maps.
///
/// ## Performance
///
/// Every operation is implemented as a single pass over the sorted & disjoint ranges, with minimal memory.
///
/// This is true even when applying multiple operations. The last example below demonstrates this.
///
/// ## Standard Iterators
///
/// Many `core::iter` adapters preserve this marker trait when the inner iterator already
/// implements it, including `filter`, `take_while`, `skip_while`, `fuse`, `skip`, `take`,
/// and `peekable`. `empty`/`once` iterators and `Option`-based `flatten`/`flat_map` are also
/// supported.
///
/// ## Examples
///
/// ```
/// use range_set_blaze::prelude::*;
///
/// let a0 = RangeMapBlaze::from_iter([(2..=6, "a")]);
/// let b0 = RangeMapBlaze::from_iter([(1..=2, "b"), (5..=100, "b")]);
///
/// // 'union' method and 'into_string' method
/// let (a, b) = (a0.range_values(), b0.range_values());
/// let result = a.union(b);
/// assert_eq!(result.into_string(), r#"(1..=2, "b"), (3..=4, "a"), (5..=100, "b")"#);
///
/// // '|' operator and 'equal' method
/// let (a, b) = (a0.range_values(), b0.range_values());
/// let result = a | b;
/// assert!(result.equal(CheckSortedDisjointMap::new([(1..=2, &"b"),  (3..=4, &"a"), (5..=100, &"b")])));
///
/// // multiway union of same type
/// let z0 = RangeMapBlaze::from_iter([(2..=2, "z"), (6..=200, "z")]);
/// let (z, a, b) = (z0.range_values(), a0.range_values(), b0.range_values());
/// let result = [z, a, b].union();
/// assert_eq!(result.into_string(), r#"(1..=2, "b"), (3..=4, "a"), (5..=100, "b"), (101..=200, "z")"#
/// );
///
/// // multiway union of different types
/// let (a, b) = (a0.range_values(), b0.range_values());
/// let z = CheckSortedDisjointMap::new([(2..=2, &"z"), (6..=200, &"z")]);
/// let result = union_map_dyn!(z, a, b);
/// assert_eq!(result.into_string(), r#"(1..=2, "b"), (3..=4, "a"), (5..=100, "b"), (101..=200, "z")"# );
///
/// // Applying multiple operators makes only one pass through the inputs with minimal memory.
/// let (z, a, b) = (z0.range_values(), a0.range_values(), b0.range_values());
/// let result = b - (z | a);
/// assert_eq!(result.into_string(), r#"(1..=1, "b")"#);
/// ```
/// # How to mark your type as `SortedDisjointMap`
///
/// To mark your iterator type as `SortedDisjointMap`, you implement the `SortedStartsMap` and `SortedDisjointMap` traits.
/// This is your promise to the compiler that your iterator will provide nonempty inclusive ranges
/// that are sorted by start and do not overlap. Touching ranges with logically equal values, as
/// determined by [`ValueCarrier::value_eq`], must be coalesced; touching ranges with different values
/// may remain separate.
///
/// When you do this, your iterator will get access to the
/// efficient set operations methods, such as [`intersection`] and [`complement`].
///
/// > To use operators such as `&` and `!`, you must also implement the [`BitAnd`], [`Not`], etc. traits.
/// >
/// > If you want others to use your marked iterator type, reexport:
/// > `pub use range_set_blaze::{SortedDisjointMap, SortedStartsMap};`
pub trait SortedDisjointMap<T, VC>: SortedStartsMap<T, VC>
where
    T: Integer,
    VC: ValueCarrier,
{
    /// Fills the gaps in this sorted, disjoint map stream with `None` values.
    ///
    /// The returned stream covers the full integer domain from `T::min_value()`
    /// through `T::max_value()`. Existing ranges retain their values as
    /// `Some(value)`.
    ///
    /// The result implements `SortedDisjointMap<T, Option<VC>>`. Here, `None`
    /// is an ordinary logical map value, so the result's key domain is universal:
    /// [`SortedDisjointMap::into_sorted_disjoint`] covers the full domain and
    /// [`SortedDisjointMap::complement`] is empty.
    ///
    /// # Examples
    ///
    /// ```
    /// use range_set_blaze::{CheckSortedDisjointMap, SortedDisjointMap};
    ///
    /// let stream = CheckSortedDisjointMap::new([(1..=3, &"red"), (7..=10, &"blue")]);
    /// let filled = stream.fill_gaps().collect::<Vec<_>>();
    /// assert_eq!(filled[0], (0..=0, None));
    /// assert_eq!(filled[1], (1..=3, Some(&"red")));
    /// assert_eq!(filled[2], (4..=6, None));
    /// assert_eq!(filled[3], (7..=10, Some(&"blue")));
    /// assert_eq!(filled[4], (11..=u8::MAX, None));
    /// ```
    #[inline]
    fn fill_gaps(self) -> FillGapsIterMap<T, VC, Self>
    where
        Self: Sized,
    {
        FillGapsIterMap::new(self)
    }

    /// Converts a [`SortedDisjointMap`] iterator into a [`SortedDisjoint`] iterator.
    ///```
    /// use range_set_blaze::prelude::*;
    ///
    /// let a = CheckSortedDisjointMap::new([(1..=3, &"a"), (100..=100, &"b")]);
    /// let b = a.into_sorted_disjoint();
    /// assert!(b.into_string() == "1..=3, 100..=100");
    /// ```
    #[inline]
    fn into_sorted_disjoint(self) -> RangeValuesToRangesIter<T, VC, Self>
    where
        Self: Sized,
    {
        RangeValuesToRangesIter::new(self)
    }
    /// Given two [`SortedDisjointMap`] iterators, efficiently returns a [`SortedDisjointMap`] iterator of their union.
    ///
    /// [`SortedDisjointMap`]: trait.SortedDisjointMap.html#table-of-contents
    ///
    /// # Examples
    ///
    /// ```
    /// use range_set_blaze::prelude::*;
    ///
    /// let a0 = RangeMapBlaze::from_iter([(2..=3, "a")]);
    /// let a = a0.range_values();
    /// let b = CheckSortedDisjointMap::new([(1..=2, &"b")]);
    /// let union = a.union(b);
    /// assert_eq!(union.into_string(), r#"(1..=2, "b"), (3..=3, "a")"#);
    ///
    /// // Alternatively, we can use "|" because CheckSortedDisjointMap defines
    /// // ops::bitor as SortedDisjointMap::union.
    /// let a = a0.range_values();
    /// let b = CheckSortedDisjointMap::new([(1..=2, &"b")]);
    /// let union = a | b;
    /// assert_eq!(union.into_string(), r#"(1..=2, "b"), (3..=3, "a")"#);
    /// ```
    #[inline]
    fn union<R>(self, other: R) -> UnionMergeMap<T, VC, Self, R::IntoIter>
    where
        R: IntoIterator<Item = Self::Item>,
        R::IntoIter: SortedDisjointMap<T, VC>,
        Self: Sized,
    {
        UnionIterMap::new2(self, other.into_iter())
    }

    /// Given two [`SortedDisjointMap`] iterators, efficiently returns a [`SortedDisjointMap`] iterator of their intersection.
    ///
    /// [`SortedDisjointMap`]: trait.SortedDisjointMap.html#table-of-contents
    ///
    /// # Examples
    ///
    /// ```
    /// use range_set_blaze::prelude::*;
    ///
    /// let a0 = RangeMapBlaze::from_iter([(2..=3, "a")]);
    /// let a = a0.range_values();
    /// let b = CheckSortedDisjointMap::new([(1..=2, &"b")]);
    /// let intersection = a.intersection(b);
    /// assert_eq!(intersection.into_string(), r#"(2..=2, "b")"#);
    ///
    /// // Alternatively, we can use "&" because CheckSortedDisjointMap defines
    /// // `ops::BitAnd` as `SortedDisjointMap::intersection`.
    /// let a0 = RangeMapBlaze::from_iter([(2..=3, "a")]);
    /// let a = a0.range_values();
    /// let b = CheckSortedDisjointMap::new([(1..=2, &"b")]);
    /// let intersection = a & b;
    /// assert_eq!(intersection.into_string(), r#"(2..=2, "b")"#);
    /// ```
    #[inline]
    fn intersection<R>(self, other: R) -> IntersectionMap<T, VC, Self, R::IntoIter>
    where
        R: IntoIterator<Item = Self::Item>,
        R::IntoIter: SortedDisjointMap<T, VC>,
        Self: Sized,
    {
        let other = other.into_iter();
        let sorted_disjoint = self.into_sorted_disjoint();
        IntersectionIterMap::new(other, sorted_disjoint)
    }

    /// Given a [`SortedDisjointMap`] iterator and a [`SortedDisjoint`] iterator,
    /// efficiently returns a [`SortedDisjointMap`] iterator of their intersection.
    ///
    /// [`SortedDisjointMap`]: trait.SortedDisjointMap.html#table-of-contents
    /// [`SortedDisjoint`]: trait.SortedDisjoint.html#table-of-contents
    ///
    /// # Examples
    ///
    /// ```
    /// use range_set_blaze::prelude::*;
    ///
    /// let a = CheckSortedDisjointMap::new([(1..=2, &"a")]);
    /// let b = CheckSortedDisjoint::new([2..=3]);
    /// let intersection = a.map_and_set_intersection(b);
    /// assert_eq!(intersection.into_string(), r#"(2..=2, "a")"#);
    /// ```
    #[inline]
    fn map_and_set_intersection<R>(self, other: R) -> IntersectionIterMap<T, VC, Self, R::IntoIter>
    where
        R: IntoIterator<Item = RangeInclusive<T>>,
        R::IntoIter: SortedDisjoint<T>,
        Self: Sized,
    {
        IntersectionIterMap::new(self, other.into_iter())
    }

    /// Given two [`SortedDisjointMap`] iterators, efficiently returns a [`SortedDisjointMap`] iterator of their set difference.
    ///
    /// [`SortedDisjointMap`]: trait.SortedDisjointMap.html#table-of-contents
    ///
    /// # Examples
    ///
    /// ```
    /// use range_set_blaze::prelude::*;
    ///
    /// let a = CheckSortedDisjointMap::new([(1..=2, &"a")]);
    /// let b0 = RangeMapBlaze::from_iter([(2..=3, "b")]);
    /// let b = b0.range_values();
    /// let difference = a.difference(b);
    /// assert_eq!(difference.into_string(), r#"(1..=1, "a")"#);
    ///
    /// // Alternatively, we can use "-" because `CheckSortedDisjointMap` defines
    /// // `ops::Sub` as `SortedDisjointMap::difference`.
    /// let a = CheckSortedDisjointMap::new([(1..=2, &"a")]);
    /// let b = b0.range_values();
    /// let difference = a - b;
    /// assert_eq!(difference.into_string(), r#"(1..=1, "a")"#);
    /// ```
    #[inline]
    fn difference<R>(self, other: R) -> DifferenceMap<T, VC, Self, R::IntoIter>
    where
        R: IntoIterator<Item = Self::Item>,
        R::IntoIter: SortedDisjointMap<T, VC>,
        Self: Sized,
    {
        let sorted_disjoint_map = other.into_iter();
        let complement = sorted_disjoint_map.complement();
        IntersectionIterMap::new(self, complement)
    }

    /// Given a [`SortedDisjointMap`] iterator and a [`SortedDisjoint`] iterator,
    /// efficiently returns a [`SortedDisjointMap`] iterator of their set difference.
    ///
    /// [`SortedDisjointMap`]: trait.SortedDisjointMap.html#table-of-contents
    /// [`SortedDisjoint`]: trait.SortedDisjoint.html#table-of-contents
    ///
    /// # Examples
    ///
    /// ```
    /// use range_set_blaze::prelude::*;
    ///
    /// let a = CheckSortedDisjointMap::new([(1..=2, &"a")]);
    /// let b = RangeMapBlaze::from_iter([(2..=3, "b")]).into_ranges();
    /// let difference = a.map_and_set_difference(b);
    /// assert_eq!(difference.into_string(), r#"(1..=1, "a")"#);
    /// ```
    #[inline]
    fn map_and_set_difference<R>(self, other: R) -> DifferenceMapInternal<T, VC, Self, R::IntoIter>
    where
        R: IntoIterator<Item = RangeInclusive<T>>,
        R::IntoIter: SortedDisjoint<T>,
        Self: Sized,
    {
        let sorted_disjoint = other.into_iter();
        let complement = sorted_disjoint.complement();
        IntersectionIterMap::new(self, complement)
    }

    /// Returns the complement of a [`SortedDisjointMap`]'s keys as a [`SortedDisjoint`] iterator.
    ///
    /// [`SortedDisjointMap`]: trait.SortedDisjointMap.html#table-of-contents
    /// [`SortedDisjoint`]: trait.SortedDisjoint.html#table-of-contents
    ///
    /// # Examples
    ///
    /// ```
    /// use range_set_blaze::prelude::*;
    ///
    /// let a = CheckSortedDisjointMap::new([(10_u8..=20, &"a"), (100..=200, &"b")]);
    /// let complement = a.complement();
    /// assert_eq!(complement.into_string(), "0..=9, 21..=99, 201..=255");
    ///
    /// // Alternatively, we can use "!" because `CheckSortedDisjointMap` implements
    /// // `ops::Not` as `complement`.
    /// let a = CheckSortedDisjointMap::new([(10_u8..=20, &"a"), (100..=200, &"b")]);
    /// let complement_using_not = !a;
    /// assert_eq!(complement_using_not.into_string(), "0..=9, 21..=99, 201..=255");
    /// ```
    #[inline]
    fn complement(self) -> NotIter<T, RangeValuesToRangesIter<T, VC, Self>>
    where
        Self: Sized,
    {
        let sorted_disjoint = self.into_sorted_disjoint();
        sorted_disjoint.complement()
    }

    /// Returns the complement of a [`SortedDisjointMap`]'s keys, associating each range with the provided value `v`.
    /// The result is a [`SortedDisjointMap`] iterator.
    ///
    /// [`SortedDisjointMap`]: trait.SortedDisjointMap.html#table-of-contents
    ///
    /// # Examples
    ///
    /// ```
    /// use range_set_blaze::prelude::*;
    ///
    /// let a = CheckSortedDisjointMap::new([(10_u8..=20, &"a"), (100..=200, &"b")]);
    /// let complement = a.complement_with(&"z");
    /// assert_eq!(complement.into_string(), r#"(0..=9, "z"), (21..=99, "z"), (201..=255, "z")"#);
    /// ```
    #[inline]
    fn complement_with(
        self,
        v: &VC::Value,
    ) -> RangeToRangeValueIter<'_, T, VC::Value, NotIter<T, impl SortedDisjoint<T>>>
    where
        Self: Sized,
    {
        let complement = self.complement();
        RangeToRangeValueIter::new(complement, v)
    }

    /// Given two [`SortedDisjointMap`] iterators, efficiently returns a [`SortedDisjointMap`] iterator
    /// of their symmetric difference.
    ///
    /// [`SortedDisjointMap`]: trait.SortedDisjointMap.html#table-of-contents
    ///
    /// # Examples
    ///
    /// ```
    /// use range_set_blaze::prelude::*;
    ///
    /// let a = CheckSortedDisjointMap::new([(1..=2, &"a")]);
    /// let b0 = RangeMapBlaze::from_iter([(2..=3, "b")]);
    /// let b = b0.range_values();
    /// let symmetric_difference = a.symmetric_difference(b);
    /// assert_eq!(symmetric_difference.into_string(), r#"(1..=1, "a"), (3..=3, "b")"#);
    ///
    /// // Alternatively, we can use "^" because CheckSortedDisjointMap defines
    /// // ops::bitxor as SortedDisjointMap::symmetric_difference.
    /// let a = CheckSortedDisjointMap::new([(1..=2, &"a")]);
    /// let b = b0.range_values();
    /// let symmetric_difference = a ^ b;
    /// assert_eq!(symmetric_difference.into_string(), r#"(1..=1, "a"), (3..=3, "b")"#);
    /// ```
    #[inline]
    fn symmetric_difference<R>(self, other: R) -> SymDiffMergeMap<T, VC, Self, R::IntoIter>
    where
        R: IntoIterator<Item = Self::Item>,
        R::IntoIter: SortedDisjointMap<T, VC>,
        Self: Sized,
        VC: ValueCarrier,
    {
        SymDiffIterMap::new2(self, other.into_iter())
    }

    /// Given two [`SortedDisjointMap`] iterators, efficiently tells if they are equal. Unlike most equality testing in Rust,
    /// this method takes ownership of the iterators and consumes them.
    ///
    /// [`SortedDisjointMap`]: trait.SortedDisjointMap.html#table-of-contents
    ///
    /// # Examples
    ///
    /// ```
    /// use range_set_blaze::prelude::*;
    ///
    /// let a = CheckSortedDisjointMap::new([(1..=2, &"a")]);
    /// let b0 = RangeMapBlaze::from_iter([(1..=2, "a")]);
    /// let b = b0.range_values();
    /// assert!(a.equal(b));
    /// ```
    fn equal<R>(self, other: R) -> bool
    where
        R: IntoIterator<Item = Self::Item>,
        R::IntoIter: SortedDisjointMap<T, VC>,
        Self: Sized,
    {
        use itertools::Itertools;

        self.zip_longest(other).all(|pair| {
            match pair {
                itertools::EitherOrBoth::Both(
                    (self_range, self_value),
                    (other_range, other_value),
                ) => {
                    // Place your custom equality logic here for matching elements
                    self_range == other_range && self_value.value_eq(&other_value)
                }
                _ => false, // Handles the case where iterators are of different lengths
            }
        })
    }

    /// Returns `true` if the [`SortedDisjointMap`] contains no elements.
    ///
    /// [`SortedDisjointMap`]: trait.SortedDisjointMap.html#table-of-contents
    ///
    /// # Examples
    ///
    /// ```
    /// use range_set_blaze::prelude::*;
    ///
    /// let a = CheckSortedDisjointMap::new([(1..=2, &"a")]);
    /// assert!(!a.is_empty());
    /// ```
    #[inline]
    #[allow(clippy::wrong_self_convention)]
    fn is_empty(mut self) -> bool
    where
        Self: Sized,
    {
        self.next().is_none()
    }

    /// Returns `true` if the map contains all possible integers.
    ///
    /// For type `T`, this means the ranges start at `T::min_value()` and continue contiguously up to `T::max_value()`.
    /// Complexity: O(n) to verify contiguity across ranges.
    ///
    /// # Examples
    ///
    /// ```
    /// use range_set_blaze::prelude::*;
    ///
    /// let b = CheckSortedDisjointMap::new([(0_u8..=100, &"x"), (101..=255, &"y")]);
    /// assert!(b.is_universal());
    ///
    /// let c = CheckSortedDisjointMap::new([(1_u8..=255, &"z")]);
    /// assert!(!c.is_universal());
    /// ```
    #[inline]
    #[allow(clippy::wrong_self_convention)]
    fn is_universal(self) -> bool
    where
        Self: Sized,
    {
        let mut expected_start = T::min_value();

        for (range, _) in self {
            let (start, end) = range.into_inner();

            // Check if this range starts where we expect
            if start != expected_start {
                return false;
            }

            // If this range reaches the maximum value, we're done
            if end == T::max_value() {
                return true;
            }

            // Set up for the next range
            expected_start = end.add_one();
        }

        // If we get here, we didn't reach the maximum value
        false
    }

    /// Create a [`RangeMapBlaze`] from a [`SortedDisjointMap`] iterator.
    ///
    /// *For more about constructors and performance, see [`RangeMapBlaze` Constructors](struct.RangeMapBlaze.html#rangemapblaze-constructors).*
    ///
    /// [`SortedDisjointMap`]: trait.SortedDisjointMap.html#table-of-contents
    /// # Examples
    ///
    /// ```
    /// use range_set_blaze::prelude::*;
    ///
    /// let a0 = RangeMapBlaze::from_sorted_disjoint_map(CheckSortedDisjointMap::new([(-10..=-5, &"a"), (1..=2, &"b")]));
    /// let a1: RangeMapBlaze<i32,_> = CheckSortedDisjointMap::new([(-10..=-5, &"a"), (1..=2, &"b")]).into_range_map_blaze();
    /// assert!(a0 == a1 && a0.to_string() == r#"(-10..=-5, "a"), (1..=2, "b")"#);
    /// ```
    fn into_range_map_blaze(self) -> RangeMapBlaze<T, VC::Value>
    where
        Self: Sized,
    {
        RangeMapBlaze::from_sorted_disjoint_map(self)
    }
}

/// Converts the implementing type into a String by consuming it.
pub trait IntoString {
    /// Consumes the implementing type and converts it into a String.
    fn into_string(self) -> String;
}

impl<T, I> IntoString for I
where
    T: Debug,
    I: Iterator<Item = T>,
{
    fn into_string(self) -> String {
        self.map(|item| format!("{item:?}"))
            .collect::<Vec<String>>()
            .join(", ")
    }
}

/// Gives the [`SortedDisjointMap`] trait to any iterator of range-value pairs. Will panic
/// if the trait is not satisfied.
///
/// The iterator will panic
/// if/when it finds that the ranges are not actually sorted and disjoint or if the values overlap inappropriately.
///
/// [`SortedDisjointMap`]: crate::SortedDisjointMap.html#table-of-contents
///
/// # Performance
///
/// All checking is done at runtime, but it should still be fast.
///
/// # Example
///
/// ```
/// use range_set_blaze::prelude::*;
///
/// let a = CheckSortedDisjointMap::new([(4..=6, &"a")]);
/// let b = CheckSortedDisjointMap::new([(1..=3, &"z"), (5..=10, &"b")]);
/// let union = a | b;
/// assert_eq!(union.into_string(), r#"(1..=3, "z"), (4..=4, "a"), (5..=10, "b")"#);
/// ```
///
/// Here the ranges are not sorted and disjoint, so the iterator will panic.
/// ```should_panic
/// use range_set_blaze::prelude::*;
///
/// let a = CheckSortedDisjointMap::new([(1..=3, &"a"), (5..=10, &"b")]);
/// let b = CheckSortedDisjointMap::new([(4..=6, &"c"), (-10..=12, &"d")]);
/// let union = a | b;
/// assert_eq!(union.into_string(), "1..=3 -> a, 5..=10 -> b");
/// ```
#[allow(clippy::module_name_repetitions)]
#[must_use = "iterators are lazy and do nothing unless consumed"]
#[derive(Debug, Clone)]
pub struct CheckSortedDisjointMap<T, VC, I> {
    iter: I,
    seen_none: bool,
    previous: Option<(RangeInclusive<T>, VC)>,
}

// define new
impl<T, VC, I> CheckSortedDisjointMap<T, VC, I>
where
    T: Integer,
    VC: ValueCarrier,
    I: Iterator<Item = (RangeInclusive<T>, VC)>,
{
    /// Creates a new [`CheckSortedDisjointMap`] from an iterator of ranges and values. See [`CheckSortedDisjointMap`] for details and examples.
    #[inline]
    #[must_use = "iterators are lazy and do nothing unless consumed"]
    pub fn new<J>(iter: J) -> Self
    where
        J: IntoIterator<Item = (RangeInclusive<T>, VC), IntoIter = I>,
    {
        Self {
            iter: iter.into_iter(),
            seen_none: false,
            previous: None,
        }
    }
}

impl<T, VC, I> Default for CheckSortedDisjointMap<T, VC, I>
where
    T: Integer,
    VC: ValueCarrier,
    I: Iterator<Item = (RangeInclusive<T>, VC)> + Default,
{
    fn default() -> Self {
        // Utilize I::default() to satisfy the iterator requirement.
        Self::new(I::default())
    }
}

impl<T, VC, I> FusedIterator for CheckSortedDisjointMap<T, VC, I>
where
    T: Integer,
    VC: ValueCarrier,
    I: Iterator<Item = (RangeInclusive<T>, VC)>,
{
}

fn range_value_clone<T, VC>(range_value: &(RangeInclusive<T>, VC)) -> (RangeInclusive<T>, VC)
where
    T: Integer,
    VC: ValueCarrier,
{
    let (range, value) = range_value;
    (range.clone(), value.clone())
}

impl<T, VC, I> Iterator for CheckSortedDisjointMap<T, VC, I>
where
    T: Integer,
    VC: ValueCarrier,
    I: Iterator<Item = (RangeInclusive<T>, VC)>,
{
    type Item = (RangeInclusive<T>, VC);

    #[allow(clippy::manual_assert)] // We use "if...panic!" for coverage auditing.
    fn next(&mut self) -> Option<Self::Item> {
        // Get the next item
        let range_value = self.iter.next();

        // If it's None, we're done (but remember that we've seen None)
        let Some(range_value) = range_value else {
            self.seen_none = true;
            return None;
        };

        // if the next item is Some, check that we haven't seen None before
        if self.seen_none {
            panic!("a value must not be returned after None")
        }

        // Check that the range is not empty
        let (range, _) = &range_value;
        let (start, end) = range.clone().into_inner();
        if start > end {
            panic!("start must be <= end")
        }

        // If previous is None, we're done (but remember this pair as previous)
        let Some(previous) = self.previous.take() else {
            self.previous = Some(range_value_clone(&range_value));
            return Some(range_value);
        };

        // The next_item is Some and previous is Some, so check that the ranges are disjoint and sorted
        let (previous_range, previous_value) = previous;
        let previous_end = *previous_range.end();
        if previous_end >= start {
            panic!("ranges must be disjoint and sorted")
        }

        let (_, range_value_value) = &range_value;
        if previous_end.add_one() == start && previous_value.value_eq(range_value_value) {
            panic!("touching ranges must have different values")
        }

        // Remember this pair as previous
        self.previous = Some(range_value_clone(&range_value));
        Some(range_value)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

/// Used internally by `MergeMap`.
#[derive(Clone, Debug)]
pub struct Priority<T, VC> {
    range_value: (RangeInclusive<T>, VC),
    priority_number: usize,
}

impl<T, VC> Priority<T, VC> {
    pub(crate) const fn new(range_value: (RangeInclusive<T>, VC), priority_number: usize) -> Self {
        Self {
            range_value,
            priority_number,
        }
    }
}

impl<T, VC> Priority<T, VC>
where
    T: Integer,
    VC: ValueCarrier,
{
    /// Returns a reference to `range_value`.
    pub const fn range_value(&self) -> &(RangeInclusive<T>, VC) {
        &self.range_value
    }

    /// Consumes `Priority` and returns `range_value`.
    pub fn into_range_value(self) -> (RangeInclusive<T>, VC) {
        self.range_value
    }

    /// Updates the range part of `range_value`.
    pub const fn set_range(&mut self, range: RangeInclusive<T>) {
        let (stored_range, _) = &mut self.range_value;
        *stored_range = range;
    }

    /// Returns the start of the range.
    pub const fn start(&self) -> T {
        let (range, _) = &self.range_value;
        *range.start()
    }

    /// Returns the end of the range.
    pub const fn end(&self) -> T {
        let (range, _) = &self.range_value;
        *range.end()
    }

    /// Returns the start and end of the range. (Assuming direct access to start and end)
    pub const fn start_and_end(&self) -> (T, T) {
        let (range, _) = &self.range_value;
        (*range.start(), *range.end())
    }

    /// Returns a reference to the value part of `range_value`.
    pub const fn value(&self) -> &VC {
        let (_, value) = &self.range_value;
        value
    }
}

// Implement `PartialEq` to allow comparison (needed for `Eq`).
impl<T, VC> PartialEq for Priority<T, VC>
where
    T: Integer,
    VC: ValueCarrier,
{
    fn eq(&self, other: &Self) -> bool {
        self.priority_number == other.priority_number
    }
}

// Implement `Eq` because `BinaryHeap` requires it.
impl<T, VC> Eq for Priority<T, VC>
where
    T: Integer,
    VC: ValueCarrier,
{
}

// Implement `Ord` so the heap knows how to compare elements.
impl<T, VC> Ord for Priority<T, VC>
where
    T: Integer,
    VC: ValueCarrier,
{
    fn cmp(&self, other: &Self) -> Ordering {
        debug_assert_ne!(
            self.priority_number, other.priority_number,
            "Priority numbers are expected to be distinct for comparison."
        );
        // bigger is better
        self.priority_number.cmp(&other.priority_number)
    }
}

// Implement `PartialOrd` to allow comparison (needed for `Ord`).
impl<T, VC> PartialOrd for Priority<T, VC>
where
    T: Integer,
    VC: ValueCarrier,
{
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// Used internally by `complement_with`.
#[must_use = "iterators are lazy and do nothing unless consumed"]
#[derive(Clone, Debug)]
pub struct RangeToRangeValueIter<'a, T, V, I> {
    inner: I,
    value: &'a V,
    phantom: PhantomData<T>,
}

impl<'a, T, V, I> RangeToRangeValueIter<'a, T, V, I>
where
    T: Integer,
    V: Eq + Clone,
    I: SortedDisjoint<T>,
{
    pub(crate) const fn new(inner: I, value: &'a V) -> Self {
        Self {
            inner,
            value,
            phantom: PhantomData,
        }
    }
}

impl<T, V, I> FusedIterator for RangeToRangeValueIter<'_, T, V, I>
where
    T: Integer,
    V: Eq + Clone,
    I: SortedDisjoint<T>,
{
}

impl<'a, T, V, I> Iterator for RangeToRangeValueIter<'a, T, V, I>
where
    T: Integer,
    V: Eq + Clone,
    I: SortedDisjoint<T>,
{
    type Item = (RangeInclusive<T>, &'a V);

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(|range| (range, self.value))
    }
}

// implements SortedDisjointMap
impl<'a, T, V, I> SortedStartsMap<T, &'a V> for RangeToRangeValueIter<'a, T, V, I>
where
    T: Integer,
    V: Eq + Clone,
    I: SortedDisjoint<T>,
{
}
impl<'a, T, V, I> SortedDisjointMap<T, &'a V> for RangeToRangeValueIter<'a, T, V, I>
where
    T: Integer,
    V: Eq + Clone,
    I: SortedDisjoint<T>,
{
}

macro_rules! impl_sorted_map_traits_and_ops {
    ($IterType:ty, $V:ty, $VC:ty, $($more_generics:tt)*) => {

        #[allow(single_use_lifetimes)]
        impl<$($more_generics)*, T> SortedStartsMap<T, $VC> for $IterType
        where
            T: Integer,
        {
        }

        #[allow(single_use_lifetimes)]
        impl<$($more_generics)*, T> SortedDisjointMap<T, $VC> for $IterType
        where
            T: Integer,
        {
        }

        #[allow(single_use_lifetimes)]
        impl<$($more_generics)*, T> ops::Not for $IterType
        where
            T: Integer,
        {
            type Output = NotMap<T, $VC, Self>;

            #[inline]
            fn not(self) -> Self::Output {
                self.complement()
            }
        }

        #[allow(single_use_lifetimes)]
        impl<$($more_generics)*, T, R> ops::BitOr<R> for $IterType
        where
            T: Integer,
            R: SortedDisjointMap<T, $VC>,
        {
            type Output = UnionMergeMap<T, $VC, Self, R>;

            #[inline]
            fn bitor(self, other: R) -> Self::Output {
                SortedDisjointMap::union(self, other)
            }
        }

        #[allow(single_use_lifetimes)]
        impl<$($more_generics)*, T, R> ops::Sub<R> for $IterType
        where
            T: Integer,
            R: SortedDisjointMap<T, $VC>,
        {
            type Output = DifferenceMap<T, $VC, Self, R>;

            #[inline]
            fn sub(self, other: R) -> Self::Output {
                SortedDisjointMap::difference(self, other)
            }
        }

        #[allow(single_use_lifetimes)]
        impl<$($more_generics)*, T, R> ops::BitXor<R> for $IterType
        where
            T: Integer,
            R: SortedDisjointMap<T, $VC>,
        {
            type Output = SymDiffMergeMap<T,  $VC, Self, R>;

            #[allow(clippy::suspicious_arithmetic_impl)]
            #[inline]
            fn bitxor(self, other: R) -> Self::Output {
                SortedDisjointMap::symmetric_difference(self, other)
            }
        }

        #[allow(single_use_lifetimes)]
        impl<$($more_generics)*, T, R> ops::BitAnd<R> for $IterType
        where
            T: Integer,
            R: SortedDisjointMap<T, $VC>,
        {
            type Output = IntersectionMap<T, $VC, Self, R>;

            #[inline]
            fn bitand(self, other: R) -> Self::Output {
                SortedDisjointMap::intersection(self, other)
            }
        }

    }
}

// CheckList: Be sure that these are all tested in 'test_every_sorted_disjoint_map_method'
impl_sorted_map_traits_and_ops!(CheckSortedDisjointMap<T, VC, I>, VC::Value, VC, VC: ValueCarrier, I: Iterator<Item = (RangeInclusive<T>,  VC)>);
impl_sorted_map_traits_and_ops!(DynSortedDisjointMap<'a, T, VC>, VC::Value, VC, 'a, VC: ValueCarrier);
impl_sorted_map_traits_and_ops!(FillGapsIterMap<T, VC, I>, Option<VC::Value>, Option<VC>, VC: ValueCarrier, I: SortedDisjointMap<T, VC>);
impl_sorted_map_traits_and_ops!(FillGapsIter<T, I>, bool, bool, I: SortedDisjoint<T>);
impl_sorted_map_traits_and_ops!(IntersectionIterMap<T, VC, I0, I1>,  VC::Value, VC, VC: ValueCarrier, I0: SortedDisjointMap<T, VC>, I1: SortedDisjoint<T>);
impl_sorted_map_traits_and_ops!(IntoRangeValuesIter<T, V>, V, Rc<V>, V: Eq + Clone);
impl_sorted_map_traits_and_ops!(RangeValuesIter<'a, T, V>, V, &'a V, 'a, V: Eq + Clone);
impl_sorted_map_traits_and_ops!(SymDiffIterMap<T, VC, I>, VC::Value, VC, VC: ValueCarrier, I: PrioritySortedStartsMap<T, VC>);
impl_sorted_map_traits_and_ops!(UnionIterMap<T, VC, I>, VC::Value, VC, VC: ValueCarrier, I: PrioritySortedStartsMap<T, VC>);

#[cfg(test)]
mod tests {
    use super::*;
    use core::iter::{empty, once};

    #[test]
    fn test_union_std_iters_map() {
        let a = empty::<(RangeInclusive<u64>, &&str)>();
        #[allow(clippy::iter_skip_zero)]
        let b = once((10u64..=20, &"a"))
            .skip_while(|_| false)
            .take_while(|_| true)
            .fuse()
            .skip(0)
            .peekable();
        #[allow(clippy::iter_on_single_items)]
        let b = Some(b).into_iter().flat_map(|x| x.filter(|_| true));
        #[allow(clippy::iter_on_single_items)]
        let b = Some(b).into_iter().flatten();
        assert_eq!(Some((10u64..=20, &"a")), a.union(b).next());
    }
}
