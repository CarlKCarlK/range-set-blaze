//! CUDA kernels for `Engine::CudaOnly`: a stable LSD radix sort of `u32` keys and the range
//! search that follows it. Every kernel runs blocks of `BLOCK` threads over tiles of `TILE`
//! items, and the host (`src/gpu/cuda.rs`) launches them by name from the generated PTX.

use cuda_device::atomic::{AtomicOrdering, BlockAtomicU32};
use cuda_device::{SharedArray, kernel, launch_bounds, thread, warp};
use cuda_host::cuda_module;

const BLOCK: u32 = 256;
const WARPS: u32 = BLOCK / 32;
const ITEMS: u32 = 16;
const TILE: u32 = BLOCK * ITEMS;
const RADIX: u32 = 256;

#[cuda_module]
mod kernels {
    use super::*;

    /// Counts each tile's 8-bit digits at `shift`. Writes digit-major, `counts[digit * tiles +
    /// tile]`, so one exclusive scan of `counts` gives every tile's start for every digit.
    #[kernel]
    #[launch_bounds(256)]
    pub fn radix_histogram(keys: &[u32], shift: u32, tiles: u32, counts: *mut u32) {
        static mut HISTOGRAM: SharedArray<u32, 256> = SharedArray::UNINIT;
        let tid = thread::threadIdx_x();
        let tile = thread::blockIdx_x();
        let len = keys.len() as u32;
        unsafe {
            HISTOGRAM[tid as usize] = 0;
        }
        thread::sync_threads();
        let base = tile * TILE;
        let mut round = 0;
        while round < ITEMS {
            let index = base + round * BLOCK + tid;
            if index < len {
                let digit = (keys[index as usize] >> shift) & (RADIX - 1);
                unsafe {
                    BlockAtomicU32::from_ptr(
                        SharedArray::as_raw_mut_ptr(&raw mut HISTOGRAM).add(digit as usize),
                    )
                    .fetch_add(1, AtomicOrdering::Relaxed);
                }
            }
            round += 1;
        }
        thread::sync_threads();
        unsafe {
            *counts.add((tid * tiles + tile) as usize) = HISTOGRAM[tid as usize];
        }
    }

    /// Exclusive scan of each `TILE` of `data[..len]`, in place; each tile's total goes to
    /// `tile_sums[tile]`.
    #[kernel]
    #[launch_bounds(256)]
    pub fn scan_tiles(data: *mut u32, len: u32, tile_sums: *mut u32) {
        static mut ITEMS_IN_TILE: SharedArray<u32, 4096> = SharedArray::UNINIT;
        static mut WARP_SUMS: SharedArray<u32, 8> = SharedArray::UNINIT;
        let tid = thread::threadIdx_x();
        let tile = thread::blockIdx_x();
        let base = tile * TILE;
        let mut round = 0;
        while round < ITEMS {
            let local = round * BLOCK + tid;
            let index = base + local;
            let value = if index < len {
                unsafe { *data.add(index as usize) }
            } else {
                0
            };
            unsafe {
                ITEMS_IN_TILE[local as usize] = value;
            }
            round += 1;
        }
        thread::sync_threads();

        // Each thread scans its own ITEMS consecutive items.
        let first = tid * ITEMS;
        let mut thread_sum = 0;
        let mut item = 0;
        while item < ITEMS {
            let value = unsafe { ITEMS_IN_TILE[(first + item) as usize] };
            unsafe {
                ITEMS_IN_TILE[(first + item) as usize] = thread_sum;
            }
            thread_sum += value;
            item += 1;
        }

        // Exclusive scan of the thread sums: within each warp, then across warps.
        let lane = warp::lane_id();
        let warp_index = tid / 32;
        let mut inclusive = thread_sum;
        let mut delta = 1;
        while delta < 32 {
            let up = warp::shuffle_up_sync(0xffff_ffff, inclusive, delta);
            if lane >= delta {
                inclusive += up;
            }
            delta *= 2;
        }
        if lane == 31 {
            unsafe {
                WARP_SUMS[warp_index as usize] = inclusive;
            }
        }
        thread::sync_threads();
        let mut warp_offset = 0;
        let mut total = 0;
        let mut w = 0;
        while w < WARPS {
            let sum = unsafe { WARP_SUMS[w as usize] };
            if w < warp_index {
                warp_offset += sum;
            }
            total += sum;
            w += 1;
        }
        let thread_offset = warp_offset + inclusive - thread_sum;
        item = 0;
        while item < ITEMS {
            unsafe {
                ITEMS_IN_TILE[(first + item) as usize] += thread_offset;
            }
            item += 1;
        }
        thread::sync_threads();

        round = 0;
        while round < ITEMS {
            let local = round * BLOCK + tid;
            let index = base + local;
            if index < len {
                unsafe {
                    *data.add(index as usize) = ITEMS_IN_TILE[local as usize];
                }
            }
            round += 1;
        }
        if tid == 0 {
            unsafe {
                *tile_sums.add(tile as usize) = total;
            }
        }
    }

    /// Adds each tile's scanned offset to its items: the second half of a multi-tile scan.
    #[kernel]
    #[launch_bounds(256)]
    pub fn add_tile_offsets(data: *mut u32, len: u32, tile_offsets: &[u32]) {
        let tile = thread::blockIdx_x();
        let offset = tile_offsets[tile as usize];
        let base = tile * TILE;
        let mut round = 0;
        while round < ITEMS {
            let index = base + round * BLOCK + thread::threadIdx_x();
            if index < len {
                unsafe {
                    *data.add(index as usize) += offset;
                }
            }
            round += 1;
        }
    }

    /// Stable scatter of each tile's keys (and, if `values` is non-empty, their values) to the
    /// positions given by the scanned histogram. Within a tile, items go in order: by round,
    /// then warp, then lane, with ranks among equal digits from `match_any_sync`.
    #[kernel]
    #[launch_bounds(256)]
    pub fn radix_scatter(
        keys: &[u32],
        values: &[u32],
        shift: u32,
        tiles: u32,
        offsets: &[u32],
        keys_out: *mut u32,
        values_out: *mut u32,
    ) {
        static mut DIGIT_OFFSETS: SharedArray<u32, 256> = SharedArray::UNINIT;
        static mut WARP_COUNTS: SharedArray<u32, 2048> = SharedArray::UNINIT;
        let tid = thread::threadIdx_x();
        let tile = thread::blockIdx_x();
        let len = keys.len() as u32;
        let with_values = !values.is_empty();
        let lane_below = warp::lanemask_lt();
        let warp_index = tid / 32;
        unsafe {
            DIGIT_OFFSETS[tid as usize] = offsets[(tid * tiles + tile) as usize];
        }
        let mut w = 0;
        while w < WARPS {
            unsafe {
                WARP_COUNTS[(w * RADIX + tid) as usize] = 0;
            }
            w += 1;
        }
        thread::sync_threads();

        let base = tile * TILE;
        let mut round = 0;
        while round < ITEMS {
            let index = base + round * BLOCK + tid;
            let valid = index < len;
            let key = if valid { keys[index as usize] } else { 0 };
            // Past the end, a digit no real key has, so those lanes match only each other.
            let digit = if valid {
                (key >> shift) & (RADIX - 1)
            } else {
                RADIX
            };
            let peers = warp::match_any_sync(0xffff_ffff, digit);
            let rank = (peers & lane_below).count_ones();
            if valid && rank == 0 {
                unsafe {
                    WARP_COUNTS[(warp_index * RADIX + digit) as usize] = peers.count_ones();
                }
            }
            thread::sync_threads();

            // Thread `tid` owns digit `tid`: an exclusive scan of its count across warps.
            let mut running = 0;
            w = 0;
            while w < WARPS {
                let slot = (w * RADIX + tid) as usize;
                let count = unsafe { WARP_COUNTS[slot] };
                unsafe {
                    WARP_COUNTS[slot] = running;
                }
                running += count;
                w += 1;
            }
            thread::sync_threads();

            if valid {
                let destination = unsafe {
                    DIGIT_OFFSETS[digit as usize]
                        + WARP_COUNTS[(warp_index * RADIX + digit) as usize]
                        + rank
                } as usize;
                unsafe {
                    *keys_out.add(destination) = key;
                    if with_values {
                        *values_out.add(destination) = values[index as usize];
                    }
                }
            }
            thread::sync_threads();

            unsafe {
                DIGIT_OFFSETS[tid as usize] += running;
            }
            w = 0;
            while w < WARPS {
                unsafe {
                    WARP_COUNTS[(w * RADIX + tid) as usize] = 0;
                }
                w += 1;
            }
            thread::sync_threads();
            round += 1;
        }
    }

    /// Writes 1 where a sorted key starts a range, else 0. The keys `gap_left` and `gap_right`
    /// count as adjacent when `has_gap` is 1 (for `char`, across the surrogates).
    #[kernel]
    #[launch_bounds(256)]
    pub fn mark_starts(
        sorted: &[u32],
        gap_left: u32,
        gap_right: u32,
        has_gap: u32,
        starts: *mut u32,
    ) {
        let index = thread::blockIdx_x() * BLOCK + thread::threadIdx_x();
        if index < sorted.len() as u32 {
            let is_start = index == 0
                || !joins(
                    sorted[index as usize - 1],
                    sorted[index as usize],
                    gap_left,
                    gap_right,
                    has_gap,
                );
            unsafe {
                *starts.add(index as usize) = is_start as u32;
            }
        }
    }

    /// From the exclusive scan of `mark_starts`, writes each range's first key to `starts` and
    /// its last key to `ends`.
    #[kernel]
    #[launch_bounds(256)]
    pub fn emit_ranges(
        sorted: &[u32],
        scanned: &[u32],
        gap_left: u32,
        gap_right: u32,
        has_gap: u32,
        starts: *mut u32,
        ends: *mut u32,
    ) {
        let index = thread::blockIdx_x() * BLOCK + thread::threadIdx_x();
        let len = sorted.len() as u32;
        if index < len {
            let i = index as usize;
            let key = sorted[i];
            let is_start = index == 0 || !joins(sorted[i - 1], key, gap_left, gap_right, has_gap);
            let is_end =
                index + 1 == len || !joins(key, sorted[i + 1], gap_left, gap_right, has_gap);
            let range = scanned[i] + is_start as u32;
            unsafe {
                if is_start {
                    *starts.add(range as usize - 1) = key;
                }
                if is_end {
                    *ends.add(range as usize - 1) = key;
                }
            }
        }
    }

    /// As [`mark_starts`], for 64-bit keys sorted as `(hi, lo)` word pairs. The gap is
    /// `(gap_left_hi, gap_left_lo)` to `(gap_right_hi, gap_right_lo)`.
    #[kernel]
    #[launch_bounds(256)]
    pub fn mark_starts_64(
        hi: &[u32],
        lo: &[u32],
        gap_left_hi: u32,
        gap_left_lo: u32,
        gap_right_hi: u32,
        gap_right_lo: u32,
        has_gap: u32,
        starts: *mut u32,
    ) {
        let index = thread::blockIdx_x() * BLOCK + thread::threadIdx_x();
        if index < hi.len() as u32 {
            let i = index as usize;
            let gap = Gap64 {
                left_hi: gap_left_hi,
                left_lo: gap_left_lo,
                right_hi: gap_right_hi,
                right_lo: gap_right_lo,
                enabled: has_gap == 1,
            };
            let is_start = index == 0 || !joins_64(hi[i - 1], lo[i - 1], hi[i], lo[i], &gap);
            unsafe {
                *starts.add(i) = is_start as u32;
            }
        }
    }

    /// As [`emit_ranges`], for 64-bit keys: each range's ends go out as `(hi, lo)` words.
    #[kernel]
    #[launch_bounds(256)]
    pub fn emit_ranges_64(
        hi: &[u32],
        lo: &[u32],
        scanned: &[u32],
        gap_left_hi: u32,
        gap_left_lo: u32,
        gap_right_hi: u32,
        gap_right_lo: u32,
        has_gap: u32,
        starts_hi: *mut u32,
        starts_lo: *mut u32,
        ends_hi: *mut u32,
        ends_lo: *mut u32,
    ) {
        let index = thread::blockIdx_x() * BLOCK + thread::threadIdx_x();
        let len = hi.len() as u32;
        if index < len {
            let i = index as usize;
            let gap = Gap64 {
                left_hi: gap_left_hi,
                left_lo: gap_left_lo,
                right_hi: gap_right_hi,
                right_lo: gap_right_lo,
                enabled: has_gap == 1,
            };
            let is_start = index == 0 || !joins_64(hi[i - 1], lo[i - 1], hi[i], lo[i], &gap);
            let is_end = index + 1 == len || !joins_64(hi[i], lo[i], hi[i + 1], lo[i + 1], &gap);
            let range = (scanned[i] + is_start as u32) as usize - 1;
            unsafe {
                if is_start {
                    *starts_hi.add(range) = hi[i];
                    *starts_lo.add(range) = lo[i];
                }
                if is_end {
                    *ends_hi.add(range) = hi[i];
                    *ends_lo.add(range) = lo[i];
                }
            }
        }
    }

    /// Writes `0..len` to `positions`: each map entry's input position, for the sort to carry.
    #[kernel]
    #[launch_bounds(256)]
    pub fn fill_positions(positions: *mut u32, len: u32) {
        let index = thread::blockIdx_x() * BLOCK + thread::threadIdx_x();
        if index < len {
            unsafe {
                *positions.add(index as usize) = index;
            }
        }
    }

    /// Writes 1 where a sorted map entry is the last of its key (so its value wins), else 0.
    #[kernel]
    #[launch_bounds(256)]
    pub fn mark_survivors(keys: &[u32], keep: *mut u32) {
        let index = thread::blockIdx_x() * BLOCK + thread::threadIdx_x();
        let len = keys.len() as u32;
        if index < len {
            let i = index as usize;
            let survives = index + 1 == len || keys[i + 1] != keys[i];
            unsafe {
                *keep.add(i) = survives as u32;
            }
        }
    }

    /// From the exclusive scan of `mark_survivors`, packs the surviving keys and positions.
    #[kernel]
    #[launch_bounds(256)]
    pub fn compact_survivors(
        keys: &[u32],
        positions: &[u32],
        scanned: &[u32],
        keys_out: *mut u32,
        positions_out: *mut u32,
    ) {
        let index = thread::blockIdx_x() * BLOCK + thread::threadIdx_x();
        let len = keys.len() as u32;
        if index < len {
            let i = index as usize;
            if index + 1 == len || keys[i + 1] != keys[i] {
                let slot = scanned[i] as usize;
                unsafe {
                    *keys_out.add(slot) = keys[i];
                    *positions_out.add(slot) = positions[i];
                }
            }
        }
    }

    /// Writes 1 where a surviving map entry starts a range: its key doesn't follow the previous
    /// survivor's, or its value differs. Values are `value_lo` words, plus `value_hi` words if
    /// that slice isn't empty.
    #[kernel]
    #[launch_bounds(256)]
    pub fn mark_map_starts(
        keys: &[u32],
        positions: &[u32],
        value_lo: &[u32],
        value_hi: &[u32],
        gap_left: u32,
        gap_right: u32,
        has_gap: u32,
        starts: *mut u32,
    ) {
        let index = thread::blockIdx_x() * BLOCK + thread::threadIdx_x();
        if index < keys.len() as u32 {
            let i = index as usize;
            let values = Values {
                lo: value_lo,
                hi: value_hi,
            };
            let is_start = index == 0
                || !map_joins(
                    keys,
                    positions,
                    &values,
                    i - 1,
                    gap_left,
                    gap_right,
                    has_gap,
                );
            unsafe {
                *starts.add(i) = is_start as u32;
            }
        }
    }

    /// From the exclusive scan of `mark_map_starts`, writes each range's first and last keys and
    /// the input position of its first entry (whose value is the range's).
    #[kernel]
    #[launch_bounds(256)]
    pub fn emit_map_ranges(
        keys: &[u32],
        positions: &[u32],
        value_lo: &[u32],
        value_hi: &[u32],
        scanned: &[u32],
        gap_left: u32,
        gap_right: u32,
        has_gap: u32,
        starts: *mut u32,
        ends: *mut u32,
        firsts: *mut u32,
    ) {
        let index = thread::blockIdx_x() * BLOCK + thread::threadIdx_x();
        let len = keys.len() as u32;
        if index < len {
            let i = index as usize;
            let values = Values {
                lo: value_lo,
                hi: value_hi,
            };
            let is_start = index == 0
                || !map_joins(
                    keys,
                    positions,
                    &values,
                    i - 1,
                    gap_left,
                    gap_right,
                    has_gap,
                );
            let is_end = index + 1 == len
                || !map_joins(keys, positions, &values, i, gap_left, gap_right, has_gap);
            let range = (scanned[i] + is_start as u32) as usize - 1;
            unsafe {
                if is_start {
                    *starts.add(range) = keys[i];
                    *firsts.add(range) = positions[i];
                }
                if is_end {
                    *ends.add(range) = keys[i];
                }
            }
        }
    }
}

struct Gap64 {
    left_hi: u32,
    left_lo: u32,
    right_hi: u32,
    right_lo: u32,
    enabled: bool,
}

/// Whether sorted 64-bit keys `(left_hi, left_lo)` then `(right_hi, right_lo)` belong to the
/// same range.
fn joins_64(left_hi: u32, left_lo: u32, right_hi: u32, right_lo: u32, gap: &Gap64) -> bool {
    let same = left_hi == right_hi && left_lo == right_lo;
    let next_in_word = left_hi == right_hi && left_lo != u32::MAX && right_lo == left_lo + 1;
    let carry =
        left_lo == u32::MAX && right_lo == 0 && left_hi != u32::MAX && right_hi == left_hi + 1;
    let exceptional = gap.enabled
        && left_hi == gap.left_hi
        && left_lo == gap.left_lo
        && right_hi == gap.right_hi
        && right_lo == gap.right_lo;
    same || next_in_word || carry || exceptional
}

/// A map's values as one or two words each, indexed by input position.
struct Values<'a> {
    lo: &'a [u32],
    hi: &'a [u32],
}

/// Whether survivors `left` and `left + 1` share a range: adjacent keys and equal values.
fn map_joins(
    keys: &[u32],
    positions: &[u32],
    values: &Values<'_>,
    left: usize,
    gap_left: u32,
    gap_right: u32,
    has_gap: u32,
) -> bool {
    let key = keys[left];
    let next = keys[left + 1];
    let adjacent = (key != u32::MAX && next == key + 1)
        || (has_gap == 1 && key == gap_left && next == gap_right);
    if !adjacent {
        return false;
    }
    let a = positions[left] as usize;
    let b = positions[left + 1] as usize;
    values.lo[a] == values.lo[b] && (values.hi.is_empty() || values.hi[a] == values.hi[b])
}

/// Whether sorted keys `left` then `right` belong to the same range.
fn joins(left: u32, right: u32, gap_left: u32, gap_right: u32, has_gap: u32) -> bool {
    left == right
        || (left != u32::MAX && right == left + 1)
        || (has_gap == 1 && left == gap_left && right == gap_right)
}

fn main() {}
