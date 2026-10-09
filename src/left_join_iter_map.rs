use core::{cmp::min, iter::FusedIterator, ops::RangeInclusive};

use crate::Integer;
use crate::{SortedDisjointMap, map::ValueCarrier};

/// This `struct` is created by the [`left_join`] method on [`SortedDisjointMap`].
///
/// It yields every disjoint range covered by the left input, with the left value and the right
/// input's value there, or `None`.
///
/// [`SortedDisjointMap`]: crate::SortedDisjointMap
/// [`left_join`]: crate::SortedDisjointMap::left_join
#[must_use = "iterators are lazy and do nothing unless consumed"]
#[derive(Clone, Debug)]
pub struct LeftJoinIterMap<T, VCL, VCR, IL, IR> {
    iter_left: IL,
    iter_right: IR,
    left: Option<(RangeInclusive<T>, VCL)>,
    right: Option<(RangeInclusive<T>, VCR)>,
}

impl<T, VCL, VCR, IL, IR> LeftJoinIterMap<T, VCL, VCR, IL, IR>
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

impl<T, VCL, VCR, IL, IR> FusedIterator for LeftJoinIterMap<T, VCL, VCR, IL, IR>
where
    T: Integer,
    VCL: ValueCarrier,
    VCR: ValueCarrier,
    IL: SortedDisjointMap<T, VCL> + FusedIterator,
    IR: SortedDisjointMap<T, VCR> + FusedIterator,
{
}

// The output needs no coalescing, for the same reason as the full join: every boundary between
// two touching output ranges is a range boundary of one input, where that input's value changes.
//
// Once the left input is exhausted, the iterator stops without reading the rest of the right
// input; this is what makes a left join cheaper than filtering a full join.
impl<T, VCL, VCR, IL, IR> Iterator for LeftJoinIterMap<T, VCL, VCR, IL, IR>
where
    T: Integer,
    VCL: ValueCarrier,
    VCR: ValueCarrier,
    IL: SortedDisjointMap<T, VCL>,
    IR: SortedDisjointMap<T, VCR>,
{
    type Item = (RangeInclusive<T>, (VCL, Option<VCR>));

    fn next(&mut self) -> Option<Self::Item> {
        let (left_range, left_value) = self.left.take().or_else(|| self.iter_left.next())?;
        let (left_start, left_end) = left_range.into_inner();

        // Skip right ranges that end before this left range starts.
        let mut right = self.right.take().or_else(|| self.iter_right.next());
        while let Some((right_range, _)) = &right
            && *right_range.end() < left_start
        {
            right = self.iter_right.next();
        }

        let Some((right_range, right_value)) = right else {
            // No more right ranges: the whole left range has no right value.
            return Some((left_start..=left_end, (left_value, None)));
        };
        let (right_start, right_end) = right_range.into_inner();

        if left_start < right_start {
            // A left-only prefix, up to the right range's start or the left range's end.
            let end = min(left_end, right_start.sub_one());
            if end < left_end {
                self.left = Some((end.add_one()..=left_end, left_value.clone()));
            }
            self.right = Some((right_start..=right_end, right_value));
            return Some((left_start..=end, (left_value, None)));
        }

        // The right range covers `left_start`: emit the overlap and keep any remainders.
        let end = min(left_end, right_end);
        if end < left_end {
            self.left = Some((end.add_one()..=left_end, left_value.clone()));
        }
        if end < right_end {
            self.right = Some((end.add_one()..=right_end, right_value.clone()));
        }
        Some((left_start..=end, (left_value, Some(right_value))))
    }
}

#[cfg(test)]
mod tests {
    use alloc::{vec, vec::Vec};
    use core::cell::Cell;
    use core::ops::RangeInclusive;

    use crate::prelude::*;

    type Joined = (RangeInclusive<u8>, (&'static str, Option<&'static str>));

    fn left(
        left: &[(RangeInclusive<u8>, &'static str)],
        right: &[(RangeInclusive<u8>, &'static str)],
    ) -> Vec<Joined> {
        let left: RangeMapBlaze<_, _> = left.iter().cloned().collect();
        let right: RangeMapBlaze<_, _> = right.iter().cloned().collect();
        let actual: Vec<Joined> = left
            .range_values()
            .left_join(right.range_values())
            .map(|(range, (l, r))| (range, (*l, r.copied())))
            .collect();

        // Oracle: the full join, keeping only ranges where the left input is present.
        let expected: Vec<Joined> = left
            .range_values()
            .full_join(right.range_values())
            .filter_map(|(range, (l, r))| Some((range, (*l?, r.copied()))))
            .collect();
        assert_eq!(actual, expected);
        actual
    }

    #[test]
    fn empty_and_one_sided() {
        assert!(left(&[], &[]).is_empty());
        assert!(left(&[], &[(1..=2, "R")]).is_empty());
        assert_eq!(left(&[(1..=2, "L")], &[]), vec![(1..=2, ("L", None))]);
    }

    #[test]
    fn partial_overlap_and_containment() {
        assert_eq!(
            left(&[(1..=5, "a")], &[(4..=8, "b")]),
            vec![(1..=3, ("a", None)), (4..=5, ("a", Some("b")))]
        );
        assert_eq!(
            left(&[(1..=10, "L")], &[(3..=4, "R")]),
            vec![
                (1..=2, ("L", None)),
                (3..=4, ("L", Some("R"))),
                (5..=10, ("L", None)),
            ]
        );
        assert_eq!(
            left(&[(3..=4, "L")], &[(1..=10, "R")]),
            vec![(3..=4, ("L", Some("R")))]
        );
    }

    #[test]
    fn one_range_spans_several_and_maximum_key() {
        assert_eq!(
            left(
                &[(0..=10, "L")],
                &[(1..=2, "a"), (4..=5, "b"), (9..=20, "c")]
            ),
            vec![
                (0..=0, ("L", None)),
                (1..=2, ("L", Some("a"))),
                (3..=3, ("L", None)),
                (4..=5, ("L", Some("b"))),
                (6..=8, ("L", None)),
                (9..=10, ("L", Some("c"))),
            ]
        );
        assert_eq!(
            left(&[(250..=255, "L")], &[(0..=0, "lo"), (255..=255, "hi")]),
            vec![(250..=254, ("L", None)), (255..=255, ("L", Some("hi")))]
        );
    }

    #[test]
    fn stops_when_left_is_exhausted() {
        // A small left near the start of a large right: the right input must not be read to the end.
        let small = RangeMapBlaze::from_iter([(0..=1u32, "s")]);
        let large: RangeMapBlaze<u32, &str> =
            (0..10_000u32).map(|i| (i * 2..=i * 2, "x")).collect();
        let read = Cell::new(0usize);
        let right =
            CheckSortedDisjointMap::new(large.range_values().inspect(|_| read.set(read.get() + 1)));
        let joined: Vec<_> = small.range_values().left_join(right).collect();
        assert_eq!(joined.len(), 2);
        assert!(read.get() <= 3, "read {} right ranges", read.get());
    }

    #[test]
    fn exhaustive_small_inputs_match_oracle() {
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
                    let l: Vec<_> = left_pattern
                        .iter()
                        .enumerate()
                        .map(|(index, range)| (range.clone(), values[(index + left_offset) % 2]))
                        .collect();
                    let r: Vec<_> = right_pattern
                        .iter()
                        .enumerate()
                        .map(|(index, range)| (range.clone(), values[(index + right_offset) % 2]))
                        .collect();
                    left(&l, &r);
                }
            }
        }
    }
}
