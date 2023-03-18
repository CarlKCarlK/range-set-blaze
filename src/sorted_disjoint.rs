use std::ops::{self, RangeInclusive};

use crate::{
    BitAndMerge, BitOrMerge, BitSubMerge, BitXOrTee, Integer, NotIter, SortedDisjoint,
    SortedDisjointIterator, SortedStarts,
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
/// use range_set_int::{CheckSortedDisjoint, SortedDisjointIterator};
///
/// let a = CheckSortedDisjoint::new([1..=2, 5..=100].into_iter());
/// let b = CheckSortedDisjoint::new([2..=6].into_iter());
/// let union = a | b;
/// assert_eq!(union.to_string(), "1..=100");
/// ```
///
/// Here the ranges are not sorted and disjoint, so the iterator will panic.
///```should_panic
/// use range_set_int::{CheckSortedDisjoint, SortedDisjointIterator};
///
/// let a = CheckSortedDisjoint::new([1..=5, 5..=100].into_iter());
/// let b = CheckSortedDisjoint::new([2..=6,-10..=-5].into_iter());
/// let union = a | b;
/// assert_eq!(union.to_string(), "1..=100");
/// ```

pub struct CheckSortedDisjoint<T, I>
where
    T: Integer,
    I: Iterator<Item = RangeInclusive<T>>,
{
    pub(crate) iter: I,
    prev_end: Option<T>,
    seen_none: bool,
}

impl<T: Integer, I> SortedDisjoint for CheckSortedDisjoint<T, I> where
    I: Iterator<Item = RangeInclusive<T>>
{
}
impl<T: Integer, I> SortedStarts for CheckSortedDisjoint<T, I> where
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

impl<T, I> Iterator for CheckSortedDisjoint<T, I>
where
    T: Integer,
    I: Iterator<Item = RangeInclusive<T>>,
{
    type Item = RangeInclusive<T>;

    //cmk coverage test every panic
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
            //cmk give safe_max_value a better name and do a text search
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

    // !!!cmk rule add a size hint, but think about if it is correct with respect to other fields
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
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
    R: Iterator<Item = RangeInclusive<T>> + SortedDisjoint,
{
    type Output = BitOrMerge<T, Self, R>;

    fn bitor(self, other: R) -> Self::Output {
        SortedDisjointIterator::bitor(self, other)
    }
}

impl<T: Integer, R, L> ops::BitAnd<R> for CheckSortedDisjoint<T, L>
where
    L: Iterator<Item = RangeInclusive<T>>,
    R: Iterator<Item = RangeInclusive<T>> + SortedDisjoint,
{
    type Output = BitAndMerge<T, Self, R>;

    fn bitand(self, other: R) -> Self::Output {
        SortedDisjointIterator::bitand(self, other)
    }
}

impl<T: Integer, R, L> ops::Sub<R> for CheckSortedDisjoint<T, L>
where
    L: Iterator<Item = RangeInclusive<T>>,
    R: Iterator<Item = RangeInclusive<T>> + SortedDisjoint,
{
    type Output = BitSubMerge<T, Self, R>;

    fn sub(self, other: R) -> Self::Output {
        SortedDisjointIterator::sub(self, other)
    }
}

impl<T: Integer, R, L> ops::BitXor<R> for CheckSortedDisjoint<T, L>
where
    L: Iterator<Item = RangeInclusive<T>>,
    R: Iterator<Item = RangeInclusive<T>> + SortedDisjoint,
{
    type Output = BitXOrTee<T, Self, R>;

    fn bitxor(self, other: R) -> Self::Output {
        SortedDisjointIterator::bitxor(self, other)
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
/// use range_set_int::{DynSortedDisjoint, MultiwaySortedDisjoint, SortedDisjointIterator, RangeSetInt};
///
/// let a = RangeSetInt::from([1u8..=6, 8..=9, 11..=15]);
/// let b = RangeSetInt::from([5..=13, 18..=29]);
/// let c = RangeSetInt::from([38..=42]);
/// let union = [
///     DynSortedDisjoint::new(a.ranges()),
///     DynSortedDisjoint::new(!b.ranges()),
///     DynSortedDisjoint::new(c.ranges()),
/// ]
/// .union();
/// assert_eq!(union.to_string(), "0..=6, 8..=9, 11..=17, 30..=255");
/// ```

pub struct DynSortedDisjoint<'a, T> {
    iter: Box<dyn Iterator<Item = T> + 'a>,
}

impl<'a, T> DynSortedDisjoint<'a, T> {
    /// Create a [`DynSortedDisjoint`] from any [`SortedDisjoint`] iterator. See [`DynSortedDisjoint`] for an example.
    pub fn new<I>(iter: I) -> Self
    where
        I: Iterator<Item = T> + SortedDisjoint + 'a,
    {
        Self {
            iter: Box::new(iter),
        }
    }
}

// All DynSortedDisjoint's are SortedDisjoint's
impl<'a, T> SortedStarts for DynSortedDisjoint<'a, T> {}
impl<'a, T> SortedDisjoint for DynSortedDisjoint<'a, T> {}

impl<'a, T> Iterator for DynSortedDisjoint<'a, T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }

    // cmk rule Implement size_hint if possible and ExactSizeIterator if possible
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
/// use range_set_int::{intersection_dyn, union_dyn, RangeSetInt, SortedDisjointIterator};
///
/// let a = RangeSetInt::from([1..=6, 8..=9, 11..=15]);
/// let b = RangeSetInt::from([5..=13, 18..=29]);
/// let c = RangeSetInt::from([38..=42]);
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
/// use range_set_int::{intersection_dyn, union_dyn, RangeSetInt, SortedDisjointIterator};
///
/// let a = RangeSetInt::from([1..=6, 8..=9, 11..=15]);
/// let b = RangeSetInt::from([5..=13, 18..=29]);
/// let c = RangeSetInt::from([38..=42]);
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
