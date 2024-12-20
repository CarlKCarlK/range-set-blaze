#![deprecated(
    note = "The rog ('range or gap') module is experimental and may be changed or removed in future versions.
    Changes may not be reflected in the semantic versioning."
)]

use alloc::collections::btree_map;
use alloc::vec::Vec;
use core::ops::{RangeBounds, RangeInclusive};

use crate::{set::extract_range, Integer, RangeSetBlaze};

/// Experimental: This struct is created by the [`rogs_range`] method on  [`RangeSetBlaze`].
/// See [`rogs_range`] for more information.
///
/// [`RangeSetBlaze`]: crate::RangeSetBlaze
/// [`rogs_range`]: crate::RangeSetBlaze::rogs_range
#[must_use = "iterators are lazy and do nothing unless consumed"]
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
                let result = Rog::Gap(self.final_gap_start.unwrap()..=start_el.sub_one());
                if end_el < &self.end_in {
                    self.next_rog = Some(Rog::Range(*start_el..=*end_el));
                    debug_assert!(end_el < &self.end_in); // so +1 is safe
                    self.final_gap_start = Some(end_el.add_one());
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

/// Experimental: Represents an range or gap in a [`RangeSetBlaze`].
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

#[derive(Debug, PartialEq, Eq)]
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
    pub const fn start(&self) -> T {
        match self {
            Self::Gap(r) | Self::Range(r) => *r.start(),
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
    pub const fn end(&self) -> T {
        match self {
            Self::Gap(r) | Self::Range(r) => *r.end(),
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
            Self::Gap(r) | Self::Range(r) => r.contains(&value),
        }
    }
}

impl<T: Integer> RangeSetBlaze<T> {
    /// Experimental: Returns the [`Rog`] (range or gap) containing the given integer. If the
    /// [`RangeSetBlaze`] contains the integer, returns a [`Rog::Range`]. If the
    /// [`RangeSetBlaze`] does not contain the integer, returns a [`Rog::Gap`].
    ///
    /// # Enabling
    ///
    /// This method is experimental and must be enabled with the `rog-experimental` feature.
    /// ```bash
    /// cargo add range-set-blaze --features "rog-experimental"
    /// ```
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
        let mut before = self.btree_map.range(..=value).rev();
        if let Some((start_before, end_before)) = before.next() {
            if end_before < &value {
                // case 1: range doesn't touch the before range
                let start_out = end_before.add_one();
                if let Some((start_next, _)) = self.btree_map.range(value..).next() {
                    debug_assert!(start_before < start_next); // so -1 is safe
                    Rog::Gap(start_out..=start_next.sub_one())
                } else {
                    Rog::Gap(start_out..=T::max_value())
                }
            } else {
                // case 2&3: the range touches the before range
                Rog::Range(*start_before..=*end_before)
            }
        } else {
            // case 4: there is no before range
            if let Some((start_next, _)) = self.btree_map.range(value..).next() {
                debug_assert!(value < *start_next); // so -1 is safe
                Rog::Gap(T::min_value()..=start_next.sub_one())
            } else {
                Rog::Gap(T::min_value()..=T::max_value())
            }
        }
    }

    /// Experimental: Constructs an iterator over a sub-range of [`Rog`]'s (ranges and gaps) in the [`RangeSetBlaze`].
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
    /// # Enabling
    ///
    /// This method is experimental and must be enabled with the `rog-experimental` feature.
    /// ```bash
    /// cargo add range-set-blaze --features "rog-experimental"
    /// ```
    ///
    /// # Examples
    ///
    /// ```
    /// use range_set_blaze::{RangeSetBlaze, Rog};
    /// use core::ops::Bound::Included;
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
        assert!(
            start_in <= end_in,
            "start must be less than or equal to end"
        );

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
                    final_gap_start: Some(end_before.add_one()),
                    btree_map_iter: self.btree_map.range(start_in.add_one()..),
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
    // cmk extract_range can now return empty range. Test that this does the right thing.
    #[doc(hidden)]
    pub fn rogs_range_slow<R>(&self, range: R) -> Vec<Rog<T>>
    where
        R: RangeBounds<T>,
    {
        let (start_in, end_in) = extract_range(range);
        let rsb_in = Self::from_iter([start_in..=end_in]);
        let ranges = &rsb_in & self;
        let gaps = rsb_in - self;
        let ranges = ranges.ranges().map(|r| Rog::Range(r));
        let gaps = gaps.ranges().map(|r| Rog::Gap(r));
        let mut result = ranges.chain(gaps).collect::<Vec<Rog<T>>>();
        result.sort_by_key(Rog::start);
        result
    }

    /// Used internally to test `rogs_get`.
    #[doc(hidden)]
    pub fn rogs_get_slow(&self, value: T) -> Rog<T> {
        let all_rogs = self.rogs_range_slow(..);
        for rog in all_rogs {
            if rog.contains(value) {
                return rog;
            }
        }
        unreachable!("value must be in something");
    }
}
