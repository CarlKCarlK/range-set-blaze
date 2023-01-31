use rand::seq::SliceRandom;
use rand::{rngs::StdRng, Rng, SeedableRng};
use std::cmp::max;
use std::collections::BTreeMap;
use thousands::Separable;

fn main() {
    test_demo_c1();
    test_demo_c2();
    test_demo_a();
    test_demo_b3();
    test_demo_b1();
    test_demo_b2();
    test_demo_d1();
    test_demo_d2();

    // test_demo();
    // test1_c2();
    // test1_c();
    // test1();
    // test2();
    // test7a();
    // test7();

    // cmk bring back in
    // test2_c();
    // test2_c2();
    // test3();
    // test3c();

    // test4();
    // test5();
    // test5_c();
    // test6();
    // test6_c();
}

fn test_demo_c1() {
    // before_or_equal_exists	1
    // equal?	0
    // is_included	0
    //     INSERT
    let (start, length) = (12u128, 1u128);
    let end = start + length;
    let mut items = BTreeMap::<u128, u128>::new();
    items.insert(10, 1);
    println!("{:?}", items);
    b_d_cmk(&mut items, start, end);
    println!("{:?}", items);
    assert!(items.len() == 2);
    let first_entry = items.first_entry().unwrap();
    assert!(*first_entry.key() == 10);
    assert!(*first_entry.get() == 1);
    let entry = items.iter().nth(1).unwrap();
    assert!(*entry.0 == 12);
    assert!(*entry.1 == 1);
}

fn test_demo_c2() {
    // before_or_equal_exists	1
    // equal?	0
    // is_included	0
    //     INSERT
    let (start, length) = (12u128, 1u128);
    let end = start + length;
    let mut items = BTreeMap::<u128, u128>::new();
    items.insert(10, 1);
    items.insert(13, 1);
    println!("{:?}", items);
    b_d_cmk(&mut items, start, end);
    println!("{:?}", items);
    assert!(items.len() == 2);
    let first_entry = items.first_entry().unwrap();
    assert!(*first_entry.key() == 10);
    assert!(*first_entry.get() == 1);
    let entry = items.iter().nth(1).unwrap();
    assert!(*entry.0 == 12);
    let e1 = *entry.1;
    assert!(e1 == 2);
}
fn test_demo_d1() {
    // before_or_equal_exists	1
    // equal?	1
    // is_included	n/a
    // fits?	1
    //     DONE
    let (start, length) = (10u128, 1u128);
    let end = start + length;
    let mut items = BTreeMap::<u128, u128>::new();
    items.insert(10, 5);
    println!("{:?}", items);
    b_d_cmk(&mut items, start, end);

    println!("{:?}", items);
    assert!(items.len() == 1);
    let first_entry = items.first_entry().unwrap();
    assert!(*first_entry.key() == 10);
    assert!(*first_entry.get() == 5);
}

fn test_demo_d2() {
    // before_or_equal_exists	1
    // equal?	1
    // is_included	n/a
    // fits?	0
    // next?    0
    //     DONE
    let (start, length) = (10u128, 10u128);
    let end = start + length;
    let mut items = BTreeMap::<u128, u128>::new();
    items.insert(10, 5);
    items.insert(16, 1);
    println!("{:?}", items);
    b_d_cmk(&mut items, start, end);
    // println!("after {:?}", after);
    // let next = range.next();
    // println!("next {:?}", next);
    // // assert!(next.is_some());
    println!("{:?}", items);

    assert!(items.len() == 1);
    let first_entry = items.first_entry().unwrap();
    assert!(*first_entry.key() == 10);
    assert!(*first_entry.get() == 10);
}

fn b_d_cmk(items: &mut BTreeMap<u128, u128>, start: u128, end: u128) {
    assert!(start < end); // !!!cmk check that length is not zero
                          // !!! cmk would be nice to have a partition_point function that returns two iterators
    let mut before = items.range_mut(..=start);
    // println!("before {:?}", before.collect::<Vec<_>>());
    let (start_x, length_x) = before.next().unwrap();
    let start_x2 = *start_x;
    println!("start_x {:?}, length_x {:?}", start_x, length_x);
    let end_x: u128 = start_x + *length_x;
    if end_x < start {
        let was_there = items.insert(start, end - start);
        assert!(was_there.is_none()); // !!!cmk real assert
        delete_extra(items, start, end);
    } else if end > end_x {
        *length_x = end - start_x;
        delete_extra(items, start_x2, end);
    } else {
        // do nothing
    }
}

fn test_demo_b1() {
    // before_or_equal_exists	1
    // equal?	0
    // is_included	1
    // fits?	0
    // next?    0
    //     DONE
    let (start, length) = (12u128, 6u128);
    let end = start + length;
    let mut items = BTreeMap::<u128, u128>::new();
    items.insert(10, 5);
    println!("{:?}", items);

    b_d_cmk(&mut items, start, end);

    println!("{:?}", items);
    assert!(items.len() == 1);
    let first_entry = items.first_entry().unwrap();
    assert!(*first_entry.key() == 10);
    assert!(*first_entry.get() == 8);
}

fn test_demo_b2() {
    // before_or_equal_exists	1
    // equal?	0
    // is_included	1
    // fits?	0
    // next?    1
    // delete how many? 1
    //     DONE
    let (start, length) = (12u128, 6u128);
    let end = start + length;
    let mut items = BTreeMap::<u128, u128>::new();
    items.insert(10, 5);
    items.insert(16, 1);
    println!("{:?}", items);
    // !!! cmk would be nice to have a partition_point function that returns two iterators
    b_d_cmk(&mut items, start, end);
    // println!("after {:?}", after);
    // let next = range.next();
    // println!("next {:?}", next);
    // // assert!(next.is_some());
    println!("{:?}", items);
    assert!(items.len() == 1);
    let first_entry = items.first_entry().unwrap();
    assert!(*first_entry.key() == 10);
    assert!(*first_entry.get() == 8);
}

fn test_demo_b3() {
    // before_or_equal_exists	1
    // equal?	0
    // is_included	1
    // fits?	0
    // next?    1
    // delete how many? 0
    //     DONE
    let (start, length) = (12u128, 6u128);
    let end = start + length;
    let mut items = BTreeMap::<u128, u128>::new();
    items.insert(10, 5);
    items.insert(160, 1);
    println!("{:?}", items);
    // !!! cmk would be nice to have a partition_point function that returns two iterators
    b_d_cmk(&mut items, start, end);
    // println!("after {:?}", after);
    // let next = range.next();
    // println!("next {:?}", next);
    // // assert!(next.is_some());
    println!("{:?}", items);
    assert!(items.len() == 2);
    let first_entry = items.first_entry().unwrap();
    assert!(*first_entry.key() == 10);
    assert!(*first_entry.get() == 8);
    let entry = items.iter().nth(1).unwrap();
    assert!(*entry.0 == 160);
    assert!(*entry.1 == 1);
}
fn delete_extra(items: &mut BTreeMap<u128, u128>, start: u128, end: u128) {
    let mut after = items.range_mut(start..);
    let (start2, length2) = after.next().unwrap(); // !!! cmk assert that there is a next
    assert!(start == *start2 && start2 + *length2 == end); // !!! cmk real assert
                                                           // !!!cmk would be nice to have a delete_range function
    let mut new_end = end;
    let delete_list = after
        .map_while(|(start3, length3)| {
            if *start3 <= end {
                new_end = max(new_end, start3 + *length3);
                Some(*start3)
            } else {
                None
            }
        })
        .collect::<Vec<_>>();
    println!("delete_list {:?}", delete_list);
    if new_end > end {
        *length2 = new_end - start;
    }
    for start in delete_list {
        items.remove(&start);
    }
}

fn test_demo_a() {
    // before_or_equal_exists	1
    // equal?	0
    // is_included	1
    // fits?	1
    //     DONE
    let (start, length) = (12u128, 1u128);
    let end = start + length;
    let mut items = BTreeMap::<u128, u128>::new();
    items.insert(10, 5);
    println!("{:?}", items);
    b_d_cmk(&mut items, start, end);
    println!("{:?}", items);
    assert!(items.len() == 1);
    let first_entry = items.first_entry().unwrap();
    assert!(*first_entry.key() == 10);
    assert!(*first_entry.get() == 5);
}

fn test_demo() {
    let mut items = BTreeMap::<u128, u128>::new();
    println!("{:?}", items);
    let range = items.range(0..);
    println!("0.. {:?}", range);
    items.insert(10, 200);
    items.insert(11, 200);
    println!("{:?}", items);
    let range = items.range(..=0).rev();
    println!("0 {:?}", range);
    let range = items.range(..=10).rev();
    println!("10 {:?}", range);
    let range = items.range(..=20).rev();
    println!("20 {:?}", range);

    let mut range = items.range_mut(..=10).peekable();
    let peek = range.peek();
    println!("10.. peek {:?}", peek);

    let (_, value) = range.next().unwrap();
    *value = 201;

    // if let Some(peek) = peek {
    //     let peek = *peek;
    //     if *peek.0 == 10 {
    //         let (_, value) = range.next().unwrap();
    //         *value = 201;
    //         println!("{:?}", items);
    //     }
    // }
    println!("{:?}", items);
}

fn test7() {
    let mut range_set = RangeSetInt::new();
    let mut index = 0u128;
    #[allow(clippy::explicit_counter_loop)]
    for value in RandomData::new(
        0,
        RangeX {
            start: 20,
            length: 1_300_300_010,
        },
        1_000_000,
    ) {
        if index % 100_000_000 == 0 {
            println!(
                "index {}, range_count {}",
                index.separate_with_commas(),
                range_set._items.len()
            );
        }
        index += 1;
        range_set._internal_add(value, 1);
        // println!("{value} {:?}", range_set._items);
    }
    // println!("{:?}", range_set._items);
}

struct RandomData {
    rng: StdRng,
    current: Option<RangeX>,
    data_range: Vec<RangeX>,
    small_enough: u128,
}

impl RandomData {
    fn new(seed: u64, range: RangeX, small_enough: u128) -> Self {
        Self {
            rng: StdRng::seed_from_u64(seed),
            current: None,
            data_range: vec![range],
            small_enough,
        }
    }
}

impl Iterator for RandomData {
    type Item = u128;
    fn next(&mut self) -> Option<Self::Item> {
        if let Some(current) = &mut self.current {
            let value = current.start;
            self.current = if current.length > 1 {
                Some(RangeX {
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
            if range.length <= self.small_enough {
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
    range: RangeX,
    rng: &mut StdRng,
    delete_fraction: f64,
    dup_fraction: f64,
) -> Vec<RangeX> {
    let mut part_list = Vec::<RangeX>::new();
    for i in 0..split {
        let start = i * range.length / split + range.start;
        let end = (i + 1) * range.length / split + range.start;

        if rng.gen::<f64>() < delete_fraction {
            continue;
        }

        part_list.push(RangeX {
            start,
            length: end - start,
        });

        if rng.gen::<f64>() < dup_fraction {
            part_list.push(RangeX {
                start,
                length: end - start,
            });
        }
    }
    // shuffle the list
    part_list.shuffle(rng);
    part_list
}

fn test7a() {
    let mut range_set = RangeSetInt::new();
    range_set._internal_add(38, 1);
    range_set._internal_add(39, 1);
    assert!(range_set.len() == 2);
    assert!(range_set._items.len() == 1);
    let first_entry = range_set._items.first_entry().unwrap();
    assert!(*first_entry.key() == 38);
    assert!(*first_entry.get() == 2);
}
fn test1() {
    let mut range_set = RangeSetInt::new();
    assert!(range_set.len() == 0);
    range_set._internal_add(2, 3);
    assert!(range_set.len() == 3);
    assert!(range_set._items.len() == 1);
    let first_entry = range_set._items.first_entry().unwrap();
    assert!(*first_entry.key() == 2);
    assert!(*first_entry.get() == 3);
}

fn test1_c2() {
    let mut range_set = RangeSetInt::new();
    assert!(range_set.len() == 0);
    range_set._internal_add(1, 1);
    range_set._internal_add(1, 4);
    assert!(range_set.len() == 4);
    assert!(range_set._items.len() == 1);
    let first_entry = range_set._items.first_entry().unwrap();
    assert!(*first_entry.key() == 1);
    assert!(*first_entry.get() == 4);
}

fn test1_c() {
    let mut range_set = RangeSetInt::new();
    assert!(range_set.len() == 0);
    range_set._internal_add(2, 3);
    range_set._internal_add(1, 1);
    assert!(range_set.len() == 4);
    assert!(range_set._items.len() == 1);
    let first_entry = range_set._items.first_entry().unwrap();
    assert!(*first_entry.key() == 1);
    assert!(*first_entry.get() == 4);
}

// !!!cmk what if connects with next range(s)?
fn test2() {
    let mut range_set = RangeSetInt::new();
    assert!(range_set.len() == 0);
    range_set._internal_add(2, 3);
    assert!(range_set.len() == 3);
    assert!(range_set._items.len() == 1);
    let first_entry = range_set._items.first_entry().unwrap();
    assert!(*first_entry.key() == 2);
    assert!(*first_entry.get() == 3);
    range_set._internal_add(2, 1);
    assert!(range_set.len() == 3);
    assert!(range_set._items.len() == 1);
    let first_entry = range_set._items.first_entry().unwrap();
    assert!(*first_entry.key() == 2);
    assert!(*first_entry.get() == 3);
    range_set._internal_add(2, 4);
    assert!(range_set.len() == 4);
    assert!(range_set._items.len() == 1);
    let first_entry = range_set._items.first_entry().unwrap();
    assert!(*first_entry.key() == 2);
    assert!(*first_entry.get() == 4);
}

// !!!cmk bring back in

// fn test2_c() {
//     let mut range_set = RangeSetInt::new();
//     assert!(range_set.len() == 0);
//     range_set._internal_add(2, 1);
//     range_set._internal_add(4, 1);
//     range_set._internal_add(6, 2);
//     assert!(range_set.len() == 4);
//     assert!(range_set._items.len() == 3);
//     assert!(range_set._items[0].start == 2);
//     assert!(range_set._items[0].length == 1);
//     assert!(range_set._items[1].start == 4);
//     assert!(range_set._items[1].length == 1);
//     assert!(range_set._items[2].start == 6);
//     assert!(range_set._items[2].length == 2);
//     range_set._internal_add(2, 10);
//     assert!(range_set.len() == 10);
//     assert!(range_set._items.len() == 1);
//     assert!(range_set._items[0].start == 2);
//     assert!(range_set._items[0].length == 10);
// }

// fn test2_c2() {
//     let mut range_set = RangeSetInt::new();
//     assert!(range_set.len() == 0);
//     range_set._internal_add(2, 1);
//     range_set._internal_add(4, 1);
//     range_set._internal_add(6, 20);
//     assert!(range_set.len() == 22);
//     assert!(range_set._items.len() == 3);
//     assert!(range_set._items[0].start == 2);
//     assert!(range_set._items[0].length == 1);
//     assert!(range_set._items[1].start == 4);
//     assert!(range_set._items[1].length == 1);
//     assert!(range_set._items[2].start == 6);
//     assert!(range_set._items[2].length == 20);
//     range_set._internal_add(2, 10);
//     assert!(range_set.len() == 24);
//     assert!(range_set._items.len() == 1);
//     assert!(range_set._items[0].start == 2);
//     assert!(range_set._items[0].length == 24);
// }

// fn test3() {
//     let mut range_set = RangeSetInt::new();
//     assert!(range_set.len() == 0);
//     range_set._internal_add(2, 3);
//     assert!(range_set.len() == 3);
//     assert!(range_set._items.len() == 1);
//     range_set._internal_add(0, 1);
//     assert!(range_set.len() == 4);
//     assert!(range_set._items.len() == 2);
//     assert!(range_set._items[0].start == 0);
//     assert!(range_set._items[0].length == 1);
//     assert!(range_set._items[1].start == 2);
//     assert!(range_set._items[1].length == 3);
// }

// fn test3c() {
//     let mut range_set = RangeSetInt::new();
//     assert!(range_set.len() == 0);
//     range_set._internal_add(2, 3);
//     assert!(range_set.len() == 3);
//     assert!(range_set._items.len() == 1);
//     range_set._internal_add(0, 3);
//     assert!(range_set.len() == 5);
//     assert!(range_set._items.len() == 1);
//     assert!(range_set._items[0].start == 0);
//     assert!(range_set._items[0].length == 5);
// }

// fn test4() {
//     let mut range_set = RangeSetInt::new();
//     assert!(range_set.len() == 0);
//     range_set._internal_add(0, 2);
//     range_set._internal_add(5, 1);
//     assert!(range_set.len() == 3);
//     assert!(range_set._items.len() == 2);
//     range_set._internal_add(1, 1);
//     assert!(range_set.len() == 3);
//     assert!(range_set._items.len() == 2);
//     assert!(range_set._items[0].start == 0);
//     assert!(range_set._items[0].length == 2);
//     assert!(range_set._items[1].start == 5);
//     assert!(range_set._items[1].length == 1);
// }

// fn test5() {
//     let mut range_set = RangeSetInt::new();
//     assert!(range_set.len() == 0);
//     range_set._internal_add(0, 2);
//     range_set._internal_add(5, 1);
//     assert!(range_set.len() == 3);
//     assert!(range_set._items.len() == 2);
//     range_set._internal_add(1, 2);
//     assert!(range_set.len() == 4);
//     assert!(range_set._items.len() == 2);
//     assert!(range_set._items[0].start == 0);
//     assert!(range_set._items[0].length == 3);
//     assert!(range_set._items[1].start == 5);
//     assert!(range_set._items[1].length == 1);
// }

// fn test5_c() {
//     let mut range_set = RangeSetInt::new();
//     assert!(range_set.len() == 0);
//     range_set._internal_add(0, 2);
//     range_set._internal_add(5, 1);
//     assert!(range_set.len() == 3);
//     assert!(range_set._items.len() == 2);
//     range_set._internal_add(1, 10);
//     assert!(range_set.len() == 11);
//     assert!(range_set._items.len() == 1);
//     assert!(range_set._items[0].start == 0);
//     assert!(range_set._items[0].length == 11);
// }

// fn test6() {
//     let mut range_set = RangeSetInt::new();
//     assert!(range_set.len() == 0);
//     range_set._internal_add(0, 2);
//     range_set._internal_add(5, 1);
//     assert!(range_set.len() == 3);
//     assert!(range_set._items.len() == 2);
//     range_set._internal_add(3, 1);
//     assert!(range_set.len() == 4);
//     assert!(range_set._items.len() == 3);
//     assert!(range_set._items[0].start == 0);
//     assert!(range_set._items[0].length == 2);
//     assert!(range_set._items[1].start == 3);
//     assert!(range_set._items[1].length == 1);
//     assert!(range_set._items[2].start == 5);
//     assert!(range_set._items[2].length == 1);
// }

// fn test6_c() {
//     let mut range_set = RangeSetInt::new();
//     assert!(range_set.len() == 0);
//     range_set._internal_add(0, 2);
//     range_set._internal_add(5, 1);
//     assert!(range_set.len() == 3);
//     assert!(range_set._items.len() == 2);
//     range_set._internal_add(3, 2);
//     assert!(range_set.len() == 5);
//     assert!(range_set._items.len() == 2);
//     assert!(range_set._items[0].start == 0);
//     assert!(range_set._items[0].length == 2);
//     assert!(range_set._items[1].start == 3);
//     assert!(range_set._items[1].length == 3);
// }

// !!!cmk can I use a Rust range?
// !!!cmk allow negatives and any size
#[derive(Debug)]
struct RangeX {
    start: u128,
    length: u128,
}

impl RangeX {
    fn end(&self) -> u128 {
        self.start + self.length
    }
}

struct RangeSetInt {
    _items: BTreeMap<u128, u128>, // !!!cmk usize?
                                  // !!!cmk underscore?
}

impl RangeSetInt {
    fn new() -> RangeSetInt {
        RangeSetInt {
            _items: BTreeMap::new(),
        }
    }

    fn clear(&mut self) {
        self._items.clear();
    }

    // !!!cmk keep this in a field
    fn len(&self) -> u128 {
        self._items.values().fold(0, |acc, length| acc + length)
    }

    fn _internal_add(&mut self, start: u128, length: u128) {
        // !!!cmk put this shortcut back?
        // if self._items.len() == 0 {
        //     self._items.insert(start, length);
        //     return;
        // }

        // https://stackoverflow.com/questions/49599833/how-to-find-next-smaller-key-in-btreemap-btreeset
        // https://stackoverflow.com/questions/35663342/how-to-modify-partially-remove-a-range-from-a-btreemap
        // !!!cmk rename index to "range"
        let range = self._items.range(..start);
        let mut peekable_forward = range.clone().peekable();
        let peek_forward = peekable_forward.peek();
        let mut peekable_backwards = range.rev().peekable();
        let peek_backwards = peekable_backwards.peek();
        if let Some(peek_forward) = peek_forward {
            let mut peek_forward = *peek_forward;
            if *peek_forward.0 == start {
                if length > *peek_forward.1 {
                    peek_forward.1 = &length;
                    // previous_range = peek_forward;
                    // peek_forward = peekable_forward.next(); // index should point to the following range for the remainder of this method
                    todo!()
                } else {
                    todo!();
                }
            }
        } else {
            println!("self._items.insert(start, length);");
            if let Some(previous_range) = peek_backwards {
                // nothing
            } else {
                return;
            }
        }

        todo!();
        //             return;
        //         }
        //     } else if index == 0 {
        //         self._items.insert(index, RangeX { start, length });
        //         previous_index = index;
        //         index += 1 // index_of_miss should point to the following range for the remainder of this method
        //     } else {
        //         previous_index = index - 1;
        //         let previous_range: &mut RangeX = &mut self._items[previous_index];

        //         if previous_range.end() >= start {
        //             let new_length = start + length - previous_range.start;
        //             if new_length <= previous_range.length {
        //                 return;
        //             } else {
        //                 previous_range.length = new_length;
        //             }
        //         } else {
        //             // after previous range, not contiguous with previous range
        //             self._items.insert(index, RangeX { start, length });
        //             previous_index = index;
        //             index += 1;
        //         }
        //     }
        // }

        // let previous_range: &RangeX = &self._items[previous_index];
        // let previous_end = previous_range.end();
        // while index < self._items.len() {
        //     let range: &RangeX = &self._items[index];
        //     if previous_end < range.start {
        //         break;
        //     }
        //     let range_end = range.end();
        //     if previous_end < range_end {
        //         self._items[previous_index].length = range_end - previous_range.start;
        //         index += 1;
        //         break;
        //     }
        //     index += 1;
        // }
        // self._items.drain(previous_index + 1..index);
    }
}
