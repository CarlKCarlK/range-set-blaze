use std::ops::RangeInclusive;

use rand::rngs::StdRng;
use rand::Rng;
use rand::SeedableRng;

pub struct MemorylessData {
    option_range_inclusive: Option<RangeInclusive<u64>>,
    rng: StdRng,
    len: u128,
    range_len: u64,
    average_coverage_per_clump: f64,
}

impl MemorylessData {
    pub fn new(seed: u64, range_len: u64, len: u128, coverage_goal: f64) -> Self {
        let average_coverage_per_clump = 1.0 - (1.0 - coverage_goal).powf(1.0 / (range_len as f64));
        Self {
            rng: StdRng::seed_from_u64(seed),
            option_range_inclusive: None,
            len,
            range_len,
            average_coverage_per_clump,
        }
    }
}

impl Iterator for MemorylessData {
    type Item = u64;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(range_inclusive) = self.option_range_inclusive.clone() {
            let (self_start, self_end) = range_inclusive.into_inner();
            let value = self_start;
            if self_start == self_end {
                self.option_range_inclusive = None;
            } else {
                self.option_range_inclusive = Some(self_start + 1..=self_end);
            }
            Some(value)
        } else if self.range_len == 0 {
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
            self.option_range_inclusive = Some(start..=start + delta);
            self.next()
        }
    }
}
