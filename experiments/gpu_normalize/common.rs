use std::hint::black_box;
use std::iter::FusedIterator;
use std::ops::RangeInclusive;
use std::time::{Duration, Instant};

use range_set_blaze::{RangeSetBlaze, SortedDisjoint, SortedStarts};

#[derive(Clone, Copy, Debug)]
pub(crate) enum Distribution {
    RandomGaps,
    DuplicateHeavy,
    Clumps4,
    Clumps16,
    Clumps64,
    Clumps256,
    Clumps1024,
    Clumps4096,
}

impl Distribution {
    pub(crate) const ALL: [Self; 8] = [
        Self::RandomGaps,
        Self::DuplicateHeavy,
        Self::Clumps4,
        Self::Clumps16,
        Self::Clumps64,
        Self::Clumps256,
        Self::Clumps1024,
        Self::Clumps4096,
    ];

    pub(crate) const fn name(self) -> &'static str {
        match self {
            Self::RandomGaps => "random-gaps",
            Self::DuplicateHeavy => "duplicate-heavy",
            Self::Clumps4 => "clumps-4",
            Self::Clumps16 => "clumps-16",
            Self::Clumps64 => "clumps-64",
            Self::Clumps256 => "clumps-256",
            Self::Clumps1024 => "clumps-1024",
            Self::Clumps4096 => "clumps-4096",
        }
    }

    const fn clump_len(self) -> Option<usize> {
        match self {
            Self::RandomGaps | Self::DuplicateHeavy => None,
            Self::Clumps4 => Some(4),
            Self::Clumps16 => Some(16),
            Self::Clumps64 => Some(64),
            Self::Clumps256 => Some(256),
            Self::Clumps1024 => Some(1024),
            Self::Clumps4096 => Some(4096),
        }
    }
}

#[cfg(feature = "gpu-cub")]
pub(crate) const UNSORTED_DISTRIBUTIONS: [Distribution; 5] = [
    Distribution::RandomGaps,
    Distribution::DuplicateHeavy,
    Distribution::Clumps16,
    Distribution::Clumps256,
    Distribution::Clumps4096,
];

pub(crate) fn generate_sorted(len: usize, distribution: Distribution) -> Vec<u32> {
    let mut values = Vec::with_capacity(len);
    let mut value = 0u32;
    let mut state = 0x9e37_79b9u32;

    for index in 0..len {
        state ^= state << 13;
        state ^= state >> 17;
        state ^= state << 5;
        let increment = match distribution.clump_len() {
            Some(clump_len) => 1 + u32::from(index != 0 && index % clump_len == 0),
            None => match distribution {
                Distribution::RandomGaps => 1 + state % 31,
                Distribution::DuplicateHeavy => u32::from(index % 8 == 0) * (1 + state % 7),
                _ => unreachable!(),
            },
        };
        value = value
            .checked_add(increment)
            .expect("benchmark generator exceeded the u32 domain");
        values.push(value);
    }
    values
}

#[cfg(feature = "gpu-cub")]
pub(crate) fn generate_shuffled(len: usize, distribution: Distribution) -> Vec<u32> {
    let mut values = generate_sorted(len, distribution);
    let mut state = 0xd1b5_4a32_d192_ed03u64;
    for index in (1..values.len()).rev() {
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        values.swap(index, state as usize % (index + 1));
    }
    values
}

pub(crate) fn cpu_sorted_ranges(values: &[u32]) -> Vec<RangeInclusive<u32>> {
    let Some((&first, rest)) = values.split_first() else {
        return Vec::new();
    };
    let mut ranges = Vec::new();
    let mut start = first;
    let mut end = first;
    for &value in rest {
        assert!(
            value >= end,
            "the post-sort benchmark requires sorted input"
        );
        if value == end {
            continue;
        }
        if end != u32::MAX && value == end + 1 {
            end = value;
            continue;
        }
        ranges.push(start..=end);
        start = value;
        end = value;
    }
    ranges.push(start..=end);
    ranges
}

struct TrustedGpuRanges<I> {
    inner: I,
}

impl<I> TrustedGpuRanges<I> {
    fn new(inner: I) -> Self {
        Self { inner }
    }
}

impl<I> Iterator for TrustedGpuRanges<I>
where
    I: Iterator<Item = RangeInclusive<u32>>,
{
    type Item = RangeInclusive<u32>;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

impl<I> FusedIterator for TrustedGpuRanges<I> where
    I: Iterator<Item = RangeInclusive<u32>> + FusedIterator
{
}

// The boundary/scan/compaction pipeline emits one nonempty range per paired
// start/end, in ascending order, and suppresses duplicate and adjacent splits.
// Untimed correctness checks compare the emitted vector with the CPU result.
impl<I> SortedStarts<u32> for TrustedGpuRanges<I> where
    I: Iterator<Item = RangeInclusive<u32>> + FusedIterator
{
}
impl<I> SortedDisjoint<u32> for TrustedGpuRanges<I> where
    I: Iterator<Item = RangeInclusive<u32>> + FusedIterator
{
}

pub(crate) fn set_from_gpu_ranges(ranges: Vec<RangeInclusive<u32>>) -> RangeSetBlaze<u32> {
    RangeSetBlaze::from_sorted_disjoint(TrustedGpuRanges::new(ranges.into_iter()))
}

fn copy_range(range: &RangeInclusive<u32>) -> RangeInclusive<u32> {
    *range.start()..=*range.end()
}

pub(crate) fn set_from_trusted_range_slice(ranges: &[RangeInclusive<u32>]) -> RangeSetBlaze<u32> {
    let copied = ranges.iter().map(copy_range);
    RangeSetBlaze::from_sorted_disjoint(TrustedGpuRanges::new(copied))
}

pub(crate) fn durations(iterations: usize, mut operation: impl FnMut()) -> Vec<Duration> {
    let mut samples = Vec::with_capacity(iterations);
    for _ in 0..iterations {
        let start = Instant::now();
        operation();
        samples.push(start.elapsed());
    }
    samples.sort_unstable();
    samples
}

pub(crate) fn median(samples: &[Duration]) -> Duration {
    samples[samples.len() / 2]
}

pub(crate) fn iterations(len: usize) -> usize {
    match len {
        0..=100_000 => 9,
        100_001..=1_000_000 => 7,
        1_000_001..=10_000_000 => 5,
        _ => 3,
    }
}

pub(crate) struct CpuTimings {
    pub(crate) from_slice_total: Duration,
    pub(crate) normalize_ranges: Duration,
    pub(crate) build_from_ranges: Duration,
}

#[cfg(feature = "gpu-cub")]
pub(crate) struct UnsortedCpuTimings {
    pub(crate) from_slice_total: Duration,
    pub(crate) from_iter_total: Duration,
}

#[cfg(feature = "gpu-cub")]
pub(crate) fn benchmark_unsorted_cpu(values: &[u32], iterations: usize) -> UnsortedCpuTimings {
    let from_slice = durations(iterations, || {
        black_box(RangeSetBlaze::from_slice(black_box(values)));
    });
    let from_iter = durations(iterations, || {
        black_box(RangeSetBlaze::from_iter(black_box(values.iter().copied())));
    });
    UnsortedCpuTimings {
        from_slice_total: median(&from_slice),
        from_iter_total: median(&from_iter),
    }
}

pub(crate) fn benchmark_cpu(
    values: &[u32],
    perfect_ranges: &[RangeInclusive<u32>],
    iterations: usize,
) -> CpuTimings {
    let normalize_ranges = durations(iterations, || {
        black_box(cpu_sorted_ranges(black_box(values)));
    });
    let from_slice = durations(iterations, || {
        black_box(RangeSetBlaze::from_slice(black_box(values)));
    });
    let build_from_ranges = durations(iterations, || {
        black_box(set_from_trusted_range_slice(black_box(perfect_ranges)));
    });
    CpuTimings {
        from_slice_total: median(&from_slice),
        normalize_ranges: median(&normalize_ranges),
        build_from_ranges: median(&build_from_ranges),
    }
}

pub(crate) fn requested_sizes() -> Vec<usize> {
    let sizes: Vec<_> = std::env::args()
        .skip(1)
        .filter(|value| !value.starts_with("--"))
        .map(|value| {
            let size = value.parse().expect("sizes must be positive integers");
            assert!(size > 0, "sizes must be positive integers");
            size
        })
        .collect();
    if sizes.is_empty() {
        vec![100_000, 1_000_000, 10_000_000, 100_000_000]
    } else {
        sizes
    }
}

pub(crate) fn milliseconds(duration: Duration) -> f64 {
    duration.as_secs_f64() * 1_000.0
}
