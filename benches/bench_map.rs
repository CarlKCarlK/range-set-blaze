use criterion::BatchSize;
use criterion::{
    AxisScale, BenchmarkId, Criterion, PlotConfiguration, criterion_group, criterion_main,
};
use rand::{SeedableRng, distr::Uniform, prelude::Distribution, rngs::StdRng};
use range_set_blaze::prelude::*;
use std::{
    collections::{BTreeMap, HashMap},
    ops::RangeInclusive,
};
use tests_common::{How, MemorylessMapIter, MemorylessMapRange, k_maps, width_to_range_u32};

fn map_worst(c: &mut Criterion) {
    let group_name = "map_worst";
    let uniform_key = Uniform::new(0, 1000).unwrap();
    let iter_len_list = [1u32, 10, 100, 1_000, 10_000, 100_000];
    let seed = 0;
    let n = 5u32;
    let uniform_value = Uniform::new(0, n).unwrap();

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
                    let _answer = RangeMapBlaze::from_iter(vec.iter().rev());
                })
            },
        );
        group.bench_with_input(
            BenchmarkId::new("RangeMapBlaze::extend", parameter),
            &parameter,
            |b, _| {
                b.iter(|| {
                    let mut answer: RangeMapBlaze<u32, u32> = RangeMapBlaze::new();
                    answer.extend(vec.iter().cloned());
                })
            },
        );

        group.bench_with_input(
            BenchmarkId::new("BTreeMap", parameter),
            &parameter,
            |b, _| {
                b.iter(|| {
                    let _answer = BTreeMap::from_iter(vec.iter().cloned());
                })
            },
        );
        group.bench_with_input(
            BenchmarkId::new("HashMap", parameter),
            &parameter,
            |b, _| {
                b.iter(|| {
                    let _answer: HashMap<u32, u32> = HashMap::from_iter(vec.iter().cloned());
                })
            },
        );
        group.bench_with_input(
            BenchmarkId::new("rangemap", parameter),
            &parameter,
            |b, _| {
                b.iter(|| {
                    let _answer: rangemap::RangeInclusiveMap<u32, u32> =
                        rangemap::RangeInclusiveMap::from_iter(
                            vec.iter().map(|(k, v)| (*k..=*k, *v)),
                        );
                })
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
    let n = 5u32;

    let mut group = c.benchmark_group(group_name);
    group.plot_config(PlotConfiguration::default().summary_scale(AxisScale::Logarithmic));
    // group.sample_size(40);

    for average_width in average_width_list {
        let parameter = average_width;

        let (range_len, range) = width_to_range_u32(iter_len, average_width, coverage_goal);

        let vec: Vec<(u32, u32)> = MemorylessMapIter::new(
            &mut StdRng::seed_from_u64(seed),
            range_len,
            range.clone(),
            coverage_goal,
            k,
            how,
            n,
        )
        .collect();
        let vec_range: Vec<(RangeInclusive<u32>, u32)> = MemorylessMapRange::new(
            &mut StdRng::seed_from_u64(seed),
            range_len,
            range.clone(),
            coverage_goal,
            k,
            how,
            n,
        )
        .collect();

        group.bench_with_input(
            BenchmarkId::new("RangeMapBlaze (integers)", parameter),
            &parameter,
            |b, _| {
                b.iter(|| {
                    let _answer = RangeMapBlaze::from_iter(vec.iter().rev());
                })
            },
        );

        group.bench_with_input(
            BenchmarkId::new("RangeMapBlaze (ranges)", parameter),
            &parameter,
            |b, _| {
                b.iter(|| {
                    let _answer = RangeMapBlaze::from_iter(vec_range.iter().rev());
                })
            },
        );

        group.bench_with_input(
            BenchmarkId::new("BTreeMap", parameter),
            &parameter,
            |b, _| {
                b.iter(|| {
                    let _answer = BTreeMap::from_iter(vec.iter().cloned());
                })
            },
        );
        group.bench_with_input(
            BenchmarkId::new("HashMap", parameter),
            &parameter,
            |b, _| {
                b.iter(|| {
                    let _answer: HashMap<u32, u32> = HashMap::from_iter(vec.iter().cloned());
                })
            },
        );

        group.bench_with_input(
            BenchmarkId::new("rangemap (range)", parameter),
            &parameter,
            |b, _| {
                b.iter(|| {
                    let _answer: rangemap::RangeInclusiveMap<u32, u32> =
                        rangemap::RangeInclusiveMap::from_iter(vec_range.iter().cloned());
                })
            },
        );

        group.bench_with_input(
            BenchmarkId::new("rangemap (integers)", parameter),
            &parameter,
            |b, _| {
                b.iter(|| {
                    let _answer: rangemap::RangeInclusiveMap<u32, u32> =
                        rangemap::RangeInclusiveMap::from_iter(
                            vec.iter().map(|(k, v)| (*k..=*k, *v)),
                        );
                })
            },
        );
    }
    group.finish();
}

fn map_ingest_clumps_ranges(c: &mut Criterion) {
    let group_name = "map_ingest_clumps_ranges";
    let k = 1;
    let average_width = 1000;
    let coverage_goal_list = [0.10, 0.25, 0.50, 0.75, 0.90];
    let how = How::None;
    let seed = 0;
    let iter_len = 1_000_000;
    let n = 5u32;

    let mut group = c.benchmark_group(group_name);
    group.plot_config(PlotConfiguration::default().summary_scale(AxisScale::Logarithmic));
    // group.sample_size(40);

    for coverage_goal in coverage_goal_list {
        let parameter = coverage_goal;

        let (range_len, range) = width_to_range_u32(iter_len, average_width, coverage_goal);

        let vec: Vec<(u32, u32)> = MemorylessMapIter::new(
            &mut StdRng::seed_from_u64(seed),
            range_len,
            range.clone(),
            coverage_goal,
            k,
            how,
            n,
        )
        .collect();
        let vec_range: Vec<(RangeInclusive<u32>, u32)> = MemorylessMapRange::new(
            &mut StdRng::seed_from_u64(seed),
            range_len,
            range.clone(),
            coverage_goal,
            k,
            how,
            n,
        )
        .collect();

        group.bench_with_input(
            BenchmarkId::new("RangeMapBlaze (integers)", parameter),
            &parameter,
            |b, _| {
                b.iter(|| {
                    let _answer = RangeMapBlaze::from_iter(vec.iter().rev());
                })
            },
        );

        group.bench_with_input(
            BenchmarkId::new("RangeMapBlaze (ranges)", parameter),
            &parameter,
            |b, _| {
                b.iter(|| {
                    let _answer = RangeMapBlaze::from_iter(vec_range.iter().rev());
                })
            },
        );

        group.bench_with_input(
            BenchmarkId::new("rangemap (range)", parameter),
            &parameter,
            |b, _| {
                b.iter(|| {
                    let _answer: rangemap::RangeInclusiveMap<u32, u32> =
                        rangemap::RangeInclusiveMap::from_iter(vec_range.iter().cloned());
                })
            },
        );

        group.bench_with_input(
            BenchmarkId::new("rangemap (integers)", parameter),
            &parameter,
            |b, _| {
                b.iter(|| {
                    let _answer: rangemap::RangeInclusiveMap<u32, u32> =
                        rangemap::RangeInclusiveMap::from_iter(
                            vec.iter().map(|(k, v)| (*k..=*k, *v)),
                        );
                })
            },
        );
    }
    group.finish();
}

fn map_union_two_sets(c: &mut Criterion) {
    let group_name = "map_union_two_sets";
    let range = 0..=99_999_999u32;
    let range_len0 = 1_000;
    let range_len_list1 = [1, 10, 100, 1000, 10_000, 100_000];
    let coverage_goal_list = [0.1];
    let how = How::None;
    let seed = 0;
    let n = 5u32;

    let mut group = c.benchmark_group(group_name);
    group.plot_config(PlotConfiguration::default().summary_scale(AxisScale::Logarithmic));
    let mut rng = StdRng::seed_from_u64(seed);

    for coverage_goal in coverage_goal_list {
        let temp: Vec<RangeMapBlaze<u32, u32>> =
            k_maps(1, range_len0, &range, coverage_goal, how, &mut rng, n);
        let map0 = &temp[0];
        let rangemap_map0 = &rangemap::RangeInclusiveMap::from_iter(map0.range_values());

        for range_len1 in &range_len_list1 {
            let map1 = &k_maps(1, *range_len1, &range, coverage_goal, how, &mut rng, n)[0];
            let rangemap_map1 = rangemap::RangeInclusiveMap::from_iter(map1.range_values());

            let parameter = map1.ranges_len();

            group.bench_with_input(
                BenchmarkId::new(
                    format!("1. RangeMapBlaze (bit_or_assign owned) {coverage_goal}"),
                    parameter,
                ),
                &parameter,
                |b, _| {
                    b.iter_batched(
                        || (map0.clone(), map1.clone()),
                        |(map00, mut map10)| {
                            map10 |= map00;
                        },
                        BatchSize::SmallInput,
                    );
                },
            );

            group.bench_with_input(
                BenchmarkId::new(
                    format!("2. RangeMapBlaze (bit_or_assign borrowed) {coverage_goal}"),
                    parameter,
                ),
                &parameter,
                |b, _| {
                    b.iter_batched(
                        || map0.clone(),
                        |mut map00| {
                            map00 |= map1;
                        },
                        BatchSize::SmallInput,
                    );
                },
            );

            group.bench_with_input(
                BenchmarkId::new(
                    format!("3. RangeMapBlaze (extend) {coverage_goal}"),
                    parameter,
                ),
                &parameter,
                |b, _| {
                    b.iter_batched(
                        || map0.clone(),
                        |mut map00| {
                            map00.extend(map1.range_values().map(|(r, v)| (r.clone(), *v)));
                        },
                        BatchSize::SmallInput,
                    );
                },
            );

            group.bench_with_input(
                BenchmarkId::new(format!("4. rangemap {coverage_goal}"), parameter),
                &parameter,
                |b, _| {
                    b.iter_batched(
                        || rangemap_map0.clone(),
                        |mut map00| {
                            map00.extend(rangemap_map1.iter().map(|(r, v)| (r.clone(), *v)));
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
    let n = 5u32;
    let mut group = c.benchmark_group(group_name);
    group.plot_config(PlotConfiguration::default().summary_scale(AxisScale::Logarithmic));
    let setup_vec = range_len_list
        .iter()
        .map(|range_len| {
            (
                range_len,
                k_maps(
                    k,
                    *range_len,
                    &range,
                    coverage_goal,
                    how,
                    &mut StdRng::seed_from_u64(0),
                    n,
                ),
            )
        })
        .collect::<Vec<_>>();

    for (_range_len, setup) in &setup_vec {
        let parameter = setup[0].ranges_len();
        group.bench_with_input(BenchmarkId::new("union", parameter), &parameter, |b, _k| {
            b.iter_batched(
                || setup,
                |maps| {
                    let _answer = &maps[0] | &maps[1];
                },
                BatchSize::SmallInput,
            );
        });
        group.bench_with_input(
            BenchmarkId::new("intersection", parameter),
            &parameter,
            |b, _k| {
                b.iter_batched(
                    || setup,
                    |maps| {
                        let _answer = &maps[0] & &maps[1];
                    },
                    BatchSize::SmallInput,
                );
            },
        );
        group.bench_with_input(
            BenchmarkId::new("difference", parameter),
            &parameter,
            |b, _k| {
                b.iter_batched(
                    || setup,
                    |maps| {
                        let _answer = &maps[0] - &maps[1];
                    },
                    BatchSize::SmallInput,
                );
            },
        );
        group.bench_with_input(
            BenchmarkId::new("symmetric difference", parameter),
            &parameter,
            |b, _k| {
                b.iter_batched(
                    || setup,
                    |maps| {
                        let _answer = &maps[0] ^ &maps[1];
                    },
                    BatchSize::SmallInput,
                );
            },
        );
        group.bench_with_input(
            BenchmarkId::new("complement", parameter),
            &parameter,
            |b, _k| {
                b.iter_batched(
                    || setup,
                    |maps| {
                        let _answer = !&maps[0];
                    },
                    BatchSize::SmallInput,
                );
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
    let n = 5u32;
    let parameter = 0;

    let mut group = c.benchmark_group(group_name);
    group.plot_config(PlotConfiguration::default().summary_scale(AxisScale::Logarithmic));
    // group.sample_size(40);

    let (range_len, range) = width_to_range_u32(iter_len, average_width, coverage_goal);
    let vec_range: Vec<(RangeInclusive<u32>, u32)> = MemorylessMapRange::new(
        &mut StdRng::seed_from_u64(seed),
        range_len,
        range.clone(),
        coverage_goal,
        k,
        how,
        n,
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
        BenchmarkId::new("RangeMapBlaze (extend_cmk)", parameter),
        &parameter,
        |b, _| {
            b.iter(|| {
                let mut answer: RangeMapBlaze<u32, u32> = RangeMapBlaze::new();
                answer.extend_simple(vec_range.iter().cloned());
            })
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
    //     BenchmarkId::new("RangeMapBlaze (ranges_insert_cmk)", parameter),
    //     &parameter,
    //     |b, _| {
    //         b.iter(|| {
    //             let mut answer: RangeMapBlaze<u32, u32> = RangeMapBlaze::new();
    //             for (r, v) in vec_range.iter() {
    //                 answer.ranges_insert_cmk(r.clone(), *v);
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
            })
        },
    );

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
);

criterion_main!(benches_map);
