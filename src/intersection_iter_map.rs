use core::{
    cmp::{max, min},
    iter::FusedIterator,
    marker::PhantomData,
    ops::RangeInclusive,
};

use crate::{map::CloneBorrow, SortedDisjoint, SortedDisjointMap};
use crate::{map::ValueOwned, Integer};

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
#[allow(dead_code)]
#[must_use = "iterators are lazy and do nothing unless consumed"]
pub struct IntersectionIterMap<T, V, VR, IM, IS>
where
    T: Integer,
    V: ValueOwned,
    VR: CloneBorrow<V>,
    IM: SortedDisjointMap<T, V, VR>,
    IS: SortedDisjoint<T>,
{
    iter_map: IM,
    iter_set: IS,
    current_range: Option<RangeInclusive<T>>,
    current_range_value: Option<(RangeInclusive<T>, VR)>,
    phantom: PhantomData<V>,
}

impl<'a, T, V, VR, IM, IS> IntersectionIterMap<T, V, VR, IM, IS>
where
    T: Integer,
    V: ValueOwned,
    VR: CloneBorrow<V>,
    IM: SortedDisjointMap<T, V, VR>,
    IS: SortedDisjoint<T>,
{
    // cmk fix the comment on the set size. It should say inputs are SortedStarts not SortedDisjoint.
    /// Creates a new [`IntersectionIterMap`] from zero or more [`SortedStartsMap`] iterators. See [`IntersectionIterMap`] for more details and examples.
    #[allow(dead_code)]
    pub fn new(iter_map: IM, iter_set: IS) -> Self {
        Self {
            iter_map,
            iter_set,
            current_range: None,
            current_range_value: None,
            phantom: PhantomData,
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

impl<'a, T, V, VR, IM, IS> FusedIterator for IntersectionIterMap<T, V, VR, IM, IS>
where
    T: Integer,
    V: ValueOwned,
    VR: CloneBorrow<V>,
    IM: SortedDisjointMap<T, V, VR>,
    IS: SortedDisjoint<T>,
{
}

impl<'a, T, V, VR, IM, IS> Iterator for IntersectionIterMap<T, V, VR, IM, IS>
where
    T: Integer,
    V: ValueOwned,
    VR: CloneBorrow<V>,
    IM: SortedDisjointMap<T, V, VR>,
    IS: SortedDisjoint<T>,
{
    type Item = (RangeInclusive<T>, VR);

    fn next(&mut self) -> Option<(RangeInclusive<T>, VR)> {
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
            let (current_range_start, current_range_end) = current_range.into_inner();
            let (current_range_value_start, current_range_value_end) =
                (&current_range_value).0.clone().into_inner();
            // println!("cmk {:?} {:?}", current_range, current_range_value.range);

            // if current_range ends before current_range_value, clear it and loop for a new value.
            if current_range_end < current_range_value_start {
                // println!("cmk getting new range");
                self.current_range = None;
                self.current_range_value = Some(current_range_value);
                continue;
            }

            // if current_range_value ends before current_range, clear it and loop for a new value.
            // cmk00 do I want to assign .0 to something?
            if current_range_value_end < current_range_start {
                // println!("cmk getting new range value");
                self.current_range =
                    Some(RangeInclusive::new(current_range_start, current_range_end));
                self.current_range_value = None;
                continue;
            }

            // Thus, they overlap
            let start = max(current_range_start, current_range_value_start);
            let end = min(current_range_end, current_range_value_end);

            // remove any ranges that match "end" and set them None

            let value = match (current_range_end == end, current_range_value_end == end) {
                (true, true) => {
                    self.current_range = None;
                    self.current_range_value = None;
                    current_range_value.1
                }
                (true, false) => {
                    self.current_range = None;
                    let value = current_range_value.1.clone_borrow();
                    self.current_range_value = Some(current_range_value);
                    value
                }
                (false, true) => {
                    self.current_range =
                        Some(RangeInclusive::new(current_range_start, current_range_end));
                    self.current_range_value = None;
                    current_range_value.1
                }
                (false, false) => {
                    panic!("impossible case")
                }
            };

            let range_value = (start..=end, value);
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

// impl<'a, T, V, VR, IM, IS> SortedStartsMap<'a, T, V, VR>
//     for IntersectionIterMap<'a, T, V, VR, IM, IS>
// where
//     T: Integer,
//     V: ValueOwned,
//     VR: CloneBorrow<V> + 'a,
//     IM: SortedDisjointMap<'a, T, V, VR> + 'a,
//     IS: SortedDisjoint<T>,
// {
// }
// impl<'a, T, V, VR, IM, IS> SortedDisjointMap<'a, T, V, VR>
//     for IntersectionIterMap<'a, T, V, VR, IM, IS>
// where
//     T: Integer,
//     V: ValueOwned,
//     VR: CloneBorrow<V> + 'a,
//     IM: SortedDisjointMap<'a, T, V, VR> + 'a,
//     IS: SortedDisjoint<T>,
// {
// }
