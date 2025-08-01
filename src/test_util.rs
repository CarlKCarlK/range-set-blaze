use crate::Integer;
use crate::RangeMapBlaze;
use crate::RangeSetBlaze;
use alloc::vec;
use alloc::vec::Vec;
use core::ops::RangeInclusive;
use num_traits::identities::One;
use rand::Rng;
use rand::distr::uniform::SampleUniform;
use rand::rngs::StdRng;

#[must_use]
#[allow(clippy::cast_precision_loss, clippy::cast_possible_truncation)]
pub fn width_to_range(
    iter_len: usize,
    average_width: usize,
    coverage_goal: f64,
) -> (usize, core::ops::RangeInclusive<i32>) {
    let range_len = iter_len / average_width;
    let one_fraction: f64 = 1.0 - (1.0 - coverage_goal).powf(1.0 / range_len as f64);
    let range = 0..=(((average_width as f64 / one_fraction) - 0.5) as i32);
    (range_len, range)
}

#[must_use]
#[allow(
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss
)]
pub fn width_to_range_u32(
    iter_len: usize,
    average_width: usize,
    coverage_goal: f64,
) -> (usize, core::ops::RangeInclusive<u32>) {
    let range_len = iter_len / average_width;
    let one_fraction: f64 = 1.0 - (1.0 - coverage_goal).powf(1.0 / range_len as f64);
    let range = 0..=((average_width as f64 / one_fraction) as u32);
    (range_len, range)
}

// Not reliable if the range is too small, especially if the range_len
// is small. Might have some off-by-one errors that aren't material in practice.
#[must_use = "iterators are lazy and do nothing unless consumed"]
pub struct MemorylessRange<'a, T: Integer + SampleUniform> {
    rng: &'a mut StdRng,
    range_len: usize,
    range: RangeInclusive<T>,
    average_width: f64,
}

#[derive(Debug, Clone, Copy)]
pub enum How {
    Union,
    Intersection,
    None,
}

impl<'a, T: Integer + SampleUniform> MemorylessRange<'a, T> {
    #[allow(clippy::cast_precision_loss)]
    pub fn new(
        rng: &'a mut StdRng,
        range_len: usize,
        range: RangeInclusive<T>,
        coverage_goal: f64,
        k: usize,
        how: How,
    ) -> Self {
        let len: f64 = T::safe_len_to_f64_lossy(T::safe_len(&range));
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
            range,
            average_width: average_coverage_per_clump * len,
        }
    }
}

impl<T: Integer + SampleUniform> Iterator for MemorylessRange<'_, T> {
    type Item = RangeInclusive<T>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.range_len == 0 {
            None
        } else {
            self.range_len -= 1;
            // if the expected_width is < 1, then we sometimes output empty ranges.
            // if the expected_width is > 1, then ranges always have width >= 1.
            // We never wrap around the end of the range, so the ends can be over represented.
            let actual_width: T::SafeLen;
            if self.average_width < 1.0 {
                if self.rng.random::<f64>() < self.average_width {
                    //could precompute
                    actual_width = <T::SafeLen>::one();
                } else {
                    let min_value = T::min_value();
                    //could precompute
                    return Some(min_value.add_one()..=min_value); // empty range
                }
            } else if self.range_len >= 30 {
                // pick a width between about 1 and 2*average_width
                let mut actual_width_f64 = self.rng.random::<f64>()
                    * (2.0 * self.average_width.floor() - 1.0).floor()
                    + 1.0;
                if self.rng.random::<f64>() < self.average_width.fract() {
                    actual_width_f64 += 1.0;
                }
                // If actual_width is very, very large, then to f64_to_safe_len_lossy will be imprecise.
                actual_width = T::f64_to_safe_len_lossy(actual_width_f64);
            } else {
                // pick a width of exactly average_width
                let mut actual_width_f64 = self.average_width.floor();
                if self.rng.random::<f64>() < self.average_width.fract() {
                    actual_width_f64 += 1.0;
                }
                // If actual_width is very, very large, then to f64_to_safe_len_lossy will be imprecise.
                actual_width = T::f64_to_safe_len_lossy(actual_width_f64);
            }

            // choose random one point in the range
            let one_point: T = self.rng.random_range(self.range.clone());
            // go up or down from this point, but not past the ends of the range
            if self.rng.random::<f64>() > 0.5 {
                let rest = one_point..=*self.range.end();
                if actual_width <= T::safe_len(&rest) {
                    Some(one_point..=T::inclusive_end_from_start(one_point, actual_width))
                } else {
                    Some(rest)
                }
            } else {
                let rest = *self.range.start()..=one_point;
                if actual_width <= T::safe_len(&rest) {
                    Some(T::start_from_inclusive_end(one_point, actual_width)..=one_point)
                } else {
                    Some(rest)
                }
            }
        }
    }
}

#[must_use = "iterators are lazy and do nothing unless consumed"]
pub struct MemorylessIter<'a, T: Integer + SampleUniform> {
    option_range: Option<RangeInclusive<T>>,
    iter: MemorylessRange<'a, T>,
}

impl<'a, T: Integer + SampleUniform> MemorylessIter<'a, T> {
    #[inline]
    pub fn new(
        rng: &'a mut StdRng,
        range_len: usize,
        range: RangeInclusive<T>,
        coverage_goal: f64,
        k: usize,
        how: How,
    ) -> Self {
        let memoryless_range = MemorylessRange::new(rng, range_len, range, coverage_goal, k, how);
        Self {
            option_range: None,
            iter: memoryless_range,
        }
    }
}

impl<T: Integer + SampleUniform> Iterator for MemorylessIter<'_, T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        let range = self
            .option_range
            .take()
            .or_else(|| self.iter.find(|range| range.start() <= range.end()))?;
        let (start, end) = range.into_inner();
        if start < end {
            self.option_range = Some(start.add_one()..=end);
        }
        Some(start)
    }
}

#[must_use = "iterators are lazy and do nothing unless consumed"]
pub struct ClumpyMapIter<'a, T: Integer + SampleUniform> {
    iter: ClumpyMapRange<'a, T>,
    option_range_value: Option<(RangeInclusive<T>, u32)>,
}

#[allow(clippy::too_many_arguments)]
impl<'a, T: Integer + SampleUniform> ClumpyMapIter<'a, T> {
    #[inline]
    pub fn new(
        rng: &'a mut StdRng,
        clump_len: usize,
        range: RangeInclusive<T>,
        coverage_goal: f64,
        k: usize,
        how: How,
        value_count: u32,
        range_per_clump: usize,
    ) -> Self {
        let iter = ClumpyMapRange::new(
            rng,
            clump_len,
            range,
            coverage_goal,
            k,
            how,
            value_count,
            range_per_clump,
        );
        Self {
            iter,
            option_range_value: None,
        }
    }
}

impl<T: Integer + SampleUniform> Iterator for ClumpyMapIter<'_, T> {
    type Item = (T, u32);

    fn next(&mut self) -> Option<Self::Item> {
        let (range, value) = self.option_range_value.take().or_else(|| {
            let range_and_value = self
                .iter
                .find(|(range, _value)| range.start() <= range.end())?;
            Some(range_and_value)
        })?;

        let (start, end) = range.into_inner();
        if start < end {
            self.option_range_value = Some((start.add_one()..=end, value));
        }
        Some((start, value))
    }
}

#[must_use = "iterators are lazy and do nothing unless consumed"]
pub struct ClumpyMapRange<'a, T: Integer + SampleUniform> {
    rng_clone: StdRng,
    clump_iter: MemorylessRange<'a, T>,
    value_count: u32,
    range_per_clump: usize,
    outputs_iter: std::vec::IntoIter<(RangeInclusive<T>, u32)>,
}

#[allow(clippy::too_many_arguments)]
impl<'a, T: Integer + SampleUniform> ClumpyMapRange<'a, T> {
    #[inline]
    pub fn new(
        value_rng: &'a mut StdRng,
        clump_len: usize,
        range: RangeInclusive<T>,
        coverage_goal: f64,
        k: usize,
        how: How,
        value_count: u32,
        range_per_clump: usize,
    ) -> Self {
        assert!(range_per_clump > 0, "range_per_clump must be > 0");
        let rng_clone = value_rng.clone();
        let clump_iter = MemorylessRange::new(value_rng, clump_len, range, coverage_goal, k, how);
        let outputs: Vec<(RangeInclusive<T>, u32)> = vec![];
        Self {
            rng_clone,
            clump_iter,
            value_count,
            range_per_clump,
            outputs_iter: outputs.into_iter(),
        }
    }
}

impl<T: Integer + SampleUniform> Iterator for ClumpyMapRange<'_, T> {
    type Item = (RangeInclusive<T>, u32);

    #[allow(clippy::needless_collect)]
    fn next(&mut self) -> Option<Self::Item> {
        if let Some(item) = self.outputs_iter.next() {
            return Some(item);
        }
        let clump = self.clump_iter.next()?;
        let value = self.rng_clone.random_range(0..self.value_count);
        let mut points: Vec<T> = vec![*clump.start(), *clump.end()];
        for _ in 0..self.range_per_clump {
            points.push(self.rng_clone.random_range(clump.clone()));
        }
        points.sort_unstable();
        let outputs: Vec<(RangeInclusive<T>, u32)> = points
            .windows(3)
            .map(|triple| (triple[0]..=triple[2], value))
            .collect();
        self.outputs_iter = outputs.into_iter();
        self.outputs_iter.next()
    }
}

pub fn k_sets<T: Integer + SampleUniform>(
    k: usize,
    range_len: usize,
    range: &RangeInclusive<T>,
    coverage_goal: f64,
    how: How,
    rng: &mut StdRng,
) -> Vec<RangeSetBlaze<T>> {
    (0..k)
        .map(|_i| {
            MemorylessRange::new(rng, range_len, range.clone(), coverage_goal, k, how)
                .collect::<RangeSetBlaze<T>>()
        })
        .collect()
}

#[allow(clippy::too_many_arguments)]
/// k is how many maps to generate. If How is union they will meet coverage_goal when unioned, but if None
/// then they will each meet the coverage goal individually.
pub fn k_maps<T: Integer + SampleUniform>(
    k: usize,
    clump_len: usize,
    range: &RangeInclusive<T>,
    coverage_goal: f64,
    how: How,
    rng: &mut StdRng,
    value_count: u32,
    range_per_clump: usize,
) -> Vec<RangeMapBlaze<T, u32>> {
    (0..k)
        .map(|_i| {
            ClumpyMapRange::new(
                rng,
                clump_len,
                range.clone(),
                coverage_goal,
                k,
                how,
                value_count,
                range_per_clump,
            )
            .collect::<RangeMapBlaze<T, u32>>()
        })
        .collect()
}
