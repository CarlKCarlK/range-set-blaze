//! Integration tests for the `RangeSetBlaze`.

#![cfg(test)]
use range_set_blaze::{
    AssumeSortedStarts, IntoIter, IntoRangesIter, Iter, KMerge, MapIntoRangesIter, MapRangesIter,
    Merge, RangeValuesIter, RangeValuesToRangesIter, RangesIter,
};

use wasm_bindgen_test::*;
wasm_bindgen_test_configure!(run_in_browser);

use core::fmt;
use core::fmt::Debug;
use core::iter::FusedIterator;
#[cfg(feature = "from_slice")]
use core::mem::size_of;
use core::ops::BitAndAssign;
#[cfg(feature = "rog_experimental")]
#[allow(deprecated)]
#[cfg(not(target_arch = "wasm32"))]
use core::ops::Bound;
use core::ops::RangeInclusive;
#[cfg(target_os = "linux")]
use criterion::{BatchSize, BenchmarkId, Criterion};
use itertools::Itertools;
use num_traits::identities::One;
use num_traits::identities::Zero;
#[cfg(not(target_arch = "wasm32"))]
use quickcheck_macros::quickcheck;
use rand::Rng;
use rand::SeedableRng;
use rand::rngs::StdRng;
#[cfg(feature = "rog_experimental")]
#[allow(deprecated)]
use range_set_blaze::Rog;
use range_set_blaze::SymDiffIter;
#[cfg(not(target_arch = "wasm32"))]
use range_set_blaze::test_util::{How, MemorylessIter, MemorylessRange, k_sets, width_to_range};
use range_set_blaze::{Integer, NotIter, SortedStarts, prelude::*};
use range_set_blaze::{UnionIter, symmetric_difference_dyn};
use std::any::Any;
use std::cmp::Ordering;
#[cfg(feature = "rog_experimental")]
#[allow(deprecated)]
use std::panic::AssertUnwindSafe;
#[cfg(feature = "rog_experimental")]
#[allow(deprecated)]
use std::panic::{self};
#[cfg(not(target_arch = "wasm32"))]
use std::time::Instant;
use std::{collections::BTreeSet, ops::BitOr};
use syntactic_for::syntactic_for;

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
fn insert_255u8() {
    let range_set_blaze = RangeSetBlaze::from_iter([255u8]);
    assert_eq!(range_set_blaze.to_string(), "255..=255");
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
fn insert_max_u128() {
    let _ = RangeSetBlaze::<u128>::from_iter([u128::MAX]);
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
fn complement0() {
    syntactic_for! { ty in [i8, u8, isize, usize,  i16, u16, i32, u32, i64, u64, isize, usize, i128, u128] {
        $(
        let empty = RangeSetBlaze::<$ty>::new();
        let full = !&empty;
        println!("empty: {empty} (len {}), full: {full} (len {})", empty.len(), full.len());
        )*
    }};
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
fn repro_bit_and() {
    let a = RangeSetBlaze::from_iter([1u8, 2, 3]);
    let b = RangeSetBlaze::from_iter([2u8, 3, 4]);

    let result = &a & &b;
    println!("{result}");
    assert_eq!(result, RangeSetBlaze::from_iter([2u8, 3]));
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
fn doctest1() {
    let a = RangeSetBlaze::<u8>::from_iter([1, 2, 3]);
    let b = RangeSetBlaze::<u8>::from_iter([3, 4, 5]);

    let result = &a | &b;
    assert_eq!(result, RangeSetBlaze::<u8>::from_iter([1, 2, 3, 4, 5]));
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
fn doctest2() {
    let set = RangeSetBlaze::<u8>::from_iter([1, 2, 3]);
    assert!(set.contains(1));
    assert!(!set.contains(4));
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
fn doctest3() {
    let mut a = RangeSetBlaze::from_iter([1u8..=3]);
    let mut b = RangeSetBlaze::from_iter([3u8..=5]);

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
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
fn doctest4() {
    let a = RangeSetBlaze::<i8>::from_iter([1, 2, 3]);

    let result = !&a;
    assert_eq!(result.to_string(), "-128..=0, 4..=127");
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
fn compare() {
    let mut btree_set = BTreeSet::<u128>::new();
    btree_set.insert(3);
    btree_set.insert(1);
    let string = btree_set.iter().join(", ");
    println!("{string:#?}");
    assert!(string == "1, 3");
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
fn add_in_order() {
    let mut range_set = RangeSetBlaze::new();
    for i in 0u64..1000 {
        range_set.insert(i);
    }
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
fn iters() {
    let range_set_blaze = RangeSetBlaze::from_iter([1u8..=6, 8..=9, 11..=15]);
    assert!(range_set_blaze.len() == 13);
    for i in &range_set_blaze {
        println!("{i}");
    }
    for range in range_set_blaze.ranges() {
        println!("{range:?}");
    }
    let mut rs = range_set_blaze.ranges();
    println!("{:?}", rs.next());
    println!("{range_set_blaze}");
    println!("{:?}", rs.len());
    println!("{:?}", rs.next());
    for i in &range_set_blaze {
        println!("{i}");
    }
    // range_set_blaze.len();

    let mut rs = range_set_blaze.ranges().complement();
    println!("{:?}", rs.next());
    println!("{range_set_blaze}");
    // !!! assert that can't use range_set_blaze again
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
fn missing_doctest_ops() {
    // note that may be borrowed or owned in any combination.

    // Returns the union of `self` and `rhs` as a new [`RangeSetBlaze`].
    let a = RangeSetBlaze::from_iter([1, 2, 3]);
    let b = RangeSetBlaze::from_iter([3, 4, 5]);

    let result = &a | &b;
    assert_eq!(result, RangeSetBlaze::from_iter([1, 2, 3, 4, 5]));
    let result = a | &b;
    assert_eq!(result, RangeSetBlaze::from_iter([1, 2, 3, 4, 5]));

    // Returns the complement of `self` as a new [`RangeSetBlaze`].
    let a = RangeSetBlaze::<i8>::from_iter([1, 2, 3]);

    let result = !&a;
    assert_eq!(result.to_string(), "-128..=0, 4..=127");
    let result = !a;
    assert_eq!(result.to_string(), "-128..=0, 4..=127");

    // Returns the intersection of `self` and `rhs` as a new `RangeSetBlaze<T>`.

    let a = RangeSetBlaze::from_iter([1, 2, 3]);
    let b = RangeSetBlaze::from_iter([2, 3, 4]);

    let result = a & &b;
    assert_eq!(result, RangeSetBlaze::from_iter([2, 3]));
    let a = RangeSetBlaze::from_iter([1, 2, 3]);
    let result = a & b;
    assert_eq!(result, RangeSetBlaze::from_iter([2, 3]));

    // Returns the symmetric difference of `self` and `rhs` as a new `RangeSetBlaze<T>`.
    let a = RangeSetBlaze::from_iter([1, 2, 3]);
    let b = RangeSetBlaze::from_iter([2, 3, 4]);

    let result = a ^ b;
    assert_eq!(result, RangeSetBlaze::from_iter([1, 4]));

    // Returns the set difference of `self` and `rhs` as a new `RangeSetBlaze<T>`.
    let a = RangeSetBlaze::from_iter([1, 2, 3]);
    let b = RangeSetBlaze::from_iter([2, 3, 4]);

    let result = a - b;
    assert_eq!(result, RangeSetBlaze::from_iter([1]));
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
fn multi_op() {
    // Union
    let a = RangeSetBlaze::from_iter([1..=6, 8..=9, 11..=15]);
    let b = RangeSetBlaze::from_iter([5..=13, 18..=29]);
    let c = RangeSetBlaze::from_iter([38..=42]);
    let d = &(&a | &b) | &c;
    assert_eq!(d, RangeSetBlaze::from_iter([1..=15, 18..=29, 38..=42]));
    let d = a | b | &c;
    assert_eq!(d, RangeSetBlaze::from_iter([1..=15, 18..=29, 38..=42]));

    let a = RangeSetBlaze::from_iter([1..=6, 8..=9, 11..=15]);
    let b = RangeSetBlaze::from_iter([5..=13, 18..=29]);
    let c = RangeSetBlaze::from_iter([38..=42]);

    let _ = [&a, &b, &c].union();
    let d = [a, b, c].iter().intersection();
    assert_eq!(d, RangeSetBlaze::new());

    assert_eq!(
        MultiwayRangeSetBlazeRef::<u8>::union([]),
        RangeSetBlaze::new()
    );

    // Intersection
    let a = RangeSetBlaze::from_iter([1..=6, 8..=9, 11..=15]);
    let b = RangeSetBlaze::from_iter([5..=13, 18..=29]);
    let c = RangeSetBlaze::from_iter([1..=42]);

    let _ = &a & &b;
    let d = [&a, &b, &c].intersection();
    // let d = RangeSetBlaze::intersection([a, b, c]);
    println!("{d}");
    assert_eq!(d, RangeSetBlaze::from_iter([5..=6, 8..=9, 11..=13]));

    assert_eq!(
        MultiwayRangeSetBlazeRef::<u8>::intersection([]),
        RangeSetBlaze::from_iter([0..=255])
    );

    // Symmetrical difference
    let a = RangeSetBlaze::from_iter([1..=6, 8..=9, 11..=15]);
    let b = RangeSetBlaze::from_iter([5..=13, 18..=29]);
    let c = RangeSetBlaze::from_iter([38..=42]);
    let d = &(&a ^ &b) ^ &c;
    assert_eq!(
        d,
        RangeSetBlaze::from_iter([1..=4, 7..=7, 10..=10, 14..=15, 18..=29, 38..=42])
    );
    let d = a ^ b ^ &c;
    assert_eq!(
        d,
        RangeSetBlaze::from_iter([1..=4, 7..=7, 10..=10, 14..=15, 18..=29, 38..=42])
    );

    let a = RangeSetBlaze::from_iter([1..=6, 8..=9, 11..=15]);
    let b = RangeSetBlaze::from_iter([5..=13, 18..=29]);
    let c = RangeSetBlaze::from_iter([38..=42]);

    let _ = [&a, &b, &c].symmetric_difference();

    assert_eq!(
        MultiwayRangeSetBlazeRef::<u8>::symmetric_difference([]),
        RangeSetBlaze::new()
    );
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
fn custom_multi() {
    // Union
    let a = RangeSetBlaze::from_iter([1..=6, 8..=9, 11..=15]);
    let b = RangeSetBlaze::from_iter([5..=13, 18..=29]);
    let c = RangeSetBlaze::from_iter([38..=42]);

    let union_stream = b.ranges() | c.ranges();
    let a_less = a.ranges() - union_stream;
    let d: RangeSetBlaze<_> = a_less.into_range_set_blaze();
    assert_eq!(d, RangeSetBlaze::from_iter([1..=4, 14..=15]));

    let d: RangeSetBlaze<_> =
        (a.ranges() - [b.ranges(), c.ranges()].union()).into_range_set_blaze();
    assert_eq!(d, RangeSetBlaze::from_iter([1..=4, 14..=15]));

    // Intersection
    let a = RangeSetBlaze::from_iter([1..=6, 8..=9, 11..=15]);
    let b = RangeSetBlaze::from_iter([5..=13, 18..=29]);
    let c = RangeSetBlaze::from_iter([1..=42]);

    let intersection_stream = b.ranges() & c.ranges();
    let a_less = a.ranges() - intersection_stream;
    let d: RangeSetBlaze<_> = a_less.into_range_set_blaze();
    assert_eq!(d, RangeSetBlaze::from_iter([1..=4, 14..=15]));
    println!("{d}");

    let d: RangeSetBlaze<_> =
        (a.ranges() - [b.ranges(), c.ranges()].intersection()).into_range_set_blaze();
    assert_eq!(d, RangeSetBlaze::from_iter([1..=4, 14..=15]));

    // Symmetrical difference
    let a = RangeSetBlaze::from_iter([1..=6, 8..=9, 11..=15]);
    let b = RangeSetBlaze::from_iter([5..=13, 18..=29]);
    let c = RangeSetBlaze::from_iter([38..=42]);

    let sym_diff_stream = b.ranges() ^ c.ranges();
    let a_less = a.ranges() - sym_diff_stream;
    let d: RangeSetBlaze<_> = a_less.into_range_set_blaze();
    assert_eq!(d, RangeSetBlaze::from_iter([1..=4, 14..=15]));

    let d: RangeSetBlaze<_> =
        (a.ranges() - [b.ranges(), c.ranges()].symmetric_difference()).into_range_set_blaze();
    assert_eq!(d, RangeSetBlaze::from_iter([1..=4, 14..=15]));
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
fn from_string() {
    let a = RangeSetBlaze::from_iter([0..=4, 14..=17, 30..=255, 0..=37, 43..=65535]);
    assert_eq!(a, RangeSetBlaze::from_iter([0..=65535]));
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
fn nand_repro() {
    let b = &RangeSetBlaze::from_iter([5u8..=13, 18..=29]);
    let c = &RangeSetBlaze::from_iter([38..=42]);
    println!("about to nand");
    let d = !b | !c;
    assert_eq!(
        d,
        RangeSetBlaze::from_iter([0..=4, 14..=17, 30..=255, 0..=37, 43..=255])
    );
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
fn parity() {
    let a = &RangeSetBlaze::from_iter([1..=6, 8..=9, 11..=15]);
    let b = &RangeSetBlaze::from_iter([5..=13, 18..=29]);
    let c = &RangeSetBlaze::from_iter([38..=42]);
    assert_eq!(
        a & !b & !c | !a & b & !c | !a & !b & c | a & b & c,
        RangeSetBlaze::from_iter([1..=4, 7..=7, 10..=10, 14..=15, 18..=29, 38..=42])
    );
    assert_eq!(
        a ^ b ^ c,
        RangeSetBlaze::from_iter([1..=4, 7..=7, 10..=10, 14..=15, 18..=29, 38..=42])
    );
    let _d = [a.ranges()].intersection();
    let _parity: RangeSetBlaze<u8> = [[a.ranges()].intersection()].union().into_range_set_blaze();
    let _parity: RangeSetBlaze<u8> = [a.ranges()].intersection().into_range_set_blaze();
    let _parity: RangeSetBlaze<u8> = [a.ranges()].union().into_range_set_blaze();
    println!("!b {}", !b);
    println!("!c {}", !c);
    println!("!b|!c {}", !b | !c);
    println!(
        "!b|!c {}",
        RangeSetBlaze::from_sorted_disjoint(b.ranges().complement() | c.ranges().complement())
    );

    let _a = RangeSetBlaze::from_iter([1..=6, 8..=9, 11..=15]);
    let u = [DynSortedDisjoint::new(a.ranges())].union();
    assert_eq!(
        RangeSetBlaze::from_sorted_disjoint(u),
        RangeSetBlaze::from_iter([1..=6, 8..=9, 11..=15])
    );
    let u = union_dyn!(a.ranges());
    assert_eq!(
        RangeSetBlaze::from_sorted_disjoint(u),
        RangeSetBlaze::from_iter([1..=6, 8..=9, 11..=15])
    );
    let u = union_dyn!(a.ranges(), b.ranges(), c.ranges());
    assert_eq!(
        RangeSetBlaze::from_sorted_disjoint(u),
        RangeSetBlaze::from_iter([1..=15, 18..=29, 38..=42])
    );

    let u = [
        intersection_dyn!(a.ranges(), b.ranges().complement(), c.ranges().complement()),
        intersection_dyn!(a.ranges().complement(), b.ranges(), c.ranges().complement()),
        intersection_dyn!(a.ranges().complement(), b.ranges().complement(), c.ranges()),
        intersection_dyn!(a.ranges(), b.ranges(), c.ranges()),
    ]
    .union();
    assert_eq!(
        RangeSetBlaze::from_sorted_disjoint(u),
        RangeSetBlaze::from_iter([1..=4, 7..=7, 10..=10, 14..=15, 18..=29, 38..=42])
    );

    assert_eq!(
        symmetric_difference_dyn!(a.ranges(), b.ranges(), c.ranges()).into_range_set_blaze(),
        RangeSetBlaze::from_iter([1..=4, 7..=7, 10..=10, 14..=15, 18..=29, 38..=42])
    );
}

// skip this test because the expected error message is not stable
// );
// #[test]
// fn ui() {
//     let t = trybuild::TestCases::new();
//     t.compile_fail("tests/ui/*.rs");
// }

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
#[allow(clippy::many_single_char_names)]
fn complement() {
    // RangeSetBlaze, RangesIter, NotIter, UnionIter, Tee, UnionIter(g)
    let a0 = RangeSetBlaze::from_iter([1..=6]);
    let a1 = RangeSetBlaze::from_iter([8..=9, 11..=15]);
    let a = &a0 | &a1;
    let not_a = !&a;
    let b = a.ranges();
    let c = !not_a.ranges();
    let d = a0.ranges() | a1.ranges();

    let f = a0.ranges().union(a1.ranges());
    let not_b = !b;
    let not_c = !c;
    let not_d = !d;
    let not_f = !f;
    assert!(not_a.ranges().equal(not_b));
    assert!(not_a.ranges().equal(not_c));
    assert!(not_a.ranges().equal(not_d));
    assert!(not_a.ranges().equal(not_f));
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
fn union_test() {
    let a0 = RangeSetBlaze::from_iter([1..=6]);
    let a1 = RangeSetBlaze::from_iter([8..=9]);
    let a2 = RangeSetBlaze::from_iter([11..=15]);
    let a12 = &a1 | &a2;
    let not_a0 = !&a0;
    let a = &a0 | &a1 | &a2;
    let b = a0.ranges() | a1.ranges() | a2.ranges();
    let c = !not_a0.ranges() | a12.ranges();
    let d = a0.ranges() | a1.ranges() | a2.ranges();

    assert!(a.ranges().equal(b));
    assert!(a.ranges().equal(c));
    assert!(a.ranges().equal(d));
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
#[allow(clippy::many_single_char_names)]
fn xor() {
    let a0 = RangeSetBlaze::from_iter([1..=6]);
    let a1 = RangeSetBlaze::from_iter([8..=9]);
    let a2 = RangeSetBlaze::from_iter([11..=15]);
    let a01 = &a0 | &a1;
    let not_a01 = !&a01;
    let a = &a01 ^ &a2;
    let b = a01.ranges() ^ a2.ranges();
    let c = !not_a01.ranges() ^ a2.ranges();
    let d = (a0.ranges() | a1.ranges()) ^ a2.ranges();
    let e = a01.ranges().symmetric_difference(a2.ranges());
    assert!(a.ranges().equal(b));
    assert!(a.ranges().equal(c));
    assert!(a.ranges().equal(d));
    assert!(a.ranges().equal(e));
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
#[allow(
    clippy::zero_repeat_side_effects,
    clippy::too_many_lines,
    clippy::similar_names
)]
fn empty_it() {
    use range_set_blaze::RangesIter;

    let universe = RangeSetBlaze::from_iter([0u8..=255]);
    let universe = universe.ranges();
    let arr: [u8; 0] = [];
    let a0 = RangeSetBlaze::<u8>::from_iter(arr);
    assert!(!(a0.ranges()).equal(universe.clone()));
    assert!((!a0).ranges().equal(universe));
    let _a0 = RangeSetBlaze::from_iter([0..=0; 0]);
    let _a = RangeSetBlaze::<i32>::new();

    let a_iter = std::iter::empty::<i32>();
    let a = a_iter.collect::<RangeSetBlaze<i32>>();
    let b = RangeSetBlaze::from_iter([0i32; 0]);
    let mut c3 = a.clone();
    let mut c5 = a.clone();

    let c0 = (&a).bitor(&b);
    let c1a = &a | &b;
    let c1b = &a | b.clone();
    let c1c = a.clone() | &b;
    let c1d = a.clone() | b.clone();
    let c2: RangeSetBlaze<_> = (a.ranges() | b.ranges()).into_range_set_blaze();
    c3.append(&mut b.clone());
    c5.extend(b);

    let answer = RangeSetBlaze::from_iter([0; 0]);
    assert_eq!(&c0, &answer);
    assert_eq!(&c1a, &answer);
    assert_eq!(&c1b, &answer);
    assert_eq!(&c1c, &answer);
    assert_eq!(&c1d, &answer);
    assert_eq!(&c2, &answer);
    assert_eq!(&c3, &answer);
    assert_eq!(&c5, &answer);

    let a_iter = std::iter::empty::<i32>();
    let a = a_iter.collect::<RangeSetBlaze<i32>>();
    let b = RangeSetBlaze::from_iter([0; 0]);
    let c0 = a.ranges() | b.ranges();
    let c1 = [a.ranges(), b.ranges()].union();
    let c_list2: [RangesIter<'_, i32>; 0] = [];
    let c2 = c_list2.clone().union();
    let c3 = union_dyn!(a.ranges(), b.ranges());
    let c4 = c_list2.map(DynSortedDisjoint::new).union();

    let answer = RangeSetBlaze::from_iter([0; 0]);
    assert!(c0.equal(answer.ranges()));
    assert!(c1.equal(answer.ranges()));
    assert!(c2.equal(answer.ranges()));
    assert!(c3.equal(answer.ranges()));
    assert!(c4.equal(answer.ranges()));

    let c0 = !(a.ranges() & b.ranges());
    let c1 = ![a.ranges(), b.ranges()].intersection();
    let c_list2: [RangesIter<'_, i32>; 0] = [];
    let c2 = !!c_list2.clone().intersection();
    let c3 = !intersection_dyn!(a.ranges(), b.ranges());
    let c4 = !!c_list2.map(DynSortedDisjoint::new).intersection();

    let answer = !RangeSetBlaze::from_iter([0; 0]);
    assert!(c0.equal(answer.ranges()));
    assert!(c1.equal(answer.ranges()));
    assert!(c2.equal(answer.ranges()));
    assert!(c3.equal(answer.ranges()));
    assert!(c4.equal(answer.ranges()));
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
#[allow(clippy::reversed_empty_ranges)]
fn tricky_case1() {
    let a = RangeSetBlaze::from_iter([1..=0]);
    let b = RangeSetBlaze::from_iter([2..=1]);
    assert_eq!(a, b);
    assert!(a.ranges().equal(b.ranges()));
    assert_eq!(a.ranges().len(), 0);
    assert_eq!(a.ranges().len(), b.ranges().len());
    let a = RangeSetBlaze::from_iter([i32::MIN..=i32::MAX]);
    println!("tc1 '{a}'");
    assert_eq!(
        i128::from(a.len()),
        i128::from(i32::MAX) - i128::from(i32::MIN) + 1
    );
    let a = !RangeSetBlaze::from_iter([1..=0]);
    println!("tc1 '{a}'");
    assert_eq!(
        i128::from(a.len()),
        i128::from(i32::MAX) - i128::from(i32::MIN) + 1
    );

    let a = !RangeSetBlaze::from_iter([1i128..=0]);
    println!("tc1 '{a}', {}", a.len());
    assert_eq!(a.len(), UIntPlusOne::MaxPlusOne);
    let a = !RangeSetBlaze::from_iter([1u128..=0]);
    println!("tc1 '{a}', {}", a.len());
    assert_eq!(a.len(), UIntPlusOne::MaxPlusOne);
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
fn tricky_case2() {
    let _a = RangeSetBlaze::from_iter([-1..=i128::MAX]);
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
fn tricky_case3() {
    let _a = RangeSetBlaze::from_iter([0..=u128::MAX]);
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
#[allow(unused_assignments)]
fn constructors() {
    // #9: new
    let mut range_set_blaze;
    range_set_blaze = RangeSetBlaze::<i32>::new();
    // #10 collect / from_iter T
    range_set_blaze = [1, 5, 6, 5].into_iter().collect();
    range_set_blaze = RangeSetBlaze::from_iter([1, 5, 6, 5]);
    // #11 into / from array T
    range_set_blaze = [1, 5, 6, 5].into();
    range_set_blaze = RangeSetBlaze::from_iter([1, 5, 6, 5]);
    //#13 collect / from_iter range
    range_set_blaze = [5..=6, 1..=5].into_iter().collect();
    range_set_blaze = RangeSetBlaze::from_iter([5..=6, 1..=5]);
    // #16 into / from iter (T,T) + SortedDisjoint
    range_set_blaze = range_set_blaze.ranges().into_range_set_blaze();
    range_set_blaze = RangeSetBlaze::from_sorted_disjoint(range_set_blaze.ranges());
}

#[cfg(target_os = "linux")]
#[test]
fn debug_k_play() {
    let mut c = Criterion::default();
    k_play(&mut c);
}

#[cfg(target_os = "linux")]
fn k_play(c: &mut Criterion) {
    let range = 0..=9_999_999;
    let range_len = 1_000;
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
                    let _answer: RangeSetBlaze<_> = sets.intersection().into_range_set_blaze();
                },
                BatchSize::SmallInput,
            );
        });
    }
    group.finish();
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
#[allow(clippy::unwrap_used)]
fn data_gen() {
    let range = -10_000_000i32..=10_000_000;
    let range_len = 1000;
    let coverage_goal = 0.75;
    let k = 100;

    for how in [How::None, How::Union, How::Intersection] {
        let mut option_range_int_set: Option<RangeSetBlaze<_>> = None;
        for seed in 0..k as u64 {
            let r2: RangeSetBlaze<i32> = MemorylessRange::new(
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

#[cfg(not(target_arch = "wasm32"))]
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

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn ingest_clumps_base() {
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
        let range_set_blaze: RangeSetBlaze<_> =
            MemorylessRange::new(&mut rng, range_len, range.clone(), coverage_goal, k, how)
                .collect();

        let range_count_no_dups = range_set_blaze.ranges_len();
        let item_count_no_dups = range_set_blaze.len();
        let fraction_value = fraction(&range_set_blaze, &range);
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
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
fn doc_test_insert1() {
    let mut set = RangeSetBlaze::new();

    assert!(set.insert(2));
    assert!(!set.insert(2));
    assert_eq!(set.len(), 1u64);
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
fn doc_test_len() {
    let mut v = RangeSetBlaze::new();
    assert_eq!(v.len(), 0u64);
    v.insert(1);
    assert_eq!(v.len(), 1u64);

    let v = RangeSetBlaze::from_iter([
        -170_141_183_460_469_231_731_687_303_715_884_105_728i128..=10,
        -10..=170_141_183_460_469_231_731_687_303_715_884_105_726,
    ]);
    assert_eq!(
        v.len(),
        UIntPlusOne::UInt(340_282_366_920_938_463_463_374_607_431_768_211_455)
    );
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
fn test_pops() {
    let mut set = RangeSetBlaze::from_iter([1..=2, 4..=5, 10..=11]);
    let len = set.len();
    assert_eq!(set.pop_first(), Some(1));
    assert_eq!(set.len(), len - 1u64);
    assert_eq!(set, RangeSetBlaze::from_iter([2..=2, 4..=5, 10..=11]));
    assert_eq!(set.pop_last(), Some(11));
    println!("{set:#?}");
    assert_eq!(set, RangeSetBlaze::from_iter([2..=2, 4..=5, 10..=10]));
    assert_eq!(set.len(), len - 2u64);
    assert_eq!(set.pop_last(), Some(10));
    assert_eq!(set.len(), len - 3u64);
    assert_eq!(set, RangeSetBlaze::from_iter([2..=2, 4..=5]));
    assert_eq!(set.pop_first(), Some(2));
    assert_eq!(set.len(), len - 4u64);
    assert_eq!(set, RangeSetBlaze::from_iter([4..=5]));
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
fn eq() {
    assert!(RangeSetBlaze::from_iter([0, 2]) > RangeSetBlaze::from_iter([0, 1]));
    assert!(RangeSetBlaze::from_iter([0, 2]) > RangeSetBlaze::from_iter([0..=100]));
    assert!(RangeSetBlaze::from_iter([2..=2]) > RangeSetBlaze::from_iter([1..=2]));
    for use_0 in [false, true] {
        for use_1 in [false, true] {
            for use_2 in [false, true] {
                for use_3 in [false, true] {
                    for use_4 in [false, true] {
                        for use_5 in [false, true] {
                            let mut a = RangeSetBlaze::new();
                            let mut b = RangeSetBlaze::new();
                            if use_0 {
                                a.insert(0);
                            }
                            if use_1 {
                                a.insert(1);
                            }
                            if use_2 {
                                a.insert(2);
                            }
                            if use_3 {
                                b.insert(0);
                            }
                            if use_4 {
                                b.insert(1);
                            }
                            if use_5 {
                                b.insert(2);
                            }
                            let a2 = BTreeSet::from_iter(&a);
                            let b2 = BTreeSet::from_iter(&b);
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
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
fn insert2() {
    let set = RangeSetBlaze::from_iter([1..=2, 4..=5, 10..=20, 30..=30]);
    for insert in 0..=31 {
        println!("inserting  {insert}");
        let mut a = set.clone();
        let mut a2 = BTreeSet::from_iter(&a);
        let b2 = a2.insert(insert);
        let b = a.insert(insert);
        assert_eq!(a, RangeSetBlaze::from_iter(&a2));
        assert_eq!(b, b2);
    }
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
fn remove() {
    let mut set = RangeSetBlaze::from_iter([1..=2, 4..=5, 10..=11]);
    let len = set.len();
    assert!(set.remove(4));
    assert_eq!(set.len(), len - 1u64);
    assert_eq!(set, RangeSetBlaze::from_iter([1..=2, 5..=5, 10..=11]));
    assert!(!set.remove(4));
    assert_eq!(set.len(), len - 1u64);
    assert_eq!(set, RangeSetBlaze::from_iter([1..=2, 5..=5, 10..=11]));
    assert!(set.remove(5));
    assert_eq!(set.len(), len - 2u64);
    assert_eq!(set, RangeSetBlaze::from_iter([1..=2, 10..=11]));

    let mut set = RangeSetBlaze::from_iter([1..=2, 4..=5, 10..=100, 1000..=1000]);
    let len = set.len();
    assert!(!set.remove(0));
    assert_eq!(set.len(), len);
    assert!(!set.remove(3));
    assert_eq!(set.len(), len);
    assert!(set.remove(2));
    assert_eq!(set.len(), len - 1u64);
    assert!(set.remove(1000));
    assert_eq!(set.len(), len - 2u64);
    assert!(set.remove(10));
    assert_eq!(set.len(), len - 3u64);
    assert!(set.remove(50));
    assert_eq!(set.len(), len - 4u64);
    assert_eq!(
        set,
        RangeSetBlaze::from_iter([1..=1, 4..=5, 11..=49, 51..=100])
    );
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
fn remove2() {
    let set = RangeSetBlaze::from_iter([1..=2, 4..=5, 10..=20, 30..=30]);
    for remove in 0..=31 {
        println!("removing  {remove}");
        let mut a = set.clone();
        let mut a2 = BTreeSet::from_iter(&a);
        let b2 = a2.remove(&remove);
        let b = a.remove(remove);
        assert_eq!(a, RangeSetBlaze::from_iter(&a2));
        assert_eq!(b, b2);
    }
    let set = RangeSetBlaze::new();
    for remove in 0..=0 {
        println!("removing  {remove}");
        let mut a = set.clone();
        let mut a2 = BTreeSet::from_iter(&a);
        let b2 = a2.remove(&remove);
        let b = a.remove(remove);
        assert_eq!(a, RangeSetBlaze::from_iter(&a2));
        assert_eq!(b, b2);
    }
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
fn split_off() {
    let set = RangeSetBlaze::from_iter([1..=2, 4..=5, 10..=20, 30..=30]);
    for split in 0..=31 {
        println!("splitting at {split}");
        let mut a = set.clone();
        let mut a2 = BTreeSet::from_iter(&a);
        let b2 = a2.split_off(&split);
        let b = a.split_off(split);
        assert_eq!(a, RangeSetBlaze::from_iter(&a2));
        assert_eq!(b, RangeSetBlaze::from_iter(&b2));
    }
    let set = RangeSetBlaze::new();
    for split in 0..=0 {
        println!("splitting at {split}");
        let mut a = set.clone();
        let mut a2 = BTreeSet::from_iter(&a);
        let b2 = a2.split_off(&split);
        let b = a.split_off(split);
        assert_eq!(a, RangeSetBlaze::from_iter(&a2));
        assert_eq!(b, RangeSetBlaze::from_iter(&b2));
    }
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
fn retrain() {
    let mut set = RangeSetBlaze::from_iter([1..=6]);
    // Keep only the even numbers.
    set.retain(|k| k % 2 == 0);
    assert_eq!(set, RangeSetBlaze::from_iter([2, 4, 6]));
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
fn ranges_retain() {
    let mut set = RangeSetBlaze::from_iter([1..=6, 12..=20]);
    // Keep only the even numbers.
    set.ranges_retain(|k| k.start() % 2 == 0);
    assert_eq!(set, RangeSetBlaze::from_iter([12..=20]));
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
fn sync_and_send() {
    fn assert_sync_and_send<S: Sync + Send>() {}
    assert_sync_and_send::<RangeSetBlaze<i32>>();
}

#[cfg(not(target_arch = "wasm32"))]
fn fraction<T: Integer>(range_int_set: &RangeSetBlaze<T>, range: &RangeInclusive<T>) -> f64 {
    T::safe_len_to_f64_lossy(range_int_set.len()) / T::safe_len_to_f64_lossy(T::safe_len(range))
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
#[allow(clippy::unwrap_used)]
fn example_3() {
    let line = "chr15   29370   37380   29370,32358,36715   30817,32561,37380";

    // split the line on white space
    let mut iter = line.split_whitespace();
    let chrom = iter.next().unwrap();

    // Parse the start and end of the transcription region into a RangeSetBlaze
    let trans_start: i32 = iter.next().unwrap().parse().unwrap();
    let trans_end: i32 = iter.next().unwrap().parse().unwrap();
    let trans = RangeSetBlaze::from_iter([trans_start..=trans_end]);
    assert_eq!(trans, RangeSetBlaze::from_iter([29370..=37380]));

    // Parse the start and end of the exons into a RangeSetBlaze
    let exon_starts = iter.next().unwrap().split(',').map(str::parse::<i32>);
    let exon_ends = iter.next().unwrap().split(',').map(str::parse::<i32>);
    let exon_ranges = exon_starts
        .zip(exon_ends)
        .map(|(s, e)| s.unwrap()..=e.unwrap());
    let exons = exon_ranges.collect::<RangeSetBlaze<_>>();
    assert_eq!(
        exons,
        RangeSetBlaze::from_iter([29370..=30817, 32358..=32561, 36715..=37380])
    );

    // Use 'set subtraction' to find the introns
    let intron = trans - exons;
    assert_eq!(
        intron,
        RangeSetBlaze::from_iter([30818..=32357, 32562..=36714])
    );
    for range in intron.ranges() {
        let (start, end) = range.into_inner();
        println!("{chrom}\t{start}\t{end}");
    }
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
fn trick_dyn() {
    let bad = [1..=2, 0..=5];
    // let u = union_dyn!(bad.iter().cloned());
    let good = RangeSetBlaze::from_iter(bad);
    let _u = union_dyn!(good.ranges());
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
fn multiway2() {
    use range_set_blaze::MultiwaySortedDisjoint;

    let a = RangeSetBlaze::from_iter([1..=6, 8..=9, 11..=15]);
    let b = RangeSetBlaze::from_iter([5..=13, 18..=29]);
    let c = RangeSetBlaze::from_iter([25..=100]);

    let union = [a.ranges(), b.ranges(), c.ranges()].union();
    assert_eq!(union.into_string(), "1..=15, 18..=100");

    let union = MultiwaySortedDisjoint::union([a.ranges(), b.ranges(), c.ranges()]);
    assert_eq!(union.into_string(), "1..=15, 18..=100");
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
fn check_sorted_disjoint() {
    use range_set_blaze::CheckSortedDisjoint;

    let a = CheckSortedDisjoint::new([1..=2, 5..=100]);
    let b = CheckSortedDisjoint::new([2..=6]);
    let c = a | b;

    assert_eq!(c.into_string(), "1..=100");
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
fn dyn_sorted_disjoint_example() {
    let a = RangeSetBlaze::from_iter([1u8..=6, 8..=9, 11..=15]);
    let b = RangeSetBlaze::from_iter([5..=13, 18..=29]);
    let c = RangeSetBlaze::from_iter([38..=42]);
    let union = [
        DynSortedDisjoint::new(a.ranges()),
        DynSortedDisjoint::new(!b.ranges()),
        DynSortedDisjoint::new(c.ranges()),
    ]
    .union();
    assert_eq!(union.into_string(), "0..=6, 8..=9, 11..=17, 30..=255");
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
fn not_iter_example() {
    let a = CheckSortedDisjoint::new([1u8..=2, 5..=100]);
    let b = !a;
    assert_eq!(b.into_string(), "0..=0, 3..=4, 101..=255");

    // Or, equivalently:
    let b = !CheckSortedDisjoint::new([1u8..=2, 5..=100]);
    assert_eq!(b.into_string(), "0..=0, 3..=4, 101..=255");
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
fn len_demo() {
    let len: <u8 as Integer>::SafeLen = RangeSetBlaze::from_iter([0u8..=255]).len();
    assert_eq!(len, 256);

    assert_eq!(<u8 as Integer>::safe_len(&(0..=255)), 256);
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
fn bitor() {
    let a = CheckSortedDisjoint::new([1..=1]);
    let b = RangeSetBlaze::from_iter([2..=2]).into_ranges();
    let union = core::ops::BitOr::bitor(a, b);
    assert_eq!(union.into_string(), "1..=2");

    let a = CheckSortedDisjoint::new([1..=1]);
    let b = CheckSortedDisjoint::new([2..=2]);
    let c = range_set_blaze::SortedDisjoint::union(a, b);
    assert_eq!(c.into_string(), "1..=2");

    let a = CheckSortedDisjoint::new([1..=1]);
    let b = CheckSortedDisjoint::new([2..=2]);
    let c = core::ops::BitOr::bitor(a, b);
    assert_eq!(c.into_string(), "1..=2");

    let a = CheckSortedDisjoint::new([1..=1]);
    let b = RangeSetBlaze::from_iter([2..=2]).into_ranges();
    let c = range_set_blaze::SortedDisjoint::union(a, b);
    assert_eq!(c.into_string(), "1..=2");
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
fn range_set_blaze_constructors() {
    // Create an empty set with 'new' or 'default'.
    let a0 = RangeSetBlaze::<i32>::new();
    let a1 = RangeSetBlaze::<i32>::default();
    assert!(a0 == a1 && a0.is_empty());

    // 'from_iter'/'collect': From an iterator of integers.
    // Duplicates and out-of-order elements are fine.
    let a0 = RangeSetBlaze::from_iter([3, 2, 1, 100, 1]);
    let a1: RangeSetBlaze<i32> = [3, 2, 1, 100, 1].into_iter().collect();
    assert!(a0 == a1 && a0.to_string() == "1..=3, 100..=100");

    // 'from_iter'/'collect': From an iterator of inclusive ranges, start..=end.
    // Overlapping, out-of-order, and empty ranges are fine.
    #[allow(clippy::reversed_empty_ranges)]
    let a0 = RangeSetBlaze::from_iter([1..=2, 2..=2, -10..=-5, 1..=0]);
    #[allow(clippy::reversed_empty_ranges)]
    let a1: RangeSetBlaze<i32> = [1..=2, 2..=2, -10..=-5, 1..=0].into_iter().collect();
    assert!(a0 == a1 && a0.to_string() == "-10..=-5, 1..=2");

    // If we know the ranges are sorted and disjoint, we can use 'from'/'into'.
    let a0 = RangeSetBlaze::from_sorted_disjoint(CheckSortedDisjoint::new([-10..=-5, 1..=2]));
    let a1: RangeSetBlaze<i32> = CheckSortedDisjoint::new([-10..=-5, 1..=2]).into_range_set_blaze();
    assert!(a0 == a1 && a0.to_string() == "-10..=-5, 1..=2");

    // For compatibility with `BTreeSet`, we also support
    // 'from'/'into' from arrays of integers.
    let a0 = RangeSetBlaze::from([3, 2, 1, 100, 1]);
    let a1: RangeSetBlaze<i32> = [3, 2, 1, 100, 1].into();
    assert!(a0 == a1 && a0.to_string() == "1..=3, 100..=100");
}

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
#[cfg(feature = "from_slice")]
#[allow(unexpected_cfgs)]
fn print_features() {
    println!("feature\tcould\tare");
    syntactic_for! { feature in [
        "aes",
        "pclmulqdq",
        "rdrand",
        "rdseed",
        "tsc",
        "mmx",
        "sse",
        "sse2",
        "sse3",
        "ssse3",
        "sse4.1",
        "sse2",
        "sse4a",
        "sha",
        "avx",
        "avx2",
        "avx512f",
        "avx512cd",
        "avx512er",
        "avx512pf",
        "avx512bw",
        "avx512dq",
        "avx512vl",
        "avx512ifma",
        "avx512vbmi",
        "avx512vpopcntdq",
        "fma",
        "bmi1",
        "bmi2",
        "abm",
        "lzcnt",
        "tbm",
        "popcnt",
        "fxsr",
        "xsave",
        "xsaveopt",
        "xsaves",
        "xsavec",
        ] {$(
            println!("{}\t{}\t{}",$feature,is_x86_feature_detected!($feature),cfg!(target_feature = $feature));

    )*}};
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
#[cfg(feature = "from_slice")]
fn from_slice_all_types() {
    syntactic_for! { ty in [i8, u8] {
        $(
            println!("ty={:#?}",size_of::<$ty>() * 8);
            let v: Vec<$ty> = (0..=127).collect();
            let a2 = RangeSetBlaze::from_slice(&v);
            assert!(a2.to_string() == "0..=127");
        )*
    }};

    syntactic_for! { ty in [i16, u16, i32, u32, i64, u64, isize, usize, i128, u128] {
        $(
            println!("ty={:#?}",size_of::<$ty>() * 8);
            let v: Vec<$ty> = (0..=5000).collect();
            let a2 = RangeSetBlaze::from_slice(&v);
            assert!(a2.to_string() == "0..=5000");
        )*
    }};
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
#[cfg(feature = "from_slice")]
fn range_set_blaze_slice_constructor() {
    print_features();
    let k = 1;
    let average_width = 1000;
    let coverage_goal = 0.10;
    let how = How::None;
    let seed = 0;

    #[allow(clippy::single_element_loop)]
    for iter_len in [1000, 1500, 1750, 2000, 10_000, 1_000_000] {
        let (range_len, range) =
            range_set_blaze::test_util::width_to_range_u32(iter_len, average_width, coverage_goal);

        let vec: Vec<u32> = MemorylessIter::new(
            &mut StdRng::seed_from_u64(seed),
            range_len,
            range.clone(),
            coverage_goal,
            k,
            how,
        )
        .collect();
        let b0 = RangeSetBlaze::from_iter(&vec);
        let b1 = RangeSetBlaze::from_slice(&vec);
        if b0 != b1 {
            println!(
                "{iter_len} error: b0={b0:#?}, b1={b1:#?}, diff={:#?}",
                &b0 ^ &b1
            );
        }
        assert!(b0 == b1);
    }

    let v: Vec<i32> = (100..=150).collect();
    let a2 = RangeSetBlaze::from_slice(v);
    assert!(a2.to_string() == "100..=150");

    // For compatibility with `BTreeSet`, we also support
    // 'from'/'into' from arrays of integers.
    let a0 = RangeSetBlaze::from([3, 2, 1, 100, 1]);
    let a1: RangeSetBlaze<i32> = [3, 2, 1, 100, 1].into();
    assert!(a0 == a1 && a0.to_string() == "1..=3, 100..=100");

    #[allow(clippy::needless_borrows_for_generic_args)]
    let a2 = RangeSetBlaze::from_slice(&[3, 2, 1, 100, 1]);
    assert!(a0 == a2 && a2.to_string() == "1..=3, 100..=100");

    let a2 = RangeSetBlaze::from_slice([3, 2, 1, 100, 1]);
    assert!(a0 == a2 && a2.to_string() == "1..=3, 100..=100");
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
fn range_set_blaze_operators() {
    let a = RangeSetBlaze::from_iter([1..=2, 5..=100]);
    let b = RangeSetBlaze::from_iter([2..=6]);

    // Union of two 'RangeSetBlaze's.
    let result = &a | &b;
    // Alternatively, we can take ownership via 'a | b'.
    assert_eq!(result.to_string(), "1..=100");

    // Intersection of two 'RangeSetBlaze's.
    let result = &a & &b; // Alternatively, 'a & b'.
    assert_eq!(result.to_string(), "2..=2, 5..=6");

    // Set difference of two 'RangeSetBlaze's.
    let result = &a - &b; // Alternatively, 'a - b'.
    assert_eq!(result.to_string(), "1..=1, 7..=100");

    // Symmetric difference of two 'RangeSetBlaze's.
    let result = &a ^ &b; // Alternatively, 'a ^ b'.
    assert_eq!(result.to_string(), "1..=1, 3..=4, 7..=100");

    // complement of a 'RangeSetBlaze'.
    let result = !&a; // Alternatively, '!a'.
    assert_eq!(
        result.to_string(),
        "-2147483648..=0, 3..=4, 101..=2147483647"
    );

    // Multiway union of 'RangeSetBlaze's.
    let c = RangeSetBlaze::from_iter([2..=2, 6..=200]);
    let result = [&a, &b, &c].union();
    assert_eq!(result.to_string(), "1..=200");

    // Multiway intersection of 'RangeSetBlaze's.
    let result = [&a, &b, &c].intersection();
    assert_eq!(result.to_string(), "2..=2, 6..=6");

    // Combining multiple operations
    let result0 = &a - (&b | &c); // Creates a temporary 'RangeSetBlaze'.

    // Alternatively, we can use the 'SortedDisjoint' API and avoid the temporary 'RangeSetBlaze'.
    let result1 = RangeSetBlaze::from_sorted_disjoint(a.ranges() - (b.ranges() | c.ranges()));
    assert!(result0 == result1 && result0.to_string() == "1..=1");
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
fn sorted_disjoint_constructors() {
    // RangeSetBlaze's .ranges(), .range().clone() and .into_ranges()
    let r = RangeSetBlaze::from_iter([3, 2, 1, 100, 1]);
    let a = r.ranges();
    let b = a.clone();
    assert!(a.into_string() == "1..=3, 100..=100");
    assert!(b.into_string() == "1..=3, 100..=100");
    //    'into_ranges' takes ownership of the 'RangeSetBlaze'
    let a = RangeSetBlaze::from_iter([3, 2, 1, 100, 1]).into_ranges();
    assert!(a.into_string() == "1..=3, 100..=100");

    // CheckSortedDisjoint -- unsorted or overlapping input ranges will cause a panic.
    let a = CheckSortedDisjoint::new([1..=3, 100..=100]);
    assert!(a.into_string() == "1..=3, 100..=100");

    // tee of a SortedDisjoint iterator
    let _a = CheckSortedDisjoint::new([1..=3, 100..=100]);

    // DynamicSortedDisjoint of a SortedDisjoint iterator
    let a = CheckSortedDisjoint::new([1..=3, 100..=100]);
    let b = DynSortedDisjoint::new(a);
    assert!(b.into_string() == "1..=3, 100..=100");
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
fn iterator_example() {
    #[must_use = "iterators are lazy and do nothing unless consumed"]
    struct OrdinalWeekends2023 {
        next_range: RangeInclusive<i32>,
    }
    impl SortedStarts<i32> for OrdinalWeekends2023 {}
    impl SortedDisjoint<i32> for OrdinalWeekends2023 {}

    impl OrdinalWeekends2023 {
        fn new() -> Self {
            Self { next_range: 0..=1 }
        }
    }
    impl FusedIterator for OrdinalWeekends2023 {}
    impl Iterator for OrdinalWeekends2023 {
        type Item = RangeInclusive<i32>;
        fn next(&mut self) -> Option<Self::Item> {
            let (start, end) = self.next_range.clone().into_inner();
            if start > 365 {
                None
            } else {
                self.next_range = (start + 7)..=(end + 7);
                Some(start.max(1)..=end.min(365))
            }
        }
    }

    let weekends = OrdinalWeekends2023::new();
    let sept = CheckSortedDisjoint::new([244..=273]);
    let sept_weekdays = sept.intersection(weekends.complement());
    assert_eq!(
        sept_weekdays.into_string(),
        "244..=244, 247..=251, 254..=258, 261..=265, 268..=272"
    );
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
#[allow(clippy::tuple_array_conversions)]
fn sorted_disjoint_operators() {
    let a0 = RangeSetBlaze::from_iter([1..=2, 5..=100]);
    let b0 = RangeSetBlaze::from_iter([2..=6]);
    let c0 = RangeSetBlaze::from_iter([2..=2, 6..=200]);

    // 'union' method and 'to_string' method
    let (a, b) = (a0.ranges(), b0.ranges());
    let result = a.union(b);
    assert_eq!(result.into_string(), "1..=100");

    // '|' operator and 'equal' method
    let (a, b) = (a0.ranges(), b0.ranges());
    let result = a | b;
    assert!(result.equal(CheckSortedDisjoint::new([1..=100])));

    // multiway union of same type
    let (a, b, c) = (a0.ranges(), b0.ranges(), c0.ranges());
    let result = [a, b, c].union();
    assert_eq!(result.into_string(), "1..=200");

    // multiway union of different types
    let (a, b, c) = (a0.ranges(), b0.ranges(), c0.ranges());
    let result = union_dyn!(a, b, !c);
    assert_eq!(result.into_string(), "-2147483648..=100, 201..=2147483647");
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
fn range_example() {
    let mut set = RangeSetBlaze::new();
    set.insert(3);
    set.insert(5);
    set.insert(8);
    for elem in &(&set & RangeSetBlaze::from_iter([4..=8])) {
        println!("{elem}");
    }

    let intersection = &set & RangeSetBlaze::from_iter([4..=i32::MAX]);
    assert_eq!(Some(5), intersection.first());
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
fn range_test() {
    use core::ops::Bound::Included;
    use range_set_blaze::RangeSetBlaze;

    let mut set = RangeSetBlaze::new();
    set.insert(3);
    set.insert(5);
    set.insert(8);
    for elem in set.range((Included(4), Included(8))) {
        println!("{elem}");
    }
    assert_eq!(Some(5), set.range(4..).next());
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
#[allow(clippy::bool_assert_comparison)]
fn is_subset_check() {
    let sup = CheckSortedDisjoint::new([1..=3]);
    let set: CheckSortedDisjoint<i32, _> = [].into();
    assert_eq!(set.is_subset(sup), true);

    let sup = CheckSortedDisjoint::new([1..=3]);
    let set = CheckSortedDisjoint::new([2..=2]);
    assert_eq!(set.is_subset(sup), true);

    let sup = CheckSortedDisjoint::new([1..=3]);
    let set = CheckSortedDisjoint::new([2..=2, 4..=4]);
    assert_eq!(set.is_subset(sup), false);
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
fn cmp_range_set_blaze() {
    let a = RangeSetBlaze::from_iter([1..=3, 5..=7]);
    let b = RangeSetBlaze::from_iter([2..=2]);
    assert!(a < b); // Lexicographic comparison
    assert!(b.is_subset(&a)); // Subset comparison

    // Lexicographic comparisons
    assert!(a <= b);
    assert!(b > a);
    assert!(b >= a);
    assert!(a != b);
    assert!(a == a);
    assert_eq!(a.cmp(&b), Ordering::Less);
    assert_eq!(a.partial_cmp(&b), Some(Ordering::Less));
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
fn cmp_btree_set_int() {
    let a = BTreeSet::from([1, 2, 3, 5, 6, 7]);
    let b = BTreeSet::from([2]);
    assert!(a < b); // Lexicographic comparison
    assert!(b.is_subset(&a)); // Subset comparison

    // Lexicographic comparisons
    assert!(a <= b);
    assert!(b > a);
    assert!(b >= a);
    assert!(a != b);
    assert!(a == a);
    assert_eq!(a.cmp(&b), Ordering::Less);
    assert_eq!(a.partial_cmp(&b), Some(Ordering::Less));
}

#[cfg(not(target_arch = "wasm32"))]
#[test] // wasm skips this Criterion related test
fn run_rangemap_crate() {
    let mut rng = StdRng::seed_from_u64(0);
    let range_len = 1_000_000;

    let vec_range: Vec<_> =
        MemorylessRange::new(&mut rng, range_len, 0..=99_999_999, 0.01, 1, How::None).collect();

    let _start = Instant::now();

    let rangemap_set0 = &vec_range
        .iter()
        .cloned()
        .collect::<rangemap::RangeInclusiveSet<_>>();
    let _rangemap_set1 = &rangemap_set0
        .iter()
        .cloned()
        .collect::<rangemap::RangeInclusiveSet<_>>();
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
fn from_iter_coverage() {
    let arr = [1..=2, 2..=2, -10..=-5];
    let a0 = RangeSetBlaze::from_iter(&arr);
    let a1: RangeSetBlaze<i32> = arr.iter().collect();
    assert!(a0 == a1 && a0.to_string() == "-10..=-5, 1..=2");
}

// fn _some_fn() {
//     let guaranteed = RangeSetBlaze::from_iter([1..=2, 3..=4, 5..=6]).into_ranges();
//     let _range_set_blaze = RangeSetBlaze::from_sorted_disjoint(guaranteed);
//     let not_guaranteed = [1..=2, 3..=4, 5..=6].into_iter();
//     let _range_set_blaze = RangeSetBlaze::from_sorted_disjoint(not_guaranteed);
// }

// fn _some_fn() {
//     let _integer_set = RangeSetBlaze::from_iter([1, 2, 3, 5]);
//     let _char_set = RangeSetBlaze::from_iter(['a', 'b', 'c', 'd']);
// }

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
#[allow(clippy::unwrap_used)]
fn print_first_complement_gap() {
    let a = CheckSortedDisjoint::new([-10i16..=0, 1000..=2000]);
    println!("{:?}", (!a).next().unwrap()); // prints -32768..=-11
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
fn multiway_failure_example() {
    use range_set_blaze::prelude::*;

    let a = RangeSetBlaze::from_iter([1..=6, 8..=9, 11..=15]);
    let b = RangeSetBlaze::from_iter([5..=13, 18..=29]);
    let c = RangeSetBlaze::from_iter([38..=42]);

    let _i0 = [a.ranges(), b.ranges(), c.ranges()].intersection();
    // let _i1 = [!a.ranges(), b.ranges(), c.ranges()].intersection();
    let _i2 = [
        DynSortedDisjoint::new(!a.ranges()),
        DynSortedDisjoint::new(b.ranges()),
        DynSortedDisjoint::new(c.ranges()),
    ]
    .intersection();
    let _i3 = intersection_dyn!(!a.ranges(), b.ranges(), c.ranges());
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
fn complement_sample() {
    let c = !RangeSetBlaze::from([0, 3, 4, 5, 10]);
    println!("{},{},{}", c.len(), c.ranges_len(), c);
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
#[cfg(feature = "rog_experimental")]
#[allow(deprecated)]
fn test_rog_functionality() {
    let a = RangeSetBlaze::from_iter([1..=6, 8..=9, 11..=15]);
    // case 1:
    for end in 7..=16 {
        println!("case 1: {:?}", a.rogs_range_slow(7..=end));
        assert_eq!(
            a.rogs_range_slow(7..=end),
            a.rogs_range(7..=end).collect::<Vec<_>>()
        );
    }
    // case 2:
    for end in 7..=16 {
        println!("case 2: {:?}", a.rogs_range_slow(4..=end));
        assert_eq!(
            a.rogs_range_slow(4..=end),
            a.rogs_range(4..=end).collect::<Vec<_>>()
        );
    }
    // case 3:
    for start in 11..=15 {
        for end in start..=15 {
            println!("case 3: {:?}", a.rogs_range_slow(start..=end));
            assert_eq!(
                a.rogs_range_slow(start..=end),
                a.rogs_range(start..=end).collect::<Vec<_>>()
            );
        }
    }
    // case 4:
    for end in -1..=16 {
        println!("case 4: {:?}", a.rogs_range_slow(-1..=end));
        assert_eq!(
            a.rogs_range_slow(-1..=end),
            a.rogs_range(-1..=end).collect::<Vec<_>>()
        );
    }
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
#[cfg(feature = "rog_experimental")]
#[allow(clippy::reversed_empty_ranges)]
#[allow(deprecated)]
#[should_panic(expected = "start must be less than or equal to end")]
fn test_rog_functionality_empty() {
    let a = RangeSetBlaze::from_iter([1..=6, 8..=9, 11..=15]);

    let _ = a.rogs_range(1..=0).collect::<Vec<_>>();
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
#[cfg(feature = "rog_experimental")]
#[allow(deprecated)]
fn test_rogs_get_functionality() {
    let a = RangeSetBlaze::from_iter([1..=6, 8..=9, 11..=15]);
    for value in 0..=16 {
        println!("{:?}", a.rogs_get_slow(value));
        assert_eq!(a.rogs_get_slow(value), a.rogs_get(value));
    }
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
#[cfg(feature = "rog_experimental")]
#[allow(deprecated)]
fn test_rog_repro1() {
    let a = RangeSetBlaze::from_iter([1u8..=6u8]);
    assert_eq!(
        a.rogs_range_slow(1..=7),
        a.rogs_range(1..=7).collect::<Vec<_>>()
    );
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
#[cfg(feature = "rog_experimental")]
#[allow(deprecated)]
fn test_rog_repro2() {
    let a = RangeSetBlaze::from_iter([1..=6, 8..=9, 11..=15]);
    assert_eq!(
        a.rogs_range_slow(4..=8),
        a.rogs_range(4..=8).collect::<Vec<_>>()
    );
}

#[cfg(not(target_arch = "wasm32"))] // This tests panics, so it's not suitable for wasm32.
#[test] // uses panics so can't be wasm
#[cfg(feature = "rog_experimental")]
#[allow(deprecated)]
fn test_rog_coverage1() {
    let a = RangeSetBlaze::from_iter([1u8..=6u8]);
    assert!(
        panic::catch_unwind(AssertUnwindSafe(
            || a.rogs_range((Bound::Excluded(&255), Bound::Included(&255)))
        ))
        .is_err()
    );
    assert!(panic::catch_unwind(AssertUnwindSafe(|| a.rogs_range(0..0))).is_err());
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
#[cfg(feature = "rog_experimental")]
#[allow(deprecated)]
fn test_rog_extremes_u8() {
    for a in [
        RangeSetBlaze::from_iter([1u8..=6u8]),
        RangeSetBlaze::from_iter([0u8..=6u8]),
        RangeSetBlaze::from_iter([200u8..=255u8]),
        RangeSetBlaze::from_iter([0u8..=255u8]),
        RangeSetBlaze::from_iter([0u8..=5u8, 20u8..=255]),
    ] {
        for start in 0u8..=255 {
            for end in start..=255 {
                println!("{start}..={end}");
                assert_eq!(
                    a.rogs_range_slow(start..=end),
                    a.rogs_range(start..=end).collect::<Vec<_>>()
                );
            }
        }
    }
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
#[cfg(feature = "rog_experimental")]
#[allow(deprecated)]
fn test_rog_get_extremes_u8() {
    for a in [
        RangeSetBlaze::from_iter([1u8..=6u8]),
        RangeSetBlaze::from_iter([0u8..=6u8]),
        RangeSetBlaze::from_iter([200u8..=255u8]),
        RangeSetBlaze::from_iter([0u8..=255u8]),
        RangeSetBlaze::from_iter([0u8..=5u8, 20u8..=255]),
    ] {
        for value in 0u8..=255 {
            println!("{value}");
            assert_eq!(a.rogs_get_slow(value), a.rogs_get(value));
        }
    }
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
#[cfg(feature = "rog_experimental")]
#[allow(deprecated, clippy::range_minus_one)]
fn test_rog_extremes_i128() {
    for a in [
        RangeSetBlaze::from_iter([1i128..=6i128]),
        RangeSetBlaze::from_iter([i128::MIN..=6]),
        RangeSetBlaze::from_iter([200..=i128::MAX - 1]),
        RangeSetBlaze::from_iter([i128::MIN..=i128::MAX - 1]),
        RangeSetBlaze::from_iter([i128::MIN..=5, 20..=i128::MAX - 1]),
    ] {
        for start in [i128::MIN, i128::MIN + 1, 0, i128::MAX - 2, i128::MAX - 1] {
            for end in [i128::MIN, i128::MIN + 1, 0, i128::MAX - 2, i128::MAX - 1] {
                if end < start {
                    continue;
                }
                println!("{start}..={end}");
                assert_eq!(
                    a.rogs_range_slow(start..=end),
                    a.rogs_range(start..=end).collect::<Vec<_>>()
                );
            }
        }
    }
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
#[cfg(feature = "rog_experimental")]
#[allow(deprecated, clippy::range_minus_one)]
fn test_rog_extremes_get_i128() {
    for a in [
        RangeSetBlaze::from_iter([1i128..=6i128]),
        RangeSetBlaze::from_iter([i128::MIN..=6]),
        RangeSetBlaze::from_iter([200..=i128::MAX - 1]),
        RangeSetBlaze::from_iter([i128::MIN..=i128::MAX - 1]),
        RangeSetBlaze::from_iter([i128::MIN..=5, 20..=i128::MAX - 1]),
    ] {
        for value in [i128::MIN, i128::MIN + 1, 0, i128::MAX - 2, i128::MAX - 1] {
            println!("{value}");
            assert_eq!(a.rogs_get_slow(value), a.rogs_get(value));
        }
    }
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
#[cfg(feature = "rog_experimental")]
#[allow(deprecated, clippy::range_minus_one)]
fn test_rog_should_fail_i128() {
    for a in [
        RangeSetBlaze::from_iter([1i128..=6i128]),
        RangeSetBlaze::from_iter([i128::MIN..=6]),
        RangeSetBlaze::from_iter([200..=i128::MAX - 1]),
        RangeSetBlaze::from_iter([i128::MIN..=i128::MAX - 1]),
        RangeSetBlaze::from_iter([i128::MIN..=5, 20..=i128::MAX - 1]),
    ] {
        for start in [i128::MIN, i128::MIN + 1, 0, i128::MAX - 1, i128::MAX] {
            for end in [i128::MIN, i128::MIN + 1, 0, i128::MAX - 1, i128::MAX] {
                if end < start {
                    continue;
                }
                println!("{start}..={end}");
                let slow =
                    panic::catch_unwind(AssertUnwindSafe(|| a.rogs_range_slow(start..=end))).ok();
                let fast = panic::catch_unwind(AssertUnwindSafe(|| {
                    a.rogs_range(start..=end).collect::<Vec<_>>()
                }))
                .ok();
                assert_eq!(slow, fast,);
            }
        }
    }
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
#[cfg(feature = "rog_experimental")]
#[allow(deprecated, clippy::range_minus_one)]
fn test_rog_get_should_fail_i128() {
    for a in [
        RangeSetBlaze::from_iter([1i128..=6i128]),
        RangeSetBlaze::from_iter([i128::MIN..=6]),
        RangeSetBlaze::from_iter([200..=i128::MAX - 1]),
        RangeSetBlaze::from_iter([i128::MIN..=i128::MAX - 1]),
        RangeSetBlaze::from_iter([i128::MIN..=5, 20..=i128::MAX - 1]),
    ] {
        for value in [i128::MIN, i128::MIN + 1, 0, i128::MAX - 1, i128::MAX] {
            println!("{value}");
            let slow = panic::catch_unwind(AssertUnwindSafe(|| a.rogs_get_slow(value))).ok();
            let fast = panic::catch_unwind(AssertUnwindSafe(|| a.rogs_get(value))).ok();
            assert_eq!(slow, fast,);
        }
    }
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
#[cfg(feature = "rog_experimental")]
#[allow(deprecated)]
fn test_rog_get_doc() {
    use crate::RangeSetBlaze;
    let range_set_blaze = RangeSetBlaze::from([1, 2, 3]);
    assert_eq!(range_set_blaze.rogs_get(2), Rog::Range(1..=3));
    assert_eq!(range_set_blaze.rogs_get(4), Rog::Gap(4..=2_147_483_647));
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
#[cfg(feature = "rog_experimental")]
#[allow(deprecated)]
fn test_rog_range_doc() {
    use core::ops::Bound::Included;

    let mut set = RangeSetBlaze::new();
    set.insert(3);
    set.insert(5);
    set.insert(6);
    for rog in set.rogs_range((Included(4), Included(8))) {
        println!("{rog:?}");
    } // prints: Gap(4..=4)\nRange(5..=6)\nGap(7..=8)

    assert_eq!(Some(Rog::Gap(4..=4)), set.rogs_range(4..).next());

    let a = RangeSetBlaze::from_iter([1..=6, 11..=15]);
    assert_eq!(
        a.rogs_range(-5..=8).collect::<Vec<_>>(),
        vec![Rog::Gap(-5..=0), Rog::Range(1..=6), Rog::Gap(7..=8)]
    );

    let empty = RangeSetBlaze::<u8>::new();
    assert_eq!(
        empty.rogs_range(..).collect::<Vec<_>>(),
        vec![Rog::Gap(0..=255)]
    );
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
#[allow(
    clippy::many_single_char_names,
    clippy::items_after_statements,
    clippy::cognitive_complexity
)]
fn test_every_sorted_disjoint_method() {
    use range_set_blaze::{IntoRangesIter, RangesIter};
    use range_set_blaze::{MapIntoRangesIter, MapRangesIter};

    use syntactic_for::syntactic_for;
    let c0 = RangeSetBlaze::from_iter([1..=2, 5..=100]);
    let c1: RangeMapBlaze<i32, &str> = RangeMapBlaze::from_iter([(1..=2, &"a"), (5..=100, &"b")]);

    macro_rules! fresh_instances {
        () => {{
            let a: CheckSortedDisjoint<_, _> = CheckSortedDisjoint::new([1..=2, 5..=100]);
            let b: DynSortedDisjoint<'_, _> =
                DynSortedDisjoint::new(RangeSetBlaze::from_iter([1..=2, 5..=100]).into_ranges());
            let c: IntoRangesIter<_> = c0.clone().into_ranges();
            let d: MapIntoRangesIter<_, _> = c1.clone().into_ranges();
            let e: MapRangesIter<'_, _, _> = c1.ranges();
            let f: NotIter<_, _> = !!CheckSortedDisjoint::new([1..=2, 5..=100]);
            let g: RangesIter<'_, _> = c0.ranges();
            let h: SymDiffIter<_, _> = c0.ranges() ^ c0.ranges() ^ c0.ranges();
            let i: UnionIter<_, _> = c0.ranges() | c0.ranges();

            (a, b, c, d, e, f, g, h, i)
        }};
    }

    let (a, b, c, d, e, f, g, h, i) = fresh_instances!();
    syntactic_for! { sd in [a, b, c, d, e, f, g, h, i] {$(
        let z = ! $sd;
        // println!("{:?}", z.into_string());
        assert!(z.equal(CheckSortedDisjoint::new([-2_147_483_648..=0, 3..=4, 101..=2_147_483_647])));
    )*}}

    let (a, b, c, d, e, f, g, h, i) = fresh_instances!();
    syntactic_for! { sd in [a, b, c, d, e, f, g, h, i] {$(
        let z = CheckSortedDisjoint::new([-1..=0, 50..=50,1000..=10_000]);
        let z = $sd | z;
        assert!(z.equal(CheckSortedDisjoint::new([-1..=2, 5..=100, 1000..=10000])));
    )*}}

    let (a, b, c, d, e, f, g, h, i) = fresh_instances!();
    syntactic_for! { sd in [a, b, c, d, e, f, g, h, i] {$(
        let z = CheckSortedDisjoint::new([-1..=0, 50..=50,1000..=10_000]);
        let z = $sd & z;
        assert!(z.equal(CheckSortedDisjoint::new([50..=50])));
    )*}}

    let (a, b, c, d, e, f, g, h, i) = fresh_instances!();
    syntactic_for! { sd in [a, b, c, d, e, f, g, h, i] {$(
        let z = CheckSortedDisjoint::new([-1..=0, 50..=50,1000..=10_000]);
        let z = $sd ^ z;
        assert!(z.equal(CheckSortedDisjoint::new([-1..=2, 5..=49, 51..=100, 1000..=10000])));
    )*}}

    let (a, b, c, d, e, f, g, h, i) = fresh_instances!();
    syntactic_for! { sd in [a, b, c, d, e, f, g, h, i] {$(
        let z = CheckSortedDisjoint::new([-1..=0, 50..=50,1000..=10_000]);
        let z = $sd - z;
        assert!(z.equal(CheckSortedDisjoint::new([1..=2, 5..=49, 51..=100])));
    )*}}

    // FusedIterator
    fn is_fused<T: FusedIterator>(_iter: T) {}
    let (a, b, c, d, e, f, g, h, i) = fresh_instances!();
    syntactic_for! { sd in [a, b, c, d, e, f, g, h, i] {$(
        is_fused::<_>($sd);
    )*}}
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
fn multiway3() {
    let a = RangeSetBlaze::from_iter([1..=6, 8..=9, 11..=15]).into_ranges();
    let b = RangeSetBlaze::from_iter([5..=13, 18..=29]).into_ranges();
    let c = RangeSetBlaze::from_iter([-100..=100]).into_ranges();
    assert_eq!([a, b, c].union().into_string(), "-100..=100");

    let a = RangeSetBlaze::from_iter([1..=6, 8..=9, 11..=15]).into_ranges();
    let b = RangeSetBlaze::from_iter([5..=13, 18..=29]).into_ranges();
    let c = RangeSetBlaze::from_iter([-100..=100]).into_ranges();
    assert_eq!(
        [a, b, c].intersection().into_string(),
        "5..=6, 8..=9, 11..=13"
    );

    let a = RangeSetBlaze::from_iter([1..=6, 8..=9, 11..=15]).into_ranges();
    let b = RangeSetBlaze::from_iter([5..=13, 18..=29]).into_ranges();
    let c = RangeSetBlaze::from_iter([-100..=100]).into_ranges();
    assert_eq!(
        [a, b, c].symmetric_difference().into_string(),
        "-100..=0, 5..=6, 8..=9, 11..=13, 16..=17, 30..=100"
    );
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
fn multiway4() {
    let a = RangeSetBlaze::from_iter([1..=6, 8..=9, 11..=15]);
    let b = RangeSetBlaze::from_iter([5..=13, 18..=29]);
    let c = RangeSetBlaze::from_iter([-100..=100]);
    assert_eq!([a, b, c].union(), RangeSetBlaze::from_iter([-100..=100]));

    let a = RangeSetBlaze::from_iter([1..=6, 8..=9, 11..=15]);
    let b = RangeSetBlaze::from_iter([5..=13, 18..=29]);
    let c = RangeSetBlaze::from_iter([-100..=100]);
    assert_eq!(
        [a, b, c].intersection(),
        RangeSetBlaze::from_iter([5..=6, 8..=9, 11..=13])
    );

    let a = RangeSetBlaze::from_iter([1..=6, 8..=9, 11..=15]);
    let b = RangeSetBlaze::from_iter([5..=13, 18..=29]);
    let c = RangeSetBlaze::from_iter([-100..=100]);
    assert_eq!(
        [a, b, c].symmetric_difference(),
        RangeSetBlaze::from_iter([-100..=0, 5..=6, 8..=9, 11..=13, 16..=17, 30..=100])
    );
}

// Test every function in the library that does a union like thing.

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
fn test_every_union() {
    // bitor x 4
    let a = RangeSetBlaze::from_iter([1..=6, 8..=9, 11..=15]);
    let b = RangeSetBlaze::from_iter([5..=13, 18..=29]);
    let c = &a | &b;
    assert_eq!(c, RangeSetBlaze::from_iter([1..=15, 18..=29]));
    let c = a | &b;
    let a = RangeSetBlaze::from_iter([1..=6, 8..=9, 11..=15]);
    assert_eq!(c, RangeSetBlaze::from_iter([1..=15, 18..=29]));
    let c = &a | b;
    assert_eq!(c, RangeSetBlaze::from_iter([1..=15, 18..=29]));
    let b = RangeSetBlaze::from_iter([5..=13, 18..=29]);
    let c = a | b;
    assert_eq!(c, RangeSetBlaze::from_iter([1..=15, 18..=29]));

    // bitor_assign x 2
    let mut a = RangeSetBlaze::from_iter([1..=6, 8..=9, 11..=15]);
    let b = RangeSetBlaze::from_iter([5..=13, 18..=29]);
    a |= &b;
    assert_eq!(a, RangeSetBlaze::from_iter([1..=15, 18..=29]));
    let mut a = RangeSetBlaze::from_iter([1..=6, 8..=9, 11..=15]);
    a |= b;
    assert_eq!(a, RangeSetBlaze::from_iter([1..=15, 18..=29]));

    // extend x 2
    let mut a = RangeSetBlaze::from_iter([1..=6, 8..=9, 11..=15]);
    let b = RangeSetBlaze::from_iter([5..=13, 18..=29]);
    a.extend(b.ranges());
    assert_eq!(a, RangeSetBlaze::from_iter([1..=15, 18..=29]));
    let mut a = RangeSetBlaze::from_iter([1..=6, 8..=9, 11..=15]);
    a.extend(b.iter());
    assert_eq!(a, RangeSetBlaze::from_iter([1..=15, 18..=29]));

    // append
    let mut a = RangeSetBlaze::from_iter([1..=6, 8..=9, 11..=15]);
    let mut b = RangeSetBlaze::from_iter([5..=13, 18..=29]);
    a.append(&mut b);
    assert_eq!(a, RangeSetBlaze::from_iter([1..=15, 18..=29]));
    assert!(b.is_empty());

    // // .union()
    let a = RangeSetBlaze::from_iter([1..=6, 8..=9, 11..=15]);
    let b = RangeSetBlaze::from_iter([5..=13, 18..=29]);
    let c = [&a, &b].union();
    assert_eq!(c, RangeSetBlaze::from_iter([1..=15, 18..=29]));

    // union_dyn!
    let c = union_dyn!(a.ranges(), b.ranges());
    assert!(c.equal(RangeSetBlaze::from_iter([1..=15, 18..=29]).ranges()));

    // [sorted disjoints].union()
    let c = [a.ranges(), b.ranges()].union();
    assert!(c.equal(RangeSetBlaze::from_iter([1..=15, 18..=29]).ranges()));
}

#[cfg(not(target_arch = "wasm32"))]
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

#[derive(Debug, PartialEq)]
struct BooleanVector(Vec<bool>);

impl BitAndAssign for BooleanVector {
    // `rhs` is the "right-hand side" of the expression `a &= b`.
    fn bitand_assign(&mut self, rhs: Self) {
        assert_eq!(self.0.len(), rhs.0.len());
        *self = Self(
            self.0
                .iter()
                .zip(rhs.0.iter())
                .map(|(x, y)| *x && *y)
                .collect(),
        );
    }
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
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
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
fn b_tree_set() {
    let a = [1, 2, 3].into_iter().collect::<BTreeSet<i32>>();
    let b = BTreeSet::from([2, 3, 4]);
    let mut c3 = a.clone();
    let mut c4 = a.clone();
    let mut c5 = a.clone();

    let c0 = a.bitor(&b);
    let c1 = &a | &b;
    let c2 = a.union(&b).copied().collect::<BTreeSet<_>>();
    c3.append(&mut b.clone());
    c4.extend(&b);
    c5.extend(b);

    let answer = BTreeSet::from([1, 2, 3, 4]);
    assert_eq!(&c0, &answer);
    assert_eq!(&c1, &answer);
    assert_eq!(&c2, &answer);
    assert_eq!(&c3, &answer);
    assert_eq!(&c4, &answer);
    assert_eq!(&c5, &answer);
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
#[allow(clippy::similar_names)]
fn range_set_blaze() {
    let a = [1, 2, 3].into_iter().collect::<RangeSetBlaze<i32>>();
    let b = RangeSetBlaze::from_iter([2, 3, 4]);
    let mut c3 = a.clone();
    let mut c5 = a.clone();

    let c0 = (&a).bitor(&b);
    let c1a = &a | &b;
    let c1b = &a | b.clone();
    let c1c = a.clone() | &b;
    let c1d = a.clone() | b.clone();
    let c2: RangeSetBlaze<_> = (a.ranges() | b.ranges()).into_range_set_blaze();
    c3.append(&mut b.clone());
    c5.extend(b);

    let answer = RangeSetBlaze::from_iter([1, 2, 3, 4]);
    assert_eq!(&c0, &answer);
    assert_eq!(&c1a, &answer);
    assert_eq!(&c1b, &answer);
    assert_eq!(&c1c, &answer);
    assert_eq!(&c1d, &answer);
    assert_eq!(&c2, &answer);
    assert_eq!(&c3, &answer);
    assert_eq!(&c5, &answer);
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
fn sorted_disjoint() {
    let a = [1, 2, 3].into_iter().collect::<RangeSetBlaze<i32>>();
    let b = RangeSetBlaze::from_iter([2, 3, 4]);

    let c0 = a.ranges() | b.ranges();
    let c1 = [a.ranges(), b.ranges()].union();
    let c2 = [a.ranges(), b.ranges()].union();
    let c3 = union_dyn!(a.ranges(), b.ranges());
    let c4 = [a.ranges(), b.ranges()].map(DynSortedDisjoint::new).union();

    let answer = RangeSetBlaze::from_iter([1, 2, 3, 4]);
    assert!(c0.equal(answer.ranges()));
    assert!(c1.equal(answer.ranges()));
    assert!(c2.equal(answer.ranges()));
    assert!(c3.equal(answer.ranges()));
    assert!(c4.equal(answer.ranges()));
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
#[allow(clippy::no_effect_underscore_binding)]
fn sorted_disjoint_ops() {
    let a = [1, 2, 3].into_iter().collect::<RangeSetBlaze<i32>>();
    let a = a.ranges();
    let b = !a.clone();
    let _c = !!b.clone();
    let _d = a.clone() | b.clone();
    let _e = !a.clone() | b.clone();
    let _f = !(!a.clone() | !b.clone());
    let _g = BitOr::bitor(a.clone().complement(), b.clone().complement()).complement();
    let _h = SortedDisjoint::union(a.clone().complement(), b.clone().complement()).complement();
    let _z = !(!a | !b);
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
#[allow(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss
)]
fn sub0() {
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

    // Signed sub may overflow, but casting preserves correct unsigned distance
    let before = 127i8.overflowing_sub(-128i8).0;
    let after = before as u8;
    println!("before: {before}, after: {after}");
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
fn understand_into_iter() {
    let btree_set = BTreeSet::from([1, 2, 3, 4, 5]);
    for i in &btree_set {
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
    for i in &arr {
        println!("{i}");
    }

    for i in &arr {
        println!("{i}");
    }

    // let rsi = RangeSetBlaze::from_iter(1..=5);
    // for i in &rsi {
    //     println!("{i}");
    // }
    // let len = rsi.len();
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
#[allow(clippy::cognitive_complexity, clippy::float_cmp)]
fn integer_coverage() {
    syntactic_for! { ty in [i8, u8, isize, usize,  i16, u16, i32, u32, i64, u64, isize, usize, i128, u128] {
        $(
            let len = <$ty as Integer>::SafeLen::one();
            let a = $ty::zero();
            assert_eq!($ty::safe_len_to_f64_lossy(len), 1.0);
            assert_eq!($ty::inclusive_end_from_start(a,len), a);
            assert_eq!($ty::start_from_inclusive_end(a,len), a);
            assert_eq!($ty::f64_to_safe_len_lossy(1.0), len);

        )*
    }};
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
fn lib_coverage_2() {
    let v = RangeSetBlaze::<u128>::new();
    v.contains(u128::MAX);
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
fn lib_coverage_3() {
    let mut v = RangeSetBlaze::<u128>::new();
    v.remove(u128::MAX);
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
fn lib_coverage_4() {
    let mut v = RangeSetBlaze::<u128>::new();
    let _ = v.split_off(u128::MAX);
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
fn lib_coverage_6() {
    syntactic_for! { ty in [i8, u8, isize, usize, i16, u16, i32, u32, i64, u64, isize, usize, i128, u128] {
        $(
            let mut a = RangeSetBlaze::<$ty>::from_iter([1..=3, 5..=7, 9..=120]);
            a.ranges_insert(2..=100);
            assert_eq!(a, RangeSetBlaze::from_iter([1..=120]));


        )*
    }};
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
fn not_iter_coverage_0() {
    let a = CheckSortedDisjoint::new([1..=2, 5..=100]);
    let n = !a;
    let p = n.clone();
    let m = p.clone();
    assert!(n.equal(m));
    assert!(format!("{p:?}").starts_with("NotIter"));
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
fn sorted_disjoint_coverage_0() {
    let a = CheckSortedDisjoint::<i32, _>::default();
    assert!(a.is_empty());

    let a = CheckSortedDisjoint::new([1..=2, 5..=100]);
    let b = CheckSortedDisjoint::new([1..=2, 5..=100]);
    assert!((a & b).equal(CheckSortedDisjoint::new([1..=2, 5..=100])));

    let a = CheckSortedDisjoint::new([1..=2, 5..=100]);
    let b = CheckSortedDisjoint::new([1..=2, 5..=100]);
    assert!((a - b).is_empty());

    let a = CheckSortedDisjoint::new([1..=2, 5..=100]);
    let b = CheckSortedDisjoint::new([1..=2, 5..=100]);
    assert!((a ^ b).is_empty());
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
#[should_panic(expected = "iterator cannot return Some after returning None")]
fn sorted_disjoint_coverage_1() {
    struct SomeAfterNone {
        a: i32,
    }
    impl FusedIterator for SomeAfterNone {}
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
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
#[should_panic(expected = "start must be less or equal to end")]
fn sorted_disjoint_coverage_2() {
    #[allow(clippy::reversed_empty_ranges)]
    let mut a = CheckSortedDisjoint::new([1..=0]);
    a.next();
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
#[should_panic(expected = "ranges must be disjoint")]
fn sorted_disjoint_coverage_3() {
    #[allow(clippy::reversed_empty_ranges)]
    let mut a = CheckSortedDisjoint::new([1..=1, 2..=2]);
    a.next();
    a.next();
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
fn sorted_disjoint_coverage_4() {
    #[allow(clippy::reversed_empty_ranges)]
    let mut a = CheckSortedDisjoint::new([0..=i128::MAX]);
    a.next();
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
fn sorted_disjoint_iterator_coverage_0() {
    let a = CheckSortedDisjoint::new([1..=2, 5..=100]);
    let b = CheckSortedDisjoint::new([1..=2, 5..=101]);
    assert!(b.is_superset(a));
}

#[cfg(not(target_arch = "wasm32"))]
type Element = i64;
#[cfg(not(target_arch = "wasm32"))]
type Reference = std::collections::BTreeSet<Element>;

#[cfg(not(target_arch = "wasm32"))]
#[quickcheck]
#[allow(clippy::needless_pass_by_value)]
fn disjoint(a: Reference, b: Reference) -> bool {
    let a_r = RangeSetBlaze::from_iter(&a);
    let b_r = RangeSetBlaze::from_iter(&b);
    a.is_disjoint(&b) == a_r.is_disjoint(&b_r)
}

#[cfg(not(target_arch = "wasm32"))]
#[quickcheck]
#[allow(clippy::needless_pass_by_value)]
fn subset(a: Reference, b: Reference) -> bool {
    let a_r = RangeSetBlaze::from_iter(&a);
    let b_r = RangeSetBlaze::from_iter(&b);
    a.is_subset(&b) == a_r.is_subset(&b_r)
}

#[cfg(not(target_arch = "wasm32"))]
#[quickcheck]
#[allow(clippy::needless_pass_by_value)]
fn superset(a: Reference, b: Reference) -> bool {
    let a_r = RangeSetBlaze::from_iter(&a);
    let b_r = RangeSetBlaze::from_iter(&b);
    a.is_superset(&b) == a_r.is_superset(&b_r)
}

/// just a helper to get good output when a check fails
#[cfg(not(target_arch = "wasm32"))]
#[allow(clippy::needless_pass_by_value)]
fn binary_op<E: Debug, R: Eq + Debug>(a: E, b: E, expected: R, actual: R) -> bool {
    let res = expected == actual;
    if !res {
        println!("a:{a:?} b:{b:?} expected:{expected:?} actual:{actual:?}");
    }
    res
}

/// from: <https://github.com/rklaehn/sorted-iter>
/// just a helper to get good output when a check fails
#[cfg(not(target_arch = "wasm32"))]
fn check_size_hint<E: Debug>(
    input: E,
    expected: usize,
    (min, max): (usize, Option<usize>),
) -> bool {
    let res = min <= expected && max.is_none_or(|max| expected <= max && min <= max);
    if !res {
        println!("input:{input:?} expected:{expected:?} min:{min:?} max:{max:?}");
    }
    res
}

#[cfg(not(target_arch = "wasm32"))]
#[quickcheck]
fn intersection(a: Reference, b: Reference) -> bool {
    let a_r = RangeSetBlaze::from_iter(&a);
    let b_r = RangeSetBlaze::from_iter(&b);
    let expected: Reference = a.intersection(&b).copied().collect();
    let actual: Reference = (a_r & b_r).into_iter().collect();
    binary_op(a, b, expected, actual)
}

#[cfg(not(target_arch = "wasm32"))]
#[quickcheck]
fn union(a: Reference, b: Reference) -> bool {
    let a_r = RangeSetBlaze::from_iter(&a);
    let b_r = RangeSetBlaze::from_iter(&b);
    let expected: Reference = a.union(&b).copied().collect();
    let actual: Reference = (a_r | b_r).into_iter().collect();
    binary_op(a, b, expected, actual)
}

#[cfg(not(target_arch = "wasm32"))]
#[quickcheck]
#[allow(clippy::needless_pass_by_value)]
fn multi_union(inputs: Vec<Reference>) -> bool {
    let expected: Reference = inputs.iter().flatten().copied().collect();
    let actual = inputs.iter().map(RangeSetBlaze::from_iter).union();

    let res = actual.iter().eq(expected.iter().copied());
    if !res {
        let actual: Reference = actual.iter().collect();
        println!("in:{inputs:?} expected:{expected:?} out:{actual:?}");
    }
    res
}

#[cfg(not(target_arch = "wasm32"))]
#[quickcheck]
#[allow(clippy::needless_pass_by_value)]
fn difference(a: Reference, b: Reference) -> bool {
    let a_r = RangeSetBlaze::from_iter(&a);
    let b_r = RangeSetBlaze::from_iter(&b);
    let expected: Reference = a.difference(&b).copied().collect();
    let actual: Reference = (a_r - b_r).into_iter().collect();
    binary_op(a, b, expected, actual)
}

#[cfg(not(target_arch = "wasm32"))]
#[quickcheck]
#[allow(clippy::needless_pass_by_value)]
fn symmetric_difference(a: Reference, b: Reference) -> bool {
    let a_r = RangeSetBlaze::from_iter(&a);
    let b_r = RangeSetBlaze::from_iter(&b);
    let expected: Reference = a.symmetric_difference(&b).copied().collect();
    let actual: Reference = (a_r ^ b_r).into_iter().collect();
    binary_op(a, b, expected, actual)
}

#[cfg(not(target_arch = "wasm32"))]
#[quickcheck]
#[allow(clippy::needless_pass_by_value)]
fn multi_symmetric_difference(inputs: Vec<Reference>) -> bool {
    let mut expected: Reference = BTreeSet::new();
    for input in &inputs {
        expected = expected.symmetric_difference(input).copied().collect();
    }
    let actual = inputs
        .iter()
        .map(RangeSetBlaze::from_iter)
        .symmetric_difference();

    let res = actual.iter().eq(expected.iter().copied());
    if !res {
        let actual: Reference = actual.iter().collect();
        println!("in:{inputs:?} expected:{expected:?} out:{actual:?}");
    }
    res
}

#[cfg(not(target_arch = "wasm32"))]
#[quickcheck]
fn intersection_size_hint(a: Reference, b: Reference) -> bool {
    let expected = a.intersection(&b).count();
    let a_r = RangeSetBlaze::from_iter(&a);
    let b_r = RangeSetBlaze::from_iter(&b);
    let actual = (a_r & b_r).into_iter().size_hint();
    check_size_hint((a, b), expected, actual)
}

#[cfg(not(target_arch = "wasm32"))]
#[quickcheck]
fn union_size_hint(a: Reference, b: Reference) -> bool {
    let expected = a.union(&b).count();
    let a_r = RangeSetBlaze::from_iter(&a);
    let b_r = RangeSetBlaze::from_iter(&b);
    let actual = (a_r | b_r).into_iter().size_hint();
    check_size_hint((a, b), expected, actual)
}

#[cfg(not(target_arch = "wasm32"))]
#[quickcheck]
fn multi_union_size_hint(inputs: Vec<Reference>) -> bool {
    let expected: Reference = inputs.iter().flatten().copied().collect();
    let actual = inputs
        .iter()
        .map(RangeSetBlaze::from_iter)
        .union()
        .iter()
        .size_hint();
    check_size_hint(inputs, expected.len(), actual)
}

#[cfg(not(target_arch = "wasm32"))]
#[quickcheck]
fn difference_size_hint(a: Reference, b: Reference) -> bool {
    let expected = a.difference(&b).count();
    let a_r = RangeSetBlaze::from_iter(&a);
    let b_r = RangeSetBlaze::from_iter(&b);
    let actual = (a_r - b_r).into_iter().size_hint();
    check_size_hint((a, b), expected, actual)
}

#[cfg(not(target_arch = "wasm32"))]
#[quickcheck]
fn symmetric_difference_size_hint(a: Reference, b: Reference) -> bool {
    let expected = a.symmetric_difference(&b).count();
    let a_r = RangeSetBlaze::from_iter(&a);
    let b_r = RangeSetBlaze::from_iter(&b);
    let actual = (a_r ^ b_r).into_iter().size_hint();
    check_size_hint((a, b), expected, actual)
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
fn double_end_into_iter() {
    let a = RangeSetBlaze::from_iter([3..=10, 12..=12, 20..=25]);

    assert_eq!(
        a.clone().into_iter().rev().collect::<Vec<usize>>(),
        vec![25, 24, 23, 22, 21, 20, 12, 10, 9, 8, 7, 6, 5, 4, 3]
    );

    let mut iter = a.into_iter();

    assert_eq!(iter.next(), Some(3));
    assert_eq!(iter.next_back(), Some(25));
    assert_eq!(iter.next(), Some(4));
    assert_eq!(iter.next_back(), Some(24));
    assert_eq!(iter.next_back(), Some(23));
    assert_eq!(iter.next_back(), Some(22));
    assert_eq!(iter.next_back(), Some(21));
    assert_eq!(iter.next_back(), Some(20));

    // Next interval
    assert_eq!(iter.next_back(), Some(12));

    // Next interval, now same interval as front of the iterator
    assert_eq!(iter.next_back(), Some(10));
    assert_eq!(iter.next(), Some(5));
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
fn double_end_range() {
    let a = RangeSetBlaze::from_iter([3..=10, 12..=12, 20..=25]);

    let mut range = a.range(11..=22);
    assert_eq!(range.next_back(), Some(22));
    assert_eq!(range.next(), Some(12));
    assert_eq!(range.next(), Some(20));
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
fn set_random_symmetric_difference() {
    use crate::CheckSortedDisjointMap;
    use crate::RangeSetBlaze;

    for seed in 0..20 {
        println!("seed: {seed}");
        let mut rng = StdRng::seed_from_u64(seed);

        let mut set0 = RangeSetBlaze::new();
        let mut set1 = RangeSetBlaze::new();

        for _ in 0..500 {
            let key = rng.random_range(0..=255u8);
            set0.insert(key);
            print!("l{key} ");
            let key = rng.random_range(0..=255u8);
            set1.insert(key);
            print!("r{key} ");

            let symmetric_difference = set0.ranges().symmetric_difference(set1.ranges());

            // println!(
            //     "left ^ right = {}",
            //     SymDiffIter::new2(set0.ranges(), set1.ranges()).into_string()
            // );

            let map0 = CheckSortedDisjointMap::new(set0.ranges().map(|range| (range, &())))
                .into_range_map_blaze();
            let map1 = CheckSortedDisjointMap::new(set1.ranges().map(|range| (range, &())))
                .into_range_map_blaze();
            let mut expected_map = &map0 ^ &map1;

            println!();
            println!("set0: {set0}");
            println!("set1: {set1}");

            for range in symmetric_difference {
                // println!();
                // print!("removing ");
                for k in range {
                    let get0 = set0.get(k);
                    let get1 = set1.get(k);
                    match (get0, get1) {
                        (Some(_k0), Some(_k1)) => {
                            println!();
                            println!("left: {set0}");
                            println!("right: {set1}");
                            let s_d = set0
                                .ranges()
                                .symmetric_difference(set1.ranges())
                                .into_range_set_blaze();
                            panic!("left ^ right = {s_d}");
                        }
                        (Some(_k0), None) => {}
                        (None, Some(_k1)) => {}
                        (None, None) => {
                            panic!("should not happen 1");
                        }
                    }
                    assert!(expected_map.remove(k).is_some());
                }
                // println!();
            }
            if !expected_map.is_empty() {
                println!();
                println!("left: {set0}");
                println!("right: {set1}");
                let s_d = set0
                    .ranges()
                    .symmetric_difference(set1.ranges())
                    .into_range_set_blaze();
                println!("left ^ right = {s_d}");
                panic!("expected_keys should be empty: {expected_map}");
            }
        }
    }
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
fn set_sym_diff_repro1() {
    use crate::RangeSetBlaze;

    let l = RangeSetBlaze::from_iter([157..=158]);
    let r = RangeSetBlaze::from_iter([158..=158]);
    let iter = l.ranges().symmetric_difference(r.ranges());
    let v = iter.collect::<Vec<_>>();
    println!("{v:?}");
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
#[allow(deprecated)]
fn test_into_string() {
    let a = RangeSetBlaze::from_iter([1..=2, 5..=100]);
    assert_eq!(a.into_string(), "1..=2, 5..=100");
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
#[allow(deprecated)]
fn test_to_string() {
    let a = RangeSetBlaze::from_iter([1..=2, 5..=100]).into_ranges();
    assert_eq!(a.into_string(), "1..=2, 5..=100");
}
#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
fn test_next_back() {
    let a = RangeSetBlaze::from_iter([5, 5, 4, 1]);
    let mut iter = a.iter();
    assert_eq!(iter.next_back(), Some(5));
    assert_eq!(iter.next_back(), Some(4));
    assert_eq!(iter.next_back(), Some(1));
    assert_eq!(iter.next_back(), None);
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
fn test_into_ranges_iter() {
    let mut a = RangeSetBlaze::from_iter([1..=2, 5..=100]).into_ranges();
    assert_eq!(a.len(), 2);
    assert_eq!(a.next_back(), Some(5..=100));
    assert_eq!(a.len(), 1);
    assert_eq!(a.next_back(), Some(1..=2));
    assert_eq!(a.len(), 0);
    assert_eq!(a.next_back(), None);
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
fn test_range_map_intersection() {
    let c = RangeMapBlaze::from_iter([(2..=2, "c"), (6..=200, "c")]);
    let b = RangeMapBlaze::from_iter([(2..=6, "b")]);
    let a = RangeMapBlaze::from_iter([(1..=2, "a"), (5..=100, "a")]);

    let intersection = [c, b, a].intersection();

    assert_eq!(intersection.to_string(), r#"(2..=2, "a"), (6..=6, "a")"#);
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
fn test_range_map_symmetric_difference() {
    let c = RangeMapBlaze::from_iter([(2..=2, "c"), (6..=200, "c")]);
    let b = RangeMapBlaze::from_iter([(2..=6, "b")]);
    let a = RangeMapBlaze::from_iter([(1..=2, "a"), (5..=100, "a")]);

    let symmetric_difference = [c, b, a].symmetric_difference();

    assert_eq!(
        symmetric_difference.to_string(),
        r#"(1..=2, "a"), (3..=4, "b"), (6..=6, "a"), (101..=200, "c")"#
    );
}

#[cfg(feature = "rog_experimental")]
#[allow(deprecated)]
#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
fn test_rog_coverage2() {
    assert_eq!(Rog::Gap(1..=3).end(), 3);

    let range_set_blaze: RangeSetBlaze<u8> = RangeSetBlaze::from([]);
    assert_eq!(range_set_blaze.rogs_get(2), Rog::Gap(0..=255));
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
fn test_assume_sorted_starts_size_hint() {
    let m = AssumeSortedStarts::new([0..=3, 0..=2, 1..=5]);
    assert_eq!(m.size_hint(), (3, Some(3)));
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
fn more_coverage_of_range_set_blaze() {
    // Test `BitOrAssign<&Self>` where `self` is empty, covering the `self = other.clone()` line.
    let mut a: RangeSetBlaze<i32> = RangeSetBlaze::default();
    let b_non_empty = RangeSetBlaze::from_iter([1..=3, 5..=5]);
    a |= &b_non_empty;
    assert_eq!(a, b_non_empty);

    // Test `BitOr<&RangeSetBlaze<T>>` where `self` is empty, covering the `return other.clone()` line.
    let a_empty: RangeSetBlaze<i32> = RangeSetBlaze::default();
    let b_non_empty = RangeSetBlaze::from_iter([0..=2, 4..=6]);
    let union = &a_empty | &b_non_empty;
    assert_eq!(union, b_non_empty);
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
#[cfg(feature = "from_slice")]
fn additional_from_slice_iter_coverage() {
    // // Test `FromSliceIter::next` with consecutive ranges followed by a non-consecutive element.
    // let a = RangeSetBlaze::from_slice([1, 2, 3, 10]);
    // assert_eq!(a.to_string(), "1..=3, 10..=10");

    // // Test `FromSliceIter::next` with entirely non-consecutive values.
    // let a = RangeSetBlaze::from_slice([1, 3, 5, 7]);
    // assert_eq!(a.to_string(), "1..=1, 3..=3, 5..=5, 7..=7");

    // // Test empty slice to ensure it outputs an empty range set.
    // let slice: &[i32] = &[];
    // let a = RangeSetBlaze::from_slice(slice);
    // assert!(a.is_empty());

    // // Test a mix of consecutive and non-consecutive elements that require flushing `previous_range`.
    // let a = RangeSetBlaze::from_slice([1, 2, 4, 5, 10]);
    // assert_eq!(a.to_string(), "1..=2, 4..=5, 10..=10");

    // // Test `size_hint` for cases when `slice_len` is non-zero, hitting `slice_len - 1`.
    // let a = RangeSetBlaze::from_slice([1, 2, 3]);
    // assert_eq!(a.to_string(), "1..=3");

    // // Test `size_hint` in `FromSliceIter` for empty case, hitting zero case for `low` branch.
    // let slice: &[i32] = &[];
    // let a = RangeSetBlaze::from_slice(slice);
    // assert!(a.is_empty());

    // // Edge case for size_hint if slice length approaches `usize::MAX` by simulating a large collection.
    // let large_slice: Vec<i32> = (0..1_000).collect();
    // let a = RangeSetBlaze::from_slice(&large_slice);
    // assert_eq!(a.to_string(), "0..=999");

    // // Singletons from non-consecutive entries at the end of chunks, ensuring flushing logic works.
    // let a = RangeSetBlaze::from_slice([1, 2, 4, 7]);
    // assert_eq!(a.to_string(), "1..=2, 4..=4, 7..=7");
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
#[allow(clippy::type_complexity)]
fn test_empty_inputs() {
    let inputs: [RangeSetBlaze<i32>; 0] = [];
    let intersection = inputs.intersection();
    assert_eq!(intersection.to_string(), r"-2147483648..=2147483647");

    let inputs: [RangeSetBlaze<i32>; 0] = [];
    let union = inputs.union();
    assert_eq!(union.to_string(), r"");

    let inputs: [RangeSetBlaze<i32>; 0] = [];
    let symmetric_difference = inputs.symmetric_difference();
    assert_eq!(symmetric_difference.to_string(), r"");

    let inputs: [&RangeSetBlaze<i32>; 0] = [];
    let intersection = inputs.intersection();
    assert_eq!(intersection.to_string(), r"-2147483648..=2147483647");

    let inputs: [&RangeSetBlaze<i32>; 0] = [];
    let union = inputs.union();
    assert_eq!(union.to_string(), r"");

    let inputs: [&RangeSetBlaze<i32>; 0] = [];
    let symmetric_difference = inputs.symmetric_difference();
    assert_eq!(symmetric_difference.to_string(), r"");

    let inputs: [range_set_blaze::IntoRangesIter<i32>; 0] = [];
    let intersection = inputs.intersection();
    assert_eq!(intersection.into_string(), r"-2147483648..=2147483647");

    let inputs: [range_set_blaze::IntoRangesIter<i32>; 0] = [];
    let union = inputs.union();
    assert_eq!(union.into_string(), r"");

    let inputs: [range_set_blaze::IntoRangesIter<i32>; 0] = [];
    let symmetric_difference = inputs.symmetric_difference();
    assert_eq!(symmetric_difference.into_string(), r"");
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
#[should_panic(expected = "start (inclusive) must be less than or equal to end (inclusive)")]
#[allow(clippy::reversed_empty_ranges)]
fn range_expect_panic() {
    let set = RangeSetBlaze::new();
    let _ = set.range(4..=3).next();
}

#[allow(clippy::items_after_statements)]
#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
const fn check_traits() {
    // Debug/Display/Clone/PartialEq/PartialOrd/Default/Hash/Eq/Ord/Send/Sync
    type ARangeSetBlaze = RangeSetBlaze<i32>;
    is_sssu::<ARangeSetBlaze>();
    is_ddcppdheo::<ARangeSetBlaze>();
    is_like_btreeset::<ARangeSetBlaze>();

    type AIter<'a> = Iter<i32, ARangesIter<'a>>;
    is_sssu::<AIter<'_>>();
    is_like_btreeset_iter_less_exact_size::<AIter<'_>>();

    type ARangesIter<'a> = RangesIter<'a, i32>;
    is_sssu::<ARangesIter<'_>>();
    is_like_btreeset_iter::<ARangesIter<'_>>();

    type AIntoRangesIter<'a> = IntoRangesIter<i32>;
    is_sssu::<AIntoRangesIter<'_>>();
    is_like_btreeset_into_iter::<AIntoRangesIter<'_>>();

    type AMapRangesIter<'a> = MapRangesIter<'a, i32, u64>;
    is_sssu::<AMapRangesIter<'_>>();
    is_like_btreeset_iter_less_both::<AMapRangesIter<'_>>();

    type ARangeValuesToRangesIter<'a> =
        RangeValuesToRangesIter<i32, &'a u64, RangeValuesIter<'a, i32, u64>>;
    is_sssu::<ARangeValuesToRangesIter<'_>>();
    is_like_btreeset_iter_less_both::<ARangeValuesToRangesIter<'_>>();

    type AMapIntoRangesIter = MapIntoRangesIter<i32, u64>;
    is_sssu::<AMapIntoRangesIter>();
    is_like_btreeset_into_iter_less_both::<AMapIntoRangesIter>();

    type AIntoIter = IntoIter<i32>;
    is_sssu::<AIntoIter>();
    is_like_btreeset_into_iter_less_exact_size::<AIntoIter>();

    type AKMerge<'a> = crate::KMerge<i32, ARangesIter<'a>>;
    is_sssu::<AKMerge<'_>>();
    is_like_btreeset_iter_less_both::<AKMerge<'_>>();

    type AMerge<'a> = crate::Merge<i32, ARangesIter<'a>, ARangesIter<'a>>;
    is_sssu::<AMerge<'_>>();
    is_like_btreeset_iter_less_both::<AMerge<'_>>();

    type ANotIter<'a> = crate::NotIter<i32, ARangesIter<'a>>;
    is_sssu::<ANotIter<'_>>();
    is_like_btreeset_iter_less_both::<ANotIter<'_>>();

    type AUnionIter<'a> = UnionIter<i32, ARangesIter<'a>>;
    is_sssu::<AUnionIter<'_>>();
    is_like_btreeset_iter_less_both::<AUnionIter<'_>>();

    type ASymDiffIter<'a> = SymDiffIter<i32, ARangesIter<'a>>;
    is_sssu::<ASymDiffIter<'_>>();
    is_like_btreeset_iter_less_both::<ASymDiffIter<'_>>();

    type AAssumeSortedStarts<'a> = AssumeSortedStarts<i32, ARangesIter<'a>>;
    is_sssu::<AAssumeSortedStarts<'_>>();
    is_like_btreeset_iter_less_both::<AAssumeSortedStarts<'_>>();

    type ACheckSortedDisjoint<'a> = CheckSortedDisjoint<i32, ARangesIter<'a>>;
    is_sssu::<ACheckSortedDisjoint<'_>>();
    type BCheckSortedDisjoint =
        CheckSortedDisjoint<i32, std::array::IntoIter<RangeInclusive<i32>, 0>>;
    is_like_check_sorted_disjoint::<BCheckSortedDisjoint>();

    type ADynSortedDisjoint<'a> = DynSortedDisjoint<'a, i32>;
    is_like_dyn_sorted_disjoint::<ADynSortedDisjoint<'_>>();
}

const fn is_ddcppdheo<
    T: fmt::Debug
        + fmt::Display
        + Clone
        + PartialEq
        + PartialOrd
        + Default
        + core::hash::Hash
        + Eq
        + Ord
        + Send
        + Sync,
>() {
}

const fn is_sssu<T: Sized + Send + Sync + Unpin>() {}
const fn is_like_btreeset_iter<
    T: Clone + fmt::Debug + FusedIterator + Iterator + DoubleEndedIterator + ExactSizeIterator,
>() {
}
const fn is_like_btreeset_iter_less_both<T: Clone + fmt::Debug + FusedIterator + Iterator>() {}
const fn is_like_btreeset_iter_less_exact_size<
    T: Clone + fmt::Debug + FusedIterator + Iterator + DoubleEndedIterator,
>() {
}

const fn is_like_btreeset_into_iter<
    T: fmt::Debug + FusedIterator + Iterator + DoubleEndedIterator + ExactSizeIterator,
>() {
}
const fn is_like_btreeset_into_iter_less_exact_size<
    T: fmt::Debug + FusedIterator + Iterator + DoubleEndedIterator,
>() {
}
const fn is_like_btreeset_into_iter_less_both<T: fmt::Debug + FusedIterator + Iterator>() {}

const fn is_like_btreeset<
    T: Clone
        + fmt::Debug
        + Default
        + Eq
        + core::hash::Hash
        + IntoIterator
        + Ord
        + PartialEq
        + PartialOrd
        + core::panic::RefUnwindSafe
        + Send
        + Sync
        + Unpin
        + core::panic::UnwindSafe
        + Any
        + ToOwned,
>() {
}

const fn is_like_check_sorted_disjoint<
    T: Clone
        + fmt::Debug
        + Default
        + IntoIterator
        + core::panic::RefUnwindSafe
        + Send
        + Sync
        + Unpin
        + core::panic::UnwindSafe
        + Any
        + ToOwned,
>() {
}

const fn is_like_dyn_sorted_disjoint<T: IntoIterator + Unpin + Any>() {}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
fn test_multiway() {
    let a = RangeSetBlaze::from_iter([1..=6, 8..=9, 11..=15]);
    let b = RangeSetBlaze::from_iter([5..=13, 18..=29]);
    let c = RangeSetBlaze::from_iter([25..=100]);
    // use crate::multiway::MultiwayRangeSetBlaze;
    let iter = vec![a, b, c].into_iter();
    let union = iter.union();
    assert_eq!(union, RangeSetBlaze::from_iter([1..=15, 18..=100]));

    let a = RangeSetBlaze::from_iter([1..=6, 8..=9, 11..=15]);
    let b = RangeSetBlaze::from_iter([5..=13, 18..=29]);
    let c = RangeSetBlaze::from_iter([25..=100]);
    // use crate::multiway::MultiwayRangeSetBlazeRef;
    let union = [a, b, c].union();
    assert_eq!(union, RangeSetBlaze::from_iter([1..=15, 18..=100]));
}

#[allow(deprecated)]
#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
fn test_deprecated_to_string() {
    let a = CheckSortedDisjoint::new([1..=6, 8..=9, 11..=15]);
    assert_eq!(a.to_string(), "1..=6, 8..=9, 11..=15");
}
