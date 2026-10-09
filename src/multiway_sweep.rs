use alloc::{
    boxed::Box,
    collections::{BinaryHeap, binary_heap::PeekMut},
};
use core::{cmp::Reverse, iter::FusedIterator, ops::RangeInclusive};

use crate::{Integer, SortedDisjointMap, map::ValueCarrier};

/// An event from a [`MultiwaySweep`]: one input's range starting or ending.
///
/// See [`MultiwaySortedDisjointMap::sweep`] for details.
///
/// See the [joins guide][crate::joins] for how the joins fit together.
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

// An input's next range and value, not yet started; `None` once the input is exhausted.
type NextRange<T, VC> = Option<(RangeInclusive<T>, VC)>;

/// This `struct` is created by the [`sweep`] method on [`MultiwaySortedDisjointMap`].
///
/// It yields the start and end of every input range, in key order, as [`SweepEvent`]s. See
/// [`sweep`] for details.
///
/// See the [joins guide][crate::joins] for how the joins fit together.
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
    inputs: Box<[I]>,
    // Each input's next range (not yet started), pulled ahead so its start can be in the heap.
    heads: Box<[NextRange<T, VC>]>,
    // Starts of the inputs' next ranges, with input positions; smallest (start, input) on top. The
    // heap holds only small keys; the iterators and ranges stay in place in `inputs` and `heads`.
    starts: BinaryHeap<Reverse<(T, usize)>>,
    // Ends of the started-but-not-ended ranges, with input positions; smallest end on top.
    active_ends: BinaryHeap<Reverse<(T, usize)>>,
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
        let mut inputs: Box<[I]> = inputs.into_iter().collect();
        let input_count = inputs.len();
        let mut starts = BinaryHeap::with_capacity(input_count);
        let heads = inputs
            .iter_mut()
            .enumerate()
            .map(|(input, iter)| {
                let head = iter.next();
                if let Some((range, _)) = &head {
                    starts.push(Reverse((*range.start(), input)));
                }
                head
            })
            .collect();
        Self {
            inputs,
            heads,
            starts,
            active_ends: BinaryHeap::with_capacity(input_count),
        }
    }

    /// The number of inputs being swept.
    #[must_use]
    pub const fn input_count(&self) -> usize {
        self.inputs.len()
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
        let next_start = self.starts.peek().map(|&Reverse((start, _))| start);
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
        let mut top = self.starts.peek_mut()?;
        let Reverse((_, input)) = *top;
        let (range, value) = self.heads[input].take()?;
        // Pull this input's next range. Its start replaces this one at the top of the heap in
        // place (one sift instead of a pop and a push).
        let next = self.inputs[input].next();
        if let Some((next_range, _)) = &next {
            *top = Reverse((*next_range.start(), input));
        } else {
            PeekMut::pop(top);
        }
        self.heads[input] = next;
        self.active_ends.push(Reverse((*range.end(), input)));
        Some(SweepEvent::Start {
            range,
            input,
            value,
        })
    }
}
