// cmk
// #![cfg(feature = "rog-experimental")]
// #![deprecated(
//     note = "The rog ('range or gap') module is experimental and may be changed or removed in future versions."
// )]

use core::ops::{Bound, RangeBounds, RangeInclusive};

use alloc::collections::btree_map;

use crate::{Integer, RangeSetBlaze};

/// An iterator over [`Rog`]s (ranges or gaps) in a [`RangeSetBlaze`].
///
/// See [`RangeSetBlaze::rogs_range`] for more information.
pub struct RogsIter<'a, T: Integer> {
    end_in: T,
    next_rog: Option<Rog<T>>,
    final_gap_start: Option<T>,
    btree_map_iter: btree_map::Range<'a, T, T>,
}

impl<T: Integer> Iterator for RogsIter<'_, T> {
    type Item = Rog<T>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(rog) = self.next_rog.take() {
            return Some(rog);
        };

        if let Some((start_el, end_el)) = self.btree_map_iter.next() {
            if self.end_in < *start_el {
                self.btree_map_iter = btree_map::Range::default();
            } else {
                debug_assert!(self.final_gap_start.is_some()); // final_gap_start should be Some if we're in this branch
                debug_assert!(self.final_gap_start.unwrap() < *start_el); // so -1 is safe
                let result = Rog::Gap(self.final_gap_start.unwrap()..=*start_el - T::one());
                if end_el < &self.end_in {
                    self.next_rog = Some(Rog::Range(*start_el..=*end_el));
                    debug_assert!(end_el < &self.end_in); // so +1 is safe
                    self.final_gap_start = Some(*end_el + T::one());
                } else {
                    self.next_rog = Some(Rog::Range(*start_el..=self.end_in));
                    self.final_gap_start = None;
                }
                return Some(result);
            }
        };

        if let Some(gap_start) = self.final_gap_start.take() {
            return Some(Rog::Gap(gap_start..=self.end_in));
        };

        None
    }
}

/// Represents an range or gap in a [`RangeSetBlaze`].
///
/// See [`RangeSetBlaze::rogs_range`] and [`RangeSetBlaze::rogs_get`] for more information.
///
/// # Example
///
/// ```
/// use range_set_blaze::{RangeSetBlaze, Rog};
///
/// let range_set_blaze = RangeSetBlaze::from([1, 2, 3]);
/// assert_eq!(range_set_blaze.rogs_get(2), Rog::Range(1..=3));
/// assert_eq!(range_set_blaze.rogs_get(4), Rog::Gap(4..=2_147_483_647));
/// ```

#[derive(Debug, PartialEq)]
pub enum Rog<T: Integer> {
    /// A range of integers in a [`RangeSetBlaze`].
    Range(RangeInclusive<T>),
    /// A gap between integers in a [`RangeSetBlaze`].
    Gap(RangeInclusive<T>),
}

impl<T: Integer> Rog<T> {
    /// Returns the start of a [`Rog`] (range or gap).
    ///
    /// # Examples
    ///
    /// ```
    /// use range_set_blaze::Rog;
    /// assert_eq!(Rog::Gap(1..=3).start(), 1);
    /// ```
    pub fn start(&self) -> T {
        match self {
            Rog::Range(r) => *r.start(),
            Rog::Gap(r) => *r.start(),
        }
    }

    /// Returns the inclusive end of a [`Rog`] (range or gap).
    ///
    /// # Examples
    ///
    /// ```
    /// use range_set_blaze::Rog;
    /// assert_eq!(Rog::Gap(1..=3).end(), 3);
    /// ```
    pub fn end(&self) -> T {
        match self {
            Rog::Range(r) => *r.end(),
            Rog::Gap(r) => *r.end(),
        }
    }

    /// Returns `true` if the [`Rog`] (range or gap) contains the given integer.
    ///
    /// # Examples
    ///
    /// ```
    /// use range_set_blaze::Rog;
    /// assert!(Rog::Gap(1..=3).contains(2));
    /// assert!(!Rog::Gap(1..=3).contains(4));
    /// ```
    pub fn contains(&self, value: T) -> bool {
        match self {
            Rog::Range(r) => r.contains(&value),
            Rog::Gap(r) => r.contains(&value),
        }
    }
}

impl<T: Integer> RangeSetBlaze<T> {
    /// Returns the [`Rog`] (range or gap) containing the given integer. If the
    /// [`RangeSetBlaze`] contains the integer, returns a [`Rog::Range`]. If the
    /// [`RangeSetBlaze`] does not contain the integer, returns a [`Rog::Gap`].
    ///
    /// # Panics
    ///
    /// Panics if the `value > T::safe_max_value()`.
    ///
    /// # Examples
    ///
    /// ```
    /// use range_set_blaze::{RangeSetBlaze, Rog};
    ///
    /// let range_set_blaze = RangeSetBlaze::from([1, 2, 3]);
    /// assert_eq!(range_set_blaze.rogs_get(2), Rog::Range(1..=3));
    /// assert_eq!(range_set_blaze.rogs_get(4), Rog::Gap(4..=2_147_483_647));
    /// ```
    pub fn rogs_get(&self, value: T) -> Rog<T> {
        assert!(
            value <= T::safe_max_value(),
            "value must be <= T::safe_max_value()"
        );
        let mut before = self.btree_map.range(..=value).rev();
        if let Some((start_before, end_before)) = before.next() {
            if end_before < &value {
                // case 1: range doesn't touch the before range
                let start_out = *end_before + T::one();
                if let Some((start_next, _)) = self.btree_map.range(value..).next() {
                    debug_assert!(start_before < start_next); // so -1 is safe
                    Rog::Gap(start_out..=*start_next - T::one())
                } else {
                    Rog::Gap(start_out..=T::safe_max_value())
                }
            } else {
                // case 2&3: the range touches the before range
                Rog::Range(*start_before..=*end_before)
            }
        } else {
            // case 4: there is no before range
            if let Some((start_next, _)) = self.btree_map.range(value..).next() {
                debug_assert!(value < *start_next); // so -1 is safe
                Rog::Gap(T::min_value()..=*start_next - T::one())
            } else {
                Rog::Gap(T::min_value()..=T::safe_max_value())
            }
        }
    }

    /// Constructs an iterator over a sub-range of [`Rog`]'s (ranges and gaps) in the [`RangeSetBlaze`].
    /// The simplest way is to use the range syntax `min..=max`, thus `range(min..=max)` will
    /// yield elements from min (inclusive) to max (inclusive).
    /// The range may also be entered as `(Bound<T>, Bound<T>)`, so for example
    /// `range((Excluded(4), Included(10)))` will yield a left-exclusive, right-inclusive
    /// range from 4 to 10.
    ///
    /// # Panics
    ///
    /// Panics if range `start > end`.
    ///
    /// Panics if range `start == end` and both bounds are `Excluded`.
    ///
    /// Panics if range `end > T::safe_max_value()`.
    ///
    /// # Examples
    ///
    /// ```rangesetblaze::new()//
    /// use range_set_blaze::{RangeSetBlaze, Rog;};
    /// use std::ops::Bound::Included;
    ///
    /// let mut set = RangeSetBlaze::new();
    /// set.insert(3);
    /// set.insert(5);
    /// set.insert(6);
    /// for rog in set.rogs_range((Included(4), Included(8))) {
    ///     println!("{rog:?}");
    /// } // prints: Gap(4..=4)\nRange(5..=6)\nGap(7..=8)
    ///
    /// assert_eq!(Some(Rog::Gap(4..=4)), set.rogs_range(4..).next());
    ///
    /// let a = RangeSetBlaze::from_iter([1..=6, 11..=15]);
    /// assert_eq!(
    ///     a.rogs_range(-5..=8).collect::<Vec<_>>(),
    ///     vec![Rog::Gap(-5..=0), Rog::Range(1..=6), Rog::Gap(7..=8)]
    /// );
    ///
    /// let empty = RangeSetBlaze::<u8>::new();
    /// assert_eq!(
    ///     empty.rogs_range(..).collect::<Vec<_>>(),
    ///     vec![Rog::Gap(0..=255)]
    /// );
    /// ```
    pub fn rogs_range<R>(&self, range: R) -> RogsIter<T>
    where
        R: RangeBounds<T>,
    {
        let (start_in, end_in) = extract_range(range);

        let mut before = self.btree_map.range(..=start_in).rev();
        if let Some((_, end_before)) = before.next() {
            if end_before < &start_in {
                // case 1: range doesn't touch the before range
                RogsIter {
                    end_in,
                    next_rog: None,
                    final_gap_start: Some(start_in),
                    btree_map_iter: self.btree_map.range(start_in..),
                }
            } else if end_before < &end_in {
                // case 2: the range touches and extends beyond the before range
                debug_assert!(*end_before < end_in); // so +1 is safe
                debug_assert!(start_in <= *end_before); // so +1 is safe
                RogsIter {
                    end_in,
                    next_rog: Some(Rog::Range(start_in..=*end_before)),
                    final_gap_start: Some(*end_before + T::one()),
                    btree_map_iter: self.btree_map.range(start_in + T::one()..),
                }
            } else {
                // case 3 the range is completely contained in the before range
                RogsIter {
                    end_in,
                    next_rog: Some(Rog::Range(start_in..=end_in)),
                    final_gap_start: None,
                    btree_map_iter: btree_map::Range::default(),
                }
            }
        } else {
            // case 4: there is no before range
            RogsIter {
                end_in,
                next_rog: None,
                final_gap_start: Some(start_in),
                btree_map_iter: self.btree_map.range(start_in..),
            }
        }
    }

    /// Used internally to test `rogs_range`.
    #[doc(hidden)]
    pub fn _rogs_range_slow<R>(&self, range: R) -> Vec<Rog<T>>
    where
        R: RangeBounds<T>,
    {
        let (start_in, end_in) = extract_range(range);
        let rsb_in = RangeSetBlaze::from_iter([start_in..=end_in]);
        let ranges = &rsb_in & self;
        let gaps = rsb_in - self;
        let ranges = ranges.ranges().map(|r| Rog::Range(r));
        let gaps = gaps.ranges().map(|r| Rog::Gap(r));
        let mut result = ranges.chain(gaps).collect::<Vec<Rog<T>>>();
        result.sort_by_key(|a| a.start());
        result
    }

    /// Used internally to test `rogs_get`.
    #[doc(hidden)]
    pub fn rogs_get_slow(&self, value: T) -> Rog<T> {
        assert!(
            value <= T::safe_max_value(),
            "value must be <= T::safe_max_value()"
        );
        let all_rogs = self._rogs_range_slow(..);
        for rog in all_rogs {
            if rog.contains(value) {
                return rog;
            }
        }
        unreachable!("value must be in something");
    }
}

fn extract_range<T: Integer, R>(range: R) -> (T, T)
where
    R: RangeBounds<T>,
{
    let start = match range.start_bound() {
        Bound::Included(n) => *n,
        Bound::Excluded(n) => {
            assert!(
                *n < T::safe_max_value(),
                "inclusive start must be <= T::max_safe_value()"
            );
            *n + T::one()
        }
        Bound::Unbounded => T::min_value(),
    };
    let end = match range.end_bound() {
        Bound::Included(n) => *n,
        Bound::Excluded(n) => {
            assert!(
                *n > T::min_value(),
                "inclusive end must be >= T::min_value()"
            );
            *n - T::one()
        }
        Bound::Unbounded => T::safe_max_value(),
    };
    assert!(start <= end, "start must be <= end");
    assert!(
        end <= T::safe_max_value(),
        "end must be <= T::safe_max_value()"
    );

    (start, end)
}

// cmk
// #[cfg(test)]
// mod tests {
//     use std::panic::{self, AssertUnwindSafe};

//     use super::*; // Import the parent module's contents.
// }
