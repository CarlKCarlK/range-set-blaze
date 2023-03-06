// https://bheisler.github.io/criterion.rs/book/getting_started.html
// https://www.notamonadtutorial.com/benchmarking-and-analyzing-rust-performance-with-criterion-and-iai/#:~:text=It%20was%20written%20by%20the%20master%20of%20all,process%2C%20including%20calls%20from%20the%20Rust%20standard%20library.
// https://www.jibbow.com/posts/criterion-flamegraphs/
// https://github.com/orlp/glidesort
// https://nnethercote.github.io/perf-book/profiling.html

use std::collections::BTreeSet;

use criterion::{criterion_group, criterion_main, BatchSize, BenchmarkId, Criterion};
use rand::{rngs::StdRng, seq::SliceRandom, SeedableRng};
// use pprof::criterion::Output; //PProfProfiler
use range_set_int::{intersection, union, DynSortedDisjointExt, RangeSetInt};
use tests_common::{k_sets, MemorylessIter};
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
    let len = 10_000_000;
    let coverage_goal = 0.95;
    let mut group = c.benchmark_group("clumps");
    group.sample_size(10);
    // group.measurement_time(Duration::from_secs(170));
    group.bench_function("clumps range_set_int", |b| {
        b.iter_batched(
            || MemorylessIter::new(0, 10_000, len, coverage_goal, 1),
            RangeSetInt::<u64>::from_iter,
            BatchSize::SmallInput,
        );
    });
    group.bench_function("clumps btree_set", |b| {
        b.iter_batched(
            || MemorylessIter::new(0, 10_000, len, coverage_goal, 1),
            BTreeSet::<u64>::from_iter,
            BatchSize::SmallInput,
        );
    });
}

fn bitxor(c: &mut Criterion) {
    let len = 10_000_000;
    let range_len = 10_000;
    let coverage_goal = 0.50;
    let mut group = c.benchmark_group("operations");
    group.sample_size(10);
    // group.measurement_time(Duration::from_secs(170));
    group.bench_function("RangeSetInt bitxor", |b| {
        b.iter_batched(
            || two_sets(range_len, len, coverage_goal),
            |(set0, set1)| {
                let _answer = &set0 ^ &set1;
            },
            BatchSize::SmallInput,
        );
    });
    group.bench_function("BTreeSet bitxor", |b| {
        b.iter_batched(
            || btree_two_sets(range_len, len, coverage_goal),
            |(set0, set1)| {
                let _answer = &set0 ^ &set1;
            },
            BatchSize::SmallInput,
        );
    });
}

fn bitor(c: &mut Criterion) {
    let len = 10_000_000;
    let range_len = 10_000;
    let coverage_goal = 0.50;
    let mut group = c.benchmark_group("operations");
    group.sample_size(10);
    // group.measurement_time(Duration::from_secs(170));
    group.bench_function("RangeSetInt bitor", |b| {
        b.iter_batched(
            || two_sets(range_len, len, coverage_goal),
            |(set0, set1)| {
                let _answer = &set0 | &set1;
            },
            BatchSize::SmallInput,
        );
    });
    group.bench_function("BTreeSet bitor", |b| {
        b.iter_batched(
            || btree_two_sets(range_len, len, coverage_goal),
            |(set0, set1)| {
                let _answer = &set0 | &set1;
            },
            BatchSize::SmallInput,
        );
    });
}

fn bitor1(c: &mut Criterion) {
    let len = 10_000_000;
    let range_len = 10_000;
    let coverage_goal = 0.50;
    let mut group = c.benchmark_group("operations");
    group.sample_size(10);
    // group.measurement_time(Duration::from_secs(170));
    group.bench_function("RangeSetInt bitor1", |b| {
        b.iter_batched(
            || two_sets1(range_len, len, coverage_goal),
            |(set0, set1)| {
                let _answer = &set0 | &set1;
            },
            BatchSize::SmallInput,
        );
    });
    group.bench_function("BTreeSet bitor1", |b| {
        b.iter_batched(
            || btree_two_sets1(range_len, len, coverage_goal),
            |(set0, set1)| {
                let _answer = &set0 | &set1;
            },
            BatchSize::SmallInput,
        );
    });
}
fn two_sets(range_len: u64, len: u128, coverage_goal: f64) -> (RangeSetInt<u64>, RangeSetInt<u64>) {
    (
        MemorylessIter::new(0, range_len, len, coverage_goal, 2).collect(),
        MemorylessIter::new(1, range_len, len, coverage_goal, 2).collect(),
    )
}

fn two_sets1(
    range_len: u64,
    len: u128,
    coverage_goal: f64,
) -> (RangeSetInt<u64>, RangeSetInt<u64>) {
    (
        MemorylessIter::new(0, range_len, len, coverage_goal, 1).collect(),
        [range_len / 2].into(),
    )
}
fn btree_two_sets(range_len: u64, len: u128, coverage_goal: f64) -> (BTreeSet<u64>, BTreeSet<u64>) {
    (
        MemorylessIter::new(0, range_len, len, coverage_goal, 2).collect(),
        MemorylessIter::new(1, range_len, len, coverage_goal, 2).collect(),
    )
}
fn btree_two_sets1(
    range_len: u64,
    len: u128,
    coverage_goal: f64,
) -> (BTreeSet<u64>, BTreeSet<u64>) {
    (
        MemorylessIter::new(0, range_len, len, coverage_goal, 1).collect(),
        BTreeSet::<u64>::from([range_len / 2]),
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
    let len = 10_000_000;
    let range_len = 1_000;
    let coverage_goal = 0.99;
    let mut group = c.benchmark_group("k_intersect");
    group.sample_size(10);
    // group.measurement_time(Duration::from_secs(170));
    group.bench_function("RangeSetInt intersect", |b| {
        b.iter_batched(
            || k_sets(k, range_len, len, coverage_goal),
            |sets| {
                let _answer = RangeSetInt::intersection(sets.iter());
            },
            BatchSize::SmallInput,
        );
    });
    group.bench_function("RangeSetInt dyn intersect", |b| {
        b.iter_batched(
            || k_sets(k, range_len, len, coverage_goal),
            |sets| {
                let sets = sets.iter().map(|x| x.ranges().dyn_sorted_disjoint());
                let _answer: RangeSetInt<_> = intersection(sets).into();
            },
            BatchSize::SmallInput,
        );
    });
    group.bench_function("RangeSetInt intersect 2-at-a-time", |b| {
        b.iter_batched(
            || k_sets(k, range_len, len, coverage_goal),
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
    let len = 1_000_000; // 10_000_000;
    let range_len = 100; //1_000;

    let mut group = c.benchmark_group("coverage_goal");
    for coverage_goal in [0.1, 0.5, 0.75, 0.90, 0.95, 0.99].iter() {
        // group.throughput(Throughput::Bytes(*size as u64));
        group.bench_with_input(
            BenchmarkId::new("dyn", coverage_goal),
            coverage_goal,
            |b, &coverage_goal| {
                b.iter_batched(
                    || k_sets(k, range_len, len, coverage_goal),
                    |sets| {
                        let sets = sets.iter().map(|x| x.ranges().dyn_sorted_disjoint());
                        let _answer: RangeSetInt<_> = intersection(sets).into();
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
                    || k_sets(k, range_len, len, coverage_goal),
                    |sets| {
                        let _answer = RangeSetInt::intersection(sets.iter());
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
                    || k_sets(k, range_len, len, coverage_goal),
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

fn k_play(c: &mut Criterion) {
    let len = 10_000_000; // 10_000_000;
    let range_len = 1000; //1_000;
    let coverage_goal = 0.50;
    let k_list = [2u64, 25, 50, 75, 100];
    let setup_vec = k_list
        .iter()
        .map(|k| (k, k_sets(*k, range_len, len, coverage_goal)))
        .collect::<Vec<_>>();

    let mut group = c.benchmark_group("k_play");
    for (k, setup) in &setup_vec {
        // group.throughput(Throughput::Bytes(*size as u64));
        group.bench_with_input(BenchmarkId::new("dyn", k), k, |b, _k| {
            b.iter_batched(
                || setup,
                |sets| {
                    let sets = sets.iter().map(|x| x.ranges().dyn_sorted_disjoint());
                    let _answer: RangeSetInt<_> = union(sets).into();
                },
                BatchSize::SmallInput,
            );
        });
        group.bench_with_input(BenchmarkId::new("static", k), k, |b, _k| {
            b.iter_batched(
                || setup,
                |sets| {
                    let _answer = RangeSetInt::union(sets.iter());
                },
                BatchSize::SmallInput,
            );
        });
        group.bench_with_input(BenchmarkId::new("two-at-a-time", k), k, |b, _k| {
            b.iter_batched(
                || setup,
                |sets| {
                    // !!!cmk need code for size zero
                    let mut answer = sets[0].clone();
                    for set in sets.iter().skip(1) {
                        answer = answer | set;
                    }
                },
                BatchSize::SmallInput,
            );
        });
    }
    group.finish();
}

// !!!cmk000 understand data generation and make it faster, perhaps only once
// !!!cmk000 make graph show effect of size of Element.
// !!!cmk000 what is effect of # of range_elements? (k can be 2)
// !!!cmk000 shorten code for each section as much as possible.
// !!!cmk000 why is k-play so slow?
// !!!cmk000 understand why criterion_group! starts with "benches"

criterion_group!(
    benches, // insert10,
    // small_random_inserts,
    // big_random_inserts,
    shuffled,
    ascending,
    descending,
    clumps,
    bitxor,
    bitor,
    bitor1,
    k_intersect,
    coverage_goal,
    k_play
);
criterion_main!(benches);

// cmk rule cargo bench intersect
