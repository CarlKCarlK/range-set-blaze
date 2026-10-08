use alloc::{collections::BinaryHeap, vec::Vec};
use core::{cmp::Reverse, iter::FusedIterator, ops::RangeInclusive};

use crate::{
    Integer, KMergeMap, SortedDisjointMap, map::ValueCarrier, sorted_disjoint_map::Priority,
};

/// An event from a [`MultiwaySweep`]: one input's range starting or ending.
///
/// See [`MultiwaySortedDisjointMap::sweep`] for details.
///
/// [`MultiwaySortedDisjointMap::sweep`]: crate::MultiwaySortedDisjointMap::sweep
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SweepEvent<T, VC> {
    /// Input `input`'s range `range` starts at `range.start()`, carrying `value` (moved, not
    /// cloned).
    Start {
        /// The starting range.
        range: RangeInclusive<T>,
        /// The input's position.
        input: usize,
        /// The range's value carrier.
        value: VC,
    },
    /// Input `input`'s range ends after key `at` (inclusive end).
    End {
        /// The range's last key.
        at: T,
        /// The input's position.
        input: usize,
    },
}

/// This `struct` is created by the [`sweep`] method on [`MultiwaySortedDisjointMap`].
///
/// It yields the start and end of every input range, in key order, as [`SweepEvent`]s. See
/// [`sweep`] for details.
///
/// [`MultiwaySortedDisjointMap`]: crate::MultiwaySortedDisjointMap
/// [`sweep`]: crate::MultiwaySortedDisjointMap::sweep
#[must_use = "iterators are lazy and do nothing unless consumed"]
#[derive(Clone, Debug)]
pub struct MultiwaySweep<T, VC, I>
where
    T: Integer,
    VC: ValueCarrier,
    I: SortedDisjointMap<T, VC>,
{
    // Starts: a heap over the inputs, ordered by start, tagged with input position.
    merged: KMergeMap<T, VC, I>,
    // The next start from `merged`, already pulled so it can be compared with the next end.
    upcoming: Option<Priority<T, VC>>,
    // Ends of the started-but-not-ended ranges, with input positions; smallest end on top.
    active_ends: BinaryHeap<Reverse<(T, usize)>>,
    input_count: usize,
}

impl<T, VC, I> MultiwaySweep<T, VC, I>
where
    T: Integer,
    VC: ValueCarrier,
    I: SortedDisjointMap<T, VC>,
{
    pub(crate) fn new<K>(inputs: K) -> Self
    where
        K: IntoIterator<Item = I>,
    {
        let inputs: Vec<I> = inputs.into_iter().collect();
        let input_count = inputs.len();
        Self {
            merged: KMergeMap::new(inputs),
            upcoming: None,
            active_ends: BinaryHeap::with_capacity(input_count),
            input_count,
        }
    }

    /// The number of inputs being swept.
    #[must_use]
    pub const fn input_count(&self) -> usize {
        self.input_count
    }
}

impl<T, VC, I> FusedIterator for MultiwaySweep<T, VC, I>
where
    T: Integer,
    VC: ValueCarrier,
    I: SortedDisjointMap<T, VC> + FusedIterator,
{
}

impl<T, VC, I> Iterator for MultiwaySweep<T, VC, I>
where
    T: Integer,
    VC: ValueCarrier,
    I: SortedDisjointMap<T, VC>,
{
    type Item = SweepEvent<T, VC>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.upcoming.is_none() {
            self.upcoming = self.merged.next();
        }
        let next_start = self.upcoming.as_ref().map(Priority::start);
        let earliest_end = self.active_ends.peek().map(|&Reverse((end, _))| end);
        // A range ending at `end` closes before a range starting at `start` exactly when
        // `end < start`; otherwise the start comes first (they overlap or touch at `start`).
        let end_first = match (earliest_end, next_start) {
            (Some(end), Some(start)) => end < start,
            (Some(_), None) => true,
            (None, _) => false,
        };
        if end_first {
            let Reverse((at, input)) = self.active_ends.pop()?;
            return Some(SweepEvent::End { at, input });
        }
        let item = self.upcoming.take()?;
        let input = item.priority_number();
        let (range, value) = item.into_range_value();
        self.active_ends.push(Reverse((*range.end(), input)));
        Some(SweepEvent::Start {
            range,
            input,
            value,
        })
    }
}
