use core::{
    cmp::max,
    iter::FusedIterator,
    ops::{self, RangeInclusive},
};

use alloc::vec;
use itertools::Itertools;

use crate::Integer;
use crate::{map::BitOrMergeMap, unsorted_disjoint_map::AssumeSortedStartsMap};
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
    V: PartialEq,
    I: SortedStartsMap<'a, T, V>,
{
    pub(crate) iter: I,
    pub(crate) option_range_value: Option<RangeValue<T, &'a V>>,
}

impl<'a, T, V, I> UnionIterMap<'a, T, V, I>
where
    T: Integer,
    V: PartialEq,
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

// impl<T: Integer, V: PartialEq, const N: usize> From<[T; N]>
//     for UnionIterMap<T, V, SortedRangeInclusiveVec<T, V>>
// {
//     fn from(arr: [T; N]) -> Self {
//         arr.as_slice().into()
//     }
// }

// impl<T: Integer, V: PartialEq> From<&[T]> for UnionIterMap<T, V, SortedRangeInclusiveVec<T, V>> {
//     fn from(slice: &[T]) -> Self {
//         slice.iter().cloned().collect()
//     }
// }

// impl<T: Integer, V: PartialEq, const N: usize> From<[RangeValue<T, V>; N]>
//     for UnionIterMap<T, V, SortedRangeInclusiveVec<T, V>>
// {
//     fn from(arr: [RangeValue<T, V>; N]) -> Self {
//         arr.as_slice().into()
//     }
// }

type SortedRangeInclusiveVec<'a, T, V> =
    AssumeSortedStartsMap<'a, T, V, vec::IntoIter<RangeValue<T, &'a V>>>;

// from iter (T, V) to UnionIterMap
impl<'a, T: Integer, V: PartialEq + 'a> FromIterator<(T, &'a V)>
    for UnionIterMap<'a, T, V, SortedRangeInclusiveVec<'a, T, V>>
{
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = (T, &'a V)>,
    {
        iter.into_iter().map(|(x, value)| (x..=x, value)).collect()
    }
}

// from iter (RangeInclusive<T>, V) to UnionIterMap
impl<'a, T: Integer, V: PartialEq + 'a> FromIterator<(RangeInclusive<T>, &'a V)>
    for UnionIterMap<'a, T, V, SortedRangeInclusiveVec<'a, T, V>>
{
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = (RangeInclusive<T>, &'a V)>,
    {
        let iter = iter.into_iter();
        let iter = iter.map(|(range, value)| RangeValue { range, value });
        iter.collect()
    }
}

// from iter RangeValue<T, V> to UnionIterMap
impl<'a, T: Integer, V: PartialEq + 'a> FromIterator<RangeValue<T, &'a V>>
    for UnionIterMap<'a, T, V, SortedRangeInclusiveVec<'a, T, V>>
{
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = RangeValue<T, &'a V>>,
    {
        UnsortedDisjointMap::from(iter.into_iter()).into()
    }
}

// from from UnsortedDisjointMap to UnionIterMap
impl<'a, T, V, I> From<UnsortedDisjointMap<'a, T, V, I>>
    for UnionIterMap<'a, T, V, SortedRangeInclusiveVec<'a, T, V>>
where
    T: Integer,
    V: PartialEq + 'a,
    I: Iterator<Item = RangeValue<T, &'a V>>, // Any iterator is OK, because we will sort
{
    fn from(unsorted_disjoint: UnsortedDisjointMap<'a, T, V, I>) -> Self {
        let iter = AssumeSortedStartsMap {
            iter: unsorted_disjoint.sorted_by_key(|range_value| range_value.range.start()),
        };
        Self {
            iter,
            option_range_value: None,
        }
    }
}

impl<'a, T: Integer, V: PartialEq, I> FusedIterator for UnionIterMap<'a, T, V, I> where
    I: SortedStartsMap<'a, T, V> + FusedIterator
{
}

impl<'a, T: Integer, V: PartialEq, I> Iterator for UnionIterMap<'a, T, V, I>
where
    I: SortedStartsMap<'a, T, V>,
{
    type Item = RangeValue<T, &'a V>;

    fn next(&mut self) -> Option<RangeValue<T, &'a V>> {
        loop {
            let range_value = match self.iter.next() {
                Some(range_value) => range_value,
                None => return self.option_range_value.take(),
            };

            let (start, end) = range_value.range.into_inner();
            if end < start {
                continue;
            }

            let current_range_value = match self.option_range_value {
                Some(crv) => crv,
                None => {
                    let crv = RangeValue {
                        range: start..=end,
                        value: range_value.value,
                    };
                    self.option_range_value = Some(crv);
                    continue;
                }
            };

            let (current_start, current_end) = current_range_value.range.into_inner();
            debug_assert!(current_start <= start); // real assert
            if start <= current_end
                || (current_end < T::safe_max_value() && start <= current_end + T::one())
            {
                let crv = RangeValue {
                    range: current_start..=max(current_end, end),
                    value: current_range_value.value,
                };
                self.option_range_value = Some(crv);
                continue;
            } else {
                let cr0 = RangeValue {
                    range: start..=end,
                    value: range_value.value,
                };
                self.option_range_value = Some(cr0);
                let cr1 = RangeValue {
                    range: current_start..=current_end,
                    value: current_range_value.value,
                };
                return Some(cr1);
            }
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
// impl<T: Integer, V: PartialEq, I> ops::Not for UnionIterMap<T, V, I>
// where
//     I: SortedStartsMap<T, V>,
// {
//     type Output = NotIterMap<T, V, Self>;

//     fn not(self) -> Self::Output {
//         self.complement()
//     }
// }

impl<'a, T: Integer, V: PartialEq + 'a, R, L> ops::BitOr<R> for UnionIterMap<'a, T, V, L>
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

// impl<T: Integer, V: PartialEq, R, L> ops::Sub<R> for UnionIterMap<T, V, L>
// where
//     L: SortedStartsMap<T, V>,
//     R: SortedDisjointMap<T, V>,
// {
//     type Output = BitSubMergeMap<T, V, Self, R>;

//     fn sub(self, rhs: R) -> Self::Output {
//         SortedDisjointMap::difference(self, rhs)
//     }
// }

// impl<T: Integer, V: PartialEq, R, L> ops::BitXor<R> for UnionIterMap<T, V, L>
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

// impl<T: Integer, V: PartialEq, R, L> ops::BitAnd<R> for UnionIterMap<T, V, L>
// where
//     L: SortedStartsMap<T, V>,
//     R: SortedDisjointMap<T, V>,
// {
//     type Output = BitAndMergeMap<T, V, Self, R>;

//     fn bitand(self, other: R) -> Self::Output {
//         SortedDisjointMap::intersection(self, other)
//     }
// }
