#![cfg(test)]

use std::collections::BTreeSet;

use super::*;
use rand::seq::SliceRandom;
use rand::{rngs::StdRng, Rng, SeedableRng};
use syntactic_for::syntactic_for;
// use thousands::Separable;

#[test]
fn complement() {
    syntactic_for! { ty in [i8, u8, i16, u16, i32, u32, i64, u64 ] { // , isize, usize, i128, u128,
        $(
        let empty = RangeSetInt::<$ty>::new();
        let full = !&empty;
        println!("empty: {empty}, full: {full}");
        )*
    }};
}

#[test]
fn repro_bit_and() {
    let a = RangeSetInt::from([1, 2, 3]);
    let b = RangeSetInt::from([2, 3, 4]);

    let result = &a & &b;
    println!("{result}");
    assert_eq!(result, RangeSetInt::from([2, 3]));
}

#[test]
fn repro1() {
    let mut range_set_int = RangeSetInt::<i8>::from("20..22,24..25,25..30");
    range_set_int.internal_add(25, 26);
    println!("{range_set_int}");
    assert!(range_set_int.to_string() == "20..22,24..30");
}

#[test]
fn repro2() {
    let mut range_set_int = RangeSetInt::<i8>::from([-8, 8, -2, -1, 3, 2]);
    range_set_int.internal_add(25, 26);
    println!("{range_set_int}");
    assert!(range_set_int.to_string() == "-8..-7,-2..0,2..4,8..9,25..26");
}

#[test]
fn doctest1() {
    // use rangeset_int::RangeSetInt;

    let a = RangeSetInt::<u64>::from([1, 2, 3]);
    let b = RangeSetInt::<u64>::from([3, 4, 5]);

    let result = &a | &b;
    assert_eq!(result, RangeSetInt::<u64>::from([1, 2, 3, 4, 5]));
}

#[test]
fn doctest2() {
    let set = RangeSetInt::<u64>::from([1, 2, 3]);
    assert!(set.contains(1));
    assert!(!set.contains(4));
}

#[test]
fn doctest3() {
    let mut a = RangeSetInt::<u64>::from("1..4");
    let mut b = RangeSetInt::<u64>::from("3..6");

    a.append(&mut b);

    assert_eq!(a.len(), 5);
    assert_eq!(b.len(), 0);

    assert!(a.contains(1));
    assert!(a.contains(2));
    assert!(a.contains(3));
    assert!(a.contains(4));
    assert!(a.contains(5));
}

#[test]
fn doctest4() {
    let a = RangeSetInt::<i8>::from([1, 2, 3]);

    let result = !&a;
    assert_eq!(result.to_string(), "-128..1,4..127");
}

#[test]
fn compare() {
    let mut btree_set = BTreeSet::<u128>::new();
    btree_set.insert(3);
    btree_set.insert(1);
    let string = btree_set.iter().join(",");
    println!("{string:#?}");
    assert!(string == "1,3");
}

#[test]
fn demo_c1() {
    // before_or_equal_exists	1
    // equal?	0
    // is_included	0
    //     INSERT
    let mut range_set_int = RangeSetInt::<u64>::from("10..11");
    range_set_int.internal_add(12, 13);
    assert!(range_set_int.to_string() == *"10..11,12..13");
    assert!(range_set_int.len_slow() == range_set_int.len());
}

#[test]
fn demo_c2() {
    // before_or_equal_exists	1
    // equal?	0
    // is_included	0
    //     INSERT
    let mut range_set_int = RangeSetInt::<u64>::from("10..11,13..14");
    range_set_int.internal_add(12, 13);
    assert!(range_set_int.to_string() == *"10..11,12..14");
    assert!(range_set_int.len_slow() == range_set_int.len());
}

#[test]
fn demo_f1() {
    // before_or_equal_exists	0
    //     INSERT, etc

    let mut range_set_int = RangeSetInt::<u64>::from("11..16,22..27");
    range_set_int.internal_add(10, 11);
    assert!(range_set_int.to_string() == *"10..16,22..27");
    assert!(range_set_int.len_slow() == range_set_int.len());
}

#[test]
fn demo_d1() {
    // before_or_equal_exists	1
    // equal?	1
    // is_included	n/a
    // fits?	1
    //     DONE

    let mut range_set_int = RangeSetInt::<u64>::from("10..15");
    range_set_int.internal_add(10, 11);
    assert!(range_set_int.to_string() == *"10..15");
    assert!(range_set_int.len_slow() == range_set_int.len());
}

#[test]
fn demo_e1() {
    // before_or_equal_exists	1
    // equal?	1
    // is_included	n/a
    // fits?	0
    // next?    0
    //     DONE

    let mut range_set_int = RangeSetInt::<u64>::from("10..15,16..17");
    range_set_int.internal_add(10, 20);
    assert!(range_set_int.to_string() == *"10..20");
    assert!(range_set_int.len_slow() == range_set_int.len());
}

#[test]
fn demo_b1() {
    // before_or_equal_exists	1
    // equal?	0
    // is_included	1
    // fits?	0
    // next?    0
    //     DONE

    let mut range_set_int = RangeSetInt::<u64>::from("10..15");
    range_set_int.internal_add(12, 18);
    assert!(range_set_int.to_string() == *"10..18");
    assert!(range_set_int.len_slow() == range_set_int.len());
}

#[test]
fn demo_b2() {
    // before_or_equal_exists	1
    // equal?	0
    // is_included	1
    // fits?	0
    // next?    1
    // delete how many? 1
    //     DONE

    let mut range_set_int = RangeSetInt::<u64>::from("10..15,16..17");
    range_set_int.internal_add(12, 18);
    assert!(range_set_int.to_string() == *"10..18");
    assert!(range_set_int.len_slow() == range_set_int.len());
}

#[test]
fn demo_b3() {
    // before_or_equal_exists	1
    // equal?	0
    // is_included	1
    // fits?	0
    // next?    1
    // delete how many? 0
    //     DONE

    let mut range_set_int = RangeSetInt::<u64>::from("10..15,160..161");
    range_set_int.internal_add(12, 18);
    assert!(range_set_int.to_string() == *"10..18,160..161");
    assert!(range_set_int.len_slow() == range_set_int.len());
}

#[test]
fn demo_a() {
    // before_or_equal_exists	1
    // equal?	0
    // is_included	1
    // fits?	1
    //     DONE
    let mut range_set_int = RangeSetInt::<u64>::from("10..15");
    range_set_int.internal_add(12, 13);
    assert!(range_set_int.to_string() == *"10..15");
    assert!(range_set_int.len_slow() == range_set_int.len());
}

// #[test]
// fn test7() {
//     let mut range_set = RangeSetInt::new();
//     let mut index = 0u128;
//     #[allow(clippy::explicit_counter_loop)]
//     for value in RandomData::new(
//         0,
//         // RangeX {
//         //     start: 20,
//         //     length: 10,
//         // },
//         // 1,
//         RangeX {
//             start: 20,
//             length: 1_300_300_010,
//         },
//         100_000,
//     ) {
//         if index % 10_000_000 == 0 {
//             println!(
//                 "index {}, range_count {}",
//                 index.separate_with_commas(),
//                 range_set.items.len().separate_with_commas()
//             );
//         }
//         index += 1;
//         range_set.internal_add(value, value + 1);
//         // println!("{value}->{range_set}");
//     }
//     // println!("{:?}", range_set._items);
// }

// #[test]
// fn test7a() {
//     let mut range_set = RangeSetInt::new();
//     range_set._internal_add(38, 1);
//     range_set._internal_add(39, 1);
//     assert!(range_set.len() == 2);
//     assert!(range_set._items.len() == 1);
//     let first_entry = range_set._items.first_entry().unwrap();
//     assert!(*first_entry.key() == 38);
//     assert!(*first_entry.get() == 2);
// }

// #[test]
// fn test1() {
//     let mut range_set = RangeSetInt::new();
//     assert!(range_set.len() == 0);
//     range_set._internal_add(2, 3);
//     assert!(range_set.len() == 3);
//     assert!(range_set._items.len() == 1);
//     let first_entry = range_set._items.first_entry().unwrap();
//     assert!(*first_entry.key() == 2);
//     assert!(*first_entry.get() == 3);
// }

// #[test]
// fn test1_c2() {
//     let mut range_set = RangeSetInt::new();
//     assert!(range_set.len() == 0);
//     range_set._internal_add(1, 1);
//     range_set._internal_add(1, 4);
//     assert!(range_set.len() == 4);
//     assert!(range_set._items.len() == 1);
//     let first_entry = range_set._items.first_entry().unwrap();
//     assert!(*first_entry.key() == 1);
//     assert!(*first_entry.get() == 4);
// }

// #[test]
// fn test1_c() {
//     let mut range_set = RangeSetInt::new();
//     assert!(range_set.len() == 0);
//     range_set._internal_add(2, 3);
//     range_set._internal_add(1, 1);
//     assert!(range_set.len() == 4);
//     assert!(range_set._items.len() == 1);
//     let first_entry = range_set._items.first_entry().unwrap();
//     assert!(*first_entry.key() == 1);
//     assert!(*first_entry.get() == 4);
// }

// // !!!cmk what if connects with next range(s)?
// #[test]
// fn test2() {
//     let mut range_set = RangeSetInt::new();
//     assert!(range_set.len() == 0);
//     range_set._internal_add(2, 3);
//     assert!(range_set.len() == 3);
//     assert!(range_set._items.len() == 1);
//     let first_entry = range_set._items.first_entry().unwrap();
//     assert!(*first_entry.key() == 2);
//     assert!(*first_entry.get() == 3);
//     range_set._internal_add(2, 1);
//     assert!(range_set.len() == 3);
//     assert!(range_set._items.len() == 1);
//     let first_entry = range_set._items.first_entry().unwrap();
//     assert!(*first_entry.key() == 2);
//     assert!(*first_entry.get() == 3);
//     range_set._internal_add(2, 4);
//     assert!(range_set.len() == 4);
//     assert!(range_set._items.len() == 1);
//     let first_entry = range_set._items.first_entry().unwrap();
//     assert!(*first_entry.key() == 2);
//     assert!(*first_entry.get() == 4);
// }

// !!!cmk bring back in

//#[test]
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

//#[test]
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

//#[test]
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

//#[test]
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

//#[test]
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
//#[test]
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
//#[test]
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
//#[test]
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
//#[test]
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

#[derive(Debug)]
struct RangeX {
    start: u128,
    length: u128,
}

// impl RangeX {
//     fn end(&self) -> u128 {
//         self.start + self.length
//     }
// }

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
