//! Stable 8-bit LSD radix sort, ported from the CUDA-Oxide engine's Onesweep algorithm.
//! Kernel pointers refer to complete allocations held by DeviceBuffer; offload's
//! pointer-argument mapping only needs to look up their already-associated base addresses.

#[cfg(any(target_arch = "nvptx64", target_arch = "amdgpu"))]
use super::device;
#[cfg(any(target_arch = "nvptx64", target_arch = "amdgpu"))]
use super::device::{block_idx_x, sync_block, thread_idx_x};
use super::key::Key;
#[cfg(target_os = "linux")]
use super::ranges;
#[cfg(target_os = "linux")]
use super::resident::DeviceBuffer;
use core::mem::size_of;
#[cfg(target_os = "linux")]
use core::offload::offload;
use core::offload::offload_kernel;
#[cfg(target_os = "linux")]
use core::slice::from_raw_parts_mut;
#[cfg(any(target_arch = "nvptx64", target_arch = "amdgpu"))]
use core::sync::atomic::{AtomicU64, Ordering};

const BLOCK: u32 = 256;
const RADIX: u32 = 256;
const TILE: u32 = 4096;
// Shared-memory bytes per kernel. NVIDIA declares them statically in PTX; AMD
// allocates them at launch (`dyn_cache`), so the host passes these sizes there.
const COUNTS_SHARED: usize = 8192;
/// `radix_scatter`'s shared bytes: a tile of keys, then 10,280 bytes of counters (2048
/// warp slots, two 256-entry prefix arrays, 8 warp totals, a ticket, all u32). Sizing the
/// key area by key width lets 32-bit sorts fit three blocks per SM instead of two.
const fn scatter_shared(key_bytes: usize) -> usize {
    TILE as usize * key_bytes + 10_280
}
#[cfg(target_os = "linux")]
const fn launch_shared(bytes: usize) -> u32 {
    if cfg!(rsb_offload_amdgpu) {
        bytes as u32
    } else {
        0
    }
}
#[cfg(any(target_arch = "nvptx64", target_arch = "amdgpu"))]
const WARPS: u32 = 8;
#[cfg(any(target_arch = "nvptx64", target_arch = "amdgpu"))]
const WARP_ITEMS: u32 = 512;
#[cfg(any(target_arch = "nvptx64", target_arch = "amdgpu"))]
const WARP_ROUNDS: u32 = 16;
#[cfg(any(target_arch = "nvptx64", target_arch = "amdgpu"))]
const AGGREGATE: u64 = 1 << 32;
#[cfg(any(target_arch = "nvptx64", target_arch = "amdgpu"))]
const INCLUSIVE: u64 = 2 << 32;
#[cfg(any(target_arch = "nvptx64", target_arch = "amdgpu"))]
const READY: u64 = AGGREGATE | INCLUSIVE;

/// Count every byte once, using block-local histograms before updating global counts.
#[offload_kernel]
pub(super) fn digit_counts<K: Key>(keys: *mut K, len: u32, passes: u32, counts: *mut [u64; 2048]) {
    // SAFETY: the launch has 256 threads; shared storage has 2048 aligned u32 counters.
    let (tid, block, shared) = unsafe {
        (
            thread_idx_x(),
            block_idx_x(),
            device::shared::<COUNTS_SHARED>().cast::<u32>(),
        )
    };
    for pass in 0..passes {
        // SAFETY: each thread initializes a distinct counter for each pass.
        unsafe { *shared.add((pass * RADIX + tid) as usize) = 0 };
    }
    // SAFETY: every thread reaches the barrier before any histogram updates.
    unsafe { sync_block() };
    for round in 0..16 {
        let index = block * TILE + round * BLOCK + tid;
        if index < len {
            // SAFETY: keys is resident for len initialized elements, and index is checked.
            let key = unsafe { *keys.add(index as usize) };
            for pass in 0..passes {
                // SAFETY: counters are initialized and shared. Plain block-scope atomics,
                // as in CUDA-Oxide: warp-aggregating with match_any first made this kernel
                // compute-bound (Nsight Compute: 92% SM, 10% memory throughput).
                unsafe {
                    device::add_shared(
                        shared.add((pass * RADIX + key.digit(pass * 8)) as usize),
                        1,
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
/// as in src/gpu/cuda.rs, with generic keys and stdarch/PTX operations here.
#[offload_kernel]
pub(super) fn radix_scatter<K: Key>(
    keys: *mut K,
    len: u32,
    shift: u32,
    digit_bases: *mut u64,
    bases_offset: u32,
    status: *mut u64,
    keys_out: *mut K,
) {
    // SAFETY: the block-shared allocation is aligned and contains these disjoint arrays:
    // 4096 keys, 2048 u32 warp slots, two 256-u32 prefix arrays, 8 warp totals, and one
    // ticket. Every read follows initialization and a warp/block barrier. The size test
    // is a compile-time constant, so each key width declares only its own storage.
    let (sorted_keys, warp_slots, tile_starts, global_starts, warp_totals, tile_index) = unsafe {
        let shared = if size_of::<K>() == 4 {
            device::shared::<{ scatter_shared(4) }>()
        } else {
            device::shared::<{ scatter_shared(8) }>()
        };
        let counters = TILE as usize * size_of::<K>();
        (
            shared.cast::<K>(),
            shared.add(counters).cast::<u32>(),
            shared.add(counters + 8192).cast::<u32>(),
            shared.add(counters + 9216).cast::<u32>(),
            shared.add(counters + 10_240).cast::<u32>(),
            shared.add(counters + 10_272).cast::<u32>(),
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
            unsafe { *keys.add(index as usize) }.digit(shift)
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
            K::ZERO
        };
        let digit = if valid { key.digit(shift) } else { RADIX };
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
        let digit = key.digit(shift) as usize;
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

/// Where `sort_on_device` left the sorted keys.
#[cfg(target_os = "linux")]
enum Sorted<'a, K: Key> {
    /// Every byte was constant, so the host keys were already sorted (all equal).
    OnHost,
    /// The sorted keys are in `sorted`; `free` is the other, now unused buffer.
    /// `in_keys` says whether `sorted` is associated with the caller's `keys`.
    OnDevice {
        sorted: DeviceBuffer<'a, K>,
        free: DeviceBuffer<'a, K>,
        in_keys: bool,
    },
}

/// Uploads keys and radix-sorts them on the GPU, leaving the result there.
#[cfg(target_os = "linux")]
fn sort_on_device<'a, K: Key>(
    keys: &'a mut [K],
    scratch: &'a mut [K],
    workspace: &mut [u64],
    bits: u32,
) -> Sorted<'a, K> {
    assert!(matches!(bits, 32 | 64));
    let len = keys.len();
    let passes = bits / 8;
    let blocks = (len as u32).div_ceil(TILE);
    let workspace_len = 1 + blocks as usize * RADIX as usize;
    assert_eq!(workspace.len(), workspace_len, "offload workspace length");
    let mut input = DeviceBuffer::upload(keys);
    let mut histogram = [0_u64; 2048];
    {
        let mut counts = DeviceBuffer::upload(&mut histogram);
        offload! { kernel = digit_counts::<K>, workgroup_dim = [blocks, 1, 1],
        thread_dim = [BLOCK, 1, 1], dyn_cache = launch_shared(COUNTS_SHARED), device = 0,
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
        return Sorted::OnHost;
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
        offload! { kernel = radix_scatter::<K>, workgroup_dim = [blocks, 1, 1],
        thread_dim = [BLOCK, 1, 1], dyn_cache = launch_shared(scatter_shared(size_of::<K>())), device = 0,
        args = (from, len as u32, pass as u32 * 8, bases.pointer(),
            pass as u32 * 256, status.pointer(), to), }
        in_original = !in_original;
    }
    if in_original {
        Sorted::OnDevice {
            sorted: input,
            free: output,
            in_keys: true,
        }
    } else {
        Sorted::OnDevice {
            sorted: output,
            free: input,
            in_keys: false,
        }
    }
}

/// # Safety
/// keys and scratch must point to len distinct initialized host words, and workspace
/// to 1 + ceil(len / 4096) * 256 words, all exclusively borrowed. Device 0 must exist.
/// bits must be 32 or 64 and cover every key's significant bits; len is in 2..=2^30.
#[cfg(target_os = "linux")]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rsb_offload_radix_sort(
    keys: *mut u64,
    scratch: *mut u64,
    workspace: *mut u64,
    len: usize,
    bits: u32,
) {
    let workspace_len = 1 + len.div_ceil(TILE as usize) * RADIX as usize;
    // SAFETY: all three disjoint host allocations and their exclusive lifetimes are
    // provided by the caller; their lengths follow the documented workspace formula.
    let (keys, scratch, workspace) = unsafe {
        (
            from_raw_parts_mut(keys, len),
            from_raw_parts_mut(scratch, len),
            from_raw_parts_mut(workspace, workspace_len),
        )
    };
    match sort_on_device(keys, scratch, workspace, bits) {
        Sorted::OnHost => {}
        Sorted::OnDevice {
            sorted,
            free,
            in_keys: true,
        } => {
            drop(free);
            sorted.download();
        }
        Sorted::OnDevice {
            sorted,
            free,
            in_keys: false,
        } => free.download_from(sorted),
    }
}

/// `rsb_offload_radix_ranges` left the sorted keys in `keys`; assemble ranges on the host.
pub const RANGES_SORTED_IN_KEYS: u32 = 0;
/// The `(first, last)` range pairs are in `keys[..2 * count]`.
pub const RANGES_IN_KEYS: u32 = 1;
/// The `(first, last)` range pairs are in `scratch[..2 * count]`.
pub const RANGES_IN_SCRATCH: u32 = 2;
/// Every byte was constant, so all keys equal `keys[0]`: one range.
pub const RANGES_ALL_EQUAL: u32 = 3;

/// Sorts keys and finds their ranges on the GPU, downloading only the ranges. Returns
/// where the result is (`RANGES_*`) and stores the number of ranges in `count`. When
/// ranges would need more than `len` keys (most keys isolated), it downloads the
/// sorted keys instead (`RANGES_SORTED_IN_KEYS`), since that is no larger.
///
/// # Safety
/// As for `rsb_offload_radix_sort`, with 32-bit keys; `count` must be writable. Keys
/// `gap_left` and `gap_right` (which must fit in u32) also count as adjacent when
/// `has_gap` is 1.
#[cfg(target_os = "linux")]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rsb_offload_radix_ranges_u32(
    keys: *mut u32,
    scratch: *mut u32,
    workspace: *mut u64,
    len: usize,
    gap_left: u64,
    gap_right: u64,
    has_gap: u32,
    count: *mut usize,
) -> u32 {
    // SAFETY: forwarded from this function's own contract.
    unsafe {
        radix_ranges(
            keys, scratch, workspace, len, 32, gap_left, gap_right, has_gap, count,
        )
    }
}

/// As `rsb_offload_radix_ranges_u32`, for 64-bit keys whose significant bits `bits`
/// (32 or 64) covers.
///
/// # Safety
/// As for `rsb_offload_radix_ranges_u32`, with 64-bit keys.
#[cfg(target_os = "linux")]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rsb_offload_radix_ranges_u64(
    keys: *mut u64,
    scratch: *mut u64,
    workspace: *mut u64,
    len: usize,
    bits: u32,
    gap_left: u64,
    gap_right: u64,
    has_gap: u32,
    count: *mut usize,
) -> u32 {
    // SAFETY: forwarded from this function's own contract.
    unsafe {
        radix_ranges(
            keys, scratch, workspace, len, bits, gap_left, gap_right, has_gap, count,
        )
    }
}

/// # Safety
/// See `rsb_offload_radix_ranges_u32`.
#[cfg(target_os = "linux")]
#[allow(clippy::too_many_arguments)]
unsafe fn radix_ranges<K: Key>(
    keys: *mut K,
    scratch: *mut K,
    workspace: *mut u64,
    len: usize,
    bits: u32,
    gap_left: u64,
    gap_right: u64,
    has_gap: u32,
    count: *mut usize,
) -> u32 {
    let workspace_len = 1 + len.div_ceil(TILE as usize) * RADIX as usize;
    // SAFETY: as in rsb_offload_radix_sort.
    let (keys, scratch, workspace) = unsafe {
        (
            from_raw_parts_mut(keys, len),
            from_raw_parts_mut(scratch, len),
            from_raw_parts_mut(workspace, workspace_len),
        )
    };
    // SAFETY: the caller provides a writable count.
    let count = unsafe { &mut *count };
    let (mut sorted, mut free, in_keys) =
        match sort_on_device(&mut *keys, &mut *scratch, &mut *workspace, bits) {
            Sorted::OnHost => {
                *count = 1;
                return RANGES_ALL_EQUAL;
            }
            Sorted::OnDevice {
                sorted,
                free,
                in_keys,
            } => (sorted, free, in_keys),
        };
    let blocks = len.div_ceil(TILE as usize);
    let total = {
        // SAFETY: clear_workspace initializes every count before count_starts adds to it.
        let mut counts = unsafe { DeviceBuffer::scratch(&mut workspace[..blocks]) };
        offload! { kernel = clear_workspace,
        workgroup_dim = [(blocks as u32).div_ceil(BLOCK), 1, 1],
        thread_dim = [BLOCK, 1, 1], device = 0,
        args = (counts.pointer(), blocks as u32), }
        offload! { kernel = ranges::count_starts::<K>, workgroup_dim = [blocks as u32, 1, 1],
        thread_dim = [BLOCK, 1, 1], device = 0,
        args = (sorted.pointer(), len as u32, gap_left, gap_right, has_gap, counts.pointer()), }
        counts.download_prefix(blocks);
        // Exclusive scan: each tile's first range number.
        let mut total = 0;
        for word in &mut workspace[..blocks] {
            let tile_count = *word;
            *word = total;
            total += tile_count;
        }
        total as usize
    };
    if 2 * total > len {
        // The ranges are no smaller than the keys; download the sorted keys instead.
        if in_keys {
            drop(free);
            sorted.download();
        } else {
            free.download_from(sorted);
        }
        *count = 0;
        return RANGES_SORTED_IN_KEYS;
    }
    {
        let mut offsets = DeviceBuffer::upload(&mut workspace[..blocks]);
        offload! { kernel = ranges::emit_ranges::<K>, workgroup_dim = [blocks as u32, 1, 1],
        thread_dim = [BLOCK, 1, 1], dyn_cache = launch_shared(ranges::EMIT_SHARED), device = 0,
        args = (sorted.pointer(), len as u32, gap_left, gap_right, has_gap,
            offsets.pointer(), free.pointer()), }
    }
    drop(sorted);
    free.download_prefix(2 * total);
    *count = total;
    // The free buffer is the one the sorted keys were not in.
    if in_keys {
        RANGES_IN_SCRATCH
    } else {
        RANGES_IN_KEYS
    }
}
