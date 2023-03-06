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

impl MemorylessRange {
    pub fn new(seed: u64, range_len: u64, len: u128, coverage_goal: f64, k: u64) -> Self {
        let average_coverage_per_clump =
            1.0 - (1.0 - coverage_goal).powf(1.0 / ((range_len as f64) * (k as f64)));
        Self {
            rng: StdRng::seed_from_u64(seed),
            len,
            range_len,
            average_coverage_per_clump,
        }
    }
}

impl Iterator for MemorylessRange {
    type Item = RangeInclusive<u64>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.range_len == 0 {
            None
        } else {
            self.range_len -= 1;
            let mut start_fraction = self.rng.gen::<f64>();
            let mut width_fraction = self.rng.gen::<f64>() * self.average_coverage_per_clump * 2.0;
            if start_fraction + width_fraction > 1.0 {
                if self.rng.gen::<f64>() < 0.5 {
                    width_fraction = 1.0 - (start_fraction + width_fraction);
                    start_fraction = 0.0;
                } else {
                    width_fraction = 1.0 - start_fraction;
                }
            }
            let len_f64: f64 = self.len as f64;
            let current_lower_f64: f64 = len_f64 * start_fraction;
            let start = current_lower_f64 as u64;
            let delta = (len_f64 * width_fraction) as u64;
            Some(start..=start + delta)
        }
    }
}

pub struct MemorylessIter {
    option_range_inclusive: Option<RangeInclusive<u64>>,
    iter: MemorylessRange,
}

impl MemorylessIter {
    pub fn new(seed: u64, range_len: u64, len: u128, coverage_goal: f64, k: u64) -> Self {
        let memoryless_range = MemorylessRange::new(seed, range_len, len, coverage_goal, k);
        Self {
            option_range_inclusive: None,
            iter: memoryless_range,
        }
    }
}

impl Iterator for MemorylessIter {
    type Item = u64;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(range_inclusive) = &self.option_range_inclusive {
            let (start, stop) = range_inclusive.clone().into_inner();
            if start == stop {
                self.option_range_inclusive = None;
            } else {
                self.option_range_inclusive = Some((start + 1)..=stop);
            }
            Some(start)
        } else if let Some(range_inclusive) = self.iter.next() {
            self.option_range_inclusive = Some(range_inclusive);
            self.next() // will recurse at most once
        } else {
            None
        }
    }
}

pub fn k_sets(k: u64, range_len: u64, len: u128, coverage_goal: f64) -> Vec<RangeSetInt<u64>> {
    (0..k)
        .map(|i| {
            RangeSetInt::<u64>::from_iter(MemorylessRange::new(i, range_len, len, coverage_goal, k))
        })
        .collect()
}
