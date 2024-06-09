// cmk move this file to integration tests
#![cfg(test)]
#![cfg(not(target_arch = "wasm32"))]

use crate::unsorted_disjoint_map::AssumePrioritySortedStartsMap;
use crate::unsorted_disjoint_map::UnsortedPriorityDisjointMap;
use crate::Integer;
use crate::RangeMapBlaze;
use crate::UnionIterMap;
use alloc::collections::BTreeMap;
use core::fmt;
use core::ops::RangeInclusive;
use itertools::Itertools;
use quickcheck_macros::quickcheck;

#[test]
fn map_step_by_step() {
    let (s1, s2) = ("a".to_string(), "b".to_string());
    let input = [(1, &s2), (2, &s2), (0, &s1)];

    let iter = input.into_iter();
    let iter = iter.map(|(x, value)| (x..=x, value));
    let iter = UnsortedPriorityDisjointMap::new(iter.into_iter());
    let vs = format!("{:?}", iter.collect::<Vec<_>>());
    println!("{vs}");
    assert_eq!(
        vs,
        r#"[Priority { range_value: (1..=2, "b"), priority_number: 0 }, Priority { range_value: (0..=0, "a"), priority_number: 2 }]"#
    );

    let iter = input.into_iter();
    let iter = iter.map(|(x, value)| (x..=x, value));
    let iter = UnsortedPriorityDisjointMap::new(iter.into_iter());
    let iter = iter.into_iter().sorted_by(|a, b| {
        // We sort only by start -- priority is not used until later.
        a.start().cmp(&b.start())
    });
    let iter = AssumePrioritySortedStartsMap::new(iter);
    let vs = format!("{:?}", iter.collect::<Vec<_>>());
    println!("{vs}");
    assert_eq!(vs, "[Priority { range_value: (0..=0, \"a\"), priority_number: 2 }, Priority { range_value: (1..=2, \"b\"), priority_number: 0 }]");

    let iter = input.into_iter();
    let iter = iter.map(|(x, value)| (x..=x, value));
    let iter = UnsortedPriorityDisjointMap::new(iter.into_iter());
    let iter = iter.into_iter().sorted_by(|a, b| {
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

// use self::map::ValueOwned;
// use super::*;
// use crate::intersection_iter_map::IntersectionIterMap;
// use crate::sorted_disjoint_map::DebugToString;
// use crate::sorted_disjoint_map::SortedDisjointMap;
// use crate::sym_diff_iter_map::SymDiffIterMap;
// use crate::union_iter_map::UnionIterMap;
// use crate::unsorted_disjoint_map::{AssumePrioritySortedStartsMap, UnsortedPriorityDisjointMap};
// use alloc::collections::BTreeMap;
// use itertools::Itertools;
// use rand::seq::SliceRandom;
// use rand::{rngs::StdRng, Rng, SeedableRng};
// // cmk what if they forget to import this the thing that lets | work?

// // cmk must test coverage

#[test]
fn map_repro_206() {
    let input_string = "127e 2d 29e 84a 17a 79d 174e 125b 123a 123b 98c 132d 99e 186b 253d 31d 121c 151a 168e 208c 47e 42e 86a 21b 7b 238d 148a 151a 227d 173d 145b 18e 219e 16c 214b 213a 155e 27e 24d 38c 59c 16c 183d 125d 210d 99e 43e 189e 147a 90d 42a 220e 35b 120d 185d 177a 102a 22b 124b 140a 199e 143c 32d 225a 223e 137e 177e 234e 97a 166a 83e 213a 147b 128a 150c 12c 199c 152c 79b 164b 204b 235e 37e 14c 19b 49a 1c 115b 31d 102b 59b 129b 104d 70c 229b 205b 101c 58d 114a 228d 173e 139d 147b 32c 198e 194c 18a 77a 100e 196a 46b 81a 63d 198a 242a 131b 153e 113b 19d 253e 195c 209e 201c 139d 47a 223d 240b 203d 84a 214d 129e 73d 55d 193e 129d 7c 193e 2c 235c 39c 88d 175c 190c 239a 219d 121a 88d 175d 117e 23a 102d 165a 58a 229a 100b 13b 113e 26a 49e 37e 126a 251b 47e 77a 206b ";
    let mut input = Vec::<(u8, &u8)>::new();
    for pair in input_string.split_whitespace() {
        let bytes = pair.as_bytes(); // Get the byte slice of the pair
        let c = &bytes[bytes.len() - 1]; // Last byte as char
        let num = pair[..pair.len() - 1].parse::<u8>().unwrap();
        input.push((num, c)); // Add the (u8, &str) pair to inputs
    }

    let iter = input.clone().into_iter();
    let iter = iter.map(|(x, value)| (x..=x, value));
    // let vs = format_range_values(iter);
    // println!("{vs}");
    // assert_eq!(vs, "127..=127e 2..=2d 29..=29e 84..=84a 17..=17a 79..=79d 174..=174e 125..=125b 123..=123a 123..=123b 98..=98c 132..=132d 99..=99e 186..=186b 253..=253d 31..=31d 121..=121c 151..=151a 168..=168e 208..=208c 47..=47e 42..=42e 86..=86a 21..=21b 7..=7b 238..=238d 148..=148a 151..=151a 227..=227d 173..=173d 145..=145b 18..=18e 219..=219e 16..=16c 214..=214b 213..=213a 155..=155e 27..=27e 24..=24d 38..=38c 59..=59c 16..=16c 183..=183d 125..=125d 210..=210d 99..=99e 43..=43e 189..=189e 147..=147a 90..=90d 42..=42a 220..=220e 35..=35b 120..=120d 185..=185d 177..=177a 102..=102a 22..=22b 124..=124b 140..=140a 199..=199e 143..=143c 32..=32d 225..=225a 223..=223e 137..=137e 177..=177e 234..=234e 97..=97a 166..=166a 83..=83e 213..=213a 147..=147b 128..=128a 150..=150c 12..=12c 199..=199c 152..=152c 79..=79b 164..=164b 204..=204b 235..=235e 37..=37e 14..=14c 19..=19b 49..=49a 1..=1c 115..=115b 31..=31d 102..=102b 59..=59b 129..=129b 104..=104d 70..=70c 229..=229b 205..=205b 101..=101c 58..=58d 114..=114a 228..=228d 173..=173e 139..=139d 147..=147b 32..=32c 198..=198e 194..=194c 18..=18a 77..=77a 100..=100e 196..=196a 46..=46b 81..=81a 63..=63d 198..=198a 242..=242a 131..=131b 153..=153e 113..=113b 19..=19d 253..=253e 195..=195c 209..=209e 201..=201c 139..=139d 47..=47a 223..=223d 240..=240b 203..=203d 84..=84a 214..=214d 129..=129e 73..=73d 55..=55d 193..=193e 129..=129d 7..=7c 193..=193e 2..=2c 235..=235c 39..=39c 88..=88d 175..=175c 190..=190c 239..=239a 219..=219d 121..=121a 88..=88d 175..=175d 117..=117e 23..=23a 102..=102d 165..=165a 58..=58a 229..=229a 100..=100b 13..=13b 113..=113e 26..=26a 49..=49e 37..=37e 126..=126a 251..=251b 47..=47e 77..=77a 206..=206b ");

    let iter = UnsortedPriorityDisjointMap::new(iter.into_iter());
    let iter = iter.into_iter().sorted_by(|a, b| {
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
fn map_repro_106() {
    let input_string = "100e 106b 97c 98c 97e";
    let mut input = Vec::<(u8, &u8)>::new();
    for pair in input_string.split_whitespace() {
        let bytes = pair.as_bytes(); // Get the byte slice of the pair
        let c = &bytes[bytes.len() - 1]; // Last byte as char
        let num = pair[..pair.len() - 1].parse::<u8>().unwrap();
        input.push((num, c)); // Add the (u8, &str) pair to inputs
    }

    let iter = input.clone().into_iter();
    let iter = iter.map(|(x, value)| (x..=x, value));
    let iter = UnsortedPriorityDisjointMap::new(iter.into_iter());
    let iter = iter.into_iter().sorted_by(|a, b| {
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

// #[test]
// fn map_random_from_iter_item() {
//     // cmk all these tests should test on size zero, too.
//     for seed in 0..20 {
//         println!("seed: {seed}");
//         let mut rng = StdRng::seed_from_u64(seed);

//         let mut btree_map = BTreeMap::new();

//         let mut inputs = Vec::new();
//         for _ in 0..500 {
//             let key = rng.gen_range(0..=255u32); // cmk change back to u8s
//             let value = ['a', 'b', 'c'].choose(&mut rng).unwrap(); // cmk allow more than references

//             print!("{key}{value} ");
//             inputs.push((key, value));

//             // cmk fix so don't need to clone and can use .iter()
//             let range_map_blaze = RangeMapBlaze::<_, char>::from_iter(inputs.clone());
//             // Only insert if the key does not already exist
//             btree_map.entry(key).or_insert(value);
//             if !equal_maps(&range_map_blaze, &btree_map) {
//                 println!();
//                 let _range_map_blaze = RangeMapBlaze::<_, char>::from_iter(inputs.clone());
//                 panic!();
//             }
//         }
//     }
// }

// #[test]
// fn map_random_from_iter_range() {
//     for seed in 0..20 {
//         println!("seed: {seed}");
//         let mut rng = StdRng::seed_from_u64(seed);

//         let mut btree_map = BTreeMap::new();

//         let mut inputs = Vec::new();
//         for _ in 0..500 {
//             let start = rng.gen_range(0..=255u8);
//             let end = rng.gen_range(start..=255u8);
//             let key = start..=end;
//             let value = ['a', 'b', 'c'].choose(&mut rng).unwrap(); // cmk allow more than references

//             // print!("{key}{value} ");
//             inputs.push((key.clone(), value));

//             // cmk fix so don't need to clone and can use .iter()
//             let range_map_blaze = RangeMapBlaze::<u8, char>::from_iter(inputs.clone());
//             for k in key.clone() {
//                 btree_map.entry(k).or_insert(value);
//             }
//             if !equal_maps(&range_map_blaze, &btree_map) {
//                 let _range_map_blaze = RangeMapBlaze::<u8, char>::from_iter(inputs.clone());
//                 panic!();
//             }
//         }
//     }
// }

// #[test]
// fn map_random_insert() {
//     for seed in 0..20 {
//         println!("seed: {seed}");
//         let mut rng = StdRng::seed_from_u64(seed);

//         let mut btree_map = BTreeMap::new();
//         let mut range_map_blaze = RangeMapBlaze::new();
//         let mut inputs = Vec::new();

//         for _ in 0..500 {
//             let key = rng.gen_range(0..=255u8);
//             let value = ["aaa", "bbb", "ccc"].choose(&mut rng).unwrap();
//             // print!("{key}{value} ");

//             btree_map.insert(key, value);
//             range_map_blaze.insert(key, *value);
//             if equal_maps(&range_map_blaze, &btree_map) {
//                 inputs.push((key, value));
//                 continue;
//             }

//             // if range_map_blaze and btree_map are not equal, then we have a bug, so repro it:

//             let mut range_map_blaze = RangeMapBlaze::from_iter(inputs.clone().into_iter());
//             range_map_blaze.insert(key, *value);
//             assert!(equal_maps(&range_map_blaze, &btree_map));
//         }
//     }
// }

// #[test]
// fn map_random_insert_range() {
//     for seed in 0..20 {
//         println!("seed: {seed}");
//         let mut rng = StdRng::seed_from_u64(seed);

//         let mut btree_map = BTreeMap::new();
//         let mut range_map_blaze = RangeMapBlaze::new();
//         let mut inputs = Vec::new();

//         for _ in 0..500 {
//             let start = rng.gen_range(0..=255u8);
//             let end = rng.gen_range(start..=255u8);
//             let key = start..=end;
//             let value = ["aaa", "bbb", "ccc"].choose(&mut rng).unwrap();
//             // print!("{key}{value} ");

//             for k in key.clone() {
//                 btree_map.insert(k, value);
//             }
//             range_map_blaze.ranges_insert(key.clone(), *value);
//             if equal_maps(&range_map_blaze, &btree_map) {
//                 inputs.push((key.clone(), value));
//                 continue;
//             }

//             // if range_map_blaze and btree_map are not equal, then we have a bug, so repro it:

//             let mut range_map_blaze = RangeMapBlaze::from_iter(inputs.clone().into_iter());
//             println!("{range_map_blaze}");
//             println!("About to insert {}..={} -> {value}", key.start(), key.end());
//             range_map_blaze.ranges_insert(key.clone(), *value);
//             assert!(equal_maps(&range_map_blaze, &btree_map));
//         }
//     }
// }

// #[test]
// fn map_random_ranges() {
//     let values = ['a', 'b', 'c'];
//     for seed in 0..20 {
//         println!("seed: {seed}");
//         let mut rng = StdRng::seed_from_u64(seed);

//         let mut range_set_blaze = RangeSetBlaze::new();
//         let mut range_map_blaze = RangeMapBlaze::new();
//         let mut inputs = Vec::<(u8, &char)>::new();

//         for _ in 0..500 {
//             let key = rng.gen_range(0..=255u8);
//             let value = values.choose(&mut rng).unwrap();
//             // print!("{key}{value} ");

//             range_set_blaze.insert(key);
//             range_map_blaze.insert(key, *value);
//             if range_set_blaze.ranges().eq(range_map_blaze.ranges()) {
//                 inputs.push((key, value));
//                 continue;
//             }

//             // if range_map_blaze and btree_map are not equal, then we have a bug, so repro it:

//             let mut range_map_blaze = RangeMapBlaze::from_iter(inputs.clone().into_iter());
//             range_map_blaze.insert(key, *value);
//             assert!(range_set_blaze.ranges().eq(range_map_blaze.ranges()));
//         }
//     }
// }

// #[test]
// fn map_random_ranges_ranges() {
//     let values = ['a', 'b', 'c'];
//     for seed in 0..20 {
//         println!("seed: {seed}");
//         let mut rng = StdRng::seed_from_u64(seed);

//         let mut range_set_blaze = RangeSetBlaze::new();
//         let mut range_map_blaze = RangeMapBlaze::new();
//         let mut inputs = Vec::new();

//         for _ in 0..500 {
//             let start = rng.gen_range(0..=255u8);
//             let end = rng.gen_range(start..=255u8);
//             let key = start..=end;
//             let value = values.choose(&mut rng).unwrap();
//             // print!("{key}{value} ");

//             range_set_blaze.ranges_insert(key.clone());
//             range_map_blaze.ranges_insert(key.clone(), *value);
//             if range_set_blaze.ranges().eq(range_map_blaze.ranges()) {
//                 inputs.push((key.clone(), value));
//                 continue;
//             }

//             // if range_map_blaze and btree_map are not equal, then we have a bug, so repro it:

//             let mut range_map_blaze = RangeMapBlaze::from_iter(inputs.clone().into_iter());
//             range_map_blaze.ranges_insert(key.clone(), *value);
//             assert!(range_set_blaze.ranges().eq(range_map_blaze.ranges()));
//         }
//     }
// }

// #[test]
// fn map_random_intersection() {
//     let values = ['a', 'b', 'c'];
//     for seed in 0..20 {
//         println!("seed: {seed}");
//         let mut rng = StdRng::seed_from_u64(seed);

//         let mut set0 = RangeSetBlaze::new();
//         let mut map0 = RangeMapBlaze::new();
//         // let mut inputs = Vec::<(u8, &char)>::new();

//         for _ in 0..500 {
//             let element = rng.gen_range(0..=255u8);
//             let key = rng.gen_range(0..=255u8);
//             let value = values.choose(&mut rng).unwrap();
//             // print!("{element},{key}{value} ");

//             set0.insert(element);
//             map0.insert(key, *value);

//             let intersection = IntersectionIterMap::new(map0.range_values(), set0.ranges());

//             let mut expected_keys = map0
//                 .ranges()
//                 .intersection(set0.ranges())
//                 .collect::<RangeSetBlaze<_>>();
//             if !expected_keys.is_empty() {
//                 // println!("expected_keys: {expected_keys}");
//             }
//             for range_value in intersection {
//                 let (range, value) = range_value;
//                 // println!();
//                 // print!("removing ");
//                 for k in range {
//                     assert_eq!(map0.get(k), Some(value));
//                     assert!(set0.contains(k));
//                     // print!("{k} ");
//                     assert!(expected_keys.remove(k));
//                 }
//                 // println!();
//             }
//             if !expected_keys.is_empty() {
//                 // eprintln!("{set0}");
//                 // eprintln!("{map0}");
//                 panic!("expected_keys should be empty: {expected_keys}");
//             }
//         }
//     }
// }

// #[test]
// fn map_tiny_symmetric_difference0() {
//     let mut map0 = RangeMapBlaze::new();
//     map0.insert(84, 'c');
//     map0.insert(85, 'c');
//     let mut map1 = RangeMapBlaze::new();
//     map1.insert(85, 'a');
//     let symmetric_difference = SymDiffIterMap::new2(map0.range_values(), map1.range_values());
//     assert_eq!(symmetric_difference.into_string(), "(84..=84, 'c')");
// }

// #[test]
// fn map_tiny_symmetric_difference1() {
//     let mut map0 = RangeMapBlaze::new();
//     map0.insert(187, 'a');
//     map0.insert(188, 'a');
//     map0.insert(189, 'a');
//     let mut map1 = RangeMapBlaze::new();
//     map1.insert(187, 'b');
//     map1.insert(189, 'c');
//     let symmetric_difference = SymDiffIterMap::new2(map0.range_values(), map1.range_values());
//     assert_eq!(symmetric_difference.into_string(), "(188..=188, 'a')");
// }

// #[test]
// fn map_random_symmetric_difference() {
//     let values = ['a', 'b', 'c'];
//     for seed in 0..20 {
//         println!("seed: {seed}");
//         let mut rng = StdRng::seed_from_u64(seed);

//         let mut map0 = RangeMapBlaze::new();
//         let mut map1 = RangeMapBlaze::new();
//         // let mut inputs = Vec::<(u8, &char)>::new();

//         for _ in 0..500 {
//             let key = rng.gen_range(0..=255u8);
//             let value = values.choose(&mut rng).unwrap();
//             map0.insert(key, *value);
//             print!("l{key}{value} ");
//             let key = rng.gen_range(0..=255u8);
//             let value = values.choose(&mut rng).unwrap();
//             map1.insert(key, *value);
//             print!("r{key}{value} ");

//             let symmetric_difference =
//                 SymDiffIterMap::new2(map0.range_values(), map1.range_values());

//             // println!(
//             //     "left ^ right = {}",
//             //     SymDiffIterMap::new2(map0.range_values(), map1.range_values()).into_string()
//             // );

//             let mut expected_keys = map0
//                 .ranges()
//                 .symmetric_difference(map1.ranges())
//                 .collect::<RangeSetBlaze<_>>();
//             for range_value in symmetric_difference {
//                 let (range, value) = range_value;
//                 // println!();
//                 // print!("removing ");
//                 for k in range {
//                     let get0 = map0.get(k);
//                     let get1 = map1.get(k);
//                     match (get0, get1) {
//                         (Some(_v0), Some(_v1)) => {
//                             println!();
//                             println!("left: {}", map0);
//                             println!("right: {}", map1);
//                             let s_d =
//                                 SymDiffIterMap::new2(map0.range_values(), map1.range_values())
//                                     .into_range_map_blaze();
//                             panic!("left ^ right = {s_d}");
//                         }
//                         (Some(v0), None) => {
//                             assert_eq!(v0, value);
//                         }
//                         (None, Some(v1)) => {
//                             assert_eq!(v1, value);
//                         }
//                         (None, None) => {
//                             panic!("should not happen 1");
//                         }
//                     }
//                     assert!(expected_keys.remove(k));
//                 }
//                 // println!();
//             }
//             if !expected_keys.is_empty() {
//                 println!();
//                 println!("left: {}", map0);
//                 println!("right: {}", map1);
//                 let s_d = SymDiffIterMap::new2(map0.range_values(), map1.range_values())
//                     .into_range_map_blaze();
//                 println!("left ^ right = {s_d}");
//                 panic!("expected_keys should be empty: {expected_keys}");
//             }
//         }
//     }
// }
// #[test]
// fn map_repro_insert_1() {
//     let mut range_map_blaze = RangeMapBlaze::new();
//     range_map_blaze.insert(123, "Hello");
//     range_map_blaze.insert(123, "World");
//     assert_eq!(range_map_blaze.to_string(), r#"(123..=123, "World")"#);
// }

// fn equal_maps<T: Integer, V: ValueOwned + fmt::Debug + std::fmt::Display>(
//     range_map_blaze: &RangeMapBlaze<T, V>,
//     btree_map: &BTreeMap<T, &V>,
// ) -> bool
// where
//     usize: std::convert::From<<T as Integer>::SafeLen>,
// {
//     // also, check that the ranges are really sorted and disjoint
//     // cmk range_values should return a tuple not a struct
//     // cmk implement iter for RangeMapBlaze
//     let mut previous: Option<(RangeInclusive<T>, &V)> = None;
//     for (range, value) in range_map_blaze.range_values() {
//         let v = range_value.1;
//         let range = range_value.0.clone();

//         if let Some(previous) = previous {
//             if (previous.1 == v && *previous.0.end().add_one() >= *range.start())
//                 || previous.0.end() >= range.start()
//             {
//                 eprintln!(
//                     "two ranges are not disjoint: {:?}->{} and {range:?}->{v}",
//                     previous.0, previous.1
//                 );
//                 return false;
//             }
//         }

//         debug_assert!(range.start() <= range.end());
//         let mut k = *range.start();
//         loop {
//             if btree_map.get(&k).map_or(true, |v2| v != *v2) {
//                 eprintln!(
//                     "range_map_blaze contains {k} -> {v}, btree_map contains {k} -> {:?}",
//                     btree_map.get(&k)
//                 );
//                 return false;
//             }
//             if k == *range.end() {
//                 break;
//             }
//             k = k.add_one();
//         }
//         previous = Some(range_value);
//     }

//     let len0: usize = range_map_blaze.len().into();
//     if len0 != btree_map.len() {
//         eprintln!(
//             "range_map_blaze.len() = {len0}, btree_map.len() = {}",
//             btree_map.len()
//         );
//         return false; // Different number of elements means they can't be the same map
//     }

//     true
// }

// fn format_range_values<T>(iter: impl Iterator<Item = (RangeInclusive<T>, &u8)>) -> String
// where
//     T: Integer + fmt::Display + 'a, // Assuming T implements Display for formatting
//                                     // V: ValueOwned + fmt::Display + 'a, // V must implement Display to be formatted with {}
// {
//     let mut vs = String::new();
//     for range_value in iter {
//         vs.push_str(&format!(
//             "{}..={}{} ",
//             range_value.0.start(),
//             range_value.0.end(),
//             *range_value.1 as char,
//         ));
//     }
//     vs
// }

// #[test]
// fn map_repro_106() {
//     let input_string = "100e 106b 97c 98c 97e";
//     let mut input = Vec::<(u8, &u8)>::new();
//     for pair in input_string.split_whitespace() {
//         let bytes = pair.as_bytes(); // Get the byte slice of the pair
//         let c = &bytes[bytes.len() - 1]; // Last byte as char
//         let num = pair[..pair.len() - 1].parse::<u8>().unwrap();
//         input.push((num, c)); // Add the (u8, &str) pair to inputs
//     }

//     let iter = input.clone().into_iter();
//     let iter = iter.map(|(x, value)| (x..=x, value));
//     let iter = iter.map(|(range, value)| (range, value));
//     let iter = UnsortedPriorityDisjointMap::new(iter.into_iter());
//     let iter = iter.into_iter().sorted_by(|a, b| {
//         // We sort only by start -- priority is not used until later.
//         a.start().cmp(&b.start())
//     });
//     let iter = AssumePrioritySortedStartsMap::new(iter);
//     let iter = UnionIterMap::new(iter);
//     let vs = format_range_values(iter);
//     println!("{vs}");
//     assert_eq!(vs, "97..=98c 100..=100e 106..=106b ");

//     let range_map_blaze = RangeMapBlaze::<u8, u8>::from_iter(input.clone());
//     assert_eq!(
//         range_map_blaze.to_string(),
//         "(97..=98, 99), (100..=100, 101), (106..=106, 98)"
//     );
// }

// #[test]
// fn map_repro_206() {
//     let input_string = "127e 2d 29e 84a 17a 79d 174e 125b 123a 123b 98c 132d 99e 186b 253d 31d 121c 151a 168e 208c 47e 42e 86a 21b 7b 238d 148a 151a 227d 173d 145b 18e 219e 16c 214b 213a 155e 27e 24d 38c 59c 16c 183d 125d 210d 99e 43e 189e 147a 90d 42a 220e 35b 120d 185d 177a 102a 22b 124b 140a 199e 143c 32d 225a 223e 137e 177e 234e 97a 166a 83e 213a 147b 128a 150c 12c 199c 152c 79b 164b 204b 235e 37e 14c 19b 49a 1c 115b 31d 102b 59b 129b 104d 70c 229b 205b 101c 58d 114a 228d 173e 139d 147b 32c 198e 194c 18a 77a 100e 196a 46b 81a 63d 198a 242a 131b 153e 113b 19d 253e 195c 209e 201c 139d 47a 223d 240b 203d 84a 214d 129e 73d 55d 193e 129d 7c 193e 2c 235c 39c 88d 175c 190c 239a 219d 121a 88d 175d 117e 23a 102d 165a 58a 229a 100b 13b 113e 26a 49e 37e 126a 251b 47e 77a 206b ";
//     let mut input = Vec::<(u8, &u8)>::new();
//     for pair in input_string.split_whitespace() {
//         let bytes = pair.as_bytes(); // Get the byte slice of the pair
//         let c = &bytes[bytes.len() - 1]; // Last byte as char
//         let num = pair[..pair.len() - 1].parse::<u8>().unwrap();
//         input.push((num, c)); // Add the (u8, &str) pair to inputs
//     }

//     let iter = input.clone().into_iter();
//     let iter = iter.map(|(x, value)| (x..=x, value));
//     let iter = iter.map(|(range, value)| (range, value));
//     // let vs = format_range_values(iter);
//     // println!("{vs}");
//     // assert_eq!(vs, "127..=127e 2..=2d 29..=29e 84..=84a 17..=17a 79..=79d 174..=174e 125..=125b 123..=123a 123..=123b 98..=98c 132..=132d 99..=99e 186..=186b 253..=253d 31..=31d 121..=121c 151..=151a 168..=168e 208..=208c 47..=47e 42..=42e 86..=86a 21..=21b 7..=7b 238..=238d 148..=148a 151..=151a 227..=227d 173..=173d 145..=145b 18..=18e 219..=219e 16..=16c 214..=214b 213..=213a 155..=155e 27..=27e 24..=24d 38..=38c 59..=59c 16..=16c 183..=183d 125..=125d 210..=210d 99..=99e 43..=43e 189..=189e 147..=147a 90..=90d 42..=42a 220..=220e 35..=35b 120..=120d 185..=185d 177..=177a 102..=102a 22..=22b 124..=124b 140..=140a 199..=199e 143..=143c 32..=32d 225..=225a 223..=223e 137..=137e 177..=177e 234..=234e 97..=97a 166..=166a 83..=83e 213..=213a 147..=147b 128..=128a 150..=150c 12..=12c 199..=199c 152..=152c 79..=79b 164..=164b 204..=204b 235..=235e 37..=37e 14..=14c 19..=19b 49..=49a 1..=1c 115..=115b 31..=31d 102..=102b 59..=59b 129..=129b 104..=104d 70..=70c 229..=229b 205..=205b 101..=101c 58..=58d 114..=114a 228..=228d 173..=173e 139..=139d 147..=147b 32..=32c 198..=198e 194..=194c 18..=18a 77..=77a 100..=100e 196..=196a 46..=46b 81..=81a 63..=63d 198..=198a 242..=242a 131..=131b 153..=153e 113..=113b 19..=19d 253..=253e 195..=195c 209..=209e 201..=201c 139..=139d 47..=47a 223..=223d 240..=240b 203..=203d 84..=84a 214..=214d 129..=129e 73..=73d 55..=55d 193..=193e 129..=129d 7..=7c 193..=193e 2..=2c 235..=235c 39..=39c 88..=88d 175..=175c 190..=190c 239..=239a 219..=219d 121..=121a 88..=88d 175..=175d 117..=117e 23..=23a 102..=102d 165..=165a 58..=58a 229..=229a 100..=100b 13..=13b 113..=113e 26..=26a 49..=49e 37..=37e 126..=126a 251..=251b 47..=47e 77..=77a 206..=206b ");

//     let iter = UnsortedPriorityDisjointMap::new(iter.into_iter());
//     let iter = iter.into_iter().sorted_by(|a, b| {
//         // We sort only by start -- priority is not used until later.
//         a.start().cmp(&b.start())
//     });
//     let iter = AssumePrioritySortedStartsMap::new(iter);
//     // let vs = format_range_values(iter);
//     // println!("{vs}");
//     // assert_eq!(vs, "1..=1c 2..=2c 2..=2d 7..=7c 7..=7b 12..=12c 13..=13b 14..=14c 16..=16c 16..=16c 17..=17a 18..=18a 18..=18e 19..=19d 19..=19b 21..=21b 22..=22b 23..=23a 24..=24d 26..=26a 27..=27e 29..=29e 31..=31d 31..=31d 32..=32c 32..=32d 35..=35b 37..=37e 37..=37e 38..=38c 39..=39c 42..=42a 42..=42e 43..=43e 46..=46b 47..=47e 47..=47a 47..=47e 49..=49e 49..=49a 55..=55d 58..=58a 58..=58d 59..=59b 59..=59c 63..=63d 70..=70c 73..=73d 77..=77a 77..=77a 79..=79b 79..=79d 81..=81a 83..=83e 84..=84a 84..=84a 86..=86a 88..=88d 88..=88d 90..=90d 97..=97a 98..=98c 99..=99e 99..=99e 100..=100b 100..=100e 101..=101c 102..=102d 102..=102b 102..=102a 104..=104d 113..=113e 113..=113b 114..=114a 115..=115b 117..=117e 120..=120d 121..=121a 121..=121c 123..=123b 123..=123a 124..=124b 125..=125d 125..=125b 126..=126a 127..=127e 128..=128a 129..=129d 129..=129e 129..=129b 131..=131b 132..=132d 137..=137e 139..=139d 139..=139d 140..=140a 143..=143c 145..=145b 147..=147b 147..=147b 147..=147a 148..=148a 150..=150c 151..=151a 151..=151a 152..=152c 153..=153e 155..=155e 164..=164b 165..=165a 166..=166a 168..=168e 173..=173e 173..=173d 174..=174e 175..=175d 175..=175c 177..=177e 177..=177a 183..=183d 185..=185d 186..=186b 189..=189e 190..=190c 193..=193e 193..=193e 194..=194c 195..=195c 196..=196a 198..=198a 198..=198e 199..=199c 199..=199e 201..=201c 203..=203d 204..=204b 205..=205b 206..=206b 208..=208c 209..=209e 210..=210d 213..=213a 213..=213a 214..=214d 214..=214b 219..=219d 219..=219e 220..=220e 223..=223d 223..=223e 225..=225a 227..=227d 228..=228d 229..=229a 229..=229b 234..=234e 235..=235c 235..=235e 238..=238d 239..=239a 240..=240b 242..=242a 251..=251b 253..=253e 253..=253d ");

//     let iter = UnionIterMap::new(iter);
//     let vs = format_range_values(iter);
//     println!("{vs}");
//     assert_eq!(vs, "1..=1c 2..=2d 7..=7b 12..=12c 13..=13b 14..=14c 16..=16c 17..=17a 18..=18e 19..=19b 21..=22b 23..=23a 24..=24d 26..=26a 27..=27e 29..=29e 31..=32d 35..=35b 37..=37e 38..=39c 42..=43e 46..=46b 47..=47e 49..=49a 55..=55d 58..=58d 59..=59c 63..=63d 70..=70c 73..=73d 77..=77a 79..=79d 81..=81a 83..=83e 84..=84a 86..=86a 88..=88d 90..=90d 97..=97a 98..=98c 99..=100e 101..=101c 102..=102a 104..=104d 113..=113b 114..=114a 115..=115b 117..=117e 120..=120d 121..=121c 123..=123a 124..=125b 126..=126a 127..=127e 128..=128a 129..=129b 131..=131b 132..=132d 137..=137e 139..=139d 140..=140a 143..=143c 145..=145b 147..=148a 150..=150c 151..=151a 152..=152c 153..=153e 155..=155e 164..=164b 165..=166a 168..=168e 173..=173d 174..=174e 175..=175c 177..=177a 183..=183d 185..=185d 186..=186b 189..=189e 190..=190c 193..=193e 194..=195c 196..=196a 198..=199e 201..=201c 203..=203d 204..=206b 208..=208c 209..=209e 210..=210d 213..=213a 214..=214b 219..=220e 223..=223e 225..=225a 227..=228d 229..=229b 234..=235e 238..=238d 239..=239a 240..=240b 242..=242a 251..=251b 253..=253d ");

//     // let range_map_blaze = RangeMapBlaze::<u8, u8>::from_iter(input.clone());
//     // assert_eq!(
//     //     range_map_blaze.to_string(),
//     //     "(97..=97, 101), (98..=98, 99), (100..=100, 101), (106..=106, 98)"
//     // );
// }

// #[test]
// fn map_repro_123() {
//     let input = [(123, 'a'), (123, 'b')];

//     let range_map_blaze = RangeMapBlaze::<u8, char>::from_iter(input);
//     assert_eq!(range_map_blaze.to_string(), "(123..=123, 'a')");
// }

// #[test]
// fn map_insert_255u8() {
//     let iter = [
//         (255u8..=255, "Hello".to_string()), // cmk to u8
//         (25..=25, "There".to_string()),
//     ]
//     .into_iter();
//     let range_map_blaze = RangeMapBlaze::<_, String>::from_iter(iter);
//     assert_eq!(
//         range_map_blaze.to_string(),
//         r#"(25..=25, "There"), (255..=255, "Hello")"#
//     );
// }

// // cmk
// #[test]
// fn map_insert_str() {
//     let s1 = "Hello".to_string();
//     let s2 = "There".to_string();
//     let range_map_blaze = RangeMapBlaze::<u8, String>::from_iter([(255, &s1), (25, &s2)]);
//     assert_eq!(
//         range_map_blaze.to_string(),
//         r#"(25..=25, "There"), (255..=255, "Hello")"#
//     );
// }

// #[test]
// fn map_repro_bit_or() {
//     let a = RangeSetBlaze::from_iter([1u8, 2, 3]);
//     let b = RangeSetBlaze::from_iter([2u8, 3, 4]);

//     let result = a.ranges().union(b.ranges());
//     let result = result.into_range_set_blaze();
//     println!("{result}");
//     assert_eq!(result, RangeSetBlaze::from_iter([1u8, 2, 3, 4]));

//     let result = a | b;
//     println!("{result}");
//     assert_eq!(result, RangeSetBlaze::from_iter([1u8, 2, 3, 4]));

//     let a = RangeMapBlaze::from_iter([(1u8, "Hello"), (2, "Hello"), (3, "Hello")]);
//     let b = RangeMapBlaze::from_iter([(2u8, "World"), (3, "World"), (4, "World")]);
//     let result = a
//         .range_values()
//         .union(b.range_values())
//         .into_range_map_blaze();
//     println!("{result}");
//     assert_eq!(
//         result,
//         RangeMapBlaze::from_iter([(1u8, "Hello"), (2u8, "Hello"), (3, "Hello"), (4, "World")])
//     );

//     let result = a | b;
//     println!("{result}");
//     assert_eq!(
//         result,
//         RangeMapBlaze::from_iter([(1u8, "Hello"), (2u8, "Hello"), (3, "Hello"), (4, "World")])
//     );
// }

// #[test]
// fn map_repro_bit_and() {
//     let a = RangeSetBlaze::from_iter([1u8, 2, 3]);
//     let b = RangeSetBlaze::from_iter([2u8, 3, 4]);

//     let result = a.ranges().intersection(b.ranges());
//     let result = result.into_range_set_blaze();
//     println!("{result}");
//     assert_eq!(result, RangeSetBlaze::from_iter([2, 3]));

//     let result = a & b;
//     println!("{result}");
//     assert_eq!(result, RangeSetBlaze::from_iter([2, 3]));

//     let a = RangeMapBlaze::from_iter([(1, "Hello"), (2, "World"), (3, "World")]);
//     let b = RangeMapBlaze::from_iter([(2, "Go"), (3, "Go"), (4, "Go")]);

//     let result = a
//         .range_values()
//         .intersection_with_set(b.ranges())
//         .into_range_map_blaze();
//     println!("{result}");
//     assert_eq!(result, RangeMapBlaze::from_iter([(2..=3, "World")]));

//     let result = a & b;
//     println!("{result}");
//     assert_eq!(result, RangeMapBlaze::from_iter([(2..=3, "World")]));
// }

// #[test]
// fn map_step_by_step() {
//     let (s1, s2) = ("a".to_string(), "b".to_string());
//     let input = [(1, &s2), (2, &s2), (0, &s1)];

//     let iter = input.into_iter();
//     let iter = iter.map(|(x, value)| (x..=x, value));
//     let iter = iter.map(|(range, value)| (range, value));

//     let iter = UnsortedPriorityDisjointMap::new(iter.into_iter());
//     let vs = format!("{:?}", iter.collect::<Vec<_>>());
//     println!("{vs}");
//     assert_eq!(
//         vs,
//         r#"[Priority { range_value: (1..=2, "b"), priority_number: 0 }, Priority { range_value: (0..=0, "a"), priority_number: 2 }]"#
//     );

//     let iter = input.into_iter();
//     let iter = iter.map(|(x, value)| (x..=x, value));
//     let iter = iter.map(|(range, value)| (range, value));
//     let iter = UnsortedPriorityDisjointMap::new(iter.into_iter());
//     let iter = iter.into_iter().sorted_by(|a, b| {
//         // We sort only by start -- priority is not used until later.
//         a.start().cmp(&b.start())
//     });
//     let iter = AssumePrioritySortedStartsMap::new(iter);
//     let vs = format!("{:?}", iter.collect::<Vec<_>>());
//     println!("{vs}");
//     assert_eq!(vs, "[Priority { range_value: (0..=0, \"a\"), priority_number: 2 }, Priority { range_value: (1..=2, \"b\"), priority_number: 0 }]");

//     let iter = input.into_iter();
//     let iter = iter.map(|(x, value)| (x..=x, value));
//     let iter = iter.map(|(range, value)| (range, value));
//     let iter = UnsortedPriorityDisjointMap::new(iter.into_iter());
//     let iter = iter.into_iter().sorted_by(|a, b| {
//         // We sort only by start -- priority is not used until later.
//         a.start().cmp(&b.start())
//     });
//     let iter = AssumePrioritySortedStartsMap::new(iter);
//     let iter = UnionIterMap::new(iter);
//     let vs = format!("{:?}", iter.collect::<Vec<_>>());
//     println!("{vs}");
//     assert_eq!(vs, "[(0..=0, \"a\"), (1..=2, \"b\")]");

//     let range_map_blaze = RangeMapBlaze::<u8, String>::from_iter(input);
//     println!("{range_map_blaze}");
//     assert_eq!(range_map_blaze.to_string(), r#"(0..=0, "a"), (1..=2, "b")"#);
// }

// #[test]
// fn map_repro1() {
//     let (s1, s2, s3) = ("a".to_string(), "b".to_string(), "c".to_string());
//     let mut range_map_blaze =
//         RangeMapBlaze::from_iter([(20..=21, &s1), (24..=24, &s2), (25..=29, &s2)]);
//     println!("{range_map_blaze}");
//     assert_eq!(
//         range_map_blaze.to_string(),
//         r#"(20..=21, "a"), (24..=29, "b")"#
//     );
//     range_map_blaze.internal_add(25..=25, &s3);
//     println!("{range_map_blaze}");
//     assert_eq!(
//         range_map_blaze.to_string(),
//         r#"(20..=21, "a"), (24..=24, "b"), (25..=25, "c"), (26..=29, "b")"#
//     );
// }

// #[test]
// fn map_repro2() {
//     let a = "a".to_string();
//     let b = "b".to_string();
//     let c = "c".to_string();
//     let mut range_map_blaze = RangeMapBlaze::<i8, _>::from_iter([
//         (-8, &a),
//         (8, &a),
//         (-2, &a),
//         (-1, &a),
//         (3, &a),
//         (2, &b),
//     ]);
//     range_map_blaze.ranges_insert(25..=25, c);
//     println!("{range_map_blaze}");
//     assert!(
//         range_map_blaze.to_string()
//             == r#"(-8..=-8, "a"), (-2..=-1, "a"), (2..=2, "b"), (3..=3, "a"), (8..=8, "a"), (25..=25, "c")"#
//     );
// }

// #[test]
// fn map_doctest1() {
//     let a = RangeMapBlaze::<u8, _>::from_iter([(1, "Hello"), (2, "World"), (3, "World")]);
//     let b = RangeMapBlaze::<u8, _>::from_iter([(3, "Go"), (4, "Go"), (5, "Go")]);

//     let result = &a | &b;
//     assert_eq!(
//         result,
//         RangeMapBlaze::<u8, _>::from_iter([
//             (1, "Hello"),
//             (2, "World"),
//             (3, "World"),
//             (4, "Go"),
//             (5, "Go")
//         ])
//     );
// }

// #[test]
// fn map_doctest2() {
//     let set = RangeMapBlaze::<u8, _>::from_iter([(1, "Hello"), (2, "World"), (3, "World")]);
//     assert_eq!(set.get(1), Some(&"Hello"));
//     assert_eq!(set.get(4), None);
// }

// #[test]
// fn map_doctest3() {
//     let mut a = RangeMapBlaze::from_iter([(1..=3, "Hello")]);
//     let mut b = RangeMapBlaze::from_iter([(3..=5, "World")]);

//     a.append(&mut b);

//     assert_eq!(a.len(), 5usize);
//     assert_eq!(b.len(), 0usize);

//     assert_eq!(a.get(1), Some(&"Hello"));
//     assert_eq!(a.get(2), Some(&"Hello"));
//     assert_eq!(a.get(3), Some(&"World"));
//     assert_eq!(a.get(4), Some(&"World"));
//     assert_eq!(a.get(5), Some(&"World"));
// }

// #[test]
// fn map_missing_doctest_ops() {
//     // note that may be borrowed or owned in any combination.

//     // Returns the union of `self` and `rhs` as a new [`RangeMapBlaze`].
//     let a = RangeMapBlaze::from_iter([(1, "Hello"), (2, "World"), (3, "World")]);
//     let b = RangeMapBlaze::from_iter([(3, "Go"), (4, "Go"), (5, "Go")]);

//     let result = &a | &b;
//     assert_eq!(
//         result,
//         RangeMapBlaze::from_iter([
//             (1, "Hello"),
//             (2, "World"),
//             (3, "World"),
//             (4, "Go"),
//             (5, "Go")
//         ])
//     );
//     let result = a | &b;
//     assert_eq!(
//         result,
//         RangeMapBlaze::from_iter([
//             (1, "Hello"),
//             (2, "World"),
//             (3, "World"),
//             (4, "Go"),
//             (5, "Go")
//         ])
//     );

//     // Returns the intersection of `self` and `rhs` as a new `RangeMapBlaze<T>`.

//     let a = RangeMapBlaze::from_iter([(1, "Hello"), (2, "World"), (3, "World")]);
//     let b = RangeMapBlaze::from_iter([(2, "Go"), (3, "Go"), (4, "Go")]);

//     let result = a & &b;
//     assert_eq!(
//         result,
//         RangeMapBlaze::from_iter([(2, "World"), (3, "World")])
//     );
//     let a = RangeMapBlaze::from_iter([(1, "Hello"), (2, "World"), (3, "World")]);
//     let result = a & b;
//     assert_eq!(
//         result,
//         RangeMapBlaze::from_iter([(2, "World"), (3, "World")])
//     );

//     // Returns the symmetric difference of `self` and `rhs` as a new `RangeMapBlaze<T>`.
//     let a = RangeMapBlaze::from_iter([(1, "Hello"), (2, "World"), (3, "World")]);
//     let b = RangeMapBlaze::from_iter([(3, "Go"), (4, "Go"), (5, "Go")]);

//     let result = a ^ b;
//     assert_eq!(
//         result,
//         RangeMapBlaze::from_iter([(1..=1, "Hello"), (2..=2, "World"), (4..=5, "Go")])
//     );

//     // Returns the set difference of `self` and `rhs` as a new `RangeMapBlaze<T>`.
//     let a = RangeMapBlaze::from_iter([(1, "Hello"), (2, "World"), (3, "World")]);
//     let b = RangeMapBlaze::from_iter([(2, "Go"), (3, "Go"), (4, "Go")]);

//     let result = a - b;
//     assert_eq!(result, RangeMapBlaze::from_iter([(1, "Hello")]));
// }

// // #[test]
// // fn multi_op() {
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
// //     let d = [a, b, c].iter().intersection();
// //     assert_eq!(d, RangeMapBlaze::new());

// //     assert_eq!(
// //         !MultiwayRangeSetBlazeRef::<u8>::union([]),
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
// //         MultiwayRangeSetBlazeRef::<u8>::intersection([]),
// //         RangeMapBlaze::from_iter([0..=255])
// //     );
// // }

// // // https://stackoverflow.com/questions/21747136/how-do-i-print-in-rust-the-type-of-a-variable/58119924#58119924
// // // fn print_type_of<T>(_: &T) {
// // //     println!("{}", std::any::type_name::<T>())
// // // }

// // #[test]
// // fn custom_multi() {
// //     let a = RangeMapBlaze::from_iter([1..=6, 8..=9, 11..=15]);
// //     let b = RangeMapBlaze::from_iter([5..=13, 18..=29]);
// //     let c = RangeMapBlaze::from_iter([38..=42]);

// //     let union_stream = b.ranges() | c.ranges();
// //     let a_less = a.ranges() - union_stream;
// //     let d: RangeMapBlaze<_> = a_less.into_range_set_blaze();
// //     println!("{d}");

// //     let d: RangeMapBlaze<_> =
// //         (a.ranges() - [b.ranges(), c.ranges()].union()).into_range_set_blaze();
// //     println!("{d}");
// // }

// // #[test]
// // fn from_string() {
// //     let a = RangeMapBlaze::from_iter([0..=4, 14..=17, 30..=255, 0..=37, 43..=65535]);
// //     assert_eq!(a, RangeMapBlaze::from_iter([0..=65535]));
// // }

// // #[test]
// // fn nand_repro() {
// //     let b = &RangeMapBlaze::from_iter([5u8..=13, 18..=29]);
// //     let c = &RangeMapBlaze::from_iter([38..=42]);
// //     println!("about to nand");
// //     let d = !b | !c;
// //     assert_eq!(
// //         d,
// //         RangeMapBlaze::from_iter([0..=4, 14..=17, 30..=255, 0..=37, 43..=255])
// //     );
// // }

// // #[test]
// // fn bit_or_iter() {
// //     let i = UnionIter::from([1, 3, 4, 2, 2, 43, -1, 4, 22]);
// //     let j = UnionIter::from([11, 3, 4, 42, 2, 43, 23, 2, 543]);

// //     let _not_i = !i.clone();
// //     let k = i - j;
// //     assert_eq!(k.into_string(), "-1..=-1, 1..=1, 22..=22");
// // }

// // #[test]
// // fn empty() {
// //     let universe: UnionIter<u8, _> = [0..=255].into_iter().collect();
// //     let arr: [u8; 0] = [];
// //     let a0 = RangeMapBlaze::<u8>::from_iter(arr);
// //     assert!(!(a0.ranges()).equal(universe.clone()));
// //     assert!((!a0).ranges().equal(universe));
// //     let _a0 = RangeMapBlaze::from_iter([0..=0; 0]);
// //     let _a = RangeMapBlaze::<i32>::new();

// //     let a_iter: std::array::IntoIter<i32, 0> = [].into_iter();
// //     let a = a_iter.collect::<RangeMapBlaze<i32, &str>>();
// //     let arr: [i32; 0] = [];
// //     let b = RangeMapBlaze::from_iter(arr);
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

// //     let answer = RangeMapBlaze::from_iter(arr);
// //     assert_eq!(&c0, &answer);
// //     assert_eq!(&c1a, &answer);
// //     assert_eq!(&c1b, &answer);
// //     assert_eq!(&c1c, &answer);
// //     assert_eq!(&c1d, &answer);
// //     assert_eq!(&c2, &answer);
// //     assert_eq!(&c3, &answer);
// //     assert_eq!(&c5, &answer);

// //     let a_iter: std::array::IntoIter<i32, 0> = [].into_iter();
// //     let a = a_iter.collect::<RangeMapBlaze<i32, &str>>();
// //     let b = RangeMapBlaze::from_iter([0i32; 0]);

// //     let c0 = a.ranges() | b.ranges();
// //     let c1 = [a.ranges(), b.ranges()].union();
// //     let c_list2: [RangesIter<i32>; 0] = [];
// //     let c2 = c_list2.clone().union();
// //     let c3 = union_dyn!(a.ranges(), b.ranges());
// //     let c4 = c_list2.map(DynSortedDisjoint::new).union();

// //     let answer = RangeMapBlaze::from_iter(arr);
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

// //     let answer = !RangeMapBlaze::from_iter([0i32; 0]);
// //     assert!(c0.equal(answer.ranges()));
// //     assert!(c1.equal(answer.ranges()));
// //     assert!(c2.equal(answer.ranges()));
// //     assert!(c3.equal(answer.ranges()));
// //     assert!(c4.equal(answer.ranges()));
// // }

// // // Can't implement fmt::Display fmt must take ownership
// // impl<T, I> UnsortedDisjoint<T, I>
// // where
// //     T: Integer,
// //     I: Iterator<Item = RangeInclusive<T>>,
// // {
// //     #[allow(clippy::inherent_to_string)]
// //     #[allow(clippy::wrong_self_convention)]
// //     pub(crate) fn to_string(self) -> String {
// //         self.map(|range| format!("{range:?}")).join(", ")
// //     }
// // }
// // #[allow(clippy::reversed_empty_ranges)]
// // #[test]
// // fn private_constructor() {
// //     let unsorted_disjoint = UnsortedDisjoint::from([5..=6, 1..=5, 1..=0, -12..=-10, 3..=3]);
// //     // println!("{}", unsorted_disjoint.fmt());
// //     assert_eq!(unsorted_disjoint.into_string(), "1..=6, -12..=-10, 3..=3");

// //     let unsorted_disjoint = UnsortedDisjoint::from([5..=6, 1..=5, 1..=0, -12..=-10, 3..=3]);
// //     let union_iter = UnionIter::from(unsorted_disjoint);
// //     // println!("{}", union_iter.fmt());
// //     assert_eq!(union_iter.into_string(), "-12..=-10, 1..=6");

// //     let union_iter: UnionIter<_, _> = [5, 6, 1, 2, 3, 4, 5, -12, -11, -10, 3]
// //         .into_iter()
// //         .collect();
// //     assert_eq!(union_iter.into_string(), "-12..=-10, 1..=6");
// // }

// // fn is_ddcppdheo<
// //     T: std::fmt::Debug
// //         + Display
// //         + Clone
// //         + PartialEq
// //         + PartialOrd
// //         + Default
// //         + std::hash::Hash
// //         + Eq
// //         + Ord
// //         + Send
// //         + Sync,
// // >() {
// // }

// // fn is_sssu<T: Sized + Send + Sync + Unpin>() {}
// // fn is_like_btreeset_iter<T: Clone + std::fmt::Debug + FusedIterator + Iterator>() {}
// // // cmk removed DoubleEndedIterator +ExactSizeIterator for now
// // #[test]
// // fn iter_traits() {
// //     type ARangesIter<'a> = RangesIter<'a, i32>;
// //     type AIter<'a> = Iter<i32, ARangesIter<'a>>;
// //     is_sssu::<AIter>();
// //     is_like_btreeset_iter::<AIter>();
// // }

// // fn is_like_btreeset_into_iter<T: std::fmt::Debug + FusedIterator + Iterator>() {}

// // fn is_like_btreeset<
// //     T: Clone
// //         + std::fmt::Debug
// //         + Default
// //         + Eq
// //         + std::hash::Hash
// //         + IntoIterator
// //         + Ord
// //         + PartialEq
// //         + PartialOrd
// //         + RefUnwindSafe
// //         + Send
// //         + Sync
// //         + Unpin
// //         + UnwindSafe
// //         + Any
// //         + ToOwned,
// // >() {
// // }

// // fn is_like_check_sorted_disjoint<
// //     T: Clone
// //         + std::fmt::Debug
// //         + Default
// //         + IntoIterator
// //         + RefUnwindSafe
// //         + Send
// //         + Sync
// //         + Unpin
// //         + UnwindSafe
// //         + Any
// //         + ToOwned,
// // >() {
// // }

// // fn is_like_dyn_sorted_disjoint<T: IntoIterator + Unpin + Any>() {}

// // #[test]
// // fn check_traits() {
// //     // Debug/Display/Clone/PartialEq/PartialOrd/Default/Hash/Eq/Ord/Send/Sync
// //     type ARangeSetBlaze = RangeMapBlaze<i32, &str>;
// //     is_sssu::<ARangeSetBlaze>();
// //     is_ddcppdheo::<ARangeSetBlaze>();
// //     is_like_btreeset::<ARangeSetBlaze>();

// //     type ARangesIter<'a> = RangesIter<'a, i32>;
// //     is_sssu::<ARangesIter>();
// //     is_like_btreeset_iter::<ARangesIter>();

// //     type AIter<'a> = Iter<i32, ARangesIter<'a>>;
// //     is_sssu::<AIter>();
// //     is_like_btreeset_iter::<AIter>();

// //     is_sssu::<IntoIter<i32>>();
// //     is_like_btreeset_into_iter::<IntoIter<i32>>();

// //     type AMerge<'a> = Merge<i32, ARangesIter<'a>, ARangesIter<'a>>;
// //     is_sssu::<AMerge>();
// //     is_like_btreeset_iter::<AMerge>();

// //     let a = RangeMapBlaze::from_iter([1..=2, 3..=4]);
// //     println!("{:?}", a.ranges());

// //     type AKMerge<'a> = KMerge<i32, ARangesIter<'a>>;
// //     is_sssu::<AKMerge>();
// //     is_like_btreeset_iter::<AKMerge>();

// //     type ANotIter<'a> = NotIter<i32, ARangesIter<'a>>;
// //     is_sssu::<ANotIter>();
// //     is_like_btreeset_iter::<ANotIter>();

// //     type AIntoRangesIter = IntoRangesIter<i32>;
// //     is_sssu::<AIntoRangesIter>();
// //     is_like_btreeset_into_iter::<AIntoRangesIter>();

// //     type ACheckSortedDisjoint<'a> = CheckSortedDisjoint<i32, ARangesIter<'a>>;
// //     is_sssu::<ACheckSortedDisjoint>();
// //     type BCheckSortedDisjoint =
// //         CheckSortedDisjoint<i32, std::array::IntoIter<RangeInclusive<i32>, 0>>;
// //     is_like_check_sorted_disjoint::<BCheckSortedDisjoint>();

// //     type ADynSortedDisjoint<'a> = DynSortedDisjoint<'a, i32>;
// //     is_like_dyn_sorted_disjoint::<ADynSortedDisjoint>();

// //     type AUnionIter<'a> = UnionIter<i32, ARangesIter<'a>>;
// //     is_sssu::<AUnionIter>();
// //     is_like_btreeset_iter::<AUnionIter>();

// //     type AAssumeSortedStarts<'a> = AssumeSortedStarts<i32, ARangesIter<'a>>;
// //     is_sssu::<AAssumeSortedStarts>();
// //     is_like_btreeset_iter::<AAssumeSortedStarts>();
// // }

// // #[test]
// // fn integer_coverage() {
// //     syntactic_for! { ty in [i8, u8, isize, usize,  i16, u16, i32, u32, i64, u64, isize, usize, i128, u128] {
// //         $(
// //             let len = <$ty as Integer>::SafeLen::one();
// //             let a = $ty::zero();
// //             assert_eq!($ty::safe_len_to_f64(len), 1.0);
// //             assert_eq!($ty::add_len_less_one(a,len), a);
// //             assert_eq!($ty::sub_len_less_one(a,len), a);
// //             assert_eq!($ty::f64_to_safe_len(1.0), len);

// //         )*
// //     }};
// // }

// // #[test]
// // #[allow(clippy::bool_assert_comparison)]
// // fn lib_coverage_0() {
// //     let a = RangeMapBlaze::from_iter([1..=2, 3..=4]);
// //     let mut hasher = DefaultHasher::new();
// //     a.hash(&mut hasher);
// //     let _d = RangeMapBlaze::<i32>::default();
// //     assert_eq!(a, a);

// //     let mut set = RangeMapBlaze::new();
// //     assert_eq!(set.first(), None);
// //     set.insert(1);
// //     assert_eq!(set.first(), Some(1));
// //     set.insert(2);
// //     assert_eq!(set.first(), Some(1));

// //     let set = RangeMapBlaze::from_iter([(1, "Hello"), (2, "World"), (3, "World")]);
// //     assert_eq!(set.get(2), Some(2));
// //     assert_eq!(set.get(4), None);

// //     let mut set = RangeMapBlaze::new();
// //     assert_eq!(set.last(), None);
// //     set.insert(1);
// //     assert_eq!(set.last(), Some(1));
// //     set.insert(2);
// //     assert_eq!(set.last(), Some(2));

// //     assert_eq!(a.len(), a.len_slow());

// //     let mut a = RangeMapBlaze::from_iter([1..=3]);
// //     let mut b = RangeMapBlaze::from_iter([3..=5]);

// //     a.append(&mut b);

// //     assert_eq!(a.len(), 5 as I32SafeLen);
// //     assert_eq!(b.len(), 0 as I32SafeLen);

// //     assert!(a.contains(1));
// //     assert!(a.contains(2));
// //     assert!(a.contains(3));
// //     assert!(a.contains(4));
// //     assert!(a.contains(5));

// //     let mut v = RangeMapBlaze::new();
// //     v.insert(1);
// //     v.clear();
// //     assert!(v.is_empty());

// //     let mut v = RangeMapBlaze::new();
// //     assert!(v.is_empty());
// //     v.insert(1);
// //     assert!(!v.is_empty());

// //     let sup = RangeMapBlaze::from_iter([1..=3]);
// //     let mut set = RangeMapBlaze::new();

// //     assert_eq!(set.is_subset(&sup), true);
// //     set.insert(2);
// //     assert_eq!(set.is_subset(&sup), true);
// //     set.insert(4);
// //     assert_eq!(set.is_subset(&sup), false);

// //     let sub = RangeMapBlaze::from_iter([1, 2]);
// //     let mut set = RangeMapBlaze::new();

// //     assert_eq!(set.is_superset(&sub), false);

// //     set.insert(0);
// //     set.insert(1);
// //     assert_eq!(set.is_superset(&sub), false);

// //     set.insert(2);
// //     assert_eq!(set.is_superset(&sub), true);

// //     let a = RangeMapBlaze::from_iter([1..=3]);
// //     let mut b = RangeMapBlaze::new();

// //     assert_eq!(a.is_disjoint(&b), true);
// //     b.insert(4);
// //     assert_eq!(a.is_disjoint(&b), true);
// //     b.insert(1);
// //     assert_eq!(a.is_disjoint(&b), false);

// //     let mut set = RangeMapBlaze::new();
// //     set.insert(3);
// //     set.insert(5);
// //     set.insert(8);
// //     assert_eq!(Some(5), set.0(4..).next());
// //     assert_eq!(Some(3), set.0(..).next());
// //     assert_eq!(None, set.0(..=2).next());
// //     assert_eq!(None, set.0(1..2).next());
// //     assert_eq!(
// //         Some(3),
// //         set.0((Bound::Excluded(2), Bound::Excluded(4))).next()
// //     );

// //     let mut set = RangeMapBlaze::new();

// //     assert_eq!(set.ranges_insert(2..=5), true);
// //     assert_eq!(set.ranges_insert(5..=6), true);
// //     assert_eq!(set.ranges_insert(3..=4), false);
// //     assert_eq!(set.len(), 5 as I32SafeLen);
// //     let mut set = RangeMapBlaze::from_iter([(1, "Hello"), (2, "World"), (3, "World")]);
// //     assert_eq!(set.take(2), Some(2));
// //     assert_eq!(set.take(2), None);

// //     let mut set = RangeMapBlaze::new();
// //     assert!(set.replace(5).is_none());
// //     assert!(set.replace(5).is_some());

// //     let mut a = RangeMapBlaze::from_iter([1..=3]);
// //     #[allow(clippy::reversed_empty_ranges)]
// //     a.internal_add(2..=1);

// //     assert_eq!(a.partial_cmp(&a), Some(Ordering::Equal));

// //     let mut a = RangeMapBlaze::from_iter([1..=3]);
// //     a.extend(std::iter::once(4));
// //     assert_eq!(a.len(), 4 as I32SafeLen);

// //     let mut a = RangeMapBlaze::from_iter([1..=3]);
// //     a.extend(4..=5);
// //     assert_eq!(a.len(), 5 as I32SafeLen);

// //     let mut set = RangeMapBlaze::new();

// //     set.insert(1);
// //     while let Some(n) = set.pop_first() {
// //         assert_eq!(n, 1);
// //     }
// //     assert!(set.is_empty());

// //     let mut set = RangeMapBlaze::new();

// //     set.insert(1);
// //     while let Some(n) = set.pop_last() {
// //         assert_eq!(n, 1);
// //     }
// //     assert!(set.is_empty());

// //     let a = RangeMapBlaze::from_iter([1..=3]);
// //     let i = a.iter();
// //     let j = i.clone();
// //     assert_eq!(i.size_hint(), j.size_hint());
// //     assert_eq!(format!("{:?}", &i), format!("{:?}", &j));

// //     let a = RangeMapBlaze::from_iter([1..=3]);
// //     let i = a.into_iter();
// //     assert_eq!(i.size_hint(), j.size_hint());
// //     assert_eq!(
// //         format!("{:?}", &i),
// //         "IntoIter { option_range_front: None, option_range_back: None, into_iter: [(1, 3)] }"
// //     );

// //     let mut a = RangeMapBlaze::from_iter([1..=3]);
// //     a.extend([1..=3]);
// //     assert_eq!(a.len(), 3 as I32SafeLen);

// //     let a = RangeMapBlaze::from_iter([1..=3]);
// //     let b = <RangeMapBlaze<i32, &str> as Clone>::clone(&a);
// //     assert_eq!(a, b);
// //     let c = <RangeMapBlaze<i32, &str> as Default>::default();
// //     assert_eq!(c, RangeMapBlaze::new());

// //     syntactic_for! { ty in [i8, u8, isize, usize,  i16, u16, i32, u32, i64, u64, isize, usize, i128, u128] {
// //         $(
// //             let a = RangeMapBlaze::<$ty>::new();
// //             println!("{a:#?}");
// //             assert_eq!(a.iter().next(), None);

// //             let mut a = RangeMapBlaze::from_iter([$ty::one()..=3]);
// //             let mut b = RangeMapBlaze::from_iter([3..=5]);

// //             a.append(&mut b);

// //             // assert_eq!(a.len(), 5usize);
// //             assert_eq!(b.len(), <$ty as Integer>::SafeLen::zero());

// //             assert!(a.contains(1));
// //             assert!(a.contains(2));
// //             assert!(a.contains(3));
// //             assert!(a.contains(4));
// //             assert!(a.contains(5));

// //             assert!(b.is_empty());

// //             let a = RangeMapBlaze::from_iter([$ty::one()..=3]);
// //             let b = RangeMapBlaze::from_iter([3..=5]);
// //             assert!(!a.is_subset(&b));
// //             assert!(!a.is_superset(&b));

// //         )*
// //     }};

// //     let a = RangeMapBlaze::from_iter([1u128..=3]);
// //     assert!(a.contains(1));
// //     assert!(!a.is_disjoint(&a));
// // }

// // #[test]
// // #[should_panic]
// // fn lib_coverage_2() {
// //     let v = RangeMapBlaze::<u128>::new();
// //     v.contains(u128::MAX);
// // }

// // #[test]
// // #[should_panic]
// // fn lib_coverage_3() {
// //     let mut v = RangeMapBlaze::<u128>::new();
// //     v.remove(u128::MAX);
// // }

// // #[test]
// // #[should_panic]
// // fn lib_coverage_4() {
// //     let mut v = RangeMapBlaze::<u128>::new();
// //     v.split_off(u128::MAX);
// // }

// // #[test]
// // #[should_panic]
// // fn lib_coverage_5() {
// //     let mut v = RangeMapBlaze::<u128>::new();
// //     v.internal_add(0..=u128::MAX);
// // }

// // #[test]
// // fn lib_coverage_6() {
// //     syntactic_for! { ty in [i8, u8, isize, usize,  i16, u16, i32, u32, i64, u64, isize, usize, i128, u128] {
// //         $(
// //             let mut a = RangeMapBlaze::<$ty>::from_iter([1..=3, 5..=7, 9..=120]);
// //             a.ranges_insert(2..=100);
// //             assert_eq!(a, RangeMapBlaze::from_iter([1..=120]));

// //         )*
// //     }};
// // }

// // #[test]
// // fn merge_coverage_0() {
// //     let a = CheckSortedDisjoint::new([1..=2, 5..=100]);
// //     let b = CheckSortedDisjoint::new([2..=6]);
// //     let m = Merge::new(a, b);
// //     let n = m.clone();
// //     let p = n.clone();
// //     let union1 = UnionIter::new(m);
// //     let union2 = UnionIter::new(n);
// //     assert!(union1.equal(union2));
// //     assert!(format!("{p:?}").starts_with("Merge"));

// //     let a = CheckSortedDisjoint::new([1..=2, 5..=100]);
// //     let b = CheckSortedDisjoint::new([2..=6]);
// //     let c = CheckSortedDisjoint::new([-1..=-1]);
// //     let m = KMerge::new([a, b, c]);
// //     let n = m.clone();
// //     let p = n.clone();
// //     let union1 = UnionIter::new(m);
// //     let union2 = UnionIter::new(n);
// //     assert!(union1.equal(union2));
// //     assert!(format!("{p:?}").starts_with("KMerge"));
// // }

// // #[test]
// // fn not_iter_coverage_0() {
// //     let a = CheckSortedDisjoint::new([1..=2, 5..=100]);
// //     let n = NotIter::new(a);
// //     let p = n.clone();
// //     let m = p.clone();
// //     assert!(n.equal(m));
// //     assert!(format!("{p:?}").starts_with("NotIter"));
// // }

// // #[test]
// // fn ranges_coverage_0() {
// //     let a = RangeMapBlaze::from_iter([1..=2, 5..=100]);
// //     let r = a.ranges();
// //     let p = r.as_ref();
// //     assert!(format!("{p:?}").starts_with("Ranges"));
// //     assert_eq!(r.len(), 2);

// //     let r2 = a.into_ranges();
// //     let n2 = !!r2;
// //     let a = RangeMapBlaze::from_iter([1..=2, 5..=100]);
// //     assert!(n2.equal(a.ranges()));
// //     let a = RangeMapBlaze::from_iter([1..=2, 5..=100]);
// //     let b = a.into_ranges();
// //     let a = RangeMapBlaze::from_iter([1..=2, 5..=100]);
// //     let c = a.into_ranges();
// //     let a = RangeMapBlaze::from_iter([1..=2, 5..=100]);
// //     assert!((b | c).equal(a.ranges()));

// //     let a = RangeMapBlaze::from_iter([1..=2, 5..=100]).into_ranges();
// //     let b = RangeMapBlaze::from_iter([1..=2, 5..=100]).into_ranges();
// //     assert!((a - b).is_empty());

// //     let a = RangeMapBlaze::from_iter([1..=2, 5..=100]).into_ranges();
// //     let b = RangeMapBlaze::from_iter([1..=2, 5..=100]).into_ranges();
// //     assert!((a ^ b).is_empty());

// //     let a = RangeMapBlaze::from_iter([1..=2, 5..=100]).into_ranges();
// //     let b = RangeMapBlaze::from_iter([1..=2, 5..=100]).into_ranges();
// //     assert!((a & b).equal(RangeMapBlaze::from_iter([1..=2, 5..=100]).into_ranges()));

// //     assert_eq!(
// //         RangeMapBlaze::from_iter([1..=2, 5..=100])
// //             .into_ranges()
// //             .len(),
// //         2
// //     );
// //     assert!(format!(
// //         "{:?}",
// //         RangeMapBlaze::from_iter([1..=2, 5..=100]).into_ranges()
// //     )
// //     .starts_with("IntoRanges"));
// // }

// // #[test]
// // fn sorted_disjoint_coverage_0() {
// //     let a = CheckSortedDisjoint::<i32, _>::default();
// //     assert!(a.is_empty());

// //     let a = CheckSortedDisjoint::new([1..=2, 5..=100]);
// //     let b = CheckSortedDisjoint::new([1..=2, 5..=100]);
// //     assert!((a & b).equal(CheckSortedDisjoint::new([1..=2, 5..=100])));

// //     let a = CheckSortedDisjoint::new([1..=2, 5..=100]);
// //     let b = CheckSortedDisjoint::new([1..=2, 5..=100]);
// //     assert!((a - b).is_empty());

// //     let a = CheckSortedDisjoint::new([1..=2, 5..=100]);
// //     let b = CheckSortedDisjoint::new([1..=2, 5..=100]);
// //     assert!((a ^ b).is_empty());
// // }

// // #[test]
// // #[should_panic]
// // fn sorted_disjoint_coverage_1() {
// //     struct SomeAfterNone {
// //         a: i32,
// //     }
// //     impl Iterator for SomeAfterNone {
// //         type Item = RangeInclusive<i32>;
// //         fn next(&mut self) -> Option<Self::Item> {
// //             self.a += 1;
// //             if self.a % 2 == 0 {
// //                 Some(self.a..=self.a)
// //             } else {
// //                 None
// //             }
// //         }
// //     }

// //     let mut a = CheckSortedDisjoint::new(SomeAfterNone { a: 0 });
// //     a.next();
// //     a.next();
// //     a.next();
// // }

// // #[test]
// // #[should_panic]
// // fn sorted_disjoint_coverage_2() {
// //     #[allow(clippy::reversed_empty_ranges)]
// //     let mut a = CheckSortedDisjoint::new([1..=0]);
// //     a.next();
// // }

// // #[test]
// // #[should_panic]
// // fn sorted_disjoint_coverage_3() {
// //     #[allow(clippy::reversed_empty_ranges)]
// //     let mut a = CheckSortedDisjoint::new([1..=1, 2..=2]);
// //     a.next();
// //     a.next();
// // }

// // #[test]
// // fn sorted_disjoint_coverage_4() {
// //     #[allow(clippy::reversed_empty_ranges)]
// //     let mut a = CheckSortedDisjoint::new([0..=i128::MAX]);
// //     a.next();
// // }

// // #[test]
// // fn sorted_disjoint_iterator_coverage_0() {
// //     let a = CheckSortedDisjoint::new([1..=2, 5..=100]);
// //     let b = CheckSortedDisjoint::new([1..=2, 5..=101]);
// //     assert!(b.is_superset(a));
// // }

// // #[test]
// // fn union_iter_coverage_0() {
// //     let a = CheckSortedDisjoint::new([1..=2, 5..=100]);
// //     let b = CheckSortedDisjoint::new([1..=2, 5..=101]);
// //     let c = a.union(b);
// //     assert!(format!("{c:?}").starts_with("UnionIter"));
// // }

// // #[test]
// // fn unsorted_disjoint_coverage_0() {
// //     let a = AssumeSortedStarts::new([1..=2, 5..=100].into_iter());
// //     assert!(format!("{a:?}").starts_with("AssumeSortedStarts"));
// // }

// // #[test]
// // fn test_coverage_0() {
// //     let a = BooleanVector(vec![true, true, false, false]);
// //     assert!(format!("{a:?}").starts_with("BooleanVector"));

// //     let a = How::Union;
// //     #[allow(clippy::clone_on_copy)]
// //     let _b = a.clone();

// //     let mut rng = StdRng::seed_from_u64(0);
// //     let a = MemorylessRange::new(&mut rng, 1000, 0..=10, 0.5, 1, How::Intersection);
// //     let v: Vec<_> = a.take(100).collect();
// //     println!("{v:?}");

// //     let a = MemorylessIter::new(&mut rng, 1000, 0..=10, 0.5, 1, How::Intersection);
// //     let v: Vec<_> = a.take(100).collect();
// //     println!("{v:?}");
// // }

#[quickcheck]
fn extend(mut a: BTreeMap<i8, u8>, b: Vec<(i8, u8)>) -> bool {
    let mut a_r = RangeMapBlaze::from_iter(a.clone().into_iter());
    a.extend(b.to_owned().into_iter());
    a_r.extend(b.into_iter());
    a_r == RangeMapBlaze::from_iter(a.into_iter())
}

// // #[should_panic]
// // #[test]
// // fn demo_read() {
// //     let _a: RangeMapBlaze<i32, &str> = demo_read_ranges_from_file("tests/no_such_file").unwrap();
// // }

// // #[test]
// // fn double_end_iter() {
// //     let a = RangeMapBlaze::from_iter([3..=10, 12..=12, 20..=25]);

// //     assert_eq!(
// //         a.iter().rev().collect::<Vec<usize>>(),
// //         vec![25, 24, 23, 22, 21, 20, 12, 10, 9, 8, 7, 6, 5, 4, 3]
// //     );

// //     {
// //         let mut iter = a.iter();

// //         assert_eq!(iter.next(), Some(3));
// //         assert_eq!(iter.next_back(), Some(25));
// //         assert_eq!(iter.next(), Some(4));
// //         assert_eq!(iter.next_back(), Some(24));
// //         assert_eq!(iter.next_back(), Some(23));
// //         assert_eq!(iter.next_back(), Some(22));
// //         assert_eq!(iter.next_back(), Some(21));
// //         assert_eq!(iter.next_back(), Some(20));

// //         // Next interval
// //         assert_eq!(iter.next_back(), Some(12));

// //         // Next interval, now same interval as front of the iterator
// //         assert_eq!(iter.next_back(), Some(10));
// //         assert_eq!(iter.next(), Some(5));
// //     }
// // }
// // #[test]
// // fn double_end_into_iter() {
// //     let a = RangeMapBlaze::from_iter([3..=10, 12..=12, 20..=25]);

// //     assert_eq!(
// //         a.clone().into_iter().rev().collect::<Vec<usize>>(),
// //         vec![25, 24, 23, 22, 21, 20, 12, 10, 9, 8, 7, 6, 5, 4, 3]
// //     );

// //     let mut iter = a.into_iter();

// //     assert_eq!(iter.next(), Some(3));
// //     assert_eq!(iter.next_back(), Some(25));
// //     assert_eq!(iter.next(), Some(4));
// //     assert_eq!(iter.next_back(), Some(24));
// //     assert_eq!(iter.next_back(), Some(23));
// //     assert_eq!(iter.next_back(), Some(22));
// //     assert_eq!(iter.next_back(), Some(21));
// //     assert_eq!(iter.next_back(), Some(20));

// //     // Next interval
// //     assert_eq!(iter.next_back(), Some(12));

// //     // Next interval, now same interval as front of the iterator
// //     assert_eq!(iter.next_back(), Some(10));
// //     assert_eq!(iter.next(), Some(5));
// // }
// // #[test]
// // fn double_end_range() {
// //     let a = RangeMapBlaze::from_iter([3..=10, 12..=12, 20..=25]);

// //     let mut range = a.0(11..=22);
// //     assert_eq!(range.next_back(), Some(22));
// //     assert_eq!(range.next(), Some(12));
// //     assert_eq!(range.next(), Some(20));
// // }

#[test]
fn test_coverage_8() {
    let mut a = RangeMapBlaze::from_iter([(1u128..=2, "Hello"), (3..=4, "World")]);
    a.internal_add(0..=u128::MAX, "Hello");
}

#[test]
fn test_coverage_9() {
    let mut a = RangeMapBlaze::from_iter([(1u128..=2, "Hello"), (3..=4, "World")]);
    let b = a.clone();
    a.internal_add(1..=0, "Hello"); // adding empty
    assert_eq!(a, b);
}
