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
}

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
    let use_radix = match var("RANGE_SET_BLAZE_OFFLOAD_SORT") {
        Ok(value) if value == "radix" => true,
        Ok(value) if value == "bitonic" => false,
        Ok(_) => panic!("RANGE_SET_BLAZE_OFFLOAD_SORT must be radix or bitonic"),
        Err(VarError::NotPresent) => len > 1024,
        Err(error) => panic!("invalid offload sort setting: {error}"),
    };
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

fn sort_radix(keys: &mut [u64], bits: u32) {
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
