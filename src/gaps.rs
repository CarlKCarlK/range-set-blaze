use core::{iter::FusedIterator, ops::RangeInclusive};

use crate::{Integer, SortedDisjoint, SortedDisjointMap, map::ValueCarrier};

/// A lazy iterator that fills the gaps in a sorted, disjoint set stream.
///
/// Present ranges are returned with `true`, and missing portions of the
/// integer domain are returned with `false`. The output covers the complete
/// domain from [`Integer::min_value`] through [`Integer::max_value`].
/// In other words, this is a total Boolean-valued map stream: `true` means
/// membership in the input set and `false` means a gap. The `false` values
/// are ordinary map values, so the resulting map's key domain is universal.
///
/// This iterator is created by [`RangeSetBlaze::fill_gaps`] and
/// [`SortedDisjoint::fill_gaps`].
///
/// The iterator implements [`SortedDisjointMap<T, bool>`], so it supports the
/// sorted-disjoint map operations and can be collected into a
/// [`RangeMapBlaze<T, bool>`].
///
/// Because `true` and `false` are ordinary map values, map operators act on the
/// stream's key ranges; for example, `!filled` is empty rather than Boolean
/// negation.
///
/// [`RangeSetBlaze::fill_gaps`]: crate::RangeSetBlaze::fill_gaps
/// [`RangeMapBlaze<T, bool>`]: crate::RangeMapBlaze
///
/// # Example
///
/// ```
/// use range_set_blaze::{
///     CheckSortedDisjoint, RangeMapBlaze, SortedDisjoint, SortedDisjointMap,
/// };
///
/// let input = CheckSortedDisjoint::new([1..=3, 7..=10]);
/// let output = input.fill_gaps().collect::<Vec<_>>();
/// assert_eq!(output[0], (i32::MIN..=0, false));
/// assert_eq!(output[1], (1..=3, true));
/// assert_eq!(output[2], (4..=6, false));
/// assert_eq!(output[3], (7..=10, true));
/// assert_eq!(output[4], (11..=i32::MAX, false));
///
/// let input = CheckSortedDisjoint::new([1_u8..=3, 7..=10]);
/// let map: RangeMapBlaze<u8, bool> = input.fill_gaps().into_range_map_blaze();
/// assert_eq!(map.get(2), Some(&true));
/// assert_eq!(map.get(5), Some(&false));
/// ```
#[derive(Clone, Debug)]
#[must_use = "iterators are lazy and do nothing unless consumed"]
pub struct FillGapsIter<T, I> {
    iter: I,
    next_start: T,
    pending: Option<RangeInclusive<T>>,
    done: bool,
}

impl<T, I> FillGapsIter<T, I>
where
    T: Integer,
    I: SortedDisjoint<T>,
{
    /// Creates a gap-filling iterator from a sorted, disjoint set stream.
    pub(crate) fn new(iter: I) -> Self {
        Self {
            iter,
            next_start: T::min_value(),
            pending: None,
            done: false,
        }
    }
}

impl<T, I> Iterator for FillGapsIter<T, I>
where
    T: Integer,
    I: SortedDisjoint<T>,
{
    type Item = (RangeInclusive<T>, bool);

    fn next(&mut self) -> Option<Self::Item> {
        if self.done {
            return None;
        }

        let Some(range) = self.pending.take().or_else(|| self.iter.next()) else {
            self.done = true;
            return Some((self.next_start..=T::max_value(), false));
        };

        let (start, end) = range.clone().into_inner();
        debug_assert!(start <= end);

        if self.next_start < start {
            let gap = self.next_start..=start.sub_one();
            self.next_start = start;
            self.pending = Some(range);
            return Some((gap, false));
        }

        if end == T::max_value() {
            self.done = true;
        } else {
            self.next_start = end.add_one();
        }
        Some((range, true))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        if self.done {
            return (0, Some(0));
        }

        let (low, high) = self.iter.size_hint();
        let pending = usize::from(self.pending.is_some());
        let low = low.saturating_add(pending);
        let high = high.and_then(|high| high.checked_mul(2)?.checked_add(pending)?.checked_add(1));
        (low, high)
    }
}

impl<T, I> FusedIterator for FillGapsIter<T, I>
where
    T: Integer,
    I: SortedDisjoint<T>,
{
}

/// A lazy iterator that fills the gaps in a sorted, disjoint map stream.
///
/// Mapped ranges are returned with `Some(value)`, and missing portions of the
/// integer domain are returned with `None`. The output covers the complete
/// domain from [`Integer::min_value`] through [`Integer::max_value`].
/// In other words, this is a total map stream from a
/// [`SortedDisjointMap<T, VC>`]: its logical output is `Option<VC::Value>`,
/// with `Some(value)` for mapped input ranges and `None` for gaps. The `None`
/// values are ordinary map values, so the resulting map's key domain is
/// universal.
///
/// This iterator is created by [`RangeMapBlaze::fill_gaps`] and
/// [`SortedDisjointMap::fill_gaps`].
///
/// [`RangeMapBlaze::fill_gaps`]: crate::RangeMapBlaze::fill_gaps
///
/// # Example
///
/// ```
/// use range_set_blaze::{CheckSortedDisjointMap, SortedDisjointMap};
///
/// let input = CheckSortedDisjointMap::new([(1..=3, &"red"), (7..=10, &"blue")]);
/// let output = input.fill_gaps().collect::<Vec<_>>();
/// assert_eq!(output[0], (0..=0, None));
/// assert_eq!(output[1], (1..=3, Some(&"red")));
/// assert_eq!(output[2], (4..=6, None));
/// assert_eq!(output[3], (7..=10, Some(&"blue")));
/// assert_eq!(output[4], (11..=u8::MAX, None));
/// ```
#[derive(Clone, Debug)]
#[must_use = "iterators are lazy and do nothing unless consumed"]
pub struct FillGapsIterMap<T, VC, I> {
    iter: I,
    next_start: T,
    pending: Option<(RangeInclusive<T>, VC)>,
    done: bool,
}

impl<T, VC, I> FillGapsIterMap<T, VC, I>
where
    T: Integer,
    VC: ValueCarrier,
    I: SortedDisjointMap<T, VC>,
{
    /// Creates a gap-filling iterator from a sorted, disjoint map stream.
    pub(crate) fn new(iter: I) -> Self {
        Self {
            iter,
            next_start: T::min_value(),
            pending: None,
            done: false,
        }
    }
}

impl<T, VC, I> Iterator for FillGapsIterMap<T, VC, I>
where
    T: Integer,
    VC: ValueCarrier,
    I: SortedDisjointMap<T, VC>,
{
    type Item = (RangeInclusive<T>, Option<VC>);

    fn next(&mut self) -> Option<Self::Item> {
        if self.done {
            return None;
        }

        let Some((range, value)) = self.pending.take().or_else(|| self.iter.next()) else {
            self.done = true;
            return Some((self.next_start..=T::max_value(), None));
        };

        let (start, end) = range.clone().into_inner();
        debug_assert!(start <= end);

        if self.next_start < start {
            let gap = self.next_start..=start.sub_one();
            self.next_start = start;
            self.pending = Some((range, value));
            return Some((gap, None));
        }

        if end == T::max_value() {
            self.done = true;
        } else {
            self.next_start = end.add_one();
        }
        Some((range, Some(value)))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        if self.done {
            return (0, Some(0));
        }

        let (low, high) = self.iter.size_hint();
        let pending = usize::from(self.pending.is_some());
        let low = low.saturating_add(pending);
        let high = high.and_then(|high| high.checked_mul(2)?.checked_add(pending)?.checked_add(1));
        (low, high)
    }
}

impl<T, VC, I> FusedIterator for FillGapsIterMap<T, VC, I>
where
    T: Integer,
    VC: ValueCarrier,
    I: SortedDisjointMap<T, VC>,
{
}
