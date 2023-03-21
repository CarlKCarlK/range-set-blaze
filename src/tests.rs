#![cfg(test)]

// !!!cmk rule: Use the same testing as with macros to check that the types are correct
// !!!cmk rule make illegal states unpresentable (example u8.len->usize, but u128 needs safe_max_value), UnionIter
// !!!cmk rule detail: Note that this nicely fails to compile if you try to chain when you shouldn't
// !!!cmk rule detail:  let chain = b.ranges().chain(c.ranges());
// !!!cmk rule detail:  let a_less = a.ranges().sub(chain);
// !!!cmk rule test near extreme values
// !!!cmk test it across threads
use std::{
    any::Any,
    collections::{hash_map::DefaultHasher, BTreeSet},
    fmt::Display,
    hash::Hash,
    iter::FusedIterator,
    ops::BitOr,
    panic::{RefUnwindSafe, UnwindSafe},
}; // , time::Instant

use super::*;
use itertools::Itertools;
use rand::{rngs::StdRng, SeedableRng};
// use sorted_iter::assume::AssumeSortedByKeyExt;
// use rand::{rngs::StdRng, seq::SliceRandom, SeedableRng};
use syntactic_for::syntactic_for;
use tests_common::{How, MemorylessIter, MemorylessRange};
// use thousands::Separable;
use std::ops::BitAndAssign;

#[test]
fn insert_255u8() {
    let range_set_int = RangeSetInt::<u8>::from_iter([255]);
    assert!(range_set_int.to_string() == "255..=255");
}

#[test]
#[should_panic]
fn insert_max_u128() {
    let a = RangeSetInt::<u128>::from_iter([u128::MAX]);
    println!("a: {a}");
}

#[test]
fn sub() {
    for start in i8::MIN..i8::MAX {
        for end in start..i8::MAX {
            let diff = i8::safe_len(&(start..=end));
            let diff2 = (end as i16) - (start as i16) + 1;
            assert_eq!(diff as i16, diff2);
        }
    }
    for start in u8::MIN..u8::MAX {
        for end in start..u8::MAX {
            let diff = u8::safe_len(&(start..=end));
            let diff2 = (end as i16) - (start as i16) + 1;
            assert_eq!(diff as i16, diff2);
        }
    }

    let before = 127i8.overflowing_sub(-128i8).0;
    let after = before as u8;
    println!("before: {before}, after: {after}");
}

#[test]
fn complement0() {
    syntactic_for! { ty in [i8, u8, isize, usize,  i16, u16, i32, u32, i64, u64, isize, usize, i128, u128] {
        $(
        let empty = RangeSetInt::<$ty>::new();
        let full = !&empty;
        println!("empty: {empty} (len {}), full: {full} (len {})", empty.len(), full.len());
        )*
    }};
}

#[test]
fn repro_bit_and() {
    let a = RangeSetInt::from_iter([1u8, 2, 3]);
    let b = RangeSetInt::from_iter([2u8, 3, 4]);

    let result = &a & &b;
    println!("{result}");
    assert_eq!(result, RangeSetInt::from_iter([2u8, 3]));
}

#[test]
fn repro1() {
    let mut range_set_int = RangeSetInt::from_iter([20..=21, 24..=24, 25..=29]);
    println!("{range_set_int}");
    assert!(range_set_int.to_string() == "20..=21, 24..=29");
    range_set_int.internal_add(25..=25);
    println!("{range_set_int}");
    assert!(range_set_int.to_string() == "20..=21, 24..=29");
}

#[test]
fn repro2() {
    let mut range_set_int = RangeSetInt::<i8>::from_iter([-8, 8, -2, -1, 3, 2]);
    range_set_int.internal_add(25..=25);
    println!("{range_set_int}");
    assert!(range_set_int.to_string() == "-8..=-8, -2..=-1, 2..=3, 8..=8, 25..=25");
}

#[test]
fn doctest1() {
    let a = RangeSetInt::<u8>::from_iter([1, 2, 3]);
    let b = RangeSetInt::<u8>::from_iter([3, 4, 5]);

    let result = &a | &b;
    assert_eq!(result, RangeSetInt::<u8>::from_iter([1, 2, 3, 4, 5]));
}

#[test]
fn doctest2() {
    let set = RangeSetInt::<u8>::from_iter([1, 2, 3]);
    assert!(set.contains(1));
    assert!(!set.contains(4));
}

#[test]
fn doctest3() {
    let mut a = RangeSetInt::from_iter([1..=3]);
    let mut b = RangeSetInt::from_iter([3..=5]);

    a.append(&mut b);

    assert_eq!(a.len(), 5usize);
    assert_eq!(b.len(), 0usize);

    assert!(a.contains(1));
    assert!(a.contains(2));
    assert!(a.contains(3));
    assert!(a.contains(4));
    assert!(a.contains(5));
}

#[test]
fn doctest4() {
    let a = RangeSetInt::<i8>::from_iter([1, 2, 3]);

    let result = !&a;
    assert_eq!(result.to_string(), "-128..=0, 4..=127");
}

#[test]
fn compare() {
    let mut btree_set = BTreeSet::<u128>::new();
    btree_set.insert(3);
    btree_set.insert(1);
    let string = btree_set.iter().join(", ");
    println!("{string:#?}");
    assert!(string == "1, 3");
}

#[test]
fn demo_c1() {
    // before_or_equal_exists	1
    // equal?	0
    // is_included	0
    //     INSERT
    let mut range_set_int = RangeSetInt::from_iter([10..=10]);
    range_set_int.internal_add(12..=12);
    assert!(range_set_int.to_string() == "10..=10, 12..=12");
    assert!(range_set_int._len_slow() == range_set_int.len());
}

#[test]
fn demo_c2() {
    // before_or_equal_exists	1
    // equal?	0
    // is_included	0
    //     INSERT
    let mut range_set_int = RangeSetInt::from_iter([10..=10, 13..=13]);
    range_set_int.internal_add(12..=12);
    assert!(range_set_int.to_string() == "10..=10, 12..=13");
    assert!(range_set_int._len_slow() == range_set_int.len());
}

#[test]
fn demo_f1() {
    // before_or_equal_exists	0
    //     INSERT, etc

    let mut range_set_int = RangeSetInt::from_iter([11..=14, 22..=26]);
    range_set_int.internal_add(10..=10);
    assert!(range_set_int.to_string() == "10..=14, 22..=26");
    println!(
        "demo_1 range_set_int = {:?}, _len_slow = {}, len = {}",
        range_set_int,
        range_set_int._len_slow(),
        range_set_int.len()
    );

    assert!(range_set_int._len_slow() == range_set_int.len());
}

#[test]
fn demo_d1() {
    // before_or_equal_exists	1
    // equal?	1
    // is_included	n/a
    // fits?	1
    //     DONE

    let mut range_set_int = RangeSetInt::from_iter([10..=14]);
    range_set_int.internal_add(10..=10);
    assert!(range_set_int.to_string() == "10..=14");
    assert!(range_set_int._len_slow() == range_set_int.len());
}

#[test]
fn demo_e1() {
    // before_or_equal_exists	1
    // equal?	1
    // is_included	n/a
    // fits?	0
    // next?    0
    //     DONE

    let mut range_set_int = RangeSetInt::from_iter([10..=14, 16..=16]);
    range_set_int.internal_add(10..=19);
    assert!(range_set_int.to_string() == "10..=19");
    assert!(range_set_int._len_slow() == range_set_int.len());
}

#[test]
fn demo_b1() {
    // before_or_equal_exists	1
    // equal?	0
    // is_included	1
    // fits?	0
    // next?    0
    //     DONE

    let mut range_set_int = RangeSetInt::from_iter([10..=14]);
    range_set_int.internal_add(12..=17);
    assert!(range_set_int.to_string() == "10..=17");
    assert!(range_set_int._len_slow() == range_set_int.len());
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

    let mut range_set_int = RangeSetInt::from_iter([10..=14, 16..=16]);
    range_set_int.internal_add(12..=17);
    assert!(range_set_int.to_string() == "10..=17");
    assert!(range_set_int._len_slow() == range_set_int.len());
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

    let mut range_set_int = RangeSetInt::from_iter([10..=15, 160..=160]);
    range_set_int.internal_add(12..=17);
    assert!(range_set_int.to_string() == "10..=17, 160..=160");
    assert!(range_set_int._len_slow() == range_set_int.len());
}

#[test]
fn demo_a() {
    // before_or_equal_exists	1
    // equal?	0
    // is_included	1
    // fits?	1
    //     DONE
    let mut range_set_int = RangeSetInt::from_iter([10..=14]);
    range_set_int.internal_add(12..=12);
    assert!(range_set_int.to_string() == "10..=14");
    println!(
        "demo_a range_set_int = {:?}, _len_slow = {}, len = {}",
        range_set_int,
        range_set_int._len_slow(),
        range_set_int.len()
    );
    assert!(range_set_int._len_slow() == range_set_int.len());
}

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

#[test]
fn add_in_order() {
    let mut range_set = RangeSetInt::new();
    for i in 0u64..1000 {
        range_set.insert(i);
    }
}

// #[test]
// fn memoryless_data() {
//     let len = 100_000_000;
//     let coverage_goal = 0.75;
//     let memoryless_data = MemorylessData::new(0, 10_000_000, len, coverage_goal);
//     let range_set_int = RangeSetInt::from_iter(memoryless_data);
//     let coverage = range_set_int.len() as f64 / len as f64;
//     println!(
//         "coverage {coverage:?} range_len {:?}",
//         range_set_int.range_len().separate_with_commas()
//     );
// }

// #[test]
// fn memoryless_vec() {
//     let len = 100_000_000;
//     let coverage_goal = 0.75;
//     let memoryless_data = MemorylessData::new(0, 10_000_000, len, coverage_goal);
//     let data_as_vec: Vec<u64> = memoryless_data.collect();
//     let start = Instant::now();
//     // let range_set_int = RangeSetInt::from_mut_slice(data_as_vec.as_mut_slice());
//     let range_set_int = RangeSetInt::from_iter(data_as_vec);
//     let coverage = range_set_int.len() as f64 / len as f64;
//     println!(
//         "coverage {coverage:?} range_len {:?}",
//         range_set_int.range_len().separate_with_commas()
//     );
//     println!(
//         "xTime elapsed in expensive_function() is: {} ms",
//         start.elapsed().as_millis()
//     );
// }

#[test]
fn optimize() {
    let end = 8u8;
    for a in 0..=end {
        for b in 0..=end {
            for c in 0..=end {
                for d in 0..=end {
                    let restart = (a >= 2 && a - 2 >= d) || (c >= 2 && c - 2 >= b);
                    print!("{a}\t{b}\t{c}\t{d}\t");
                    if a > b {
                        println!("impossible");
                    } else if c > d {
                        println!("error");
                    } else {
                        let mut range_set_int = RangeSetInt::new();
                        range_set_int.internal_add(a..=b);
                        range_set_int.internal_add(c..=d);
                        if range_set_int.ranges_len() == 1 {
                            let vec = range_set_int.into_iter().collect::<Vec<u8>>();
                            println! {"combine\t{}\t{}", vec[0], vec[vec.len()-1]};
                            assert!(!restart);
                        } else {
                            println!("restart");
                            assert!(restart);
                        }
                    }
                }
            }
        }
    }
}

#[test]
fn understand_into_iter() {
    let btree_set = BTreeSet::from([1, 2, 3, 4, 5]);
    for i in btree_set.iter() {
        println!("{i}");
    }

    let s = "abc".to_string();
    for c in s.chars() {
        println!("{c}");
    }
    println!("{s:?}");
    // println!("{btree_set:?}");

    // let ri = 1..=5;
    // let rii = ri.into_iter();
    // let val = rii.next();
    // let len = rii.len();
    // // for i in ri() {
    // //     println!("{i} {}", ri.len());
    // // }
    // // println!("{ri:?}");
    let s = "hello".to_string();
    let mut si = s.bytes();
    let _val = si.next();
    let _len = si.len();
    let _len2 = s.len();

    let arr = [1, 2, 3, 4, 5];
    for i in arr.iter() {
        println!("{i}");
    }

    for i in arr {
        println!("{i}");
    }

    // let rsi = RangeSetInt::from_iter(1..=5);
    // for i in rsi.iter() {
    //     println!("{i}");
    // }
    // let len = rsi.len();
}

// !!!cmk what's this about?
#[derive(Debug, PartialEq)]
struct BooleanVector(Vec<bool>);

impl BitAndAssign for BooleanVector {
    // `rhs` is the "right-hand side" of the expression `a &= b`.
    fn bitand_assign(&mut self, rhs: Self) {
        assert_eq!(self.0.len(), rhs.0.len());
        *self = BooleanVector(
            self.0
                .iter()
                .zip(rhs.0.iter())
                .map(|(x, y)| *x && *y)
                .collect(),
        );
    }
}

#[test]
fn understand_bitand_assign() {
    let mut a = 3u8;
    let b = 5u8;
    a &= b;
    println!("{a}");
    println!("{b}");

    let mut bv = BooleanVector(vec![true, true, false, false]);
    let bv2 = BooleanVector(vec![true, false, true, false]);
    bv &= bv2;
    let expected = BooleanVector(vec![true, false, false, false]);
    assert_eq!(bv, expected);
    // println!("{bv2:?}");
}

#[test]
fn iters() {
    let range_set_int = RangeSetInt::from_iter([1..=6, 8..=9, 11..=15]);
    assert!(range_set_int.len() == 13usize);
    for i in range_set_int.iter() {
        println!("{i}");
    }
    for range in range_set_int.ranges() {
        let (start, end) = range.into_inner();
        println!("{start}..={end}");
    }
    let mut rs = range_set_int.ranges();
    println!("{:?}", rs.next());
    println!("{range_set_int}");
    println!("{:?}", rs.len());
    println!("{:?}", rs.next());
    for i in range_set_int.iter() {
        println!("{i}");
    }
    // range_set_int.len();

    let mut rs = !range_set_int.ranges();
    println!("{:?}", rs.next());
    println!("{range_set_int}");
    // !!! assert that can't use range_set_int again
}

#[test]
fn missing_doctest_ops() {
    // note that may be borrowed or owned in any combination.

    // Returns the union of `self` and `rhs` as a new [`RangeSetInt`].
    let a = RangeSetInt::from_iter([1, 2, 3]);
    let b = RangeSetInt::from_iter([3, 4, 5]);

    let result = &a | &b;
    assert_eq!(result, RangeSetInt::from_iter([1, 2, 3, 4, 5]));
    let result = a | &b;
    assert_eq!(result, RangeSetInt::from_iter([1, 2, 3, 4, 5]));

    // Returns the complement of `self` as a new [`RangeSetInt`].
    let a = RangeSetInt::<i8>::from_iter([1, 2, 3]);

    let result = !&a;
    assert_eq!(result.to_string(), "-128..=0, 4..=127");
    let result = !a;
    assert_eq!(result.to_string(), "-128..=0, 4..=127");

    // Returns the intersection of `self` and `rhs` as a new `RangeSetInt<T>`.

    let a = RangeSetInt::from_iter([1, 2, 3]);
    let b = RangeSetInt::from_iter([2, 3, 4]);

    let result = a & &b;
    assert_eq!(result, RangeSetInt::from_iter([2, 3]));
    let a = RangeSetInt::from_iter([1, 2, 3]);
    let result = a & b;
    assert_eq!(result, RangeSetInt::from_iter([2, 3]));

    // Returns the symmetric difference of `self` and `rhs` as a new `RangeSetInt<T>`.
    let a = RangeSetInt::from_iter([1, 2, 3]);
    let b = RangeSetInt::from_iter([2, 3, 4]);

    let result = a ^ b;
    assert_eq!(result, RangeSetInt::from_iter([1, 4]));

    // Returns the set difference of `self` and `rhs` as a new `RangeSetInt<T>`.
    let a = RangeSetInt::from_iter([1, 2, 3]);
    let b = RangeSetInt::from_iter([2, 3, 4]);

    let result = a - b;
    assert_eq!(result, RangeSetInt::from_iter([1]));
}

#[test]
fn multi_op() {
    let a = RangeSetInt::from_iter([1..=6, 8..=9, 11..=15]);
    let b = RangeSetInt::from_iter([5..=13, 18..=29]);
    let c = RangeSetInt::from_iter([38..=42]);
    // cmkRule make these work d= a|b; d= a|b|c; d=&a|&b|&c;
    let d = &(&a | &b) | &c;
    println!("{d}");
    let d = a | b | &c;
    println!("{d}");

    let a = RangeSetInt::from_iter([1..=6, 8..=9, 11..=15]);
    let b = RangeSetInt::from_iter([5..=13, 18..=29]);
    let c = RangeSetInt::from_iter([38..=42]);

    let _ = [&a, &b, &c].union();
    let d = [a, b, c].iter().intersection();
    assert_eq!(d, RangeSetInt::new());

    assert_eq!(
        !MultiwayRangeSetInt::<u8>::union([]),
        RangeSetInt::from_iter([0..=255])
    );

    let a = RangeSetInt::from_iter([1..=6, 8..=9, 11..=15]);
    let b = RangeSetInt::from_iter([5..=13, 18..=29]);
    let c = RangeSetInt::from_iter([1..=42]);

    let _ = &a & &b;
    let d = [&a, &b, &c].intersection();
    // let d = RangeSetInt::intersection([a, b, c]);
    println!("{d}");
    assert_eq!(d, RangeSetInt::from_iter([5..=6, 8..=9, 11..=13]));

    assert_eq!(
        MultiwayRangeSetInt::<u8>::intersection([]),
        RangeSetInt::from_iter([0..=255])
    );
}

// https://stackoverflow.com/questions/21747136/how-do-i-print-in-rust-the-type-of-a-variable/58119924#58119924
// fn print_type_of<T>(_: &T) {
//     println!("{}", std::any::type_name::<T>())
// }

#[test]
fn custom_multi() {
    let a = RangeSetInt::from_iter([1..=6, 8..=9, 11..=15]);
    let b = RangeSetInt::from_iter([5..=13, 18..=29]);
    let c = RangeSetInt::from_iter([38..=42]);

    let union_stream = b.ranges() | c.ranges();
    let a_less = a.ranges() - union_stream;
    let d: RangeSetInt<_> = a_less.into();
    println!("{d}");

    let d: RangeSetInt<_> = (a.ranges() - [b.ranges(), c.ranges()].union()).into();
    println!("{d}");
}

#[test]
fn from_string() {
    let a = RangeSetInt::from_iter([0..=4, 14..=17, 30..=255, 0..=37, 43..=65535]);
    assert_eq!(a, RangeSetInt::from_iter([0..=65535]));
}

#[test]
fn nand_repro() {
    let b = &RangeSetInt::from_iter([5u8..=13, 18..=29]);
    let c = &RangeSetInt::from_iter([38..=42]);
    println!("about to nand");
    let d = !b | !c;
    println!("cmk '{d}'");
    assert_eq!(
        d,
        RangeSetInt::from_iter([0..=4, 14..=17, 30..=255, 0..=37, 43..=255])
    );
}

#[test]
fn parity() {
    let a = &RangeSetInt::from_iter([1..=6, 8..=9, 11..=15]);
    let b = &RangeSetInt::from_iter([5..=13, 18..=29]);
    let c = &RangeSetInt::from_iter([38..=42]);
    assert_eq!(
        a & !b & !c | !a & b & !c | !a & !b & c | a & b & c,
        RangeSetInt::from_iter([1..=4, 7..=7, 10..=10, 14..=15, 18..=29, 38..=42])
    );
    let _d = [a.ranges()].intersection();
    let _parity: RangeSetInt<u8> = [[a.ranges()].intersection()].union().into();
    let _parity: RangeSetInt<u8> = [a.ranges()].intersection().into();
    let _parity: RangeSetInt<u8> = [a.ranges()].union().into();
    println!("!b {}", !b);
    println!("!c {}", !c);
    println!("!b|!c {}", !b | !c);
    println!("!b|!c {}", RangeSetInt::from(!b.ranges() | !c.ranges()));

    let _a = RangeSetInt::from_iter([1..=6, 8..=9, 11..=15]);
    let u = union_dyn!(a.ranges());
    assert_eq!(
        RangeSetInt::from(u),
        RangeSetInt::from_iter([1..=6, 8..=9, 11..=15])
    );
    let u = union_dyn!(a.ranges(), b.ranges(), c.ranges());
    assert_eq!(
        RangeSetInt::from(u),
        RangeSetInt::from_iter([1..=15, 18..=29, 38..=42])
    );

    let u = [
        intersection_dyn!(a.ranges(), !b.ranges(), !c.ranges()),
        intersection_dyn!(!a.ranges(), b.ranges(), !c.ranges()),
        intersection_dyn!(!a.ranges(), !b.ranges(), c.ranges()),
        intersection_dyn!(a.ranges(), b.ranges(), c.ranges()),
    ]
    .union();
    assert_eq!(
        RangeSetInt::from(u),
        RangeSetInt::from_iter([1..=4, 7..=7, 10..=10, 14..=15, 18..=29, 38..=42])
    );
}

#[test]
fn bit_or_iter() {
    let i = UnionIter::from([1, 3, 4, 2, 2, 43, -1, 4, 22]);
    let j = UnionIter::from([11, 3, 4, 42, 2, 43, 23, 2, 543]);

    let _not_i = !i.clone();
    let k = i - j;
    assert_eq!(k.to_string(), "-1..=-1, 1..=1, 22..=22");
}

#[test]
fn empty() {
    let universe: UnionIter<u8, _> = [0..=255].into_iter().collect();
    let arr: [u8; 0] = [];
    let a0 = RangeSetInt::<u8>::from_iter(arr);
    assert!(!(a0.ranges()).equal(universe.clone()));
    assert!((!a0).ranges().equal(universe));
    let _a0 = RangeSetInt::from_iter([0..=0; 0]);
    let _a = RangeSetInt::<i32>::new();

    let a_iter: std::array::IntoIter<i32, 0> = [].into_iter();
    let a = a_iter.collect::<RangeSetInt<i32>>();
    let arr: [i32; 0] = [];
    let b = RangeSetInt::from_iter(arr);
    let mut c3 = a.clone();
    let mut c5 = a.clone();

    let c0 = (&a).bitor(&b);
    let c1a = &a | &b;
    let c1b = &a | b.clone();
    let c1c = a.clone() | &b;
    let c1d = a.clone() | b.clone();
    let c2: RangeSetInt<_> = (a.ranges() | b.ranges()).into();
    c3.append(&mut b.clone());
    c5.extend(b);

    let answer = RangeSetInt::from_iter(arr);
    assert_eq!(&c0, &answer);
    assert_eq!(&c1a, &answer);
    assert_eq!(&c1b, &answer);
    assert_eq!(&c1c, &answer);
    assert_eq!(&c1d, &answer);
    assert_eq!(&c2, &answer);
    assert_eq!(&c3, &answer);
    assert_eq!(&c5, &answer);

    let a_iter: std::array::IntoIter<i32, 0> = [].into_iter();
    let a = a_iter.collect::<RangeSetInt<i32>>();
    let b = RangeSetInt::from_iter([0i32; 0]);

    let c0 = a.ranges() | b.ranges();
    let c1 = [a.ranges(), b.ranges()].union();
    let c_list2: [RangesIter<i32>; 0] = [];
    let c2 = c_list2.clone().union();
    let c3 = union_dyn!(a.ranges(), b.ranges());
    let c4 = c_list2.map(DynSortedDisjoint::new).union();

    let answer = RangeSetInt::from_iter(arr);
    assert!(c0.equal(answer.ranges()));
    assert!(c1.equal(answer.ranges()));
    assert!(c2.equal(answer.ranges()));
    assert!(c3.equal(answer.ranges()));
    assert!(c4.equal(answer.ranges()));

    let c0 = !(a.ranges() & b.ranges());
    let c1 = ![a.ranges(), b.ranges()].intersection();
    let c_list2: [RangesIter<i32>; 0] = [];
    let c2 = !!c_list2.clone().intersection();
    let c3 = !intersection_dyn!(a.ranges(), b.ranges());
    let c4 = !!c_list2.map(DynSortedDisjoint::new).intersection();

    let answer = !RangeSetInt::from_iter([0i32; 0]);
    assert!(c0.equal(answer.ranges()));
    assert!(c1.equal(answer.ranges()));
    assert!(c2.equal(answer.ranges()));
    assert!(c3.equal(answer.ranges()));
    assert!(c4.equal(answer.ranges()));
}

// Can't implement fmt::Display fmt must take ownership
impl<T, I> UnsortedDisjoint<T, I>
where
    T: Integer,
    I: Iterator<Item = RangeInclusive<T>>,
{
    #[allow(clippy::inherent_to_string)]
    #[allow(clippy::wrong_self_convention)]
    pub(crate) fn to_string(self) -> String {
        self.map(|range| {
            let (start, end) = range.into_inner();
            format!("{start}..={end}") // cmk could we format RangeInclusive directly?
        })
        .join(", ")
    }
}
#[allow(clippy::reversed_empty_ranges)]
#[test]
fn private_constructor() {
    let unsorted_disjoint = UnsortedDisjoint::from([5..=6, 1..=5, 1..=0, -12..=-10, 3..=3]);
    // println!("{}", unsorted_disjoint.fmt());
    assert_eq!(unsorted_disjoint.to_string(), "1..=6, -12..=-10, 3..=3");

    let unsorted_disjoint = UnsortedDisjoint::from([5..=6, 1..=5, 1..=0, -12..=-10, 3..=3]);
    let union_iter = UnionIter::from(unsorted_disjoint);
    // println!("{}", union_iter.fmt());
    assert_eq!(union_iter.to_string(), "-12..=-10, 1..=6");

    let union_iter: UnionIter<_, _> = [5, 6, 1, 2, 3, 4, 5, -12, -11, -10, 3]
        .into_iter()
        .collect();
    assert_eq!(union_iter.to_string(), "-12..=-10, 1..=6");
}

fn is_sssu<T: Sized + Send + Sync + Unpin>() {}

fn is_ddcppdheo<
    T: std::fmt::Debug
        + Display
        + Clone
        + PartialEq
        + PartialOrd
        + Default
        + std::hash::Hash
        + Eq
        + Ord
        + Send
        + Sync,
>() {
}

// DoubleEndedIterator +ExactSizeIterator
fn is_like_btreeset_iter<T: Clone + std::fmt::Debug + FusedIterator + Iterator>() {}
fn is_like_btreeset_into_iter<T: std::fmt::Debug + FusedIterator + Iterator>() {}

fn is_like_btreeset<
    T: Clone
        + std::fmt::Debug
        + Default
        + Eq
        + std::hash::Hash
        + IntoIterator
        + Ord
        + PartialEq
        + PartialOrd
        + RefUnwindSafe
        + Send
        + Sync
        + Unpin
        + UnwindSafe
        + Any
        + ToOwned,
>() {
}

fn is_like_check_sorted_disjoint<
    T: Clone
        + std::fmt::Debug
        + Default
        + IntoIterator
        + RefUnwindSafe
        + Send
        + Sync
        + Unpin
        + UnwindSafe
        + Any
        + ToOwned,
>() {
}

fn is_like_dyn_sorted_disjoint<T: IntoIterator + Unpin + Any>() {}

// cmk rule: Test that you're implementing all the traits from the data structures you're using as a model. I was leaving out Debug and FusedIterator
#[test]
fn check_traits() {
    // Debug/Display/Clone/PartialEq/PartialOrd/Default/Hash/Eq/Ord/Send/Sync
    type ARangeSetInt = RangeSetInt<i32>;
    is_sssu::<ARangeSetInt>();
    is_ddcppdheo::<ARangeSetInt>();
    is_like_btreeset::<ARangeSetInt>();

    type ARangesIter<'a> = RangesIter<'a, i32>;
    is_sssu::<ARangesIter>();
    is_like_btreeset_iter::<ARangesIter>();

    type AIter<'a> = Iter<i32, ARangesIter<'a>>;
    is_sssu::<AIter>();
    is_like_btreeset_iter::<AIter>();

    is_sssu::<IntoIter<i32>>();
    is_like_btreeset_into_iter::<IntoIter<i32>>();

    type AMerge<'a> = Merge<i32, ARangesIter<'a>, ARangesIter<'a>>;
    is_sssu::<AMerge>();
    is_like_btreeset_iter::<AMerge>();

    let a = RangeSetInt::from_iter([1..=2, 3..=4]);
    println!("{:?}", a.ranges());

    type AKMerge<'a> = KMerge<i32, ARangesIter<'a>>;
    is_sssu::<AKMerge>();
    is_like_btreeset_iter::<AKMerge>();

    type ANotIter<'a> = NotIter<i32, ARangesIter<'a>>;
    is_sssu::<ANotIter>();
    is_like_btreeset_iter::<ANotIter>();

    type AIntoRangesIter = IntoRangesIter<i32>;
    is_sssu::<AIntoRangesIter>();
    is_like_btreeset_into_iter::<AIntoRangesIter>();

    type ACheckSortedDisjoint<'a> = CheckSortedDisjoint<i32, ARangesIter<'a>>;
    is_sssu::<ACheckSortedDisjoint>();
    type BCheckSortedDisjoint =
        CheckSortedDisjoint<i32, std::array::IntoIter<RangeInclusive<i32>, 0>>;
    is_like_check_sorted_disjoint::<BCheckSortedDisjoint>();

    type ADynSortedDisjoint<'a> = DynSortedDisjoint<'a, i32>;
    is_like_dyn_sorted_disjoint::<ADynSortedDisjoint>();

    type AUnionIter<'a> = UnionIter<i32, ARangesIter<'a>>;
    is_sssu::<AUnionIter>();
    is_like_btreeset_iter::<AUnionIter>();

    type AAssumeSortedStarts<'a> = AssumeSortedStarts<i32, ARangesIter<'a>>;
    is_sssu::<AAssumeSortedStarts>();
    is_like_btreeset_iter::<AAssumeSortedStarts>();
}

#[test]
fn integer_coverage() {
    syntactic_for! { ty in [i8, u8, isize, usize,  i16, u16, i32, u32, i64, u64, isize, usize, i128, u128] {
        $(
            let len = <$ty as Integer>::SafeLen::one();
            let a = $ty::zero();
            assert_eq!($ty::safe_len_to_f64(len), 1.0);
            assert_eq!($ty::add_len_less_one(a,len), a);
            assert_eq!($ty::sub_len_less_one(a,len), a);
            assert_eq!($ty::f64_to_safe_len(1.0), len);
            assert!($ty::safe_max_value()<=$ty::max_value());
            assert!(<$ty as Integer>::safe_max_value()<=$ty::max_value());

        )*
    }};
}

#[test]
#[allow(clippy::bool_assert_comparison)]
fn lib_coverage_0() {
    let a = RangeSetInt::from_iter([1..=2, 3..=4]);
    let _b = a.clone();
    let mut hasher = DefaultHasher::new();
    a.hash(&mut hasher);
    let _d = RangeSetInt::<i32>::default();
    assert_eq!(a, a);

    let mut set = RangeSetInt::new();
    assert_eq!(set.first(), None);
    set.insert(1);
    assert_eq!(set.first(), Some(1));
    set.insert(2);
    assert_eq!(set.first(), Some(1));

    let set = RangeSetInt::from_iter([1, 2, 3]);
    assert_eq!(set.get(2), Some(2));
    assert_eq!(set.get(4), None);

    let mut set = RangeSetInt::new();
    assert_eq!(set.last(), None);
    set.insert(1);
    assert_eq!(set.last(), Some(1));
    set.insert(2);
    assert_eq!(set.last(), Some(2));

    assert_eq!(a.len(), a._len_slow());

    let mut a = RangeSetInt::from_iter([1..=3]);
    let mut b = RangeSetInt::from_iter([3..=5]);

    a.append(&mut b);

    assert_eq!(a.len(), 5usize);
    assert_eq!(b.len(), 0usize);

    assert!(a.contains(1));
    assert!(a.contains(2));
    assert!(a.contains(3));
    assert!(a.contains(4));
    assert!(a.contains(5));

    let mut v = RangeSetInt::new();
    v.insert(1);
    v.clear();
    assert!(v.is_empty());

    let mut v = RangeSetInt::new();
    assert!(v.is_empty());
    v.insert(1);
    assert!(!v.is_empty());

    let sup = RangeSetInt::from_iter([1..=3]);
    let mut set = RangeSetInt::new();

    assert_eq!(set.is_subset(&sup), true);
    set.insert(2);
    assert_eq!(set.is_subset(&sup), true);
    set.insert(4);
    assert_eq!(set.is_subset(&sup), false);

    let sub = RangeSetInt::from_iter([1, 2]);
    let mut set = RangeSetInt::new();

    assert_eq!(set.is_superset(&sub), false);

    set.insert(0);
    set.insert(1);
    assert_eq!(set.is_superset(&sub), false);

    set.insert(2);
    assert_eq!(set.is_superset(&sub), true);

    let a = RangeSetInt::from_iter([1..=3]);
    let mut b = RangeSetInt::new();

    assert_eq!(a.is_disjoint(&b), true);
    b.insert(4);
    assert_eq!(a.is_disjoint(&b), true);
    b.insert(1);
    assert_eq!(a.is_disjoint(&b), false);

    let mut set = RangeSetInt::new();
    set.insert(3);
    set.insert(5);
    set.insert(8);
    assert_eq!(Some(5), set.range(4..).next());
    assert_eq!(Some(3), set.range(..).next());
    assert_eq!(None, set.range(..=2).next());
    assert_eq!(None, set.range(1..2).next());
    assert_eq!(
        Some(3),
        set.range((Bound::Excluded(2), Bound::Excluded(4))).next()
    );

    let mut set = RangeSetInt::new();

    assert_eq!(set.ranges_insert(2..=5), true);
    assert_eq!(set.ranges_insert(5..=6), true);
    assert_eq!(set.ranges_insert(3..=4), false);
    assert_eq!(set.len(), 5usize);
    let mut set = RangeSetInt::from_iter([1, 2, 3]);
    assert_eq!(set.take(2), Some(2));
    assert_eq!(set.take(2), None);

    let mut set = RangeSetInt::new();
    assert!(set.replace(5).is_none());
    assert!(set.replace(5).is_some());

    let mut a = RangeSetInt::from_iter([1..=3]);
    #[allow(clippy::reversed_empty_ranges)]
    a.internal_add(2..=1);

    assert_eq!(a.partial_cmp(&a), Some(Ordering::Equal));

    let mut a = RangeSetInt::from_iter([1..=3]);
    a.extend(std::iter::once(4));
    assert_eq!(a.len(), 4usize);

    let mut a = RangeSetInt::from_iter([1..=3]);
    a.extend(4..=5);
    assert_eq!(a.len(), 5usize);

    let mut set = RangeSetInt::new();

    set.insert(1);
    while let Some(n) = set.pop_first() {
        assert_eq!(n, 1);
    }
    assert!(set.is_empty());

    let mut set = RangeSetInt::new();

    set.insert(1);
    while let Some(n) = set.pop_last() {
        assert_eq!(n, 1);
    }
    assert!(set.is_empty());

    let a = RangeSetInt::from_iter([1..=3]);
    let i = a.iter();
    let j = i.clone();
    assert_eq!(i.size_hint(), j.size_hint());
    assert_eq!(format!("{:?}", &i), format!("{:?}", &j));

    let a = RangeSetInt::from_iter([1..=3]);
    let i = a.into_iter();
    assert_eq!(i.size_hint(), j.size_hint());
    assert_eq!(
        format!("{:?}", &i),
        "IntoIter { option_range: None, into_iter: [(1, 3)] }"
    );

    let mut a = RangeSetInt::from_iter([1..=3]);
    a.extend([1..=3]);
    assert_eq!(a.len(), 3usize);

    let a = RangeSetInt::from_iter([1..=3]);
    let b = <RangeSetInt<i32> as Clone>::clone(&a);
    assert_eq!(a, b);
    let c = <RangeSetInt<i32> as Default>::default();
    assert_eq!(c, RangeSetInt::new());

    syntactic_for! { ty in [i8, u8, isize, usize,  i16, u16, i32, u32, i64, u64, isize, usize, i128, u128] {
        $(
            let a = RangeSetInt::<$ty>::new();
            println!("{a:#?}");
            assert_eq!(a.iter().next(), None);

            let mut a = RangeSetInt::from_iter([$ty::one()..=3]);
            let mut b = RangeSetInt::from_iter([3..=5]);

            a.append(&mut b);

            // assert_eq!(a.len(), 5usize);
            assert_eq!(b.len(), <$ty as Integer>::SafeLen::zero());

            assert!(a.contains(1));
            assert!(a.contains(2));
            assert!(a.contains(3));
            assert!(a.contains(4));
            assert!(a.contains(5));

            assert!(b.is_empty());

            let a = RangeSetInt::from_iter([$ty::one()..=3]);
            let b = RangeSetInt::from_iter([3..=5]);
            assert!(!a.is_subset(&b));
            assert!(!a.is_superset(&b));

        )*
    }};

    let a = RangeSetInt::from_iter([1u128..=3]);
    assert!(a.contains(1));
    assert!(!a.is_disjoint(&a));
}

#[test]
#[should_panic]
fn lib_coverage_2() {
    let v = RangeSetInt::<u128>::new();
    v.contains(u128::MAX);
}

#[test]
#[should_panic]
fn lib_coverage_3() {
    let mut v = RangeSetInt::<u128>::new();
    v.remove(u128::MAX);
}

#[test]
#[should_panic]
fn lib_coverage_4() {
    let mut v = RangeSetInt::<u128>::new();
    v.split_off(u128::MAX);
}

#[test]
#[should_panic]
fn lib_coverage_5() {
    let mut v = RangeSetInt::<u128>::new();
    v.internal_add(0..=u128::MAX);
}

#[test]
fn lib_coverage_6() {
    syntactic_for! { ty in [i8, u8, isize, usize,  i16, u16, i32, u32, i64, u64, isize, usize, i128, u128] {
        $(
            let mut a = RangeSetInt::<$ty>::from_iter([1..=3, 5..=7, 9..=120]);
            a.ranges_insert(2..=100);
            assert_eq!(a, RangeSetInt::from_iter([1..=120]));


        )*
    }};
}

#[test]
fn merge_coverage_0() {
    let a = CheckSortedDisjoint::new(vec![1..=2, 5..=100].into_iter());
    let b = CheckSortedDisjoint::from([2..=6]);
    let m = Merge::new(a, b);
    let n = m.clone();
    let p = n.clone();
    let union1 = UnionIter::new(m);
    let union2 = UnionIter::new(n);
    assert!(union1.equal(union2));
    assert!(format!("{p:?}").starts_with("Merge"));

    let a = CheckSortedDisjoint::new(vec![1..=2, 5..=100].into_iter());
    let b = CheckSortedDisjoint::new(vec![2..=6].into_iter());
    let c = CheckSortedDisjoint::new(vec![-1..=-1].into_iter());
    let m = KMerge::new([a, b, c]);
    let n = m.clone();
    let p = n.clone();
    let union1 = UnionIter::new(m);
    let union2 = UnionIter::new(n);
    assert!(union1.equal(union2));
    assert!(format!("{p:?}").starts_with("KMerge"));
}

#[test]
fn not_iter_coverage_0() {
    let a = CheckSortedDisjoint::new(vec![1..=2, 5..=100].into_iter());
    let n = NotIter::new(a);
    let p = n.clone();
    let m = p.clone();
    assert!(n.equal(m));
    assert!(format!("{p:?}").starts_with("NotIter"));
}

#[test]
fn ranges_coverage_0() {
    let a = RangeSetInt::from_iter([1..=2, 5..=100]);
    let r = a.ranges();
    let p = r.as_ref();
    assert!(format!("{p:?}").starts_with("Ranges"));
    assert_eq!(r.len(), 2);

    let r2 = a.into_ranges();
    let n2 = !!r2;
    let a = RangeSetInt::from_iter([1..=2, 5..=100]);
    assert!(n2.equal(a.ranges()));
    let a = RangeSetInt::from_iter([1..=2, 5..=100]);
    let b = a.into_ranges();
    let a = RangeSetInt::from_iter([1..=2, 5..=100]);
    let c = a.into_ranges();
    let a = RangeSetInt::from_iter([1..=2, 5..=100]);
    assert!((b | c).equal(a.ranges()));

    let a = RangeSetInt::from_iter([1..=2, 5..=100]).into_ranges();
    let b = RangeSetInt::from_iter([1..=2, 5..=100]).into_ranges();
    assert!((a - b).is_empty());

    let a = RangeSetInt::from_iter([1..=2, 5..=100]).into_ranges();
    let b = RangeSetInt::from_iter([1..=2, 5..=100]).into_ranges();
    assert!((a ^ b).is_empty());

    let a = RangeSetInt::from_iter([1..=2, 5..=100]).into_ranges();
    let b = RangeSetInt::from_iter([1..=2, 5..=100]).into_ranges();
    assert!((a & b).equal(RangeSetInt::from_iter([1..=2, 5..=100]).into_ranges()));

    assert_eq!(
        RangeSetInt::from_iter([1..=2, 5..=100]).into_ranges().len(),
        2
    );
    assert!(format!(
        "{:?}",
        RangeSetInt::from_iter([1..=2, 5..=100]).into_ranges()
    )
    .starts_with("IntoRanges"));
}

#[test]
fn sorted_disjoint_coverage_0() {
    let a = CheckSortedDisjoint::<i32, _>::default();
    assert!(a.is_empty());

    let a = CheckSortedDisjoint::new([1..=2, 5..=100].into_iter());
    let b = CheckSortedDisjoint::new([1..=2, 5..=100].into_iter());
    assert!((a & b).equal(CheckSortedDisjoint::new([1..=2, 5..=100].into_iter())));

    let a = CheckSortedDisjoint::new([1..=2, 5..=100].into_iter());
    let b = CheckSortedDisjoint::new([1..=2, 5..=100].into_iter());
    assert!((a - b).is_empty());

    let a = CheckSortedDisjoint::new([1..=2, 5..=100].into_iter());
    let b = CheckSortedDisjoint::new([1..=2, 5..=100].into_iter());
    assert!((a ^ b).is_empty());
}

#[test]
#[should_panic]
fn sorted_disjoint_coverage_1() {
    struct SomeAfterNone {
        a: i32,
    }
    impl Iterator for SomeAfterNone {
        type Item = RangeInclusive<i32>;
        fn next(&mut self) -> Option<Self::Item> {
            self.a += 1;
            if self.a % 2 == 0 {
                Some(self.a..=self.a)
            } else {
                None
            }
        }
    }

    let mut a = CheckSortedDisjoint::new(SomeAfterNone { a: 0 });
    a.next();
    a.next();
    a.next();
}

#[test]
#[should_panic]
fn sorted_disjoint_coverage_2() {
    #[allow(clippy::reversed_empty_ranges)]
    let mut a = CheckSortedDisjoint::new([1..=0].into_iter());
    a.next();
}

#[test]
#[should_panic]
fn sorted_disjoint_coverage_3() {
    #[allow(clippy::reversed_empty_ranges)]
    let mut a = CheckSortedDisjoint::new([1..=1, 2..=2].into_iter());
    a.next();
    a.next();
}

#[test]
#[should_panic]
fn sorted_disjoint_coverage_4() {
    #[allow(clippy::reversed_empty_ranges)]
    let mut a = CheckSortedDisjoint::new([0..=i128::MAX].into_iter());
    a.next();
}

#[test]
fn sorted_disjoint_iterator_coverage_0() {
    let a = CheckSortedDisjoint::new([1..=2, 5..=100].into_iter());
    let b = CheckSortedDisjoint::new([1..=2, 5..=101].into_iter());
    assert!(b.is_superset(a));
}

#[test]
fn union_iter_coverage_0() {
    let a = CheckSortedDisjoint::new(vec![1..=2, 5..=100].into_iter());
    let b = CheckSortedDisjoint::new(vec![1..=2, 5..=101].into_iter());
    let c = a.union(b);
    assert!(format!("{c:?}").starts_with("UnionIter"));
}

#[test]
fn unsorted_disjoint_coverage_0() {
    let a = AssumeSortedStarts::new([1..=2, 5..=100].into_iter());
    assert!(format!("{a:?}").starts_with("AssumeSortedStarts"));
}

#[test]
fn test_coverage_0() {
    let a = BooleanVector(vec![true, true, false, false]);
    assert!(format!("{a:?}").starts_with("BooleanVector"));

    let a = How::Union;
    #[allow(clippy::clone_on_copy)]
    let _b = a.clone();

    let mut rng = StdRng::seed_from_u64(0);
    let a = MemorylessRange::new(&mut rng, 1000, 0..=10, 0.5, 1, How::Intersection);
    let v: Vec<_> = a.take(100).collect();
    println!("{v:?}");

    let a = MemorylessIter::new(&mut rng, 1000, 0..=10, 0.5, 1, How::Intersection);
    let v: Vec<_> = a.take(100).collect();
    println!("{v:?}");
}
