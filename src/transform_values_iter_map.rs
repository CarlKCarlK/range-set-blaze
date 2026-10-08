use core::{iter::FusedIterator, marker::PhantomData, ops::RangeInclusive};

use crate::Integer;
use crate::{Owned, SortedDisjointMap, map::ValueCarrier};

/// This `struct` is created by the [`transform_values`] method on [`SortedDisjointMap`].
///
/// It yields the same ranges with each value replaced by the closure's result, merging touching
/// ranges whose new values are equal.
///
/// [`SortedDisjointMap`]: crate::SortedDisjointMap
/// [`transform_values`]: crate::SortedDisjointMap::transform_values
#[must_use = "iterators are lazy and do nothing unless consumed"]
#[derive(Clone, Debug)]
pub struct TransformValuesIterMap<T, VC, I, F, W> {
    iter: I,
    f: F,
    pending: Option<(RangeInclusive<T>, W)>,
    // `VC` appears only in the bounds on `I` and `F`; it must be a type parameter so those
    // bounds can name it.
    phantom: PhantomData<fn(VC) -> W>,
}

impl<T, VC, I, F, W> TransformValuesIterMap<T, VC, I, F, W>
where
    T: Integer,
    VC: ValueCarrier,
    I: SortedDisjointMap<T, VC>,
    F: FnMut(VC) -> W,
    W: Eq + Clone,
{
    pub(crate) const fn new(iter: I, f: F) -> Self {
        Self {
            iter,
            f,
            pending: None,
            phantom: PhantomData,
        }
    }
}

impl<T, VC, I, F, W> FusedIterator for TransformValuesIterMap<T, VC, I, F, W>
where
    T: Integer,
    VC: ValueCarrier,
    I: SortedDisjointMap<T, VC> + FusedIterator,
    F: FnMut(VC) -> W,
    W: Eq + Clone,
{
}

impl<T, VC, I, F, W> Iterator for TransformValuesIterMap<T, VC, I, F, W>
where
    T: Integer,
    VC: ValueCarrier,
    I: SortedDisjointMap<T, VC>,
    F: FnMut(VC) -> W,
    W: Eq + Clone,
{
    type Item = (RangeInclusive<T>, Owned<W>);

    fn next(&mut self) -> Option<Self::Item> {
        for (range, carrier) in self.iter.by_ref() {
            let value = (self.f)(carrier);
            let (start, end) = range.into_inner();
            if let Some((pending_range, pending_value)) = &mut self.pending {
                // Disjoint input means `pending_end < start`, so `add_one` cannot overflow.
                let pending_end = *pending_range.end();
                if pending_end.add_one() == start && *pending_value == value {
                    *pending_range = *pending_range.start()..=end;
                    continue;
                }
            }
            if let Some((pending_range, pending_value)) = self.pending.replace((start..=end, value))
            {
                return Some((pending_range, Owned(pending_value)));
            }
        }
        self.pending
            .take()
            .map(|(range, value)| (range, Owned(value)))
    }
}

#[cfg(test)]
mod tests {
    use alloc::{string::ToString, vec, vec::Vec};

    use crate::{Owned, prelude::*};

    #[test]
    fn merges_touching_equal_results() {
        let map = RangeMapBlaze::from_iter([(1..=3u8, 10), (4..=6, 11), (8..=9, 20)]);
        let tens: Vec<_> = map
            .range_values()
            .transform_values(|value| value / 10)
            .collect();
        assert_eq!(tens, vec![(1..=6, Owned(1)), (8..=9, Owned(2))]);
    }

    #[test]
    fn empty_and_maximum_key() {
        let empty = RangeMapBlaze::<u8, u8>::new();
        assert_eq!(empty.range_values().transform_values(|v| *v).next(), None);

        let full = RangeMapBlaze::from_iter([(0..=127u8, 'a'), (128..=255, 'b')]);
        let merged: Vec<_> = full.range_values().transform_values(|_| ()).collect();
        assert_eq!(merged, vec![(0..=255, Owned(()))]);
    }

    #[test]
    fn calls_once_per_range_in_order() {
        let map = RangeMapBlaze::from_iter([(10..=19u8, 'b'), (0..=9, 'a'), (30..=39, 'c')]);
        let mut seen = Vec::new();
        let numbered: RangeMapBlaze<u8, usize> = map
            .range_values()
            .transform_values(|value| {
                seen.push(*value);
                seen.len()
            })
            .into_range_map_blaze();
        assert_eq!(seen, vec!['a', 'b', 'c']);
        assert_eq!(
            numbered.to_string(),
            "(0..=9, 1), (10..=19, 2), (30..=39, 3)"
        );
    }

    #[test]
    fn output_chains_as_sorted_disjoint_map() {
        // The result is itself a `SortedDisjointMap`, so it composes with other stream operations.
        let left = RangeMapBlaze::from_iter([(1..=5u8, "a"), (6..=8, "bb")]);
        let right = RangeMapBlaze::from_iter([(4..=9u8, 'x')]);
        let joined: Vec<_> = left
            .range_values()
            .transform_values(|value| value.len())
            .outer_join(right.range_values())
            .map(|(range, (len, ch))| (range, (len.map(|Owned(n)| n), ch.copied())))
            .collect();
        assert_eq!(
            joined,
            vec![
                (1..=3, (Some(1), None)),
                (4..=5, (Some(1), Some('x'))),
                (6..=8, (Some(2), Some('x'))),
                (9..=9, (None, Some('x'))),
            ]
        );
    }

    #[test]
    fn matches_struct_transform_values() {
        let map =
            RangeMapBlaze::from_iter([(0..=9u8, 3), (10..=19, 4), (25..=30, 5), (31..=40, 7)]);
        let streamed: RangeMapBlaze<u8, u8> = map
            .range_values()
            .transform_values(|value| value % 2)
            .into_range_map_blaze();
        assert_eq!(streamed, map.transform_values(|value| value % 2));
    }
}
