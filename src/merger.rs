use std::cmp::{max, min};

use crate::{Integer, RangeSetInt};

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
        Merger::from_iter(iter.into_iter().map(|item| (item, item)))
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
    pub fn collect_into(&mut self, range_set_int: &mut RangeSetInt<T>) {
        if let Merger::Some {
            range_list,
            lower,
            upper,
        } = self
        {
            range_list.push((*lower, *upper));
            range_list.sort_unstable_by(|a, b| a.0.cmp(&b.0));
            SortedRanges::process(range_set_int, range_list);
            *self = Merger::None;
        }
    }

    fn insert(&mut self, lower: T, upper: T) {
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
                sort_list.push((*self_lower, *self_upper));
                *self_lower = lower;
                *self_upper = upper;
            } else {
                *self_lower = min(*self_lower, lower);
                *self_upper = max(*self_upper, upper);
            }
        } else {
            *self = Merger::Some {
                range_list: Vec::new(),
                lower,
                upper,
            };
        }
    }
    // !!!cmk0 better as from_iter?
}

enum OptionRange<T: Integer> {
    None,
    Some { start: T, stop: T },
}

pub struct SortedRanges<'a, T: Integer> {
    range_set_int: &'a mut RangeSetInt<T>,
    range: OptionRange<T>,
}

impl<'a, T: Integer> SortedRanges<'a, T> {
    fn process(range_set_int: &'a mut RangeSetInt<T>, range_list: &mut Vec<(T, T)>) {
        let mut sorted_ranges = SortedRanges {
            range_set_int,
            range: OptionRange::None,
        };
        for (start, stop) in range_list {
            sorted_ranges.insert(start, stop);
        }
        if let OptionRange::Some { start, stop } = sorted_ranges.range {
            sorted_ranges.push(start, stop);
        }
    }

    fn insert(&mut self, start: &mut T, stop: &mut T) {
        self.range = match self.range {
            OptionRange::None => OptionRange::Some {
                start: *start,
                stop: *stop,
            },
            OptionRange::Some {
                start: current_start,
                stop: current_stop,
            } => {
                debug_assert!(current_start <= *start); // !!! cmk panic because not sorted;
                if current_stop < T::max_value2() && *start <= current_stop + T::one() {
                    OptionRange::Some {
                        start: current_start,
                        stop: max(current_stop, *stop),
                    }
                } else {
                    self.push(current_start, current_stop);
                    OptionRange::Some {
                        start: *start,
                        stop: *stop,
                    }
                }
            }
        };
    }

    fn push(&mut self, start: T, stop: T) {
        self.range_set_int.items.insert(start, stop);
        self.range_set_int.len += T::safe_subtract_inclusive(stop, start);
    }
}
