use alloc::{collections::BinaryHeap, vec::Vec};
use core::{iter::FusedIterator, ops::RangeInclusive};

use crate::{Integer, MultiwaySweep, SortedDisjointMap, SweepEvent, map::ValueCarrier};

/// Which ranges a [`SweepSelectMap`] keeps, by how many inputs are present there.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Mode {
    /// At least one input.
    Union,
    /// An odd number of inputs.
    SymmetricDifference,
}

impl Mode {
    const fn keeps(self, active_count: usize) -> bool {
        match self {
            Self::Union => active_count > 0,
            Self::SymmetricDifference => active_count % 2 == 1,
        }
    }
}

/// The multiway map union and symmetric difference, on [`MultiwaySweep`].
///
/// Over each stretch of keys where the set of present inputs is constant, the stretch is kept if
/// [`Mode::keeps`] the number present, with the value of the highest-numbered input present
/// (inputs to the right have priority). Touching kept stretches with equal values are merged.
/// O(log k) per input range: the sweep, plus a max-heap of active input positions.
#[derive(Clone, Debug)]
struct SweepSelectMap<T, VC, I>
where
    T: Integer,
    VC: ValueCarrier,
    I: SortedDisjointMap<T, VC>,
{
    sweep: MultiwaySweep<T, VC, I>,
    mode: Mode,
    // Each input's value while its range is active.
    slots: Vec<Option<VC>>,
    // Max-heap of active input positions, deleted lazily (an entry whose slot is `None` is stale).
    // `in_heap` keeps each position in the heap at most once, so it holds at most k entries.
    highest: BinaryHeap<usize>,
    in_heap: Vec<bool>,
    active_count: usize,
    // Start of the current stretch, while at least one input is active.
    stretch_start: Option<T>,
    // The output range being extended, not yet returned.
    pending: Option<(RangeInclusive<T>, VC)>,
}

impl<T, VC, I> SweepSelectMap<T, VC, I>
where
    T: Integer,
    VC: ValueCarrier,
    I: SortedDisjointMap<T, VC>,
{
    fn new<K>(inputs: K, mode: Mode) -> Self
    where
        K: IntoIterator<Item = I>,
    {
        let sweep = MultiwaySweep::new(inputs);
        let input_count = sweep.input_count();
        Self {
            sweep,
            mode,
            slots: (0..input_count).map(|_| None).collect(),
            highest: BinaryHeap::with_capacity(input_count),
            in_heap: (0..input_count).map(|_| false).collect(),
            active_count: 0,
            stretch_start: None,
            pending: None,
        }
    }

    // The value of the highest-numbered active input; at least one input must be active.
    fn highest_value(&mut self) -> Option<VC> {
        while let Some(&top) = self.highest.peek() {
            if let Some(value) = &self.slots[top] {
                return Some(value.clone());
            }
            self.highest.pop();
            self.in_heap[top] = false;
        }
        None
    }

    // Records the stretch `start..=end` if kept; returns the previous output range if this one does
    // not extend it.
    fn emit(&mut self, start: T, end: T) -> Option<(RangeInclusive<T>, VC)> {
        if !self.mode.keeps(self.active_count) {
            return None;
        }
        let value = self.highest_value()?;
        if let Some((pending_range, pending_value)) = &mut self.pending {
            // Stretches are disjoint and increasing, so `pending_end < start` and `add_one` is
            // safe.
            if pending_range.end().add_one() == start && pending_value.value_eq(&value) {
                *pending_range = *pending_range.start()..=end;
                return None;
            }
        }
        self.pending.replace((start..=end, value))
    }
}

impl<T, VC, I> Iterator for SweepSelectMap<T, VC, I>
where
    T: Integer,
    VC: ValueCarrier,
    I: SortedDisjointMap<T, VC>,
{
    type Item = (RangeInclusive<T>, VC);

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(event) = self.sweep.next() {
            let finished = match event {
                SweepEvent::Start {
                    range,
                    input,
                    value,
                } => {
                    let start = *range.start();
                    let finished = match self.stretch_start {
                        Some(stretch) if stretch < start => self.emit(stretch, start.sub_one()),
                        _ => None,
                    };
                    self.slots[input] = Some(value);
                    if !self.in_heap[input] {
                        self.in_heap[input] = true;
                        self.highest.push(input);
                    }
                    self.active_count += 1;
                    self.stretch_start = Some(start);
                    finished
                }
                SweepEvent::End { at, input } => {
                    let finished = match self.stretch_start {
                        Some(stretch) if stretch <= at => self.emit(stretch, at),
                        _ => None,
                    };
                    self.slots[input] = None;
                    self.active_count -= 1;
                    self.stretch_start = if self.active_count == 0 {
                        None
                    } else {
                        at.checked_add_one()
                    };
                    finished
                }
            };
            if finished.is_some() {
                return finished;
            }
        }
        self.pending.take()
    }
}

/// This `struct` is created by the [`union`] method on [`MultiwaySortedDisjointMap`]. See
/// [`union`] for details.
///
/// Its fields are private, so its implementation can change without changing its type.
///
/// [`MultiwaySortedDisjointMap`]: crate::MultiwaySortedDisjointMap
/// [`union`]: crate::MultiwaySortedDisjointMap::union
#[must_use = "iterators are lazy and do nothing unless consumed"]
#[derive(Clone, Debug)]
pub struct UnionKMergeMap<T, VC, I>
where
    T: Integer,
    VC: ValueCarrier,
    I: SortedDisjointMap<T, VC>,
{
    inner: SweepSelectMap<T, VC, I>,
}

impl<T, VC, I> UnionKMergeMap<T, VC, I>
where
    T: Integer,
    VC: ValueCarrier,
    I: SortedDisjointMap<T, VC>,
{
    pub(crate) fn new_k<K>(inputs: K) -> Self
    where
        K: IntoIterator<Item = I>,
    {
        Self {
            inner: SweepSelectMap::new(inputs, Mode::Union),
        }
    }
}

impl<T, VC, I> Iterator for UnionKMergeMap<T, VC, I>
where
    T: Integer,
    VC: ValueCarrier,
    I: SortedDisjointMap<T, VC>,
{
    type Item = (RangeInclusive<T>, VC);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
    }
}

impl<T, VC, I> FusedIterator for UnionKMergeMap<T, VC, I>
where
    T: Integer,
    VC: ValueCarrier,
    I: SortedDisjointMap<T, VC> + FusedIterator,
{
}

/// This `struct` is created by the [`symmetric_difference`] method on
/// [`MultiwaySortedDisjointMap`]. See [`symmetric_difference`] for details.
///
/// Its fields are private, so its implementation can change without changing its type.
///
/// [`MultiwaySortedDisjointMap`]: crate::MultiwaySortedDisjointMap
/// [`symmetric_difference`]: crate::MultiwaySortedDisjointMap::symmetric_difference
#[must_use = "iterators are lazy and do nothing unless consumed"]
#[derive(Clone, Debug)]
pub struct SymDiffKMergeMap<T, VC, I>
where
    T: Integer,
    VC: ValueCarrier,
    I: SortedDisjointMap<T, VC>,
{
    inner: SweepSelectMap<T, VC, I>,
}

impl<T, VC, I> SymDiffKMergeMap<T, VC, I>
where
    T: Integer,
    VC: ValueCarrier,
    I: SortedDisjointMap<T, VC>,
{
    pub(crate) fn new_k<K>(inputs: K) -> Self
    where
        K: IntoIterator<Item = I>,
    {
        Self {
            inner: SweepSelectMap::new(inputs, Mode::SymmetricDifference),
        }
    }
}

impl<T, VC, I> Iterator for SymDiffKMergeMap<T, VC, I>
where
    T: Integer,
    VC: ValueCarrier,
    I: SortedDisjointMap<T, VC>,
{
    type Item = (RangeInclusive<T>, VC);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
    }
}

impl<T, VC, I> FusedIterator for SymDiffKMergeMap<T, VC, I>
where
    T: Integer,
    VC: ValueCarrier,
    I: SortedDisjointMap<T, VC> + FusedIterator,
{
}
