use crate::map::SortedStartsInVec;
use crate::merge::KMerge;
use crate::unsorted_disjoint::UnsortedDisjoint;
use crate::{AssumeSortedStarts, Merge, SortedDisjoint, SortedStarts, UnionIterKMerge};
use crate::{Integer, UnionIterMerge};
use alloc::vec;
use core::cmp::min;
use core::iter::FusedIterator;
use core::ops::RangeInclusive;
use itertools::Itertools;

/// Turns any number of [`SortedDisjoint`] iterators into a [`SortedDisjoint`] iterator of their union,
/// i.e., all the integers in any input iterator, as sorted & disjoint ranges. Uses [`Merge`]
/// or [`KMerge`].
///
/// [`SortedDisjoint`]: crate::SortedDisjoint
/// [`Merge`]: crate::Merge
/// [`KMerge`]: crate::KMerge
///
/// # Examples
///
/// ```
/// use itertools::Itertools;
/// use range_set_blaze::{UnionIter, Merge, SortedDisjoint, CheckSortedDisjoint};
///
/// let a = CheckSortedDisjoint::new(vec![1..=2, 5..=100].into_iter());
/// let b = CheckSortedDisjoint::from([2..=6]);
/// let union = UnionIter::new2(a, b);
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
pub struct UnionIter<T, SS>
where
    T: Integer,
    SS: SortedStarts<T>,
{
    iter: SS,
    next_item: Option<RangeInclusive<T>>,
    workspace: Vec<RangeInclusive<T>>,
    gather: Option<RangeInclusive<T>>,
    ready_to_go: Option<RangeInclusive<T>>,
}

impl<T, I> Iterator for UnionIter<T, I>
where
    T: Integer,
    I: SortedStarts<T>,
{
    type Item = RangeInclusive<T>;

    fn next(&mut self) -> Option<RangeInclusive<T>> {
        // Keep doing this until we have something to return.
        loop {
            if let Some(value) = self.ready_to_go.take() {
                // If ready_to_go is Some, return the value immediately.
                // println!("cmk output1 range {:?}", value.0);
                return Some(value);
            };

            // if self.next_item should go into the workspace, then put it there, get the next, next_item, and loop
            if let Some(next_item) = self.next_item.take() {
                let (next_start, next_end) = next_item.clone().into_inner();

                // If workspace is empty, just push the next item
                let Some(best) = self.workspace.first() else {
                    // println!(
                    //     "cmk pushing self.next_item {:?} into empty workspace",
                    //     next_item.0
                    // );
                    self.workspace.push(next_item);
                    self.next_item = self.iter.next();
                    // println!(
                    //     "cmk reading new self.next_item via .next() {:?}",
                    //     cmk_debug_string(&self.next_item)
                    // );
                    // println!("cmk return to top of the main processing loop");
                    continue; // return to top of the main processing loop
                };
                if next_start == *best.start() {
                    self.workspace.push(next_item);
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
                //     next_item.0
                // );
                self.next_item = Some(next_item);
            }

            // If the workspace is empty, we are done.
            let Some(best) = self.workspace.first() else {
                debug_assert!(self.next_item.is_none());
                debug_assert!(self.ready_to_go.is_none());
                let value = self.gather.take();
                // println!("cmk output2 range {:?}", cmk_debug_string(&value));

                return value;
            };

            // We buffer for output the best item up to the start of the next item (if any).

            // Find the start of the next item, if any.
            let next_end = if let Some(next_item) = self.next_item.as_ref() {
                // println!(
                //     "cmk start-less1 {:?} {:?}",
                //     next_item.0.start(),
                //     best.0.end()
                // );
                min(*next_item.start() - T::one(), *best.end())
                // println!("cmk min {:?}", m);
            } else {
                *best.end()
            };

            // Add the front of best to the gather buffer.
            if let Some(mut gather) = self.gather.take() {
                if *gather.end() + T::one() == *best.start() {
                    // if the gather is contiguous with the best, then merge them
                    gather = *gather.start()..=next_end;
                    // println!(
                    //     "cmk merge gather {:?} best {:?} as {:?} -> {:?}",
                    //     gather.0,
                    //     best.0,
                    //     *best.0.start()..=next_end,
                    //     gather.0
                    // );
                    self.gather = Some(gather);
                } else {
                    // if the gather is not contiguous with the best, then output the gather and set the gather to the best
                    // println!(
                    //     "cmk new ready-to-go {:?}, new gather front of best {:?} as {:?}",
                    //     gather.0,
                    //     best.0,
                    //     *best.0.start()..=next_end
                    // );
                    self.ready_to_go = Some(gather);
                    self.gather = Some(*best.start()..=next_end);
                }
            } else {
                // if there is no gather, then set the gather to the best
                // println!(
                //     "cmk no gather,  capture front of best {:?} as {:?}",
                //     best.0,
                //     *best.0.start()..=next_end
                // );
                self.gather = Some(*best.start()..=next_end)
            };

            // We also update the workspace to removing any items that are completely covered by the new_start.
            // We also don't need to keep any items that have a lower priority and are shorter than the new best.
            let mut new_workspace = Vec::new();
            while let Some(item) = self.workspace.pop() {
                let mut item = item;
                if *item.end() <= next_end {
                    // too short, don't keep
                    // println!("cmk too short, don't keep in workspace {:?}", item.0);
                    continue; // while loop
                }
                item = next_end + T::one()..=*item.end();
                let Some(new_best) = new_workspace.first() else {
                    // println!("cmk no workspace, so keep {:?}", item.0);
                    // new_workspace is empty, so keep
                    new_workspace.push(item);
                    continue; // while loop
                };
                if item.end() <= new_best.end() {
                    // println!("cmk item is lower priority {:?} and shorter {:?} than best item {:?},{:?} in new workspace, so don't keep",
                    // item.priority, item.0, new_best.priority, new_best.0);
                    // not as good as new_best, and shorter, so don't keep
                    continue; // while loop
                }

                // higher priority or longer, so keep
                // println!("cmk item is higher priority {:?} or longer {:?} than best item {:?},{:?} in new workspace, so keep",
                // item.priority, item.0, new_best.priority, new_best.0);
                new_workspace.push(item);
            }
            self.workspace = new_workspace;
        } // end of main loop
    }
}

// #[allow(dead_code)]
// fn cmk_debug_string<'a, T>(item: &Option<RangeInclusive<T>>) -> String
// where
//     T: Integer,
// {
//     if let Some(item) = item {
//         format!("Some({:?})", item.0)
//     } else {
//         "None".to_string()
//     }
// }

impl<T, I> UnionIter<T, I>
where
    T: Integer,
    I: SortedStarts<T>,
{
    // cmk fix the comment on the set size. It should say inputs are SortedStarts not SortedDisjoint.
    /// Creates a new [`UnionIter`] from zero or more [`SortedStarts`] iterators. See [`UnionIter`] for more details and examples.
    pub fn new(mut iter: I) -> Self {
        let item = iter.next();
        Self {
            iter,
            next_item: item,
            workspace: Vec::new(),
            gather: None,
            ready_to_go: None,
        }
    }
}

impl<T, L, R> UnionIterMerge<T, L, R>
where
    T: Integer,
    L: SortedDisjoint<T>,
    R: SortedDisjoint<T>,
{
    // cmk fix the comment on the set size. It should say inputs are SortedStarts not SortedDisjoint.
    /// Creates a new [`SymDiffIter`] from zero or more [`SortedDisjoint`] iterators. See [`SymDiffIter`] for more details and examples.
    pub fn new2(left: L, right: R) -> Self {
        let iter: Merge<T, L, R> = Merge::new(left, right);
        Self::new(iter)
    }
}

/// cmk doc
impl<T, J> UnionIterKMerge<T, J>
where
    T: Integer,
    J: SortedDisjoint<T>,
{
    // cmk fix the comment on the set size. It should say inputs are SortedStarts not SortedDisjoint.
    /// Creates a new [`SymDiffIter`] from zero or more [`SortedDisjoint`] iterators. See [`SymDiffIter`] for more details and examples.
    pub fn new_k<K>(k: K) -> Self
    where
        K: IntoIterator<Item = J>,
    {
        let iter = KMerge::new(k);
        Self::new(iter)
    }
}

// from iter (T, &V) to UnionIter
impl<T> FromIterator<T> for UnionIter<T, SortedStartsInVec<T>>
where
    T: Integer,
{
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = T>,
    {
        let iter = iter.into_iter();
        UnionIter::from_iter(iter)
    }
}

// // from iter (RangeInclusive<T>, &V) to UnionIter
// impl<'a, T: Integer + 'a, V: ValueOwned + 'a> FromIterator<(RangeInclusive<T>, &'a V)>
//     for UnionIter<T, V, &'a V, SortedStartsInVec<T, V, &'a V>>
// {
//     fn from_iter<I>(iter: I) -> Self
//     where
//         I: IntoIterator<Item = (RangeInclusive<T>, &'a V)>,
//     {
//         let iter = iter.into_iter();
//         let iter = iter.map(|(range, value)| (range, value));
//         UnionIter::from_iter(iter)
//     }
// }

// cmk used?
#[allow(dead_code)]
type SortedRangeValueVec<T> = AssumeSortedStarts<T, vec::IntoIter<RangeInclusive<T>>>;

// cmk simplify the long types
// from iter (T, VR) to UnionIter
impl<T> FromIterator<RangeInclusive<T>> for UnionIter<T, SortedStartsInVec<T>>
where
    T: Integer,
{
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = RangeInclusive<T>>, // cmk000 add fused??
    {
        // cmk0000 simplify or optimize?
        let iter = iter.into_iter();
        let iter = UnsortedDisjoint::new(iter);
        let iter = iter.sorted_by(|a, b| a.start().cmp(&b.start()));
        let iter = AssumeSortedStarts::new(iter);
        UnionIter::new(iter)
    }
}

// // from from UnsortedDisjoint to UnionIter
// impl<T, I> From<I> for UnionIter<T, SortedStartsInVec<T>>
// where
//     T: Integer,
//     I: Iterator<Item = RangeInclusive<T>>,
// {
//     #[allow(clippy::clone_on_copy)]
//     fn from(unsorted_disjoint: I) -> Self {
//         let iter = unsorted_disjoint.sorted_by(|a, b| a.start().cmp(&b.start()));
//         let iter = AssumeSortedStarts::new(iter);
//         let result: UnionIter<T, AssumeSortedStarts<T, vec::IntoIter<RangeInclusive<T>>>> =
//             Self::new(iter);
//         result
//     }
// }

// cmk0 test that every iterator (that can be) is FusedIterator
impl<T, I> FusedIterator for UnionIter<T, I>
where
    T: Integer,
    I: SortedStarts<T> + FusedIterator,
{
}

// cmk
// impl<'a, T, I> ops::Not for UnionIter<'a, T, I>
// where
//     I: SortedStarts<T, V>,
// {
//     type Output = NotIter<T, V, Self>;

//     fn not(self) -> Self::Output {
//         self.complement()
//     }
// }

// impl<'a, T, R, L> ops::BitOr<R> for UnionIter<'a, T, L>
// where
//     T: Integer + 'a,
//     V: ValueOwned + 'a,
//     VR: CloneBorrow<V> + 'a,
//     L: SortedStarts<'a, T>,
//     R: SortedDisjoint<'a, T> + 'a,
// {
//     type Output = BitOrMerge<'a, T, Self, R>;

//     fn bitor(self, rhs: R) -> Self::Output {
//         // It might be fine to optimize to self.iter, but that would require
//         // also considering field 'range'
//         SortedDisjoint::union(self, rhs)
//     }
// }

// impl<'a, T, R, L> ops::Sub<R> for UnionIter<'a, T, L>
// where
//     L: SortedStarts<T, V>,
//     R: SortedDisjoint<T, V>,
// {
//     type Output = BitSubMerge<T, V, Self, R>;

//     fn sub(self, rhs: R) -> Self::Output {
//         SortedDisjoint::difference(self, rhs)
//     }
// }

// impl<'a, T, R, L> ops::BitXor<R> for UnionIter<'a, T, L>
// where
//     L: SortedStarts<T, V>,
//     R: SortedDisjoint<T, V>,
// {
//     type Output = BitXOrTee<T, V, Self, R>;

//     #[allow(clippy::suspicious_arithmetic_impl)]
//     fn bitxor(self, rhs: R) -> Self::Output {
//         SortedDisjoint::symmetric_difference(self, rhs)
//     }
// }

// impl<'a, T, R, L> ops::BitAnd<R> for UnionIter<'a, T, L>
// where
//     L: SortedStarts<T, V>,
//     R: SortedDisjoint<T, V>,
// {
//     type Output = BitAndMerge<T, V, Self, R>;

//     fn bitand(self, other: R) -> Self::Output {
//         SortedDisjoint::intersection(self, other)
//     }
// }

// impl<'a, T: Integer + 'a, V: ValueOwned + 'a, const N: usize> From<[(T, V); N]>
//     for UnionIter<'a, T, V, &'a V, SortedStartsInVec<'a, T, V, &'a V>>
// {
//     fn from(arr: [(T, &'a V); N]) -> Self {
//         // Directly create an iterator from the array and map it as needed
//         arr.iter()
//             .map(|&(t, v)| (t, v)) // This is a simple identity map; adjust as needed for your actual transformation
//             .collect() // Collect into UnionIter, relying on FromIterator
//     }
// }
