use core::{
    cmp::{max, min},
    iter::FusedIterator,
    ops::RangeInclusive,
};

use crate::Integer;
use crate::{
    map::{CloneRef, ValueRef},
    SortedDisjoint, SortedDisjointMap,
};

/// The output of the cmk
#[must_use = "iterators are lazy and do nothing unless consumed"]
#[derive(Clone, Debug)]
pub struct IntersectionIterMap<T, VR, IM, IS>
where
    T: Integer,
    VR: CloneRef<VR::Value> + ValueRef,
    IM: SortedDisjointMap<T, VR>,
    IS: SortedDisjoint<T>,
{
    iter_left: IM,
    iter_right: IS,
    right: Option<RangeInclusive<T>>,
    left: Option<(RangeInclusive<T>, VR)>,
}

impl<T, VR, IM, IS> IntersectionIterMap<T, VR, IM, IS>
where
    T: Integer,
    VR: CloneRef<VR::Value> + ValueRef,
    IM: SortedDisjointMap<T, VR>,
    IS: SortedDisjoint<T>,
{
    // cmk fix the comment on the set size. It should say inputs are SortedStarts not SortedDisjoint.
    /// Creates a new [`IntersectionIterMap`] from zero or more [`SortedStartsMap`] iterators. See [`IntersectionIterMap`] for more details and examples.
    ///
    /// [`SortedStartsMap`]: crate::sorted_disjoint_map::SortedStartsMap
    // cmk #[allow(dead_code)]
    pub const fn new(iter_map: IM, iter_set: IS) -> Self {
        Self {
            iter_left: iter_map,
            iter_right: iter_set,
            right: None,
            left: None,
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

impl<T, VR, IM, IS> FusedIterator for IntersectionIterMap<T, VR, IM, IS>
where
    T: Integer,
    VR: CloneRef<VR::Value> + ValueRef,
    IM: SortedDisjointMap<T, VR>,
    IS: SortedDisjoint<T>,
{
}

impl<T, VR, IM, IS> Iterator for IntersectionIterMap<T, VR, IM, IS>
where
    T: Integer,
    VR: CloneRef<VR::Value> + ValueRef,
    IM: SortedDisjointMap<T, VR>,
    IS: SortedDisjoint<T>,
{
    type Item = (RangeInclusive<T>, VR);

    fn next(&mut self) -> Option<(RangeInclusive<T>, VR)> {
        // println!("cmk begin next");
        loop {
            // Be sure both currents are loaded.
            self.left = self.left.take().or_else(|| self.iter_left.next());
            self.right = self.right.take().or_else(|| self.iter_right.next());

            // If either is still none, we are done.
            let (Some(left), Some(right)) = (self.left.take(), self.right.take()) else {
                return None;
            };
            let (left_start, left_end) = left.0.clone().into_inner();
            let (right_start, right_end) = right.into_inner();
            // println!("cmk {:?} {:?}", current_range, current_range_value.0);

            // if current_range ends before current_range_value, clear it and loop for a new value.
            if right_end < left_start {
                // println!("cmk getting new range");
                self.right = None;
                self.left = Some(left);
                continue;
            }

            // if current_range_value ends before current_range, clear it and loop for a new value.
            if left_end < right_start {
                // println!("cmk getting new range value");
                self.right = Some(RangeInclusive::new(right_start, right_end));
                self.left = None;
                continue;
            }

            // Thus, they overlap
            let start = max(right_start, left_start);
            let end = min(right_end, left_end);

            // remove any ranges that match "end" and set them None

            let value = match (right_end == end, left_end == end) {
                (true, true) => {
                    self.right = None;
                    self.left = None;
                    left.1
                }
                (true, false) => {
                    self.right = None;
                    let value = ValueRef::clone_ref(&left.1); // cmk use method
                    self.left = Some(left);
                    value
                }
                (false, true) => {
                    self.right = Some(RangeInclusive::new(right_start, right_end));
                    self.left = None;
                    left.1
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

#[test]
fn cmk_delete_me5() {
    use crate::prelude::*;

    let map = CheckSortedDisjointMap::new([(1..=2, &"a"), (5..=100, &"a")]);
    let set = CheckSortedDisjoint::new([2..=6]);
    let intersection = IntersectionIterMap::new(map, set);
    assert_eq!(intersection.into_string(), r#"(2..=2, "a"), (5..=6, "a")"#);

    // Or, equivalently:
    let a = CheckSortedDisjointMap::new([(1..=2, &"a"), (5..=100, &"a")]);
    let b = CheckSortedDisjointMap::new([(2..=6, &"b")]);
    let intersection = a & b;
    assert_eq!(intersection.into_string(), r#"(2..=2, "a"), (5..=6, "a")"#);
}
