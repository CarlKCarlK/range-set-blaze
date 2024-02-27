use core::{
    cmp::{max, min},
    iter::FusedIterator,
    marker::PhantomData,
    ops::{self, RangeInclusive},
};

use alloc::{collections::BinaryHeap, vec};
use itertools::Itertools;

use crate::{map::CloneBorrow, unsorted_disjoint_map::AssumeSortedStartsMap};
use crate::{
    map::{BitOrMergeMap, ValueOwned},
    range_values::NON_ZERO_ONE,
    Integer,
};
use crate::{
    sorted_disjoint_map::{RangeValue, SortedDisjointMap, SortedStartsMap},
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
pub struct UnionIterMap<'a, T, V, VR, I>
where
    T: Integer,
    V: ValueOwned + 'a,
    VR: CloneBorrow<V> + 'a,
    I: SortedStartsMap<'a, T, V, VR>,
{
    iter: I,
    done_with_iter: bool,
    next_item: Option<RangeValue<'a, T, V, VR>>,
    workspace: BinaryHeap<RangeValue<'a, T, V, VR>>,
    gather: Option<RangeValue<'a, T, V, VR>>,
    ready_to_go: Option<RangeValue<'a, T, V, VR>>,
    phantom0: PhantomData<&'a V>,
    phantom1: PhantomData<VR>,
    phantom2: PhantomData<T>,
    // option_range_value: Option<RangeValue<'a, T, V, VR>>,
}

impl<'a, T: Integer, V: ValueOwned, VR, I> Iterator for UnionIterMap<'a, T, V, VR, I>
where
    VR: CloneBorrow<V> + 'a,
    I: SortedStartsMap<'a, T, V, VR>,
{
    type Item = RangeValue<'a, T, V, VR>;

    fn next(&mut self) -> Option<RangeValue<'a, T, V, VR>> {
        if let Some(value) = self.ready_to_go.take() {
            // If ready_to_go was Some, return the value immediately.
            return Some(value);
        };

        loop {
            // Be sure self.next_item is loaded.
            if !self.done_with_iter && self.next_item.is_none() {
                self.next_item = self.iter.next();
                self.done_with_iter = self.next_item.is_none();
            }

            // move self.next_item into the workspace if appropriate
            if let Some(next_item) = self.next_item.take() {
                let (next_start, next_end) = next_item.range.into_inner();

                // If workspace is empty, just push the next item
                let Some(best) = self.workspace.peek() else {
                    self.workspace.push(next_item);
                    continue; // loop to get another input item
                };
                if next_start == *best.range.start() {
                    // Only push if the priority is higher or the end is greater
                    if next_item > best || next_end > best.range.end() {
                        self.workspace.push(next_item);
                    }
                    continue; // loop to get another input item
                }
                self.next_item = Some(next_item);
            }

            // If the workspace is empty, we are done.
            let Some(best) = self.workspace.peek() else {
                debug_assert!(self.next_item.is_none());
                debug_assert!(self.done_with_iter); // cmk is this needed?
                return None;
            };

            // We buffer for output the best item up to the end of the next item (if any).
            let new_start = if let Some(next_item) = self.next_item.as_ref() {
                min(*next_item.range.start(), best.range.end() + T::one())
            } else {
                best.range.end() + T::one()
            };

            // add the front of best to the output buffers
            if let Some(gather) = self.gather.take() {
                if gather.value == best.value && gather.range.end() + T::one == best.range.start() {
                    // if the gather is contiguous with the best, then merge them
                    gather.range = *gather.range.start()..=*best.range.end();
                    self.gather = Some(gather);
                } else {
                    // if the gather is not contiguous with the best, then output the gather and set the gather to the best
                    self.ready_to_go = Some(gather);
                    self.gather = Some(RangeValue::new(
                        *best.range.start()..=new_start - T::one(),
                        best.value.clone_borrow(),
                        None,
                    ));
                }
            } else {
                // if there is no gather, then set the gather to the best
                self.gather = Some(RangeValue::new(
                    *best.range.start()..=new_start - T::one(),
                    best.value.clone_borrow(),
                    None,
                ))
            };

            // We also update the workspace to removing any items that are completely covered by the new_start.
            // We also don't need to keep any items that have a lower priority and are shorter than the new best.
            let new_workspace = BinaryHeap::new();
            while let Some(mut item) = self.workspace.pop() {
                if *item.range.end() < new_start {
                    // too short, don't keep
                    continue;
                }
                item.range = new_start..=*item.range.end();
                let Some(new_best) = new_workspace.peek() else {
                    // new_workspace is empty, so keep
                    new_workspace.push(item);
                    continue;
                };
                if item < new_best && *item.range.end() <= *new_best.range.end() {
                    // not as good as new_best, and shorter, so don't keep
                    continue;
                }

                // higher priority or longer, so keep
                new_workspace.push(item);
            }
            self.workspace = new_workspace;
        } // end of main loop
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
    // cmk0 do not do this with a vec
    pub fn new(iter: I) -> Self {
        Self {
            iter,
            done_with_iter: false,
            next_item: None,
            workspace: Vec::new(),
            gather: None,
            ready_to_go: None,
            phantom0: PhantomData,
            phantom1: PhantomData,
            phantom2: PhantomData,
        }
        //     // By default all ends are inclusive (different that most programs)
        //     let mut vec_in = iter.collect_vec();
        //     // println!("vec_in: {:?}", vec_in.len()); // cmk
        //     let mut vec_mid = Vec::<RangeValue<'a, T, V, VR>>::new();
        //     let mut workspace = Vec::<RangeValue<'a, T, V, VR>>::new();
        //     let mut bar_priority = NON_ZERO_ONE;
        //     let mut bar_end = T::zero();
        //     while !vec_in.is_empty() || !workspace.is_empty() {
        //         // If there are new ranges to process and they they have the same start as the workspace
        //         if !vec_in.is_empty()
        //             && (workspace.is_empty() || *vec_in[0].range.start() == *workspace[0].range.start())
        //         {
        //             // find the index (of any) of the first index with a different start that the first one
        //             let first_start = *vec_in[0].range.start();

        //             // if bar_end is None, set it to first_start
        //             bar_end = max(bar_end, first_start);
        //             let split_index = vec_in
        //                 .iter()
        //                 .position(|x| *x.range.start() != first_start)
        //                 .unwrap_or(vec_in.len());
        //             // set same_start to the first split_index elements. Allocate for this
        //             // remove the first split_index elements from vec_in. do this in place.
        //             let same_starts: Vec<_> = vec_in.drain(0..split_index).collect();
        //             for same_start in same_starts {
        //                 let same_start_priority = same_start
        //                     .priority
        //                     .expect("Every range in UnionIterMap must have a priority");
        //                 if same_start_priority < bar_priority && same_start.range.end() < &bar_end {
        //                     continue;
        //                 }
        //                 if same_start_priority >= bar_priority {
        //                     bar_priority = same_start_priority;
        //                     bar_end = *same_start.range.end();
        //                 }
        //                 workspace.push(same_start);
        //             }
        //         }

        //         // The workspace is now full of ranges with the same start. We need to find the best one.

        //         // find the one element with priority = bar_priority
        //         // cmk use priority queue
        //         let index_of_best = workspace
        //             .iter()
        //             .position(|x| x.priority == Some(bar_priority))
        //             .unwrap();
        //         let best = &workspace[index_of_best];

        //         // output_end is the smallest end in workspace
        //         let mut output_end = *workspace.iter().map(|x| x.range.end()).min().unwrap();
        //         // if vec_is is not empty, then output_end is the minimum of output_end and the start of the first element in vec_in -1
        //         if !vec_in.is_empty() {
        //             let next_start = *vec_in[0].range.start();
        //             // cmk underflow?
        //             output_end = min(output_end, next_start - T::one())
        //         };
        //         vec_mid.push(RangeValue::new(
        //             *best.range.start()..=output_end,
        //             best.value.clone_borrow(),
        //             None,
        //         )); // best.priority,
        //             // trim the start of the ranges in workspace to output_end+1, remove any that are empty
        //             // also find the best priority and the new bar_end
        //         workspace.retain(|range_value| *range_value.range.end() > output_end);
        //         bar_priority = NON_ZERO_ONE;
        //         bar_end = output_end;
        //         // this avoids overflow
        //         if workspace.is_empty() {
        //             continue;
        //         }

        //         let new_start = output_end + T::one();
        //         for range_value in workspace.iter_mut() {
        //             let range_value_priority = range_value
        //                 .priority
        //                 .expect("Every range in UnionIterMap must have a priority");
        //             range_value.range = new_start..=*range_value.range.end();
        //             if range_value_priority > bar_priority {
        //                 bar_priority = range_value_priority;
        //                 bar_end = *range_value.range.end();
        //             }
        //         }
        //     }

        //     let mut vec_out = Vec::<RangeValue<'a, T, V, VR>>::new();
        //     let mut index = 0;
        //     while index < vec_mid.len() {
        //         let mut index_exclusive_end = index + 1;
        //         let mut previous_index = index;
        //         while index_exclusive_end < vec_mid.len()
        //             && vec_mid[previous_index].value.borrow() == vec_mid[index_exclusive_end].value.borrow()
        //             // cmk overflow?
        //             && *vec_mid[previous_index].range.end() + T::one()
        //                 == *vec_mid[index_exclusive_end].range.start()
        //         {
        //             previous_index = index_exclusive_end;
        //             index_exclusive_end += 1;
        //         }
        //         vec_out.push(RangeValue::new(
        //             *vec_mid[index].range.start()..=*vec_mid[index_exclusive_end - 1].range.end(),
        //             vec_mid[index].value.clone_borrow(),
        //             None,
        //         ));
        //         index = index_exclusive_end;
        //     }

        //     Self {
        //         iter: vec_out.into_iter(),
        //         phantom: PhantomData,
        //     }
    }
}

// impl<T: Integer, V: PartialEqClone, const N: usize> From<[T; N]>
//     for UnionIterMap<T, V, SortedRangeInclusiveVec<T, V>>
// {
//     fn from(arr: [T; N]) -> Self {
//         arr.as_slice().into()
//     }
// }

// impl<T: Integer, V: PartialEqClone> From<&[T]> for UnionIterMap<T, V, SortedRangeInclusiveVec<T, V>> {
//     fn from(slice: &[T]) -> Self {
//         slice.iter().cloned().collect()
//     }
// }

// impl<T: Integer, V: PartialEqClone, const N: usize> From<[RangeValue<T, V>; N]>
//     for UnionIterMap<T, V, SortedRangeInclusiveVec<T, V>>
// {
//     fn from(arr: [RangeValue<T, V>; N]) -> Self {
//         arr.as_slice().into()
//     }
// }

// from iter (T, &V) to UnionIterMap
impl<'a, T: Integer + 'a, V: ValueOwned + 'a> FromIterator<(T, &'a V)>
    for UnionIterMap<'a, T, V, &'a V, SortedRangeInclusiveVec<'a, T, V, &'a V>>
{
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = (T, &'a V)>,
    {
        iter.into_iter().map(|(x, value)| (x..=x, value)).collect()
    }
}

// from iter (RangeInclusive<T>, &V) to UnionIterMap
impl<'a, T: Integer + 'a, V: ValueOwned + 'a> FromIterator<(RangeInclusive<T>, &'a V)>
    for UnionIterMap<'a, T, V, &'a V, SortedRangeInclusiveVec<'a, T, V, &'a V>>
{
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = (RangeInclusive<T>, &'a V)>,
    {
        let iter = iter.into_iter();
        let iter = iter.map(|(range, value)| RangeValue::new(range, value, None));
        let iter: UnionIterMap<'a, T, V, &'a V, SortedRangeInclusiveVec<'a, T, V, &'a V>> =
            UnionIterMap::from_iter(iter);
        iter
    }
}

// from iter RangeValue<T, V> to UnionIterMap
impl<'a, T: Integer + 'a, V: ValueOwned + 'a, VR> FromIterator<RangeValue<'a, T, V, VR>>
    for UnionIterMap<'a, T, V, VR, SortedRangeInclusiveVec<'a, T, V, VR>>
where
    VR: CloneBorrow<V> + 'a,
{
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = RangeValue<'a, T, V, VR>>,
    {
        UnsortedDisjointMap::from(iter.into_iter()).into()
    }
}

// from from UnsortedDisjointMap to UnionIterMap
impl<'a, T, V, VR, I> From<UnsortedDisjointMap<'a, T, V, VR, I>>
    for UnionIterMap<'a, T, V, VR, SortedRangeInclusiveVec<'a, T, V, VR>>
where
    T: Integer,
    V: ValueOwned + 'a,
    VR: CloneBorrow<V> + 'a,
    I: Iterator<Item = RangeValue<'a, T, V, VR>>,
{
    #[allow(clippy::clone_on_copy)]
    fn from(unsorted_disjoint: UnsortedDisjointMap<'a, T, V, VR, I>) -> Self {
        let iter = unsorted_disjoint.sorted_by(|a, b| match a.range.start().cmp(b.range.start()) {
            std::cmp::Ordering::Equal => b.priority.cmp(&a.priority),
            other => other,
        });
        let iter = AssumeSortedStartsMap { iter };

        Self::new(iter)
    }
}

impl<'a, T: Integer, V: ValueOwned, VR, I> FusedIterator for UnionIterMap<'a, T, V, VR, I>
where
    VR: CloneBorrow<V> + 'a,
    I: SortedStartsMap<'a, T, V, VR> + FusedIterator,
{
}

// cmk
// impl<T: Integer, V: PartialEqClone, I> ops::Not for UnionIterMap<T, V, I>
// where
//     I: SortedStartsMap<T, V>,
// {
//     type Output = NotIterMap<T, V, Self>;

//     fn not(self) -> Self::Output {
//         self.complement()
//     }
// }

impl<'a, T, V, VR, R, L> ops::BitOr<R> for UnionIterMap<'a, T, V, VR, L>
where
    T: Integer + 'a,
    V: ValueOwned + 'a,
    VR: CloneBorrow<V> + 'a,
    L: SortedStartsMap<'a, T, V, VR>,
    R: SortedDisjointMap<'a, T, V, VR> + 'a,
{
    type Output = BitOrMergeMap<'a, T, V, VR, Self, R>;

    fn bitor(self, rhs: R) -> Self::Output {
        // It might be fine to optimize to self.iter, but that would require
        // also considering field 'range'
        SortedDisjointMap::union(self, rhs)
    }
}

// impl<T: Integer, V: PartialEqClone, R, L> ops::Sub<R> for UnionIterMap<T, V, L>
// where
//     L: SortedStartsMap<T, V>,
//     R: SortedDisjointMap<T, V>,
// {
//     type Output = BitSubMergeMap<T, V, Self, R>;

//     fn sub(self, rhs: R) -> Self::Output {
//         SortedDisjointMap::difference(self, rhs)
//     }
// }

// impl<T: Integer, V: PartialEqClone, R, L> ops::BitXor<R> for UnionIterMap<T, V, L>
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

// impl<T: Integer, V: PartialEqClone, R, L> ops::BitAnd<R> for UnionIterMap<T, V, L>
// where
//     L: SortedStartsMap<T, V>,
//     R: SortedDisjointMap<T, V>,
// {
//     type Output = BitAndMergeMap<T, V, Self, R>;

//     fn bitand(self, other: R) -> Self::Output {
//         SortedDisjointMap::intersection(self, other)
//     }
// }
