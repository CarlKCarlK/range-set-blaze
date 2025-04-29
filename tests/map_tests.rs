#![cfg(test)]
#![allow(unexpected_cfgs)]

use core::ops::Bound::Included;
use std::collections::HashMap;
extern crate alloc;
use alloc::collections::BTreeMap;
use core::cmp::Ordering;
use core::fmt;
use core::ops::RangeInclusive;
#[cfg(not(target_arch = "wasm32"))]
use quickcheck_macros::quickcheck;
use rand::seq::IndexedRandom;
use rand::{Rng, SeedableRng, rngs::StdRng};
use range_set_blaze::Integer;
use range_set_blaze::{
    IntersectionIterMap, IntoRangeValuesIter, KMergeMap, RangeValuesIter, SymDiffIterMap,
    UnionIterMap, ValueRef, prelude::*,
};
use std::borrow::Borrow;
use std::iter::FusedIterator;
use std::ops::Bound;
use std::rc::Rc;
use std::sync::Arc;
use std::{
    io::{Write, stdout},
    thread::sleep,
    time::Duration,
};
use tests_common::{How, k_maps};

use syntactic_for::syntactic_for;

use wasm_bindgen_test::*;
wasm_bindgen_test_configure!(run_in_browser);

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
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
    let _ = adm.map_and_set_intersection(bdm.into_sorted_disjoint());
    let adm = arm.range_values();
    let bdm = brm.range_values();
    let _ = adm.map_and_set_difference(bdm.into_sorted_disjoint());
    let adm = arm.range_values();
    let bdm = brm.range_values();
    let _ = adm.symmetric_difference(bdm);
    let adm = arm.range_values();
    let _ = adm.complement();

    // SortedDisjointMap/SortedDisjointSet
    // intersection, difference
    let adm = arm.range_values();
    let bds = brm.ranges();
    let _ = adm.map_and_set_intersection(bds);
    let adm = arm.range_values();
    let bds = brm.ranges();
    let _ = adm.map_and_set_difference(bds);
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
fn map_insert_255u8() {
    let btree_map = BTreeMap::from_iter([(255u8, "First")]);
    assert_eq!(btree_map.get(&255u8), Some(&"First"));
    let range_map_blaze = RangeMapBlaze::from_iter([(255u8, "First".to_string())]);
    assert_eq!(range_map_blaze.to_string(), r#"(255..=255, "First")"#);

    let iter = [
        (255u8..=255, "Hello".to_string()),
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
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
fn map_insert_max_u128() {
    let _ = RangeMapBlaze::<u128, _>::from_iter([(u128::MAX, "Too Big")]);
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
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
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
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
        .map_and_set_intersection(b.ranges())
        .into_range_map_blaze();
    println!("{result}");
    assert_eq!(result, RangeMapBlaze::from_iter([(2..=3, "World")]));

    let result = a & b;
    println!("{result}");
    assert_eq!(result, RangeMapBlaze::from_iter([(2..=3, "World")]));
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
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
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
fn map_doctest2() {
    let set = RangeMapBlaze::from_iter([(1u8, "Hello"), (2, "Hello"), (3, "Hello")]);
    assert!(set.contains_key(1));
    assert!(!set.contains_key(4));

    let set = RangeMapBlaze::<u8, _>::from_iter([(1, "Hello"), (2, "World"), (3, "World")]);
    assert_eq!(set.get(1), Some(&"Hello"));
    assert_eq!(set.get(4), None);
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
fn map_doctest3() {
    let mut a = RangeMapBlaze::from_iter([(1..=3, "Hello")]);
    let mut b = RangeMapBlaze::from_iter([(3..=5, "World")]);

    a.append(&mut b);

    assert_eq!(a.len(), 5u64);
    assert_eq!(b.len(), 0u64);

    assert_eq!(a.get(1), Some(&"Hello"));
    assert_eq!(a.get(2), Some(&"Hello"));
    assert_eq!(a.get(3), Some(&"World"));
    assert_eq!(a.get(4), Some(&"World"));
    assert_eq!(a.get(5), Some(&"World"));

    let mut a = RangeMapBlaze::from_iter([(1u8..=3, "Hello")]);
    let mut b = RangeMapBlaze::from_iter([(3u8..=5, "World")]);

    a.append(&mut b);

    assert_eq!(a.len(), 5);
    assert_eq!(b.len(), 0);

    assert!(a.contains_key(1));
    assert!(a.contains_key(2));
    assert!(a.contains_key(3));
    assert!(a.contains_key(4));
    assert!(a.contains_key(5));
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
fn map_doctest4() {
    let a = RangeMapBlaze::from_iter([(1i8, "Hello"), (2, "Hello"), (3, "Hello")]);

    let result = !&a;
    assert_eq!(result.to_string(), "-128..=0, 4..=127");
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
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
//#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
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
//#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
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
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
fn map_iters() -> Result<(), Box<dyn std::error::Error>> {
    let range_map_blaze =
        RangeMapBlaze::from_iter([(1u8..=6, "Hello"), (8..=9, "There"), (11..=15, "World")]);
    assert!(range_map_blaze.len() == 13);
    for (k, v) in &range_map_blaze {
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
    for (k, v) in &range_map_blaze {
        println!("{k}:{v}");
    }
    // range_map_blaze.len();

    let mut rs = range_map_blaze.range_values().complement();
    println!("{:?}", rs.next());
    println!("{range_map_blaze}");
    // !!! assert that can't use range_map_blaze again
    Ok(())
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
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
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
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

    let d = [a, b, c].union();
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

    assert_eq!(
        MultiwayRangeMapBlazeRef::<u8, char>::union([]),
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
    //     MultiwayRangeMapBlazeRef::<u8, char>::intersection([]),
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
        MultiwayRangeMapBlazeRef::<u8, char>::symmetric_difference([]),
        RangeMapBlaze::new()
    );

    Ok(())
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
fn map_custom_multi() -> Result<(), Box<dyn std::error::Error>> {
    let a = RangeMapBlaze::from_iter([(1..=6, 'a'), (8..=9, 'a'), (11..=15, 'a')]);
    let b = RangeMapBlaze::from_iter([(5..=13, 'b'), (18..=29, 'b')]);
    let c = RangeMapBlaze::from_iter([(38..=42, 'c')]);

    let union_stream = b.range_values().union(c.range_values());
    let a_less = a
        .range_values()
        .map_and_set_difference(union_stream.into_sorted_disjoint());
    let d: RangeMapBlaze<_, _> = a_less.into_range_map_blaze();
    assert_eq!(d, RangeMapBlaze::from_iter([(1..=4, 'a'), (14..=15, 'a')]));

    let d: RangeMapBlaze<_, _> = a
        .range_values()
        .map_and_set_difference(
            [b.range_values(), c.range_values()]
                .union()
                .into_sorted_disjoint(),
        )
        .into_range_map_blaze();
    assert_eq!(d, RangeMapBlaze::from_iter([(1..=4, 'a'), (14..=15, 'a')]));
    Ok(())
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
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
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
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
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
fn map_parity() -> Result<(), Box<dyn std::error::Error>> {
    // notice these are all borrowed
    let a = &RangeMapBlaze::from_iter([(1..=6, 'a'), (8..=9, 'a'), (11..=15, 'a')]);
    let b = &RangeMapBlaze::from_iter([(5..=13, 'b'), (18..=29, 'b')]);
    let c = &RangeMapBlaze::from_iter([(38..=42, 'c')]);
    assert_eq!(
        a & b & c | a & (&(!b & !c)) | b & (&(!a & !c)) | c & (&(!a & !b)),
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
    let b_comp = (b).range_values().complement_with(&'B');
    let c_comp = (c).range_values().complement_with(&'C');
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
            b.range_values().complement_with(&'a'),
            c.range_values().complement_with(&'a')
        ),
        intersection_map_dyn!(
            b.range_values(),
            a.range_values().complement_with(&'b'),
            c.range_values().complement_with(&'b')
        ),
        intersection_map_dyn!(
            c.range_values(),
            a.range_values().complement_with(&'c'),
            b.range_values().complement_with(&'c')
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

    // test on zero maps
    let a: SymDiffIterMap<i32, &&str, KMergeMap<i32, &&str, DynSortedDisjointMap<'_, i32, &&str>>> =
        symmetric_difference_map_dyn!();
    let a = a.into_range_map_blaze();
    let b: RangeMapBlaze<i32, &str> = RangeMapBlaze::default();
    assert_eq!(a, b);

    Ok(())
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
fn map_complement() -> Result<(), Box<dyn std::error::Error>> {
    // RangeMapBlaze, RangesIter, NotIter, UnionIterMap, Tee, UnionIterMap(g)
    let a0 = RangeMapBlaze::from_iter([(1..=6, "a0")]);
    let a1 = RangeMapBlaze::from_iter([(8..=9, "a1"), (11..=15, "a1")]);
    let a = &a0 | &a1;
    let not_a = &a.complement_with(&"A");
    let b = a.range_values();
    let c = not_a.range_values().complement_with(&"A");
    let d = a0.range_values().union(a1.range_values());
    let e = a.range_values(); // with range instead of range values used 'tee' here

    let f = UnionIterMap::from_iter([
        (15..=15, &"f"),
        (14..=14, &"f"),
        (15..=15, &"f"),
        (13..=13, &"f"),
        (12..=12, &"f"),
        (11..=11, &"f"),
        (9..=9, &"f"),
        (9..=9, &"f"),
        (8..=8, &"f"),
        (6..=6, &"f"),
        (4..=4, &"f"),
        (5..=5, &"f"),
        (3..=3, &"f"),
        (2..=2, &"f"),
        (1..=1, &"f"),
        (1..=1, &"f"),
        (1..=1, &"f"),
    ]);

    let not_b = b.complement_with(&"A");
    let not_c = c.complement_with(&"A");
    let not_d = d.complement_with(&"A");
    let not_e = e.complement_with(&"A");
    let not_f = f.complement_with(&"A");
    assert!(not_a.range_values().equal(not_b));
    assert!(not_a.range_values().equal(not_c));
    assert!(not_a.range_values().equal(not_d));
    assert!(not_a.range_values().equal(not_e));
    assert!(not_a.range_values().equal(not_f));
    Ok(())
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
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
        .complement_with(&"a0")
        .union(a12.range_values());
    let d = a0
        .range_values()
        .union(a1.range_values())
        .union(a2.range_values());
    let e = a0_tee.union(a12.range_values());

    let f = UnionIterMap::from_iter(a0.range_values())
        .union(UnionIterMap::from_iter(a1.range_values()))
        .union(UnionIterMap::from_iter(a2.range_values()));
    assert!(a.range_values().equal(b));
    assert!(a.range_values().equal(c));
    assert!(a.range_values().equal(d));
    assert!(a.range_values().equal(e));
    assert!(a.range_values().equal(f));
    Ok(())
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
fn map_sub() -> Result<(), Box<dyn std::error::Error>> {
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
    let e = a01_tee.map_and_set_difference(a2.ranges());
    assert!(a.range_values().equal(b));
    assert!(a.ranges().equal(c));
    assert!(a.range_values().equal(d));
    assert!(a.range_values().equal(e));

    Ok(())
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
fn map_xor() -> Result<(), Box<dyn std::error::Error>> {
    // RangeMapBlaze, RangesIter, NotIter, UnionIterMap, Tee, UnionIterMap(g)
    let a0 = RangeMapBlaze::from_iter([(1..=6, "a0")]);
    let a1 = RangeMapBlaze::from_iter([(8..=9, "a1")]);
    let a2 = RangeMapBlaze::from_iter([(11..=15, "a2")]);

    let a01 = &a0 | &a1;
    let a01_tee = a01.range_values(); // with range instead of range values used 'tee' here
    let not_a01 = &a01.complement_with(&"not_a01");
    let a = &a01 ^ &a2;
    let b = a01.range_values() ^ a2.range_values();
    let not_a01_complement = not_a01.complement_with(&"a1");
    let c = not_a01_complement.range_values() ^ a2.range_values();
    let d = (a0.range_values() | a1.range_values()) ^ a2.range_values();
    let e = a01_tee.symmetric_difference(a2.range_values());
    let f =
        UnionIterMap::from_iter(a01.range_values()) ^ UnionIterMap::from_iter(a2.range_values());
    assert!(a.range_values().equal(b));
    assert_eq!(
        c.into_string(),
        r#"(1..=6, "a1"), (8..=9, "a1"), (11..=15, "a2")"#
    );
    assert!(a.range_values().equal(d));
    assert!(a.range_values().equal(e));
    assert!(a.range_values().equal(f));
    Ok(())
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
fn map_bitand() -> Result<(), Box<dyn std::error::Error>> {
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
    let e = a01_tee.map_and_set_intersection(a2.ranges());
    assert!(a.range_values().equal(b));
    assert!(a.ranges().equal(c));
    assert!(a.range_values().equal(d));
    assert!(a.range_values().equal(e));
    Ok(())
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
#[allow(clippy::zero_repeat_side_effects)]
fn map_empty_it() {
    use std::ops::BitOr;

    let universe0 = RangeMapBlaze::from_iter([(0u8..=255, "Universe")]);
    let universe = universe0.range_values();
    let arr: [(u8, &str); 0] = [];
    let a0 = RangeMapBlaze::<u8, &str>::from_iter(arr);
    assert!(!(a0.ranges()).equal(universe0.ranges()));
    assert!(
        (a0.complement_with(&"Universe"))
            .range_values()
            .equal(universe)
    );
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

    // don't run panic-related code on wasm
    #[cfg(not(target_arch = "wasm32"))]
    {
        use core::panic::AssertUnwindSafe;
        use std::panic;

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
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
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
    assert_eq!(a.len(), UIntPlusOne::MaxPlusOne);
    let a = !RangeMapBlaze::from_iter([(1u128..=0, "a")]);
    println!("tc1 '{a}', {}", a.len());
    assert_eq!(a.len(), UIntPlusOne::MaxPlusOne);
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
fn map_tricky_case2() {
    let _a = RangeMapBlaze::from_iter([(-1..=i128::MAX, "a")]);
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
fn map_tricky_case3() {
    let _a = RangeMapBlaze::from_iter([(0..=u128::MAX, "a")]);
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
fn map_constructors() -> Result<(), Box<dyn std::error::Error>> {
    // use range_set_blaze::Priority;

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
    //#13 collect / from_iter range
    _range_map_blaze = [(5..=6, "a"), (1..=5, "b")].into_iter().collect();
    _range_map_blaze = RangeMapBlaze::from_iter([(5..=6, "a"), (1..=5, "b")]);
    // #16 into / from iter (T,T) + SortedDisjoint
    _range_map_blaze = _range_map_blaze.range_values().into_range_map_blaze();
    _range_map_blaze = RangeMapBlaze::from_sorted_disjoint_map(_range_map_blaze.range_values());

    let mut _sorted_disjoint_iter: UnionIterMap<_, _, _> =
        _range_map_blaze.range_values().collect();
    _sorted_disjoint_iter = UnionIterMap::from_iter(_range_map_blaze.range_values());

    Ok(())
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
fn map_doc_test_insert1() {
    let mut map = RangeMapBlaze::new();

    assert_eq!(map.insert(2, "a"), None);
    assert_eq!(map.insert(2, "b"), Some("a"));
    assert_eq!(map.len(), 1u64);
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
fn map_doc_test_len() {
    let mut v = RangeMapBlaze::new();
    assert_eq!(v.len(), 0u64);
    v.insert(1, "Hello");
    assert_eq!(v.len(), 1u64);

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
        UIntPlusOne::UInt(340282366920938463463374607431768211455)
    );
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
fn map_test_pops() {
    // Initialize the map with ranges as keys and chars as values
    let mut map = RangeMapBlaze::from_iter([(1..=2, 'a'), (4..=5, 'b'), (10..=11, 'c')]);
    let len = map.len();

    // Adjusted to expect a tuple of (single integer key, value)
    assert_eq!(map.pop_first(), Some((1, 'a')));
    assert_eq!(map.len(), len - 1u64);
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
    assert_eq!(map.len(), len - 2u64);

    // Continue popping and assert changes
    assert_eq!(map.pop_last(), Some((10, 'c'))); // Pop the last remaining element of the previous last range
    assert_eq!(map.len(), len - 3u64);
    assert_eq!(map, RangeMapBlaze::from_iter([(2..=2, 'a'), (4..=5, 'b')]));

    // Now pop the first element after previous pops, which should be 2 from the adjusted range
    assert_eq!(map.pop_first(), Some((2, 'a')));
    assert_eq!(map.len(), len - 4u64);
    assert_eq!(map, RangeMapBlaze::from_iter([(4..=5, 'b')]));

    // Finally, pop the last elements left in the map
    assert_eq!(map.pop_first(), Some((4, 'b')));
    assert_eq!(map.pop_last(), Some((5, 'b')));
    assert!(map.is_empty());
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
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
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
fn map_remove() {
    let mut map: RangeMapBlaze<i32, char> = RangeMapBlaze::new();
    assert_eq!(map.remove(4), None);

    // Initialize RangeMapBlaze with char values for simplicity
    let mut map = RangeMapBlaze::from_iter([(1..=2, 'a'), (4..=5, 'b'), (10..=11, 'c')]);
    let len = map.len();

    // Assume remove affects only a single key and returns true if the key was found and removed
    assert_eq!(map.remove(4), Some('b')); // Removing a key within a range may adjust the range
    assert_eq!(map.len(), len - 1u64);
    // The range 4..=5 with 'b' is adjusted to 5..=5 after removing 4
    assert_eq!(
        map,
        RangeMapBlaze::from_iter([(1..=2, 'a'), (5..=5, 'b'), (10..=11, 'c'),])
    );
    assert_eq!(map.remove(5), Some('b'));

    assert_eq!(map.len(), len - 2u64);
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
    let len = map.len();
    assert_eq!(map.remove(0), None);
    assert_eq!(map.len(), len);
    assert_eq!(map.remove(3), None);
    assert_eq!(map.len(), len);
    assert_eq!(map.remove(2), Some('a'));
    assert_eq!(map.len(), len - 1u64);
    assert_eq!(map.remove(1000), Some('d'));
    assert_eq!(map.len(), len - 2u64);
    assert_eq!(map.remove(10), Some('c'));
    assert_eq!(map.len(), len - 3u64);
    assert_eq!(map.remove(50), Some('c'));
    assert_eq!(map.len(), len - 4u64);
    assert_eq!(
        map,
        RangeMapBlaze::from_iter([(1..=1, 'a'), (4..=5, 'b'), (11..=49, 'c'), (51..=100, 'c'),])
    );
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
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
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
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

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
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

    // Applying multiple operators
    let result0 = &a - (&b | &c); // Creates an intermediate 'RangeMapBlaze'.

    // Alternatively, we can use the 'SortedDisjointMap' API and avoid the intermediate 'RangeMapBlaze'.
    let result1 = RangeMapBlaze::from_sorted_disjoint_map(
        a.range_values() - (b.range_values() | c.range_values()),
    );
    assert_eq!(result0.to_string(), r#"(1..=1, "one")"#);
    assert_eq!(result0, result1);

    let a = RangeMapBlaze::from_iter([(1..=2, "a"), (5..=100, "a")]);
    let b = RangeMapBlaze::from_iter([(2..=6, "b")]);

    // Symmetric difference of two 'RangeMapBlaze's.
    let result = &a ^ &b; // Alternatively, 'a ^ b'.
    assert_eq!(
        result.to_string(),
        r#"(1..=1, "a"), (3..=4, "b"), (7..=100, "a")"#
    );
}

pub fn play_movie(frames: RangeMapBlaze<i32, String>, fps: i32, skip_sleep: bool) {
    assert!(fps > 0, "fps must be positive");
    // Later: could look for missing frames
    let sleep_duration = Duration::from_secs(1) / fps as u32;
    // For every frame index (index) from 0 to the largest index in the frames ...
    for index in 0..=frames.ranges().into_range_set_blaze().last().unwrap() {
        // Look up the frame at that index (panic if none exists)
        let frame = frames.get(index).unwrap_or_else(|| {
            panic!("frame {index} not found");
        });
        // Clear the line and return the cursor to the beginning of the line
        print!("\x1B[2K\r{frame}");
        stdout().flush().unwrap(); // Flush stdout to ensure the output is displayed
        if !skip_sleep {
            sleep(sleep_duration);
        }
    }
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
        .map(|(range, value)| {
            let (start, end) = range.clone().into_inner();
            let mut a = (start - first) * scale.abs() + first;
            let mut b = (end + 1 - first) * scale.abs() + first - 1;
            let last = (last + 1 - first) * scale.abs() + first - 1;
            if scale < 0 {
                (a, b) = (last - b + first, last - a + first);
            }
            let new_range = a + shift..=b + shift;
            (new_range, value.clone())
        })
        .collect()
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
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

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
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

    let a: RangeMapBlaze<i32, &str> = RangeMapBlaze::from_iter([(0..=0, "a")]);
    let mut b: RangeValuesIter<i32, &str> = a.range_values();
    let _c: &&str = b.next().unwrap().1;
    let mut b: IntoRangeValuesIter<i32, &str> = a.into_range_values();
    let _c: &&str = b.next().unwrap().1.borrow();

    let a: RangeMapBlaze<i32, String> = RangeMapBlaze::from_iter([(0..=0, "a".to_string())]);
    let mut b: RangeValuesIter<i32, String> = a.range_values();
    let _c: &String = b.next().unwrap().1;
    let mut b: IntoRangeValuesIter<i32, String> = a.into_range_values();
    let _c: &String = b.next().unwrap().1.borrow();

    // You can get all the same types via CheckSortedDisjointMap, but values are always (clonable) references.
    let a_string = "a".to_string();
    let mut b: CheckSortedDisjointMap<i32, &String, _> =
        CheckSortedDisjointMap::new([(0..=0, &a_string)]);
    let c: &String = b.next().unwrap().1;
    let _c_clone: String = c.clone();
    let _: CheckSortedDisjointMap<i32, &&String, _> =
        CheckSortedDisjointMap::new([(0..=0, &&a_string)]);
    let _: CheckSortedDisjointMap<i32, &String, _> =
        CheckSortedDisjointMap::new([(0..=0, &"a".to_string())]);
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
fn test_every_sorted_disjoint_map_method() {
    use syntactic_for::syntactic_for;

    let e0: RangeMapBlaze<i32, &str> = RangeMapBlaze::from_iter([(1..=2, "a"), (5..=100, "a")]);

    macro_rules! fresh_instances {
        () => {{
            let a: CheckSortedDisjointMap<i32, &&str, _> =
                CheckSortedDisjointMap::new([(1..=2, &"a"), (5..=100, &"a")]);
            let b: DynSortedDisjointMap<i32, &&str> =
                DynSortedDisjointMap::new(CheckSortedDisjointMap::new([
                    (1..=2, &"a"),
                    (5..=100, &"a"),
                ]));
            let c: IntersectionIterMap<i32, &&str, _, _> = [CheckSortedDisjointMap::new([
                (1..=2, &"a"),
                (5..=100, &"a"),
            ])]
            .intersection();
            let d: IntoRangeValuesIter<i32, &str> = e0.clone().into_range_values();
            let e: RangeValuesIter<i32, &str> = e0.range_values();
            let f: SymDiffIterMap<i32, &&str, _> = [CheckSortedDisjointMap::new([
                (1..=2, &"a"),
                (5..=100, &"a"),
            ])]
            .symmetric_difference();
            let g: UnionIterMap<i32, &&str, _> = [CheckSortedDisjointMap::new([
                (1..=2, &"a"),
                (5..=100, &"a"),
            ])]
            .union();

            (a, b, c, d, e, f, g)
        }};
    }

    // check for SortedDisjointMap and FuseIterator traits
    let (a, b, c, d, e, f, g) = fresh_instances!();
    syntactic_for! { sd in [a, b, c, d, e, f, g] {$(
        is_sorted_disjoint_map::<_,_,_>($sd);
    )*}}
    fn is_fused<T: FusedIterator>(_: T) {}
    let (a, b, c, d, e, f, g) = fresh_instances!();
    syntactic_for! { sd in [a, b, c, d, e, f, g] {$(
        is_fused::<_>($sd);
    )*}}
    fn is_sorted_disjoint_map<T, VR, S>(_iter: S)
    where
        T: Integer,
        VR: ValueRef,
        S: SortedDisjointMap<T, VR>,
    {
    }

    // Complement
    let (a, b, c, d, e, f, g) = fresh_instances!();
    syntactic_for! { sd in [a,b,c,d,e,f,g] {$(
        let z = ! $sd;
        assert!(z.equal(CheckSortedDisjoint::new([-2147483648..=0, 3..=4, 101..=2147483647])));
    )*}}

    // Union
    let (a, b, c, d, e, f, g) = fresh_instances!();
    syntactic_for! { sd in [a, b, c, e, f, g] {$(
        let z: CheckSortedDisjointMap<i32, &&str, _> = CheckSortedDisjointMap::new([(-1..=0,&"z"), (50..=50, &"z"),(1000..=10_000,&"z")]);
        let z = $sd | z;
        assert!(z.equal(CheckSortedDisjointMap::new([(-1..=0, &"z"), (1..=2, &"a"), (5..=100, &"a"), (1000..=10000, &"z")])));
    )*}}
    let z: CheckSortedDisjointMap<i32, Rc<&str>, _> = CheckSortedDisjointMap::new([
        (-1..=0, Rc::new("z")),
        (50..=50, Rc::new("z")),
        (1000..=10_000, Rc::new("z")),
    ]);
    let z = d | z;
    assert!(z.equal(CheckSortedDisjointMap::new([
        (-1..=0, Rc::new("z")),
        (1..=2, Rc::new("a")),
        (5..=100, Rc::new("a")),
        (1000..=10000, Rc::new("z"))
    ])));

    // Intersection
    let (a, b, c, d, e, f, g) = fresh_instances!();
    syntactic_for! { sd in [a, b, c, e, f, g] {$(
        let z: CheckSortedDisjointMap<i32, &&str, _> = CheckSortedDisjointMap::new([(-1..=0,&"z"), (50..=50, &"z"),(1000..=10_000,&"z")]);
        let z = $sd & z;
        // println!("{}", z.into_string());
        assert!(z.equal(CheckSortedDisjointMap::new([(50..=50, &"a")])));
    )*}}
    let z: CheckSortedDisjointMap<i32, Rc<&str>, _> = CheckSortedDisjointMap::new([
        (-1..=0, Rc::new("z")),
        (50..=50, Rc::new("z")),
        (1000..=10_000, Rc::new("z")),
    ]);
    let z = d & z;
    assert!(z.equal(CheckSortedDisjointMap::new([(50..=50, Rc::new("a"))])));

    // Symmetric Difference
    let (a, b, c, d, e, f, g) = fresh_instances!();
    syntactic_for! { sd in [a, b, c, e,f,g] {$(
        let z: CheckSortedDisjointMap<i32, &&str, _> = CheckSortedDisjointMap::new([(-1..=0,&"z"), (50..=50, &"z"),(1000..=10_000,&"z")]);
        let z = $sd ^ z;
        // println!("a {}", z.into_string());
        assert!(z.equal(CheckSortedDisjointMap::new([(-1..=0, &"z"), (1..=2, &"a"), (5..=49, &"a"), (51..=100, &"a"), (1000..=10000, &"z")])));
    )*}}
    let z: CheckSortedDisjointMap<i32, Rc<&str>, _> = CheckSortedDisjointMap::new([
        (-1..=0, Rc::new("z")),
        (50..=50, Rc::new("z")),
        (1000..=10_000, Rc::new("z")),
    ]);
    let z = d ^ z;
    assert!(z.equal(CheckSortedDisjointMap::new([
        (-1..=0, Rc::new("z")),
        (1..=2, Rc::new("a")),
        (5..=49, Rc::new("a")),
        (51..=100, Rc::new("a")),
        (1000..=10_000, Rc::new("z"))
    ])));

    // set difference
    let (a, b, c, d, e, f, g) = fresh_instances!();
    syntactic_for! { sd in [a, b, c,  e,f,g] {$(
        let z: CheckSortedDisjointMap<i32, &&str, _> = CheckSortedDisjointMap::new([(-1..=0,&"z"), (50..=50, &"z"),(1000..=10_000,&"z")]);
        let z = $sd - z;
        // println!("c {}", z.into_string());
        assert!(z.equal(CheckSortedDisjointMap::new([(1..=2, &"a"), (5..=49, &"a"), (51..=100, &"a")])));
    )*}}
    let z: CheckSortedDisjointMap<i32, Rc<&str>, _> = CheckSortedDisjointMap::new([
        (-1..=0, Rc::new("z")),
        (50..=50, Rc::new("z")),
        (1000..=10_000, Rc::new("z")),
    ]);
    let z = d - z;
    assert!(z.equal(CheckSortedDisjointMap::new([
        (1..=2, Rc::new("a")),
        (5..=49, Rc::new("a")),
        (51..=100, Rc::new("a")),
    ])));
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
fn map_empty_construction() {
    use range_set_blaze::RangeMapBlaze;
    use std::collections::BTreeMap;

    // Collect from an empty iterator of (key, value) pairs
    let collected: RangeMapBlaze<u8, char> = [].iter().cloned::<(u8, char)>().collect();
    assert!(collected.is_empty());

    // Collect from an empty iterator of (range, value) pairs
    let collected_ranges: RangeMapBlaze<u8, char> =
        Vec::<(std::ops::RangeInclusive<u8>, char)>::new()
            .into_iter()
            .collect();
    assert!(collected_ranges.is_empty());

    // Construct via .new()
    let via_new = RangeMapBlaze::<u8, char>::new();
    assert!(via_new.is_empty());

    // Construct via from_iter with no elements
    let from_iter = RangeMapBlaze::<u8, char>::from_iter(std::iter::empty::<(u8, char)>());
    assert!(from_iter.is_empty());

    // Empty vs BTreeMap comparison
    let empty_btree: BTreeMap<u8, &char> = BTreeMap::new();
    assert!(equal_maps(&collected, &empty_btree));
    assert!(equal_maps(&collected_ranges, &empty_btree));
    assert!(equal_maps(&via_new, &empty_btree));
    assert!(equal_maps(&from_iter, &empty_btree));

    // .ranges() should also be empty
    assert_eq!(via_new.ranges().next(), None);
    assert_eq!(collected.ranges().next(), None);
    assert_eq!(from_iter.ranges().next(), None);
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
fn map_random_from_iter_item() {
    for seed in 0..20 {
        println!("seed: {seed}");
        let mut rng = StdRng::seed_from_u64(seed);

        let mut btree_map = BTreeMap::new();

        let mut inputs = Vec::new();
        for _ in 0..500 {
            let key = rng.random_range(0..=255u8);
            let value = ['a', 'b', 'c'].as_slice().choose(&mut rng).unwrap();

            print!("{key}{value} ");
            inputs.push((key, value));

            let range_map_blaze = inputs.iter().collect();
            // Only insert if the key does not already exist
            btree_map.entry(key).or_insert(value);
            if !equal_maps(&range_map_blaze, &btree_map) {
                println!();
                let _range_map_blaze: RangeMapBlaze<_, char> = inputs.iter().collect();
                panic!();
            }
        }
    }
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
fn map_random_from_iter_range() {
    let empty: RangeMapBlaze<u8, char> = [].iter().cloned::<(u8, char)>().collect();
    assert!(empty.is_empty());

    for seed in 0..20 {
        println!("seed: {seed}");
        let mut rng = StdRng::seed_from_u64(seed);

        let mut btree_map = BTreeMap::new();

        let mut inputs = Vec::new();
        for _ in 0..500 {
            let start = rng.random_range(0..=255u8);
            let end = rng.random_range(start..=255u8);
            let key = start..=end;
            let value = ['a', 'b', 'c'].as_slice().choose(&mut rng).unwrap();

            // print!("{key}{value} ");
            inputs.push((key.clone(), value));

            let range_map_blaze = inputs.iter().collect();
            for k in key.clone() {
                btree_map.entry(k).or_insert(value);
            }
            if !equal_maps(&range_map_blaze, &btree_map) {
                let _range_map_blaze: RangeMapBlaze<u8, char> = inputs.iter().collect();
                panic!();
            }
        }
    }
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
fn map_random_insert() {
    for seed in 0..20 {
        println!("seed: {seed}");
        let mut rng = StdRng::seed_from_u64(seed);

        let mut btree_map = BTreeMap::new();
        let mut range_map_blaze = RangeMapBlaze::new();
        let mut inputs = Vec::new();

        for _ in 0..500 {
            let key = rng.random_range(0..=255u8);
            let value = ["aaa", "bbb", "ccc"].as_slice().choose(&mut rng).unwrap();
            // print!("{key}{value} ");

            btree_map.insert(key, value);
            range_map_blaze.insert(key, *value);
            if equal_maps(&range_map_blaze, &btree_map) {
                inputs.push((key, value));
                continue;
            }

            // if range_map_blaze and btree_map are not equal, then we have a bug, so repro it:

            let mut range_map_blaze = RangeMapBlaze::from_iter(inputs.clone());
            range_map_blaze.insert(key, *value);
            assert!(equal_maps(&range_map_blaze, &btree_map));
        }
    }
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
fn map_random_insert_range() {
    for seed in 0..20 {
        println!("seed: {seed}");
        let mut rng = StdRng::seed_from_u64(seed);

        let mut btree_map = BTreeMap::new();
        let mut range_map_blaze = RangeMapBlaze::new();
        let mut inputs = Vec::new();

        for _ in 0..500 {
            let start = rng.random_range(0..=255u8);
            let end = rng.random_range(start..=255u8);
            let key = start..=end;
            let value = ["aaa", "bbb", "ccc"].as_slice().choose(&mut rng).unwrap();
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

            let mut range_map_blaze = RangeMapBlaze::from_iter(inputs.clone());
            println!("{range_map_blaze}");
            println!("About to insert {}..={} -> {value}", key.start(), key.end());
            range_map_blaze.ranges_insert(key.clone(), *value);
            assert!(equal_maps(&range_map_blaze, &btree_map));
        }
    }
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
fn map_random_ranges() {
    let values = ['a', 'b', 'c'].as_slice();
    for seed in 0..20 {
        println!("seed: {seed}");
        let mut rng = StdRng::seed_from_u64(seed);

        let mut range_set_blaze = RangeSetBlaze::new();
        let mut range_map_blaze = RangeMapBlaze::new();
        let mut inputs = Vec::<(u8, &char)>::new();

        for _ in 0..500 {
            let key = rng.random_range(0..=255u8);
            let value = values.choose(&mut rng).unwrap();
            // print!("{key}{value} ");

            range_set_blaze.insert(key);
            range_map_blaze.insert(key, *value);
            if range_set_blaze.ranges().eq(range_map_blaze.ranges()) {
                inputs.push((key, value));
                continue;
            }

            // if range_map_blaze and btree_map are not equal, then we have a bug, so repro it:

            let mut range_map_blaze = RangeMapBlaze::from_iter(inputs.clone());
            range_map_blaze.insert(key, *value);
            assert!(range_set_blaze.ranges().eq(range_map_blaze.ranges()));
        }
    }
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
fn map_random_ranges_ranges() {
    let values = ['a', 'b', 'c'];
    for seed in 0..20 {
        println!("seed: {seed}");
        let mut rng = StdRng::seed_from_u64(seed);

        let mut range_set_blaze = RangeSetBlaze::new();
        let mut range_map_blaze = RangeMapBlaze::new();
        let mut inputs = Vec::new();

        for _ in 0..500 {
            let start = rng.random_range(0..=255u8);
            let end = rng.random_range(start..=255u8);
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

            let mut range_map_blaze = RangeMapBlaze::from_iter(inputs.clone());
            range_map_blaze.ranges_insert(key.clone(), *value);
            assert!(range_set_blaze.ranges().eq(range_map_blaze.ranges()));
        }
    }
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
fn map_random_intersection() {
    let values = ['a', 'b', 'c'].as_slice();
    for seed in 0..20 {
        println!("seed: {seed}");
        let mut rng = StdRng::seed_from_u64(seed);

        let mut set0 = RangeSetBlaze::new();
        let mut map0 = RangeMapBlaze::new();
        // let mut inputs = Vec::<(u8, &char)>::new();

        for _ in 0..500 {
            let element = rng.random_range(0..=255u8);
            let key = rng.random_range(0..=255u8);
            let value = values.choose(&mut rng).unwrap();
            // print!("{element},{key}{value} ");

            set0.insert(element);
            map0.insert(key, *value);

            let intersection = map0.range_values().map_and_set_intersection(set0.ranges());

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
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
fn map_tiny_symmetric_difference0() {
    use range_set_blaze::IntoString;

    let mut map0 = RangeMapBlaze::new();
    map0.insert(84, 'c');
    map0.insert(85, 'c');
    let mut map1 = RangeMapBlaze::new();
    map1.insert(85, 'a');
    let symmetric_difference = map0
        .range_values()
        .symmetric_difference(map1.range_values());
    assert_eq!(symmetric_difference.into_string(), "(84..=84, 'c')");
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
fn map_tiny_symmetric_difference1() {
    use range_set_blaze::IntoString;

    let mut map0 = RangeMapBlaze::new();
    map0.insert(187, 'a');
    map0.insert(188, 'a');
    map0.insert(189, 'a');
    let mut map1 = RangeMapBlaze::new();
    map1.insert(187, 'b');
    map1.insert(189, 'c');
    let symmetric_difference = map0
        .range_values()
        .symmetric_difference(map1.range_values());
    assert_eq!(symmetric_difference.into_string(), "(188..=188, 'a')");
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
fn map_random_symmetric_difference() {
    let values = ['a', 'b', 'c'].as_slice();
    for seed in 0..20 {
        println!("seed: {seed}");
        let mut rng = StdRng::seed_from_u64(seed);

        let mut map0 = RangeMapBlaze::new();
        let mut map1 = RangeMapBlaze::new();
        // let mut inputs = Vec::<(u8, &char)>::new();

        for _ in 0..500 {
            let key = rng.random_range(0..=255u8);
            let value = values.choose(&mut rng).unwrap();
            map0.insert(key, *value);
            print!("l{key}{value} ");
            let key = rng.random_range(0..=255u8);
            let value = values.choose(&mut rng).unwrap();
            map1.insert(key, *value);
            print!("r{key}{value} ");

            let symmetric_difference = map0
                .range_values()
                .symmetric_difference(map1.range_values());

            // println!(
            //     "left ^ right = {}",
            //     SymDiffIterMap::new2(map0.range_values(), map1.range_values()).into_string()
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
                            println!("left: {map0}");
                            println!("right: {map1}");
                            let s_d = map0
                                .range_values()
                                .symmetric_difference(map1.range_values())
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
                println!("left: {map0}");
                println!("right: {map1}");
                let s_d = map0
                    .range_values()
                    .symmetric_difference(map1.range_values())
                    .into_range_map_blaze();
                println!("left ^ right = {s_d}");
                panic!("expected_keys should be empty: {expected_keys}");
            }
        }
    }
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
fn map_repro_insert_1() {
    let mut range_map_blaze = RangeMapBlaze::new();
    range_map_blaze.insert(123, "Hello");
    range_map_blaze.insert(123, "World");
    assert_eq!(range_map_blaze.to_string(), r#"(123..=123, "World")"#);
}

fn equal_maps<T: Integer + std::fmt::Display, V: Eq + Clone + fmt::Debug + std::fmt::Display>(
    range_map_blaze: &RangeMapBlaze<T, V>,
    btree_map: &BTreeMap<T, &V>,
) -> bool
where
    usize: std::convert::From<<T as Integer>::SafeLen>,
{
    // Also checks that the ranges are really sorted and disjoint
    let mut previous: Option<(RangeInclusive<T>, &V)> = None;
    for range_value in range_map_blaze.range_values() {
        let v = range_value.1;
        let range = range_value.0.clone();

        if let Some(previous) = previous {
            if (previous.1 == v && (*previous.0.end()).add_one() >= *range.start())
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
            if btree_map.get(&k).is_none_or(|v2| v != *v2) {
                eprintln!(
                    "range_map_blaze contains {k} -> {v}, btree_map contains {k} -> {:?}",
                    btree_map.get(&k)
                );
                return false;
            }
            if k == *range.end() {
                break;
            }
            k = k.add_one();
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

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
fn map_repro_123() {
    let input = [(123, 'a'), (123, 'b')];

    let range_map_blaze = RangeMapBlaze::<u8, char>::from_iter(input);
    assert_eq!(range_map_blaze.to_string(), "(123..=123, 'a')");
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
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
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
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
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
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
#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
fn test_coverage_0() {
    let a = RangeMapBlaze::from_iter([(1..=2, "Hello"), (3..=4, "World")]);
    let d = DynSortedDisjointMap::new(a.range_values());
    assert_eq!(d.size_hint(), a.range_values().size_hint());
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
fn test_coverage_1() {
    let a = RangeMapBlaze::from_iter([(1..=2, "Hello"), (3..=4, "World")]);
    let mut i = a.iter();
    assert_eq!(i.next_back(), Some((4, &"World")));
    assert_eq!(i.next_back(), Some((3, &"World")));
    assert_eq!(i.next_back(), Some((2, &"Hello")));
    assert_eq!(i.next_back(), Some((1, &"Hello")));
    assert_eq!(i.next_back(), None);
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
fn test_coverage_2() {
    let a = RangeMapBlaze::from_iter([(1..=2, "Hello"), (3..=4, "World")]);
    let mut i = a.into_iter();
    assert_eq!(i.size_hint(), (2, None));
    assert_eq!(i.next(), Some((1, "Hello")));
    assert_eq!(i.next(), Some((2, "Hello")));
    assert_eq!(i.next(), Some((3, "World")));
    assert_eq!(i.next(), Some((4, "World")));
    assert_eq!(i.next(), None);

    let a = RangeMapBlaze::from_iter([(1..=2, "Hello"), (3..=4, "World")]);
    let mut i = a.into_iter();
    assert_eq!(i.next_back(), Some((4, "World")));
    assert_eq!(i.next_back(), Some((3, "World")));
    assert_eq!(i.next_back(), Some((2, "Hello")));
    assert_eq!(i.next_back(), Some((1, "Hello")));
    assert_eq!(i.next_back(), None);

    let a = RangeMapBlaze::from_iter([(1..=2, "Hello"), (3..=4, "World")]);
    let mut i = a.keys();
    assert_eq!(i.size_hint(), (2, None));
    assert_eq!(i.next(), Some(1));
    assert_eq!(i.next(), Some(2));
    assert_eq!(i.next(), Some(3));
    assert_eq!(i.next(), Some(4));
    assert_eq!(i.next(), None);

    let mut i = a.keys();
    assert_eq!(i.next_back(), Some(4));
    assert_eq!(i.next_back(), Some(3));
    assert_eq!(i.next_back(), Some(2));
    assert_eq!(i.next_back(), Some(1));
    assert_eq!(i.next_back(), None);

    // test into_keys
    let a = RangeMapBlaze::from_iter([(1..=2, "Hello"), (3..=4, "World")]);
    let mut i = a.into_keys();
    assert_eq!(i.size_hint(), (2, None));
    assert_eq!(i.next(), Some(1));
    assert_eq!(i.next(), Some(2));
    assert_eq!(i.next(), Some(3));
    assert_eq!(i.next(), Some(4));
    assert_eq!(i.next(), None);

    let a = RangeMapBlaze::from_iter([(1..=2, "Hello"), (3..=4, "World")]);
    let mut i = a.into_keys();
    assert_eq!(i.next_back(), Some(4));
    assert_eq!(i.next_back(), Some(3));
    assert_eq!(i.next_back(), Some(2));
    assert_eq!(i.next_back(), Some(1));
    assert_eq!(i.next_back(), None);

    // Test values
    let a = RangeMapBlaze::from_iter([(1..=2, "Hello"), (3..=4, "World")]);
    let mut i = a.values();
    assert_eq!(i.size_hint(), (2, None));
    assert_eq!(i.next(), Some(&"Hello"));
    assert_eq!(i.next(), Some(&"Hello"));
    assert_eq!(i.next(), Some(&"World"));
    assert_eq!(i.next(), Some(&"World"));
    assert_eq!(i.next(), None);

    let mut i = a.values();
    assert_eq!(i.next_back(), Some(&"World"));
    assert_eq!(i.next_back(), Some(&"World"));
    assert_eq!(i.next_back(), Some(&"Hello"));
    assert_eq!(i.next_back(), Some(&"Hello"));
    assert_eq!(i.next_back(), None);

    // Test into_values
    let a = RangeMapBlaze::from_iter([(1..=2, "Hello"), (3..=4, "World")]);
    let mut i = a.into_values();
    assert_eq!(i.size_hint(), (2, None));
    assert_eq!(i.next(), Some("Hello"));
    assert_eq!(i.next(), Some("Hello"));
    assert_eq!(i.next(), Some("World"));
    assert_eq!(i.next(), Some("World"));
    assert_eq!(i.next(), None);

    let a = RangeMapBlaze::from_iter([(1..=2, "Hello"), (3..=4, "World")]);
    let mut i = a.into_values();
    assert_eq!(i.next_back(), Some("World"));
    assert_eq!(i.next_back(), Some("World"));
    assert_eq!(i.next_back(), Some("Hello"));
    assert_eq!(i.next_back(), Some("Hello"));
    assert_eq!(i.next_back(), None);
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
fn test_coverage_4() {
    let a = RangeMapBlaze::from_iter([(1u128..=4, "Hello")]);
    a.get(u128::MAX);
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
fn test_coverage_5() {
    let mut a = RangeMapBlaze::from_iter([(1u128..=4, "Hello")]);
    a.remove(u128::MAX);
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
fn test_coverage_6() {
    let mut a = RangeMapBlaze::from_iter([(1u128..=4, "Hello")]);
    let _ = a.split_off(u128::MAX);
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
fn test_coverage_10() {
    let mut a = RangeMapBlaze::from_iter([(1..=2, "Hello"), (3..=4, "World")]);
    assert_eq!(a.pop_last(), Some((4, "World")));
    assert_eq!(a.pop_last(), Some((3, "World")));
    assert_eq!(a.pop_last(), Some((2, "Hello")));
    assert_eq!(a.pop_last(), Some((1, "Hello")));
    assert_eq!(a.pop_last(), None);
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
fn example_2() {
    use range_set_blaze::prelude::*;

    // frames per second
    let fps = 24;
    // Create a countdown from 5 to 2
    let count_down: RangeMapBlaze<usize, String> = (2..=5)
        .rev()
        .enumerate()
        .map(|(i, c)| ((i * fps)..=((i + 1) * fps) - 1, c.to_string()))
        .collect();
    // At 5 and 8 seconds (respectively), display "Hello" and "World"
    let hello_world: RangeMapBlaze<usize, String> = RangeMapBlaze::from_iter([
        ((5 * fps)..=(7 * fps - 1), "Hello".to_string()),
        ((8 * fps)..=(10 * fps - 1), "World".to_string()),
    ]);
    // create 10 seconds of blank frames
    let blank = RangeMapBlaze::from_iter([(0..=10 * fps - 1, "".to_string())]);
    // union everything together with left-to-right precedence
    let animation = [count_down, hello_world, blank].union();
    // for every range of frames, show what is displayed
    println!("frames: text");
    for (range, text) in animation.range_values() {
        println!("{range:?}: {text}");
    }
}

#[cfg(not(target_arch = "wasm32"))]
#[quickcheck]
fn extend(mut a: BTreeMap<i8, u8>, b: Vec<(i8, u8)>) -> bool {
    let mut a_r: RangeMapBlaze<_, _> = a.clone().into_iter().collect();
    a.extend(b.clone().into_iter());
    a_r.extend(b.into_iter());
    a_r == a.into_iter().collect()
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
#[allow(clippy::reversed_empty_ranges)]
#[should_panic = "start (inclusive) must be less than or equal to end (inclusive)"]
fn test_range_method_on_range_map_blaze_panic0() {
    let map = RangeMapBlaze::<i32, &str>::from_iter([(1..=3, "a"), (4..=6, "b")]);
    let _a: RangeMapBlaze<i32, &str> = map.range(3..2).collect();
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
#[should_panic = "inclusive start must be <= T::max_safe_value()"]
fn test_range_method_on_range_map_blaze_panic1() {
    let map = RangeMapBlaze::<u8, &str>::from_iter([(1u8..=3, "a"), (4..=6, "b")]);
    let _a: RangeMapBlaze<u8, &str> = map
        .range((Bound::Excluded(255), Bound::Included(255)))
        .collect();
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
#[should_panic = "inclusive end must be >= T::min_value()"]
fn test_range_method_on_range_map_blaze_panic2() {
    let map = RangeMapBlaze::<u8, &str>::from_iter([(1u8..=3, "a"), (4..=6, "b")]);
    let _a: RangeMapBlaze<u8, &str> = map
        .range((Bound::Included(0), Bound::Excluded(0)))
        .collect();
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
fn test_index() {
    let map = RangeMapBlaze::<i32, &str>::from_iter([(1..=3, "a"), (4..=6, "b")]);
    assert_eq!(map[1], "a");
    assert_eq!(map[4], "b");
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
#[should_panic = "no entry found for key"]
fn test_index_panic() {
    let map = RangeMapBlaze::<i32, &str>::from_iter([(1..=3, "a"), (4..=6, "b")]);
    let _ = map[0];
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
fn test_range_map_blaze_comparisons() {
    // Lexicographic comparison test
    let a = RangeMapBlaze::from_iter([(1..=3, "a"), (5..=100, "a")]);
    let b = RangeMapBlaze::from_iter([(2..=2, "b")]);
    assert!(a < b); // Lexicographic comparison
    assert!(a <= b);
    assert!(b > a);
    assert!(b >= a);
    assert!(a != b);
    assert!(a == a);

    assert_eq!(a.cmp(&b), Ordering::Less);

    // Float comparison test (using comparable bits)
    let a = RangeMapBlaze::from_iter([(2..=3, 1.0f32.to_bits()), (5..=100, 2.0f32.to_bits())]);
    let b = RangeMapBlaze::from_iter([(2..=2, f32::NAN.to_bits())]);
    assert_eq!(a.partial_cmp(&b), Some(Ordering::Less));
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
#[allow(clippy::into_iter_on_ref)]
fn test_from_iter_dedup() {
    let v = vec![(3, &"a"), (2, &"a"), (1, &"a"), (100, &"b"), (1, &"c")];
    let a0 = RangeMapBlaze::from_iter(&v);
    let a1: RangeMapBlaze<i32, &str> = (&v).into_iter().collect();
    assert!(a0 == a1 && a0.to_string() == r#"(1..=3, "a"), (100..=100, "b")"#);
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
#[allow(clippy::reversed_empty_ranges)]
#[allow(clippy::into_iter_on_ref)]
fn test_range_map_blaze_from_iter() {
    let v = vec![
        (1..=2, &"a"),
        (2..=2, &"b"),
        (-10..=-5, &"c"),
        (1..=0, &"d"),
    ];
    let a0: RangeMapBlaze<i32, &str> = RangeMapBlaze::from_iter(&v);
    let a1: RangeMapBlaze<i32, &str> = (&v).into_iter().collect();
    assert!(a0 == a1 && a0.to_string() == r#"(-10..=-5, "c"), (1..=2, "a")"#);
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
#[allow(clippy::into_iter_on_ref, clippy::needless_borrow)]
fn test_range_map_blaze_from_iter_string() {
    let v = vec![(1, "a"), (2, "a"), (2, "b")];
    let a0 = RangeMapBlaze::from_iter(&v);
    let a1: RangeMapBlaze<i32, &str> = (&v).iter().collect();
    assert!(a0 == a1 && a0.to_string() == r#"(1..=2, "a")"#);
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
fn sorted_disjoint_coverage0() {
    let a = CheckSortedDisjointMap::new([(1..=2, &"a"), (3..=4, &"b")]);
    let b0 = RangeMapBlaze::from_iter([(1..=2, "a")]);
    let b = b0.range_values();
    assert!(!a.equal(b)); // This should return false because `a` and `b` have different lengths

    let a = CheckSortedDisjointMap::new([(1..=2, &"a"), (3..=4, &"b")]);
    assert!(!a.is_empty());

    let a: CheckSortedDisjointMap<i32, &&str, core::iter::Empty<(RangeInclusive<i32>, &&str)>> =
        CheckSortedDisjointMap::default();
    assert!(a.is_empty());
}

#[must_use = "iterators are lazy and do nothing unless consumed"]
#[derive(Default)]
struct NotFusedIterator {
    state: bool,
}

impl Iterator for NotFusedIterator {
    type Item = (std::ops::RangeInclusive<i32>, &'static &'static str);

    fn next(&mut self) -> Option<Self::Item> {
        self.state = !self.state;
        if self.state {
            Some((1..=2, &"a"))
        } else {
            None
        }
    }
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
#[should_panic(expected = "a value must not be returned after None")]
fn test_panic_not_fused() {
    let mut iter = CheckSortedDisjointMap::new(NotFusedIterator::default());
    assert_eq!(iter.next(), Some((1..=2, &"a")));
    assert_eq!(iter.next(), None);
    assert_eq!(iter.next(), Some((1..=2, &"a"))); // Should panic
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
#[allow(clippy::reversed_empty_ranges)]
#[should_panic(expected = "start must be <= end")]
fn test_panic_start_greater_than_end() {
    let mut iter = CheckSortedDisjointMap::new([(3..=2, &"a")]);
    assert_eq!(iter.next(), Some((3..=2, &"a"))); // Invalid range, should panic
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
#[should_panic(expected = "ranges must be disjoint and sorted")]
fn test_panic_ranges_not_disjoint_or_sorted() {
    for _ in CheckSortedDisjointMap::new([(1..=3, &"a"), (2..=4, &"b")]) {} // Overlapping range
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
#[should_panic(expected = "touching ranges must have different values")]
fn test_panic_touching_ranges_same_value() {
    for _ in CheckSortedDisjointMap::new([(1..=2, &"a"), (3..=4, &"a")]) {} // Touching ranges with the same value
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
fn test_into_range_values() {
    let mut a = RangeMapBlaze::from_iter([(1..=2, "a"), (3..=4, "b")]).into_range_values();
    assert_eq!(a.size_hint(), (2, Some(2)));
    assert_eq!(a.len(), 2);
    let _ = a.next();
    assert_eq!(a.len(), 1);

    let mut a = RangeMapBlaze::from_iter([(1..=2, "a"), (3..=4, "b")]).into_range_values();
    assert_eq!(a.next_back(), Some((3..=4, Rc::new("b"))));
    assert_eq!(a.next_back(), Some((1..=2, Rc::new("a"))));
    assert_eq!(a.next_back(), None);
    assert_eq!(a.len(), 0);
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
fn test_into_map_into_ranges() {
    let mut a = RangeMapBlaze::from_iter([(1..=2, "a"), (3..=4, "b")]).into_ranges();
    assert_eq!(a.size_hint(), (0, Some(2)));
    assert_eq!(a.next(), Some(1..=4));

    let r = RangeMapBlaze::from_iter([(1..=2, "a"), (3..=4, "b")]);
    let mut a = r.ranges();
    assert_eq!(a.size_hint(), (0, Some(2)));
    assert_eq!(a.next(), Some(1..=4));
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
fn fast_union() {
    let a = RangeMapBlaze::from_iter([(1..=2, "a")]);
    let b = RangeMapBlaze::from_iter([
        (1..=5, "x"),
        (13..=14, "b"),
        (15..=16, "c"),
        (17..=18, "d"),
        (19..=20, "e"),
    ]);
    let c = a | b;
    assert_eq!(
        c.to_string(),
        r#"(1..=2, "a"), (3..=5, "x"), (13..=14, "b"), (15..=16, "c"), (17..=18, "d"), (19..=20, "e")"#
    );
}

// cmk-1 understand why so much is commented out and if it can be deleted.

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
fn more_coverage_of_maps() {
    // Test BitOr when the right-hand side is empty, ensuring the result is the left-hand side.
    let a = RangeMapBlaze::from_iter([(1..=10, "a")]);
    let expected = a.clone();
    let b: RangeMapBlaze<i32, &str> = RangeMapBlaze::default();
    let union = a | b;
    assert_eq!(union, expected);

    // Test BitOr<&Self> when the right-hand side is empty, ensuring the result is the left-hand side.
    let a = RangeMapBlaze::from_iter([(1..=2, "a"), (5..=100, "a")]);
    let expected = a.clone();
    let b: RangeMapBlaze<i32, &str> = RangeMapBlaze::default();
    let union = a | &b;
    assert_eq!(union, expected);

    // Test BitOr<RangeMapBlaze<T, V>> for &RangeMapBlaze<T, V> with empty and non-empty other to cover all branches.

    // Case 1: 'a' is non-empty, 'b' is empty; expect 'a' unchanged.
    let a = RangeMapBlaze::from_iter([(1..=1, "a"), (5..=5, "a")]);
    let b_empty: RangeMapBlaze<i32, &str> = RangeMapBlaze::default();
    let union1 = &a | b_empty;
    assert_eq!(union1, a);

    // Case 2: 'a' and 'b' are non-empty with non-overlapping ranges and different values; expect union to include both without merging.
    let b_non_empty =
        RangeMapBlaze::from_iter([(2..=2, "b"), (4..=4, "c"), (6..=6, "d"), (8..=8, "e")]);
    let union2 = &a | b_non_empty;
    assert_eq!(
        union2.to_string(),
        r#"(1..=1, "a"), (2..=2, "b"), (4..=4, "c"), (5..=5, "a"), (6..=6, "d"), (8..=8, "e")"#
    );

    // Case 3: 'a' and 'b' are non-empty with overlapping ranges, triggering the else branch.
    let a_large = RangeMapBlaze::from_iter([(1..=2, "a"), (4..=5, "a"), (7..=8, "a")]);
    let b_small = RangeMapBlaze::from_iter([(3..=3, "b"), (6..=6, "b")]);
    let union3 = &a_large | b_small;
    assert_eq!(
        union3.to_string(),
        r#"(1..=2, "a"), (3..=3, "b"), (4..=5, "a"), (6..=6, "b"), (7..=8, "a")"#
    );

    // Test BitOr<&RangeMapBlaze<T, V>> when the right-hand side is empty and non-empty.

    // Scenario 1: 'a' is non-empty, 'b' is empty; expect 'a' cloned.
    let a = RangeMapBlaze::from_iter([(1..=2, "a"), (5..=5, "a")]);
    let b_empty: RangeMapBlaze<i32, &str> = RangeMapBlaze::default();
    let union1 = &a | b_empty;
    assert_eq!(union1.to_string(), r#"(1..=2, "a"), (5..=5, "a")"#);

    // Scenario 2: 'a' is empty, 'b' is non-empty; expect 'b' cloned.
    let a_empty: RangeMapBlaze<i32, &str> = RangeMapBlaze::default();
    let b = RangeMapBlaze::from_iter([(3..=3, "b")]);
    let expected = b.clone();
    let union2 = &a_empty | b;
    assert_eq!(union2, expected);

    // Test BitOrAssign<&Self> when the right-hand side is empty, ensuring 'self' remains unchanged.
    let mut a = RangeMapBlaze::from_iter([(1..=2, "a"), (5..=5, "a")]);
    let expected = a.clone();
    let b: RangeMapBlaze<i32, &str> = RangeMapBlaze::default();
    a |= &b;
    assert_eq!(a, expected);

    // Test BitOrAssign<Self> when the right-hand side is empty, ensuring 'self' remains unchanged.
    let mut a = RangeMapBlaze::from_iter([(1..=4, "a")]);
    let expected = a.clone();
    let other: RangeMapBlaze<i32, &str> = RangeMapBlaze::default();
    a |= other;
    assert_eq!(a, expected);

    // Test BitOr<Self> when 'self' is empty and 'other' has values, ensuring 'other' is returned as a clone.
    let a: RangeMapBlaze<i32, &str> = RangeMapBlaze::default(); // 'self' is empty
    let b = RangeMapBlaze::from_iter([(1..=2, "b"), (5..=10, "b")]); // 'other' has ranges
    let union = a | &b;
    assert_eq!(union, b);

    // Testing BitOr<RangeMapBlaze<T, V>> when 'self' is large and 'other' is small, hitting the efficiency loop.
    let a = RangeMapBlaze::from_iter([
        (1..=2, "a"),
        (5..=5, "a"),
        (10..=15, "a"),
        (20..=25, "a"),
        (30..=35, "a"),
    ]); // 'self' has multiple ranges, making it relatively "large"
    let b = RangeMapBlaze::from_iter([(3..=4, "b")]); // 'other' is relatively "small"
    let union = &a | b;
    // Assert that 'union' correctly integrates ranges from both 'self' and 'other'
    assert_eq!(
        union.to_string(),
        r#"(1..=2, "a"), (3..=4, "b"), (5..=5, "a"), (10..=15, "a"), (20..=25, "a"), (30..=35, "a")"#
    );

    // Testing `BitOr<&RangeMapBlaze>` where `other` is empty.
    let a = RangeMapBlaze::from_iter([(1..=2, "a"), (5..=5, "a"), (10..=15, "a")]);
    let b = RangeMapBlaze::default(); // `other` is empty
    let union = &a | &b;
    // Assert that `union` is identical to `a`, since `other` (b) is empty.
    assert_eq!(union, a);
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
fn test_range_values_to_ranges_iter_disjoint() {
    let a = CheckSortedDisjointMap::new([(1..=3, &"a"), (4..=4, &"b"), (5..=10, &"a")]);
    let mut iter = a.into_sorted_disjoint();
    assert_eq!(iter.next(), Some(1..=10));
    assert_eq!(iter.next(), None);
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
fn test_into_iter() {
    let a = RangeMapBlaze::from_iter([(1..=2, "a"), (5..=5, "a"), (10..=15, "a")]);
    let mut b = a.into_iter();
    assert_eq!(b.next(), Some((1, "a")));
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
fn test_retain() {
    let mut map: RangeMapBlaze<i32, i32> = (0..8).map(|x| (x, x * 10)).collect();
    // Keep only the elements with even-numbered keys.
    map.retain(|&k, _| k % 2 == 0);
    assert!(map.into_iter().eq(vec![(0, 0), (2, 20), (4, 40), (6, 60)]));
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
fn test_ranges_retain() {
    let mut map: RangeMapBlaze<i32, i32> = (0..8).map(|x| (x, x * 10)).collect();
    // Keep only the elements with even-numbered keys.
    map.ranges_retain(|k, _| k.start() % 2 == 0);
    assert!(map.into_iter().eq(vec![(0, 0), (2, 20), (4, 40), (6, 60)]));
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
#[allow(clippy::type_complexity)]
fn test_empty_inputs_union_symmetric_difference() {
    let inputs: [RangeMapBlaze<i32, &str>; 0] = [];
    let union = inputs.union();
    assert_eq!(union.to_string(), r#""#);

    let inputs: [RangeMapBlaze<i32, &str>; 0] = [];
    let symmetric_difference = inputs.symmetric_difference();
    assert_eq!(symmetric_difference.to_string(), r#""#);

    let inputs: [&RangeMapBlaze<i32, &str>; 0] = [];
    let union = inputs.union();
    assert_eq!(union.to_string(), r#""#);

    let inputs: [&RangeMapBlaze<i32, &str>; 0] = [];
    let symmetric_difference = inputs.symmetric_difference();
    assert_eq!(symmetric_difference.to_string(), r#""#);

    fn make_inputs<'a>() -> [CheckSortedDisjointMap<
        i32,
        &'a &'a str,
        std::vec::IntoIter<(RangeInclusive<i32>, &'a &'a str)>,
    >; 0] {
        []
    }
    let inputs = make_inputs();
    let union = inputs.union();
    assert_eq!(union.into_string(), r#""#);

    let inputs = make_inputs();
    let symmetric_difference = inputs.symmetric_difference();
    assert_eq!(symmetric_difference.into_string(), r#""#);
}

#[test]
#[should_panic]
fn test_empty_inputs_intersection0() {
    let inputs: [RangeMapBlaze<i32, &str>; 0] = [];
    let intersection = inputs.intersection();
    assert_eq!(intersection.to_string(), r#"should panic"#);
}

#[test]
#[should_panic]
fn test_empty_inputs_intersection1() {
    let inputs: [&RangeMapBlaze<i32, &str>; 0] = [];
    let intersection = inputs.intersection();
    assert_eq!(intersection.to_string(), r#"should panic"#);
}

#[test]
#[should_panic]
#[allow(clippy::type_complexity)]
fn test_empty_inputs_intersection2() {
    fn make_inputs<'a>() -> [CheckSortedDisjointMap<
        i32,
        &'a &'a str,
        std::vec::IntoIter<(RangeInclusive<i32>, &'a &'a str)>,
    >; 0] {
        []
    }
    let inputs = make_inputs();
    let intersection = inputs.intersection();
    assert_eq!(intersection.into_string(), r#"should panic"#);
}

#[test]
#[allow(clippy::reversed_empty_ranges)]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
fn test_range_method_on_range_map_blaze() {
    let map = RangeMapBlaze::<i32, &str>::from_iter([(1..=3, "a"), (4..=6, "b")]);
    let expected = RangeMapBlaze::<i32, &str>::from_iter([(3..=3, "a"), (4..=6, "b")]);
    let a: RangeMapBlaze<i32, &str> = map.range(3..).collect();
    assert_eq!(a, expected);

    let a: RangeMapBlaze<i32, &str> = map.range(..).collect();
    assert_eq!(
        a,
        RangeMapBlaze::<i32, &str>::from_iter([(1..=3, "a"), (4..=6, "b")])
    );

    let a: RangeMapBlaze<i32, &str> = map.range(3..7).collect();
    assert_eq!(a, expected);

    let a: RangeMapBlaze<i32, &str> = map
        .range((Bound::Excluded(2), Bound::Excluded(7)))
        .collect();
    assert_eq!(a, expected);
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
fn test_cmp() {
    fn to_bits(vv_pair: Vec<(u32, f64)>) -> Vec<(u32, u64)> {
        vv_pair.into_iter().map(|(k, v)| (k, v.to_bits())).collect()
    }

    let test_cases = vec![
        (
            vec![(2, 1.0), (11, 1.0), (12, 1.0)],
            vec![(3, 2.0), (11, 1.0), (12, f64::NAN)],
            Ordering::Less,
        ),
        // Mixed case
        (
            vec![(0, 1.0), (1, 2.0), (2, 1.0), (3, 3.0), (4, 2.0)],
            vec![(0, 1.0), (1, 1.0), (2, 1.0), (3, 2.0), (4, 2.0)],
            Ordering::Greater,
        ),
        // Equal elements
        (
            vec![(0, 1.0), (1, 1.0), (2, 1.0), (3, 2.0), (4, 2.0), (5, 2.0)],
            vec![(0, 1.0), (1, 1.0), (2, 1.0), (3, 2.0), (4, 2.0), (5, 2.0)],
            Ordering::Equal,
        ),
        // Different values
        (
            vec![(0, 1.0), (1, 1.0), (2, 1.0), (3, 2.0), (4, 2.0), (5, 2.0)],
            vec![(0, 1.0), (1, 1.0), (2, 1.0), (3, 2.0), (4, 2.0), (5, 3.0)],
            Ordering::Less,
        ),
        (
            vec![(0, 1.0), (1, 1.0), (2, 1.0), (3, 2.0), (4, 2.0), (5, 3.0)],
            vec![(0, 1.0), (1, 1.0), (2, 1.0), (3, 2.0), (4, 2.0), (5, 2.0)],
            Ordering::Greater,
        ),
        // Different keys
        (
            vec![(0, 1.0), (1, 1.0), (2, 1.0), (4, 2.0), (5, 2.0)],
            vec![(0, 1.0), (1, 1.0), (2, 1.0), (3, 2.0), (4, 2.0), (5, 2.0)],
            Ordering::Greater,
        ),
        (
            vec![(0, 1.0), (1, 1.0), (2, 1.0)],
            vec![(0, 1.0), (1, 1.0), (2, 1.0), (3, 2.0), (4, 2.0), (5, 2.0)],
            Ordering::Less,
        ),
        (
            vec![(0, 1.0), (1, 1.0), (2, 1.0), (3, 2.0), (4, 2.0), (5, 2.0)],
            vec![(0, 1.0), (1, 1.0), (2, 1.0)],
            Ordering::Greater,
        ),
        // To apply .to_bits() so that NANs are compared, too.
        (
            vec![(0, 1.0), (1, 1.0), (2, f64::NAN)],
            vec![(0, 1.0), (1, 1.0), (2, 1.0)],
            Ordering::Greater,
        ),
        (
            vec![(0, 1.0), (1, 1.0), (2, 1.0)],
            vec![(0, 1.0), (1, 1.0), (2, f64::NAN)],
            Ordering::Less,
        ),
    ];

    let test_cases = test_cases
        .into_iter()
        .map(|(a, b, expected)| (to_bits(a), to_bits(b), expected));

    for (a_data, b_data, expected) in test_cases {
        println!("expected = {expected:?}");
        let a_btree = BTreeMap::from_iter(a_data.clone());
        let b_btree = BTreeMap::from_iter(b_data.clone());
        assert_eq!(a_btree.cmp(&b_btree), expected);

        let a_range_set = RangeMapBlaze::from_iter(a_data);
        let b_range_set = RangeMapBlaze::from_iter(b_data);
        assert_eq!(a_range_set.cmp(&b_range_set), expected);
    }
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
#[allow(clippy::style)]
fn bitor_assign_coverage() {
    for (a0, b0, c0) in [
        (
            vec![(3..=3, "a"), (5..=10, "a"), (12..=15, "a")],
            vec![(2..=3, "b"), (5..=10, "b"), (12..=15, "b")],
            vec![(2..=2, "b"), (3..=3, "a"), (5..=10, "a"), (12..=15, "a")],
        ),
        (vec![(2..=3, "a")], vec![(3..=3, "b")], vec![(2..=3, "a")]),
        (vec![], vec![(3..=3, "b")], vec![(3..=3, "b")]),
    ] {
        let c = RangeMapBlaze::from_iter(c0);

        let mut a = RangeMapBlaze::from_iter(&a0);
        let b = RangeMapBlaze::from_iter(&b0);
        a = a | b;
        assert_eq!(a, c);
        let mut a = RangeMapBlaze::from_iter(&a0);
        let b = RangeMapBlaze::from_iter(&b0);
        a |= b;
        assert_eq!(a, c);
        let mut a = RangeMapBlaze::from_iter(&a0);
        let b = RangeMapBlaze::from_iter(&b0);
        a |= &b;
        assert_eq!(a, c);
        let mut b = RangeMapBlaze::from_iter(&b0);
        b.extend(a0.clone());
        assert_eq!(b, c);
    }
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
fn test_arc_clone() {
    let a = Arc::new(1);
    let b = Arc::clone(&a);
    assert_eq!(a, b);
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
fn test_range() {
    let mut map = RangeMapBlaze::new();
    map.insert(3, "a");
    map.insert(5, "b");
    map.insert(8, "c");
    for (key, value) in map.range((Included(4), Included(8))) {
        println!("{key}: {value}");
    } // prints "5: b" and "8: c"
    assert_eq!(Some((5, "b")), map.range(4..).next());
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
fn test_pop_first() {
    let mut map: RangeMapBlaze<i128, &str> = RangeMapBlaze::new();
    assert_eq!(None, map.pop_first());
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
fn test_range_values_len() {
    // We put in four ranges, but they are not sorted & disjoint.
    let map = RangeMapBlaze::from_iter([
        (10..=20, "a"),
        (15..=25, "b"),
        (30..=40, "c"),
        (28..=35, "c"),
    ]);
    // After RangeMapBlaze sorts & 'disjoint's them, we see three ranges.
    assert_eq!(map.range_values_len(), 3);
    assert_eq!(
        map.to_string(),
        r#"(10..=20, "a"), (21..=25, "b"), (28..=40, "c")"#
    );
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
fn test_into_iterator_for_ref_rangemapblaze() {
    let map = RangeMapBlaze::from_iter([(1..=2, "a")]);
    let mut iter = (&map).into_iter();

    assert_eq!(iter.next(), Some((1, &"a")));
    assert_eq!(iter.next(), Some((2, &"a")));
    assert_eq!(iter.next(), None);
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
fn test_ref_union() {
    use std::println;

    let a = RangeSetBlaze::from_iter([1..=2, 5..=100]);
    let b = RangeSetBlaze::from_iter([2..=6]);
    let c = RangeSetBlaze::from_iter([2..=2, 6..=200]);
    let d = [&a, &b, &c].union();
    println!("{d}");
    assert_eq!(d, RangeSetBlaze::from_iter([1..=200]));

    let a = RangeSetBlaze::from_iter([1..=2, 5..=100]);
    let b = RangeSetBlaze::from_iter([2..=6]);
    let c = RangeSetBlaze::from_iter([2..=2, 6..=200]);
    let d = [a, b, c].union();
    println!("{d}");
    assert_eq!(d, RangeSetBlaze::from_iter([1..=200]));

    let a = RangeMapBlaze::from_iter([(1..=2, "a"), (5..=100, "a")]);
    let b = RangeMapBlaze::from_iter([(2..=6, "b")]);
    let c = RangeMapBlaze::from_iter([(2..=2, "c"), (6..=200, "c")]);
    let d = [&a, &b, &c].union();
    println!("{d}");
    assert_eq!(
        d,
        RangeMapBlaze::from_iter([(1..=2, "a"), (3..=4, "b"), (5..=100, "a"), (101..=200, "c")])
    );

    let a = RangeMapBlaze::from_iter([(1..=2, "a"), (5..=100, "a")]);
    let b = RangeMapBlaze::from_iter([(2..=6, "b")]);
    let c = RangeMapBlaze::from_iter([(2..=2, "c"), (6..=200, "c")]);
    let d: RangeMapBlaze<i32, &str> = [a, b, c].union();
    println!("{d}");
    assert_eq!(
        d,
        RangeMapBlaze::from_iter([(1..=2, "a"), (3..=4, "b"), (5..=100, "a"), (101..=200, "c")])
    );
}

#[test]
fn test_worst() {
    use rand::{SeedableRng, distr::Uniform, prelude::Distribution, rngs::StdRng};

    let iter_len = 20;
    let uniform_key = Uniform::new(0, 5).unwrap();
    let seed = 0;
    let n = 1;
    let uniform_value = Uniform::new(0, n).unwrap();

    let mut rng = StdRng::seed_from_u64(seed);
    let vec: Vec<(u32, u32)> = (0..iter_len)
        .map(|_| (uniform_key.sample(&mut rng), uniform_value.sample(&mut rng)))
        .collect();
    println!("{vec:?}");

    let a0a = RangeMapBlaze::from_iter(vec.iter().rev());
    let mut a0b = RangeMapBlaze::<u32, u32>::new();
    a0b.extend(vec.iter().cloned());
    let a1 = BTreeMap::from_iter(vec.iter().cloned());
    let a2: HashMap<u32, u32> = HashMap::from_iter(vec.iter().cloned());
    let a3 = rangemap::RangeInclusiveMap::from_iter(vec.iter().map(|(k, v)| (*k..=*k, *v)));
    println!("{a0a}");
    println!("{a0b}");
    println!("{a1:?}");
    println!("{a2:?}");
    println!("{a3:?}");
}

#[test]
fn test_union() {
    let range = 0..=99_999_999u32;
    let range_len0 = 5;
    let range_len = 2;
    let coverage_goal = 0.5;
    let how = How::None;
    let seed = 1;
    let n = 5u32;

    let mut rng = StdRng::seed_from_u64(seed);
    let temp: Vec<RangeMapBlaze<u32, u32>> =
        k_maps(1, range_len0, &range, coverage_goal, how, &mut rng, n);
    let map0 = &temp[0];
    let rangemap_map0 = &rangemap::RangeInclusiveMap::from_iter(map0.range_values());

    let map1 = &k_maps(1, range_len, &range, coverage_goal, how, &mut rng, n)[0];
    let rangemap_map1 = rangemap::RangeInclusiveMap::from_iter(map1.range_values());

    let mut a0a = map0.clone();
    a0a |= map1;

    let mut a0b = map1.clone();
    a0b |= map0;

    let mut a1 = map0.clone();
    a1.extend(map1.range_values().map(|(r, v)| (r.clone(), *v)));

    let mut a2 = rangemap_map0.clone();
    a2.extend(rangemap_map1.iter().map(|(r, v)| (r.clone(), *v)));

    println!("{a0a}");
    println!("{a0b}");
    println!("{a1}");
    println!("{a2:?}");
}
