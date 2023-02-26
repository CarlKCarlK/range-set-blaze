// !!!cmkRule add integration tests
#![cfg(test)]

use itertools::Itertools;
use std::{collections::BTreeSet, ops::BitOr};
use syntactic_for::syntactic_for;

// !!!cmk should users use a prelude? If not, are these reasonable imports?
use range_set_int::{
    intersection_dyn, union_dyn, BitOrIter, DynSortedDisjointExt, ItertoolsPlus2, RangeSetInt,
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
// cmk0 support 'collect' not just 'from'

// https://stackoverflow.com/questions/21747136/how-do-i-print-in-rust-the-type-of-a-variable/58119924#58119924
// fn print_type_of<T>(_: &T) {
//     println!("{}", std::any::type_name::<T>())
// }

#[test]
fn custom_multi() {
    let a = RangeSetInt::<u8>::from("1..=6,8..=9,11..=15");
    let b = RangeSetInt::<u8>::from("5..=13,18..=29");
    let c = RangeSetInt::<u8>::from("38..=42");

    let union_stream = b.ranges() | c.ranges();
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
    // !!!cmk0 time itertools.tee (?) vs range.clone()
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
        RangeSetInt::from(b.ranges().not() | c.ranges().not())
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
fn complement() {
    // RangeSetInt, Ranges, NotIter, BitOrIter, Tee, BitOrIter(g)
    let a0 = RangeSetInt::<u8>::from("1..=6");
    let a1 = RangeSetInt::from("8..=9,11..=15");
    let a = &a0 | &a1;
    let not_a = !&a;
    let b = a.ranges();
    let c = !not_a.ranges();
    let d = a0.ranges() | a1.ranges();
    let (e, _) = a.ranges().tee();
    let f: BitOrIter<_, _> = [15, 14, 15, 13, 12, 11, 9, 9, 8, 6, 4, 5, 3, 2, 1, 1, 1]
        .into_iter()
        .collect();
    let not_b = !b;
    let not_c = !c;
    let not_d = !d;
    let not_e = e.not();
    let not_f = !f;
    assert!(not_a.ranges().equal(not_b));
    assert!(not_a.ranges().equal(not_c));
    assert!(not_a.ranges().equal(not_d));
    assert!(not_a.ranges().equal(not_e));
    assert!(not_a.ranges().equal(not_f));
}

#[test]
fn union() {
    // RangeSetInt, Ranges, NotIter, BitOrIter, Tee, BitOrIter(g)
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
    let f = a0.iter().collect::<BitOrIter<_, _>>()
        | a1.iter().collect::<BitOrIter<_, _>>()
        | a2.iter().collect::<BitOrIter<_, _>>();
    assert!(a.ranges().equal(b));
    assert!(a.ranges().equal(c));
    assert!(a.ranges().equal(d));
    assert!(a.ranges().equal(e));
    assert!(a.ranges().equal(f));
}

#[test]
fn sub() {
    // RangeSetInt, Ranges, NotIter, BitOrIter, Tee, BitOrIter(g)
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
    let f = a01.iter().collect::<BitOrIter<_, _>>() - a2.iter().collect::<BitOrIter<_, _>>();
    assert!(a.ranges().equal(b));
    assert!(a.ranges().equal(c));
    assert!(a.ranges().equal(d));
    assert!(a.ranges().equal(e));
    assert!(a.ranges().equal(f));
}

#[test]
fn xor() {
    // RangeSetInt, Ranges, NotIter, BitOrIter, Tee, BitOrIter(g)
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
    let f = a01.iter().collect::<BitOrIter<_, _>>() ^ a2.iter().collect::<BitOrIter<_, _>>();
    assert!(a.ranges().equal(b));
    assert!(a.ranges().equal(c));
    assert!(a.ranges().equal(d));
    assert!(a.ranges().equal(e));
    assert!(a.ranges().equal(f));
}

#[test]
fn bitand() {
    // RangeSetInt, Ranges, NotIter, BitOrIter, Tee, BitOrIter(g)
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
    let f = a01.iter().collect::<BitOrIter<_, _>>() & a2.iter().collect::<BitOrIter<_, _>>();
    assert!(a.ranges().equal(b));
    assert!(a.ranges().equal(c));
    assert!(a.ranges().equal(d));
    assert!(a.ranges().equal(e));
    assert!(a.ranges().equal(f));
}

#[test]
fn empty() {
    // let a0 = RangeSetInt::<u8>::from("1..=6");
    let a = [1, 2, 3].into_iter().collect::<RangeSetInt<i32>>();
    let b = RangeSetInt::from([2, 3, 4]);
    let b2 = 2;
    let b3 = 3;
    let b4 = 4;
    let b_ref = [&b2, &b3, &b4];
    let mut c3 = a.clone();
    let mut c4 = a.clone();
    let mut c5 = a.clone();

    let c0 = (&a).bitor(&b);
    let c1a = &a | &b;
    let c1b = &a | b.clone();
    let c1c = a.clone() | &b;
    let c1d = a.clone() | b.clone();
    let c2: RangeSetInt<_> = (a.ranges() | b.ranges()).into();
    c3.append(&mut b.clone());
    c4.extend(b_ref);
    c5.extend(b);

    let answer = RangeSetInt::from([1, 2, 3, 4]);
    assert_eq!(&c0, &answer);
    assert_eq!(&c1a, &answer);
    assert_eq!(&c1b, &answer);
    assert_eq!(&c1c, &answer);
    assert_eq!(&c1d, &answer);
    assert_eq!(&c2, &answer);
    assert_eq!(&c3, &answer);
    assert_eq!(&c4, &answer);
    assert_eq!(&c5, &answer);

    use range_set_int::ItertoolsPlus2;
    let a = [1, 2, 3].into_iter().collect::<RangeSetInt<i32>>();
    let b = RangeSetInt::from([2, 3, 4]);

    let c0 = a.ranges() | b.ranges();
    let c1 = range_set_int::union([a.ranges(), b.ranges()]);
    let c2 = [a.ranges(), b.ranges()].union();
    let c3 = union_dyn!(a.ranges(), b.ranges());
    let c4 = [a.ranges(), b.ranges()]
        .map(|x| x.dyn_sorted_disjoint())
        .union();

    let answer = RangeSetInt::from([1, 2, 3, 4]);
    assert!(c0.equal(answer.ranges()));
    assert!(c1.equal(answer.ranges()));
    assert!(c2.equal(answer.ranges()));
    assert!(c3.equal(answer.ranges()));
    assert!(c4.equal(answer.ranges()));
}
