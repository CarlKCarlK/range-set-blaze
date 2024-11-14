use crate::map::BitAndRangesMap2;
use crate::map::BitSubRangesMap;
use crate::map::BitSubRangesMap2;
use crate::map::ValueRef;
use crate::range_values::RangeValuesIter;
use crate::range_values::RangeValuesToRangesIter;
use crate::sym_diff_iter_map::SymDiffIterMap;
use crate::BitOrMapMerge;
use crate::BitXorMapMerge;
use crate::DynSortedDisjointMap;
use crate::IntoRangeValuesIter;
use alloc::format;
use alloc::rc::Rc;
use alloc::string::String;
use alloc::vec::Vec;
use core::cmp::Ordering;
use core::fmt::Debug;
use core::iter::FusedIterator;
use core::marker::PhantomData;
// use alloc::format;
// use alloc::string::String;
// use core::{
//     iter::FusedIterator,
//     ops::{self, RangeInclusive},
// };
use crate::intersection_iter_map::IntersectionIterMap;
use crate::map::BitAndRangesMap;
use crate::sorted_disjoint::SortedDisjoint;
use crate::NotIter;
use crate::{map::EqClone, union_iter_map::UnionIterMap, Integer, RangeMapBlaze};
use core::ops;
use core::ops::RangeInclusive;

/// Used internally. Marks iterators that provide `(range, value)` pairs that are sorted by the range's start, but
/// that are not necessarily disjoint.
pub trait SortedStartsMap<T, VR>: Iterator<Item = (RangeInclusive<T>, VR)> + FusedIterator
where
    T: Integer,
    VR: ValueRef,
{
}

/// Used internally by [`UnionIterMap`] and [`SymDiffIterMap`].
pub trait PrioritySortedStartsMap<T, VR>: Iterator<Item = Priority<T, VR>> + FusedIterator
where
    T: Integer,
    VR: ValueRef,
{
}

// cmk should there be a section name how to mark your type as SortedDisjointMap? with an example.

/// Marks iterators that provide `(range, value)` pairs that are sorted and disjoint. Set operations on
/// iterators that implement this trait can be performed in linear time.
///
/// # Table of Contents
/// * [`SortedDisjointMap` Constructors](#sorteddisjointmap-constructors)
///   * [Examples](#constructor-examples)
/// * [`SortedDisjointMap` Set Operations](#sorteddisjointmap-set-operations)
///   * [Performance](#performance)
///   * [Examples](#examples)
///
/// # `SortedDisjointMap` Constructors
///
/// You'll usually construct a `SortedDisjointMap` iterator from a [`RangeMapBlaze`] or a [`CheckSortedDisjointMap`].
/// Here is a summary table, followed by [examples](#constructor-examples). cmk not written You can also x define your own
/// `SortedDisjointMap`xx#how-to-mark-your-type-as-sorteddisjointmaps x.
///
/// | Input type | Method |
/// |------------|--------|
/// | [`RangeMapBlaze`] | [`ranges`] |
/// | [`RangeMapBlaze`] | [`into_ranges`] |
/// | [`RangeMapBlaze`]'s [`RangesIter`] | [`clone`] |
/// | sorted & disjoint ranges | [`CheckSortedDisjointMap::new`] |
/// | `SortedDisjointMap` iterator | [`crate::dyn_sorted_disjoint_map::DynSortedDisjointMap::new`] cmk looks bad|
/// |  *your iterator type* | *xHow to mark your type as `SortedDisjointMap`xx1x* |
///
/// [`SortedDisjointMap`]: trait.SortedDisjointMap.html#table-of-contents
/// [`ranges`]: RangeMapBlaze::ranges
/// [`into_ranges`]: RangeMapBlaze::into_ranges
/// [`clone`]: crate::RangesIter::clone
/// [`RangesIter`]: crate::RangesIter
///
/// ## Constructor Examples
/// ```
/// use range_set_blaze::prelude::*;
///
/// // RangeMapBlaze's .range_values(), and .into_range_values()
/// let r = RangeMapBlaze::from_iter([(3, "a"), (2, "a"), (1, "a"), (100, "b"), (1, "c")]);
/// let a = r.range_values();
/// assert!(a.into_string() == r#"(1..=3, "a"), (100..=100, "b")"#);
/// // 'into_range_values' takes ownership of the 'RangeMapBlaze'
/// let a = r.into_range_values();
/// assert!(a.into_string() == r#"(1..=3, "a"), (100..=100, "b")"#);
///
/// // CheckSortedDisjointMap -- unsorted or overlapping input ranges will cause a panic.
/// let a = CheckSortedDisjointMap::new([(1..=3, &"a"), (100..=100, &"b")]);
/// assert!(a.into_string() == r#"(1..=3, "a"), (100..=100, "b")"#);
///
/// // DynamicSortedDisjointMap of a SortedDisjointMap iterator
/// let a = CheckSortedDisjointMap::new([(1..=3, &"a"), (100..=100, &"b")]);
/// let b = DynSortedDisjointMap::new(a);
/// assert!(b.into_string() == r#"(1..=3, "a"), (100..=100, "b")"#);
/// ```
///
/// # `SortedDisjointMap` Set Operations
///
/// You can perform set operations on `SortedDisjointMap`s and `SortedDisjoint`s using operators.
/// In the table below, `a`, `b`, and `c` are `SortedDisjointMap` and `s` is a `SortedDisjoint`.
///
/// | Set Operator               | Operator                      | Multiway (same type)                                      | Multiway (different types)                     |
/// |----------------------------|-------------------------------|-----------------------------------------------------------|-----------------------------------------------|
/// | [`union`]                  | [`a` &#124; `b`]              | `[a, b, c].`[`union`][multiway_union]`() `                | [`union_map_dyn!`](a, b, c)                    |
/// | [`intersection`]           | [`a & b`]                     | `[a, b, c].`[`intersection`][multiway_intersection]`() `  | [`intersection_map_dyn!`](a, b, c)             |
/// | `intersection`             | [`a.map_and_set_intersection(s)`] | *n/a*                                                     | *n/a*                                          |
/// | [`difference`]             | [`a - b`]                     | *n/a*                                                     | *n/a*                                          |
/// | `difference`               | [`a.map_and_set_difference(s)`] | *n/a*                                                     | *n/a*                                          |
/// | [`symmetric_difference`]   | [`a ^ b`]                     | `[a, b, c].`[`symmetric_difference`][multiway_symmetric_difference]`() ` | [`symmetric_difference_map_dyn!`](a, b, c) |
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
/// let a0 = RangeMapBlaze::from_iter([(1..=2, "a"), (5..=100, "a")]);
/// let b0 = RangeMapBlaze::from_iter([(2..=6, "b")]);
///
/// // 'union' method and 'into_string' method
/// let (a, b) = (a0.range_values(), b0.range_values());
/// let result = a.union(b);
/// assert_eq!(result.into_string(), r#"(1..=2, "a"), (3..=4, "b"), (5..=100, "a")"#);
///
/// // '|' operator and 'equal' method
/// let (a, b) = (a0.range_values(), b0.range_values());
/// let result = a | b;
/// assert!(result.equal(CheckSortedDisjointMap::new([(1..=2, &"a"),  (3..=4, &"b"), (5..=100, &"a")])));
///
/// // multiway union of same type
/// let c0 = RangeMapBlaze::from_iter([(2..=2, "c"), (6..=200, "c")]);
/// let (a, b, c) = (a0.range_values(), b0.range_values(), c0.range_values());
/// let result = [a, b, c].union();
/// assert_eq!(result.into_string(), r#"(1..=2, "a"), (3..=4, "b"), (5..=100, "a"), (101..=200, "c")"#
/// );
///
/// // multiway union of different types
/// let (a, b) = (a0.range_values(), b0.range_values());
/// let c = CheckSortedDisjointMap::new([(2..=2, &"c"), (6..=200, &"c")]);
/// let result = union_map_dyn!(a, b, c);
/// assert_eq!(result.into_string(), r#"(1..=2, "a"), (3..=4, "b"), (5..=100, "a"), (101..=200, "c")"# );
///
/// // Applying multiple operators makes only one pass through the inputs with minimal memory.
/// let (a, b, c) = (a0.range_values(), b0.range_values(), c0.range_values());
/// let result = a - (b | c);
/// assert_eq!(result.into_string(), r#"(1..=1, "a")"#);
/// ```
pub trait SortedDisjointMap<T, VR>: SortedStartsMap<T, VR>
where
    T: Integer,
    VR: ValueRef,
{
    /// cmk doc
    ///```
    /// use range_set_blaze::prelude::*;
    ///
    /// let a = CheckSortedDisjointMap::new([(1..=3, &"a"), (100..=100, &"b")]);
    /// let b = a.into_sorted_disjoint();
    /// assert!(b.into_string() == "1..=3, 100..=100");
    /// ```
    #[inline]
    fn into_sorted_disjoint(self) -> RangeValuesToRangesIter<T, VR, Self>
    where
        Self: Sized,
    {
        RangeValuesToRangesIter::new(self)
    }
    // cmk I think this is 'Sized' because will sometimes want to create a struct (e.g. BitOrIter) that contains a field of this type

    /// Given two [`SortedDisjointMap`] iterators, efficiently returns a [`SortedDisjointMap`] iterator of their union.
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
    /// let union = a.union(b);
    /// assert_eq!(union.into_string(), r#"(1..=2, "a"), (3..=3, "b")"#);
    ///
    /// // Alternatively, we can use "|" because CheckSortedDisjointMap defines
    /// // ops::bitor as SortedDisjointMap::union.
    /// let a = CheckSortedDisjointMap::new([(1..=2, &"a")]);
    /// let b = b0.range_values();
    /// let union = a | b;
    /// assert_eq!(union.into_string(), r#"(1..=2, "a"), (3..=3, "b")"#);
    /// ```
    #[inline]
    fn union<R>(self, other: R) -> BitOrMapMerge<T, VR, Self, R::IntoIter>
    where
        R: IntoIterator<Item = Self::Item>,
        R::IntoIter: SortedDisjointMap<T, VR>,
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
    /// let a = CheckSortedDisjointMap::new([(1..=2, &"a")]);
    /// let b0 = RangeMapBlaze::from_iter([(2..=3, "b")]);
    /// let b = b0.range_values();
    /// let intersection = a.intersection(b);
    /// assert_eq!(intersection.into_string(), r#"(2..=2, "a")"#);
    ///
    /// // Alternatively, we can use "&" because CheckSortedDisjointMap defines
    /// // `ops::BitAnd` as `SortedDisjointMap::intersection`.
    /// let a = CheckSortedDisjointMap::new([(1..=2, &"a")]);
    /// let b = b0.range_values();
    /// let intersection = a & b;
    /// assert_eq!(intersection.into_string(), r#"(2..=2, "a")"#);
    /// ```
    #[inline]
    fn intersection<R>(self, other: R) -> BitAndRangesMap2<T, VR, Self, R::IntoIter>
    where
        R: IntoIterator<Item = Self::Item>,
        R::IntoIter: SortedDisjointMap<T, VR>,
        Self: Sized,
    {
        let sorted_disjoint_map = other.into_iter();
        let sorted_disjoint = sorted_disjoint_map.into_sorted_disjoint();
        IntersectionIterMap::new(self, sorted_disjoint)
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
    fn map_and_set_intersection<R>(self, other: R) -> BitAndRangesMap<T, VR, Self, R::IntoIter>
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
    fn difference<R>(self, other: R) -> BitSubRangesMap2<T, VR, Self, R::IntoIter>
    where
        R: IntoIterator<Item = Self::Item>,
        R::IntoIter: SortedDisjointMap<T, VR>,
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
    fn map_and_set_difference<R>(self, other: R) -> BitSubRangesMap<T, VR, Self, R::IntoIter>
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
    fn complement(self) -> NotIter<T, RangeValuesToRangesIter<T, VR, Self>>
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
        v: &VR::Value,
    ) -> RangeToRangeValueIter<T, VR::Value, NotIter<T, impl SortedDisjoint<T>>>
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
    fn symmetric_difference<R>(self, other: R) -> BitXorMapMerge<T, VR, Self, R::IntoIter>
    where
        R: IntoIterator<Item = Self::Item>,
        R::IntoIter: SortedDisjointMap<T, VR>,
        Self: Sized,
        VR: ValueRef,
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
        R::IntoIter: SortedDisjointMap<T, VR>,
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
                    self_range == other_range && self_value.borrow() == other_value.borrow()
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
    fn into_range_map_blaze(self) -> RangeMapBlaze<T, VR::Value>
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

// cmk0
// impl<'a, T, V, VR> PartialEq for RangeValue<T, V, VR>
// where
//     T: Integer,
//     V: ValueOwned + 'a,
//     VR: CloneBorrow<V> + 'a,
// {
//     fn eq(&self, other: &Self) -> bool {
//         self.range == other.range && self.1.borrow() == other.1.borrow()
//     }
// }

// // Implement `Eq` because `BinaryHeap` requires it.
// impl<'a, T, V, VR> Eq for RangeValue<T, V, VR>
// where
//     T: Integer,
//     V: ValueOwned + 'a,
//     VR: CloneBorrow<V> + 'a,
// {
// }

/// Gives any iterator of cmk implements the [`SortedDisjointMap`] trait cmk without any checking.
// cmk0 why was this hidden? check for others#[doc(hidden)]
/// doc
#[allow(clippy::module_name_repetitions)]
pub struct CheckSortedDisjointMap<T, VR, I>
where
    T: Integer,
    VR: ValueRef,
    I: Iterator<Item = (RangeInclusive<T>, VR)>,
{
    iter: I,
    seen_none: bool,
    previous: Option<(RangeInclusive<T>, VR)>,
}

// define new
impl<T, VR, I> CheckSortedDisjointMap<T, VR, I>
where
    T: Integer,
    VR: ValueRef,
    I: Iterator<Item = (RangeInclusive<T>, VR)>,
{
    // Does CheckSortedDisjointMap and CheckSortedDisjoint need both from and public 'new'?
    /// cmk doc
    #[inline]
    #[must_use]
    pub fn new<J>(iter: J) -> Self
    where
        J: IntoIterator<Item = (RangeInclusive<T>, VR), IntoIter = I>,
    {
        Self {
            iter: iter.into_iter(),
            seen_none: false,
            previous: None,
        }
    }
}

impl<T, VR, I> Default for CheckSortedDisjointMap<T, VR, I>
where
    T: Integer,
    VR: ValueRef,
    I: Iterator<Item = (RangeInclusive<T>, VR)> + Default,
{
    fn default() -> Self {
        // Utilize I::default() to satisfy the iterator requirement.
        Self::new(I::default())
    }
}
// implement fused
impl<T, VR, I> FusedIterator for CheckSortedDisjointMap<T, VR, I>
where
    T: Integer,
    VR: ValueRef,
    I: Iterator<Item = (RangeInclusive<T>, VR)>,
{
}

fn range_value_clone<T, VR>(range_value: &(RangeInclusive<T>, VR)) -> (RangeInclusive<T>, VR)
where
    T: Integer,
    VR: ValueRef,
{
    let (range, value) = range_value;
    (range.clone(), value.clone_ref())
}

impl<T, VR, I> Iterator for CheckSortedDisjointMap<T, VR, I>
where
    T: Integer,
    VR: ValueRef,
    I: Iterator<Item = (RangeInclusive<T>, VR)>,
{
    type Item = (RangeInclusive<T>, VR);

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
        };

        // Check that the range is not empty
        let (start, end) = range_value.0.clone().into_inner();
        if start > end {
            panic!("start must be <= end")
        };

        // If previous is None, we're done (but remember this pair as previous)
        let Some(previous) = self.previous.take() else {
            self.previous = Some(range_value_clone(&range_value));
            return Some(range_value);
        };

        // The next_item is Some and previous is Some, so check that the ranges are disjoint and sorted
        let previous_end = *previous.0.end();
        if previous_end >= start {
            panic!("ranges must be disjoint and sorted")
        };
        if previous_end.add_one() == start && previous.1.borrow() == range_value.1.borrow() {
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

/// Used internally by [`UnionIterMap`] and [`SymDiffIterMap`].
#[derive(Clone, Debug)]
pub struct Priority<T, VR>
where
    T: Integer,
    VR: ValueRef,
{
    range_value: (RangeInclusive<T>, VR),
    priority_number: usize,
}

impl<T, VR> Priority<T, VR>
where
    T: Integer,
    VR: ValueRef,
{
    /// cmk doc
    pub const fn new(range_value: (RangeInclusive<T>, VR), priority_number: usize) -> Self {
        Self {
            range_value,
            priority_number,
        }
    }
}

impl<T, VR> Priority<T, VR>
where
    T: Integer,
    VR: ValueRef,
{
    /// Returns a reference to `range_value`.
    pub const fn range_value(&self) -> &(RangeInclusive<T>, VR) {
        &self.range_value
    }

    /// Consumes `Priority` and returns `range_value`.
    pub fn into_range_value(self) -> (RangeInclusive<T>, VR) {
        self.range_value
    }

    /// Updates the range part of `range_value`.
    pub fn set_range(&mut self, range: RangeInclusive<T>) {
        self.range_value.0 = range;
    }

    /// Returns the start of the range.
    pub const fn start(&self) -> T {
        *self.range_value.0.start()
    }

    /// Returns the end of the range.
    pub const fn end(&self) -> T {
        *self.range_value.0.end()
    }

    /// Returns the start and end of the range. (Assuming direct access to start and end)
    pub const fn start_and_end(&self) -> (T, T) {
        ((*self.range_value.0.start()), (*self.range_value.0.end()))
    }

    /// Returns a reference to the value part of `range_value`.
    pub const fn value(&self) -> &VR {
        &self.range_value.1
    }
}

// Implement `PartialEq` to allow comparison (needed for `Eq`).
impl<T, VR> PartialEq for Priority<T, VR>
where
    T: Integer,
    VR: ValueRef,
{
    fn eq(&self, other: &Self) -> bool {
        self.priority_number == other.priority_number
    }
}

// Implement `Eq` because `BinaryHeap` requires it.
impl<T, VR> Eq for Priority<T, VR>
where
    T: Integer,
    VR: ValueRef,
{
}

// Implement `Ord` so the heap knows how to compare elements.
impl<T, VR> Ord for Priority<T, VR>
where
    T: Integer,
    VR: ValueRef,
{
    fn cmp(&self, other: &Self) -> Ordering {
        // smaller is better
        other.priority_number.cmp(&self.priority_number)
    }
}

// Implement `PartialOrd` to allow comparison (needed for `Ord`).
impl<T, VR> PartialOrd for Priority<T, VR>
where
    T: Integer,
    VR: ValueRef,
{
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

pub struct RangeToRangeValueIter<'a, T, V, I>
where
    T: Integer,
    V: EqClone,
    I: SortedDisjoint<T>,
{
    inner: I,
    value: &'a V,
    phantom: PhantomData<T>,
}

impl<'a, T, V, I> RangeToRangeValueIter<'a, T, V, I>
where
    T: Integer,
    V: EqClone,
    I: SortedDisjoint<T>,
{
    pub const fn new(inner: I, value: &'a V) -> Self {
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
    V: EqClone,
    I: SortedDisjoint<T>,
{
}

impl<'a, T, V, I> Iterator for RangeToRangeValueIter<'a, T, V, I>
where
    T: Integer,
    V: EqClone,
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
    V: EqClone,
    I: SortedDisjoint<T>,
{
}
impl<'a, T, V, I> SortedDisjointMap<T, &'a V> for RangeToRangeValueIter<'a, T, V, I>
where
    T: Integer,
    V: EqClone,
    I: SortedDisjoint<T>,
{
}

macro_rules! impl_sorted_map_traits_and_ops {
    ($IterType:ty, $V:ty, $VR:ty, $($more_generics:tt)*) => {

        #[allow(single_use_lifetimes)]
        impl<$($more_generics)*, T> SortedStartsMap<T, $VR> for $IterType
        where
            T: Integer,
        {
        }

        #[allow(single_use_lifetimes)]
        impl<$($more_generics)*, T> SortedDisjointMap<T, $VR> for $IterType
        where
            T: Integer,
        {
        }

        #[allow(single_use_lifetimes)]
        impl<$($more_generics)*, T> ops::Not for $IterType
        where
            T: Integer,
        {
            type Output = NotIter<T, RangeValuesToRangesIter<T, $VR, Self>>;

            fn not(self) -> Self::Output {
                self.complement()
            }
        }

        #[allow(single_use_lifetimes)]
        impl<$($more_generics)*, T, R> ops::BitOr<R> for $IterType
        where
            T: Integer,
            R: SortedDisjointMap<T, $VR>,
        {
            type Output = BitOrMapMerge<T, $VR, Self, R>;

            fn bitor(self, other: R) -> Self::Output {
                SortedDisjointMap::union(self, other)
            }
        }

        #[allow(single_use_lifetimes)]
        impl<$($more_generics)*, T, R> ops::Sub<R> for $IterType
        where
            T: Integer,
            R: SortedDisjointMap<T, $VR>,
        {
            type Output = BitSubRangesMap<T, $VR, Self, RangeValuesToRangesIter<T, $VR, R>>;

            fn sub(self, other: R) -> Self::Output {
                SortedDisjointMap::difference(self, other)
            }
        }

        #[allow(single_use_lifetimes)]
        impl<$($more_generics)*, T, R> ops::BitXor<R> for $IterType
        where
            T: Integer,
            R: SortedDisjointMap<T, $VR>,
        {
            type Output = BitXorMapMerge<T,  $VR, Self, R>;

            #[allow(clippy::suspicious_arithmetic_impl)]
            fn bitxor(self, other: R) -> Self::Output {
                SortedDisjointMap::symmetric_difference(self, other)
            }
        }

        #[allow(single_use_lifetimes)]
        impl<$($more_generics)*, T, R> ops::BitAnd<R> for $IterType
        where
            T: Integer,
            R: SortedDisjointMap<T, $VR>,
        {
            type Output = BitAndRangesMap<T, $VR, Self, RangeValuesToRangesIter<T, $VR, R>>;

            fn bitand(self, other: R) -> Self::Output {
                SortedDisjointMap::intersection(self, other)
            }
        }

    }
}

// cmk CheckList: Be sure that these are all tested in 'test_every_sorted_disjoint_method'
impl_sorted_map_traits_and_ops!(CheckSortedDisjointMap<T, VR, I>, VR::Value, VR, VR: ValueRef, I: Iterator<Item = (RangeInclusive<T>,  VR)>);
impl_sorted_map_traits_and_ops!(UnionIterMap<T, VR, I>, VR::Value, VR, VR: ValueRef, I: PrioritySortedStartsMap<T, VR>);
impl_sorted_map_traits_and_ops!(IntersectionIterMap<T, VR, I0, I1>,  VR::Value, VR, VR: ValueRef, I0: SortedDisjointMap<T, VR>, I1: SortedDisjoint<T>);
impl_sorted_map_traits_and_ops!(SymDiffIterMap<T, VR, I>, VR::Value, VR, VR: ValueRef, I: PrioritySortedStartsMap<T, VR>);
impl_sorted_map_traits_and_ops!(DynSortedDisjointMap<'a, T, VR>, VR::Value, VR, 'a, VR: ValueRef);
impl_sorted_map_traits_and_ops!(RangeValuesIter<'a, T, V>, V, &'a V, 'a, V: EqClone);
impl_sorted_map_traits_and_ops!(IntoRangeValuesIter<T, V>, V, Rc<V>, V: EqClone);

#[test]
fn cmk_delete_temp() {
    let a = CheckSortedDisjointMap::new([(1..=2, &"a")]);
    let b0 = RangeMapBlaze::from_iter([(2..=3, "b")]);
    let b = b0.range_values();
    let intersection = a.intersection(b);
    assert_eq!(intersection.into_string(), r#"(2..=2, "a")"#);

    // Alternatively, we can use "&" because CheckSortedDisjointMap defines
    // `ops::BitAnd` as `SortedDisjointMap::intersection`.
    let a = CheckSortedDisjointMap::new([(1..=2, &"a")]);
    let b = b0.range_values();
    let intersection = a & b;
    assert_eq!(intersection.into_string(), r#"(2..=2, "a")"#);
}

#[test]
fn cmk_delete_temp2() {
    use std::collections::{BTreeSet, HashSet};
    use std::println;

    let mut set = HashSet::new();
    set.insert("apple");
    set.insert("banana");
    set.insert("cherry");

    let _ = set.iter().for_each(|x| println!("{}", x));

    // let _set2 = BTreeSet::from_iter(&set); // clippy likes this
    // let _set3 = BTreeSet::from_iter(set.iter()); // clippy NOT like this
    // let _set4: BTreeSet<_> = set.iter().collect(); // clippy like this

    // Prove that HashSet is still usable after the loop.
    println!("{set:?}");
}

// cmk0000 remove extra .iter()'s in examples.
// cmk0000 be sure that .iter() is just define as into_iter() of the reference (and that both are always defined)
