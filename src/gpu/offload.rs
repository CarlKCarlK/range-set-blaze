//! Host adapter for the separately compiled, core-only Rust offload kernels.

use std::env::{VarError, var, var_os};
use std::vec;
use std::vec::Vec;

unsafe extern "C" {
    fn rsb_offload_devices() -> i32;
    fn rsb_offload_sort(keys: *mut u64, len: usize);
    fn rsb_offload_radix_sort(
        keys: *mut u64,
        scratch: *mut u64,
        workspace: *mut u64,
        len: usize,
        bits: u32,
    );
    fn rsb_offload_radix_ranges(
        keys: *mut u64,
        scratch: *mut u64,
        workspace: *mut u64,
        len: usize,
        bits: u32,
        gap_left: u64,
        gap_right: u64,
        has_gap: u32,
        count: *mut usize,
    ) -> u32;
}

// Where rsb_offload_radix_ranges left its result (keep in sync with radix.rs).
const RANGES_SORTED_IN_KEYS: u32 = 0;
const RANGES_IN_KEYS: u32 = 1;
const RANGES_IN_SCRATCH: u32 = 2;
const RANGES_ALL_EQUAL: u32 = 3;

pub(super) fn available() -> bool {
    // SAFETY: the native helper takes no arguments and only queries LLVM's runtime.
    let available = unsafe { rsb_offload_devices() > 0 };
    if var_os("RANGE_SET_BLAZE_REQUIRE_OFFLOAD_GPU").is_some() {
        assert!(available, "the required offload GPU is unavailable");
    }
    available
}

pub(super) fn sort(keys: &mut Vec<u64>) {
    sort_bits(keys, 64);
}

pub(super) fn sort_bits(keys: &mut Vec<u64>, bits: u32) {
    assert!(available(), "Engine::Offload: no offload GPU is available");
    let len = keys.len();
    if len < 2 {
        return;
    }
    assert!(
        matches!(bits, 32 | 64),
        "offload keys must have 32 or 64 bits"
    );
    assert!(len <= 1 << 30, "Engine::Offload: input exceeds 2^30 keys");
    let use_radix = use_radix(len);
    if use_radix {
        sort_radix(keys, bits);
        return;
    }
    let padded = len
        .checked_next_power_of_two()
        .expect("offload size overflow");
    assert!(
        padded <= 1 << 30,
        "Engine::Offload: input exceeds 2^30 keys"
    );
    keys.resize(padded, u64::MAX);
    // SAFETY: `keys` is exclusively borrowed, initialized, and has power-of-two length
    // between 2 and 2^30. The native helper completes all launches before returning.
    unsafe { rsb_offload_sort(keys.as_mut_ptr(), keys.len()) };
    // Padding is maximal, so removing it also works when real keys equal u64::MAX.
    keys.truncate(len);
}

/// Radix above 1,024 keys; `RANGE_SET_BLAZE_OFFLOAD_SORT` (`radix` or `bitonic`) overrides.
fn use_radix(len: usize) -> bool {
    match var("RANGE_SET_BLAZE_OFFLOAD_SORT") {
        Ok(value) if value == "radix" => true,
        Ok(value) if value == "bitonic" => false,
        Ok(_) => panic!("RANGE_SET_BLAZE_OFFLOAD_SORT must be radix or bitonic"),
        Err(VarError::NotPresent) => len > 1024,
        Err(error) => panic!("invalid offload sort setting: {error}"),
    }
}

/// Sorts keys on the GPU and returns their ranges as `(first, last)` key pairs. Keys
/// `gap.0` and `gap.1` also count as adjacent. With radix sort, the GPU finds the
/// ranges too, and only they return to the host.
pub(super) fn ranges_bits(
    keys: &mut Vec<u64>,
    bits: u32,
    gap: Option<(u64, u64)>,
) -> Vec<(u64, u64)> {
    let len = keys.len();
    if len < 2 || !use_radix(len) {
        sort_bits(keys, bits);
        return host_ranges(keys, gap);
    }
    assert!(available(), "Engine::Offload: no offload GPU is available");
    check_radix_input(keys, bits);
    let mut scratch = vec![0_u64; len];
    let mut workspace = vec![0_u64; 1 + len.div_ceil(4096) * 256];
    let (gap_left, gap_right) = gap.unwrap_or((0, 0));
    let mut count = 0;
    // SAFETY: all three initialized allocations are disjoint and exclusively borrowed
    // until synchronous kernels finish. Lengths and significant bits are checked above.
    let location = unsafe {
        rsb_offload_radix_ranges(
            keys.as_mut_ptr(),
            scratch.as_mut_ptr(),
            workspace.as_mut_ptr(),
            len,
            bits,
            gap_left,
            gap_right,
            u32::from(gap.is_some()),
            &raw mut count,
        )
    };
    let pairs = match location {
        RANGES_SORTED_IN_KEYS => return host_ranges(keys, gap),
        RANGES_IN_KEYS => &keys[..2 * count],
        RANGES_IN_SCRATCH => &scratch[..2 * count],
        RANGES_ALL_EQUAL => return vec![(keys[0], keys[0])],
        _ => unreachable!("unknown offload ranges location {location}"),
    };
    pairs
        .as_chunks::<2>()
        .0
        .iter()
        .map(|&pair| pair.into())
        .collect()
}

/// Ranges of sorted keys, on the host; the same rule as the GPU's `ranges.rs`.
fn host_ranges(sorted: &[u64], gap: Option<(u64, u64)>) -> Vec<(u64, u64)> {
    let joins = |left: u64, right: u64| {
        left == right || left.checked_add(1) == Some(right) || gap == Some((left, right))
    };
    let mut ranges: Vec<(u64, u64)> = Vec::new();
    for &key in sorted {
        match ranges.last_mut() {
            Some((_, last)) if joins(*last, key) => *last = key,
            _ => ranges.push((key, key)),
        }
    }
    ranges
}

/// The radix kernels' preconditions: their unsafe code relies on these.
fn check_radix_input(keys: &[u64], bits: u32) {
    assert!(
        matches!(bits, 32 | 64),
        "offload key width must be 32 or 64"
    );
    assert!(
        (2..=1 << 30).contains(&keys.len()),
        "invalid offload radix length"
    );
    assert!(
        bits == 64 || keys.iter().all(|&key| u32::try_from(key).is_ok()),
        "offload 32-bit keys exceed u32"
    );
}

fn sort_radix(keys: &mut [u64], bits: u32) {
    check_radix_input(keys, bits);
    let mut scratch = vec![0_u64; keys.len()];
    let mut workspace = vec![0_u64; 1 + keys.len().div_ceil(4096) * 256];
    // SAFETY: all three initialized allocations are disjoint and exclusively borrowed
    // until synchronous kernels finish. Lengths and significant bits are checked above.
    unsafe {
        rsb_offload_radix_sort(
            keys.as_mut_ptr(),
            scratch.as_mut_ptr(),
            workspace.as_mut_ptr(),
            keys.len(),
            bits,
        );
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Engine, RangeMapBlaze, RangeSetBlaze};

    #[test]
    fn radix_matches_cpu_across_tiles_and_constant_bytes() {
        if !available() {
            return;
        }
        for len in [
            2_usize, 3, 31, 32, 255, 256, 4095, 4096, 4097, 65537, 1_000_003,
        ] {
            for pattern in 0..5 {
                let mut keys: Vec<u64> = (0..len)
                    .map(|index| {
                        let random = (index as u64).wrapping_mul(0x9e37_79b9_7f4a_7c15);
                        match pattern {
                            0 => random,
                            1 => random & u64::from(u32::MAX),
                            2 => 0xabcd_1234_5678_0000 | (random & 1023),
                            3 => u64::MAX,
                            _ => 42,
                        }
                    })
                    .collect();
                let mut expected = keys.clone();
                expected.sort_unstable();
                sort_radix(&mut keys, if pattern == 1 { 32 } else { 64 });
                assert_eq!(keys, expected, "len={len}, pattern={pattern}");
            }
        }
    }

    #[test]
    fn gpu_ranges_match_host_ranges() {
        if !available() {
            return;
        }
        let gap = Some((0xd7ff, 0xe000));
        for len in [1025_usize, 4095, 4096, 4097, 65537, 1_000_003] {
            for pattern in 0..7 {
                let keys: Vec<u64> = (0..len as u64)
                    .map(|index| {
                        let random = index.wrapping_mul(0x9e37_79b9_7f4a_7c15);
                        match pattern {
                            // Clumps of 256 with one-key gaps, shuffled.
                            0 => (random % len as u64) / 256 * 257 + random % 256,
                            // Mostly isolated keys: ranges outgrow the keys (host fallback).
                            1 => random & u64::from(u32::MAX),
                            // Many duplicates within a few ranges.
                            2 => random % 1000 + (random % 7) * 5000,
                            // Keys across the char surrogate gap, which joins them.
                            3 => 0xd7f0 + random % 16 + (random % 2) * 0x0810,
                            // Every key equal: no radix pass runs.
                            4 => 42,
                            // The largest keys, where key + 1 would overflow.
                            5 => u64::MAX - random % 300,
                            // Consecutive keys: one range spanning every tile.
                            _ => index,
                        }
                    })
                    .collect();
                let bits = if keys.iter().all(|&key| u32::try_from(key).is_ok()) {
                    32
                } else {
                    64
                };
                let mut sorted = keys.clone();
                sorted.sort_unstable();
                let expected = host_ranges(&sorted, gap);
                let mut keys = keys;
                let actual = ranges_bits(&mut keys, bits, gap);
                assert_eq!(actual, expected, "len={len}, pattern={pattern}");
            }
        }
    }

    #[test]
    #[should_panic(expected = "unsupported key type")]
    fn strict_engine_rejects_unsupported_set_keys() {
        drop(RangeSetBlaze::<u128>::from_shuffled_slice(
            &mut [1],
            Engine::Offload,
        ));
    }

    #[test]
    #[should_panic(expected = "unsupported key type")]
    fn strict_engine_rejects_unsupported_map_keys() {
        drop(RangeMapBlaze::from_shuffled_slices(
            &[1_u64],
            &[1],
            Engine::Offload,
        ));
    }

    #[test]
    fn sort_matches_cpu_when_available() {
        if !available() {
            return;
        }
        for len in [0_u64, 1, 2, 3, 255, 256, 257, 1023, 1025] {
            let mut keys: Vec<u64> = (0..len)
                .map(|index| index.wrapping_mul(0x9e37_79b9_7f4a_7c15))
                .collect();
            keys.extend([u64::MAX, 0, u64::MAX, 0]);
            let mut expected = keys.clone();
            expected.sort_unstable();
            sort(&mut keys);
            assert_eq!(keys, expected);
        }
    }

    #[test]
    fn reused_host_allocation_gets_fresh_data_and_releases_its_mapping() {
        if !available() {
            return;
        }
        // Reserve the padding too, so every call maps exactly the same host address.
        let mut keys = Vec::with_capacity(1024);
        keys.resize(513, 0_u64);
        let address = keys.as_ptr();
        for seed in 0..4_u64 {
            for (index, key) in keys.iter_mut().enumerate() {
                *key = (index as u64).wrapping_mul(0x9e37_79b9_7f4a_7c15) ^ seed;
            }
            let mut expected = keys.clone();
            expected.sort_unstable();
            sort(&mut keys);
            assert_eq!(keys.as_ptr(), address);
            assert_eq!(keys, expected);
        }
    }
}
