#![allow(missing_docs)]
//! cmk000 crate docs

use criterion::BatchSize;
use criterion::{
    AxisScale, BenchmarkId, Criterion, PlotConfiguration, criterion_group, criterion_main,
};
use itertools::iproduct;
use rand::{SeedableRng, distr::Uniform, prelude::Distribution, rngs::StdRng};
use range_set_blaze::prelude::*;
use std::{
    collections::{BTreeMap, HashMap},
    ops::RangeInclusive,
};
use tests_common::{ClumpyMapIter, ClumpyMapRange, How, k_maps, width_to_range_u32};

fn map_worst(c: &mut Criterion) {
    let group_name = "map_worst";
    let uniform_key = Uniform::new(0, 1000).expect("Uniform::new failed");
    let iter_len_list = [1u32, 10, 100, 1_000, 10_000, 100_000];
    let seed = 0;
    let n = 5u32;
    let uniform_value = Uniform::new(0, n).expect("Uniform::new failed");

    let mut group = c.benchmark_group(group_name);
    group.plot_config(PlotConfiguration::default().summary_scale(AxisScale::Logarithmic));

    for iter_len in iter_len_list {
        let parameter = iter_len;

        let mut rng = StdRng::seed_from_u64(seed);
        let vec: Vec<(u32, u32)> = (0..iter_len)
            .map(|_| (uniform_key.sample(&mut rng), uniform_value.sample(&mut rng)))
            .collect();

        group.bench_with_input(
            BenchmarkId::new("RangeMapBlaze::from_iter(rev)", parameter),
            &parameter,
            |b, _| {
                b.iter(|| {
                    let _answer = vec.iter().rev().collect::<RangeMapBlaze<_, _>>();
                });
            },
        );
        group.bench_with_input(
            BenchmarkId::new("RangeMapBlaze::extend", parameter),
            &parameter,
            |b, _| {
                b.iter(|| {
                    let mut answer: RangeMapBlaze<u32, u32> = RangeMapBlaze::new();
                    answer.extend(vec.iter().copied());
                });
            },
        );

        group.bench_with_input(
            BenchmarkId::new("RangeMapBlaze::extend_simple", parameter),
            &parameter,
            |b, _| {
                b.iter(|| {
                    let mut answer: RangeMapBlaze<u32, u32> = RangeMapBlaze::new();
                    answer.extend_simple(vec.iter().map(|(k, v)| (*k..=*k, *v)));
                });
            },
        );
        group.bench_with_input(
            BenchmarkId::new("BTreeMap", parameter),
            &parameter,
            |b, _| {
                b.iter(|| {
                    let _answer = vec.iter().copied().collect::<BTreeMap<_, _>>();
                });
            },
        );
        group.bench_with_input(
            BenchmarkId::new("HashMap", parameter),
            &parameter,
            |b, _| {
                b.iter(|| {
                    let _answer: HashMap<u32, u32> = vec.iter().copied().collect::<HashMap<_, _>>();
                });
            },
        );
        group.bench_with_input(
            BenchmarkId::new("rangemap", parameter),
            &parameter,
            |b, _| {
                b.iter(|| {
                    let _answer: rangemap::RangeInclusiveMap<u32, u32> = vec
                        .iter()
                        .map(|(k, v)| (*k..=*k, *v))
                        .collect::<rangemap::RangeInclusiveMap<_, _>>();
                });
            },
        );
    }
    group.finish();
}

fn map_ingest_clumps_base(c: &mut Criterion) {
    println!("Running map_ingest_clumps_base...");
    let group_name = "map_ingest_clumps_base";
    let k = 1;
    let average_width_list = [1, 10, 100, 1000, 10_000, 100_000];
    let coverage_goal = 0.10;
    let how = How::None;
    let seed = 0;
    let iter_len = 1_000_000;
    let value_count = 5u32;
    let range_per_clump = 1;

    let mut group = c.benchmark_group(group_name);
    group.plot_config(PlotConfiguration::default().summary_scale(AxisScale::Logarithmic));
    // group.sample_size(40);

    for average_width in average_width_list {
        let parameter = average_width;

        let (clump_len, range) = width_to_range_u32(iter_len, average_width, coverage_goal);

        let vec: Vec<(u32, u32)> = ClumpyMapIter::new(
            &mut StdRng::seed_from_u64(seed),
            clump_len,
            range.clone(),
            coverage_goal,
            k,
            how,
            value_count,
            range_per_clump,
        )
        .collect();
        // let vec_range: Vec<(RangeInclusive<u32>, u32)> = ClumpyMapRange::new(
        //     &mut StdRng::seed_from_u64(seed),
        //     clump_len,
        //     range.clone(),
        //     coverage_goal,
        //     k,
        //     how,
        //     value_count,
        //     range_per_clump,
        // )
        // .collect();

        group.bench_with_input(
            BenchmarkId::new("4. RangeMapBlaze", parameter),
            &parameter,
            |b, _| {
                b.iter(|| {
                    let _answer = vec.iter().rev().collect::<RangeMapBlaze<_, _>>();
                });
            },
        );

        // group.bench_with_input(
        //     BenchmarkId::new("RangeMapBlaze (ranges)", parameter),
        //     &parameter,
        //     |b, _| {
        //         b.iter(|| {
        //             let _answer = RangeMapBlaze::from_iter(vec_range.iter().rev());
        //         })
        //     },
        // );

        group.bench_with_input(
            BenchmarkId::new("3. BTreeMap", parameter),
            &parameter,
            |b, _| {
                b.iter(|| {
                    let _answer = vec.iter().copied().collect::<BTreeMap<_, _>>();
                });
            },
        );
        group.bench_with_input(
            BenchmarkId::new("2. HashMap", parameter),
            &parameter,
            |b, _| {
                b.iter(|| {
                    let _answer: HashMap<u32, u32> = vec.iter().copied().collect();
                });
            },
        );

        // group.bench_with_input(
        //     BenchmarkId::new("rangemap (range)", parameter),
        //     &parameter,
        //     |b, _| {
        //         b.iter(|| {
        //             let _answer: rangemap::RangeInclusiveMap<u32, u32> =
        //                 rangemap::RangeInclusiveMap::from_iter(vec_range.iter().cloned());
        //         })
        //     },
        // );

        group.bench_with_input(
            BenchmarkId::new("1. rangemap", parameter),
            &parameter,
            |b, _| {
                b.iter(|| {
                    let _answer: rangemap::RangeInclusiveMap<u32, u32> = vec
                        .iter()
                        .map(|(k, v)| (*k..=*k, *v))
                        .collect::<rangemap::RangeInclusiveMap<_, _>>();
                });
            },
        );
    }
    group.finish();
}

fn map_ingest_clumps_ranges(c: &mut Criterion) {
    let group_name = "map_ingest_clumps_ranges";
    let k = 1;
    let average_width = 1000;
    let coverage_goal = 0.10;
    let how = How::None;
    let seed = 0;
    let iter_len = 1_000_000;
    let value_count = 5u32;
    let range_per_clump_list = [1, 2, 5, 10, 50];

    let mut group = c.benchmark_group(group_name);
    group.plot_config(PlotConfiguration::default().summary_scale(AxisScale::Logarithmic));

    for range_per_clump in range_per_clump_list {
        let parameter = range_per_clump;

        let (clump_len, range) = width_to_range_u32(iter_len, average_width, coverage_goal);

        let vec: Vec<(u32, u32)> = ClumpyMapIter::new(
            &mut StdRng::seed_from_u64(seed),
            clump_len,
            range.clone(),
            coverage_goal,
            k,
            how,
            value_count,
            range_per_clump,
        )
        .collect();
        let vec_range: Vec<(RangeInclusive<u32>, u32)> = ClumpyMapRange::new(
            &mut StdRng::seed_from_u64(seed),
            clump_len,
            range.clone(),
            coverage_goal,
            k,
            how,
            value_count,
            range_per_clump,
        )
        .collect();

        group.bench_with_input(
            BenchmarkId::new("2. RangeMapBlaze (integers)", parameter),
            &parameter,
            |b, _| {
                b.iter(|| {
                    let _answer = vec.iter().rev().collect::<RangeMapBlaze<_, _>>();
                });
            },
        );

        group.bench_with_input(
            BenchmarkId::new("5. RangeMapBlaze (ranges)", parameter),
            &parameter,
            |b, _| {
                b.iter(|| {
                    let _answer = vec_range.iter().rev().collect::<RangeMapBlaze<_, _>>();
                });
            },
        );

        group.bench_with_input(
            BenchmarkId::new("4. RangeMapBlaze (extend_simple)", parameter),
            &parameter,
            |b, _| {
                b.iter(|| {
                    let mut answer: RangeMapBlaze<u32, u32> = RangeMapBlaze::new();
                    answer.extend_simple(vec_range.iter().cloned());
                });
            },
        );

        group.bench_with_input(
            BenchmarkId::new("3. rangemap (range)", parameter),
            &parameter,
            |b, _| {
                b.iter(|| {
                    let _answer: rangemap::RangeInclusiveMap<u32, u32> =
                        vec_range
                            .iter()
                            .cloned()
                            .collect::<rangemap::RangeInclusiveMap<_, _>>();
                });
            },
        );

        group.bench_with_input(
            BenchmarkId::new("1. rangemap (integers)", parameter),
            &parameter,
            |b, _| {
                b.iter(|| {
                    let _answer: rangemap::RangeInclusiveMap<u32, u32> = vec
                        .iter()
                        .map(|(k, v)| (*k..=*k, *v))
                        .collect::<rangemap::RangeInclusiveMap<_, _>>();
                });
            },
        );
    }
    group.finish();
}

fn map_union_two_sets(c: &mut Criterion) {
    let group_name = "map_union_two_sets";
    let range = 0..=99_999_999u32;
    let clump_len0 = 1_000;
    let range_len_list1 = [1, 10, 100, 1000, 10_000, 100_000];
    let coverage_goal_list = [0.1];
    let how = How::None;
    let seed = 0;
    let value_count = 5u32;
    let range_per_clump = 1; // making this 1 or 100 changes nothing.

    let mut group = c.benchmark_group(group_name);
    group.plot_config(PlotConfiguration::default().summary_scale(AxisScale::Logarithmic));
    let mut rng = StdRng::seed_from_u64(seed);

    for coverage_goal in coverage_goal_list {
        let temp: Vec<RangeMapBlaze<u32, u32>> = k_maps(
            1,
            clump_len0,
            &range,
            coverage_goal,
            how,
            &mut rng,
            value_count,
            range_per_clump,
        );
        let map0 = &temp[0];
        let rangemap_map0 = &map0
            .range_values()
            .collect::<rangemap::RangeInclusiveMap<_, _>>();

        for range_len1 in &range_len_list1 {
            let map1 = &k_maps(
                1,
                *range_len1,
                &range,
                coverage_goal,
                how,
                &mut rng,
                value_count,
                range_per_clump,
            )[0];
            let rangemap_map1 = map1
                .range_values()
                .collect::<rangemap::RangeInclusiveMap<_, _>>();

            let parameter = map1.ranges_len();

            group.bench_with_input(
                BenchmarkId::new("1. RangeMapBlaze (b | a)".to_string(), parameter),
                &parameter,
                |b, _| {
                    b.iter_batched(
                        || (map0.clone(), map1.clone()),
                        |(map00, map10)| {
                            let _ = map10 | map00;
                        },
                        BatchSize::SmallInput,
                    );
                },
            );

            // group.bench_with_input(
            //     BenchmarkId::new("2. RangeMapBlaze (b |= a)".to_string(), parameter),
            //     &parameter,
            //     |b, _| {
            //         b.iter_batched(
            //             || (map0.clone(), map1.clone()),
            //             |(map00, mut map10)| {
            //                 map10 |= map00;
            //             },
            //             BatchSize::SmallInput,
            //         );
            //     },
            // );
            group.bench_with_input(
                BenchmarkId::new("4. RangeMapBlaze (b | borrow a)".to_string(), parameter),
                &parameter,
                |b, _| {
                    b.iter_batched(
                        || (map0.clone(), map1.clone()),
                        |(map00, map10)| {
                            let _ = map10 | &map00;
                        },
                        BatchSize::SmallInput,
                    );
                },
            );

            // group.bench_with_input(
            //     BenchmarkId::new("1. RangeMapBlaze (b |= &a)".to_string(), parameter),
            //     &parameter,
            //     |b, _| {
            //         b.iter_batched(
            //             || (map0.clone(), map1.clone()),
            //             |(map00, mut map10)| {
            //                 map10 |= &map00;
            //             },
            //             BatchSize::SmallInput,
            //         );
            //     },
            // );
            group.bench_with_input(
                BenchmarkId::new("2. RangeMapBlaze (extend_simple)".to_string(), parameter),
                &parameter,
                |b, _| {
                    b.iter_batched(
                        || (map0.clone(), map1.clone()),
                        |(mut map00, map10)| {
                            map00.extend_simple(map10.range_values().map(|(r, v)| (r, *v)));
                        },
                        BatchSize::SmallInput,
                    );
                },
            );
            group.bench_with_input(
                BenchmarkId::new("3. rangemap ".to_string(), parameter),
                &parameter,
                |b, _| {
                    b.iter_batched(
                        || (rangemap_map0.clone(), rangemap_map1.clone()),
                        |(mut map00, map10)| {
                            map00.extend(map10.iter().map(|(r, v)| (r.clone(), *v)));
                        },
                        BatchSize::SmallInput,
                    );
                },
            );
        }
    }
    group.finish();
}

fn map_union_left_to_right(c: &mut Criterion) {
    let group_name = "map_union_left_to_right";
    let range = 0..=99_999_999u32;
    let clump_len0 = 1_000;
    let range_len_list1 = [1, 10, 100, 1000, 10_000, 100_000];
    let coverage_goal_list = [0.1];
    let how = How::None;
    let seed = 0;
    let value_count = 5u32;
    let range_per_clump = 1; // making this 1 or 100 changes nothing.

    let mut group = c.benchmark_group(group_name);
    group.plot_config(PlotConfiguration::default().summary_scale(AxisScale::Logarithmic));
    let mut rng = StdRng::seed_from_u64(seed);

    for coverage_goal in coverage_goal_list {
        let temp: Vec<RangeMapBlaze<u32, u32>> = k_maps(
            1,
            clump_len0,
            &range,
            coverage_goal,
            how,
            &mut rng,
            value_count,
            range_per_clump,
        );
        let map0 = &temp[0];
        let rangemap_map0 = &map0
            .range_values()
            .collect::<rangemap::RangeInclusiveMap<_, _>>();

        for range_len1 in &range_len_list1 {
            let map1 = &k_maps(
                1,
                *range_len1,
                &range,
                coverage_goal,
                how,
                &mut rng,
                value_count,
                range_per_clump,
            )[0];
            let rangemap_map1 = map1
                .range_values()
                .collect::<rangemap::RangeInclusiveMap<_, _>>();

            let parameter = map1.ranges_len();

            group.bench_with_input(
                BenchmarkId::new("1. RangeMapBlaze (a | b)".to_string(), parameter),
                &parameter,
                |b, _| {
                    b.iter_batched(
                        || (map0.clone(), map1.clone()),
                        |(map00, map10)| {
                            let _ = map00 | map10;
                        },
                        BatchSize::SmallInput,
                    );
                },
            );

            // group.bench_with_input(
            //     BenchmarkId::new("2. RangeMapBlaze (a |= b)".to_string(), parameter),
            //     &parameter,
            //     |b, _| {
            //         b.iter_batched(
            //             || (map0.clone(), map1.clone()),
            //             |(mut map00, map10)| {
            //                 map00 |= map10;
            //             },
            //             BatchSize::SmallInput,
            //         );
            //     },
            // );
            group.bench_with_input(
                BenchmarkId::new("4. RangeMapBlaze (a | borrow b)".to_string(), parameter),
                &parameter,
                |b, _| {
                    b.iter_batched(
                        || (map0.clone(), map1.clone()),
                        |(map00, map10)| {
                            let _ = map00 | &map10;
                        },
                        BatchSize::SmallInput,
                    );
                },
            );

            // group.bench_with_input(
            //     BenchmarkId::new("1. RangeMapBlaze (a |= &b)".to_string(), parameter),
            //     &parameter,
            //     |b, _| {
            //         b.iter_batched(
            //             || (map0.clone(), map1.clone()),
            //             |(mut map00, map10)| {
            //                 map00 |= &map10;
            //             },
            //             BatchSize::SmallInput,
            //         );
            //     },
            // );
            group.bench_with_input(
                BenchmarkId::new("2. RangeMapBlaze (extend_simple)".to_string(), parameter),
                &parameter,
                |b, _| {
                    b.iter_batched(
                        || (map0.clone(), map1.clone()),
                        |(map00, mut map10)| {
                            map10.extend_simple(map00.range_values().map(|(r, v)| (r, *v)));
                        },
                        BatchSize::SmallInput,
                    );
                },
            );
            group.bench_with_input(
                BenchmarkId::new("3. rangemap ".to_string(), parameter),
                &parameter,
                |b, _| {
                    b.iter_batched(
                        || (rangemap_map0.clone(), rangemap_map1.clone()),
                        |(map00, mut map10)| {
                            map10.extend(map00.iter().map(|(r, v)| (r.clone(), *v)));
                        },
                        BatchSize::SmallInput,
                    );
                },
            );
        }
    }
    group.finish();
}

fn map_every_op_blaze(c: &mut Criterion) {
    let group_name = "map_every_op_blaze";
    let k = 2;
    let range_len_list = [1usize, 10, 100, 1000, 10_000, 100_000];
    let range = 0..=99_999_999;
    let coverage_goal = 0.5;
    let how = How::None;
    let value_count = 5u32;
    let range_per_clump = 1;

    let mut group = c.benchmark_group(group_name);
    group.plot_config(PlotConfiguration::default().summary_scale(AxisScale::Logarithmic));
    let setup_vec = range_len_list
        .iter()
        .map(|clump_len| {
            (
                clump_len,
                k_maps(
                    k,
                    *clump_len,
                    &range,
                    coverage_goal,
                    how,
                    &mut StdRng::seed_from_u64(0),
                    value_count,
                    range_per_clump,
                ),
            )
        })
        .collect::<Vec<_>>();

    for (_range_len, setup) in &setup_vec {
        let parameter = setup[0].ranges_len();
        group.bench_with_input(BenchmarkId::new("union", parameter), &parameter, |b, _k| {
            b.iter_batched(|| setup, |maps| &maps[0] | &maps[1], BatchSize::SmallInput);
        });
        group.bench_with_input(
            BenchmarkId::new("intersection", parameter),
            &parameter,
            |b, _k| {
                b.iter_batched(|| setup, |maps| &maps[0] & &maps[1], BatchSize::SmallInput);
            },
        );
        group.bench_with_input(
            BenchmarkId::new("difference", parameter),
            &parameter,
            |b, _k| {
                b.iter_batched(|| setup, |maps| &maps[0] - &maps[1], BatchSize::SmallInput);
            },
        );
        group.bench_with_input(
            BenchmarkId::new("symmetric difference", parameter),
            &parameter,
            |b, _k| {
                b.iter_batched(|| setup, |maps| &maps[0] ^ &maps[1], BatchSize::SmallInput);
            },
        );
        group.bench_with_input(
            BenchmarkId::new("complement", parameter),
            &parameter,
            |b, _k| {
                b.iter_batched(|| setup, |maps| !&maps[0], BatchSize::SmallInput);
            },
        );
    }
    group.finish();
}

fn map_insert_speed(c: &mut Criterion) {
    let group_name = "map_insert_speed";
    let k = 1;
    let average_width = 1000;
    let coverage_goal = 0.10;
    let how = How::None;
    let seed = 0;
    let iter_len = 1_000_000;
    let value_count = 5u32;
    let range_per_clump = 1;
    let parameter = 0;

    let mut group = c.benchmark_group(group_name);
    group.plot_config(PlotConfiguration::default().summary_scale(AxisScale::Logarithmic));
    // group.sample_size(40);

    let (clump_len, range) = width_to_range_u32(iter_len, average_width, coverage_goal);
    let vec_range: Vec<(RangeInclusive<u32>, u32)> = ClumpyMapRange::new(
        &mut StdRng::seed_from_u64(seed),
        clump_len,
        range,
        coverage_goal,
        k,
        how,
        value_count,
        range_per_clump,
    )
    .collect();

    // group.bench_with_input(
    //     BenchmarkId::new("RangeMapBlaze (extend)", parameter),
    //     &parameter,
    //     |b, _| {
    //         b.iter(|| {
    //             let mut answer: RangeMapBlaze<u32, u32> = RangeMapBlaze::new();
    //             answer.extend(vec_range.iter().cloned());
    //         })
    //     },
    // );

    group.bench_with_input(
        BenchmarkId::new("RangeMapBlaze (extend_simple)", parameter),
        &parameter,
        |b, _| {
            b.iter(|| {
                let mut answer: RangeMapBlaze<u32, u32> = RangeMapBlaze::new();
                answer.extend_simple(vec_range.iter().cloned());
            });
        },
    );
    // group.bench_with_input(
    //     BenchmarkId::new("RangeMapBlaze (ranges_insert)", parameter),
    //     &parameter,
    //     |b, _| {
    //         b.iter(|| {
    //             let mut answer: RangeMapBlaze<u32, u32> = RangeMapBlaze::new();
    //             for (r, v) in vec_range.iter() {
    //                 answer.ranges_insert(r.clone(), *v);
    //             }
    //         })
    //     },
    // );

    // group.bench_with_input(
    //     BenchmarkId::new("RangeMapBlaze (ranges_insert_x3)", parameter),
    //     &parameter,
    //     |b, _| {
    //         b.iter(|| {
    //             let mut answer: RangeMapBlaze<u32, u32> = RangeMapBlaze::new();
    //             for (r, v) in vec_range.iter() {
    //                 answer.ranges_insert_x3(r.clone(), *v);
    //             }
    //         })
    //     },
    // );
    // group.bench_with_input(
    //     BenchmarkId::new("rangemap (from_iter)", parameter),
    //     &parameter,
    //     |b, _| {
    //         b.iter(|| {
    //             let _answer: rangemap::RangeInclusiveMap<u32, u32> =
    //                 rangemap::RangeInclusiveMap::from_iter(vec_range.iter().cloned());
    //         })
    //     },
    // );

    // group.bench_with_input(
    //     BenchmarkId::new("rangemap (insert)", parameter),
    //     &parameter,
    //     |b, _| {
    //         b.iter(|| {
    //             let mut answer: rangemap::RangeInclusiveMap<u32, u32> =
    //                 rangemap::RangeInclusiveMap::new();
    //             for (r, v) in vec_range.iter() {
    //                 answer.insert(r.clone(), *v);
    //             }
    //         })
    //     },
    // );

    group.bench_with_input(
        BenchmarkId::new("rangemap (extend)", parameter),
        &parameter,
        |b, _| {
            b.iter(|| {
                let mut answer: rangemap::RangeInclusiveMap<u32, u32> =
                    rangemap::RangeInclusiveMap::new();
                answer.extend(vec_range.iter().cloned());
            });
        },
    );

    group.finish();
}

const fn access_k(&x: &(usize, usize)) -> usize {
    x.0
}
#[allow(dead_code)]
const fn access_r(&x: &(usize, usize)) -> usize {
    x.1
}
fn map_intersect_k(c: &mut Criterion) {
    let k_list = [2usize, 5, 10, 25, 50, 100];
    let clump_len_list = [1000usize];
    let value_count = 5u32;
    let range_per_clump = 1;

    parameter_vary_internal(
        c,
        "map_intersect_k",
        true,
        How::Intersection,
        &k_list,
        &clump_len_list,
        access_k,
        value_count,
        range_per_clump,
    );
}

#[allow(clippy::too_many_arguments)]
fn parameter_vary_internal<F: Fn(&(usize, usize)) -> usize>(
    c: &mut Criterion,
    group_name: &str,
    include_two_at_a_time: bool,
    how: How,
    k_list: &[usize],
    clump_len_list: &[usize],
    access: F,
    value_count: u32,
    range_per_clump: usize,
) {
    let range = 0..=99_999_999;
    let coverage_goal = 0.25;
    let setup_vec = iproduct!(k_list, clump_len_list)
        .map(|(k, clump_len)| {
            let k = *k;
            let clump = *clump_len;
            (
                (k, clump),
                k_maps(
                    k,
                    clump,
                    &range,
                    coverage_goal,
                    how,
                    &mut StdRng::seed_from_u64(0),
                    value_count,
                    range_per_clump,
                ),
            )
        })
        .collect::<Vec<_>>();

    let mut group = c.benchmark_group(group_name);
    group.plot_config(PlotConfiguration::default().summary_scale(AxisScale::Logarithmic));
    for (k_and_range_len, setup) in &setup_vec {
        let parameter = access(k_and_range_len);

        group.bench_with_input(
            BenchmarkId::new("RangeMapBlaze (multiway dyn)", parameter),
            &parameter,
            |b, _k_and_range_len| {
                b.iter_batched(
                    || setup,
                    |maps| {
                        let maps = maps
                            .iter()
                            .map(|x| DynSortedDisjointMap::new(x.range_values()));
                        let _answer: RangeMapBlaze<_, _> = match how {
                            How::Intersection => maps.intersection().into_range_map_blaze(),
                            How::Union => maps.union().into_range_map_blaze(),
                            How::None => panic!("should not happen"),
                        };
                    },
                    BatchSize::SmallInput,
                );
            },
        );
        group.bench_with_input(
            BenchmarkId::new("RangeMapBlaze (multiway static)", parameter),
            &parameter,
            |b, _k| {
                b.iter_batched(
                    || setup,
                    |sets| {
                        let _answer = match how {
                            How::Intersection => sets.intersection(),
                            How::Union => sets.union(),
                            How::None => panic!("should not happen"),
                        };
                    },
                    BatchSize::SmallInput,
                );
            },
        );
        if include_two_at_a_time {
            group.bench_with_input(
                BenchmarkId::new("RangeMapBlaze (2-at-a-time)", parameter),
                &parameter,
                |b, _k| {
                    b.iter_batched(
                        || setup,
                        |sets| {
                            // FUTURE need code for size zero
                            let mut answer = sets[0].clone();
                            match how {
                                How::Intersection => {
                                    for set in sets.iter().skip(1) {
                                        answer = answer & set;
                                    }
                                }
                                How::Union => {
                                    for set in sets.iter().skip(1) {
                                        answer |= set;
                                    }
                                }
                                How::None => panic!("should not happen"),
                            }
                        },
                        BatchSize::SmallInput,
                    );
                },
            );
        }
    }
    group.finish();
}

#[allow(clippy::too_many_lines)]
fn map_union_label(c: &mut Criterion) {
    let group_name = "map_union_label";
    let range = 0..=99_999_999u32;
    let k = 1;
    let coverage_goal = 0.10;
    let how = How::None;
    let seed = 0;
    let value_count = 5u32;
    let range_per_clump = 1;

    let a_len = 1000;
    let b_len_list = [10, 1000, 100_000];

    let mut group = c.benchmark_group(group_name);
    group.plot_config(PlotConfiguration::default().summary_scale(AxisScale::Logarithmic));
    // group.sample_size(40);

    let map0: RangeMapBlaze<u32, u32> = ClumpyMapRange::new(
        &mut StdRng::seed_from_u64(seed),
        a_len,
        range.clone(),
        coverage_goal,
        k,
        how,
        value_count,
        range_per_clump,
    )
    .collect();

    for b_len in &b_len_list {
        let map1: RangeMapBlaze<u32, u32> = ClumpyMapRange::new(
            &mut StdRng::seed_from_u64(seed + 1),
            *b_len,
            range.clone(),
            coverage_goal,
            k,
            how,
            value_count,
            range_per_clump,
        )
        .collect();

        let parameter = map1.ranges_len();

        group.bench_with_input(
            BenchmarkId::new("merge".to_string(), parameter),
            &parameter,
            |b, _| {
                b.iter_batched(
                    || (map0.clone(), map1.clone()),
                    |(map0, map1)| {
                        (map0.range_values() | map1.range_values()).into_range_map_blaze()
                    },
                    BatchSize::SmallInput,
                );
            },
        );

        group.bench_with_input(
            BenchmarkId::new("wrong_simple".to_string(), parameter),
            &parameter,
            |b, _| {
                b.iter_batched(
                    || (map0.clone(), map1.clone()),
                    |(mut map0, map1)| {
                        map0.extend_simple(map1.range_values().map(|(r, v)| (r, *v)));
                    },
                    BatchSize::SmallInput,
                );
            },
        );

        group.bench_with_input(
            BenchmarkId::new("clone_b_extend".to_string(), parameter),
            &parameter,
            |b, _| {
                b.iter_batched(
                    || (map0.clone(), map1.clone()),
                    |(map0, mut map1)| {
                        map1.extend_simple(map0.range_values().map(|(r, v)| (r, *v)));
                    },
                    BatchSize::SmallInput,
                );
            },
        );

        group.bench_with_input(
            BenchmarkId::new("b_extend".to_string(), parameter),
            &parameter,
            |b, _| {
                b.iter_batched(
                    || (map0.clone(), map1.clone()),
                    |(map0, mut map1)| {
                        map1.extend_simple(map0.range_values().map(|(r, v)| (r, *v)));
                    },
                    BatchSize::SmallInput,
                );
            },
        );

        group.bench_with_input(
            BenchmarkId::new("union via difference".to_string(), parameter),
            &parameter,
            |b, _| {
                b.iter_batched(
                    || (map0.clone(), map1.clone()),
                    |(mut map0, map1)| {
                        let difference = map1 - &map0;
                        map0.extend_simple(difference.range_values().map(|(r, v)| (r, *v)));
                    },
                    BatchSize::SmallInput,
                );
            },
        );

        group.bench_with_input(
            BenchmarkId::new("|= borrowed".to_string(), parameter),
            &parameter,
            |b, _| {
                b.iter_batched(
                    || (map0.clone(), map1.clone()),
                    |(mut map0, map1)| {
                        map0 |= &map1;
                    },
                    BatchSize::SmallInput,
                );
            },
        );

        group.bench_with_input(
            BenchmarkId::new("|= owned".to_string(), parameter),
            &parameter,
            |b, _| {
                b.iter_batched(
                    || (map0.clone(), map1.clone()),
                    |(mut map0, map1)| {
                        map0 |= map1;
                    },
                    BatchSize::SmallInput,
                );
            },
        );
    }

    group.finish();
}

criterion_group!(
    name = benches_map;
    config = Criterion::default();
    targets =
    map_worst,
    map_ingest_clumps_base,
    map_ingest_clumps_ranges,
    map_every_op_blaze,
    map_union_two_sets,
    map_insert_speed,
    map_intersect_k,
    map_union_label,
    map_union_left_to_right,
);

criterion_main!(benches_map);
