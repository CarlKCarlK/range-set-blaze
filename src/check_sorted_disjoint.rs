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

    fn bitor(self, rhs: R) -> Self::Output {
        SortedDisjointIterator::bitor(self, rhs)
    }
}

impl<T: Integer, R, L> ops::BitAnd<R> for CheckSortedDisjoint<T, L>
where
    L: Iterator<Item = RangeInclusive<T>>,
    R: Iterator<Item = RangeInclusive<T>> + SortedDisjoint,
{
    type Output = BitAndMerge<T, Self, R>;

    fn bitand(self, rhs: R) -> Self::Output {
        SortedDisjointIterator::bitand(self, rhs)
    }
}

impl<T: Integer, R, L> ops::Sub<R> for CheckSortedDisjoint<T, L>
where
    L: Iterator<Item = RangeInclusive<T>>,
    R: Iterator<Item = RangeInclusive<T>> + SortedDisjoint,
{
    type Output = BitSubMerge<T, Self, R>;

    fn sub(self, rhs: R) -> Self::Output {
        SortedDisjointIterator::sub(self, rhs)
    }
}

impl<T: Integer, R, L> ops::BitXor<R> for CheckSortedDisjoint<T, L>
where
    L: Iterator<Item = RangeInclusive<T>>,
    R: Iterator<Item = RangeInclusive<T>> + SortedDisjoint,
{
    type Output = BitXOrTee<T, Self, R>;

    fn bitxor(self, rhs: R) -> Self::Output {
        SortedDisjointIterator::bitxor(self, rhs)
    }
}
