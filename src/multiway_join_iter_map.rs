use alloc::vec::Vec;
use core::{iter::FusedIterator, ops::RangeInclusive};

use crate::{Integer, MultiwaySweep, Owned, SortedDisjointMap, SweepEvent, map::ValueCarrier};

/// The stretch walker shared by the multiway joins.
///
/// It consumes a [`MultiwaySweep`] (each input range's start and end, in key order, O(log k) per
/// range) and splits the key line into maximal "stretches" over which the set of active inputs
/// (and their values) is constant. The per-input `slots` are updated in place as events arrive,
/// so a stretch never rebuilds an O(k) slice.
#[derive(Clone, Debug)]
struct JoinSweep<T, VC, I>
where
    T: Integer,
    VC: ValueCarrier,
    I: SortedDisjointMap<T, VC>,
{
    events: MultiwaySweep<T, VC, I>,
    // The next event, already pulled so it can be inspected.
    upcoming: Option<SweepEvent<T, VC>>,
    // One slot per input: its value over the current stretch, or `None` if it is inactive there.
    slots: Vec<Option<VC>>,
    active_count: usize,
    // The end of the stretch most recently returned. Its ranges are deactivated at the start of the
    // next call, after the caller has read `slots`.
    last_end: Option<T>,
    tracking: Tracking,
    // `Tracking::Activated`: positions of inputs activated since the consumer last drained this.
    // Used by the inner join to keep its dense buffer current; past `slots.len()` entries,
    // `changed_overflow` is set instead, and the consumer rebuilds (amortized O(1) per change
    // either way).
    changed: Vec<usize>,
    changed_overflow: bool,
}

/// What the sweep records about slot changes, for the consumer.
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
            events,
            upcoming: None,
            slots: (0..input_count).map(|_| None).collect(),
            active_count: 0,
            last_end: None,
            tracking,
            changed: Vec::new(),
            changed_overflow: false,
        }
    }

    fn peek(&mut self) -> Option<&SweepEvent<T, VC>> {
        if self.upcoming.is_none() {
            self.upcoming = self.events.next();
        }
        self.upcoming.as_ref()
    }

    fn deactivate(&mut self, index: usize) {
        self.slots[index] = None;
        self.active_count -= 1;
    }

    fn activate(&mut self, index: usize, value: VC) {
        debug_assert!(
            self.slots[index].is_none(),
            "an input's ranges are disjoint"
        );
        self.slots[index] = Some(value);
        self.active_count += 1;
        match self.tracking {
            Tracking::None => {}
            Tracking::Activated => {
                if self.changed.len() < self.slots.len() {
                    self.changed.push(index);
                } else {
                    self.changed_overflow = true;
                }
            }
        }
    }

    /// Advances to the next stretch and returns its range; `slots` then describes it.
    fn next_stretch(&mut self) -> Option<RangeInclusive<T>> {
        // Close out the previous stretch: deactivate every range that ended with it. Those ends are
        // the next events, since the stretch ended at the earliest end or just before a start.
        let mut start = None;
        if let Some(last_end) = self.last_end.take() {
            while let Some(SweepEvent::End { at, .. }) = self.peek()
                && *at == last_end
            {
                if let Some(SweepEvent::End { input, .. }) = self.upcoming.take() {
                    self.deactivate(input);
                }
            }
            // Every range ends at or before the maximum key, so nothing can follow it.
            start = Some(last_end.checked_add_one()?);
        }

        // With nothing active, jump to the next range's start.
        let start = if self.active_count == 0 {
            match self.peek()? {
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
        while let Some(SweepEvent::Start { range, .. }) = self.peek()
            && *range.start() == start
        {
            if let Some(SweepEvent::Start { input, value, .. }) = self.upcoming.take() {
                self.activate(input, value);
            }
        }

        // The stretch runs to the next event: an end (inclusive), or just before a start.
        let end = match self.peek() {
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

/// This `struct` is created by the [`outer_join`] method on [`MultiwaySortedDisjointMap`].
///
/// It yields every range covered by at least one input, with the closure's result for that
/// range's per-input values. See [`outer_join`] for details.
///
/// [`MultiwaySortedDisjointMap`]: crate::MultiwaySortedDisjointMap
/// [`outer_join`]: crate::MultiwaySortedDisjointMap::outer_join
#[must_use = "iterators are lazy and do nothing unless consumed"]
#[derive(Clone, Debug)]
pub struct MultiwayOuterJoinIterMap<T, VC, I, F, W>
where
    T: Integer,
    VC: ValueCarrier,
    I: SortedDisjointMap<T, VC>,
{
    sweep: JoinSweep<T, VC, I>,
    f: F,
    pending: Option<(RangeInclusive<T>, W)>,
}

impl<T, VC, I, F, W> MultiwayOuterJoinIterMap<T, VC, I, F, W>
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

impl<T, VC, I, F, W> FusedIterator for MultiwayOuterJoinIterMap<T, VC, I, F, W>
where
    T: Integer,
    VC: ValueCarrier,
    I: SortedDisjointMap<T, VC> + FusedIterator,
    F: FnMut(&[Option<VC>]) -> W,
    W: Eq + Clone,
{
}

impl<T, VC, I, F, W> Iterator for MultiwayOuterJoinIterMap<T, VC, I, F, W>
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
            let value = (self.f)(&self.sweep.slots);
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
/// [`MultiwaySortedDisjointMap`]: crate::MultiwaySortedDisjointMap
/// [`inner_join`]: crate::MultiwaySortedDisjointMap::inner_join
#[must_use = "iterators are lazy and do nothing unless consumed"]
#[derive(Clone, Debug)]
pub struct MultiwayInnerJoinIterMap<T, VC, I, F, W>
where
    T: Integer,
    VC: ValueCarrier,
    I: SortedDisjointMap<T, VC>,
{
    sweep: JoinSweep<T, VC, I>,
    f: F,
    pending: Option<(RangeInclusive<T>, W)>,
    // Every input's value, valid whenever all inputs are active. Kept current by copying only the
    // slots that changed (see `JoinSweep::changed`).
    dense: Vec<VC>,
    // With zero inputs, "all inputs present" holds everywhere: the result is the universal range.
    zero_inputs_done: bool,
}

impl<T, VC, I, F, W> MultiwayInnerJoinIterMap<T, VC, I, F, W>
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

    // Brings `dense` up to date with `sweep.slots`; called only when every input is active.
    fn sync_dense(&mut self) {
        let slots = &self.sweep.slots;
        if self.dense.len() != slots.len() || self.sweep.changed_overflow {
            self.dense = slots.iter().flatten().cloned().collect();
            debug_assert_eq!(self.dense.len(), slots.len(), "every input is active");
        } else {
            for &index in &self.sweep.changed {
                if let Some(value) = &slots[index] {
                    self.dense[index] = value.clone();
                }
            }
        }
        self.sweep.changed.clear();
        self.sweep.changed_overflow = false;
    }
}

impl<T, VC, I, F, W> FusedIterator for MultiwayInnerJoinIterMap<T, VC, I, F, W>
where
    T: Integer,
    VC: ValueCarrier,
    I: SortedDisjointMap<T, VC> + FusedIterator,
    F: FnMut(&[VC]) -> W,
    W: Eq + Clone,
{
}

impl<T, VC, I, F, W> Iterator for MultiwayInnerJoinIterMap<T, VC, I, F, W>
where
    T: Integer,
    VC: ValueCarrier,
    I: SortedDisjointMap<T, VC>,
    F: FnMut(&[VC]) -> W,
    W: Eq + Clone,
{
    type Item = (RangeInclusive<T>, Owned<W>);

    fn next(&mut self) -> Option<Self::Item> {
        let input_count = self.sweep.slots.len();
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
