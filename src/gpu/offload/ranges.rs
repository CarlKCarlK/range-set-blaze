//! Turn sorted keys into ranges on the GPU, so only the ranges return to the host.
//!
//! Two kernels share one definition of "starts a range": `count_starts` counts the
//! starts in each 4096-key tile; the host scans those counts into tile offsets; then
//! `emit_ranges` writes each range as a `(first, last)` pair at its offset.

#[cfg(any(target_arch = "nvptx64", target_arch = "amdgpu"))]
use super::device;
#[cfg(any(target_arch = "nvptx64", target_arch = "amdgpu"))]
use super::device::{block_idx_x, sync_block, thread_idx_x};
use core::offload::offload_kernel;
#[cfg(any(target_arch = "nvptx64", target_arch = "amdgpu"))]
use core::sync::atomic::{AtomicU64, Ordering};

pub(super) const BLOCK: u32 = 256;
pub(super) const TILE: u32 = 4096;
#[cfg(any(target_arch = "nvptx64", target_arch = "amdgpu"))]
const ITEMS: u32 = TILE / BLOCK;
#[cfg(any(target_arch = "nvptx64", target_arch = "amdgpu"))]
const WARPS: u32 = BLOCK / 32;
/// Shared-memory bytes `emit_ranges` uses: one u32 total per warp.
pub(super) const EMIT_SHARED: usize = 32;

/// Whether sorted keys `left` then `right` belong to the same range. `gap_left` and
/// `gap_right` also count as adjacent when `has_gap` is 1 (for `char`, the surrogates).
#[cfg(any(target_arch = "nvptx64", target_arch = "amdgpu"))]
#[inline(always)]
fn joins(left: u64, right: u64, gap_left: u64, gap_right: u64, has_gap: u32) -> bool {
    left == right
        || (left != u64::MAX && right == left + 1)
        || (has_gap == 1 && left == gap_left && right == gap_right)
}

/// Whether the key at `index` starts a range.
///
/// # Safety
/// `keys` must hold `len` initialized words and `index < len`.
#[cfg(any(target_arch = "nvptx64", target_arch = "amdgpu"))]
#[inline(always)]
unsafe fn is_start(
    keys: *mut u64,
    index: u32,
    gap_left: u64,
    gap_right: u64,
    has_gap: u32,
) -> bool {
    // SAFETY: the caller bounds index; index - 1 is read only when index > 0.
    index == 0
        || unsafe {
            !joins(
                *keys.add(index as usize - 1),
                *keys.add(index as usize),
                gap_left,
                gap_right,
                has_gap,
            )
        }
}

/// Adds the number of range starts in each tile to `counts[tile]`, which must be zero.
#[offload_kernel]
pub(super) fn count_starts(
    keys: *mut u64,
    len: u32,
    gap_left: u64,
    gap_right: u64,
    has_gap: u32,
    counts: *mut u64,
) {
    // SAFETY: these intrinsics only read this thread's launch coordinates.
    let (tid, tile) = unsafe { (thread_idx_x(), block_idx_x()) };
    let mut count = 0_u32;
    let mut round = 0;
    while round < ITEMS {
        // Consecutive threads read consecutive keys.
        let index = tile * TILE + round * BLOCK + tid;
        // SAFETY: index < len bounds the read of the resident sorted keys.
        if index < len && unsafe { is_start(keys, index, gap_left, gap_right, has_gap) } {
            count += 1;
        }
        round += 1;
    }
    // Sum each warp's counts so one lane per warp updates the tile's total.
    let lane = tid % 32;
    let mut inclusive = count;
    let mut delta = 1;
    while delta < 32 {
        // SAFETY: every lane of the warp executes this with the same delta in 1..32.
        let up = unsafe { device::shuffle_up(inclusive, delta) };
        if lane >= delta {
            inclusive += up;
        }
        delta *= 2;
    }
    if lane == 31 && inclusive != 0 {
        // SAFETY: counts holds one zeroed word per tile; atomics combine the warps.
        unsafe {
            AtomicU64::from_ptr(counts.add(tile as usize))
                .fetch_add(u64::from(inclusive), Ordering::Relaxed);
        }
    }
}

/// Writes range `r` as `ranges[2r] = first key`, `ranges[2r + 1] = last key`, given each
/// tile's exclusive start offset from the scanned `count_starts` totals.
#[offload_kernel]
pub(super) fn emit_ranges(
    keys: *mut u64,
    len: u32,
    gap_left: u64,
    gap_right: u64,
    has_gap: u32,
    tile_offsets: *mut u64,
    ranges: *mut u64,
) {
    // SAFETY: the launch provides EMIT_SHARED bytes, aligned, for eight warp totals.
    let warp_totals = unsafe { device::shared::<EMIT_SHARED>().cast::<u32>() };
    // SAFETY: these intrinsics only read this thread's launch coordinates.
    let (tid, tile) = unsafe { (thread_idx_x(), block_idx_x()) };
    // Each thread owns ITEMS consecutive keys, so its ranges are numbered in order.
    let first = tile * TILE + tid * ITEMS;
    let mut count = 0_u32;
    let mut item = 0;
    while item < ITEMS {
        let index = first + item;
        // SAFETY: index < len bounds the read.
        if index < len && unsafe { is_start(keys, index, gap_left, gap_right, has_gap) } {
            count += 1;
        }
        item += 1;
    }

    // Exclusive scan of the thread counts: within each warp, then across warps.
    let lane = tid % 32;
    let warp_index = tid / 32;
    let mut inclusive = count;
    let mut delta = 1;
    while delta < 32 {
        // SAFETY: every lane of the warp executes this with the same delta in 1..32.
        let up = unsafe { device::shuffle_up(inclusive, delta) };
        if lane >= delta {
            inclusive += up;
        }
        delta *= 2;
    }
    if lane == 31 {
        // SAFETY: one lane owns each warp-total slot.
        unsafe { *warp_totals.add(warp_index as usize) = inclusive };
    }
    // SAFETY: all threads reach the barrier before reading the warp totals.
    unsafe { sync_block() };
    let mut before = inclusive - count;
    let mut w = 0;
    while w < warp_index {
        // SAFETY: the barrier published every warp total.
        before += unsafe { *warp_totals.add(w as usize) };
        w += 1;
    }

    // `started` counts the ranges that start at or before the current key, so the
    // current key belongs to range `started - 1`. Key 0 always starts a range.
    // SAFETY: tile_offsets holds one scanned word per tile.
    let mut started = unsafe { *tile_offsets.add(tile as usize) } + u64::from(before);
    item = 0;
    while item < ITEMS {
        let index = first + item;
        if index < len {
            // SAFETY: index < len bounds every read; index + 1 is read only below len.
            unsafe {
                let key = *keys.add(index as usize);
                if is_start(keys, index, gap_left, gap_right, has_gap) {
                    started += 1;
                    *ranges.add(2 * (started - 1) as usize) = key;
                }
                if index + 1 == len
                    || !joins(
                        key,
                        *keys.add(index as usize + 1),
                        gap_left,
                        gap_right,
                        has_gap,
                    )
                {
                    *ranges.add(2 * (started - 1) as usize + 1) = key;
                }
            }
        }
        item += 1;
    }
}
