#![cfg(test)]
#![cfg(not(target_arch = "wasm32"))]

use crate::sorted_disjoint_map::Priority;
use crate::unsorted_priority_map::AssumePrioritySortedStartsMap;

use super::*;
use core::any::Any;
use core::cmp::Ordering;
use core::fmt;
use core::iter::FusedIterator;
use core::ops::Bound;
use core::ops::RangeInclusive;
use num_traits::{One, Zero};
use set::extract_range;
use std::collections::hash_map::DefaultHasher;
use syntactic_for::syntactic_for;
// use thousands::Separable;
use core::array;
#[cfg(target_os = "linux")] // cmk why is BitAndAssign tied to linux?
use core::ops::BitAndAssign;
use std::prelude::v1::*;
use std::{print, println, vec};

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
fn demo_f1() {
    // before_or_equal_exists	0
    //     INSERT, etc

    let mut range_set_blaze = RangeSetBlaze::from_iter([11..=14, 22..=26]);
    range_set_blaze.internal_add(10..=10);
    assert_eq!(range_set_blaze.to_string(), "10..=14, 22..=26");
    println!(
        "demo_1 range_set_blaze = {:?}, len_slow = {}, len = {}",
        range_set_blaze,
        range_set_blaze.len_slow(),
        range_set_blaze.len()
    );

    assert!(range_set_blaze.len_slow() == range_set_blaze.len());
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
fn demo_d1() {
    // before_or_equal_exists	1
    // equal?	1
    // is_included	n/a
    // fits?	1
    //     DONE

    let mut range_set_blaze = RangeSetBlaze::from_iter([10..=14]);
    range_set_blaze.internal_add(10..=10);
    assert_eq!(range_set_blaze.to_string(), "10..=14");
    assert_eq!(range_set_blaze.len_slow(), range_set_blaze.len());
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
fn demo_e1() {
    // before_or_equal_exists	1
    // equal?	1
    // is_included	n/a
    // fits?	0
    // next?    0
    //     DONE

    let mut range_set_blaze = RangeSetBlaze::from_iter([10..=14, 16..=16]);
    range_set_blaze.internal_add(10..=19);
    assert_eq!(range_set_blaze.to_string(), "10..=19");
    assert_eq!(range_set_blaze.len_slow(), range_set_blaze.len());
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
fn demo_b1() {
    // before_or_equal_exists	1
    // equal?	0
    // is_included	1
    // fits?	0
    // next?    0
    //     DONE

    let mut range_set_blaze = RangeSetBlaze::from_iter([10..=14]);
    range_set_blaze.internal_add(12..=17);
    assert_eq!(range_set_blaze.to_string(), "10..=17");
    assert_eq!(range_set_blaze.len_slow(), range_set_blaze.len());
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
fn demo_b2() {
    // before_or_equal_exists	1
    // equal?	0
    // is_included	1
    // fits?	0
    // next?    1
    // delete how many? 1
    //     DONE

    let mut range_set_blaze = RangeSetBlaze::from_iter([10..=14, 16..=16]);
    range_set_blaze.internal_add(12..=17);
    assert_eq!(range_set_blaze.to_string(), "10..=17");
    assert_eq!(range_set_blaze.len_slow(), range_set_blaze.len());
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
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
                        let mut range_set_blaze = RangeSetBlaze::new();
                        range_set_blaze.internal_add(a..=b);
                        range_set_blaze.internal_add(c..=d);
                        if range_set_blaze.ranges_len() == 1 {
                            let vec = range_set_blaze.into_iter().collect::<Vec<u8>>();
                            println!("combine\t{}\t{}", vec[0], vec[vec.len() - 1]);
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
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
#[allow(
    clippy::bool_assert_comparison,
    clippy::many_single_char_names,
    clippy::cognitive_complexity,
    clippy::too_many_lines
)]
fn lib_coverage_0() {
    use std::hash::Hash;

    let a = RangeSetBlaze::from_iter([1..=2, 3..=4]);
    let mut hasher = DefaultHasher::new();
    a.hash(&mut hasher);
    let _d = RangeSetBlaze::<i32>::default();
    assert_eq!(a, a);

    let mut set = RangeSetBlaze::new();
    assert_eq!(set.first(), None);
    set.insert(1);
    assert_eq!(set.first(), Some(1));
    set.insert(2);
    assert_eq!(set.first(), Some(1));

    let set = RangeSetBlaze::from_iter([1, 2, 3]);
    assert_eq!(set.get(2), Some(2));
    assert_eq!(set.get(4), None);

    let mut set = RangeSetBlaze::new();
    assert_eq!(set.last(), None);
    set.insert(1);
    assert_eq!(set.last(), Some(1));
    set.insert(2);
    assert_eq!(set.last(), Some(2));

    assert_eq!(a.len(), a.len_slow());

    let mut a = RangeSetBlaze::from_iter([1..=3]);
    let mut b = RangeSetBlaze::from_iter([3..=5]);

    a.append(&mut b);

    assert_eq!(a.len(), 5u64);
    assert_eq!(b.len(), 0u64);

    assert!(a.contains(1));
    assert!(a.contains(2));
    assert!(a.contains(3));
    assert!(a.contains(4));
    assert!(a.contains(5));

    let mut v = RangeSetBlaze::new();
    v.insert(1);
    v.clear();
    assert!(v.is_empty());

    let mut v = RangeSetBlaze::new();
    assert!(v.is_empty());
    v.insert(1);
    assert!(!v.is_empty());

    let superset = RangeSetBlaze::from_iter([1..=3]);
    let mut set = RangeSetBlaze::new();

    assert_eq!(set.is_subset(&superset), true);
    set.insert(2);
    assert_eq!(set.is_subset(&superset), true);
    set.insert(4);
    assert_eq!(set.is_subset(&superset), false);

    let subset = RangeSetBlaze::from_iter([1, 2]);
    let mut set = RangeSetBlaze::new();

    assert_eq!(set.is_superset(&subset), false);

    set.insert(0);
    set.insert(1);
    assert_eq!(set.is_superset(&subset), false);

    set.insert(2);
    assert_eq!(set.is_superset(&subset), true);

    let a = RangeSetBlaze::from_iter([1..=3]);
    let mut b = RangeSetBlaze::new();

    assert_eq!(a.is_disjoint(&b), true);
    b.insert(4);
    assert_eq!(a.is_disjoint(&b), true);
    b.insert(1);
    assert_eq!(a.is_disjoint(&b), false);

    let mut set = RangeSetBlaze::new();
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

    let mut set = RangeSetBlaze::new();

    assert_eq!(set.ranges_insert(2..=5), true);
    assert_eq!(set.ranges_insert(5..=6), true);
    assert_eq!(set.ranges_insert(3..=4), false);
    assert_eq!(set.len(), 5u64);
    let mut set = RangeSetBlaze::from_iter([1, 2, 3]);
    assert_eq!(set.take(2), Some(2));
    assert_eq!(set.take(2), None);

    let mut set = RangeSetBlaze::new();
    assert!(set.replace(5).is_none());
    assert!(set.replace(5).is_some());

    let mut a = RangeSetBlaze::from_iter([1..=3]);
    #[allow(clippy::reversed_empty_ranges)]
    a.internal_add(2..=1);

    assert_eq!(a.partial_cmp(&a), Some(Ordering::Equal));

    let mut a = RangeSetBlaze::from_iter([1..=3]);
    a.extend(std::iter::once(4));
    assert_eq!(a.len(), 4u64);

    let mut a = RangeSetBlaze::from_iter([1..=3]);
    a.extend(4..=5);
    assert_eq!(a.len(), 5u64);

    let mut set = RangeSetBlaze::new();

    set.insert(1);
    while let Some(n) = set.pop_first() {
        assert_eq!(n, 1);
    }
    assert!(set.is_empty());

    let mut set = RangeSetBlaze::new();

    set.insert(1);
    while let Some(n) = set.pop_last() {
        assert_eq!(n, 1);
    }
    assert!(set.is_empty());

    let a = RangeSetBlaze::from_iter([1..=3]);
    let i = a.iter();
    let j = i.clone();
    assert_eq!(i.size_hint(), j.size_hint());

    let a = RangeSetBlaze::from_iter([1..=3]);
    let i = a.into_iter();
    assert_eq!(i.size_hint(), j.size_hint());
    assert_eq!(
        alloc::format!("{:?}", &i),
        "IntoIter { option_range_front: None, option_range_back: None, btree_map_into_iter: [(1, 3)] }"
    );

    let mut a = RangeSetBlaze::from_iter([1..=3]);
    a.extend([1..=3]);
    assert_eq!(a.len(), 3u64);

    let a = RangeSetBlaze::from_iter([1..=3]);
    let b = <RangeSetBlaze<i32> as Clone>::clone(&a);
    assert_eq!(a, b);
    let c = <RangeSetBlaze<i32> as Default>::default();
    assert_eq!(c, RangeSetBlaze::new());

    syntactic_for! { ty in [i8, u8, isize, usize,  i16, u16, i32, u32, i64, u64, isize, usize, i128, u128] {
        $(
            let a = RangeSetBlaze::<$ty>::new();
            println!("{a:#?}");
            assert_eq!(a.first(), None);

            let mut a = RangeSetBlaze::from_iter([$ty::one()..=3]);
            let mut b = RangeSetBlaze::from_iter([3..=5]);

            a.append(&mut b);

            // assert_eq!(a.len(), 5);
            assert_eq!(b.len(), <$ty as Integer>::SafeLen::zero());

            assert!(a.contains(1));
            assert!(a.contains(2));
            assert!(a.contains(3));
            assert!(a.contains(4));
            assert!(a.contains(5));

            assert!(b.is_empty());

            let a = RangeSetBlaze::from_iter([$ty::one()..=3]);
            let b = RangeSetBlaze::from_iter([3..=5]);
            assert!(!a.is_subset(&b));
            assert!(!a.is_superset(&b));

        )*
    }};

    let a = RangeSetBlaze::from_iter([1u128..=3]);
    assert!(a.contains(1));
    assert!(!a.is_disjoint(&a));
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
#[should_panic(expected = "start (inclusive) must be less than or equal to end (inclusive)")]
#[allow(clippy::reversed_empty_ranges)]
fn range_expect_panic() {
    let set = RangeSetBlaze::new();
    let _ = set.range(4..=3).next();
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
fn lib_coverage_5() {
    let mut v = RangeSetBlaze::<u128>::new();
    v.internal_add(0..=u128::MAX);
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
#[allow(clippy::cognitive_complexity, clippy::iter_on_empty_collections)]
fn sdi1() {
    let a = [157..=158, 158..=158].into_iter();
    let a = AssumeSortedStarts::new(a);
    let mut iter = SymDiffIter::new(a);
    assert_eq!(iter.next(), Some(157..=157));
    assert_eq!(iter.next(), None);

    let a = [0..=0, 0..=0, 0..=1, 2..=100].into_iter();
    let a = AssumeSortedStarts::new(a);
    let mut iter = SymDiffIter::new(a);
    assert_eq!(iter.next(), Some(0..=100));
    assert_eq!(iter.next(), None);

    let a = [0..=0, 0..=1, 2..=100].into_iter();
    let a = AssumeSortedStarts::new(a);
    let mut iter = SymDiffIter::new(a);
    assert_eq!(iter.next(), Some(1..=100));
    assert_eq!(iter.next(), None);

    let a = [0..=0, 0..=0, 2..=100].into_iter();
    let a = AssumeSortedStarts::new(a);
    let mut iter = SymDiffIter::new(a);
    assert_eq!(iter.next(), Some(2..=100));
    assert_eq!(iter.next(), None);

    let a = [0..=0, 0..=0, 0..=0, 2..=100].into_iter();
    let a = AssumeSortedStarts::new(a);
    let mut iter = SymDiffIter::new(a);
    assert_eq!(iter.next(), Some(0..=0));
    assert_eq!(iter.next(), Some(2..=100));
    assert_eq!(iter.next(), None);
    {
        let a = [0..=1, 0..=0].into_iter();
        let a = AssumeSortedStarts::new(a);
        let mut iter = SymDiffIter::new(a);
        assert_eq!(iter.next(), Some(1..=1));
        assert_eq!(iter.next(), None);

        let a = [0..=1, 0..=0, 0..=0].into_iter();
        let a = AssumeSortedStarts::new(a);
        let mut iter = SymDiffIter::new(a);
        assert_eq!(iter.next(), Some(0..=1));
        assert_eq!(iter.next(), None);

        let a = [0..=0, 0..=0, 0..=0].into_iter();
        let a = AssumeSortedStarts::new(a);
        let mut iter = SymDiffIter::new(a);
        assert_eq!(iter.next(), Some(0..=0));
        assert_eq!(iter.next(), None);

        let a = [0..=0, 0..=0].into_iter();
        let a = AssumeSortedStarts::new(a);
        let mut iter = SymDiffIter::new(a);
        assert_eq!(iter.next(), None);

        let a = [0..=0, 1..=1].into_iter();
        let a = AssumeSortedStarts::new(a);
        let mut iter = SymDiffIter::new(a);
        assert_eq!(iter.next(), Some(0..=1));
        assert_eq!(iter.next(), None);

        let a = [0..=0, 1..=1].into_iter();
        let a = AssumeSortedStarts::new(a);
        let mut iter = SymDiffIter::new(a);
        assert_eq!(iter.next(), Some(0..=1));
        assert_eq!(iter.next(), None);

        let a = [0..=0, 2..=2].into_iter();
        let a = AssumeSortedStarts::new(a);
        let mut iter = SymDiffIter::new(a);
        assert_eq!(iter.next(), Some(0..=0));
        assert_eq!(iter.next(), Some(2..=2));
        assert_eq!(iter.next(), None);

        let a = std::iter::once(0..=0);
        let a = AssumeSortedStarts::new(a);
        let mut iter = SymDiffIter::new(a);
        assert_eq!(iter.next(), Some(0..=0));
        assert_eq!(iter.next(), None);

        let a: array::IntoIter<RangeInclusive<i32>, 0> = [].into_iter();
        let a = AssumeSortedStarts::new(a);
        let iter = SymDiffIter::new(a);
        let v = iter.collect::<Vec<_>>();
        assert_eq!(v, vec![]);
    }
}

// // FUTURE: use fn range to implement one-at-a-time intersection, difference, etc. and then add more inplace ops.
#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
fn convert_challenge() {
    use itertools::Itertools;
    use unsorted_priority_map::UnsortedPriorityMap;

    //===========================
    // Map - ranges
    //===========================

    // * from sorted_disjoint
    let a = CheckSortedDisjointMap::new([(1..=2, &"a"), (5..=100, &"a")]);
    assert!(a.equal(CheckSortedDisjointMap::new([
        (1..=2, &"a"),
        (5..=100, &"a")
    ])));

    // * from (priority) sorted_starts
    let a = [(1..=4, &"a"), (5..=100, &"a"), (5..=5, &"b")].into_iter();
    let a = a
        .enumerate()
        .map(|(i, range_value)| Priority::new(range_value, i));
    let a = AssumePrioritySortedStartsMap::new(a);
    let a = UnionIterMap::new(a);
    // is_sorted_disjoint_map::<_, _, _, _>(a);
    assert!(a.equal(CheckSortedDisjointMap::new([(1..=100, &"a"),])));

    // * from unsorted_priority_map
    let iter = [(5..=100, &"a"), (5..=5, &"b"), (1..=4, &"a")].into_iter();
    let iter = iter
        .enumerate()
        .map(|(i, range_value)| Priority::new(range_value, i));
    let iter = iter.into_iter().sorted_by(|a, b| {
        // We sort only by start -- priority is not used until later.
        a.start().cmp(&b.start())
    });
    let iter = AssumePrioritySortedStartsMap::new(iter);
    let iter = UnionIterMap::new(iter);
    assert!(iter.equal(CheckSortedDisjointMap::new([(1..=100, &"a"),])));

    // * anything
    let iter = [(5, &"a"), (5, &"b"), (1, &"a")]
        .into_iter()
        .map(|(x, y)| (x..=x, y));
    let iter = UnsortedPriorityMap::new(iter);
    let iter = iter.sorted_by(|a, b| {
        // We sort only by start -- priority is not used until later.
        a.start().cmp(&b.start())
    });
    let iter = AssumePrioritySortedStartsMap::new(iter);
    let iter = UnionIterMap::new(iter);
    assert!(iter.equal(CheckSortedDisjointMap::new([(1..=1, &"a"), (5..=5, &"a"),])));

    //===========================
    // Map - points
    //===========================

    // * from sorted_disjoint
    let a = [(1, &"a"), (5, &"a")].into_iter().map(|(x, y)| (x..=x, y));
    let a = CheckSortedDisjointMap::new(a);
    assert!(a.equal(CheckSortedDisjointMap::new([(1..=1, &"a"), (5..=5, &"a")])));

    // * from (priority) sorted_starts
    let a = [(1, &"a"), (5, &"a"), (5, &"b")].into_iter();
    let a = a
        .enumerate()
        .map(|(i, (k, v))| Priority::new((k..=k, v), i));
    let a = AssumePrioritySortedStartsMap::new(a);
    let a = UnionIterMap::new(a);
    // is_sorted_disjoint_map::<_, _, _, _>(a);
    assert!(a.equal(CheckSortedDisjointMap::new([(1..=1, &"a"), (5..=5, &"a")])));

    // * from unsorted_priority_map
    let iter = [(5, &"a"), (5, &"b"), (1, &"a")].into_iter();
    let iter = iter
        .enumerate()
        .map(|(i, (k, v))| Priority::new((k..=k, v), i));
    let iter = iter.into_iter().sorted_by(|a, b| {
        // We sort only by start -- priority is not used until later.
        a.start().cmp(&b.start())
    });
    let iter = AssumePrioritySortedStartsMap::new(iter);
    let iter = UnionIterMap::new(iter);
    assert!(iter.equal(CheckSortedDisjointMap::new([(1..=1, &"a"), (5..=5, &"a")])));

    // * anything
    let iter = [(5..=100, &"a"), (5..=5, &"b"), (1..=4, &"a")].into_iter();
    let iter = UnsortedPriorityMap::new(iter);
    let iter = iter.sorted_by(|a, b| {
        // We sort only by start -- priority is not used until later.
        a.start().cmp(&b.start())
    });
    let iter = AssumePrioritySortedStartsMap::new(iter);
    let iter = UnionIterMap::new(iter);
    assert!(iter.equal(CheckSortedDisjointMap::new([(1..=100, &"a"),])));

    //===========================
    // Set - ranges
    //===========================

    // * from sorted_disjoint
    let a = CheckSortedDisjoint::new([1..=2, 5..=100]);
    assert!(a.equal(CheckSortedDisjoint::new([1..=2, 5..=100])));

    // * from (priority) sorted_starts
    let a = [1..=4, 5..=100, 5..=5].into_iter();
    let a = AssumeSortedStarts::new(a);
    let a = UnionIter::new(a);
    assert!(a.equal(CheckSortedDisjoint::new([1..=100])));

    // * from unsorted_priority_map
    let iter = [5..=100, 5..=5, 1..=4].into_iter();
    let iter = iter.into_iter().sorted_by(|a, b| {
        // We sort only by start -- priority is not used until later.
        a.start().cmp(b.start())
    });
    let iter = AssumeSortedStarts::new(iter);
    let iter = UnionIter::new(iter);
    assert!(iter.equal(CheckSortedDisjoint::new([1..=100])));

    // * anything
    let iter = [5..=100, 5..=5, 1..=5].into_iter();
    let iter = iter.sorted_by(|a, b| {
        // We sort only by start -- priority is not used until later.
        a.start().cmp(b.start())
    });
    let iter = AssumeSortedStarts::new(iter);
    let iter = UnionIter::new(iter);
    assert!(iter.equal(CheckSortedDisjoint::new([1..=100])));
    // Set - points

    // what about multiple inputs?
}

#[cfg(feature = "from_slice")]
#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
fn understand_slice_iter() {
    use std::simd::Simd;

    use from_slice::FromSliceIter;
    use integer::LANES;

    let slice: [u8; 0] = [];
    let iter = FromSliceIter::<u8, LANES>::new(&slice);
    assert_eq!(iter.size_hint(), (0, Some(0)));
    assert_eq!(iter.count(), 0);

    // 1st 500 even numbers
    let slice: &[_] = &(0..1000).step_by(2).collect::<Vec<_>>();
    let iter = FromSliceIter::<_, LANES>::new(slice);
    assert_eq!(iter.size_hint(), (1, Some(500)));
    assert_eq!(iter.count(), 500);

    // 32 consecutive u8's as a slice
    let slice: &[_] = &(0..64i64).collect::<Vec<_>>();
    let slice = Simd::<_, 64>::from_slice(slice);
    let iter = FromSliceIter::<_, LANES>::new(slice.as_array());
    assert_eq!(iter.size_hint(), (1, Some(64)));
    assert_eq!(iter.count(), 1);
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
fn test_extract_range() {
    use std::ops::Bound::{Excluded, Included};

    assert_eq!(extract_range((Excluded(0), Included(1))), (1, 1));
    assert_eq!(extract_range(0..1), (0, 0));
}

#[allow(clippy::items_after_statements)]
#[test]
const fn check_traits() {
    // Debug/Display/Clone/PartialEq/PartialOrd/Default/Hash/Eq/Ord/Send/Sync
    type ARangeSetBlaze = RangeSetBlaze<i32>;
    is_sssu::<ARangeSetBlaze>();
    is_ddcppdheo::<ARangeSetBlaze>();
    is_like_btreeset::<ARangeSetBlaze>();

    type AIter<'a> = Iter<i32, ARangesIter<'a>>;
    is_sssu::<AIter>();
    is_like_btreeset_iter::<AIter>();

    type ARangesIter<'a> = RangesIter<'a, i32>;
    is_sssu::<ARangesIter>();
    is_like_btreeset_iter::<ARangesIter>();

    type AIntoRangesIter<'a> = IntoRangesIter<i32>;
    is_sssu::<AIntoRangesIter>();
    is_like_btreeset_into_iter::<AIntoRangesIter>();

    type AMapRangesIter<'a> = MapRangesIter<'a, i32, u64>;
    is_sssu::<AMapRangesIter>();
    is_like_btreeset_iter::<AMapRangesIter>();

    type ARangeValuesToRangesIter<'a> =
        RangeValuesToRangesIter<i32, &'a u64, RangeValuesIter<'a, i32, u64>>;
    is_sssu::<ARangeValuesToRangesIter>();
    is_like_btreeset_iter::<ARangeValuesToRangesIter>();

    type AMapIntoRangesIter = MapIntoRangesIter<i32, u64>;
    is_sssu::<AMapIntoRangesIter>();
    is_like_btreeset_into_iter::<AMapIntoRangesIter>();

    type AIntoIter = IntoIter<i32>;
    is_sssu::<AIntoIter>();
    is_like_btreeset_into_iter::<AIntoIter>();

    type AKMerge<'a> = crate::KMerge<i32, ARangesIter<'a>>;
    is_sssu::<AKMerge>();
    is_like_btreeset_iter::<AKMerge>();

    type AMerge<'a> = crate::Merge<i32, ARangesIter<'a>, ARangesIter<'a>>;
    is_sssu::<AMerge>();
    is_like_btreeset_iter::<AMerge>();

    type ANotIter<'a> = crate::NotIter<i32, ARangesIter<'a>>;
    is_sssu::<ANotIter>();
    is_like_btreeset_iter::<ANotIter>();

    type AUnionIter<'a> = UnionIter<i32, ARangesIter<'a>>;
    is_sssu::<AUnionIter>();
    is_like_btreeset_iter::<AUnionIter>();

    type ASymDiffIter<'a> = SymDiffIter<i32, ARangesIter<'a>>;
    is_sssu::<ASymDiffIter>();
    is_like_btreeset_iter::<ASymDiffIter>();

    type AAssumeSortedStarts<'a> = AssumeSortedStarts<i32, ARangesIter<'a>>;
    is_sssu::<AAssumeSortedStarts>();
    is_like_btreeset_iter::<AAssumeSortedStarts>();

    type ACheckSortedDisjoint<'a> = CheckSortedDisjoint<i32, ARangesIter<'a>>;
    is_sssu::<ACheckSortedDisjoint>();
    type BCheckSortedDisjoint =
        CheckSortedDisjoint<i32, std::array::IntoIter<RangeInclusive<i32>, 0>>;
    is_like_check_sorted_disjoint::<BCheckSortedDisjoint>();

    type ADynSortedDisjoint<'a> = DynSortedDisjoint<'a, i32>;
    is_like_dyn_sorted_disjoint::<ADynSortedDisjoint>();
}

const fn is_ddcppdheo<
    T: std::fmt::Debug
        + fmt::Display
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

const fn is_sssu<T: Sized + Send + Sync + Unpin>() {}
const fn is_like_btreeset_iter<
    T: Clone + std::fmt::Debug + FusedIterator + Iterator, // cmk DoubleEndedIterator  + ExactSizeIterator,
>() {
}

const fn is_like_btreeset_into_iter<T: std::fmt::Debug + FusedIterator + Iterator>() {}

const fn is_like_btreeset<
    T: Clone
        + std::fmt::Debug
        + Default
        + Eq
        + std::hash::Hash
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
        + std::fmt::Debug
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
fn test_merge() {
    let a = RangeSetBlaze::from_iter([1..=2, 5..=100]);
    let b = RangeSetBlaze::from_iter([1..=2, 5..=6]);
    let m = Merge::new(a.ranges(), b.ranges());
    // Just sorts by start, doesn't merge ranges.
    assert_eq!(m.size_hint(), (4, Some(4)));

    let c = RangeSetBlaze::from_iter([1..=2, 5..=100]);
    let m = KMerge::new([a.ranges(), b.ranges(), c.ranges()]);
    assert_eq!(m.size_hint(), (6, Some(6)));
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
#[allow(clippy::many_single_char_names)]
fn sub1() {
    let a0 = RangeSetBlaze::from_iter([1..=6]);
    let a1 = RangeSetBlaze::from_iter([8..=9]);
    let a2 = RangeSetBlaze::from_iter([11..=15]);
    let a01 = &a0 | &a1;
    let not_a01 = !&a01;
    let a = &a01 - &a2;
    let b = a01.ranges() - a2.ranges();
    let c = !not_a01.ranges() - a2.ranges();
    let d = (a0.ranges() | a1.ranges()) - a2.ranges();
    let f = UnionIter::new(a01.ranges()) - UnionIter::new(a2.ranges());
    assert!(a.ranges().equal(b));
    assert!(a.ranges().equal(c));
    assert!(a.ranges().equal(d));
    assert!(a.ranges().equal(f));
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
#[allow(clippy::many_single_char_names)]
fn bitand() {
    let a0 = RangeSetBlaze::from_iter([1..=6]);
    let a1 = RangeSetBlaze::from_iter([8..=9]);
    let a2 = RangeSetBlaze::from_iter([11..=15]);
    let a01 = &a0 | &a1;
    let not_a01 = !&a01;
    let a = &a01 & &a2;
    let b = a01.ranges() & a2.ranges();
    let c = !not_a01.ranges() & a2.ranges();
    let d = (a0.ranges() | a1.ranges()) & a2.ranges();
    let f = UnionIter::new(a01.ranges()) & UnionIter::new(a2.ranges());
    assert!(a.ranges().equal(b));
    assert!(a.ranges().equal(c));
    assert!(a.ranges().equal(d));
    assert!(a.ranges().equal(f));
}
