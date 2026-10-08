use alloc::{collections::BinaryHeap, vec::Vec};
use core::{cmp::Reverse, iter::FusedIterator, ops::RangeInclusive};

use crate::{
    Integer, KMergeMap, Owned, SortedDisjointMap, map::ValueCarrier, sorted_disjoint_map::Priority,
};

/// The sweep shared by the multiway joins.
///
/// It walks the key line once, splitting it into maximal "stretches" over which the set of active
/// inputs (and their values) is constant. Starts come from a [`KMergeMap`] (a heap over the k
/// inputs, ordered by start, tagged with input position); ends come from a min-heap of the active
/// ranges. Each input range costs O(log k) to activate and O(log k) to deactivate, and the
/// per-input `slots` are updated in place, so a stretch never rebuilds an O(k) slice. This matches
/// the cost of the existing multiway union.
#[derive(Clone, Debug)]
struct JoinSweep<T, VC, I>
where
    T: Integer,
    VC: ValueCarrier,
    I: SortedDisjointMap<T, VC>,
{
    merged: KMergeMap<T, VC, I>,
    // The next range from `merged`, already pulled so its start can be inspected.
    upcoming: Option<Priority<T, VC>>,
    // Ends of the active ranges, with their input positions; smallest end on top.
    active_ends: BinaryHeap<Reverse<(T, usize)>>,
    // One slot per input: its value over the current stretch, or `None` if it is inactive there.
    slots: Vec<Option<VC>>,
    active_count: usize,
    // The end of the stretch most recently returned. Its ranges are deactivated at the start of the
    // next call, after the caller has read `slots`.
    last_end: Option<T>,
    // Positions of inputs activated since the consumer last drained this. Used by the inner join
    // to keep its dense buffer current; past `slots.len()` entries, `changed_overflow` is set
    // instead, and the consumer rebuilds (amortized O(1) per change either way).
    track_changes: bool,
    changed: Vec<usize>,
    changed_overflow: bool,
}

impl<T, VC, I> JoinSweep<T, VC, I>
where
    T: Integer,
    VC: ValueCarrier,
    I: SortedDisjointMap<T, VC>,
{
    fn new<K>(inputs: K, track_changes: bool) -> Self
    where
        K: IntoIterator<Item = I>,
    {
        let inputs: Vec<I> = inputs.into_iter().collect();
        let input_count = inputs.len();
        Self {
            merged: KMergeMap::new(inputs),
            upcoming: None,
            active_ends: BinaryHeap::with_capacity(input_count),
            slots: (0..input_count).map(|_| None).collect(),
            active_count: 0,
            last_end: None,
            track_changes,
            changed: Vec::new(),
            changed_overflow: false,
        }
    }

    fn upcoming_start(&mut self) -> Option<T> {
        if self.upcoming.is_none() {
            self.upcoming = self.merged.next();
        }
        self.upcoming.as_ref().map(Priority::start)
    }

    fn activate(&mut self, item: Priority<T, VC>) {
        let index = item.priority_number();
        let (range, value) = item.into_range_value();
        debug_assert!(
            self.slots[index].is_none(),
            "an input's ranges are disjoint"
        );
        self.slots[index] = Some(value);
        self.active_ends.push(Reverse((*range.end(), index)));
        self.active_count += 1;
        if self.track_changes {
            if self.changed.len() < self.slots.len() {
                self.changed.push(index);
            } else {
                self.changed_overflow = true;
            }
        }
    }

    /// Advances to the next stretch and returns its range; `slots` then describes it.
    fn next_stretch(&mut self) -> Option<RangeInclusive<T>> {
        // Close out the previous stretch: deactivate every range that ended with it.
        let mut start = None;
        if let Some(last_end) = self.last_end.take() {
            while let Some(&Reverse((end, index))) = self.active_ends.peek() {
                if end != last_end {
                    break;
                }
                self.active_ends.pop();
                self.slots[index] = None;
                self.active_count -= 1;
            }
            // Every range ends at or before the maximum key, so nothing can follow it.
            start = Some(last_end.checked_add_one()?);
        }

        // With nothing active, jump to the next range's start.
        let start = if self.active_count == 0 {
            self.upcoming_start()?
        } else {
            let Some(start) = start else {
                unreachable!("inputs are only active after a stretch has been returned")
            };
            start
        };

        // Activate every range that starts here. Ranges never start before `start`: each stretch
        // ends just before the next upcoming start.
        while self.upcoming_start() == Some(start) {
            if let Some(item) = self.upcoming.take() {
                self.activate(item);
            }
        }

        // The stretch runs to the earliest active end, or to just before the next start.
        let Some(&Reverse((earliest_end, _))) = self.active_ends.peek() else {
            unreachable!("at least one range starts at `start`")
        };
        let end = match self.upcoming_start() {
            Some(next_start) if next_start <= earliest_end => next_start.sub_one(),
            _ => earliest_end,
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
            sweep: JoinSweep::new(inputs, false),
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
            sweep: JoinSweep::new(inputs, true),
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
