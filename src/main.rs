use std::cmp::max;
use std::collections::HashMap;
use std::slice::partition_point;

fn main() {
    println!("Hello, world!");
}

// RangeSetInt implements Set trait

struct RangeSetInt {
    _start_items: Vec<i32>, // !!!cmk i32?
    _start_to_length: HashMap<i32, usize>, // !!! cmk use more efficient no hash
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

    fn _internal_add(&mut self, start: i32, length: usize) {
        assert!(self._start_items.len() == self._start_to_length.len()); // !!!cmk real assert
        let mut index = self._start_items.partition_point(|&x| x < start);
        let mut previous: i32;
        let mut stop: i32;
        if index != self._start_items.len() && self._start_items[index] == start {
            if length <= self._start_to_length[&start]
            {
                return
            }
            else
            {
                self._start_to_length[&start] = length;
                index += 1;     // index should point to the following range for the remainder of this method
                previous = start;
                stop = &start + (length as i32); // !!!cmk check the types
            }
        }
        else
        {
        if index == 0
        {
            self._start_items.insert(index, start);
            self._start_to_length[&start] = length;
            previous = start;
            stop = start + (length as i32);
            index += 1  // index_of_miss should point to the following range for the remainder of this method
        }
        else
        {
            previous = self._start_items[index - 1];
            stop = previous + (self._start_to_length[&previous] as i32);

            if start <= stop
            {
                let new_length = start - previous + (length as i32);
                assert!(new_length > 0); // real assert
                if new_length < (self._start_to_length[&previous] as i32)
                {
                    return
                }
                else
                {
                    self._start_to_length[&previous] = new_length as usize;
                    stop = previous + new_length;
                }
            }
            else // after previous range, not contiguous with previous range
        {
                self._start_items.insert(index, start);
                self._start_to_length[&start] = length;
                previous = start;
                stop = start + (length as i32);
                index += 1;
        }

        }
    }
        
        if index == self._start_items.len()
        {
            return
        }

        // collapse next range into this one
        let next = self._start_items[index];
        while stop >= next
        {
            let new_stop = max(stop, next + (self._start_to_length[&next] as i32);
            self._start_to_length[previous] = new_stop - (previous as i32) ; // ItemToLength[previous] + ItemToLength[next]
            self._start_to_length.remove(&next);
            self._start_items.remove(index);
            stop = new_stop;
            if index >= self._start_items.len())
            {
                break
            }
            next = self._start_items[index];
        }
        return
    }
}
