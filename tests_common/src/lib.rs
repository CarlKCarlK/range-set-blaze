use std::cmp::min;
use std::ops::RangeInclusive;

use rand::rngs::StdRng;
use rand::Rng;
use rand::SeedableRng;
use range_set_int::RangeSetInt;

pub struct MemorylessRange {
    rng: StdRng,
    len: u128,
    range_len: u64,
    average_coverage_per_clump: f64,
}

#[derive(Debug, Clone, Copy)]
pub enum How {
    Union,
    Intersection,
    None,
}

impl MemorylessRange {
    pub fn new(seed: u64, range_len: u64, len: u128, coverage_goal: f64, k: u64, how: How) -> Self {
        let average_coverage_per_clump = match how {
            How::Union => {
                let goal2 = coverage_goal.powf(1.0 / (k as f64));
                1.0 - (1.0 - goal2).powf(1.0 / (range_len as f64))
            }
            How::Intersection => {
                1.0 - (1.0 - coverage_goal).powf(1.0 / ((range_len as f64) * (k as f64)))
            }
            How::None => 1.0 - (1.0 - coverage_goal).powf(1.0 / (range_len as f64)),
        };
        Self {
            rng: StdRng::seed_from_u64(seed),
            len,
            range_len,
            average_coverage_per_clump,
        }
    }
}

impl Iterator for MemorylessRange {
    type Item = [RangeInclusive<u64>; 2];

    fn next(&mut self) -> Option<Self::Item> {
        if self.range_len == 0 {
            None
        } else {
            self.range_len -= 1;
            let start_fraction = self.rng.gen::<f64>();
            let end_fraction =
                start_fraction + self.average_coverage_per_clump * self.rng.gen::<f64>() * 2.0;
            let start = (start_fraction * self.len as f64) as u64;
            let end = (end_fraction * self.len as f64) as u64;
            let first = start..=min(end, (self.len - 1) as u64);
            #[allow(clippy::reversed_empty_ranges)]
            let second = if (end as u128) < self.len {
                1u64..=0
            } else {
                0u64..=(end - self.len as u64)
            };
            // println!("cmk000 1st, 2nd = {first:?}, {second:?}");
            Some([first, second])
        }
    }
}

pub struct MemorylessIter {
    option_pair: Option<[RangeInclusive<u64>; 2]>,
    iter: MemorylessRange,
}

impl MemorylessIter {
    pub fn new(seed: u64, range_len: u64, len: u128, coverage_goal: f64, k: u64, how: How) -> Self {
        let memoryless_range = MemorylessRange::new(seed, range_len, len, coverage_goal, k, how);
        Self {
            option_pair: None,
            iter: memoryless_range,
        }
    }
}

impl Iterator for MemorylessIter {
    type Item = u64;

    #[allow(clippy::reversed_empty_ranges)]
    fn next(&mut self) -> Option<Self::Item> {
        if let Some(pair) = &self.option_pair {
            let (start0, stop0) = pair[0].clone().into_inner();
            if start0 == stop0 {
                let (start1, stop1) = pair[1].clone().into_inner();
                if start1 > stop1 {
                    self.option_pair = None;
                } else {
                    self.option_pair = Some([start1..=stop1, 1..=0]);
                }
            } else {
                self.option_pair = Some([(start0 + 1)..=stop0, pair[1].clone()]);
            }
            Some(start0)
        } else if let Some(range_inclusive_pair) = self.iter.next() {
            self.option_pair = Some(range_inclusive_pair);
            self.next() // will recurse at most once
        } else {
            None
        }
    }
}

pub fn k_sets(
    k: u64,
    range_len: u64,
    len: u128,
    coverage_goal: f64,
    how: How,
    seed_offset: u64,
) -> Vec<RangeSetInt<u64>> {
    (0..k)
        .map(|i| {
            RangeSetInt::<u64>::from_iter(
                MemorylessRange::new(i + seed_offset, range_len, len, coverage_goal, k, how)
                    .flatten(),
            )
        })
        .collect()
}

pub fn fraction(range_int_set: &RangeSetInt<u64>, len: u128) -> f64 {
    range_int_set.len() as f64 / len as f64
}
