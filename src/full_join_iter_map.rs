use core::{cmp::min, iter::FusedIterator, ops::RangeInclusive};

use crate::Integer;
use crate::{SortedDisjointMap, map::ValueCarrier};

/// This `struct` is created by the [`full_join`] method on [`SortedDisjointMap`].
/// It yields every disjoint range covered by at least one input, with each input's value or `None`.
///
/// [`SortedDisjointMap`]: crate::SortedDisjointMap
/// [`full_join`]: crate::SortedDisjointMap::full_join
#[must_use = "iterators are lazy and do nothing unless consumed"]
#[derive(Clone, Debug)]
pub struct FullJoinIterMap<T, VCL, VCR, IL, IR> {
    iter_left: IL,
    iter_right: IR,
    left: Option<(RangeInclusive<T>, VCL)>,
    right: Option<(RangeInclusive<T>, VCR)>,
}

impl<T, VCL, VCR, IL, IR> FullJoinIterMap<T, VCL, VCR, IL, IR>
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
            left: None,
            right: None,
        }
    }
}

impl<T, VCL, VCR, IL, IR> FusedIterator for FullJoinIterMap<T, VCL, VCR, IL, IR>
where
    T: Integer,
    VCL: ValueCarrier,
    VCR: ValueCarrier,
    IL: SortedDisjointMap<T, VCL> + FusedIterator,
    IR: SortedDisjointMap<T, VCR> + FusedIterator,
{
}

// The output needs no coalescing. Every boundary between two touching output ranges is a range
// boundary of one input, and that input's value changes there: either it appears or disappears, or
// it moves to a touching range, which the `SortedDisjointMap` invariant requires to hold a
// different value. So touching outputs always differ in at least one component.
impl<T, VCL, VCR, IL, IR> Iterator for FullJoinIterMap<T, VCL, VCR, IL, IR>
where
    T: Integer,
    VCL: ValueCarrier,
    VCR: ValueCarrier,
    IL: SortedDisjointMap<T, VCL>,
    IR: SortedDisjointMap<T, VCR>,
{
    type Item = (RangeInclusive<T>, (Option<VCL>, Option<VCR>));

    fn next(&mut self) -> Option<Self::Item> {
        self.left = self.left.take().or_else(|| self.iter_left.next());
        self.right = self.right.take().or_else(|| self.iter_right.next());

        match (self.left.take(), self.right.take()) {
            (None, None) => None,
            (Some((left_range, left_value)), None) => Some((left_range, (Some(left_value), None))),
            (None, Some((right_range, right_value))) => {
                Some((right_range, (None, Some(right_value))))
            }
            (Some((left_range, left_value)), Some((right_range, right_value))) => {
                let (left_start, left_end) = left_range.into_inner();
                let (right_start, right_end) = right_range.into_inner();

                if left_start < right_start {
                    // Left-only prefix, up to the right range's start or the left range's end.
                    let end = min(left_end, right_start.sub_one());
                    if end < left_end {
                        self.left = Some((end.add_one()..=left_end, left_value.clone()));
                    }
                    self.right = Some((right_start..=right_end, right_value));
                    return Some((left_start..=end, (Some(left_value), None)));
                }

                if right_start < left_start {
                    // Right-only prefix, up to the left range's start or the right range's end.
                    let end = min(right_end, left_start.sub_one());
                    if end < right_end {
                        self.right = Some((end.add_one()..=right_end, right_value.clone()));
                    }
                    self.left = Some((left_start..=left_end, left_value));
                    return Some((right_start..=end, (None, Some(right_value))));
                }

                // Both start together: emit the overlap and keep any remainder.
                let end = min(left_end, right_end);
                if end < left_end {
                    self.left = Some((end.add_one()..=left_end, left_value.clone()));
                }
                if end < right_end {
                    self.right = Some((end.add_one()..=right_end, right_value.clone()));
                }
                Some((left_start..=end, (Some(left_value), Some(right_value))))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use alloc::{vec, vec::Vec};
    use core::ops::RangeInclusive;

    use crate::prelude::*;

    type Joined = (
        RangeInclusive<u8>,
        (Option<&'static str>, Option<&'static str>),
    );

    fn outer(
        left: &[(RangeInclusive<u8>, &'static str)],
        right: &[(RangeInclusive<u8>, &'static str)],
    ) -> Vec<Joined> {
        let left: RangeMapBlaze<_, _> = left.iter().cloned().collect();
        let right: RangeMapBlaze<_, _> = right.iter().cloned().collect();
        let actual: Vec<Joined> = left
            .range_values()
            .full_join(right.range_values())
            .map(|(range, (l, r))| (range, (l.copied(), r.copied())))
            .collect();

        // Oracle: make both sides total with `fill_gaps`, inner-join, and drop `(None, None)`.
        let expected: Vec<Joined> = left
            .range_values()
            .fill_gaps()
            .inner_join(right.range_values().fill_gaps())
            .filter(|(_, (l, r))| l.is_some() || r.is_some())
            .map(|(range, (l, r))| (range, (l.copied(), r.copied())))
            .collect();
        assert_eq!(actual, expected);
        actual
    }

    #[test]
    fn empty_inputs() {
        assert!(outer(&[], &[]).is_empty());
        assert_eq!(
            outer(&[(1..=2, "L")], &[]),
            vec![(1..=2, (Some("L"), None))]
        );
        assert_eq!(
            outer(&[], &[(1..=2, "R")]),
            vec![(1..=2, (None, Some("R")))]
        );
    }

    #[test]
    fn disjoint_and_touching() {
        assert_eq!(
            outer(&[(1..=2, "L")], &[(5..=6, "R")]),
            vec![(1..=2, (Some("L"), None)), (5..=6, (None, Some("R")))]
        );
        assert_eq!(
            outer(&[(5..=6, "L")], &[(1..=2, "R")]),
            vec![(1..=2, (None, Some("R"))), (5..=6, (Some("L"), None))]
        );
        assert_eq!(
            outer(&[(1..=2, "L")], &[(3..=4, "R")]),
            vec![(1..=2, (Some("L"), None)), (3..=4, (None, Some("R")))]
        );
    }

    #[test]
    fn partial_overlap() {
        assert_eq!(
            outer(&[(1..=5, "a")], &[(4..=8, "b")]),
            vec![
                (1..=3, (Some("a"), None)),
                (4..=5, (Some("a"), Some("b"))),
                (6..=8, (None, Some("b"))),
            ]
        );
    }

    #[test]
    fn containment_and_equal_ends() {
        assert_eq!(
            outer(&[(1..=10, "L")], &[(3..=4, "R")]),
            vec![
                (1..=2, (Some("L"), None)),
                (3..=4, (Some("L"), Some("R"))),
                (5..=10, (Some("L"), None)),
            ]
        );
        assert_eq!(
            outer(&[(3..=5, "L")], &[(1..=5, "R")]),
            vec![(1..=2, (None, Some("R"))), (3..=5, (Some("L"), Some("R")))]
        );
        assert_eq!(
            outer(&[(1..=5, "L")], &[(1..=5, "R")]),
            vec![(1..=5, (Some("L"), Some("R")))]
        );
    }

    #[test]
    fn one_range_spans_several() {
        assert_eq!(
            outer(
                &[(0..=10, "L")],
                &[(1..=2, "a"), (4..=5, "b"), (9..=20, "c")]
            ),
            vec![
                (0..=0, (Some("L"), None)),
                (1..=2, (Some("L"), Some("a"))),
                (3..=3, (Some("L"), None)),
                (4..=5, (Some("L"), Some("b"))),
                (6..=8, (Some("L"), None)),
                (9..=10, (Some("L"), Some("c"))),
                (11..=20, (None, Some("c"))),
            ]
        );
    }

    #[test]
    fn touching_input_ranges_with_different_values() {
        assert_eq!(
            outer(&[(1..=3, "a"), (4..=6, "b")], &[(3..=4, "x")]),
            vec![
                (1..=2, (Some("a"), None)),
                (3..=3, (Some("a"), Some("x"))),
                (4..=4, (Some("b"), Some("x"))),
                (5..=6, (Some("b"), None)),
            ]
        );
    }

    #[test]
    fn minimum_and_maximum_keys() {
        assert_eq!(
            outer(&[(0..=0, "lo")], &[(255..=255, "hi")]),
            vec![(0..=0, (Some("lo"), None)), (255..=255, (None, Some("hi")))]
        );
        assert_eq!(
            outer(&[(0..=255, "L")], &[(0..=0, "lo"), (255..=255, "hi")]),
            vec![
                (0..=0, (Some("L"), Some("lo"))),
                (1..=254, (Some("L"), None)),
                (255..=255, (Some("L"), Some("hi"))),
            ]
        );
    }

    #[test]
    fn universal_inputs_match_inner_join() {
        let left = RangeMapBlaze::from_iter([(0..=99u8, "a"), (100..=255, "b")]);
        let right = RangeMapBlaze::from_iter([(0..=49u8, "x"), (50..=255, "y")]);
        let inner: Vec<_> = left
            .range_values()
            .inner_join(right.range_values())
            .map(|(range, (l, r))| (range, (Some(l), Some(r))))
            .collect();
        let outer: Vec<_> = left
            .range_values()
            .full_join(right.range_values())
            .collect();
        assert_eq!(inner, outer);
    }

    #[test]
    fn output_is_sorted_disjoint_map() {
        let left = RangeMapBlaze::from_iter([(1..=5u8, "a")]);
        let right = RangeMapBlaze::from_iter([(4..=8u8, "b")]);
        let joined: RangeMapBlaze<u8, (Option<&str>, Option<&str>)> = left
            .range_values()
            .full_join(right.range_values())
            .into_range_map_blaze();
        assert_eq!(joined.ranges().into_string(), "1..=8");
    }

    #[test]
    fn exhaustive_small_inputs_match_oracle() {
        // Every pair of maps over keys 0..=5 built from a few range patterns and two values.
        let patterns: [&[RangeInclusive<u8>]; 6] = [
            &[],
            &[0..=5],
            &[0..=1, 2..=3],
            &[1..=1, 3..=4],
            &[0..=0, 5..=5],
            &[2..=4],
        ];
        let values = ["p", "q"];
        for left_pattern in patterns {
            for right_pattern in patterns {
                for (left_offset, right_offset) in [(0, 0), (0, 1), (1, 0), (1, 1)] {
                    let left: Vec<_> = left_pattern
                        .iter()
                        .enumerate()
                        .map(|(index, range)| (range.clone(), values[(index + left_offset) % 2]))
                        .collect();
                    let right: Vec<_> = right_pattern
                        .iter()
                        .enumerate()
                        .map(|(index, range)| (range.clone(), values[(index + right_offset) % 2]))
                        .collect();
                    outer(&left, &right);
                }
            }
        }
    }
}
