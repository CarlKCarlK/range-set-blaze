// https://bheisler.github.io/criterion.rs/book/getting_started.html
// https://www.notamonadtutorial.com/benchmarking-and-analyzing-rust-performance-with-criterion-and-iai/#:~:text=It%20was%20written%20by%20the%20master%20of%20all,process%2C%20including%20calls%20from%20the%20Rust%20standard%20library.
// https://www.jibbow.com/posts/criterion-flamegraphs/
// https://github.com/orlp/glidesort
// https://nnethercote.github.io/perf-book/profiling.html
// cmk rule: When benchmarking, don't fight criterion.  It's smarter than you are. Make 'em fast.

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
use range_set_int::{
    multiway_intersection, multiway_union, DynSortedDisjointExt, Integer, RangeSetInt,
};
use syntactic_for::syntactic_for;
use tests_common::{k_sets, width_to_range, How, MemorylessIter, MemorylessRange};
// use thousands::Separable;

// fn insert10(c: &mut Criterion) {
//     let array = black_box([1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);
//     c.bench_function("test7", |b| {
//         b.iter(|| {
//             let _x = RangeSetInt::<u8>::from(array);
//             // x = x & x;
//         })
//     });
// }

// fn small_random_inserts(c: &mut Criterion) {
//     c.bench_function("small_random_inserts", |b| b.iter(test7));
// }

// fn big_random_inserts(c: &mut Criterion) {
//     c.bench_function("big_random_inserts", |b| b.iter(test7big));
// }

// fn test7() {
//     let mut range_set = RangeSetInt::<u64>::new();
//     // let mut index = 0u64;
//     #[allow(clippy::explicit_counter_loop)]
//     for value in RandomData::new(
//         0,
//         RangeX {
//             start: 20,
//             length: 10,
//         },
//         1,
//         //     RangeX {
//         //         start: 20,
//         //         length: 1_300_300_010,
//         //     },
//         //     100_000,
//     ) {
//         // if index % 10_000_000 == 0 {
//         //     println!(
//         //         "index {}, range_count {}",
//         //         index.separate_with_commas(),
//         //         range_set.items.len().separate_with_commas()
//         //     );
//         // }
//         // index += 1;
//         range_set.insert(value);
//         // println!("{value}->{range_set}");
//     }
//     // println!("{:?}", range_set._items);
// }

// fn test7big() {
//     let mut range_set = RangeSetInt::<u64>::new();
//     #[allow(clippy::explicit_counter_loop)]
//     for value in RandomData::new(
//         0,
//         RangeX {
//             start: 20,
//             length: 1_300_300_010,
//         },
//         100_000,
//     ) {
//         range_set.insert(value);
//     }
// }

// #[derive(Debug)]
// struct RangeX {
//     start: u64,
//     length: u64,
// }

// // impl RangeX {
// //     fn end(&self) -> u128 {
// //         self.start + self.length
// //     }
// // }

// struct RandomData {
//     rng: StdRng,
//     current: Option<RangeX>,
//     data_range: Vec<RangeX>,
//     small_enough: u64,
// }

// impl RandomData {
//     fn new(seed: u64, range: RangeX, small_enough: u64) -> Self {
//         Self {
//             rng: StdRng::seed_from_u64(seed),
//             current: None,
//             data_range: vec![range],
//             small_enough,
//         }
//     }
// }

// impl Iterator for RandomData {
//     type Item = u64;
//     fn next(&mut self) -> Option<Self::Item> {
//         if let Some(current) = &mut self.current {
//             let value = current.start;
//             self.current = if current.length > 1 {
//                 Some(RangeX {
//                     start: current.start + 1,
//                     length: current.length - 1,
//                 })
//             } else {
//                 None
//             };
//             Some(value)
//         } else if self.data_range.is_empty() {
//             None
//         } else {
//             let range = self.data_range.pop().unwrap();
//             if range.length <= self.small_enough {
//                 self.current = Some(range);
//                 self.next()
//             } else {
//                 let split = 5;
//                 let delete_fraction = 0.1;
//                 let dup_fraction = 0.01;
//                 let part_list =
//                     _process_this_level(split, range, &mut self.rng, delete_fraction, dup_fraction);
//                 self.data_range.splice(0..0, part_list);
//                 self.next()
//             }
//         }
//     }
// }

// fn _process_this_level(
//     split: u64,
//     range: RangeX,
//     rng: &mut StdRng,
//     delete_fraction: f64,
//     dup_fraction: f64,
// ) -> Vec<RangeX> {
//     let mut part_list = Vec::<RangeX>::new();
//     for i in 0..split {
//         let start = i * range.length / split + range.start;
//         let end = (i + 1) * range.length / split + range.start;

//         if rng.gen::<f64>() < delete_fraction {
//             continue;
//         }

//         part_list.push(RangeX {
//             start,
//             length: end - start,
//         });

//         if rng.gen::<f64>() < dup_fraction {
//             part_list.push(RangeX {
//                 start,
//                 length: end - start,
//             });
//         }
//     }
//     // shuffle the list
//     part_list.shuffle(rng);
//     part_list
// }

// fn shuffled(c: &mut Criterion) {
//     c.bench_function("shuffled", |b| b.iter(shuffled_test))
//         .sample_size(10);
// }

pub fn shuffled(c: &mut Criterion) {
    let seed = 0;
    let len = 2u32.pow(23); // 25 cmk
    let mut group = c.benchmark_group("shuffled");
    group.sample_size(10);
    // group.measurement_time(Duration::from_secs(170));
    group.bench_function("shuffled RangeSetInt", |b| {
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
    let len = 2u32.pow(20); // 25 cmk
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
    let len = 2u32.pow(20); // 25 cmk
    let mut group = c.benchmark_group("descending");
    group.sample_size(10);
    // group.measurement_time(Duration::from_secs(170));
    group.bench_function("descending range_set_int", |b| {
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
    let range_set_int = RangeSetInt::<u32>::from(data.as_slice());
    assert!(range_set_int.ranges_len() == range_len && range_set_int.len() == len);
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
    group.bench_function("clumps range_set_int", |b| {
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
            RangeSetInt::<u64>::from_iter,
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
    group.bench_function("RangeSetInt bitxor", |b| {
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
    group.bench_function("RangeSetInt bitor", |b| {
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
    group.bench_function("RangeSetInt bitor1", |b| {
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
) -> (RangeSetInt<T>, RangeSetInt<T>) {
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
) -> (RangeSetInt<T>, RangeSetInt<T>) {
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
        [*range.start()].into(),
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

// cmk rule use benchmarking -- your random data is important -- automate graphs
fn k_intersect(c: &mut Criterion) {
    let k = 100;
    let range = 0..=9_999_999;
    let range_len = 1_000;
    let coverage_goal = 0.99;
    let mut group = c.benchmark_group("k_intersect");
    let how = How::Union;
    group.sample_size(10);
    // group.measurement_time(Duration::from_secs(170));
    group.bench_function("RangeSetInt intersect", |b| {
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
                let _answer = RangeSetInt::multiway_intersection(sets.iter());
            },
            BatchSize::SmallInput,
        );
    });
    group.bench_function("RangeSetInt dyn intersect", |b| {
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
                let sets = sets.iter().map(|x| x.ranges().dyn_sorted_disjoint());
                let _answer: RangeSetInt<_> = multiway_intersection(sets).into();
            },
            BatchSize::SmallInput,
        );
    });
    group.bench_function("RangeSetInt intersect 2-at-a-time", |b| {
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
                // !!!cmk need code for size zero
                let mut answer = sets[0].clone();
                for set in sets.iter().skip(1) {
                    answer = answer & set;
                }
            },
            BatchSize::SmallInput,
        );
    });
    // group.bench_function("BTreeSet intersect 2-at-a-time", |b| {
    //     b.iter_batched(
    //         || btree_k_sets(k, range_len, len, coverage_goal),
    //         |sets| {
    //             // !!!cmk need code for size zero
    //             let mut answer = sets[0].clone();
    //             for set in sets.iter().skip(1) {
    //                 answer = &answer & set;
    //             }
    //         },
    //         BatchSize::SmallInput,
    //     );
    // });
}

fn coverage_goal(c: &mut Criterion) {
    let k = 10; // 100;
    let range = 0..=999_999; // 0..=99_999_999;
    let range_len = 100; //1_000;
    let how = How::Union;

    let mut group = c.benchmark_group("coverage_goal");
    for coverage_goal in [0.1, 0.5, 0.75, 0.90, 0.95, 0.99].iter() {
        // group.throughput(Throughput::Bytes(*size as u64));
        group.bench_with_input(
            BenchmarkId::new("dyn", coverage_goal),
            coverage_goal,
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
                        let sets = sets.iter().map(|x| x.ranges().dyn_sorted_disjoint());
                        let _answer: RangeSetInt<_> = multiway_intersection(sets).into();
                    },
                    BatchSize::SmallInput,
                );
            },
        );
        group.bench_with_input(
            BenchmarkId::new("static", coverage_goal),
            coverage_goal,
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
                        let _answer = RangeSetInt::multiway_intersection(sets.iter());
                    },
                    BatchSize::SmallInput,
                );
            },
        );
        group.bench_with_input(
            BenchmarkId::new("two-at-a-time", coverage_goal),
            coverage_goal,
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
                        // !!!cmk need code for size zero
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
fn intersection_vary_k_w_2_at_a_time(c: &mut Criterion) {
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
    let range = 0..=99_999;
    let coverage_goal = 0.75;
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
        // group.throughput(Throughput::Bytes(*size as u64));
        group.bench_with_input(
            BenchmarkId::new("dyn", parameter),
            &parameter,
            |b, _k_and_range_len| {
                b.iter_batched(
                    || setup,
                    |sets| {
                        let sets = sets.iter().map(|x| x.ranges().dyn_sorted_disjoint());
                        let _answer: RangeSetInt<_> = match how {
                            How::Intersection => multiway_intersection(sets).into(),
                            How::Union => multiway_union(sets).into(),
                            How::None => panic!("should not happen"),
                        };
                    },
                    BatchSize::SmallInput,
                );
            },
        );
        group.bench_with_input(
            BenchmarkId::new("static", parameter),
            &parameter,
            |b, _k| {
                b.iter_batched(
                    || setup,
                    |sets| {
                        let _answer = match how {
                            How::Intersection => RangeSetInt::multiway_intersection(sets.iter()),
                            How::Union => RangeSetInt::multiway_union(sets),
                            How::None => panic!("should not happen"),
                        };
                    },
                    BatchSize::SmallInput,
                );
            },
        );
        if include_two_at_a_time {
            group.bench_with_input(
                BenchmarkId::new("2-at-a-time", parameter),
                &parameter,
                |b, _k| {
                    b.iter_batched(
                        || setup,
                        |sets| {
                            // !!!cmk need code for size zero
                            let mut answer = sets[0].clone();
                            match how {
                                How::Intersection => {
                                    for set in sets.iter().skip(1) {
                                        answer = answer & set;
                                    }
                                }
                                How::Union => {
                                    for set in sets.iter().skip(1) {
                                        answer = answer | set;
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
    let coverage_goal = 0.50;
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
    let range_len_list1 = [1, 5, 10, 100, 1000, 10_000, 100_000];
    let coverage_goal = 0.5;
    let how = How::None;
    let seed = 0;

    let mut group = c.benchmark_group(group_name);
    group.plot_config(PlotConfiguration::default().summary_scale(AxisScale::Logarithmic));
    let mut rng = StdRng::seed_from_u64(seed);

    let set0 = &k_sets(1, range_len0, &range, coverage_goal, how, &mut rng)[0];

    for range_len1 in &range_len_list1 {
        let parameter = range_len1;

        let set1 = &k_sets(1, *range_len1, &range, coverage_goal, how, &mut rng)[0];
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

fn vs_btree_set(c: &mut Criterion) {
    let group_name = "vs_btree_set";
    let k = 1;
    let average_width_list = [1, 10, 100, 1000, 10_000, 100_000, 1_000_000];
    let coverage_goal = 0.10;
    let how = How::None;
    let seed = 0;
    let iter_len = 1_000_000;

    let mut group = c.benchmark_group(group_name);
    group.plot_config(PlotConfiguration::default().summary_scale(AxisScale::Logarithmic));

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
            BenchmarkId::new("RangeSetInt (integers)", parameter),
            &parameter,
            |b, _| {
                b.iter(|| {
                    let _answer = RangeSetInt::from_iter(vec.iter().cloned());
                })
            },
        );
        group.bench_with_input(
            BenchmarkId::new("RangeSetInt (ranges)", parameter),
            &parameter,
            |b, _| {
                b.iter(|| {
                    let _answer = RangeSetInt::from_iter(vec_range.iter().cloned());
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

fn worst(c: &mut Criterion) {
    let group_name = "worst";
    let range = 0..=1000;
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
            BenchmarkId::new("RangeSetInt", parameter),
            &parameter,
            |b, _| {
                b.iter(|| {
                    let _answer = RangeSetInt::from_iter(vec.iter().cloned());
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

criterion_group! {
    name = benches;
    config = Criterion::default();
    targets =
    shuffled,
    ascending,
    descending,
    clumps,
    bitxor,
    bitor,
    bitor1,
    k_intersect,
    coverage_goal,
    union_vary_k,
    union_vary_k_w_2_at_a_time,
    intersection_vary_k,
    intersection_vary_k_w_2_at_a_time,
    union_vary_range_len,
    intersection_vary_range_len,
    every_op,
    vary_coverage_goal,
    vary_type,
    stream_vs_adhoc,
    vs_btree_set,
    worst
}
criterion_main!(benches);

// cmk rule cargo bench intersect
