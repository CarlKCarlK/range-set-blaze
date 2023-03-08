use std::ops::RangeInclusive;

use rand::rngs::StdRng;
use rand::Rng;
use range_set_int::Integer;
use range_set_int::RangeSetInt;

// Not reliable if the range_inclusive is too small, especially if the range_len
// is small. Might have some off-by-one errors that aren't material in practice.
pub struct MemorylessRange<'a, T: Integer> {
    rng: &'a mut StdRng,
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

impl<'a, T: Integer> MemorylessRange<'a, T> {
    pub fn new(
        rng: &'a mut StdRng,
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
            rng,
            range_len,
            range_inclusive,
            average_coverage_per_clump,
        }
    }
}

impl<'a, T: Integer> Iterator for MemorylessRange<'a, T> {
    type Item = RangeInclusive<T>;

    fn next(&mut self) -> Option<Self::Item> {
        let len: f64 = T::into_f64(T::safe_inclusive_len(&self.range_inclusive));
        // This may not work for all ranges, but it works for the ones we're using.
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

pub struct MemorylessIter<'a, T: Integer> {
    option_range_inclusive: Option<RangeInclusive<T>>,
    iter: MemorylessRange<'a, T>,
}

impl<'a, T: Integer> MemorylessIter<'a, T> {
    pub fn new(
        rng: &'a mut StdRng,
        range_len: usize,
        range_inclusive: RangeInclusive<T>,
        coverage_goal: f64,
        k: usize,
        how: How,
    ) -> Self {
        let memoryless_range =
            MemorylessRange::new(rng, range_len, range_inclusive, coverage_goal, k, how);
        Self {
            option_range_inclusive: None,
            iter: memoryless_range,
        }
    }
}

impl<'a, T: Integer> Iterator for MemorylessIter<'a, T> {
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
    rng: &mut StdRng,
) -> Vec<RangeSetInt<T>> {
    (0..k)
        .map(|_i| {
            RangeSetInt::<T>::from_iter(MemorylessRange::new(
                rng,
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
