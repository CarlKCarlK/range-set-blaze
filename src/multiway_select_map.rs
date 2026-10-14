use alloc::{boxed::Box, collections::BinaryHeap};
use core::{iter::FusedIterator, ops::RangeInclusive};

use crate::{
    Integer, MultiwaySweep, SortedDisjointMap, SortedStartsMap, SweepEvent, map::ValueCarrier,
};

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
    values: Box<[Option<VC>]>,
    // Max-heap of active input positions, deleted lazily (an entry whose value is `None` is stale).
    // `in_heap` keeps each position in the heap at most once, so it holds at most k entries.
    highest: BinaryHeap<usize>,
    in_heap: Box<[bool]>,
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
            values: (0..input_count).map(|_| None).collect(),
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
            if let Some(value) = &self.values[top] {
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
                    self.values[input] = Some(value);
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
                    self.values[input] = None;
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
pub struct MultiwayUnionMap<T, VC, I>
where
    T: Integer,
    VC: ValueCarrier,
    I: SortedDisjointMap<T, VC>,
{
    inner: SweepSelectMap<T, VC, I>,
}

impl<T, VC, I> MultiwayUnionMap<T, VC, I>
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

impl<T, VC, I> Iterator for MultiwayUnionMap<T, VC, I>
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

impl<T, VC, I> FusedIterator for MultiwayUnionMap<T, VC, I>
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
pub struct MultiwaySymmetricDifferenceMap<T, VC, I>
where
    T: Integer,
    VC: ValueCarrier,
    I: SortedDisjointMap<T, VC>,
{
    inner: SweepSelectMap<T, VC, I>,
}

impl<T, VC, I> MultiwaySymmetricDifferenceMap<T, VC, I>
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

impl<T, VC, I> Iterator for MultiwaySymmetricDifferenceMap<T, VC, I>
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

impl<T, VC, I> FusedIterator for MultiwaySymmetricDifferenceMap<T, VC, I>
where
    T: Integer,
    VC: ValueCarrier,
    I: SortedDisjointMap<T, VC> + FusedIterator,
{
}

/// Two map-stream types as one, so a two-input operation can use the k-way code, which takes
/// inputs of a single type.
#[derive(Clone, Debug)]
enum EitherMap<L, R> {
    Left(L),
    Right(R),
}

impl<T, VC, L, R> Iterator for EitherMap<L, R>
where
    T: Integer,
    VC: ValueCarrier,
    L: Iterator<Item = (RangeInclusive<T>, VC)>,
    R: Iterator<Item = (RangeInclusive<T>, VC)>,
{
    type Item = (RangeInclusive<T>, VC);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::Left(left) => left.next(),
            Self::Right(right) => right.next(),
        }
    }
}

impl<L: FusedIterator, R: FusedIterator> FusedIterator for EitherMap<L, R> where Self: Iterator {}

impl<T, VC, L, R> SortedStartsMap<T, VC> for EitherMap<L, R>
where
    T: Integer,
    VC: ValueCarrier,
    L: SortedDisjointMap<T, VC>,
    R: SortedDisjointMap<T, VC>,
{
}

impl<T, VC, L, R> SortedDisjointMap<T, VC> for EitherMap<L, R>
where
    T: Integer,
    VC: ValueCarrier,
    L: SortedDisjointMap<T, VC>,
    R: SortedDisjointMap<T, VC>,
{
}

/// This `struct` is created by the [`union`] method on [`SortedDisjointMap`] (and the `|`
/// operator). See [`union`] for details.
///
/// Its fields are private, so its implementation can change without changing its type.
///
/// [`SortedDisjointMap`]: crate::SortedDisjointMap
/// [`union`]: crate::SortedDisjointMap::union
#[must_use = "iterators are lazy and do nothing unless consumed"]
#[derive(Clone, Debug)]
pub struct UnionMap<T, VC, L, R>
where
    T: Integer,
    VC: ValueCarrier,
    L: SortedDisjointMap<T, VC>,
    R: SortedDisjointMap<T, VC>,
{
    // The k-way union over the two inputs; the right input has priority.
    inner: MultiwayUnionMap<T, VC, EitherMap<L, R>>,
}

impl<T, VC, L, R> UnionMap<T, VC, L, R>
where
    T: Integer,
    VC: ValueCarrier,
    L: SortedDisjointMap<T, VC>,
    R: SortedDisjointMap<T, VC>,
{
    pub(crate) fn new2(left: L, right: R) -> Self {
        Self {
            inner: MultiwayUnionMap::new_k([EitherMap::Left(left), EitherMap::Right(right)]),
        }
    }
}

impl<T, VC, L, R> Iterator for UnionMap<T, VC, L, R>
where
    T: Integer,
    VC: ValueCarrier,
    L: SortedDisjointMap<T, VC>,
    R: SortedDisjointMap<T, VC>,
{
    type Item = (RangeInclusive<T>, VC);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
    }
}

impl<T, VC, L, R> FusedIterator for UnionMap<T, VC, L, R>
where
    T: Integer,
    VC: ValueCarrier,
    L: SortedDisjointMap<T, VC>,
    R: SortedDisjointMap<T, VC>,
{
}

/// This `struct` is created by the [`symmetric_difference`] method on [`SortedDisjointMap`] (and
/// the `^` operator). See [`symmetric_difference`] for details.
///
/// Its fields are private, so its implementation can change without changing its type.
///
/// [`SortedDisjointMap`]: crate::SortedDisjointMap
/// [`symmetric_difference`]: crate::SortedDisjointMap::symmetric_difference
#[must_use = "iterators are lazy and do nothing unless consumed"]
#[derive(Clone, Debug)]
pub struct SymmetricDifferenceMap<T, VC, L, R>
where
    T: Integer,
    VC: ValueCarrier,
    L: SortedDisjointMap<T, VC>,
    R: SortedDisjointMap<T, VC>,
{
    // The k-way symmetric difference over the two inputs; the right input has priority.
    inner: MultiwaySymmetricDifferenceMap<T, VC, EitherMap<L, R>>,
}

impl<T, VC, L, R> SymmetricDifferenceMap<T, VC, L, R>
where
    T: Integer,
    VC: ValueCarrier,
    L: SortedDisjointMap<T, VC>,
    R: SortedDisjointMap<T, VC>,
{
    pub(crate) fn new2(left: L, right: R) -> Self {
        Self {
            inner: MultiwaySymmetricDifferenceMap::new_k([
                EitherMap::Left(left),
                EitherMap::Right(right),
            ]),
        }
    }
}

impl<T, VC, L, R> Iterator for SymmetricDifferenceMap<T, VC, L, R>
where
    T: Integer,
    VC: ValueCarrier,
    L: SortedDisjointMap<T, VC>,
    R: SortedDisjointMap<T, VC>,
{
    type Item = (RangeInclusive<T>, VC);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
    }
}

impl<T, VC, L, R> FusedIterator for SymmetricDifferenceMap<T, VC, L, R>
where
    T: Integer,
    VC: ValueCarrier,
    L: SortedDisjointMap<T, VC>,
    R: SortedDisjointMap<T, VC>,
{
}
