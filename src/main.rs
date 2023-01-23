// use std::cmp::max;

use std::cmp::max;

fn main() {
    test1();
    test1_c();
    test2();
    test2_c();
    test2_c2();
    test3();
    test3c();

    test4();
    test5();
    test5_c();
    test6();
    test6_c();
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

fn test1_c() {
    let mut range_set = RangeSetInt::new();
    assert!(range_set.len() == 0);
    range_set._internal_add(2, 3);
    range_set._internal_add(1, 1);
    assert!(range_set.len() == 4);
    assert!(range_set._items.len() == 1);
    assert!(range_set._items[0].start == 1);
    assert!(range_set._items[0].length == 4);
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

fn test2_c() {
    let mut range_set = RangeSetInt::new();
    assert!(range_set.len() == 0);
    range_set._internal_add(2, 1);
    range_set._internal_add(4, 1);
    range_set._internal_add(6, 2);
    assert!(range_set.len() == 4);
    assert!(range_set._items.len() == 3);
    assert!(range_set._items[0].start == 2);
    assert!(range_set._items[0].length == 1);
    assert!(range_set._items[1].start == 4);
    assert!(range_set._items[1].length == 1);
    assert!(range_set._items[2].start == 6);
    assert!(range_set._items[2].length == 2);
    range_set._internal_add(2, 10);
    assert!(range_set.len() == 10);
    assert!(range_set._items.len() == 1);
    assert!(range_set._items[0].start == 2);
    assert!(range_set._items[0].length == 10);
}

fn test2_c2() {
    let mut range_set = RangeSetInt::new();
    assert!(range_set.len() == 0);
    range_set._internal_add(2, 1);
    range_set._internal_add(4, 1);
    range_set._internal_add(6, 20);
    assert!(range_set.len() == 22);
    assert!(range_set._items.len() == 3);
    assert!(range_set._items[0].start == 2);
    assert!(range_set._items[0].length == 1);
    assert!(range_set._items[1].start == 4);
    assert!(range_set._items[1].length == 1);
    assert!(range_set._items[2].start == 6);
    assert!(range_set._items[2].length == 20);
    range_set._internal_add(2, 10);
    assert!(range_set.len() == 24);
    assert!(range_set._items.len() == 1);
    assert!(range_set._items[0].start == 2);
    assert!(range_set._items[0].length == 24);
}

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

fn test3c() {
    let mut range_set = RangeSetInt::new();
    assert!(range_set.len() == 0);
    range_set._internal_add(2, 3);
    assert!(range_set.len() == 3);
    assert!(range_set._items.len() == 1);
    range_set._internal_add(0, 3);
    assert!(range_set.len() == 5);
    assert!(range_set._items.len() == 1);
    assert!(range_set._items[0].start == 0);
    assert!(range_set._items[0].length == 5);
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

fn test5_c() {
    let mut range_set = RangeSetInt::new();
    assert!(range_set.len() == 0);
    range_set._internal_add(0, 2);
    range_set._internal_add(5, 1);
    assert!(range_set.len() == 3);
    assert!(range_set._items.len() == 2);
    range_set._internal_add(1, 10);
    assert!(range_set.len() == 11);
    assert!(range_set._items.len() == 1);
    assert!(range_set._items[0].start == 0);
    assert!(range_set._items[0].length == 11);
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

fn test6_c() {
    let mut range_set = RangeSetInt::new();
    assert!(range_set.len() == 0);
    range_set._internal_add(0, 2);
    range_set._internal_add(5, 1);
    assert!(range_set.len() == 3);
    assert!(range_set._items.len() == 2);
    range_set._internal_add(3, 2);
    assert!(range_set.len() == 5);
    assert!(range_set._items.len() == 2);
    assert!(range_set._items[0].start == 0);
    assert!(range_set._items[0].length == 2);
    assert!(range_set._items[1].start == 3);
    assert!(range_set._items[1].length == 3);
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
        let mut previous_index;

        if index == self._items.len() {
            self._items.push(Range { start, length }); // !!!cmk why copy?
            previous_index = index;
            index += 1; // index should point to the following range for the remainder of this method
                        // !!!cmk what if connects with previous range?
        } else {
            let range: &mut Range = &mut self._items[index];
            if range.start == start {
                if length > range.length {
                    range.length = length;
                    previous_index = index;
                    index += 1; // index should point to the following range for the remainder of this method
                } else {
                    return;
                }
            } else {
                if index == 0 {
                    self._items.insert(index, Range { start, length });
                    previous_index = index;
                    index += 1 // index_of_miss should point to the following range for the remainder of this method
                } else {
                    previous_index = index - 1;
                    let previous_range: &mut Range = &mut self._items[previous_index];
                    let previous_stop = previous_range.start + previous_range.length;

                    if previous_stop >= start {
                        let new_length = start + length - previous_range.start;
                        assert!(new_length > 0); // real assert
                        if new_length <= previous_range.length {
                            return;
                        } else {
                            previous_range.length = new_length;
                        }
                    } else {
                        // after previous range, not contiguous with previous range
                        self._items.insert(index, Range { start, length });
                        previous_index = index;
                        index += 1;
                    }
                }
            }
        }

        let delete_start = index;
        let mut delete_stop = index;
        while index < self._items.len() {
            let previous_range1: &Range = &self._items[previous_index];
            let range: &Range = &self._items[index];
            if previous_range1.start + previous_range1.length < range.start {
                break;
            }
            let new_length = max(
                range.start + range.length - previous_range1.start,
                previous_range1.length,
            );
            assert!(new_length > 0); // real assert
            let previous_range2: &mut Range = &mut self._items[previous_index];
            previous_range2.length = new_length;
            delete_stop += 1;
            index += 1;
        }
        self._items.drain(delete_start..delete_stop);
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
