use core::{
    cmp::{max, min},
    iter::FusedIterator,
    marker::PhantomData,
    ops::{self, RangeInclusive},
};

use alloc::vec;
use itertools::Itertools;

use crate::{map::BitOrMergeMap, unsorted_disjoint_map::AssumeSortedStartsMap};
use crate::{map::ValueOwned, Integer};
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
pub struct UnionIterMap<'a, T, V, I>
where
    T: Integer,
    V: ValueOwned,
    I: SortedStartsMap<'a, T, V>,
{
    iter: <std::vec::Vec<RangeValue<'a, T, V>> as std::iter::IntoIterator>::IntoIter,
    phantom: PhantomData<I>,
    // option_range_value: Option<RangeValue<'a, T, V>>,
}

impl<'a, T, V, I> UnionIterMap<'a, T, V, I>
where
    T: Integer,
    V: ValueOwned,
    I: SortedStartsMap<'a, T, V>,
{
    // cmk fix the comment on the set size. It should say inputs are SortedStarts not SortedDisjoint.
    /// Creates a new [`UnionIterMap`] from zero or more [`SortedStartsMap`] iterators. See [`UnionIterMap`] for more details and examples.
    pub fn new(iter: I) -> Self {
        // By default all ends are inclusive (different that most programs)
        let mut vec_in = iter.collect_vec();
        println!("vec_in: {:?}", vec_in.len());
        let mut vec_mid = Vec::<RangeValue<'a, T, V>>::new();
        let mut workspace = Vec::<RangeValue<'a, T, V>>::new();
        let mut bar_priority = 0usize;
        let mut bar_end = T::zero();
        while !vec_in.is_empty() || !workspace.is_empty() {
            if !vec_in.is_empty() {
                // find the index (of any) of the first index with a different start that the first one
                let first_start = *vec_in[0].range.start();
                // if bar_end is None, set it to first_start
                bar_end = max(bar_end, first_start);
                let first_diff = vec_in.iter().position(|x| *x.range.start() != first_start);
                // If None, then set it to the length
                let first_diff = first_diff.unwrap_or(vec_in.len());
                // set same_start to the first first_diff elements. Allocate for this
                // remove the first first_diff elements from vec_in. do this in place.
                let same_starts: Vec<_> = vec_in.drain(0..first_diff).collect();
                for same_start in same_starts {
                    if same_start.priority < bar_priority && same_start.range.end() < &bar_end {
                        continue;
                    }
                    if same_start.priority >= bar_priority {
                        bar_priority = same_start.priority;
                        bar_end = *same_start.range.end();
                    }
                    workspace.push(same_start);
                }
            }

            // find the one element with priority = bar_priority
            // cmk use priority queue
            let index_of_best = workspace.iter().position(|x| x.priority == bar_priority);
            let best = &workspace[index_of_best.unwrap()];
            // output_end is the smallest wend in workspace
            let mut output_end = *workspace.iter().map(|x| x.range.end()).min().unwrap();
            // if vec_is is not empty, then output_end is the minimum of output_end and the start of the first element in vec_in -1
            if !vec_in.is_empty() {
                let next_start = *vec_in[0].range.start();
                // cmk underflow?
                output_end = min(output_end, next_start - T::one())
            };
            let first_start = *workspace[0].range.start();
            vec_mid.push(RangeValue {
                range: first_start..=output_end,
                value: best.value,
                priority: best.priority,
            });
            // trim the start of the ranges in workspace to output_end+1, remove any that are empty
            // also find the best priority and the new bar_end
            workspace.retain(|x| *x.range.end() > output_end);
            // cmk check for overflow?
            let new_start = output_end + T::one();
            bar_priority = 0;
            bar_end = output_end;
            for x in workspace.iter_mut() {
                x.range = new_start..=*x.range.end();
                if x.priority > bar_priority {
                    bar_priority = x.priority;
                    bar_end = *x.range.end();
                }
            }
        }

        let mut vec_out = Vec::<RangeValue<'a, T, V>>::new();
        let mut index = 0;
        while index < vec_mid.len() {
            let mut index_exclusive_end = index;
            while index_exclusive_end < vec_mid.len()
                && vec_mid[index_exclusive_end].value == vec_mid[index].value
            {
                index_exclusive_end += 1;
            }
            vec_out.push(RangeValue {
                range: *vec_mid[index].range.start()
                    ..=*vec_mid[index_exclusive_end - 1].range.end(),
                value: vec_mid[index].value,
                priority: 0, // cmk priority should never be exposed or re-used.
            });
            index = index_exclusive_end;
        }

        Self {
            iter: vec_out.into_iter(),
            phantom: PhantomData,
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

pub(crate) type SortedRangeInclusiveVec<'a, T, V> =
    AssumeSortedStartsMap<'a, T, V, vec::IntoIter<RangeValue<'a, T, V>>>;

// from iter (T, V) to UnionIterMap
impl<'a, T: Integer + 'a, V: ValueOwned + 'a> FromIterator<(T, &'a V)>
    for UnionIterMap<'a, T, V, SortedRangeInclusiveVec<'a, T, V>>
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
    for UnionIterMap<'a, T, V, SortedRangeInclusiveVec<'a, T, V>>
{
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = (RangeInclusive<T>, &'a V)>,
    {
        let iter = iter.into_iter();
        let iter = iter.enumerate();
        let iter = iter.map(|(priority, (range, value))| RangeValue {
            range,
            value,
            priority,
        });
        let iter: UnionIterMap<'a, T, V, SortedRangeInclusiveVec<'a, T, V>> =
            UnionIterMap::from_iter(iter);
        iter
    }
}

// from iter RangeValue<T, V> to UnionIterMap
impl<'a, T: Integer + 'a, V: ValueOwned + 'a> FromIterator<RangeValue<'a, T, V>>
    for UnionIterMap<'a, T, V, SortedRangeInclusiveVec<'a, T, V>>
{
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = RangeValue<'a, T, V>>,
    {
        UnsortedDisjointMap::from(iter.into_iter()).into()
    }
}

// from from UnsortedDisjointMap to UnionIterMap
impl<'a, T, V, I> From<UnsortedDisjointMap<'a, T, V, I>>
    for UnionIterMap<'a, T, V, SortedRangeInclusiveVec<'a, T, V>>
where
    T: Integer,
    V: ValueOwned + 'a,
    I: Iterator<Item = RangeValue<'a, T, V>>,
{
    #[allow(clippy::clone_on_copy)]
    fn from(unsorted_disjoint: UnsortedDisjointMap<'a, T, V, I>) -> Self {
        let iter = unsorted_disjoint.sorted_by(|a, b| match a.range.start().cmp(b.range.start()) {
            std::cmp::Ordering::Equal => b.priority.cmp(&a.priority),
            other => other,
        });
        let iter = AssumeSortedStartsMap { iter };

        Self::new(iter)
    }
}

impl<'a, T: Integer, V: ValueOwned, I> FusedIterator for UnionIterMap<'a, T, V, I> where
    I: SortedStartsMap<'a, T, V> + FusedIterator
{
}

impl<'a, T: Integer, V: ValueOwned, I> Iterator for UnionIterMap<'a, T, V, I>
where
    I: SortedStartsMap<'a, T, V>,
{
    type Item = RangeValue<'a, T, V>;

    fn next(&mut self) -> Option<RangeValue<'a, T, V>> {
        self.iter.next()
    }

    // // There could be a few as 1 (or 0 if the iter is empty) or as many as the iter.
    // // Plus, possibly one more if we have a range is in progress.
    // fn size_hint(&self) -> (usize, Option<usize>) {
    //     let (low, high) = self.iter.size_hint();
    //     let low = low.min(1);
    //     if self.option_range_value.is_some() {
    //         (low, high.map(|x| x + 1))
    //     } else {
    //         (low, high)
    //     }
    // }
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

impl<'a, T: Integer, V: ValueOwned + 'a, R, L> ops::BitOr<R> for UnionIterMap<'a, T, V, L>
where
    L: SortedStartsMap<'a, T, V>,
    R: SortedDisjointMap<'a, T, V>,
{
    type Output = BitOrMergeMap<'a, T, V, Self, R>;

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
