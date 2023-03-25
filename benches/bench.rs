// https://bheisler.github.io/criterion.rs/book/getting_started.html
// https://www.notamonadtutorial.com/benchmarking-and-analyzing-rust-performance-with-criterion-and-iai/#:~:text=It%20was%20written%20by%20the%20master%20of%20all,process%2C%20including%20calls%20from%20the%20Rust%20standard%20library.
// https://www.jibbow.com/posts/criterion-flamegraphs/
// https://github.com/orlp/glidesort
// https://nnethercote.github.io/perf-book/profiling.html
// todo rule: When benchmarking, don't fight criterion.  It's smarter than you are. Make 'em fast.

use std::{
    collections::{BTreeSet, HashSet},
    ops::RangeInclusive,
};

use criterion::{
    criterion_group, criterion_main, AxisScale, BatchSize, BenchmarkId, Criterion,
    PlotConfiguration,
};
use itertools::iproduct;
use rand::{
    distributions::Uniform, prelude::Distribution, rngs::StdRng, seq::SliceRandom, SeedableRng,
};
// use pprof::criterion::Output; //PProfProfiler
use range_set_blaze::{
    DynSortedDisjoint, Integer, MultiwayRangeSetInt, MultiwaySortedDisjoint, RangeSetBlaze,
};
use syntactic_for::syntactic_for;
use tests_common::{k_sets, width_to_range, How, MemorylessIter, MemorylessRange};

pub fn shuffled(c: &mut Criterion) {
    let seed = 0;
    let len = 2u32.pow(23); // was 25
    let mut group = c.benchmark_group("shuffled");
    group.sample_size(10);
    // group.measurement_time(Duration::from_secs(170));
    group.bench_function("shuffled RangeSetBlaze", |b| {
        b.iter_batched(
            || gen_data_shuffled(seed, len),
            |data| range_set_test(data, 1, len as usize),
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
            |data| range_set_test(data, 1, len as usize),
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
            |data| range_set_test(data, 1, len as usize),
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

fn range_set_test(data: Vec<u32>, range_len: usize, len: usize) {
    let range_set_blaze = RangeSetBlaze::<u32>::from_iter(data);
    assert!(range_set_blaze.ranges_len() == range_len && range_set_blaze.len() == len);
}

fn btree_set_test(data: Vec<u32>, _range_len: usize, len: usize) {
    let btree_set = BTreeSet::<u32>::from_iter(data);
    assert!(btree_set.len() == len);
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
fn two_sets<T: Integer>(
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

fn two_sets1<T: Integer>(
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
fn btree_two_sets<T: Integer>(
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
fn btree_two_sets1<T: Integer>(
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

// todo rule use benchmarking -- your random data is important -- automate graphs
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
                let _answer: RangeSetBlaze<_> = sets.intersection().into();
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
                        let _answer: RangeSetBlaze<_> = sets.intersection().into();
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
fn intersection_k_w_2_at_a_time(c: &mut Criterion) {
    let k_list = [2usize, 5, 10, 25, 50, 100];
    let range_len_list = [1000usize];
    parameter_vary_internal(
        c,
        "intersection_k_w_2_at_a_time",
        true,
        How::Intersection,
        &k_list,
        &range_len_list,
        access_k,
    );
}

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
fn access_r(&x: &(usize, usize)) -> usize {
    x.1
}
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
                            How::Intersection => sets.intersection().into(),
                            How::Union => sets.union().into(),
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

fn every_op(c: &mut Criterion) {
    let group_name = "every_op";
    let k = 2;
    let range_len_list = [1usize, 10, 100, 1000, 10_000, 100_000];
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
        group.bench_with_input(BenchmarkId::new("sub", parameter), &parameter, |b, _k| {
            b.iter_batched(
                || setup,
                |sets| {
                    let _answer = &sets[0] - &sets[1];
                },
                BatchSize::SmallInput,
            );
        });
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
            BenchmarkId::new("negation", parameter),
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

fn stream_vs_adhoc(c: &mut Criterion) {
    let group_name = "stream_vs_adhoc";
    // let k = 2;
    let range = 0..=99_999_999;
    let range_len0 = 1_000;
    let range_len_list1 = [1, 10, 100, 1000, 10_000, 100_000];
    let coverage_goal_list = [0.1];
    let how = How::None;
    let seed = 0;

    let mut group = c.benchmark_group(group_name);
    group.plot_config(PlotConfiguration::default().summary_scale(AxisScale::Logarithmic));
    let mut rng = StdRng::seed_from_u64(seed);

    for coverage_goal in coverage_goal_list {
        let set0 = &k_sets(1, range_len0, &range, coverage_goal, how, &mut rng)[0];
        let rangemap_set0 = &rangemap::RangeInclusiveSet::from_iter(set0.ranges());

        for range_len1 in &range_len_list1 {
            let set1 = &k_sets(1, *range_len1, &range, coverage_goal, how, &mut rng)[0];
            let rangemap_set1 = rangemap::RangeInclusiveSet::from_iter(set1.ranges());

            let parameter = set1.ranges_len();

            // group.bench_with_input(
            //     BenchmarkId::new(format!("RangeSetBlaze stream {coverage_goal}"), parameter),
            //     &parameter,
            //     |b, _| {
            //         b.iter_batched(
            //             || set0,
            //             |set00| {
            //                 let _answer = set00 | set1;
            //             },
            //             BatchSize::SmallInput,
            //         );
            //     },
            // );
            group.bench_with_input(
                BenchmarkId::new(format!("RangeSetBlaze (hybrid) {coverage_goal}"), parameter),
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
            //     BenchmarkId::new(format!("RangeSetBlaze ad_hoc {coverage_goal}"), parameter),
            //     &parameter,
            //     |b, _| {
            //         b.iter_batched(
            //             || set0.clone(),
            //             |mut set00| {
            //                 set00.extend(set1.ranges());
            //             },
            //             BatchSize::SmallInput,
            //         );
            //     },
            // );

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
    let average_width_list = [1, 10, 100, 1000, 10_000, 100_000, 1_000_000];
    let coverage_goal = 0.10;
    let how = How::None;
    let seed = 0;
    let iter_len = 1_000_000;

    let mut group = c.benchmark_group(group_name);
    group.plot_config(PlotConfiguration::default().summary_scale(AxisScale::Logarithmic));
    // group.sample_size(40);

    for average_width in average_width_list {
        let parameter = average_width;

        let (range_len, range) = width_to_range(iter_len, average_width, coverage_goal);

        let vec: Vec<i32> = MemorylessIter::new(
            &mut StdRng::seed_from_u64(seed),
            range_len,
            range.clone(),
            coverage_goal,
            k,
            how,
        )
        .collect();
        let vec_range: Vec<RangeInclusive<i32>> = MemorylessRange::new(
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
                    let _answer = RangeSetBlaze::from_iter(vec.iter().cloned());
                })
            },
        );
        group.bench_with_input(
            BenchmarkId::new("RangeSetBlaze (ranges)", parameter),
            &parameter,
            |b, _| {
                b.iter(|| {
                    let _answer = RangeSetBlaze::from_iter(vec_range.iter().cloned());
                })
            },
        );
        group.bench_with_input(
            BenchmarkId::new("BTreeSet", parameter),
            &parameter,
            |b, _| {
                b.iter(|| {
                    let _answer = BTreeSet::from_iter(vec.iter().cloned());
                })
            },
        );
        group.bench_with_input(
            BenchmarkId::new("HashSet", parameter),
            &parameter,
            |b, _| {
                b.iter(|| {
                    let _answer: HashSet<i32> = HashSet::from_iter(vec.iter().cloned());
                })
            },
        );
    }
    group.finish();
}

fn ingest_clumps_integers(c: &mut Criterion) {
    let group_name = "ingest_clumps_integers";
    let k = 1;
    let average_width_list = [1, 10, 100, 1000, 10_000, 100_000, 1_000_000];
    let coverage_goal = 0.10;
    let how = How::None;
    let seed = 0;
    let iter_len = 1_000_000;

    let mut group = c.benchmark_group(group_name);
    group.plot_config(PlotConfiguration::default().summary_scale(AxisScale::Logarithmic));
    group.sample_size(40);

    for average_width in average_width_list {
        let parameter = average_width;

        let (range_len, range) = width_to_range(iter_len, average_width, coverage_goal);

        let vec: Vec<i32> = MemorylessIter::new(
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
                    let _answer = RangeSetBlaze::from_iter(vec.iter().cloned());
                })
            },
        );
        group.bench_with_input(
            BenchmarkId::new("rangemap (integers)", parameter),
            &parameter,
            |b, _| {
                b.iter(|| {
                    let _answer: rangemap::RangeInclusiveSet<i32> =
                        rangemap::RangeInclusiveSet::from_iter(vec.iter().map(|x| *x..=*x));
                })
            },
        );
        group.bench_with_input(
            BenchmarkId::new("BTreeSet", parameter),
            &parameter,
            |b, _| {
                b.iter(|| {
                    let _answer = BTreeSet::from_iter(vec.iter().cloned());
                })
            },
        );
        group.bench_with_input(
            BenchmarkId::new("HashSet", parameter),
            &parameter,
            |b, _| {
                b.iter(|| {
                    let _answer: HashSet<i32> = HashSet::from_iter(vec.iter().cloned());
                })
            },
        );
    }
    group.finish();
}

fn ingest_clumps_ranges(c: &mut Criterion) {
    let group_name = "ingest_clumps_ranges";
    let k = 1;
    let average_width_list = [1, 10, 100, 1000, 10_000, 100_000, 1_000_000];
    let coverage_goal = 0.10;
    let how = How::None;
    let seed = 0;
    let iter_len = 1_000_000;

    let mut group = c.benchmark_group(group_name);
    group.plot_config(PlotConfiguration::default().summary_scale(AxisScale::Logarithmic));
    group.sample_size(40);

    for average_width in average_width_list {
        let parameter = average_width;

        let (range_len, range) = width_to_range(iter_len, average_width, coverage_goal);

        let vec_range: Vec<RangeInclusive<i32>> = MemorylessRange::new(
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
                    let _answer: rangemap::RangeInclusiveSet<i32> =
                        rangemap::RangeInclusiveSet::from_iter(vec_range.iter().cloned());
                })
            },
        );

        group.bench_with_input(
            BenchmarkId::new("RangeSetBlaze (ranges)", parameter),
            &parameter,
            |b, _| {
                b.iter(|| {
                    let _answer = RangeSetBlaze::from_iter(vec_range.iter().cloned());
                })
            },
        );
    }
    group.finish();
}
fn worst(c: &mut Criterion) {
    let group_name = "worst";
    let range = 0..=999;
    let iter_len_list = [1, 10, 100, 1_000, 10_000];
    let seed = 0;

    let mut group = c.benchmark_group(group_name);
    group.plot_config(PlotConfiguration::default().summary_scale(AxisScale::Logarithmic));

    for iter_len in iter_len_list {
        let parameter = iter_len;

        let mut rng = StdRng::seed_from_u64(seed);
        let uniform = Uniform::from(range.clone());
        let vec: Vec<i32> = (0..iter_len).map(|_| uniform.sample(&mut rng)).collect();

        group.bench_with_input(
            BenchmarkId::new("RangeSetBlaze", parameter),
            &parameter,
            |b, _| {
                b.iter(|| {
                    let _answer = RangeSetBlaze::from_iter(vec.iter().cloned());
                })
            },
        );
        group.bench_with_input(
            BenchmarkId::new("BTreeSet", parameter),
            &parameter,
            |b, _| {
                b.iter(|| {
                    let _answer = BTreeSet::from_iter(vec.iter().cloned());
                })
            },
        );

        group.bench_with_input(
            BenchmarkId::new("HashSet", parameter),
            &parameter,
            |b, _| {
                b.iter(|| {
                    let _answer: HashSet<i32> = HashSet::from_iter(vec.iter().cloned());
                })
            },
        );
    }
    group.finish();
}

// criterion_group! {
//     name = benches;
//     config = Criterion::default();
//     targets =
//     shuffled,
//     ascending,
//     descending,
//     clumps,
//     bitxor,
//     bitor,
//     bitor1,
//     k_intersect,
//     coverage_goal,
//     union_vary_k,
//     union_vary_k_w_2_at_a_time,
//     intersection_vary_k,
//     intersection_k_w_2_at_a_time,
//     union_vary_range_len,
//     intersection_vary_range_len,
//     every_op,
//     vary_coverage_goal,
//     vary_type,
//     stream_vs_adhoc,
//     str_vs_ad_by_cover,
//     ingest_clumps_base,
//     worst,
//     ingest_clumps_integers,
//     ingest_clumps_ranges,
// }

criterion_group! {
    name = benches;
    config = Criterion::default();
    targets =
    intersection_k_w_2_at_a_time,
    every_op,
    stream_vs_adhoc,
    ingest_clumps_base,
    worst,
    ingest_clumps_integers,
    ingest_clumps_ranges,
}
criterion_main!(benches);

// todo rule cargo bench intersect
