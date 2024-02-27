use core::{
    cmp::min,
    iter::FusedIterator,
    ops::{self, RangeInclusive},
};

use alloc::collections::BinaryHeap;
use itertools::Itertools;

use crate::{
    map::{BitOrMergeMap, ValueOwned},
    Integer,
};
use crate::{
    map::{CloneBorrow, SortedStartsInVecMap},
    unsorted_disjoint_map::AssumeSortedStartsMap,
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
pub struct UnionIterMap<'a, T, V, VR, SS>
where
    T: Integer,
    V: ValueOwned + 'a,
    VR: CloneBorrow<V> + 'a,
    SS: SortedStartsMap<'a, T, V, VR>,
{
    iter: SS,
    next_item: Option<RangeValue<'a, T, V, VR>>,
    workspace: BinaryHeap<RangeValue<'a, T, V, VR>>,
    gather: Option<RangeValue<'a, T, V, VR>>,
    ready_to_go: Option<RangeValue<'a, T, V, VR>>,
}

impl<'a, T: Integer, V: ValueOwned, VR, I> Iterator for UnionIterMap<'a, T, V, VR, I>
where
    VR: CloneBorrow<V> + 'a,
    I: SortedStartsMap<'a, T, V, VR>,
{
    type Item = RangeValue<'a, T, V, VR>;

    fn next(&mut self) -> Option<RangeValue<'a, T, V, VR>> {
        loop {
            if let Some(value) = self.ready_to_go.take() {
                // If ready_to_go was Some, return the value immediately.
                return Some(value);
            };

            // move self.next_item into the workspace if appropriate
            if let Some(next_item) = self.next_item.take() {
                let (next_start, next_end) = next_item.range.clone().into_inner();

                // If workspace is empty, just push the next item
                let Some(best) = self.workspace.peek() else {
                    self.workspace.push(next_item);
                    self.next_item = self.iter.next();
                    continue; // loop to get another input item
                };
                if next_start == *best.range.start() {
                    // Only push if the priority is higher or the end is greater
                    if next_item > *best || next_end > *best.range.end() {
                        self.workspace.push(next_item);
                    }
                    self.next_item = self.iter.next();
                    continue; // loop to get another input item
                }
                self.next_item = Some(next_item);
            }

            // If the workspace is empty, we are done.
            let Some(best) = self.workspace.peek() else {
                debug_assert!(self.next_item.is_none());
                return None;
            };

            // We buffer for output the best item up to the end of the next item (if any).
            let next_end = if let Some(next_item) = self.next_item.as_ref() {
                min(*next_item.range.start() - T::one(), *best.range.end())
            } else {
                *best.range.end()
            };

            // add the front of best to the output buffers
            if let Some(mut gather) = self.gather.take() {
                if gather.value.borrow() == best.value.borrow()
                    && *gather.range.end() + T::one() == *best.range.start()
                {
                    // if the gather is contiguous with the best, then merge them
                    gather.range = *gather.range.start()..=*best.range.end();
                    self.gather = Some(gather);
                } else {
                    // if the gather is not contiguous with the best, then output the gather and set the gather to the best
                    self.ready_to_go = Some(gather);
                    self.gather = Some(RangeValue::new(
                        *best.range.start()..=next_end,
                        best.value.clone_borrow(),
                        None,
                    ));
                }
            } else {
                // if there is no gather, then set the gather to the best
                self.gather = Some(RangeValue::new(
                    *best.range.start()..=next_end,
                    best.value.clone_borrow(),
                    None,
                ))
            };

            // We also update the workspace to removing any items that are completely covered by the new_start.
            // We also don't need to keep any items that have a lower priority and are shorter than the new best.
            let mut new_workspace = BinaryHeap::new();
            while let Some(mut item) = self.workspace.pop() {
                if *item.range.end() < next_end {
                    // too short, don't keep
                    continue;
                }
                item.range = next_end + T::one()..=*item.range.end();
                let Some(new_best) = new_workspace.peek() else {
                    // new_workspace is empty, so keep
                    new_workspace.push(item);
                    continue;
                };
                if &item < new_best && *item.range.end() <= *new_best.range.end() {
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
    for UnionIterMap<'a, T, V, &'a V, SortedStartsInVecMap<'a, T, V, &'a V>>
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

// cmk simplify the long types
// from iter RangeValue<T, V> to UnionIterMap
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
