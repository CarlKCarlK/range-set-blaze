use core::{
    cmp::max,
    iter::FusedIterator,
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
    pub(crate) iter: I,
    pub(crate) option_range_value: Option<RangeValue<'a, T, V>>,
}

impl<'a, T, V, I> UnionIterMap<'a, T, V, I>
where
    T: Integer,
    V: ValueOwned,
    I: SortedStartsMap<'a, T, V>,
{
    /// Creates a new [`UnionIterMap`] from zero or more [`SortedDisjointMap`] iterators. See [`UnionIterMap`] for more details and examples.
    pub fn new(iter: I) -> Self {
        Self {
            iter,
            option_range_value: None,
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

        Self {
            iter,
            option_range_value: None,
        }
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
        loop {
            // If there is no next range, return the current range (if any)
            let Some(next_range_value) = self.iter.next() else {
                return self.option_range_value.take();
            };

            // if the next range is empty, try again
            let (next_start, next_end) = next_range_value.range.clone().into_inner();
            if next_end < next_start {
                continue;
            }

            // if the current range is empty, replace it with the next range and try again
            let Some(current_range_value) = self.option_range_value.take() else {
                self.option_range_value = Some(next_range_value);
                continue;
            };

            let (current_start, current_end) = current_range_value.range.clone().into_inner();
            debug_assert!(current_start <= next_start); // real assert

            // if not touching or overlapping  then return the current range and replace it with the next range
            let touch_or_overlap = next_start <= current_end
                || (current_end < T::safe_max_value() && next_start <= current_end + T::one());
            if !touch_or_overlap {
                self.option_range_value = Some(next_range_value);
                return Some(current_range_value);
            }

            // if touching or overlapping and the values are the same, then merge the ranges and try again
            let same_value = next_range_value.value == current_range_value.value;
            if touch_or_overlap && same_value {
                let crv = RangeValue {
                    range: current_start..=max(current_end, next_end),
                    value: current_range_value.value,
                    priority: current_range_value.priority,
                };
                self.option_range_value = Some(crv);
                continue;
            }

            debug_assert!(touch_or_overlap && !same_value); // real assert
                                                            // if current starts before next, we could return a range from current to next_start - 1 but the remainders remain in play.
            todo!("return a range from current to next_start - 1");

            debug_assert!(current_start == next_start);
        }
    }

    // There could be a few as 1 (or 0 if the iter is empty) or as many as the iter.
    // Plus, possibly one more if we have a range is in progress.
    fn size_hint(&self) -> (usize, Option<usize>) {
        let (low, high) = self.iter.size_hint();
        let low = low.min(1);
        if self.option_range_value.is_some() {
            (low, high.map(|x| x + 1))
        } else {
            (low, high)
        }
    }
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
