use crate::RangeSetBlaze;
use crate::map::ValueRef;
use crate::range_values::{MapIntoRangesIter, MapRangesIter, RangeValuesToRangesIter};
use crate::ranges_iter::RangesIter;
use crate::sorted_disjoint_map::IntoString;
use crate::{IntoRangesIter, UnionIter, UnionMerge};
use alloc::string::String;
use core::array;
use core::{
    iter::FusedIterator,
    ops::{self, RangeInclusive},
};

use crate::SortedDisjointMap;

use crate::{
    DifferenceMerge, DynSortedDisjoint, Integer, IntersectionMerge, NotIter, SymDiffIter,
    SymDiffMerge,
};

/// Used internally. Marks iterators that provide ranges sorted by start, but
/// that are not necessarily disjoint. The ranges are non-empty.
pub trait SortedStarts<T: Integer>: Iterator<Item = RangeInclusive<T>> + FusedIterator {}

/// Marks iterators that provide ranges that are sorted by start and disjoint. Set operations on
/// iterators that implement this trait can be performed in linear time.
///
/// # Table of Contents
/// * [`SortedDisjoint` Constructors](#sorteddisjoint-constructors)
///   * [Examples](#constructor-examples)
/// * [`SortedDisjoint` Set Operations](#sorteddisjoint-set-operations)
///   * [Performance](#performance)
///   * [Examples](#examples)
/// * [How to mark your type as `SortedDisjoint`](#how-to-mark-your-type-as-sorteddisjoint)
///   * [Example – Find the ordinal weekdays in September 2023](#example--find-the-ordinal-weekdays-in-september-2023)
///
/// # `SortedDisjoint` Constructors
///
/// You'll usually construct a `SortedDisjoint` iterator from a [`RangeSetBlaze`] or a [`CheckSortedDisjoint`].
/// Here is a summary table, followed by [examples](#constructor-examples). You can also [define your own
/// `SortedDisjoint`](#how-to-mark-your-type-as-sorteddisjoint).
///
/// | Input type | Method |
/// |------------|--------|
/// | [`RangeSetBlaze`] | [`ranges`] |
/// | [`RangeSetBlaze`] | [`into_ranges`] |
/// | sorted & disjoint ranges | [`CheckSortedDisjoint::new`] |
/// | [`RangeInclusive`] | [`RangeOnce::new`] |
/// |  *your iterator type* | *[How to mark your type as `SortedDisjoint`][1]* |
///
/// [`ranges`]: RangeSetBlaze::ranges
/// [`into_ranges`]: RangeSetBlaze::into_ranges
/// [1]: #how-to-mark-your-type-as-sorteddisjoint
/// [`RangesIter`]: crate::RangesIter
/// [SortedDisjoint]: crate::SortedDisjoint.html#table-of-contents
///
/// ## Constructor Examples
/// ```
/// use range_set_blaze::prelude::*;
///
/// // RangeSetBlaze's .ranges() and .into_ranges()
/// let r = RangeSetBlaze::from_iter([3, 2, 1, 100, 1]);
/// let a = r.ranges();
/// assert!(a.into_string() == "1..=3, 100..=100");
/// // 'into_ranges' takes ownership of the 'RangeSetBlaze'
/// let a = RangeSetBlaze::from_iter([3, 2, 1, 100, 1]).into_ranges();
/// assert!(a.into_string() == "1..=3, 100..=100");
///
/// // CheckSortedDisjoint -- unsorted or overlapping input ranges will cause a panic.
/// let a = CheckSortedDisjoint::new([1..=3, 100..=100]);
/// assert!(a.into_string() == "1..=3, 100..=100");
/// ```
///
/// # `SortedDisjoint` Set Operations
///
/// You can perform set operations on `SortedDisjoint`s using operators.
///
/// | Set Operators                             | Operator    | Multiway (same type)                              | Multiway (different types)           |
/// |------------------------------------|-------------|---------------------------------------------------|--------------------------------------|
/// | [`union`]                      | [`a` &#124; `b`] | `[a, b, c].`[`union`][multiway_union]`() `        | [`union_dyn!`]`(a, b, c)`         |
/// | [`intersection`]               | [`a & b`]     | `[a, b, c].`[`intersection`][multiway_intersection]`() ` | [`intersection_dyn!`]`(a, b, c)`|
/// | [`difference`]                 | [`a - b`]     | *n/a*                                             | *n/a*                                |
/// | [`symmetric_difference`]       | [`a ^ b`]     | `[a, b, c].`[`symmetric_difference`][multiway_symmetric_difference]`() ` | [`symmetric_difference_dyn!`]`(a, b, c)` |
/// | [`complement`]                 | [`!a`]        | *n/a*                                             | *n/a*                                |
///
/// [`a` &#124; `b`]: trait.SortedDisjoint.html#method.union
/// [`a & b`]: trait.SortedDisjoint.html#method.intersection
/// [`a - b`]: trait.SortedDisjoint.html#method.difference
/// [`a ^ b`]: trait.SortedDisjoint.html#method.symmetric_difference
/// [`!a`]: trait.SortedDisjoint.html#method.complement
/// [multiway_union]: trait.MultiwaySortedDisjoint.html#method.union
/// [multiway_intersection]: trait.MultiwaySortedDisjoint.html#method.intersection
/// [multiway_symmetric_difference]: trait.MultiwaySortedDisjoint.html#method.symmetric_difference
/// [`union_dyn!`]: macro.union_dyn.html
/// [`intersection_dyn!`]: macro.intersection_dyn.html
/// [`symmetric_difference_dyn!`]: macro.symmetric_difference_dyn.html
///
/// ## Performance
///
/// Every operation is implemented as a single pass over the sorted & disjoint ranges, with minimal memory.
///
/// This is true even when applying multiple operations. The last example below demonstrates this.
///
/// ## Examples
///
/// ```
/// use range_set_blaze::prelude::*;
///
/// let a0 = RangeSetBlaze::from_iter([1..=2, 5..=100]);
/// let b0 = RangeSetBlaze::from_iter([2..=6]);
///
/// // 'union' method and 'to_string' method
/// let (a, b) = (a0.ranges(), b0.ranges());
/// let result = a.union(b);
/// assert_eq!(result.into_string(), "1..=100");
///
/// // '|' operator and 'equal' method
/// let (a, b) = (a0.ranges(), b0.ranges());
/// let result = a | b;
/// assert!(result.equal(CheckSortedDisjoint::new([1..=100])));
///
/// // multiway union of same type
/// let c0 = RangeSetBlaze::from_iter([2..=2, 6..=200]);
/// let (a, b, c) = (a0.ranges(), b0.ranges(), c0.ranges());
/// let result = [a, b, c].union();
/// assert_eq!(result.into_string(), "1..=200");
///
/// // multiway union of different types
/// let (a, b, c) = (a0.ranges(), b0.ranges(), c0.ranges());
/// let result = union_dyn!(a, b, !c);
/// assert_eq!(result.into_string(), "-2147483648..=100, 201..=2147483647");
///
/// // Applying multiple operators makes only one pass through the inputs with minimal memory.
/// let (a, b, c) = (a0.ranges(), b0.ranges(), c0.ranges());
/// let result = a - (b | c);
/// assert!(result.into_string() == "1..=1");
/// ```
///
/// # How to mark your type as `SortedDisjoint`
///
/// To mark your iterator type as `SortedDisjoint`, you implement the `SortedStarts` and `SortedDisjoint` traits.
/// This is your promise to the compiler that your iterator will provide inclusive ranges that are
/// disjoint and sorted by start.
///
/// When you do this, your iterator will get access to the
/// efficient set operations methods, such as [`intersection`] and [`complement`]. The example below shows this.
///
/// > To use operators such as `&` and `!`, you must also implement the [`BitAnd`], [`Not`], etc. traits.
/// >
/// > If you want others to use your marked iterator type, reexport:
/// > `pub use range_set_blaze::{SortedDisjoint, SortedStarts};`
///
/// [`BitAnd`]: core::ops::BitAnd
/// [`Not`]: core::ops::Not
/// [`intersection`]: SortedDisjoint::intersection
/// [`complement`]: SortedDisjoint::complement
/// [`union`]: SortedDisjoint::union
/// [`symmetric_difference`]: SortedDisjoint::symmetric_difference
/// [`difference`]: SortedDisjoint::difference
/// [`to_string`]: SortedDisjoint::to_string
/// [`equal`]: SortedDisjoint::equal
/// [multiway_union]: crate::MultiwaySortedDisjoint::union
/// [multiway_intersection]: crate::MultiwaySortedDisjoint::intersection
///
/// ## Example -- Find the ordinal weekdays in September 2023
/// ```
/// use core::ops::RangeInclusive;
/// use core::iter::FusedIterator;
/// pub use range_set_blaze::{SortedDisjoint, SortedStarts};
///
/// // Ordinal dates count January 1 as day 1, February 1 as day 32, etc.
/// struct OrdinalWeekends2023 {
///     next_range: RangeInclusive<i32>,
/// }
///
/// // We promise the compiler that our iterator will provide
/// // ranges that are sorted and disjoint.
/// impl FusedIterator for OrdinalWeekends2023 {}
/// impl SortedStarts<i32> for OrdinalWeekends2023 {}
/// impl SortedDisjoint<i32> for OrdinalWeekends2023 {}
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
/// use range_set_blaze::prelude::*;
///
/// let weekends = OrdinalWeekends2023::new();
/// let september = CheckSortedDisjoint::new([244..=273]);
/// let september_weekdays = september.intersection(weekends.complement());
/// assert_eq!(
///     september_weekdays.into_string(),
///     "244..=244, 247..=251, 254..=258, 261..=265, 268..=272"
/// );
/// ```
pub trait SortedDisjoint<T: Integer>: SortedStarts<T> {
    // I think this is 'Sized' because will sometimes want to create a struct (e.g. BitOrIter) that contains a field of this type

    /// Given two [`SortedDisjoint`] iterators, efficiently returns a [`SortedDisjoint`] iterator of their union.
    ///
    /// [SortedDisjoint]: crate::SortedDisjoint.html#table-of-contents
    ///
    /// # Examples
    ///
    /// ```
    /// use range_set_blaze::prelude::*;
    ///
    /// let a = CheckSortedDisjoint::new([1..=1]);
    /// let b = RangeSetBlaze::from_iter([2..=2]).into_ranges();
    /// let union = a.union(b);
    /// assert_eq!(union.into_string(), "1..=2");
    ///
    /// // Alternatively, we can use "|" because CheckSortedDisjoint defines
    /// // ops::bitor as SortedDisjoint::union.
    /// let a = CheckSortedDisjoint::new([1..=1]);
    /// let b = RangeSetBlaze::from_iter([2..=2]).into_ranges();
    /// let union = a | b;
    /// assert_eq!(union.into_string(), "1..=2");
    /// ```
    #[inline]
    fn union<R>(self, other: R) -> UnionMerge<T, Self, R::IntoIter>
    where
        R: IntoIterator<Item = Self::Item>,
        R::IntoIter: SortedDisjoint<T>,
        Self: Sized,
    {
        UnionMerge::new2(self, other.into_iter())
    }

    /// Given two [`SortedDisjoint`] iterators, efficiently returns a [`SortedDisjoint`] iterator of their intersection.
    ///
    /// [SortedDisjoint]: crate::SortedDisjoint.html#table-of-contents
    ///
    /// # Examples
    ///
    /// ```
    /// use range_set_blaze::prelude::*;
    ///
    /// let a = CheckSortedDisjoint::new([1..=2]);
    /// let b = RangeSetBlaze::from_iter([2..=3]).into_ranges();
    /// let intersection = a.intersection(b);
    /// assert_eq!(intersection.into_string(), "2..=2");
    ///
    /// // Alternatively, we can use "&" because CheckSortedDisjoint defines
    /// // ops::bitand as SortedDisjoint::intersection.
    /// let a = CheckSortedDisjoint::new([1..=2]);
    /// let b = RangeSetBlaze::from_iter([2..=3]).into_ranges();
    /// let intersection = a & b;
    /// assert_eq!(intersection.into_string(), "2..=2");
    /// ```
    #[inline]
    fn intersection<R>(self, other: R) -> IntersectionMerge<T, Self, R::IntoIter>
    where
        R: IntoIterator<Item = Self::Item>,
        R::IntoIter: SortedDisjoint<T>,
        Self: Sized,
    {
        !(self.complement() | other.into_iter().complement())
    }

    /// Given two [`SortedDisjoint`] iterators, efficiently returns a [`SortedDisjoint`] iterator of their set difference.
    ///
    /// [SortedDisjoint]: crate::SortedDisjoint.html#table-of-contents
    ///
    /// # Examples
    ///
    /// ```
    /// use range_set_blaze::prelude::*;
    ///
    /// let a = CheckSortedDisjoint::new([1..=2]);
    /// let b = RangeSetBlaze::from_iter([2..=3]).into_ranges();
    /// let difference = a.difference(b);
    /// assert_eq!(difference.into_string(), "1..=1");
    ///
    /// // Alternatively, we can use "-" because CheckSortedDisjoint defines
    /// // ops::sub as SortedDisjoint::difference.
    /// let a = CheckSortedDisjoint::new([1..=2]);
    /// let b = RangeSetBlaze::from_iter([2..=3]).into_ranges();
    /// let difference = a - b;
    /// assert_eq!(difference.into_string(), "1..=1");
    /// ```
    #[inline]
    fn difference<R>(self, other: R) -> DifferenceMerge<T, Self, R::IntoIter>
    where
        R: IntoIterator<Item = Self::Item>,
        R::IntoIter: SortedDisjoint<T>,
        Self: Sized,
    {
        !(self.complement() | other.into_iter())
    }

    /// Given a [`SortedDisjoint`] iterator, efficiently returns a [`SortedDisjoint`] iterator of its complement.
    ///
    /// [SortedDisjoint]: crate::SortedDisjoint.html#table-of-contents
    ///
    /// # Examples
    ///
    /// ```
    /// use range_set_blaze::prelude::*;
    ///
    /// let a = CheckSortedDisjoint::new([10_u8..=20, 100..=200]);
    /// let complement = a.complement();
    /// assert_eq!(complement.into_string(), "0..=9, 21..=99, 201..=255");
    ///
    /// // Alternatively, we can use "!" because CheckSortedDisjoint defines
    /// // `ops::Not` as `SortedDisjoint::complement`.
    /// let a = CheckSortedDisjoint::new([10_u8..=20, 100..=200]);
    /// let complement = !a;
    /// assert_eq!(complement.into_string(), "0..=9, 21..=99, 201..=255");
    /// ```
    #[inline]
    fn complement(self) -> NotIter<T, Self>
    where
        Self: Sized,
    {
        NotIter::new(self)
    }

    /// Given two [`SortedDisjoint`] iterators, efficiently returns a [`SortedDisjoint`] iterator
    /// of their symmetric difference.
    ///
    /// [SortedDisjoint]: crate::SortedDisjoint.html#table-of-contents
    /// # Examples
    ///
    /// ```
    /// use range_set_blaze::prelude::*;
    ///
    /// let a = CheckSortedDisjoint::new([1..=2]);
    /// let b = RangeSetBlaze::from_iter([2..=3]).into_ranges();
    /// let symmetric_difference = a.symmetric_difference(b);
    /// assert_eq!(symmetric_difference.into_string(), "1..=1, 3..=3");
    ///
    /// // Alternatively, we can use "^" because CheckSortedDisjoint defines
    /// // ops::bitxor as SortedDisjoint::symmetric_difference.
    /// let a = CheckSortedDisjoint::new([1..=2]);
    /// let b = RangeSetBlaze::from_iter([2..=3]).into_ranges();
    /// let symmetric_difference = a ^ b;
    /// assert_eq!(symmetric_difference.into_string(), "1..=1, 3..=3");
    /// ```
    #[inline]
    fn symmetric_difference<R>(self, other: R) -> SymDiffMerge<T, Self, R::IntoIter>
    where
        R: IntoIterator<Item = Self::Item>,
        R::IntoIter: SortedDisjoint<T>,
        <R as IntoIterator>::IntoIter:,
        Self: Sized,
    {
        let result: SymDiffIter<T, crate::Merge<T, Self, <R as IntoIterator>::IntoIter>> =
            SymDiffIter::new2(self, other.into_iter());
        result
    }

    /// Given two [`SortedDisjoint`] iterators, efficiently tells if they are equal. Unlike most equality testing in Rust,
    /// this method takes ownership of the iterators and consumes them.
    ///
    /// [SortedDisjoint]: crate::SortedDisjoint.html#table-of-contents
    ///
    /// # Examples
    ///
    /// ```
    /// use range_set_blaze::prelude::*;
    ///
    /// let a = CheckSortedDisjoint::new([1..=2]);
    /// let b = RangeSetBlaze::from_iter([1..=2]).into_ranges();
    /// assert!(a.equal(b));
    /// ```
    fn equal<R>(self, other: R) -> bool
    where
        R: IntoIterator<Item = Self::Item>,
        R::IntoIter: SortedDisjoint<T>,
        Self: Sized,
    {
        itertools::equal(self, other)
    }

    /// Deprecated. Use [`into_string`] instead.
    ///
    /// [`into_string`]: trait.IntoString.html
    #[deprecated(since = "0.2.0", note = "Use `into_string` instead")]
    fn to_string(self) -> String
    where
        Self: Sized,
    {
        self.into_string()
    }

    /// Returns `true` if the set contains no elements.
    ///
    /// # Examples
    ///
    /// ```
    /// use range_set_blaze::prelude::*;
    ///
    /// let a = CheckSortedDisjoint::new([1..=2]);
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

    /// Returns `true` if the set contains all possible integers.
    ///
    /// For type `T`, this means exactly one range spanning `T::min_value()`..=`T::max_value()`.
    /// Complexity: O(1) on the first item.
    ///
    /// # Examples
    ///
    /// ```
    /// use range_set_blaze::prelude::*;
    ///
    /// let a = CheckSortedDisjoint::new([1_u8..=2]);
    /// assert!(!a.is_universal());
    ///
    /// let universal = CheckSortedDisjoint::new([0_u8..=255]);
    /// assert!(universal.is_universal());
    /// ```
    #[inline]
    #[allow(clippy::wrong_self_convention)]
    fn is_universal(mut self) -> bool
    where
        Self: Sized,
    {
        self.next().is_some_and(|range| {
            let (start, end) = range.into_inner();
            start == T::min_value() && end == T::max_value()
        })
    }

    /// Returns `true` if the set is a subset of another,
    /// i.e., `other` contains at least all the elements in `self`.
    ///
    /// # Examples
    ///
    /// ```
    /// use range_set_blaze::prelude::*;
    ///
    /// let sup = CheckSortedDisjoint::new([1..=3]);
    /// let set: CheckSortedDisjoint<i32, _> = [].into();
    /// assert_eq!(set.is_subset(sup), true);
    ///
    /// let sup = CheckSortedDisjoint::new([1..=3]);
    /// let set = CheckSortedDisjoint::new([2..=2]);
    /// assert_eq!(set.is_subset(sup), true);
    ///
    /// let sup = CheckSortedDisjoint::new([1..=3]);
    /// let set = CheckSortedDisjoint::new([2..=2, 4..=4]);
    /// assert_eq!(set.is_subset(sup), false);
    /// ```
    #[must_use]
    #[inline]
    #[allow(clippy::wrong_self_convention)]
    fn is_subset<R>(self, other: R) -> bool
    where
        R: IntoIterator<Item = Self::Item>,
        R::IntoIter: SortedDisjoint<T>,
        Self: Sized,
    {
        // LATER: Could be made a little more efficient by coding the logic directly into the iterators.
        self.difference(other).is_empty()
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
    #[inline]
    #[must_use]
    #[allow(clippy::wrong_self_convention)]
    fn is_superset<R>(self, other: R) -> bool
    where
        R: IntoIterator<Item = Self::Item>,
        R::IntoIter: SortedDisjoint<T>,
        Self: Sized,
    {
        other.into_iter().is_subset(self)
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
    #[allow(clippy::wrong_self_convention)]
    fn is_disjoint<R>(self, other: R) -> bool
    where
        R: IntoIterator<Item = Self::Item>,
        R::IntoIter: SortedDisjoint<T>,
        Self: Sized,
    {
        self.intersection(other).is_empty()
    }

    /// Create a [`RangeSetBlaze`] from a [`SortedDisjoint`] iterator.
    ///
    /// *For more about constructors and performance, see [`RangeSetBlaze` Constructors](struct.RangeSetBlaze.html#rangesetblaze-constructors).*
    ///
    /// [SortedDisjoint]: crate::SortedDisjoint.html#table-of-contents
    ///
    /// # Examples
    ///
    /// ```
    /// use range_set_blaze::prelude::*;
    ///
    /// let a0 = RangeSetBlaze::from_sorted_disjoint(CheckSortedDisjoint::new([-10..=-5, 1..=2]));
    /// let a1: RangeSetBlaze<i32> = CheckSortedDisjoint::new([-10..=-5, 1..=2]).into_range_set_blaze();
    /// assert!(a0 == a1 && a0.to_string() == "-10..=-5, 1..=2");
    /// ```
    fn into_range_set_blaze(self) -> RangeSetBlaze<T>
    where
        Self: Sized,
    {
        RangeSetBlaze::from_sorted_disjoint(self)
    }
}

/// Gives the [`SortedDisjoint`] trait to any iterator of ranges. The iterator will panic
/// if/when it finds that the ranges are not actually sorted and disjoint.
///
/// [SortedDisjoint]: crate::SortedDisjoint.html#table-of-contents
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
/// let a = CheckSortedDisjoint::new([1..=2, 5..=100]);
/// let b = CheckSortedDisjoint::new([2..=6]);
/// let union = a | b;
/// assert_eq!(union.into_string(), "1..=100");
/// ```
///
/// Here the ranges are not sorted and disjoint, so the iterator will panic.
///```should_panic
/// use range_set_blaze::prelude::*;
///
/// let a = CheckSortedDisjoint::new([1..=2, 5..=100]);
/// let b = CheckSortedDisjoint::new([2..=6,-10..=-5]);
/// let union = a | b;
/// assert_eq!(union.into_string(), "1..=100");
/// ```
#[derive(Debug, Clone)]
#[must_use = "iterators are lazy and do nothing unless consumed"]
#[allow(clippy::module_name_repetitions)]
pub struct CheckSortedDisjoint<T, I>
where
    T: Integer,
    I: Iterator<Item = RangeInclusive<T>> + FusedIterator,
{
    pub(crate) iter: I,
    prev_end: Option<T>,
    seen_none: bool,
}

impl<T, I> CheckSortedDisjoint<T, I>
where
    T: Integer,
    I: Iterator<Item = RangeInclusive<T>> + FusedIterator,
{
    /// Creates a new [`CheckSortedDisjoint`] from an iterator of ranges. See [`CheckSortedDisjoint`] for details and examples.
    #[inline]
    pub fn new<J: IntoIterator<IntoIter = I>>(iter: J) -> Self {
        Self {
            iter: iter.into_iter(),
            prev_end: None,
            seen_none: false,
        }
    }
}

impl<T> Default for CheckSortedDisjoint<T, array::IntoIter<RangeInclusive<T>, 0>>
where
    T: Integer,
{
    // Default is an empty iterator.
    fn default() -> Self {
        Self::new([])
    }
}

impl<T, I> FusedIterator for CheckSortedDisjoint<T, I>
where
    T: Integer,
    I: Iterator<Item = RangeInclusive<T>> + FusedIterator,
{
}

impl<T, I> Iterator for CheckSortedDisjoint<T, I>
where
    T: Integer,
    I: Iterator<Item = RangeInclusive<T>> + FusedIterator,
{
    type Item = RangeInclusive<T>;

    fn next(&mut self) -> Option<Self::Item> {
        let next = self.iter.next();

        let Some(range) = next.as_ref() else {
            self.seen_none = true;
            return next;
        };

        assert!(
            !self.seen_none,
            "iterator cannot return Some after returning None"
        );
        let (start, end) = range.clone().into_inner();
        assert!(start <= end, "start must be less or equal to end");
        if let Some(prev_end) = self.prev_end {
            assert!(
                prev_end < T::max_value() && prev_end.add_one() < start,
                "ranges must be disjoint"
            );
        }
        self.prev_end = Some(end);

        next
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

impl<T: Integer, const N: usize> From<[RangeInclusive<T>; N]>
    for CheckSortedDisjoint<T, array::IntoIter<RangeInclusive<T>, N>>
{
    /// Deprecated: Use `new` instead.
    fn from(arr: [RangeInclusive<T>; N]) -> Self {
        Self::new(arr)
    }
}

pub trait AnythingGoes<T: Integer>: Iterator<Item = RangeInclusive<T>> + FusedIterator {}
impl<T: Integer, I> AnythingGoes<T> for I where I: Iterator<Item = RangeInclusive<T>> + FusedIterator
{}

/// `RangeOnce` is analogous to [`core::iter::Once`], but modified to treat an
/// empty [`RangeInclusive`] as an empty [`Iterator`]. This allows `RangeOnce`
/// to be safely used as a [`SortedDisjoint`] Iterator.
///
/// # Example
///
/// ```
/// use range_set_blaze::{ RangeSetBlaze, RangeOnce };
///
/// let a = RangeOnce::new(0..=10);
/// let b = RangeOnce::new(3..=2); // empty range
/// let c = RangeOnce::new(5..=15);
///
/// let combined = RangeSetBlaze::from_sorted_disjoint(a | b | c);
/// assert_eq!(combined.into_string(), "0..=15");
/// ```
pub struct RangeOnce<T>(std::option::IntoIter<RangeInclusive<T>>);

impl<T: Integer> RangeOnce<T> {
    /// Creates a new [`RangeOnce`] from a single range. See [`RangeOnce`] for details and examples.
    pub fn new(range: RangeInclusive<T>) -> Self {
        Self((!range.is_empty()).then_some(range).into_iter())
    }
}

impl<T: Integer> From<RangeInclusive<T>> for RangeOnce<T> {
    #[inline]
    fn from(value: RangeInclusive<T>) -> Self {
        Self::new(value)
    }
}

impl<T: Integer> Iterator for RangeOnce<T> {
    type Item = RangeInclusive<T>;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
}

impl<T: Integer> DoubleEndedIterator for RangeOnce<T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.0.next_back()
    }
}

impl<T: Integer> ExactSizeIterator for RangeOnce<T> {
    fn len(&self) -> usize {
        self.0.len()
    }
}

impl<T: Integer> FusedIterator for RangeOnce<T> {}

macro_rules! impl_sorted_traits_and_ops {
    ($IterType:ty, $($more_generics:tt)*) => {
        #[allow(single_use_lifetimes)]
        impl<$($more_generics)*, T: Integer> SortedStarts<T> for $IterType {}
        #[allow(single_use_lifetimes)]
        impl<$($more_generics)*, T: Integer> SortedDisjoint<T> for $IterType {}

        #[allow(single_use_lifetimes)]
        impl<$($more_generics)*, T: Integer> ops::Not for $IterType
        {
            type Output = NotIter<T, Self>;

            fn not(self) -> Self::Output {
                self.complement()
            }
        }

        #[allow(single_use_lifetimes)]
        impl<$($more_generics)*, T: Integer, R> ops::BitOr<R> for $IterType
        where
            R: SortedDisjoint<T>,
        {
            type Output = UnionMerge<T, Self, R>;

            fn bitor(self, other: R) -> Self::Output {
                SortedDisjoint::union(self, other)
            }
        }

        #[allow(single_use_lifetimes)]
        impl<$($more_generics)*, T: Integer, R> ops::Sub<R> for $IterType
        where
            R: SortedDisjoint<T>,
        {
            type Output = DifferenceMerge<T, Self, R>;

            fn sub(self, other: R) -> Self::Output {
                // It would be fun to optimize !!self.iter into self.iter
                // but that would require also considering fields 'start_not' and 'next_time_return_none'.
                SortedDisjoint::difference(self, other)
            }
        }

        #[allow(single_use_lifetimes)]
        impl<$($more_generics)*, T: Integer, R> ops::BitXor<R> for $IterType
        where
            R: SortedDisjoint<T>,
        {
            type Output = SymDiffMerge<T, Self, R>;

            #[allow(clippy::suspicious_arithmetic_impl)]
            fn bitxor(self, other: R) -> Self::Output {
                SortedDisjoint::symmetric_difference(self, other)
            }
        }

        #[allow(single_use_lifetimes)]
        impl<$($more_generics)*, T: Integer, R> ops::BitAnd<R> for $IterType
        where
            R: SortedDisjoint<T>,
        {
            type Output = IntersectionMerge<T, Self, R>;

            fn bitand(self, other: R) -> Self::Output {
                SortedDisjoint::intersection(self, other)
            }
        }
    };
}

//CheckList: Be sure that these are all tested in 'test_every_sorted_disjoint_method'
impl_sorted_traits_and_ops!(CheckSortedDisjoint<T, I>, I: AnythingGoes<T>);
impl_sorted_traits_and_ops!(DynSortedDisjoint<'a, T>, 'a);
impl_sorted_traits_and_ops!(IntoRangesIter<T>, 'ignore);
impl_sorted_traits_and_ops!(MapIntoRangesIter<T, V>, V: Eq + Clone);
impl_sorted_traits_and_ops!(MapRangesIter<'a, T, V>, 'a, V: Eq + Clone);
impl_sorted_traits_and_ops!(NotIter<T, I>, I: SortedDisjoint<T>);
impl_sorted_traits_and_ops!(RangesIter<'a, T>, 'a);
impl_sorted_traits_and_ops!(RangeValuesToRangesIter<T, VR, I>, VR: ValueRef, I: SortedDisjointMap<T, VR>);
impl_sorted_traits_and_ops!(SymDiffIter<T, I>, I: SortedStarts<T>);
impl_sorted_traits_and_ops!(UnionIter<T, I>, I: SortedStarts<T>);
impl_sorted_traits_and_ops!(RangeOnce<T>, 'ignore);
