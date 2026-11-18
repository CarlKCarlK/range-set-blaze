//! [`Engine::Cuda`]: the GPU pipeline as CUDA kernels written in Rust with cuda-oxide.
//!
//! Kernels and host code share this file: `#[cuda_module]` compiles the kernels to device code
//! and generates a typed launcher for each, so building the `cuda` feature needs `cargo oxide`
//! (`cargo oxide build -- --features cuda`), which supplies cuda-oxide's codegen backend.
//!
//! Each kernel declares a launch contract (1-D, `BLOCK` threads per block), so every launch is
//! checked against it. Kernels that write only their own thread's element take a
//! `DisjointSlice`, and launching them is safe. Kernels that write wherever the data says (the
//! sort's scatter, scans, and range emission) take raw pointers and are `unsafe fn`; each launch
//! says why its pointers are valid.
//!
//! Each call is one-shot: only the CUDA context is kept between calls. The module and every
//! buffer are created per call, as the wgpu engine creates its pipelines and buffers per call.

#![allow(
    clippy::cast_possible_truncation,
    clippy::too_many_arguments,
    reason = "kernels index with u32 and take many arguments, as CUDA kernels do"
)]

use std::string::String;
use std::sync::{Arc, OnceLock};
use std::vec::Vec;

use cuda_core::simt::memory::memcpy_htod_async;
use cuda_core::sys::cudaError_enum_CUDA_ERROR_OUT_OF_MEMORY;
use cuda_core::{
    CudaContext, CudaStream, DeviceBuffer, DeviceCopy, DriverError, LaunchConfig1D,
    LaunchContractError, PinnedHostBuffer,
};
use cuda_device::atomic::{AtomicOrdering, BlockAtomicU32, DeviceAtomicU32, DeviceAtomicU64};
use cuda_device::{
    DisjointSlice, SharedArray, kernel, launch_bounds, launch_contract, thread, warp,
};
use cuda_host::cuda_module;

use super::{high_word, join_words, low_word};

#[cfg(doc)]
use super::Engine;

/// Threads per block.
const BLOCK: u32 = 256;
const WARPS: u32 = BLOCK / 32;
/// Items per thread, and per block (a tile), in the tiled kernels.
const ITEMS: u32 = 16;
const TILE: u32 = BLOCK * ITEMS;
/// Values of an 8-bit radix-sort digit.
const RADIX: u32 = 256;
/// Items per warp in `radix_scatter`, and the rounds of 32 they take.
const WARP_ITEMS: u32 = TILE / WARPS;
const WARP_ROUNDS: u32 = WARP_ITEMS / 32;
/// Flags on a look-back `status` word, above its 32-bit count: published, and published as the
/// count of every tile so far (rather than of this tile alone).
const AGGREGATE: u64 = 1 << 32;
const INCLUSIVE: u64 = 2 << 32;
const READY: u64 = AGGREGATE | INCLUSIVE;
/// Keys per upload chunk: each is encoded into pinned memory while the one before it copies.
const CHUNK: usize = 1 << 18;
/// Inputs up to this many keys are encoded into ordinary memory and copied at once: allocating
/// pinned memory costs more than it saves (about 0.6 ms per MB, plus freeing it), while beyond
/// about 4 MB a fresh ordinary buffer pays more in page faults.
const PAGEABLE_UPLOAD: usize = 1 << 20;

#[cuda_module]
mod kernels {
    use super::*;

    /// Writes `0..len` to `positions`: each map entry's input position, for the sort to carry.
    #[kernel]
    #[launch_bounds(256)]
    #[launch_contract(domain = 1, block = (256, 1, 1))]
    pub(super) fn fill_positions(mut positions: DisjointSlice<'_, u32>) {
        if let Some((position, index)) = positions.get_mut_indexed() {
            *position = index.get() as u32;
        }
    }

    /// Adds every key's four 8-bit digit counts to `counts[pass * 256 + digit]`. The host reads
    /// them to find where each digit starts, and to skip passes where every key has the same
    /// digit.
    ///
    /// # Safety
    ///
    /// `counts` must point to 1,024 zeroed words. Launch one block per `TILE` keys.
    #[kernel]
    #[launch_bounds(256)]
    #[launch_contract(domain = 1, block = (256, 1, 1))]
    pub(super) unsafe fn digit_counts(keys: &[u32], counts: *mut u32) {
        static mut HISTOGRAMS: SharedArray<u32, 1024> = SharedArray::UNINIT;
        let tid = thread::threadIdx_x();
        let len = keys.len() as u32;
        let mut pass = 0;
        while pass < 4 {
            unsafe {
                HISTOGRAMS[(pass * RADIX + tid) as usize] = 0;
            }
            pass += 1;
        }
        thread::sync_threads();
        let base = thread::blockIdx_x() * TILE;
        let mut round = 0;
        while round < ITEMS {
            let index = base + round * BLOCK + tid;
            if index < len {
                let key = keys[index as usize];
                pass = 0;
                while pass < 4 {
                    let digit = (key >> (pass * 8)) & (RADIX - 1);
                    unsafe {
                        BlockAtomicU32::from_ptr(
                            SharedArray::as_raw_mut_ptr(&raw mut HISTOGRAMS)
                                .add((pass * RADIX + digit) as usize),
                        )
                        .fetch_add(1, AtomicOrdering::Relaxed);
                    }
                    pass += 1;
                }
            }
            round += 1;
        }
        thread::sync_threads();
        pass = 0;
        while pass < 4 {
            let slot = (pass * RADIX + tid) as usize;
            let count = unsafe { HISTOGRAMS[slot] };
            if count != 0 {
                unsafe {
                    DeviceAtomicU32::from_ptr(counts.add(slot))
                        .fetch_add(count, AtomicOrdering::Relaxed);
                }
            }
            pass += 1;
        }
    }

    /// One pass of a "Onesweep" LSD radix sort: a stable scatter of each tile's keys (and, if
    /// `values` is non-empty, their values) by their 8-bit digit at `shift`.
    /// `digit_bases[bases_offset + d]` is where digit `d` starts in the output (from
    /// `digit_counts`). Each tile learns where its items of digit `d` go by decoupled
    /// look-back: it publishes its own count in `status`, then adds up its predecessors'
    /// published counts until it reaches one that already includes everything before it, and
    /// publishes that inclusive total for its successors. Tiles take their indices from
    /// `ticket` in launch order, so a tile only ever waits on tiles that started before it.
    ///
    /// Each warp owns `WARP_ITEMS` consecutive items of the tile. The warps count their digits
    /// (phase A), which gives each warp its start for each digit within the tile (B); they then
    /// rank their items again and place them in shared memory in digit order (C); and the block
    /// copies them out in that order, so neighboring threads mostly write neighboring addresses
    /// (D). Ranks come from `match_any_sync`, in item order, so the sort is stable.
    ///
    /// # Safety
    ///
    /// `keys_out` (and `values_out`, if there are values) must point to `keys.len()` words;
    /// `status` to one zeroed word per tile and digit; and `ticket` to one zeroed word. Launch
    /// one block per `TILE` keys.
    #[kernel]
    #[launch_bounds(256)]
    #[launch_contract(domain = 1, block = (256, 1, 1))]
    pub(super) unsafe fn radix_scatter(
        keys: &[u32],
        values: &[u32],
        shift: u32,
        digit_bases: &[u32],
        bases_offset: u32,
        status: *mut u64,
        ticket: *mut u32,
        keys_out: *mut u32,
        values_out: *mut u32,
    ) {
        static mut SORTED_KEYS: SharedArray<u32, 4096> = SharedArray::UNINIT;
        static mut SORTED_VALUES: SharedArray<u32, 4096> = SharedArray::UNINIT;
        // Per warp and digit: first a count, then that warp's next slot in the tile.
        static mut WARP_SLOTS: SharedArray<u32, 2048> = SharedArray::UNINIT;
        static mut TILE_STARTS: SharedArray<u32, 256> = SharedArray::UNINIT;
        static mut GLOBAL_STARTS: SharedArray<u32, 256> = SharedArray::UNINIT;
        static mut WARP_TOTALS: SharedArray<u32, 8> = SharedArray::UNINIT;
        static mut TILE_INDEX: SharedArray<u32, 1> = SharedArray::UNINIT;
        let tid = thread::threadIdx_x();
        if tid == 0 {
            unsafe {
                TILE_INDEX[0] =
                    DeviceAtomicU32::from_ptr(ticket).fetch_add(1, AtomicOrdering::Relaxed);
            }
        }
        thread::sync_threads();
        let tile = unsafe { TILE_INDEX[0] };
        let len = keys.len() as u32;
        let with_values = !values.is_empty();
        let lane = warp::lane_id();
        let lane_below = warp::lanemask_lt();
        let warp_index = tid / 32;
        let base = tile * TILE;
        let segment = base + warp_index * WARP_ITEMS;
        let mut w = 0;
        while w < WARPS {
            unsafe {
                WARP_SLOTS[(w * RADIX + tid) as usize] = 0;
            }
            w += 1;
        }
        thread::sync_threads();

        // A: count each warp's digits.
        let mut round = 0;
        while round < WARP_ROUNDS {
            let index = segment + round * 32 + lane;
            let valid = index < len;
            // Past the end, a digit no real key has, so those lanes match only each other.
            let digit = if valid {
                (keys[index as usize] >> shift) & (RADIX - 1)
            } else {
                RADIX
            };
            let peers = warp::match_any_sync(0xffff_ffff, digit);
            if valid && (peers & lane_below) == 0 {
                unsafe {
                    WARP_SLOTS[(warp_index * RADIX + digit) as usize] += peers.count_ones();
                }
            }
            warp::sync_mask(0xffff_ffff);
            round += 1;
        }
        thread::sync_threads();

        // B: thread `tid` owns digit `tid`. Prefix its counts across warps, then scan the tile's
        // digit totals across the block.
        let mut total = 0;
        w = 0;
        while w < WARPS {
            let slot = (w * RADIX + tid) as usize;
            let count = unsafe { WARP_SLOTS[slot] };
            unsafe {
                WARP_SLOTS[slot] = total;
            }
            total += count;
            w += 1;
        }
        let mut inclusive = total;
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
                WARP_TOTALS[warp_index as usize] = inclusive;
            }
        }
        // Look back: the count of digit `tid` in all tiles before this one.
        let slot = |tile: u32| unsafe {
            DeviceAtomicU64::from_ptr(status.add((tile * RADIX + tid) as usize))
        };
        let prefix = if tile == 0 {
            0
        } else {
            slot(tile).store(AGGREGATE | u64::from(total), AtomicOrdering::Release);
            let mut prefix = 0;
            let mut previous = tile;
            loop {
                previous -= 1;
                let mut word = slot(previous).load(AtomicOrdering::Acquire);
                while word & READY == 0 {
                    word = slot(previous).load(AtomicOrdering::Acquire);
                }
                prefix += word as u32;
                if word & INCLUSIVE != 0 {
                    break;
                }
            }
            prefix
        };
        slot(tile).store(
            INCLUSIVE | u64::from(prefix + total),
            AtomicOrdering::Release,
        );
        unsafe {
            GLOBAL_STARTS[tid as usize] = digit_bases[(bases_offset + tid) as usize] + prefix;
        }
        thread::sync_threads();
        let mut tile_start = inclusive - total;
        w = 0;
        while w < warp_index {
            tile_start += unsafe { WARP_TOTALS[w as usize] };
            w += 1;
        }
        unsafe {
            TILE_STARTS[tid as usize] = tile_start;
        }
        w = 0;
        while w < WARPS {
            unsafe {
                WARP_SLOTS[(w * RADIX + tid) as usize] += tile_start;
            }
            w += 1;
        }
        thread::sync_threads();

        // C: rank again and place each item in shared memory, in digit order.
        round = 0;
        while round < WARP_ROUNDS {
            let index = segment + round * 32 + lane;
            let valid = index < len;
            let key = if valid { keys[index as usize] } else { 0 };
            let digit = if valid {
                (key >> shift) & (RADIX - 1)
            } else {
                RADIX
            };
            let peers = warp::match_any_sync(0xffff_ffff, digit);
            let rank = (peers & lane_below).count_ones();
            let slot = (warp_index * RADIX + digit) as usize;
            let first = if valid {
                unsafe { WARP_SLOTS[slot] }
            } else {
                0
            };
            warp::sync_mask(0xffff_ffff);
            if valid {
                let place = (first + rank) as usize;
                unsafe {
                    SORTED_KEYS[place] = key;
                    if with_values {
                        SORTED_VALUES[place] = values[index as usize];
                    }
                    if rank == 0 {
                        WARP_SLOTS[slot] = first + peers.count_ones();
                    }
                }
            }
            warp::sync_mask(0xffff_ffff);
            round += 1;
        }
        thread::sync_threads();

        // D: copy out in digit order. Item `place` of digit `d` goes `place - TILE_STARTS[d]`
        // past this tile's global start for `d`.
        let in_tile = if len - base < TILE { len - base } else { TILE };
        let mut place = tid;
        while place < in_tile {
            let key = unsafe { SORTED_KEYS[place as usize] };
            let digit = ((key >> shift) & (RADIX - 1)) as usize;
            let destination = unsafe { GLOBAL_STARTS[digit] + place - TILE_STARTS[digit] } as usize;
            unsafe {
                *keys_out.add(destination) = key;
                if with_values {
                    *values_out.add(destination) = SORTED_VALUES[place as usize];
                }
            }
            place += BLOCK;
        }
    }

    /// Exclusive scan of each `TILE` of `data[..len]`, in place; each tile's total goes to
    /// `tile_sums[tile]`.
    ///
    /// # Safety
    ///
    /// `data` must point to `len` words, and `tile_sums` to one word per tile. Launch one block
    /// per `TILE` words.
    #[kernel]
    #[launch_bounds(256)]
    #[launch_contract(domain = 1, block = (256, 1, 1))]
    pub(super) unsafe fn scan_tiles(data: *mut u32, len: u32, tile_sums: *mut u32) {
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
    ///
    /// # Safety
    ///
    /// `data` must point to `len` words. Launch one block per `TILE` words.
    #[kernel]
    #[launch_bounds(256)]
    #[launch_contract(domain = 1, block = (256, 1, 1))]
    pub(super) unsafe fn add_tile_offsets(data: *mut u32, len: u32, tile_offsets: &[u32]) {
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

    /// Writes 1 where a sorted key starts a range, else 0. The keys `gap_left` and `gap_right`
    /// count as adjacent when `has_gap` is 1 (for `char`, across the surrogates).
    #[kernel]
    #[launch_bounds(256)]
    #[launch_contract(domain = 1, block = (256, 1, 1))]
    pub(super) fn mark_starts(
        sorted: &[u32],
        gap_left: u32,
        gap_right: u32,
        has_gap: u32,
        mut starts: DisjointSlice<'_, u32>,
    ) {
        if let Some((start, index)) = starts.get_mut_indexed() {
            let i = index.get();
            *start =
                u32::from(i == 0 || !joins(sorted[i - 1], sorted[i], gap_left, gap_right, has_gap));
        }
    }

    /// From the exclusive scan of `mark_starts`, writes each range's first key to `starts` and
    /// its last key to `ends`.
    ///
    /// # Safety
    ///
    /// `starts` and `ends` must point to one word per range. Launch one thread per key.
    #[kernel]
    #[launch_bounds(256)]
    #[launch_contract(domain = 1, block = (256, 1, 1))]
    pub(super) unsafe fn emit_ranges(
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
            let range = (scanned[i] + u32::from(is_start)) as usize - 1;
            unsafe {
                if is_start {
                    *starts.add(range) = key;
                }
                if is_end {
                    *ends.add(range) = key;
                }
            }
        }
    }

    /// As [`mark_starts`], for 64-bit keys sorted as `(hi, lo)` word pairs. The gap is
    /// `(gap_left_hi, gap_left_lo)` to `(gap_right_hi, gap_right_lo)`.
    #[kernel]
    #[launch_bounds(256)]
    #[launch_contract(domain = 1, block = (256, 1, 1))]
    pub(super) fn mark_starts_64(
        hi: &[u32],
        lo: &[u32],
        gap_left_hi: u32,
        gap_left_lo: u32,
        gap_right_hi: u32,
        gap_right_lo: u32,
        has_gap: u32,
        mut starts: DisjointSlice<'_, u32>,
    ) {
        if let Some((start, index)) = starts.get_mut_indexed() {
            let i = index.get();
            let gap = Gap64 {
                left_hi: gap_left_hi,
                left_lo: gap_left_lo,
                right_hi: gap_right_hi,
                right_lo: gap_right_lo,
                enabled: has_gap == 1,
            };
            *start = u32::from(i == 0 || !joins_64(hi[i - 1], lo[i - 1], hi[i], lo[i], &gap));
        }
    }

    /// As [`emit_ranges`], for 64-bit keys: each range's ends go out as `(hi, lo)` words.
    ///
    /// # Safety
    ///
    /// The four outputs must each point to one word per range. Launch one thread per key.
    #[kernel]
    #[launch_bounds(256)]
    #[launch_contract(domain = 1, block = (256, 1, 1))]
    pub(super) unsafe fn emit_ranges_64(
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
            let range = (scanned[i] + u32::from(is_start)) as usize - 1;
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

    /// Writes 1 where a sorted map entry is the last of its key (so its value wins), else 0.
    #[kernel]
    #[launch_bounds(256)]
    #[launch_contract(domain = 1, block = (256, 1, 1))]
    pub(super) fn mark_survivors(keys: &[u32], mut keep: DisjointSlice<'_, u32>) {
        if let Some((survives, index)) = keep.get_mut_indexed() {
            let i = index.get();
            *survives = u32::from(i + 1 == keys.len() || keys[i + 1] != keys[i]);
        }
    }

    /// From the exclusive scan of `mark_survivors`, packs the surviving keys and positions.
    ///
    /// # Safety
    ///
    /// `keys_out` and `positions_out` must point to one word per survivor. Launch one thread
    /// per entry.
    #[kernel]
    #[launch_bounds(256)]
    #[launch_contract(domain = 1, block = (256, 1, 1))]
    pub(super) unsafe fn compact_survivors(
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
    #[launch_contract(domain = 1, block = (256, 1, 1))]
    pub(super) fn mark_map_starts(
        keys: &[u32],
        positions: &[u32],
        value_lo: &[u32],
        value_hi: &[u32],
        gap_left: u32,
        gap_right: u32,
        has_gap: u32,
        mut starts: DisjointSlice<'_, u32>,
    ) {
        if let Some((start, index)) = starts.get_mut_indexed() {
            let i = index.get();
            let values = Values {
                lo: value_lo,
                hi: value_hi,
            };
            *start = u32::from(
                i == 0
                    || !map_joins(
                        keys,
                        positions,
                        &values,
                        i - 1,
                        gap_left,
                        gap_right,
                        has_gap,
                    ),
            );
        }
    }

    /// From the exclusive scan of `mark_map_starts`, writes each range's first and last keys and
    /// the input position of its first entry (whose value is the range's).
    ///
    /// # Safety
    ///
    /// `starts`, `ends`, and `firsts` must point to one word per range. Launch one thread per
    /// survivor.
    #[kernel]
    #[launch_bounds(256)]
    #[launch_contract(domain = 1, block = (256, 1, 1))]
    pub(super) unsafe fn emit_map_ranges(
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
            let range = (scanned[i] + u32::from(is_start)) as usize - 1;
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

/// Whether sorted keys `left` then `right` belong to the same range.
fn joins(left: u32, right: u32, gap_left: u32, gap_right: u32, has_gap: u32) -> bool {
    left == right
        || (left != u32::MAX && right == left + 1)
        || (has_gap == 1 && left == gap_left && right == gap_right)
}

/// The exceptional adjacent pair of 64-bit keys, if `enabled`, as words.
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

/// The process-wide CUDA handle: the context and the loaded kernels. Like the wgpu engine's
/// context and pipelines, these are code and connections, not data, so they are created once and
/// kept; every buffer still belongs to a single call.
pub(super) struct Cuda {
    context: Arc<CudaContext>,
    module: kernels::LoadedModule,
}

/// The [`Cuda`] handle, or `None` if there is no CUDA GPU (or its kernels won't load).
pub(super) fn cuda_context() -> Option<&'static Cuda> {
    static CUDA: OnceLock<Option<Cuda>> = OnceLock::new();
    CUDA.get_or_init(|| {
        let context = CudaContext::new(0).ok()?;
        // SAFETY: this crate owns the device code embedded for `kernels`.
        let module = unsafe { kernels::load(&context) }.ok()?;
        Some(Cuda { context, module })
    })
    .as_ref()
}

/// The ranges of `values` encoded as `u32` keys, as `(start, end)` key pairs, or `None` if the GPU
/// ran out of memory. `gap` names two keys that count as adjacent (as for `char`, across the
/// surrogates).
///
/// # Panics
///
/// Panics on any other CUDA error.
pub(super) fn ranges_32<T: Copy>(
    cuda: &Cuda,
    values: &[T],
    encode: impl Fn(T) -> u32,
    gap: Option<(u32, u32)>,
) -> Option<Vec<(u32, u32)>> {
    finish(try_ranges_32(cuda, values, encode, gap))
}

/// As [`ranges_32`], for values encoded as `u64` keys.
pub(super) fn ranges_64<T: Copy>(
    cuda: &Cuda,
    values: &[T],
    encode: impl Fn(T) -> u64,
    gap: Option<(u64, u64)>,
) -> Option<Vec<(u64, u64)>> {
    finish(try_ranges_64(cuda, values, encode, gap))
}

/// A map's ranges from its `u32` key words and its values' one or two `u32` words: each range's
/// `(start, end)` key words, and the input position of its first entry (whose value is the
/// range's). Where a key repeats, its last entry wins. `None` if the GPU ran out of memory.
///
/// # Panics
///
/// Panics on any other CUDA error.
pub(super) fn map_ranges(
    cuda: &Cuda,
    keys: &[u32],
    gap: Option<(u32, u32)>,
    value_lo: &[u32],
    value_hi: Option<&[u32]>,
) -> Option<MapRanges> {
    finish(try_map_ranges(cuda, keys, gap, value_lo, value_hi))
}

/// A map's ranges as `(start, end)` key words, and each range's first input position.
type MapRanges = (Vec<(u32, u32)>, Vec<u32>);

/// Why a CUDA call failed: the GPU ran out of memory (the caller may fall back to the CPU), or
/// anything else (a bug or a broken driver).
#[derive(Debug)]
enum Failure {
    OutOfMemory,
    Other(String),
}

impl From<DriverError> for Failure {
    fn from(error: DriverError) -> Self {
        if error.0 == cudaError_enum_CUDA_ERROR_OUT_OF_MEMORY {
            Self::OutOfMemory
        } else {
            Self::Other(std::format!("{error:?}"))
        }
    }
}

impl From<LaunchContractError> for Failure {
    fn from(error: LaunchContractError) -> Self {
        match error {
            LaunchContractError::Driver(error) => error.into(),
            error => Self::Other(std::format!("{error}")),
        }
    }
}

fn finish<R>(result: Result<R, Failure>) -> Option<R> {
    match result {
        Ok(ranges) => Some(ranges),
        Err(Failure::OutOfMemory) => None,
        Err(Failure::Other(error)) => panic!("CUDA range execution failed: {error}"),
    }
}

/// The kernels and the stream they run on, for one call.
struct Gpu<'a> {
    context: &'a Arc<CudaContext>,
    stream: Arc<CudaStream>,
    module: &'a kernels::LoadedModule,
}

impl<'a> Gpu<'a> {
    /// Makes the context current on this thread (calls may come from any thread), for one call.
    fn new(cuda: &'a Cuda) -> Result<Self, Failure> {
        cuda.context.bind_to_thread()?;
        Ok(Self {
            context: &cuda.context,
            stream: cuda.context.default_stream(),
            module: &cuda.module,
        })
    }

    /// A launch of one 256-thread block per `per_block` of `items`.
    fn blocks(items: usize, per_block: u32) -> LaunchConfig1D {
        let blocks = u32::try_from(items.div_ceil(per_block as usize).max(1))
            .expect("CUDA grid exceeds u32 blocks");
        LaunchConfig1D::new(blocks, BLOCK, 0)
    }

    /// A launch of one thread per item.
    fn threads(items: usize) -> LaunchConfig1D {
        Self::blocks(items, BLOCK)
    }

    /// A launch of one block per tile of `items`.
    fn tiles(items: usize) -> LaunchConfig1D {
        Self::blocks(items, TILE)
    }

    /// A stream-ordered (`cuMemAllocAsync`) buffer that a kernel or copy fills before anything
    /// reads it. Plain `cuMemFree` of large buffers is slow (about 95 ms per 100M-key call under
    /// WSL), so free buffers with [`Gpu::release`].
    fn scratch<T: DeviceCopy>(&self, len: usize) -> Result<DeviceBuffer<T>, Failure> {
        // SAFETY: every caller writes the whole buffer (by copy or kernel) before reading it.
        Ok(unsafe { DeviceBuffer::uninitialized_async(&self.stream, len)? })
    }

    /// A zeroed stream-ordered buffer.
    fn zeroed<T: DeviceCopy>(&self, len: usize) -> Result<DeviceBuffer<T>, Failure> {
        let mut buffer = self.scratch(len)?;
        buffer.zero_async(&self.stream)?;
        Ok(buffer)
    }

    /// Frees `buffers` in stream order. (Dropping a buffer instead synchronizes the context.)
    fn release<T: DeviceCopy>(
        &self,
        buffers: impl IntoIterator<Item = DeviceBuffer<T>>,
    ) -> Result<(), Failure> {
        for buffer in buffers {
            // SAFETY: all of this module's work runs on `self.stream`, so nothing else can still
            // be using the buffer when the free runs.
            unsafe { buffer.drop_async(&self.stream)? };
        }
        Ok(())
    }

    /// Waits for all queued work, so the freed memory is back with the driver.
    fn finish(&self) -> Result<(), Failure> {
        Ok(self.stream.synchronize()?)
    }

    /// Encodes `values` into a new device buffer. Large inputs go a chunk at a time through two
    /// pinned host buffers, so encoding one chunk overlaps copying the one before it. (Encoding
    /// all of a large input into fresh memory first is slow: its page faults cost more than the
    /// copy.)
    fn upload<T: Copy>(
        &self,
        values: &[T],
        encode: impl Fn(T) -> u32,
    ) -> Result<DeviceBuffer<u32>, Failure> {
        let keys = self.scratch(values.len())?;
        if values.len() <= PAGEABLE_UPLOAD {
            let encoded: Vec<u32> = values.iter().map(|&value| encode(value)).collect();
            // SAFETY: the destination is `keys`, of this length, and the copy finishes before
            // `encoded` is dropped (we synchronize below).
            unsafe {
                memcpy_htod_async(
                    keys.cu_deviceptr(),
                    encoded.as_ptr(),
                    size_of_val(encoded.as_slice()),
                    self.stream.cu_stream(),
                )?;
            }
            self.stream.synchronize()?;
            return Ok(keys);
        }
        let mut staging = [
            PinnedHostBuffer::<u32>::zeroed(self.context, CHUNK)?,
            PinnedHostBuffer::<u32>::zeroed(self.context, CHUNK)?,
        ];
        for (index, values) in values.chunks(CHUNK).enumerate() {
            let buffer = &mut staging[index % 2];
            for (key, &value) in buffer.iter_mut().zip(values) {
                *key = encode(value);
            }
            // The copy issued two chunks ago read this buffer; wait for it (and the one after).
            self.stream.synchronize()?;
            let offset = (index * CHUNK * size_of::<u32>()) as u64;
            // SAFETY: the destination range lies inside `keys`, the source is pinned and stays
            // untouched until the next synchronize, and both outlive the copy (we synchronize
            // below).
            unsafe {
                memcpy_htod_async(
                    keys.cu_deviceptr() + offset,
                    buffer.as_ptr(),
                    values.len() * size_of::<u32>(),
                    self.stream.cu_stream(),
                )?;
            }
        }
        self.stream.synchronize()?;
        Ok(keys)
    }

    /// Reads a scan's one-word total.
    fn read_count(&self, total: &DeviceBuffer<u32>) -> Result<usize, Failure> {
        Ok(total.to_host_vec(&self.stream)?[0] as usize)
    }

    /// Stable LSD radix sort of `keys`: up to four 8-bit passes between `keys` and `keys_spare`,
    /// carrying `values` (and their spare) along if given. The result ends up back in `keys` (and
    /// `values`). A pass where every key has the same digit would change nothing, so it is
    /// skipped.
    fn radix_sort(
        &self,
        keys: &mut DeviceBuffer<u32>,
        keys_spare: &mut DeviceBuffer<u32>,
        mut values: Option<(&mut DeviceBuffer<u32>, &mut DeviceBuffer<u32>)>,
    ) -> Result<(), Failure> {
        let len = u32::try_from(keys.len()).expect("CUDA input exceeds u32 positions");
        let module = self.module;
        let tiles = Self::tiles(keys.len());
        let mut digit_bases = self.zeroed::<u32>(4 * RADIX as usize)?;
        let prepared = module.prepare_digit_counts(tiles)?;
        // SAFETY: `digit_bases` holds 1,024 zeroed words, with one block per tile.
        unsafe {
            module.digit_counts(
                &self.stream,
                &prepared,
                keys,
                digit_bases.cu_deviceptr() as *mut u32,
            )?;
        }
        let counts = digit_bases.to_host_vec(&self.stream)?;
        // Where each digit starts, for each pass: exclusive sums of its counts.
        let bases: Vec<u32> = counts
            .chunks(RADIX as usize)
            .flat_map(|pass| {
                pass.iter().scan(0, |start, &count| {
                    let base = *start;
                    *start += count;
                    Some(base)
                })
            })
            .collect();
        digit_bases.copy_from_host(&self.stream, &bases)?;
        let tile_count = keys.len().div_ceil(TILE as usize);
        let no_values = self.scratch::<u32>(0)?;
        let mut status = self.scratch::<u64>(tile_count * RADIX as usize)?;
        let mut ticket = self.scratch::<u32>(1)?;
        let prepared = module.prepare_radix_scatter(tiles)?;
        for (pass, shift) in [0_u32, 8, 16, 24].into_iter().enumerate() {
            if counts[pass * RADIX as usize..][..RADIX as usize].contains(&len) {
                continue;
            }
            status.zero_async(&self.stream)?;
            ticket.zero_async(&self.stream)?;
            let (values_in, values_out) = values.as_ref().map_or_else(
                || (&no_values, core::ptr::null_mut()),
                |(values, values_spare)| (&**values, values_spare.cu_deviceptr() as *mut u32),
            );
            // SAFETY: `keys_spare` (and the values' spare, if any) hold `len` words, `status`
            // one zeroed word per tile and digit, and `ticket` one zeroed word; one block per
            // tile.
            unsafe {
                module.radix_scatter(
                    &self.stream,
                    &prepared,
                    keys,
                    values_in,
                    shift,
                    &digit_bases,
                    pass as u32 * RADIX,
                    status.cu_deviceptr() as *mut u64,
                    ticket.cu_deviceptr() as *mut u32,
                    keys_spare.cu_deviceptr() as *mut u32,
                    values_out,
                )?;
            }
            core::mem::swap(keys, keys_spare);
            if let Some((values, values_spare)) = &mut values {
                core::mem::swap(*values, *values_spare);
            }
        }
        self.release([status])?;
        self.release([digit_bases, no_values, ticket])
    }

    /// Exclusive scan of `data`, in place. Returns a one-word buffer holding the total.
    fn exclusive_scan(&self, data: &DeviceBuffer<u32>) -> Result<DeviceBuffer<u32>, Failure> {
        let module = self.module;
        let len = u32::try_from(data.len()).expect("CUDA scan exceeds u32 words");
        let tiles = data.len().div_ceil(TILE as usize).max(1);
        let tile_sums = self.scratch::<u32>(tiles)?;
        let prepared = module.prepare_scan_tiles(Self::tiles(data.len()))?;
        // SAFETY: `data` holds `len` words, and `tile_sums` one word per tile.
        unsafe {
            module.scan_tiles(
                &self.stream,
                &prepared,
                data.cu_deviceptr() as *mut u32,
                len,
                tile_sums.cu_deviceptr() as *mut u32,
            )?;
        }
        if tiles == 1 {
            return Ok(tile_sums);
        }
        let total = self.exclusive_scan(&tile_sums)?;
        let prepared = module.prepare_add_tile_offsets(Self::tiles(data.len()))?;
        // SAFETY: `data` holds `len` words, with one block per tile.
        unsafe {
            module.add_tile_offsets(
                &self.stream,
                &prepared,
                data.cu_deviceptr() as *mut u32,
                len,
                &tile_sums,
            )?;
        }
        self.release([tile_sums])?;
        Ok(total)
    }
}

/// The input length, or `None` if the input is empty.
fn length(len: usize) -> Option<usize> {
    (len != 0).then(|| {
        assert!(
            u32::try_from(len).is_ok(),
            "CUDA input exceeds u32 positions"
        );
        len
    })
}

fn try_ranges_32<T: Copy>(
    cuda: &Cuda,
    values: &[T],
    encode: impl Fn(T) -> u32,
    gap: Option<(u32, u32)>,
) -> Result<Vec<(u32, u32)>, Failure> {
    let Some(n) = length(values.len()) else {
        return Ok(Vec::new());
    };
    let gpu = Gpu::new(cuda)?;
    let module = gpu.module;
    let mut sorted = gpu.upload(values, encode)?;
    let mut marks = gpu.scratch(n)?;
    gpu.radix_sort(&mut sorted, &mut marks, None)?;

    // Ranges: mark each range's first key, scan the marks, then emit each range's ends.
    let (gap_left, gap_right) = gap.unwrap_or((0, 0));
    let has_gap = u32::from(gap.is_some());
    let prepared = module.prepare_mark_starts(Gpu::threads(n))?;
    module.mark_starts(
        &gpu.stream,
        &prepared,
        &sorted,
        gap_left,
        gap_right,
        has_gap,
        &mut marks,
    )?;
    let total = gpu.exclusive_scan(&marks)?;
    let count = gpu.read_count(&total)?;
    let starts = gpu.scratch::<u32>(count)?;
    let ends = gpu.scratch::<u32>(count)?;
    let prepared = module.prepare_emit_ranges(Gpu::threads(n))?;
    // SAFETY: `starts` and `ends` hold one word per range (the scan's total).
    unsafe {
        module.emit_ranges(
            &gpu.stream,
            &prepared,
            &sorted,
            &marks,
            gap_left,
            gap_right,
            has_gap,
            starts.cu_deviceptr() as *mut u32,
            ends.cu_deviceptr() as *mut u32,
        )?;
    }
    let starts_host = starts.to_host_vec(&gpu.stream)?;
    let ends_host = ends.to_host_vec(&gpu.stream)?;
    gpu.release([sorted, marks, total, starts, ends])?;
    gpu.finish()?;
    Ok(starts_host.into_iter().zip(ends_host).collect())
}

fn try_ranges_64<T: Copy>(
    cuda: &Cuda,
    values: &[T],
    encode: impl Fn(T) -> u64,
    gap: Option<(u64, u64)>,
) -> Result<Vec<(u64, u64)>, Failure> {
    let Some(n) = length(values.len()) else {
        return Ok(Vec::new());
    };
    let gpu = Gpu::new(cuda)?;
    let module = gpu.module;
    let mut lo = gpu.upload(values, |value| low_word(encode(value)))?;
    let mut hi = gpu.upload(values, |value| high_word(encode(value)))?;

    // Sort by the low word carrying the high word, then stably by the high word carrying the
    // low word: the keys end up in `(hi, lo)` order.
    let mut lo_spare = gpu.scratch(n)?;
    let mut hi_spare = gpu.scratch(n)?;
    gpu.radix_sort(&mut lo, &mut lo_spare, Some((&mut hi, &mut hi_spare)))?;
    gpu.radix_sort(&mut hi, &mut hi_spare, Some((&mut lo, &mut lo_spare)))?;

    let (gap_left, gap_right) = gap.unwrap_or((0, 0));
    let (gap_left_hi, gap_left_lo) = (high_word(gap_left), low_word(gap_left));
    let (gap_right_hi, gap_right_lo) = (high_word(gap_right), low_word(gap_right));
    let has_gap = u32::from(gap.is_some());
    let mut marks = lo_spare;
    let prepared = module.prepare_mark_starts_64(Gpu::threads(n))?;
    module.mark_starts_64(
        &gpu.stream,
        &prepared,
        &hi,
        &lo,
        gap_left_hi,
        gap_left_lo,
        gap_right_hi,
        gap_right_lo,
        has_gap,
        &mut marks,
    )?;
    let total = gpu.exclusive_scan(&marks)?;
    let count = gpu.read_count(&total)?;
    let outputs = [
        gpu.scratch::<u32>(count)?,
        gpu.scratch::<u32>(count)?,
        gpu.scratch::<u32>(count)?,
        gpu.scratch::<u32>(count)?,
    ];
    let [starts_hi, starts_lo, ends_hi, ends_lo] = outputs
        .each_ref()
        .map(|buffer| buffer.cu_deviceptr() as *mut u32);
    let prepared = module.prepare_emit_ranges_64(Gpu::threads(n))?;
    // SAFETY: each of the four outputs holds one word per range (the scan's total).
    unsafe {
        module.emit_ranges_64(
            &gpu.stream,
            &prepared,
            &hi,
            &lo,
            &marks,
            gap_left_hi,
            gap_left_lo,
            gap_right_hi,
            gap_right_lo,
            has_gap,
            starts_hi,
            starts_lo,
            ends_hi,
            ends_lo,
        )?;
    }
    let mut columns = Vec::with_capacity(outputs.len());
    for buffer in &outputs {
        columns.push(buffer.to_host_vec(&gpu.stream)?);
    }
    gpu.release([hi, lo, hi_spare, marks, total])?;
    gpu.release(outputs)?;
    gpu.finish()?;
    Ok((0..count)
        .map(|range| {
            (
                join_words(columns[0][range], columns[1][range]),
                join_words(columns[2][range], columns[3][range]),
            )
        })
        .collect())
}

fn try_map_ranges(
    cuda: &Cuda,
    keys: &[u32],
    gap: Option<(u32, u32)>,
    value_lo: &[u32],
    value_hi: Option<&[u32]>,
) -> Result<MapRanges, Failure> {
    if length(keys.len()).is_none() {
        return Ok((Vec::new(), Vec::new()));
    }
    let gpu = Gpu::new(cuda)?;
    let module = gpu.module;
    let (survivor_keys, survivor_positions) = survivors(&gpu, keys)?;
    let survivors = survivor_keys.len();

    // Ranges among the survivors: keys adjacent and values equal.
    let lo_device = gpu.upload(value_lo, |word| word)?;
    // No high words: an empty slice, so the kernels compare one word.
    let hi_device = match value_hi {
        Some(value_hi) => gpu.upload(value_hi, |word| word)?,
        None => gpu.scratch(0)?,
    };
    let (gap_left, gap_right) = gap.unwrap_or((0, 0));
    let has_gap = u32::from(gap.is_some());
    let mut marks = gpu.scratch(survivors)?;
    let prepared = module.prepare_mark_map_starts(Gpu::threads(survivors))?;
    module.mark_map_starts(
        &gpu.stream,
        &prepared,
        &survivor_keys,
        &survivor_positions,
        &lo_device,
        &hi_device,
        gap_left,
        gap_right,
        has_gap,
        &mut marks,
    )?;
    let total = gpu.exclusive_scan(&marks)?;
    let count = gpu.read_count(&total)?;
    let starts = gpu.scratch::<u32>(count)?;
    let ends = gpu.scratch::<u32>(count)?;
    let firsts = gpu.scratch::<u32>(count)?;
    let prepared = module.prepare_emit_map_ranges(Gpu::threads(survivors))?;
    // SAFETY: `starts`, `ends`, and `firsts` hold one word per range (the scan's total).
    unsafe {
        module.emit_map_ranges(
            &gpu.stream,
            &prepared,
            &survivor_keys,
            &survivor_positions,
            &lo_device,
            &hi_device,
            &marks,
            gap_left,
            gap_right,
            has_gap,
            starts.cu_deviceptr() as *mut u32,
            ends.cu_deviceptr() as *mut u32,
            firsts.cu_deviceptr() as *mut u32,
        )?;
    }
    let starts_host = starts.to_host_vec(&gpu.stream)?;
    let ends_host = ends.to_host_vec(&gpu.stream)?;
    let firsts_host = firsts.to_host_vec(&gpu.stream)?;
    gpu.release([
        survivor_keys,
        survivor_positions,
        lo_device,
        hi_device,
        marks,
        total,
        starts,
        ends,
        firsts,
    ])?;
    gpu.finish()?;
    Ok((
        starts_host.into_iter().zip(ends_host).collect(),
        firsts_host,
    ))
}

/// Sorts the keys carrying their input positions, then keeps the last entry of each key: the
/// sort is stable, so that entry has the key's last value. Returns the survivors' keys and
/// positions, in key order.
fn survivors(
    gpu: &Gpu<'_>,
    keys: &[u32],
) -> Result<(DeviceBuffer<u32>, DeviceBuffer<u32>), Failure> {
    let module = gpu.module;
    let n = keys.len();
    let mut sorted = gpu.upload(keys, |key| key)?;
    let mut positions = gpu.scratch(n)?;
    let prepared = module.prepare_fill_positions(Gpu::threads(n))?;
    module.fill_positions(&gpu.stream, &prepared, &mut positions)?;
    let mut sorted_spare = gpu.scratch(n)?;
    let mut positions_spare = gpu.scratch(n)?;
    gpu.radix_sort(
        &mut sorted,
        &mut sorted_spare,
        Some((&mut positions, &mut positions_spare)),
    )?;

    let mut marks = sorted_spare;
    let prepared = module.prepare_mark_survivors(Gpu::threads(n))?;
    module.mark_survivors(&gpu.stream, &prepared, &sorted, &mut marks)?;
    let total = gpu.exclusive_scan(&marks)?;
    let count = gpu.read_count(&total)?;
    let survivor_keys = gpu.scratch::<u32>(count)?;
    let survivor_positions = gpu.scratch::<u32>(count)?;
    let prepared = module.prepare_compact_survivors(Gpu::threads(n))?;
    // SAFETY: both outputs hold one word per survivor (the scan's total).
    unsafe {
        module.compact_survivors(
            &gpu.stream,
            &prepared,
            &sorted,
            &positions,
            &marks,
            survivor_keys.cu_deviceptr() as *mut u32,
            survivor_positions.cu_deviceptr() as *mut u32,
        )?;
    }
    gpu.release([sorted, positions, positions_spare, marks, total])?;
    Ok((survivor_keys, survivor_positions))
}

#[cfg(test)]
mod tests {
    use std::vec::Vec;

    use crate::{Engine, RangeMapBlaze, RangeSetBlaze};

    macro_rules! check {
        ($values:expr) => {{
            let values = $values;
            let expected = RangeSetBlaze::from_iter(values.iter().copied());
            let found = super::super::from_shuffled_slice(&mut values.to_vec(), Engine::Cuda);
            assert_eq!(found, expected, "len {}", values.len());
        }};
    }

    fn random(seed: u64) -> impl FnMut() -> u64 {
        let mut state = seed | 1;
        move || {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            state
        }
    }

    #[test]
    fn small_and_edge_cases() {
        if super::cuda_context().is_none() {
            return;
        }
        check!(&[] as &[u32]);
        check!(&[7_u32]);
        check!(&[u32::MAX, 0, u32::MAX - 1, 1, 5, 5, 5]);
        check!(&[3_u32, 2, 1, 0]);
        check!(&[i32::MIN, -1, 0, i32::MAX, i32::MAX - 1]);
        // `char` ranges join across the surrogates.
        check!(&['\u{D7FF}', '\u{E000}', 'a', 'b', 'd']);
    }

    #[test]
    fn random_cases_across_tile_boundaries() {
        if super::cuda_context().is_none() {
            return;
        }
        let mut next = random(0x9e37_79b9_7f4a_7c15);
        for len in [
            1, 31, 32, 255, 256, 257, 4095, 4096, 4097, 8192, 100_000, 1_000_003,
        ] {
            for spread in [4_u64, 1_000, u64::from(u32::MAX)] {
                let values: Vec<u32> = (0..len)
                    .map(|_| u32::try_from(next() % spread).expect("fits in u32"))
                    .collect();
                check!(&values);
            }
        }
    }

    #[test]
    fn random_64_bit_cases_across_tile_boundaries() {
        if super::cuda_context().is_none() {
            return;
        }
        let mut next = random(0x2545_f491_4f6c_dd1d);
        for len in [1, 33, 4096, 4097, 100_000] {
            // Spreads that keep keys within one low word, cross it, and use the whole range.
            for (base, spread) in [
                (0_u64, 1_000_u64),
                (u64::from(u32::MAX) - 500, 1_000),
                (0, u64::MAX),
            ] {
                let values: Vec<u64> = (0..len)
                    .map(|_| base.wrapping_add(next() % spread))
                    .collect();
                check!(&values);
                let signed: Vec<i64> = values.iter().map(|&value| value.cast_signed()).collect();
                check!(&signed);
            }
        }
        check!(&[u64::MAX, 0, u64::MAX - 1, 1, 5, 5, 5]);
    }

    #[test]
    fn random_maps_across_tile_boundaries() {
        if super::cuda_context().is_none() {
            return;
        }
        let mut next = random(0x1405_7b7e_f767_814f);
        for len in [1, 33, 4096, 4097, 100_000, 1_000_003] {
            for (key_spread, value_spread) in [(64_u64, 2_u64), (100_000, 3), (u64::MAX, u64::MAX)]
            {
                let keys: Vec<u32> = (0..len)
                    .map(|_| u32::try_from(next() % key_spread.min(1 << 32)).expect("fits"))
                    .collect();
                let narrow: Vec<u32> = (0..len)
                    .map(|_| u32::try_from(next() % value_spread.min(1 << 32)).expect("fits"))
                    .collect();
                let wide: Vec<u64> = narrow
                    .iter()
                    .map(|&value| u64::from(value) << 32 | u64::from(value))
                    .collect();
                check_map(&keys, &narrow);
                check_map(&keys, &wide);
            }
        }
    }

    fn check_map<V: crate::Integer + Eq + Clone + core::fmt::Debug>(keys: &[u32], values: &[V]) {
        let expected: RangeMapBlaze<u32, V> =
            keys.iter().copied().zip(values.iter().cloned()).collect();
        let found = RangeMapBlaze::from_shuffled_slices(keys, values, Engine::Cuda);
        assert_eq!(found, expected, "len {}", keys.len());
    }

    #[test]
    fn cuda_or_cpu_falls_back_for_unsupported_types() {
        let set = RangeSetBlaze::<u128>::from_shuffled_slice(&mut [3, 1, 2, 9], Engine::CudaOrCpu);
        assert_eq!(set, RangeSetBlaze::from_iter([1_u128, 2, 3, 9]));
        let map = RangeMapBlaze::from_shuffled_slices(
            &[2_u32, 1, 2],
            &["b", "a", "c"],
            Engine::CudaOrCpu,
        );
        assert_eq!(map, RangeMapBlaze::from_iter([(1_u32, "a"), (2, "c")]));
    }

    #[test]
    #[should_panic(expected = "no GPU representation")]
    fn unsupported_keys_panic() {
        let _ = RangeSetBlaze::<u128>::from_shuffled_slice(&mut [1, 2], Engine::Cuda);
    }

    #[test]
    #[should_panic(expected = "no GPU representation")]
    fn unsupported_map_values_panic() {
        let _ = RangeMapBlaze::from_shuffled_slices(&[1_u32], &["a"], Engine::Cuda);
    }
}
