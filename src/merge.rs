use std::{iter::FusedIterator, ops::RangeInclusive};

use itertools::{Itertools, KMergeBy, MergeBy};

use crate::{sorted_disjoint_iterator::SortedStartsIterator, Integer, SortedDisjointIterator};

/// Works with [`UnionIter`] to turn any number of [`SortedDisjoint`] iterators into a [`SortedDisjoint`] iterator of their union,
/// i.e., all the integers in any input iterator, as sorted & disjoint ranges.
///
/// Also see [`KMerge`].
///
/// [`SortedDisjoint`]: crate::SortedDisjoint
/// [`UnionIter`]: crate::UnionIter
///
/// # Examples
///
/// ```
/// use itertools::Itertools;
/// use range_set_blaze::{UnionIter, Merge, SortedDisjointIterator, CheckSortedDisjoint};
///
/// let a = CheckSortedDisjoint::new(vec![1..=2, 5..=100].into_iter());
/// let b = CheckSortedDisjoint::from([2..=6]);
/// let union = UnionIter::new(Merge::new(a, b));
/// assert_eq!(union.to_string(), "1..=100");
///
/// // Or, equivalently:
/// let a = CheckSortedDisjoint::new(vec![1..=2, 5..=100].into_iter());
/// let b = CheckSortedDisjoint::from([2..=6]);
/// let c = a | b;
/// assert_eq!(c.to_string(), "1..=100")
/// ```
#[derive(Clone, Debug)]
#[must_use = "iterators are lazy and do nothing unless consumed"]
pub struct Merge<T, L, R>
where
    T: Integer,
    L: SortedDisjointIterator<T>,
    R: SortedDisjointIterator<T>,
{
    #[allow(clippy::type_complexity)]
    iter: MergeBy<L, R, fn(&RangeInclusive<T>, &RangeInclusive<T>) -> bool>,
}

impl<T, L, R> Merge<T, L, R>
where
    T: Integer,
    L: SortedDisjointIterator<T>,
    R: SortedDisjointIterator<T>,
{
    /// Creates a new [`Merge`] iterator from two [`SortedDisjoint`] iterators. See [`Merge`] for more details and examples.
    pub fn new(left: L, right: R) -> Self {
        Self {
            iter: left.merge_by(right, |a, b| a.start() < b.start()),
        }
    }
}

impl<T, L, R> FusedIterator for Merge<T, L, R>
where
    T: Integer,
    L: SortedDisjointIterator<T>,
    R: SortedDisjointIterator<T>,
{
}

impl<T, L, R> Iterator for Merge<T, L, R>
where
    T: Integer,
    L: SortedDisjointIterator<T>,
    R: SortedDisjointIterator<T>,
{
    type Item = RangeInclusive<T>;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

impl<T, L, R> SortedStartsIterator<T> for Merge<T, L, R>
where
    T: Integer,
    L: SortedDisjointIterator<T>,
    R: SortedDisjointIterator<T>,
{
}

/// Works with [`UnionIter`] to turn two [`SortedDisjoint`] iterators into a [`SortedDisjoint`] iterator of their union,
/// i.e., all the integers in any input iterator, as sorted & disjoint ranges.
///
/// Also see [`Merge`].
///
/// [`SortedDisjoint`]: crate::SortedDisjoint
/// [`UnionIter`]: crate::UnionIter
///
/// # Examples
///
/// ```
/// use itertools::Itertools;
/// use range_set_blaze::{UnionIter, KMerge, MultiwaySortedDisjoint, SortedDisjointIterator, CheckSortedDisjoint};
///
/// let a = CheckSortedDisjoint::new(vec![1..=2, 5..=100].into_iter());
/// let b = CheckSortedDisjoint::new(vec![2..=6].into_iter());
/// let c = CheckSortedDisjoint::new(vec![-1..=-1].into_iter());
/// let union = UnionIter::new(KMerge::new([a, b, c]));
/// assert_eq!(union.to_string(), "-1..=-1, 1..=100");
///
/// // Or, equivalently:
/// let a = CheckSortedDisjoint::new(vec![1..=2, 5..=100].into_iter());
/// let b = CheckSortedDisjoint::new(vec![2..=6].into_iter());
/// let c = CheckSortedDisjoint::new(vec![-1..=-1].into_iter());
/// let union = [a, b, c].union();
/// assert_eq!(union.to_string(), "-1..=-1, 1..=100");
/// ```
#[derive(Clone, Debug)]
#[must_use = "iterators are lazy and do nothing unless consumed"]
pub struct KMerge<T, I>
where
    T: Integer,
    I: SortedDisjointIterator<T>,
{
    #[allow(clippy::type_complexity)]
    iter: KMergeBy<I, fn(&RangeInclusive<T>, &RangeInclusive<T>) -> bool>,
}

impl<T, I> KMerge<T, I>
where
    T: Integer,
    I: SortedDisjointIterator<T>,
{
    /// Creates a new [`KMerge`] iterator from zero or more [`SortedDisjoint`] iterators. See [`KMerge`] for more details and examples.
    pub fn new<J>(iter: J) -> Self
    where
        J: IntoIterator<Item = I>,
    {
        Self {
            iter: iter.into_iter().kmerge_by(|a, b| a.start() < b.start()),
        }
    }
}

impl<T, I> FusedIterator for KMerge<T, I>
where
    T: Integer,
    I: SortedDisjointIterator<T>,
{
}

impl<T, I> Iterator for KMerge<T, I>
where
    T: Integer,
    I: SortedDisjointIterator<T>,
{
    type Item = RangeInclusive<T>;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

impl<T, I> SortedStartsIterator<T> for KMerge<T, I>
where
    T: Integer,
    I: SortedDisjointIterator<T>,
{
}
