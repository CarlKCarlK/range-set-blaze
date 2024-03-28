use core::cmp::min;

use alloc::collections::BinaryHeap;

use crate::{
    map::{CloneBorrow, ValueOwned},
    merge_map::KMergeMap,
    sorted_disjoint_map::{Priority, PrioritySortedStartsMap},
    Integer, MergeMap, RangeValue, SortedDisjointMap, SymDiffIterMapKMerge, SymDiffIterMapMerge,
};

/// Turns any number of [`SortedDisjointMap`] iterators into a [`SortedDisjointMap`] iterator of their union,
/// i.e., all the integers in any input iterator, as sorted & disjoint ranges. Uses [`Merge`]
/// or [`KMerge`].
///
/// [`SortedDisjointMap`]: crate::SortedDisjointMap
/// [`Merge`]: crate::Merge
/// [`KMerge`]: crate::KMerge
///
/// # Examples
///
/// ```
/// use itertools::Itertools;
/// use range_set_blaze::{SymDiffIterMap, Merge, SortedDisjointMap, CheckSortedDisjoint};
///
/// let a = CheckSortedDisjoint::new(vec![1..=2, 5..=100].into_iter());
/// let b = CheckSortedDisjoint::from([2..=6]);
/// let union = SymDiffIterMap::new(Merge::new(a, b));
/// assert_eq!(union.to_string(), "1..=100");
///
/// // Or, equivalently:
/// let a = CheckSortedDisjoint::new(vec![1..=2, 5..=100].into_iter());
/// let b = CheckSortedDisjoint::from([2..=6]);
/// let union = a | b;
/// assert_eq!(union.to_string(), "1..=100")
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
    gather: Option<RangeValue<T, V, VR>>,
    ready_to_go: Option<RangeValue<T, V, VR>>,
}

fn min_next_end<T, V, VR>(next_end: &Option<T>, next_item: &RangeValue<T, V, VR>) -> Option<T>
where
    T: Integer,
    V: ValueOwned,
    VR: CloneBorrow<V>,
{
    Some(next_end.map_or_else(
        || *next_item.range.end(),
        |current_end| std::cmp::min(current_end, *next_item.range.end()),
    ))
}

impl<T, V, VR, I> Iterator for SymDiffIterMap<T, V, VR, I>
where
    T: Integer,
    V: ValueOwned,
    VR: CloneBorrow<V>,
    I: PrioritySortedStartsMap<T, V, VR>,
{
    type Item = RangeValue<T, V, VR>;

    fn next(&mut self) -> Option<RangeValue<T, V, VR>> {
        // Keep doing this until we have something to return.
        loop {
            if let Some(value) = self.ready_to_go.take() {
                // If ready_to_go is Some, return the value immediately.
                // println!("cmk output1 range {:?}", value.range);
                return Some(value);
            };

            // if self.next_item should go into the workspace, then put it there, get the next, next_item, and loop
            if let Some(next_item) = self.next_item.take() {
                let (next_start, _next_end) = next_item.range_value.range.clone().into_inner();

                // If workspace is empty, just push the next item
                let Some(best) = self.workspace.peek() else {
                    // println!(
                    //     "cmk pushing self.next_item {:?} into empty workspace",
                    //     next_item.range
                    // );
                    self.workspace_next_end =
                        min_next_end(&self.workspace_next_end, &next_item.range_value);
                    self.workspace.push(next_item);
                    self.next_item = self.iter.next();
                    // println!(
                    //     "cmk reading new self.next_item via .next() {:?}",
                    //     cmk_debug_string(&self.next_item)
                    // );
                    // println!("cmk return to top of the main processing loop");
                    continue; // return to top of the main processing loop
                };
                let best = &best.range_value;
                if next_start == *best.range.start() {
                    // Always push (this differs from UnionIterMap)
                    self.workspace_next_end =
                        min_next_end(&self.workspace_next_end, &next_item.range_value);
                    self.workspace.push(next_item);
                    self.next_item = self.iter.next();
                    continue; // return to top of the main processing loop
                }

                // It does not go into the workspace, so just hold it and keep processing.
                // println!(
                //     "cmk new start, so hold self.next_item {:?} for later",
                //     next_item.range
                // );
                self.next_item = Some(next_item);
            }

            // If the workspace is empty, we are done.
            let Some(best) = self.workspace.peek() else {
                debug_assert!(self.next_item.is_none());
                debug_assert!(self.ready_to_go.is_none());
                let value = self.gather.take();
                // println!("cmk output2 range {:?}", cmk_debug_string(&value));

                return value;
            };
            let best = &best.range_value;

            // We buffer for output the best item up to the start of the next item (if any).

            // Find the start of the next item, if any.
            // unwrap() is safe because we know the workspace is not empty
            let mut next_end = self.workspace_next_end.take().unwrap();
            if let Some(next_item) = self.next_item.as_ref() {
                next_end = min(*next_item.range_value.range.start() - T::one(), next_end);
            }

            // Add the front of best to the gather buffer.
            if let Some(mut gather) = self.gather.take() {
                if gather.value.borrow() == best.value.borrow()
                    && *gather.range.end() + T::one() == *best.range.start()
                {
                    if self.workspace.len() % 2 == 1 {
                        // if the gather is contiguous with the best, then merge them
                        gather.range = *gather.range.start()..=next_end;
                        // println!(
                        //     "cmk merge gather {:?} best {:?} as {:?} -> {:?}",
                        //     gather.range,
                        //     best.range,
                        //     *best.range.start()..=next_end,
                        //     gather.range
                        // );
                        self.gather = Some(gather);
                    } else {
                        // if an even number of items in the workspace, then flush the gather
                        self.ready_to_go = Some(gather);
                        debug_assert!(self.gather.is_none());
                    }
                } else {
                    // if the gather is not contiguous with the best, then output the gather and set the gather to the best
                    // println!(
                    //     "cmk new ready-to-go {:?}, new gather front of best {:?} as {:?}",
                    //     gather.range,
                    //     best.range,
                    //     *best.range.start()..=next_end
                    // );
                    self.ready_to_go = Some(gather);
                    // cmk this code appear twice
                    if self.workspace.len() % 2 == 1 {
                        self.gather = Some(RangeValue::new(
                            *best.range.start()..=next_end,
                            best.value.clone_borrow(),
                        ));
                    } else {
                        debug_assert!(self.gather.is_none());
                    }
                }
            } else {
                // if there is no gather, then set the gather to the best
                // println!(
                //     "cmk no gather,  capture front of best {:?} as {:?}",
                //     best.range,
                //     *best.range.start()..=next_end
                // );
                if self.workspace.len() % 2 == 1 {
                    self.gather = Some(RangeValue::new(
                        *best.range.start()..=next_end,
                        best.value.clone_borrow(),
                    ));
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
                if *item.range_value.range.end() <= next_end {
                    // too short, don't keep
                    // println!("cmk too short, don't keep in workspace {:?}", item.range);
                    continue; // while loop
                }
                item.range_value.range = next_end + T::one()..=*item.range_value.range.end();
                new_next_end = min_next_end(&new_next_end, &item.range_value);
                new_workspace.push(item);
            }
            self.workspace = new_workspace;
            self.workspace_next_end = new_next_end;
        } // end of main loop
    }
}

#[allow(dead_code)]
fn cmk_debug_string<'a, T, V, VR>(item: &Option<RangeValue<T, V, VR>>) -> String
where
    T: Integer,
    V: ValueOwned,
    VR: CloneBorrow<V> + 'a,
{
    if let Some(item) = item {
        format!("Some({:?})", item.range)
    } else {
        "None".to_string()
    }
}

// cmk000 is there a BitOrMergeMap?
// cmk000 is there a BitOrKMergeMap?
// cmk000 is there a BitXOrKMergeMap
// cmk000 Are operators defined on four results?
// cmk000 where is this where is this new2 used and should BitOr(K)Merge map use a new(2), too?

impl<T, V, VR, L, R> SymDiffIterMapMerge<T, V, VR, L, R>
where
    T: Integer,
    V: ValueOwned,
    VR: CloneBorrow<V>,
    L: SortedDisjointMap<T, V, VR>,
    R: SortedDisjointMap<T, V, VR>,
{
    // cmk00 should this be new2 and have a new, too (like UnionIterMap)?
    // cmk fix the comment on the set size. It should say inputs are SortedStarts not SortedDisjoint.
    /// Creates a new [`SymDiffIterMap`] from zero or more [`SortedDisjointMap`] iterators. See [`SymDiffIterMap`] for more details and examples.
    pub fn new2(left: L, right: R) -> Self {
        let iter = MergeMap::new(left, right);
        Self::new(iter)
    }
}

/// cmk doc
impl<T, V, VR, J> SymDiffIterMapKMerge<T, V, VR, J>
where
    T: Integer,
    V: ValueOwned,
    VR: CloneBorrow<V>,
    J: SortedDisjointMap<T, V, VR>,
{
    // cmk00 should this be new2 and have a new, too (like UnionIterMap)?
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

/// cmk000
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
