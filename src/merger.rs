/// Merger collects items and pairs into a vector of ranges
/// that it sorts and then sends to SortedRanges::process.
///
/// SortedRanges::process merges the ranges into an *empty* RangeSetInt.
/// cmk00 however it doesn't check that the RangeSetInt is empty. bug bug bug
///
/// It is used by RangeSetInt::from_iter (single items) and from("1..=5, 7..=9"), which is
///   not as good as using btreemap.from_iter
///
/// But it is doing the wrong thing for extend. cmk0000
use std::cmp::{max, min};

use crate::{Integer, OptionRange, RangeSetInt};

// !!!cmk0 improve the name and understanding of Merger
#[derive(Debug)]
pub enum Merger<T: Integer> {
    None,
    Some {
        range_list: Vec<(T, T)>,
        lower: T,
        upper: T,
    },
}

impl<T: Integer> FromIterator<T> for Merger<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        iter.into_iter().map(|item| (item, item)).collect()
    }
}

impl<T: Integer> FromIterator<(T, T)> for Merger<T> {
    fn from_iter<I: IntoIterator<Item = (T, T)>>(iter: I) -> Self {
        let mut merger = Merger::None;
        for (lower, upper) in iter {
            merger.insert(lower, upper);
        }
        merger
    }
}

impl<T: Integer> From<Merger<T>> for RangeSetInt<T> {
    fn from(mut merger: Merger<T>) -> Self {
        let mut range_set_int = RangeSetInt::new();
        merger.collect_into(&mut range_set_int);
        range_set_int
    }
}

impl<T: Integer> Merger<T> {
    // cmk note that iterator's collect_into is experimental

    pub fn collect_into(&mut self, range_set_int: &mut RangeSetInt<T>) {
        if let Merger::Some {
            range_list,
            lower,
            upper,
        } = self
        {
            range_list.push((*lower, *upper));
            range_list.sort_unstable_by(|a, b| a.0.cmp(&b.0));
            let iter = range_list.iter().cloned();
            SortedRanges::process(range_set_int, iter);
            *self = Merger::None;
        }
    }

    fn insert(&mut self, lower: T, upper: T) {
        // cmk println!("inserting ({lower:?}, {upper:?})");
        let two = T::one() + T::one();
        assert!(lower <= upper && upper <= T::max_value2());
        if let Merger::Some {
            range_list: sort_list,
            lower: self_lower,
            upper: self_upper,
        } = self
        {
            if (lower >= two && lower - two >= *self_upper)
                || (*self_lower >= two && *self_lower - two >= upper)
            {
                // cmk println!("pushing onto sort list, saving ({lower:?}, {upper:?})");
                sort_list.push((*self_lower, *self_upper));
                *self_lower = lower;
                *self_upper = upper;
            } else {
                // cmk println!("merging in ({lower:?}, {upper:?})");
                *self_lower = min(*self_lower, lower);
                *self_upper = max(*self_upper, upper);
                // cmk println!("merging in, now ({self_lower:?}, {self_upper:?})");
            }
        } else {
            // cmk println!("creating empty sort list, saving ({lower:?}, {upper:?})");
            *self = Merger::Some {
                range_list: Vec::new(),
                lower,
                upper,
            };
        }
    }
}

// !!! cmk0 change this to build from an array. Benchmark it.
pub struct SortedRanges<'a, T: Integer> {
    range_set_int: &'a mut RangeSetInt<T>,
    range: OptionRange<T>,
}

impl<'a, T: Integer> SortedRanges<'a, T> {
    pub fn process<I>(range_set_int: &'a mut RangeSetInt<T>, sorted_range_iter: I)
    where
        I: Iterator<Item = (T, T)>,
    {
        // cmk println!("SR: start process");
        let mut sorted_ranges = SortedRanges {
            range_set_int,
            range: OptionRange::None,
        };
        for (start, stop) in sorted_range_iter {
            sorted_ranges.insert(start, stop);
        }
        if let OptionRange::Some { start, stop } = sorted_ranges.range {
            // cmk println!("SR: final push");
            sorted_ranges.push(start, stop);
        }
    }

    fn insert(&mut self, start: T, stop: T) {
        // cmk println!("SR: inserting ({start:?}, {stop:?})");
        self.range = match self.range {
            OptionRange::None => OptionRange::Some { start, stop },
            OptionRange::Some {
                start: current_start,
                stop: current_stop,
            } => {
                debug_assert!(current_start <= start); // panic if not sorted
                if start <= current_stop
                    || (current_stop < T::max_value2() && start <= current_stop + T::one())
                {
                    // cmk println!("SR: merging");
                    OptionRange::Some {
                        start: current_start,
                        stop: max(current_stop, stop),
                    }
                } else {
                    // cmk println!("SR: push & new");
                    self.push(current_start, current_stop);
                    OptionRange::Some { start, stop }
                }
            }
        };
        // if let OptionRange::Some { start, stop } = self.range {
        //     // cmk println!("SR: range is now {start}..={stop}");
        // } else {
        //     // cmk println!("SR: range is now None");
        // }
    }

    fn push(&mut self, start: T, stop: T) {
        self.range_set_int.items.insert(start, stop);
        self.range_set_int.len += T::safe_subtract_inclusive(stop, start);
    }
}
