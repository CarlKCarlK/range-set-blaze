use std::{
    cmp::{max, min},
    collections::BTreeMap,
};

use crate::{Integer, RangeSetInt, SafeSubtract};

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
    fn from(mut val: Merger<T>) -> Self {
        let mut range_set_int = RangeSetInt::new();
        val.extract(&mut range_set_int);
        range_set_int
    }
}

impl<T: Integer> Merger<T> {
    // !!!cmk rename to something better
    pub fn extract(&mut self, range_set_int: &mut RangeSetInt<T>) {
        if let Merger::Some {
            range_list,
            lower,
            upper,
        } = self
        {
            range_list.push((*lower, *upper));
            MergeRangeList::merge(range_list, &mut range_set_int.items, &mut range_set_int.len);
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
        let mut sortie = Merger::new();
        for (lower, upper) in iter {
            sortie.insert(lower, upper);
        }
        sortie
    }
}

pub enum MergeRangeList<T: Integer> {
    None,
    Some { start: T, stop: T },
}

impl<T: Integer> MergeRangeList<T> {
    pub fn merge(
        sort_list: &mut Vec<(T, T)>,
        items: &mut BTreeMap<T, T>,
        len: &mut <T as SafeSubtract>::Output,
    ) {
        sort_list.sort_unstable_by(|a, b| a.0.cmp(&b.0));

        let mut x32 = MergeRangeList::None;
        for (start, stop) in sort_list {
            x32.insert(start, stop, items, len);
        }
        x32.extract(items, len);
    }

    fn insert(
        &mut self,
        start: &mut T,
        stop: &mut T,
        items: &mut BTreeMap<T, T>,
        len: &mut <T as SafeSubtract>::Output,
    ) {
        match self {
            MergeRangeList::None => {
                *self = MergeRangeList::Some {
                    start: *start,
                    stop: *stop,
                };
            }
            MergeRangeList::Some {
                start: current_start,
                stop: current_stop,
            } => {
                // !!!cmk check for overflow with the +1
                if *start <= *current_stop + T::one() {
                    *current_stop = max(*current_stop, *stop);
                } else {
                    items.insert(*current_start, *current_stop);
                    *len += T::safe_subtract_inclusive(*current_stop, *current_start);
                    *current_start = *start;
                    *current_stop = *stop;
                }
            }
        }
    }

    fn extract(&mut self, items: &mut BTreeMap<T, T>, len: &mut <T as SafeSubtract>::Output) {
        if let MergeRangeList::Some { start, stop } = self {
            items.insert(*start, *stop);
            *len += T::safe_subtract_inclusive(*stop, *start);
        }
        *self = MergeRangeList::None;
    }
}
