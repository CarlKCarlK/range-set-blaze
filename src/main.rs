use rand::seq::SliceRandom;
use rand::{rngs::StdRng, Rng, SeedableRng};

fn main() {
    // let rng = StdRng::seed_from_u64(0);

    for value in RandomData::new(
        0,
        Range {
            start: 20,
            length: 31,
        },
    ) {
        println!("{value}");
    }

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

struct RandomData {
    rng: StdRng,
    current: Option<Range>,
    data_range: Vec<Range>,
}

impl RandomData {
    fn new(seed: u64, range: Range) -> Self {
        Self {
            rng: StdRng::seed_from_u64(seed),
            current: None,
            data_range: vec![range],
        }
    }
}

impl Iterator for RandomData {
    type Item = u128;
    fn next(&mut self) -> Option<Self::Item> {
        if let Some(current) = &mut self.current {
            let value = current.start;
            self.current = if current.length == 1 {
                Some(Range {
                    start: current.start + 1,
                    length: current.length - 1,
                })
            } else {
                None
            };
            Some(value)
        } else if self.data_range.is_empty() {
            None
        } else {
            let range = self.data_range.pop().unwrap();
            if range.length < 100 {
                self.current = Some(range);
                self.next()
            } else {
                let split = 5;
                let delete_fraction = 0.1;
                let dup_fraction = 0.01;
                let part_list =
                    _process_this_level(split, range, &mut self.rng, delete_fraction, dup_fraction);
                self.data_range.splice(0..0, part_list);
                self.next()
            }
        }
    }
}

fn _process_this_level(
    split: u128,
    range: Range,
    rng: &mut StdRng,
    delete_fraction: f64,
    dup_fraction: f64,
) -> Vec<Range> {
    let mut part_list = Vec::<Range>::new();
    for i in 0..split {
        let start = i * range.length / split + range.start;
        let end = (i + 1) * range.length / split + range.start;

        if rng.gen::<f64>() < delete_fraction {
            continue;
        }

        part_list.push(Range {
            start,
            length: end - start,
        });

        if rng.gen::<f64>() < dup_fraction {
            part_list.push(Range {
                start,
                length: end - start,
            });
        }
    }
    // shuffle the list
    part_list.shuffle(rng);
    part_list
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

// !!!cmk can I use a Rust range?
// !!!cmk allow negatives and any size
struct Range {
    start: u128,
    length: u128,
}

impl Range {
    fn end(&self) -> u128 {
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

    fn len(&self) -> u128 {
        self._items.iter().fold(0, |acc, x| acc + x.length)
    }

    fn _internal_add(&mut self, start: u128, length: u128) {
        let mut index = self._items.partition_point(|x| x.start < start);
        let mut previous_index;

        if index == self._items.len() {
            self._items.push(Range { start, length }); // !!!cmk why copy?
            previous_index = index;
            index += 1; // index should point to the following range for the remainder of this method
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
            } else if index == 0 {
                self._items.insert(index, Range { start, length });
                previous_index = index;
                index += 1 // index_of_miss should point to the following range for the remainder of this method
            } else {
                previous_index = index - 1;
                let previous_range: &mut Range = &mut self._items[previous_index];

                if previous_range.end() >= start {
                    let new_length = start + length - previous_range.start;
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

        let previous_range: &Range = &self._items[previous_index];
        let previous_end = previous_range.end();
        while index < self._items.len() {
            let range: &Range = &self._items[index];
            if previous_end < range.start {
                break;
            }
            let range_end = range.end();
            if previous_end < range_end {
                self._items[previous_index].length = range_end - previous_range.start;
                index += 1;
                break;
            }
            index += 1;
        }
        self._items.drain(previous_index + 1..index);
    }
}
