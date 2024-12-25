#![cfg(test)]

use crate::keys::IntoKeys;
use crate::keys::Keys;
use crate::sorted_disjoint_map::Priority;
use crate::sorted_disjoint_map::RangeToRangeValueIter;
use crate::unsorted_priority_map::AssumePrioritySortedStartsMap;
use crate::unsorted_priority_map::UnsortedPriorityMap;
use crate::values::IntoValues;
use crate::values::Values;
use crate::CheckSortedDisjointMap;
use crate::DynSortedDisjointMap;
use crate::Integer;
use crate::IntersectionIterMap;
use crate::IntoIterMap;
use crate::IntoRangeValuesIter;
use crate::IterMap;
use crate::KMergeMap;
use crate::MergeMap;
use crate::RangeMapBlaze;
use crate::RangeValuesIter;
use crate::RangesIter;
use crate::SymDiffIterMap;
use crate::UnionIterMap;
use alloc::string::ToString;
use core::any::Any;
use core::fmt;
use core::iter::FusedIterator;
use core::ops::RangeInclusive;
use itertools::Itertools;
use std::prelude::v1::*;
use std::vec;
use std::{format, println};

use wasm_bindgen_test::*;
wasm_bindgen_test_configure!(run_in_browser);

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
fn map_step_by_step() {
    let (s1, s2) = ("a".to_string(), "b".to_string());
    let input = [(1, &s2), (2, &s2), (0, &s1)];

    let iter = input.into_iter();
    let iter = iter.map(|(x, value)| (x..=x, value));
    let iter = UnsortedPriorityMap::new(iter);
    let vs = format!("{:?}", iter.collect::<Vec<_>>());
    println!("{vs}");
    assert_eq!(
        vs,
        r#"[Priority { range_value: (1..=2, "b"), priority_number: 0 }, Priority { range_value: (0..=0, "a"), priority_number: 2 }]"#
    );

    let iter = input.into_iter();
    let iter = iter.map(|(x, value)| (x..=x, value));
    let iter = UnsortedPriorityMap::new(iter);
    let iter = iter.sorted_by(|a, b| {
        // We sort only by start -- priority is not used until later.
        a.start().cmp(&b.start())
    });
    let iter = AssumePrioritySortedStartsMap::new(iter);
    let vs = format!("{:?}", iter.collect::<Vec<_>>());
    println!("{vs}");
    assert_eq!(vs, "[Priority { range_value: (0..=0, \"a\"), priority_number: 2 }, Priority { range_value: (1..=2, \"b\"), priority_number: 0 }]");

    let iter = input.into_iter();
    let iter = iter.map(|(x, value)| (x..=x, value));
    let iter = UnsortedPriorityMap::new(iter);
    let iter = iter.sorted_by(|a, b| {
        // We sort only by start -- priority is not used until later.
        a.start().cmp(&b.start())
    });
    let iter = AssumePrioritySortedStartsMap::new(iter);
    let iter = UnionIterMap::new(iter);
    let vs = format!("{:?}", iter.collect::<Vec<_>>());
    println!("{vs}");
    assert_eq!(vs, "[(0..=0, \"a\"), (1..=2, \"b\")]");

    let range_map_blaze = RangeMapBlaze::<u8, String>::from_iter(input);
    println!("{range_map_blaze}");
    assert_eq!(range_map_blaze.to_string(), r#"(0..=0, "a"), (1..=2, "b")"#);
}

fn format_range_values<'a, T>(iter: impl Iterator<Item = (RangeInclusive<T>, &'a u8)>) -> String
where
    T: Integer + fmt::Display + 'a, // Assuming T implements Display for formatting
                                    // V: ValueOwned + fmt::Display + 'a, // V must implement Display to be formatted with {}
{
    let mut vs = String::new();
    for (range, value) in iter {
        vs.push_str(&format!("{:?}{} ", range, *value as char,));
    }
    vs
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
fn map_repro_206() {
    let input_string = "127e 2d 29e 84a 17a 79d 174e 125b 123a 123b 98c 132d 99e 186b 253d 31d 121c 151a 168e 208c 47e 42e 86a 21b 7b 238d 148a 151a 227d 173d 145b 18e 219e 16c 214b 213a 155e 27e 24d 38c 59c 16c 183d 125d 210d 99e 43e 189e 147a 90d 42a 220e 35b 120d 185d 177a 102a 22b 124b 140a 199e 143c 32d 225a 223e 137e 177e 234e 97a 166a 83e 213a 147b 128a 150c 12c 199c 152c 79b 164b 204b 235e 37e 14c 19b 49a 1c 115b 31d 102b 59b 129b 104d 70c 229b 205b 101c 58d 114a 228d 173e 139d 147b 32c 198e 194c 18a 77a 100e 196a 46b 81a 63d 198a 242a 131b 153e 113b 19d 253e 195c 209e 201c 139d 47a 223d 240b 203d 84a 214d 129e 73d 55d 193e 129d 7c 193e 2c 235c 39c 88d 175c 190c 239a 219d 121a 88d 175d 117e 23a 102d 165a 58a 229a 100b 13b 113e 26a 49e 37e 126a 251b 47e 77a 206b ";
    let mut input = Vec::<(u8, &u8)>::new();
    for pair in input_string.split_whitespace() {
        let bytes = pair.as_bytes(); // Get the byte slice of the pair
        let c = &bytes[bytes.len() - 1]; // Last byte as char
        let num = pair[..pair.len() - 1].parse::<u8>().expect("parse failed");
        input.push((num, c)); // Add the (u8, &str) pair to inputs
    }

    let iter = input.clone().into_iter();
    let iter = iter.map(|(x, value)| (x..=x, value));
    // let vs = format_range_values(iter);
    // println!("{vs}");
    // assert_eq!(vs, "127..=127e 2..=2d 29..=29e 84..=84a 17..=17a 79..=79d 174..=174e 125..=125b 123..=123a 123..=123b 98..=98c 132..=132d 99..=99e 186..=186b 253..=253d 31..=31d 121..=121c 151..=151a 168..=168e 208..=208c 47..=47e 42..=42e 86..=86a 21..=21b 7..=7b 238..=238d 148..=148a 151..=151a 227..=227d 173..=173d 145..=145b 18..=18e 219..=219e 16..=16c 214..=214b 213..=213a 155..=155e 27..=27e 24..=24d 38..=38c 59..=59c 16..=16c 183..=183d 125..=125d 210..=210d 99..=99e 43..=43e 189..=189e 147..=147a 90..=90d 42..=42a 220..=220e 35..=35b 120..=120d 185..=185d 177..=177a 102..=102a 22..=22b 124..=124b 140..=140a 199..=199e 143..=143c 32..=32d 225..=225a 223..=223e 137..=137e 177..=177e 234..=234e 97..=97a 166..=166a 83..=83e 213..=213a 147..=147b 128..=128a 150..=150c 12..=12c 199..=199c 152..=152c 79..=79b 164..=164b 204..=204b 235..=235e 37..=37e 14..=14c 19..=19b 49..=49a 1..=1c 115..=115b 31..=31d 102..=102b 59..=59b 129..=129b 104..=104d 70..=70c 229..=229b 205..=205b 101..=101c 58..=58d 114..=114a 228..=228d 173..=173e 139..=139d 147..=147b 32..=32c 198..=198e 194..=194c 18..=18a 77..=77a 100..=100e 196..=196a 46..=46b 81..=81a 63..=63d 198..=198a 242..=242a 131..=131b 153..=153e 113..=113b 19..=19d 253..=253e 195..=195c 209..=209e 201..=201c 139..=139d 47..=47a 223..=223d 240..=240b 203..=203d 84..=84a 214..=214d 129..=129e 73..=73d 55..=55d 193..=193e 129..=129d 7..=7c 193..=193e 2..=2c 235..=235c 39..=39c 88..=88d 175..=175c 190..=190c 239..=239a 219..=219d 121..=121a 88..=88d 175..=175d 117..=117e 23..=23a 102..=102d 165..=165a 58..=58a 229..=229a 100..=100b 13..=13b 113..=113e 26..=26a 49..=49e 37..=37e 126..=126a 251..=251b 47..=47e 77..=77a 206..=206b ");

    let iter = UnsortedPriorityMap::new(iter);
    let iter = iter.sorted_by(|a, b| {
        // We sort only by start -- priority is not used until later.
        a.start().cmp(&b.start())
    });
    let iter = AssumePrioritySortedStartsMap::new(iter);
    // let vs = format_range_values(iter);
    // println!("{vs}");
    // assert_eq!(vs, "1..=1c 2..=2c 2..=2d 7..=7c 7..=7b 12..=12c 13..=13b 14..=14c 16..=16c 16..=16c 17..=17a 18..=18a 18..=18e 19..=19d 19..=19b 21..=21b 22..=22b 23..=23a 24..=24d 26..=26a 27..=27e 29..=29e 31..=31d 31..=31d 32..=32c 32..=32d 35..=35b 37..=37e 37..=37e 38..=38c 39..=39c 42..=42a 42..=42e 43..=43e 46..=46b 47..=47e 47..=47a 47..=47e 49..=49e 49..=49a 55..=55d 58..=58a 58..=58d 59..=59b 59..=59c 63..=63d 70..=70c 73..=73d 77..=77a 77..=77a 79..=79b 79..=79d 81..=81a 83..=83e 84..=84a 84..=84a 86..=86a 88..=88d 88..=88d 90..=90d 97..=97a 98..=98c 99..=99e 99..=99e 100..=100b 100..=100e 101..=101c 102..=102d 102..=102b 102..=102a 104..=104d 113..=113e 113..=113b 114..=114a 115..=115b 117..=117e 120..=120d 121..=121a 121..=121c 123..=123b 123..=123a 124..=124b 125..=125d 125..=125b 126..=126a 127..=127e 128..=128a 129..=129d 129..=129e 129..=129b 131..=131b 132..=132d 137..=137e 139..=139d 139..=139d 140..=140a 143..=143c 145..=145b 147..=147b 147..=147b 147..=147a 148..=148a 150..=150c 151..=151a 151..=151a 152..=152c 153..=153e 155..=155e 164..=164b 165..=165a 166..=166a 168..=168e 173..=173e 173..=173d 174..=174e 175..=175d 175..=175c 177..=177e 177..=177a 183..=183d 185..=185d 186..=186b 189..=189e 190..=190c 193..=193e 193..=193e 194..=194c 195..=195c 196..=196a 198..=198a 198..=198e 199..=199c 199..=199e 201..=201c 203..=203d 204..=204b 205..=205b 206..=206b 208..=208c 209..=209e 210..=210d 213..=213a 213..=213a 214..=214d 214..=214b 219..=219d 219..=219e 220..=220e 223..=223d 223..=223e 225..=225a 227..=227d 228..=228d 229..=229a 229..=229b 234..=234e 235..=235c 235..=235e 238..=238d 239..=239a 240..=240b 242..=242a 251..=251b 253..=253e 253..=253d ");

    let iter = UnionIterMap::new(iter);
    let vs = format_range_values(iter);
    println!("{vs}");
    assert_eq!(vs, "1..=1c 2..=2d 7..=7b 12..=12c 13..=13b 14..=14c 16..=16c 17..=17a 18..=18e 19..=19b 21..=22b 23..=23a 24..=24d 26..=26a 27..=27e 29..=29e 31..=32d 35..=35b 37..=37e 38..=39c 42..=43e 46..=46b 47..=47e 49..=49a 55..=55d 58..=58d 59..=59c 63..=63d 70..=70c 73..=73d 77..=77a 79..=79d 81..=81a 83..=83e 84..=84a 86..=86a 88..=88d 90..=90d 97..=97a 98..=98c 99..=100e 101..=101c 102..=102a 104..=104d 113..=113b 114..=114a 115..=115b 117..=117e 120..=120d 121..=121c 123..=123a 124..=125b 126..=126a 127..=127e 128..=128a 129..=129b 131..=131b 132..=132d 137..=137e 139..=139d 140..=140a 143..=143c 145..=145b 147..=148a 150..=150c 151..=151a 152..=152c 153..=153e 155..=155e 164..=164b 165..=166a 168..=168e 173..=173d 174..=174e 175..=175c 177..=177a 183..=183d 185..=185d 186..=186b 189..=189e 190..=190c 193..=193e 194..=195c 196..=196a 198..=199e 201..=201c 203..=203d 204..=206b 208..=208c 209..=209e 210..=210d 213..=213a 214..=214b 219..=220e 223..=223e 225..=225a 227..=228d 229..=229b 234..=235e 238..=238d 239..=239a 240..=240b 242..=242a 251..=251b 253..=253d ");

    // let range_map_blaze = RangeMapBlaze::<u8, u8>::from_iter(input.clone());
    // assert_eq!(
    //     range_map_blaze.to_string(),
    //     "(97..=97, 101), (98..=98, 99), (100..=100, 101), (106..=106, 98)"
    // );
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
fn map_repro_106() {
    let input_string = "100e 106b 97c 98c 97e";
    let mut input = Vec::<(u8, &u8)>::new();
    for pair in input_string.split_whitespace() {
        let bytes = pair.as_bytes(); // Get the byte slice of the pair
        let c = &bytes[bytes.len() - 1]; // Last byte as char
        let num = pair[..pair.len() - 1].parse::<u8>().expect("parse failed");
        input.push((num, c)); // Add the (u8, &str) pair to inputs
    }

    let iter = input.clone().into_iter();
    let iter = iter.map(|(x, value)| (x..=x, value));
    let iter = UnsortedPriorityMap::new(iter);
    let iter = iter.sorted_by(|a, b| {
        // We sort only by start -- priority is not used until later.
        a.start().cmp(&b.start())
    });
    let iter = AssumePrioritySortedStartsMap::new(iter);
    let iter = UnionIterMap::new(iter);
    let vs = format_range_values(iter);
    println!("{vs}");
    assert_eq!(vs, "97..=98c 100..=100e 106..=106b ");

    let range_map_blaze = RangeMapBlaze::<u8, u8>::from_iter(input.clone());
    assert_eq!(
        range_map_blaze.to_string(),
        "(97..=98, 99), (100..=100, 101), (106..=106, 98)"
    );
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
fn map_repro1() {
    let (s1, s2, s3) = ("a".to_string(), "b".to_string(), "c".to_string());
    let mut range_map_blaze =
        RangeMapBlaze::from_iter([(20..=21, &s1), (24..=24, &s2), (25..=29, &s2)]);
    println!("{range_map_blaze}");
    assert_eq!(
        range_map_blaze.to_string(),
        r#"(20..=21, "a"), (24..=29, "b")"#
    );
    range_map_blaze.internal_add(25..=25, &s3);
    println!("{range_map_blaze}");
    assert_eq!(
        range_map_blaze.to_string(),
        r#"(20..=21, "a"), (24..=24, "b"), (25..=25, "c"), (26..=29, "b")"#
    );
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
fn test_coverage_8() {
    let mut a = RangeMapBlaze::from_iter([(1u128..=2, "Hello"), (3..=4, "World")]);
    a.internal_add(0..=u128::MAX, "Hello");
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
#[allow(clippy::reversed_empty_ranges)]
fn test_coverage_9() {
    let mut a = RangeMapBlaze::from_iter([(1u128..=2, "Hello"), (3..=4, "World")]);
    let b = a.clone();
    a.internal_add(1..=0, "Hello"); // adding empty
    assert_eq!(a, b);
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
fn test_eq_priority() {
    let a = Priority::new((1..=2, &"a"), 0);
    let b = Priority::new((1..=2, &"a"), 1);
    assert!(a != b);
}

#[allow(clippy::items_after_statements)]
#[test]
const fn check_traits() {
    // Debug/Display/Clone/PartialEq/PartialOrd/Default/Hash/Eq/Ord/Send/Sync
    type ARangeMapBlaze = RangeMapBlaze<i32, u64>;
    is_sssu::<ARangeMapBlaze>();
    is_ddcppdheo::<ARangeMapBlaze>();
    is_like_btreemap::<ARangeMapBlaze>();

    type ARangeValuesIter<'a> = RangeValuesIter<'a, i32, u64>;
    is_sssu::<ARangeValuesIter>();
    is_like_btreemap_iter::<ARangeValuesIter>();

    type AIntoRangeValuesIter = IntoRangeValuesIter<i32, u64>;
    is_sssu::<AIntoRangeValuesIter>();
    is_like_btreemap_into_iter::<AIntoRangeValuesIter>();

    type AIterMap<'a> = IterMap<i32, &'a u64, ARangeValuesIter<'a>>;
    is_sssu::<AIterMap>();
    is_like_btreemap_iter::<AIterMap>();

    type AIntoIterMap = IntoIterMap<i32, u64>;
    is_sssu::<AIntoIterMap>();
    is_like_btreemap_into_iter::<AIntoIterMap>();

    type AKMergeMap<'a> = crate::KMergeMap<i32, &'a u64, ARangeValuesIter<'a>>;
    is_sssu::<AKMergeMap>();
    is_like_btreemap_iter::<AKMergeMap>();

    type AMergeMap<'a> = crate::MergeMap<i32, &'a u64, ARangeValuesIter<'a>, ARangeValuesIter<'a>>;
    is_sssu::<AMergeMap>();
    is_like_btreemap_iter::<AMergeMap>();

    type AAssumePrioritySortedStartsMap<'a> =
        AssumePrioritySortedStartsMap<i32, &'a u64, vec::IntoIter<Priority<i32, &'a u64>>>;
    is_sssu::<AAssumePrioritySortedStartsMap>();
    is_like_btreemap_iter::<AAssumePrioritySortedStartsMap>();

    type AUnionIterMap<'a> = UnionIterMap<i32, &'a u64, AAssumePrioritySortedStartsMap<'a>>;
    is_sssu::<AUnionIterMap>();
    is_like_btreemap_iter::<AUnionIterMap>();

    type ASymDiffIterMap<'a> = SymDiffIterMap<i32, &'a u64, AAssumePrioritySortedStartsMap<'a>>;
    is_sssu::<ASymDiffIterMap>();
    is_like_btreemap_iter::<ASymDiffIterMap>();

    type ARangesIter<'a> = RangesIter<'a, i32>;

    type AIntersectionIterMap<'a> =
        IntersectionIterMap<i32, &'a u64, ARangeValuesIter<'a>, ARangesIter<'a>>;
    is_sssu::<AIntersectionIterMap>();
    is_like_btreemap_iter::<AIntersectionIterMap>();

    type AKeys<'a> = Keys<i32, &'a u64, ARangeValuesIter<'a>>;
    is_sssu::<AKeys>();
    is_like_btreemap_iter::<AKeys>();

    type AIntoKeys = IntoKeys<i32, u64>;
    is_sssu::<AIntoKeys>();
    is_like_btreemap_into_iter::<AIntoKeys>();

    type AValues<'a> = Values<i32, &'a u64, ARangeValuesIter<'a>>;
    is_sssu::<AValues>();
    is_like_btreemap_iter::<AValues>();

    type AIntoValues = IntoValues<i32, u64>;
    is_sssu::<AIntoValues>();
    is_like_btreemap_into_iter::<AIntoValues>();

    type ARangeToRangeValueIter<'a> = RangeToRangeValueIter<'a, i32, u64, ARangesIter<'a>>;
    is_sssu::<ARangeToRangeValueIter>();
    is_like_btreemap_iter::<ARangeToRangeValueIter>();

    type ACheckSortedDisjointMap<'a> = CheckSortedDisjointMap<i32, &'a u64, ARangeValuesIter<'a>>;
    is_sssu::<ACheckSortedDisjointMap>();
    type BCheckSortedDisjointMap<'a> = CheckSortedDisjointMap<
        i32,
        &'a u64,
        std::array::IntoIter<(RangeInclusive<i32>, &'a u64), 0>,
    >;
    is_like_check_sorted_disjoint_map::<BCheckSortedDisjointMap>();

    type ADynSortedDisjointMap<'a> = DynSortedDisjointMap<'a, i32, &'a u64>;
    is_like_dyn_sorted_disjoint_map::<ADynSortedDisjointMap>();
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
const fn is_like_btreemap_iter<
    T: Clone + std::fmt::Debug + FusedIterator + Iterator, // cmk DoubleEndedIterator  + ExactSizeIterator,
>() {
}

const fn is_like_btreemap_into_iter<T: std::fmt::Debug + FusedIterator + Iterator>() {}

const fn is_like_btreemap<
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
        + core::any::Any
        + ToOwned,
>() {
}

const fn is_like_check_sorted_disjoint_map<
    T: Clone
        + std::fmt::Debug
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

const fn is_like_dyn_sorted_disjoint_map<T: IntoIterator + Unpin + Any>() {}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
fn test_merge_map() {
    let a = RangeMapBlaze::from_iter([(1..=2, "a"), (3..=4, "b")]).into_range_values();
    let b = RangeMapBlaze::from_iter([(1..=2, "a"), (13..=14, "b")]).into_range_values();
    assert_eq!(MergeMap::new(a, b).size_hint(), (0, None));

    let a = RangeMapBlaze::from_iter([(1..=2, "a"), (3..=4, "b")]).into_range_values();
    let b = RangeMapBlaze::from_iter([(1..=2, "a"), (13..=14, "b")]).into_range_values();
    let c = RangeMapBlaze::from_iter([(1..=2, "a"), (3..=4, "b")]).into_range_values();
    assert_eq!(KMergeMap::new([a, b, c]).size_hint(), (3, None));
}
