use std::cmp::max;
use std::collections::HashMap;

fn main() {
    println!("Hello, world!");

    let mut range_set_int = RangeSetInt::new();
    range_set_int._internal_add(1, 2);
    println!(
        "{},\n{:?},\n{:?}",
        range_set_int.len(),
        range_set_int._start_items,
        range_set_int._start_to_length
    );
    range_set_int._internal_add(3, 4);
    println!(
        "{},\n{:?},\n{:?}",
        range_set_int.len(),
        range_set_int._start_items,
        range_set_int._start_to_length
    );
    range_set_int._internal_add(1, 3);
    println!(
        "{},\n{:?},\n{:?}",
        range_set_int.len(),
        range_set_int._start_items,
        range_set_int._start_to_length
    );
    range_set_int.clear();
    println!(
        "{},\n{:?},\n{:?}",
        range_set_int.len(),
        range_set_int._start_items,
        range_set_int._start_to_length
    );
}

// RangeSetInt implements Set trait

struct RangeSetInt {
    _start_items: Vec<usize>, // !!!cmk usize?
    _start_to_length: HashMap<usize, usize>, // !!! cmk use more efficient no hash
                              // !!!cmk underscore?
}

impl RangeSetInt {
    fn new() -> RangeSetInt {
        RangeSetInt {
            _start_items: Vec::new(),
            _start_to_length: HashMap::new(),
        }
    }

    fn clear(&mut self) {
        self._start_items.clear();
        self._start_to_length.clear();
    }

    fn len(&self) -> usize {
        self._start_to_length.values().sum()
    }

    fn _internal_add(&mut self, start: usize, length: usize) {
        assert!(self._start_items.len() == self._start_to_length.len()); // !!!cmk real assert
        let mut index = self._start_items.partition_point(|&x| x < start);
        let mut previous: usize;
        let mut stop: usize;
        if index != self._start_items.len() && self._start_items[index] == start {
            if length <= self._start_to_length[&start] {
                return;
            } else {
                self._start_to_length.entry(start).or_insert(length);
                index += 1; // index should point to the following range for the remainder of this method
                previous = start;
                stop = &start + length; // !!!cmk check the types
            }
        } else {
            if index == 0 {
                self._start_items.insert(index, start);
                self._start_to_length.entry(start).or_insert(length);
                previous = start;
                stop = start + length;
                index += 1 // index_of_miss should point to the following range for the remainder of this method
            } else {
                previous = self._start_items[index - 1];
                stop = previous + self._start_to_length[&previous];

                if start <= stop {
                    let new_length = start - previous + length;
                    assert!(new_length > 0); // real assert
                    if new_length < self._start_to_length[&previous] {
                        return;
                    } else {
                        self._start_to_length.entry(previous).or_insert(new_length);
                        stop = previous + new_length;
                    }
                } else {
                    // after previous range, not contiguous with previous range
                    self._start_items.insert(index, start);
                    self._start_to_length.entry(start).or_insert(length);
                    previous = start;
                    stop = start + length;
                    index += 1;
                }
            }
        }

        if index == self._start_items.len() {
            return;
        }

        // collapse next range into this one
        let mut next = self._start_items[index];
        while stop >= next {
            let new_stop = max(stop, next + self._start_to_length[&next]);
            self._start_to_length
                .entry(previous)
                .or_insert(new_stop - previous); // ItemToLength[previous] + ItemToLength[next]
            self._start_to_length.remove(&next);
            self._start_items.remove(index);
            stop = new_stop;
            if index >= self._start_items.len() {
                break;
            }
            next = self._start_items[index];
        }
        return;
    }
}
