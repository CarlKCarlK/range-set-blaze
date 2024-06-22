use core::{
    cmp::max,
    iter::FusedIterator,
    ops::{self, RangeInclusive},
};

use itertools::Itertools;

use crate::{
    unsorted_disjoint::{AssumeSortedStarts, UnsortedDisjoint},
    BitAndMerge, BitOrMerge, BitSubMerge, BitXOrTee, Integer, NotIter, SortedDisjoint,
    SortedStarts,
};

/// Turns any number of [`SortedDisjoint`] iterators into a [`SortedDisjoint`] iterator of their union,
/// i.e., all the integers in any input iterator, as sorted & disjoint ranges. Uses [`Merge`]
/// or [`KMerge`].
///
/// [`SortedDisjoint`]: crate::SortedDisjoint
/// [`Merge`]: crate::Merge
/// [`KMerge`]: crate::KMerge
///
/// # Examples
///
/// ```
/// use itertools::Itertools;
/// use range_set_blaze::{UnionIter, Merge, SortedDisjoint, CheckSortedDisjoint};
///
/// let a = CheckSortedDisjoint::new(vec![1..=2, 5..=100].into_iter());
/// let b = CheckSortedDisjoint::from([2..=6]);
/// let union = UnionIter::new(Merge::new(a, b));
/// assert_eq!(union.to_string(), "1..=100");
///
/// // Or, equivalently:
/// let a = CheckSortedDisjoint::new(vec![1..=2, 5..=100].into_iter());
/// let b = CheckSortedDisjoint::from([2..=6]);
/// let union = a | b;
/// assert_eq!(union.to_string(), "1..=100")
/// ```
#[derive(Clone, Debug)]
#[must_use = "iterators are lazy and do nothing unless consumed"]
pub struct UnionIter<T, I>
where
    T: Integer,
    I: SortedStarts<T>,
{
    pub(crate) iter: I,
    pub(crate) option_range: Option<RangeInclusive<T>>,
}

impl<T, I> UnionIter<T, I>
where
    T: Integer,
    I: SortedStarts<T>,
{
    /// Creates a new [`UnionIter`] from zero or more [`SortedDisjoint`] iterators. See [`UnionIter`] for more details and examples.
    pub fn new(iter: I) -> Self {
        Self {
            iter,
            option_range: None,
        }
    }
}

impl<T: Integer, const N: usize> From<[T; N]> for UnionIter<T, SortedRangeInclusiveVec<T>> {
    fn from(arr: [T; N]) -> Self {
        arr.as_slice().into()
    }
}

impl<T: Integer> From<&[T]> for UnionIter<T, SortedRangeInclusiveVec<T>> {
    fn from(slice: &[T]) -> Self {
        slice.iter().cloned().collect()
    }
}

impl<T: Integer, const N: usize> From<[RangeInclusive<T>; N]>
    for UnionIter<T, SortedRangeInclusiveVec<T>>
{
    fn from(arr: [RangeInclusive<T>; N]) -> Self {
        arr.as_slice().into()
    }
}

impl<T: Integer> From<&[RangeInclusive<T>]> for UnionIter<T, SortedRangeInclusiveVec<T>> {
    fn from(slice: &[RangeInclusive<T>]) -> Self {
        slice.iter().cloned().collect()
    }
}

type SortedRangeInclusiveVec<T> = AssumeSortedStarts<T, alloc::vec::IntoIter<RangeInclusive<T>>>;

impl<T: Integer> FromIterator<T> for UnionIter<T, SortedRangeInclusiveVec<T>> {
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = T>,
    {
        iter.into_iter().map(|x| x..=x).collect()
    }
}

impl<T: Integer> FromIterator<RangeInclusive<T>> for UnionIter<T, SortedRangeInclusiveVec<T>> {
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = RangeInclusive<T>>,
    {
        UnsortedDisjoint::from(iter.into_iter()).into()
    }
}

impl<T, I> From<UnsortedDisjoint<T, I>> for UnionIter<T, SortedRangeInclusiveVec<T>>
where
    T: Integer,
    I: Iterator<Item = RangeInclusive<T>>, // Any iterator is OK, because we will sort
{
    fn from(unsorted_disjoint: UnsortedDisjoint<T, I>) -> Self {
        let iter = AssumeSortedStarts {
            iter: unsorted_disjoint.sorted_by_key(|range| *range.start()),
        };
        Self {
            iter,
            option_range: None,
        }
    }
}

impl<T: Integer, I> FusedIterator for UnionIter<T, I> where I: SortedStarts<T> + FusedIterator {}

impl<T: Integer, I> Iterator for UnionIter<T, I>
where
    I: SortedStarts<T>,
{
    type Item = RangeInclusive<T>;

    fn next(&mut self) -> Option<RangeInclusive<T>> {
        loop {
            if let Some(range) = self.iter.next() {
                let (start, end) = range.into_inner();
                if end < start {
                    continue;
                }
                if let Some(current_range) = self.option_range.clone() {
                    let (current_start, current_end) = current_range.into_inner();
                    debug_assert!(current_start <= start); // real assert
                    if start <= current_end
                        || (current_end < T::safe_max_value() && start <= current_end + T::one())
                    {
                        self.option_range = Some(current_start..=max(current_end, end));
                        continue;
                    } else {
                        self.option_range = Some(start..=end);
                        return Some(current_start..=current_end);
                    }
                } else {
                    self.option_range = Some(start..=end);
                    continue;
                }
            } else {
                return self.option_range.take();
            }
        }
    }

    // There could be a few as 1 (or 0 if the iter is empty) or as many as the iter.
    // Plus, possibly one more if we have a range is in progress.
    fn size_hint(&self) -> (usize, Option<usize>) {
        let (low, high) = self.iter.size_hint();
        let low = low.min(1);
        if self.option_range.is_some() {
            (low, high.map(|x| x + 1))
        } else {
            (low, high)
        }
    }
}

impl<T: Integer, I> ops::Not for UnionIter<T, I>
where
    I: SortedStarts<T>,
{
    type Output = NotIter<T, Self>;

    fn not(self) -> Self::Output {
        self.complement()
    }
}

impl<T: Integer, R, L> ops::BitOr<R> for UnionIter<T, L>
where
    L: SortedStarts<T>,
    R: SortedDisjoint<T>,
{
    type Output = BitOrMerge<T, Self, R>;

    fn bitor(self, rhs: R) -> Self::Output {
        // It might be fine to optimize to self.iter, but that would require
        // also considering field 'range'
        SortedDisjoint::union(self, rhs)
    }
}

impl<T: Integer, R, L> ops::Sub<R> for UnionIter<T, L>
where
    L: SortedStarts<T>,
    R: SortedDisjoint<T>,
{
    type Output = BitSubMerge<T, Self, R>;

    fn sub(self, rhs: R) -> Self::Output {
        SortedDisjoint::difference(self, rhs)
    }
}

impl<T: Integer, R, L> ops::BitXor<R> for UnionIter<T, L>
where
    L: SortedStarts<T>,
    R: SortedDisjoint<T>,
{
    type Output = BitXOrTee<T, Self, R>;

    #[allow(clippy::suspicious_arithmetic_impl)]
    fn bitxor(self, rhs: R) -> Self::Output {
        SortedDisjoint::symmetric_difference(self, rhs)
    }
}

impl<T: Integer, R, L> ops::BitAnd<R> for UnionIter<T, L>
where
    L: SortedStarts<T>,
    R: SortedDisjoint<T>,
{
    type Output = BitAndMerge<T, Self, R>;

    fn bitand(self, other: R) -> Self::Output {
        SortedDisjoint::intersection(self, other)
    }
}
