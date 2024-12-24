use crate::map::ValueRef;
use crate::merge_map::KMergeMap;
use crate::sorted_disjoint_map::{Priority, PrioritySortedStartsMap};
use crate::{AssumeSortedStarts, MergeMap, SortedDisjointMap, UnionKMergeMap, UnionMergeMap};
use alloc::{collections::BinaryHeap, vec};
use core::cmp::min;
use core::iter::FusedIterator;
use core::ops::RangeInclusive;
use itertools::Itertools;

use crate::unsorted_priority_map::AssumePrioritySortedStartsMap;
use crate::unsorted_priority_map::UnsortedPriorityMap;
use crate::Integer;

type SortedStartsInVecMap<T, VR> =
    AssumePrioritySortedStartsMap<T, VR, vec::IntoIter<Priority<T, VR>>>;
#[allow(clippy::redundant_pub_crate)]
pub(crate) type SortedStartsInVec<T> = AssumeSortedStarts<T, vec::IntoIter<RangeInclusive<T>>>;

/// This `struct` is created by the [`union`] method on [`SortedDisjointMap`]. See [`union`]'s
/// documentation for more.
///
/// [`SortedDisjointMap`]: crate::SortedDisjointMap
/// [`union`]: crate::SortedDisjointMap::union
#[derive(Clone, Debug)]
#[must_use = "iterators are lazy and do nothing unless consumed"]
pub struct UnionIterMap<T, VR, SS>
where
    T: Integer,
    VR: ValueRef,
    SS: PrioritySortedStartsMap<T, VR>,
{
    iter: SS,
    next_item: Option<Priority<T, VR>>,
    workspace: BinaryHeap<Priority<T, VR>>,
    gather: Option<(RangeInclusive<T>, VR)>,
    ready_to_go: Option<(RangeInclusive<T>, VR)>,
}

impl<T, VR, I> Iterator for UnionIterMap<T, VR, I>
where
    T: Integer,
    VR: ValueRef,
    I: PrioritySortedStartsMap<T, VR>,
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
                    self.gather = Some((best.start()..=next_end, best.value().clone()));
                }
            } else {
                // if there is no gather, then set the gather to the best
                self.gather = Some((best.start()..=next_end, best.value().clone()));
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

impl<T, VR, I> UnionIterMap<T, VR, I>
where
    T: Integer,
    VR: ValueRef,
    I: PrioritySortedStartsMap<T, VR>,
{
    #[inline]
    pub(crate) fn new(mut iter: I) -> Self {
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

impl<T, VR, L, R> UnionMergeMap<T, VR, L, R>
where
    T: Integer,
    VR: ValueRef,
    L: SortedDisjointMap<T, VR>,
    R: SortedDisjointMap<T, VR>,
{
    #[inline]
    pub(crate) fn new2(left: L, right: R) -> Self {
        let iter = MergeMap::new(left, right);
        Self::new(iter)
    }
}

impl<T, VR, J> UnionKMergeMap<T, VR, J>
where
    T: Integer,
    VR: ValueRef,
    J: SortedDisjointMap<T, VR>,
{
    #[inline]
    pub(crate) fn new_k<K>(k: K) -> Self
    where
        K: IntoIterator<Item = J>,
    {
        let iter = KMergeMap::new(k);
        Self::new(iter)
    }
}

// from iter (T, VR) to UnionIterMap
impl<T, VR> FromIterator<(RangeInclusive<T>, VR)>
    for UnionIterMap<T, VR, SortedStartsInVecMap<T, VR>>
where
    T: Integer,
    VR: ValueRef,
{
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = (RangeInclusive<T>, VR)>,
    {
        let iter = iter.into_iter();
        let unsorted_priority = UnsortedPriorityMap::new(iter);
        Self::from(unsorted_priority)
    }
}

// from UnsortedDisjointMap to UnionIterMap
impl<T, VR, I> From<UnsortedPriorityMap<T, VR, I>>
    for UnionIterMap<T, VR, SortedStartsInVecMap<T, VR>>
where
    T: Integer,
    VR: ValueRef,
    I: Iterator<Item = (RangeInclusive<T>, VR)>,
{
    #[allow(clippy::clone_on_copy)]
    fn from(unsorted_priority_map: UnsortedPriorityMap<T, VR, I>) -> Self {
        let iter = unsorted_priority_map.sorted_by(|a, b| {
            // We sort only by start -- priority is not used until later.
            a.start().cmp(&b.start())
        });
        let iter = AssumePrioritySortedStartsMap::new(iter);
        Self::new(iter)
    }
}

impl<T, VR, I> FusedIterator for UnionIterMap<T, VR, I>
where
    T: Integer,
    VR: ValueRef,
    I: PrioritySortedStartsMap<T, VR> + FusedIterator,
{
}
