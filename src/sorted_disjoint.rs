use std::{
    iter::FusedIterator,
    ops::{self, RangeInclusive},
};

use crate::{
    BitAndMerge, BitOrMerge, BitSubMerge, BitXOrTee, Integer, NotIter, SortedDisjointIterator,
    SortedStartsIterator,
};

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

impl<T: Integer, I> SortedDisjointIterator<T> for CheckSortedDisjoint<T, I> where
    I: Iterator<Item = RangeInclusive<T>>
{
}
impl<T: Integer, I> SortedStartsIterator<T> for CheckSortedDisjoint<T, I> where
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
    R: SortedDisjointIterator<T>,
{
    type Output = BitOrMerge<T, Self, R>;

    fn bitor(self, other: R) -> Self::Output {
        SortedDisjointIterator::union(self, other)
    }
}

impl<T: Integer, R, L> ops::BitAnd<R> for CheckSortedDisjoint<T, L>
where
    L: Iterator<Item = RangeInclusive<T>>,
    R: SortedDisjointIterator<T>,
{
    type Output = BitAndMerge<T, Self, R>;

    fn bitand(self, other: R) -> Self::Output {
        SortedDisjointIterator::intersection(self, other)
    }
}

impl<T: Integer, R, L> ops::Sub<R> for CheckSortedDisjoint<T, L>
where
    L: Iterator<Item = RangeInclusive<T>>,
    R: SortedDisjointIterator<T>,
{
    type Output = BitSubMerge<T, Self, R>;

    fn sub(self, other: R) -> Self::Output {
        SortedDisjointIterator::difference(self, other)
    }
}

impl<T: Integer, R, L> ops::BitXor<R> for CheckSortedDisjoint<T, L>
where
    L: Iterator<Item = RangeInclusive<T>>,
    R: SortedDisjointIterator<T>,
{
    type Output = BitXOrTee<T, Self, R>;

    fn bitxor(self, other: R) -> Self::Output {
        SortedDisjointIterator::symmetric_difference(self, other)
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
        I: SortedDisjointIterator<T> + 'a,
    {
        Self {
            iter: Box::new(iter),
        }
    }
}

// All DynSortedDisjoint's are SortedDisjoint's
impl<'a, T: Integer> SortedStartsIterator<T> for DynSortedDisjoint<'a, T> {}
impl<'a, T: Integer> SortedDisjointIterator<T> for DynSortedDisjoint<'a, T> {}

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
