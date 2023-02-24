// !!!cmkRule add integration tests
#![cfg(test)]

use itertools::Itertools;
use std::collections::BTreeSet;
use syntactic_for::syntactic_for;

// !!!cmk should users use a prelude?
use range_set_int::{
    intersection_dyn, union_dyn, DynSortedDisjointExt, ItertoolsPlus2, RangeSetInt,
    SortedDisjointIterator,
};

#[test]
fn insert_255u8() {
    let range_set_int = RangeSetInt::from([255u8]);
    assert!(range_set_int.to_string() == "255..=255");
}

#[test]
#[should_panic]
fn insert_max_u128() {
    let _ = RangeSetInt::<u128>::from([u128::MAX]);
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
//         range_set_int.ranges_len().separate_with_commas()
//     );
//     println!(
//         "xTime elapsed in expensive_function() is: {} ms",
//         start.elapsed().as_millis()
//     );
// }

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

    let mut rs = range_set_int.ranges().not();
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

    // cmk0 list all the ways that we and BTreeMap does intersection. Do they make sense? Work when empty?
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

// cmk0 use merge in example
// cmk0 support 'from' not just 'from_sorted_disjoint_iter'
// cmk0 support 'collect' not just 'from'
// cmk much too easy to make errors -- need types!
// cmk0 need to be able to do a|b|c
// cmk type are very hard to read

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
    let a_less = a.ranges().sub(union_stream);
    let d: RangeSetInt<_> = a_less.into();
    println!("{d}");

    let d: RangeSetInt<_> = a.ranges().sub([b.ranges(), c.ranges()].union()).into();
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
    let _d = [a.ranges()].intersection();
    let _parity: RangeSetInt<u8> = [[a.ranges()].intersection()].union().into();
    let _parity: RangeSetInt<u8> = [a.ranges()].intersection().into();
    let _parity: RangeSetInt<u8> = [a.ranges()].union().into();
    println!("!b {}", !b);
    println!("!c {}", !c);
    println!("!b|!c {}", !b | !c);
    println!(
        "!b|!c {}",
        RangeSetInt::from(b.ranges().not().bitor(c.ranges().not()))
    );

    let _a = RangeSetInt::<u8>::from("1..=6,8..=9,11..=15");
    let u = [a.ranges().dyn_sorted_disjoint()].union();
    assert_eq!(
        RangeSetInt::from(u),
        RangeSetInt::from("1..=6,8..=9,11..=15")
    );
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

    let u = [
        intersection_dyn!(a.ranges(), b.ranges().not(), c.ranges().not()),
        intersection_dyn!(a.ranges().not(), b.ranges(), c.ranges().not()),
        intersection_dyn!(a.ranges().not(), b.ranges().not(), c.ranges()),
        intersection_dyn!(a.ranges(), b.ranges(), c.ranges()),
    ]
    .union();
    assert_eq!(
        RangeSetInt::from(u),
        RangeSetInt::from("1..=4,7..=7,10..=10,14..=15,18..=29,38..=42")
    );
}

#[test]
fn ui() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/ui/*.rs");
}

#[test]
fn complement2() {
    // RangeSetInt, Ranges, NotIter, BitOrIter
    let a0 = RangeSetInt::<u8>::from("1..=6");
    let a1 = RangeSetInt::from("8..=9,11..=15");
    let a = &a0 | &a1;
    let not_a = !&a;
    let b = a.ranges();
    let c = !not_a.ranges();
    let d = a0.ranges() | a1.ranges();
    let (e, _) = a.ranges().tee();
    let not_b = !b;
    let not_c = !c;
    let not_d = !d;
    let not_e = e.not();
    assert!(not_a.ranges().equal(not_b));
    assert!(not_a.ranges().equal(not_c));
    assert!(not_a.ranges().equal(not_d));
    assert!(not_a.ranges().equal(not_e));
}

#[test]
fn union() {
    // RangeSetInt, Ranges, NotIter, BitOrIter
    let a0 = RangeSetInt::<u8>::from("1..=6");
    let (a0_tee, _) = a0.ranges().tee();
    let a1 = RangeSetInt::<u8>::from("8..=9");
    let a2 = RangeSetInt::from("11..=15");
    let a12 = &a1 | &a2;
    let not_a0 = !&a0;
    let a = &a0 | &a1 | &a2;
    let b = a0.ranges() | a1.ranges() | a2.ranges();
    let c = !not_a0.ranges() | a12.ranges();
    let d = a0.ranges() | a1.ranges() | a2.ranges();
    let e = a0_tee.bitor(a12.ranges());
    assert!(a.ranges().equal(b));
    assert!(a.ranges().equal(c));
    assert!(a.ranges().equal(d));
    assert!(a.ranges().equal(e));
}

#[test]
fn sub() {
    // RangeSetInt, Ranges, NotIter, BitOrIter
    let a0 = RangeSetInt::<u8>::from("1..=6");
    let a1 = RangeSetInt::<u8>::from("8..=9");
    let a2 = RangeSetInt::from("11..=15");
    let a01 = &a0 | &a1;
    let (a01_tee, _) = a01.ranges().tee();
    let not_a01 = !&a01;
    let a = &a01 - &a2;
    let b = a01.ranges() - a2.ranges();
    let c = !not_a01.ranges() - a2.ranges();
    let d = (a0.ranges() | a1.ranges()) - a2.ranges();
    let e = a01_tee.sub(a2.ranges());
    assert!(a.ranges().equal(b));
    assert!(a.ranges().equal(c));
    assert!(a.ranges().equal(d));
    assert!(a.ranges().equal(e));
}

#[test]
fn xor() {
    // RangeSetInt, Ranges, NotIter, BitOrIter
    let a0 = RangeSetInt::<u8>::from("1..=6");
    let a1 = RangeSetInt::<u8>::from("8..=9");
    let a2 = RangeSetInt::from("11..=15");
    let a01 = &a0 | &a1;
    let (a01_tee, _) = a01.ranges().tee();
    let not_a01 = !&a01;
    let a = &a01 ^ &a2;
    let b = a01.ranges() ^ a2.ranges();
    let c = !not_a01.ranges() ^ a2.ranges();
    let d = (a0.ranges() | a1.ranges()) ^ a2.ranges();
    let e = a01_tee.bitxor(a2.ranges());
    assert!(a.ranges().equal(b));
    assert!(a.ranges().equal(c));
    assert!(a.ranges().equal(d));
    assert!(a.ranges().equal(e));
}

#[test]
fn bitand() {
    // RangeSetInt, Ranges, NotIter, BitOrIter
    let a0 = RangeSetInt::<u8>::from("1..=6");
    let a1 = RangeSetInt::<u8>::from("8..=9");
    let a2 = RangeSetInt::from("11..=15");
    let a01 = &a0 | &a1;
    let (a01_tee, _) = a01.ranges().tee();
    let not_a01 = !&a01;
    let a = &a01 & &a2;
    let b = a01.ranges() & a2.ranges();
    let c = !not_a01.ranges() & a2.ranges();
    let d = (a0.ranges() | a1.ranges()) & a2.ranges();
    let e = a01_tee.bitand(a2.ranges());
    assert!(a.ranges().equal(b));
    assert!(a.ranges().equal(c));
    assert!(a.ranges().equal(d));
    assert!(a.ranges().equal(e));
}
