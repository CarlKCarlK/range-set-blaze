use std::ops::RangeInclusive;

use rand::rngs::StdRng;
use rand::Rng;
use rand::SeedableRng;
use range_set_int::Integer;
use range_set_int::RangeSetInt;

pub struct MemorylessRange<T: Integer> {
    rng: StdRng,
    range_len: usize,
    range_inclusive: RangeInclusive<T>,
    average_coverage_per_clump: f64,
}

#[derive(Debug, Clone, Copy)]
pub enum How {
    Union,
    Intersection,
    None,
}

impl<T: Integer> MemorylessRange<T> {
    pub fn new(
        seed: u64,
        range_len: usize,
        range_inclusive: RangeInclusive<T>,
        coverage_goal: f64,
        k: usize,
        how: How,
    ) -> Self {
        // let len: f64 = T::into_f64(T::safe_inclusive_len(&range_inclusive));
        let average_coverage_per_clump = match how {
            How::Intersection => {
                let goal2 = coverage_goal.powf(1.0 / (k as f64));
                1.0 - (1.0 - goal2).powf(1.0 / range_len as f64)
            }
            How::Union => 1.0 - (1.0 - coverage_goal).powf(1.0 / (range_len as f64 * k as f64)),
            How::None => 1.0 - (1.0 - coverage_goal).powf(1.0 / range_len as f64),
        };
        Self {
            rng: StdRng::from_entropy(), //cmk00000 ::seed_from_u64(seed),
            range_len,
            range_inclusive,
            average_coverage_per_clump,
        }
    }
}

impl<T: Integer> Iterator for MemorylessRange<T> {
    type Item = RangeInclusive<T>;

    fn next(&mut self) -> Option<Self::Item> {
        let len: f64 = T::into_f64(T::safe_inclusive_len(&self.range_inclusive));
        let offset: f64 = T::into_f64(T::safe_inclusive_len(
            &(*self.range_inclusive.start()..=T::zero()),
        ));
        //cmk000 what if zero is not in range?
        if self.range_len == 0 {
            None
        } else {
            self.range_len -= 1;
            let mid_fraction = self.rng.gen::<f64>();
            let start_fraction =
                (mid_fraction - self.rng.gen::<f64>() * self.average_coverage_per_clump).max(0.0);
            let stop_fraction =
                (mid_fraction + self.rng.gen::<f64>() * self.average_coverage_per_clump).min(1.0);
            // cmk000 println!("start_fraction: {start_fraction}, stop_fraction: {stop_fraction}, delta={}, a_c_p_c={}", stop_fraction - start_fraction, self.average_coverage_per_clump, start_fraction=start_fraction, stop_fraction=stop_fraction);
            let start: T = T::from_f64(start_fraction * len - offset);
            let stop: T = T::from_f64(stop_fraction * len - offset);
            // let fraction_value: f64 = T::into_f64(T::safe_inclusive_len(&(start..=stop))) / len;
            // cmk000 println!("fraction_value: {}", fraction_value);
            Some(start..=stop)
        }
    }
}

pub struct MemorylessIter<T: Integer> {
    option_range_inclusive: Option<RangeInclusive<T>>,
    iter: MemorylessRange<T>,
}

impl<T: Integer> MemorylessIter<T> {
    pub fn new(
        seed: u64,
        range_len: usize,
        range_inclusive: RangeInclusive<T>,
        coverage_goal: f64,
        k: usize,
        how: How,
    ) -> Self {
        let memoryless_range =
            MemorylessRange::new(seed, range_len, range_inclusive, coverage_goal, k, how);
        Self {
            option_range_inclusive: None,
            iter: memoryless_range,
        }
    }
}

impl<T: Integer> Iterator for MemorylessIter<T> {
    type Item = T;

    #[allow(clippy::reversed_empty_ranges)]
    fn next(&mut self) -> Option<Self::Item> {
        if let Some(range_inclusive) = &self.option_range_inclusive {
            let (start, stop) = range_inclusive.clone().into_inner();
            if start == stop {
                self.option_range_inclusive = None;
            } else {
                self.option_range_inclusive = Some(start + T::one()..=stop);
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

pub fn k_sets<T: Integer>(
    k: usize,
    range_len: usize,
    range_inclusive: &RangeInclusive<T>,
    coverage_goal: f64,
    how: How,
    seed_offset: u64,
) -> Vec<RangeSetInt<T>> {
    (0..k)
        .map(|i| {
            RangeSetInt::<T>::from_iter(MemorylessRange::new(
                i as u64 + seed_offset,
                range_len,
                range_inclusive.clone(),
                coverage_goal,
                k,
                how,
            ))
        })
        .collect()
}

pub fn fraction<T: Integer>(
    range_int_set: &RangeSetInt<T>,
    range_inclusive: &RangeInclusive<T>,
) -> f64 {
    T::into_f64(range_int_set.len()) / T::into_f64(T::safe_inclusive_len(range_inclusive))
}
