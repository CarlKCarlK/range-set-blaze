#![cfg(test)]

use crate::{Integer, RangeMapBlaze, RangeSetBlaze};
use alloc::{vec, vec::Vec};
use core::{hint::black_box, ops::RangeInclusive};
#[cfg(not(target_arch = "wasm32"))]
use std::{prelude::v1::*, time::Instant};

#[test]
fn cursor_lookup_matches_baseline_exhaustive_small_domain() {
    for mask in 0_u16..=u16::from(u8::MAX) {
        let set = (0_u8..8)
            .filter(|value| mask & (1 << value) != 0)
            .collect::<RangeSetBlaze<_>>();
        let map = (0_u8..8)
            .filter(|value| mask & (1 << value) != 0)
            .map(|value| (value, value % 3))
            .collect::<RangeMapBlaze<_, _>>();

        for key in u8::MIN..=u8::MAX {
            assert_eq!(
                set.range_or_gap_at_cursor(key),
                set.range_or_gap_at_baseline(key),
                "set mask={mask:#010b}, key={key}"
            );
            assert_eq!(
                map.range_or_gap_at_cursor(key),
                map.range_or_gap_at_baseline(key),
                "map mask={mask:#010b}, key={key}"
            );
        }
    }
}

#[test]
fn cursor_lookup_matches_baseline_randomized() {
    let mut state = 0x4d59_5df4_d0f3_3173_u64;
    for _ in 0..256 {
        let mut set_ranges = Vec::new();
        let mut map_ranges = Vec::new();
        for _ in 0..64 {
            let start = i16::from_ne_bytes(next_random(&mut state).to_ne_bytes());
            let width = i16::try_from(next_random(&mut state) % 32).expect("width fits");
            let end = start.saturating_add(width);
            set_ranges.push(start..=end);
            map_ranges.push((start..=end, next_random(&mut state) % 5));
        }
        let set = RangeSetBlaze::from_iter(set_ranges);
        let map = RangeMapBlaze::from_iter(map_ranges);

        for key in [i16::MIN, i16::MAX]
            .into_iter()
            .chain((0..256).map(|_| i16::from_ne_bytes(next_random(&mut state).to_ne_bytes())))
        {
            assert_eq!(
                set.range_or_gap_at_cursor(key),
                set.range_or_gap_at_baseline(key)
            );
            assert_eq!(
                map.range_or_gap_at_cursor(key),
                map.range_or_gap_at_baseline(key)
            );
        }
    }
}

#[test]
fn cursor_lookup_matches_baseline_at_char_surrogate_gap() {
    let set = RangeSetBlaze::from_iter(['\u{D7FD}'..='\u{D7FF}', '\u{E001}'..='\u{E003}']);
    let map = RangeMapBlaze::from_iter([
        ('\u{D7FD}'..='\u{D7FF}', 1_u8),
        ('\u{E001}'..='\u{E003}', 2_u8),
    ]);

    for key in [
        '\u{D7FC}', '\u{D7FD}', '\u{D7FF}', '\u{E000}', '\u{E001}', '\u{E003}', '\u{E004}',
    ] {
        assert_eq!(
            set.range_or_gap_at_cursor(key),
            set.range_or_gap_at_baseline(key)
        );
        assert_eq!(
            map.range_or_gap_at_cursor(key),
            map.range_or_gap_at_baseline(key)
        );
    }
}

fn next_random(state: &mut u64) -> u16 {
    *state = state
        .wrapping_mul(6_364_136_223_846_793_005)
        .wrapping_add(1);
    u16::try_from((*state >> 32) & u64::from(u16::MAX)).expect("masked value fits")
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
#[ignore = "run in release mode to compare private baseline and cursor lookups directly"]
fn benchmark_cursor_lookup_direct() {
    const ITERATIONS: u64 = 200_000;

    println!("method\tcase\tr\tbaseline ns/op\tcursor ns/op\tspeedup");
    for range_count in [0, 1, 32, 1_000, 8_192] {
        let (set, map) = benchmark_collections(range_count);
        for (name, key) in benchmark_keys(range_count) {
            let baseline = time_set_query(
                &set,
                key,
                RangeSetBlaze::range_or_gap_at_baseline,
                ITERATIONS,
            );
            let cursor =
                time_set_query(&set, key, RangeSetBlaze::range_or_gap_at_cursor, ITERATIONS);
            println!(
                "set_range_or_gap_at\t{name}\t{range_count}\t{baseline:.2}\t{cursor:.2}\t{:.3}x",
                baseline / cursor
            );

            let baseline = time_map_query(
                &map,
                key,
                RangeMapBlaze::range_or_gap_at_baseline,
                ITERATIONS,
            );
            let cursor =
                time_map_query(&map, key, RangeMapBlaze::range_or_gap_at_cursor, ITERATIONS);
            println!(
                "map_range_or_gap_at\t{name}\t{range_count}\t{baseline:.2}\t{cursor:.2}\t{:.3}x",
                baseline / cursor
            );
        }
    }

    let large_gap_set = RangeSetBlaze::from_iter([-1_000_000..=-999_998, 1_000_000..=1_000_002]);
    let large_gap_map = RangeMapBlaze::from_iter([
        (-1_000_000..=-999_998, 1_u32),
        (1_000_000..=1_000_002, 2_u32),
    ]);
    print_i32_benchmark_case(
        "large_internal_gap",
        &large_gap_set,
        &large_gap_map,
        0,
        ITERATIONS,
    );

    let boundary_set = RangeSetBlaze::from_iter([i32::MIN..=i32::MIN, i32::MAX..=i32::MAX]);
    let boundary_map =
        RangeMapBlaze::from_iter([(i32::MIN..=i32::MIN, 1_u32), (i32::MAX..=i32::MAX, 2_u32)]);
    print_i32_benchmark_case(
        "present_min",
        &boundary_set,
        &boundary_map,
        i32::MIN,
        ITERATIONS,
    );
    print_i32_benchmark_case(
        "present_max",
        &boundary_set,
        &boundary_map,
        i32::MAX,
        ITERATIONS,
    );

    let char_set = RangeSetBlaze::from_iter(['\u{D7FD}'..='\u{D7FF}', '\u{E001}'..='\u{E003}']);
    let char_map = RangeMapBlaze::from_iter([
        ('\u{D7FD}'..='\u{D7FF}', 1_u32),
        ('\u{E001}'..='\u{E003}', 2_u32),
    ]);
    let baseline = time_set_query(
        &char_set,
        '\u{E000}',
        RangeSetBlaze::range_or_gap_at_baseline,
        ITERATIONS,
    );
    let cursor = time_set_query(
        &char_set,
        '\u{E000}',
        RangeSetBlaze::range_or_gap_at_cursor,
        ITERATIONS,
    );
    println!(
        "set_range_or_gap_at\tchar_surrogate_boundary\t2\t{baseline:.2}\t{cursor:.2}\t{:.3}x",
        baseline / cursor
    );
    let baseline = time_map_query(
        &char_map,
        '\u{E000}',
        RangeMapBlaze::range_or_gap_at_baseline,
        ITERATIONS,
    );
    let cursor = time_map_query(
        &char_map,
        '\u{E000}',
        RangeMapBlaze::range_or_gap_at_cursor,
        ITERATIONS,
    );
    println!(
        "map_range_or_gap_at\tchar_surrogate_boundary\t2\t{baseline:.2}\t{cursor:.2}\t{:.3}x",
        baseline / cursor
    );
}

#[cfg(not(target_arch = "wasm32"))]
fn benchmark_collections(range_count: usize) -> (RangeSetBlaze<i32>, RangeMapBlaze<i32, u32>) {
    let ranges = (0..range_count).map(|index| {
        let start = -100_000 + i32::try_from(index).expect("benchmark range count fits") * 8;
        start..=start + 2
    });
    let set = ranges.clone().collect::<RangeSetBlaze<_>>();
    let map = ranges
        .enumerate()
        .map(|(index, range)| (range, u32::try_from(index).expect("index fits")))
        .collect::<RangeMapBlaze<_, _>>();
    (set, map)
}

#[cfg(not(target_arch = "wasm32"))]
fn benchmark_keys(range_count: usize) -> Vec<(&'static str, i32)> {
    if range_count == 0 {
        return vec![
            ("empty_min", i32::MIN),
            ("empty_middle", 0),
            ("empty_max", i32::MAX),
        ];
    }

    let middle = i32::try_from(range_count / 2).expect("benchmark range count fits");
    let start = -100_000 + middle * 8;
    let last_start =
        -100_000 + i32::try_from(range_count - 1).expect("benchmark range count fits") * 8;
    vec![
        ("present_exact_start", start),
        ("present_interior", start + 1),
        ("present_exact_end", start + 2),
        ("close_gap", start + 3),
        ("before_first", i32::MIN),
        ("after_last", i32::MAX),
        ("large_gap", last_start.saturating_add(1_000_000)),
    ]
}

#[cfg(not(target_arch = "wasm32"))]
fn print_i32_benchmark_case(
    name: &str,
    set: &RangeSetBlaze<i32>,
    map: &RangeMapBlaze<i32, u32>,
    key: i32,
    iterations: u64,
) {
    let baseline = time_set_query(
        set,
        key,
        RangeSetBlaze::range_or_gap_at_baseline,
        iterations,
    );
    let cursor = time_set_query(set, key, RangeSetBlaze::range_or_gap_at_cursor, iterations);
    println!(
        "set_range_or_gap_at\t{name}\t{}\t{baseline:.2}\t{cursor:.2}\t{:.3}x",
        set.ranges_len(),
        baseline / cursor
    );

    let baseline = time_map_query(
        map,
        key,
        RangeMapBlaze::range_or_gap_at_baseline,
        iterations,
    );
    let cursor = time_map_query(map, key, RangeMapBlaze::range_or_gap_at_cursor, iterations);
    println!(
        "map_range_or_gap_at\t{name}\t{}\t{baseline:.2}\t{cursor:.2}\t{:.3}x",
        map.range_values_len(),
        baseline / cursor
    );
}

#[cfg(not(target_arch = "wasm32"))]
fn time_set_query<T: Integer>(
    set: &RangeSetBlaze<T>,
    key: T,
    query: fn(&RangeSetBlaze<T>, T) -> (RangeInclusive<T>, bool),
    iterations: u64,
) -> f64 {
    median_time(|| {
        for _ in 0..iterations {
            black_box(query(black_box(set), black_box(key)));
        }
    }) / f64::from(u32::try_from(iterations).expect("benchmark iterations fit"))
}

#[cfg(not(target_arch = "wasm32"))]
fn time_map_query<T: Integer>(
    map: &RangeMapBlaze<T, u32>,
    key: T,
    query: MapLookup<T>,
    iterations: u64,
) -> f64 {
    median_time(|| {
        for _ in 0..iterations {
            black_box(query(black_box(map), black_box(key)));
        }
    }) / f64::from(u32::try_from(iterations).expect("benchmark iterations fit"))
}

type MapLookup<T> =
    for<'a> fn(&'a RangeMapBlaze<T, u32>, T) -> (RangeInclusive<T>, Option<&'a u32>);

#[cfg(not(target_arch = "wasm32"))]
fn median_time(mut run: impl FnMut()) -> f64 {
    const SAMPLES: usize = 9;
    let mut samples = [0.0; SAMPLES];
    for sample in &mut samples {
        let start = Instant::now();
        run();
        *sample = start.elapsed().as_secs_f64() * 1_000_000_000.0;
    }
    samples.sort_by(f64::total_cmp);
    samples[SAMPLES / 2]
}
