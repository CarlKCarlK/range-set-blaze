use core::iter::FusedIterator;

use itertools::{Itertools, KMergeBy, MergeBy};

use crate::Integer;
use crate::map::ValueRef;
use crate::range_values::SetPriorityMap;

use crate::sorted_disjoint_map::{Priority, PrioritySortedStartsMap, SortedDisjointMap};

/// Used internally by `UnionIterMap` and `SymDiffIterMap`.
#[derive(Clone, Debug)]
#[must_use = "iterators are lazy and do nothing unless consumed"]
pub struct MergeMap<T, VR, L, R>
where
    T: Integer,
    VR: ValueRef,
    L: SortedDisjointMap<T, VR>,
    R: SortedDisjointMap<T, VR>,
{
    #[allow(clippy::type_complexity)]
    iter: MergeBy<
        SetPriorityMap<T, VR, L>,
        SetPriorityMap<T, VR, R>,
        fn(&Priority<T, VR>, &Priority<T, VR>) -> bool,
    >,
}

impl<T, VR, L, R> MergeMap<T, VR, L, R>
where
    T: Integer,
    VR: ValueRef,
    L: SortedDisjointMap<T, VR>,
    R: SortedDisjointMap<T, VR>,
{
    pub(crate) fn new(left: L, right: R) -> Self {
        let left = SetPriorityMap::new(left, 0);
        let right = SetPriorityMap::new(right, 1);
        Self {
            // We sort only by start -- priority is not used until later.
            iter: left.merge_by(right, |a, b| a.start() < b.start()),
        }
    }
}

impl<T, VR, L, R> FusedIterator for MergeMap<T, VR, L, R>
where
    T: Integer,
    VR: ValueRef,
    L: SortedDisjointMap<T, VR>,
    R: SortedDisjointMap<T, VR>,
{
}

impl<T, VR, L, R> Iterator for MergeMap<T, VR, L, R>
where
    T: Integer,
    VR: ValueRef,
    L: SortedDisjointMap<T, VR>,
    R: SortedDisjointMap<T, VR>,
{
    type Item = Priority<T, VR>;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

impl<T, VR, L, R> PrioritySortedStartsMap<T, VR> for MergeMap<T, VR, L, R>
where
    T: Integer,
    VR: ValueRef,
    L: SortedDisjointMap<T, VR>,
    R: SortedDisjointMap<T, VR>,
{
}

/// Used internally by `UnionIterMap` and `SymDiffIterMap`.
#[derive(Clone, Debug)]
#[allow(clippy::module_name_repetitions)]
#[must_use = "iterators are lazy and do nothing unless consumed"]
pub struct KMergeMap<T, VR, I>
where
    T: Integer,
    VR: ValueRef,
    I: SortedDisjointMap<T, VR>,
{
    #[allow(clippy::type_complexity)]
    iter: KMergeBy<SetPriorityMap<T, VR, I>, fn(&Priority<T, VR>, &Priority<T, VR>) -> bool>,
}

type KMergeSetPriorityMap<T, VR, I> =
    KMergeBy<SetPriorityMap<T, VR, I>, fn(&Priority<T, VR>, &Priority<T, VR>) -> bool>;

impl<T, VR, I> KMergeMap<T, VR, I>
where
    T: Integer,
    VR: ValueRef,
    I: SortedDisjointMap<T, VR>,
{
    /// Creates a new [`KMergeMap`] iterator from zero or more [`SortedDisjointMap`] iterators. See [`KMergeMap`] for more details and examples.
    ///
    /// [`SortedDisjointMap`]: trait.SortedDisjointMap.html#table-of-contents
    pub(crate) fn new<K>(iter: K) -> Self
    where
        K: IntoIterator<Item = I>,
    {
        // Prioritize from right to left
        let iter = iter.into_iter().enumerate().map(|(i, x)| {
            let priority_number =  i;
            SetPriorityMap::new(x, priority_number)
        });
        // Merge RangeValues by start with ties broken by priority
        let iter: KMergeSetPriorityMap<T, VR, I> = iter.kmerge_by(|a, b| {
            // We sort only by start -- priority is not used until later.
            a.start() < b.start()
        });
        Self { iter }
    }
}

impl<T, VR, I> FusedIterator for KMergeMap<T, VR, I>
where
    T: Integer,
    VR: ValueRef,
    I: SortedDisjointMap<T, VR>,
{
}

impl<T, VR, I> Iterator for KMergeMap<T, VR, I>
where
    T: Integer,
    VR: ValueRef,
    I: SortedDisjointMap<T, VR>,
{
    type Item = Priority<T, VR>;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

impl<T, VR, I> PrioritySortedStartsMap<T, VR> for KMergeMap<T, VR, I>
where
    T: Integer,
    VR: ValueRef,
    I: SortedDisjointMap<T, VR>,
{
}
