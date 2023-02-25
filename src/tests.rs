#![cfg(test)]

// !!!cmk rule: Use the same testing as with macros to check that the types are correct
// !!!cmk rule make illegal states unpresentable (example u8.len->usize, but u128 needs max_value2), SortedDisjointIter
// !!!cmk rule detail: Note that this nicely fails to compile if you try to chain when you shouldn't
// !!!cmk rule detail:  let chain = b.ranges().chain(c.ranges());
// !!!cmk rule detail:  let a_less = a.ranges().sub(chain);
// !!!cmk rule test near extreme values
// !!!cmk test it across threads
use std::collections::BTreeSet; // , time::Instant

use super::*;
// use sorted_iter::assume::AssumeSortedByKeyExt;
// use rand::{rngs::StdRng, seq::SliceRandom, SeedableRng};
use syntactic_for::syntactic_for;
// use thousands::Separable;
use std::ops::BitAndAssign;

#[test]
fn insert_255u8() {
    let range_set_int = RangeSetInt::<u8>::from([255]);
    assert!(range_set_int.to_string() == "255..=255");
}

#[test]
#[should_panic]
fn insert_max_u128() {
    let a = RangeSetInt::<u128>::from([u128::MAX]);
    println!("a: {a}");
}

#[test]
fn sub() {
    for start in i8::MIN..i8::MAX {
        for end in start..i8::MAX {
            let diff = i8::safe_subtract_inclusive(end, start);
            let diff2 = (end as i16) - (start as i16) + 1;
            assert_eq!(diff as i16, diff2);
        }
    }
    for start in u8::MIN..u8::MAX {
        for end in start..u8::MAX {
            let diff = u8::safe_subtract_inclusive(end, start);
            let diff2 = (end as i16) - (start as i16) + 1;
            assert_eq!(diff as i16, diff2);
        }
    }

    let before = 127i8.overflowing_sub(-128i8).0;
    let after = before as u8;
    println!("before: {before}, after: {after}");
}

#[test]
fn complement() {
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
    let a = RangeSetInt::from([1u8, 2, 3]);
    let b = RangeSetInt::from([2u8, 3, 4]);

    let result = &a & &b;
    println!("{result}");
    assert_eq!(result, RangeSetInt::from([2u8, 3]));
}

#[test]
fn repro1() {
    let mut range_set_int = RangeSetInt::<i8>::from("20..=21,24..=24,25..=29");
    println!("{range_set_int}");
    assert!(range_set_int.to_string() == "20..=21,24..=29");
    range_set_int.internal_add(25, 25);
    println!("{range_set_int}");
    assert!(range_set_int.to_string() == "20..=21,24..=29");
}

#[test]
fn repro2() {
    let mut range_set_int = RangeSetInt::<i8>::from([-8, 8, -2, -1, 3, 2]);
    range_set_int.internal_add(25, 25);
    println!("{range_set_int}");
    assert!(range_set_int.to_string() == "-8..=-8,-2..=-1,2..=3,8..=8,25..=25");
}

#[test]
fn doctest1() {
    let a = RangeSetInt::<u8>::from([1, 2, 3]);
    let b = RangeSetInt::<u8>::from([3, 4, 5]);

    let result = &a | &b;
    assert_eq!(result, RangeSetInt::<u8>::from([1, 2, 3, 4, 5]));
}

#[test]
fn doctest2() {
    let set = RangeSetInt::<u8>::from([1, 2, 3]);
    assert!(set.contains(1));
    assert!(!set.contains(4));
}

#[test]
fn doctest3() {
    let mut a = RangeSetInt::<u8>::from("1..=3");
    let mut b = RangeSetInt::<u8>::from("3..=5");

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
    assert_eq!(result.to_string(), "-128..=0,4..=127");
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
    let mut range_set_int = RangeSetInt::<u8>::from("10..=10");
    range_set_int.internal_add(12, 12);
    assert!(range_set_int.to_string() == "10..=10,12..=12");
    assert!(range_set_int._len_slow() == range_set_int.len());
}

#[test]
fn demo_c2() {
    // before_or_equal_exists	1
    // equal?	0
    // is_included	0
    //     INSERT
    let mut range_set_int = RangeSetInt::<u8>::from("10..=10,13..=13");
    range_set_int.internal_add(12, 12);
    assert!(range_set_int.to_string() == "10..=10,12..=13");
    assert!(range_set_int._len_slow() == range_set_int.len());
}

#[test]
fn demo_f1() {
    // before_or_equal_exists	0
    //     INSERT, etc

    let mut range_set_int = RangeSetInt::<u8>::from("11..=14,22..=26");
    range_set_int.internal_add(10, 10);
    assert!(range_set_int.to_string() == "10..=14,22..=26");
    assert!(range_set_int._len_slow() == range_set_int.len());
}

#[test]
fn demo_d1() {
    // before_or_equal_exists	1
    // equal?	1
    // is_included	n/a
    // fits?	1
    //     DONE

    let mut range_set_int = RangeSetInt::<u8>::from("10..=14");
    range_set_int.internal_add(10, 10);
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

    let mut range_set_int = RangeSetInt::<u8>::from("10..=14,16..=16");
    range_set_int.internal_add(10, 19);
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

    let mut range_set_int = RangeSetInt::<u8>::from("10..=14");
    range_set_int.internal_add(12, 17);
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

    let mut range_set_int = RangeSetInt::<u8>::from("10..=14,16..=16");
    range_set_int.internal_add(12, 17);
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

    let mut range_set_int = RangeSetInt::<u8>::from("10..=15,160..=160");
    range_set_int.internal_add(12, 17);
    assert!(range_set_int.to_string() == "10..=17,160..=160");
    assert!(range_set_int._len_slow() == range_set_int.len());
}

#[test]
fn demo_a() {
    // before_or_equal_exists	1
    // equal?	0
    // is_included	1
    // fits?	1
    //     DONE
    let mut range_set_int = RangeSetInt::<u8>::from("10..=14");
    range_set_int.internal_add(12, 12);
    assert!(range_set_int.to_string() == "10..=14");
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
    let stop = 8u8;
    for a in 0..=stop {
        for b in 0..=stop {
            for c in 0..=stop {
                for d in 0..=stop {
                    let restart = (a >= 2 && a - 2 >= d) || (c >= 2 && c - 2 >= b);
                    print!("{a}\t{b}\t{c}\t{d}\t");
                    if a > b {
                        println!("impossible");
                    } else if c > d {
                        println!("error");
                    } else {
                        let mut range_set_int = RangeSetInt::new();
                        range_set_int.internal_add(a, b);
                        range_set_int.internal_add(c, d);
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
    let range_set_int = RangeSetInt::<u8>::from("1..=6,8..=9,11..=15");
    assert!(range_set_int.len() == 13);
    // !!!cmk0
    // assert!(range_set_int.ranges.len() == 3);
    // // !!!cmk0 i is &u8
    for i in range_set_int.iter() {
        println!("{i}");
    }
    for (start, stop) in range_set_int.ranges() {
        println!("{start} {stop}");
    }
    let mut rs = range_set_int.ranges();
    println!("{:?}", rs.next());
    println!("{range_set_int}");
    println!("{:?}", rs.len());
    println!("{:?}", rs.next());
    for i in range_set_int.iter() {
        println!("{i}");
    }
    range_set_int.len();

    let mut rs = !range_set_int.ranges();
    println!("{:?}", rs.next());
    println!("{range_set_int}");
    // !!! assert that can't use range_set_int again
}

#[test]
fn missing_doctest_ops() {
    // note that may be borrowed or owned in any combination.

    // Returns the union of `self` and `rhs` as a new `RangeSetInt`.
    let a = RangeSetInt::from([1, 2, 3]);
    let b = RangeSetInt::from([3, 4, 5]);

    let result = &a | &b;
    assert_eq!(result, RangeSetInt::from([1, 2, 3, 4, 5]));
    let result = a | &b;
    assert_eq!(result, RangeSetInt::from([1, 2, 3, 4, 5]));

    // Returns the complement of `self` as a new `RangeSetInt`.
    let a = RangeSetInt::<i8>::from([1, 2, 3]);

    let result = !&a;
    assert_eq!(result.to_string(), "-128..=0,4..=127");
    let result = !a;
    assert_eq!(result.to_string(), "-128..=0,4..=127");

    // Returns the intersection of `self` and `rhs` as a new `RangeSetInt<T>`.

    let a = RangeSetInt::from([1, 2, 3]);
    let b = RangeSetInt::from([2, 3, 4]);

    let result = a & &b;
    assert_eq!(result, RangeSetInt::from([2, 3]));
    let a = RangeSetInt::from([1, 2, 3]);
    let result = a & b;
    assert_eq!(result, RangeSetInt::from([2, 3]));

    // Returns the symmetric difference of `self` and `rhs` as a new `RangeSetInt<T>`.
    let a = RangeSetInt::from([1, 2, 3]);
    let b = RangeSetInt::from([2, 3, 4]);

    let result = a ^ b;
    assert_eq!(result, RangeSetInt::from([1, 4]));

    // Returns the set difference of `self` and `rhs` as a new `RangeSetInt<T>`.
    let a = RangeSetInt::from([1, 2, 3]);
    let b = RangeSetInt::from([2, 3, 4]);

    let result = a - b;
    assert_eq!(result, RangeSetInt::from([1]));
}

#[test]
fn multi_op() {
    let a = RangeSetInt::<u8>::from("1..=6,8..=9,11..=15");
    let b = RangeSetInt::<u8>::from("5..=13,18..=29");
    let c = RangeSetInt::<u8>::from("38..=42");
    // cmkRule make these work d= a|b; d= a|b|c; d=&a|&b|&c;
    let d = &(&a | &b) | &c;
    println!("{d}");
    let d = a | b | &c;
    println!("{d}");

    let a = RangeSetInt::<u8>::from("1..=6,8..=9,11..=15");
    let b = RangeSetInt::<u8>::from("5..=13,18..=29");
    let c = RangeSetInt::<u8>::from("38..=42");

    // !!!cmk0 must with on empty, with ref and with owned

    let _ = RangeSetInt::union([&a, &b, &c]);
    let d = RangeSetInt::intersection([a, b, c].iter());
    assert_eq!(d, RangeSetInt::new());

    assert_eq!(!RangeSetInt::<u8>::union([]), RangeSetInt::from("0..=255"));

    let a = RangeSetInt::<u8>::from("1..=6,8..=9,11..=15");
    let b = RangeSetInt::<u8>::from("5..=13,18..=29");
    let c = RangeSetInt::<u8>::from("1..=42");

    let _ = &a & &b;
    let d = RangeSetInt::intersection([&a, &b, &c]);
    // let d = RangeSetInt::intersection([a, b, c]);
    println!("{d}");
    assert_eq!(d, RangeSetInt::from("5..=6,8..=9,11..=13"));

    assert_eq!(
        RangeSetInt::<u8>::intersection([]),
        RangeSetInt::from("0..=255")
    );
}

// cmk000 print iters of sorted ranges
// cmk0 use merge in example
// cmk0 support 'collect' not just 'from'
// cmk much too easy to make errors -- need types!

// https://stackoverflow.com/questions/21747136/how-do-i-print-in-rust-the-type-of-a-variable/58119924#58119924
// fn print_type_of<T>(_: &T) {
//     println!("{}", std::any::type_name::<T>())
// }

#[test]
fn custom_multi() {
    let a = RangeSetInt::<u8>::from("1..=6,8..=9,11..=15");
    let b = RangeSetInt::<u8>::from("5..=13,18..=29");
    let c = RangeSetInt::<u8>::from("38..=42");

    let union_stream = b.ranges().bitor(c.ranges());
    let a_less = a.ranges() - union_stream;
    let d: RangeSetInt<_> = a_less.into();
    println!("{d}");

    let d: RangeSetInt<_> = (a.ranges() - union([b.ranges(), c.ranges()])).into();
    println!("{d}");
}

#[test]
fn from_string() {
    let a = RangeSetInt::<u16>::from("0..=4,14..=17,30..=255,0..=37,43..=65535");
    assert_eq!(a, RangeSetInt::from("0..=65535"));
}

#[test]
fn nand_repro() {
    let b = &RangeSetInt::<u8>::from("5..=13,18..=29");
    let c = &RangeSetInt::<u8>::from("38..=42");
    println!("about to nand");
    let d = !b | !c;
    println!("cmk '{d}'");
    assert_eq!(
        d,
        RangeSetInt::from("0..=4,14..=17,30..=255,0..=37,43..=255")
    );
}

#[test]
fn parity() {
    let a = &RangeSetInt::<u8>::from("1..=6,8..=9,11..=15");
    let b = &RangeSetInt::<u8>::from("5..=13,18..=29");
    let c = &RangeSetInt::<u8>::from("38..=42");
    // !!!cmk0 time itertools.split (?) vs range.clone()
    // !!!cmk explain why need both "Merge" with "KMerge"
    // !!!cmk0 empty needs to work. Go back to slices?
    assert_eq!(
        a & !b & !c | !a & b & !c | !a & !b & c | a & b & c,
        RangeSetInt::from("1..=4,7..=7,10..=10,14..=15,18..=29,38..=42")
    );
    let _d = intersection([a.ranges()]);
    let _parity: RangeSetInt<u8> = union([intersection([a.ranges()])]).into();
    let _parity: RangeSetInt<u8> = intersection([a.ranges()]).into();
    let _parity: RangeSetInt<u8> = union([a.ranges()]).into();
    println!("!b {}", !b);
    println!("!c {}", !c);
    println!("!b|!c {}", !b | !c);
    println!("!b|!c {}", RangeSetInt::from(!b.ranges() | !c.ranges()));

    let _a = RangeSetInt::<u8>::from("1..=6,8..=9,11..=15");
    let u = union_dyn!(a.ranges());
    assert_eq!(
        RangeSetInt::from(u),
        RangeSetInt::from("1..=6,8..=9,11..=15")
    );
    let u = union_dyn!(a.ranges(), b.ranges(), c.ranges());
    assert_eq!(
        RangeSetInt::from(u),
        RangeSetInt::from("1..=15,18..=29,38..=42")
    );

    let u = union([
        intersection_dyn!(a.ranges(), !b.ranges(), !c.ranges()),
        intersection_dyn!(!a.ranges(), b.ranges(), !c.ranges()),
        intersection_dyn!(!a.ranges(), !b.ranges(), c.ranges()),
        intersection_dyn!(a.ranges(), b.ranges(), c.ranges()),
    ]);
    assert_eq!(
        RangeSetInt::from(u),
        RangeSetInt::from("1..=4,7..=7,10..=10,14..=15,18..=29,38..=42")
    );
}

// cmk000
// #[test]
// fn sorted_disjoint_iter() {
//     let i: SortedDisjointFromIter<_> = [1, 3, 4, 2, 2, 43, 4, 5, 4, 23, 2, 43]
//         .into_iter()
//         .collect();
//     let (i0, i1) = i.tee();
//     let _j = i0.bitor(i1);
// }
