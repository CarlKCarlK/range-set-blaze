#![cfg(test)]
#![cfg(not(target_arch = "wasm32"))]
extern crate alloc;
// cmk000 use self::map::ValueOwned;
// cmk000 use super::*;
// use crate::intersection_iter_map::IntersectionIterMap;
// use crate::sorted_disjoint_map::DebugToString;
// use crate::sorted_disjoint_map::SortedDisjointMap;
// use crate::sym_diff_iter_map::SymDiffIterMap;
// use crate::union_iter_map::UnionIterMap;
// use crate::unsorted_disjoint_map::{ UnsortedPriorityDisjointMap};
use alloc::collections::BTreeMap;
use core::fmt;
use core::ops::RangeInclusive;
use itertools::Itertools;
use rand::seq::SliceRandom;
use rand::{rngs::StdRng, Rng, SeedableRng};
use range_set_blaze::AssumePrioritySortedStartsMap;
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
// use range_map_blaze::Rog;
// // cmk add RangeMapBlaze to prelude
// use std::collections::BTreeMap;
use range_set_blaze::prelude::*;

// cmk0 add some to prelude?
use range_set_blaze::range_values::RangeValuesIter;
use range_set_blaze::CloneBorrow;
use range_set_blaze::IntoRangeValuesIter;
use range_set_blaze::UnionIterMap;
use range_set_blaze::UniqueValue;
use range_set_blaze::ValueOwned;
use range_set_blaze::{
    CheckSortedDisjoint, CheckSortedDisjointMap, IntersectionIterMap, MultiwaySortedDisjointMap,
    SymDiffIterMap,
};
use std::iter::FusedIterator;

// use range_set_blaze::{
//     MultiwayRangeMapBlaze, RangeMapBlaze, RangeSetBlaze, SortedDisjoint, SortedDisjointMap,
// };
// cmk not tested use range_map_blaze::multiway_map::MultiwayRangeMapBlazeRef;
use range_set_blaze::Integer;
type I32SafeLen = <i32 as Integer>::SafeLen;
use std::{
    io::{stdout, Write},
    thread::sleep,
    time::Duration,
};

//     prelude::*, AssumeSortedStarts, Integer, NotIter, RangeMapBlaze, RangesIter, SortedStarts,
//     UnionIterMap,
// };
// use std::cmp::Ordering;
// #[cfg(feature = "rog-experimental")]
// use std::panic::AssertUnwindSafe;
// #[cfg(feature = "rog-experimental")]
// use std::panic::{self};
// use std::time::Instant;
// use std::{collections::BTreeSet, ops::BitOr};
use syntactic_for::syntactic_for;
// use tests_common::{k_sets, width_to_range, How, MemorylessIter, MemorylessRange};

// type I32SafeLen = <i32 as range_map_blaze::Integer>::SafeLen;

#[test]
fn map_map_operators() {
    let arm = RangeMapBlaze::from_iter([(1, "Hello"), (2, "World"), (3, "World")]);
    let brm = RangeMapBlaze::from_iter([(2, "Go"), (3, "Go"), (4, "Go")]);
    let adm = arm.range_values();
    let bdm = brm.range_values();
    let ads = arm.ranges();
    let bds = brm.ranges();
    let ars = ads.into_range_set_blaze();
    let brs = bds.into_range_set_blaze();

    // RangeSetBlaze
    // union, intersection, difference, symmetric_difference, complement
    let _ = &ars | &brs;
    let _ = &ars & &brs;
    let _ = &ars - &brs;
    let _ = &ars ^ &brs;
    let _ = !&ars;

    // SortedDisjointSet
    // union, intersection, difference, symmetric_difference, complement
    let ads = arm.ranges();
    let bds = brm.ranges();
    let _ = ads.union(bds);
    let ads = arm.ranges();
    let bds = brm.ranges();
    let _ = ads.intersection(bds);
    let ads = arm.ranges();
    let bds = brm.ranges();
    let _ = ads.difference(bds);
    let ads = arm.ranges();
    let bds = brm.ranges();
    let _ = ads.symmetric_difference(bds);
    let ads = arm.ranges();
    let _ = ads.complement();

    // RangeMapBlaze/RangeMapBlaze
    // union, intersection, difference, symmetric_difference, complement
    let _ = &arm | &brm;
    let _ = &arm & &brm;
    let _ = &arm - &brm;
    let _ = &arm ^ &brm;
    let _ = !&arm;

    // RangeMapBlaze/RangeSetBlaze
    // intersection, difference
    let _ = &arm & &brs;
    let _ = &arm - &brs;

    // SortedDisjointMap/SortedDisjointMap
    // union, intersection, difference, symmetric_difference, complement
    let _ = adm.union(bdm);
    let adm = arm.range_values();
    let bdm = brm.range_values();
    let _ = adm.intersection_with_set(bdm.into_sorted_disjoint());
    let adm = arm.range_values();
    let bdm = brm.range_values();
    let _ = adm.difference_with_set(bdm.into_sorted_disjoint());
    // symmetric_difference on streams not supported because
    // efficient implementation would require a new iterator type.
    // let adm = arm.range_values();
    // let bdm = brm.range_values();
    // let _ = adm.symmetric_difference(bdm);
    let adm = arm.range_values();
    let _ = adm.complement_to_set();

    // SortedDisjointMap/SortedDisjointSet
    // intersection, difference
    let adm = arm.range_values();
    let bds = brm.ranges();
    let _ = adm.intersection_with_set(bds);
    let adm = arm.range_values();
    let bds = brm.ranges();
    let _ = adm.difference_with_set(bds);
}

#[test]
fn map_insert_255u8() {
    let btree_map = BTreeMap::from_iter([(255u8, "First")]);
    assert_eq!(btree_map.get(&255u8), Some(&"First"));
    let range_map_blaze = RangeMapBlaze::from_iter([(255u8, "First".to_string())]);
    assert_eq!(range_map_blaze.to_string(), r#"(255..=255, "First")"#);

    let iter = [
        (255u8..=255, "Hello".to_string()), // cmk to u8
        (25..=25, "There".to_string()),
    ]
    .into_iter();
    let range_map_blaze = RangeMapBlaze::<_, String>::from_iter(iter);
    assert_eq!(
        range_map_blaze.to_string(),
        r#"(25..=25, "There"), (255..=255, "Hello")"#
    );
}

#[test]
#[should_panic]
fn map_insert_max_u128() {
    let _ = RangeMapBlaze::<u128, _>::from_iter([(u128::MAX, "Too Big")]);
}

#[test]
fn map_complement0() {
    syntactic_for! { ty in [i8, u8, isize, usize,  i16, u16, i32, u32, i64, u64, isize, usize, i128, u128] {
        $(
        let empty = RangeMapBlaze::<$ty,u8>::new();
        let full = !&empty;
        println!("empty: {empty} (len {}), full: {full} (len {})", empty.len(), full.len());
        )*
    }};
}

#[test]
fn map_repro_bit_and() {
    let a = RangeMapBlaze::from_iter([(1u8, "Hello"), (2, "Hello"), (3, "Hello")]);
    let b = RangeMapBlaze::from_iter([(2u8, "World"), (3, "World"), (4, "World")]);

    let result = &a & &b;
    assert_eq!(result, RangeMapBlaze::from_iter([(2u8..=3, "Hello")]));

    let a = RangeSetBlaze::from_iter([1u8, 2, 3]);
    let b = RangeSetBlaze::from_iter([2u8, 3, 4]);

    let result = a.ranges().intersection(b.ranges());
    let result = result.into_range_set_blaze();
    println!("{result}");
    assert_eq!(result, RangeSetBlaze::from_iter([2, 3]));

    let result = a & b;
    println!("{result}");
    assert_eq!(result, RangeSetBlaze::from_iter([2, 3]));

    let a = RangeMapBlaze::from_iter([(1, "Hello"), (2, "World"), (3, "World")]);
    let b = RangeMapBlaze::from_iter([(2, "Go"), (3, "Go"), (4, "Go")]);

    let result = a
        .range_values()
        .intersection_with_set(b.ranges())
        .into_range_map_blaze();
    println!("{result}");
    assert_eq!(result, RangeMapBlaze::from_iter([(2..=3, "World")]));

    let result = a & b;
    println!("{result}");
    assert_eq!(result, RangeMapBlaze::from_iter([(2..=3, "World")]));
}

#[test]
fn map_doctest1() {
    let a = RangeMapBlaze::from_iter([(1u8, "Hello"), (2, "Hello"), (3, "Hello")]);
    let b = RangeMapBlaze::from_iter([(3u8, "World"), (4, "World"), (5, "World")]);

    let result = &a | &b;
    assert_eq!(
        result,
        RangeMapBlaze::<u8, &str>::from_iter([(1..=3, "Hello"), (4..=5, "World")])
    );

    let a = RangeMapBlaze::<u8, _>::from_iter([(1, "Hello"), (2, "World"), (3, "World")]);
    let b = RangeMapBlaze::<u8, _>::from_iter([(3, "Go"), (4, "Go"), (5, "Go")]);

    let result = &a | &b;
    assert_eq!(
        result,
        RangeMapBlaze::<u8, _>::from_iter([
            (1, "Hello"),
            (2, "World"),
            (3, "World"),
            (4, "Go"),
            (5, "Go")
        ])
    );
}

#[test]
fn map_doctest2() {
    let set = RangeMapBlaze::from_iter([(1u8, "Hello"), (2, "Hello"), (3, "Hello")]);
    assert!(set.contains_key(1));
    assert!(!set.contains_key(4));

    let set = RangeMapBlaze::<u8, _>::from_iter([(1, "Hello"), (2, "World"), (3, "World")]);
    assert_eq!(set.get(1), Some(&"Hello"));
    assert_eq!(set.get(4), None);
}

#[test]
fn map_doctest3() {
    let mut a = RangeMapBlaze::from_iter([(1..=3, "Hello")]);
    let mut b = RangeMapBlaze::from_iter([(3..=5, "World")]);

    a.append(&mut b);

    assert_eq!(a.len(), 5usize);
    assert_eq!(b.len(), 0usize);

    assert_eq!(a.get(1), Some(&"Hello"));
    assert_eq!(a.get(2), Some(&"Hello"));
    assert_eq!(a.get(3), Some(&"World"));
    assert_eq!(a.get(4), Some(&"World"));
    assert_eq!(a.get(5), Some(&"World"));

    let mut a = RangeMapBlaze::from_iter([(1u8..=3, "Hello")]);
    let mut b = RangeMapBlaze::from_iter([(3u8..=5, "World")]);

    a.append(&mut b);

    assert_eq!(a.len(), 5usize);
    assert_eq!(b.len(), 0usize);

    assert!(a.contains_key(1));
    assert!(a.contains_key(2));
    assert!(a.contains_key(3));
    assert!(a.contains_key(4));
    assert!(a.contains_key(5));
}

#[test]
fn map_doctest4() {
    let a = RangeMapBlaze::from_iter([(1i8, "Hello"), (2, "Hello"), (3, "Hello")]);

    let result = !&a;
    assert_eq!(result.to_string(), "-128..=0, 4..=127");
}

#[test]
fn map_add_in_order() {
    let mut range_map = RangeMapBlaze::new();
    for i in 0u64..1000 {
        range_map.insert(i, i);
    }
    assert_eq!(
        range_map,
        RangeMapBlaze::from_iter((0..1000).map(|i| (i, i)))
    );
}

// cmk do these benchmark related
// #[test]
// fn map_memoryless_data() {
//     let len = 100_000_000;
//     let coverage_goal = 0.75;
//     let memoryless_data = MemorylessData::new(0, 10_000_000, len, coverage_goal);
//     let range_map_blaze = RangeMapBlaze::from_iter(memoryless_data);
//     let coverage = range_map_blaze.len() as f64 / len as f64;
//     println!(
//         "coverage {coverage:?} range_len {:?}",
//         range_map_blaze.range_len().separate_with_commas()
//     );
// }

// #[test]
// fn map_memoryless_vec() {
//     let len = 100_000_000;
//     let coverage_goal = 0.75;
//     let memoryless_data = MemorylessData::new(0, 10_000_000, len, coverage_goal);
//     let data_as_vec: Vec<u64> = memoryless_data.collect();
//     let start = Instant::now();
//     // let range_map_blaze = RangeMapBlaze::from_mut_slice(data_as_vec.as_mut_slice());
//     let range_map_blaze = RangeMapBlaze::from_iter(data_as_vec);
//     let coverage = range_map_blaze.len() as f64 / len as f64;
//     println!(
//         "coverage {coverage:?} range_len {:?}",
//         range_map_blaze.ranges_len().separate_with_commas()
//     );
//     println!(
//         "xTime elapsed in expensive_function() is: {} ms",
//         start.elapsed().as_millis()
//     );
// }

#[test]
fn map_iters() -> Result<(), Box<dyn std::error::Error>> {
    let range_map_blaze =
        RangeMapBlaze::from_iter([(1u8..=6, "Hello"), (8..=9, "There"), (11..=15, "World")]);
    assert!(range_map_blaze.len() == 13);
    for (k, v) in range_map_blaze.iter() {
        println!("{k}:{v}");
    }
    for range in range_map_blaze.ranges() {
        println!("{range:?}");
    }
    let mut rs = range_map_blaze.range_values();
    println!("{:?}", rs.next());
    println!("{range_map_blaze}");
    println!("{:?}", rs.len());
    println!("{:?}", rs.next());
    for (k, v) in range_map_blaze.iter() {
        println!("{k}:{v}");
    }
    // range_map_blaze.len();

    let mut rs = range_map_blaze.range_values().complement_to_set();
    println!("{:?}", rs.next());
    println!("{range_map_blaze}");
    // !!! assert that can't use range_map_blaze again
    Ok(())
}

#[test]
fn map_missing_doctest_ops() {
    // note that may be borrowed or owned in any combination.

    // Returns the union of `self` and `rhs` as a new [`RangeMapBlaze`].
    let a = RangeMapBlaze::from_iter([(1, "Hello"), (2, "Hello"), (3, "Hello")]);
    let b = RangeMapBlaze::from_iter([(3, "World"), (4, "World"), (5, "World")]);

    let result = &a | &b;
    assert_eq!(
        result,
        RangeMapBlaze::from_iter([
            (1, "Hello"),
            (2, "Hello"),
            (3, "Hello"),
            (4, "World"),
            (5, "World")
        ])
    );
    let result = a | &b;
    assert_eq!(
        result,
        RangeMapBlaze::from_iter([
            (1, "Hello"),
            (2, "Hello"),
            (3, "Hello"),
            (4, "World"),
            (5, "World")
        ])
    );

    // Returns the complement of `self` as a new [`RangeMapBlaze`].
    let a = RangeMapBlaze::<i8, _>::from_iter([(1, "Hello"), (2, "Hello"), (3, "Hello")]);

    let result = !&a;
    assert_eq!(result.to_string(), "-128..=0, 4..=127");
    let result = !a;
    assert_eq!(result.to_string(), "-128..=0, 4..=127");

    // Returns the intersection of `self` and `rhs` as a new `RangeMapBlaze<T>`.

    let a = RangeMapBlaze::from_iter([(1, "Hello"), (2, "Hello"), (3, "Hello")]);
    let b = RangeMapBlaze::from_iter([(3, "World"), (4, "World"), (5, "World")]);

    let result = a & &b;
    assert_eq!(result, RangeMapBlaze::from_iter([(3, "Hello")]));
    let a = RangeMapBlaze::from_iter([(1, "Hello"), (2, "Hello"), (3, "Hello")]);
    let result = a & b;
    assert_eq!(result, RangeMapBlaze::from_iter([(3, "Hello")]));

    // Returns the symmetric difference of `self` and `rhs` as a new `RangeMapBlaze<T>`.
    let a = RangeMapBlaze::from_iter([(1, "Hello"), (2, "Hello"), (3, "Hello")]);
    let b = RangeMapBlaze::from_iter([(2, "World"), (3, "World"), (4, "World")]);

    let result = a ^ b;
    assert_eq!(
        result,
        RangeMapBlaze::from_iter([(1, "Hello"), (4, "World")])
    );

    // Returns the set difference of `self` and `rhs` as a new `RangeMapBlaze<T>`.
    let a = RangeMapBlaze::from_iter([(1, "Hello"), (2, "Hello"), (3, "Hello")]);
    let b = RangeMapBlaze::from_iter([(2, "World"), (3, "World"), (4, "World")]);

    let result = a - b;
    assert_eq!(result, RangeMapBlaze::from_iter([(1, "Hello")]));

    // note that may be borrowed or owned in any combination.

    // Returns the union of `self` and `rhs` as a new [`RangeMapBlaze`].
    let a = RangeMapBlaze::from_iter([(1, "Hello"), (2, "World"), (3, "World")]);
    let b = RangeMapBlaze::from_iter([(3, "Go"), (4, "Go"), (5, "Go")]);

    let result = &a | &b;
    assert_eq!(
        result,
        RangeMapBlaze::from_iter([
            (1, "Hello"),
            (2, "World"),
            (3, "World"),
            (4, "Go"),
            (5, "Go")
        ])
    );
    let result = a | &b;
    assert_eq!(
        result,
        RangeMapBlaze::from_iter([
            (1, "Hello"),
            (2, "World"),
            (3, "World"),
            (4, "Go"),
            (5, "Go")
        ])
    );

    // Returns the intersection of `self` and `rhs` as a new `RangeMapBlaze<T>`.

    let a = RangeMapBlaze::from_iter([(1, "Hello"), (2, "World"), (3, "World")]);
    let b = RangeMapBlaze::from_iter([(2, "Go"), (3, "Go"), (4, "Go")]);

    let result = a & &b;
    assert_eq!(
        result,
        RangeMapBlaze::from_iter([(2, "World"), (3, "World")])
    );
    let a = RangeMapBlaze::from_iter([(1, "Hello"), (2, "World"), (3, "World")]);
    let result = a & b;
    assert_eq!(
        result,
        RangeMapBlaze::from_iter([(2, "World"), (3, "World")])
    );

    // Returns the symmetric difference of `self` and `rhs` as a new `RangeMapBlaze<T>`.
    let a = RangeMapBlaze::from_iter([(1, "Hello"), (2, "World"), (3, "World")]);
    let b = RangeMapBlaze::from_iter([(3, "Go"), (4, "Go"), (5, "Go")]);

    let result = a ^ b;
    assert_eq!(
        result,
        RangeMapBlaze::from_iter([(1..=1, "Hello"), (2..=2, "World"), (4..=5, "Go")])
    );

    // Returns the set difference of `self` and `rhs` as a new `RangeMapBlaze<T>`.
    let a = RangeMapBlaze::from_iter([(1, "Hello"), (2, "World"), (3, "World")]);
    let b = RangeMapBlaze::from_iter([(2, "Go"), (3, "Go"), (4, "Go")]);

    let result = a - b;
    assert_eq!(result, RangeMapBlaze::from_iter([(1, "Hello")]));
}

#[test]
fn map_multi_op() -> Result<(), Box<dyn std::error::Error>> {
    // Union
    let a = RangeMapBlaze::from_iter([(1..=6, 'a'), (8..=9, 'a'), (11..=15, 'a')]);
    let b = RangeMapBlaze::from_iter([(5..=13, 'b'), (18..=29, 'b')]);
    let c = RangeMapBlaze::from_iter([(38..=42, 'c')]);
    let d = &(&a | &b) | &c;
    assert_eq!(
        d,
        RangeMapBlaze::from_iter([
            (1..=6, 'a'),
            (7..=7, 'b'),
            (8..=9, 'a'),
            (10..=10, 'b'),
            (11..=15, 'a'),
            (18..=29, 'b'),
            (38..=42, 'c')
        ])
    );
    let d = a | b | &c;
    assert_eq!(
        d,
        RangeMapBlaze::from_iter([
            (1..=6, 'a'),
            (7..=7, 'b'),
            (8..=9, 'a'),
            (10..=10, 'b'),
            (11..=15, 'a'),
            (18..=29, 'b'),
            (38..=42, 'c')
        ])
    );

    let a = RangeMapBlaze::from_iter([(1..=6, 'a'), (8..=9, 'a'), (11..=15, 'a')]);
    let b = RangeMapBlaze::from_iter([(5..=13, 'b'), (18..=29, 'b')]);
    let c = RangeMapBlaze::from_iter([(38..=42, 'c')]);

    let _ = [&a, &b, &c].union();
    let d = [a, b, c].intersection();
    assert_eq!(d, RangeMapBlaze::new());

    assert_eq!(
        MultiwayRangeMapBlaze::<u8, char>::union([]),
        RangeMapBlaze::new()
    );

    // Intersection
    let a = RangeMapBlaze::from_iter([(1..=6, 'a'), (8..=9, 'a'), (11..=15, 'a')]);
    let b = RangeMapBlaze::from_iter([(5..=13, 'b'), (18..=29, 'b')]);
    let c = RangeMapBlaze::from_iter([(1..=42, 'c')]);

    let _ = &a & &b;
    let d = [&a, &b, &c].intersection();
    // let d = RangeMapBlaze::intersection([a, b, c]);
    println!("{d}");
    assert_eq!(
        d,
        RangeMapBlaze::from_iter([(5..=6, 'a'), (8..=9, 'a'), (11..=13, 'a')])
    );

    // not defined on 0 maps because the range would be the universe (fine), but we don't know what value to use.
    // assert_eq!(
    //     MultiwayRangeMapBlaze::<u8, char>::intersection([]),
    //     RangeMapBlaze::from_iter([(0..=255, '?')])
    // );

    // Symmetric Difference

    let a = RangeMapBlaze::from_iter([(1..=6, 'a'), (8..=9, 'a'), (11..=15, 'a')]);
    let b = RangeMapBlaze::from_iter([(5..=13, 'b'), (18..=29, 'b')]);
    let c = RangeMapBlaze::from_iter([(38..=42, 'c')]);
    let d = &(&a ^ &b) ^ &c;
    assert_eq!(
        d,
        RangeMapBlaze::from_iter([
            (1..=4, 'a'),
            (7..=7, 'b'),
            (10..=10, 'b'),
            (14..=15, 'a'),
            (18..=29, 'b'),
            (38..=42, 'c')
        ])
    );
    let d = a ^ b ^ &c;
    assert_eq!(
        d,
        RangeMapBlaze::from_iter([
            (1..=4, 'a'),
            (7..=7, 'b'),
            (10..=10, 'b'),
            (14..=15, 'a'),
            (18..=29, 'b'),
            (38..=42, 'c')
        ])
    );

    let a = RangeMapBlaze::from_iter([(1..=6, 'a'), (8..=9, 'a'), (11..=15, 'a')]);
    let b = RangeMapBlaze::from_iter([(5..=13, 'b'), (18..=29, 'b')]);
    let c = RangeMapBlaze::from_iter([(38..=42, 'c')]);

    let _ = [&a, &b, &c].symmetric_difference();

    assert_eq!(
        MultiwayRangeMapBlaze::<u8, char>::symmetric_difference([]),
        RangeMapBlaze::new()
    );

    Ok(())
}

#[test]
fn map_custom_multi() -> Result<(), Box<dyn std::error::Error>> {
    let a = RangeMapBlaze::from_iter([(1..=6, 'a'), (8..=9, 'a'), (11..=15, 'a')]);
    let b = RangeMapBlaze::from_iter([(5..=13, 'b'), (18..=29, 'b')]);
    let c = RangeMapBlaze::from_iter([(38..=42, 'c')]);

    let union_stream = b.range_values().union(c.range_values());
    let a_less = a
        .range_values()
        .difference_with_set(union_stream.into_sorted_disjoint());
    let d: RangeMapBlaze<_, _> = a_less.into_range_map_blaze();
    assert_eq!(d, RangeMapBlaze::from_iter([(1..=4, 'a'), (14..=15, 'a')]));

    let d: RangeMapBlaze<_, _> = a
        .range_values()
        .difference_with_set(
            [b.range_values(), c.range_values()]
                .union()
                .into_sorted_disjoint(),
        )
        .into_range_map_blaze();
    assert_eq!(d, RangeMapBlaze::from_iter([(1..=4, 'a'), (14..=15, 'a')]));
    Ok(())
}

#[test]
fn map_from_string() -> Result<(), Box<dyn std::error::Error>> {
    let a = RangeMapBlaze::from_iter([
        (0..=4, 'a'),
        (14..=17, 'a'),
        (30..=255, 'a'),
        (0..=37, 'a'),
        (43..=65535, 'a'),
    ]);
    assert_eq!(a, RangeMapBlaze::from_iter([(0..=65535, 'a')]));
    Ok(())
}

#[test]
fn map_nand_repro() -> Result<(), Box<dyn std::error::Error>> {
    let b = &RangeMapBlaze::from_iter([(5u8..=13, 'a'), (18..=29, 'a')]);
    let c = &RangeMapBlaze::from_iter([(38..=42, 'b')]);
    println!("about to nand");
    let d = !b | !c;
    assert_eq!(
        d,
        RangeSetBlaze::from_iter([0..=4, 14..=17, 30..=255, 0..=37, 43..=255])
    );
    Ok(())
}

#[test]
fn map_parity() -> Result<(), Box<dyn std::error::Error>> {
    // notice these are all borrowed
    let a = &RangeMapBlaze::from_iter([(1..=6, 'a'), (8..=9, 'a'), (11..=15, 'a')]);
    let b = &RangeMapBlaze::from_iter([(5..=13, 'b'), (18..=29, 'b')]);
    let c = &RangeMapBlaze::from_iter([(38..=42, 'c')]);
    assert_eq!(
        a & b & c
            | a.intersection_with_set(&(!b & !c))
            | b.intersection_with_set(&(!a & !c))
            | c.intersection_with_set(&(!a & !b)),
        RangeMapBlaze::from_iter([
            (1..=4, 'a'),
            (7..=7, 'b'),
            (10..=10, 'b'),
            (14..=15, 'a'),
            (18..=29, 'b'),
            (38..=42, 'c')
        ])
    );
    assert_eq!(
        a ^ b ^ c,
        RangeMapBlaze::from_iter([
            (1..=4, 'a'),
            (7..=7, 'b'),
            (10..=10, 'b'),
            (14..=15, 'a'),
            (18..=29, 'b'),
            (38..=42, 'c')
        ])
    );

    let _d = [a.range_values()].intersection();
    let _parity: RangeMapBlaze<u8, _> = [[a.range_values()].intersection()]
        .union()
        .into_range_map_blaze();
    let _parity: RangeMapBlaze<u8, _> = [a.range_values()].intersection().into_range_map_blaze();
    let _parity: RangeMapBlaze<u8, _> = [a.range_values()].union().into_range_map_blaze();
    println!("!b {}", !b);
    println!("!c {}", !c);
    println!("!b|!c {}", !b | !c);
    let b_comp = (b).range_values().complement(&'B');
    let c_comp = (c).range_values().complement(&'C');
    println!(
        "!b|!c {}",
        RangeMapBlaze::from_sorted_disjoint_map(b_comp.union(c_comp))
    );

    let a = RangeMapBlaze::from_iter([(1..=6, 'a'), (8..=9, 'a'), (11..=15, 'a')]);

    let u = [DynSortedDisjointMap::new(a.range_values())].union();
    assert_eq!(
        RangeMapBlaze::from_sorted_disjoint_map(u),
        RangeMapBlaze::from_iter([(1..=6, 'a'), (8..=9, 'a'), (11..=15, 'a')])
    );
    let u = union_map_dyn!(a.range_values());
    assert_eq!(
        RangeMapBlaze::from_sorted_disjoint_map(u),
        RangeMapBlaze::from_iter([(1..=6, 'a'), (8..=9, 'a'), (11..=15, 'a')])
    );
    let u = union_map_dyn!(a.range_values(), b.range_values(), c.range_values());
    assert_eq!(
        RangeMapBlaze::from_sorted_disjoint_map(u),
        RangeMapBlaze::from_iter([
            (1..=6, 'a'),
            (7..=7, 'b'),
            (8..=9, 'a'),
            (10..=10, 'b'),
            (11..=15, 'a'),
            (18..=29, 'b'),
            (38..=42, 'c')
        ])
    );

    let u = [
        intersection_map_dyn!(
            a.range_values(),
            b.range_values().complement(&'a'),
            c.range_values().complement(&'a')
        ),
        intersection_map_dyn!(
            b.range_values(),
            a.range_values().complement(&'b'),
            c.range_values().complement(&'b')
        ),
        intersection_map_dyn!(
            c.range_values(),
            a.range_values().complement(&'c'),
            b.range_values().complement(&'c')
        ),
        intersection_map_dyn!(a.range_values(), b.range_values(), c.range_values()),
    ]
    .union();
    assert_eq!(
        RangeMapBlaze::from_sorted_disjoint_map(u),
        RangeMapBlaze::from_iter([
            (1..=4, 'a'),
            (7..=7, 'b'),
            (10..=10, 'b'),
            (14..=15, 'a'),
            (18..=29, 'b'),
            (38..=42, 'c')
        ])
    );

    assert_eq!(
        symmetric_difference_map_dyn!(a.range_values(), b.range_values(), c.range_values())
            .into_range_map_blaze(),
        RangeMapBlaze::from_iter([
            (1..=4, 'a'),
            (7..=7, 'b'),
            (10..=10, 'b'),
            (14..=15, 'a'),
            (18..=29, 'b'),
            (38..=42, 'c')
        ])
    );
    Ok(())
}

#[test]
fn map_complement() -> Result<(), Box<dyn std::error::Error>> {
    // RangeMapBlaze, RangesIter, NotIter, UnionIterMap, Tee, UnionIterMap(g)
    let a0 = RangeMapBlaze::from_iter([(1..=6, "a0")]);
    let a1 = RangeMapBlaze::from_iter([(8..=9, "a1"), (11..=15, "a1")]);
    let a = &a0 | &a1;
    let not_a = &a.complement_with("A");
    let b = a.range_values();
    let c = not_a.range_values().complement(&"A");
    let d = a0.range_values().union(a1.range_values());
    let e = a.range_values(); // with range instead of range values used 'tee' here

    let f = UnionIterMap::from_iter([
        (15, &"f"),
        (14, &"f"),
        (15, &"f"),
        (13, &"f"),
        (12, &"f"),
        (11, &"f"),
        (9, &"f"),
        (9, &"f"),
        (8, &"f"),
        (6, &"f"),
        (4, &"f"),
        (5, &"f"),
        (3, &"f"),
        (2, &"f"),
        (1, &"f"),
        (1, &"f"),
        (1, &"f"),
    ]);

    let not_b = b.complement(&"A");
    let not_c = c.complement(&"A");
    let not_d = d.complement(&"A");
    let not_e = e.complement(&"A");
    let not_f = f.complement(&"A");
    // cmk0 make .to_string_work
    // println!("not a: {:?}", not_a.range_values().into_range_map_blaze());
    // println!("not b: {:?}", not_b.into_range_map_blaze());
    // println!("not c: {:?}", not_c.into_range_map_blaze());
    // println!("not d: {:?}", not_d.into_range_map_blaze());
    // println!("not e: {:?}", not_e.into_range_map_blaze());
    // println!("not f: {:?}", not_f.into_range_map_blaze());
    assert!(not_a.range_values().equal(not_b));
    assert!(not_a.range_values().equal(not_c));
    assert!(not_a.range_values().equal(not_d));
    assert!(not_a.range_values().equal(not_e));
    assert!(not_a.range_values().equal(not_f));
    Ok(())
}

#[test]
fn map_union_test() -> Result<(), Box<dyn std::error::Error>> {
    // RangeMapBlaze, RangesIter, NotIter, UnionIterMap, Tee, UnionIterMap(g)
    let a0 = RangeMapBlaze::from_iter([(1..=6, "a0")]);
    let a0_tee = a0.range_values(); // with range instead of range values used 'tee' here
    let a1 = RangeMapBlaze::from_iter([(8..=9, "a1")]);
    let a2 = RangeMapBlaze::from_iter([(11..=15, "a2")]);
    let a12 = &a1 | &a2;
    let not_a0 = &a0.complement_with(&"A0");
    let a = &a0 | &a1 | &a2;
    let b = a0
        .range_values()
        .union(a1.range_values())
        .union(a2.range_values());
    let c = not_a0
        .range_values()
        .complement(&"a0")
        .union(a12.range_values());
    let d = a0
        .range_values()
        .union(a1.range_values())
        .union(a2.range_values());
    let e = a0_tee.union(a12.range_values());

    let f = UnionIterMap::from_iter(a0.iter())
        .union(UnionIterMap::from_iter(a1.iter()))
        .union(UnionIterMap::from_iter(a2.iter()));
    assert!(a.range_values().equal(b));
    assert!(a.range_values().equal(c));
    assert!(a.range_values().equal(d));
    assert!(a.range_values().equal(e));
    assert!(a.range_values().equal(f));
    Ok(())
}

#[test]
fn map_sub() -> Result<(), Box<dyn std::error::Error>> {
    // use range_set_blaze::UnionIter;

    // RangeMapBlaze, RangesIter, NotIter, UnionIterMap, Tee, UnionIterMap(g)
    let a0 = RangeMapBlaze::from_iter([(1..=6, "a0")]);
    let a1 = RangeMapBlaze::from_iter([(8..=9, "a1")]);
    let a2 = RangeMapBlaze::from_iter([(11..=15, "a2")]);

    let a01 = &a0 | &a1;
    let a01_tee = a01.range_values(); // with range instead of range values used 'tee' here
    let not_a01 = &a01.complement_with(&"A01");
    let a = &a01 - &a2;
    let b = a01.range_values() - a2.range_values();
    let c = !not_a01.range_values() - a2.ranges();
    let d = (a0.range_values() | a1.range_values()) - a2.range_values();
    let e = a01_tee.difference_with_set(a2.ranges());
    // cmk0 let f = UnionIterMap::from_iter(a01.iter()) - UnionIter::from_iter(a2.keys());
    assert!(a.range_values().equal(b));
    assert!(a.ranges().equal(c));
    assert!(a.range_values().equal(d));
    assert!(a.range_values().equal(e));
    // cmk0 assert!(a.range_values().equal(f));

    Ok(())
}

// cmk streaming xor not currently implemented
// #[test]
// fn map_xor() -> Result<(), Box<dyn std::error::Error>> {
//     // RangeMapBlaze, RangesIter, NotIter, UnionIterMap, Tee, UnionIterMap(g)
//     let a0 = RangeMapBlaze::from_iter([(1..=6, "a0")]);
//     let a1 = RangeMapBlaze::from_iter([(8..=9, "a1")]);
//     let a2 = RangeMapBlaze::from_iter([(11..=15, "a2")]);

//     let a01 = &a0 | &a1;
//     let a01_tee = a01.range_values(); // with range instead of range values used 'tee' here
//     let not_a01 = !&a01;
//     let a = &a01 ^ &a2;
//     let b = a01.range_values() ^ a2.range_values();
//     let c = !not_a01.range_values() ^ a2.range_values();
//     let d = (a0.range_values() | a1.range_values()) ^ a2.range_values();
//     let e = a01_tee.symmetric_difference(a2.range_values());
//     let f = UnionIterMap::from_iter(a01.iter()) ^ UnionIterMap::from_iter(a2.iter());
//     assert!(a.range_values().equal(b));
//     assert!(a.range_values().equal(c));
//     assert!(a.range_values().equal(d));
//     assert!(a.range_values().equal(e));
//     assert!(a.range_values().equal(f));
//     Ok(())
// }

#[test]
fn map_bitand() -> Result<(), Box<dyn std::error::Error>> {
    // use range_set_blaze::UnionIter;

    // RangeMapBlaze, RangesIter, NotIter, UnionIterMap, Tee, UnionIterMap(g)
    let a0 = RangeMapBlaze::from_iter([(1..=6, "a0")]);
    let a1 = RangeMapBlaze::from_iter([(8..=9, "a1")]);
    let a2 = RangeMapBlaze::from_iter([(11..=15, "a2")]);

    let a01 = &a0 | &a1;
    let a01_tee = a01.range_values(); // with range instead of range values used 'tee' here
    let not_a01 = &a01.complement_with(&"A01");
    let a = &a01 & &a2;
    let b = a01.range_values() & a2.range_values();
    let c = !not_a01.range_values() & a2.ranges();
    let d = (a0.range_values() | a1.range_values()) & a2.range_values();
    let e = a01_tee.intersection_with_set(a2.ranges());
    // cmk00 let f = UnionIterMap::from_iter(a01.iter()) & UnionIter::from_iter(a2.keys());
    assert!(a.range_values().equal(b));
    assert!(a.ranges().equal(c));
    assert!(a.range_values().equal(d));
    assert!(a.range_values().equal(e));
    // cmk00 assert!(a.range_values().equal(f));
    Ok(())
}

// cmk add symmetric_difference test
#[test]
fn map_empty_it() {
    use core::panic::AssertUnwindSafe;
    use std::ops::BitOr;
    use std::panic;

    let universe0 = RangeMapBlaze::from_iter([(0u8..=255, "Universe")]);
    let universe = universe0.range_values();
    let arr: [(u8, &str); 0] = [];
    let a0 = RangeMapBlaze::<u8, &str>::from_iter(arr);
    assert!(!(a0.ranges()).equal(universe0.ranges()));
    assert!((a0.complement_with(&"Universe"))
        .range_values()
        .equal(universe));
    let _a0 = RangeMapBlaze::from_iter([(0..=0, "One"); 0]);
    let _a = RangeMapBlaze::<i32, &str>::new();

    let a_iter: std::array::IntoIter<(i32, &str), 0> = [].into_iter();
    let a = a_iter.collect::<RangeMapBlaze<i32, &str>>();
    let b = RangeMapBlaze::from_iter([(0i32, &"ignored"); 0]);
    let mut c3 = a.clone();
    let mut c5 = a.clone();

    let c0 = (&a).bitor(&b);
    let c1a = &a | &b;
    let c1b = &a | b.clone();
    let c1c = a.clone() | &b;
    let c1d = a.clone() | b.clone();
    let c2: RangeMapBlaze<_, _> = (a.range_values() | b.range_values()).into_range_map_blaze();
    c3.append(&mut b.clone());
    c5.extend(b);

    let answer = RangeMapBlaze::from_iter([(0, &"ignored"); 0]);
    assert_eq!(&c0, &answer);
    assert_eq!(&c1a, &answer);
    assert_eq!(&c1b, &answer);
    assert_eq!(&c1c, &answer);
    assert_eq!(&c1d, &answer);
    assert_eq!(&c2, &answer);
    assert_eq!(&c3, &answer);
    assert_eq!(&c5, &answer);
    let a_iter: std::array::IntoIter<(i32, &str), 0> = [].into_iter();
    let a = a_iter.collect::<RangeMapBlaze<i32, &str>>();
    let b = RangeMapBlaze::from_iter([(0, &"ignore"); 0]);

    let c0 = a.range_values() | b.range_values();
    let c1 = [a.range_values(), b.range_values()].union();
    let c_list2: [RangeValuesIter<i32, &str>; 0] = [];
    let c2 = c_list2.clone().union();
    let c3 = union_map_dyn!(a.range_values(), b.range_values());
    let c4 = c_list2.map(DynSortedDisjointMap::new).union();

    let val = "ignored";
    let answer = RangeMapBlaze::from_iter([(0, &val); 0]);
    assert!(c0.equal(answer.range_values()));
    let answer = RangeMapBlaze::from_iter([(0, &val); 0]);
    assert!(c1.equal(answer.range_values()));
    let answer = RangeMapBlaze::from_iter([(0, &val); 0]);
    assert!(c2.equal(answer.range_values()));
    let answer = RangeMapBlaze::from_iter([(0, &val); 0]);
    assert!(c3.equal(answer.range_values()));
    let answer = RangeMapBlaze::from_iter([(0, &val); 0]);
    assert!(c4.equal(answer.range_values()));

    let c0 = !(a.range_values() & b.range_values());
    let c1 = ![a.range_values(), b.range_values()].intersection();
    let c_list2: [RangeValuesIter<i32, &str>; 0] = [];
    assert!(
        panic::catch_unwind(AssertUnwindSafe(|| { !!c_list2.clone().intersection() })).is_err(),
        "Expected a panic."
    );
    let c3 = !intersection_map_dyn!(a.range_values(), b.range_values());
    assert!(
        panic::catch_unwind(AssertUnwindSafe(|| {
            !!c_list2.map(DynSortedDisjointMap::new).intersection()
        }))
        .is_err(),
        "Expected a panic."
    );

    let answer = !RangeMapBlaze::from_iter([(0, "ignored"); 0]);
    assert!(c0.equal(answer.ranges()));
    assert!(c1.equal(answer.ranges()));
    assert!(c3.equal(answer.ranges()));
}

#[test]
#[allow(clippy::reversed_empty_ranges)]
fn map_tricky_case1() {
    let a = RangeMapBlaze::from_iter([(1..=0, "a")]);
    let b = RangeMapBlaze::from_iter([(2..=1, "b")]);
    assert_eq!(a, b);
    assert!(a.range_values().equal(b.range_values()));
    assert_eq!(a.range_values().len(), 0);
    assert_eq!(a.range_values().len(), b.range_values().len());
    let a = RangeMapBlaze::from_iter([(i32::MIN..=i32::MAX, "a")]);
    println!("tc1 '{a}'");
    assert_eq!(a.len() as i128, (i32::MAX as i128) - (i32::MIN as i128) + 1);
    let a = !RangeMapBlaze::from_iter([(1..=0, "a")]);
    println!("tc1 '{a}'");
    assert_eq!(a.len() as i128, (i32::MAX as i128) - (i32::MIN as i128) + 1);

    let a = !RangeMapBlaze::from_iter([(1i128..=0, "a")]);
    println!("tc1 '{a}', {}", a.len());
    assert_eq!(a.len(), u128::MAX);
    let a = !RangeMapBlaze::from_iter([(1u128..=0, "a")]);
    println!("tc1 '{a}', {}", a.len());
    assert_eq!(a.len(), u128::MAX);
}

// should fail
#[test]
#[should_panic]
fn map_tricky_case2() {
    let _a = RangeMapBlaze::from_iter([(-1..=i128::MAX, "a")]);
}

#[test]
#[should_panic]
fn map_tricky_case3() {
    let _a = RangeMapBlaze::from_iter([(0..=u128::MAX, "a")]);
}

#[test]
fn map_constructors() -> Result<(), Box<dyn std::error::Error>> {
    use range_set_blaze::Priority;

    // #9: new
    let mut _range_map_blaze;
    _range_map_blaze = RangeMapBlaze::<i32, &str>::new();
    // #10 collect / from_iter T
    _range_map_blaze = [(1, "a"), (5, "b"), (6, "b"), (5, "b")]
        .into_iter()
        .collect();
    _range_map_blaze = RangeMapBlaze::from_iter([(1, "a"), (5, "b"), (6, "b"), (5, "b")]);
    // #11 into / from array T
    _range_map_blaze = [(1, "a"), (5, "b"), (6, "b"), (5, "b")].into();
    _range_map_blaze = RangeMapBlaze::from_iter([(1, "a"), (5, "b"), (6, "b"), (5, "b")]);
    // #12 into / from slice T
    // _range_map_blaze = [(1, "a"), (5, "b"), (6, "b"), (5, "b")][1..=2].into();
    // _range_map_blaze = RangeMapBlaze::from_iter([(1, "a"), (5, "b"), (6, "b"), (5, "b")].as_slice());
    //#13 collect / from_iter range
    _range_map_blaze = [(5..=6, "a"), (1..=5, "b")].into_iter().collect();
    _range_map_blaze = RangeMapBlaze::from_iter([(5..=6, "a"), (1..=5, "b")]);
    // #16 into / from iter (T,T) + SortedDisjoint
    _range_map_blaze = _range_map_blaze.range_values().into_range_map_blaze();
    _range_map_blaze = RangeMapBlaze::from_sorted_disjoint_map(_range_map_blaze.range_values());

    let sorted_starts = AssumePrioritySortedStartsMap::new(
        [
            Priority::new((5..=6, UniqueValue::new("a")), 0),
            Priority::new((1..=5, UniqueValue::new("b")), 1),
        ]
        .into_iter(),
    );
    let mut _sorted_disjoint_iter;
    _sorted_disjoint_iter = UnionIterMap::new(sorted_starts);
    // // #10 collect / from_iter T
    let arr0 = [
        (1..=1, UniqueValue::new("a")),
        (5..=5, UniqueValue::new("b")),
        (6..=6, UniqueValue::new("b")),
        (5..=5, UniqueValue::new("b")),
    ];
    let mut _sorted_disjoint_iter: UnionIterMap<_, _, _, _> = arr0.into_iter().collect();
    let arr0 = [
        (1..=1, UniqueValue::new("a")),
        (5..=5, UniqueValue::new("b")),
        (6..=6, UniqueValue::new("b")),
        (5..=5, UniqueValue::new("b")),
    ];
    _sorted_disjoint_iter = UnionIterMap::from_iter(arr0);
    // // // #11 into / from array T
    // _sorted_disjoint_iter = arr0.into(); // decided not to implement
    // _sorted_disjoint_iter = UnionIterMap::from(arr0); // decided not to implement
    // // // #12 into / from slice T
    // _sorted_disjoint_iter = [(1, "a"), (5, "b"), (6, "b"), (5, "b")][1..=2].into();
    // _sorted_disjoint_iter = UnionIterMap::from([(1, "a"), (5, "b"), (6, "b"), (5, "b")].as_slice());
    // // //#13 collect / from_iter range
    // _sorted_disjoint_iter = [(5..=6, "a"), (1..=5, "b")].into_iter().collect();
    // _sorted_disjoint_iter = UnionIterMap::from_iter([(5..=6, "a"), (1..=5, "b")]);
    // // // #14 from into array range
    // _sorted_disjoint_iter = [(5..=6, "a"), (1..=5, "b")].into();
    // _sorted_disjoint_iter = UnionIterMap::from([(5..=6, "a"), (1..=5, "b")]);
    // // // #15 from into slice range
    // _sorted_disjoint_iter = [(5..=6, "a"), (1..=5, "b")][0..=1].into();
    // _sorted_disjoint_iter = UnionIterMap::from([(5..=6, "a"), (1..=5, "b")].as_slice());
    // // // #16 into / from iter (T,T) + SortedDisjoint
    let mut _sorted_disjoint_iter: UnionIterMap<_, _, _, _> =
        _range_map_blaze.range_values().collect();
    _sorted_disjoint_iter = UnionIterMap::from_iter(_range_map_blaze.range_values());

    Ok(())
}

// // #[test]
// // fn map_debug_k_play() {
// //     let mut c = Criterion::default();
// //     k_play(&mut c);
// // }

// // fn map_k_play(c: &mut Criterion) {
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
// //                     let sets = sets.iter().map(|x| DynSortedDisjointMap::new(x.range_values()));
// //                     let _answer: RangeMapBlaze<_,_> = sets.intersection().into_range_map_blaze();
// //                 },
// //                 BatchSize::SmallInput,
// //             );
// //         });
// //     }
// //     group.finish();
// // }

// // #[test]
// // fn map_data_gen() {
// //     let range = -10_000_000i32..=10_000_000;
// //     let range_len = 1000;
// //     let coverage_goal = 0.75;
// //     let k = 100;

// //     for how in [How::None, How::Union, How::Intersection] {
// //         let mut option_range_int_set: Option<RangeMapBlaze<_,_>> = None;
// //         for seed in 0..k as u64 {
// //             let r2: RangeMapBlaze<(i32,&str)> = MemorylessRange::new(
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
// // fn map_vary_coverage_goal() {
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
// // fn map_ingest_clumps_base() {
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
// //         let range_map_blaze: RangeMapBlaze<_,_> =
// //             MemorylessRange::new(&mut rng, range_len, range.clone(), coverage_goal, k, how)
// //                 .collect();

// //         let range_count_no_dups = range_map_blaze.ranges_len();
// //         let item_count_no_dups = range_map_blaze.len();
// //         let fraction_value = fraction(&range_map_blaze, &range);
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

#[test]
fn map_doc_test_insert1() {
    let mut map = RangeMapBlaze::new();

    assert_eq!(map.insert(2, "a"), None);
    assert_eq!(map.insert(2, "b"), Some("a"));
    assert_eq!(map.len(), 1 as I32SafeLen);
}

#[test]
fn map_doc_test_len() {
    let mut v = RangeMapBlaze::new();
    assert_eq!(v.len(), 0 as I32SafeLen);
    v.insert(1, "Hello");
    assert_eq!(v.len(), 1 as I32SafeLen);

    let v = RangeMapBlaze::from_iter([
        (
            -170_141_183_460_469_231_731_687_303_715_884_105_728i128..=10,
            "a",
        ),
        (
            -10..=170_141_183_460_469_231_731_687_303_715_884_105_726,
            "a",
        ),
    ]);
    assert_eq!(
        v.len(),
        340_282_366_920_938_463_463_374_607_431_768_211_455u128
    );
}

#[test]
fn map_test_pops() {
    // Initialize the map with ranges as keys and chars as values
    let mut map = RangeMapBlaze::from_iter([(1..=2, 'a'), (4..=5, 'b'), (10..=11, 'c')]);
    let len = map.len() as I32SafeLen;

    // Adjusted to expect a tuple of (single integer key, value)
    assert_eq!(map.pop_first(), Some((1, 'a')));
    assert_eq!(map.len(), len - 1);
    // After popping the first, the range 1..=2 should now start from 2
    assert_eq!(
        map,
        RangeMapBlaze::from_iter([
            (2..=2, 'a'), // Adjusted range after popping key 1
            (4..=5, 'b'),
            (10..=11, 'c')
        ])
    );

    assert_eq!(map.pop_last(), Some((11, 'c')));
    println!("{map:#?}");
    // After popping the last, the range 10..=11 should now end at 10
    assert_eq!(
        map,
        RangeMapBlaze::from_iter([
            (2..=2, 'a'),
            (4..=5, 'b'),
            (10..=10, 'c') // Adjusted range after popping key 11
        ])
    );
    assert_eq!(map.len(), len - 2);

    // Continue popping and assert changes
    assert_eq!(map.pop_last(), Some((10, 'c'))); // Pop the last remaining element of the previous last range
    assert_eq!(map.len(), len - 3);
    assert_eq!(map, RangeMapBlaze::from_iter([(2..=2, 'a'), (4..=5, 'b')]));

    // Now pop the first element after previous pops, which should be 2 from the adjusted range
    assert_eq!(map.pop_first(), Some((2, 'a')));
    assert_eq!(map.len(), len - 4);
    assert_eq!(map, RangeMapBlaze::from_iter([(4..=5, 'b')]));

    // Finally, pop the last elements left in the map
    assert_eq!(map.pop_first(), Some((4, 'b')));
    assert_eq!(map.pop_last(), Some((5, 'b')));
    assert!(map.is_empty());
}

#[test]
fn map_insert2() {
    let map =
        RangeMapBlaze::from_iter([(1..=2, 'a'), (4..=5, 'a'), (10..=20, 'a'), (30..=30, 'b')]);

    for insert in 0..=31 {
        println!("inserting  {insert}");
        let mut a = map.clone();
        let mut a2: BTreeMap<_, _> = a.iter().map(|(k, v)| (k, *v)).collect();
        let b2 = a2.insert(insert, 'x');
        let b = a.insert(insert, 'x');
        assert_eq!(
            a,
            RangeMapBlaze::from_iter(a2.iter().map(|(k, v)| (*k, *v)))
        );
        assert_eq!(b, b2);
    }
}

#[test]
fn map_remove() {
    // Initialize RangeMapBlaze with char values for simplicity
    let mut map = RangeMapBlaze::from_iter([(1..=2, 'a'), (4..=5, 'b'), (10..=11, 'c')]);
    let len = map.len() as I32SafeLen;

    // Assume remove affects only a single key and returns true if the key was found and removed
    assert_eq!(map.remove(4), Some('b')); // Removing a key within a range may adjust the range
    assert_eq!(map.len(), len - 1);
    // The range 4..=5 with 'b' is adjusted to 5..=5 after removing 4
    assert_eq!(
        map,
        RangeMapBlaze::from_iter([(1..=2, 'a'), (5..=5, 'b'), (10..=11, 'c'),])
    );
    assert_eq!(map.remove(5), Some('b'));

    assert_eq!(map.len(), len - 2 as I32SafeLen);
    assert_eq!(
        map,
        RangeMapBlaze::from_iter([(1..=2, 'a'), (10..=11, 'c'),])
    );

    let mut map = RangeMapBlaze::from_iter([
        (1..=2, 'a'),
        (4..=5, 'b'),
        (10..=100, 'c'),
        (1000..=1000, 'd'),
    ]);
    let len = map.len() as I32SafeLen;
    assert_eq!(map.remove(0), None);
    assert_eq!(map.len(), len);
    assert_eq!(map.remove(3), None);
    assert_eq!(map.len(), len);
    assert_eq!(map.remove(2), Some('a'));
    assert_eq!(map.len(), len - 1 as I32SafeLen);
    assert_eq!(map.remove(1000), Some('d'));
    assert_eq!(map.len(), len - 2 as I32SafeLen);
    assert_eq!(map.remove(10), Some('c'));
    assert_eq!(map.len(), len - 3 as I32SafeLen);
    assert_eq!(map.remove(50), Some('c'));
    assert_eq!(map.len(), len - 4 as I32SafeLen);
    assert_eq!(
        map,
        RangeMapBlaze::from_iter([(1..=1, 'a'), (4..=5, 'b'), (11..=49, 'c'), (51..=100, 'c'),])
    );
}

#[test]
fn map_remove2() {
    // Initialize RangeMapBlaze with char values
    let map =
        RangeMapBlaze::from_iter([(1..=2, 'a'), (4..=5, 'b'), (10..=20, 'c'), (30..=30, 'd')]);

    for remove in 0..=31 {
        println!("removing  {remove}");
        let mut a = map.clone();
        let mut a2: BTreeMap<_, _> = a.iter().map(|(k, v)| (k, *v)).collect();
        // In a map, remove operation returns the value if the key was present
        let b2 = a2.remove(&remove);
        let b = a.remove(remove);
        assert_eq!(
            a,
            RangeMapBlaze::from_iter(a2.iter().map(|(&k, &v)| (k, v)))
        );
        assert_eq!(b, b2);
    }
    // Testing with an empty RangeMapBlaze
    let map: RangeMapBlaze<_, _> = RangeMapBlaze::new();

    for remove in 0..=0 {
        println!("removing  {remove}");
        let mut a: RangeMapBlaze<_, char> = map.clone();
        let mut a2: BTreeMap<_, _> = a.iter().map(|(k, v)| (k, *v)).collect();
        let b2 = a2.remove(&remove);
        let b = a.remove(remove);
        assert_eq!(
            a,
            RangeMapBlaze::from_iter(a2.iter().map(|(&k, &v)| (k, v)))
        );
        assert_eq!(b, b2);
    }
}

#[test]
fn map_split_off() {
    // Initialize RangeMapBlaze with ranges and associated char values
    let map =
        RangeMapBlaze::from_iter([(1..=2, 'a'), (4..=5, 'b'), (10..=20, 'c'), (30..=30, 'd')]);

    for split in 0..=31 {
        println!("splitting at {split}");
        let mut a = map.clone();
        let mut a2: BTreeMap<_, _> = a.iter().map(|(k, v)| (k, *v)).collect();
        // BTreeMap split_off method returns the part of the map with keys greater than or equal to split
        let b2 = a2.split_off(&split);
        // Assuming RangeMapBlaze.split_off behaves similarly and splits by key, not range
        let b = a.split_off(split);

        let a_iter = a2.iter().map(|(&k, &v)| (k, v)).filter(|&(k, _)| k < split);
        let aa = RangeMapBlaze::from_iter(a_iter);
        assert_eq!(a.len(), aa.len());
        assert_eq!(a, aa);

        let b2_iter = b2.iter().map(|(&k, &v)| (k, v));
        let b2b = RangeMapBlaze::from_iter(b2_iter);
        assert_eq!(b, b2b);
    }

    // Testing with an empty RangeMapBlaze
    let map: RangeMapBlaze<_, _> = RangeMapBlaze::new();

    for split in 0..=0 {
        println!("splitting at {split}");
        let mut a: range_set_blaze::RangeMapBlaze<_, char> = map.clone();
        let mut a2: BTreeMap<_, _> = a.iter().map(|(k, v)| (k, *v)).collect();
        let b2 = a2.split_off(&split);
        let b = a.split_off(split);
        assert_eq!(
            a,
            RangeMapBlaze::from_iter(a2.iter().map(|(&k, &v)| (k, v)).filter(|&(k, _)| k < split))
        );
        assert_eq!(
            b,
            RangeMapBlaze::from_iter(b2.iter().map(|(&k, &v)| (k, v)))
        );
    }
}

// // #[test]
// // fn map_retrain() {
// //     let mut set = RangeMapBlaze::from_iter([1..=6]);
// //     // Keep only the even numbers.
// //     set.retain(|k| k % 2 == 0);
// //     assert_eq!(set, RangeMapBlaze::from_iter([2, 4, 6]));
// // }

// // #[test]
// // fn map_sync_and_send() {
// //     fn map_assert_sync_and_send<S: Sync + Send>() {}
// //     assert_sync_and_send::<RangeMapBlaze<(i32,&str)>>();
// //     assert_sync_and_send::<RangesIter<i32>>();
// // }

// // fn map_fraction<T: Integer>(range_int_set: &RangeMapBlaze<T>, range: &RangeInclusive<T>) -> f64 {
// //     T::safe_len_to_f64(range_int_set.len()) / T::safe_len_to_f64(T::safe_len(range))
// // }

// // #[test]
// // fn map_example_2() {
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
// //     for range in intron.range_values() {
// //         let (start, end) = range.into_inner();
// //         println!("{chr}\t{start}\t{end}");
// //     }
// // }

// // #[test]
// // fn map_trick_dyn() {
// //     let bad = [1..=2, 0..=5];
// //     // let u = union_map_dyn!(bad.iter().cloned());
// //     let good = RangeMapBlaze::from_iter(bad);
// //     let _u = union_map_dyn!(good.range_values());
// // }

// // #[test]
// // fn map_multiway2() {
// //     use range_map_blaze::MultiwaySortedDisjoint;

// //     let a = RangeMapBlaze::from_iter([1..=6, 8..=9, 11..=15]);
// //     let b = RangeMapBlaze::from_iter([5..=13, 18..=29]);
// //     let c = RangeMapBlaze::from_iter([25..=100]);

// //     let union = [a.range_values(), b.range_values(), c.range_values()].union();
// //     assert_eq!(union.to_string(), "1..=15, 18..=100");

// //     let union = MultiwaySortedDisjoint::union([a.range_values(), b.range_values(), c.range_values()]);
// //     assert_eq!(union.to_string(), "1..=15, 18..=100");
// // }

// // #[test]
// // fn map_check_sorted_disjoint() {
// //     use range_map_blaze::CheckSortedDisjoint;

// //     let a = CheckSortedDisjoint::new([1..=2, 5..=100].into_iter());
// //     let b = CheckSortedDisjoint::from([2..=6]);
// //     let c = a | b;

// //     assert_eq!(c.to_string(), "1..=100");
// // }

// // #[test]
// // fn map_dyn_sorted_disjoint_example() {
// //     let a = RangeMapBlaze::from_iter([1u8..=6, 8..=9, 11..=15]);
// //     let b = RangeMapBlaze::from_iter([5..=13, 18..=29]);
// //     let c = RangeMapBlaze::from_iter([38..=42]);
// //     let union = [
// //         DynSortedDisjointMap::new(a.range_values()),
// //         DynSortedDisjointMap::new(!b.range_values()),
// //         DynSortedDisjointMap::new(c.range_values()),
// //     ]
// //     .union();
// //     assert_eq!(union.to_string(), "0..=6, 8..=9, 11..=17, 30..=255");
// // }

// // #[test]
// // fn map_not_iter_example() {
// //     let a = CheckSortedDisjoint::from([1u8..=2, 5..=100]);
// //     let b = NotIter::new(a);
// //     assert_eq!(b.to_string(), "0..=0, 3..=4, 101..=255");

// //     // Or, equivalently:
// //     let b = !CheckSortedDisjoint::from([1u8..=2, 5..=100]);
// //     assert_eq!(b.to_string(), "0..=0, 3..=4, 101..=255");
// // }

// // #[test]
// // fn map_len_demo() {
// //     let len: <u8 as Integer>::SafeLen = RangeMapBlaze::from_iter([0u8..=255]).len();
// //     assert_eq!(len, 256);

// //     assert_eq!(<u8 as Integer>::safe_len(&(0..=255)), 256);
// // }

// // #[test]
// // fn map_union_iter() {
// //     use range_map_blaze::{CheckSortedDisjoint, UnionIterMap};

// //     let a = CheckSortedDisjoint::new([1..=2, 5..=100].into_iter());
// //     let b = CheckSortedDisjoint::from([2..=6]);
// //     let c = UnionIterMap::new(AssumeSortedStarts::new(
// //         a.merge_by(b, |a_range, b_range| a_range.start() <= b_range.start()),
// //     ));
// //     assert_eq!(c.to_string(), "1..=100");

// //     // Or, equivalently:
// //     let a = CheckSortedDisjoint::new([1..=2, 5..=100].into_iter());
// //     let b = CheckSortedDisjoint::from([2..=6]);
// //     let c = SortedDisjoint::union(a, b);
// //     assert_eq!(c.to_string(), "1..=100")
// // }

// // #[test]
// // fn map_bitor() {
// //     let a = CheckSortedDisjoint::from([1..=1]);
// //     let b = RangeMapBlaze::from_iter([2..=2]).into_ranges();
// //     let union = core::ops::BitOr::bitor(a, b);
// //     assert_eq!(union.to_string(), "1..=2");

// //     let a = CheckSortedDisjoint::from([1..=1]);
// //     let b = CheckSortedDisjoint::from([2..=2]);
// //     let c = range_map_blaze::SortedDisjoint::union(a, b);
// //     assert_eq!(c.to_string(), "1..=2");

// //     let a = CheckSortedDisjoint::from([1..=1]);
// //     let b = CheckSortedDisjoint::from([2..=2]);
// //     let c = core::ops::BitOr::bitor(a, b);
// //     assert_eq!(c.to_string(), "1..=2");

// //     let a = CheckSortedDisjoint::from([1..=1]);
// //     let b = RangeMapBlaze::from_iter([2..=2]).into_ranges();
// //     let c = range_map_blaze::SortedDisjoint::union(a, b);
// //     assert_eq!(c.to_string(), "1..=2");
// // }

// // #[test]
// // fn map_range_set_int_constructors() {
// //     // Create an empty set with 'new' or 'default'.
// //     let a0 = RangeMapBlaze::<i32>::new();
// //     let a1 = RangeMapBlaze::<i32>::default();
// //     assert!(a0 == a1 && a0.is_empty());

// //     // 'from_iter'/'collect': From an iterator of integers.
// //     // Duplicates and out-of-order elements are fine.
// //     let a0 = RangeMapBlaze::from_iter([3, 2, 1, 100, 1]);
// //     let a1: RangeMapBlaze<(i32,&str)> = [3, 2, 1, 100, 1].into_iter().collect();
// //     assert!(a0 == a1 && a0.to_string() == "1..=3, 100..=100");

// //     // 'from_iter'/'collect': From an iterator of inclusive ranges, start..=end.
// //     // Overlapping, out-of-order, and empty ranges are fine.
// //     #[allow(clippy::reversed_empty_ranges)]
// //     let a0 = RangeMapBlaze::from_iter([1..=2, 2..=2, -10..=-5, 1..=0]);
// //     #[allow(clippy::reversed_empty_ranges)]
// //     let a1: RangeMapBlaze<(i32,&str)> = [1..=2, 2..=2, -10..=-5, 1..=0].into_iter().collect();
// //     assert!(a0 == a1 && a0.to_string() == "-10..=-5, 1..=2");

// //     // If we know the ranges are sorted and disjoint, we can use 'from'/'into'.
// //     let a0 = RangeMapBlaze::from_sorted_disjoint_map(CheckSortedDisjoint::from([-10..=-5, 1..=2]));
// //     let a1: RangeMapBlaze<(i32,&str)> =
// //         CheckSortedDisjoint::from([-10..=-5, 1..=2]).into_range_map_blaze();
// //     assert!(a0 == a1 && a0.to_string() == "-10..=-5, 1..=2");

// //     // For compatibility with `BTreeSet`, we also support
// //     // 'from'/'into' from arrays of integers.
// //     let a0 = RangeMapBlaze::from([3, 2, 1, 100, 1]);
// //     let a1: RangeMapBlaze<(i32,&str)> = [3, 2, 1, 100, 1].into();
// //     assert!(a0 == a1 && a0.to_string() == "1..=3, 100..=100");
// // }

// // #[cfg(feature = "from_slice")]
// // fn map_print_features() {
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
// // fn map_from_slice_all_types() {
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
// // fn map_range_set_int_slice_constructor() {
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
// //     let a1: RangeMapBlaze<(i32,&str)> = [3, 2, 1, 100, 1].into();
// //     assert!(a0 == a1 && a0.to_string() == "1..=3, 100..=100");

// //     #[allow(clippy::needless_borrows_for_generic_args)]
// //     let a2 = RangeMapBlaze::from_slice(&[3, 2, 1, 100, 1]);
// //     assert!(a0 == a2 && a2.to_string() == "1..=3, 100..=100");

// //     let a2 = RangeMapBlaze::from_slice([3, 2, 1, 100, 1]);
// //     assert!(a0 == a2 && a2.to_string() == "1..=3, 100..=100");
// // }

// cmk add symmetric_difference tests
#[test]
fn map_range_map_blaze_operators() {
    let a = RangeMapBlaze::from_iter([(1..=2, "one"), (5..=100, "two")]);
    let b = RangeMapBlaze::from_iter([(2..=6, "three")]);

    // Union of two 'RangeMapBlaze's. Early values in from_iter and union have higher priority.
    let result = &a | &b;
    println!("{result:#?}");
    // Alternatively, we can take ownership via 'a | b'.
    assert_eq!(
        result.to_string(),
        r#"(1..=2, "one"), (3..=4, "three"), (5..=100, "two")"#
    );

    // Intersection of two 'RangeMapBlaze's. Later values in intersection have higher priority so `a` acts as a filter or mask.
    let result = &a & &b; // Alternatively, 'a & b'.
    assert_eq!(result.to_string(), r#"(2..=2, "one"), (5..=6, "two")"#);

    // Set difference of two 'RangeMapBlaze's.
    let result = &a - &b; // Alternatively, 'a - b'.
    assert_eq!(result.to_string(), r#"(1..=1, "one"), (7..=100, "two")"#);

    // Symmetric difference of two 'RangeMapBlaze's.
    let result = &a ^ &b; // Alternatively, 'a ^ b'.
    assert_eq!(
        result.to_string(),
        r#"(1..=1, "one"), (3..=4, "three"), (7..=100, "two")"#
    );

    // complement of a 'RangeMapBlaze'.
    let result = !(&a.range_values().into_range_map_blaze());
    assert_eq!(
        result.to_string(),
        "-2147483648..=0, 3..=4, 101..=2147483647"
    );

    // Multiway union of 'RangeMapBlaze's.
    let c = RangeMapBlaze::from_iter([(2..=2, "six"), (6..=200, "seven")]);
    let result = [&a, &b, &c].union();
    assert_eq!(
        result.to_string(),
        r#"(1..=2, "one"), (3..=4, "three"), (5..=100, "two"), (101..=200, "seven")"#
    );

    // // Multiway intersection of 'RangeMapBlaze's.
    let c = RangeMapBlaze::from_iter([(2..=2, "six"), (6..=200, "seven")]);
    let result = [&a, &b, &c].intersection();
    assert_eq!(result.to_string(), r#"(2..=2, "one"), (6..=6, "two")"#);

    // // Combining multiple operations
    // let result0 = &a - (&b | &c); // Creates a temporary 'RangeMapBlaze'.

    // // Alternatively, we can use the 'SortedDisjoint' API and avoid the temporary 'RangeMapBlaze'.
    // let result1 = RangeMapBlaze::from_sorted_disjoint_map(a.range_values() - (b.range_values() | c.range_values()));
    // assert!(result0 == result1 && result0.to_string() == "1..=1");
}

// // #[test]
// // fn map_sorted_disjoint_constructors() {
// //     // RangeMapBlaze's .range_values(), .0().clone() and .into_ranges()
// //     let r = RangeMapBlaze::from_iter([3, 2, 1, 100, 1]);
// //     let a = r.range_values();
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
// //     let b = DynSortedDisjointMap::new(a);
// //     assert!(b.to_string() == "1..=3, 100..=100");
// // }

// // #[test]
// // fn map_iterator_example() {
// //     struct OrdinalWeekends2023 {
// //         next_range: RangeInclusive<i32>,
// //     }
// //     impl SortedStarts<i32> for OrdinalWeekends2023 {}
// //     impl SortedDisjoint<i32> for OrdinalWeekends2023 {}

// //     impl OrdinalWeekends2023 {
// //         fn map_new() -> Self {
// //             Self { next_range: 0..=1 }
// //         }
// //     }
// //     impl Iterator for OrdinalWeekends2023 {
// //         type Item = RangeInclusive<i32>;
// //         fn map_next(&mut self) -> Option<Self::Item> {
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
// // fn map_sorted_disjoint_operators() {
// //     let a0 = RangeMapBlaze::from_iter([1..=2, 5..=100]);
// //     let b0 = RangeMapBlaze::from_iter([2..=6]);
// //     let c0 = RangeMapBlaze::from_iter([2..=2, 6..=200]);

// //     // 'union' method and 'to_string' method
// //     let (a, b) = (a0.range_values(), b0.range_values());
// //     let result = a.union(b);
// //     assert_eq!(result.to_string(), "1..=100");

// //     // '|' operator and 'equal' method
// //     let (a, b) = (a0.range_values(), b0.range_values());
// //     let result = a | b;
// //     assert!(result.equal(CheckSortedDisjoint::from([1..=100])));

// //     // multiway union of same type
// //     let (a, b, c) = (a0.range_values(), b0.range_values(), c0.range_values());
// //     let result = [a, b, c].union();
// //     assert_eq!(result.to_string(), "1..=200");

// //     // multiway union of different types
// //     let (a, b, c) = (a0.range_values(), b0.range_values(), c0.range_values());
// //     let result = union_map_dyn!(a, b, !c);
// //     assert_eq!(result.to_string(), "-2147483648..=100, 201..=2147483647");
// // }

// // #[test]
// // fn map_range_example() {
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
// // fn map_range_test() {
// //     use core::ops::Bound::Included;
// //     use range_map_blaze::RangeMapBlaze;

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
// // fn map_is_subset_check() {
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
// // fn map_cmp_range_set_int() {
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
// // fn map_run_rangemap_crate() {
// //     let mut rng = StdRng::seed_from_u64(0);
// //     let range_len = 1_000_000;

// //     let vec_range: Vec<_> =
// //         MemorylessRange::new(&mut rng, range_len, 0..=99_999_999, 0.01, 1, How::None).collect();

// //     let _start = Instant::now();

// //     let rangemap_set0 = &rangemap::RangeInclusiveSet::from_iter(vec_range.iter().cloned());
// //     let _rangemap_set1 = &rangemap::RangeInclusiveSet::from_iter(rangemap_set0.iter().cloned());
// // }

// // #[test]
// // fn map_from_iter_coverage() {
// //     let vec_range = vec![1..=2, 2..=2, -10..=-5];
// //     let a0 = RangeMapBlaze::from_iter(vec_range.iter());
// //     let a1: RangeMapBlaze<(i32,&str)> = vec_range.iter().collect();
// //     assert!(a0 == a1 && a0.to_string() == "-10..=-5, 1..=2");
// // }

// // // fn map__some_fn() {
// // //     let guaranteed = RangeMapBlaze::from_iter([1..=2, 3..=4, 5..=6]).into_ranges();
// // //     let _range_map_blaze = RangeMapBlaze::from_sorted_disjoint_map(guaranteed);
// // //     let not_guaranteed = [1..=2, 3..=4, 5..=6].into_iter();
// // //     let _range_map_blaze = RangeMapBlaze::from_sorted_disjoint_map(not_guaranteed);
// // // }

// // // fn map__some_fn() {
// // //     let _integer_set = RangeMapBlaze::from_iter([1, 2, 3, 5]);
// // //     let _char_set = RangeMapBlaze::from_iter(['a', 'b', 'c', 'd']);
// // // }

// // #[test]
// // fn map_print_first_complement_gap() {
// //     let a = CheckSortedDisjoint::from([-10i16..=0, 1000..=2000]);
// //     println!("{:?}", (!a).next().unwrap()); // prints -32768..=-11
// // }

// // #[test]
// // fn map_multiway_failure_example() {
// //     use range_map_blaze::prelude::*;

// //     let a = RangeMapBlaze::from_iter([1..=6, 8..=9, 11..=15]);
// //     let b = RangeMapBlaze::from_iter([5..=13, 18..=29]);
// //     let c = RangeMapBlaze::from_iter([38..=42]);

// //     let _i0 = [a.range_values(), b.range_values(), c.range_values()].intersection();
// //     // let _i1 = [!a.range_values(), b.range_values(), c.range_values()].intersection();
// //     let _i2 = [
// //         DynSortedDisjointMap::new(!a.range_values()),
// //         DynSortedDisjointMap::new(b.range_values()),
// //         DynSortedDisjointMap::new(c.range_values()),
// //     ]
// //     .intersection();
// //     let _i3 = intersection_map_dyn!(!a.range_values(), b.range_values(), c.range_values());
// // }

// // #[test]
// // fn map_complement_sample() {
// //     let c = !RangeMapBlaze::from([0, 3, 4, 5, 10]);
// //     println!("{},{},{}", c.len(), c.ranges_len(), c);
// // }

// // #[cfg(feature = "rog-experimental")]
// // #[test]
// // fn map_test_rog_functionality() {
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
// // fn map_test_rogs_get_functionality() {
// //     let a = RangeMapBlaze::from_iter([1..=6, 8..=9, 11..=15]);
// //     for value in 0..=16 {
// //         println!("{:?}", a.rogs_get_slow(value));
// //         assert_eq!(a.rogs_get_slow(value), a.rogs_get(value));
// //     }
// // }

// // #[cfg(feature = "rog-experimental")]
// // #[test]
// // fn map_test_rog_repro1() {
// //     let a = RangeMapBlaze::from_iter([1u8..=6u8]);
// //     assert_eq!(
// //         a._rogs_range_slow(1..=7),
// //         a.rogs_range(1..=7).collect::<Vec<_>>()
// //     );
// // }

// // #[cfg(feature = "rog-experimental")]
// // #[test]
// // fn map_test_rog_repro2() {
// //     let a = RangeMapBlaze::from_iter([1..=6, 8..=9, 11..=15]);
// //     assert_eq!(
// //         a._rogs_range_slow(4..=8),
// //         a.rogs_range(4..=8).collect::<Vec<_>>()
// //     );
// // }

// // #[cfg(feature = "rog-experimental")]
// // #[test]
// // fn map_test_rog_coverage1() {
// //     let a = RangeMapBlaze::from_iter([1u8..=6u8]);
// //     assert!(panic::catch_unwind(AssertUnwindSafe(
// //         || a.rogs_range((Bound::Excluded(&255), Bound::Included(&255)))
// //     ))
// //     .is_err());
// //     assert!(panic::catch_unwind(AssertUnwindSafe(|| a.rogs_range(0..0))).is_err());
// // }

// // #[cfg(feature = "rog-experimental")]
// // #[test]
// // fn map_test_rog_extremes_u8() {
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
// // fn map_test_rog_get_extremes_u8() {
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
// // fn map_test_rog_extremes_i128() {
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
// // fn map_test_rog_extremes_get_i128() {
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
// // fn map_test_rog_should_fail_i128() {
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
// // fn map_test_rog_get_should_fail_i128() {
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
// // fn map_test_rog_get_doc() {
// //     use crate::RangeMapBlaze;
// //     let range_map_blaze = RangeMapBlaze::from([1, 2, 3]);
// //     assert_eq!(range_map_blaze.rogs_get(2), Rog::Range(1..=3));
// //     assert_eq!(range_map_blaze.rogs_get(4), Rog::Gap(4..=2_147_483_647));
// // }

// // #[cfg(feature = "rog-experimental")]
// // #[test]
// // fn map_test_rog_range_doc() {
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

pub fn play_movie(frames: RangeMapBlaze<i32, String>, fps: i32, skip_sleep: bool) {
    assert!(fps > 0, "fps must be positive");
    // cmk could look for missing frames
    let sleep_duration = Duration::from_secs(1) / fps as u32;
    // For every frame index (index) from 0 to the largest index in the frames ...
    for index in 0..=frames.ranges().into_range_set_blaze().last().unwrap() {
        // Look up the frame at that index (panic if none exists)
        let frame = frames.get(index).unwrap_or_else(|| {
            panic!("frame {} not found", index);
        });
        // Clear the line and return the cursor to the beginning of the line
        print!("\x1B[2K\r{}", frame);
        stdout().flush().unwrap(); // Flush stdout to ensure the output is displayed
        if !skip_sleep {
            sleep(sleep_duration);
        }
    }
}

// cmk try to make generic?
// cmk linear could be a method on RangeMapBlaze
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
            let (start, end) = range_value.0.clone().into_inner();
            let mut a = (start - first) * scale.abs() + first;
            let mut b = (end + 1 - first) * scale.abs() + first - 1;
            let last = (last + 1 - first) * scale.abs() + first - 1;
            if scale < 0 {
                (a, b) = (last - b + first, last - a + first);
            }
            let new_range = a + shift..=b + shift;
            (new_range, range_value.1.clone())
        })
        .collect()
}
// cmk make range_values a DoubleEndedIterator

#[test]
fn map_string_animation() {
    let fps: i32 = 24;
    let length_seconds = 15;
    let frame_count = fps * length_seconds;

    // The `main`` track starts with 15 seconds of black
    let mut main = RangeMapBlaze::from_iter([(0..=frame_count - 1, "<black>".to_string())]);
    println!("main {main:?}");

    // Create a 10 frame `digits` track with "0" to "9"".
    let mut digits = RangeMapBlaze::from_iter((0..=9).map(|i| (i..=i, i.to_string())));

    // Make frame 0 be "start"
    digits.insert(0, "start".to_string());

    // Oops, we've changed our mind and now don't want frames 8 and 9.
    digits = digits - RangeSetBlaze::from_iter([8..=9]);

    // Apply the following linear transformation to `digits``:
    // 1. Make each original frame last one second
    // 2. Reverse the order of the frames
    // 3. Shift the frames 1 second into the future
    digits = linear(&digits, -fps, fps);
    println!("digits m {digits:?}");

    // Composite these together (listed from top to bottom)
    //  1. `digits``
    //  2. `digits` shifted 10 seconds into the future
    //  3. `main`
    main = &digits | &linear(&digits, 1, 10 * fps) | &main;
    println!("main dd {main:?}");

    play_movie(main, fps, true);
}

// cmk move this to tests
#[test]
fn understand_strings_as_values() {
    // RangeMapBlaze string-like values can be &str, String, &String (or even &&str and &&String).
    let _: RangeMapBlaze<i32, &str> = RangeMapBlaze::from_iter([(0..=0, "a")]);
    let _: RangeMapBlaze<i32, &str> = RangeMapBlaze::from_iter([(0..=0, &"a")]);
    let _: RangeMapBlaze<i32, &&str> = RangeMapBlaze::from_iter([(0..=0, &"a")]);
    let _: RangeMapBlaze<i32, String> = RangeMapBlaze::from_iter([(0..=0, "a".to_string())]);
    let a_string = "a".to_string();
    let _: RangeMapBlaze<i32, String> = RangeMapBlaze::from_iter([(0..=0, &a_string)]);
    let _: RangeMapBlaze<i32, &String> = RangeMapBlaze::from_iter([(0..=0, &a_string)]);
    let _: RangeMapBlaze<i32, &&String> = RangeMapBlaze::from_iter([(0..=0, &&a_string)]);
    let _: RangeMapBlaze<i32, String> = RangeMapBlaze::from_iter([(0..=0, a_string)]);

    // With .range_values() we iterate over references to the values.
    // With .into_range_values() we iterate over the values themselves.
    // cmk00 Is there also a .into_values() and .into_ranges() and .into_keys(), etc?
    let a: RangeMapBlaze<i32, &str> = RangeMapBlaze::from_iter([(0..=0, "a")]);
    let mut b: RangeValuesIter<i32, &str> = a.range_values();
    let _c: &&str = b.next().unwrap().1;
    let mut b: IntoRangeValuesIter<i32, &str> = a.into_range_values();
    let _c: &str = b.next().unwrap().1;

    let a: RangeMapBlaze<i32, String> = RangeMapBlaze::from_iter([(0..=0, "a".to_string())]);
    let mut b: RangeValuesIter<i32, String> = a.range_values();
    let _c: &String = b.next().unwrap().1;
    let mut b: IntoRangeValuesIter<i32, String> = a.into_range_values();
    let _c: String = b.next().unwrap().1;

    // You can get all the same types via CheckSortedDisjointMap, but values are always (clonable) references.
    let a_string = "a".to_string();
    let _: CheckSortedDisjointMap<i32, &str, &&str, _> = [(0..=0, &"a")].into();
    let mut b: CheckSortedDisjointMap<i32, String, &String, _> =
        CheckSortedDisjointMap::from([(0..=0, &a_string)]);
    let c: &String = b.next().unwrap().1;
    let _c_clone: String = c.clone();
    let _: CheckSortedDisjointMap<i32, &String, &&String, _> =
        CheckSortedDisjointMap::new([(0..=0, &&a_string)]);
    let _: CheckSortedDisjointMap<i32, String, &String, _> =
        CheckSortedDisjointMap::new([(0..=0, &"a".to_string())]);
}

// cmk00 test this really is every method
#[test]
fn test_every_sorted_disjoint_map_method() {
    use syntactic_for::syntactic_for;

    let e0: RangeMapBlaze<i32, &str> = RangeMapBlaze::from_iter([(1..=2, "a"), (5..=100, "a")]);

    macro_rules! fresh_instances {
        () => {{
            let a: CheckSortedDisjointMap<i32, &str, _, _> =
                CheckSortedDisjointMap::new([(1..=2, &"a"), (5..=100, &"a")]);
            let b: UnionIterMap<i32, &str, _, _> = [CheckSortedDisjointMap::new([
                (1..=2, &"a"),
                (5..=100, &"a"),
            ])]
            .union();
            let c: IntersectionIterMap<i32, &str, _, _, _> = [CheckSortedDisjointMap::new([
                (1..=2, &"a"),
                (5..=100, &"a"),
            ])]
            .intersection();
            let d: SymDiffIterMap<i32, &str, _, _> = [CheckSortedDisjointMap::new([
                (1..=2, &"a"),
                (5..=100, &"a"),
            ])]
            .symmetric_difference();
            let e: RangeValuesIter<i32, &str> = e0.range_values();
            let f: DynSortedDisjointMap<i32, &str, _> =
                DynSortedDisjointMap::new(CheckSortedDisjointMap::new([
                    (1..=2, &"a"),
                    (5..=100, &"a"),
                ]));

            (a, b, c, d, e, f)
        }};
    }

    // check for SortedDisjointMap and FuseIterator traits
    let (a, b, c, d, e, f) = fresh_instances!();
    syntactic_for! { sd in [a, b, c, d, e, f] {$(
        is_sorted_disjoint_map::<_,_,_,_>($sd);
    )*}}
    fn is_fused<T: FusedIterator>(_: T) {}
    let (a, b, c, d, e, f) = fresh_instances!();
    syntactic_for! { sd in [a, b, c, d, e, f] {$(
        is_fused::<_>($sd);
    )*}}
    fn is_sorted_disjoint_map<T, V, VR, S>(_iter: S)
    where
        T: Integer,
        V: ValueOwned,
        VR: CloneBorrow<V>,
        S: SortedDisjointMap<T, V, VR>,
    {
    }

    // Complement
    let (a, b, c, d, e, f) = fresh_instances!();
    syntactic_for! { sd in [a,b,c,d,e,f] {$(
        let z = ! $sd;
        assert!(z.equal(CheckSortedDisjoint::from([-2147483648..=0, 3..=4, 101..=2147483647])));
    )*}}

    // Union
    let (a, b, c, d, e, f) = fresh_instances!();
    syntactic_for! { sd in [a, b, c, d, e, f] {$(
        let z: CheckSortedDisjointMap<i32, &str, _, _> = CheckSortedDisjointMap::new([(-1..=0,&"z"), (50..=50, &"z"),(1000..=10_000,&"z")]);
        let z = $sd | z;
        assert!(z.equal(CheckSortedDisjointMap::new([(-1..=0, &"z"), (1..=2, &"a"), (5..=100, &"a"), (1000..=10000, &"z")])));
    )*}}

    // Intersection
    let (a, b, c, d, e, f) = fresh_instances!();
    syntactic_for! { sd in [a, b, c, d, e, f] {$(
        let z: CheckSortedDisjointMap<i32, &str, _, _> = CheckSortedDisjointMap::new([(-1..=0,&"z"), (50..=50, &"z"),(1000..=10_000,&"z")]);
        let z = $sd & z;
        // println!("{}", z.to_string());
        assert!(z.equal(CheckSortedDisjointMap::new([(50..=50, &"a")])));
    )*}}

    // Symmetric Difference
    let (a, b, c, d, e, f) = fresh_instances!();
    syntactic_for! { sd in [a, b, c, d, e,f] {$(
        let z: CheckSortedDisjointMap<i32, &str, _, _> = CheckSortedDisjointMap::new([(-1..=0,&"z"), (50..=50, &"z"),(1000..=10_000,&"z")]);
        let z = $sd ^ z;
        // println!("a {}", z.to_string());
        assert!(z.equal(CheckSortedDisjointMap::new([(-1..=0, &"z"), (1..=2, &"a"), (5..=49, &"a"), (51..=100, &"a"), (1000..=10000, &"z")])));
    )*}}

    let (a, b, c, d, e, f) = fresh_instances!();
    syntactic_for! { sd in [a, b, c, d, e,f] {$(
        let z: CheckSortedDisjointMap<i32, &str, _, _> = CheckSortedDisjointMap::new([(-1..=0,&"z"), (50..=50, &"z"),(1000..=10_000,&"z")]);
        let z = $sd - z;
        // println!("c {}", z.to_string());
        assert!(z.equal(CheckSortedDisjointMap::new([(1..=2, &"a"), (5..=49, &"a"), (51..=100, &"a")])));
    )*}}
}
// cmk0 implement get, values, values_mut, range,

// cmk what if they forget to import this the thing that lets | work?

// cmk must test coverage

#[test]
fn map_random_from_iter_item() {
    // cmk all these tests should test on size zero, too.
    for seed in 0..20 {
        println!("seed: {seed}");
        let mut rng = StdRng::seed_from_u64(seed);

        let mut btree_map = BTreeMap::new();

        let mut inputs = Vec::new();
        for _ in 0..500 {
            let key = rng.gen_range(0..=255u32); // cmk change back to u8s
            let value = ['a', 'b', 'c'].choose(&mut rng).unwrap(); // cmk allow more than references

            print!("{key}{value} ");
            inputs.push((key, value));

            // cmk fix so don't need to clone and can use .iter()
            let range_map_blaze = RangeMapBlaze::<_, char>::from_iter(inputs.clone());
            // Only insert if the key does not already exist
            btree_map.entry(key).or_insert(value);
            if !equal_maps(&range_map_blaze, &btree_map) {
                println!();
                let _range_map_blaze = RangeMapBlaze::<_, char>::from_iter(inputs.clone());
                panic!();
            }
        }
    }
}

#[test]
fn map_random_from_iter_range() {
    for seed in 0..20 {
        println!("seed: {seed}");
        let mut rng = StdRng::seed_from_u64(seed);

        let mut btree_map = BTreeMap::new();

        let mut inputs = Vec::new();
        for _ in 0..500 {
            let start = rng.gen_range(0..=255u8);
            let end = rng.gen_range(start..=255u8);
            let key = start..=end;
            let value = ['a', 'b', 'c'].choose(&mut rng).unwrap(); // cmk allow more than references

            // print!("{key}{value} ");
            inputs.push((key.clone(), value));

            // cmk fix so don't need to clone and can use .iter()
            let range_map_blaze = RangeMapBlaze::<u8, char>::from_iter(inputs.clone());
            for k in key.clone() {
                btree_map.entry(k).or_insert(value);
            }
            if !equal_maps(&range_map_blaze, &btree_map) {
                let _range_map_blaze = RangeMapBlaze::<u8, char>::from_iter(inputs.clone());
                panic!();
            }
        }
    }
}

#[test]
fn map_random_insert() {
    for seed in 0..20 {
        println!("seed: {seed}");
        let mut rng = StdRng::seed_from_u64(seed);

        let mut btree_map = BTreeMap::new();
        let mut range_map_blaze = RangeMapBlaze::new();
        let mut inputs = Vec::new();

        for _ in 0..500 {
            let key = rng.gen_range(0..=255u8);
            let value = ["aaa", "bbb", "ccc"].choose(&mut rng).unwrap();
            // print!("{key}{value} ");

            btree_map.insert(key, value);
            range_map_blaze.insert(key, *value);
            if equal_maps(&range_map_blaze, &btree_map) {
                inputs.push((key, value));
                continue;
            }

            // if range_map_blaze and btree_map are not equal, then we have a bug, so repro it:

            let mut range_map_blaze = RangeMapBlaze::from_iter(inputs.clone().into_iter());
            range_map_blaze.insert(key, *value);
            assert!(equal_maps(&range_map_blaze, &btree_map));
        }
    }
}

#[test]
fn map_random_insert_range() {
    for seed in 0..20 {
        println!("seed: {seed}");
        let mut rng = StdRng::seed_from_u64(seed);

        let mut btree_map = BTreeMap::new();
        let mut range_map_blaze = RangeMapBlaze::new();
        let mut inputs = Vec::new();

        for _ in 0..500 {
            let start = rng.gen_range(0..=255u8);
            let end = rng.gen_range(start..=255u8);
            let key = start..=end;
            let value = ["aaa", "bbb", "ccc"].choose(&mut rng).unwrap();
            // print!("{key}{value} ");

            for k in key.clone() {
                btree_map.insert(k, value);
            }
            range_map_blaze.ranges_insert(key.clone(), *value);
            if equal_maps(&range_map_blaze, &btree_map) {
                inputs.push((key.clone(), value));
                continue;
            }

            // if range_map_blaze and btree_map are not equal, then we have a bug, so repro it:

            let mut range_map_blaze = RangeMapBlaze::from_iter(inputs.clone().into_iter());
            println!("{range_map_blaze}");
            println!("About to insert {}..={} -> {value}", key.start(), key.end());
            range_map_blaze.ranges_insert(key.clone(), *value);
            assert!(equal_maps(&range_map_blaze, &btree_map));
        }
    }
}

#[test]
fn map_random_ranges() {
    let values = ['a', 'b', 'c'];
    for seed in 0..20 {
        println!("seed: {seed}");
        let mut rng = StdRng::seed_from_u64(seed);

        let mut range_set_blaze = RangeSetBlaze::new();
        let mut range_map_blaze = RangeMapBlaze::new();
        let mut inputs = Vec::<(u8, &char)>::new();

        for _ in 0..500 {
            let key = rng.gen_range(0..=255u8);
            let value = values.choose(&mut rng).unwrap();
            // print!("{key}{value} ");

            range_set_blaze.insert(key);
            range_map_blaze.insert(key, *value);
            if range_set_blaze.ranges().eq(range_map_blaze.ranges()) {
                inputs.push((key, value));
                continue;
            }

            // if range_map_blaze and btree_map are not equal, then we have a bug, so repro it:

            let mut range_map_blaze = RangeMapBlaze::from_iter(inputs.clone().into_iter());
            range_map_blaze.insert(key, *value);
            assert!(range_set_blaze.ranges().eq(range_map_blaze.ranges()));
        }
    }
}

#[test]
fn map_random_ranges_ranges() {
    let values = ['a', 'b', 'c'];
    for seed in 0..20 {
        println!("seed: {seed}");
        let mut rng = StdRng::seed_from_u64(seed);

        let mut range_set_blaze = RangeSetBlaze::new();
        let mut range_map_blaze = RangeMapBlaze::new();
        let mut inputs = Vec::new();

        for _ in 0..500 {
            let start = rng.gen_range(0..=255u8);
            let end = rng.gen_range(start..=255u8);
            let key = start..=end;
            let value = values.choose(&mut rng).unwrap();
            // print!("{key}{value} ");

            range_set_blaze.ranges_insert(key.clone());
            range_map_blaze.ranges_insert(key.clone(), *value);
            if range_set_blaze.ranges().eq(range_map_blaze.ranges()) {
                inputs.push((key.clone(), value));
                continue;
            }

            // if range_map_blaze and btree_map are not equal, then we have a bug, so repro it:

            let mut range_map_blaze = RangeMapBlaze::from_iter(inputs.clone().into_iter());
            range_map_blaze.ranges_insert(key.clone(), *value);
            assert!(range_set_blaze.ranges().eq(range_map_blaze.ranges()));
        }
    }
}

#[test]
fn map_random_intersection() {
    let values = ['a', 'b', 'c'];
    for seed in 0..20 {
        println!("seed: {seed}");
        let mut rng = StdRng::seed_from_u64(seed);

        let mut set0 = RangeSetBlaze::new();
        let mut map0 = RangeMapBlaze::new();
        // let mut inputs = Vec::<(u8, &char)>::new();

        for _ in 0..500 {
            let element = rng.gen_range(0..=255u8);
            let key = rng.gen_range(0..=255u8);
            let value = values.choose(&mut rng).unwrap();
            // print!("{element},{key}{value} ");

            set0.insert(element);
            map0.insert(key, *value);

            let intersection = IntersectionIterMap::new(map0.range_values(), set0.ranges());

            let mut expected_keys = map0
                .ranges()
                .intersection(set0.ranges())
                .collect::<RangeSetBlaze<_>>();
            if !expected_keys.is_empty() {
                // println!("expected_keys: {expected_keys}");
            }
            for range_value in intersection {
                let (range, value) = range_value;
                // println!();
                // print!("removing ");
                for k in range {
                    assert_eq!(map0.get(k), Some(value));
                    assert!(set0.contains(k));
                    // print!("{k} ");
                    assert!(expected_keys.remove(k));
                }
                // println!();
            }
            if !expected_keys.is_empty() {
                // eprintln!("{set0}");
                // eprintln!("{map0}");
                panic!("expected_keys should be empty: {expected_keys}");
            }
        }
    }
}

#[test]
fn map_tiny_symmetric_difference0() {
    use range_set_blaze::IntoString;

    let mut map0 = RangeMapBlaze::new();
    map0.insert(84, 'c');
    map0.insert(85, 'c');
    let mut map1 = RangeMapBlaze::new();
    map1.insert(85, 'a');
    let symmetric_difference = SymDiffIterMap::new2(map0.range_values(), map1.range_values());
    assert_eq!(symmetric_difference.into_string(), "(84..=84, 'c')");
}

#[test]
fn map_tiny_symmetric_difference1() {
    use range_set_blaze::IntoString;

    let mut map0 = RangeMapBlaze::new();
    map0.insert(187, 'a');
    map0.insert(188, 'a');
    map0.insert(189, 'a');
    let mut map1 = RangeMapBlaze::new();
    map1.insert(187, 'b');
    map1.insert(189, 'c');
    let symmetric_difference = SymDiffIterMap::new2(map0.range_values(), map1.range_values());
    assert_eq!(symmetric_difference.into_string(), "(188..=188, 'a')");
}

#[test]
fn map_random_symmetric_difference() {
    let values = ['a', 'b', 'c'];
    for seed in 0..20 {
        println!("seed: {seed}");
        let mut rng = StdRng::seed_from_u64(seed);

        let mut map0 = RangeMapBlaze::new();
        let mut map1 = RangeMapBlaze::new();
        // let mut inputs = Vec::<(u8, &char)>::new();

        for _ in 0..500 {
            let key = rng.gen_range(0..=255u8);
            let value = values.choose(&mut rng).unwrap();
            map0.insert(key, *value);
            print!("l{key}{value} ");
            let key = rng.gen_range(0..=255u8);
            let value = values.choose(&mut rng).unwrap();
            map1.insert(key, *value);
            print!("r{key}{value} ");

            let symmetric_difference =
                SymDiffIterMap::new2(map0.range_values(), map1.range_values());

            // println!(
            //     "left ^ right = {}",
            //     SymDiffIterMap::new2(map0.range_values(), map1.range_values()).to_string()
            // );

            let mut expected_keys = map0
                .ranges()
                .symmetric_difference(map1.ranges())
                .collect::<RangeSetBlaze<_>>();
            for range_value in symmetric_difference {
                let (range, value) = range_value;
                // println!();
                // print!("removing ");
                for k in range {
                    let get0 = map0.get(k);
                    let get1 = map1.get(k);
                    match (get0, get1) {
                        (Some(_v0), Some(_v1)) => {
                            println!();
                            println!("left: {}", map0);
                            println!("right: {}", map1);
                            let s_d =
                                SymDiffIterMap::new2(map0.range_values(), map1.range_values())
                                    .into_range_map_blaze();
                            panic!("left ^ right = {s_d}");
                        }
                        (Some(v0), None) => {
                            assert_eq!(v0, value);
                        }
                        (None, Some(v1)) => {
                            assert_eq!(v1, value);
                        }
                        (None, None) => {
                            panic!("should not happen 1");
                        }
                    }
                    assert!(expected_keys.remove(k));
                }
                // println!();
            }
            if !expected_keys.is_empty() {
                println!();
                println!("left: {}", map0);
                println!("right: {}", map1);
                let s_d = SymDiffIterMap::new2(map0.range_values(), map1.range_values())
                    .into_range_map_blaze();
                println!("left ^ right = {s_d}");
                panic!("expected_keys should be empty: {expected_keys}");
            }
        }
    }
}
#[test]
fn map_repro_insert_1() {
    let mut range_map_blaze = RangeMapBlaze::new();
    range_map_blaze.insert(123, "Hello");
    range_map_blaze.insert(123, "World");
    assert_eq!(range_map_blaze.to_string(), r#"(123..=123, "World")"#);
}

fn equal_maps<T: Integer, V: ValueOwned + fmt::Debug + std::fmt::Display>(
    range_map_blaze: &RangeMapBlaze<T, V>,
    btree_map: &BTreeMap<T, &V>,
) -> bool
where
    usize: std::convert::From<<T as Integer>::SafeLen>,
{
    // also, check that the ranges are really sorted and disjoint
    // cmk range_values should return a tuple not a struct
    // cmk implement iter for RangeMapBlaze
    let mut previous: Option<(RangeInclusive<T>, &V)> = None;
    for range_value in range_map_blaze.range_values() {
        let v = range_value.1;
        let range = range_value.0.clone();

        if let Some(previous) = previous {
            if (previous.1 == v && *previous.0.end() + T::one() >= *range.start())
                || previous.0.end() >= range.start()
            {
                eprintln!(
                    "two ranges are not disjoint: {:?}->{} and {range:?}->{v}",
                    previous.0, previous.1
                );
                return false;
            }
        }

        debug_assert!(range.start() <= range.end());
        let mut k = *range.start();
        loop {
            if btree_map.get(&k).map_or(true, |v2| v != *v2) {
                eprintln!(
                    "range_map_blaze contains {k} -> {v}, btree_map contains {k} -> {:?}",
                    btree_map.get(&k)
                );
                return false;
            }
            if k == *range.end() {
                break;
            }
            k = k + T::one();
        }
        previous = Some(range_value);
    }

    let len0: usize = range_map_blaze.len().into();
    if len0 != btree_map.len() {
        eprintln!(
            "range_map_blaze.len() = {len0}, btree_map.len() = {}",
            btree_map.len()
        );
        return false; // Different number of elements means they can't be the same map
    }

    true
}

fn format_range_values<'a, T>(iter: impl Iterator<Item = (RangeInclusive<T>, &'a u8)>) -> String
where
    T: Integer + fmt::Display + 'a, // Assuming T implements Display for formatting
                                    // V: ValueOwned + fmt::Display + 'a, // V must implement Display to be formatted with {}
{
    let mut vs = String::new();
    for range_value in iter {
        vs.push_str(&format!(
            "{}..={}{} ",
            range_value.0.start(),
            range_value.0.end(),
            *range_value.1 as char,
        ));
    }
    vs
}

#[test]
fn map_repro_123() {
    let input = [(123, 'a'), (123, 'b')];

    let range_map_blaze = RangeMapBlaze::<u8, char>::from_iter(input);
    assert_eq!(range_map_blaze.to_string(), "(123..=123, 'a')");
}

// cmk
#[test]
fn map_insert_str() {
    let s1 = "Hello".to_string();
    let s2 = "There".to_string();
    let range_map_blaze = RangeMapBlaze::<u8, String>::from_iter([(255, &s1), (25, &s2)]);
    assert_eq!(
        range_map_blaze.to_string(),
        r#"(25..=25, "There"), (255..=255, "Hello")"#
    );
}

#[test]
fn map_repro_bit_or() {
    let a = RangeSetBlaze::from_iter([1u8, 2, 3]);
    let b = RangeSetBlaze::from_iter([2u8, 3, 4]);

    let result = a.ranges().union(b.ranges());
    let result = result.into_range_set_blaze();
    println!("{result}");
    assert_eq!(result, RangeSetBlaze::from_iter([1u8, 2, 3, 4]));

    let result = a | b;
    println!("{result}");
    assert_eq!(result, RangeSetBlaze::from_iter([1u8, 2, 3, 4]));

    let a = RangeMapBlaze::from_iter([(1u8, "Hello"), (2, "Hello"), (3, "Hello")]);
    let b = RangeMapBlaze::from_iter([(2u8, "World"), (3, "World"), (4, "World")]);
    let result = a
        .range_values()
        .union(b.range_values())
        .into_range_map_blaze();
    println!("{result}");
    assert_eq!(
        result,
        RangeMapBlaze::from_iter([(1u8, "Hello"), (2u8, "Hello"), (3, "Hello"), (4, "World")])
    );

    let result = a | b;
    println!("{result}");
    assert_eq!(
        result,
        RangeMapBlaze::from_iter([(1u8, "Hello"), (2u8, "Hello"), (3, "Hello"), (4, "World")])
    );
}

#[test]
fn map_repro2() {
    let a = "a".to_string();
    let b = "b".to_string();
    let c = "c".to_string();
    let mut range_map_blaze = RangeMapBlaze::<i8, _>::from_iter([
        (-8, &a),
        (8, &a),
        (-2, &a),
        (-1, &a),
        (3, &a),
        (2, &b),
    ]);
    range_map_blaze.ranges_insert(25..=25, c);
    println!("{range_map_blaze}");
    assert!(
        range_map_blaze.to_string()
            == r#"(-8..=-8, "a"), (-2..=-1, "a"), (2..=2, "b"), (3..=3, "a"), (8..=8, "a"), (25..=25, "c")"#
    );
}

// #[test]
// fn multi_op() {
//     let a = RangeMapBlaze::from_iter([1..=6, 8..=9, 11..=15]);
//     let b = RangeMapBlaze::from_iter([5..=13, 18..=29]);
//     let c = RangeMapBlaze::from_iter([38..=42]);
//     let d = &(&a | &b) | &c;
//     println!("{d}");
//     let d = a | b | &c;
//     println!("{d}");

//     let a = RangeMapBlaze::from_iter([1..=6, 8..=9, 11..=15]);
//     let b = RangeMapBlaze::from_iter([5..=13, 18..=29]);
//     let c = RangeMapBlaze::from_iter([38..=42]);

//     let _ = [&a, &b, &c].union();
//     let d = [a, b, c].iter().intersection();
//     assert_eq!(d, RangeMapBlaze::new());

//     assert_eq!(
//         !MultiwayRangeSetBlaze::<u8>::union([]),
//         RangeMapBlaze::from_iter([0..=255])
//     );

//     let a = RangeMapBlaze::from_iter([1..=6, 8..=9, 11..=15]);
//     let b = RangeMapBlaze::from_iter([5..=13, 18..=29]);
//     let c = RangeMapBlaze::from_iter([1..=42]);

//     let _ = &a & &b;
//     let d = [&a, &b, &c].intersection();
//     // let d = RangeMapBlaze::intersection([a, b, c]);
//     println!("{d}");
//     assert_eq!(d, RangeMapBlaze::from_iter([5..=6, 8..=9, 11..=13]));

//     assert_eq!(
//         MultiwayRangeSetBlaze::<u8>::intersection([]),
//         RangeMapBlaze::from_iter([0..=255])
//     );
// }

// // https://stackoverflow.com/questions/21747136/how-do-i-print-in-rust-the-type-of-a-variable/58119924#58119924
// // fn print_type_of<T>(_: &T) {
// //     println!("{}", std::any::type_name::<T>())
// // }

// #[test]
// fn custom_multi() {
//     let a = RangeMapBlaze::from_iter([1..=6, 8..=9, 11..=15]);
//     let b = RangeMapBlaze::from_iter([5..=13, 18..=29]);
//     let c = RangeMapBlaze::from_iter([38..=42]);

//     let union_stream = b.ranges() | c.ranges();
//     let a_less = a.ranges() - union_stream;
//     let d: RangeMapBlaze<_> = a_less.into_range_set_blaze();
//     println!("{d}");

//     let d: RangeMapBlaze<_> =
//         (a.ranges() - [b.ranges(), c.ranges()].union()).into_range_set_blaze();
//     println!("{d}");
// }

// #[test]
// fn from_string() {
//     let a = RangeMapBlaze::from_iter([0..=4, 14..=17, 30..=255, 0..=37, 43..=65535]);
//     assert_eq!(a, RangeMapBlaze::from_iter([0..=65535]));
// }

// #[test]
// fn nand_repro() {
//     let b = &RangeMapBlaze::from_iter([5u8..=13, 18..=29]);
//     let c = &RangeMapBlaze::from_iter([38..=42]);
//     println!("about to nand");
//     let d = !b | !c;
//     assert_eq!(
//         d,
//         RangeMapBlaze::from_iter([0..=4, 14..=17, 30..=255, 0..=37, 43..=255])
//     );
// }

// #[test]
// fn bit_or_iter() {
//     let i = UnionIter::from([1, 3, 4, 2, 2, 43, -1, 4, 22]);
//     let j = UnionIter::from([11, 3, 4, 42, 2, 43, 23, 2, 543]);

//     let _not_i = !i.clone();
//     let k = i - j;
//     assert_eq!(k.to_string(), "-1..=-1, 1..=1, 22..=22");
// }

// #[test]
// fn empty() {
//     let universe: UnionIter<u8, _> = [0..=255].into_iter().collect();
//     let arr: [u8; 0] = [];
//     let a0 = RangeMapBlaze::<u8>::from_iter(arr);
//     assert!(!(a0.ranges()).equal(universe.clone()));
//     assert!((!a0).ranges().equal(universe));
//     let _a0 = RangeMapBlaze::from_iter([0..=0; 0]);
//     let _a = RangeMapBlaze::<i32>::new();

//     let a_iter: std::array::IntoIter<i32, 0> = [].into_iter();
//     let a = a_iter.collect::<RangeMapBlaze<i32>>();
//     let arr: [i32; 0] = [];
//     let b = RangeMapBlaze::from_iter(arr);
//     let mut c3 = a.clone();
//     let mut c5 = a.clone();

//     let c0 = (&a).bitor(&b);
//     let c1a = &a | &b;
//     let c1b = &a | b.clone();
//     let c1c = a.clone() | &b;
//     let c1d = a.clone() | b.clone();
//     let c2: RangeMapBlaze<_> = (a.ranges() | b.ranges()).into_range_set_blaze();
//     c3.append(&mut b.clone());
//     c5.extend(b);

//     let answer = RangeMapBlaze::from_iter(arr);
//     assert_eq!(&c0, &answer);
//     assert_eq!(&c1a, &answer);
//     assert_eq!(&c1b, &answer);
//     assert_eq!(&c1c, &answer);
//     assert_eq!(&c1d, &answer);
//     assert_eq!(&c2, &answer);
//     assert_eq!(&c3, &answer);
//     assert_eq!(&c5, &answer);

//     let a_iter: std::array::IntoIter<i32, 0> = [].into_iter();
//     let a = a_iter.collect::<RangeMapBlaze<i32>>();
//     let b = RangeMapBlaze::from_iter([0i32; 0]);

//     let c0 = a.ranges() | b.ranges();
//     let c1 = [a.ranges(), b.ranges()].union();
//     let c_list2: [RangesIter<i32>; 0] = [];
//     let c2 = c_list2.clone().union();
//     let c3 = union_dyn!(a.ranges(), b.ranges());
//     let c4 = c_list2.map(DynSortedDisjoint::new).union();

//     let answer = RangeMapBlaze::from_iter(arr);
//     assert!(c0.equal(answer.ranges()));
//     assert!(c1.equal(answer.ranges()));
//     assert!(c2.equal(answer.ranges()));
//     assert!(c3.equal(answer.ranges()));
//     assert!(c4.equal(answer.ranges()));

//     let c0 = !(a.ranges() & b.ranges());
//     let c1 = ![a.ranges(), b.ranges()].intersection();
//     let c_list2: [RangesIter<i32>; 0] = [];
//     let c2 = !!c_list2.clone().intersection();
//     let c3 = !intersection_dyn!(a.ranges(), b.ranges());
//     let c4 = !!c_list2.map(DynSortedDisjoint::new).intersection();

//     let answer = !RangeMapBlaze::from_iter([0i32; 0]);
//     assert!(c0.equal(answer.ranges()));
//     assert!(c1.equal(answer.ranges()));
//     assert!(c2.equal(answer.ranges()));
//     assert!(c3.equal(answer.ranges()));
//     assert!(c4.equal(answer.ranges()));
// }

// // Can't implement fmt::Display fmt must take ownership
// impl<T, I> UnsortedDisjoint<T, I>
// where
//     T: Integer,
//     I: Iterator<Item = RangeInclusive<T>>,
// {
//     #[allow(clippy::inherent_to_string)]
//     #[allow(clippy::wrong_self_convention)]
//     pub(crate) fn to_string(self) -> String {
//         self.map(|range| format!("{range:?}")).join(", ")
//     }
// }
// #[allow(clippy::reversed_empty_ranges)]
// #[test]
// fn private_constructor() {
//     let unsorted_disjoint = UnsortedDisjoint::from([5..=6, 1..=5, 1..=0, -12..=-10, 3..=3]);
//     // println!("{}", unsorted_disjoint.fmt());
//     assert_eq!(unsorted_disjoint.to_string(), "1..=6, -12..=-10, 3..=3");

//     let unsorted_disjoint = UnsortedDisjoint::from([5..=6, 1..=5, 1..=0, -12..=-10, 3..=3]);
//     let union_iter = UnionIter::from(unsorted_disjoint);
//     // println!("{}", union_iter.fmt());
//     assert_eq!(union_iter.to_string(), "-12..=-10, 1..=6");

//     let union_iter: UnionIter<_, _> = [5, 6, 1, 2, 3, 4, 5, -12, -11, -10, 3]
//         .into_iter()
//         .collect();
//     assert_eq!(union_iter.to_string(), "-12..=-10, 1..=6");
// }

// fn is_ddcppdheo<
//     T: std::fmt::Debug
//         + Display
//         + Clone
//         + PartialEq
//         + PartialOrd
//         + Default
//         + std::hash::Hash
//         + Eq
//         + Ord
//         + Send
//         + Sync,
// >() {
// }

// fn is_sssu<T: Sized + Send + Sync + Unpin>() {}
// fn is_like_btreeset_iter<T: Clone + std::fmt::Debug + FusedIterator + Iterator>() {}
// // cmk removed DoubleEndedIterator +ExactSizeIterator for now
// #[test]
// fn iter_traits() {
//     type ARangesIter<'a> = RangesIter<'a, i32>;
//     type AIter<'a> = Iter<i32, ARangesIter<'a>>;
//     is_sssu::<AIter>();
//     is_like_btreeset_iter::<AIter>();
// }

// fn is_like_btreeset_into_iter<T: std::fmt::Debug + FusedIterator + Iterator>() {}

// fn is_like_btreeset<
//     T: Clone
//         + std::fmt::Debug
//         + Default
//         + Eq
//         + std::hash::Hash
//         + IntoIterator
//         + Ord
//         + PartialEq
//         + PartialOrd
//         + RefUnwindSafe
//         + Send
//         + Sync
//         + Unpin
//         + UnwindSafe
//         + Any
//         + ToOwned,
// >() {
// }

// fn is_like_check_sorted_disjoint<
//     T: Clone
//         + std::fmt::Debug
//         + Default
//         + IntoIterator
//         + RefUnwindSafe
//         + Send
//         + Sync
//         + Unpin
//         + UnwindSafe
//         + Any
//         + ToOwned,
// >() {
// }

// fn is_like_dyn_sorted_disjoint<T: IntoIterator + Unpin + Any>() {}

// #[test]
// fn check_traits() {
//     // Debug/Display/Clone/PartialEq/PartialOrd/Default/Hash/Eq/Ord/Send/Sync
//     type ARangeSetBlaze = RangeMapBlaze<i32>;
//     is_sssu::<ARangeSetBlaze>();
//     is_ddcppdheo::<ARangeSetBlaze>();
//     is_like_btreeset::<ARangeSetBlaze>();

//     type ARangesIter<'a> = RangesIter<'a, i32>;
//     is_sssu::<ARangesIter>();
//     is_like_btreeset_iter::<ARangesIter>();

//     type AIter<'a> = Iter<i32, ARangesIter<'a>>;
//     is_sssu::<AIter>();
//     is_like_btreeset_iter::<AIter>();

//     is_sssu::<IntoIter<i32>>();
//     is_like_btreeset_into_iter::<IntoIter<i32>>();

//     type AMerge<'a> = Merge<i32, ARangesIter<'a>, ARangesIter<'a>>;
//     is_sssu::<AMerge>();
//     is_like_btreeset_iter::<AMerge>();

//     let a = RangeMapBlaze::from_iter([1..=2, 3..=4]);
//     println!("{:?}", a.ranges());

//     type AKMerge<'a> = KMerge<i32, ARangesIter<'a>>;
//     is_sssu::<AKMerge>();
//     is_like_btreeset_iter::<AKMerge>();

//     type ANotIter<'a> = NotIter<i32, ARangesIter<'a>>;
//     is_sssu::<ANotIter>();
//     is_like_btreeset_iter::<ANotIter>();

//     type AIntoRangesIter = IntoRangesIter<i32>;
//     is_sssu::<AIntoRangesIter>();
//     is_like_btreeset_into_iter::<AIntoRangesIter>();

//     type ACheckSortedDisjoint<'a> = CheckSortedDisjoint<i32, ARangesIter<'a>>;
//     is_sssu::<ACheckSortedDisjoint>();
//     type BCheckSortedDisjoint =
//         CheckSortedDisjoint<i32, std::array::IntoIter<RangeInclusive<i32>, 0>>;
//     is_like_check_sorted_disjoint::<BCheckSortedDisjoint>();

//     type ADynSortedDisjoint<'a> = DynSortedDisjoint<'a, i32>;
//     is_like_dyn_sorted_disjoint::<ADynSortedDisjoint>();

//     type AUnionIter<'a> = UnionIter<i32, ARangesIter<'a>>;
//     is_sssu::<AUnionIter>();
//     is_like_btreeset_iter::<AUnionIter>();

//     type AAssumeSortedStarts<'a> = AssumeSortedStarts<i32, ARangesIter<'a>>;
//     is_sssu::<AAssumeSortedStarts>();
//     is_like_btreeset_iter::<AAssumeSortedStarts>();
// }

// #[test]
// fn integer_coverage() {
//     syntactic_for! { ty in [i8, u8, isize, usize,  i16, u16, i32, u32, i64, u64, isize, usize, i128, u128] {
//         $(
//             let len = <$ty as Integer>::SafeLen::one();
//             let a = $ty::zero();
//             assert_eq!($ty::safe_len_to_f64(len), 1.0);
//             assert_eq!($ty::add_len_less_one(a,len), a);
//             assert_eq!($ty::sub_len_less_one(a,len), a);
//             assert_eq!($ty::f64_to_safe_len(1.0), len);
//             assert!($ty::safe_max_value()<=$ty::max_value());
//             assert!(<$ty as Integer>::safe_max_value()<=$ty::max_value());

//         )*
//     }};
// }

// #[test]
// #[allow(clippy::bool_assert_comparison)]
// fn lib_coverage_0() {
//     let a = RangeMapBlaze::from_iter([1..=2, 3..=4]);
//     let mut hasher = DefaultHasher::new();
//     a.hash(&mut hasher);
//     let _d = RangeMapBlaze::<i32>::default();
//     assert_eq!(a, a);

//     let mut set = RangeMapBlaze::new();
//     assert_eq!(set.first(), None);
//     set.insert(1);
//     assert_eq!(set.first(), Some(1));
//     set.insert(2);
//     assert_eq!(set.first(), Some(1));

//     let set = RangeMapBlaze::from_iter([(1, "Hello"), (2, "World"), (3, "World")]);
//     assert_eq!(set.get(2), Some(2));
//     assert_eq!(set.get(4), None);

//     let mut set = RangeMapBlaze::new();
//     assert_eq!(set.last(), None);
//     set.insert(1);
//     assert_eq!(set.last(), Some(1));
//     set.insert(2);
//     assert_eq!(set.last(), Some(2));

//     assert_eq!(a.len(), a.len_slow());

//     let mut a = RangeMapBlaze::from_iter([1..=3]);
//     let mut b = RangeMapBlaze::from_iter([3..=5]);

//     a.append(&mut b);

//     assert_eq!(a.len(), 5 as I32SafeLen);
//     assert_eq!(b.len(), 0 as I32SafeLen);

//     assert!(a.contains(1));
//     assert!(a.contains(2));
//     assert!(a.contains(3));
//     assert!(a.contains(4));
//     assert!(a.contains(5));

//     let mut v = RangeMapBlaze::new();
//     v.insert(1);
//     v.clear();
//     assert!(v.is_empty());

//     let mut v = RangeMapBlaze::new();
//     assert!(v.is_empty());
//     v.insert(1);
//     assert!(!v.is_empty());

//     let sup = RangeMapBlaze::from_iter([1..=3]);
//     let mut set = RangeMapBlaze::new();

//     assert_eq!(set.is_subset(&sup), true);
//     set.insert(2);
//     assert_eq!(set.is_subset(&sup), true);
//     set.insert(4);
//     assert_eq!(set.is_subset(&sup), false);

//     let sub = RangeMapBlaze::from_iter([1, 2]);
//     let mut set = RangeMapBlaze::new();

//     assert_eq!(set.is_superset(&sub), false);

//     set.insert(0);
//     set.insert(1);
//     assert_eq!(set.is_superset(&sub), false);

//     set.insert(2);
//     assert_eq!(set.is_superset(&sub), true);

//     let a = RangeMapBlaze::from_iter([1..=3]);
//     let mut b = RangeMapBlaze::new();

//     assert_eq!(a.is_disjoint(&b), true);
//     b.insert(4);
//     assert_eq!(a.is_disjoint(&b), true);
//     b.insert(1);
//     assert_eq!(a.is_disjoint(&b), false);

//     let mut set = RangeMapBlaze::new();
//     set.insert(3);
//     set.insert(5);
//     set.insert(8);
//     assert_eq!(Some(5), set.0(4..).next());
//     assert_eq!(Some(3), set.0(..).next());
//     assert_eq!(None, set.0(..=2).next());
//     assert_eq!(None, set.0(1..2).next());
//     assert_eq!(
//         Some(3),
//         set.0((Bound::Excluded(2), Bound::Excluded(4))).next()
//     );

//     let mut set = RangeMapBlaze::new();

//     assert_eq!(set.ranges_insert(2..=5), true);
//     assert_eq!(set.ranges_insert(5..=6), true);
//     assert_eq!(set.ranges_insert(3..=4), false);
//     assert_eq!(set.len(), 5 as I32SafeLen);
//     let mut set = RangeMapBlaze::from_iter([(1, "Hello"), (2, "World"), (3, "World")]);
//     assert_eq!(set.take(2), Some(2));
//     assert_eq!(set.take(2), None);

//     let mut set = RangeMapBlaze::new();
//     assert!(set.replace(5).is_none());
//     assert!(set.replace(5).is_some());

//     let mut a = RangeMapBlaze::from_iter([1..=3]);
//     #[allow(clippy::reversed_empty_ranges)]
//     a.internal_add(2..=1);

//     assert_eq!(a.partial_cmp(&a), Some(Ordering::Equal));

//     let mut a = RangeMapBlaze::from_iter([1..=3]);
//     a.extend(std::iter::once(4));
//     assert_eq!(a.len(), 4 as I32SafeLen);

//     let mut a = RangeMapBlaze::from_iter([1..=3]);
//     a.extend(4..=5);
//     assert_eq!(a.len(), 5 as I32SafeLen);

//     let mut set = RangeMapBlaze::new();

//     set.insert(1);
//     while let Some(n) = set.pop_first() {
//         assert_eq!(n, 1);
//     }
//     assert!(set.is_empty());

//     let mut set = RangeMapBlaze::new();

//     set.insert(1);
//     while let Some(n) = set.pop_last() {
//         assert_eq!(n, 1);
//     }
//     assert!(set.is_empty());

//     let a = RangeMapBlaze::from_iter([1..=3]);
//     let i = a.iter();
//     let j = i.clone();
//     assert_eq!(i.size_hint(), j.size_hint());
//     assert_eq!(format!("{:?}", &i), format!("{:?}", &j));

//     let a = RangeMapBlaze::from_iter([1..=3]);
//     let i = a.into_iter();
//     assert_eq!(i.size_hint(), j.size_hint());
//     assert_eq!(
//         format!("{:?}", &i),
//         "IntoIter { option_range_front: None, option_range_back: None, into_iter: [(1, 3)] }"
//     );

//     let mut a = RangeMapBlaze::from_iter([1..=3]);
//     a.extend([1..=3]);
//     assert_eq!(a.len(), 3 as I32SafeLen);

//     let a = RangeMapBlaze::from_iter([1..=3]);
//     let b = <RangeMapBlaze<i32> as Clone>::clone(&a);
//     assert_eq!(a, b);
//     let c = <RangeMapBlaze<i32> as Default>::default();
//     assert_eq!(c, RangeMapBlaze::new());

//     syntactic_for! { ty in [i8, u8, isize, usize,  i16, u16, i32, u32, i64, u64, isize, usize, i128, u128] {
//         $(
//             let a = RangeMapBlaze::<$ty>::new();
//             println!("{a:#?}");
//             assert_eq!(a.iter().next(), None);

//             let mut a = RangeMapBlaze::from_iter([$ty::one()..=3]);
//             let mut b = RangeMapBlaze::from_iter([3..=5]);

//             a.append(&mut b);

//             // assert_eq!(a.len(), 5usize);
//             assert_eq!(b.len(), <$ty as Integer>::SafeLen::zero());

//             assert!(a.contains(1));
//             assert!(a.contains(2));
//             assert!(a.contains(3));
//             assert!(a.contains(4));
//             assert!(a.contains(5));

//             assert!(b.is_empty());

//             let a = RangeMapBlaze::from_iter([$ty::one()..=3]);
//             let b = RangeMapBlaze::from_iter([3..=5]);
//             assert!(!a.is_subset(&b));
//             assert!(!a.is_superset(&b));

//         )*
//     }};

//     let a = RangeMapBlaze::from_iter([1u128..=3]);
//     assert!(a.contains(1));
//     assert!(!a.is_disjoint(&a));
// }

// #[test]
// #[should_panic]
// fn lib_coverage_2() {
//     let v = RangeMapBlaze::<u128>::new();
//     v.contains(u128::MAX);
// }

// #[test]
// #[should_panic]
// fn lib_coverage_3() {
//     let mut v = RangeMapBlaze::<u128>::new();
//     v.remove(u128::MAX);
// }

// #[test]
// #[should_panic]
// fn lib_coverage_4() {
//     let mut v = RangeMapBlaze::<u128>::new();
//     v.split_off(u128::MAX);
// }

// #[test]
// #[should_panic]
// fn lib_coverage_5() {
//     let mut v = RangeMapBlaze::<u128>::new();
//     v.internal_add(0..=u128::MAX);
// }

// #[test]
// fn lib_coverage_6() {
//     syntactic_for! { ty in [i8, u8, isize, usize,  i16, u16, i32, u32, i64, u64, isize, usize, i128, u128] {
//         $(
//             let mut a = RangeMapBlaze::<$ty>::from_iter([1..=3, 5..=7, 9..=120]);
//             a.ranges_insert(2..=100);
//             assert_eq!(a, RangeMapBlaze::from_iter([1..=120]));

//         )*
//     }};
// }

// #[test]
// fn merge_coverage_0() {
//     let a = CheckSortedDisjoint::new([1..=2, 5..=100].into_iter());
//     let b = CheckSortedDisjoint::from([2..=6]);
//     let m = Merge::new(a, b);
//     let n = m.clone();
//     let p = n.clone();
//     let union1 = UnionIter::new(m);
//     let union2 = UnionIter::new(n);
//     assert!(union1.equal(union2));
//     assert!(format!("{p:?}").starts_with("Merge"));

//     let a = CheckSortedDisjoint::new([1..=2, 5..=100].into_iter());
//     let b = CheckSortedDisjoint::new([2..=6].into_iter());
//     let c = CheckSortedDisjoint::new([-1..=-1].into_iter());
//     let m = KMerge::new([a, b, c]);
//     let n = m.clone();
//     let p = n.clone();
//     let union1 = UnionIter::new(m);
//     let union2 = UnionIter::new(n);
//     assert!(union1.equal(union2));
//     assert!(format!("{p:?}").starts_with("KMerge"));
// }

// #[test]
// fn not_iter_coverage_0() {
//     let a = CheckSortedDisjoint::new([1..=2, 5..=100].into_iter());
//     let n = NotIter::new(a);
//     let p = n.clone();
//     let m = p.clone();
//     assert!(n.equal(m));
//     assert!(format!("{p:?}").starts_with("NotIter"));
// }

// #[test]
// fn ranges_coverage_0() {
//     let a = RangeMapBlaze::from_iter([1..=2, 5..=100]);
//     let r = a.ranges();
//     let p = r.as_ref();
//     assert!(format!("{p:?}").starts_with("Ranges"));
//     assert_eq!(r.len(), 2);

//     let r2 = a.into_ranges();
//     let n2 = !!r2;
//     let a = RangeMapBlaze::from_iter([1..=2, 5..=100]);
//     assert!(n2.equal(a.ranges()));
//     let a = RangeMapBlaze::from_iter([1..=2, 5..=100]);
//     let b = a.into_ranges();
//     let a = RangeMapBlaze::from_iter([1..=2, 5..=100]);
//     let c = a.into_ranges();
//     let a = RangeMapBlaze::from_iter([1..=2, 5..=100]);
//     assert!((b | c).equal(a.ranges()));

//     let a = RangeMapBlaze::from_iter([1..=2, 5..=100]).into_ranges();
//     let b = RangeMapBlaze::from_iter([1..=2, 5..=100]).into_ranges();
//     assert!((a - b).is_empty());

//     let a = RangeMapBlaze::from_iter([1..=2, 5..=100]).into_ranges();
//     let b = RangeMapBlaze::from_iter([1..=2, 5..=100]).into_ranges();
//     assert!((a ^ b).is_empty());

//     let a = RangeMapBlaze::from_iter([1..=2, 5..=100]).into_ranges();
//     let b = RangeMapBlaze::from_iter([1..=2, 5..=100]).into_ranges();
//     assert!((a & b).equal(RangeMapBlaze::from_iter([1..=2, 5..=100]).into_ranges()));

//     assert_eq!(
//         RangeMapBlaze::from_iter([1..=2, 5..=100])
//             .into_ranges()
//             .len(),
//         2
//     );
//     assert!(format!(
//         "{:?}",
//         RangeMapBlaze::from_iter([1..=2, 5..=100]).into_ranges()
//     )
//     .starts_with("IntoRanges"));
// }

// #[test]
// fn sorted_disjoint_coverage_0() {
//     let a = CheckSortedDisjoint::<i32, _>::default();
//     assert!(a.is_empty());

//     let a = CheckSortedDisjoint::new([1..=2, 5..=100].into_iter());
//     let b = CheckSortedDisjoint::new([1..=2, 5..=100].into_iter());
//     assert!((a & b).equal(CheckSortedDisjoint::new([1..=2, 5..=100].into_iter())));

//     let a = CheckSortedDisjoint::new([1..=2, 5..=100].into_iter());
//     let b = CheckSortedDisjoint::new([1..=2, 5..=100].into_iter());
//     assert!((a - b).is_empty());

//     let a = CheckSortedDisjoint::new([1..=2, 5..=100].into_iter());
//     let b = CheckSortedDisjoint::new([1..=2, 5..=100].into_iter());
//     assert!((a ^ b).is_empty());
// }

// #[test]
// #[should_panic]
// fn sorted_disjoint_coverage_1() {
//     struct SomeAfterNone {
//         a: i32,
//     }
//     impl Iterator for SomeAfterNone {
//         type Item = RangeInclusive<i32>;
//         fn next(&mut self) -> Option<Self::Item> {
//             self.a += 1;
//             if self.a % 2 == 0 {
//                 Some(self.a..=self.a)
//             } else {
//                 None
//             }
//         }
//     }

//     let mut a = CheckSortedDisjoint::new(SomeAfterNone { a: 0 });
//     a.next();
//     a.next();
//     a.next();
// }

// #[test]
// #[should_panic]
// fn sorted_disjoint_coverage_2() {
//     #[allow(clippy::reversed_empty_ranges)]
//     let mut a = CheckSortedDisjoint::new([1..=0].into_iter());
//     a.next();
// }

// #[test]
// #[should_panic]
// fn sorted_disjoint_coverage_3() {
//     #[allow(clippy::reversed_empty_ranges)]
//     let mut a = CheckSortedDisjoint::new([1..=1, 2..=2].into_iter());
//     a.next();
//     a.next();
// }

// #[test]
// #[should_panic]
// fn sorted_disjoint_coverage_4() {
//     #[allow(clippy::reversed_empty_ranges)]
//     let mut a = CheckSortedDisjoint::new([0..=i128::MAX].into_iter());
//     a.next();
// }

// #[test]
// fn sorted_disjoint_iterator_coverage_0() {
//     let a = CheckSortedDisjoint::new([1..=2, 5..=100].into_iter());
//     let b = CheckSortedDisjoint::new([1..=2, 5..=101].into_iter());
//     assert!(b.is_superset(a));
// }

// #[test]
// fn union_iter_coverage_0() {
//     let a = CheckSortedDisjoint::new([1..=2, 5..=100].into_iter());
//     let b = CheckSortedDisjoint::new([1..=2, 5..=101].into_iter());
//     let c = a.union(b);
//     assert!(format!("{c:?}").starts_with("UnionIter"));
// }

// #[test]
// fn unsorted_disjoint_coverage_0() {
//     let a = AssumeSortedStarts::new([1..=2, 5..=100].into_iter());
//     assert!(format!("{a:?}").starts_with("AssumeSortedStarts"));
// }

// #[test]
// fn test_coverage_0() {
//     let a = BooleanVector(vec![true, true, false, false]);
//     assert!(format!("{a:?}").starts_with("BooleanVector"));

//     let a = How::Union;
//     #[allow(clippy::clone_on_copy)]
//     let _b = a.clone();

//     let mut rng = StdRng::seed_from_u64(0);
//     let a = MemorylessRange::new(&mut rng, 1000, 0..=10, 0.5, 1, How::Intersection);
//     let v: Vec<_> = a.take(100).collect();
//     println!("{v:?}");

//     let a = MemorylessIter::new(&mut rng, 1000, 0..=10, 0.5, 1, How::Intersection);
//     let v: Vec<_> = a.take(100).collect();
//     println!("{v:?}");
// }

// type Element = i64;
// type Reference = std::collections::BTreeSet<Element>;

// #[quickcheck]
// fn disjoint(a: Reference, b: Reference) -> bool {
//     let a_r = RangeMapBlaze::from_iter(&a);
//     let b_r = RangeMapBlaze::from_iter(&b);
//     a.is_disjoint(&b) == a_r.is_disjoint(&b_r)
// }

// #[quickcheck]
// fn subset(a: Reference, b: Reference) -> bool {
//     let a_r = RangeMapBlaze::from_iter(&a);
//     let b_r = RangeMapBlaze::from_iter(&b);
//     a.is_subset(&b) == a_r.is_subset(&b_r)
// }

// #[quickcheck]
// fn superset(a: Reference, b: Reference) -> bool {
//     let a_r = RangeMapBlaze::from_iter(&a);
//     let b_r = RangeMapBlaze::from_iter(&b);
//     a.is_superset(&b) == a_r.is_superset(&b_r)
// }

// /// just a helper to get good output when a check fails
// fn binary_op<E: Debug, R: Eq + Debug>(a: E, b: E, expected: R, actual: R) -> bool {
//     let res = expected == actual;
//     if !res {
//         println!("a:{a:?} b:{b:?} expected:{expected:?} actual:{actual:?}");
//     }
//     res
// }

// /// from: https://github.com/rklaehn/sorted-iter
// /// just a helper to get good output when a check fails
// fn check_size_hint<E: Debug>(
//     input: E,
//     expected: usize,
//     (min, max): (usize, Option<usize>),
// ) -> bool {
//     let res = min <= expected && max.map_or(true, |max| expected <= max && min <= max);
//     if !res {
//         println!("input:{input:?} expected:{expected:?} min:{min:?} max:{max:?}");
//     }
//     res
// }

// #[quickcheck]
// fn intersection(a: Reference, b: Reference) -> bool {
//     let a_r = RangeMapBlaze::from_iter(&a);
//     let b_r = RangeMapBlaze::from_iter(&b);
//     let expected: Reference = a.intersection(&b).cloned().collect();
//     let actual: Reference = (a_r & b_r).into_iter().collect();
//     binary_op(a, b, expected, actual)
// }

// #[quickcheck]
// fn union(a: Reference, b: Reference) -> bool {
//     let a_r = RangeMapBlaze::from_iter(&a);
//     let b_r = RangeMapBlaze::from_iter(&b);
//     let expected: Reference = a.union(&b).cloned().collect();
//     let actual: Reference = (a_r | b_r).into_iter().collect();
//     binary_op(a, b, expected, actual)
// }

// #[quickcheck]
// fn symmetric_difference_cmk_update(a: Reference, b: Reference) -> bool {
//     let a_r = RangeMapBlaze::from_iter(&a);
//     let b_r = RangeMapBlaze::from_iter(&b);
//     let expected: Reference = a.union(&b).cloned().collect();
//     let actual: Reference = (a_r | b_r).into_iter().collect();
//     binary_op(a, b, expected, actual)
// }

// #[quickcheck]
// fn multi_union(inputs: Vec<Reference>) -> bool {
//     use crate::MultiwayRangeSetBlazeRef;

//     let expected: Reference = inputs.iter().flatten().copied().collect();
//     let actual = inputs.iter().map(RangeMapBlaze::from_iter).union();

//     let res = actual.iter().eq(expected.iter().cloned());
//     if !res {
//         let actual: Reference = actual.iter().collect();
//         println!("in:{inputs:?} expected:{expected:?} out:{actual:?}");
//     }
//     res
// }

// #[quickcheck]
// fn difference(a: Reference, b: Reference) -> bool {
//     let a_r = RangeMapBlaze::from_iter(&a);
//     let b_r = RangeMapBlaze::from_iter(&b);
//     let expected: Reference = a.difference(&b).cloned().collect();
//     let actual: Reference = (a_r - b_r).into_iter().collect();
//     binary_op(a, b, expected, actual)
// }

// #[quickcheck]
// fn symmetric_difference(a: Reference, b: Reference) -> bool {
//     let a_r = RangeMapBlaze::from_iter(&a);
//     let b_r = RangeMapBlaze::from_iter(&b);
//     let expected: Reference = a.symmetric_difference(&b).cloned().collect();
//     let actual: Reference = (a_r ^ b_r).into_iter().collect();
//     binary_op(a, b, expected, actual)
// }

// #[quickcheck]
// fn intersection_size_hint(a: Reference, b: Reference) -> bool {
//     let expected = a.intersection(&b).count();
//     let a_r = RangeMapBlaze::from_iter(&a);
//     let b_r = RangeMapBlaze::from_iter(&b);
//     let actual = (a_r & b_r).into_iter().size_hint();
//     check_size_hint((a, b), expected, actual)
// }

// #[quickcheck]
// fn union_size_hint(a: Reference, b: Reference) -> bool {
//     let expected = a.union(&b).count();
//     let a_r = RangeMapBlaze::from_iter(&a);
//     let b_r = RangeMapBlaze::from_iter(&b);
//     let actual = (a_r | b_r).into_iter().size_hint();
//     check_size_hint((a, b), expected, actual)
// }

// #[quickcheck]
// fn multi_union_size_hint(inputs: Vec<Reference>) -> bool {
//     let expected: Reference = inputs.iter().flatten().copied().collect();
//     let actual = inputs
//         .iter()
//         .map(RangeMapBlaze::from_iter)
//         .union()
//         .iter()
//         .size_hint();
//     check_size_hint(inputs, expected.len(), actual)
// }

// #[quickcheck]
// fn difference_size_hint(a: Reference, b: Reference) -> bool {
//     let expected = a.difference(&b).count();
//     let a_r = RangeMapBlaze::from_iter(&a);
//     let b_r = RangeMapBlaze::from_iter(&b);
//     let actual = (a_r - b_r).into_iter().size_hint();
//     check_size_hint((a, b), expected, actual)
// }

// #[quickcheck]
// fn symmetric_difference_size_hint(a: Reference, b: Reference) -> bool {
//     let expected = a.symmetric_difference(&b).count();
//     let a_r = RangeMapBlaze::from_iter(&a);
//     let b_r = RangeMapBlaze::from_iter(&b);
//     let actual = (a_r ^ b_r).into_iter().size_hint();
//     check_size_hint((a, b), expected, actual)
// }

// #[should_panic]
// #[test]
// fn demo_read() {
//     let _a: RangeMapBlaze<i32> = demo_read_ranges_from_file("tests/no_such_file").unwrap();
// }

// #[test]
// fn double_end_iter() {
//     let a = RangeMapBlaze::from_iter([3..=10, 12..=12, 20..=25]);

//     assert_eq!(
//         a.iter().rev().collect::<Vec<usize>>(),
//         vec![25, 24, 23, 22, 21, 20, 12, 10, 9, 8, 7, 6, 5, 4, 3]
//     );

//     {
//         let mut iter = a.iter();

//         assert_eq!(iter.next(), Some(3));
//         assert_eq!(iter.next_back(), Some(25));
//         assert_eq!(iter.next(), Some(4));
//         assert_eq!(iter.next_back(), Some(24));
//         assert_eq!(iter.next_back(), Some(23));
//         assert_eq!(iter.next_back(), Some(22));
//         assert_eq!(iter.next_back(), Some(21));
//         assert_eq!(iter.next_back(), Some(20));

//         // Next interval
//         assert_eq!(iter.next_back(), Some(12));

//         // Next interval, now same interval as front of the iterator
//         assert_eq!(iter.next_back(), Some(10));
//         assert_eq!(iter.next(), Some(5));
//     }
// }
// #[test]
// fn double_end_into_iter() {
//     let a = RangeMapBlaze::from_iter([3..=10, 12..=12, 20..=25]);

//     assert_eq!(
//         a.clone().into_iter().rev().collect::<Vec<usize>>(),
//         vec![25, 24, 23, 22, 21, 20, 12, 10, 9, 8, 7, 6, 5, 4, 3]
//     );

//     let mut iter = a.into_iter();

//     assert_eq!(iter.next(), Some(3));
//     assert_eq!(iter.next_back(), Some(25));
//     assert_eq!(iter.next(), Some(4));
//     assert_eq!(iter.next_back(), Some(24));
//     assert_eq!(iter.next_back(), Some(23));
//     assert_eq!(iter.next_back(), Some(22));
//     assert_eq!(iter.next_back(), Some(21));
//     assert_eq!(iter.next_back(), Some(20));

//     // Next interval
//     assert_eq!(iter.next_back(), Some(12));

//     // Next interval, now same interval as front of the iterator
//     assert_eq!(iter.next_back(), Some(10));
//     assert_eq!(iter.next(), Some(5));
// }
// #[test]
// fn double_end_range() {
//     let a = RangeMapBlaze::from_iter([3..=10, 12..=12, 20..=25]);

//     let mut range = a.0(11..=22);
//     assert_eq!(range.next_back(), Some(22));
//     assert_eq!(range.next(), Some(12));
//     assert_eq!(range.next(), Some(20));
// }
