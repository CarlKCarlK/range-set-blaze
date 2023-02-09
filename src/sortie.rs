use num_traits::Zero;
use std::{
    cmp::{max, min},
    collections::BTreeMap,
};

use crate::{Integer, SafeSubtract};

#[derive(Debug)]
pub struct Sortie<T: Integer> {
    sort_list: Vec<(T, T)>,
    is_empty: bool,
    lower: T,
    upper: T,
    two: T, // !!!cmk constant
}

impl<T: Integer> Sortie<T> {
    pub fn new() -> Self {
        Self {
            sort_list: Vec::new(),
            is_empty: true,
            lower: T::zero(),
            upper: T::zero(),
            two: T::one() + T::one(),
        }
    }
    pub fn insert_range(&mut self, lower: T, upper: T) {
        assert!(lower <= upper && upper <= T::max_value2());
        if self.is_empty {
            self.lower = lower;
            self.upper = upper;
            self.is_empty = false;
        } else if (self.lower >= self.two && self.lower - self.two >= upper)
            || (lower >= self.two && lower - self.two >= self.upper)
        {
            self.sort_list.push((self.lower, self.upper));
            self.lower = lower;
            self.upper = upper;
        } else {
            self.lower = min(self.lower, lower);
            self.upper = max(self.upper, upper);
        }
    }
    // !!!cmk0 better as from_iter?
    pub fn insert(&mut self, i: T) {
        assert!(i <= T::max_value2()); // !!!cmk raise error
        if self.is_empty {
            self.lower = i;
            self.upper = i;
            self.is_empty = false;
        } else {
            if self.lower <= i && i <= self.upper {
                return;
            }
            if T::zero() < self.lower && self.lower - T::one() == i {
                self.lower = i;
                return;
            }
            if self.upper < T::max_value2() && self.upper + T::one() == i {
                self.upper = i;
                return;
            }
            self.sort_list.push((self.lower, self.upper));
            self.lower = i;
            self.upper = i;
        }
    }

    pub fn range_int_set(self) -> (BTreeMap<T, T>, <T as SafeSubtract>::Output) {
        let mut items = BTreeMap::new();
        let mut len = <T as SafeSubtract>::Output::zero();
        self.extend_x(&mut items, &mut len);
        (items, len)
    }

    // !!!cmk rename to something better
    pub fn extend_x(mut self, items: &mut BTreeMap<T, T>, len: &mut <T as SafeSubtract>::Output) {
        if !self.is_empty {
            self.sort_list.push((self.lower, self.upper));
            self.is_empty = true;
        }
        let mut sort_list = self.sort_list;
        sort_list.sort_unstable_by(|a, b| a.0.cmp(&b.0));

        let mut is_empty = true;
        let mut current_start = T::zero();
        let mut current_stop = T::zero();
        for (start, stop) in sort_list {
            if is_empty {
                current_start = start;
                current_stop = stop;
                is_empty = false;
            }
            // !!!cmk check for overflow with the +1
            else if start <= current_stop + T::one() {
                current_stop = max(current_stop, stop);
            } else {
                items.insert(current_start, current_stop);
                *len += T::safe_subtract_inclusive(current_stop, current_start);
                current_start = start;
                current_stop = stop;
            }
        }
        if !is_empty {
            items.insert(current_start, current_stop);
            *len += T::safe_subtract_inclusive(current_stop, current_start);
        }
    }

    // !!! cmk what if forget to call this?

    // fn merge(mut self, mut other: Self) -> Self {
    //     self.save();
    //     other.save();
    //     self.sort_list.extend(other.sort_list);
    //     self
    // }
}
