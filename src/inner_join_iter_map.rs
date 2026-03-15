use core::{
    cmp::{max, min},
    iter::FusedIterator,
    ops::RangeInclusive,
};

use crate::Integer;
use crate::{SortedDisjointMap, map::ValueRef};

/// This `struct` is created by the [`inner_join`] method on [`SortedDisjointMap`].
/// It yields the common disjoint overlap and both values for each overlapping range.
///
/// [`SortedDisjointMap`]: crate::SortedDisjointMap
/// [`inner_join`]: crate::SortedDisjointMap::inner_join
// todo000 need more tests
// todo000 need docs updated
// todo000 consider adding to RMS
#[must_use = "iterators are lazy and do nothing unless consumed"]
#[derive(Clone, Debug)]
pub struct InnerJoinIterMap<T, VRL, VRR, IL, IR> {
    iter_left: IL,
    iter_right: IR,
    right: Option<(RangeInclusive<T>, VRR)>,
    left: Option<(RangeInclusive<T>, VRL)>,
}

impl<T, VRL, VRR, IL, IR> InnerJoinIterMap<T, VRL, VRR, IL, IR>
where
    T: Integer,
    VRL: ValueRef,
    VRR: ValueRef,
    IL: SortedDisjointMap<T, VRL>,
    IR: SortedDisjointMap<T, VRR>,
{
    pub(crate) const fn new(iter_left: IL, iter_right: IR) -> Self {
        Self {
            iter_left,
            iter_right,
            right: None,
            left: None,
        }
    }
}

impl<T, VRL, VRR, IL, IR> FusedIterator for InnerJoinIterMap<T, VRL, VRR, IL, IR>
where
    T: Integer,
    VRL: ValueRef,
    VRR: ValueRef,
    IL: SortedDisjointMap<T, VRL> + FusedIterator,
    IR: SortedDisjointMap<T, VRR> + FusedIterator,
{
}

impl<T, VRL, VRR, IL, IR> Iterator for InnerJoinIterMap<T, VRL, VRR, IL, IR>
where
    T: Integer,
    VRL: ValueRef,
    VRR: ValueRef,
    IL: SortedDisjointMap<T, VRL>,
    IR: SortedDisjointMap<T, VRR>,
{
    type Item = (RangeInclusive<T>, (VRL, VRR));

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            self.left = self.left.take().or_else(|| self.iter_left.next());
            self.right = self.right.take().or_else(|| self.iter_right.next());

            let (Some((left_range, left_value)), Some((right_range, right_value))) =
                (self.left.take(), self.right.take())
            else {
                return None;
            };

            let (left_start, left_end) = left_range.clone().into_inner();
            let (right_start, right_end) = right_range.clone().into_inner();

            if left_end < right_start {
                self.left = None;
                self.right = Some((right_range, right_value));
                continue;
            }

            if right_end < left_start {
                self.left = Some((left_range, left_value));
                self.right = None;
                continue;
            }

            let overlap_start = max(left_start, right_start);
            let overlap_end = min(left_end, right_end);

            if left_end == overlap_end && right_end == overlap_end {
                return Some((overlap_start..=overlap_end, (left_value, right_value)));
            }

            if left_end == overlap_end {
                self.right = Some((overlap_end.add_one()..=right_end, right_value.clone()));
                return Some((overlap_start..=overlap_end, (left_value, right_value)));
            }

            self.left = Some((overlap_end.add_one()..=left_end, left_value.clone()));
            return Some((overlap_start..=overlap_end, (left_value, right_value)));
        }
    }
}
