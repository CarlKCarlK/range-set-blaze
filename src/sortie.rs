use num_traits::Zero;
use std::{cmp::max, collections::BTreeMap};

use crate::{Integer, RangeSetInt, SafeSubtract};

#[derive(Debug)]
pub struct Sortie<T: Integer> {
    sort_list: Vec<(T, T)>,
    is_empty: bool,
    lower: T,
    upper: T,
}

impl<T: Integer> Sortie<T> {
    pub fn new() -> Self {
        Self {
            sort_list: Vec::new(),
            is_empty: true,
            lower: T::zero(),
            upper: T::zero(),
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

    pub fn range_int_set(mut self) -> (BTreeMap<T, T>, <T as SafeSubtract>::Output) {
        // !!!cmk fix do can't forget 'save'
        if !self.is_empty {
            self.sort_list.push((self.lower, self.upper));
            self.is_empty = true;
        }
        let mut sort_list = self.sort_list;
        sort_list.sort_unstable_by(|a, b| a.0.cmp(&b.0));

        let mut items = BTreeMap::new();
        let mut len = <T as SafeSubtract>::Output::zero();

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
                len += T::safe_subtract_inclusive(current_stop, current_start);
                current_start = start;
                current_stop = stop;
            }
        }
        if !is_empty {
            items.insert(current_start, current_stop);
            len += T::safe_subtract_inclusive(current_stop, current_start);
        }
        (items, len)
    }

    // !!! cmk what if forget to call this?

    // fn merge(mut self, mut other: Self) -> Self {
    //     self.save();
    //     other.save();
    //     self.sort_list.extend(other.sort_list);
    //     self
    // }
}
