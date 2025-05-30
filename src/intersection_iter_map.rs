use core::{
    cmp::{max, min},
    iter::FusedIterator,
    ops::RangeInclusive,
};

use crate::Integer;
use crate::{SortedDisjoint, SortedDisjointMap, map::ValueRef};

/// This `struct` is created by the [`intersection`] and [`map_and_set_intersection`] methods on [`SortedDisjointMap`].
/// See the methods' documentation for more.
///
/// [`SortedDisjointMap`]: crate::SortedDisjointMap
/// [`intersection`]: crate::SortedDisjointMap::intersection
/// [`map_and_set_intersection`]: crate::SortedDisjointMap::map_and_set_intersection
#[must_use = "iterators are lazy and do nothing unless consumed"]
#[derive(Clone, Debug)]
pub struct IntersectionIterMap<T, VR, IM, IS>
where
    T: Integer,
    VR: ValueRef,
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
    VR: ValueRef,
    IM: SortedDisjointMap<T, VR>,
    IS: SortedDisjoint<T>,
{
    pub(crate) const fn new(iter_map: IM, iter_set: IS) -> Self {
        Self {
            iter_left: iter_map,
            iter_right: iter_set,
            right: None,
            left: None,
        }
    }
}

impl<T, VR, IM, IS> FusedIterator for IntersectionIterMap<T, VR, IM, IS>
where
    T: Integer,
    VR: ValueRef,
    IM: SortedDisjointMap<T, VR> + FusedIterator,
    IS: SortedDisjoint<T> + FusedIterator,
{
}

// Note: ExactSizeIterator cannot be safely implemented for IntersectionIterMap
// because we can't determine the exact number of items that will be produced
// by the intersection without fully processing both iterators.
// An upper bound would be min(left.len(), right.len()), but that's not exact.

impl<T, VR, IM, IS> Iterator for IntersectionIterMap<T, VR, IM, IS>
where
    T: Integer,
    VR: ValueRef,
    IM: SortedDisjointMap<T, VR>,
    IS: SortedDisjoint<T>,
{
    type Item = (RangeInclusive<T>, VR);

    fn next(&mut self) -> Option<(RangeInclusive<T>, VR)> {
        // println!("begin next");
        loop {
            // Be sure both currents are loaded.
            self.left = self.left.take().or_else(|| self.iter_left.next());
            self.right = self.right.take().or_else(|| self.iter_right.next());

            // If either is still none, we are done.
            let (Some(left), Some(right)) = (self.left.take(), self.right.take()) else {
                return None;
            };
            let (left_range, left_value) = left;
            let (left_start, left_end) = left_range.clone().into_inner();
            let (right_start, right_end) = right.into_inner();
            // println!("{:?} {:?}", current_range, current_range_value.0);

            // if current_range ends before current_range_value, clear it and loop for a new value.
            if right_end < left_start {
                // println!("getting new range");
                self.right = None;
                self.left = Some((left_range, left_value));
                continue;
            }

            // if current_range_value ends before current_range, clear it and loop for a new value.
            if left_end < right_start {
                // println!("getting new range value");
                self.right = Some(RangeInclusive::new(right_start, right_end));
                self.left = None;
                continue;
            }

            // Thus, they overlap
            let start = max(right_start, left_start);
            let end = min(right_end, left_end);

            // Modified logic: Now prioritize right range boundaries instead of left
            let value = if left_end != end {
                // right_end != end, left_end != end is impossible
                debug_assert!(right_end == end);

                // right_end == end, left_end != end
                let value = left_value.clone();
                self.right = None;
                self.left = Some((left_range, left_value));
                value
            } else if right_end == end {
                // right_end == end, left_end == end
                self.left = None;
                self.right = None;
                left_value
            } else {
                // right_end != end, left_end == end
                self.right = Some(RangeInclusive::new(right_start, right_end));
                self.left = None;
                left_value
            };

            let range_value = (start..=end, value);
            return Some(range_value);
        }
    }

    // TODO: Implement size_hint -- this is similar code from the set version.
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
