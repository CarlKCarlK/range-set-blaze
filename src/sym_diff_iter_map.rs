use core::{
    cmp::{self, min},
    iter::FusedIterator,
    ops::RangeInclusive,
};

use alloc::collections::BinaryHeap;

use crate::{
    Integer, MergeMap, SortedDisjointMap, SymDiffKMergeMap, SymDiffMergeMap,
    map::ValueCarrier,
    merge_map::KMergeMap,
    sorted_disjoint_map::{Priority, PrioritySortedStartsMap},
};

/// This `struct` is created by the [`symmetric_difference`] method on [`SortedDisjointMap`]. See [`symmetric_difference`]'s
/// documentation for more.
///
/// [`SortedDisjointMap`]: crate::SortedDisjointMap
/// [`symmetric_difference`]: crate::SortedDisjointMap::symmetric_difference
#[derive(Clone, Debug)]
#[must_use = "iterators are lazy and do nothing unless consumed"]
pub struct SymDiffIterMap<T, VC, I> {
    iter: I,
    next_item: Option<Priority<T, VC>>,
    workspace: BinaryHeap<Priority<T, VC>>,
    workspace_next_end: Option<T>,
    gather: Option<(RangeInclusive<T>, VC)>,
    ready_to_go: Option<(RangeInclusive<T>, VC)>,
}

#[expect(clippy::ref_option)]
fn min_next_end<T: Integer>(next_end: &Option<T>, next_item_end: T) -> T {
    next_end.map_or_else(
        || next_item_end,
        |current_end| cmp::min(current_end, next_item_end),
    )
}

impl<T, VC, I> FusedIterator for SymDiffIterMap<T, VC, I>
where
    T: Integer,
    VC: ValueCarrier,
    I: PrioritySortedStartsMap<T, VC>,
{
}

impl<T, VC, I> Iterator for SymDiffIterMap<T, VC, I>
where
    T: Integer,
    VC: ValueCarrier,
    I: PrioritySortedStartsMap<T, VC>,
{
    type Item = (RangeInclusive<T>, VC);

    fn next(&mut self) -> Option<(RangeInclusive<T>, VC)> {
        // Keep doing this until we have something to return.
        loop {
            if let Some(value) = self.ready_to_go.take() {
                // If ready_to_go is Some, return the value immediately.
                return Some(value);
            }

            // if self.next_item should go into the workspace, then put it there, get the next, next_item, and loop
            if let Some(next_item) = self.next_item.take() {
                let (next_start, next_end) = next_item.start_and_end();

                // If workspace is empty, just push the next item
                let Some(best) = self.workspace.peek() else {
                    self.workspace_next_end =
                        Some(min_next_end(&self.workspace_next_end, next_end));
                    self.workspace.push(next_item);
                    self.next_item = self.iter.next();
                    continue; // return to top of the main processing loop
                };
                let (best_range, _) = best.range_value();
                if next_start == *best_range.start() {
                    // Always push (this differs from UnionIterMap)
                    self.workspace_next_end =
                        Some(min_next_end(&self.workspace_next_end, next_end));
                    self.workspace.push(next_item);
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
            let (best_range, best_value) = best.range_value();

            // We buffer for output the best item up to the start of the next item (if any).

            // Find the start of the next item, if any.
            let mut next_end = self
                .workspace_next_end
                .take()
                .expect("Real Assert: safe because we know the workspace is not empty");
            if let Some(next_item) = self.next_item.as_ref() {
                next_end = min(next_item.start().sub_one(), next_end);
            }

            // Add the front of best to the gather buffer.
            if let Some((mut gather_range, gather_value)) = self.gather.take() {
                if gather_value.value_eq(best_value)
                    && (*gather_range.end()).add_one() == *best_range.start()
                {
                    if self.workspace.len().is_odd() {
                        // if the gather is contiguous with the best, then merge them
                        gather_range = *gather_range.start()..=next_end;
                        self.gather = Some((gather_range, gather_value));
                    } else {
                        // if an even number of items in the workspace, then flush the gather
                        self.ready_to_go = Some((gather_range, gather_value));
                        debug_assert!(self.gather.is_none());
                    }
                } else {
                    // if the gather is not contiguous with the best, then output the gather and set the gather to the best
                    self.ready_to_go = Some((gather_range, gather_value));
                    // FYI: this code appear twice # 1 of 2
                    if self.workspace.len().is_odd() {
                        self.gather = Some((*best_range.start()..=next_end, best_value.clone()));
                    } else {
                        debug_assert!(self.gather.is_none());
                    }
                }
            } else {
                // if there is no gather, then set the gather to the best
                // FYI: this code appear twice # 2 of 2
                if self.workspace.len().is_odd() {
                    self.gather = Some((*best_range.start()..=next_end, best_value.clone()));
                } else {
                    debug_assert!(self.gather.is_none());
                }
            }

            // We also update the workspace to removing any items that are completely covered by the new_start.
            // (Unlike UnionIterMap, we must keep any items that have a lower priority and are shorter than the new best.)
            self.workspace_next_end = None;
            self.workspace = self
                .workspace
                .drain()
                .filter(|item| item.end() > next_end)
                .map(|mut item| {
                    item.set_range(next_end.add_one()..=item.end());
                    self.workspace_next_end =
                        Some(min_next_end(&self.workspace_next_end, item.end()));
                    item
                })
                .collect();
        } // end of main loop
    }
}

impl<T, VC, L, R> SymDiffMergeMap<T, VC, L, R>
where
    T: Integer,
    VC: ValueCarrier,
    L: SortedDisjointMap<T, VC>,
    R: SortedDisjointMap<T, VC>,
{
    #[inline]
    pub(crate) fn new2(left: L, right: R) -> Self {
        let iter = MergeMap::new(left, right);
        Self::new(iter)
    }
}

impl<T, VC, J> SymDiffKMergeMap<T, VC, J>
where
    T: Integer,
    VC: ValueCarrier,
    J: SortedDisjointMap<T, VC>,
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

impl<T, VC, I> SymDiffIterMap<T, VC, I>
where
    T: Integer,
    VC: ValueCarrier,
    I: PrioritySortedStartsMap<T, VC>,
{
    #[inline]
    pub(crate) fn new(mut iter: I) -> Self {
        let item = iter.next();
        Self {
            iter,
            next_item: item,
            workspace: BinaryHeap::new(),
            workspace_next_end: None,
            gather: None,
            ready_to_go: None,
        }
    }
}

#[allow(clippy::wrong_self_convention)]
#[allow(clippy::redundant_pub_crate)]
pub(crate) trait UsizeExtensions {
    fn is_odd(self) -> bool;
}

impl UsizeExtensions for usize {
    #[inline]
    fn is_odd(self) -> bool {
        self & 1 != 0
    }
}
