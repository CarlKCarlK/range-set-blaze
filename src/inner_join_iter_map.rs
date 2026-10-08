use core::{
    cmp::{max, min},
    iter::FusedIterator,
    ops::RangeInclusive,
};

use crate::Integer;
use crate::{SortedDisjointMap, map::ValueCarrier};

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
pub struct InnerJoinIterMap<T, VCL, VCR, IL, IR> {
    iter_left: IL,
    iter_right: IR,
    right: Option<(RangeInclusive<T>, VCR)>,
    left: Option<(RangeInclusive<T>, VCL)>,
}

impl<T, VCL, VCR, IL, IR> InnerJoinIterMap<T, VCL, VCR, IL, IR>
where
    T: Integer,
    VCL: ValueCarrier,
    VCR: ValueCarrier,
    IL: SortedDisjointMap<T, VCL>,
    IR: SortedDisjointMap<T, VCR>,
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

impl<T, VCL, VCR, IL, IR> FusedIterator for InnerJoinIterMap<T, VCL, VCR, IL, IR>
where
    T: Integer,
    VCL: ValueCarrier,
    VCR: ValueCarrier,
    IL: SortedDisjointMap<T, VCL> + FusedIterator,
    IR: SortedDisjointMap<T, VCR> + FusedIterator,
{
}

impl<T, VCL, VCR, IL, IR> Iterator for InnerJoinIterMap<T, VCL, VCR, IL, IR>
where
    T: Integer,
    VCL: ValueCarrier,
    VCR: ValueCarrier,
    IL: SortedDisjointMap<T, VCL>,
    IR: SortedDisjointMap<T, VCR>,
{
    type Item = (RangeInclusive<T>, (VCL, VCR));

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

#[cfg(test)]
mod tests {
    use alloc::{vec, vec::Vec};

    use crate::prelude::*;

    fn join(
        left: &[(core::ops::RangeInclusive<u8>, &'static str)],
        right: &[(core::ops::RangeInclusive<u8>, &'static str)],
    ) -> Vec<(core::ops::RangeInclusive<u8>, (&'static str, &'static str))> {
        let left = RangeMapBlaze::from_iter(left.iter().cloned());
        let right = RangeMapBlaze::from_iter(right.iter().cloned());
        left.range_values()
            .inner_join(right.range_values())
            .map(|(range, (l, r))| (range, (*l, *r)))
            .collect()
    }

    #[test]
    fn empty_inputs() {
        assert!(join(&[], &[]).is_empty());
        assert!(join(&[(1..=2, "L")], &[]).is_empty());
        assert!(join(&[], &[(1..=2, "R")]).is_empty());
    }

    #[test]
    fn disjoint_and_touching_do_not_overlap() {
        assert!(join(&[(1..=2, "L")], &[(5..=6, "R")]).is_empty());
        assert!(join(&[(5..=6, "L")], &[(1..=2, "R")]).is_empty());
        assert!(join(&[(1..=2, "L")], &[(3..=4, "R")]).is_empty());
    }

    #[test]
    fn single_point_overlap() {
        assert_eq!(
            join(&[(2..=3, "L")], &[(1..=2, "R")]),
            vec![(2..=2, ("L", "R"))]
        );
        assert_eq!(
            join(&[(1..=2, "L")], &[(2..=3, "R")]),
            vec![(2..=2, ("L", "R"))]
        );
    }

    #[test]
    fn equal_ends_and_containment() {
        assert_eq!(
            join(&[(1..=5, "L")], &[(3..=5, "R")]),
            vec![(3..=5, ("L", "R"))]
        );
        assert_eq!(
            join(&[(1..=10, "L")], &[(3..=4, "R")]),
            vec![(3..=4, ("L", "R"))]
        );
        assert_eq!(
            join(&[(3..=4, "L")], &[(1..=10, "R")]),
            vec![(3..=4, ("L", "R"))]
        );
    }

    #[test]
    fn one_range_spans_several() {
        assert_eq!(
            join(
                &[(0..=10, "L")],
                &[(1..=2, "a"), (4..=5, "b"), (9..=20, "c")]
            ),
            vec![
                (1..=2, ("L", "a")),
                (4..=5, ("L", "b")),
                (9..=10, ("L", "c")),
            ]
        );
        assert_eq!(
            join(
                &[(1..=2, "a"), (4..=5, "b"), (9..=20, "c")],
                &[(0..=10, "R")]
            ),
            vec![
                (1..=2, ("a", "R")),
                (4..=5, ("b", "R")),
                (9..=10, ("c", "R")),
            ]
        );
    }

    #[test]
    fn minimum_and_maximum_keys() {
        assert_eq!(
            join(&[(0..=255, "L")], &[(0..=0, "lo"), (255..=255, "hi")]),
            vec![(0..=0, ("L", "lo")), (255..=255, ("L", "hi"))]
        );
        assert_eq!(
            join(&[(0..=255, "L")], &[(0..=255, "R")]),
            vec![(0..=255, ("L", "R"))]
        );
    }

    #[test]
    fn composes_with_difference() {
        let left = RangeMapBlaze::from_iter([(0..=9u8, "L")]);
        let right = RangeMapBlaze::from_iter([(3..=5u8, "R")]);
        let overlap = left.range_values().inner_join(right.range_values());
        let hole = RangeSetBlaze::from_iter([4..=4u8]);
        let rest = overlap.map_and_set_difference(hole.ranges());
        assert_eq!(
            rest.map(|(range, _)| range).collect::<Vec<_>>(),
            vec![3..=3, 5..=5]
        );
    }

    #[test]
    fn std_iterator_adapters_remain_sorted_disjoint() {
        // Guards the released `std::iter::*` trait impls: they must not be removed.
        let set = RangeSetBlaze::from_iter([1..=2, 5..=6, 9..=9]);
        let skipped = set.ranges().skip(1);
        assert_eq!(skipped.into_string(), "5..=6, 9..=9");
    }
}
