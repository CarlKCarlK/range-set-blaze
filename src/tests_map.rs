#![cfg(test)]

use crate::{
    CheckSortedDisjointMap, DynSortedDisjointMap, FillGapsIter, FillGapsIterMap, Integer,
    IntersectionIterMap, IntoIterMap, IntoRangeValuesIter, IterMap, KMergeMap, MergeMap,
    RangeMapBlaze, RangeValuesIter, RangesIter, SymDiffIterMap, UnionIterMap,
    keys::{IntoKeys, Keys},
    sorted_disjoint_map::{Priority, RangeToRangeValueIter},
    unsorted_priority_map::{AssumePrioritySortedStartsMap, UnsortedPriorityMap},
    values::{IntoValues, Values},
};
use alloc::{
    borrow::ToOwned,
    string::{String, ToString},
    vec,
    vec::Vec,
};
use core::{
    any::Any,
    array,
    fmt::{self, Write as FmtWrite}, // Renamed to avoid conflict with std::fmt::Write
    hash::Hash,
    iter::FusedIterator,
    ops::RangeInclusive,
    panic::{RefUnwindSafe, UnwindSafe},
    prelude::v1::*,
};
use itertools::Itertools;

use wasm_bindgen_test::*;
wasm_bindgen_test_configure!(run_in_browser);

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
fn map_step_by_step() {
    use alloc::format;

    let (s1, s2) = ("a".to_string(), "b".to_string());
    let input = [(1, &s2), (2, &s2), (0, &s1)];

    let iter = input.into_iter();
    let iter = iter.map(|(x, value)| (x..=x, value));
    let iter = UnsortedPriorityMap::new(iter);
    let vs = format!("{:?}", iter.collect::<Vec<_>>());
    // println!("{vs}");
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
    // println!("{vs}");
    assert_eq!(
        vs,
        "[Priority { range_value: (0..=0, \"a\"), priority_number: 2 }, Priority { range_value: (1..=2, \"b\"), priority_number: 0 }]"
    );

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
    // println!("{vs}");
    assert_eq!(vs, "[(0..=0, \"a\"), (1..=2, \"b\")]");

    let range_map_blaze = RangeMapBlaze::<u8, String>::from_iter(input);
    // println!("{range_map_blaze}");
    assert_eq!(range_map_blaze.to_string(), r#"(0..=0, "a"), (1..=2, "b")"#);
}

fn format_range_values<'a, T>(iter: impl Iterator<Item = (RangeInclusive<T>, &'a u8)>) -> String
where
    T: Integer + fmt::Display + 'a, // Assuming T implements Display for formatting
                                    // V: ValueOwned + fmt::Display + 'a, // V must implement Display to be formatted with {}
{
    use alloc::string::String;

    let mut vs = String::new();
    for (range, value) in iter {
        write!(vs, "{:?}{} ", range, *value as char).expect("Failed to write to string");
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

    let iter = UnsortedPriorityMap::new(iter);
    let iter = iter.sorted_by(|a, b| {
        // We sort only by start -- priority is not used until later.
        a.start().cmp(&b.start())
    });
    let iter = AssumePrioritySortedStartsMap::new(iter);

    let iter = UnionIterMap::new(iter);
    let vs = format_range_values(iter);
    // println!("{vs}");
    assert_eq!(
        vs,
        "1..=2c 7..=7c 12..=12c 13..=13b 14..=14c 16..=16c 17..=18a 19..=19d 21..=22b 23..=23a 24..=24d 26..=26a 27..=27e 29..=29e 31..=31d 32..=32c 35..=35b 37..=37e 38..=39c 42..=42a 43..=43e 46..=46b 47..=47e 49..=49e 55..=55d 58..=58a 59..=59b 63..=63d 70..=70c 73..=73d 77..=77a 79..=79b 81..=81a 83..=83e 84..=84a 86..=86a 88..=88d 90..=90d 97..=97a 98..=98c 99..=99e 100..=100b 101..=101c 102..=102d 104..=104d 113..=113e 114..=114a 115..=115b 117..=117e 120..=120d 121..=121a 123..=124b 125..=125d 126..=126a 127..=127e 128..=128a 129..=129d 131..=131b 132..=132d 137..=137e 139..=139d 140..=140a 143..=143c 145..=145b 147..=147b 148..=148a 150..=150c 151..=151a 152..=152c 153..=153e 155..=155e 164..=164b 165..=166a 168..=168e 173..=174e 175..=175d 177..=177e 183..=183d 185..=185d 186..=186b 189..=189e 190..=190c 193..=193e 194..=195c 196..=196a 198..=198a 199..=199c 201..=201c 203..=203d 204..=206b 208..=208c 209..=209e 210..=210d 213..=213a 214..=214d 219..=219d 220..=220e 223..=223d 225..=225a 227..=228d 229..=229a 234..=234e 235..=235c 238..=238d 239..=239a 240..=240b 242..=242a 251..=251b 253..=253e "
    );

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
    // println!("{vs}");
    assert_eq!(vs, "97..=97e 98..=98c 100..=100e 106..=106b ");

    let range_map_blaze = RangeMapBlaze::<u8, u8>::from_iter(input.clone());
    assert_eq!(
        range_map_blaze.to_string(),
        "(97..=97, 101), (98..=98, 99), (100..=100, 101), (106..=106, 98)"
    );
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
fn map_repro1() {
    let (s1, s2, s3) = ("a".to_string(), "b".to_string(), "c".to_string());
    let mut range_map_blaze =
        RangeMapBlaze::from_iter([(20..=21, &s1), (24..=24, &s2), (25..=29, &s2)]);
    // println!("{range_map_blaze}");
    assert_eq!(
        range_map_blaze.to_string(),
        r#"(20..=21, "a"), (24..=29, "b")"#
    );
    range_map_blaze.internal_add(25..=25, &s3);
    // println!("{range_map_blaze}");
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

#[cfg(feature = "map_insert_cursor_experimental")]
fn assert_cursor_insert_matches<T, V>(
    initial: impl IntoIterator<Item = (RangeInclusive<T>, V)>,
    insertion: RangeInclusive<T>,
    value: V,
) where
    T: Integer + fmt::Debug,
    V: Eq + Clone + fmt::Debug,
{
    let mut baseline = RangeMapBlaze::new();
    for (range, value) in initial {
        baseline.internal_add_baseline(range, value);
    }
    let mut cursor = baseline.clone();

    baseline.internal_add_baseline(insertion.clone(), value.clone());
    cursor.internal_add_cursor(insertion, value);

    assert_eq!(cursor, baseline);
    assert!(cursor.len() == baseline.len());
    assert_eq!(cursor.ranges_len(), baseline.ranges_len());
    assert_eq!(
        cursor
            .range_values()
            .map(|(range, value)| (range, value.clone()))
            .collect::<Vec<_>>(),
        baseline
            .range_values()
            .map(|(range, value)| (range, value.clone()))
            .collect::<Vec<_>>()
    );
    assert!(cursor.len() == cursor.len_slow());
    assert!(baseline.len() == baseline.len_slow());
}

#[cfg(feature = "map_insert_cursor_experimental")]
#[test]
fn map_cursor_insert_targeted_differential() {
    let empty_start = 3;
    let empty_end = 2;
    let cases = [
        ("empty range", vec![(1..=3, 0)], empty_start..=empty_end, 1),
        ("empty map", vec![], 2..=4, 1),
        ("gap", vec![(1..=2, 0), (7..=8, 1)], 4..=5, 2),
        ("equal range", vec![(2..=6, 0)], 2..=6, 1),
        ("inside same", vec![(2..=8, 1)], 4..=6, 1),
        ("inside different", vec![(2..=8, 0)], 4..=6, 1),
        ("left edge", vec![(4..=8, 0)], 2..=5, 1),
        ("right edge", vec![(2..=5, 0)], 4..=8, 1),
        (
            "multiple ranges",
            vec![(1..=2, 0), (4..=5, 1), (7..=8, 2)],
            2..=7,
            1,
        ),
        ("whole map", vec![(2..=3, 0), (5..=7, 1)], 0..=9, 2),
        ("same predecessor", vec![(1..=3, 1)], 4..=6, 1),
        ("same successor", vec![(7..=9, 1)], 4..=6, 1),
        ("same on both sides", vec![(1..=3, 1), (7..=9, 1)], 4..=6, 1),
        ("different predecessor", vec![(1..=3, 0)], 4..=6, 1),
        ("different successor", vec![(7..=9, 0)], 4..=6, 1),
        (
            "exact start exposes predecessor",
            vec![(1..=3, 0), (4..=6, 1)],
            4..=5,
            0,
        ),
        (
            "exact end exposes successor",
            vec![(1..=3, 0), (4..=6, 1), (7..=9, 2)],
            4..=7,
            2,
        ),
        ("one point", vec![(4..=4, 0)], 4..=4, 1),
        ("minimum", vec![(1..=3, 0)], u8::MIN..=1, 1),
        ("maximum", vec![(252..=254, 0)], 254..=u8::MAX, 1),
        (
            "full domain",
            vec![(1..=3, 0), (250..=254, 1)],
            u8::MIN..=u8::MAX,
            2,
        ),
        (
            "alternating chain",
            vec![(0..=0, 1), (1..=1, 0), (2..=2, 1), (3..=3, 2), (4..=4, 1)],
            1..=3,
            1,
        ),
    ];

    for (name, initial, insertion, value) in cases {
        std::println!("cursor insertion case: {name}");
        assert_cursor_insert_matches(initial, insertion, value);
    }

    assert_cursor_insert_matches(
        [('\u{D7FE}'..='\u{D7FF}', 0), ('\u{E000}'..='\u{E001}', 0)],
        '\u{D7FF}'..='\u{E000}',
        0,
    );
    assert_cursor_insert_matches(
        [(char::MIN..='\u{0001}', 0), ('\u{10FFFE}'..=char::MAX, 1)],
        char::MIN..=char::MAX,
        2,
    );
}

#[cfg(feature = "map_insert_cursor_experimental")]
#[test]
fn map_cursor_insert_exhaustive_small_domain() {
    const DOMAIN_END: u8 = 4;
    const DOMAIN_LEN: usize = 5;
    const MAP_COUNT: usize = 1_024;
    const STATE_COUNT: usize = 4; // absent, or one of three values

    for mut encoded_map in 0..MAP_COUNT {
        let mut states = [None; DOMAIN_LEN];
        for state in &mut states {
            let digit = encoded_map % STATE_COUNT;
            *state = match digit {
                0 => None,
                1 => Some(0),
                2 => Some(1),
                3 => Some(2),
                _ => unreachable!("a base-four digit is in 0..=3"),
            };
            encoded_map /= STATE_COUNT;
        }

        let mut initial = RangeMapBlaze::new();
        for key in 0..=DOMAIN_END {
            if let Some(value) = states[usize::from(key)] {
                initial.internal_add_baseline(key..=key, value);
            }
        }

        for start in 0..=DOMAIN_END {
            for end in 0..=DOMAIN_END {
                for value in 0..3 {
                    let mut baseline = initial.clone();
                    let mut cursor = initial.clone();
                    baseline.internal_add_baseline(start..=end, value);
                    cursor.internal_add_cursor(start..=end, value);

                    assert_eq!(cursor, baseline);
                    assert_eq!(cursor.len(), cursor.len_slow());
                    assert_eq!(cursor.ranges_len(), baseline.ranges_len());
                    for key in 0..=DOMAIN_END {
                        let expected = if start <= end && start <= key && key <= end {
                            Some(value)
                        } else {
                            states[key as usize]
                        };
                        assert_eq!(cursor.get(key).copied(), expected);
                    }
                }
            }
        }
    }
}

#[cfg(feature = "map_insert_cursor_experimental")]
#[test]
fn map_cursor_insert_randomized_differential() {
    use rand::{SeedableRng, distr::Uniform, prelude::Distribution, rngs::StdRng};

    let mut rng = StdRng::seed_from_u64(0x5eed_c0de);
    let keys = Uniform::new_inclusive(0u16, 2_000).expect("valid key distribution");
    let values = Uniform::new(0u8, 7).expect("valid value distribution");
    let mut baseline = RangeMapBlaze::new();
    let mut cursor = RangeMapBlaze::new();

    for _ in 0..10_000 {
        let start = keys.sample(&mut rng);
        let end = keys.sample(&mut rng);
        let value = values.sample(&mut rng);
        baseline.internal_add_baseline(start..=end, value);
        cursor.internal_add_cursor(start..=end, value);
        assert_eq!(cursor, baseline);
        assert_eq!(cursor.len(), cursor.len_slow());
    }
}

#[cfg(feature = "map_insert_cursor_experimental")]
#[test]
fn map_cursor_insert_clones_only_for_two_residuals() {
    use alloc::rc::Rc;
    use core::cell::Cell;

    #[derive(Debug)]
    struct CloneCounted {
        id: u8,
        clone_count: Rc<Cell<usize>>,
    }

    impl Clone for CloneCounted {
        fn clone(&self) -> Self {
            self.clone_count.set(self.clone_count.get() + 1);
            Self {
                id: self.id,
                clone_count: Rc::clone(&self.clone_count),
            }
        }
    }

    impl PartialEq for CloneCounted {
        fn eq(&self, other: &Self) -> bool {
            self.id == other.id
        }
    }

    impl Eq for CloneCounted {}

    fn clone_count_for(initial: RangeInclusive<u8>, insertion: RangeInclusive<u8>) -> usize {
        let clone_count = Rc::new(Cell::new(0));
        let old_value = CloneCounted {
            id: 0,
            clone_count: Rc::clone(&clone_count),
        };
        let mut map = RangeMapBlaze::new();
        map.internal_add_baseline(initial, old_value);
        clone_count.set(0);
        map.internal_add_cursor(
            insertion,
            CloneCounted {
                id: 1,
                clone_count: Rc::new(Cell::new(0)),
            },
        );
        clone_count.get()
    }

    assert_eq!(clone_count_for(0..=100, 40..=60), 1);
    assert_eq!(clone_count_for(0..=60, 40..=100), 0);
    assert_eq!(clone_count_for(40..=100, 0..=60), 0);
    assert_eq!(clone_count_for(40..=60, 0..=100), 0);
}

#[cfg(all(
    feature = "map_insert_cursor_experimental",
    not(target_arch = "wasm32")
))]
fn direct_benchmark_map(
    ranges: impl IntoIterator<Item = (RangeInclusive<u32>, u32)>,
) -> RangeMapBlaze<u32, u32> {
    let mut map = RangeMapBlaze::new();
    for (range, value) in ranges {
        map.internal_add_baseline(range, value);
    }
    map
}

#[cfg(all(
    feature = "map_insert_cursor_experimental",
    not(target_arch = "wasm32")
))]
fn benchmark_direct_insert_case(
    group: &mut criterion::BenchmarkGroup<'_, criterion::measurement::WallTime>,
    name: &str,
    initial: &RangeMapBlaze<u32, u32>,
    insertion: RangeInclusive<u32>,
    value: u32,
) {
    use criterion::{BatchSize, BenchmarkId, black_box};

    group.bench_function(BenchmarkId::new("baseline", name), |bencher| {
        bencher.iter_batched_ref(
            || initial.clone(),
            |map| {
                map.internal_add_baseline(insertion.clone(), value);
                black_box(map);
            },
            BatchSize::SmallInput,
        );
    });
    group.bench_function(BenchmarkId::new("cursor", name), |bencher| {
        bencher.iter_batched_ref(
            || initial.clone(),
            |map| {
                map.internal_add_cursor(insertion.clone(), value);
                black_box(map);
            },
            BatchSize::SmallInput,
        );
    });
}

#[cfg(all(
    feature = "map_insert_cursor_experimental",
    not(target_arch = "wasm32")
))]
fn benchmark_direct_ingestion_case(
    group: &mut criterion::BenchmarkGroup<'_, criterion::measurement::WallTime>,
) {
    use criterion::{BatchSize, BenchmarkId, black_box};

    let ingestion: Vec<_> = (0..1_000)
        .map(|i| ((i * 7)..=(i * 7 + i % 5), i % 3))
        .collect();
    for (implementation, insert) in [
        (
            "baseline",
            RangeMapBlaze::internal_add_baseline
                as fn(&mut RangeMapBlaze<u32, u32>, RangeInclusive<u32>, u32),
        ),
        ("cursor", RangeMapBlaze::internal_add_cursor),
    ] {
        group.bench_function(
            BenchmarkId::new(implementation, "repeated_ingestion_1000"),
            |bencher| {
                bencher.iter_batched_ref(
                    RangeMapBlaze::new,
                    |map| {
                        for &(ref range, value) in &ingestion {
                            insert(map, range.clone(), value);
                        }
                        black_box(map);
                    },
                    BatchSize::SmallInput,
                );
            },
        );
    }
}

#[cfg(all(
    feature = "map_insert_cursor_experimental",
    not(target_arch = "wasm32")
))]
#[test]
#[ignore = "run explicitly to compare private baseline and cursor insertion with Criterion"]
fn benchmark_map_cursor_insert_direct() {
    use criterion::Criterion;
    use std::time::Duration;

    let sparse_small = direct_benchmark_map((0..32).map(|i| ((i * 10)..=(i * 10 + 2), i % 2)));
    let sparse = direct_benchmark_map((0..1_000).map(|i| ((i * 10)..=(i * 10 + 2), i % 2)));
    let sparse_large = direct_benchmark_map((0..8_192).map(|i| ((i * 10)..=(i * 10 + 2), i % 2)));
    let dense = direct_benchmark_map((0..1_000).map(|i| (i..=i, i % 2)));
    let containing = direct_benchmark_map([(0..=100_000, 1)]);
    let small_overlap =
        direct_benchmark_map([(0..=9, 0), (20..=29, 1), (40..=49, 0), (60..=69, 1)]);
    let coalescing = direct_benchmark_map([(0..=9, 1), (20..=29, 0), (40..=49, 1)]);
    let large_overwrite = direct_benchmark_map((0..4_096).map(|i| (i..=i, i % 2)));

    let mut criterion = Criterion::default()
        .sample_size(30)
        .warm_up_time(Duration::from_secs(1))
        .measurement_time(Duration::from_secs(2))
        .without_plots();
    {
        let mut group = criterion.benchmark_group("map_insert_direct");

        benchmark_direct_insert_case(
            &mut group,
            "single_sparse_r32_k0",
            &sparse_small,
            165..=165,
            3,
        );
        benchmark_direct_insert_case(
            &mut group,
            "single_sparse_r1000_k0",
            &sparse,
            5_005..=5_005,
            3,
        );
        benchmark_direct_insert_case(
            &mut group,
            "single_sparse_r8192_k0",
            &sparse_large,
            40_965..=40_965,
            3,
        );
        benchmark_direct_insert_case(&mut group, "single_dense_r1000_k1", &dense, 500..=500, 3);
        benchmark_direct_insert_case(&mut group, "gap_r1000_k0", &sparse, 5_005..=5_007, 3);
        benchmark_direct_insert_case(
            &mut group,
            "contained_same_r1_k1",
            &containing,
            40_000..=60_000,
            1,
        );
        benchmark_direct_insert_case(
            &mut group,
            "exact_start_same_r1_k1",
            &containing,
            0..=60_000,
            1,
        );
        benchmark_direct_insert_case(
            &mut group,
            "contained_different_r1_k1",
            &containing,
            40_000..=60_000,
            2,
        );
        benchmark_direct_insert_case(
            &mut group,
            "small_overlap_r4_k2",
            &small_overlap,
            25..=44,
            2,
        );
        benchmark_direct_insert_case(
            &mut group,
            "large_overwrite_r4096_k17",
            &large_overwrite,
            2_040..=2_056,
            3,
        );
        benchmark_direct_insert_case(
            &mut group,
            "large_overwrite_r4096_k257",
            &large_overwrite,
            1_920..=2_176,
            3,
        );
        benchmark_direct_insert_case(
            &mut group,
            "large_overwrite_r4096_k3073",
            &large_overwrite,
            512..=3_584,
            3,
        );
        benchmark_direct_insert_case(&mut group, "coalesce_left_r3", &coalescing, 10..=15, 1);
        benchmark_direct_insert_case(&mut group, "coalesce_right_r3", &coalescing, 35..=39, 1);
        benchmark_direct_insert_case(&mut group, "coalesce_both_r3_k1", &coalescing, 10..=39, 1);

        benchmark_direct_ingestion_case(&mut group);
        group.finish();
    }
    criterion.final_summary();
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
fn test_eq_priority() {
    let a = Priority::new((1..=2, &"a"), 1);
    let b = Priority::new((1..=2, &"a"), 0);
    assert_ne!(a, b);
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
    is_sssu::<ARangeValuesIter<'_>>();
    is_like_btreemap_iter::<ARangeValuesIter<'_>>();

    type AIntoRangeValuesIter = IntoRangeValuesIter<i32, u64>;
    is_sssu::<AIntoRangeValuesIter>();
    is_like_btreemap_into_iter::<AIntoRangeValuesIter>();

    type AIterMap<'a> = IterMap<i32, &'a u64, ARangeValuesIter<'a>>;
    is_sssu::<AIterMap<'_>>();
    is_like_btreemap_iter_less_exact_size::<AIterMap<'_>>();

    type AIntoIterMap = IntoIterMap<i32, u64>;
    is_sssu::<AIntoIterMap>();
    is_like_btreemap_into_iter_less_exact_size::<AIntoIterMap>();

    type AKMergeMap<'a> = KMergeMap<i32, &'a u64, ARangeValuesIter<'a>>;
    is_sssu::<AKMergeMap<'_>>();
    is_like_btreemap_iter_less_both::<AKMergeMap<'_>>();

    type AMergeMap<'a> = MergeMap<i32, &'a u64, ARangeValuesIter<'a>, ARangeValuesIter<'a>>;
    is_sssu::<AMergeMap<'_>>();
    is_like_btreemap_iter_less_both::<AMergeMap<'_>>();

    type AAssumePrioritySortedStartsMap<'a> =
        AssumePrioritySortedStartsMap<vec::IntoIter<Priority<i32, &'a u64>>>;
    is_sssu::<AAssumePrioritySortedStartsMap<'_>>();
    is_like_btreemap_iter_less_both::<AAssumePrioritySortedStartsMap<'_>>();

    type AUnionIterMap<'a> = UnionIterMap<i32, &'a u64, AAssumePrioritySortedStartsMap<'a>>;
    is_sssu::<AUnionIterMap<'_>>();
    is_like_btreemap_iter_less_both::<AUnionIterMap<'_>>();

    type ASymDiffIterMap<'a> = SymDiffIterMap<i32, &'a u64, AAssumePrioritySortedStartsMap<'a>>;
    is_sssu::<ASymDiffIterMap<'_>>();
    is_like_btreemap_iter_less_both::<ASymDiffIterMap<'_>>();

    type ARangesIter<'a> = RangesIter<'a, i32>;

    type AIntersectionIterMap<'a> =
        IntersectionIterMap<i32, &'a u64, ARangeValuesIter<'a>, ARangesIter<'a>>;
    is_sssu::<AIntersectionIterMap<'_>>();
    is_like_btreemap_iter_less_both::<AIntersectionIterMap<'_>>();

    type AFillGapsIterMap<'a> = FillGapsIterMap<i32, &'a u64, ARangeValuesIter<'a>>;
    is_sssu::<AFillGapsIterMap<'_>>();
    is_like_btreemap_iter_less_both::<AFillGapsIterMap<'_>>();

    type AFillGapsIter<'a> = FillGapsIter<i32, ARangesIter<'a>>;
    is_sssu::<AFillGapsIter<'_>>();
    is_like_btreemap_iter_less_both::<AFillGapsIter<'_>>();

    type AKeys<'a> = Keys<i32, &'a u64, ARangeValuesIter<'a>>;
    is_sssu::<AKeys<'_>>();
    is_like_btreemap_iter_less_exact_size::<AKeys<'_>>();

    type AIntoKeys = IntoKeys<i32, u64>;
    is_sssu::<AIntoKeys>();
    is_like_btreemap_into_iter_less_exact_size::<AIntoKeys>();

    type AValues<'a> = Values<i32, &'a u64, ARangeValuesIter<'a>>;
    is_sssu::<AValues<'_>>();
    is_like_btreemap_iter_less_exact_size::<AValues<'_>>();

    type AIntoValues = IntoValues<i32, u64>;
    is_sssu::<AIntoValues>();
    is_like_btreemap_into_iter_less_exact_size::<AIntoValues>();

    type ARangeToRangeValueIter<'a> = RangeToRangeValueIter<'a, i32, u64, ARangesIter<'a>>;
    is_sssu::<ARangeToRangeValueIter<'_>>();
    is_like_btreemap_iter_less_both::<ARangeToRangeValueIter<'_>>();

    type ACheckSortedDisjointMap<'a> = CheckSortedDisjointMap<i32, &'a u64, ARangeValuesIter<'a>>;
    is_sssu::<ACheckSortedDisjointMap<'_>>();
    type BCheckSortedDisjointMap<'a> =
        CheckSortedDisjointMap<i32, &'a u64, array::IntoIter<(RangeInclusive<i32>, &'a u64), 0>>;
    is_like_check_sorted_disjoint_map::<BCheckSortedDisjointMap<'_>>();

    type ADynSortedDisjointMap<'a> = DynSortedDisjointMap<'a, i32, &'a u64>;
    is_like_dyn_sorted_disjoint_map::<ADynSortedDisjointMap<'_>>();
}

const fn is_ddcppdheo<
    T: fmt::Debug
        + fmt::Display
        + Clone
        + PartialEq
        + PartialOrd
        + Default
        + Hash
        + Eq
        + Ord
        + Send
        + Sync,
>() {
}

const fn is_sssu<T: Sized + Send + Sync + Unpin>() {}
const fn is_like_btreemap_iter<
    T: Clone + fmt::Debug + FusedIterator + Iterator + DoubleEndedIterator + ExactSizeIterator,
>() {
}

const fn is_like_btreemap_iter_less_exact_size<
    T: Clone + fmt::Debug + FusedIterator + Iterator + DoubleEndedIterator,
>() {
}

const fn is_like_btreemap_iter_less_both<T: Clone + fmt::Debug + FusedIterator + Iterator>() {}
const fn is_like_btreemap_into_iter<
    T: fmt::Debug + FusedIterator + Iterator + DoubleEndedIterator + ExactSizeIterator,
>() {
}
const fn is_like_btreemap_into_iter_less_exact_size<
    T: fmt::Debug + FusedIterator + Iterator + DoubleEndedIterator,
>() {
}

const fn is_like_btreemap<
    T: Clone
        + fmt::Debug
        + Default
        + Eq
        + Hash
        + IntoIterator
        + Ord
        + PartialEq
        + PartialOrd
        + RefUnwindSafe
        + Send
        + Sync
        + Unpin
        + UnwindSafe
        + Any
        + ToOwned,
>() {
}

const fn is_like_check_sorted_disjoint_map<
    T: Clone
        + fmt::Debug
        + IntoIterator
        + RefUnwindSafe
        + Send
        + Sync
        + Unpin
        + UnwindSafe
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

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
fn test_len_slow() {
    let a = RangeMapBlaze::from_iter([(1..=2, "a"), (5..=100, "a")]);
    assert_eq!(a.len_slow(), a.len());
    assert_eq!(a.len_slow(), 98u64);
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
fn test_union_2() {
    use rand::{SeedableRng, distr::Uniform, prelude::Distribution, rngs::StdRng};
    // use std::println;

    fn create(len: usize, v0: char, v1: char) -> RangeMapBlaze<u32, char> {
        let high = 99999;
        let mut rng = StdRng::seed_from_u64(0);
        let uniform_key =
            Uniform::new(0u32, high + 1).expect("Failed to create uniform distribution");
        if len == 0 {
            return RangeMapBlaze::new();
        }
        let mut result = RangeMapBlaze::from_iter([(0..=high, v0)]);
        for _ in 0..=high * 2 {
            // to avoid endless loop bug
            if result.ranges_len() >= len {
                return result;
            }
            let index = uniform_key.sample(&mut rng);
            result.insert(index, v1);
            // println!(
            //     "len={} of {len}", // inserted {index} for {result:?}",
            //     result.ranges_len()
            // );
        }
        panic!("Endless loop in test_union_2");
    }
    let len_list = [0, 1, 100, 10_000];
    for a_len in &len_list {
        let a = create(*a_len, 'A', 'a');
        for b_len in &len_list {
            let b = create(*b_len, 'B', 'b');
            let c0: RangeMapBlaze<u32, char> = a.range_values().chain(b.range_values()).collect();
            let c1 = a.clone() | b.clone();
            assert_eq!(c0, c1);
            let mut c2 = a.clone();
            c2 |= &b;
            assert_eq!(c0, c2);
            let mut c3 = a.clone();
            c3 |= b.clone();
            assert_eq!(c0, c3);
            let mut c4 = a.clone();
            c4.extend_simple(b.range_values().map(|(r, v)| (r, *v)));
            assert_eq!(c0, c4);
            let mut c5 = a.clone();
            c5.extend(b.range_values().map(|(r, v)| (r, *v)));
            assert_eq!(c0, c5);
            // extend_with and extend_from
            let mut c6 = a.clone();
            c6.extend_with(&b);
            assert_eq!(c0, c6);
            let mut c7 = a.clone();
            c7.extend_from(b.clone());
            assert_eq!(c0, c7);
            let mut c8 = a.clone();
            let mut b_clone = b.clone();
            c8.append(&mut b_clone);
            assert_eq!(c0, c8);
            assert!(b_clone.is_empty());
            let c9 = a.clone() | b.clone();
            assert_eq!(c0, c9);
            let c10 = a.clone() | &b;
            assert_eq!(c0, c10);
            let c11 = &a | b.clone();
            assert_eq!(c0, c11);
            let c12 = &a | &b;
            assert_eq!(c0, c12);
        }
    }
}
