use core::ops::RangeInclusive;
use num_traits::identities::One;
use rand::distributions::uniform::SampleUniform;
use rand::rngs::StdRng;
use rand::Rng;
use range_set_blaze::Integer;
use range_set_blaze::RangeSetBlaze;

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
    pub fn new(
        rng: &'a mut StdRng,
        range_len: usize,
        range: RangeInclusive<T>,
        coverage_goal: f64,
        k: usize,
        how: How,
    ) -> Self {
        let len: f64 = T::safe_len_to_f64(T::safe_len(&range));
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
                if self.rng.gen::<f64>() < self.average_width {
                    //could precompute
                    actual_width = <T::SafeLen>::one();
                } else {
                    let min_value = T::min_value();
                    //could precompute
                    return Some(min_value.add_one()..=min_value); // empty range
                }
            } else if self.range_len >= 30 {
                // pick a width between about 1 and 2*average_width
                let mut actual_width_f64 =
                    self.rng.gen::<f64>() * (2.0 * self.average_width.floor() - 1.0).floor() + 1.0;
                if self.rng.gen::<f64>() < self.average_width.fract() {
                    actual_width_f64 += 1.0;
                }
                // If actual_width is very, very large, then to f64_to_safe_len will be imprecise.
                actual_width = T::f64_to_safe_len(actual_width_f64);
            } else {
                // pick a width of exactly average_width
                let mut actual_width_f64 = self.average_width.floor();
                if self.rng.gen::<f64>() < self.average_width.fract() {
                    actual_width_f64 += 1.0;
                }
                // If actual_width is very, very large, then to f64_to_safe_len will be imprecise.
                actual_width = T::f64_to_safe_len(actual_width_f64);
            }

            // choose random one point in the range
            let one_point: T = self.rng.gen_range(self.range.clone());
            // go up or down from this point, but not past the ends of the range
            if self.rng.gen::<f64>() > 0.5 {
                let rest = one_point..=*self.range.end();
                if actual_width <= T::safe_len(&rest) {
                    Some(one_point..=T::add_len_less_one(one_point, actual_width))
                } else {
                    Some(rest)
                }
            } else {
                let rest = *self.range.start()..=one_point;
                if actual_width <= T::safe_len(&rest) {
                    Some(T::sub_len_less_one(one_point, actual_width)..=one_point)
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
            RangeSetBlaze::<T>::from_iter(MemorylessRange::new(
                rng,
                range_len,
                range.clone(),
                coverage_goal,
                k,
                how,
            ))
        })
        .collect()
}

// #[allow(clippy::type_complexity)]
// pub fn read_roaring_data(top: &Path) -> Result<Vec<(String, Vec<Vec<u32>>)>, std::io::Error> {
//     let subfolders: Vec<_> = top.read_dir()?.map(|entry| entry.unwrap().path()).collect();
//     let mut name_to_vec_vec: HashMap<String, Vec<Vec<u32>>> = HashMap::new();
//     for subfolder in subfolders {
//         let subfolder_name = subfolder.file_name().unwrap().to_string_lossy().into_string();
//         let mut data: Vec<Vec<u32>> = Vec::new();
//         for file in subfolder.read_dir()? {
//             let file = file.unwrap().path();
//             let contents = fs::read_to_string(&file)?;
//             let contents = contents.trim_end_matches('\n');
//             let nums: Vec<u32> = contents.split(',').map(|s| s.parse().unwrap()).collect();
//             data.push(nums);
//         }
//         name_to_vec_vec.insert(subfolder_name, data);
//     }

//     // sort the keys
//     let mut keys: Vec<String> = name_to_vec_vec.keys().cloned().collect();
//     keys.sort();
//     let mut name_and_vec_vec_list: Vec<(String, Vec<Vec<u32>>)> = Vec::new();
//     for key in keys {
//         let vec_vec = name_to_vec_vec.remove(&key).unwrap();
//         name_and_vec_vec_list.push((key, vec_vec));
//     }

//     Ok(name_and_vec_vec_list)
// }
