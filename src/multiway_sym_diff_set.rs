use core::{iter::FusedIterator, ops::RangeInclusive};

use crate::{
    Integer, MultiwaySweep, SortedDisjoint, SortedDisjointMap, SortedStartsMap, SweepEvent,
};

/// A set's ranges as a map stream with `true` as every value, so sets can use [`MultiwaySweep`].
#[derive(Clone, Debug)]
struct SetAsMap<I>(I);

impl<T, I> Iterator for SetAsMap<I>
where
    T: Integer,
    I: Iterator<Item = RangeInclusive<T>>,
{
    type Item = (RangeInclusive<T>, bool);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(|range| (range, true))
    }
}

impl<T, I> FusedIterator for SetAsMap<I>
where
    T: Integer,
    I: Iterator<Item = RangeInclusive<T>> + FusedIterator,
{
}

impl<T: Integer, I: SortedDisjoint<T>> SortedStartsMap<T, bool> for SetAsMap<I> {}
impl<T: Integer, I: SortedDisjoint<T>> SortedDisjointMap<T, bool> for SetAsMap<I> {}

/// This `struct` is created by the [`symmetric_difference`] method on [`MultiwaySortedDisjoint`].
/// See [`symmetric_difference`] for details.
///
/// Its fields are private, so its implementation can change without changing its type.
///
/// [`MultiwaySortedDisjoint`]: crate::MultiwaySortedDisjoint
/// [`symmetric_difference`]: crate::MultiwaySortedDisjoint::symmetric_difference
#[must_use = "iterators are lazy and do nothing unless consumed"]
#[derive(Clone, Debug)]
pub struct MultiwaySymmetricDifference<T, I>
where
    T: Integer,
    I: SortedDisjoint<T>,
{
    // Keys covered by an odd number of inputs, found with one sweep: O(log k) per input range.
    sweep: MultiwaySweep<T, bool, SetAsMap<I>>,
    active_count: usize,
    // Start of the current stretch, while at least one input is active.
    stretch_start: Option<T>,
    // The output range being extended, not yet returned.
    pending: Option<RangeInclusive<T>>,
}

impl<T, I> MultiwaySymmetricDifference<T, I>
where
    T: Integer,
    I: SortedDisjoint<T>,
{
    pub(crate) fn new_k<K>(inputs: K) -> Self
    where
        K: IntoIterator<Item = I>,
    {
        Self {
            sweep: MultiwaySweep::new(inputs.into_iter().map(SetAsMap)),
            active_count: 0,
            stretch_start: None,
            pending: None,
        }
    }

    // Records the stretch `start..=end` if an odd number of inputs cover it; returns the previous
    // output range if this one does not extend it.
    fn emit(&mut self, start: T, end: T) -> Option<RangeInclusive<T>> {
        if self.active_count.is_multiple_of(2) {
            return None;
        }
        if let Some(pending) = &mut self.pending {
            // Stretches are disjoint and increasing, so `pending_end < start` and `add_one` is
            // safe.
            if pending.end().add_one() == start {
                *pending = *pending.start()..=end;
                return None;
            }
        }
        self.pending.replace(start..=end)
    }
}

impl<T, I> Iterator for MultiwaySymmetricDifference<T, I>
where
    T: Integer,
    I: SortedDisjoint<T>,
{
    type Item = RangeInclusive<T>;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(event) = self.sweep.next() {
            let finished = match event {
                SweepEvent::Start { range, .. } => {
                    let start = *range.start();
                    let finished = match self.stretch_start {
                        Some(stretch) if stretch < start => self.emit(stretch, start.sub_one()),
                        _ => None,
                    };
                    self.active_count += 1;
                    self.stretch_start = Some(start);
                    finished
                }
                SweepEvent::End { at, .. } => {
                    let finished = match self.stretch_start {
                        Some(stretch) if stretch <= at => self.emit(stretch, at),
                        _ => None,
                    };
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

impl<T, I> FusedIterator for MultiwaySymmetricDifference<T, I>
where
    T: Integer,
    I: SortedDisjoint<T>,
{
}
