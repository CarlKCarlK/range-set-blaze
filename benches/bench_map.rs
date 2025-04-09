use criterion::BatchSize;
use criterion::{
    AxisScale, BenchmarkId, Criterion, PlotConfiguration, criterion_group, criterion_main,
};
use rand::{SeedableRng, rngs::StdRng};
use range_set_blaze::prelude::*;
use std::{
    collections::{BTreeMap, HashMap},
    ops::RangeInclusive,
};
use tests_common::{How, MemorylessMapIter, MemorylessMapRange, k_maps, width_to_range_u32};

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
            BenchmarkId::new("RangeMapBlaze (integers-iter)", parameter),
            &parameter,
            |b, _| {
                b.iter(|| {
                    let _answer = RangeMapBlaze::from_iter(&vec);
                })
            },
        );

        group.bench_with_input(
            BenchmarkId::new("RangeMapBlaze (ranges)", parameter),
            &parameter,
            |b, _| {
                b.iter(|| {
                    let _answer = RangeMapBlaze::from_iter(&vec_range);
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

fn map_ingest_clumps_ranges(c: &mut Criterion) {
    let group_name = "map_ingest_clumps_ranges";
    let k = 1;
    let average_width_list = [1, 10, 100, 1000, 10_00, 100_000];
    let coverage_goal = 0.10;
    let how = How::None;
    let seed = 0;
    let iter_len = 1_000_000;
    let n = 5u32;

    let mut group = c.benchmark_group(group_name);
    group.plot_config(PlotConfiguration::default().summary_scale(AxisScale::Logarithmic));
    group.sample_size(40);

    for average_width in average_width_list {
        let parameter = average_width;

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

        group.bench_with_input(
            BenchmarkId::new("rangemap (ranges-BTreeSet)", parameter),
            &parameter,
            |b, _| {
                b.iter(|| {
                    let _answer: rangemap::RangeInclusiveMap<u32, u32> =
                        rangemap::RangeInclusiveMap::from_iter(vec_range.iter().cloned());
                })
            },
        );

        group.bench_with_input(
            BenchmarkId::new("RangeMapBlaze (ranges-BTreeSet)", parameter),
            &parameter,
            |b, _| {
                b.iter(|| {
                    let _answer = RangeMapBlaze::from_iter(&vec_range);
                })
            },
        );
    }
    group.finish();
}
criterion_group!(
    name = benches_map;
    config = Criterion::default();
    targets =
    map_ingest_clumps_base,
    map_every_op_blaze,
    map_ingest_clumps_ranges,
);

criterion_main!(benches_map);
