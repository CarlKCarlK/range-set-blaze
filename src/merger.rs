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

impl<T: Integer> Default for Merger<T> {
    fn default() -> Self {
        Self::None
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
    // !!!cmk rename to something better
    pub fn collect_into(&mut self, range_set_int: &mut RangeSetInt<T>) {
        if let Merger::Some {
            range_list,
            lower,
            upper,
        } = self
        {
            range_list.push((*lower, *upper));
            range_list.sort_unstable_by(|a, b| a.0.cmp(&b.0));

            let mut merge_range_list = MergeRange::new(range_set_int);
            merge_range_list.extend(range_list);
            *self = Merger::None;
        }
    }

    fn new() -> Self {
        Self::None
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

impl<T: Integer> FromIterator<(T, T)> for Merger<T> {
    fn from_iter<I: IntoIterator<Item = (T, T)>>(iter: I) -> Self {
        let mut merger = Merger::new();
        for (lower, upper) in iter {
            merger.insert(lower, upper);
        }
        merger
    }
}

enum RangeEnum<T: Integer> {
    None,
    Some { start: T, stop: T },
}

pub struct MergeRange<'a, T: Integer> {
    range_set_int: &'a mut RangeSetInt<T>,
    range: RangeEnum<T>,
}

impl<'a, T: Integer> MergeRange<'a, T> {
    fn new(range_set_int: &'a mut RangeSetInt<T>) -> Self {
        MergeRange {
            range_set_int,
            range: RangeEnum::None,
        }
    }

    fn extend(&mut self, range_list: &mut Vec<(T, T)>) {
        for (start, stop) in range_list {
            self.insert_sorted(start, stop);
        }
        if let RangeEnum::Some { start, stop } = self.range {
            self.range_set_int.items.insert(start, stop);
            self.range_set_int.len += T::safe_subtract_inclusive(stop, start);
        }
    }

    fn insert_sorted(&mut self, start: &mut T, stop: &mut T) {
        match self.range {
            RangeEnum::None => {
                self.range = RangeEnum::Some {
                    start: *start,
                    stop: *stop,
                };
            }
            RangeEnum::Some {
                start: mut current_start,
                stop: mut current_stop,
            } => {
                if current_stop < T::max_value2() && *start <= current_stop + T::one() {
                    current_stop = max(current_stop, *stop);
                } else {
                    self.range_set_int.items.insert(current_start, current_stop);
                    self.range_set_int.len +=
                        T::safe_subtract_inclusive(current_stop, current_start);
                    current_start = *start;
                    current_stop = *stop;
                }
                self.range = RangeEnum::Some {
                    start: current_start,
                    stop: current_stop,
                };
            }
        }
    }
}
