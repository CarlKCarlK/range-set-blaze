use std::{
    iter::FusedIterator,
    ops::{self, RangeInclusive},
};

use itertools::Itertools;

use crate::{
    BitAndMerge, BitOrMerge, BitSubMerge, BitXOrTee, Integer, Merge, NotIter, RangeSetBlaze,
    UnionIter,
};

/// cmk000
pub trait SortedStarts<T: Integer>: Iterator<Item = RangeInclusive<T>> + Sized {}

/// The trait used to provide methods common to iterators with the [`SortedDisjoint`] trait.
/// Methods include [`to_string`], [`equal`], [`union`], [`intersection`]
/// [`symmetric_difference`], [`difference`], [`complement`].
///
/// [`to_string`]: SortedDisjoint::to_string
/// [`equal`]: SortedDisjoint::equal
/// [`union`]: SortedDisjoint::union
/// [`intersection`]: SortedDisjoint::intersection
/// [`symmetric_difference`]: SortedDisjoint::symmetric_difference
/// [`difference`]: SortedDisjoint::difference
/// [`complement`]: SortedDisjoint::complement
pub trait SortedDisjoint<T: Integer>: SortedStarts<T> + Sized {
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
    fn complement(self) -> NotIter<T, Self> {
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
    {
        let (lhs0, lhs1) = self.tee();
        let (rhs0, rhs1) = other.into_iter().tee();
        lhs0.difference(rhs0) | rhs1.difference(lhs1)
    }

    // todo rule: Prefer IntoIterator to Iterator
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
    {
        itertools::equal(self, other)
    }

    // todo rule: You can't define traits on combinations of traits, so use this method to define methods on traits
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
    fn to_string(self) -> String {
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
    fn is_empty(mut self) -> bool {
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
    /// todo rule which functions should be must_use? iterator, constructor, predicates, first, last,
    #[must_use]
    #[inline]
    #[allow(clippy::wrong_self_convention)]
    fn is_disjoint<R>(self, other: R) -> bool
    where
        R: IntoIterator<Item = Self::Item>,
        R::IntoIter: SortedDisjoint<T>,
    {
        self.intersection(other).is_empty()
    }

    /// cmk000
    fn into_range_set_blaze(self) -> RangeSetBlaze<T>
    where
        T: Integer,
    {
        RangeSetBlaze::from_cmk(self)
    }
}

// cmk
// // todo rule: You can't define traits on combinations of traits, so use this method to define methods on traits
// impl<T, I> SortedDisjoint<T> for I
// where
//     T: Integer,
//     I: Iterator<Item = RangeInclusive<T>> + SortedDisjoint,
// {
// }

#[derive(Clone)]
#[must_use = "iterators are lazy and do nothing unless consumed"]

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

#[derive(Debug)]
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

impl<T> Default for CheckSortedDisjoint<T, std::array::IntoIter<RangeInclusive<T>, 0>>
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

    // !!!todo rule add a size hint, but think about if it is correct with respect to other fields
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

impl<T: Integer, const N: usize> From<[RangeInclusive<T>; N]>
    for CheckSortedDisjoint<T, std::array::IntoIter<RangeInclusive<T>, N>>
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

impl<T, I> ops::Not for CheckSortedDisjoint<T, I>
where
    T: Integer,
    I: Iterator<Item = RangeInclusive<T>>,
{
    type Output = NotIter<T, Self>;

    fn not(self) -> Self::Output {
        NotIter::new(self)
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

#[must_use = "iterators are lazy and do nothing unless consumed"]
/// Gives [`SortedDisjoint`] iterators a uniform type. Used by the [`union_dyn`] and [`intersection_dyn`] macros to give all
/// their input iterators the same type.
///
/// [`union_dyn`]: crate::union_dyn
/// [`intersection_dyn`]: crate::intersection_dyn
///
/// # Example
/// ```
/// use range_set_blaze::prelude::*;
///
/// let a = RangeSetBlaze::from_iter([1u8..=6, 8..=9, 11..=15]);
/// let b = RangeSetBlaze::from_iter([5..=13, 18..=29]);
/// let c = RangeSetBlaze::from_iter([38..=42]);
/// let union = [
///     DynSortedDisjoint::new(a.ranges()),
///     DynSortedDisjoint::new(!b.ranges()),
///     DynSortedDisjoint::new(c.ranges()),
/// ]
/// .union();
/// assert_eq!(union.to_string(), "0..=6, 8..=9, 11..=17, 30..=255");
/// ```

pub struct DynSortedDisjoint<'a, T: Integer> {
    iter: Box<dyn Iterator<Item = RangeInclusive<T>> + 'a>,
}

impl<'a, T: Integer> DynSortedDisjoint<'a, T> {
    /// Create a [`DynSortedDisjoint`] from any [`SortedDisjoint`] iterator. See [`DynSortedDisjoint`] for an example.
    pub fn new<I>(iter: I) -> Self
    where
        I: SortedDisjoint<T> + 'a,
    {
        Self {
            iter: Box::new(iter),
        }
    }
}

// All DynSortedDisjoint's are SortedDisjoint's
impl<'a, T: Integer> SortedStarts<T> for DynSortedDisjoint<'a, T> {}
impl<'a, T: Integer> SortedDisjoint<T> for DynSortedDisjoint<'a, T> {}

impl<'a, T: Integer> Iterator for DynSortedDisjoint<'a, T> {
    type Item = RangeInclusive<T>;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }

    // todo rule Implement size_hint if possible and ExactSizeIterator if possible
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

/// Intersects one or more [`SortedDisjoint`] iterators, creating a new [`SortedDisjoint`] iterator.
/// The input iterators need not to be of the same type.
///
/// For input iterators of the same type, [`intersection`] may be slightly faster.
///
/// # Performance
///   All work is done on demand, in one pass through the input iterators. Minimal memory is used.
///
/// # Example: 3-Input Parity
///
/// Find the integers that appear an odd number of times in the [`SortedDisjoint`] iterators.
///
/// [`intersection`]: crate::MultiwaySortedDisjoint::intersection
/// ```
/// use range_set_blaze::prelude::*;
///
/// let a = RangeSetBlaze::from_iter([1..=6, 8..=9, 11..=15]);
/// let b = RangeSetBlaze::from_iter([5..=13, 18..=29]);
/// let c = RangeSetBlaze::from_iter([38..=42]);
///
/// let parity = union_dyn!(
///     intersection_dyn!(a.ranges(), !b.ranges(), !c.ranges()),
///     intersection_dyn!(!a.ranges(), b.ranges(), !c.ranges()),
///     intersection_dyn!(!a.ranges(), !b.ranges(), c.ranges()),
///     intersection_dyn!(a.ranges(), b.ranges(), c.ranges())
/// );
/// assert_eq!(
///     parity.to_string(),
///     "1..=4, 7..=7, 10..=10, 14..=15, 18..=29, 38..=42"
/// );
/// ```
#[macro_export]
macro_rules! intersection_dyn {
    ($($val:expr),*) => {$crate::MultiwaySortedDisjoint::intersection([$($crate::DynSortedDisjoint::new($val)),*])}
}

/// Unions one or more [`SortedDisjoint`] iterators, creating a new [`SortedDisjoint`] iterator.
/// The input iterators need not to be of the same type.
///
/// For input iterators of the same type, [`union`] may be slightly faster.
///
/// # Performance
///   All work is done on demand, in one pass through the input iterators. Minimal memory is used.
///
/// # Example: 3-Input Parity
///
/// Find the integers that appear an odd number of times in the [`SortedDisjoint`] iterators.
///
/// [`union`]: crate::MultiwaySortedDisjoint::union
/// ```
/// use range_set_blaze::prelude::*;
///
/// let a = RangeSetBlaze::from_iter([1..=6, 8..=9, 11..=15]);
/// let b = RangeSetBlaze::from_iter([5..=13, 18..=29]);
/// let c = RangeSetBlaze::from_iter([38..=42]);
///
/// let parity = union_dyn!(
///     intersection_dyn!(a.ranges(), !b.ranges(), !c.ranges()),
///     intersection_dyn!(!a.ranges(), b.ranges(), !c.ranges()),
///     intersection_dyn!(!a.ranges(), !b.ranges(), c.ranges()),
///     intersection_dyn!(a.ranges(), b.ranges(), c.ranges())
/// );
/// assert_eq!(
///     parity.to_string(),
///     "1..=4, 7..=7, 10..=10, 14..=15, 18..=29, 38..=42"
/// );
/// ```
#[macro_export]
macro_rules! union_dyn {
    ($($val:expr),*) => {
                        $crate::MultiwaySortedDisjoint::union([$($crate::DynSortedDisjoint::new($val)),*])
                        }
}
