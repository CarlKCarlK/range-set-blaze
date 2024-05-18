use core::{
    cmp::{self, min},
    iter::FusedIterator,
    ops::RangeInclusive,
};

use alloc::collections::BinaryHeap;

use crate::{
    map::{CloneBorrow, ValueOwned},
    merge_map::KMergeMap,
    sorted_disjoint_map::{Priority, PrioritySortedStartsMap},
    BitXorMapKMerge, BitXorMapMerge, Integer, MergeMap, SortedDisjointMap,
};

/// Turns any number of [`SortedDisjointMap`] iterators into a [`SortedDisjointMap`] iterator of their union,
/// i.e., all the integers in any input iterator, as sorted & disjoint ranges. Uses [`Merge`]
/// or [`KMerge`].
///
/// [`SortedDisjointMap`]: crate::SortedDisjointMap
/// [`Merge`]: crate::merge::Merge
/// [`KMerge`]: crate::merge::KMerge
///
/// # Examples
///
/// ```
/// use itertools::Itertools;
/// use range_set_blaze::{prelude::*, SymDiffIterMap};
///
/// let a = CheckSortedDisjointMap::new([(1..=2, &"a"), (5..=100, &"a")]);
/// let b = CheckSortedDisjointMap::new([(2..=6, &"b")]);
/// let sym_diff = SymDiffIterMap::new2(a, b);
/// assert_eq!(sym_diff.into_string(), r#"(1..=1, "a"), (3..=4, "b"), (7..=100, "a")"#);
///
/// // Or, equivalently:
/// let a = CheckSortedDisjointMap::new([(1..=2, &"a"), (5..=100, &"a")]);
/// let b = CheckSortedDisjointMap::new([(2..=6, &"b")]);
/// let sym_diff = a ^ b;
/// assert_eq!(sym_diff.into_string(),r#"(1..=1, "a"), (3..=4, "b"), (7..=100, "a")"#);
/// ```
// cmk #[derive(Clone, Debug)]
#[must_use = "iterators are lazy and do nothing unless consumed"]
pub struct SymDiffIterMap<T, V, VR, I>
where
    T: Integer,
    V: ValueOwned,
    VR: CloneBorrow<V>,
    I: PrioritySortedStartsMap<T, V, VR>,
{
    iter: I,
    next_item: Option<Priority<T, V, VR>>,
    workspace: BinaryHeap<Priority<T, V, VR>>,
    workspace_next_end: Option<T>,
    gather: Option<(RangeInclusive<T>, VR)>,
    ready_to_go: Option<(RangeInclusive<T>, VR)>,
}

fn min_next_end<T>(next_end: &Option<T>, next_item_end: T) -> Option<T>
where
    T: Integer,
{
    Some(next_end.map_or_else(
        || next_item_end,
        |current_end| cmp::min(current_end, next_item_end),
    ))
}

impl<T, V, VR, I> FusedIterator for SymDiffIterMap<T, V, VR, I>
where
    T: Integer,
    V: ValueOwned,
    VR: CloneBorrow<V>,
    I: PrioritySortedStartsMap<T, V, VR>,
{
}

impl<T, V, VR, I> Iterator for SymDiffIterMap<T, V, VR, I>
where
    T: Integer,
    V: ValueOwned,
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
                    self.workspace_next_end = min_next_end(&self.workspace_next_end, next_end);
                    self.workspace.push(next_item);
                    self.next_item = self.iter.next();
                    continue; // return to top of the main processing loop
                };
                let best = best.range_value();
                if next_start == *best.0.start() {
                    // Always push (this differs from UnionIterMap)
                    self.workspace_next_end = min_next_end(&self.workspace_next_end, next_end);
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
            let best = best.range_value();

            // We buffer for output the best item up to the start of the next item (if any).

            // Find the start of the next item, if any.
            // unwrap() is safe because we know the workspace is not empty
            let mut next_end = self.workspace_next_end.take().unwrap();
            if let Some(next_item) = self.next_item.as_ref() {
                next_end = min(next_item.start().sub_one(), next_end);
            }

            // Add the front of best to the gather buffer.
            if let Some(mut gather) = self.gather.take() {
                if gather.1.borrow() == best.1.borrow()
                    && (*gather.0.end()).add_one() == *best.0.start()
                {
                    if self.workspace.len() % 2 == 1 {
                        // if the gather is contiguous with the best, then merge them
                        gather.0 = *gather.0.start()..=next_end;
                        self.gather = Some(gather);
                    } else {
                        // if an even number of items in the workspace, then flush the gather
                        self.ready_to_go = Some(gather);
                        debug_assert!(self.gather.is_none());
                    }
                } else {
                    // if the gather is not contiguous with the best, then output the gather and set the gather to the best
                    self.ready_to_go = Some(gather);
                    // cmk this code appear twice
                    if self.workspace.len() % 2 == 1 {
                        self.gather = Some((*best.0.start()..=next_end, best.1.clone_borrow()));
                    } else {
                        debug_assert!(self.gather.is_none());
                    }
                }
            } else {
                // if there is no gather, then set the gather to the best
                if self.workspace.len() % 2 == 1 {
                    self.gather = Some((*best.0.start()..=next_end, best.1.clone_borrow()));
                } else {
                    debug_assert!(self.gather.is_none());
                }
            };

            // We also update the workspace to removing any items that are completely covered by the new_start.
            // (Unlike UnionIterMap, we must keep any items that have a lower priority and are shorter than the new best.)
            // cmk use .filter() ?
            let mut new_workspace = BinaryHeap::new();
            let mut new_next_end = None;
            while let Some(item) = self.workspace.pop() {
                let mut item = item;
                if item.end() <= next_end {
                    // too short, don't keep
                    continue; // while loop
                }
                item.set_range(next_end.add_one()..=item.end());
                new_next_end = min_next_end(&new_next_end, item.end());
                new_workspace.push(item);
            }
            self.workspace = new_workspace;
            self.workspace_next_end = new_next_end;
        } // end of main loop
    }
}

impl<T, V, VR, L, R> BitXorMapMerge<T, V, VR, L, R>
where
    T: Integer,
    V: ValueOwned,
    VR: CloneBorrow<V>,
    L: SortedDisjointMap<T, V, VR>,
    R: SortedDisjointMap<T, V, VR>,
{
    // cmk fix the comment on the set size. It should say inputs are SortedStarts not SortedDisjoint.
    /// Creates a new [`SymDiffIterMap`] from zero or more [`SortedDisjointMap`] iterators. See [`SymDiffIterMap`] for more details and examples.
    pub fn new2(left: L, right: R) -> Self {
        let iter = MergeMap::new(left, right);
        Self::new(iter)
    }
}

/// cmk doc
impl<T, V, VR, J> BitXorMapKMerge<T, V, VR, J>
where
    T: Integer,
    V: ValueOwned,
    VR: CloneBorrow<V>,
    J: SortedDisjointMap<T, V, VR>,
{
    // cmk fix the comment on the set size. It should say inputs are SortedStarts not SortedDisjoint.
    /// Creates a new [`SymDiffIterMap`] from zero or more [`SortedDisjointMap`] iterators. See [`SymDiffIterMap`] for more details and examples.
    pub fn new_k<K>(k: K) -> Self
    where
        K: IntoIterator<Item = J>,
    {
        let iter = KMergeMap::new(k);
        Self::new(iter)
    }
}

impl<T, V, VR, I> SymDiffIterMap<T, V, VR, I>
where
    T: Integer,
    V: ValueOwned,
    VR: CloneBorrow<V>,
    I: PrioritySortedStartsMap<T, V, VR>,
{
    /// Creates a new [`SymDiffIterMap`] from zero or more [`SortedDisjointMap`] iterators.
    /// See [`SymDiffIterMap`] for more details and examples.
    pub fn new(mut iter: I) -> Self {
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

#[test]
fn cmk_delete_me6() {
    use crate::{prelude::*, SymDiffIterMap};

    let a = CheckSortedDisjointMap::new([(1..=2, &"a"), (5..=100, &"a")]);
    let b = CheckSortedDisjointMap::new([(2..=6, &"b")]);
    let sym_diff = SymDiffIterMap::new2(a, b);
    assert_eq!(
        sym_diff.into_string(),
        r#"(1..=1, "a"), (3..=4, "b"), (7..=100, "a")"#
    );

    // Or, equivalently:
    let a = CheckSortedDisjointMap::new([(1..=2, &"a"), (5..=100, &"a")]);
    let b = CheckSortedDisjointMap::new([(2..=6, &"b")]);
    let sym_diff = a ^ b;
    assert_eq!(
        sym_diff.into_string(),
        r#"(1..=1, "a"), (3..=4, "b"), (7..=100, "a")"#
    );
}
