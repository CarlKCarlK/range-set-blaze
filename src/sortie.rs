use num_traits::Zero;
use std::{
    cmp::{max, min},
    collections::BTreeMap,
};

use crate::{Integer, SafeSubtract};

#[derive(Debug)]
pub enum Sortie<T: Integer> {
    None,
    Some {
        sort_list: Vec<(T, T)>,
        lower: T,
        upper: T,
    },
}

impl<T: Integer> Sortie<T> {
    pub fn new() -> Self {
        Self::None
    }
    pub fn insert(&mut self, lower: T, upper: T) {
        let two = T::one() + T::one();
        assert!(lower <= upper && upper <= T::max_value2());
        if let Sortie::Some {
            sort_list,
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
            *self = Sortie::Some {
                sort_list: Vec::new(),
                lower,
                upper,
            };
        }
    }
    // !!!cmk0 better as from_iter?

    pub fn range_int_set(mut self) -> (BTreeMap<T, T>, <T as SafeSubtract>::Output) {
        let mut items = BTreeMap::new();
        let mut len = <T as SafeSubtract>::Output::zero();
        self.extract(&mut items, &mut len);
        (items, len)
    }

    // !!!cmk rename to something better
    pub fn extract(&mut self, items: &mut BTreeMap<T, T>, len: &mut <T as SafeSubtract>::Output) {
        if let Sortie::Some {
            sort_list,
            lower: self_lower,
            upper: self_upper,
        } = self
        {
            sort_list.push((*self_lower, *self_upper));
            sort_list.sort_unstable_by(|a, b| a.0.cmp(&b.0));

            // !!!cmk0 remove this is_empty
            let mut x32 = X32::None;
            for (start, stop) in sort_list {
                match &mut x32 {
                    X32::None => {
                        x32 = X32::Some {
                            start: *start,
                            stop: *stop,
                        };
                    }
                    X32::Some {
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
            if let X32::Some {
                start: current_start,
                stop: current_stop,
            } = x32
            {
                items.insert(current_start, current_stop);
                *len += T::safe_subtract_inclusive(current_stop, current_start);
            }
            *self = Sortie::None;
        }
    }
}

pub enum X32<T: Integer> {
    None,
    Some { start: T, stop: T },
}
