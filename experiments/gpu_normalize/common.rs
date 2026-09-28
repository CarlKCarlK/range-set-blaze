#[derive(Clone, Copy, Debug)]
enum Distribution {
    RandomGaps,
    DuplicateHeavy,
    Clumps16,
    Clumps256,
    Clumps4096,
}

impl Distribution {
    const fn name(self) -> &'static str {
        match self {
            Self::RandomGaps => "random-gaps",
            Self::DuplicateHeavy => "duplicate-heavy",
            Self::Clumps16 => "clumps-16",
            Self::Clumps256 => "clumps-256",
            Self::Clumps4096 => "clumps-4096",
        }
    }

    const fn clump_len(self) -> Option<usize> {
        match self {
            Self::RandomGaps | Self::DuplicateHeavy => None,
            Self::Clumps16 => Some(16),
            Self::Clumps256 => Some(256),
            Self::Clumps4096 => Some(4096),
        }
    }
}

const UNSORTED_DISTRIBUTIONS: [Distribution; 5] = [
    Distribution::RandomGaps,
    Distribution::DuplicateHeavy,
    Distribution::Clumps16,
    Distribution::Clumps256,
    Distribution::Clumps4096,
];

fn generate_sorted(len: usize, distribution: Distribution) -> Vec<u32> {
    let mut values = Vec::with_capacity(len);
    let mut value = 0u32;
    let mut state = 0x9e37_79b9u32;

    for index in 0..len {
        state ^= state << 13;
        state ^= state >> 17;
        state ^= state << 5;
        let increment = distribution.clump_len().map_or_else(
            || match distribution {
                Distribution::RandomGaps => 1 + state % 31,
                Distribution::DuplicateHeavy => u32::from(index % 8 == 0) * (1 + state % 7),
                _ => unreachable!(),
            },
            |clump_len| 1 + u32::from(index != 0 && index % clump_len == 0),
        );
        value = value
            .checked_add(increment)
            .expect("benchmark generator exceeded the u32 domain");
        values.push(value);
    }
    values
}

fn generate_shuffled(len: usize, distribution: Distribution) -> Vec<u32> {
    let mut values = generate_sorted(len, distribution);
    let mut state = 0xd1b5_4a32_d192_ed03u64;
    for index in (1..values.len()).rev() {
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        let modulus = u64::try_from(index + 1).expect("index exceeded u64");
        let swap_index = usize::try_from(state % modulus).expect("swap index exceeded usize");
        values.swap(index, swap_index);
    }
    values
}

struct TrustedGpuRanges<I> {
    inner: I,
}

impl<I> TrustedGpuRanges<I> {
    const fn new(inner: I) -> Self {
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

fn set_from_gpu_ranges(ranges: Vec<RangeInclusive<u32>>) -> RangeSetBlaze<u32> {
    RangeSetBlaze::from_sorted_disjoint(TrustedGpuRanges::new(ranges.into_iter()))
}

fn durations(iterations: usize, mut operation: impl FnMut()) -> Vec<Duration> {
    let mut samples = Vec::with_capacity(iterations);
    for _ in 0..iterations {
        let start = Instant::now();
        operation();
        samples.push(start.elapsed());
    }
    samples.sort_unstable();
    samples
}

fn median(samples: &[Duration]) -> Duration {
    samples[samples.len() / 2]
}

const fn iterations(len: usize) -> usize {
    match len {
        0..=100_000 => 9,
        100_001..=1_000_000 => 7,
        1_000_001..=10_000_000 => 5,
        _ => 3,
    }
}

struct UnsortedCpuTimings {
    from_iter_total: Duration,
}

fn benchmark_unsorted_cpu(values: &[u32], iterations: usize) -> UnsortedCpuTimings {
    let from_iter = durations(iterations, || {
        drop(black_box(values.iter().copied()).collect::<RangeSetBlaze<_>>());
    });
    UnsortedCpuTimings {
        from_iter_total: median(&from_iter),
    }
}

fn requested_sizes() -> Vec<usize> {
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

fn milliseconds(duration: Duration) -> f64 {
    duration.as_secs_f64() * 1_000.0
}
