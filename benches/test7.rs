// https://bheisler.github.io/criterion.rs/book/getting_started.html
// https://www.jibbow.com/posts/criterion-flamegraphs/

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rand::seq::SliceRandom;
use rand::{rngs::StdRng, Rng, SeedableRng};
use rangeset_int::RangeSetInt;
// use thousands::Separable;

fn insert10(c: &mut Criterion) {
    let array = black_box([1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);
    c.bench_function("test7", |b| {
        b.iter(|| {
            let _x = RangeSetInt::<u8>::from(array);
            // x = x & x;
        })
    });
}

fn small_random_inserts(c: &mut Criterion) {
    c.bench_function("small_random_inserts", |b| b.iter(test7));
}

fn big_random_inserts(c: &mut Criterion) {
    c.bench_function("big_random_inserts", |b| b.iter(test7big));
}

fn test7() {
    let mut range_set = RangeSetInt::<u64>::new();
    // let mut index = 0u64;
    #[allow(clippy::explicit_counter_loop)]
    for value in RandomData::new(
        0,
        RangeX {
            start: 20,
            length: 10,
        },
        1,
        //     RangeX {
        //         start: 20,
        //         length: 1_300_300_010,
        //     },
        //     100_000,
    ) {
        // if index % 10_000_000 == 0 {
        //     println!(
        //         "index {}, range_count {}",
        //         index.separate_with_commas(),
        //         range_set.items.len().separate_with_commas()
        //     );
        // }
        // index += 1;
        range_set.insert(value);
        // println!("{value}->{range_set}");
    }
    // println!("{:?}", range_set._items);
}

fn test7big() {
    let mut range_set = RangeSetInt::<u64>::new();
    #[allow(clippy::explicit_counter_loop)]
    for value in RandomData::new(
        0,
        RangeX {
            start: 20,
            length: 1_300_300_010,
        },
        100_000,
    ) {
        range_set.insert(value);
    }
}

#[derive(Debug)]
struct RangeX {
    start: u64,
    length: u64,
}

// impl RangeX {
//     fn end(&self) -> u128 {
//         self.start + self.length
//     }
// }

struct RandomData {
    rng: StdRng,
    current: Option<RangeX>,
    data_range: Vec<RangeX>,
    small_enough: u64,
}

impl RandomData {
    fn new(seed: u64, range: RangeX, small_enough: u64) -> Self {
        Self {
            rng: StdRng::seed_from_u64(seed),
            current: None,
            data_range: vec![range],
            small_enough,
        }
    }
}

impl Iterator for RandomData {
    type Item = u64;
    fn next(&mut self) -> Option<Self::Item> {
        if let Some(current) = &mut self.current {
            let value = current.start;
            self.current = if current.length > 1 {
                Some(RangeX {
                    start: current.start + 1,
                    length: current.length - 1,
                })
            } else {
                None
            };
            Some(value)
        } else if self.data_range.is_empty() {
            None
        } else {
            let range = self.data_range.pop().unwrap();
            if range.length <= self.small_enough {
                self.current = Some(range);
                self.next()
            } else {
                let split = 5;
                let delete_fraction = 0.1;
                let dup_fraction = 0.01;
                let part_list =
                    _process_this_level(split, range, &mut self.rng, delete_fraction, dup_fraction);
                self.data_range.splice(0..0, part_list);
                self.next()
            }
        }
    }
}

fn _process_this_level(
    split: u64,
    range: RangeX,
    rng: &mut StdRng,
    delete_fraction: f64,
    dup_fraction: f64,
) -> Vec<RangeX> {
    let mut part_list = Vec::<RangeX>::new();
    for i in 0..split {
        let start = i * range.length / split + range.start;
        let end = (i + 1) * range.length / split + range.start;

        if rng.gen::<f64>() < delete_fraction {
            continue;
        }

        part_list.push(RangeX {
            start,
            length: end - start,
        });

        if rng.gen::<f64>() < dup_fraction {
            part_list.push(RangeX {
                start,
                length: end - start,
            });
        }
    }
    // shuffle the list
    part_list.shuffle(rng);
    part_list
}

criterion_group!(benches, insert10, small_random_inserts, big_random_inserts);
criterion_main!(benches);
