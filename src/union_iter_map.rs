use crate::alloc::string::ToString;
use crate::sorted_disjoint_map::Priority;
use alloc::format;
use alloc::string::String;
use alloc::{collections::BinaryHeap, vec};
use core::ops::RangeInclusive;
use core::{cmp::min, iter::FusedIterator};
use itertools::Itertools;

use crate::{map::ValueOwned, Integer};
use crate::{
    map::{CloneBorrow, SortedStartsInVecMap},
    unsorted_disjoint_map::AssumeSortedStartsMap,
};
use crate::{
    sorted_disjoint_map::{RangeValue, SortedStartsMap},
    unsorted_disjoint_map::UnsortedDisjointMap,
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
/// use range_set_blaze::{UnionIterMap, Merge, SortedDisjointMap, CheckSortedDisjoint};
///
/// let a = CheckSortedDisjoint::new(vec![1..=2, 5..=100].into_iter());
/// let b = CheckSortedDisjoint::from([2..=6]);
/// let union = UnionIterMap::new(Merge::new(a, b));
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
pub struct UnionIterMap<'a, T, V, VR, SS>
where
    T: Integer + 'a,
    V: ValueOwned + 'a,
    VR: CloneBorrow<V> + 'a,
    SS: SortedStartsMap<'a, T, V, VR>,
{
    iter: SS,
    next_item: Option<RangeValue<'a, T, V, VR>>,
    workspace: BinaryHeap<Priority<'a, T, V, VR>>,
    gather: Option<RangeValue<'a, T, V, VR>>,
    ready_to_go: Option<RangeValue<'a, T, V, VR>>,
}

impl<'a, T, V, VR, I> Iterator for UnionIterMap<'a, T, V, VR, I>
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
                let (next_start, next_end) = next_item.range.clone().into_inner();

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
                    // Only push if the priority is higher or the end is greater
                    if next_item.priority > best.priority || next_end > *best.range.end() {
                        // println!("cmk pushing next_item {:?} into workspace", next_item.range);
                        self.workspace.push(Priority(next_item));
                    } else {
                        // println!(
                        //     "cmk throwing away next_item {:?} because of priority and length",
                        //     next_item.range
                        // );
                    }
                    self.next_item = self.iter.next();
                    // println!(
                    //     "cmk .next() self.next_item {:?}",
                    //     cmk_debug_string(&self.next_item)
                    // );
                    // println!("cmk return to top of the main processing loop");
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
            let next_end = if let Some(next_item) = self.next_item.as_ref() {
                // println!(
                //     "cmk start-less1 {:?} {:?}",
                //     next_item.range.start(),
                //     best.range.end()
                // );
                min(*next_item.range.start() - T::one(), *best.range.end())
                // println!("cmk min {:?}", m);
            } else {
                *best.range.end()
            };

            // Add the front of best to the gather buffer.
            if let Some(mut gather) = self.gather.take() {
                if gather.value.borrow() == best.value.borrow()
                    && *gather.range.end() + T::one() == *best.range.start()
                {
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
                    // if the gather is not contiguous with the best, then output the gather and set the gather to the best
                    // println!(
                    //     "cmk new ready-to-go {:?}, new gather front of best {:?} as {:?}",
                    //     gather.range,
                    //     best.range,
                    //     *best.range.start()..=next_end
                    // );
                    self.ready_to_go = Some(gather);
                    self.gather = Some(RangeValue::new(
                        *best.range.start()..=next_end,
                        best.value.clone_borrow(),
                        None,
                    ));
                }
            } else {
                // if there is no gather, then set the gather to the best
                // println!(
                //     "cmk no gather,  capture front of best {:?} as {:?}",
                //     best.range,
                //     *best.range.start()..=next_end
                // );
                self.gather = Some(RangeValue::new(
                    *best.range.start()..=next_end,
                    best.value.clone_borrow(),
                    None,
                ))
            };

            // We also update the workspace to removing any items that are completely covered by the new_start.
            // We also don't need to keep any items that have a lower priority and are shorter than the new best.
            let mut new_workspace = BinaryHeap::new();
            while let Some(item) = self.workspace.pop() {
                let mut item = item.0;
                if *item.range.end() <= next_end {
                    // too short, don't keep
                    // println!("cmk too short, don't keep in workspace {:?}", item.range);
                    continue; // while loop
                }
                item.range = next_end + T::one()..=*item.range.end();
                let Some(new_best) = new_workspace.peek() else {
                    // println!("cmk no workspace, so keep {:?}", item.range);
                    // new_workspace is empty, so keep
                    new_workspace.push(Priority(item));
                    continue; // while loop
                };
                let new_best = &new_best.0;
                if item.priority < new_best.priority && *item.range.end() <= *new_best.range.end() {
                    // println!("cmk item is lower priority {:?} and shorter {:?} than best item {:?},{:?} in new workspace, so don't keep",
                    // item.priority, item.range, new_best.priority, new_best.range);
                    // not as good as new_best, and shorter, so don't keep
                    continue; // while loop
                }

                // higher priority or longer, so keep
                // println!("cmk item is higher priority {:?} or longer {:?} than best item {:?},{:?} in new workspace, so keep",
                // item.priority, item.range, new_best.priority, new_best.range);
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

impl<'a, T, V, VR, I> UnionIterMap<'a, T, V, VR, I>
where
    T: Integer,
    V: ValueOwned,
    VR: CloneBorrow<V> + 'a,
    I: SortedStartsMap<'a, T, V, VR>,
{
    // cmk fix the comment on the set size. It should say inputs are SortedStarts not SortedDisjoint.
    /// Creates a new [`UnionIterMap`] from zero or more [`SortedStartsMap`] iterators. See [`UnionIterMap`] for more details and examples.
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

// impl<'a, T, V, VR, const N: usize> From<[T; N]>
//     for UnionIterMap<'a, T, V, VR, SortedRangeValueVec<'a, T, V, VR>>
// where
//     T: Integer,
//     V: ValueOwned,
//     VR: CloneBorrow<V> + 'a,
// {
//     fn from(arr: [T; N]) -> Self {
//         arr.as_slice().into()
//     }
// }

// // cmk00
// impl<'a, T, V, VR> From<&[T]> for UnionIterMap<'a, T, V, VR, SortedRangeValueVec<'a, T, V, VR>>
// where
//     T: Integer,
//     V: ValueOwned,
//     VR: CloneBorrow<V> + 'a,
// {
//     fn from(slice: &[T]) -> Self {
//         slice.iter().cloned().collect()
//     }
// }

// // cmk00
// impl<'a, T, V, VR, const N: usize> From<[RangeValue<'a, T, V, VR>; N]>
//     for UnionIterMap<'a, T, V, VR, SortedRangeValueVec<'a, T, V, VR>>
// where
//     T: Integer,
//     V: ValueOwned,
//     VR: CloneBorrow<V> + 'a,
// {
//     fn from(arr: [RangeValue<'a, T, V, VR>; N]) -> Self {
//         arr.as_slice().into()
//     }
// }

// from iter (T, &V) to UnionIterMap
impl<'a, T, V> FromIterator<(T, &'a V)>
    for UnionIterMap<'a, T, V, &'a V, SortedStartsInVecMap<'a, T, V, &'a V>>
where
    T: Integer + 'a,
    V: ValueOwned + 'a,
{
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = (T, &'a V)>,
    {
        let iter = iter.into_iter().map(|(x, value)| (x..=x, value));
        UnionIterMap::from_iter(iter)
    }
}

// from iter (RangeInclusive<T>, &V) to UnionIterMap
impl<'a, T: Integer + 'a, V: ValueOwned + 'a> FromIterator<(RangeInclusive<T>, &'a V)>
    for UnionIterMap<'a, T, V, &'a V, SortedStartsInVecMap<'a, T, V, &'a V>>
{
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = (RangeInclusive<T>, &'a V)>,
    {
        let iter = iter.into_iter();
        let iter = iter.map(|(range, value)| RangeValue::new(range, value, None));
        UnionIterMap::from_iter(iter)
    }
}

// cmk used?
#[allow(dead_code)]
type SortedRangeValueVec<'a, T, V, VR> =
    AssumeSortedStartsMap<'a, T, V, VR, vec::IntoIter<RangeValue<'a, T, V, VR>>>;

// cmk simplify the long types
// from iter RangeValue<'a, T, V, VR> to UnionIterMap
impl<'a, T: Integer + 'a, V: ValueOwned + 'a, VR> FromIterator<RangeValue<'a, T, V, VR>>
    for UnionIterMap<'a, T, V, VR, SortedStartsInVecMap<'a, T, V, VR>>
where
    VR: CloneBorrow<V> + 'a,
{
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = RangeValue<'a, T, V, VR>>,
    {
        let iter = iter.into_iter();
        // let iter = iter.map(|x| {
        //     println!("cmk x.priority {:?}", x.priority);
        //     x
        // });
        let iter = UnsortedDisjointMap::from(iter);
        UnionIterMap::from(iter)
    }
}

// from from UnsortedDisjointMap to UnionIterMap
impl<'a, T, V, VR, I> From<UnsortedDisjointMap<'a, T, V, VR, I>>
    for UnionIterMap<'a, T, V, VR, SortedStartsInVecMap<'a, T, V, VR>>
where
    T: Integer,
    V: ValueOwned + 'a,
    VR: CloneBorrow<V> + 'a,
    I: Iterator<Item = RangeValue<'a, T, V, VR>>,
{
    #[allow(clippy::clone_on_copy)]
    fn from(unsorted_disjoint: UnsortedDisjointMap<'a, T, V, VR, I>) -> Self {
        let iter = unsorted_disjoint.sorted_by(|a, b| match a.range.start().cmp(b.range.start()) {
            core::cmp::Ordering::Equal => {
                // println!(
                //     "cmk a.priority {:?} b.priority {:?}",
                //     a.priority, b.priority
                // );
                // println!("cmk a.range {:?} b.range {:?}", a.range, b.range);
                debug_assert!(a.priority != b.priority, "Priorities must not be equal");
                b.priority.cmp(&a.priority)
            }
            other => other,
        });
        let iter = AssumeSortedStartsMap { iter };

        Self::new(iter)
    }
}

impl<'a, T, V, VR, I> FusedIterator for UnionIterMap<'a, T, V, VR, I>
where
    T: Integer + 'a,
    V: ValueOwned + 'a,
    VR: CloneBorrow<V> + 'a,
    I: SortedStartsMap<'a, T, V, VR> + FusedIterator,
{
}

// cmk
// impl<'a, T, V, VR, I> ops::Not for UnionIterMap<'a, T, V, VR, I>
// where
//     I: SortedStartsMap<T, V>,
// {
//     type Output = NotIterMap<T, V, Self>;

//     fn not(self) -> Self::Output {
//         self.complement()
//     }
// }

// impl<'a, T, V, VR, R, L> ops::BitOr<R> for UnionIterMap<'a, T, V, VR, L>
// where
//     T: Integer + 'a,
//     V: ValueOwned + 'a,
//     VR: CloneBorrow<V> + 'a,
//     L: SortedStartsMap<'a, T, V, VR>,
//     R: SortedDisjointMap<'a, T, V, VR> + 'a,
// {
//     type Output = BitOrMergeMap<'a, T, V, VR, Self, R>;

//     fn bitor(self, rhs: R) -> Self::Output {
//         // It might be fine to optimize to self.iter, but that would require
//         // also considering field 'range'
//         SortedDisjointMap::union(self, rhs)
//     }
// }

// impl<'a, T, V, VR, R, L> ops::Sub<R> for UnionIterMap<'a, T, V, VR, L>
// where
//     L: SortedStartsMap<T, V>,
//     R: SortedDisjointMap<T, V>,
// {
//     type Output = BitSubMergeMap<T, V, Self, R>;

//     fn sub(self, rhs: R) -> Self::Output {
//         SortedDisjointMap::difference(self, rhs)
//     }
// }

// impl<'a, T, V, VR, R, L> ops::BitXor<R> for UnionIterMap<'a, T, V, VR, L>
// where
//     L: SortedStartsMap<T, V>,
//     R: SortedDisjointMap<T, V>,
// {
//     type Output = BitXOrTeeMap<T, V, Self, R>;

//     #[allow(clippy::suspicious_arithmetic_impl)]
//     fn bitxor(self, rhs: R) -> Self::Output {
//         SortedDisjointMap::symmetric_difference(self, rhs)
//     }
// }

// impl<'a, T, V, VR, R, L> ops::BitAnd<R> for UnionIterMap<'a, T, V, VR, L>
// where
//     L: SortedStartsMap<T, V>,
//     R: SortedDisjointMap<T, V>,
// {
//     type Output = BitAndMergeMap<T, V, Self, R>;

//     fn bitand(self, other: R) -> Self::Output {
//         SortedDisjointMap::intersection(self, other)
//     }
// }

// impl<'a, T: Integer + 'a, V: ValueOwned + 'a, const N: usize> From<[(T, V); N]>
//     for UnionIterMap<'a, T, V, &'a V, SortedStartsInVecMap<'a, T, V, &'a V>>
// {
//     fn from(arr: [(T, &'a V); N]) -> Self {
//         // Directly create an iterator from the array and map it as needed
//         arr.iter()
//             .map(|&(t, v)| (t, v)) // This is a simple identity map; adjust as needed for your actual transformation
//             .collect() // Collect into UnionIterMap, relying on FromIterator
//     }
// }
