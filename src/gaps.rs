//! # Ranges and gaps
//!
//! [`RangeSetBlaze`][crate::RangeSetBlaze] and [`RangeMapBlaze`][crate::RangeMapBlaze] store integers as a sorted list
//! of ranges instead of individual integers, and they always merge
//! neighboring or overlapping ranges as you insert. So a `RangeSetBlaze`
//! built from `1..=3` and `7..=10` holds exactly those two ranges — no more,
//! no fewer. Everything else — `4..=6`, and everything below `1` or above
//! `10` — is a gap: a run of integers not covered by any range.
//!
//! # Table of Contents
//! * [`RangeSetBlaze`](#rangesetblaze)
//!    * [`range_at`: only present ranges](#range_at-only-present-ranges)
//!    * [`range_or_gap_at`: present range or gap](#range_or_gap_at-present-range-or-gap)
//!    * [`fill_gaps`: fill gaps with `false`](#fill_gaps-fill-gaps-with-false)
//!    * [Streaming `fill_gaps` for `RangeSetBlaze`](#streaming-fill_gaps-for-rangesetblaze)
//! * [`RangeMapBlaze`](#rangemapblaze)
//!    * [`range_at`: only mapped ranges](#range_at-only-mapped-ranges)
//!    * [`range_or_gap_at`: mapped range or gap](#range_or_gap_at-mapped-range-or-gap)
//!    * [`fill_gaps`: fill gaps with `None`](#fill_gaps-fill-gaps-with-none)
//!    * [Streaming `fill_gaps` for `RangeMapBlaze`](#streaming-fill_gaps-for-rangemapblaze)
//! * [A filled map has an entry for every integer](#a-filled-map-has-an-entry-for-every-integer)
//!
//! ## `RangeSetBlaze`
//!
//! ### `range_at`: only present ranges
//!
//! Suppose you want to find which range a value belongs to.
//! [`RangeSetBlaze::range_at`][crate::RangeSetBlaze::range_at] returns the maximal range
//! containing `value`:
//!
//! ```
//! use range_set_blaze::RangeSetBlaze;
//!
//! let set = RangeSetBlaze::from_iter([1_i8..=3, 7..=10]);
//! assert_eq!(set.range_at(2), Some(1..=3));
//! ```
//!
//! What if the value isn't in any range? `range_at` returns `None`:
//!
//! ```
//! use range_set_blaze::RangeSetBlaze;
//!
//! let set = RangeSetBlaze::from_iter([1_i8..=3, 7..=10]);
//! assert_eq!(set.range_at(5), None); // 5 is in the gap between the two ranges
//! ```
//!
//! ### `range_or_gap_at`: present range or gap
//!
//! What if you want the gap itself, instead of `None`?
//! [`RangeSetBlaze::range_or_gap_at`][crate::RangeSetBlaze::range_or_gap_at] always returns the maximal
//! range containing `value`, as `(range, bool)` where `true` means present:
//!
//! ```
//! use range_set_blaze::RangeSetBlaze;
//!
//! let set = RangeSetBlaze::from_iter([1_i8..=3, 7..=10]);
//! assert_eq!(set.range_or_gap_at(5), (4..=6, false)); // the gap between the two ranges
//! ```
//!
//! A gap also extends to the domain's own bounds, so a query below the first
//! range or above the last one returns the leading or trailing gap:
//!
//! ```
//! use range_set_blaze::RangeSetBlaze;
//!
//! let set = RangeSetBlaze::from_iter([1_i8..=3, 7..=10]);
//! assert_eq!(set.range_or_gap_at(i8::MIN), (i8::MIN..=0, false));
//! assert_eq!(set.range_or_gap_at(i8::MAX), (11..=i8::MAX, false));
//! ```
//!
//! ### `fill_gaps`: fill gaps with `false`
//!
//! [`RangeSetBlaze::fill_gaps`][crate::RangeSetBlaze::fill_gaps] builds a new `RangeMapBlaze<T, bool>`
//! with an entry for every integer, not just the ones in the original set:
//! `true` for values that were present, `false` for values that were in a
//! gap.
//!
//! ```
//! use range_set_blaze::{RangeMapBlaze, RangeSetBlaze};
//!
//! let set = RangeSetBlaze::from_iter([1_i8..=3, 7..=10]);
//! let filled_set = set.fill_gaps();
//! assert_eq!(
//!     filled_set,
//!     RangeMapBlaze::from_iter([
//!         (i8::MIN..=0, false),
//!         (1..=3, true),
//!         (4..=6, false),
//!         (7..=10, true),
//!         (11..=i8::MAX, false),
//!     ])
//! );
//! ```
//!
//! ### Streaming `fill_gaps` for `RangeSetBlaze`
//!
//! If the whole materialized `RangeMapBlaze` is not needed, [`SortedDisjoint::fill_gaps`]
//! streams the same result lazily from a set stream such as `set.ranges()`,
//! yielding `(range, bool)` and including the leading and trailing gaps:
//!
//! ```
//! use range_set_blaze::{RangeSetBlaze, SortedDisjoint};
//!
//! let set = RangeSetBlaze::from_iter([1_i8..=3, 7..=10]);
//! let mut set_stream = set.ranges().fill_gaps();
//! assert_eq!(set_stream.next(), Some((i8::MIN..=0, false)));
//! assert_eq!(set_stream.next(), Some((1..=3, true)));
//! assert_eq!(set_stream.next(), Some((4..=6, false)));
//! assert_eq!(set_stream.next(), Some((7..=10, true)));
//! assert_eq!(set_stream.next(), Some((11..=i8::MAX, false)));
//! assert_eq!(set_stream.next(), None);
//! ```
//!
//! ## `RangeMapBlaze`
//!
//! ### `range_at`: only mapped ranges
//!
//! What if this is a map instead of a set? [`RangeMapBlaze::range_at`][crate::RangeMapBlaze::range_at]
//! works the same way, returning the maximal range and its value:
//!
//! ```
//! use range_set_blaze::RangeMapBlaze;
//!
//! let map = RangeMapBlaze::from_iter([(1_i8..=3, "red"), (7..=10, "blue")]);
//! assert_eq!(map.range_at(2), Some((1..=3, &"red")));
//! assert_eq!(map.range_at(5), None); // 5 is in the gap between the two ranges
//! ```
//!
//! ### `range_or_gap_at`: mapped range or gap
//!
//! [`RangeMapBlaze::range_or_gap_at`][crate::RangeMapBlaze::range_or_gap_at] is the map counterpart of
//! `range_or_gap_at` above: it always returns the maximal range containing
//! `key`, as `(range, Option<&V>)` where `Some` means mapped:
//!
//! ```
//! use range_set_blaze::RangeMapBlaze;
//!
//! let map = RangeMapBlaze::from_iter([(1_i8..=3, "red"), (7..=10, "blue")]);
//! assert_eq!(map.range_or_gap_at(5), (4..=6, None)); // the gap between the two ranges
//! assert_eq!(map.range_or_gap_at(8), (7..=10, Some(&"blue")));
//! ```
//!
//! ### `fill_gaps`: fill gaps with `None`
//!
//! [`RangeMapBlaze::fill_gaps`][crate::RangeMapBlaze::fill_gaps] builds a new
//! `RangeMapBlaze<T, Option<V>>` with an entry for every integer, not just
//! the ones in the original map: `Some(value)` for keys that were mapped,
//! `None` for keys that were in a gap.
//!
//! ```
//! use range_set_blaze::RangeMapBlaze;
//!
//! let map = RangeMapBlaze::from_iter([(1_i8..=3, "red"), (7..=10, "blue")]);
//! let filled_map = map.fill_gaps();
//! assert_eq!(
//!     filled_map,
//!     RangeMapBlaze::from_iter([
//!         (i8::MIN..=0, None),
//!         (1..=3, Some("red")),
//!         (4..=6, None),
//!         (7..=10, Some("blue")),
//!         (11..=i8::MAX, None),
//!     ])
//! );
//! ```
//!
//! ### Streaming `fill_gaps` for `RangeMapBlaze`
//!
//! If the whole materialized `RangeMapBlaze` is not needed, [`SortedDisjointMap::fill_gaps`]
//! streams the same result lazily from a map stream such as
//! `map.range_values()`, yielding `(range, Option<&V>)` and borrowing the
//! original values instead of cloning them:
//!
//! ```
//! use range_set_blaze::{RangeMapBlaze, SortedDisjointMap};
//!
//! let map = RangeMapBlaze::from_iter([(1_i8..=3, "red"), (7..=10, "blue")]);
//! let mut map_stream = map.range_values().fill_gaps();
//! assert_eq!(map_stream.next(), Some((i8::MIN..=0, None)));
//! assert_eq!(map_stream.next(), Some((1..=3, Some(&"red"))));
//! assert_eq!(map_stream.next(), Some((4..=6, None)));
//! assert_eq!(map_stream.next(), Some((7..=10, Some(&"blue"))));
//! assert_eq!(map_stream.next(), Some((11..=i8::MAX, None)));
//! assert_eq!(map_stream.next(), None);
//! ```
//!
//! ## A filled map has an entry for every integer
//!
//! After `fill_gaps`, `false`/`None` are just ordinary values sitting in the
//! map — the map itself now has a key for every integer, with no gaps left
//! at all. This matters if you then apply the `!` (complement) operator,
//! since `!` on a `RangeMapBlaze` means "the keys *not* in this map," not
//! "flip each `bool`/`Option` value." Because a filled map already has every
//! key, its complement is always empty — it does **not** flip `true` to
//! `false`:
//!
//! ```
//! use range_set_blaze::RangeSetBlaze;
//!
//! let set = RangeSetBlaze::from_iter([1_i8..=3, 7..=10]);
//! let filled_set = set.fill_gaps();
//! assert!((!filled_set).is_empty()); // not the Boolean negation you might expect
//! ```

use core::{iter::FusedIterator, ops::RangeInclusive};

use crate::{Integer, SortedDisjoint, SortedDisjointMap, map::ValueCarrier};

/// TODO0000 A lazy iterator that fills the gaps in a sorted, disjoint set stream.
///
/// Present ranges are returned with `true`, and missing portions of the
/// integer domain are returned with `false`. The output covers the complete
/// domain from [`Integer::min_value`] through [`Integer::max_value`].
/// In other words, this is a total Boolean-valued map stream: `true` means
/// membership in the input set and `false` means a gap. The `false` values
/// are ordinary map values, so the resulting map's key domain is universal.
///
/// This iterator is created by [`SortedDisjoint::fill_gaps`], typically from a
/// set stream such as [`RangeSetBlaze::ranges`]. To materialize the result
/// instead of streaming it, use [`RangeSetBlaze::fill_gaps`].
///
/// The iterator implements [`SortedDisjointMap<T, bool>`], so it supports the
/// sorted-disjoint map operations and can be collected into a
/// [`RangeMapBlaze<T, bool>`].
///
/// Because `true` and `false` are ordinary map values, map operators act on the
/// stream's key ranges; for example, `!filled` is empty rather than Boolean
/// negation.
///
/// [`SortedDisjointMap<T, bool>`]: crate::SortedDisjointMap
/// [`RangeMapBlaze<T, bool>`]: crate::RangeMapBlaze
/// [`RangeSetBlaze::ranges`]: crate::RangeSetBlaze::ranges
/// [`RangeSetBlaze::fill_gaps`]: crate::RangeSetBlaze::fill_gaps
///
/// For the set and map APIs together, see the [Ranges and gaps guide][crate::gaps].
///
/// # Example
///
/// ```
/// use range_set_blaze::{
///     CheckSortedDisjoint, RangeMapBlaze, RangeSetBlaze, SortedDisjoint, SortedDisjointMap,
/// };
///
/// // From the streaming layer directly.
/// let input = CheckSortedDisjoint::new([1..=3, 7..=10]);
/// let output = input.fill_gaps().collect::<Vec<_>>();
/// assert_eq!(output[0], (i32::MIN..=0, false));
/// assert_eq!(output[1], (1..=3, true));
/// assert_eq!(output[2], (4..=6, false));
/// assert_eq!(output[3], (7..=10, true));
/// assert_eq!(output[4], (11..=i32::MAX, false));
///
/// // Or explicitly materialize the streaming result into a map.
/// let set = RangeSetBlaze::from_iter([1_u8..=3, 7..=10]);
/// let map: RangeMapBlaze<u8, bool> = set.ranges().fill_gaps().into_range_map_blaze();
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

/// todo000 A lazy iterator that fills the gaps in a sorted, disjoint map stream.
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
/// This iterator is created by [`SortedDisjointMap::fill_gaps`], typically from
/// a map stream such as [`RangeMapBlaze::range_values`]. To materialize the
/// result instead of streaming it, use [`RangeMapBlaze::fill_gaps`].
///
/// [`RangeMapBlaze::range_values`]: crate::RangeMapBlaze::range_values
/// [`RangeMapBlaze::fill_gaps`]: crate::RangeMapBlaze::fill_gaps
///
/// For the set and map APIs together, see the [Ranges and gaps guide][crate::gaps].
///
/// # Example
///
/// ```
/// use range_set_blaze::{
///     CheckSortedDisjointMap, RangeMapBlaze, SortedDisjointMap,
/// };
///
/// // From the streaming layer directly.
/// let input = CheckSortedDisjointMap::new([(1..=3, &"red"), (7..=10, &"blue")]);
/// let output = input.fill_gaps().collect::<Vec<_>>();
/// assert_eq!(output[0], (0..=0, None));
/// assert_eq!(output[1], (1..=3, Some(&"red")));
/// assert_eq!(output[2], (4..=6, None));
/// assert_eq!(output[3], (7..=10, Some(&"blue")));
/// assert_eq!(output[4], (11..=u8::MAX, None));
///
/// // Or explicitly materialize the streaming result into a map.
/// let map = RangeMapBlaze::from_iter([(1_u8..=3, "red"), (7..=10, "blue")]);
/// let filled: RangeMapBlaze<u8, Option<&str>> = map.range_values().fill_gaps()
///     .into_range_map_blaze();
/// assert_eq!(filled.get(5), Some(&None));
/// assert_eq!(filled.get(2), Some(&Some("red")));
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
