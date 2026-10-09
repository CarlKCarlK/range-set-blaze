use core::iter::FusedIterator;
use core::ops::RangeInclusive;

use itertools::{Itertools, MergeBy};

use crate::Integer;
use crate::map::ValueCarrier;
use crate::range_values::SetPriorityMap;

use crate::sorted_disjoint_map::{Priority, PrioritySortedStartsMap, SortedDisjointMap};

/// Used internally by `UnionIterMap` and `SymDiffIterMap`.
#[derive(Clone, Debug)]
#[must_use = "iterators are lazy and do nothing unless consumed"]
pub struct MergeMap<
    T,
    VC,
    L: Iterator<Item = (RangeInclusive<T>, VC)>,
    R: Iterator<Item = (RangeInclusive<T>, VC)>,
> {
    #[allow(clippy::type_complexity)]
    iter: MergeBy<
        SetPriorityMap<T, VC, L>,
        SetPriorityMap<T, VC, R>,
        fn(&Priority<T, VC>, &Priority<T, VC>) -> bool,
    >,
}

impl<T, VC, L, R> MergeMap<T, VC, L, R>
where
    T: Integer,
    VC: ValueCarrier,
    L: SortedDisjointMap<T, VC>,
    R: SortedDisjointMap<T, VC>,
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

impl<T, VC, L, R> FusedIterator for MergeMap<T, VC, L, R>
where
    T: Integer,
    VC: ValueCarrier,
    L: SortedDisjointMap<T, VC>,
    R: SortedDisjointMap<T, VC>,
{
}

impl<T, VC, L, R> Iterator for MergeMap<T, VC, L, R>
where
    T: Integer,
    VC: ValueCarrier,
    L: SortedDisjointMap<T, VC>,
    R: SortedDisjointMap<T, VC>,
{
    type Item = Priority<T, VC>;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

impl<T, VC, L, R> PrioritySortedStartsMap<T, VC> for MergeMap<T, VC, L, R>
where
    T: Integer,
    VC: ValueCarrier,
    L: SortedDisjointMap<T, VC>,
    R: SortedDisjointMap<T, VC>,
{
}
