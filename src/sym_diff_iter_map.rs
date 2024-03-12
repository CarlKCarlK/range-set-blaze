use core::cmp::min;

use alloc::collections::BinaryHeap;

use crate::{
    map::{CloneBorrow, ValueOwned},
    range_values::{AdjustPriorityMap, NON_ZERO_MAX, NON_ZERO_MIN},
    sorted_disjoint_map::Priority,
    BitXorAdjusted, Integer, MergeMap, RangeValue, SortedDisjointMap, SortedStartsMap,
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
pub struct SymDiffIterMap<'a, T, V, VR, I>
where
    T: Integer + 'a,
    V: ValueOwned + 'a,
    VR: CloneBorrow<V> + 'a,
    I: SortedStartsMap<'a, T, V, VR>,
{
    iter: I,
    next_item: Option<RangeValue<'a, T, V, VR>>,
    workspace: BinaryHeap<Priority<'a, T, V, VR>>,
    gather: Option<RangeValue<'a, T, V, VR>>,
    ready_to_go: Option<RangeValue<'a, T, V, VR>>,
}

impl<'a, T, V, VR, I> Iterator for SymDiffIterMap<'a, T, V, VR, I>
where
    T: Integer + 'a,
    V: ValueOwned + 'a,
    VR: CloneBorrow<V> + 'a,
    I: SortedStartsMap<'a, T, V, VR>,
{
    type Item = RangeValue<'a, T, V, VR>;

    fn next(&mut self) -> Option<RangeValue<'a, T, V, VR>> {
        // Keep doing this until we have something to return.
        loop {
            if let Some(value) = self.ready_to_go.take() {
                // If ready_to_go is Some, return the value immediately.
                // println!("cmk output1 range {:?}", value.range);
                return Some(value);
            };

            // if self.next_item should go into the workspace, then put it there, get the next, next_item, and loop
            if let Some(next_item) = self.next_item.take() {
                let (next_start, _next_end) = next_item.range.clone().into_inner();

                // If workspace is empty, just push the next item
                let Some(best) = self.workspace.peek() else {
                    // println!(
                    //     "cmk pushing self.next_item {:?} into empty workspace",
                    //     next_item.range
                    // );
                    self.workspace.push(Priority(next_item));
                    self.next_item = self.iter.next();
                    // println!(
                    //     "cmk reading new self.next_item via .next() {:?}",
                    //     cmk_debug_string(&self.next_item)
                    // );
                    // println!("cmk return to top of the main processing loop");
                    continue; // return to top of the main processing loop
                };
                let best = &best.0;
                if next_start == *best.range.start() {
                    // Always push (this differs from UnionIterMap)
                    self.workspace.push(Priority(next_item));
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
            let best = &best.0;

            // We buffer for output the best item up to the start of the next item (if any).

            // Find the start of the next item, if any.
            // cmk00000000 keep a running total instead of using .map().
            let mut next_end = *self
                .workspace
                .iter()
                .map(|x| x.0.range.end())
                .min()
                .unwrap(); // unwrap() is safe because we know the workspace is not empty
            if let Some(next_item) = self.next_item.as_ref() {
                next_end = min(*next_item.range.start() - T::one(), next_end);
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
                            None,
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
                        None,
                    ));
                } else {
                    debug_assert!(self.gather.is_none());
                }
            };

            // We also update the workspace to removing any items that are completely covered by the new_start.
            // (Unlike UnionIterMap, we must keep any items that have a lower priority and are shorter than the new best.)
            // cmk use .filter() ?
            let mut new_workspace = BinaryHeap::new();
            while let Some(item) = self.workspace.pop() {
                let mut item = item.0;
                if *item.range.end() <= next_end {
                    // too short, don't keep
                    // println!("cmk too short, don't keep in workspace {:?}", item.range);
                    continue; // while loop
                }
                item.range = next_end + T::one()..=*item.range.end();
                new_workspace.push(Priority(item));
            }
            self.workspace = new_workspace;
        } // end of main loop
    }
}

#[allow(dead_code)]
fn cmk_debug_string<'a, T, V, VR>(item: &Option<RangeValue<'a, T, V, VR>>) -> String
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

impl<'a, T, V, VR, L, R> BitXorAdjusted<'a, T, V, VR, L, R>
where
    T: Integer,
    V: ValueOwned,
    VR: CloneBorrow<V> + 'a,
    L: SortedDisjointMap<'a, T, V, VR>,
    R: SortedDisjointMap<'a, T, V, VR>,
{
    // cmk fix the comment on the set size. It should say inputs are SortedStarts not SortedDisjoint.
    /// Creates a new [`SymDiffIterMap`] from zero or more [`SortedDisjointMap`] iterators. See [`SymDiffIterMap`] for more details and examples.
    pub fn new2(left: L, right: R) -> Self {
        let left = AdjustPriorityMap::new(left, Some(NON_ZERO_MAX));
        let right = AdjustPriorityMap::new(right, Some(NON_ZERO_MIN));
        let mut iter = MergeMap::new(left, right);
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
