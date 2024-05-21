use crate::merge_map::KMergeMap;
use crate::sorted_disjoint_map::{Priority, PrioritySortedStartsMap};
use crate::{BitOrMapKMerge, BitOrMapMerge, MergeMap, SortedDisjointMap};
use alloc::{collections::BinaryHeap, vec};
use core::cmp::min;
use core::iter::FusedIterator;
use core::ops::RangeInclusive;
use itertools::Itertools;

use crate::unsorted_disjoint_map::UnsortedPriorityDisjointMap;
use crate::{map::PartialEqClone, Integer};
use crate::{
    map::{CloneBorrow, SortedStartsInVecMap},
    unsorted_disjoint_map::AssumePrioritySortedStartsMap,
};

/// The output of cmk.
#[derive(Clone, Debug)]
#[must_use = "iterators are lazy and do nothing unless consumed"]
pub struct UnionIterMap<T, V, VR, SS>
where
    T: Integer,
    V: PartialEqClone,
    VR: CloneBorrow<V>,
    SS: PrioritySortedStartsMap<T, V, VR>,
{
    iter: SS,
    next_item: Option<Priority<T, V, VR>>,
    workspace: BinaryHeap<Priority<T, V, VR>>,
    gather: Option<(RangeInclusive<T>, VR)>,
    ready_to_go: Option<(RangeInclusive<T>, VR)>,
}

impl<T, V, VR, I> Iterator for UnionIterMap<T, V, VR, I>
where
    T: Integer,
    V: PartialEqClone,
    VR: CloneBorrow<V>,
    I: PrioritySortedStartsMap<T, V, VR>,
{
    type Item = (RangeInclusive<T>, VR);

    fn next(&mut self) -> Option<(RangeInclusive<T>, VR)> {
        // Keep doing this until we have something to return.
        loop {
            if let Some(value) = self.ready_to_go.take() {
                // If ready_to_go is Some, return the value immediately.
                return Some(value);
            };

            // if self.next_item should go into the workspace, then put it there, get the next, next_item, and loop
            if let Some(next_item) = self.next_item.take() {
                let (next_start, next_end) = next_item.start_and_end();

                // If workspace is empty, just push the next item
                let Some(best) = self.workspace.peek() else {
                    self.workspace.push(next_item);
                    self.next_item = self.iter.next();
                    continue; // return to top of the main processing loop
                };
                // LATER: Could add this special case: If next value is the same as best value and the ending is later, and the start overlaps/touches, then just extend the best value.
                if next_start == best.start() {
                    // Only push if the priority is better or the end is greater
                    if &next_item > best || next_end > best.end() {
                        self.workspace.push(next_item);
                    }
                    self.next_item = self.iter.next();
                    continue; // return to top of the main processing loop
                }

                // It does not go into the workspace, so just hold it and keep processing.
                self.next_item = Some(next_item);
            }

            // If the workspace is empty, we are done.
            let Some(best) = self.workspace.peek() else {
                debug_assert!(self.next_item.is_none());
                debug_assert!(self.ready_to_go.is_none());
                return self.gather.take();
            };

            // We buffer for output the best item up to the start of the next item (if any).

            // Find the start of the next item, if any.
            let next_end = self.next_item.as_ref().map_or_else(
                || best.end(),
                |next_item| min(next_item.start().sub_one(), best.end()),
            );

            // Add the front of best to the gather buffer.
            if let Some(mut gather) = self.gather.take() {
                if gather.1.borrow() == best.value().borrow()
                    && (*gather.0.end()).add_one() == best.start()
                {
                    // if the gather is contiguous with the best, then merge them
                    gather.0 = *gather.0.start()..=next_end;
                    self.gather = Some(gather);
                } else {
                    // if the gather is not contiguous with the best, then output the gather and set the gather to the best
                    self.ready_to_go = Some(gather);
                    self.gather = Some((best.start()..=next_end, best.value().clone_borrow()));
                }
            } else {
                // if there is no gather, then set the gather to the best
                self.gather = Some((best.start()..=next_end, best.value().clone_borrow()));
            };

            // We also update the workspace to removing any items that are completely covered by the new_start.
            // We also don't need to keep any items that have a lower priority and are shorter than the new best.
            let mut new_workspace = BinaryHeap::new();
            while let Some(item) = self.workspace.pop() {
                let mut item = item;
                if item.end() <= next_end {
                    // too short, don't keep
                    continue; // while loop
                }
                item.set_range(next_end.add_one()..=item.end());
                let Some(new_best) = new_workspace.peek() else {
                    // new_workspace is empty, so keep
                    new_workspace.push(item);
                    continue; // while loop
                };
                if &item < new_best && item.end() <= new_best.end() {
                    // item.priority, item.0, new_best.priority, new_best.0);
                    // not as good as new_best, and shorter, so don't keep
                    continue; // while loop
                }

                // higher priority or longer, so keep
                // item.priority, item.0, new_best.priority, new_best.0);
                new_workspace.push(item);
            }
            self.workspace = new_workspace;
        } // end of main loop
    }
}

impl<T, V, VR, I> UnionIterMap<T, V, VR, I>
where
    T: Integer,
    V: PartialEqClone,
    VR: CloneBorrow<V>,
    I: PrioritySortedStartsMap<T, V, VR>,
{
    // cmk fix the comment on the set size. It should say inputs are SortedStarts not SortedDisjoint.
    /// Creates a new [`UnionIterMap`] from zero or more [`crate::sorted_disjoint_map::SortedStartsMap`] iterators. See [`UnionIterMap`] for more details and examples.
    pub fn new(mut iter: I) -> Self {
        let item = iter.next();
        Self {
            iter,
            next_item: item,
            workspace: BinaryHeap::new(),
            gather: None,
            ready_to_go: None,
        }
    }
}

impl<T, V, VR, L, R> BitOrMapMerge<T, V, VR, L, R>
where
    T: Integer,
    V: PartialEqClone,
    VR: CloneBorrow<V>,
    L: SortedDisjointMap<T, V, VR>,
    R: SortedDisjointMap<T, V, VR>,
{
    // cmk fix the comment on the set size. It should say inputs are SortedStarts not SortedDisjoint.
    /// Creates a new [`crate::sym_diff_iter_map::SymDiffIterMap`] from zero or more [`SortedDisjointMap`] iterators. See [`crate::sym_diff_iter_map::SymDiffIterMap`] for more details and examples.
    pub fn new2(left: L, right: R) -> Self {
        let iter = MergeMap::new(left, right);
        Self::new(iter)
    }
}

/// cmk doc
impl<T, V, VR, J> BitOrMapKMerge<T, V, VR, J>
where
    T: Integer,
    V: PartialEqClone,
    VR: CloneBorrow<V>,
    J: SortedDisjointMap<T, V, VR>,
{
    // cmk fix the comment on the set size. It should say inputs are SortedStarts not SortedDisjoint.
    /// Creates a new [`SymDiffIterMap`] from zero or more [`SortedDisjointMap`] iterators. See [`SymDiffIterMap`] for more details and examples.
    ///
    /// [`SymDiffIterMap`]: crate::SymDiffIterMap
    pub fn new_k<K>(k: K) -> Self
    where
        K: IntoIterator<Item = J>,
    {
        let iter = KMergeMap::new(k);
        Self::new(iter)
    }
}

// cmk used?
#[allow(dead_code)]
type SortedRangeValueVec<T, V, VR> =
    AssumePrioritySortedStartsMap<T, V, VR, vec::IntoIter<(RangeInclusive<T>, VR)>>;

// cmk simplify the long types
// from iter (T, VR) to UnionIterMap
impl<T, V, VR> FromIterator<(RangeInclusive<T>, VR)>
    for UnionIterMap<T, V, VR, SortedStartsInVecMap<T, V, VR>>
where
    T: Integer,
    V: PartialEqClone,
    VR: CloneBorrow<V>,
{
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = (RangeInclusive<T>, VR)>,
    {
        UnsortedPriorityDisjointMap::new(iter.into_iter()).into()
    }
}

// from from UnsortedDisjointMap to UnionIterMap
impl<T, V, VR, I> From<UnsortedPriorityDisjointMap<T, V, VR, I>>
    for UnionIterMap<T, V, VR, SortedStartsInVecMap<T, V, VR>>
where
    T: Integer,
    V: PartialEqClone,
    VR: CloneBorrow<V>,
    I: Iterator<Item = (RangeInclusive<T>, VR)>,
{
    #[allow(clippy::clone_on_copy)]
    fn from(unsorted_disjoint: UnsortedPriorityDisjointMap<T, V, VR, I>) -> Self {
        let iter = unsorted_disjoint.sorted_by(|a, b| {
            // We sort only by start -- priority is not used until later.
            a.start().cmp(&b.start())
        });
        let iter = AssumePrioritySortedStartsMap::new(iter);
        Self::new(iter)
    }
}

// cmk0 test that every iterator (that can be) is FusedIterator
impl<T, V, VR, I> FusedIterator for UnionIterMap<T, V, VR, I>
where
    T: Integer,
    V: PartialEqClone,
    VR: CloneBorrow<V>,
    I: PrioritySortedStartsMap<T, V, VR> + FusedIterator,
{
}

#[test]
fn cmk_delete_me5() {
    use crate::prelude::*;

    let a = CheckSortedDisjointMap::new([(1..=2, &"a"), (5..=100, &"a")]);
    let b = CheckSortedDisjointMap::new([(2..=6, &"b")]);
    let union = UnionIterMap::new2(a, b);
    assert_eq!(
        union.into_string(),
        r#"(1..=2, "a"), (3..=4, "b"), (5..=100, "a")"#
    );

    // Or, equivalently:
    let a = CheckSortedDisjointMap::new([(1..=2, &"a"), (5..=100, &"a")]);
    let b = CheckSortedDisjointMap::new([(2..=6, &"b")]);
    let union = a | b;
    assert_eq!(
        union.into_string(),
        r#"(1..=2, "a"), (3..=4, "b"), (5..=100, "a")"#
    );
}
