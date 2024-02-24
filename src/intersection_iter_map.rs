use core::{
    borrow::Borrow,
    cmp::{max, min},
    iter::FusedIterator,
    marker::PhantomData,
    ops::{self, RangeInclusive},
};

use alloc::vec;
use itertools::Itertools;

use crate::{map::BitOrMergeMap, unsorted_disjoint_map::AssumeSortedStartsMap, SortedDisjoint};
use crate::{map::ValueOwned, Integer};
use crate::{
    sorted_disjoint_map::{RangeValue, SortedDisjointMap, SortedStartsMap},
    unsorted_disjoint_map::UnsortedDisjointMap,
};

/// Turns one [`SortedDisjoint`] iterator and one [`SortedDisjointMap`] iterator into
/// the [`SortedDisjointMap`] iterator of their intersection,
///
/// cmk
///
/// [`SortedDisjointMap`]: crate::SortedDisjointMap
/// [`Merge`]: crate::Merge
/// [`KMerge`]: crate::KMerge
///
/// # Examples
///
/// ```
/// use itertools::Itertools;
/// use range_set_blaze::{IntersectionIterMap, Merge, SortedDisjointMap, CheckSortedDisjoint};
///
/// let a = CheckSortedDisjoint::new(vec![1..=2, 5..=100].into_iter());
/// let b = CheckSortedDisjoint::from([2..=6]);
/// let intersection = IntersectionIterMap::new(Merge::new(a, b));
/// assert_eq!(intersection.to_string(), "1..=100");
///
/// // Or, equivalently:
/// let a = CheckSortedDisjoint::new(vec![1..=2, 5..=100].into_iter());
/// let b = CheckSortedDisjoint::from([2..=6]);
/// let intersection = a | b;
/// assert_eq!(intersection.to_string(), "1..=100")
/// ```
// cmk #[derive(Clone, Debug)]
#[must_use = "iterators are lazy and do nothing unless consumed"]
pub struct IntersectionIterMap<'a, T, V, VR, IS, IM>
where
    T: Integer,
    V: ValueOwned,
    VR: Borrow<V> + 'a,
    IS: SortedDisjoint<T>,
    IM: SortedDisjointMap<'a, T, V, VR> + 'a,
{
    iter_set: IS,
    iter_map: IM,
    current_range: Option<RangeInclusive<T>>,
    current_range_value: Option<RangeValue<'a, T, V, VR>>,
    _phantom0: PhantomData<&'a T>,
    _phantom1: PhantomData<&'a V>,
}

impl<'a, T, V, VR, IS, IM> IntersectionIterMap<'a, T, V, VR, IS, IM>
where
    T: Integer,
    V: ValueOwned,
    VR: Borrow<V> + 'a,
    IS: SortedDisjoint<T>,
    IM: SortedDisjointMap<'a, T, V, VR> + 'a,
{
    // cmk fix the comment on the set size. It should say inputs are SortedStarts not SortedDisjoint.
    /// Creates a new [`IntersectionIterMap`] from zero or more [`SortedStartsMap`] iterators. See [`IntersectionIterMap`] for more details and examples.
    pub fn new(iter_set: IS, iter_map: IM) -> Self {
        Self {
            iter_set,
            iter_map,
            current_range: None,
            current_range_value: None,
            _phantom0: PhantomData,
            _phantom1: PhantomData,
        }
    }
}

// impl<T: Integer, V: PartialEqClone, const N: usize> From<[T; N]>
//     for IntersectionIterMap<T, V, SortedRangeInclusiveVec<T, V>>
// {
//     fn from(arr: [T; N]) -> Self {
//         arr.as_slice().into()
//     }
// }

// impl<T: Integer, V: PartialEqClone> From<&[T]> for IntersectionIterMap<T, V, SortedRangeInclusiveVec<T, V>> {
//     fn from(slice: &[T]) -> Self {
//         slice.iter().cloned().collect()
//     }
// }

// impl<T: Integer, V: PartialEqClone, const N: usize> From<[RangeValue<T, V>; N]>
//     for IntersectionIterMap<T, V, SortedRangeInclusiveVec<T, V>>
// {
//     fn from(arr: [RangeValue<T, V>; N]) -> Self {
//         arr.as_slice().into()
//     }
// }

impl<'a, T, V, VR, IS, IM> Iterator for IntersectionIterMap<'a, T, V, VR, IS, IM>
where
    T: Integer,
    V: ValueOwned,
    VR: Borrow<V> + 'a,
    IS: SortedDisjoint<T>,
    IM: SortedDisjointMap<'a, T, V, VR>,
{
    type Item = RangeValue<'a, T, V, VR>;

    fn next(&mut self) -> Option<RangeValue<'a, T, V, VR>> {
        // println!("cmk begin next");
        loop {
            // Be sure both currents are loaded.
            self.current_range = self.current_range.take().or_else(|| self.iter_set.next());
            self.current_range_value = self
                .current_range_value
                .take()
                .or_else(|| self.iter_map.next());

            // If either is still none, we are done.
            let (Some(current_range), Some(current_range_value)) =
                (self.current_range.take(), self.current_range_value.take())
            else {
                return None;
            };
            // println!("cmk {:?} {:?}", current_range, current_range_value.range);

            // if current_range ends before current_range_value, clear it and loop for a new value.
            if current_range.end() < current_range_value.range.start() {
                // println!("cmk getting new range");
                self.current_range = None;
                self.current_range_value = Some(current_range_value);
                continue;
            }

            // if current_range_value ends before current_range, clear it and loop for a new value.
            if current_range_value.range.end() < current_range.start() {
                // println!("cmk getting new range value");
                self.current_range = Some(current_range);
                self.current_range_value = None;
                continue;
            }

            // Thus, they overlap
            let start = *max(current_range.start(), current_range_value.range.start());
            let end = *min(current_range.end(), current_range_value.range.end());
            let range_value = RangeValue {
                range: start..=end,
                value: current_range_value.value,
                priority: 0,
                phantom: PhantomData,
            };

            // remove any ranges that match "end" and set them None
            self.current_range = if *current_range.end() == end {
                None
            } else {
                Some(current_range)
            };
            self.current_range_value = if *current_range_value.range.end() == end {
                None
            } else {
                Some(current_range_value)
            };
            return Some(range_value);
        }
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
// impl<T: Integer, V: PartialEqClone, I> ops::Not for IntersectionIterMap<T, V, I>
// where
//     I: SortedStartsMap<T, V>,
// {
//     type Output = NotIterMap<T, V, Self>;

//     fn not(self) -> Self::Output {
//         self.complement()
//     }
// }

// impl<'a, T: Integer, V: ValueOwned + 'a, R, L> ops::BitOr<R> for IntersectionIterMap<'a, T, V, L>
// where
//     L: SortedStartsMap<'a, T, V>,
//     R: SortedDisjointMap<'a, T, V>,
// {
//     type Output = BitOrMergeMap<'a, T, V, Self, R>;

//     fn bitor(self, rhs: R) -> Self::Output {
//         // It might be fine to optimize to self.iter, but that would require
//         // also considering field 'range'
//         SortedDisjointMap::intersection(self, rhs)
//     }
// }

// impl<T: Integer, V: PartialEqClone, R, L> ops::Sub<R> for IntersectionIterMap<T, V, L>
// where
//     L: SortedStartsMap<T, V>,
//     R: SortedDisjointMap<T, V>,
// {
//     type Output = BitSubMergeMap<T, V, Self, R>;

//     fn sub(self, rhs: R) -> Self::Output {
//         SortedDisjointMap::difference(self, rhs)
//     }
// }

// impl<T: Integer, V: PartialEqClone, R, L> ops::BitXor<R> for IntersectionIterMap<T, V, L>
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

// impl<T: Integer, V: PartialEqClone, R, L> ops::BitAnd<R> for IntersectionIterMap<T, V, L>
// where
//     L: SortedStartsMap<T, V>,
//     R: SortedDisjointMap<T, V>,
// {
//     type Output = BitAndMergeMap<T, V, Self, R>;

//     fn bitand(self, other: R) -> Self::Output {
//         SortedDisjointMap::intersection(self, other)
//     }
// }
