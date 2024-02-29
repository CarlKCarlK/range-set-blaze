// #![cfg(test)]
// #![cfg(not(target_arch = "wasm32"))]

// #[cfg(feature = "from_slice")]
// use core::mem::size_of;
// #[cfg(feature = "rog-experimental")]
// use core::ops::Bound;
// use core::ops::RangeInclusive;
// use criterion::{BatchSize, BenchmarkId, Criterion};
// use itertools::Itertools;
// use rand::rngs::StdRng;
// use rand::SeedableRng;
// #[cfg(feature = "rog-experimental")]
// use range_set_blaze::Rog;
// // cmk add RangeMapBlaze to prelude
// use std::collections::BTreeMap;

use std::{
    io::{stdout, Write},
    thread::sleep,
    time::Duration,
};

use range_set_blaze::prelude::*;
//     prelude::*, AssumeSortedStarts, Integer, NotIter, RangeMapBlaze, RangesIter, SortedStarts,
//     UnionIter,
// };
// use std::cmp::Ordering;
// #[cfg(feature = "rog-experimental")]
// use std::panic::AssertUnwindSafe;
// #[cfg(feature = "rog-experimental")]
// use std::panic::{self};
// use std::time::Instant;
// use std::{collections::BTreeSet, ops::BitOr};
// use syntactic_for::syntactic_for;
// use tests_common::{k_sets, width_to_range, How, MemorylessIter, MemorylessRange};

// type I32SafeLen = <i32 as range_set_blaze::Integer>::SafeLen;

// // #[test]
// // fn insert_255u8() {
// //     let btree_map = BTreeMap::from_iter([(255u8, "First")]);
// //     assert_eq!(btree_map.get(&255u8), Some(&"First"));
// //     // cmk
// //     let range_set_blaze = RangeMapBlaze::from_iter([(255u8, "First".to_string())]);
// //     // assert!(range_set_blaze.to_string() == "255..=255");
// // }

// // #[test]
// // #[should_panic]
// // fn insert_max_u128() {
// //     let _ = RangeMapBlaze::<u128>::from_iter([u128::MAX]);
// // }

// // #[test]
// // fn complement0() {
// //     syntactic_for! { ty in [i8, u8, isize, usize,  i16, u16, i32, u32, i64, u64, isize, usize, i128, u128] {
// //         $(
// //         let empty = RangeMapBlaze::<$ty>::new();
// //         let full = !&empty;
// //         println!("empty: {empty} (len {}), full: {full} (len {})", empty.len(), full.len());
// //         )*
// //     }};
// // }

// // #[test]
// // fn repro_bit_and() {
// //     let a = RangeMapBlaze::from_iter([1u8, 2, 3]);
// //     let b = RangeMapBlaze::from_iter([2u8, 3, 4]);

// //     let result = &a & &b;
// //     println!("{result}");
// //     assert_eq!(result, RangeMapBlaze::from_iter([2u8, 3]));
// // }

// // #[test]
// // fn doctest1() {
// //     let a = RangeMapBlaze::<u8>::from_iter([1, 2, 3]);
// //     let b = RangeMapBlaze::<u8>::from_iter([3, 4, 5]);

// //     let result = &a | &b;
// //     assert_eq!(result, RangeMapBlaze::<u8>::from_iter([1, 2, 3, 4, 5]));
// // }

// // #[test]
// // fn doctest2() {
// //     let set = RangeMapBlaze::<u8>::from_iter([1, 2, 3]);
// //     assert!(set.contains(1));
// //     assert!(!set.contains(4));
// // }

// // #[test]
// // fn doctest3() -> Result<(), Box<dyn std::error::Error>> {
// //     let mut a = RangeMapBlaze::from_iter([1u8..=3]);
// //     let mut b = RangeMapBlaze::from_iter([3u8..=5]);

// //     a.append(&mut b);

// //     assert_eq!(a.len(), 5usize);
// //     assert_eq!(b.len(), 0usize);

// //     assert!(a.contains(1));
// //     assert!(a.contains(2));
// //     assert!(a.contains(3));
// //     assert!(a.contains(4));
// //     assert!(a.contains(5));
// //     Ok(())
// // }

// // #[test]
// // fn doctest4() {
// //     let a = RangeMapBlaze::<i8>::from_iter([1, 2, 3]);

// //     let result = !&a;
// //     assert_eq!(result.to_string(), "-128..=0, 4..=127");
// // }

// // #[test]
// // fn compare() {
// //     let mut btree_set = BTreeSet::<u128>::new();
// //     btree_set.insert(3);
// //     btree_set.insert(1);
// //     let string = btree_set.iter().join(", ");
// //     println!("{string:#?}");
// //     assert!(string == "1, 3");
// // }

// // #[test]
// // fn add_in_order() {
// //     let mut range_set = RangeMapBlaze::new();
// //     for i in 0u64..1000 {
// //         range_set.insert(i);
// //     }
// // }

// // // #[test]
// // // fn memoryless_data() {
// // //     let len = 100_000_000;
// // //     let coverage_goal = 0.75;
// // //     let memoryless_data = MemorylessData::new(0, 10_000_000, len, coverage_goal);
// // //     let range_set_blaze = RangeMapBlaze::from_iter(memoryless_data);
// // //     let coverage = range_set_blaze.len() as f64 / len as f64;
// // //     println!(
// // //         "coverage {coverage:?} range_len {:?}",
// // //         range_set_blaze.range_len().separate_with_commas()
// // //     );
// // // }

// // // #[test]
// // // fn memoryless_vec() {
// // //     let len = 100_000_000;
// // //     let coverage_goal = 0.75;
// // //     let memoryless_data = MemorylessData::new(0, 10_000_000, len, coverage_goal);
// // //     let data_as_vec: Vec<u64> = memoryless_data.collect();
// // //     let start = Instant::now();
// // //     // let range_set_blaze = RangeMapBlaze::from_mut_slice(data_as_vec.as_mut_slice());
// // //     let range_set_blaze = RangeMapBlaze::from_iter(data_as_vec);
// // //     let coverage = range_set_blaze.len() as f64 / len as f64;
// // //     println!(
// // //         "coverage {coverage:?} range_len {:?}",
// // //         range_set_blaze.ranges_len().separate_with_commas()
// // //     );
// // //     println!(
// // //         "xTime elapsed in expensive_function() is: {} ms",
// // //         start.elapsed().as_millis()
// // //     );
// // // }

// // #[test]
// // fn iters() -> Result<(), Box<dyn std::error::Error>> {
// //     let range_set_blaze = RangeMapBlaze::from_iter([1u8..=6, 8..=9, 11..=15]);
// //     assert!(range_set_blaze.len() == 13);
// //     for i in range_set_blaze.iter() {
// //         println!("{i}");
// //     }
// //     for range in range_set_blaze.ranges() {
// //         println!("{range:?}");
// //     }
// //     let mut rs = range_set_blaze.ranges();
// //     println!("{:?}", rs.next());
// //     println!("{range_set_blaze}");
// //     println!("{:?}", rs.len());
// //     println!("{:?}", rs.next());
// //     for i in range_set_blaze.iter() {
// //         println!("{i}");
// //     }
// //     // range_set_blaze.len();

// //     let mut rs = range_set_blaze.ranges().complement();
// //     println!("{:?}", rs.next());
// //     println!("{range_set_blaze}");
// //     // !!! assert that can't use range_set_blaze again
// //     Ok(())
// // }

// // #[test]
// // fn missing_doctest_ops() {
// //     // note that may be borrowed or owned in any combination.

// //     // Returns the union of `self` and `rhs` as a new [`RangeMapBlaze`].
// //     let a = RangeMapBlaze::from_iter([1, 2, 3]);
// //     let b = RangeMapBlaze::from_iter([3, 4, 5]);

// //     let result = &a | &b;
// //     assert_eq!(result, RangeMapBlaze::from_iter([1, 2, 3, 4, 5]));
// //     let result = a | &b;
// //     assert_eq!(result, RangeMapBlaze::from_iter([1, 2, 3, 4, 5]));

// //     // Returns the complement of `self` as a new [`RangeMapBlaze`].
// //     let a = RangeMapBlaze::<i8>::from_iter([1, 2, 3]);

// //     let result = !&a;
// //     assert_eq!(result.to_string(), "-128..=0, 4..=127");
// //     let result = !a;
// //     assert_eq!(result.to_string(), "-128..=0, 4..=127");

// //     // Returns the intersection of `self` and `rhs` as a new `RangeMapBlaze<T>`.

// //     let a = RangeMapBlaze::from_iter([1, 2, 3]);
// //     let b = RangeMapBlaze::from_iter([2, 3, 4]);

// //     let result = a & &b;
// //     assert_eq!(result, RangeMapBlaze::from_iter([2, 3]));
// //     let a = RangeMapBlaze::from_iter([1, 2, 3]);
// //     let result = a & b;
// //     assert_eq!(result, RangeMapBlaze::from_iter([2, 3]));

// //     // Returns the symmetric difference of `self` and `rhs` as a new `RangeMapBlaze<T>`.
// //     let a = RangeMapBlaze::from_iter([1, 2, 3]);
// //     let b = RangeMapBlaze::from_iter([2, 3, 4]);

// //     let result = a ^ b;
// //     assert_eq!(result, RangeMapBlaze::from_iter([1, 4]));

// //     // Returns the set difference of `self` and `rhs` as a new `RangeMapBlaze<T>`.
// //     let a = RangeMapBlaze::from_iter([1, 2, 3]);
// //     let b = RangeMapBlaze::from_iter([2, 3, 4]);

// //     let result = a - b;
// //     assert_eq!(result, RangeMapBlaze::from_iter([1]));
// // }

// // #[test]
// // fn multi_op() -> Result<(), Box<dyn std::error::Error>> {
// //     let a = RangeMapBlaze::from_iter([1..=6, 8..=9, 11..=15]);
// //     let b = RangeMapBlaze::from_iter([5..=13, 18..=29]);
// //     let c = RangeMapBlaze::from_iter([38..=42]);
// //     let d = &(&a | &b) | &c;
// //     println!("{d}");
// //     let d = a | b | &c;
// //     println!("{d}");

// //     let a = RangeMapBlaze::from_iter([1..=6, 8..=9, 11..=15]);
// //     let b = RangeMapBlaze::from_iter([5..=13, 18..=29]);
// //     let c = RangeMapBlaze::from_iter([38..=42]);

// //     let _ = [&a, &b, &c].union();
// //     let d = [a, b, c].intersection();
// //     assert_eq!(d, RangeMapBlaze::new());

// //     assert_eq!(
// //         !MultiwayRangeMapBlaze::<u8>::union([]),
// //         RangeMapBlaze::from_iter([0..=255])
// //     );

// //     let a = RangeMapBlaze::from_iter([1..=6, 8..=9, 11..=15]);
// //     let b = RangeMapBlaze::from_iter([5..=13, 18..=29]);
// //     let c = RangeMapBlaze::from_iter([1..=42]);

// //     let _ = &a & &b;
// //     let d = [&a, &b, &c].intersection();
// //     // let d = RangeMapBlaze::intersection([a, b, c]);
// //     println!("{d}");
// //     assert_eq!(d, RangeMapBlaze::from_iter([5..=6, 8..=9, 11..=13]));

// //     assert_eq!(
// //         MultiwayRangeMapBlaze::<u8>::intersection([]),
// //         RangeMapBlaze::from_iter([0..=255])
// //     );
// //     Ok(())
// // }

// // #[test]
// // fn custom_multi() -> Result<(), Box<dyn std::error::Error>> {
// //     let a = RangeMapBlaze::from_iter([1..=6, 8..=9, 11..=15]);
// //     let b = RangeMapBlaze::from_iter([5..=13, 18..=29]);
// //     let c = RangeMapBlaze::from_iter([38..=42]);

// //     let union_stream = b.ranges() | c.ranges();
// //     let a_less = a.ranges().difference(union_stream);
// //     let d: RangeMapBlaze<_> = a_less.into_range_set_blaze();
// //     println!("{d}");

// //     let d: RangeMapBlaze<_> = a
// //         .ranges()
// //         .difference([b.ranges(), c.ranges()].union())
// //         .into_range_set_blaze();
// //     println!("{d}");
// //     Ok(())
// // }

// // #[test]
// // fn from_string() -> Result<(), Box<dyn std::error::Error>> {
// //     let a = RangeMapBlaze::from_iter([0..=4, 14..=17, 30..=255, 0..=37, 43..=65535]);
// //     assert_eq!(a, RangeMapBlaze::from_iter([0..=65535]));
// //     Ok(())
// // }

// // #[test]
// // fn nand_repro() -> Result<(), Box<dyn std::error::Error>> {
// //     let b = &RangeMapBlaze::from_iter([5u8..=13, 18..=29]);
// //     let c = &RangeMapBlaze::from_iter([38..=42]);
// //     println!("about to nand");
// //     let d = !b | !c;
// //     assert_eq!(
// //         d,
// //         RangeMapBlaze::from_iter([0..=4, 14..=17, 30..=255, 0..=37, 43..=255])
// //     );
// //     Ok(())
// // }

// // #[test]
// // fn parity() -> Result<(), Box<dyn std::error::Error>> {
// //     let a = &RangeMapBlaze::from_iter([1..=6, 8..=9, 11..=15]);
// //     let b = &RangeMapBlaze::from_iter([5..=13, 18..=29]);
// //     let c = &RangeMapBlaze::from_iter([38..=42]);
// //     assert_eq!(
// //         a & !b & !c | !a & b & !c | !a & !b & c | a & b & c,
// //         RangeMapBlaze::from_iter([1..=4, 7..=7, 10..=10, 14..=15, 18..=29, 38..=42])
// //     );
// //     let _d = [a.ranges()].intersection();
// //     let _parity: RangeMapBlaze<u8> = [[a.ranges()].intersection()].union().into_range_set_blaze();
// //     let _parity: RangeMapBlaze<u8> = [a.ranges()].intersection().into_range_set_blaze();
// //     let _parity: RangeMapBlaze<u8> = [a.ranges()].union().into_range_set_blaze();
// //     println!("!b {}", !b);
// //     println!("!c {}", !c);
// //     println!("!b|!c {}", !b | !c);
// //     println!(
// //         "!b|!c {}",
// //         RangeMapBlaze::from_sorted_disjoint(b.ranges().complement() | c.ranges().complement())
// //     );

// //     let _a = RangeMapBlaze::from_iter([1..=6, 8..=9, 11..=15]);
// //     let u = [DynSortedDisjoint::new(a.ranges())].union();
// //     assert_eq!(
// //         RangeMapBlaze::from_sorted_disjoint(u),
// //         RangeMapBlaze::from_iter([1..=6, 8..=9, 11..=15])
// //     );
// //     let u = union_dyn!(a.ranges());
// //     assert_eq!(
// //         RangeMapBlaze::from_sorted_disjoint(u),
// //         RangeMapBlaze::from_iter([1..=6, 8..=9, 11..=15])
// //     );
// //     let u = union_dyn!(a.ranges(), b.ranges(), c.ranges());
// //     assert_eq!(
// //         RangeMapBlaze::from_sorted_disjoint(u),
// //         RangeMapBlaze::from_iter([1..=15, 18..=29, 38..=42])
// //     );

// //     let u = [
// //         intersection_dyn!(a.ranges(), b.ranges().complement(), c.ranges().complement()),
// //         intersection_dyn!(a.ranges().complement(), b.ranges(), c.ranges().complement()),
// //         intersection_dyn!(a.ranges().complement(), b.ranges().complement(), c.ranges()),
// //         intersection_dyn!(a.ranges(), b.ranges(), c.ranges()),
// //     ]
// //     .union();
// //     assert_eq!(
// //         RangeMapBlaze::from_sorted_disjoint(u),
// //         RangeMapBlaze::from_iter([1..=4, 7..=7, 10..=10, 14..=15, 18..=29, 38..=42])
// //     );
// //     Ok(())
// // }

// // // skip this test because the expected error message is not stable
// // // #[test]
// // // fn ui() {
// // //     let t = trybuild::TestCases::new();
// // //     t.compile_fail("tests/ui/*.rs");
// // // }

// // #[test]
// // fn complement() -> Result<(), Box<dyn std::error::Error>> {
// //     // RangeMapBlaze, RangesIter, NotIter, UnionIter, Tee, UnionIter(g)
// //     let a0 = RangeMapBlaze::from_iter([1..=6]);
// //     let a1 = RangeMapBlaze::from_iter([8..=9, 11..=15]);
// //     let a = &a0 | &a1;
// //     let not_a = !&a;
// //     let b = a.ranges();
// //     let c = !not_a.ranges();
// //     let d = a0.ranges() | a1.ranges();
// //     let (e, _) = a.ranges().tee();

// //     let f = UnionIter::from([15, 14, 15, 13, 12, 11, 9, 9, 8, 6, 4, 5, 3, 2, 1, 1, 1]);
// //     let not_b = !b;
// //     let not_c = !c;
// //     let not_d = !d;
// //     let not_e = e.complement();
// //     let not_f = !f;
// //     assert!(not_a.ranges().equal(not_b));
// //     assert!(not_a.ranges().equal(not_c));
// //     assert!(not_a.ranges().equal(not_d));
// //     assert!(not_a.ranges().equal(not_e));
// //     assert!(not_a.ranges().equal(not_f));
// //     Ok(())
// // }

// // #[test]
// // fn union_test() -> Result<(), Box<dyn std::error::Error>> {
// //     // RangeMapBlaze, RangesIter, NotIter, UnionIter, Tee, UnionIter(g)
// //     let a0 = RangeMapBlaze::from_iter([1..=6]);
// //     let (a0_tee, _) = a0.ranges().tee();
// //     let a1 = RangeMapBlaze::from_iter([8..=9]);
// //     let a2 = RangeMapBlaze::from_iter([11..=15]);
// //     let a12 = &a1 | &a2;
// //     let not_a0 = !&a0;
// //     let a = &a0 | &a1 | &a2;
// //     let b = a0.ranges() | a1.ranges() | a2.ranges();
// //     let c = !not_a0.ranges() | a12.ranges();
// //     let d = a0.ranges() | a1.ranges() | a2.ranges();
// //     let e = a0_tee.union(a12.ranges());

// //     let f = UnionIter::from_iter(a0.iter())
// //         | UnionIter::from_iter(a1.iter())
// //         | UnionIter::from_iter(a2.iter());
// //     assert!(a.ranges().equal(b));
// //     assert!(a.ranges().equal(c));
// //     assert!(a.ranges().equal(d));
// //     assert!(a.ranges().equal(e));
// //     assert!(a.ranges().equal(f));
// //     Ok(())
// // }

// // #[test]
// // fn sub() -> Result<(), Box<dyn std::error::Error>> {
// //     // RangeMapBlaze, RangesIter, NotIter, UnionIter, Tee, UnionIter(g)
// //     let a0 = RangeMapBlaze::from_iter([1..=6]);
// //     let a1 = RangeMapBlaze::from_iter([8..=9]);
// //     let a2 = RangeMapBlaze::from_iter([11..=15]);
// //     let a01 = &a0 | &a1;
// //     let (a01_tee, _) = a01.ranges().tee();
// //     let not_a01 = !&a01;
// //     let a = &a01 - &a2;
// //     let b = a01.ranges() - a2.ranges();
// //     let c = !not_a01.ranges() - a2.ranges();
// //     let d = (a0.ranges() | a1.ranges()) - a2.ranges();
// //     let e = a01_tee.difference(a2.ranges());
// //     let f = UnionIter::from_iter(a01.iter()) - UnionIter::from_iter(a2.iter());
// //     assert!(a.ranges().equal(b));
// //     assert!(a.ranges().equal(c));
// //     assert!(a.ranges().equal(d));
// //     assert!(a.ranges().equal(e));
// //     assert!(a.ranges().equal(f));

// //     Ok(())
// // }

// // #[test]
// // fn xor() -> Result<(), Box<dyn std::error::Error>> {
// //     // RangeMapBlaze, RangesIter, NotIter, UnionIter, Tee, UnionIter(g)
// //     let a0 = RangeMapBlaze::from_iter([1..=6]);
// //     let a1 = RangeMapBlaze::from_iter([8..=9]);
// //     let a2 = RangeMapBlaze::from_iter([11..=15]);
// //     let a01 = &a0 | &a1;
// //     let (a01_tee, _) = a01.ranges().tee();
// //     let not_a01 = !&a01;
// //     let a = &a01 ^ &a2;
// //     let b = a01.ranges() ^ a2.ranges();
// //     let c = !not_a01.ranges() ^ a2.ranges();
// //     let d = (a0.ranges() | a1.ranges()) ^ a2.ranges();
// //     let e = a01_tee.symmetric_difference(a2.ranges());
// //     let f = UnionIter::from_iter(a01.iter()) ^ UnionIter::from_iter(a2.iter());
// //     assert!(a.ranges().equal(b));
// //     assert!(a.ranges().equal(c));
// //     assert!(a.ranges().equal(d));
// //     assert!(a.ranges().equal(e));
// //     assert!(a.ranges().equal(f));
// //     Ok(())
// // }

// // #[test]
// // fn bitand() -> Result<(), Box<dyn std::error::Error>> {
// //     // RangeMapBlaze, RangesIter, NotIter, UnionIter, Tee, UnionIter(g)
// //     let a0 = RangeMapBlaze::from_iter([1..=6]);
// //     let a1 = RangeMapBlaze::from_iter([8..=9]);
// //     let a2 = RangeMapBlaze::from_iter([11..=15]);
// //     let a01 = &a0 | &a1;
// //     let (a01_tee, _) = a01.ranges().tee();
// //     let not_a01 = !&a01;
// //     let a = &a01 & &a2;
// //     let b = a01.ranges() & a2.ranges();
// //     let c = !not_a01.ranges() & a2.ranges();
// //     let d = (a0.ranges() | a1.ranges()) & a2.ranges();
// //     let e = a01_tee.intersection(a2.ranges());
// //     let f = UnionIter::from_iter(a01.iter()) & UnionIter::from_iter(a2.iter());
// //     assert!(a.ranges().equal(b));
// //     assert!(a.ranges().equal(c));
// //     assert!(a.ranges().equal(d));
// //     assert!(a.ranges().equal(e));
// //     assert!(a.ranges().equal(f));
// //     Ok(())
// // }

// // #[test]
// // fn empty_it() {
// //     let universe = RangeMapBlaze::from_iter([0u8..=255]);
// //     let universe = universe.ranges();
// //     let arr: [u8; 0] = [];
// //     let a0 = RangeMapBlaze::<u8>::from_iter(arr);
// //     assert!(!(a0.ranges()).equal(universe.clone()));
// //     assert!((!a0).ranges().equal(universe));
// //     let _a0 = RangeMapBlaze::from_iter([0..=0; 0]);
// //     let _a = RangeMapBlaze::<i32>::new();

// //     let a_iter: std::array::IntoIter<i32, 0> = [].into_iter();
// //     let a = a_iter.collect::<RangeMapBlaze<i32>>();
// //     let b = RangeMapBlaze::from_iter([0i32; 0]);
// //     let mut c3 = a.clone();
// //     let mut c5 = a.clone();

// //     let c0 = (&a).bitor(&b);
// //     let c1a = &a | &b;
// //     let c1b = &a | b.clone();
// //     let c1c = a.clone() | &b;
// //     let c1d = a.clone() | b.clone();
// //     let c2: RangeMapBlaze<_> = (a.ranges() | b.ranges()).into_range_set_blaze();
// //     c3.append(&mut b.clone());
// //     c5.extend(b);

// //     let answer = RangeMapBlaze::from_iter([0; 0]);
// //     assert_eq!(&c0, &answer);
// //     assert_eq!(&c1a, &answer);
// //     assert_eq!(&c1b, &answer);
// //     assert_eq!(&c1c, &answer);
// //     assert_eq!(&c1d, &answer);
// //     assert_eq!(&c2, &answer);
// //     assert_eq!(&c3, &answer);
// //     assert_eq!(&c5, &answer);

// //     let a_iter: std::array::IntoIter<i32, 0> = [].into_iter();
// //     let a = a_iter.collect::<RangeMapBlaze<i32>>();
// //     let b = RangeMapBlaze::from_iter([0; 0]);

// //     let c0 = a.ranges() | b.ranges();
// //     let c1 = [a.ranges(), b.ranges()].union();
// //     let c_list2: [RangesIter<i32>; 0] = [];
// //     let c2 = c_list2.clone().union();
// //     let c3 = union_dyn!(a.ranges(), b.ranges());
// //     let c4 = c_list2.map(DynSortedDisjoint::new).union();

// //     let answer = RangeMapBlaze::from_iter([0; 0]);
// //     assert!(c0.equal(answer.ranges()));
// //     assert!(c1.equal(answer.ranges()));
// //     assert!(c2.equal(answer.ranges()));
// //     assert!(c3.equal(answer.ranges()));
// //     assert!(c4.equal(answer.ranges()));

// //     let c0 = !(a.ranges() & b.ranges());
// //     let c1 = ![a.ranges(), b.ranges()].intersection();
// //     let c_list2: [RangesIter<i32>; 0] = [];
// //     let c2 = !!c_list2.clone().intersection();
// //     let c3 = !intersection_dyn!(a.ranges(), b.ranges());
// //     let c4 = !!c_list2.map(DynSortedDisjoint::new).intersection();

// //     let answer = !RangeMapBlaze::from_iter([0; 0]);
// //     assert!(c0.equal(answer.ranges()));
// //     assert!(c1.equal(answer.ranges()));
// //     assert!(c2.equal(answer.ranges()));
// //     assert!(c3.equal(answer.ranges()));
// //     assert!(c4.equal(answer.ranges()));
// // }

// // #[test]
// // #[allow(clippy::reversed_empty_ranges)]
// // fn tricky_case1() {
// //     let a = RangeMapBlaze::from_iter([1..=0]);
// //     let b = RangeMapBlaze::from_iter([2..=1]);
// //     assert_eq!(a, b);
// //     assert!(a.ranges().equal(b.ranges()));
// //     assert_eq!(a.ranges().len(), 0);
// //     assert_eq!(a.ranges().len(), b.ranges().len());
// //     let a = RangeMapBlaze::from_iter([i32::MIN..=i32::MAX]);
// //     println!("tc1 '{a}'");
// //     assert_eq!(a.len() as i128, (i32::MAX as i128) - (i32::MIN as i128) + 1);
// //     let a = !RangeMapBlaze::from_iter([1..=0]);
// //     println!("tc1 '{a}'");
// //     assert_eq!(a.len() as i128, (i32::MAX as i128) - (i32::MIN as i128) + 1);

// //     let a = !RangeMapBlaze::from_iter([1i128..=0]);
// //     println!("tc1 '{a}', {}", a.len());
// //     assert_eq!(a.len(), u128::MAX);
// //     let a = !RangeMapBlaze::from_iter([1u128..=0]);
// //     println!("tc1 '{a}', {}", a.len());
// //     assert_eq!(a.len(), u128::MAX);
// // }

// // // should fail
// // #[test]
// // #[should_panic]
// // fn tricky_case2() {
// //     let _a = RangeMapBlaze::from_iter([-1..=i128::MAX]);
// // }

// // #[test]
// // #[should_panic]
// // fn tricky_case3() {
// //     let _a = RangeMapBlaze::from_iter([0..=u128::MAX]);
// // }

// // #[test]
// // fn constructors() -> Result<(), Box<dyn std::error::Error>> {
// //     // #9: new
// //     let mut _range_set_int;
// //     _range_set_int = RangeMapBlaze::<i32>::new();
// //     // #10 collect / from_iter T
// //     _range_set_int = [1, 5, 6, 5].into_iter().collect();
// //     _range_set_int = RangeMapBlaze::from_iter([1, 5, 6, 5]);
// //     // #11 into / from array T
// //     _range_set_int = [1, 5, 6, 5].into();
// //     _range_set_int = RangeMapBlaze::from_iter([1, 5, 6, 5]);
// //     // #12 into / from slice T
// //     // _range_set_int = [1, 5, 6, 5][1..=2].into();
// //     // _range_set_int = RangeMapBlaze::from_iter([1, 5, 6, 5].as_slice());
// //     //#13 collect / from_iter range
// //     _range_set_int = [5..=6, 1..=5].into_iter().collect();
// //     _range_set_int = RangeMapBlaze::from_iter([5..=6, 1..=5]);
// //     // #16 into / from iter (T,T) + SortedDisjoint
// //     _range_set_int = _range_set_int.ranges().into_range_set_blaze();
// //     _range_set_int = RangeMapBlaze::from_sorted_disjoint(_range_set_int.ranges());

// //     let sorted_starts = AssumeSortedStarts::new([1..=5, 6..=10].into_iter());
// //     let mut _sorted_disjoint_iter;
// //     _sorted_disjoint_iter = UnionIter::new(sorted_starts);
// //     // #10 collect / from_iter T
// //     let mut _sorted_disjoint_iter: UnionIter<_, _> = [1, 5, 6, 5].into_iter().collect();
// //     _sorted_disjoint_iter = UnionIter::from_iter([1, 5, 6, 5]);
// //     // // #11 into / from array T
// //     _sorted_disjoint_iter = [1, 5, 6, 5].into();
// //     _sorted_disjoint_iter = UnionIter::from([1, 5, 6, 5]);
// //     // // #12 into / from slice T
// //     _sorted_disjoint_iter = [1, 5, 6, 5][1..=2].into();
// //     _sorted_disjoint_iter = UnionIter::from([1, 5, 6, 5].as_slice());
// //     // //#13 collect / from_iter range
// //     _sorted_disjoint_iter = [5..=6, 1..=5].into_iter().collect();
// //     _sorted_disjoint_iter = UnionIter::from_iter([5..=6, 1..=5]);
// //     // // #14 from into array range
// //     _sorted_disjoint_iter = [5..=6, 1..=5].into();
// //     _sorted_disjoint_iter = UnionIter::from([5..=6, 1..=5]);
// //     // // #15 from into slice range
// //     _sorted_disjoint_iter = [5..=6, 1..=5][0..=1].into();
// //     _sorted_disjoint_iter = UnionIter::from([5..=6, 1..=5].as_slice());
// //     // // #16 into / from iter (T,T) + SortedDisjoint
// //     let mut _sorted_disjoint_iter: UnionIter<_, _> = _range_set_int.ranges().collect();
// //     _sorted_disjoint_iter = UnionIter::from_iter(_range_set_int.ranges());

// //     Ok(())
// // }

// // #[test]
// // fn debug_k_play() {
// //     let mut c = Criterion::default();
// //     k_play(&mut c);
// // }

// // fn k_play(c: &mut Criterion) {
// //     let range = 0..=9_999_999;
// //     let range_len = 1_000;
// //     let coverage_goal = 0.50;

// //     let mut group = c.benchmark_group("k_play");
// //     {
// //         let k = &25;
// //         // group.throughput(Throughput::Bytes(*size as u64));
// //         group.bench_with_input(BenchmarkId::new("dyn", k), k, |b, &k| {
// //             b.iter_batched(
// //                 || {
// //                     k_sets(
// //                         k,
// //                         range_len,
// //                         &range,
// //                         coverage_goal,
// //                         How::Intersection,
// //                         &mut StdRng::seed_from_u64(0),
// //                     )
// //                 },
// //                 |sets| {
// //                     let sets = sets.iter().map(|x| DynSortedDisjoint::new(x.ranges()));
// //                     let _answer: RangeMapBlaze<_> = sets.intersection().into_range_set_blaze();
// //                 },
// //                 BatchSize::SmallInput,
// //             );
// //         });
// //     }
// //     group.finish();
// // }

// // #[test]
// // fn data_gen() {
// //     let range = -10_000_000i32..=10_000_000;
// //     let range_len = 1000;
// //     let coverage_goal = 0.75;
// //     let k = 100;

// //     for how in [How::None, How::Union, How::Intersection] {
// //         let mut option_range_int_set: Option<RangeMapBlaze<_>> = None;
// //         for seed in 0..k as u64 {
// //             let r2: RangeMapBlaze<i32> = MemorylessRange::new(
// //                 &mut StdRng::seed_from_u64(seed),
// //                 range_len,
// //                 range.clone(),
// //                 coverage_goal,
// //                 k,
// //                 how,
// //             )
// //             .collect();
// //             option_range_int_set = Some(if let Some(range_int_set) = &option_range_int_set {
// //                 match how {
// //                     How::Intersection => range_int_set & r2,
// //                     How::Union => range_int_set | r2,
// //                     How::None => r2,
// //                 }
// //             } else {
// //                 r2
// //             });
// //             let range_int_set = option_range_int_set.as_ref().unwrap();
// //             println!(
// //                 "range_int_set.len={}, ri={:#?}, how={how:#?} {seed} range_len={}, fraction={}",
// //                 range_int_set.len(),
// //                 &range,
// //                 range_int_set.ranges_len(),
// //                 fraction(range_int_set, &range)
// //             );
// //         }
// //         let range_int_set = option_range_int_set.unwrap();
// //         let fraction = fraction(&range_int_set, &range);
// //         println!("how={how:#?}, goal={coverage_goal}, fraction={fraction}");
// //         assert!(coverage_goal * 0.95 < fraction && fraction < coverage_goal * 1.05);
// //         // Don't check this because of known off-by-one-error that don't matter in practice.
// //         // let first = range_int_set.first().unwrap();
// //         // let last = range_int_set.last().unwrap();
// //         // println!("first={first}, last={last}, range={range:#?}");
// //         // assert!(first >= *range.start());
// //         // assert!(last <= *range.end());
// //     }
// // }

// // #[test]
// // fn vary_coverage_goal() {
// //     let k = 2;
// //     let range_len = 1_000;
// //     let range = 0..=99_999_999;
// //     let coverage_goal_list = [0.01, 0.1, 0.25, 0.5, 0.75, 0.9, 0.99];
// //     let setup_vec = coverage_goal_list
// //         .iter()
// //         .map(|coverage_goal| {
// //             (
// //                 coverage_goal,
// //                 k_sets(
// //                     k,
// //                     range_len,
// //                     &range,
// //                     *coverage_goal,
// //                     How::None,
// //                     &mut StdRng::seed_from_u64(0),
// //                 ),
// //             )
// //         })
// //         .collect::<Vec<_>>();

// //     for (range_len, sets) in &setup_vec {
// //         let parameter = *range_len;

// //         let answer = &sets[0] | &sets[1];
// //         let fraction_val = fraction(&answer, &range);
// //         println!(
// //             "u: {parameter}, {fraction_val}, {}+{}={}",
// //             sets[0].ranges_len(),
// //             sets[1].ranges_len(),
// //             answer.ranges_len()
// //         );
// //         let answer = &sets[0] & &sets[1];
// //         let fraction_val = fraction(&answer, &range);
// //         println!(
// //             "i: {parameter}, {fraction_val}, {}+{}={}",
// //             sets[0].ranges_len(),
// //             sets[1].ranges_len(),
// //             answer.ranges_len()
// //         );
// //     }
// // }

// // #[test]
// // fn ingest_clumps_base() {
// //     let k = 1;
// //     let average_width_list = [2, 1, 3, 4, 5, 10, 100, 1000, 10_000, 100_000, 1_000_000];
// //     let coverage_goal = 0.10;
// //     let assert_tolerance = 0.005;
// //     let how = How::None;
// //     let seed = 0;
// //     let iter_len = 1_000_000;

// //     println!(
// //         "{:#?},{:#?},{:#?},{:#?},{:#?},{:#?},{:#?},{:#?},{:#?},{:#?}",
// //         "seed",
// //         "average_width",
// //         "coverage_goal",
// //         "iter_len",
// //         "range",
// //         "range_count_with_dups",
// //         "item_count_with_dups",
// //         "range_count_without_dups",
// //         "item_count_without_dups",
// //         "fraction",
// //     );

// //     for average_width in average_width_list {
// //         let (range_len, range) = width_to_range(iter_len, average_width, coverage_goal);

// //         let mut rng = StdRng::seed_from_u64(seed);
// //         let memoryless_range =
// //             MemorylessRange::new(&mut rng, range_len, range.clone(), coverage_goal, k, how);
// //         let range_count_with_dups = memoryless_range.count();
// //         let mut rng = StdRng::seed_from_u64(seed);
// //         let memoryless_iter =
// //             MemorylessIter::new(&mut rng, range_len, range.clone(), coverage_goal, k, how);
// //         let item_count_with_dups = memoryless_iter.count();
// //         let mut rng = StdRng::seed_from_u64(seed);
// //         let range_set_blaze: RangeMapBlaze<_> =
// //             MemorylessRange::new(&mut rng, range_len, range.clone(), coverage_goal, k, how)
// //                 .collect();

// //         let range_count_no_dups = range_set_blaze.ranges_len();
// //         let item_count_no_dups = range_set_blaze.len();
// //         let fraction_value = fraction(&range_set_blaze, &range);
// //         println!(
// //             "{:#?},{:#?},{:#?},{:#?},{:#?},{:#?},{:#?},{:#?},{:#?},{:#?}",
// //             seed,
// //             average_width,
// //             coverage_goal,
// //             iter_len,
// //             range.end() + 1,
// //             range_count_with_dups,
// //             item_count_with_dups,
// //             range_count_no_dups,
// //             item_count_no_dups,
// //             fraction_value
// //         );
// //         assert!((fraction_value - coverage_goal).abs() < assert_tolerance);

// //         // count of iter with dups
// //         // count of iter without dups
// //         // range with dups
// //         // range without dups
// //         // fraction
// //     }
// // }

// // #[test]
// // fn doc_test_insert1() {
// //     let mut set = RangeMapBlaze::new();

// //     assert!(set.insert(2));
// //     assert!(!set.insert(2));
// //     assert_eq!(set.len(), 1 as I32SafeLen);
// // }

// #[test]
// fn doc_test_len() {
//     let mut v = RangeMapBlaze::new();
//     assert_eq!(v.len(), 0 as I32SafeLen);
//     v.insert(1, "Hello".to_string());
//     assert_eq!(v.len(), 1 as I32SafeLen);

//     // cmk
//     // let v = RangeMapBlaze::from_iter([
//     //     -170_141_183_460_469_231_731_687_303_715_884_105_728i128..=10,
//     //     -10..=170_141_183_460_469_231_731_687_303_715_884_105_726,
//     // ]);
//     // assert_eq!(
//     //     v.len(),
//     //     340_282_366_920_938_463_463_374_607_431_768_211_455u128
//     // );
// }

// // #[test]
// // fn test_pops() {
// //     let mut set = RangeMapBlaze::from_iter([1..=2, 4..=5, 10..=11]);
// //     let len = set.len();
// //     assert_eq!(set.pop_first(), Some(1));
// //     assert_eq!(set.len(), len - 1);
// //     assert_eq!(set, RangeMapBlaze::from_iter([2..=2, 4..=5, 10..=11]));
// //     assert_eq!(set.pop_last(), Some(11));
// //     println!("{set:#?}");
// //     assert_eq!(set, RangeMapBlaze::from_iter([2..=2, 4..=5, 10..=10]));
// //     assert_eq!(set.len(), len - 2);
// //     assert_eq!(set.pop_last(), Some(10 as I32SafeLen));
// //     assert_eq!(set.len(), len - 3);
// //     assert_eq!(set, RangeMapBlaze::from_iter([2..=2, 4..=5]));
// //     assert_eq!(set.pop_first(), Some(2));
// //     assert_eq!(set.len(), len - 4);
// //     assert_eq!(set, RangeMapBlaze::from_iter([4..=5]));
// // }

// // #[test]
// // fn eq() {
// //     assert!(RangeMapBlaze::from_iter([0, 2]) > RangeMapBlaze::from_iter([0, 1]));
// //     assert!(RangeMapBlaze::from_iter([0, 2]) > RangeMapBlaze::from_iter([0..=100]));
// //     assert!(RangeMapBlaze::from_iter([2..=2]) > RangeMapBlaze::from_iter([1..=2]));
// //     for use_0 in [false, true] {
// //         for use_1 in [false, true] {
// //             for use_2 in [false, true] {
// //                 for use_3 in [false, true] {
// //                     for use_4 in [false, true] {
// //                         for use_5 in [false, true] {
// //                             let mut a = RangeMapBlaze::new();
// //                             let mut b = RangeMapBlaze::new();
// //                             if use_0 {
// //                                 a.insert(0);
// //                             };
// //                             if use_1 {
// //                                 a.insert(1);
// //                             };
// //                             if use_2 {
// //                                 a.insert(2);
// //                             };
// //                             if use_3 {
// //                                 b.insert(0);
// //                             };
// //                             if use_4 {
// //                                 b.insert(1);
// //                             };
// //                             if use_5 {
// //                                 b.insert(2);
// //                             };
// //                             let a2: BTreeSet<_> = a.iter().collect();
// //                             let b2: BTreeSet<_> = b.iter().collect();
// //                             assert!((a == b) == (a2 == b2));
// //                             println!("{a:?} <= {b:?}? RSI {}", a <= b);
// //                             println!("{a:?} <= {b:?}? BTS {}", a2 <= b2);
// //                             assert!((a <= b) == (a2 <= b2));
// //                             assert!((a < b) == (a2 < b2));
// //                         }
// //                     }
// //                 }
// //             }
// //         }
// //     }
// // }

// // #[test]
// // fn insert2() {
// //     let set = RangeMapBlaze::from_iter([1..=2, 4..=5, 10..=20, 30..=30]);
// //     for insert in 0..=31 {
// //         println!("inserting  {insert}");
// //         let mut a = set.clone();
// //         let mut a2: BTreeSet<_> = a.iter().collect();
// //         let b2 = a2.insert(insert);
// //         let b = a.insert(insert);
// //         assert_eq!(a, RangeMapBlaze::from_iter(a2.iter().cloned()));
// //         assert_eq!(b, b2);
// //     }
// // }

// // #[test]
// // fn remove() {
// //     let mut set = RangeMapBlaze::from_iter([1..=2, 4..=5, 10..=11]);
// //     let len = set.len();
// //     assert!(set.remove(4));
// //     assert_eq!(set.len(), len - 1 as I32SafeLen);
// //     assert_eq!(set, RangeMapBlaze::from_iter([1..=2, 5..=5, 10..=11]));
// //     assert!(!set.remove(4));
// //     assert_eq!(set.len(), len - 1 as I32SafeLen);
// //     assert_eq!(set, RangeMapBlaze::from_iter([1..=2, 5..=5, 10..=11]));
// //     assert!(set.remove(5));
// //     assert_eq!(set.len(), len - 2 as I32SafeLen);
// //     assert_eq!(set, RangeMapBlaze::from_iter([1..=2, 10..=11]));

// //     let mut set = RangeMapBlaze::from_iter([1..=2, 4..=5, 10..=100, 1000..=1000]);
// //     let len = set.len();
// //     assert!(!set.remove(0));
// //     assert_eq!(set.len(), len);
// //     assert!(!set.remove(3));
// //     assert_eq!(set.len(), len);
// //     assert!(set.remove(2));
// //     assert_eq!(set.len(), len - 1 as I32SafeLen);
// //     assert!(set.remove(1000));
// //     assert_eq!(set.len(), len - 2 as I32SafeLen);
// //     assert!(set.remove(10));
// //     assert_eq!(set.len(), len - 3 as I32SafeLen);
// //     assert!(set.remove(50));
// //     assert_eq!(set.len(), len - 4 as I32SafeLen);
// //     assert_eq!(
// //         set,
// //         RangeMapBlaze::from_iter([1..=1, 4..=5, 11..=49, 51..=100])
// //     );
// // }

// // #[test]
// // fn remove2() {
// //     let set = RangeMapBlaze::from_iter([1..=2, 4..=5, 10..=20, 30..=30]);
// //     for remove in 0..=31 {
// //         println!("removing  {remove}");
// //         let mut a = set.clone();
// //         let mut a2: BTreeSet<_> = a.iter().collect();
// //         let b2 = a2.remove(&remove);
// //         let b = a.remove(remove);
// //         assert_eq!(a, RangeMapBlaze::from_iter(a2.iter().cloned()));
// //         assert_eq!(b, b2);
// //     }
// //     let set = RangeMapBlaze::new();
// //     for remove in 0..=0 {
// //         println!("removing  {remove}");
// //         let mut a = set.clone();
// //         let mut a2: BTreeSet<_> = a.iter().collect();
// //         let b2 = a2.remove(&remove);
// //         let b = a.remove(remove);
// //         assert_eq!(a, RangeMapBlaze::from_iter(a2.iter().cloned()));
// //         assert_eq!(b, b2);
// //     }
// // }

// // #[test]
// // fn split_off() {
// //     let set = RangeMapBlaze::from_iter([1..=2, 4..=5, 10..=20, 30..=30]);
// //     for split in 0..=31 {
// //         println!("splitting at {split}");
// //         let mut a = set.clone();
// //         let mut a2: BTreeSet<_> = a.iter().collect();
// //         let b2 = a2.split_off(&split);
// //         let b = a.split_off(split);
// //         assert_eq!(a, RangeMapBlaze::from_iter(a2.iter().cloned()));
// //         assert_eq!(b, RangeMapBlaze::from_iter(b2.iter().cloned()));
// //     }
// //     let set = RangeMapBlaze::new();
// //     for split in 0..=0 {
// //         println!("splitting at {split}");
// //         let mut a = set.clone();
// //         let mut a2: BTreeSet<_> = a.iter().collect();
// //         let b2 = a2.split_off(&split);
// //         let b = a.split_off(split);
// //         assert_eq!(a, RangeMapBlaze::from_iter(a2.iter().cloned()));
// //         assert_eq!(b, RangeMapBlaze::from_iter(b2.iter().cloned()));
// //     }
// // }

// // #[test]
// // fn retrain() {
// //     let mut set = RangeMapBlaze::from_iter([1..=6]);
// //     // Keep only the even numbers.
// //     set.retain(|k| k % 2 == 0);
// //     assert_eq!(set, RangeMapBlaze::from_iter([2, 4, 6]));
// // }

// // #[test]
// // fn sync_and_send() {
// //     fn assert_sync_and_send<S: Sync + Send>() {}
// //     assert_sync_and_send::<RangeMapBlaze<i32>>();
// //     assert_sync_and_send::<RangesIter<i32>>();
// // }

// // fn fraction<T: Integer>(range_int_set: &RangeMapBlaze<T>, range: &RangeInclusive<T>) -> f64 {
// //     T::safe_len_to_f64(range_int_set.len()) / T::safe_len_to_f64(T::safe_len(range))
// // }

// // #[test]
// // fn example_2() {
// //     let line = "chr15   29370   37380   29370,32358,36715   30817,32561,37380";

// //     // split the line on white space
// //     let mut iter = line.split_whitespace();
// //     let chr = iter.next().unwrap();

// //     // Parse the start and end of the transcription region into a RangeMapBlaze
// //     let trans_start: i32 = iter.next().unwrap().parse().unwrap();
// //     let trans_end: i32 = iter.next().unwrap().parse().unwrap();
// //     let trans = RangeMapBlaze::from_iter([trans_start..=trans_end]);
// //     assert_eq!(trans, RangeMapBlaze::from_iter([29370..=37380]));

// //     // Parse the start and end of the exons into a RangeMapBlaze
// //     let exon_starts = iter.next().unwrap().split(',').map(|s| s.parse::<i32>());
// //     let exon_ends = iter.next().unwrap().split(',').map(|s| s.parse::<i32>());
// //     let exon_ranges = exon_starts
// //         .zip(exon_ends)
// //         .map(|(s, e)| s.unwrap()..=e.unwrap());
// //     let exons = RangeMapBlaze::from_iter(exon_ranges);
// //     assert_eq!(
// //         exons,
// //         RangeMapBlaze::from_iter([29370..=30817, 32358..=32561, 36715..=37380])
// //     );

// //     // Use 'set subtraction' to find the introns
// //     let intron = trans - exons;
// //     assert_eq!(
// //         intron,
// //         RangeMapBlaze::from_iter([30818..=32357, 32562..=36714])
// //     );
// //     for range in intron.ranges() {
// //         let (start, end) = range.into_inner();
// //         println!("{chr}\t{start}\t{end}");
// //     }
// // }

// // #[test]
// // fn trick_dyn() {
// //     let bad = [1..=2, 0..=5];
// //     // let u = union_dyn!(bad.iter().cloned());
// //     let good = RangeMapBlaze::from_iter(bad);
// //     let _u = union_dyn!(good.ranges());
// // }

// // #[test]
// // fn multiway2() {
// //     use range_set_blaze::MultiwaySortedDisjoint;

// //     let a = RangeMapBlaze::from_iter([1..=6, 8..=9, 11..=15]);
// //     let b = RangeMapBlaze::from_iter([5..=13, 18..=29]);
// //     let c = RangeMapBlaze::from_iter([25..=100]);

// //     let union = [a.ranges(), b.ranges(), c.ranges()].union();
// //     assert_eq!(union.to_string(), "1..=15, 18..=100");

// //     let union = MultiwaySortedDisjoint::union([a.ranges(), b.ranges(), c.ranges()]);
// //     assert_eq!(union.to_string(), "1..=15, 18..=100");
// // }

// // #[test]
// // fn check_sorted_disjoint() {
// //     use range_set_blaze::CheckSortedDisjoint;

// //     let a = CheckSortedDisjoint::new(vec![1..=2, 5..=100].into_iter());
// //     let b = CheckSortedDisjoint::from([2..=6]);
// //     let c = a | b;

// //     assert_eq!(c.to_string(), "1..=100");
// // }

// // #[test]
// // fn dyn_sorted_disjoint_example() {
// //     let a = RangeMapBlaze::from_iter([1u8..=6, 8..=9, 11..=15]);
// //     let b = RangeMapBlaze::from_iter([5..=13, 18..=29]);
// //     let c = RangeMapBlaze::from_iter([38..=42]);
// //     let union = [
// //         DynSortedDisjoint::new(a.ranges()),
// //         DynSortedDisjoint::new(!b.ranges()),
// //         DynSortedDisjoint::new(c.ranges()),
// //     ]
// //     .union();
// //     assert_eq!(union.to_string(), "0..=6, 8..=9, 11..=17, 30..=255");
// // }

// // #[test]
// // fn not_iter_example() {
// //     let a = CheckSortedDisjoint::from([1u8..=2, 5..=100]);
// //     let b = NotIter::new(a);
// //     assert_eq!(b.to_string(), "0..=0, 3..=4, 101..=255");

// //     // Or, equivalently:
// //     let b = !CheckSortedDisjoint::from([1u8..=2, 5..=100]);
// //     assert_eq!(b.to_string(), "0..=0, 3..=4, 101..=255");
// // }

// // #[test]
// // fn len_demo() {
// //     let len: <u8 as Integer>::SafeLen = RangeMapBlaze::from_iter([0u8..=255]).len();
// //     assert_eq!(len, 256);

// //     assert_eq!(<u8 as Integer>::safe_len(&(0..=255)), 256);
// // }

// // #[test]
// // fn union_iter() {
// //     use range_set_blaze::{CheckSortedDisjoint, UnionIter};

// //     let a = CheckSortedDisjoint::new(vec![1..=2, 5..=100].into_iter());
// //     let b = CheckSortedDisjoint::from([2..=6]);
// //     let c = UnionIter::new(AssumeSortedStarts::new(
// //         a.merge_by(b, |a_range, b_range| a_range.start() <= b_range.start()),
// //     ));
// //     assert_eq!(c.to_string(), "1..=100");

// //     // Or, equivalently:
// //     let a = CheckSortedDisjoint::new(vec![1..=2, 5..=100].into_iter());
// //     let b = CheckSortedDisjoint::from([2..=6]);
// //     let c = SortedDisjoint::union(a, b);
// //     assert_eq!(c.to_string(), "1..=100")
// // }

// // #[test]
// // fn bitor() {
// //     let a = CheckSortedDisjoint::from([1..=1]);
// //     let b = RangeMapBlaze::from_iter([2..=2]).into_ranges();
// //     let union = core::ops::BitOr::bitor(a, b);
// //     assert_eq!(union.to_string(), "1..=2");

// //     let a = CheckSortedDisjoint::from([1..=1]);
// //     let b = CheckSortedDisjoint::from([2..=2]);
// //     let c = range_set_blaze::SortedDisjoint::union(a, b);
// //     assert_eq!(c.to_string(), "1..=2");

// //     let a = CheckSortedDisjoint::from([1..=1]);
// //     let b = CheckSortedDisjoint::from([2..=2]);
// //     let c = core::ops::BitOr::bitor(a, b);
// //     assert_eq!(c.to_string(), "1..=2");

// //     let a = CheckSortedDisjoint::from([1..=1]);
// //     let b = RangeMapBlaze::from_iter([2..=2]).into_ranges();
// //     let c = range_set_blaze::SortedDisjoint::union(a, b);
// //     assert_eq!(c.to_string(), "1..=2");
// // }

// // #[test]
// // fn range_set_int_constructors() {
// //     // Create an empty set with 'new' or 'default'.
// //     let a0 = RangeMapBlaze::<i32>::new();
// //     let a1 = RangeMapBlaze::<i32>::default();
// //     assert!(a0 == a1 && a0.is_empty());

// //     // 'from_iter'/'collect': From an iterator of integers.
// //     // Duplicates and out-of-order elements are fine.
// //     let a0 = RangeMapBlaze::from_iter([3, 2, 1, 100, 1]);
// //     let a1: RangeMapBlaze<i32> = [3, 2, 1, 100, 1].into_iter().collect();
// //     assert!(a0 == a1 && a0.to_string() == "1..=3, 100..=100");

// //     // 'from_iter'/'collect': From an iterator of inclusive ranges, start..=end.
// //     // Overlapping, out-of-order, and empty ranges are fine.
// //     #[allow(clippy::reversed_empty_ranges)]
// //     let a0 = RangeMapBlaze::from_iter([1..=2, 2..=2, -10..=-5, 1..=0]);
// //     #[allow(clippy::reversed_empty_ranges)]
// //     let a1: RangeMapBlaze<i32> = [1..=2, 2..=2, -10..=-5, 1..=0].into_iter().collect();
// //     assert!(a0 == a1 && a0.to_string() == "-10..=-5, 1..=2");

// //     // If we know the ranges are sorted and disjoint, we can use 'from'/'into'.
// //     let a0 = RangeMapBlaze::from_sorted_disjoint(CheckSortedDisjoint::from([-10..=-5, 1..=2]));
// //     let a1: RangeMapBlaze<i32> =
// //         CheckSortedDisjoint::from([-10..=-5, 1..=2]).into_range_set_blaze();
// //     assert!(a0 == a1 && a0.to_string() == "-10..=-5, 1..=2");

// //     // For compatibility with `BTreeSet`, we also support
// //     // 'from'/'into' from arrays of integers.
// //     let a0 = RangeMapBlaze::from([3, 2, 1, 100, 1]);
// //     let a1: RangeMapBlaze<i32> = [3, 2, 1, 100, 1].into();
// //     assert!(a0 == a1 && a0.to_string() == "1..=3, 100..=100");
// // }

// // #[cfg(feature = "from_slice")]
// // fn print_features() {
// //     println!("feature\tcould\tare");
// //     syntactic_for! { feature in [
// //         "aes",
// //         "pclmulqdq",
// //         "rdrand",
// //         "rdseed",
// //         "tsc",
// //         "mmx",
// //         "sse",
// //         "sse2",
// //         "sse3",
// //         "ssse3",
// //         "sse4.1",
// //         "sse2",
// //         "sse4a",
// //         "sha",
// //         "avx",
// //         "avx2",
// //         "avx512f",
// //         "avx512cd",
// //         "avx512er",
// //         "avx512pf",
// //         "avx512bw",
// //         "avx512dq",
// //         "avx512vl",
// //         "avx512ifma",
// //         "avx512vbmi",
// //         "avx512vpopcntdq",
// //         "fma",
// //         "bmi1",
// //         "bmi2",
// //         "abm",
// //         "lzcnt",
// //         "tbm",
// //         "popcnt",
// //         "fxsr",
// //         "xsave",
// //         "xsaveopt",
// //         "xsaves",
// //         "xsavec",
// //         ] {$(
// //             println!("{}\t{}\t{}",$feature,is_x86_feature_detected!($feature),cfg!(target_feature = $feature));

// //     )*}};
// // }

// // #[cfg(feature = "from_slice")]
// // #[test]
// // fn from_slice_all_types() {
// //     syntactic_for! { ty in [i8, u8] {
// //         $(
// //             println!("ty={:#?}",size_of::<$ty>() * 8);
// //             let v: Vec<$ty> = (0..=127).collect();
// //             let a2 = RangeMapBlaze::from_slice(&v);
// //             assert!(a2.to_string() == "0..=127");
// //         )*
// //     }};

// //     syntactic_for! { ty in [i16, u16, i32, u32, i64, u64, isize, usize, i128, u128] {
// //         $(
// //             println!("ty={:#?}",size_of::<$ty>() * 8);
// //             let v: Vec<$ty> = (0..=5000).collect();
// //             let a2 = RangeMapBlaze::from_slice(&v);
// //             assert!(a2.to_string() == "0..=5000");
// //         )*
// //     }};
// // }

// // #[cfg(feature = "from_slice")]
// // #[test]
// // fn range_set_int_slice_constructor() {
// //     print_features();
// //     let k = 1;
// //     let average_width = 1000;
// //     let coverage_goal = 0.10;
// //     let how = How::None;
// //     let seed = 0;

// //     #[allow(clippy::single_element_loop)]
// //     for iter_len in [1000, 1500, 1750, 2000, 10_000, 1_000_000] {
// //         let (range_len, range) =
// //             tests_common::width_to_range_u32(iter_len, average_width, coverage_goal);

// //         let vec: Vec<u32> = MemorylessIter::new(
// //             &mut StdRng::seed_from_u64(seed),
// //             range_len,
// //             range.clone(),
// //             coverage_goal,
// //             k,
// //             how,
// //         )
// //         .collect();
// //         let b0 = RangeMapBlaze::from_iter(&vec);
// //         let b1 = RangeMapBlaze::from_slice(&vec);
// //         if b0 != b1 {
// //             println!(
// //                 "{iter_len} error: b0={b0:#?}, b1={b1:#?}, diff={:#?}",
// //                 &b0 ^ &b1
// //             );
// //         }
// //         assert!(b0 == b1);
// //     }

// //     let v: Vec<i32> = (100..=150).collect();
// //     let a2 = RangeMapBlaze::from_slice(v);
// //     assert!(a2.to_string() == "100..=150");

// //     // For compatibility with `BTreeSet`, we also support
// //     // 'from'/'into' from arrays of integers.
// //     let a0 = RangeMapBlaze::from([3, 2, 1, 100, 1]);
// //     let a1: RangeMapBlaze<i32> = [3, 2, 1, 100, 1].into();
// //     assert!(a0 == a1 && a0.to_string() == "1..=3, 100..=100");

// //     #[allow(clippy::needless_borrows_for_generic_args)]
// //     let a2 = RangeMapBlaze::from_slice(&[3, 2, 1, 100, 1]);
// //     assert!(a0 == a2 && a2.to_string() == "1..=3, 100..=100");

// //     let a2 = RangeMapBlaze::from_slice([3, 2, 1, 100, 1]);
// //     assert!(a0 == a2 && a2.to_string() == "1..=3, 100..=100");
// // }

#[test]
fn range_set_int_operators() {
    let a = RangeMapBlaze::from_iter([(1..=2, "one"), (5..=100, "two")]);
    let b = RangeMapBlaze::from_iter([(2..=6, "three")]);

    // Union of two 'RangeMapBlaze's. Later values in from_iter and union have higher priority.
    let result = &a | &b;
    // Alternatively, we can take ownership via 'a | b'.
    assert_eq!(
        result.to_string(),
        r#"(1..=1, "one"), (2..=6, "three"), (7..=100, "two")"#
    );

    // Intersection of two 'RangeMapBlaze's. Later values in intersection have higher priority so `a` acts as a filter or mask.
    let result = &a & &b; // Alternatively, 'a & b'.
    assert_eq!(result.to_string(), r#"(2..=2, "three"), (5..=6, "three")"#);

    // Set difference of two 'RangeMapBlaze's.
    let result = &a - &b; // Alternatively, 'a - b'.
    assert_eq!(result.to_string(), r#"(1..=1, "one"), (7..=100, "two")"#);

    // Symmetric difference of two 'RangeMapBlaze's.
    let result = &a ^ &b; // Alternatively, 'a ^ b'.
    assert_eq!(
        result.to_string(),
        r#"(1..=1, "one"), (3..=4, "three"), (7..=100, "two")"#
    );

    // cmk0 define complement or .to_range_set_blaze() or .into_range_set_blaze(), etc.
    // complement of a 'RangeMapBlaze'.
    let result = !(&a.ranges().into_range_set_blaze());
    assert_eq!(
        result.to_string(),
        "-2147483648..=0, 3..=4, 101..=2147483647"
    );

    // cmk0
    // // Multiway union of 'RangeMapBlaze's.
    // let c = RangeMapBlaze::from_iter([2..=2, 6..=200]);
    // let result = [&a, &b, &c].union();
    // assert_eq!(result.to_string(), "1..=200");

    // // Multiway intersection of 'RangeMapBlaze's.
    // let result = [&a, &b, &c].intersection();
    // assert_eq!(result.to_string(), "2..=2, 6..=6");

    // // Combining multiple operations
    // let result0 = &a - (&b | &c); // Creates a temporary 'RangeMapBlaze'.

    // // Alternatively, we can use the 'SortedDisjoint' API and avoid the temporary 'RangeMapBlaze'.
    // let result1 = RangeMapBlaze::from_sorted_disjoint(a.ranges() - (b.ranges() | c.ranges()));
    // assert!(result0 == result1 && result0.to_string() == "1..=1");
}

// // #[test]
// // fn sorted_disjoint_constructors() {
// //     // RangeMapBlaze's .ranges(), .range().clone() and .into_ranges()
// //     let r = RangeMapBlaze::from_iter([3, 2, 1, 100, 1]);
// //     let a = r.ranges();
// //     let b = a.clone();
// //     assert!(a.to_string() == "1..=3, 100..=100");
// //     assert!(b.to_string() == "1..=3, 100..=100");
// //     //    'into_ranges' takes ownership of the 'RangeMapBlaze'
// //     let a = RangeMapBlaze::from_iter([3, 2, 1, 100, 1]).into_ranges();
// //     assert!(a.to_string() == "1..=3, 100..=100");

// //     // CheckSortedDisjoint -- unsorted or overlapping input ranges will cause a panic.
// //     let a = CheckSortedDisjoint::from([1..=3, 100..=100]);
// //     assert!(a.to_string() == "1..=3, 100..=100");

// //     // tee of a SortedDisjoint iterator
// //     let a = CheckSortedDisjoint::from([1..=3, 100..=100]);
// //     let (a, b) = a.tee();
// //     assert!(a.to_string() == "1..=3, 100..=100");
// //     assert!(b.to_string() == "1..=3, 100..=100");

// //     // DynamicSortedDisjoint of a SortedDisjoint iterator
// //     let a = CheckSortedDisjoint::from([1..=3, 100..=100]);
// //     let b = DynSortedDisjoint::new(a);
// //     assert!(b.to_string() == "1..=3, 100..=100");
// // }

// // #[test]
// // fn iterator_example() {
// //     struct OrdinalWeekends2023 {
// //         next_range: RangeInclusive<i32>,
// //     }
// //     impl SortedStarts<i32> for OrdinalWeekends2023 {}
// //     impl SortedDisjoint<i32> for OrdinalWeekends2023 {}

// //     impl OrdinalWeekends2023 {
// //         fn new() -> Self {
// //             Self { next_range: 0..=1 }
// //         }
// //     }
// //     impl Iterator for OrdinalWeekends2023 {
// //         type Item = RangeInclusive<i32>;
// //         fn next(&mut self) -> Option<Self::Item> {
// //             let (start, end) = self.next_range.clone().into_inner();
// //             if start > 365 {
// //                 None
// //             } else {
// //                 self.next_range = (start + 7)..=(end + 7);
// //                 Some(start.max(1)..=end.min(365))
// //             }
// //         }
// //     }

// //     let weekends = OrdinalWeekends2023::new();
// //     let sept = CheckSortedDisjoint::from([244..=273]);
// //     let sept_weekdays = sept.intersection(weekends.complement());
// //     assert_eq!(
// //         sept_weekdays.to_string(),
// //         "244..=244, 247..=251, 254..=258, 261..=265, 268..=272"
// //     );
// // }

// // #[test]
// // fn sorted_disjoint_operators() {
// //     let a0 = RangeMapBlaze::from_iter([1..=2, 5..=100]);
// //     let b0 = RangeMapBlaze::from_iter([2..=6]);
// //     let c0 = RangeMapBlaze::from_iter([2..=2, 6..=200]);

// //     // 'union' method and 'to_string' method
// //     let (a, b) = (a0.ranges(), b0.ranges());
// //     let result = a.union(b);
// //     assert_eq!(result.to_string(), "1..=100");

// //     // '|' operator and 'equal' method
// //     let (a, b) = (a0.ranges(), b0.ranges());
// //     let result = a | b;
// //     assert!(result.equal(CheckSortedDisjoint::from([1..=100])));

// //     // multiway union of same type
// //     let (a, b, c) = (a0.ranges(), b0.ranges(), c0.ranges());
// //     let result = [a, b, c].union();
// //     assert_eq!(result.to_string(), "1..=200");

// //     // multiway union of different types
// //     let (a, b, c) = (a0.ranges(), b0.ranges(), c0.ranges());
// //     let result = union_dyn!(a, b, !c);
// //     assert_eq!(result.to_string(), "-2147483648..=100, 201..=2147483647");
// // }

// // #[test]
// // fn range_example() {
// //     let mut set = RangeMapBlaze::new();
// //     set.insert(3);
// //     set.insert(5);
// //     set.insert(8);
// //     for elem in (&set & RangeMapBlaze::from_iter([4..=8])).iter() {
// //         println!("{elem}");
// //     }

// //     let intersection = &set & RangeMapBlaze::from_iter([4..=i32::MAX]);
// //     assert_eq!(Some(5), intersection.iter().next());
// // }

// // #[test]
// // fn range_test() {
// //     use core::ops::Bound::Included;
// //     use range_set_blaze::RangeMapBlaze;

// //     let mut set = RangeMapBlaze::new();
// //     set.insert(3);
// //     set.insert(5);
// //     set.insert(8);
// //     for elem in set.range((Included(4), Included(8))) {
// //         println!("{elem}");
// //     }
// //     assert_eq!(Some(5), set.range(4..).next());
// // }

// // #[test]
// // #[allow(clippy::bool_assert_comparison)]
// // fn is_subset_check() {
// //     let sup = CheckSortedDisjoint::from([1..=3]);
// //     let set: CheckSortedDisjoint<i32, _> = [].into();
// //     assert_eq!(set.is_subset(sup), true);

// //     let sup = CheckSortedDisjoint::from([1..=3]);
// //     let set = CheckSortedDisjoint::from([2..=2]);
// //     assert_eq!(set.is_subset(sup), true);

// //     let sup = CheckSortedDisjoint::from([1..=3]);
// //     let set = CheckSortedDisjoint::from([2..=2, 4..=4]);
// //     assert_eq!(set.is_subset(sup), false);
// // }

// // #[test]
// // fn cmp_range_set_int() {
// //     let a = RangeMapBlaze::from_iter([1..=3, 5..=7]);
// //     let b = RangeMapBlaze::from_iter([2..=2]);
// //     assert!(a < b); // Lexicographic comparison
// //     assert!(b.is_subset(&a)); // Subset comparison

// //     // Lexicographic comparisons
// //     assert!(a <= b);
// //     assert!(b > a);
// //     assert!(b >= a);
// //     assert!(a != b);
// //     assert!(a == a);
// //     assert_eq!(a.cmp(&b), Ordering::Less);
// //     assert_eq!(a.partial_cmp(&b), Some(Ordering::Less));
// // }

// // #[test]
// // fn run_rangemap_crate() {
// //     let mut rng = StdRng::seed_from_u64(0);
// //     let range_len = 1_000_000;

// //     let vec_range: Vec<_> =
// //         MemorylessRange::new(&mut rng, range_len, 0..=99_999_999, 0.01, 1, How::None).collect();

// //     let _start = Instant::now();

// //     let rangemap_set0 = &rangemap::RangeInclusiveSet::from_iter(vec_range.iter().cloned());
// //     let _rangemap_set1 = &rangemap::RangeInclusiveSet::from_iter(rangemap_set0.iter().cloned());
// // }

// // #[test]
// // fn from_iter_coverage() {
// //     let vec_range = vec![1..=2, 2..=2, -10..=-5];
// //     let a0 = RangeMapBlaze::from_iter(vec_range.iter());
// //     let a1: RangeMapBlaze<i32> = vec_range.iter().collect();
// //     assert!(a0 == a1 && a0.to_string() == "-10..=-5, 1..=2");
// // }

// // // fn _some_fn() {
// // //     let guaranteed = RangeMapBlaze::from_iter([1..=2, 3..=4, 5..=6]).into_ranges();
// // //     let _range_set_int = RangeMapBlaze::from_sorted_disjoint(guaranteed);
// // //     let not_guaranteed = [1..=2, 3..=4, 5..=6].into_iter();
// // //     let _range_set_int = RangeMapBlaze::from_sorted_disjoint(not_guaranteed);
// // // }

// // // fn _some_fn() {
// // //     let _integer_set = RangeMapBlaze::from_iter([1, 2, 3, 5]);
// // //     let _char_set = RangeMapBlaze::from_iter(['a', 'b', 'c', 'd']);
// // // }

// // #[test]
// // fn print_first_complement_gap() {
// //     let a = CheckSortedDisjoint::from([-10i16..=0, 1000..=2000]);
// //     println!("{:?}", (!a).next().unwrap()); // prints -32768..=-11
// // }

// // #[test]
// // fn multiway_failure_example() {
// //     use range_set_blaze::prelude::*;

// //     let a = RangeMapBlaze::from_iter([1..=6, 8..=9, 11..=15]);
// //     let b = RangeMapBlaze::from_iter([5..=13, 18..=29]);
// //     let c = RangeMapBlaze::from_iter([38..=42]);

// //     let _i0 = [a.ranges(), b.ranges(), c.ranges()].intersection();
// //     // let _i1 = [!a.ranges(), b.ranges(), c.ranges()].intersection();
// //     let _i2 = [
// //         DynSortedDisjoint::new(!a.ranges()),
// //         DynSortedDisjoint::new(b.ranges()),
// //         DynSortedDisjoint::new(c.ranges()),
// //     ]
// //     .intersection();
// //     let _i3 = intersection_dyn!(!a.ranges(), b.ranges(), c.ranges());
// // }

// // #[test]
// // fn complement_sample() {
// //     let c = !RangeMapBlaze::from([0, 3, 4, 5, 10]);
// //     println!("{},{},{}", c.len(), c.ranges_len(), c);
// // }

// // #[cfg(feature = "rog-experimental")]
// // #[test]
// // fn test_rog_functionality() {
// //     let a = RangeMapBlaze::from_iter([1..=6, 8..=9, 11..=15]);
// //     // case 1:
// //     for end in 7..=16 {
// //         println!("case 1: {:?}", a._rogs_range_slow(7..=end));
// //         assert_eq!(
// //             a._rogs_range_slow(7..=end),
// //             a.rogs_range(7..=end).collect::<Vec<_>>()
// //         );
// //     }
// //     // case 2:
// //     for end in 7..=16 {
// //         println!("case 2: {:?}", a._rogs_range_slow(4..=end));
// //         assert_eq!(
// //             a._rogs_range_slow(4..=end),
// //             a.rogs_range(4..=end).collect::<Vec<_>>()
// //         );
// //     }
// //     // case 3:
// //     for start in 11..=15 {
// //         for end in start..=15 {
// //             println!("case 3: {:?}", a._rogs_range_slow(start..=end));
// //             assert_eq!(
// //                 a._rogs_range_slow(start..=end),
// //                 a.rogs_range(start..=end).collect::<Vec<_>>()
// //             );
// //         }
// //     }
// //     // case 4:
// //     for end in -1..=16 {
// //         println!("case 4: {:?}", a._rogs_range_slow(-1..=end));
// //         assert_eq!(
// //             a._rogs_range_slow(-1..=end),
// //             a.rogs_range(-1..=end).collect::<Vec<_>>()
// //         );
// //     }
// // }

// // #[cfg(feature = "rog-experimental")]
// // #[test]
// // fn test_rogs_get_functionality() {
// //     let a = RangeMapBlaze::from_iter([1..=6, 8..=9, 11..=15]);
// //     for value in 0..=16 {
// //         println!("{:?}", a.rogs_get_slow(value));
// //         assert_eq!(a.rogs_get_slow(value), a.rogs_get(value));
// //     }
// // }

// // #[cfg(feature = "rog-experimental")]
// // #[test]
// // fn test_rog_repro1() {
// //     let a = RangeMapBlaze::from_iter([1u8..=6u8]);
// //     assert_eq!(
// //         a._rogs_range_slow(1..=7),
// //         a.rogs_range(1..=7).collect::<Vec<_>>()
// //     );
// // }

// // #[cfg(feature = "rog-experimental")]
// // #[test]
// // fn test_rog_repro2() {
// //     let a = RangeMapBlaze::from_iter([1..=6, 8..=9, 11..=15]);
// //     assert_eq!(
// //         a._rogs_range_slow(4..=8),
// //         a.rogs_range(4..=8).collect::<Vec<_>>()
// //     );
// // }

// // #[cfg(feature = "rog-experimental")]
// // #[test]
// // fn test_rog_coverage1() {
// //     let a = RangeMapBlaze::from_iter([1u8..=6u8]);
// //     assert!(panic::catch_unwind(AssertUnwindSafe(
// //         || a.rogs_range((Bound::Excluded(&255), Bound::Included(&255)))
// //     ))
// //     .is_err());
// //     assert!(panic::catch_unwind(AssertUnwindSafe(|| a.rogs_range(0..0))).is_err());
// // }

// // #[cfg(feature = "rog-experimental")]
// // #[test]
// // fn test_rog_extremes_u8() {
// //     for a in [
// //         RangeMapBlaze::from_iter([1u8..=6u8]),
// //         RangeMapBlaze::from_iter([0u8..=6u8]),
// //         RangeMapBlaze::from_iter([200u8..=255u8]),
// //         RangeMapBlaze::from_iter([0u8..=255u8]),
// //         RangeMapBlaze::from_iter([0u8..=5u8, 20u8..=255]),
// //     ] {
// //         for start in 0u8..=255 {
// //             for end in start..=255 {
// //                 println!("{start}..={end}");
// //                 assert_eq!(
// //                     a._rogs_range_slow(start..=end),
// //                     a.rogs_range(start..=end).collect::<Vec<_>>()
// //                 );
// //             }
// //         }
// //     }
// // }

// // #[cfg(feature = "rog-experimental")]
// // #[test]
// // fn test_rog_get_extremes_u8() {
// //     for a in [
// //         RangeMapBlaze::from_iter([1u8..=6u8]),
// //         RangeMapBlaze::from_iter([0u8..=6u8]),
// //         RangeMapBlaze::from_iter([200u8..=255u8]),
// //         RangeMapBlaze::from_iter([0u8..=255u8]),
// //         RangeMapBlaze::from_iter([0u8..=5u8, 20u8..=255]),
// //     ] {
// //         for value in 0u8..=255 {
// //             println!("{value}");
// //             assert_eq!(a.rogs_get_slow(value), a.rogs_get(value));
// //         }
// //     }
// // }

// // #[cfg(feature = "rog-experimental")]
// // #[test]
// // fn test_rog_extremes_i128() {
// //     for a in [
// //         RangeMapBlaze::from_iter([1i128..=6i128]),
// //         RangeMapBlaze::from_iter([i128::MIN..=6]),
// //         RangeMapBlaze::from_iter([200..=i128::MAX - 1]),
// //         RangeMapBlaze::from_iter([i128::MIN..=i128::MAX - 1]),
// //         RangeMapBlaze::from_iter([i128::MIN..=5, 20..=i128::MAX - 1]),
// //     ] {
// //         for start in [i128::MIN, i128::MIN + 1, 0, i128::MAX - 2, i128::MAX - 1] {
// //             for end in [i128::MIN, i128::MIN + 1, 0, i128::MAX - 2, i128::MAX - 1] {
// //                 if end < start {
// //                     continue;
// //                 }
// //                 println!("{start}..={end}");
// //                 assert_eq!(
// //                     a._rogs_range_slow(start..=end),
// //                     a.rogs_range(start..=end).collect::<Vec<_>>()
// //                 );
// //             }
// //         }
// //     }
// // }

// // #[cfg(feature = "rog-experimental")]
// // #[test]
// // fn test_rog_extremes_get_i128() {
// //     for a in [
// //         RangeMapBlaze::from_iter([1i128..=6i128]),
// //         RangeMapBlaze::from_iter([i128::MIN..=6]),
// //         RangeMapBlaze::from_iter([200..=i128::MAX - 1]),
// //         RangeMapBlaze::from_iter([i128::MIN..=i128::MAX - 1]),
// //         RangeMapBlaze::from_iter([i128::MIN..=5, 20..=i128::MAX - 1]),
// //     ] {
// //         for value in [i128::MIN, i128::MIN + 1, 0, i128::MAX - 2, i128::MAX - 1] {
// //             println!("{value}");
// //             assert_eq!(a.rogs_get_slow(value), a.rogs_get(value));
// //         }
// //     }
// // }

// // #[cfg(feature = "rog-experimental")]
// // #[test]
// // fn test_rog_should_fail_i128() {
// //     for a in [
// //         RangeMapBlaze::from_iter([1i128..=6i128]),
// //         RangeMapBlaze::from_iter([i128::MIN..=6]),
// //         RangeMapBlaze::from_iter([200..=i128::MAX - 1]),
// //         RangeMapBlaze::from_iter([i128::MIN..=i128::MAX - 1]),
// //         RangeMapBlaze::from_iter([i128::MIN..=5, 20..=i128::MAX - 1]),
// //     ] {
// //         for start in [i128::MIN, i128::MIN + 1, 0, i128::MAX - 1, i128::MAX] {
// //             for end in [i128::MIN, i128::MIN + 1, 0, i128::MAX - 1, i128::MAX] {
// //                 if end < start {
// //                     continue;
// //                 }
// //                 println!("{start}..={end}");
// //                 let slow =
// //                     panic::catch_unwind(AssertUnwindSafe(|| a._rogs_range_slow(start..=end))).ok();
// //                 let fast = panic::catch_unwind(AssertUnwindSafe(|| {
// //                     a.rogs_range(start..=end).collect::<Vec<_>>()
// //                 }))
// //                 .ok();
// //                 assert_eq!(slow, fast,);
// //             }
// //         }
// //     }
// // }

// // #[cfg(feature = "rog-experimental")]
// // #[test]
// // fn test_rog_get_should_fail_i128() {
// //     for a in [
// //         RangeMapBlaze::from_iter([1i128..=6i128]),
// //         RangeMapBlaze::from_iter([i128::MIN..=6]),
// //         RangeMapBlaze::from_iter([200..=i128::MAX - 1]),
// //         RangeMapBlaze::from_iter([i128::MIN..=i128::MAX - 1]),
// //         RangeMapBlaze::from_iter([i128::MIN..=5, 20..=i128::MAX - 1]),
// //     ] {
// //         for value in [i128::MIN, i128::MIN + 1, 0, i128::MAX - 1, i128::MAX] {
// //             println!("{value}");
// //             let slow = panic::catch_unwind(AssertUnwindSafe(|| a.rogs_get_slow(value))).ok();
// //             let fast = panic::catch_unwind(AssertUnwindSafe(|| a.rogs_get(value))).ok();
// //             assert_eq!(slow, fast,);
// //         }
// //     }
// // }

// // #[cfg(feature = "rog-experimental")]
// // #[test]
// // fn test_rog_get_doc() {
// //     use crate::RangeMapBlaze;
// //     let range_set_blaze = RangeMapBlaze::from([1, 2, 3]);
// //     assert_eq!(range_set_blaze.rogs_get(2), Rog::Range(1..=3));
// //     assert_eq!(range_set_blaze.rogs_get(4), Rog::Gap(4..=2_147_483_647));
// // }

// // #[cfg(feature = "rog-experimental")]
// // #[test]
// // fn test_rog_range_doc() {
// //     use core::ops::Bound::Included;

// //     let mut set = RangeMapBlaze::new();
// //     set.insert(3);
// //     set.insert(5);
// //     set.insert(6);
// //     for rog in set.rogs_range((Included(4), Included(8))) {
// //         println!("{rog:?}");
// //     } // prints: Gap(4..=4)\nRange(5..=6)\nGap(7..=8)

// //     assert_eq!(Some(Rog::Gap(4..=4)), set.rogs_range(4..).next());

// //     let a = RangeMapBlaze::from_iter([1..=6, 11..=15]);
// //     assert_eq!(
// //         a.rogs_range(-5..=8).collect::<Vec<_>>(),
// //         vec![Rog::Gap(-5..=0), Rog::Range(1..=6), Rog::Gap(7..=8)]
// //     );

// //     let empty = RangeMapBlaze::<u8>::new();
// //     assert_eq!(
// //         empty.rogs_range(..).collect::<Vec<_>>(),
// //         vec![Rog::Gap(0..=255)]
// //     );
// // }

pub fn play_movie(frames: RangeMapBlaze<i32, String>, fps: i32) {
    assert!(fps > 0, "fps must be positive");
    let sleep_duration = Duration::from_secs(1) / fps as u32;
    // For every frame index (index) from 0 to the largest index in the frames ...
    for index in 0..=frames.ranges().into_range_set_blaze().last().unwrap() {
        // Look up the frame at that index (panic if none exists)
        let frame = frames.get(index).unwrap();
        // Clear the line and return the cursor to the beginning of the line
        print!("\x1B[2K\r{}", frame);
        stdout().flush().unwrap(); // Flush stdout to ensure the output is displayed
        sleep(sleep_duration);
    }
}

pub fn multiply(frames: &RangeMapBlaze<i32, String>, factor: i32) -> RangeMapBlaze<i32, String> {
    assert!(factor > 0, "factor must be positive");
    frames
        .range_values()
        .map(|range_value| {
            // cmk This could be faster if we used a custom iterator with the SortedDisjointMap trait
            // cmk could also do something in place to avoid the clone
            let (start, end) = range_value.range.clone().into_inner();
            let new_range = start * factor..=((end + 1) * factor) - 1;
            (new_range, range_value.value.clone())
        })
        .collect()
}

pub fn plus(frames: &RangeMapBlaze<i32, String>, addend: i32) -> RangeMapBlaze<i32, String> {
    assert!(addend > 0, "factor must be positive");
    frames
        .range_values()
        .map(|range_value| {
            // cmk This could be faster if we used a custom iterator with the SortedDisjointMap trait
            // cmk could also do something in place to avoid the clone
            let new_range = range_value.range.start() + addend..=range_value.range.end() + addend;
            (new_range, range_value.value.clone())
        })
        .collect()
}

pub fn reverse(frames: &RangeMapBlaze<i32, String>) -> RangeMapBlaze<i32, String> {
    let first = frames.first_key_value().unwrap().0; // cmk all these unwraps and asserts could be Results
    let last = frames.last_key_value().unwrap().0;
    frames
        .range_values()
        .map(|range_value| {
            let (start, end) = range_value.range.into_inner();
            let new_range = (last - end + first)..=(last - start + first);
            (new_range, range_value.value.clone())
        })
        .collect()
}

pub fn linear(
    range_map_blaze: &RangeMapBlaze<i32, String>,
    scale: i32,
    shift: i32,
) -> RangeMapBlaze<i32, String> {
    if range_map_blaze.is_empty() {
        return RangeMapBlaze::new();
    }

    let first = range_map_blaze.first_key_value().unwrap().0;
    let last = range_map_blaze.last_key_value().unwrap().0;

    range_map_blaze
        .range_values()
        .map(|range_value| {
            let (start, end) = range_value.range.clone().into_inner();
            let mut a = (start - first) * scale.abs() + first;
            let mut b = (end + 1 - first) * scale.abs() + first - 1;
            let last = (last + 1 - first) * scale.abs() + first - 1;
            if scale < 0 {
                (a, b) = (last - b + first, last - a + first);
            }
            a += shift;
            b += shift;
            let new_range = a..=b;
            (new_range, range_value.value.clone())
        })
        .collect()
}
// cmk make range_values a DoubleEndedIterator

#[test]
fn string_animation() {
    let fps: i32 = 24;
    let length_seconds = 10;
    let frame_count = fps * length_seconds;

    let mut main = RangeMapBlaze::from_iter([(0..=frame_count - 1, "<blank>".to_string())]);

    // Create frames of 0 to 9
    let mut digits = RangeMapBlaze::from_iter((0..=9).map(|i| (i..=i, i.to_string())));
    // Replace 0 with "start"
    digits.insert(0, "start".to_string());
    // Trim 8 and 9 -- panic if missing
    assert!(digits.remove(8).is_some());
    assert!(digits.remove(9).is_some());

    // cmk ALTERNATIVES
    // digits.ranges_remove(8..=9); // also needed on RangeSetBlaze
    // digits =- 8..=9;
    // digits = digits - 8..=9;

    // Make each number last 1 second
    // cmk could be a method on RangeMapBlaze
    digits = linear(&digits, -fps, 5);
    println!("digits m {digits:?}");
    // reverse it
    // digits = linear(&digits, -1, 0);
    // println!("digits r '{digits:?}'");
    // paste it on top of main
    main = &main | &digits;
    // // shift it 10 seconds and paste that on top of main
    main = &main | &linear(&digits, 1, 10 * fps);
    println!("main dd {main:?}");
    play_movie(main, fps);
}
