//! Builds a [`RangeSetBlaze`] from a globally sorted slice by finding its ranges on several
//! threads at once. [`Engine::Cpu`](super::Engine::Cpu) and
//! [`Engine::CpuRadix`](super::Engine::CpuRadix) use it after their sort, for every key type that
//! has the SIMD range finder (see `GpuElement::from_sorted_slice`).
//!
//! # The idea
//!
//! After a global sort, contiguous chunks of the slice are already ordered relative to each
//! other: every value in chunk `i` is at most every value in chunk `i + 1`. So:
//!
//! 1. Each Rayon worker runs the existing SIMD range finder on its own chunk. Because the chunk is
//!    sorted, the ranges it finds are already sorted, disjoint, and not touching.
//! 2. No general union is needed to combine the chunks. Interior ranges can't interact with
//!    anything. Only the last range of one chunk and the first range of the next nonempty chunk
//!    can need merging: they can share a value (duplicates straddle the cut) or be adjacent
//!    (`41` ends one chunk, `42` starts the next).
//! 3. [`SeamRanges`] walks the per-chunk vectors in order, lazily, holding one pending range so a
//!    single range can grow across several cuts, and feeds
//!    [`RangeSetBlaze::from_sorted_disjoint`] directly. There is no concatenated vector and no
//!    second sort.

use core::{iter::FusedIterator, mem, num::NonZeroUsize, ops::RangeInclusive};
use std::{vec, vec::Vec};

use fearless_simd::Level;
use rayon::{iter::ParallelIterator, slice::ParallelSlice};

use super::thread_pool;

use crate::{
    Integer, RangeSetBlaze, SortedDisjoint, SortedStarts,
    from_slice::{SimdInteger, sorted_slice_ranges},
};

/// The fewest values worth giving a thread of their own. Below this, starting the parallel work
/// costs more than it saves: at 1 million `u32` values, one thread found the ranges in 0.65 ms, 8
/// chunks of 125,000 in 0.53 ms, and 16 chunks of 62,500 in 0.83 ms.
// TODO Tune MIN_CHUNK_LEN by measurement; 2^18 is a judgment call between those data points.
const MIN_CHUNK_LEN: usize = 1 << 18;

/// Builds a [`RangeSetBlaze`] from a globally sorted slice, finding ranges on up to `threads`
/// threads (fewer for short slices; see [`MIN_CHUNK_LEN`]).
///
/// Only step 1 runs on the thread pool. Steps 2 and 3 run on the calling thread: they're one pass
/// over the (usually far fewer) ranges, and `RangeSetBlaze` can't cross threads for every `T`
/// anyway.
pub(super) fn from_sorted_slice<T>(sorted: &[T], threads: NonZeroUsize) -> RangeSetBlaze<T>
where
    T: SimdInteger + Send + Sync,
{
    let chunk_count = sorted.len().div_ceil(MIN_CHUNK_LEN).min(threads.get());
    match NonZeroUsize::new(chunk_count) {
        // One chunk (or an empty slice): no thread pool needed, so `Engine::Cpu(1)` never starts one.
        None | Some(NonZeroUsize::MIN) => {
            from_chunk_ranges(vec![sorted_slice_ranges(simd_level(), sorted)])
        }
        Some(chunk_count) => {
            from_chunk_ranges(thread_pool(threads).install(|| chunk_ranges(sorted, chunk_count)))
        }
    }
}

/// Step 1: splits `sorted` into about `chunk_count` contiguous chunks and finds each chunk's
/// ranges in parallel, with SIMD. Returns one vector per chunk, in slice order.
///
/// Each chunk uses the crate's existing SIMD range finder, which returns `(start, end)` pairs
/// (smaller than `RangeInclusive` and so faster to collect). For a sorted chunk they come out
/// sorted, disjoint, and not touching: exactly what [`SeamRanges`] needs.
pub(super) fn chunk_ranges<T>(sorted: &[T], chunk_count: NonZeroUsize) -> Vec<Vec<(T, T)>>
where
    T: SimdInteger + Send + Sync,
{
    // Detect once here rather than once per chunk.
    let level = simd_level();
    // `max(1)`: an empty slice still needs a positive chunk length (and yields no chunks).
    let chunk_len = sorted.len().div_ceil(chunk_count.get()).max(1);
    sorted
        .par_chunks(chunk_len)
        .map(|chunk| sorted_slice_ranges(level, chunk))
        .collect()
}

/// The SIMD level to use. Without `std`, `fearless_simd` can't detect CPU features at runtime;
/// this module needs `std` (for Rayon) anyway.
fn simd_level() -> Level {
    Level::try_detect().unwrap_or(Level::baseline())
}

/// Steps 2 and 3: joins the per-chunk ranges at the seams, lazily, and builds the set.
///
/// [`SeamRanges`] implements the [`SortedDisjoint`] marker itself, so no per-range
/// `CheckSortedDisjoint` wrapper is needed. On sparse input (10 million ranges), dropping that
/// wrapper cut this step from about 158 ms to 112 ms.
pub(super) fn from_chunk_ranges<T: Integer>(chunk_ranges: Vec<Vec<(T, T)>>) -> RangeSetBlaze<T> {
    RangeSetBlaze::from_sorted_disjoint(SeamRanges::new(chunk_ranges))
}

/// An iterator that turns per-chunk ranges into one sorted, disjoint stream, merging ranges only
/// at the seams between chunks.
///
/// The input must be one vector per contiguous chunk of a single sorted slice, in slice order:
/// each vector sorted, disjoint, and not touching, and every range in a vector ending at or before
/// the start of every range in later vectors. Empty vectors are fine. A seam whose ranges come
/// out of order panics.
///
/// Besides the input vectors and their iterators, the state is one pending range and one flag.
pub(super) struct SeamRanges<T> {
    /// The chunks not started yet.
    chunks: vec::IntoIter<Vec<(T, T)>>,
    /// The rest of the current chunk.
    chunk: vec::IntoIter<(T, T)>,
    /// The latest range, held back because the next chunk's first range might extend it.
    pending: Option<(T, T)>,
    /// Whether the next range read from `chunk` is the first of its chunk, the only kind of range
    /// that can merge into `pending`.
    at_seam: bool,
}

impl<T> SeamRanges<T> {
    pub(super) fn new(chunk_ranges: Vec<Vec<(T, T)>>) -> Self {
        Self {
            chunks: chunk_ranges.into_iter(),
            chunk: Vec::new().into_iter(),
            pending: None,
            at_seam: false,
        }
    }
}

impl<T: Integer> Iterator for SeamRanges<T> {
    type Item = RangeInclusive<T>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let Some(range) = self.chunk.next() else {
                // This chunk is used up: move to the next one, whose first range is at a seam.
                // An empty chunk just passes through here again, leaving `at_seam` set.
                let Some(chunk) = self.chunks.next() else {
                    return self.pending.take().map(|(start, end)| start..=end);
                };
                self.chunk = chunk.into_iter();
                self.at_seam = true;
                continue;
            };
            let at_seam = mem::replace(&mut self.at_seam, false);
            let Some(pending) = &mut self.pending else {
                self.pending = Some(range);
                continue;
            };
            if at_seam && continues(*pending, range) {
                // Sorted input means `range.1 >= range.0 >= pending.1`, so this only grows it.
                // The merged range may keep growing at later seams (for example, when the
                // next chunk is a single range that continues it too).
                pending.1 = range.1;
                continue;
            }
            // An interior range, or a seam that doesn't merge: emit the pending range.
            // Interior ranges are sorted and disjoint by construction (see the `SortedDisjoint`
            // impl below); in debug builds, check anyway. (`range.0 > pending.1 >=
            // T::min_value()`, so `sub_one` can't overflow.)
            debug_assert!(
                pending.1 < range.0 && pending.1 < range.0.sub_one(),
                "interior chunk ranges must be sorted, disjoint, and not touching"
            );
            let (start, end) = mem::replace(pending, range);
            return Some(start..=end);
        }
    }
}

impl<T: Integer> FusedIterator for SeamRanges<T> {}

// The `SortedDisjoint` contract (sorted, disjoint, not touching, nonempty) holds because:
// - Interior ranges come from `sorted_slice_ranges` on a sorted chunk, which merges touching and
//   overlapping neighbors, so within a chunk they're already sorted, disjoint, and not touching.
// - At each seam, `continues` asserts (in release builds too) that the chunks are in order, and
//   merges the two ranges exactly when they touch or overlap.
// Violating it is a logic error, not undefined behavior: the set would just hold wrong ranges.
impl<T: Integer> SortedStarts<T> for SeamRanges<T> {}
impl<T: Integer> SortedDisjoint<T> for SeamRanges<T> {}

/// Whether `next`, the first range after a seam, continues `pending`: it starts at the same value
/// (duplicates straddle the cut) or at the value right after `pending`'s end. Adjacency is the
/// type's own [`Integer::sub_one`], so `char` joins across the surrogates and `NotNan` floats join
/// across their missing `-0.0`, just as [`RangeSetBlaze::from_slice`] does.
fn continues<T: Integer>((_, pending_end): (T, T), (next_start, _): (T, T)) -> bool {
    assert!(
        pending_end <= next_start,
        "chunk ranges must come from consecutive chunks of one sorted slice"
    );
    // When `next_start > pending_end`, `next_start > T::min_value()`, so `sub_one` can't overflow.
    next_start == pending_end || next_start.sub_one() == pending_end
}

#[cfg(test)]
mod tests {
    use core::fmt::Debug;

    use std::vec;

    use rand::{Rng, SeedableRng, rngs::StdRng};

    use super::*;
    use crate::not_nan::nnf32;

    /// Chunk counts that cover one chunk, a few, many, and more chunks than elements.
    fn chunk_counts(len: usize) -> Vec<NonZeroUsize> {
        [1, 2, 3, 4, 7, 8, 16, 32, len, len + 1, 2 * len + 5]
            .into_iter()
            .filter_map(NonZeroUsize::new)
            .collect()
    }

    /// Checks that, for every chunk count, the parallel path matches the existing constructor
    /// on `values` (sorted first), and that the seam iterator's output is a valid sorted,
    /// disjoint stream.
    fn check<T: SimdInteger + Send + Sync + Debug>(values: &[T]) {
        let mut sorted = values.to_vec();
        sorted.sort_unstable();
        // The existing CPU engine sorts and then calls `from_slice`.
        let expected = RangeSetBlaze::from_slice(&sorted);
        let expected_ranges: Vec<RangeInclusive<T>> = expected.ranges().collect();
        for chunk_count in chunk_counts(sorted.len()) {
            let pieces = chunk_ranges(&sorted, chunk_count);
            assert!(pieces.len() <= chunk_count.get());
            let ranges: Vec<RangeInclusive<T>> = SeamRanges::new(pieces.clone()).collect();
            assert_valid_sorted_disjoint(&ranges);
            assert_eq!(ranges, expected_ranges, "chunk_count={chunk_count}");
            assert_eq!(from_chunk_ranges(pieces), expected);
            assert_eq!(from_sorted_slice(&sorted, chunk_count), expected);
        }
    }

    /// What [`RangeSetBlaze::from_sorted_disjoint`] needs: nonempty ranges, each ending more than
    /// one value before the next starts.
    fn assert_valid_sorted_disjoint<T: Integer + Debug>(ranges: &[RangeInclusive<T>]) {
        for range in ranges {
            assert!(range.start() <= range.end(), "empty range {range:?}");
        }
        for pair in ranges.windows(2) {
            let (left, right) = (&pair[0], &pair[1]);
            assert!(left.end() < right.start(), "{left:?} then {right:?}");
            // `right.start() > left.end() >= T::min_value()`, so `sub_one` can't overflow.
            assert!(
                *left.end() < right.start().sub_one(),
                "{left:?} touches {right:?}"
            );
        }
    }

    #[test]
    fn empty_and_tiny_inputs() {
        check::<u32>(&[]);
        check(&[5_u32]);
        check(&[5_u32, 5]);
        check(&[1_u32, 2]);
        check(&[1_u32, 3]);
        check(&[3_u32, 1, 2]);
    }

    #[test]
    fn all_identical_values() {
        check(&[7_u32; 1000]);
        check(&[i64::MIN; 300]);
        check(&[u8::MAX; 300]);
    }

    #[test]
    fn one_range_spanning_every_chunk() {
        check(&(0..10_000_u32).collect::<Vec<_>>());
        // The whole domain: every seam merges, and the range ends at `MAX`.
        check(&(i8::MIN..=i8::MAX).collect::<Vec<_>>());
        check(&(u8::MIN..=u8::MAX).collect::<Vec<_>>());
        // With duplicates, so that seams fall both inside runs of one value and between values.
        let doubled: Vec<u16> = (0..5000).flat_map(|value| [value, value]).collect();
        check(&doubled);
    }

    #[test]
    fn duplicates_crossing_chunk_boundaries() {
        // With 2, 3, or 4 chunks, the cuts land inside the runs of 0 and of 10.
        check(&[0_u32, 0, 0, 0, 10, 10, 10, 10]);
        check(&[0_u32, 0, 0, 0, 1, 1, 1, 1]);
        let mut clumps: Vec<i32> = (0..200).map(|index| index / 7 * 3).collect();
        clumps.extend([-5; 50]);
        check(&clumps);
    }

    #[test]
    fn adjacent_values_crossing_chunk_boundaries() {
        check(&[1_u32, 2]);
        check(&[1_u32, 2, 3, 4, 10, 11, 12, 13]);
        check(&[-2_i32, -1, 0, 1]);
    }

    #[test]
    fn multiple_and_sparse_ranges() {
        let mut multiple: Vec<u32> = Vec::new();
        for start in [0_u32, 100, 102, 1000, 5000] {
            multiple.extend(start..start + 37);
        }
        check(&multiple);
        // Every value its own range.
        check(&(0..4000_u32).map(|value| value * 2).collect::<Vec<_>>());
        check(
            &(0..4000_i64)
                .map(|value| value * 3 - 6000)
                .collect::<Vec<_>>(),
        );
    }

    #[test]
    fn integer_limits() {
        check(&[u32::MAX - 1, u32::MAX, u32::MAX, u32::MAX]);
        check(&[0_u32, 0, u32::MAX, u32::MAX]);
        check(&[i64::MIN, i64::MIN + 1, i64::MAX - 1, i64::MAX]);
        check(&[i64::MIN, i64::MIN, i64::MAX, i64::MAX]);
        check(&[u64::MAX; 9]);
        check(&[u128::MAX - 2, u128::MAX - 1, u128::MAX]);
        check(&[i128::MIN, i128::MIN + 1, 0, i128::MAX]);
        // A 128-bit run whose low half carries into its high half.
        check(
            &(0..300_u128)
                .map(|value| u128::from(u64::MAX) - 150 + value)
                .collect::<Vec<_>>(),
        );
    }

    #[test]
    fn exceptional_adjacency() {
        // `char` treats '\u{D7FF}' and '\u{E000}' as adjacent (the surrogates aren't chars).
        let around_surrogates: Vec<char> = (0xD7F0..=0xD7FF_u32)
            .chain(0xE000..=0xE00F)
            .map(|code| char::from_u32(code).expect("not a surrogate"))
            .collect();
        check(&around_surrogates);
        check(&['\u{D7FF}', '\u{E000}']);
        check(&['\u{D7FF}', '\u{D7FF}', '\u{E000}', '\u{E000}']);
        // `NotNan` has no `-0.0`, so the smallest negative subnormal is adjacent to `+0.0`.
        let tiny = |bits: u32| nnf32(f32::from_bits(bits));
        let around_zero = [
            tiny(0x8000_0002),
            tiny(0x8000_0001),
            tiny(0),
            tiny(1),
            tiny(2),
        ];
        check(&around_zero);
        check(&[tiny(0x8000_0001), tiny(0)]);
        check(&[tiny(0x8000_0001), tiny(0x8000_0001), tiny(0), tiny(0)]);
    }

    #[test]
    fn randomized_against_existing_constructor() {
        let mut rng = StdRng::seed_from_u64(0x5ea7_0a11);
        for _ in 0..300 {
            let len = rng.random_range(0..600);
            // Narrow domains give long runs and many duplicates; wide ones give sparse values.
            let domain = *[4_i64, 64, 1000, 1 << 40]
                .get(rng.random_range(0..4))
                .expect("index is in range");
            let offset = rng.random_range(-domain..=domain);
            let values: Vec<i64> = (0..len)
                .map(|_| offset + rng.random_range(0..domain))
                .collect();
            check(&values);
            // The same values, wrapped into smaller unsigned types.
            let bytes: Vec<u8> = values
                .iter()
                .map(|&value| u8::try_from(value.rem_euclid(1 << 8)).expect("wrapped into u8"))
                .collect();
            check(&bytes);
            let words: Vec<u32> = values
                .iter()
                .map(|&value| u32::try_from(value.rem_euclid(1 << 32)).expect("wrapped into u32"))
                .collect();
            check(&words);
        }
    }

    // Iterator-specific tests, on hand-built chunk ranges.

    fn seams(chunk_ranges: Vec<Vec<(u32, u32)>>) -> Vec<RangeInclusive<u32>> {
        SeamRanges::new(chunk_ranges).collect()
    }

    #[test]
    fn seam_iterator_empty() {
        assert_eq!(seams(vec![]), []);
        assert_eq!(seams(vec![vec![], vec![], vec![]]), []);
    }

    #[test]
    fn seam_iterator_emits_interior_ranges_and_non_merging_seams() {
        assert_eq!(
            seams(vec![vec![(0, 1), (5, 6)], vec![(9, 9), (20, 21)]]),
            [0..=1, 5..=6, 9..=9, 20..=21]
        );
    }

    #[test]
    fn seam_iterator_merges_overlapping_and_adjacent_seams() {
        // Overlap: the value 6 straddles the cut. Adjacent: 9 then 10.
        assert_eq!(
            seams(vec![
                vec![(0, 1), (5, 6)],
                vec![(6, 9)],
                vec![(10, 12), (20, 21)]
            ]),
            [0..=1, 5..=12, 20..=21]
        );
    }

    #[test]
    fn seam_iterator_single_range_chunk_merges_with_both_neighbors() {
        // The middle chunks each hold one range that joins both the range before and after,
        // and empty chunks in between don't break the chain.
        assert_eq!(
            seams(vec![
                vec![(0, 3)],
                vec![(4, 5)],
                vec![],
                vec![(5, 9)],
                vec![],
                vec![(10, 10), (20, 25)],
                vec![(26, 30)],
            ]),
            [0..=10, 20..=30]
        );
    }

    #[test]
    fn seam_iterator_is_fused() {
        let mut iter = SeamRanges::new(vec![vec![(1_u32, 2)]]);
        assert_eq!(iter.next(), Some(1..=2));
        assert_eq!(iter.next(), None);
        assert_eq!(iter.next(), None);
    }

    #[test]
    #[should_panic(expected = "consecutive chunks of one sorted slice")]
    fn seam_iterator_panics_on_out_of_order_chunks() {
        seams(vec![vec![(10, 20)], vec![(0, 5)]]);
    }

    #[test]
    #[should_panic(expected = "consecutive chunks of one sorted slice")]
    fn seam_iterator_panics_on_overlapping_chunks() {
        seams(vec![vec![(0, 20)], vec![(10, 30)]]);
    }

    // The seam iterator trusts interior ranges in release builds; debug builds check them.
    #[test]
    #[cfg(debug_assertions)]
    #[should_panic(expected = "interior chunk ranges must be sorted, disjoint, and not touching")]
    fn from_chunk_ranges_rejects_unsorted_interior_ranges() {
        from_chunk_ranges(vec![vec![(10_u32, 20), (0, 5)]]);
    }

    #[test]
    #[cfg(debug_assertions)]
    #[should_panic(expected = "interior chunk ranges must be sorted, disjoint, and not touching")]
    fn from_chunk_ranges_rejects_touching_interior_ranges() {
        from_chunk_ranges(vec![vec![(0_u32, 5), (6, 9)]]);
    }
}

/// A release-mode benchmark that times each stage separately. Run it with:
///
/// `cargo test --release --features shuffled_slices --lib parallel_ranges::benchmark -- --ignored --nocapture`
///
/// Set `PARALLEL_RANGES_SIZES` (comma separated, default `1000000,10000000,100000000`) and
/// `PARALLEL_RANGES_CHUNKS` (default `1,8,16,32`) to change what runs.
///
/// It lives in the crate, rather than in `experiments/gpu_ingest.rs`, because the stage
/// timings need this module's internals. It uses `gpu_ingest`'s workloads, seed, warm-up, and
/// median rules, and it times the existing engines through the same `from_shuffled_slice`
/// function the public API calls.
#[cfg(test)]
mod benchmark {
    use core::time::Duration;
    use std::{env, hint::black_box, string::String, time::Instant};

    use super::*;
    use crate::gpu::{
        Engine, GpuElement, cpu_radix, from_shuffled_slice, sort_in_place, thread_pool,
    };

    const SEED: u64 = 0xd1b5_4a32_d192_ed03;
    const CLUMP_LEN: u64 = 256;
    const DUPLICATE_REPEATS: u64 = 16;
    const WARM_UP: Duration = Duration::from_millis(1000);
    /// Above this size, skip the one-thread sorts and totals (each takes many seconds and
    /// shows only that one thread sorts slowly).
    const LARGEST_ONE_THREAD_TOTAL: u64 = 10_000_000;

    trait BenchKey: GpuElement + SimdInteger + Send + Sync {
        const NAME: &'static str;
        fn from_index(index: u64) -> Self;
    }

    impl BenchKey for u32 {
        const NAME: &'static str = "u32";
        fn from_index(index: u64) -> Self {
            Self::try_from(index).expect("index fits in u32")
        }
    }

    impl BenchKey for u64 {
        const NAME: &'static str = "u64";
        // As in `gpu_ingest`: start just below 2^32, so runs cross into the high word.
        fn from_index(index: u64) -> Self {
            0xffff_ff00 + index
        }
    }

    #[derive(Clone, Copy, Debug)]
    enum Workload {
        /// `n` distinct values in clumps of 256: `n / 256` ranges.
        Distinct,
        /// `n / 16` distinct values in clumps of 256, each repeated 16 times.
        Duplicates,
        /// `n` distinct values, each its own range: the most ranges `n` values can make.
        Sparse,
    }

    #[test]
    #[ignore = "benchmark; run in release mode with --ignored --nocapture"]
    fn benchmark() {
        let sizes = list_from_env(
            "PARALLEL_RANGES_SIZES",
            &[1_000_000, 10_000_000, 100_000_000],
        );
        let chunk_counts = list_from_env("PARALLEL_RANGES_CHUNKS", &[1, 8, 16, 32]);
        std::println!(
            "workload,type,n,ranges,threads,sort_cmp_ms,sort_radix_ms,from_slice_1t_ms,\
             ranges_parallel_ms,seam_only_ms,seam_build_ms,total_cpu_old_ms,total_cpu_ms,\
             total_radix_old_ms,total_radix_ms"
        );
        for &n in &sizes {
            for workload in [Workload::Duplicates, Workload::Distinct, Workload::Sparse] {
                run::<u32>(workload, n, &chunk_counts);
                run::<u64>(workload, n, &chunk_counts);
            }
        }
    }

    /// Times the stages *inside* single end-to-end radix-sort calls, for the existing path
    /// (sort, `from_slice`) and the new one (sort, parallel ranges, seams + build), plus freeing
    /// the result. The isolated stage timings in [`benchmark`] run each stage on warm, reused
    /// inputs, which can hide costs (cold caches, fresh allocations) that a real call pays.
    ///
    /// `cargo test --release --features shuffled_slices --lib parallel_ranges::benchmark::split -- --ignored --nocapture`
    ///
    /// Uses `PARALLEL_RANGES_SIZES` (default `10000000`) and `PARALLEL_RANGES_CHUNKS` (default `16`).
    #[test]
    #[ignore = "benchmark; run in release mode with --ignored --nocapture"]
    fn split() {
        let sizes = list_from_env("PARALLEL_RANGES_SIZES", &[10_000_000]);
        let chunk_counts = list_from_env("PARALLEL_RANGES_CHUNKS", &[16]);
        std::println!(
            "workload,type,n,ranges,threads,old_sort_ms,old_from_slice_ms,old_free_ms,old_total_ms,\
             new_sort_ms,new_ranges_ms,new_seam_build_ms,new_free_ms,new_total_ms"
        );
        for &n in &sizes {
            for &threads in &chunk_counts {
                let threads =
                    NonZeroUsize::new(usize::try_from(threads).expect("fits")).expect("nonzero");
                for workload in [Workload::Duplicates, Workload::Distinct, Workload::Sparse] {
                    split_one::<u32>(workload, n, threads);
                    split_one::<u64>(workload, n, threads);
                }
            }
        }
    }

    fn split_one<T: BenchKey>(workload: Workload, n: u64, threads: NonZeroUsize) {
        const SAMPLES: usize = 7;
        let shuffled = values::<T>(workload, n);
        let pool = thread_pool(threads);
        let mut old_laps: Vec<[Duration; 3]> = Vec::new();
        let mut new_laps: Vec<[Duration; 4]> = Vec::new();
        let mut ranges = 0;
        // The first round of each warms up and is discarded.
        for round in 0..=SAMPLES {
            let mut copy = shuffled.clone();
            let start = Instant::now();
            pool.install(|| cpu_radix::sort_by_key(&mut copy, T::encode));
            let sorted_at = Instant::now();
            let set = RangeSetBlaze::from_slice(black_box(&copy));
            let built_at = Instant::now();
            ranges = set.ranges_len();
            drop(set);
            let freed_at = Instant::now();
            if round > 0 {
                old_laps.push([sorted_at - start, built_at - sorted_at, freed_at - built_at]);
            }

            let mut copy = shuffled.clone();
            let start = Instant::now();
            pool.install(|| cpu_radix::sort_by_key(&mut copy, T::encode));
            let sorted_at = Instant::now();
            let pieces = pool.install(|| chunk_ranges(black_box(&copy), threads));
            let ranged_at = Instant::now();
            let set = from_chunk_ranges(pieces);
            let built_at = Instant::now();
            assert_eq!(set.ranges_len(), ranges);
            drop(set);
            let freed_at = Instant::now();
            if round > 0 {
                new_laps.push([
                    sorted_at - start,
                    ranged_at - sorted_at,
                    built_at - ranged_at,
                    freed_at - built_at,
                ]);
            }
        }
        let old_cells: Vec<String> = (0..3)
            .map(|stage| cell(Some(median_of(old_laps.iter().map(|laps| laps[stage])))))
            .collect();
        let new_cells: Vec<String> = (0..4)
            .map(|stage| cell(Some(median_of(new_laps.iter().map(|laps| laps[stage])))))
            .collect();
        let old_total = cell(Some(median_of(
            old_laps.iter().map(|laps| laps.iter().sum()),
        )));
        let new_total = cell(Some(median_of(
            new_laps.iter().map(|laps| laps.iter().sum()),
        )));
        std::println!(
            "{workload:?},{},{n},{ranges},{threads},{},{old_total},{},{new_total}",
            T::NAME,
            old_cells.join(","),
            new_cells.join(","),
        );
    }

    fn median_of(durations: impl Iterator<Item = Duration>) -> Duration {
        let mut durations: Vec<Duration> = durations.collect();
        durations.sort_unstable();
        durations[durations.len() / 2]
    }

    fn run<T: BenchKey>(workload: Workload, n: u64, chunk_counts: &[u64]) {
        let shuffled = values::<T>(workload, n);
        let mut sorted = shuffled.clone();
        sorted.sort_unstable();
        let expected = RangeSetBlaze::from_slice(&sorted);
        let samples = if n >= 100_000_000 { 3 } else { 5 };
        // The existing one-threaded range finder on already-sorted data: what step 2 replaces.
        let from_slice_1t = median(
            samples,
            || (),
            |()| RangeSetBlaze::from_slice(black_box(&sorted)),
        );
        for &threads in chunk_counts {
            let threads =
                NonZeroUsize::new(usize::try_from(threads).expect("fits")).expect("nonzero");
            let pool = thread_pool(threads);
            let skip_sorts = threads.get() == 1 && n > LARGEST_ONE_THREAD_TOTAL;

            // Every candidate must build the same set.
            assert_eq!(from_sorted_slice(&sorted, threads), expected);

            let sort_cmp = (!skip_sorts).then(|| {
                median(
                    samples,
                    || shuffled.clone(),
                    |mut copy| sort_in_place(&mut copy, threads),
                )
            });
            let sort_radix = (!skip_sorts).then(|| {
                median(
                    samples,
                    || shuffled.clone(),
                    |mut copy| pool.install(|| cpu_radix::sort_by_key(&mut copy, T::encode)),
                )
            });
            let ranges_parallel = median(
                samples,
                || (),
                |()| pool.install(|| chunk_ranges(black_box(&sorted), threads)),
            );
            let pieces = pool.install(|| chunk_ranges(&sorted, threads));
            let seam_only = median(
                samples,
                || pieces.clone(),
                |pieces| SeamRanges::new(pieces).count(),
            );
            let seam_build = median(samples, || pieces.clone(), from_chunk_ranges);

            let engine_total = |engine: Engine| {
                (!skip_sorts).then(|| {
                    assert_eq!(from_shuffled_slice(&mut shuffled.clone(), engine), expected);
                    median(
                        samples,
                        || shuffled.clone(),
                        |mut copy| from_shuffled_slice(&mut copy, engine),
                    )
                })
            };
            // The engines' previous path: the same sort, then the one-threaded `from_slice`.
            let old_total = |radix: bool| {
                let build = |values: &mut [T]| {
                    if radix {
                        pool.install(|| cpu_radix::sort_by_key(values, T::encode));
                    } else {
                        sort_in_place(values, threads);
                    }
                    RangeSetBlaze::from_slice(&*values)
                };
                (!skip_sorts).then(|| {
                    assert_eq!(build(&mut shuffled.clone()), expected);
                    median(samples, || shuffled.clone(), |mut copy| build(&mut copy))
                })
            };
            let total_cpu_old = old_total(false);
            let total_cpu = engine_total(Engine::Cpu(threads));
            let total_radix_old = old_total(true);
            let total_radix = engine_total(Engine::CpuRadix(threads));

            std::println!(
                "{workload:?},{},{n},{},{threads},{},{},{},{},{},{},{},{},{},{}",
                T::NAME,
                expected.ranges_len(),
                cell(sort_cmp),
                cell(sort_radix),
                cell(Some(from_slice_1t)),
                cell(Some(ranges_parallel)),
                cell(Some(seam_only)),
                cell(Some(seam_build)),
                cell(total_cpu_old),
                cell(total_cpu),
                cell(total_radix_old),
                cell(total_radix),
            );
        }
    }

    /// `gpu_ingest`'s data layout, shuffled with its seed.
    fn values<T: BenchKey>(workload: Workload, n: u64) -> Vec<T> {
        let index_of = |position: u64| match workload {
            Workload::Distinct => clump_index(position),
            Workload::Duplicates => clump_index(position / DUPLICATE_REPEATS),
            Workload::Sparse => position * 2,
        };
        let mut values: Vec<T> = (0..n)
            .map(|position| T::from_index(index_of(position)))
            .collect();
        shuffle(&mut values, SEED);
        values
    }

    /// The index of the `j`-th distinct value: clumps of `CLUMP_LEN`, a gap of one between them.
    const fn clump_index(j: u64) -> u64 {
        j + j / CLUMP_LEN
    }

    /// Fisher–Yates shuffle driven by xorshift from `seed` (as in `gpu_ingest`).
    fn shuffle<T>(values: &mut [T], seed: u64) {
        let mut state = seed;
        for index in (1..values.len()).rev() {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            let modulus = u64::try_from(index + 1).expect("index exceeded u64");
            let swap_index = usize::try_from(state % modulus).expect("swap index exceeded usize");
            values.swap(index, swap_index);
        }
    }

    /// The median of `samples` timed runs, after warming up for at least [`WARM_UP`]. `setup`
    /// runs outside the timer; freeing the result is inside it (as in `gpu_ingest`).
    fn median<S, R>(
        samples: usize,
        mut setup: impl FnMut() -> S,
        mut operation: impl FnMut(S) -> R,
    ) -> Duration {
        let warm_up = Instant::now();
        drop(operation(setup()));
        while warm_up.elapsed() < WARM_UP {
            drop(operation(setup()));
        }
        let mut times: Vec<Duration> = (0..samples)
            .map(|_| {
                let input = setup();
                let start = Instant::now();
                drop(operation(input));
                start.elapsed()
            })
            .collect();
        times.sort_unstable();
        times[times.len() / 2]
    }

    fn cell(duration: Option<Duration>) -> String {
        duration.map_or_else(String::new, |duration| {
            std::format!("{:.3}", duration.as_secs_f64() * 1_000.0)
        })
    }

    fn list_from_env(name: &str, default: &[u64]) -> Vec<u64> {
        env::var(name).map_or_else(
            |_| default.to_vec(),
            |list| {
                list.split(',')
                    .map(|item| {
                        item.trim()
                            .parse()
                            .expect("a comma-separated list of positive integers")
                    })
                    .collect()
            },
        )
    }
}
