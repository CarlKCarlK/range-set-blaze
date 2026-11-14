//! A parallel radix sort for the CPU, so [`Engine::CpuRadix`](super::Engine::CpuRadix)
//! compares against the GPU engines with the same kind of sort they use. Safe Rust only.
//!
//! 1. Split by the highest 8 significant key bits, in parallel: each thread bucket-sorts
//!    its chunk into its own buffer, then each bucket gathers its pieces into its own
//!    disjoint slice of the output. Using the bits just below the highest set bit keeps
//!    narrow key ranges spread over all 256 buckets.
//! 2. Sort each bucket in parallel with a least-significant-digit radix sort over the
//!    remaining bits, skipping passes whose digit is the same for every key.
//!
//! Both steps are stable, so equal keys keep their input order.

use rayon::iter::{
    IndexedParallelIterator, IntoParallelIterator, IntoParallelRefIterator, ParallelIterator,
};
use rayon::slice::ParallelSlice;
use std::vec;
use std::vec::Vec;

const RADIX: usize = 256;
/// Below this, splitting across threads costs more than it saves.
const PARALLEL_MIN: usize = 1 << 16;
/// Buckets this small sort faster with a comparison sort than with counting passes.
const SMALL_BUCKET: usize = 64;

/// Sorts `values` by `key`, which must order them as `Ord` does. Call it inside the
/// Rayon pool whose threads should do the work.
pub(super) fn sort_by_key<T, F>(values: &mut [T], key: F)
where
    T: Copy + Send + Sync,
    F: Fn(T) -> u64 + Sync,
{
    if values.len() < PARALLEL_MIN {
        values.sort_unstable_by_key(|&value| key(value));
        return;
    }
    let significant = values
        .par_iter()
        .map(|&value| key(value))
        .reduce(|| 0, |left, right| left | right);
    let bits = u64::BITS - significant.leading_zeros();
    if bits == 0 {
        return; // Every key is zero.
    }
    let shift = bits.saturating_sub(8);
    let sizes = split_by_top_digit(values, &key, shift);
    let mut buckets = Vec::with_capacity(RADIX);
    let mut rest = &mut *values;
    for size in sizes {
        let (bucket, tail) = rest.split_at_mut(size);
        buckets.push(bucket);
        rest = tail;
    }
    buckets
        .into_par_iter()
        .for_each(|bucket| sort_low_digits(bucket, &key, shift));
}

const fn digit(key: u64, shift: u32) -> usize {
    ((key >> shift) & 0xff) as usize
}

/// Reorders `values` so their top digits ascend, keeping input order within a digit.
/// Returns how many values have each top digit.
fn split_by_top_digit<T, F>(values: &mut [T], key: &F, shift: u32) -> [usize; RADIX]
where
    T: Copy + Send + Sync,
    F: Fn(T) -> u64 + Sync,
{
    let chunk_len = values.len().div_ceil(rayon::current_num_threads() * 4);
    // Each chunk, counting-sorted into its own buffer, with where each digit starts.
    let pieces: Vec<(Vec<T>, [usize; RADIX + 1])> = values
        .par_chunks(chunk_len)
        .map(|chunk| {
            let mut starts = [0_usize; RADIX + 1];
            for &value in chunk {
                starts[digit(key(value), shift) + 1] += 1;
            }
            for index in 0..RADIX {
                starts[index + 1] += starts[index];
            }
            let mut next = starts;
            let mut sorted = vec![chunk[0]; chunk.len()];
            for &value in chunk {
                let slot = &mut next[digit(key(value), shift)];
                sorted[*slot] = value;
                *slot += 1;
            }
            (sorted, starts)
        })
        .collect();
    // Hand each digit its own slice of `values`, then gather its pieces in chunk order.
    let mut sizes = [0_usize; RADIX];
    for (bucket, size) in sizes.iter_mut().enumerate() {
        *size = pieces
            .iter()
            .map(|(_, starts)| starts[bucket + 1] - starts[bucket])
            .sum();
    }
    let mut buckets = Vec::with_capacity(RADIX);
    let mut rest = &mut *values;
    for size in sizes {
        let (head, tail) = rest.split_at_mut(size);
        buckets.push(head);
        rest = tail;
    }
    buckets
        .into_par_iter()
        .enumerate()
        .for_each(|(bucket, destination)| {
            let mut at = 0;
            for (sorted, starts) in &pieces {
                let piece = &sorted[starts[bucket]..starts[bucket + 1]];
                destination[at..at + piece.len()].copy_from_slice(piece);
                at += piece.len();
            }
        });
    sizes
}

/// Sorts one bucket by the key bits below `shift`. Buckets fit in cache, so digits up to
/// 11 bits wide save passes; one read counts every pass's digits, as the GPU sort does.
fn sort_low_digits<T, F>(bucket: &mut [T], key: &F, shift: u32)
where
    T: Copy,
    F: Fn(T) -> u64,
{
    if bucket.len() <= SMALL_BUCKET {
        bucket.sort_by_key(|&value| key(value));
        return;
    }
    if shift == 0 {
        return;
    }
    let passes = shift.div_ceil(11);
    let width = shift.div_ceil(passes);
    let radix = 1_usize << width;
    let mask = (radix - 1) as u64;
    // The mask keeps digits below 2^11, so the conversion can't fail.
    let low_digit = |value: T, pass: u32| {
        usize::try_from((key(value) >> (pass * width)) & mask).expect("masked digit fits")
    };
    let mut counts = vec![0_usize; passes as usize * radix];
    for &value in &*bucket {
        for pass in 0..passes {
            counts[pass as usize * radix + low_digit(value, pass)] += 1;
        }
    }
    let mut scratch = vec![bucket[0]; bucket.len()];
    let mut in_bucket = true;
    for pass in 0..passes {
        let starts = &mut counts[pass as usize * radix..(pass as usize + 1) * radix];
        if starts.contains(&bucket.len()) {
            continue; // Every key has the same digit here; the order already holds.
        }
        let mut total = 0;
        for start in starts.iter_mut() {
            let count = *start;
            *start = total;
            total += count;
        }
        let (from, to): (&[T], &mut [T]) = if in_bucket {
            (bucket, &mut scratch)
        } else {
            (&scratch, bucket)
        };
        for &value in from {
            let slot = &mut starts[low_digit(value, pass)];
            to[*slot] = value;
            *slot += 1;
        }
        in_bucket = !in_bucket;
    }
    if !in_bucket {
        bucket.copy_from_slice(&scratch);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matches_sort_unstable() {
        for len in [
            0_usize,
            1,
            63,
            65,
            1000,
            PARALLEL_MIN - 1,
            PARALLEL_MIN,
            300_007,
        ] {
            for pattern in 0..6 {
                let keys: Vec<u64> = (0..len as u64)
                    .map(|index| {
                        let random = index.wrapping_mul(0x9e37_79b9_7f4a_7c15);
                        match pattern {
                            0 => random >> 32,  // full u32 range
                            1 => random % 1000, // narrow range, many duplicates
                            2 => (random % len.max(1) as u64) / 256 * 257 + random % 256,
                            3 => 7,                        // every key equal
                            4 => random,                   // full u64 range
                            _ => u64::MAX - random % 5000, // near the top
                        }
                    })
                    .collect();
                let mut expected = keys.clone();
                expected.sort_unstable();
                let mut actual = keys;
                sort_by_key(&mut actual, |key| key);
                assert_eq!(actual, expected, "len={len}, pattern={pattern}");
            }
        }
    }

    #[test]
    fn is_stable() {
        // Sort (key, input position) pairs by key alone: positions must stay ascending.
        let mut pairs: Vec<(u64, u32)> = (0..200_000_u32)
            .map(|position| {
                (
                    u64::from(position.wrapping_mul(2_654_435_761) % 997),
                    position,
                )
            })
            .collect();
        sort_by_key(&mut pairs, |(key, _)| key);
        assert!(
            pairs
                .windows(2)
                .all(|pair| pair[0].0 < pair[1].0
                    || (pair[0].0 == pair[1].0 && pair[0].1 < pair[1].1))
        );
    }
}
