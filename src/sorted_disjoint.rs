use core::{
    iter::FusedIterator,
    ops::{self, RangeInclusive},
};

use alloc::{format, string::String};
use itertools::Itertools;

use crate::{
    BitAndMerge, BitOrMerge, BitSubMerge, BitXOrTee, Integer, Merge, NotIter, RangeSetBlaze,
    UnionIter,
};

/// Internally, a trait used to mark iterators that provide ranges sorted by start, but not necessarily by end,
/// and may overlap.
#[doc(hidden)]
pub trait SortedStarts<T: Integer>: Iterator<Item = RangeInclusive<T>> {}

/// The trait used to mark iterators that provide ranges that are sorted by start and disjoint. Set operations on
/// iterators that implement this trait can be performed in linear time.
///
/// # Table of Contents
/// * [`SortedDisjoint` Constructors](#sorteddisjoint-constructors)
///   * [Examples](#constructor-examples)
/// * [`SortedDisjoint` Set and Other Operations](#sorteddisjoint-set-and-other-operations)
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
/// | [`RangeSetBlaze`]'s [`RangesIter`] | [`clone`] |
/// | sorted & disjoint ranges | [`CheckSortedDisjoint::new`] |
/// | `SortedDisjoint` iterator | [itertools `tee`] |
/// | `SortedDisjoint` iterator | [`crate::dyn_sorted_disjoint::DynSortedDisjoint::new`] |
/// |  *your iterator type* | *[How to mark your type as `SortedDisjoint`][1]* |
///
/// [`ranges`]: RangeSetBlaze::ranges
/// [`into_ranges`]: RangeSetBlaze::into_ranges
/// [`clone`]: crate::RangesIter::clone
/// [itertools `tee`]: https://docs.rs/itertools/latest/itertools/trait.Itertools.html#method.tee
/// [1]: #how-to-mark-your-type-as-sorteddisjoint
/// [`RangesIter`]: crate::RangesIter
///
/// ## Constructor Examples
///
/// ```
/// use range_set_blaze::prelude::*;
/// use itertools::Itertools;
///
/// // RangeSetBlaze's .ranges(), .range().clone() and .into_ranges()
/// let r = RangeSetBlaze::from_iter([3, 2, 1, 100, 1]);
/// let a = r.ranges();
/// let b = a.clone();
/// assert!(a.to_string() == "1..=3, 100..=100");
/// assert!(b.to_string() == "1..=3, 100..=100");
/// //    'into_ranges' takes ownership of the 'RangeSetBlaze'
/// let a = RangeSetBlaze::from_iter([3, 2, 1, 100, 1]).into_ranges();
/// assert!(a.to_string() == "1..=3, 100..=100");
///
/// // CheckSortedDisjoint -- unsorted or overlapping input ranges will cause a panic.
/// let a = CheckSortedDisjoint::from([1..=3, 100..=100]);
/// assert!(a.to_string() == "1..=3, 100..=100");
///
/// // tee of a SortedDisjoint iterator
/// let a = CheckSortedDisjoint::from([1..=3, 100..=100]);
/// let (a, b) = a.tee();
/// assert!(a.to_string() == "1..=3, 100..=100");
/// assert!(b.to_string() == "1..=3, 100..=100");
///
/// // DynamicSortedDisjoint of a SortedDisjoint iterator
/// let a = CheckSortedDisjoint::from([1..=3, 100..=100]);
/// let b = DynSortedDisjoint::new(a);
/// assert!(b.to_string() == "1..=3, 100..=100");
/// ```
///
/// # `SortedDisjoint` Set Operations
///
/// | Method | Operator | Multiway (same type) | Multiway (different types) |
/// |--------|----------|----------------------|----------------------------|
/// | `a.`[`union`]`(b)` | `a` &#124; `b` | `[a, b, c].`[`union`][crate::MultiwaySortedDisjoint::union]`()` | [`crate::MultiwayRangeSetBlaze::union`]`!(a, b, c)` |
/// | `a.`[`intersection`]`(b)` | `a & b` | `[a, b, c].`[`intersection`][crate::MultiwaySortedDisjoint::intersection]`()` | [`crate::MultiwayRangeSetBlaze::intersection`]`!(a, b, c)` |
/// | `a.`[`difference`]`(b)` | `a - b` |  |  |
/// | `a.`[`symmetric_difference`]`(b)` | `a ^ b` |  |  |
/// | `a.`[`complement`]`()` | `!a` |  |  |
///
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
/// let c0 = RangeSetBlaze::from_iter([2..=2, 6..=200]);
///
/// // 'union' method and 'to_string' method
/// let (a, b) = (a0.ranges(), b0.ranges());
/// let result = a.union(b);
/// assert_eq!(result.to_string(), "1..=100");
///
/// // '|' operator and 'equal' method
/// let (a, b) = (a0.ranges(), b0.ranges());
/// let result = a | b;
/// assert!(result.equal(CheckSortedDisjoint::from([1..=100])));
///
/// // multiway union of same type
/// let (a, b, c) = (a0.ranges(), b0.ranges(), c0.ranges());
/// let result = [a, b, c].union();
/// assert_eq!(result.to_string(), "1..=200");
///
/// // multiway union of different types
/// let (a, b, c) = (a0.ranges(), b0.ranges(), c0.ranges());
/// let result = union_dyn!(a, b, !c);
/// assert_eq!(result.to_string(), "-2147483648..=100, 201..=2147483647");
///
/// // Applying multiple operators makes only one pass through the inputs with minimal memory.
/// let (a, b, c) = (a0.ranges(), b0.ranges(), c0.ranges());
/// let result = a - (b | c);
/// assert!(result.to_string() == "1..=1");
/// ```
///
/// # How to mark your type as `SortedDisjoint`
///
/// To mark your iterator type as `SortedDisjoint`, you implement the `SortedStarts` and `SortedDisjoint` traits.
/// This is your promise to the compiler that your iterator will provide inclusive ranges that disjoint and sorted by start.
///
/// When you do this, your iterator will get access to the
/// efficient set operations methods, such as [`intersection`] and [`complement`]. The example below shows this.
///
/// > To use operators such as `&` and `!`, you must also implement the [`BitAnd`], [`Not`], etc. traits.
/// >
/// > If you want others to use your marked iterator type, reexport:
/// > `pub use range_set_blaze::{SortedDisjoint, SortedStarts};`
///
/// [`BitAnd`]: https://doc.rust-lang.org/std/ops/trait.BitAnd.html
/// [`Not`]: https://doc.rust-lang.org/std/ops/trait.Not.html
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
/// pub use range_set_blaze::{SortedDisjoint, SortedStarts};
///
/// // Ordinal dates count January 1 as day 1, February 1 as day 32, etc.
/// struct OrdinalWeekends2023 {
///     next_range: RangeInclusive<i32>,
/// }
///
/// // We promise the compiler that our iterator will provide
/// // ranges that are sorted and disjoint.
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
/// let september = CheckSortedDisjoint::from([244..=273]);
/// let september_weekdays = september.intersection(weekends.complement());
/// assert_eq!(
///     september_weekdays.to_string(),
///     "244..=244, 247..=251, 254..=258, 261..=265, 268..=272"
/// );
/// ```
pub trait SortedDisjoint<T: Integer>: SortedStarts<T> {
    // I think this is 'Sized' because will sometimes want to create a struct (e.g. BitOrIter) that contains a field of this type

    /// Given two [`SortedDisjoint`] iterators, efficiently returns a [`SortedDisjoint`] iterator of their union.
    ///
    /// # Examples
    ///
    /// ```
    /// use range_set_blaze::prelude::*;
    ///
    /// let a = CheckSortedDisjoint::from([1..=1]);
    /// let b = RangeSetBlaze::from_iter([2..=2]).into_ranges();
    /// let union = a.union(b);
    /// assert_eq!(union.to_string(), "1..=2");
    ///
    /// // Alternatively, we can use "|" because CheckSortedDisjoint defines
    /// // ops::bitor as SortedDisjoint::union.
    /// let a = CheckSortedDisjoint::from([1..=1]);
    /// let b = RangeSetBlaze::from_iter([2..=2]).into_ranges();
    /// let union = a | b;
    /// assert_eq!(union.to_string(), "1..=2");
    /// ```
    #[inline]
    fn union<R>(self, other: R) -> BitOrMerge<T, Self, R::IntoIter>
    where
        R: IntoIterator<Item = Self::Item>,
        R::IntoIter: SortedDisjoint<T>,
        Self: Sized,
    {
        UnionIter::new(Merge::new(self, other.into_iter()))
    }

    /// Given two [`SortedDisjoint`] iterators, efficiently returns a [`SortedDisjoint`] iterator of their intersection.
    ///
    /// # Examples
    ///
    /// ```
    /// use range_set_blaze::prelude::*;
    ///
    /// let a = CheckSortedDisjoint::from([1..=2]);
    /// let b = RangeSetBlaze::from_iter([2..=3]).into_ranges();
    /// let intersection = a.intersection(b);
    /// assert_eq!(intersection.to_string(), "2..=2");
    ///
    /// // Alternatively, we can use "&" because CheckSortedDisjoint defines
    /// // ops::bitand as SortedDisjoint::intersection.
    /// let a = CheckSortedDisjoint::from([1..=2]);
    /// let b = RangeSetBlaze::from_iter([2..=3]).into_ranges();
    /// let intersection = a & b;
    /// assert_eq!(intersection.to_string(), "2..=2");
    /// ```
    #[inline]
    fn intersection<R>(self, other: R) -> BitAndMerge<T, Self, R::IntoIter>
    where
        R: IntoIterator<Item = Self::Item>,
        R::IntoIter: SortedDisjoint<T>,
        Self: Sized,
    {
        !(self.complement() | other.into_iter().complement())
    }

    /// Given two [`SortedDisjoint`] iterators, efficiently returns a [`SortedDisjoint`] iterator of their set difference.
    ///
    /// # Examples
    ///
    /// ```
    /// use range_set_blaze::prelude::*;
    ///
    /// let a = CheckSortedDisjoint::from([1..=2]);
    /// let b = RangeSetBlaze::from_iter([2..=3]).into_ranges();
    /// let difference = a.difference(b);
    /// assert_eq!(difference.to_string(), "1..=1");
    ///
    /// // Alternatively, we can use "-" because CheckSortedDisjoint defines
    /// // ops::sub as SortedDisjoint::difference.
    /// let a = CheckSortedDisjoint::from([1..=2]);
    /// let b = RangeSetBlaze::from_iter([2..=3]).into_ranges();
    /// let difference = a - b;
    /// assert_eq!(difference.to_string(), "1..=1");
    /// ```
    #[inline]
    fn difference<R>(self, other: R) -> BitSubMerge<T, Self, R::IntoIter>
    where
        R: IntoIterator<Item = Self::Item>,
        R::IntoIter: SortedDisjoint<T>,
        Self: Sized,
    {
        !(self.complement() | other.into_iter())
    }

    /// Given a [`SortedDisjoint`] iterator, efficiently returns a [`SortedDisjoint`] iterator of its complement.
    ///
    /// # Examples
    ///
    /// ```
    /// use range_set_blaze::prelude::*;
    ///
    /// let a = CheckSortedDisjoint::from([-10i16..=0, 1000..=2000]);
    /// let complement = a.complement();
    /// assert_eq!(complement.to_string(), "-32768..=-11, 1..=999, 2001..=32767");
    ///
    /// // Alternatively, we can use "!" because CheckSortedDisjoint defines
    /// // ops::not as SortedDisjoint::complement.
    /// let a = CheckSortedDisjoint::from([-10i16..=0, 1000..=2000]);
    /// let complement = !a;
    /// assert_eq!(complement.to_string(), "-32768..=-11, 1..=999, 2001..=32767");
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
    /// # Examples
    ///
    /// ```
    /// use range_set_blaze::prelude::*;
    ///
    /// let a = CheckSortedDisjoint::from([1..=2]);
    /// let b = RangeSetBlaze::from_iter([2..=3]).into_ranges();
    /// let symmetric_difference = a.symmetric_difference(b);
    /// assert_eq!(symmetric_difference.to_string(), "1..=1, 3..=3");
    ///
    /// // Alternatively, we can use "^" because CheckSortedDisjoint defines
    /// // ops::bitxor as SortedDisjoint::symmetric_difference.
    /// let a = CheckSortedDisjoint::from([1..=2]);
    /// let b = RangeSetBlaze::from_iter([2..=3]).into_ranges();
    /// let symmetric_difference = a ^ b;
    /// assert_eq!(symmetric_difference.to_string(), "1..=1, 3..=3");
    /// ```
    #[inline]
    fn symmetric_difference<R>(self, other: R) -> BitXOrTee<T, Self, R::IntoIter>
    where
        R: IntoIterator<Item = Self::Item>,
        R::IntoIter: SortedDisjoint<T>,
        Self: Sized,
    {
        let (lhs0, lhs1) = self.tee();
        let (rhs0, rhs1) = other.into_iter().tee();
        lhs0.difference(rhs0) | rhs1.difference(lhs1)
    }

    /// Given two [`SortedDisjoint`] iterators, efficiently tells if they are equal. Unlike most equality testing in Rust,
    /// this method takes ownership of the iterators and consumes them.
    ///
    /// # Examples
    ///
    /// ```
    /// use range_set_blaze::prelude::*;
    ///
    /// let a = CheckSortedDisjoint::from([1..=2]);
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

    /// Given a [`SortedDisjoint`] iterators, produces a string version. Unlike most `to_string` and `fmt` in Rust,
    /// this method takes ownership of the iterator and consumes it.
    ///
    /// # Examples
    ///
    /// ```
    /// use range_set_blaze::prelude::*;
    ///
    /// let a = CheckSortedDisjoint::from([1..=2]);
    /// assert_eq!(a.to_string(), "1..=2");
    /// ```
    fn to_string(self) -> String
    where
        Self: Sized,
    {
        self.map(|range| format!("{range:?}")).join(", ")
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
    #[inline]
    #[allow(clippy::wrong_self_convention)]
    fn is_empty(mut self) -> bool
    where
        Self: Sized,
    {
        self.next().is_none()
    }

    /// Returns `true` if the set is a subset of another,
    /// i.e., `other` contains at least all the elements in `self`.
    ///
    /// # Examples
    ///
    /// ```
    /// use range_set_blaze::prelude::*;
    ///
    /// let sup = CheckSortedDisjoint::from([1..=3]);
    /// let set: CheckSortedDisjoint<i32, _> = [].into();
    /// assert_eq!(set.is_subset(sup), true);
    ///
    /// let sup = CheckSortedDisjoint::from([1..=3]);
    /// let set = CheckSortedDisjoint::from([2..=2]);
    /// assert_eq!(set.is_subset(sup), true);
    ///
    /// let sup = CheckSortedDisjoint::from([1..=3]);
    /// let set = CheckSortedDisjoint::from([2..=2, 4..=4]);
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
    /// *For more about constructors and performance, see [`RangeSetBlaze` Constructors](struct.RangeSetBlaze.html#constructors).*
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
/// # Performance
///
/// All checking is done at runtime, but it should still be fast.
///
/// # Example
///
/// ```
/// use range_set_blaze::prelude::*;
///
/// let a = CheckSortedDisjoint::new(vec![1..=2, 5..=100].into_iter());
/// let b = CheckSortedDisjoint::from([2..=6]);
/// let union = a | b;
/// assert_eq!(union.to_string(), "1..=100");
/// ```
///
/// Here the ranges are not sorted and disjoint, so the iterator will panic.
///```should_panic
/// use range_set_blaze::prelude::*;
///
/// let a = CheckSortedDisjoint::new(vec![1..=2, 5..=100].into_iter());
/// let b = CheckSortedDisjoint::from([2..=6,-10..=-5]);
/// let union = a | b;
/// assert_eq!(union.to_string(), "1..=100");
/// ```
#[derive(Debug, Clone)]
#[must_use = "iterators are lazy and do nothing unless consumed"]
pub struct CheckSortedDisjoint<T, I>
where
    T: Integer,
    I: Iterator<Item = RangeInclusive<T>>,
{
    pub(crate) iter: I,
    prev_end: Option<T>,
    seen_none: bool,
}

impl<T: Integer, I> SortedDisjoint<T> for CheckSortedDisjoint<T, I> where
    I: Iterator<Item = RangeInclusive<T>>
{
}
impl<T: Integer, I> SortedStarts<T> for CheckSortedDisjoint<T, I> where
    I: Iterator<Item = RangeInclusive<T>>
{
}

impl<T, I> CheckSortedDisjoint<T, I>
where
    T: Integer,
    I: Iterator<Item = RangeInclusive<T>>,
{
    /// Creates a new [`CheckSortedDisjoint`] from an iterator of ranges. See [`CheckSortedDisjoint`] for details and examples.
    pub fn new(iter: I) -> Self {
        CheckSortedDisjoint {
            iter,
            prev_end: None,
            seen_none: false,
        }
    }
}

impl<T> Default for CheckSortedDisjoint<T, core::array::IntoIter<RangeInclusive<T>, 0>>
where
    T: Integer,
{
    // Default is an empty iterator.
    fn default() -> Self {
        Self::new([].into_iter())
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
    I: Iterator<Item = RangeInclusive<T>>,
{
    type Item = RangeInclusive<T>;

    fn next(&mut self) -> Option<Self::Item> {
        let next = self.iter.next();
        if let Some(range) = next.as_ref() {
            assert!(
                !self.seen_none,
                "iterator cannot return Some after returning None"
            );
            let (start, end) = range.clone().into_inner();
            assert!(start <= end, "start must be less or equal to end");
            assert!(
                end <= T::safe_max_value(),
                "end must be less than or equal to safe_max_value"
            );
            if let Some(prev_end) = self.prev_end {
                assert!(
                    prev_end < T::safe_max_value() && prev_end + T::one() < start,
                    "ranges must be disjoint"
                );
            }
            self.prev_end = Some(end);
        } else {
            self.seen_none = true;
        }
        next
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

impl<T: Integer, const N: usize> From<[RangeInclusive<T>; N]>
    for CheckSortedDisjoint<T, core::array::IntoIter<RangeInclusive<T>, N>>
{
    /// You may create a [`CheckSortedDisjoint`] from an array of integers.
    ///
    /// # Examples
    ///
    /// ```
    /// use range_set_blaze::prelude::*;
    ///
    /// let a0 = CheckSortedDisjoint::from([1..=3, 100..=100]);
    /// let a1: CheckSortedDisjoint<_,_> = [1..=3, 100..=100].into();
    /// assert_eq!(a0.to_string(), "1..=3, 100..=100");
    /// assert_eq!(a1.to_string(), "1..=3, 100..=100");
    /// ```
    fn from(arr: [RangeInclusive<T>; N]) -> Self {
        let iter = arr.into_iter();
        Self::new(iter)
    }
}

impl<T: Integer, I> ops::Not for CheckSortedDisjoint<T, I>
where
    I: Iterator<Item = RangeInclusive<T>>,
{
    type Output = NotIter<T, Self>;

    fn not(self) -> Self::Output {
        self.complement()
    }
}

impl<T: Integer, R, L> ops::BitOr<R> for CheckSortedDisjoint<T, L>
where
    L: Iterator<Item = RangeInclusive<T>>,
    R: SortedDisjoint<T>,
{
    type Output = BitOrMerge<T, Self, R>;

    fn bitor(self, other: R) -> Self::Output {
        SortedDisjoint::union(self, other)
    }
}

impl<T: Integer, R, L> ops::BitAnd<R> for CheckSortedDisjoint<T, L>
where
    L: Iterator<Item = RangeInclusive<T>>,
    R: SortedDisjoint<T>,
{
    type Output = BitAndMerge<T, Self, R>;

    fn bitand(self, other: R) -> Self::Output {
        SortedDisjoint::intersection(self, other)
    }
}

impl<T: Integer, R, L> ops::Sub<R> for CheckSortedDisjoint<T, L>
where
    L: Iterator<Item = RangeInclusive<T>>,
    R: SortedDisjoint<T>,
{
    type Output = BitSubMerge<T, Self, R>;

    fn sub(self, other: R) -> Self::Output {
        SortedDisjoint::difference(self, other)
    }
}

impl<T: Integer, R, L> ops::BitXor<R> for CheckSortedDisjoint<T, L>
where
    L: Iterator<Item = RangeInclusive<T>>,
    R: SortedDisjoint<T>,
{
    type Output = BitXOrTee<T, Self, R>;

    fn bitxor(self, other: R) -> Self::Output {
        SortedDisjoint::symmetric_difference(self, other)
    }
}
