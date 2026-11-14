//! Stable 8-bit LSD radix sort, ported from the CUDA-Oxide engine's Onesweep algorithm.
//! Kernel pointers refer to complete allocations held by DeviceBuffer; offload's
//! pointer-argument mapping only needs to look up their already-associated base addresses.

#[cfg(target_arch = "nvptx64")]
use super::device;
#[cfg(target_arch = "nvptx64")]
use super::device::{block_idx_x, sync_block, thread_idx_x};
#[cfg(not(target_arch = "nvptx64"))]
use super::resident::DeviceBuffer;
#[cfg(not(target_arch = "nvptx64"))]
use core::offload::offload;
use core::offload::offload_kernel;
#[cfg(not(target_arch = "nvptx64"))]
use core::slice::from_raw_parts_mut;
#[cfg(target_arch = "nvptx64")]
use core::sync::atomic::{AtomicU64, Ordering};

const BLOCK: u32 = 256;
const RADIX: u32 = 256;
const TILE: u32 = 4096;
#[cfg(target_arch = "nvptx64")]
const WARPS: u32 = 8;
#[cfg(target_arch = "nvptx64")]
const WARP_ITEMS: u32 = 512;
#[cfg(target_arch = "nvptx64")]
const WARP_ROUNDS: u32 = 16;
#[cfg(target_arch = "nvptx64")]
const AGGREGATE: u64 = 1 << 32;
#[cfg(target_arch = "nvptx64")]
const INCLUSIVE: u64 = 2 << 32;
#[cfg(target_arch = "nvptx64")]
const READY: u64 = AGGREGATE | INCLUSIVE;

/// Count every byte once, using block-local histograms before updating global counts.
#[offload_kernel]
fn digit_counts(keys: *mut u64, len: u32, passes: u32, counts: *mut [u64; 2048]) {
    // SAFETY: the launch has 256 threads; shared storage has 2048 aligned u32 counters.
    let (tid, block, shared) = unsafe {
        (
            thread_idx_x(),
            block_idx_x(),
            device::shared::<8192>().cast::<u32>(),
        )
    };
    for pass in 0..passes {
        // SAFETY: each thread initializes a distinct counter for each pass.
        unsafe { *shared.add((pass * RADIX + tid) as usize) = 0 };
    }
    // SAFETY: every thread reaches the barrier before any histogram updates.
    unsafe { sync_block() };
    let lane_below = (1_u32 << (tid % 32)).wrapping_sub(1);
    for round in 0..16 {
        let index = block * TILE + round * BLOCK + tid;
        let valid = index < len;
        // SAFETY: keys is resident for len initialized elements, and index is checked.
        let key = if valid {
            unsafe { *keys.add(index as usize) }
        } else {
            0
        };
        for pass in 0..passes {
            let digit = if valid {
                ((key >> (pass * 8)) & 255) as u32
            } else {
                RADIX
            };
            // SAFETY: all lanes, including invalid tail lanes, execute every match.
            let peers = unsafe { device::match_any(digit) };
            if valid && peers & lane_below == 0 {
                // SAFETY: counters are initialized and shared. One leader adds the
                // entire peer group, avoiding 32 contending updates for a constant byte.
                unsafe {
                    device::add_shared(
                        shared.add((pass * RADIX + digit) as usize),
                        peers.count_ones(),
                    );
                }
            }
        }
    }
    // SAFETY: every thread reaches this barrier before reading the completed histogram.
    unsafe { sync_block() };
    for pass in 0..passes {
        let slot = (pass * RADIX + tid) as usize;
        // SAFETY: the block barrier completed every shared counter update.
        let count = unsafe { *shared.add(slot) };
        if count != 0 {
            // SAFETY: counts maps 2048 initialized, aligned u64 counters. Atomics combine
            // contributions from blocks; the synchronous launch finishes before readback.
            unsafe {
                AtomicU64::from_ptr(counts.cast::<u64>().add(slot))
                    .fetch_add(u64::from(count), Ordering::Relaxed);
            }
        }
    }
}

/// Initialize this pass's ticket and look-back words before any scatter starts.
#[offload_kernel]
fn clear_workspace(workspace: *mut u64, len: u32) {
    // SAFETY: these intrinsics only read this thread's launch coordinates.
    let index = unsafe { block_idx_x() * BLOCK + thread_idx_x() };
    if index < len {
        // SAFETY: each thread owns one distinct, in-bounds word of the resident workspace.
        unsafe { *workspace.add(index as usize) = 0 };
    }
}

/// Stable scatter of one byte. Ticket order ensures look-back waits only for blocks
/// that have started. Each block uses warp ranking and shared-memory coalesced output,
/// as in src/gpu/cuda.rs, with u64 keys and stdarch/PTX operations here.
#[offload_kernel]
fn radix_scatter(
    keys: *mut u64,
    len: u32,
    shift: u32,
    digit_bases: *mut u64,
    bases_offset: u32,
    status: *mut u64,
    keys_out: *mut u64,
) {
    // SAFETY: the block-shared allocation is aligned and contains these disjoint arrays:
    // 4096 u64 keys, 2048 u32 warp slots, two 256-u32 prefix arrays, 8 warp totals,
    // and one ticket. Every read follows initialization and a warp/block barrier.
    let (sorted_keys, warp_slots, tile_starts, global_starts, warp_totals, tile_index) = unsafe {
        let shared = device::shared::<43048>();
        (
            shared.cast::<u64>(),
            shared.add(32768).cast::<u32>(),
            shared.add(40960).cast::<u32>(),
            shared.add(41984).cast::<u32>(),
            shared.add(43008).cast::<u32>(),
            shared.add(43040).cast::<u32>(),
        )
    };

    // Per warp and digit: first a count, then that warp's next slot in the tile.
    // SAFETY: this launch has exactly 256 threads; this reads the local thread index.
    let tid = unsafe { thread_idx_x() };
    if tid == 0 {
        // SAFETY: reset initialized the ticket. A single leader claims this block's
        // unique tile and initializes its shared slot before the block barrier.
        unsafe {
            *tile_index.add(0) = AtomicU64::from_ptr(status).fetch_add(1, Ordering::Relaxed) as u32;
        }
    }
    // SAFETY: all 256 threads reach this block barrier before the next phase.
    unsafe { sync_block() };
    // SAFETY: the preceding block barrier published the leader's initialized ticket.
    let tile = unsafe { *tile_index.add(0) };
    let lane = tid % 32;
    let lane_below = (1_u32 << lane).wrapping_sub(1);
    let warp_index = tid / 32;
    let base = tile * TILE;
    let segment = base + warp_index * WARP_ITEMS;
    let mut w = 0;
    while w < WARPS {
        // SAFETY: each thread owns its digit column in all eight disjoint warp rows.
        unsafe {
            *warp_slots.add((w * RADIX + tid) as usize) = 0;
        }
        w += 1;
    }
    // SAFETY: all 256 threads reach this block barrier before the next phase.
    unsafe { sync_block() };

    // A: count each warp's digits.
    let mut round = 0;
    while round < WARP_ROUNDS {
        let index = segment + round * 32 + lane;
        let valid = index < len;
        // Past the end, a digit no real key has, so those lanes match only each other.
        let digit = if valid {
            // SAFETY: valid bounds this index within the fully mapped input.
            ((unsafe { *keys.add(index as usize) } >> shift) & 255) as u32
        } else {
            RADIX
        };
        // SAFETY: all 32 lanes execute this operation, including invalid tail lanes.
        let peers = unsafe { device::match_any(digit) };
        if valid && (peers & lane_below) == 0 {
            // SAFETY: only the first peer updates this warp/digit slot this round.
            unsafe {
                *warp_slots.add((warp_index * RADIX + digit) as usize) += peers.count_ones();
            }
        }
        // SAFETY: all warp lanes participate, ordering shared-slot reads and updates.
        unsafe { device::sync_warp() };
        round += 1;
    }
    // SAFETY: all 256 threads reach this block barrier before the next phase.
    unsafe { sync_block() };

    // B: thread `tid` owns digit `tid`. Prefix its counts across warps, then scan the tile's
    // digit totals across the block.
    let mut total = 0;
    w = 0;
    while w < WARPS {
        let slot = (w * RADIX + tid) as usize;
        // SAFETY: block synchronization completed counting, and tid owns this column.
        let count = unsafe { *warp_slots.add(slot) };
        // SAFETY: columns are disjoint; this replaces a count with its exclusive prefix.
        unsafe {
            *warp_slots.add(slot) = total;
        }
        total += count;
        w += 1;
    }
    let mut inclusive = total;
    let mut delta = 1;
    while delta < 32 {
        // SAFETY: all warp lanes participate with the same delta in 1..32.
        let up = unsafe { device::shuffle_up(inclusive, delta) };
        if lane >= delta {
            inclusive += up;
        }
        delta *= 2;
    }
    if lane == 31 {
        // SAFETY: one lane owns each of the eight warp-total slots.
        unsafe {
            *warp_totals.add(warp_index as usize) = inclusive;
        }
    }
    // Look back: the count of digit `tid` in all tiles before this one.
    // SAFETY: each (tile, digit) names one aligned, reset status word. Only this
    // ticket's block publishes it; other blocks access it through acquire loads.
    let slot =
        |tile: u32| unsafe { AtomicU64::from_ptr(status.add(1 + (tile * RADIX + tid) as usize)) };
    let prefix = if tile == 0 {
        0
    } else {
        slot(tile).store(AGGREGATE | u64::from(total), Ordering::Release);
        let mut prefix = 0;
        let mut previous = tile;
        loop {
            previous -= 1;
            let mut word = slot(previous).load(Ordering::Acquire);
            while word & READY == 0 {
                word = slot(previous).load(Ordering::Acquire);
            }
            prefix += word as u32;
            if word & INCLUSIVE != 0 {
                break;
            }
        }
        prefix
    };
    slot(tile).store(INCLUSIVE | u64::from(prefix + total), Ordering::Release);
    // SAFETY: tid owns this digit's start. Histogram totals bound all prefixes by len
    // (at most 2^30), so converting the u64 histogram prefix to u32 is exact.
    unsafe {
        *global_starts.add(tid as usize) =
            *digit_bases.add((bases_offset + tid) as usize) as u32 + prefix;
    }
    // SAFETY: all 256 threads reach this block barrier before the next phase.
    unsafe { sync_block() };
    let mut tile_start = inclusive - total;
    w = 0;
    while w < warp_index {
        // SAFETY: the preceding block barrier published all eight warp totals.
        tile_start += unsafe { *warp_totals.add(w as usize) };
        w += 1;
    }
    // SAFETY: each thread owns its digit's one tile-prefix slot.
    unsafe {
        *tile_starts.add(tid as usize) = tile_start;
    }
    w = 0;
    while w < WARPS {
        // SAFETY: each thread owns its column of previously initialized prefixes.
        unsafe {
            *warp_slots.add((w * RADIX + tid) as usize) += tile_start;
        }
        w += 1;
    }
    // SAFETY: all 256 threads reach this block barrier before the next phase.
    unsafe { sync_block() };

    // C: rank again and place each item in shared memory, in digit order.
    round = 0;
    while round < WARP_ROUNDS {
        let index = segment + round * 32 + lane;
        let valid = index < len;
        let key = if valid {
            // SAFETY: valid bounds this index within the resident input allocation.
            unsafe { *keys.add(index as usize) }
        } else {
            0
        };
        let digit = if valid {
            ((key >> shift) & 255) as u32
        } else {
            RADIX
        };
        // SAFETY: all 32 lanes execute this operation, including invalid tail lanes.
        let peers = unsafe { device::match_any(digit) };
        let rank = (peers & lane_below).count_ones();
        let slot = (warp_index * RADIX + digit) as usize;
        let first = if valid {
            // SAFETY: the block prefix phase initialized this warp/digit slot; warp
            // barriers separate subsequent rounds' reads and leader updates.
            unsafe { *warp_slots.add(slot) }
        } else {
            0
        };
        // SAFETY: all warp lanes participate, ordering shared-slot reads and updates.
        unsafe { device::sync_warp() };
        if valid {
            let place = (first + rank) as usize;
            // SAFETY: stable ranks assign every valid input one distinct tile slot.
            // Only its first peer advances this warp/digit's prefix for the next round.
            unsafe {
                *sorted_keys.add(place) = key;
                if rank == 0 {
                    *warp_slots.add(slot) = first + peers.count_ones();
                }
            }
        }
        // SAFETY: all warp lanes participate, ordering shared-slot reads and updates.
        unsafe { device::sync_warp() };
        round += 1;
    }
    // SAFETY: all 256 threads reach this block barrier before the next phase.
    unsafe { sync_block() };

    // D: copy out in digit order. Item `place` of digit `d` goes `place - *tile_starts.add(d)`
    // past this tile's global start for `d`.
    let in_tile = if len - base < TILE { len - base } else { TILE };
    let mut place = tid;
    while place < in_tile {
        // SAFETY: scatter initialized each slot below in_tile, then all threads synchronized.
        let key = unsafe { *sorted_keys.add(place as usize) };
        let digit = (((key >> shift) & 255) as u32) as usize;
        // SAFETY: both 256-entry prefix arrays were initialized before the block barrier.
        let destination =
            unsafe { *global_starts.add(digit) + place - *tile_starts.add(digit) } as usize;
        // SAFETY: histogram bases plus ticket-order prefixes assign disjoint destinations
        // below len, covering every output element exactly once in stable digit order.
        unsafe {
            *keys_out.add(destination) = key;
        }
        place += BLOCK;
    }
}

/// # Safety
/// keys and scratch must point to len distinct initialized host words, and workspace
/// to 1 + ceil(len / 4096) * 256 words, all exclusively borrowed. Device 0 must exist.
/// bits must be 32 or 64 and cover every key's significant bits; len is in 2..=2^30.
#[cfg(not(target_arch = "nvptx64"))]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rsb_offload_radix_sort(
    keys: *mut u64,
    scratch: *mut u64,
    workspace: *mut u64,
    len: usize,
    bits: u32,
) {
    assert!(matches!(bits, 32 | 64));
    let passes = bits / 8;
    let blocks = (len as u32).div_ceil(TILE);
    let workspace_len = 1 + blocks as usize * RADIX as usize;
    // SAFETY: all three disjoint host allocations and their exclusive lifetimes are
    // provided by the caller; their lengths follow the documented workspace formula.
    let (keys, scratch, workspace) = unsafe {
        (
            from_raw_parts_mut(keys, len),
            from_raw_parts_mut(scratch, len),
            from_raw_parts_mut(workspace, workspace_len),
        )
    };
    let mut input = DeviceBuffer::upload(keys);
    let mut histogram = [0_u64; 2048];
    {
        let mut counts = DeviceBuffer::upload(&mut histogram);
        offload! { kernel = digit_counts, workgroup_dim = [blocks, 1, 1],
        thread_dim = [BLOCK, 1, 1], device = 0,
        args = (input.pointer(), len as u32, passes, counts.kernel_argument::<2048>()), }
        counts.download();
    }
    let mut active = [false; 8];
    for pass in 0..passes as usize {
        let mut total = 0;
        let mut largest = 0;
        for digit in 0..256 {
            let slot = pass * 256 + digit;
            let count = histogram[slot];
            histogram[slot] = total;
            total += count;
            largest = largest.max(count);
        }
        assert_eq!(total, len as u64, "offload digit histogram is inconsistent");
        active[pass] = largest != len as u64;
    }
    if !active.iter().any(|&pass| pass) {
        return; // Every byte is constant: the original host keys are already sorted.
    }
    let mut bases = DeviceBuffer::upload(&mut histogram);
    // SAFETY: each scatter completely initializes its output before another pass or
    // download reads it, and clear_workspace initializes every status word before use.
    let (mut output, mut status) = unsafe {
        (
            DeviceBuffer::scratch(scratch),
            DeviceBuffer::scratch(workspace),
        )
    };
    let mut in_original = true;
    for pass in 0..passes as usize {
        if !active[pass] {
            continue;
        }
        offload! { kernel = clear_workspace,
        workgroup_dim = [(workspace_len as u32).div_ceil(BLOCK), 1, 1],
        thread_dim = [BLOCK, 1, 1], device = 0,
        args = (status.pointer(), workspace_len as u32), }
        let (from, to) = if in_original {
            (input.pointer(), output.pointer())
        } else {
            (output.pointer(), input.pointer())
        };
        offload! { kernel = radix_scatter, workgroup_dim = [blocks, 1, 1],
        thread_dim = [BLOCK, 1, 1], device = 0,
        args = (from, len as u32, pass as u32 * 8, bases.pointer(),
            pass as u32 * 256, status.pointer(), to), }
        in_original = !in_original;
    }
    if in_original {
        input.download();
    } else {
        input.download_from(output);
    }
}
