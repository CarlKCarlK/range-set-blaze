use core::{iter::FusedIterator, ops::RangeInclusive};

use itertools::{Itertools, KMergeBy, MergeBy};

use crate::{Integer, SortedDisjoint, SortedStarts};

/// Used internally by `UnionIter` and `SymDiffIter`.
#[must_use = "iterators are lazy and do nothing unless consumed"]
#[derive(Clone, Debug)]
pub struct Merge<T, L, R>
where
    T: Integer,
    L: SortedDisjoint<T>,
    R: SortedDisjoint<T>,
{
    #[allow(clippy::type_complexity)]
    iter: MergeBy<L, R, fn(&RangeInclusive<T>, &RangeInclusive<T>) -> bool>,
}

impl<T, L, R> Merge<T, L, R>
where
    T: Integer,
    L: SortedDisjoint<T>,
    R: SortedDisjoint<T>,
{
    /// Creates a new [`Merge`] iterator from two [`SortedDisjoint`] iterators. See [`Merge`] for more details and examples.
    ///
    /// [SortedDisjoint]: crate::SortedDisjoint.html#table-of-contents
    #[inline]
    pub(crate) fn new(left: L, right: R) -> Self {
        Self {
            iter: left.merge_by(right, |a, b| a.start() < b.start()),
        }
    }
}

impl<T, L, R> FusedIterator for Merge<T, L, R>
where
    T: Integer,
    L: SortedDisjoint<T>,
    R: SortedDisjoint<T>,
{
}

impl<T, L, R> Iterator for Merge<T, L, R>
where
    T: Integer,
    L: SortedDisjoint<T>,
    R: SortedDisjoint<T>,
{
    type Item = RangeInclusive<T>;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

impl<T, L, R> SortedStarts<T> for Merge<T, L, R>
where
    T: Integer,
    L: SortedDisjoint<T>,
    R: SortedDisjoint<T>,
{
}

/// Used internally by `UnionIter` and `SymDiffIter`.
#[derive(Clone, Debug)]
#[allow(clippy::module_name_repetitions)]
#[must_use = "iterators are lazy and do nothing unless consumed"]
pub struct KMerge<T, I>
where
    T: Integer,
    I: SortedDisjoint<T>,
{
    #[allow(clippy::type_complexity)]
    iter: KMergeBy<I, fn(&RangeInclusive<T>, &RangeInclusive<T>) -> bool>,
}

type RangeMergeIter<T, I> = KMergeBy<I, fn(&RangeInclusive<T>, &RangeInclusive<T>) -> bool>;

impl<T, I> KMerge<T, I>
where
    T: Integer,
    I: SortedDisjoint<T>,
{
    pub(crate) fn new<K>(iter: K) -> Self
    where
        K: IntoIterator<Item = I>,
    {
        let iter = iter.into_iter();
        // Merge RangeValues by start with ties broken by priority
        let iter: RangeMergeIter<T, I> = iter.kmerge_by(|a, b| a.start() < b.start());
        Self { iter }
    }
}

impl<T, I> FusedIterator for KMerge<T, I>
where
    T: Integer,
    I: SortedDisjoint<T>,
{
}

impl<T, I> Iterator for KMerge<T, I>
where
    T: Integer,
    I: SortedDisjoint<T>,
{
    type Item = RangeInclusive<T>;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

impl<T, I> SortedStarts<T> for KMerge<T, I>
where
    T: Integer,
    I: SortedDisjoint<T>,
{
}
