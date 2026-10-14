use alloc::{boxed::Box, vec::Vec};
use core::{
    iter::{FusedIterator, Peekable},
    ops::RangeInclusive,
};

use crate::{Integer, MultiwaySweep, Owned, SortedDisjointMap, SweepEvent, map::ValueCarrier};

/// The stretch walker shared by the multiway joins.
///
/// It consumes a [`MultiwaySweep`] (each input range's start and end, in key order, O(log k) per
/// range) and splits the key line into maximal "stretches" over which the set of active inputs
/// (and their values) is constant. The per-input `values` are updated in place as events arrive,
/// so a stretch never rebuilds an O(k) slice.
#[derive(Clone, Debug)]
struct JoinSweep<T, VC, I>
where
    T: Integer,
    VC: ValueCarrier,
    I: SortedDisjointMap<T, VC>,
{
    events: Peekable<MultiwaySweep<T, VC, I>>,
    // One value per input: its value over the current stretch, or `None` if it is inactive there.
    values: Box<[Option<VC>]>,
    active_count: usize,
    // The end of the stretch most recently returned. Its ranges are deactivated at the start of the
    // next call, after the caller has read `values`.
    last_end: Option<T>,
    tracking: Tracking,
    // `Tracking::Activated`: positions of inputs activated since the consumer last drained this.
    // Used by the inner join to keep its dense buffer current; past `values.len()` entries,
    // `changed_overflow` is set instead, and the consumer rebuilds (amortized O(1) per change
    // either way).
    changed: Vec<usize>,
    changed_overflow: bool,
}

/// What the sweep records about value changes, for the consumer.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Tracking {
    None,
    Activated,
}

impl<T, VC, I> JoinSweep<T, VC, I>
where
    T: Integer,
    VC: ValueCarrier,
    I: SortedDisjointMap<T, VC>,
{
    fn new<K>(inputs: K, tracking: Tracking) -> Self
    where
        K: IntoIterator<Item = I>,
    {
        let events = MultiwaySweep::new(inputs);
        let input_count = events.input_count();
        Self {
            events: events.peekable(),
            values: (0..input_count).map(|_| None).collect(),
            active_count: 0,
            last_end: None,
            tracking,
            changed: Vec::new(),
            changed_overflow: false,
        }
    }

    fn deactivate(&mut self, index: usize) {
        self.values[index] = None;
        self.active_count -= 1;
    }

    fn activate(&mut self, index: usize, value: VC) {
        debug_assert!(
            self.values[index].is_none(),
            "an input's ranges are disjoint"
        );
        self.values[index] = Some(value);
        self.active_count += 1;
        match self.tracking {
            Tracking::None => {}
            Tracking::Activated => {
                if self.changed.len() < self.values.len() {
                    self.changed.push(index);
                } else {
                    self.changed_overflow = true;
                }
            }
        }
    }

    /// Advances to the next stretch and returns its range; `values` then describes it.
    fn next_stretch(&mut self) -> Option<RangeInclusive<T>> {
        // Close out the previous stretch: deactivate every range that ended with it. Those ends are
        // the next events, since the stretch ended at the earliest end or just before a start.
        let mut start = None;
        if let Some(last_end) = self.last_end.take() {
            while let Some(SweepEvent::End { input, .. }) = self
                .events
                .next_if(|event| matches!(event, SweepEvent::End { at, .. } if *at == last_end))
            {
                self.deactivate(input);
            }
            // Every range ends at or before the maximum key, so nothing can follow it.
            start = Some(last_end.checked_add_one()?);
        }

        // With nothing active, jump to the next range's start.
        let start = if self.active_count == 0 {
            match self.events.peek()? {
                SweepEvent::Start { range, .. } => *range.start(),
                SweepEvent::End { .. } => unreachable!("an end event with no active range"),
            }
        } else {
            let Some(start) = start else {
                unreachable!("inputs are only active after a stretch has been returned")
            };
            start
        };

        // Activate every range that starts here.
        while let Some(SweepEvent::Start { input, value, .. }) = self.events.next_if(
            |event| matches!(event, SweepEvent::Start { range, .. } if *range.start() == start),
        ) {
            self.activate(input, value);
        }

        // The stretch runs to the next event: an end (inclusive), or just before a start.
        let end = match self.events.peek() {
            Some(SweepEvent::End { at, .. }) => *at,
            Some(SweepEvent::Start { range, .. }) => range.start().sub_one(),
            None => unreachable!("an active range has not ended"),
        };
        self.last_end = Some(end);
        Some(start..=end)
    }
}

/// Merges `value` into `pending` if it touches and is equal; otherwise replaces `pending` and
/// returns what it held.
fn push_merged<T: Integer, W: Eq>(
    pending: &mut Option<(RangeInclusive<T>, W)>,
    range: RangeInclusive<T>,
    value: W,
) -> Option<(RangeInclusive<T>, Owned<W>)> {
    if let Some((pending_range, pending_value)) = pending {
        // Stretches are disjoint and increasing, so `pending_end < start` and `add_one` is safe.
        if pending_range.end().add_one() == *range.start() && *pending_value == value {
            *pending_range = *pending_range.start()..=*range.end();
            return None;
        }
    }
    pending
        .replace((range, value))
        .map(|(range, value)| (range, Owned(value)))
}

/// This `struct` is created by the [`full_join`] method on [`MultiwaySortedDisjointMap`].
///
/// It yields every range covered by at least one input, with the closure's result for that
/// range's per-input values. See [`full_join`] for details.
///
/// See the [joins guide][crate::joins] for how the joins fit together.
///
/// [`MultiwaySortedDisjointMap`]: crate::MultiwaySortedDisjointMap
/// [`full_join`]: crate::MultiwaySortedDisjointMap::full_join
#[must_use = "iterators are lazy and do nothing unless consumed"]
#[derive(Clone, Debug)]
pub struct MultiwayFullJoinMap<T, VC, I, F, W>
where
    T: Integer,
    VC: ValueCarrier,
    I: SortedDisjointMap<T, VC>,
{
    sweep: JoinSweep<T, VC, I>,
    f: F,
    pending: Option<(RangeInclusive<T>, W)>,
}

impl<T, VC, I, F, W> MultiwayFullJoinMap<T, VC, I, F, W>
where
    T: Integer,
    VC: ValueCarrier,
    I: SortedDisjointMap<T, VC>,
    F: FnMut(&[Option<VC>]) -> W,
    W: Eq + Clone,
{
    pub(crate) fn new<K>(inputs: K, f: F) -> Self
    where
        K: IntoIterator<Item = I>,
    {
        Self {
            sweep: JoinSweep::new(inputs, Tracking::None),
            f,
            pending: None,
        }
    }
}

impl<T, VC, I, F, W> FusedIterator for MultiwayFullJoinMap<T, VC, I, F, W>
where
    T: Integer,
    VC: ValueCarrier,
    I: SortedDisjointMap<T, VC> + FusedIterator,
    F: FnMut(&[Option<VC>]) -> W,
    W: Eq + Clone,
{
}

impl<T, VC, I, F, W> Iterator for MultiwayFullJoinMap<T, VC, I, F, W>
where
    T: Integer,
    VC: ValueCarrier,
    I: SortedDisjointMap<T, VC>,
    F: FnMut(&[Option<VC>]) -> W,
    W: Eq + Clone,
{
    type Item = (RangeInclusive<T>, Owned<W>);

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(range) = self.sweep.next_stretch() {
            let value = (self.f)(&self.sweep.values);
            if let Some(done) = push_merged(&mut self.pending, range, value) {
                return Some(done);
            }
        }
        self.pending
            .take()
            .map(|(range, value)| (range, Owned(value)))
    }
}

/// This `struct` is created by the [`inner_join`] method on [`MultiwaySortedDisjointMap`].
///
/// It yields every range covered by all inputs, with the closure's result for that range's
/// values. See [`inner_join`] for details.
///
/// See the [joins guide][crate::joins] for how the joins fit together.
///
/// [`MultiwaySortedDisjointMap`]: crate::MultiwaySortedDisjointMap
/// [`inner_join`]: crate::MultiwaySortedDisjointMap::inner_join
#[must_use = "iterators are lazy and do nothing unless consumed"]
#[derive(Clone, Debug)]
pub struct MultiwayInnerJoinMap<T, VC, I, F, W>
where
    T: Integer,
    VC: ValueCarrier,
    I: SortedDisjointMap<T, VC>,
{
    sweep: JoinSweep<T, VC, I>,
    f: F,
    pending: Option<(RangeInclusive<T>, W)>,
    // Every input's value, valid whenever all inputs are active. Kept current by copying only the
    // values that changed (see `JoinSweep::changed`).
    dense: Vec<VC>,
    // With zero inputs, "all inputs present" holds everywhere: the result is the universal range.
    zero_inputs_done: bool,
}

impl<T, VC, I, F, W> MultiwayInnerJoinMap<T, VC, I, F, W>
where
    T: Integer,
    VC: ValueCarrier,
    I: SortedDisjointMap<T, VC>,
    F: FnMut(&[VC]) -> W,
    W: Eq + Clone,
{
    pub(crate) fn new<K>(inputs: K, f: F) -> Self
    where
        K: IntoIterator<Item = I>,
    {
        Self {
            sweep: JoinSweep::new(inputs, Tracking::Activated),
            f,
            pending: None,
            dense: Vec::new(),
            zero_inputs_done: false,
        }
    }

    // Brings `dense` up to date with `sweep.values`; called only when every input is active.
    fn sync_dense(&mut self) {
        let values = &self.sweep.values;
        if self.dense.len() != values.len() || self.sweep.changed_overflow {
            self.dense = values.iter().flatten().cloned().collect();
            debug_assert_eq!(self.dense.len(), values.len(), "every input is active");
        } else {
            for &index in &self.sweep.changed {
                if let Some(value) = &values[index] {
                    self.dense[index] = value.clone();
                }
            }
        }
        self.sweep.changed.clear();
        self.sweep.changed_overflow = false;
    }
}

impl<T, VC, I, F, W> FusedIterator for MultiwayInnerJoinMap<T, VC, I, F, W>
where
    T: Integer,
    VC: ValueCarrier,
    I: SortedDisjointMap<T, VC> + FusedIterator,
    F: FnMut(&[VC]) -> W,
    W: Eq + Clone,
{
}

impl<T, VC, I, F, W> Iterator for MultiwayInnerJoinMap<T, VC, I, F, W>
where
    T: Integer,
    VC: ValueCarrier,
    I: SortedDisjointMap<T, VC>,
    F: FnMut(&[VC]) -> W,
    W: Eq + Clone,
{
    type Item = (RangeInclusive<T>, Owned<W>);

    fn next(&mut self) -> Option<Self::Item> {
        let input_count = self.sweep.values.len();
        if input_count == 0 {
            if self.zero_inputs_done {
                return None;
            }
            self.zero_inputs_done = true;
            return Some((T::min_value()..=T::max_value(), Owned((self.f)(&[]))));
        }
        while let Some(range) = self.sweep.next_stretch() {
            if self.sweep.active_count < input_count {
                continue;
            }
            self.sync_dense();
            let value = (self.f)(&self.dense);
            if let Some(done) = push_merged(&mut self.pending, range, value) {
                return Some(done);
            }
        }
        self.pending
            .take()
            .map(|(range, value)| (range, Owned(value)))
    }
}
