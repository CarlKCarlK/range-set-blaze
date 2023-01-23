// use std::cmp::max;

fn main() {
    test4();
    test5();
    test6();

    test1();
    test2();
    test3();
}

fn test1() {
    let mut range_set = RangeSetInt::new();
    assert!(range_set.len() == 0);
    range_set._internal_add(2, 3);
    assert!(range_set.len() == 3);
    assert!(range_set._items.len() == 1);
    assert!(range_set._items[0].start == 2);
    assert!(range_set._items[0].length == 3);
}

// !!!cmk what if connects with next range(s)?
fn test2() {
    let mut range_set = RangeSetInt::new();
    assert!(range_set.len() == 0);
    range_set._internal_add(2, 3);
    assert!(range_set.len() == 3);
    assert!(range_set._items.len() == 1);
    assert!(range_set._items[0].start == 2);
    assert!(range_set._items[0].length == 3);
    range_set._internal_add(2, 1);
    assert!(range_set.len() == 3);
    assert!(range_set._items.len() == 1);
    assert!(range_set._items[0].start == 2);
    assert!(range_set._items[0].length == 3);
    range_set._internal_add(2, 4);
    assert!(range_set.len() == 4);
    assert!(range_set._items.len() == 1);
    assert!(range_set._items[0].start == 2);
    assert!(range_set._items[0].length == 4);
}

// !!!cmk what if connects with next range(s)?
fn test3() {
    let mut range_set = RangeSetInt::new();
    assert!(range_set.len() == 0);
    range_set._internal_add(2, 3);
    assert!(range_set.len() == 3);
    assert!(range_set._items.len() == 1);
    range_set._internal_add(0, 1);
    assert!(range_set.len() == 4);
    assert!(range_set._items.len() == 2);
    assert!(range_set._items[0].start == 0);
    assert!(range_set._items[0].length == 1);
    assert!(range_set._items[1].start == 2);
    assert!(range_set._items[1].length == 3);
}

fn test4() {
    let mut range_set = RangeSetInt::new();
    assert!(range_set.len() == 0);
    range_set._internal_add(0, 2);
    range_set._internal_add(5, 1);
    assert!(range_set.len() == 3);
    assert!(range_set._items.len() == 2);
    range_set._internal_add(1, 1);
    assert!(range_set.len() == 3);
    assert!(range_set._items.len() == 2);
    assert!(range_set._items[0].start == 0);
    assert!(range_set._items[0].length == 2);
    assert!(range_set._items[1].start == 5);
    assert!(range_set._items[1].length == 1);
}

fn test5() {
    let mut range_set = RangeSetInt::new();
    assert!(range_set.len() == 0);
    range_set._internal_add(0, 2);
    range_set._internal_add(5, 1);
    assert!(range_set.len() == 3);
    assert!(range_set._items.len() == 2);
    range_set._internal_add(1, 2);
    assert!(range_set.len() == 4);
    assert!(range_set._items.len() == 2);
    assert!(range_set._items[0].start == 0);
    assert!(range_set._items[0].length == 3);
    assert!(range_set._items[1].start == 5);
    assert!(range_set._items[1].length == 1);
}

fn test6() {
    let mut range_set = RangeSetInt::new();
    assert!(range_set.len() == 0);
    range_set._internal_add(0, 2);
    range_set._internal_add(5, 1);
    assert!(range_set.len() == 3);
    assert!(range_set._items.len() == 2);
    range_set._internal_add(3, 1);
    assert!(range_set.len() == 4);
    assert!(range_set._items.len() == 3);
    assert!(range_set._items[0].start == 0);
    assert!(range_set._items[0].length == 2);
    assert!(range_set._items[1].start == 3);
    assert!(range_set._items[1].length == 1);
    assert!(range_set._items[2].start == 5);
    assert!(range_set._items[2].length == 1);
}

struct Range {
    start: usize,
    length: usize,
}

struct RangeSetInt {
    _items: Vec<Range>, // !!!cmk usize?
                        // !!!cmk underscore?
}

impl RangeSetInt {
    fn new() -> RangeSetInt {
        RangeSetInt { _items: Vec::new() }
    }

    fn clear(&mut self) {
        self._items.clear();
    }

    fn len(&self) -> usize {
        self._items.iter().fold(0, |acc, x| acc + x.length)
    }

    fn _internal_add(&mut self, start: usize, length: usize) {
        let mut index = self._items.partition_point(|x| x.start < start);
        if index == self._items.len() {
            self._items.push(Range { start, length }); // !!!cmk why copy?
                                                       // !!!cmk what if connects with previous range?
        } else {
            let range: &mut Range = &mut self._items[index];
            let mut previous_start: usize;
            let mut previous_stop: usize;
            if range.start == start {
                if length > range.length {
                    range.length = length;
                    index += 1; // index should point to the following range for the remainder of this method
                    previous_start = start;
                    previous_stop = start + length;
                }
            } else {
                if index == 0 {
                    self._items.insert(index, Range { start, length });
                    previous_start = start;
                    previous_stop = start + length;
                    index += 1 // index_of_miss should point to the following range for the remainder of this method
                } else {
                    let previous_range: &mut Range = &mut self._items[index - 1];
                    previous_start = previous_range.start;
                    let previous_length = previous_range.length;
                    previous_stop = previous_start + previous_range.length;

                    if previous_stop >= start {
                        let new_length = start + length - previous_start;
                        assert!(new_length > 0); // real assert
                        if new_length < previous_length {
                            return;
                        } else {
                            previous_range.length = new_length;
                            previous_stop = previous_start + new_length;
                        }
                    } else {
                        // after previous range, not contiguous with previous range
                        self._items.insert(index, Range { start, length });
                        previous_start = start;
                        previous_stop = start + length;
                        index += 1;
                    }
                }
            }
        }
    }
}

//         } else {
//             let previous_range: &Range = &self._items[index - 1];
//             previous_start = previous_range.start;
//             let previous_length = previous_range.length;
//             stop = previous_start + previous_range.length;

//             if start <= stop {
//                 let new_length = start - previous_start + length;
//                 assert!(new_length > 0); // real assert
//                 if new_length < previous_length {
//                     return;
//                 } else {
//                     previous_range.length = new_length;
//                     stop = previous_start + new_length;
//                 }
//             } else {
//                 // after previous range, not contiguous with previous range
//                 self._items.insert(index, Range { start, length });
//                 previous_start = start;
//                 stop = start + length;
//                 index += 1;
//             }
//         }

//         // collapse next range(s) into this one
//         // use 'drain'
//     //     let mut next: &Range = &self._items[index];
//     //     while stop >= next.start {
//     //         let new_stop = max(stop, next.start + next.length);
//     //         let length = new_stop - previous_start;
//     //         self._start_to_length
//     //             .entry(previous)
//     //             .or_insert(new_stop - previous); // ItemToLength[previous] + ItemToLength[next]
//     //         self._start_to_length.remove(&next);
//     //         self._start_items.remove(index);
//     //         stop = new_stop;
//     //         if index >= self._start_items.len() {
//     //             break;
//     //         }
//     //         next = self._start_items[index];
//     //     }
//     // }
