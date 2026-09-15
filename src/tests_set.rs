#![cfg(test)]

use super::*;
#[cfg(not(target_arch = "wasm32"))]
use crate::set::extract_range;
use crate::{sorted_disjoint_map::Priority, unsorted_priority_map::AssumePrioritySortedStartsMap};
#[cfg(not(target_arch = "wasm32"))]
use alloc::format;
use alloc::{string::ToString, vec, vec::Vec};
#[cfg(feature = "cursor_nightly_experimental")]
use core::fmt;
use core::{array, iter::once, ops::RangeInclusive};
#[cfg(not(target_arch = "wasm32"))]
use core::{cmp::Ordering, ops::Bound};
#[cfg(not(target_arch = "wasm32"))]
use num_traits::{One, Zero};
#[cfg(not(target_arch = "wasm32"))]
use std::{collections::hash_map::DefaultHasher, prelude::v1::*};
#[cfg(not(target_arch = "wasm32"))]
use syntactic_for::syntactic_for;

use wasm_bindgen_test::*;
wasm_bindgen_test_configure!(run_in_browser);
#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
fn demo_f1() {
    // before_or_equal_exists	0
    //     INSERT, etc

    let mut range_set_blaze = RangeSetBlaze::from_iter([11..=14, 22..=26]);
    range_set_blaze.internal_add(10..=10);
    assert_eq!(range_set_blaze.to_string(), "10..=14, 22..=26");
    // println!(
    //     "demo_1 range_set_blaze = {:?}, len_slow = {}, len = {}",
    //     range_set_blaze,
    //     range_set_blaze.len_slow(),
    //     range_set_blaze.len()
    // );

    assert_eq!(range_set_blaze.len_slow(), range_set_blaze.len());
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
                    // print!("{a}\t{b}\t{c}\t{d}\t");
                    if a > b {
                        // println!("impossible");
                    } else if c > d {
                        // println!("error");
                    } else {
                        let mut range_set_blaze = RangeSetBlaze::new();
                        range_set_blaze.internal_add(a..=b);
                        range_set_blaze.internal_add(c..=d);
                        if range_set_blaze.ranges_len() == 1 {
                            // let vec = range_set_blaze.into_iter().collect::<Vec<u8>>();
                            // println!("combine\t{}\t{}", vec[0], vec[vec.len() - 1]);
                            assert!(!restart);
                        } else {
                            // println!("restart");
                            assert!(restart);
                        }
                    }
                }
            }
        }
    }
}

#[cfg(feature = "cursor_nightly_experimental")]
fn assert_set_cursor_insert_matches<T>(
    initial: impl IntoIterator<Item = RangeInclusive<T>>,
    insertion: RangeInclusive<T>,
) where
    T: Integer + fmt::Debug,
{
    let mut baseline = RangeSetBlaze::new();
    for range in initial {
        baseline.internal_add_baseline(range);
    }
    let mut cursor = baseline.clone();

    baseline.internal_add_baseline(insertion.clone());
    cursor.internal_add_cursor(insertion);

    assert_eq!(cursor, baseline);
    assert!(cursor.len() == baseline.len());
    assert_eq!(cursor.ranges_len(), baseline.ranges_len());
    assert_eq!(
        cursor.ranges().collect::<Vec<_>>(),
        baseline.ranges().collect::<Vec<_>>()
    );
    assert!(cursor.len() == cursor.len_slow());
    assert!(baseline.len() == baseline.len_slow());
}

#[cfg(feature = "cursor_nightly_experimental")]
#[test]
fn set_cursor_insert_targeted_differential() {
    let empty_start = 3;
    let empty_end = 2;
    let cases = [
        ("empty range", vec![1..=3], empty_start..=empty_end),
        ("empty map", vec![], 2..=4),
        ("clean gap", vec![1..=2, 7..=8], 4..=5),
        ("single point gap", vec![1..=2, 4..=5], 7..=7),
        ("contained", vec![2..=8], 4..=6),
        ("exact range", vec![2..=8], 2..=8),
        ("touch predecessor", vec![1..=3, 8..=9], 4..=6),
        ("touch successor", vec![1..=2, 7..=9], 4..=6),
        ("touch both", vec![1..=3, 7..=9], 4..=6),
        ("overlap predecessor", vec![1..=4, 9..=10], 3..=7),
        ("overlap successor", vec![1..=2, 6..=9], 4..=7),
        ("bridge", vec![1..=4, 7..=10], 3..=8),
        (
            "absorb many",
            vec![1..=2, 5..=6, 9..=10, 13..=14, 17..=18],
            2..=17,
        ),
        ("minimum", vec![1..=254], u8::MIN..=1),
        ("maximum", vec![1..=2, 252..=254], 254..=u8::MAX),
        (
            "full domain",
            vec![1..=3, 7..=9, 250..=254],
            u8::MIN..=u8::MAX,
        ),
    ];

    for (name, initial, insertion) in cases {
        std::println!("set cursor insertion case: {name}");
        assert_set_cursor_insert_matches(initial, insertion);
    }

    assert_set_cursor_insert_matches(
        ['\u{D7FE}'..='\u{D7FE}', '\u{E001}'..='\u{E001}'],
        '\u{D7FF}'..='\u{E000}',
    );
    assert_set_cursor_insert_matches(
        [char::MIN..='\u{0001}', '\u{10FFFE}'..=char::MAX],
        char::MIN..=char::MAX,
    );
}

#[cfg(feature = "cursor_nightly_experimental")]
#[test]
fn set_cursor_insert_exhaustive_small_domain() {
    const DOMAIN_END: u8 = 6;

    for bits in 0..(1_u16 << (DOMAIN_END + 1)) {
        let mut initial = RangeSetBlaze::new();
        for key in 0..=DOMAIN_END {
            if bits & (1 << key) != 0 {
                initial.internal_add_baseline(key..=key);
            }
        }

        for start in 0..=DOMAIN_END {
            for end in 0..=DOMAIN_END {
                let mut baseline = initial.clone();
                let mut cursor = initial.clone();
                baseline.internal_add_baseline(start..=end);
                cursor.internal_add_cursor(start..=end);

                assert_eq!(cursor, baseline);
                assert_eq!(cursor.len(), cursor.len_slow());
                assert_eq!(cursor.ranges_len(), baseline.ranges_len());
                assert_eq!(
                    cursor.ranges().collect::<Vec<_>>(),
                    baseline.ranges().collect::<Vec<_>>()
                );
                for key in 0..=DOMAIN_END {
                    let expected = bits & (1 << key) != 0 || start <= key && key <= end;
                    assert_eq!(cursor.contains(key), expected);
                }
            }
        }
    }
}

#[cfg(feature = "cursor_nightly_experimental")]
#[test]
fn set_cursor_insert_randomized_differential() {
    use rand::{SeedableRng, distr::Uniform, prelude::Distribution, rngs::StdRng};

    let mut rng = StdRng::seed_from_u64(0x5e7_c0de);
    let keys = Uniform::new_inclusive(0u16, 10_000).expect("valid key distribution");
    let mut baseline = RangeSetBlaze::new();
    let mut cursor = RangeSetBlaze::new();

    for _ in 0..20_000 {
        let start = keys.sample(&mut rng);
        let end = keys.sample(&mut rng);
        baseline.internal_add_baseline(start..=end);
        cursor.internal_add_cursor(start..=end);
        assert_eq!(cursor, baseline);
        assert_eq!(cursor.len(), cursor.len_slow());
        assert_eq!(
            cursor.ranges().collect::<Vec<_>>(),
            baseline.ranges().collect::<Vec<_>>()
        );
    }
}

#[cfg(all(feature = "cursor_nightly_experimental", not(target_arch = "wasm32")))]
fn direct_benchmark_set(
    ranges: impl IntoIterator<Item = RangeInclusive<u32>>,
) -> RangeSetBlaze<u32> {
    let mut set = RangeSetBlaze::new();
    for range in ranges {
        set.internal_add_baseline(range);
    }
    set
}

#[cfg(all(feature = "cursor_nightly_experimental", not(target_arch = "wasm32")))]
fn time_direct_set_insert<T: Integer>(
    initial: &RangeSetBlaze<T>,
    insertion: &RangeInclusive<T>,
    insert: fn(&mut RangeSetBlaze<T>, RangeInclusive<T>),
    repetitions: u32,
) -> f64 {
    use std::{hint::black_box, time::Instant};

    let mut samples = Vec::with_capacity(7);
    for _ in 0..7 {
        let mut sets =
            vec![initial.clone(); usize::try_from(repetitions).expect("repetitions fit usize")];
        let start = Instant::now();
        for set in &mut sets {
            insert(set, black_box(insertion.clone()));
        }
        let elapsed = start.elapsed();
        black_box(&sets);
        samples.push(elapsed.as_secs_f64() * 1e9 / f64::from(repetitions));
    }
    samples.sort_by(f64::total_cmp);
    samples[samples.len() / 2]
}

#[cfg(all(feature = "cursor_nightly_experimental", not(target_arch = "wasm32")))]
fn time_direct_set_ingestion(
    ingestion: &[RangeInclusive<u32>],
    insert: fn(&mut RangeSetBlaze<u32>, RangeInclusive<u32>),
) -> f64 {
    use std::{hint::black_box, time::Instant};

    let mut samples = Vec::with_capacity(7);
    for _ in 0..7 {
        let mut sets = vec![RangeSetBlaze::new(); 200];
        let start = Instant::now();
        for set in &mut sets {
            for range in ingestion {
                insert(set, black_box(range.clone()));
            }
        }
        let elapsed = start.elapsed();
        black_box(&sets);
        samples.push(elapsed.as_secs_f64() * 1e9 / 200.0);
    }
    samples.sort_by(f64::total_cmp);
    samples[samples.len() / 2]
}

#[cfg(all(feature = "cursor_nightly_experimental", not(target_arch = "wasm32")))]
struct SetInsertBenchmarkCase {
    name: &'static str,
    initial: RangeSetBlaze<u32>,
    insertion: RangeInclusive<u32>,
    repetitions: u32,
}

#[cfg(all(feature = "cursor_nightly_experimental", not(target_arch = "wasm32")))]
fn set_insert_benchmark_cases_first() -> [SetInsertBenchmarkCase; 9] {
    let sparse = |count| direct_benchmark_set((0..count).map(|i| (i * 10)..=(i * 10 + 2)));
    let separated_points = |count| direct_benchmark_set((0..count).map(|i| (i * 4)..=(i * 4)));
    let local = direct_benchmark_set([0..=9, 20..=29, 40..=49, 60..=69]);
    [
        SetInsertBenchmarkCase {
            name: "empty_r0_k0",
            initial: RangeSetBlaze::new(),
            insertion: 10..=12,
            repetitions: 50_000,
        },
        SetInsertBenchmarkCase {
            name: "single_point_r2_k0",
            initial: direct_benchmark_set([0..=2, 10..=12]),
            insertion: 6..=6,
            repetitions: 30_000,
        },
        SetInsertBenchmarkCase {
            name: "sparse_gap_r32_k0",
            initial: sparse(32),
            insertion: 165..=167,
            repetitions: 20_000,
        },
        SetInsertBenchmarkCase {
            name: "sparse_gap_r1000_k0",
            initial: sparse(1_000),
            insertion: 5_005..=5_007,
            repetitions: 3_000,
        },
        SetInsertBenchmarkCase {
            name: "sparse_gap_r8192_k0",
            initial: sparse(8_192),
            insertion: 40_965..=40_967,
            repetitions: 500,
        },
        SetInsertBenchmarkCase {
            name: "dense_local_r1000_k2",
            initial: separated_points(1_000),
            insertion: 1_998..=2_002,
            repetitions: 3_000,
        },
        SetInsertBenchmarkCase {
            name: "contained_r1_k1",
            initial: direct_benchmark_set([0..=100_000]),
            insertion: 40_000..=60_000,
            repetitions: 50_000,
        },
        SetInsertBenchmarkCase {
            name: "exact_r1_k1",
            initial: direct_benchmark_set([0..=100_000]),
            insertion: 0..=100_000,
            repetitions: 50_000,
        },
        SetInsertBenchmarkCase {
            name: "coalesce_left_r4_k1",
            initial: local,
            insertion: 10..=15,
            repetitions: 20_000,
        },
    ]
}

#[cfg(all(feature = "cursor_nightly_experimental", not(target_arch = "wasm32")))]
fn set_insert_benchmark_cases_second() -> [SetInsertBenchmarkCase; 9] {
    let separated_points = |count| direct_benchmark_set((0..count).map(|i| (i * 4)..=(i * 4)));
    let local = direct_benchmark_set([0..=9, 20..=29, 40..=49, 60..=69]);
    [
        SetInsertBenchmarkCase {
            name: "coalesce_right_r4_k1",
            initial: local.clone(),
            insertion: 34..=39,
            repetitions: 20_000,
        },
        SetInsertBenchmarkCase {
            name: "coalesce_both_r2_k2",
            initial: direct_benchmark_set([0..=9, 20..=29]),
            insertion: 10..=19,
            repetitions: 20_000,
        },
        SetInsertBenchmarkCase {
            name: "small_overlap_r4_k2",
            initial: local,
            insertion: 25..=44,
            repetitions: 20_000,
        },
        SetInsertBenchmarkCase {
            name: "minimum_boundary_r1_k1",
            initial: direct_benchmark_set([1..=10]),
            insertion: u32::MIN..=1,
            repetitions: 30_000,
        },
        SetInsertBenchmarkCase {
            name: "maximum_boundary_r1_k1",
            initial: direct_benchmark_set([(u32::MAX - 10)..=(u32::MAX - 1)]),
            insertion: (u32::MAX - 1)..=u32::MAX,
            repetitions: 30_000,
        },
        SetInsertBenchmarkCase {
            name: "full_domain_r3_k3",
            initial: direct_benchmark_set([1..=3, 100..=200, (u32::MAX - 3)..=(u32::MAX - 1)]),
            insertion: u32::MIN..=u32::MAX,
            repetitions: 20_000,
        },
        SetInsertBenchmarkCase {
            name: "absorb_r4096_k17",
            initial: separated_points(4_096),
            insertion: 8_160..=8_224,
            repetitions: 1_000,
        },
        SetInsertBenchmarkCase {
            name: "absorb_r4096_k257",
            initial: separated_points(4_096),
            insertion: 7_680..=8_704,
            repetitions: 500,
        },
        SetInsertBenchmarkCase {
            name: "absorb_r4096_k3073",
            initial: separated_points(4_096),
            insertion: 2_048..=14_336,
            repetitions: 100,
        },
    ]
}

#[cfg(all(feature = "cursor_nightly_experimental", not(target_arch = "wasm32")))]
#[test]
#[ignore = "run in release mode to compare private baseline and cursor insertion directly"]
fn benchmark_set_cursor_insert_direct() {
    std::println!("case\tbaseline ns\tcursor ns\tspeedup");
    let mut speedups = Vec::new();
    for case in set_insert_benchmark_cases_first()
        .into_iter()
        .chain(set_insert_benchmark_cases_second())
    {
        let baseline = time_direct_set_insert(
            &case.initial,
            &case.insertion,
            RangeSetBlaze::internal_add_baseline,
            case.repetitions,
        );
        let cursor = time_direct_set_insert(
            &case.initial,
            &case.insertion,
            RangeSetBlaze::internal_add_cursor,
            case.repetitions,
        );
        let speedup = baseline / cursor;
        speedups.push((case.name, speedup));
        std::println!("{}\t{baseline:.2}\t{cursor:.2}\t{speedup:.3}x", case.name);
    }

    let mut char_initial = RangeSetBlaze::new();
    char_initial.internal_add_baseline('\u{D7FE}'..='\u{D7FE}');
    char_initial.internal_add_baseline('\u{E001}'..='\u{E001}');
    let char_insertion = '\u{D7FF}'..='\u{E000}';
    let baseline = time_direct_set_insert(
        &char_initial,
        &char_insertion,
        RangeSetBlaze::internal_add_baseline,
        30_000,
    );
    let cursor = time_direct_set_insert(
        &char_initial,
        &char_insertion,
        RangeSetBlaze::internal_add_cursor,
        30_000,
    );
    let speedup = baseline / cursor;
    speedups.push(("char_surrogate_bridge_r2_k2", speedup));
    std::println!("char_surrogate_bridge_r2_k2\t{baseline:.2}\t{cursor:.2}\t{speedup:.3}x");

    let ingestion = (0..1_000)
        .map(|i| (i * 7)..=(i * 7 + i % 5))
        .collect::<Vec<_>>();
    let baseline = time_direct_set_ingestion(&ingestion, RangeSetBlaze::internal_add_baseline);
    let cursor = time_direct_set_ingestion(&ingestion, RangeSetBlaze::internal_add_cursor);
    let speedup = baseline / cursor;
    speedups.push(("repeated_ingestion_n1000", speedup));
    std::println!("repeated_ingestion_n1000\t{baseline:.2}\t{cursor:.2}\t{speedup:.3}x");

    let case_count = u32::try_from(speedups.len()).expect("benchmark case count fits u32");
    let geometric_mean = (speedups
        .iter()
        .map(|(_, speedup)| speedup.ln())
        .sum::<f64>()
        / f64::from(case_count))
    .exp();
    let biggest_win = speedups
        .iter()
        .max_by(|a, b| a.1.total_cmp(&b.1))
        .expect("at least one benchmark case");
    let biggest_regression = speedups
        .iter()
        .min_by(|a, b| a.1.total_cmp(&b.1))
        .expect("at least one benchmark case");
    std::println!("geometric mean\t{geometric_mean:.3}x");
    std::println!("biggest win\t{}\t{:.3}x", biggest_win.0, biggest_win.1);
    std::println!(
        "biggest regression\t{}\t{:.3}x",
        biggest_regression.0,
        biggest_regression.1
    );
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
#[allow(
    clippy::bool_assert_comparison,
    clippy::many_single_char_names,
    clippy::cognitive_complexity,
    clippy::too_many_lines
)]
fn lib_coverage_0() {
    use core::hash::Hash;

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

    let v = RangeSetBlaze::<u32>::new();
    assert!(!v.is_universal());

    let mut v = RangeSetBlaze::<u32>::new();
    v.ranges_insert(u32::MIN..=u32::MAX);
    assert!(v.is_universal());

    let mut v = RangeSetBlaze::new();
    v.remove(2);
    assert!(!v.is_universal());

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
    a.extend(once(4));
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
        format!("{i:?}"),
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
            // println!("{a:#?}");
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

        let a = once(0..=0);
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
    let a = [(1..=4, &"a"), (5..=5, &"b"), (5..=100, &"a")].into_iter();
    let a = a
        .enumerate()
        .map(|(i, range_value)| Priority::new(range_value, i));
    let a = AssumePrioritySortedStartsMap::new(a);
    let a = UnionIterMap::new(a);
    assert!(a.equal(CheckSortedDisjointMap::new([(1..=100, &"a")])));

    // * from unsorted_priority_map
    let iter = [(5..=5, &"b"), (5..=100, &"a"), (1..=4, &"a")].into_iter();
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
    let iter = [(5, &"b"), (5, &"a"), (1, &"a")]
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
    let a = [(1, &"a"), (5, &"b"), (5, &"a")].into_iter();
    let a = a
        .enumerate()
        .map(|(i, (k, v))| Priority::new((k..=k, v), i));
    let a = AssumePrioritySortedStartsMap::new(a);
    let a = UnionIterMap::new(a);
    // is_sorted_disjoint_map::<_, _, _, _>(a);
    assert!(a.equal(CheckSortedDisjointMap::new([(1..=1, &"a"), (5..=5, &"a")])));

    // * from unsorted_priority_map
    let iter = [(5, &"b"), (5, &"a"), (1, &"a")].into_iter();
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
    let iter = [(5..=5, &"b"), (5..=100, &"a"), (1..=4, &"a")].into_iter();
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

#[cfg(feature = "from_slice")]
#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
fn test_is_consecutive() {
    use crate::from_slice::SimdInteger;
    use core::array;
    use std::simd::Simd;

    let simd: Simd<i8, 64> = Simd::from_array(array::from_fn(|i| {
        i8::try_from(10 + i).expect("i in 0..64 so 10+i fits in i8")
    }));
    assert!(i8::is_consecutive(simd));
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn test_extract_range() {
    use std::ops::Bound::{Excluded, Included};

    assert_eq!(extract_range((Excluded(0), Included(1))), (1, 1));
    assert_eq!(extract_range(0..1), (0, 0));
}
