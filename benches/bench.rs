// https://bheisler.github.io/criterion.rs/book/getting_started.html
// https://www.notamonadtutorial.com/benchmarking-and-analyzing-rust-performance-with-criterion-and-iai/#:~:text=It%20was%20written%20by%20the%20master%20of%20all,process%2C%20including%20calls%20from%20the%20Rust%20standard%20library.
// https://www.jibbow.com/posts/criterion-flamegraphs/
// https://github.com/orlp/glidesort
// https://nnethercote.github.io/perf-book/profiling.html

// compare with hi_sparse_bitset

use criterion::{
    black_box, criterion_group, criterion_main, AxisScale, BatchSize, BenchmarkId, Criterion,
    PlotConfiguration,
};
use itertools::iproduct;
use rand::{
    distributions::{uniform::SampleUniform, Uniform},
    prelude::Distribution,
    rngs::StdRng,
    seq::SliceRandom,
    Rng, SeedableRng,
};
use range_set_blaze::{prelude::*, Integer};
use roaring::RoaringBitmap;
use std::{
    collections::{BTreeSet, HashSet},
    ops::RangeInclusive,
};

use syntactic_for::syntactic_for;
use tests_common::{k_sets, width_to_range_u32, How, MemorylessIter, MemorylessRange};

#[cfg(feature = "from_slice")]
const LANES: usize = 16;
#[cfg(feature = "from_slice")]
const SIMD_SUFFIX: &str = if cfg!(target_feature = "avx512f") {
    "avx512f"
} else if cfg!(target_feature = "avx2") {
    "avx2"
} else if cfg!(target_feature = "sse2") {
    "sse2"
} else {
    "error"
};

pub fn shuffled(c: &mut Criterion) {
    let seed = 0;
    let len = 2u32.pow(23); // was 25
    let mut group = c.benchmark_group("shuffled");
    group.sample_size(10);
    // group.measurement_time(Duration::from_secs(170));
    group.bench_function("shuffled RangeSetBlaze", |b| {
        b.iter_batched(
            || gen_data_shuffled(seed, len),
            |data| range_set_test(data, 1, len as u64),
            BatchSize::SmallInput,
        );
    });
    group.bench_function("shuffled BTreeSet", |b| {
        b.iter_batched(
            || gen_data_shuffled(seed, len),
            |data| btree_set_test(data, 1, len as usize),
            BatchSize::SmallInput,
        );
    });
}

fn gen_data_shuffled(seed: u64, len: u32) -> Vec<u32> {
    let mut rng = StdRng::seed_from_u64(seed);
    let mut data: Vec<u32> = (0..len).collect();
    data.shuffle(&mut rng);
    data
}

pub fn ascending(c: &mut Criterion) {
    let seed = 0;
    let len = 2u32.pow(20); // was 25
    let mut group = c.benchmark_group("ascending");
    group.sample_size(10);
    // group.measurement_time(Duration::from_secs(170));
    group.bench_function("ascending", |b| {
        b.iter_batched(
            || gen_data_ascending(seed, len),
            |data| range_set_test(data, 1, len as u64),
            BatchSize::SmallInput,
        );
    });
}

pub fn descending(c: &mut Criterion) {
    let seed = 0;
    let len = 2u32.pow(20); // was 25
    let mut group = c.benchmark_group("descending");
    group.sample_size(10);
    // group.measurement_time(Duration::from_secs(170));
    group.bench_function("descending range_set_blaze", |b| {
        b.iter_batched(
            || gen_data_descending(seed, len),
            |data| range_set_test(data, 1, len as u64),
            BatchSize::SmallInput,
        );
    });
    group.bench_function("descending btree_set", |b| {
        b.iter_batched(
            || gen_data_descending(seed, len),
            |data| btree_set_test(data, 1, len as usize),
            BatchSize::SmallInput,
        );
    });
}

fn gen_data_ascending(_seed: u64, len: u32) -> Vec<u32> {
    let data: Vec<u32> = (0..len).collect();
    data
}

fn gen_data_descending(_seed: u64, len: u32) -> Vec<u32> {
    let data: Vec<u32> = (0..len).rev().collect();
    data
}

fn range_set_test(data: Vec<u32>, range_len: usize, len: u64) {
    let range_set_blaze = RangeSetBlaze::<u32>::from_iter(data);
    assert!(range_set_blaze.ranges_len() == range_len && range_set_blaze.len() == len);
}

fn btree_set_test(data: Vec<u32>, _range_len: usize, len: usize) {
    let btree_set = BTreeSet::<u32>::from_iter(data);
    assert_eq!(btree_set.len(), len);
}

pub fn clumps(c: &mut Criterion) {
    let range = 0..=9_999_999;
    let coverage_goal = 0.95;
    let mut group = c.benchmark_group("clumps");
    group.sample_size(10);
    // group.measurement_time(Duration::from_secs(170));
    group.bench_function("clumps range_set_blaze", |b| {
        b.iter_batched(
            || {
                MemorylessIter::new(
                    &mut StdRng::seed_from_u64(0),
                    10_000,
                    range.clone(),
                    coverage_goal,
                    1,
                    How::Intersection,
                )
                .collect::<Vec<_>>()
            },
            RangeSetBlaze::<u64>::from_iter,
            BatchSize::SmallInput,
        );
    });
    group.bench_function("clumps btree_set", |b| {
        b.iter_batched(
            || {
                MemorylessIter::new(
                    &mut StdRng::seed_from_u64(0),
                    10_000,
                    range.clone(),
                    coverage_goal,
                    1,
                    How::Intersection,
                )
                .collect::<Vec<_>>()
            },
            BTreeSet::<u64>::from_iter,
            BatchSize::SmallInput,
        );
    });
}

#[allow(dead_code)]
fn bitxor(c: &mut Criterion) {
    let range = 0..=9_999_999;
    let range_len = 10_000;
    let coverage_goal = 0.50;
    let mut group = c.benchmark_group("operations");
    group.sample_size(10);
    // group.measurement_time(Duration::from_secs(170));
    group.bench_function("RangeSetBlaze bitxor", |b| {
        b.iter_batched(
            || two_sets(range_len, range.clone(), coverage_goal),
            |(set0, set1)| {
                let _answer = &set0 ^ &set1;
            },
            BatchSize::SmallInput,
        );
    });
    group.bench_function("BTreeSet bitxor", |b| {
        b.iter_batched(
            || btree_two_sets(range_len, range.clone(), coverage_goal),
            |(set0, set1)| {
                let _answer = &set0 ^ &set1;
            },
            BatchSize::SmallInput,
        );
    });
}

#[allow(dead_code)]
fn bitor(c: &mut Criterion) {
    let range = 0..=9_999_999;
    let range_len = 10_000;
    let coverage_goal = 0.50;
    let mut group = c.benchmark_group("operations");
    group.sample_size(10);
    // group.measurement_time(Duration::from_secs(170));
    group.bench_function("RangeSetBlaze bitor", |b| {
        b.iter_batched(
            || two_sets(range_len, range.clone(), coverage_goal),
            |(set0, set1)| {
                let _answer = &set0 | &set1;
            },
            BatchSize::SmallInput,
        );
    });
    group.bench_function("BTreeSet bitor", |b| {
        b.iter_batched(
            || btree_two_sets(range_len, range.clone(), coverage_goal),
            |(set0, set1)| {
                let _answer = &set0 | &set1;
            },
            BatchSize::SmallInput,
        );
    });
}

#[allow(dead_code)]
fn bitor1(c: &mut Criterion) {
    let range = 0..=9_999_999;
    let range_len = 10_000usize;
    let coverage_goal = 0.50;
    let mut group = c.benchmark_group("operations");
    group.sample_size(10);
    // group.measurement_time(Duration::from_secs(170));
    group.bench_function("RangeSetBlaze bitor1", |b| {
        b.iter_batched(
            || two_sets1(range_len, range.clone(), coverage_goal),
            |(set0, set1)| {
                let _answer = &set0 | &set1;
            },
            BatchSize::SmallInput,
        );
    });
    group.bench_function("BTreeSet bitor1", |b| {
        b.iter_batched(
            || btree_two_sets1(range_len, range.clone(), coverage_goal),
            |(set0, set1)| {
                let _answer = &set0 | &set1;
            },
            BatchSize::SmallInput,
        );
    });
}
fn two_sets<T: Integer + SampleUniform>(
    range_len: usize,
    range: RangeInclusive<T>,
    coverage_goal: f64,
) -> (RangeSetBlaze<T>, RangeSetBlaze<T>) {
    (
        MemorylessIter::new(
            &mut StdRng::seed_from_u64(0),
            range_len,
            range.clone(),
            coverage_goal,
            2,
            How::Intersection,
        )
        .collect(),
        MemorylessIter::new(
            &mut StdRng::seed_from_u64(1),
            range_len,
            range,
            coverage_goal,
            2,
            How::Intersection,
        )
        .collect(),
    )
}

#[allow(dead_code)]
fn two_sets1<T: Integer + SampleUniform>(
    range_len: usize,
    range: RangeInclusive<T>,
    coverage_goal: f64,
) -> (RangeSetBlaze<T>, RangeSetBlaze<T>) {
    (
        MemorylessRange::new(
            &mut StdRng::seed_from_u64(0),
            range_len,
            range.clone(),
            coverage_goal,
            1,
            How::Intersection,
        )
        .collect(),
        [*range.start()].into_iter().collect(),
    )
}
fn btree_two_sets<T: Integer + SampleUniform>(
    range_len: usize,
    range: RangeInclusive<T>,
    coverage_goal: f64,
) -> (BTreeSet<T>, BTreeSet<T>) {
    (
        MemorylessIter::new(
            &mut StdRng::seed_from_u64(0),
            range_len,
            range.clone(),
            coverage_goal,
            2,
            How::Intersection,
        )
        .collect(),
        MemorylessIter::new(
            &mut StdRng::seed_from_u64(1),
            range_len,
            range,
            coverage_goal,
            2,
            How::Intersection,
        )
        .collect(),
    )
}
#[allow(dead_code)]
fn btree_two_sets1<T: Integer + SampleUniform>(
    range_len: usize,
    range: RangeInclusive<T>,
    coverage_goal: f64,
) -> (BTreeSet<T>, BTreeSet<T>) {
    (
        MemorylessIter::new(
            &mut StdRng::seed_from_u64(0),
            range_len,
            range.clone(),
            coverage_goal,
            1,
            How::Intersection,
        )
        .collect(),
        BTreeSet::<T>::from([*range.start()]),
    )
}

// criterion_group! {
//     name = flame;
//     config = Criterion::default().with_profiler(PProfProfiler::new(100, Output::Flamegraph(None)));
//     targets = clumps
// }

#[allow(dead_code)]
fn k_intersect(c: &mut Criterion) {
    let k = 100;
    let range = 0..=9_999_999;
    let range_len = 1_000;
    let coverage_goal = 0.99;
    let mut group = c.benchmark_group("k_intersect");
    let how = How::Union;
    group.sample_size(10);
    // group.measurement_time(Duration::from_secs(170));
    group.bench_function("RangeSetBlaze intersect", |b| {
        b.iter_batched(
            || {
                k_sets(
                    k,
                    range_len,
                    &range,
                    coverage_goal,
                    how,
                    &mut StdRng::seed_from_u64(0),
                )
            },
            |sets| {
                let _answer = sets.intersection();
            },
            BatchSize::SmallInput,
        );
    });
    group.bench_function("RangeSetBlaze dyn intersect", |b| {
        b.iter_batched(
            || {
                k_sets(
                    k,
                    range_len,
                    &range,
                    coverage_goal,
                    how,
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
    group.bench_function("RangeSetBlaze intersect 2-at-a-time", |b| {
        b.iter_batched(
            || {
                k_sets(
                    k,
                    range_len,
                    &range,
                    coverage_goal,
                    how,
                    &mut StdRng::seed_from_u64(0),
                )
            },
            |sets| {
                // FUTURE need code for size zero
                let mut answer = sets[0].clone();
                for set in sets.iter().skip(1) {
                    answer = answer & set;
                }
            },
            BatchSize::SmallInput,
        );
    });
}

#[allow(dead_code)]
fn coverage_goal(c: &mut Criterion) {
    let k = 100;
    let range = 0..=99_999_999;
    let range_len = 1_000;
    let how = How::Intersection;
    let coverage_goal_list = [0.01, 0.25, 0.5, 0.75, 0.99];

    let mut group = c.benchmark_group("coverage_goal");
    group.plot_config(PlotConfiguration::default().summary_scale(AxisScale::Logarithmic));

    for coverage_goal in coverage_goal_list {
        // group.throughput(Throughput::Bytes(*size as u64));
        group.bench_with_input(
            BenchmarkId::new("dyn", coverage_goal),
            &coverage_goal,
            |b, &coverage_goal| {
                b.iter_batched(
                    || {
                        k_sets(
                            k,
                            range_len,
                            &range,
                            coverage_goal,
                            how,
                            &mut StdRng::seed_from_u64(0),
                        )
                    },
                    |sets| {
                        let sets = sets.iter().map(|x| DynSortedDisjoint::new(x.ranges()));
                        let _answer: RangeSetBlaze<_> = sets.intersection().into_range_set_blaze();
                    },
                    BatchSize::SmallInput,
                );
            },
        );
        group.bench_with_input(
            BenchmarkId::new("static", coverage_goal),
            &coverage_goal,
            |b, &coverage_goal| {
                b.iter_batched(
                    || {
                        k_sets(
                            k,
                            range_len,
                            &range,
                            coverage_goal,
                            how,
                            &mut StdRng::seed_from_u64(0),
                        )
                    },
                    |sets| {
                        let _answer = sets.intersection();
                    },
                    BatchSize::SmallInput,
                );
            },
        );
        group.bench_with_input(
            BenchmarkId::new("two-at-a-time", coverage_goal),
            &coverage_goal,
            |b, &coverage_goal| {
                b.iter_batched(
                    || {
                        k_sets(
                            k,
                            range_len,
                            &range,
                            coverage_goal,
                            how,
                            &mut StdRng::seed_from_u64(0),
                        )
                    },
                    |sets| {
                        // FUTURE need code for size zero
                        let mut answer = sets[0].clone();
                        for set in sets.iter().skip(1) {
                            answer = answer & set;
                        }
                    },
                    BatchSize::SmallInput,
                );
            },
        );
    }
    group.finish();
}

#[allow(dead_code)]
fn union_vary_k(c: &mut Criterion) {
    let k_list = [2usize, 5, 10, 25, 50, 100];
    let range_len_list = [1000usize];
    parameter_vary_internal(
        c,
        "union_vary_k",
        false,
        How::Union,
        &k_list,
        &range_len_list,
        access_k,
    );
}
#[allow(dead_code)]
fn union_vary_k_w_2_at_a_time(c: &mut Criterion) {
    let k_list = [2usize, 5, 10, 25, 50, 100];
    let range_len_list = [1000usize];
    parameter_vary_internal(
        c,
        "union_k_w_2_at_a_time",
        true,
        How::Union,
        &k_list,
        &range_len_list,
        access_k,
    );
}

#[allow(dead_code)]
fn intersection_vary_k(c: &mut Criterion) {
    let k_list = [2usize, 5, 10, 25, 50, 100];
    let range_len_list = [1000usize];
    parameter_vary_internal(
        c,
        "intersection_vary_k",
        false,
        How::Intersection,
        &k_list,
        &range_len_list,
        access_k,
    );
}
fn intersect_k_sets(c: &mut Criterion) {
    let k_list = [2usize, 5, 10, 25, 50, 100];
    let range_len_list = [1000usize];
    parameter_vary_internal(
        c,
        "intersect_k_sets",
        true,
        How::Intersection,
        &k_list,
        &range_len_list,
        access_k,
    );
}

#[allow(dead_code)]
fn union_vary_range_len(c: &mut Criterion) {
    let k_list = [2usize];
    let range_len_list = [1usize, 10, 100, 1000, 10_000, 100_000];
    parameter_vary_internal(
        c,
        "union_vary_range_len",
        true,
        How::Union,
        &k_list,
        &range_len_list,
        access_r,
    );
}

fn access_k(&x: &(usize, usize)) -> usize {
    x.0
}
#[allow(dead_code)]
fn access_r(&x: &(usize, usize)) -> usize {
    x.1
}
#[allow(dead_code)]
fn intersection_vary_range_len(c: &mut Criterion) {
    let k_list = [2usize];
    let range_len_list = [1usize, 10, 100, 1000, 10_000, 100_000];
    parameter_vary_internal(
        c,
        "intersection_vary_range_len",
        true,
        How::Intersection,
        &k_list,
        &range_len_list,
        access_r,
    );
}

fn parameter_vary_internal<F: Fn(&(usize, usize)) -> usize>(
    c: &mut Criterion,
    group_name: &str,
    include_two_at_a_time: bool,
    how: How,
    k_list: &[usize],
    range_len_list: &[usize],
    access: F,
) {
    let range = 0..=99_999_999;
    let coverage_goal = 0.25;
    let setup_vec = iproduct!(k_list, range_len_list)
        .map(|(k, range_len)| {
            let k = *k;
            let range_len = *range_len;
            (
                (k, range_len),
                k_sets(
                    k,
                    range_len,
                    &range,
                    coverage_goal,
                    how,
                    &mut StdRng::seed_from_u64(0),
                ),
            )
        })
        .collect::<Vec<_>>();

    let mut group = c.benchmark_group(group_name);
    group.plot_config(PlotConfiguration::default().summary_scale(AxisScale::Logarithmic));
    for (k_and_range_len, setup) in &setup_vec {
        let parameter = access(k_and_range_len);

        group.bench_with_input(
            BenchmarkId::new("RangeSetBlaze (multiway dyn)", parameter),
            &parameter,
            |b, _k_and_range_len| {
                b.iter_batched(
                    || setup,
                    |sets| {
                        let sets = sets.iter().map(|x| DynSortedDisjoint::new(x.ranges()));
                        let _answer: RangeSetBlaze<_> = match how {
                            How::Intersection => sets.intersection().into_range_set_blaze(),
                            How::Union => sets.union().into_range_set_blaze(),
                            How::None => panic!("should not happen"),
                        };
                    },
                    BatchSize::SmallInput,
                );
            },
        );
        group.bench_with_input(
            BenchmarkId::new("RangeSetBlaze (multiway static)", parameter),
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
                BenchmarkId::new("RangeSetBlaze (2-at-a-time)", parameter),
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

fn every_op_blaze(c: &mut Criterion) {
    let group_name = "every_op_blaze";
    let k = 2;
    let range_len_list = [100_000]; // cmk 1usize, 10, 100, 1000, 10_000,
    let range = 0..=99_999_999;
    let coverage_goal = 0.5;
    let how = How::None;
    let mut group = c.benchmark_group(group_name);
    group.plot_config(PlotConfiguration::default().summary_scale(AxisScale::Logarithmic));
    let setup_vec = range_len_list
        .iter()
        .map(|range_len| {
            (
                range_len,
                k_sets(
                    k,
                    *range_len,
                    &range,
                    coverage_goal,
                    how,
                    &mut StdRng::seed_from_u64(0),
                ),
            )
        })
        .collect::<Vec<_>>();

    for (_range_len, setup) in &setup_vec {
        let parameter = setup[0].ranges_len();

        // cmk
        // group.bench_with_input(BenchmarkId::new("union", parameter), &parameter, |b, _k| {
        //     b.iter_batched(
        //         || setup,
        //         |sets| {
        //             let _answer = &sets[0] | &sets[1];
        //         },
        //         BatchSize::SmallInput,
        //     );
        // });
        // group.bench_with_input(
        //     BenchmarkId::new("intersection", parameter),
        //     &parameter,
        //     |b, _k| {
        //         b.iter_batched(
        //             || setup,
        //             |sets| {
        //                 let _answer = &sets[0] & &sets[1];
        //             },
        //             BatchSize::SmallInput,
        //         );
        //     },
        // );
        // group.bench_with_input(
        //     BenchmarkId::new("difference", parameter),
        //     &parameter,
        //     |b, _k| {
        //         b.iter_batched(
        //             || setup,
        //             |sets| {
        //                 let _answer = &sets[0] - &sets[1];
        //             },
        //             BatchSize::SmallInput,
        //         );
        //     },
        // );
        group.bench_with_input(
            BenchmarkId::new("symmetric difference", parameter),
            &parameter,
            |b, _k| {
                b.iter_batched(
                    || setup,
                    |sets| {
                        let _answer = &sets[0] ^ &sets[1];
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
                    |sets| {
                        let _answer = !&sets[0];
                    },
                    BatchSize::SmallInput,
                );
            },
        );
    }
    group.finish();
}

fn every_op_roaring(c: &mut Criterion) {
    let group_name = "every_op_roaring";
    let k = 2;
    let range_len_list = [1usize, 10, 100, 1000, 10_000, 100_000];
    let range = 0..=99_999_999u32;
    let mut universe = RoaringBitmap::new();
    universe.insert_range(range.clone());
    let coverage_goal = 0.5;
    let how = How::None;
    let mut group = c.benchmark_group(group_name);
    group.plot_config(PlotConfiguration::default().summary_scale(AxisScale::Logarithmic));
    let setup_vec_0 = range_len_list
        .iter()
        .map(|range_len| {
            (
                range_len,
                k_sets(
                    k,
                    *range_len,
                    &range,
                    coverage_goal,
                    how,
                    &mut StdRng::seed_from_u64(0),
                ),
            )
        })
        .collect::<Vec<_>>();

    let mut setup_vec: Vec<(&usize, Vec<RoaringBitmap>)> = Vec::new();
    for (range_len, k_sets) in &setup_vec_0 {
        let mut range_vec_0 = Vec::new();
        for set in k_sets {
            let roaring = set.iter().collect::<RoaringBitmap>();
            range_vec_0.push(roaring);
        }
        setup_vec.push((range_len, range_vec_0));
    }

    for (i, (_range_len, setup)) in setup_vec.iter().enumerate() {
        let (_ignore, setup_0) = &setup_vec_0[i];
        let parameter = setup_0[0].ranges_len();
        group.bench_with_input(BenchmarkId::new("union", parameter), &parameter, |b, _k| {
            b.iter_batched(
                || setup,
                |sets| {
                    let _answer = &sets[0] | &sets[1];
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
                    |sets| {
                        let _answer = &sets[0] & &sets[1];
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
                    |sets| {
                        let _answer = &sets[0] - &sets[1];
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
                    |sets| {
                        let _answer = &sets[0] ^ &sets[1];
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
                    |sets| {
                        let _answer = &universe - &sets[0];
                    },
                    BatchSize::SmallInput,
                );
            },
        );
    }
    group.finish();
}

#[allow(dead_code)]
fn vary_coverage_goal(c: &mut Criterion) {
    let group_name = "vary_coverage_goal";
    let k = 2;
    let range_len = 1_000usize;
    let range = 0..=99_999_999;
    let coverage_goal_list = [0.01, 0.1, 0.25, 0.5, 0.75, 0.9, 0.99];
    let mut group = c.benchmark_group(group_name);
    let how = How::None;
    let seed = 0;
    // group.plot_config(PlotConfiguration::default().summary_scale(AxisScale::Logarithmic));
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
                    how,
                    &mut StdRng::seed_from_u64(seed),
                ),
            )
        })
        .collect::<Vec<_>>();

    for (range_len, setup) in &setup_vec {
        let parameter = *range_len;
        group.bench_with_input(BenchmarkId::new("union", parameter), &parameter, |b, _k| {
            b.iter_batched(
                || setup,
                |sets| {
                    let _answer = &sets[0] | &sets[1];
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
                    |sets| {
                        let _answer = &sets[0] & &sets[1];
                    },
                    BatchSize::SmallInput,
                );
            },
        );
    }
    group.finish();
}

#[allow(dead_code)]
fn vary_type(c: &mut Criterion) {
    let group_name = "vary_type";
    let k = 2;
    let range_len = 250;
    let coverage_goal = 0.5;
    let how = How::None;
    let seed = 0;
    let mut group = c.benchmark_group(group_name);
    // group.plot_config(PlotConfiguration::default().summary_scale(AxisScale::Logarithmic));
    syntactic_for! { ty in [u16, u32, u64, u128] {
        $(
        let range: RangeInclusive<$ty> = 0..=65535;
        let parameter = $ty::BITS;
        group.bench_with_input(BenchmarkId::new("union", parameter), &parameter, |b, _| {
            b.iter_batched(
                || k_sets(k, range_len, &range, coverage_goal, how, &mut StdRng::seed_from_u64(seed)),
                |sets| {
                    let _answer = &sets[0] | &sets[1];
                },
                BatchSize::SmallInput,
            );
        });
        )*
    }};
    group.finish();
}

fn union_two_sets(c: &mut Criterion) {
    let group_name = "union_two_sets";
    // let k = 2;
    let range = 0..=99_999_999u32;
    let range_len0 = 1_000;
    let range_len_list1 = [1, 10, 100, 1000, 10_000, 100_000];
    let coverage_goal_list = [0.1];
    let how = How::None;
    let seed = 0;

    let mut group = c.benchmark_group(group_name);
    group.plot_config(PlotConfiguration::default().summary_scale(AxisScale::Logarithmic));
    let mut rng = StdRng::seed_from_u64(seed);

    for coverage_goal in coverage_goal_list {
        let temp: Vec<RangeSetBlaze<u32>> =
            k_sets(1, range_len0, &range, coverage_goal, how, &mut rng);
        let set0 = &temp[0];
        let rangemap_set0 = &rangemap::RangeInclusiveSet::from_iter(set0.ranges());
        let roaring_set0 = RoaringBitmap::from_iter(set0.iter());

        for range_len1 in &range_len_list1 {
            let set1 = &k_sets(1, *range_len1, &range, coverage_goal, how, &mut rng)[0];
            let rangemap_set1 = rangemap::RangeInclusiveSet::from_iter(set1.ranges());
            let roaring_set1 = RoaringBitmap::from_iter(set1.iter());

            let parameter = set1.ranges_len();

            group.bench_with_input(
                BenchmarkId::new(format!("RangeSetBlaze {coverage_goal}"), parameter),
                &parameter,
                |b, _| {
                    b.iter_batched(
                        || set0.clone(),
                        |mut set00| {
                            set00 |= set1;
                        },
                        BatchSize::SmallInput,
                    );
                },
            );
            // group.bench_with_input(
            //     BenchmarkId::new(format!("Roaring insert {coverage_goal}"), parameter),
            //     &parameter,
            //     |b, _| {
            //         b.iter_batched(
            //             || roaring_set0.clone(),
            //             |mut set00| {
            //                 for x in &roaring_set1 {
            //                     set00.insert(x);
            //                 }
            //             },
            //             BatchSize::SmallInput,
            //         );
            //     },
            // );

            group.bench_with_input(
                BenchmarkId::new(format!("Roaring {coverage_goal}"), parameter),
                &parameter,
                |b, _| {
                    b.iter_batched(
                        || roaring_set0.clone(),
                        |mut set00| {
                            set00 |= &roaring_set1;
                        },
                        BatchSize::SmallInput,
                    );
                },
            );
            group.bench_with_input(
                BenchmarkId::new(format!("rangemap {coverage_goal}"), parameter),
                &parameter,
                |b, _| {
                    b.iter_batched(
                        || rangemap_set0.clone(),
                        |mut set00| {
                            set00.extend(rangemap_set1.iter().cloned());
                        },
                        BatchSize::SmallInput,
                    );
                },
            );
        }
    }
    group.finish();
}

#[allow(dead_code)]
fn str_vs_ad_by_cover(c: &mut Criterion) {
    let group_name = "str_vs_ad_by_cover";
    // let k = 2;
    let range = 0..=99_999_999;
    let range_len0 = 1_000;
    let range_len1 = 1_000;
    let coverage_goal_list = [0.01, 0.1, 0.5, 0.9, 0.99];
    let how = How::None;
    let seed = 0;

    let mut group = c.benchmark_group(group_name);
    group.plot_config(PlotConfiguration::default().summary_scale(AxisScale::Logarithmic));
    let mut rng = StdRng::seed_from_u64(seed);

    for coverage_goal in coverage_goal_list {
        let set0 = &k_sets(1, range_len0, &range, coverage_goal, how, &mut rng)[0];

        let set1 = &k_sets(1, range_len1, &range, coverage_goal, how, &mut rng)[0];
        let parameter = coverage_goal;

        group.bench_with_input(BenchmarkId::new("stream", parameter), &parameter, |b, _| {
            b.iter_batched(
                || set0,
                |set00| {
                    let _answer = set00 | set1;
                },
                BatchSize::SmallInput,
            );
        });
        group.bench_with_input(BenchmarkId::new("ad_hoc", parameter), &parameter, |b, _| {
            b.iter_batched(
                || set0.clone(),
                |mut set00| {
                    set00.extend(set1.ranges());
                },
                BatchSize::SmallInput,
            );
        });
    }
    group.finish();
}

fn ingest_clumps_base(c: &mut Criterion) {
    let group_name = "ingest_clumps_base";
    let k = 1;
    let average_width_list = [1, 10, 100, 1000, 10_000, 100_000];
    let coverage_goal = 0.10;
    let how = How::None;
    let seed = 0;
    let iter_len = 1_000_000;

    let mut group = c.benchmark_group(group_name);
    group.plot_config(PlotConfiguration::default().summary_scale(AxisScale::Logarithmic));
    // group.sample_size(40);

    for average_width in average_width_list {
        let parameter = average_width;

        let (range_len, range) = width_to_range_u32(iter_len, average_width, coverage_goal);

        let vec: Vec<u32> = MemorylessIter::new(
            &mut StdRng::seed_from_u64(seed),
            range_len,
            range.clone(),
            coverage_goal,
            k,
            how,
        )
        .collect();
        let vec_range: Vec<RangeInclusive<u32>> = MemorylessRange::new(
            &mut StdRng::seed_from_u64(seed),
            range_len,
            range.clone(),
            coverage_goal,
            k,
            how,
        )
        .collect();

        group.bench_with_input(
            BenchmarkId::new("RangeSetBlaze (integers, iter)", parameter),
            &parameter,
            |b, _| {
                b.iter(|| {
                    let _answer = RangeSetBlaze::from_iter(&vec);
                })
            },
        );

        #[cfg(feature = "from_slice")]
        group.bench_with_input(
            BenchmarkId::new(
                format!("RangeSetBlaze (integers, slice_{})", SIMD_SUFFIX),
                parameter,
            ),
            &parameter,
            |b, _| {
                b.iter(|| {
                    let _answer = RangeSetBlaze::from_slice(&vec);
                })
            },
        );

        group.bench_with_input(
            BenchmarkId::new("Roaring (integers)", parameter),
            &parameter,
            |b, _| {
                b.iter(|| {
                    let _answer = RoaringBitmap::from_iter(vec.iter());
                })
            },
        );

        group.bench_with_input(
            BenchmarkId::new("RangeSetBlaze (ranges)", parameter),
            &parameter,
            |b, _| {
                b.iter(|| {
                    let _answer = RangeSetBlaze::from_iter(&vec_range);
                })
            },
        );

        group.bench_with_input(
            BenchmarkId::new("Roaring (ranges)", parameter),
            &parameter,
            |b, _| {
                b.iter(|| {
                    let mut answer = RoaringBitmap::new();
                    for range in &vec_range {
                        answer.insert_range(range.clone());
                    }
                })
            },
        );

        group.bench_with_input(
            BenchmarkId::new("BTreeSet", parameter),
            &parameter,
            |b, _| {
                b.iter(|| {
                    let _answer = BTreeSet::from_iter(&vec);
                })
            },
        );
        group.bench_with_input(
            BenchmarkId::new("HashSet", parameter),
            &parameter,
            |b, _| {
                b.iter(|| {
                    let _answer: HashSet<u32> = HashSet::from_iter(vec.iter().cloned());
                })
            },
        );
    }
    group.finish();
}

fn ingest_clumps_integers(c: &mut Criterion) {
    let group_name = "ingest_clumps_integers";
    let k = 1;
    let average_width_list = [1, 10, 100, 1000, 10_000, 100_000];
    let coverage_goal = 0.10;
    let how = How::None;
    let seed = 0;
    let iter_len = 1_000_000;

    let mut group = c.benchmark_group(group_name);
    group.plot_config(PlotConfiguration::default().summary_scale(AxisScale::Logarithmic));
    group.sample_size(40);

    for average_width in average_width_list {
        let parameter = average_width;

        let (range_len, range) = width_to_range_u32(iter_len, average_width, coverage_goal);

        let vec: Vec<u32> = MemorylessIter::new(
            &mut StdRng::seed_from_u64(seed),
            range_len,
            range.clone(),
            coverage_goal,
            k,
            how,
        )
        .collect();

        group.bench_with_input(
            BenchmarkId::new("RangeSetBlaze (integers)", parameter),
            &parameter,
            |b, _| {
                b.iter(|| {
                    let _answer = RangeSetBlaze::from_iter(&vec);
                })
            },
        );

        #[cfg(feature = "from_slice")]
        group.bench_with_input(
            BenchmarkId::new(
                format!("RangeSetBlaze (from_slice_{})", SIMD_SUFFIX),
                parameter,
            ),
            &parameter,
            |b, _| {
                b.iter(|| {
                    let _answer = RangeSetBlaze::from_slice(&vec);
                })
            },
        );

        group.bench_with_input(
            BenchmarkId::new("Roaring (integers)", parameter),
            &parameter,
            |b, _| {
                b.iter(|| {
                    let _answer = RoaringBitmap::from_iter(vec.iter());
                })
            },
        );

        group.bench_with_input(
            BenchmarkId::new("rangemap (integers)", parameter),
            &parameter,
            |b, _| {
                b.iter(|| {
                    let _answer: rangemap::RangeInclusiveSet<u32> =
                        rangemap::RangeInclusiveSet::from_iter(vec.iter().map(|x| *x..=*x));
                })
            },
        );
        group.bench_with_input(
            BenchmarkId::new("BTreeSet", parameter),
            &parameter,
            |b, _| {
                b.iter(|| {
                    let _answer = BTreeSet::from_iter(&vec);
                })
            },
        );
        group.bench_with_input(
            BenchmarkId::new("HashSet", parameter),
            &parameter,
            |b, _| {
                b.iter(|| {
                    let _answer: HashSet<u32> = HashSet::from_iter(vec.iter().cloned());
                })
            },
        );
    }
    group.finish();
}

#[cfg(feature = "from_slice")]
fn ingest_clumps_iter_v_slice(c: &mut Criterion) {
    let group_name = "ingest_clumps_iter_v_slice";
    let k = 1;
    let average_width_list = [1, 10, 100, 1000, 10_000, 100_000];
    let coverage_goal = 0.10;
    let how = How::None;
    let seed = 0;
    let iter_len = 1_000_000;

    let mut group = c.benchmark_group(group_name);
    group.plot_config(PlotConfiguration::default().summary_scale(AxisScale::Logarithmic));
    group.sample_size(40);

    for average_width in average_width_list {
        let parameter = average_width;

        let (range_len, range) = width_to_range_u32(iter_len, average_width, coverage_goal);

        let vec: Vec<u32> = MemorylessIter::new(
            &mut StdRng::seed_from_u64(seed),
            range_len,
            range.clone(),
            coverage_goal,
            k,
            how,
        )
        .collect();

        group.bench_with_input(
            // format!("RangeSetBlaze (from_slice_{})", LANES)
            // "RangeSetBlaze (from_slice)"
            BenchmarkId::new(format!("RangeSetBlaze (from_slice_{})", LANES), parameter),
            &parameter,
            |b, _| {
                b.iter(|| {
                    black_box(RangeSetBlaze::from_slice(vec.as_slice()));
                })
            },
        );

        group.bench_with_input(
            BenchmarkId::new("RangeSetBlaze (from_iter)", parameter),
            &parameter,
            |b, _| {
                b.iter(|| {
                    black_box(RangeSetBlaze::from_iter(&vec));
                })
            },
        );
    }
    group.finish();
}

fn ingest_clumps_ranges(c: &mut Criterion) {
    let group_name = "ingest_clumps_ranges";
    let k = 1;
    let average_width_list = [1, 10, 100, 1000, 10_000, 100_000];
    let coverage_goal = 0.10;
    let how = How::None;
    let seed = 0;
    let iter_len = 1_000_000;

    let mut group = c.benchmark_group(group_name);
    group.plot_config(PlotConfiguration::default().summary_scale(AxisScale::Logarithmic));
    group.sample_size(40);

    for average_width in average_width_list {
        let parameter = average_width;

        let (range_len, range) = width_to_range_u32(iter_len, average_width, coverage_goal);

        let vec_range: Vec<RangeInclusive<u32>> = MemorylessRange::new(
            &mut StdRng::seed_from_u64(seed),
            range_len,
            range.clone(),
            coverage_goal,
            k,
            how,
        )
        .collect();

        group.bench_with_input(
            BenchmarkId::new("rangemap (ranges)", parameter),
            &parameter,
            |b, _| {
                b.iter(|| {
                    let _answer: rangemap::RangeInclusiveSet<u32> =
                        rangemap::RangeInclusiveSet::from_iter(vec_range.iter().cloned());
                })
            },
        );

        group.bench_with_input(
            BenchmarkId::new("RangeSetBlaze (ranges)", parameter),
            &parameter,
            |b, _| {
                b.iter(|| {
                    let _answer = RangeSetBlaze::from_iter(&vec_range);
                })
            },
        );

        group.bench_with_input(
            BenchmarkId::new("Roaring (ranges)", parameter),
            &parameter,
            |b, _| {
                b.iter(|| {
                    let mut answer = RoaringBitmap::new();
                    for range in vec_range.iter().cloned() {
                        answer.insert_range(range);
                    }
                })
            },
        );
    }
    group.finish();
}

fn ingest_clumps_easy(c: &mut Criterion) {
    let group_name = "ingest_clumps_easy";
    let k = 1;
    let average_width_list = [1, 10];
    let coverage_goal = 0.10;
    let how = How::None;
    let seed = 0;
    let iter_len = 100_000;

    let mut group = c.benchmark_group(group_name);
    group.plot_config(PlotConfiguration::default().summary_scale(AxisScale::Logarithmic));
    group.sample_size(40);

    for average_width in average_width_list {
        let parameter = average_width;

        let (range_len, range) = width_to_range_u32(iter_len, average_width, coverage_goal);

        let vec_range: Vec<RangeInclusive<u32>> = MemorylessRange::new(
            &mut StdRng::seed_from_u64(seed),
            range_len,
            range.clone(),
            coverage_goal,
            k,
            how,
        )
        .collect();

        group.bench_with_input(
            BenchmarkId::new("rangemap (ranges, BTreeSet)", parameter),
            &parameter,
            |b, _| {
                b.iter(|| {
                    let _answer: rangemap::RangeInclusiveSet<u32> =
                        rangemap::RangeInclusiveSet::from_iter(vec_range.iter().cloned());
                })
            },
        );

        group.bench_with_input(
            BenchmarkId::new("RangeSetBlaze (ranges, BTreeSet)", parameter),
            &parameter,
            |b, _| {
                b.iter(|| {
                    let _answer = RangeSetBlaze::from_iter(&vec_range);
                })
            },
        );

        group.bench_with_input(
            BenchmarkId::new("Roaring (ranges)", parameter),
            &parameter,
            |b, _| {
                b.iter(|| {
                    let mut answer = RoaringBitmap::new();
                    for range in vec_range.iter().cloned() {
                        answer.insert_range(range);
                    }
                })
            },
        );

        group.bench_with_input(
            BenchmarkId::new("range_collections (ranges, SmallVec)", parameter),
            &parameter,
            |b, _| {
                b.iter(|| {
                    let mut answer = range_collections::RangeSet2::from(1..1);
                    for range in &vec_range {
                        let (start, end) = range.clone().into_inner();
                        let b = range_collections::RangeSet::from(start..end + 1);
                        answer |= b;
                    }
                })
            },
        );

        group.bench_with_input(
            BenchmarkId::new("range_set (ranges, SmallVec)", parameter),
            &parameter,
            |b, _| {
                b.iter(|| {
                    let mut answer = range_set::RangeSet::<[RangeInclusive<u32>; 1]>::new();
                    for range in &vec_range {
                        answer.insert_range(range.clone());
                    }
                })
            },
        );
    }
    group.finish();
}

fn worst(c: &mut Criterion) {
    let group_name = "worst";
    let range = 0..=999u32;
    let iter_len_list = [1u32, 10, 100, 1_000, 10_000];
    let seed = 0;

    let mut group = c.benchmark_group(group_name);
    group.plot_config(PlotConfiguration::default().summary_scale(AxisScale::Logarithmic));

    for iter_len in iter_len_list {
        let parameter = iter_len;

        let mut rng = StdRng::seed_from_u64(seed);
        let uniform = Uniform::from(range.clone());
        let vec: Vec<u32> = (0..iter_len).map(|_| uniform.sample(&mut rng)).collect();

        group.bench_with_input(
            BenchmarkId::new("RangeSetBlaze (from_iter)", parameter),
            &parameter,
            |b, _| {
                b.iter(|| {
                    let _answer = RangeSetBlaze::from_iter(&vec);
                })
            },
        );

        #[cfg(feature = "from_slice")]
        group.bench_with_input(
            BenchmarkId::new(
                format!("RangeSetBlaze (from_slice_{})", SIMD_SUFFIX),
                parameter,
            ),
            &parameter,
            |b, _| {
                b.iter(|| {
                    let _answer = RangeSetBlaze::from_slice(&vec);
                })
            },
        );

        group.bench_with_input(
            BenchmarkId::new("BTreeSet", parameter),
            &parameter,
            |b, _| {
                b.iter(|| {
                    let _answer = BTreeSet::from_iter(&vec);
                })
            },
        );
        group.bench_with_input(
            BenchmarkId::new("HashSet", parameter),
            &parameter,
            |b, _| {
                b.iter(|| {
                    let _answer: HashSet<u32> = HashSet::from_iter(vec.iter().cloned());
                })
            },
        );
        group.bench_with_input(
            BenchmarkId::new("Roaring", parameter),
            &parameter,
            |b, _| {
                b.iter(|| {
                    let _answer: RoaringBitmap = RoaringBitmap::from_iter(vec.iter());
                })
            },
        );
    }
    group.finish();
}

fn overflow(c: &mut Criterion) {
    let group_name = "overflow";
    let mut group = c.benchmark_group(group_name);
    let num_iterations = 500;
    let seed = 0;
    let parameter = seed;

    group.bench_with_input(
        BenchmarkId::new("A: cast i128", parameter),
        &parameter,
        |bencher, _| {
            let mut rng = StdRng::seed_from_u64(seed);
            bencher.iter_batched(
                || gen_pair(&mut rng),
                |(a, b)| {
                    let result = (a as i128) + 1 < (b as i128);
                    criterion::black_box(result);
                },
                BatchSize::NumIterations(num_iterations),
            );
        },
    );
    group.bench_with_input(
        BenchmarkId::new("B: short circuit", parameter),
        &parameter,
        |bencher, _| {
            let mut rng = StdRng::seed_from_u64(seed);
            bencher.iter_batched(
                || gen_pair(&mut rng),
                |(a, b)| {
                    let result = a < b && a + 1 < b;
                    criterion::black_box(result);
                },
                BatchSize::NumIterations(num_iterations),
            );
        },
    );

    // group.bench_with_input(
    //     BenchmarkId::new("human_branchless", parameter),
    //     &parameter,
    //     |bencher, _| {
    //         let mut rng = StdRng::seed_from_u64(seed);
    //         bencher.iter_batched(
    //             || gen_pair(&mut rng),
    //             |(a, b)| {
    //                 let (plus_1_maybe_bad, overflow) = a.overflowing_add(1);
    //                 let result = (a < b) & !overflow & (plus_1_maybe_bad < b);
    //                 criterion::black_box(result);
    //             },
    //             BatchSize::NumIterations(num_iterations),
    //         );
    //     },
    // );

    group.bench_with_input(
        BenchmarkId::new("C: ChatGPT 4", parameter),
        &parameter,
        |bencher, _| {
            let mut rng = StdRng::seed_from_u64(seed);
            bencher.iter_batched(
                || gen_pair(&mut rng),
                |(a, b)| {
                    let result = match a.checked_add(1) {
                        Some(sum) => sum < b,
                        None => false,
                    };
                    criterion::black_box(result);
                },
                BatchSize::NumIterations(num_iterations),
            );
        },
    );

    // group.bench_with_input(
    //     BenchmarkId::new("D: ChatGPT 4 b", parameter),
    //     &parameter,
    //     |bencher, _| {
    //         let mut rng = StdRng::seed_from_u64(seed);
    //         bencher.iter_batched(
    //             || gen_pair(&mut rng),
    //             |(a, b)| {
    //                 let result = a.checked_add(1).map_or(false, |result| result < b);
    //                 criterion::black_box(result);
    //             },
    //             BatchSize::NumIterations(num_iterations),
    //         );
    //     },
    // );

    // group.bench_with_input(
    //     BenchmarkId::new("E: ChatGPT 3.5", parameter),
    //     &parameter,
    //     |bencher, _| {
    //         let mut rng = StdRng::seed_from_u64(seed);
    //         bencher.iter_batched(
    //             || gen_pair(&mut rng),
    //             |(a, b)| {
    //                 let result = if let Some(result) = a.checked_add(1) {
    //                     result < b
    //                 } else {
    //                     false
    //                 };
    //                 criterion::black_box(result);
    //             },
    //             BatchSize::NumIterations(num_iterations),
    //         );
    //     },
    // );

    group.bench_with_input(
        BenchmarkId::new("D: cast i64", parameter),
        &parameter,
        |bencher, _| {
            let mut rng = StdRng::seed_from_u64(seed);
            bencher.iter_batched(
                || gen_pair(&mut rng),
                |(a, b)| {
                    let result = (a as i64) + 1 < (b as i64);
                    criterion::black_box(result);
                },
                BatchSize::NumIterations(num_iterations),
            );
        },
    );

    group.bench_with_input(
        BenchmarkId::new("Z: Human2", parameter),
        &parameter,
        |bencher, _| {
            let mut rng = StdRng::seed_from_u64(seed);
            bencher.iter_batched(
                || gen_pair(&mut rng),
                |(a, b)| {
                    let result = a.saturating_add(1) < b;
                    criterion::black_box(result);
                },
                BatchSize::NumIterations(num_iterations),
            );
        },
    );

    //     group.bench_with_input(
    //         BenchmarkId::new("not max short circuit", parameter),
    //         &parameter,
    //         |bencher, _| {
    //             let mut rng = StdRng::seed_from_u64(seed);
    //             bencher.iter_batched(
    //                 || gen_pair(&mut rng),
    //                 |(a, b)| {
    //                     let result = a != i32::MAX && a + 1 < b;
    //                     criterion::black_box(result);
    //                 },
    //                 BatchSize::NumIterations(num_iterations),
    //             );
    //         },
    //     );

    //     group.bench_with_input(
    //         BenchmarkId::new("chat 4 map", parameter),
    //         &parameter,
    //         |bencher, _| {
    //             let mut rng = StdRng::seed_from_u64(seed);
    //             bencher.iter_batched(
    //                 || gen_pair(&mut rng),
    //                 |(a, b)| {
    //                     let result = a.checked_add(1).map_or(false, |val| val < b);
    //                     criterion::black_box(result);
    //                 },
    //                 BatchSize::NumIterations(num_iterations),
    //             );
    //         },
    //     );

    group.bench_with_input(
        BenchmarkId::new("E: i8bit to 16bit", parameter),
        &parameter,
        |bencher, _| {
            let mut rng = StdRng::seed_from_u64(seed);
            bencher.iter_batched(
                || gen_pair_i8(&mut rng),
                |(a, b)| {
                    let result = a as i16 + 1 < b as i16;
                    criterion::black_box(result);
                },
                BatchSize::NumIterations(num_iterations),
            );
        },
    );

    group.bench_with_input(
        BenchmarkId::new("F: i8bit to isize", parameter),
        &parameter,
        |bencher, _| {
            let mut rng = StdRng::seed_from_u64(seed);
            bencher.iter_batched(
                || gen_pair_i8(&mut rng),
                |(a, b)| {
                    let result = a as isize + 1 < b as isize;
                    criterion::black_box(result);
                },
                BatchSize::NumIterations(num_iterations),
            );
        },
    );

    group.bench_with_input(
        BenchmarkId::new("F: u8bit to isize", parameter),
        &parameter,
        |bencher, _| {
            let mut rng = StdRng::seed_from_u64(seed);
            bencher.iter_batched(
                || gen_pair_u8(&mut rng),
                |(a, b)| {
                    let result = a as isize + 1 < b as isize;
                    criterion::black_box(result);
                },
                BatchSize::NumIterations(num_iterations),
            );
        },
    );

    group.bench_with_input(
        BenchmarkId::new("G: u8bit to usize", parameter),
        &parameter,
        |bencher, _| {
            let mut rng = StdRng::seed_from_u64(seed);
            bencher.iter_batched(
                || gen_pair_u8(&mut rng),
                |(a, b)| {
                    let result = a as usize + 1 < b as usize;
                    criterion::black_box(result);
                },
                BatchSize::NumIterations(num_iterations),
            );
        },
    );

    group.bench_with_input(
        BenchmarkId::new("H: u8 reddit2", parameter),
        &parameter,
        |bencher, _| {
            let mut rng = StdRng::seed_from_u64(seed);
            bencher.iter_batched(
                || gen_pair_u8(&mut rng),
                |(a, b)| {
                    let result = if u8::BITS < isize::BITS {
                        (a as isize + 1) < (b as isize)
                    } else {
                        Some(a) < b.checked_sub(1)
                    };
                    criterion::black_box(result);
                },
                BatchSize::NumIterations(num_iterations),
            );
        },
    );

    group.finish();
}

fn gen_pair(rng: &mut StdRng) -> (i32, i32) {
    (
        rng.gen_range(i32::MIN..=i32::MAX),
        rng.gen_range(i32::MIN..=i32::MAX),
    )
}

fn gen_pair_i8(rng: &mut StdRng) -> (i8, i8) {
    (
        rng.gen_range(i8::MIN..=i8::MAX),
        rng.gen_range(i8::MIN..=i8::MAX),
    )
}

fn gen_pair_u8(rng: &mut StdRng) -> (u8, u8) {
    (
        rng.gen_range(u8::MIN..=u8::MAX),
        rng.gen_range(u8::MIN..=u8::MAX),
    )
}

// fn stand_alone_and(c: &mut Criterion) {
//     let top = Path::new(r"M:\projects\roaring_data");
//     let name_and_vec_vec_list = read_roaring_data(top).unwrap();

//     let op_roaring = BitAnd::bitand;
//     let op_blaze = RangeSetBlaze::bitand;

//     let mut group = c.benchmark_group("stand_alone_and");
//     // group.plot_config(PlotConfiguration::default().summary_scale(AxisScale::Logarithmic));
//     group.sample_size(40);

//     for (i, (_name, vec_vec)) in name_and_vec_vec_list.iter().enumerate() {
//         let parameter = i;

//         let pairs_roaring = &vec_vec
//             .iter()
//             .map(|v| RoaringBitmap::from_iter(v.iter()))
//             .tuple_windows::<(_, _)>()
//             .collect::<Vec<_>>();

//         let pairs_blaze = &vec_vec
//             .iter()
//             .map(|v| RangeSetBlaze::from_iter(&v)))
//             .tuple_windows::<(_, _)>()
//             .collect::<Vec<_>>();

//         group.bench_function(BenchmarkId::new("Roaring", parameter), |b| {
//             b.iter_batched(
//                 || pairs_roaring.clone(),
//                 |bitmaps| {
//                     for (a, b) in bitmaps {
//                         black_box(op_roaring(a, b));
//                     }
//                 },
//                 BatchSize::SmallInput,
//             );
//         });

//         group.bench_function(BenchmarkId::new("RangeSetBlaze", parameter), |b| {
//             b.iter_batched(
//                 || pairs_blaze.clone(),
//                 |sets| {
//                     for (a, b) in sets {
//                         black_box(op_blaze(a, b));
//                     }
//                 },
//                 BatchSize::SmallInput,
//             );
//         });
//     }
//     group.finish();
// }

fn worst_op_blaze(c: &mut Criterion) {
    let group_name = "worst_op_blaze";
    let iter_len_list = [
        1usize, 3, 10, 30, 100, 300, 1000, 3_000, 10_000, 30_000, 100_000, 300_000, 1_000_000,
    ];
    let pair_count = 20;
    let range = 0..=100_000;
    let mut group = c.benchmark_group(group_name);
    group.plot_config(PlotConfiguration::default().summary_scale(AxisScale::Logarithmic));

    let seed = 0;
    let mut rng = StdRng::seed_from_u64(seed);
    let uniform = Uniform::from(range);

    // a list of number a vec of sets
    let setup_vec_blaze = iter_len_list
        .iter()
        .map(|iter_len| {
            (
                *iter_len,
                (0..pair_count)
                    .map(|_| {
                        (0..2)
                            .map(|_| {
                                (0..*iter_len)
                                    .map(|_| uniform.sample(&mut rng))
                                    .collect::<RangeSetBlaze<_>>()
                            })
                            .collect::<Vec<_>>()
                    })
                    .collect::<Vec<_>>(),
            )
        })
        .collect::<Vec<_>>();

    let setup_vec_roaring = &setup_vec_blaze
        .iter()
        .map(|(iter_len, vec_of_set)| {
            (
                iter_len,
                vec_of_set
                    .iter()
                    .map(|list_of_sets| {
                        list_of_sets
                            .iter()
                            .map(|set| RoaringBitmap::from_iter(set.iter()))
                            .collect::<Vec<_>>()
                    })
                    .collect::<Vec<_>>(),
            )
        })
        .collect::<Vec<_>>();

    let setup_vec_btreeset = &setup_vec_blaze
        .iter()
        .map(|(iter_len, vec_of_set)| {
            (
                iter_len,
                vec_of_set
                    .iter()
                    .map(|list_of_sets| {
                        list_of_sets
                            .iter()
                            .map(BTreeSet::from_iter)
                            .collect::<Vec<_>>()
                    })
                    .collect::<Vec<_>>(),
            )
        })
        .collect::<Vec<_>>();

    let setup_vec_hashset = &setup_vec_blaze
        .iter()
        .map(|(iter_len, vec_of_set)| {
            (
                iter_len,
                vec_of_set
                    .iter()
                    .map(|list_of_sets| {
                        list_of_sets
                            .iter()
                            .map(|set| HashSet::from_iter(set.iter()))
                            .collect::<Vec<HashSet<_>>>()
                    })
                    .collect::<Vec<_>>(),
            )
        })
        .collect::<Vec<_>>();

    for (i, (range_len, setup_roaring)) in setup_vec_roaring.iter().enumerate() {
        let (_ignore, setup_blaze) = &setup_vec_blaze[i];
        let (_, setup_btreeset) = &setup_vec_btreeset[i];
        let (_, setup_hashset) = &setup_vec_hashset[i];
        let parameter = range_len;

        group.bench_with_input(
            BenchmarkId::new("RangeSetBlaze (intersection)", parameter),
            &parameter,
            |b, _k| {
                b.iter_batched(
                    || setup_blaze,
                    |sets_list| {
                        for sets in sets_list {
                            let _answer = black_box(&sets[0] & &sets[1]);
                        }
                    },
                    BatchSize::SmallInput,
                );
            },
        );

        group.bench_with_input(
            BenchmarkId::new("Roaring (intersection)", parameter),
            &parameter,
            |b, _k| {
                b.iter_batched(
                    || setup_roaring,
                    |sets_list| {
                        for sets in sets_list {
                            let _answer = black_box(&sets[0] & &sets[1]);
                        }
                    },
                    BatchSize::SmallInput,
                );
            },
        );

        group.bench_with_input(
            BenchmarkId::new("BTreeSet (intersection)", parameter),
            &parameter,
            |b, _k| {
                b.iter_batched(
                    || setup_btreeset,
                    |sets_list| {
                        for sets in sets_list {
                            let _answer = black_box(&sets[0] & &sets[1]);
                        }
                    },
                    BatchSize::SmallInput,
                );
            },
        );

        group.bench_with_input(
            BenchmarkId::new("HashSet (intersection)", parameter),
            &parameter,
            |b, _k| {
                b.iter_batched(
                    || setup_hashset,
                    |sets_list| {
                        for sets in sets_list {
                            let _answer = black_box(&sets[0] & &sets[1]);
                        }
                    },
                    BatchSize::SmallInput,
                );
            },
        );
    }

    group.finish();
}

// Define two separate criterion groups for different features
#[cfg(feature = "from_slice")]
criterion_group!(
    name = benches_with_from_slice;
    config = Criterion::default();
    targets =
    ingest_clumps_iter_v_slice,
    ingest_clumps_integers,
    ingest_clumps_base,
    worst
);

criterion_group!(
    name = benches_without_from_slice;
    config = Criterion::default();
    targets =
    intersect_k_sets,
    every_op_blaze,
    every_op_roaring,
    union_two_sets,
    ingest_clumps_base,
    worst,
    ingest_clumps_integers,
    ingest_clumps_ranges,
    ingest_clumps_easy,
    overflow,
    worst_op_blaze
);

// Conditionally select and execute the appropriate group based on the feature
#[cfg(feature = "from_slice")]
criterion_main!(benches_with_from_slice);
#[cfg(not(feature = "from_slice"))]
criterion_main!(benches_without_from_slice);
