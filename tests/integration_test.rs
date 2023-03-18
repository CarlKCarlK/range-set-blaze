// !!!cmkRule add integration tests
#![cfg(test)]

use criterion::{BatchSize, BenchmarkId, Criterion};
use itertools::Itertools;
use rand::rngs::StdRng;
use rand::SeedableRng;
use range_set_int::{CheckSortedDisjoint, DynSortedDisjoint, NotIter, UnionIter};
use std::ops::RangeInclusive;
use std::{collections::BTreeSet, ops::BitOr};
use syntactic_for::syntactic_for;
use tests_common::{k_sets, width_to_range, How, MemorylessIter, MemorylessRange};

// !!!cmk should users use a prelude? If not, are these reasonable imports?
use range_set_int::{
    intersection_dyn, union_dyn, AssumeSortedStarts, Integer, MultiwayRangeSetInt,
    MultiwaySortedDisjoint, RangeSetInt, RangesIter, SortedDisjointIterator,
};

#[test]
fn insert_255u8() {
    let range_set_int = RangeSetInt::from_iter([255u8]);
    assert!(range_set_int.to_string() == "255..=255");
}

#[test]
#[should_panic]
fn insert_max_u128() {
    let _ = RangeSetInt::<u128>::from_iter([u128::MAX]);
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
fn doctest3() -> Result<(), Box<dyn std::error::Error>> {
    let mut a = RangeSetInt::from_iter([1u8..=3]);
    let mut b = RangeSetInt::from_iter([3u8..=5]);

    a.append(&mut b);

    assert_eq!(a.len(), 5usize);
    assert_eq!(b.len(), 0usize);

    assert!(a.contains(1));
    assert!(a.contains(2));
    assert!(a.contains(3));
    assert!(a.contains(4));
    assert!(a.contains(5));
    Ok(())
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
fn iters() -> Result<(), Box<dyn std::error::Error>> {
    let range_set_int = RangeSetInt::from_iter([1u8..=6, 8..=9, 11..=15]);
    assert!(range_set_int.len() == 13);
    // !!!cmk0
    // assert!(range_set_int.ranges.len() == 3);
    // // !!!cmk0 i is &u8
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

    let mut rs = range_set_int.ranges().not();
    println!("{:?}", rs.next());
    println!("{range_set_int}");
    // !!! assert that can't use range_set_int again
    Ok(())
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
fn multi_op() -> Result<(), Box<dyn std::error::Error>> {
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

    // !!!cmk0 must with on empty, with ref and with owned

    let _ = [&a, &b, &c].union();
    let d = [a, b, c].intersection();
    assert_eq!(d, RangeSetInt::new());

    assert_eq!(
        !MultiwayRangeSetInt::<u8>::union([]),
        RangeSetInt::from_iter([0..=255])
    );

    let a = RangeSetInt::from_iter([1..=6, 8..=9, 11..=15]);
    let b = RangeSetInt::from_iter([5..=13, 18..=29]);
    let c = RangeSetInt::from_iter([1..=42]);

    // cmk0 list all the ways that we and BTreeMap does intersection. Do they make sense? Work when empty?
    let _ = &a & &b;
    let d = [&a, &b, &c].intersection();
    // let d = RangeSetInt::intersection([a, b, c]);
    println!("{d}");
    assert_eq!(d, RangeSetInt::from_iter([5..=6, 8..=9, 11..=13]));

    assert_eq!(
        MultiwayRangeSetInt::<u8>::intersection([]),
        RangeSetInt::from_iter([0..=255])
    );
    Ok(())
}

// cmk0 use merge in example
// cmk0 support 'collect' not just 'from'

// https://stackoverflow.com/questions/21747136/how-do-i-print-in-rust-the-type-of-a-variable/58119924#58119924
// fn print_type_of<T>(_: &T) {
//     println!("{}", std::any::type_name::<T>())
// }

#[test]
fn custom_multi() -> Result<(), Box<dyn std::error::Error>> {
    let a = RangeSetInt::from_iter([1..=6, 8..=9, 11..=15]);
    let b = RangeSetInt::from_iter([5..=13, 18..=29]);
    let c = RangeSetInt::from_iter([38..=42]);

    let union_stream = b.ranges() | c.ranges();
    let a_less = a.ranges().sub(union_stream);
    let d: RangeSetInt<_> = a_less.into();
    println!("{d}");

    let d: RangeSetInt<_> = a.ranges().sub([b.ranges(), c.ranges()].union()).into();
    println!("{d}");
    Ok(())
}

#[test]
fn from_string() -> Result<(), Box<dyn std::error::Error>> {
    let a = RangeSetInt::from_iter([0..=4, 14..=17, 30..=255, 0..=37, 43..=65535]);
    assert_eq!(a, RangeSetInt::from_iter([0..=65535]));
    Ok(())
}

#[test]
fn nand_repro() -> Result<(), Box<dyn std::error::Error>> {
    let b = &RangeSetInt::from_iter([5u8..=13, 18..=29]);
    let c = &RangeSetInt::from_iter([38..=42]);
    println!("about to nand");
    let d = !b | !c;
    println!("cmk '{d}'");
    assert_eq!(
        d,
        RangeSetInt::from_iter([0..=4, 14..=17, 30..=255, 0..=37, 43..=255])
    );
    Ok(())
}

#[test]
fn parity() -> Result<(), Box<dyn std::error::Error>> {
    let a = &RangeSetInt::from_iter([1..=6, 8..=9, 11..=15]);
    let b = &RangeSetInt::from_iter([5..=13, 18..=29]);
    let c = &RangeSetInt::from_iter([38..=42]);
    // !!!cmk0 time itertools.tee (?) vs range.clone()
    // !!!cmk explain why need both "Merge" with "KMerge"
    // !!!cmk0 empty needs to work. Go back to slices?
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
    println!(
        "!b|!c {}",
        RangeSetInt::from(b.ranges().not() | c.ranges().not())
    );

    let _a = RangeSetInt::from_iter([1..=6, 8..=9, 11..=15]);
    let u = [DynSortedDisjoint::new(a.ranges())].union();
    assert_eq!(
        RangeSetInt::from(u),
        RangeSetInt::from_iter([1..=6, 8..=9, 11..=15])
    );
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
        intersection_dyn!(a.ranges(), b.ranges().not(), c.ranges().not()),
        intersection_dyn!(a.ranges().not(), b.ranges(), c.ranges().not()),
        intersection_dyn!(a.ranges().not(), b.ranges().not(), c.ranges()),
        intersection_dyn!(a.ranges(), b.ranges(), c.ranges()),
    ]
    .union();
    assert_eq!(
        RangeSetInt::from(u),
        RangeSetInt::from_iter([1..=4, 7..=7, 10..=10, 14..=15, 18..=29, 38..=42])
    );
    Ok(())
}

#[test]
fn ui() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/ui/*.rs");
}

#[test]
fn complement() -> Result<(), Box<dyn std::error::Error>> {
    // RangeSetInt, RangesIter, NotIter, UnionIter, Tee, UnionIter(g)
    let a0 = RangeSetInt::from_iter([1..=6]);
    let a1 = RangeSetInt::from_iter([8..=9, 11..=15]);
    let a = &a0 | &a1;
    let not_a = !&a;
    let b = a.ranges();
    let c = !not_a.ranges();
    let d = a0.ranges() | a1.ranges();
    let (e, _) = a.ranges().tee();

    let f = UnionIter::from([15, 14, 15, 13, 12, 11, 9, 9, 8, 6, 4, 5, 3, 2, 1, 1, 1]);
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
    Ok(())
}

#[test]
fn union_test() -> Result<(), Box<dyn std::error::Error>> {
    // RangeSetInt, RangesIter, NotIter, UnionIter, Tee, UnionIter(g)
    let a0 = RangeSetInt::from_iter([1..=6]);
    let (a0_tee, _) = a0.ranges().tee();
    let a1 = RangeSetInt::from_iter([8..=9]);
    let a2 = RangeSetInt::from_iter([11..=15]);
    let a12 = &a1 | &a2;
    let not_a0 = !&a0;
    let a = &a0 | &a1 | &a2;
    let b = a0.ranges() | a1.ranges() | a2.ranges();
    let c = !not_a0.ranges() | a12.ranges();
    let d = a0.ranges() | a1.ranges() | a2.ranges();
    let e = a0_tee.bitor(a12.ranges());

    let f = UnionIter::from_iter(a0.iter())
        | UnionIter::from_iter(a1.iter())
        | UnionIter::from_iter(a2.iter());
    assert!(a.ranges().equal(b));
    assert!(a.ranges().equal(c));
    assert!(a.ranges().equal(d));
    assert!(a.ranges().equal(e));
    assert!(a.ranges().equal(f));
    Ok(())
}

#[test]
fn sub() -> Result<(), Box<dyn std::error::Error>> {
    // RangeSetInt, RangesIter, NotIter, UnionIter, Tee, UnionIter(g)
    let a0 = RangeSetInt::from_iter([1..=6]);
    let a1 = RangeSetInt::from_iter([8..=9]);
    let a2 = RangeSetInt::from_iter([11..=15]);
    let a01 = &a0 | &a1;
    let (a01_tee, _) = a01.ranges().tee();
    let not_a01 = !&a01;
    let a = &a01 - &a2;
    let b = a01.ranges() - a2.ranges();
    let c = !not_a01.ranges() - a2.ranges();
    let d = (a0.ranges() | a1.ranges()) - a2.ranges();
    let e = a01_tee.sub(a2.ranges());
    let f = UnionIter::from_iter(a01.iter()) - UnionIter::from_iter(a2.iter());
    assert!(a.ranges().equal(b));
    assert!(a.ranges().equal(c));
    assert!(a.ranges().equal(d));
    assert!(a.ranges().equal(e));
    assert!(a.ranges().equal(f));

    Ok(())
}

#[test]
fn xor() -> Result<(), Box<dyn std::error::Error>> {
    // RangeSetInt, RangesIter, NotIter, UnionIter, Tee, UnionIter(g)
    let a0 = RangeSetInt::from_iter([1..=6]);
    let a1 = RangeSetInt::from_iter([8..=9]);
    let a2 = RangeSetInt::from_iter([11..=15]);
    let a01 = &a0 | &a1;
    let (a01_tee, _) = a01.ranges().tee();
    let not_a01 = !&a01;
    let a = &a01 ^ &a2;
    let b = a01.ranges() ^ a2.ranges();
    let c = !not_a01.ranges() ^ a2.ranges();
    let d = (a0.ranges() | a1.ranges()) ^ a2.ranges();
    let e = a01_tee.bitxor(a2.ranges());
    let f = UnionIter::from_iter(a01.iter()) ^ UnionIter::from_iter(a2.iter());
    assert!(a.ranges().equal(b));
    assert!(a.ranges().equal(c));
    assert!(a.ranges().equal(d));
    assert!(a.ranges().equal(e));
    assert!(a.ranges().equal(f));
    Ok(())
}

#[test]
fn bitand() -> Result<(), Box<dyn std::error::Error>> {
    // RangeSetInt, RangesIter, NotIter, UnionIter, Tee, UnionIter(g)
    let a0 = RangeSetInt::from_iter([1..=6]);
    let a1 = RangeSetInt::from_iter([8..=9]);
    let a2 = RangeSetInt::from_iter([11..=15]);
    let a01 = &a0 | &a1;
    let (a01_tee, _) = a01.ranges().tee();
    let not_a01 = !&a01;
    let a = &a01 & &a2;
    let b = a01.ranges() & a2.ranges();
    let c = !not_a01.ranges() & a2.ranges();
    let d = (a0.ranges() | a1.ranges()) & a2.ranges();
    let e = a01_tee.bitand(a2.ranges());
    let f = UnionIter::from_iter(a01.iter()) & UnionIter::from_iter(a2.iter());
    assert!(a.ranges().equal(b));
    assert!(a.ranges().equal(c));
    assert!(a.ranges().equal(d));
    assert!(a.ranges().equal(e));
    assert!(a.ranges().equal(f));
    Ok(())
}

// // !!!cmk should each type have a .universe() and .empty() method? e.g. 0..=255 for u8
#[test]
fn empty_it() {
    let universe = RangeSetInt::from_iter([0u8..=255]);
    let universe = universe.ranges();
    let arr: [u8; 0] = [];
    let a0 = RangeSetInt::<u8>::from_iter(arr);
    assert!(!(a0.ranges()).equal(universe.clone()));
    assert!((!a0).ranges().equal(universe));
    let _a0 = RangeSetInt::from_iter([0..=0; 0]);
    let _a = RangeSetInt::<i32>::new();

    let a_iter: std::array::IntoIter<i32, 0> = [].into_iter();
    let a = a_iter.collect::<RangeSetInt<i32>>();
    let b = RangeSetInt::from_iter([0i32; 0]);
    let b_ref: [&i32; 0] = [];
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

    let answer = RangeSetInt::from_iter([0; 0]);
    assert_eq!(&c0, &answer);
    assert_eq!(&c1a, &answer);
    assert_eq!(&c1b, &answer);
    assert_eq!(&c1c, &answer);
    assert_eq!(&c1d, &answer);
    assert_eq!(&c2, &answer);
    assert_eq!(&c3, &answer);
    assert_eq!(&c4, &answer);
    assert_eq!(&c5, &answer);

    let a_iter: std::array::IntoIter<i32, 0> = [].into_iter();
    let a = a_iter.collect::<RangeSetInt<i32>>();
    let b = RangeSetInt::from_iter([0; 0]);

    let c0 = a.ranges() | b.ranges();
    let c1 = [a.ranges(), b.ranges()].union();
    let c_list2: [RangesIter<i32>; 0] = [];
    let c2 = c_list2.clone().union();
    let c3 = union_dyn!(a.ranges(), b.ranges());
    let c4 = c_list2.map(DynSortedDisjoint::new).union();

    let answer = RangeSetInt::from_iter([0; 0]);
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

    let answer = !RangeSetInt::from_iter([0; 0]);
    assert!(c0.equal(answer.ranges()));
    assert!(c1.equal(answer.ranges()));
    assert!(c2.equal(answer.ranges()));
    assert!(c3.equal(answer.ranges()));
    assert!(c4.equal(answer.ranges()));
}

#[test]
#[allow(clippy::reversed_empty_ranges)]
fn tricky_case1() {
    let a = RangeSetInt::from_iter([1..=0]);
    let b = RangeSetInt::from_iter([2..=1]);
    assert_eq!(a, b);
    assert!(a.ranges().equal(b.ranges()));
    assert_eq!(a.ranges().len(), 0);
    assert_eq!(a.ranges().len(), b.ranges().len());
    let a = RangeSetInt::from_iter([i32::MIN..=i32::MAX]);
    println!("tc1 '{a}'");
    assert_eq!(a.len() as i128, (i32::MAX as i128) - (i32::MIN as i128) + 1);
    let a = !RangeSetInt::from_iter([1..=0]);
    println!("tc1 '{a}'");
    assert_eq!(a.len() as i128, (i32::MAX as i128) - (i32::MIN as i128) + 1);

    let a = !RangeSetInt::from_iter([1i128..=0]);
    println!("tc1 '{a}', {}", a.len());
    assert_eq!(a.len(), u128::MAX);
    let a = !RangeSetInt::from_iter([1u128..=0]);
    println!("tc1 '{a}', {}", a.len());
    assert_eq!(a.len(), u128::MAX);
}

// should fail
#[test]
#[should_panic]
fn tricky_case2() {
    let _a = RangeSetInt::from_iter([-1..=i128::MAX]);
}

#[test]
#[should_panic]
fn tricky_case3() {
    let _a = RangeSetInt::from_iter([0..=u128::MAX]);
}

#[test]
fn constructors() -> Result<(), Box<dyn std::error::Error>> {
    // #9: new
    let mut _range_set_int;
    _range_set_int = RangeSetInt::<i32>::new();
    // #10 collect / from_iter T
    _range_set_int = [1, 5, 6, 5].into_iter().collect();
    _range_set_int = RangeSetInt::from_iter([1, 5, 6, 5]);
    // #11 into / from array T
    _range_set_int = [1, 5, 6, 5].into();
    _range_set_int = RangeSetInt::from_iter([1, 5, 6, 5]);
    // #12 into / from slice T
    // _range_set_int = [1, 5, 6, 5][1..=2].into();
    // _range_set_int = RangeSetInt::from_iter([1, 5, 6, 5].as_slice());
    //#13 collect / from_iter range
    _range_set_int = [5..=6, 1..=5].into_iter().collect();
    _range_set_int = RangeSetInt::from_iter([5..=6, 1..=5]);
    // #14 from into array range
    // cmk00000
    // _range_set_int = [5..=6, 1..=5].into();
    // _range_set_int = RangeSetInt::from_iter([5..=6, 1..=5]);
    // #15 from into slice range
    // cmk00000
    // _range_set_int = [5..=6, 1..=5][0..=1].into();
    // _range_set_int = RangeSetInt::from_iter([5..=6, 1..=5].as_slice());
    // #16 into / from iter (T,T) + SortedDisjoint
    _range_set_int = _range_set_int.ranges().into();
    _range_set_int = RangeSetInt::from(_range_set_int.ranges());
    // try_into / try_from string cmk`
    // cmk00000
    // _range_set_int = [5..=6, 1..=5].into();
    // _range_set_int = RangeSetInt::from_iter([5..=6, 1..=5]);

    // !!!cmk0 delete? move to test.rs?
    let sorted_starts = AssumeSortedStarts::new([1..=5, 6..=10].into_iter());
    let mut _sorted_disjoint_iter;
    _sorted_disjoint_iter = UnionIter::new(sorted_starts);
    // #10 collect / from_iter T
    let mut _sorted_disjoint_iter: UnionIter<_, _> = [1, 5, 6, 5].into_iter().collect();
    _sorted_disjoint_iter = UnionIter::from_iter([1, 5, 6, 5]);
    // // #11 into / from array T
    _sorted_disjoint_iter = [1, 5, 6, 5].into();
    _sorted_disjoint_iter = UnionIter::from([1, 5, 6, 5]);
    // // #12 into / from slice T
    _sorted_disjoint_iter = [1, 5, 6, 5][1..=2].into();
    _sorted_disjoint_iter = UnionIter::from([1, 5, 6, 5].as_slice());
    // //#13 collect / from_iter range
    _sorted_disjoint_iter = [5..=6, 1..=5].into_iter().collect();
    _sorted_disjoint_iter = UnionIter::from_iter([5..=6, 1..=5]);
    // // #14 from into array range
    _sorted_disjoint_iter = [5..=6, 1..=5].into();
    _sorted_disjoint_iter = UnionIter::from([5..=6, 1..=5]);
    // // #15 from into slice range
    _sorted_disjoint_iter = [5..=6, 1..=5][0..=1].into();
    _sorted_disjoint_iter = UnionIter::from([5..=6, 1..=5].as_slice());
    // // #16 into / from iter (T,T) + SortedDisjoint
    let mut _sorted_disjoint_iter: UnionIter<_, _> = _range_set_int.ranges().collect();
    _sorted_disjoint_iter = UnionIter::from_iter(_range_set_int.ranges());
    // // // try_into / try_from string cmk
    // // _range_set_int = [5..=6, 1..=5].into();
    // // _range_set_int = UnionIter::from([5..=6, 1..=5]);

    Ok(())
}

#[test]
fn debug_k_play() {
    let mut c = Criterion::default();
    k_play(&mut c);
}

fn k_play(c: &mut Criterion) {
    let range = 0..=9_999_999;
    let range_len = 1000; //1_000;
    let coverage_goal = 0.50;

    let mut group = c.benchmark_group("k_play");
    {
        let k = &25;
        // group.throughput(Throughput::Bytes(*size as u64));
        group.bench_with_input(BenchmarkId::new("dyn", k), k, |b, &k| {
            b.iter_batched(
                || {
                    k_sets(
                        k,
                        range_len,
                        &range,
                        coverage_goal,
                        How::Intersection,
                        &mut StdRng::seed_from_u64(0),
                    )
                },
                |sets| {
                    let sets = sets.iter().map(|x| DynSortedDisjoint::new(x.ranges()));
                    let _answer: RangeSetInt<_> = sets.intersection().into();
                },
                BatchSize::SmallInput,
            );
        });
    }
    group.finish();
}

#[test]
fn data_gen() {
    let range = -10_000_000i32..=10_000_000;
    let range_len = 1000;
    let coverage_goal = 0.75;
    let k = 100;

    for how in [How::None, How::Union, How::Intersection] {
        let mut option_range_int_set: Option<RangeSetInt<_>> = None;
        for seed in 0..k as u64 {
            let r2: RangeSetInt<i32> = MemorylessRange::new(
                &mut StdRng::seed_from_u64(seed),
                range_len,
                range.clone(),
                coverage_goal,
                k,
                how,
            )
            .collect();
            option_range_int_set = Some(if let Some(range_int_set) = &option_range_int_set {
                match how {
                    How::Intersection => range_int_set & r2,
                    How::Union => range_int_set | r2,
                    How::None => r2,
                }
            } else {
                r2
            });
            let range_int_set = option_range_int_set.as_ref().unwrap();
            println!(
                "range_int_set.len={}, ri={:#?}, how={how:#?} {seed} range_len={}, fraction={}",
                range_int_set.len(),
                &range,
                range_int_set.ranges_len(),
                fraction(range_int_set, &range)
            );
        }
        let range_int_set = option_range_int_set.unwrap();
        let fraction = fraction(&range_int_set, &range);
        println!("how={how:#?}, goal={coverage_goal}, fraction={fraction}");
        assert!(coverage_goal * 0.95 < fraction && fraction < coverage_goal * 1.05);
        // Don't check this because of known off-by-one-error that don't matter in practice.
        // let first = range_int_set.first().unwrap();
        // let last = range_int_set.last().unwrap();
        // println!("first={first}, last={last}, range={range:#?}");
        // assert!(first >= *range.start());
        // assert!(last <= *range.end());
    }
}

#[test]
fn vary_coverage_goal() {
    let k = 2;
    let range_len = 1_000;
    let range = 0..=99_999_999;
    let coverage_goal_list = [0.01, 0.1, 0.25, 0.5, 0.75, 0.9, 0.99];
    let setup_vec = coverage_goal_list
        .iter()
        .map(|coverage_goal| {
            (
                coverage_goal,
                k_sets(
                    k,
                    range_len,
                    &range,
                    *coverage_goal,
                    How::None,
                    &mut StdRng::seed_from_u64(0),
                ),
            )
        })
        .collect::<Vec<_>>();

    for (range_len, sets) in &setup_vec {
        let parameter = *range_len;

        let answer = &sets[0] | &sets[1];
        let fraction_val = fraction(&answer, &range);
        println!(
            "u: {parameter}, {fraction_val}, {}+{}={}",
            sets[0].ranges_len(),
            sets[1].ranges_len(),
            answer.ranges_len()
        );
        let answer = &sets[0] & &sets[1];
        let fraction_val = fraction(&answer, &range);
        println!(
            "i: {parameter}, {fraction_val}, {}+{}={}",
            sets[0].ranges_len(),
            sets[1].ranges_len(),
            answer.ranges_len()
        );
    }
}

#[test]
fn vs_btree_set() {
    let k = 1;
    let average_width_list = [2, 1, 3, 4, 5, 10, 100, 1000, 10_000, 100_000, 1_000_000];
    let coverage_goal = 0.10;
    let assert_tolerance = 0.005;
    let how = How::None;
    let seed = 0;
    let iter_len = 1_000_000;

    println!(
        "{:#?},{:#?},{:#?},{:#?},{:#?},{:#?},{:#?},{:#?},{:#?},{:#?}",
        "seed",
        "average_width",
        "coverage_goal",
        "iter_len",
        "range",
        "range_count_with_dups",
        "item_count_with_dups",
        "range_count_without_dups",
        "item_count_without_dups",
        "fraction",
    );

    for average_width in average_width_list {
        let (range_len, range) = width_to_range(iter_len, average_width, coverage_goal);

        let mut rng = StdRng::seed_from_u64(seed);
        let memoryless_range =
            MemorylessRange::new(&mut rng, range_len, range.clone(), coverage_goal, k, how);
        let range_count_with_dups = memoryless_range.count();
        let mut rng = StdRng::seed_from_u64(seed);
        let memoryless_iter =
            MemorylessIter::new(&mut rng, range_len, range.clone(), coverage_goal, k, how);
        let item_count_with_dups = memoryless_iter.count();
        let mut rng = StdRng::seed_from_u64(seed);
        let range_set_int: RangeSetInt<_> =
            MemorylessRange::new(&mut rng, range_len, range.clone(), coverage_goal, k, how)
                .collect();

        let range_count_no_dups = range_set_int.ranges_len();
        let item_count_no_dups = range_set_int.len();
        let fraction_value = fraction(&range_set_int, &range);
        println!(
            "{:#?},{:#?},{:#?},{:#?},{:#?},{:#?},{:#?},{:#?},{:#?},{:#?}",
            seed,
            average_width,
            coverage_goal,
            iter_len,
            range.end() + 1,
            range_count_with_dups,
            item_count_with_dups,
            range_count_no_dups,
            item_count_no_dups,
            fraction_value
        );
        assert!((fraction_value - coverage_goal).abs() < assert_tolerance);

        // count of iter with dups
        // count of iter without dups
        // range with dups
        // range without dups
        // fraction
    }
}

#[test]
fn doc_test_insert1() {
    let mut set = RangeSetInt::new();

    assert!(set.insert(2));
    assert!(!set.insert(2));
    assert_eq!(set.len(), 1usize);
}

#[test]
fn doc_test_len() {
    let mut v = RangeSetInt::new();
    assert_eq!(v.len(), 0usize);
    v.insert(1);
    assert_eq!(v.len(), 1usize);

    let v = RangeSetInt::from_iter([
        -170_141_183_460_469_231_731_687_303_715_884_105_728i128..=10,
        -10..=170_141_183_460_469_231_731_687_303_715_884_105_726,
    ]);
    assert_eq!(
        v.len(),
        340_282_366_920_938_463_463_374_607_431_768_211_455u128
    );
}

#[test]
fn test_pops() {
    let mut set = RangeSetInt::from_iter([1..=2, 4..=5, 10..=11]);
    let len = set.len();
    assert_eq!(set.pop_first(), Some(1));
    assert_eq!(set.len(), len - 1usize);
    assert_eq!(set, RangeSetInt::from_iter([2..=2, 4..=5, 10..=11]));
    assert_eq!(set.pop_last(), Some(11));
    println!("{set:#?}");
    assert_eq!(set, RangeSetInt::from_iter([2..=2, 4..=5, 10..=10]));
    assert_eq!(set.len(), len - 2usize);
    assert_eq!(set.pop_last(), Some(10));
    assert_eq!(set.len(), len - 3usize);
    assert_eq!(set, RangeSetInt::from_iter([2..=2, 4..=5]));
    assert_eq!(set.pop_first(), Some(2));
    assert_eq!(set.len(), len - 4usize);
    assert_eq!(set, RangeSetInt::from_iter([4..=5]));
}

#[test]
fn eq() {
    assert!(RangeSetInt::from_iter([0, 2]) > RangeSetInt::from_iter([0, 1]));
    assert!(RangeSetInt::from_iter([0, 2]) > RangeSetInt::from_iter([0..=100]));
    assert!(RangeSetInt::from_iter([2..=2]) > RangeSetInt::from_iter([1..=2]));
    for use_0 in [false, true] {
        for use_1 in [false, true] {
            for use_2 in [false, true] {
                for use_3 in [false, true] {
                    for use_4 in [false, true] {
                        for use_5 in [false, true] {
                            let mut a = RangeSetInt::new();
                            let mut b = RangeSetInt::new();
                            if use_0 {
                                a.insert(0);
                            };
                            if use_1 {
                                a.insert(1);
                            };
                            if use_2 {
                                a.insert(2);
                            };
                            if use_3 {
                                b.insert(0);
                            };
                            if use_4 {
                                b.insert(1);
                            };
                            if use_5 {
                                b.insert(2);
                            };
                            let a2: BTreeSet<_> = a.iter().collect();
                            let b2: BTreeSet<_> = b.iter().collect();
                            assert!((a == b) == (a2 == b2));
                            println!("{a:?} <= {b:?}? RSI {}", a <= b);
                            println!("{a:?} <= {b:?}? BTS {}", a2 <= b2);
                            assert!((a <= b) == (a2 <= b2));
                            assert!((a < b) == (a2 < b2));
                        }
                    }
                }
            }
        }
    }
}

#[test]
fn insert2() {
    let set = RangeSetInt::from_iter([1..=2, 4..=5, 10..=20, 30..=30]);
    for insert in 0..=31 {
        println!("inserting  {insert}");
        let mut a = set.clone();
        let mut a2: BTreeSet<_> = a.iter().collect();
        let b2 = a2.insert(insert);
        let b = a.insert(insert);
        assert_eq!(a, RangeSetInt::from_iter(a2.iter().cloned()));
        assert_eq!(b, b2);
    }
}

#[test]
fn remove() {
    let mut set = RangeSetInt::from_iter([1..=2, 4..=5, 10..=11]);
    let len = set.len();
    assert!(set.remove(4));
    assert_eq!(set.len(), len - 1usize);
    assert_eq!(set, RangeSetInt::from_iter([1..=2, 5..=5, 10..=11]));
    assert!(!set.remove(4));
    assert_eq!(set.len(), len - 1usize);
    assert_eq!(set, RangeSetInt::from_iter([1..=2, 5..=5, 10..=11]));
    assert!(set.remove(5));
    assert_eq!(set.len(), len - 2usize);
    assert_eq!(set, RangeSetInt::from_iter([1..=2, 10..=11]));

    let mut set = RangeSetInt::from_iter([1..=2, 4..=5, 10..=100, 1000..=1000]);
    let len = set.len();
    assert!(!set.remove(0));
    assert_eq!(set.len(), len);
    assert!(!set.remove(3));
    assert_eq!(set.len(), len);
    assert!(set.remove(2));
    assert_eq!(set.len(), len - 1usize);
    assert!(set.remove(1000));
    assert_eq!(set.len(), len - 2usize);
    assert!(set.remove(10));
    assert_eq!(set.len(), len - 3usize);
    assert!(set.remove(50));
    assert_eq!(set.len(), len - 4usize);
    assert_eq!(
        set,
        RangeSetInt::from_iter([1..=1, 4..=5, 11..=49, 51..=100])
    );
}

#[test]
fn remove2() {
    let set = RangeSetInt::from_iter([1..=2, 4..=5, 10..=20, 30..=30]);
    for remove in 0..=31 {
        println!("removing  {remove}");
        let mut a = set.clone();
        let mut a2: BTreeSet<_> = a.iter().collect();
        let b2 = a2.remove(&remove);
        let b = a.remove(remove);
        assert_eq!(a, RangeSetInt::from_iter(a2.iter().cloned()));
        assert_eq!(b, b2);
    }
    let set = RangeSetInt::new();
    for remove in 0..=0 {
        println!("removing  {remove}");
        let mut a = set.clone();
        let mut a2: BTreeSet<_> = a.iter().collect();
        let b2 = a2.remove(&remove);
        let b = a.remove(remove);
        assert_eq!(a, RangeSetInt::from_iter(a2.iter().cloned()));
        assert_eq!(b, b2);
    }
}

#[test]
fn split_off() {
    let set = RangeSetInt::from_iter([1..=2, 4..=5, 10..=20, 30..=30]);
    for split in 0..=31 {
        println!("splitting at {split}");
        let mut a = set.clone();
        let mut a2: BTreeSet<_> = a.iter().collect();
        let b2 = a2.split_off(&split);
        let b = a.split_off(split);
        assert_eq!(a, RangeSetInt::from_iter(a2.iter().cloned()));
        assert_eq!(b, RangeSetInt::from_iter(b2.iter().cloned()));
    }
    let set = RangeSetInt::new();
    for split in 0..=0 {
        println!("splitting at {split}");
        let mut a = set.clone();
        let mut a2: BTreeSet<_> = a.iter().collect();
        let b2 = a2.split_off(&split);
        let b = a.split_off(split);
        assert_eq!(a, RangeSetInt::from_iter(a2.iter().cloned()));
        assert_eq!(b, RangeSetInt::from_iter(b2.iter().cloned()));
    }
}

#[test]
fn retrain() {
    let mut set = RangeSetInt::from_iter([1..=6]);
    // Keep only the even numbers.
    set.retain(|k| k % 2 == 0);
    assert_eq!(set, RangeSetInt::from_iter([2, 4, 6]));
}

#[test]
fn sync_and_send() {
    fn assert_sync_and_send<S: Sync + Send>() {}
    assert_sync_and_send::<RangeSetInt<i32>>();
    assert_sync_and_send::<RangesIter<i32>>();
}

fn fraction<T: Integer>(range_int_set: &RangeSetInt<T>, range: &RangeInclusive<T>) -> f64 {
    T::safe_len_to_f64(range_int_set.len()) / T::safe_len_to_f64(T::safe_len(range))
}

#[test]
fn example_2() {
    let line = "chr15   29370   37380   29370,32358,36715   30817,32561,37380";

    // split the line on white space
    let mut iter = line.split_whitespace();
    let chr = iter.next().unwrap();

    // Parse the start and end of the transcription region into a RangeSetInt
    let trans_start: i32 = iter.next().unwrap().parse().unwrap();
    let trans_end: i32 = iter.next().unwrap().parse().unwrap();
    let trans = RangeSetInt::from_iter([trans_start..=trans_end]);
    assert_eq!(trans, RangeSetInt::from_iter([29370..=37380]));

    // Parse the start and end of the exons into a RangeSetInt
    let exon_starts = iter.next().unwrap().split(',').map(|s| s.parse::<i32>());
    let exon_ends = iter.next().unwrap().split(',').map(|s| s.parse::<i32>());
    let exon_ranges = exon_starts
        .zip(exon_ends)
        .map(|(s, e)| s.unwrap()..=e.unwrap());
    let exons = RangeSetInt::from_iter(exon_ranges);
    assert_eq!(
        exons,
        RangeSetInt::from_iter([29370..=30817, 32358..=32561, 36715..=37380])
    );

    // Use 'set subtraction' to find the introns
    let intron = trans - exons;
    assert_eq!(
        intron,
        RangeSetInt::from_iter([30818..=32357, 32562..=36714])
    );
    for range in intron.ranges() {
        let (start, end) = range.into_inner();
        println!("{chr}\t{start}\t{end}");
    }
}

#[test]
fn trick_dyn() {
    let bad = [1..=2, 0..=5];
    // let u = union_dyn!(bad.iter().cloned());
    let good = RangeSetInt::from_iter(bad);
    let _u = union_dyn!(good.ranges());
}

#[test]
fn multiway2() {
    use range_set_int::MultiwaySortedDisjoint;

    let a = RangeSetInt::from_iter([1..=6, 8..=9, 11..=15]);
    let b = RangeSetInt::from_iter([5..=13, 18..=29]);
    let c = RangeSetInt::from_iter([25..=100]);

    let union = [a.ranges(), b.ranges(), c.ranges()].union();
    assert_eq!(union.to_string(), "1..=15, 18..=100");

    let union = MultiwaySortedDisjoint::union([a.ranges(), b.ranges(), c.ranges()]);
    assert_eq!(union.to_string(), "1..=15, 18..=100");
}

#[test]
fn check_sorted_disjoint() {
    use range_set_int::CheckSortedDisjoint;

    let a = CheckSortedDisjoint::new([1..=2, 5..=100].into_iter());
    let b = CheckSortedDisjoint::new([2..=6].into_iter());
    let c = a | b;

    assert_eq!(c.to_string(), "1..=100");
}

#[test]
fn dyn_sorted_disjoint_example() {
    let a = RangeSetInt::from_iter([1u8..=6, 8..=9, 11..=15]);
    let b = RangeSetInt::from_iter([5..=13, 18..=29]);
    let c = RangeSetInt::from_iter([38..=42]);
    let union = [
        DynSortedDisjoint::new(a.ranges()),
        DynSortedDisjoint::new(!b.ranges()),
        DynSortedDisjoint::new(c.ranges()),
    ]
    .union();
    assert_eq!(union.to_string(), "0..=6, 8..=9, 11..=17, 30..=255");
}

#[test]
fn not_iter_example() {
    let a = CheckSortedDisjoint::new([1u8..=2, 5..=100].into_iter());
    let b = NotIter::new(a);
    assert_eq!(b.to_string(), "0..=0, 3..=4, 101..=255");

    // Or, equivalently:
    let b = !CheckSortedDisjoint::new([1u8..=2, 5..=100].into_iter());
    assert_eq!(b.to_string(), "0..=0, 3..=4, 101..=255");
}

#[test]
fn len_demo() {
    let len: <u8 as Integer>::SafeLen = RangeSetInt::from_iter([0u8..=255]).len();
    assert_eq!(len, 256);

    assert_eq!(<u8 as Integer>::safe_len(&(0..=255)), 256);
}

#[test]
fn union_iter() {
    use range_set_int::{CheckSortedDisjoint, UnionIter};

    let a = CheckSortedDisjoint::new([1..=2, 5..=100].into_iter());
    let b = CheckSortedDisjoint::new([2..=6].into_iter());
    let c = UnionIter::new(AssumeSortedStarts::new(
        a.merge_by(b, |a_range, b_range| a_range.start() <= b_range.start()),
    ));
    assert_eq!(c.to_string(), "1..=100");

    // Or, equivalently:
    let a = CheckSortedDisjoint::new([1..=2, 5..=100].into_iter());
    let b = CheckSortedDisjoint::new([2..=6].into_iter());
    let c = SortedDisjointIterator::bitor(a, b);
    assert_eq!(c.to_string(), "1..=100")
}

#[test]
fn bitor() {
    let a = CheckSortedDisjoint::new([1..=1].into_iter());
    let b = RangeSetInt::from_iter([2..=2]).into_ranges();
    let union = std::ops::BitOr::bitor(a, b);
    assert_eq!(union.to_string(), "1..=2");

    let a = CheckSortedDisjoint::new([1..=1].into_iter());
    let b = CheckSortedDisjoint::new([2..=2].into_iter());
    let c = range_set_int::SortedDisjointIterator::bitor(a, b);
    assert_eq!(c.to_string(), "1..=2");

    let a = CheckSortedDisjoint::new([1..=1].into_iter());
    let b = CheckSortedDisjoint::new([2..=2].into_iter());
    let c = std::ops::BitOr::bitor(a, b);
    assert_eq!(c.to_string(), "1..=2");

    let a = CheckSortedDisjoint::new([1..=1].into_iter());
    let b = RangeSetInt::from_iter([2..=2]).into_ranges();
    let c = range_set_int::SortedDisjointIterator::bitor(a, b);
    assert_eq!(c.to_string(), "1..=2");
}
