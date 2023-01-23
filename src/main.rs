// use std::cmp::max;

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

impl Range {
    fn end(&self) -> usize {
        self.start + self.length
    }
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

                    if previous_range.end() >= start {
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

        let previous_end = self._items[previous_index].end();
        while index < self._items.len() {
            let previous_range1: &Range = &self._items[previous_index];
            let range: &Range = &self._items[index];
            if previous_end < range.start {
                break;
            }
            let range_end = range.end();
            if previous_end >= range_end {
                index += 1;
                continue;
            }
            self._items[previous_index].length = range_end - previous_range1.start;
            index += 1;
            break;
        }
        self._items.drain(previous_index + 1..index);
    }
}
