use crate::map::BitSubRangesMap;
use crate::range_values::RangeValuesIter;
use crate::range_values::RangeValuesToRangesIter;
use crate::sym_diff_iter_map::SymDiffIterMap;
use crate::BitOrMapMerge;
use crate::BitXorMapMerge;
use crate::DynSortedDisjointMap;
use alloc::format;
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
use crate::map::CloneBorrow;
use crate::sorted_disjoint::SortedDisjoint;
use crate::NotIter;
use crate::{map::PartialEqClone, union_iter_map::UnionIterMap, Integer, RangeMapBlaze};
use core::ops;
use core::ops::RangeInclusive;

/// Internally, a trait used to mark iterators that provide ranges sorted by start, but not necessarily by end,
/// and may overlap.
pub trait SortedStartsMap<T, V, VR>:
    Iterator<Item = (RangeInclusive<T>, VR)> + FusedIterator
where
    T: Integer,
    V: PartialEqClone,
    VR: CloneBorrow<V>,
{
}

/// Used internally by [`UnionIterMap`] and [`SymDiffIterMap`].
pub trait PrioritySortedStartsMap<T, V, VR>:
    Iterator<Item = Priority<T, V, VR>> + FusedIterator
where
    T: Integer,
    V: PartialEqClone,
    VR: CloneBorrow<V>,
{
}

/// The trait used to mark iterators that provide ranges that are sorted by start and disjoint. Set operations on
/// iterators that implement this trait can be performed in linear time.
///
/// # Table of Contents
/// * [`SortedDisjointMap` Constructors](#SortedDisjointMap-constructors)
///   * [Examples](#constructor-examples)
/// * [`SortedDisjointMap` Set and Other Operations](#SortedDisjointMap-set-and-other-operations)
///   * [Performance](#performance)
///   * [Examples](#examples)
/// * [How to mark your type as `SortedDisjointMap`](#how-to-mark-your-type-as-SortedDisjointMap)
///   * [Example – Find the ordinal weekdays in September 2023](#example--find-the-ordinal-weekdays-in-september-2023)
///
/// # `SortedDisjointMap` Constructors
///
/// You'll usually construct a `SortedDisjointMap` iterator from a [`RangeMapBlaze`] or a [`CheckSortedDisjointMap`].
/// Here is a summary table, followed by [examples](#constructor-examples). You can also [define your own
/// `SortedDisjointMap`](#how-to-mark-your-type-as-SortedDisjointMap).
///
/// | Input type | Method |
/// |------------|--------|
/// | [`RangeMapBlaze`] | [`ranges`] |
/// | [`RangeMapBlaze`] | [`into_ranges`] |
/// | [`RangeMapBlaze`]'s [`RangesIter`] | [`clone`] |
/// | sorted & disjoint ranges | [`CheckSortedDisjointMap::new`] |
/// | `SortedDisjointMap` iterator | [itertools `tee`] |
/// | `SortedDisjointMap` iterator | [`crate::dyn_sorted_disjoint::DynSortedDisjointMap::new`] |
/// |  *your iterator type* | *[How to mark your type as `SortedDisjointMap`][1]* |
///
/// [`ranges`]: RangeMapBlaze::ranges
/// [`into_ranges`]: RangeMapBlaze::into_ranges
/// [`clone`]: crate::RangesIter::clone
/// [itertools `tee`]: https://docs.rs/itertools/latest/itertools/trait.Itertools.html#method.tee
/// [1]: #how-to-mark-your-type-as-SortedDisjointMap
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
/// | Method | Operator | Multiway (same type) | Multiway (different types) |
/// |--------|----------|----------------------|----------------------------|
/// | `a.`[`union`]`(b)` | `a` &#124; `b` | `[a, b, c].`[`union`][crate::MultiwaySortedDisjointMap::union]`()` | [`crate::MultiwayRangeSetBlaze::union`]`!(a, b, c)` |
/// | `a.`[`intersection`]`(b)` | `a & b` | `[a, b, c].`[`intersection`][crate::MultiwaySortedDisjointMap::intersection]`()` | [`crate::MultiwayRangeSetBlaze::intersection`]`!(a, b, c)` |
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
pub trait SortedDisjointMap<T, V, VR>: SortedStartsMap<T, V, VR>
where
    T: Integer,
    V: PartialEqClone,
    VR: CloneBorrow<V>,
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
    fn into_sorted_disjoint(self) -> RangeValuesToRangesIter<T, V, VR, Self>
    where
        Self: Sized,
    {
        RangeValuesToRangesIter::new(self)
    }
    // I think this is 'Sized' because will sometimes want to create a struct (e.g. BitOrIter) that contains a field of this type

    /// Given two [`SortedDisjointMap`] iterators, efficiently returns a [`SortedDisjointMap`] iterator of their union.
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
    /// let b0 = RangeMapBlaze::from_iter([(2..=3, "b")]);
    /// let b = b0.range_values();
    /// let union = a | b;
    /// assert_eq!(union.into_string(), r#"(1..=2, "a"), (3..=3, "b")"#);
    /// ```
    #[inline]
    fn union<R>(self, other: R) -> BitOrMapMerge<T, V, VR, Self, R::IntoIter>
    where
        // cmk why must say SortedDisjointMap here by sorted_disjoint doesn't.
        R: IntoIterator<Item = Self::Item>,
        R::IntoIter: SortedDisjointMap<T, V, VR>,
        Self: Sized,
    {
        // cmk why this into iter stuff that is not used?
        UnionIterMap::new2(self, other.into_iter())
    }

    /// Given two [`SortedDisjointMap`] iterators, efficiently returns a [`SortedDisjointMap`] iterator of their intersection.
    ///
    /// /// cmk Tell that right-and-side must be a set, not a map
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
    /// // ops::bitand as SortedDisjointMap::intersection.
    /// let a = CheckSortedDisjointMap::new([(1..=2, &"a")]);
    /// let b0 = RangeMapBlaze::from_iter([(2..=3, "b")]);
    /// let b = b0.range_values();
    /// let intersection = a & b;
    /// assert_eq!(intersection.into_string(), r#"(2..=2, "a")"#);
    /// ```
    #[inline]
    fn intersection<R>(
        self,
        other: R,
    ) -> BitAndRangesMap<T, V, VR, Self, RangeValuesToRangesIter<T, V, VR, R::IntoIter>>
    where
        R: IntoIterator<Item = Self::Item>,
        R::IntoIter: SortedDisjointMap<T, V, VR>,
        Self: Sized,
    {
        let sorted_disjoint_map = other.into_iter();
        let sorted_disjoint = sorted_disjoint_map.into_sorted_disjoint();
        IntersectionIterMap::new(self, sorted_disjoint)
    }

    /// Given two [`SortedDisjointMap`] iterators, efficiently returns a [`SortedDisjointMap`] iterator of their intersection.
    ///
    /// /// cmk Tell that right-and-side must be a set, not a map
    ///
    /// # Examples
    ///
    /// ```
    /// use range_set_blaze::prelude::*;
    ///
    /// let a = CheckSortedDisjointMap::new([(1..=2, &"a")]);
    /// let b = RangeMapBlaze::from_iter([(2..=3, "b")]).into_ranges();
    /// let intersection = a.intersection_with_set(b);
    /// assert_eq!(intersection.into_string(), r#"(2..=2, "a")"#);
    /// ```
    #[inline]
    // cmk should this be called intersection_with_ranges?
    fn intersection_with_set<R>(self, other: R) -> BitAndRangesMap<T, V, VR, Self, R::IntoIter>
    where
        R: IntoIterator<Item = RangeInclusive<T>>, // cmk0 is this bound needed?
        R::IntoIter: SortedDisjoint<T>,
        Self: Sized,
    {
        let sorted_disjoint = other.into_iter();
        IntersectionIterMap::new(self, sorted_disjoint)
    }

    /// Given two [`SortedDisjointMap`] iterators, efficiently returns a [`SortedDisjointMap`] iterator of their set difference.
    ///
    /// cmk Tell that right-and-side must be a set, not a map
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
    /// // Alternatively, we can use "-" because CheckSortedDisjointMap defines
    /// // ops::sub as SortedDisjointMap::difference.
    /// let a = CheckSortedDisjointMap::new([(1..=2, &"a")]);
    /// let b0 = RangeMapBlaze::from_iter([(2..=3, "b")]);
    /// let b = b0.range_values();
    /// let difference = a - b;
    /// assert_eq!(difference.into_string(), r#"(1..=1, "a")"#);
    /// ```
    #[inline]
    fn difference<R>(
        self,
        other: R,
    ) -> BitSubRangesMap<T, V, VR, Self, RangeValuesToRangesIter<T, V, VR, R::IntoIter>>
    where
        R: IntoIterator<Item = Self::Item>,
        R::IntoIter: SortedDisjointMap<T, V, VR>,
        Self: Sized,
    {
        let sorted_disjoint_map = other.into_iter();
        let complement = sorted_disjoint_map.complement_to_set();
        IntersectionIterMap::new(self, complement)
    }

    /// Given two [`SortedDisjointMap`] iterators, efficiently returns a [`SortedDisjointMap`] iterator of their set difference.
    ///
    /// cmk Tell that right-and-side must be a set, not a map
    ///
    /// # Examples
    ///
    /// ```
    /// use range_set_blaze::prelude::*;
    ///
    /// let a = CheckSortedDisjointMap::new([(1..=2, &"a")]);
    /// let b = RangeMapBlaze::from_iter([(2..=3, "b")]).into_ranges();
    /// let difference = a.difference_with_set(b);
    /// assert_eq!(difference.into_string(), r#"(1..=1, "a")"#);
    /// ```
    #[inline]
    // cmk rename difference_with_ranges?
    fn difference_with_set<R>(self, other: R) -> BitSubRangesMap<T, V, VR, Self, R::IntoIter>
    where
        R: IntoIterator<Item = RangeInclusive<T>>,
        R::IntoIter: SortedDisjoint<T>,
        Self: Sized,
    {
        let sorted_disjoint = other.into_iter();
        let complement = sorted_disjoint.complement();
        IntersectionIterMap::new(self, complement)
    }

    /// cmk rename 'to_range'?
    /// returns a set, not a map also see complement
    /// # Examples
    /// ```
    /// use range_set_blaze::prelude::*;
    ///
    /// let a = CheckSortedDisjointMap::new([(-10_i16..=0, &"a"), (1000..=2000, &"a")]);
    /// let complement = a.complement_to_set();
    /// assert_eq!(complement.into_string(), "-32768..=-11, 1..=999, 2001..=32767");
    /// ```
    #[inline]
    fn complement_to_set(self) -> NotIter<T, RangeValuesToRangesIter<T, V, VR, Self>>
    where
        Self: Sized,
    {
        let sorted_disjoint = self.into_sorted_disjoint();
        sorted_disjoint.complement()
    }

    /// cmk no "!" operator defined because we need a fill value also see complement_to_set
    /// returns a set, not a map
    /// # Examples
    /// ```
    /// use range_set_blaze::prelude::*;
    ///
    /// let a = CheckSortedDisjointMap::new([(-10_i16..=0, &"a"), (1000..=2000, &"a")]);
    /// let complement = a.complement(&"z");
    /// assert_eq!(complement.into_string(), r#"(-32768..=-11, "z"), (1..=999, "z"), (2001..=32767, "z")"#);
    /// ```    
    #[inline]
    fn complement(self, v: &V) -> RangeToRangeValueIter<T, V, NotIter<T, impl SortedDisjoint<T>>>
    where
        Self: Sized,
    {
        let complement = self.complement_to_set();
        RangeToRangeValueIter::new(complement, v)
    }

    /// Given two [`SortedDisjointMap`] iterators, efficiently returns a [`SortedDisjointMap`] iterator
    /// of their symmetric difference.
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
    fn symmetric_difference<R>(self, other: R) -> BitXorMapMerge<T, V, VR, Self, R::IntoIter>
    where
        R: IntoIterator<Item = Self::Item>,
        R::IntoIter: SortedDisjointMap<T, V, VR>,
        Self: Sized,
        VR: CloneBorrow<V>,
    {
        SymDiffIterMap::new2(self, other.into_iter())
    }

    /// Given two [`SortedDisjointMap`] iterators, efficiently tells if they are equal. Unlike most equality testing in Rust,
    /// this method takes ownership of the iterators and consumes them.
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
        R::IntoIter: SortedDisjointMap<T, V, VR>,
        Self: Sized,
    {
        use itertools::Itertools;

        self.zip_longest(other.into_iter()).all(|pair| {
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

    /// Returns `true` if the map contains no elements.
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
    /// *For more about constructors and performance, see [`RangeMapBlaze` Constructors](struct.RangeMapBlaze.html#constructors).*
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
    fn into_range_map_blaze(self) -> RangeMapBlaze<T, V>
    where
        Self: Sized,
        V: Clone,
    {
        RangeMapBlaze::from_sorted_disjoint_map(self)
    }
}

/// Converts the implementing type into a String by consuming it.
/// It is intended for types where items are Debug-able.
pub trait IntoString {
    /// cmk doc
    fn into_string(self) -> String;
}

impl<T, I> IntoString for I
where
    T: Debug,
    I: Iterator<Item = T>,
{
    fn into_string(self) -> String {
        self.map(|item| format!("{:?}", item))
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
pub struct CheckSortedDisjointMap<T, V, VR, I>
where
    T: Integer,
    V: PartialEqClone,
    VR: CloneBorrow<V>,
    I: Iterator<Item = (RangeInclusive<T>, VR)>,
{
    iter: I,
    seen_none: bool,
    previous: Option<(RangeInclusive<T>, VR)>,
    phantom_data: PhantomData<V>,
}

// define new
impl<T, V, VR, I> CheckSortedDisjointMap<T, V, VR, I>
where
    T: Integer,
    V: PartialEqClone,
    VR: CloneBorrow<V>,
    I: Iterator<Item = (RangeInclusive<T>, VR)>,
{
    // Does CheckSortedDisjointMap and CheckSortedDisjoint need both from and public 'new'?
    /// cmk doc
    pub fn new<J>(iter: J) -> Self
    where
        J: IntoIterator<Item = (RangeInclusive<T>, VR), IntoIter = I>,
    {
        CheckSortedDisjointMap {
            iter: iter.into_iter(),
            seen_none: false,
            previous: None,
            phantom_data: PhantomData,
        }
    }
}

impl<T, V, VR, I> Default for CheckSortedDisjointMap<T, V, VR, I>
where
    T: Integer,
    V: PartialEqClone,
    VR: CloneBorrow<V>,
    I: Iterator<Item = (RangeInclusive<T>, VR)> + Default,
{
    fn default() -> Self {
        // Utilize I::default() to satisfy the iterator requirement.
        Self::new(I::default())
    }
}
// implement fused
impl<T, V, VR, I> FusedIterator for CheckSortedDisjointMap<T, V, VR, I>
where
    T: Integer,
    V: PartialEqClone,
    VR: CloneBorrow<V>,
    I: Iterator<Item = (RangeInclusive<T>, VR)>,
{
}

fn range_value_clone<T, V, VR>(range_value: &(RangeInclusive<T>, VR)) -> (RangeInclusive<T>, VR)
where
    T: Integer,
    V: PartialEqClone,
    VR: CloneBorrow<V>,
{
    let (range, value) = range_value;
    (range.clone(), value.clone_borrow())
}

// implement iterator
impl<T, V, VR, I> Iterator for CheckSortedDisjointMap<T, V, VR, I>
where
    T: Integer,
    V: PartialEqClone,
    VR: CloneBorrow<V>,
    I: Iterator<Item = (RangeInclusive<T>, VR)>,
{
    type Item = (RangeInclusive<T>, VR);

    fn next(&mut self) -> Option<Self::Item> {
        let range_value = self.iter.next();
        let Some(range_value) = range_value else {
            self.seen_none = true;
            return None;
        };
        // cmk should test all these
        assert!(!self.seen_none, "A value must not be returned after None");
        let Some(previous) = self.previous.take() else {
            self.previous = Some(range_value_clone(&range_value));
            return Some(range_value);
        };

        let previous_end = *previous.0.end();
        let (start, end) = range_value.0.clone().into_inner();
        assert!(start <= end, "Start must be <= end.",);
        assert!(previous_end < start, "Ranges must be disjoint and sorted");
        if previous_end.add_one() == start {
            assert!(
                previous.1.borrow() != range_value.1.borrow(),
                "Touching ranges must have different values"
            );
        }
        self.previous = Some(range_value_clone(&range_value));
        Some(range_value_clone(&range_value))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

// // cmk00 check
// // cmk00 make Fused but don't require it

/// Used internally by [`UnionIterMap`] and [`SymDiffIterMap`].
#[derive(Clone, Debug)]
pub struct Priority<T, V, VR>
where
    T: Integer,
    V: PartialEqClone,
    VR: CloneBorrow<V>,
{
    range_value: (RangeInclusive<T>, VR),
    priority_number: usize,
    phantom_data: PhantomData<V>,
}

// new
impl<T, V, VR> Priority<T, V, VR>
where
    T: Integer,
    V: PartialEqClone,
    VR: CloneBorrow<V>,
{
    /// cmk doc
    pub fn new(range_value: (RangeInclusive<T>, VR), priority_number: usize) -> Self {
        Self {
            range_value,
            priority_number,
            phantom_data: PhantomData,
        }
    }
}

impl<T, V, VR> Priority<T, V, VR>
where
    T: Integer,
    V: PartialEqClone,
    VR: CloneBorrow<V>,
{
    /// Returns the priority number.
    pub fn priority_number(&self) -> usize {
        self.priority_number
    }
    /// Returns a reference to `range_value`.
    pub fn range_value(&self) -> &(RangeInclusive<T>, VR) {
        &self.range_value
    }

    /// Updates `range_value` with the given value.
    pub fn set_range_value(&mut self, value: (RangeInclusive<T>, VR)) {
        self.range_value = value;
    }

    /// Consumes `Priority` and returns `range_value`.
    pub fn into_range_value(self) -> (RangeInclusive<T>, VR) {
        self.range_value
    }

    /// Updates the range part of `range_value`.
    pub fn set_range(&mut self, range: RangeInclusive<T>) {
        self.range_value.0 = range;
    }

    /// Consumes `Priority` and returns the range part of `range_value`.
    pub fn into_range(self) -> RangeInclusive<T> {
        self.range_value.0
    }

    /// Returns the start of the range.
    pub fn start(&self) -> T {
        *self.range_value.0.start()
    }

    /// Returns the end of the range.
    pub fn end(&self) -> T {
        *self.range_value.0.end()
    }

    /// Returns the start and end of the range. (Assuming direct access to start and end)
    pub fn start_and_end(&self) -> (T, T) {
        (
            (*self.range_value.0.start()).clone(),
            (*self.range_value.0.end()).clone(),
        )
    }

    /// Returns a reference to the value part of `range_value`.
    pub fn value(&self) -> &VR {
        &self.range_value.1
    }
}
// Implement `PartialEq` to allow comparison (needed for `Eq`).
impl<T, V, VR> PartialEq for Priority<T, V, VR>
where
    T: Integer,
    V: PartialEqClone,
    VR: CloneBorrow<V>,
{
    fn eq(&self, other: &Self) -> bool {
        let result_cmk = self.priority_number == other.priority_number;
        assert!(!result_cmk, "Don't expect identical priority numbers");
        result_cmk
    }
}

// Implement `Eq` because `BinaryHeap` requires it.
impl<'a, T, V, VR> Eq for Priority<T, V, VR>
where
    T: Integer,
    V: PartialEqClone + 'a,
    VR: CloneBorrow<V> + 'a,
{
}

// Implement `Ord` so the heap knows how to compare elements.
impl<'a, T, V, VR> Ord for Priority<T, V, VR>
where
    T: Integer,
    V: PartialEqClone + 'a,
    VR: CloneBorrow<V> + 'a,
{
    fn cmp(&self, other: &Self) -> Ordering {
        // smaller is better
        other.priority_number.cmp(&self.priority_number)
    }
}

// Implement `PartialOrd` to allow comparison (needed for `Ord`).
impl<'a, T, V, VR> PartialOrd for Priority<T, V, VR>
where
    T: Integer,
    V: PartialEqClone + 'a,
    VR: CloneBorrow<V> + 'a,
{
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

pub struct RangeToRangeValueIter<'a, T, V, I>
where
    T: Integer,
    V: PartialEqClone,
    I: SortedDisjoint<T>,
{
    inner: I,
    value: &'a V,
    phantom: PhantomData<T>,
}

impl<'a, T, V, I> RangeToRangeValueIter<'a, T, V, I>
where
    T: Integer,
    V: PartialEqClone,
    I: SortedDisjoint<T>,
{
    pub fn new(inner: I, value: &'a V) -> Self {
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
    V: PartialEqClone,
    I: SortedDisjoint<T>,
{
}

impl<'a, T, V, I> Iterator for RangeToRangeValueIter<'a, T, V, I>
where
    T: Integer,
    V: PartialEqClone,
    I: SortedDisjoint<T>,
{
    type Item = (RangeInclusive<T>, &'a V);

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(|range| (range, self.value))
    }
}

// implements SortedDisjointMap
impl<'a, T, V, I> SortedStartsMap<T, V, &'a V> for RangeToRangeValueIter<'a, T, V, I>
where
    T: Integer,
    V: PartialEqClone,
    I: SortedDisjoint<T>,
{
}
impl<'a, T, V, I> SortedDisjointMap<T, V, &'a V> for RangeToRangeValueIter<'a, T, V, I>
where
    T: Integer,
    V: PartialEqClone,
    I: SortedDisjoint<T>,
{
}

macro_rules! impl_sorted_map_traits_and_ops {
    ($IterType:ty, $V:ty, $VR:ty, $($more_generics:tt)*) => {

        #[allow(single_use_lifetimes)]
        impl<$($more_generics)*, T> SortedStartsMap<T, $V, $VR> for $IterType
        where
            T: Integer,
        {
        }

        #[allow(single_use_lifetimes)]
        impl<$($more_generics)*, T> SortedDisjointMap<T, $V, $VR> for $IterType
        where
            T: Integer,
        {
        }

        #[allow(single_use_lifetimes)]
        impl<$($more_generics)*, T> ops::Not for $IterType
        where
            T: Integer,
        {
            type Output = NotIter<T, RangeValuesToRangesIter<T, $V, $VR, Self>>;

            fn not(self) -> Self::Output {
                self.complement_to_set()
            }
        }

        #[allow(single_use_lifetimes)]
        impl<$($more_generics)*, T, R> ops::BitOr<R> for $IterType
        where
            T: Integer,
            R: SortedDisjointMap<T, $V, $VR>,
        {
            type Output = BitOrMapMerge<T, $V, $VR, Self, R>;

            fn bitor(self, other: R) -> Self::Output {
                SortedDisjointMap::union(self, other)
            }
        }

        #[allow(single_use_lifetimes)]
        impl<$($more_generics)*, T, R> ops::Sub<R> for $IterType
        where
            T: Integer,
            R: SortedDisjointMap<T, $V, $VR>,
        {
            type Output = BitSubRangesMap<T, $V, $VR, Self, RangeValuesToRangesIter<T, $V, $VR, R>>;

            fn sub(self, other: R) -> Self::Output {
                SortedDisjointMap::difference(self, other)
            }
        }

        #[allow(single_use_lifetimes)]
        impl<$($more_generics)*, T, R> ops::BitXor<R> for $IterType
        where
            T: Integer,
            R: SortedDisjointMap<T, $V, $VR>,
        {
            type Output = BitXorMapMerge<T, $V, $VR, Self, R>;

            #[allow(clippy::suspicious_arithmetic_impl)]
            fn bitxor(self, other: R) -> Self::Output {
                SortedDisjointMap::symmetric_difference(self, other)
            }
        }

        #[allow(single_use_lifetimes)]
        impl<$($more_generics)*, T, R> ops::BitAnd<R> for $IterType
        where
            T: Integer,
            R: SortedDisjointMap<T, $V, $VR>,
        {
            type Output = BitAndRangesMap<T, $V, $VR, Self, RangeValuesToRangesIter<T, $V, $VR, R>>;

            fn bitand(self, other: R) -> Self::Output {
                SortedDisjointMap::intersection(self, other)
            }
        }

    }
}

// cmk CheckList: Be sure that these are all tested in 'test_every_sorted_disjoint_method'
impl_sorted_map_traits_and_ops!(CheckSortedDisjointMap<T, V, VR, I>, V, VR, V: PartialEqClone, VR: CloneBorrow<V>, I: Iterator<Item = (RangeInclusive<T>,  VR)>);
impl_sorted_map_traits_and_ops!(UnionIterMap<T, V, VR, I>, V, VR, VR: CloneBorrow<V>, V: PartialEqClone, I: PrioritySortedStartsMap<T, V, VR>);
impl_sorted_map_traits_and_ops!(IntersectionIterMap< T, V, VR, I0, I1>, V, VR, V: PartialEqClone, VR: CloneBorrow<V>, I0: SortedDisjointMap<T, V, VR>, I1: SortedDisjoint<T>);
impl_sorted_map_traits_and_ops!(SymDiffIterMap<T, V, VR, I>, V, VR, VR: CloneBorrow<V>, V: PartialEqClone, I: PrioritySortedStartsMap<T, V, VR>);
impl_sorted_map_traits_and_ops!(DynSortedDisjointMap<'a, T, V, VR>, V, VR, 'a, V: PartialEqClone, VR: CloneBorrow<V>);
impl_sorted_map_traits_and_ops!(RangeValuesIter<'a, T, V>, V, &'a V, 'a, V: PartialEqClone);
