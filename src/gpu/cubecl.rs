//! Experimental `CubeCL` engine: one set of Rust kernels that `CubeCL` compiles for NVIDIA (CUDA)
//! or AMD (HIP), whichever GPU is present. It sorts keys encoded as `u32` or `u64` and returns
//! their ranges.
//!
//! **Sort**: least-significant digit first, 4-bit digits of `key - min`, and only as many passes
//! as `max - min` has bits. Each pass takes three steps:
//!
//! 1. `digit_counts`: each tile of [`TILE`] keys counts its keys of each digit, stored
//!    digit-major (`counts[digit * tiles + tile]`), so one exclusive scan of the whole array gives
//!    every (digit, tile) pair its first output position.
//! 2. `scan`: that exclusive scan, on the GPU. Each tile scans its slice, one cube scans the
//!    tile totals, then each slice adds its offset.
//! 3. `scatter`: each unit owns [`ITEMS`] consecutive keys. A cube-wide scan of the per-unit
//!    digit counts gives each unit its first slot for each digit, so walking its keys in order
//!    keeps equal digits in order, as a least-significant-digit sort must.
//!
//! **Ranges**: `count_starts` counts the keys that start a range in each tile, `scan` turns the
//! counts into each tile's first range number, and `emit_ranges` writes each range's first and
//! last key, interleaved.
//!
//! No cube waits for another cube, so the kernels don't rely on the GPU running all cubes at
//! once.
//!
//! **Copies**: `CubeCL`'s own copies are slow for large buffers, so keys and results are copied
//! with the CUDA or HIP driver through two reused pinned buffers, straight into or out of
//! `CubeCL`'s buffers ([`PinnedStaging`]).

use core::ffi::c_void;
use core::mem::{size_of, size_of_val};
use std::boxed::Box;
use std::env::var_os;
use std::sync::{Mutex, MutexGuard, OnceLock, PoisonError};
use std::vec::Vec;

use cubecl::{Device, prelude::*, server::Handle};
use rayon::{
    iter::{IndexedParallelIterator, ParallelIterator},
    slice::{ParallelSlice, ParallelSliceMut},
};

/// Units per cube.
const UNITS: u32 = 256;
/// Consecutive keys each unit owns.
const ITEMS: u32 = 16;
/// Keys per tile, one cube's share.
const TILE: u32 = UNITS * ITEMS;
/// Bits per digit, and the number of digit values.
const DIGIT_BITS: u32 = 4;
const DIGITS: u32 = 1 << DIGIT_BITS;
/// Bytes per pinned staging buffer, and so per copy chunk.
const CHUNK_BYTES: usize = 16 << 20;

/// Exclusive scan of one `u32` per unit across the cube. Returns this unit's exclusive prefix
/// and leaves the cube total in `total[0]`.
#[cube]
fn cube_exclusive_scan(
    value: u32,
    plane_totals: &mut Shared<[u32]>,
    total: &mut Shared<[u32]>,
) -> u32 {
    let inclusive = plane_inclusive_sum(value);
    let plane = UNIT_POS / PLANE_DIM;
    if UNIT_POS_PLANE == PLANE_DIM - 1 {
        plane_totals[plane as usize] = inclusive;
    }
    sync_cube();
    if UNIT_POS == 0 {
        let planes = UNITS / PLANE_DIM;
        let mut running = 0u32;
        let mut index = 0u32;
        while index < planes {
            let plane_total = plane_totals[index as usize];
            plane_totals[index as usize] = running;
            running += plane_total;
            index += 1u32;
        }
        total[0] = running;
    }
    sync_cube();
    let result = plane_totals[plane as usize] + inclusive - value;
    // Every unit reads `plane_totals` before anyone can reuse it.
    sync_cube();
    result
}

#[cube]
fn digit_of<K: Int>(key: K, min: K, shift: u32) -> u32 {
    u32::cast_from(((key - min) >> K::cast_from(shift)) & K::cast_from(DIGITS - 1))
}

/// Fills `columns[digit * UNITS + unit]` with how many of this unit's keys have each digit.
#[cube]
fn count_unit_digits<K: Int>(
    keys: &[K],
    len: u32,
    min: K,
    shift: u32,
    columns: &mut Shared<[u32]>,
) {
    let mut digit = 0u32;
    while digit < DIGITS {
        columns[(digit * UNITS + UNIT_POS) as usize] = 0u32;
        digit += 1u32;
    }
    let first = u32::cast_from(CUBE_POS) * TILE + UNIT_POS * ITEMS;
    let mut item = 0u32;
    while item < ITEMS {
        let index = first + item;
        if index < len {
            let digit = digit_of::<K>(keys[index as usize], min, shift);
            columns[(digit * UNITS + UNIT_POS) as usize] += 1u32;
        }
        item += 1u32;
    }
}

/// `params` is `[min, gap_left, gap_right]`.
#[cube(launch_unchecked)]
fn digit_counts<K: Int>(
    keys: &[K],
    len: u32,
    params: &[K],
    shift: u32,
    tiles: u32,
    counts: &mut [u32],
) {
    let mut columns = Shared::<[u32]>::new_slice((DIGITS * UNITS) as usize);
    count_unit_digits::<K>(keys, len, params[0], shift, &mut columns);
    sync_cube();
    if UNIT_POS < DIGITS {
        let mut sum = 0u32;
        let mut unit = 0u32;
        while unit < UNITS {
            sum += columns[(UNIT_POS * UNITS + unit) as usize];
            unit += 1u32;
        }
        counts[(UNIT_POS * tiles + u32::cast_from(CUBE_POS)) as usize] = sum;
    }
}

#[cube(launch_unchecked)]
fn scatter<K: Int>(
    keys: &[K],
    out: &mut [K],
    len: u32,
    params: &[K],
    shift: u32,
    tiles: u32,
    offsets: &[u32],
) {
    let min = params[0];
    let mut columns = Shared::<[u32]>::new_slice((DIGITS * UNITS) as usize);
    let mut plane_totals = Shared::<[u32]>::new_slice(UNITS as usize);
    let mut total = Shared::<[u32]>::new_slice(1usize);
    let mut digit_starts = Shared::<[u32]>::new_slice(DIGITS as usize);
    count_unit_digits::<K>(keys, len, min, shift, &mut columns);
    sync_cube();

    // Exclusive scan of `columns` in digit-major order; unit `t` owns entries
    // `t * ITEMS .. (t + 1) * ITEMS`. Afterward `columns[digit * UNITS + unit]` counts the tile's
    // keys with a smaller digit, or with the same digit in an earlier unit.
    let first = UNIT_POS * ITEMS;
    let mut sum = 0u32;
    let mut item = 0u32;
    while item < ITEMS {
        sum += columns[(first + item) as usize];
        item += 1u32;
    }
    let mut running = cube_exclusive_scan(sum, &mut plane_totals, &mut total);
    let mut item = 0u32;
    while item < ITEMS {
        let slot = (first + item) as usize;
        let count = columns[slot];
        columns[slot] = running;
        running += count;
        item += 1u32;
    }
    sync_cube();
    if UNIT_POS < DIGITS {
        digit_starts[UNIT_POS as usize] = columns[(UNIT_POS * UNITS) as usize];
    }
    sync_cube();

    // Each unit walks its keys in order, so keys with the same digit keep their order.
    let tile = u32::cast_from(CUBE_POS);
    let first_key = tile * TILE + UNIT_POS * ITEMS;
    let mut item = 0u32;
    while item < ITEMS {
        let index = first_key + item;
        if index < len {
            let key = keys[index as usize];
            let digit = digit_of::<K>(key, min, shift);
            let slot = (digit * UNITS + UNIT_POS) as usize;
            let position = offsets[(digit * tiles + tile) as usize] + columns[slot]
                - digit_starts[digit as usize];
            columns[slot] += 1u32;
            out[position as usize] = key;
        }
        item += 1u32;
    }
}

/// Exclusive scan of each `TILE`-long slice of `data`, in place; each slice's total goes to
/// `totals[slice]`.
#[cube(launch_unchecked)]
fn scan_slices(data: &mut [u32], len: u32, totals: &mut [u32]) {
    let mut plane_totals = Shared::<[u32]>::new_slice(UNITS as usize);
    let mut total = Shared::<[u32]>::new_slice(1usize);
    let first = u32::cast_from(CUBE_POS) * TILE + UNIT_POS * ITEMS;
    let mut sum = 0u32;
    let mut item = 0u32;
    while item < ITEMS {
        let index = first + item;
        if index < len {
            sum += data[index as usize];
        }
        item += 1u32;
    }
    let mut running = cube_exclusive_scan(sum, &mut plane_totals, &mut total);
    let mut item = 0u32;
    while item < ITEMS {
        let index = first + item;
        if index < len {
            let count = data[index as usize];
            data[index as usize] = running;
            running += count;
        }
        item += 1u32;
    }
    if UNIT_POS == 0 {
        totals[CUBE_POS] = total[0];
    }
}

#[cube(launch_unchecked)]
fn add_slice_offsets(data: &mut [u32], len: u32, offsets: &[u32]) {
    let index = u32::cast_from(ABSOLUTE_POS);
    if index < len {
        data[index as usize] += offsets[(index / TILE) as usize];
    }
}

/// Whether the key at `index` starts a range: it's the first key, or it neither equals nor is
/// adjacent to the key before it (counting the gap, when `has_gap` is 1, as adjacent).
#[cube]
fn is_start<K: Int>(keys: &[K], index: u32, params: &[K], has_gap: u32) -> bool {
    // `index.max(1) - 1` keeps the read in bounds for the first key, whose result `index == 0`
    // decides anyway (GPU code may evaluate both sides of `||`).
    let left = keys[(index.max(1) - 1) as usize];
    let right = keys[index as usize];
    let joins = left == right
        || left + K::from_int(1) == right
        || (has_gap == 1 && left == params[1] && right == params[2]);
    index == 0 || !joins
}

#[cube(launch_unchecked)]
fn count_starts<K: Int>(keys: &[K], len: u32, params: &[K], has_gap: u32, counts: &mut [u32]) {
    let mut plane_totals = Shared::<[u32]>::new_slice(UNITS as usize);
    let mut total = Shared::<[u32]>::new_slice(1usize);
    let first = u32::cast_from(CUBE_POS) * TILE + UNIT_POS * ITEMS;
    let mut starts = 0u32;
    let mut item = 0u32;
    while item < ITEMS {
        let index = first + item;
        if index < len && is_start::<K>(keys, index, params, has_gap) {
            starts += 1u32;
        }
        item += 1u32;
    }
    cube_exclusive_scan(starts, &mut plane_totals, &mut total);
    if UNIT_POS == 0 {
        counts[CUBE_POS] = total[0];
    }
}

/// Writes each range's first and last key to `pairs[2 * range]` and `pairs[2 * range + 1]`.
#[cube(launch_unchecked)]
fn emit_ranges<K: Int>(
    keys: &[K],
    len: u32,
    params: &[K],
    has_gap: u32,
    offsets: &[u32],
    pairs: &mut [K],
) {
    let mut plane_totals = Shared::<[u32]>::new_slice(UNITS as usize);
    let mut total = Shared::<[u32]>::new_slice(1usize);
    let first = u32::cast_from(CUBE_POS) * TILE + UNIT_POS * ITEMS;
    let mut starts = 0u32;
    let mut item = 0u32;
    while item < ITEMS {
        let index = first + item;
        if index < len && is_start::<K>(keys, index, params, has_gap) {
            starts += 1u32;
        }
        item += 1u32;
    }
    // The number of ranges started before this unit's first key.
    let mut range = offsets[CUBE_POS] + cube_exclusive_scan(starts, &mut plane_totals, &mut total);
    let mut item = 0u32;
    while item < ITEMS {
        let index = first + item;
        if index < len {
            let key = keys[index as usize];
            if is_start::<K>(keys, index, params, has_gap) {
                pairs[(2 * range) as usize] = key;
                range += 1u32;
            }
            if index + 1 == len || is_start::<K>(keys, index + 1, params, has_gap) {
                pairs[(2 * range - 1) as usize] = key;
            }
        }
        item += 1u32;
    }
}

/// The key types the engine sorts: `u32` and `u64`.
pub(super) trait Key: Int + CubeElement + bytemuck::Pod + Ord + Send + Sync {
    fn into_u64(self) -> u64;
}

impl Key for u32 {
    fn into_u64(self) -> u64 {
        u64::from(self)
    }
}

impl Key for u64 {
    fn into_u64(self) -> u64 {
        self
    }
}

/// The GPU `CubeCL` runs on, found on first use.
#[derive(Clone)]
struct Gpu {
    device: Device,
    backend: Backend,
}

fn gpu() -> Option<&'static Gpu> {
    static GPU: OnceLock<Option<Gpu>> = OnceLock::new();
    GPU.get_or_init(|| {
        if let Ok(device) = Device::cuda(0) {
            return Some(Gpu {
                device,
                backend: Backend::Cuda,
            });
        }
        Device::rocm(0).ok().map(|device| Gpu {
            device,
            backend: Backend::Hip,
        })
    })
    .as_ref()
}

/// Whether a CUDA or HIP GPU is available. Set `RANGE_SET_BLAZE_REQUIRE_CUBECL_GPU` to make a
/// missing GPU an error instead (for tests that must run on hardware).
pub(super) fn available() -> bool {
    let available = gpu().is_some();
    if var_os("RANGE_SET_BLAZE_REQUIRE_CUBECL_GPU").is_some() {
        assert!(available, "the required CubeCL GPU is unavailable");
    }
    available
}

/// Encodes `values` as keys with `encode`, sorts the keys on the GPU, and returns their ranges:
/// maximal runs of equal or adjacent keys, also joined across `gap` (a pair of keys to treat as
/// adjacent). Call it inside the Rayon pool that should encode the keys. Panics if no GPU is
/// available.
pub(super) fn ranges<V, K>(
    values: &[V],
    encode: impl Fn(V) -> K + Sync,
    gap: Option<(K, K)>,
) -> Vec<(K, K)>
where
    V: Copy + Sync,
    K: Key,
{
    let gpu = gpu().expect("Engine::CubeCl: no CUDA or HIP GPU is available");
    let client = gpu.device.client();
    if values.is_empty() {
        return Vec::new();
    }
    let len = u32::try_from(values.len()).expect("Engine::CubeCl sorts at most u32::MAX keys");
    let len_usize = values.len();
    let key_bytes = size_of::<K>();
    let mut current = client.empty(len_usize * key_bytes);
    let (min, max) = upload(gpu, &client, &current, values, encode);
    let tiles = len.div_ceil(TILE);
    let bits = u64::BITS - (max.into_u64() - min.into_u64()).leading_zeros();
    let passes = bits.div_ceil(DIGIT_BITS);
    let (gap_left, gap_right) = gap.unwrap_or((min, min));
    let has_gap = u32::from(gap.is_some());

    let params = client.create_from_slice(K::as_bytes(&[min, gap_left, gap_right]));
    let mut other = client.empty(len_usize * key_bytes);
    let counts_len = DIGITS * tiles;
    let counts = client.empty(counts_len as usize * size_of::<u32>());

    for pass in 0..passes {
        let shift = pass * DIGIT_BITS;
        // SAFETY: every kernel bounds-checks its indices against `len`; `counts` has
        // `DIGITS * tiles` entries; `scatter` writes a permutation of `0..len` into `other`.
        unsafe {
            digit_counts::launch_unchecked::<K>(
                &client,
                CubeCount::Static(tiles, 1, 1),
                CubeDim::new_1d(UNITS),
                BufferArg::from_raw_parts(current.clone(), len_usize),
                len,
                BufferArg::from_raw_parts(params.clone(), 3),
                shift,
                tiles,
                BufferArg::from_raw_parts(counts.clone(), counts_len as usize),
            );
        }
        scan(&client, &counts, counts_len);
        // SAFETY: as above.
        unsafe {
            scatter::launch_unchecked::<K>(
                &client,
                CubeCount::Static(tiles, 1, 1),
                CubeDim::new_1d(UNITS),
                BufferArg::from_raw_parts(current.clone(), len_usize),
                BufferArg::from_raw_parts(other.clone(), len_usize),
                len,
                BufferArg::from_raw_parts(params.clone(), 3),
                shift,
                tiles,
                BufferArg::from_raw_parts(counts.clone(), counts_len as usize),
            );
        }
        core::mem::swap(&mut current, &mut other);
    }

    let tile_counts = client.empty(tiles as usize * size_of::<u32>());
    // SAFETY: `count_starts` bounds-checks against `len` and writes one count per tile.
    unsafe {
        count_starts::launch_unchecked::<K>(
            &client,
            CubeCount::Static(tiles, 1, 1),
            CubeDim::new_1d(UNITS),
            BufferArg::from_raw_parts(current.clone(), len_usize),
            len,
            BufferArg::from_raw_parts(params.clone(), 3),
            has_gap,
            BufferArg::from_raw_parts(tile_counts.clone(), tiles as usize),
        );
    }
    let total = scan(&client, &tile_counts, tiles);
    let total = client
        .read_one(total)
        .expect("Engine::CubeCl: reading the range count");
    let range_count = u32::from_bytes(&total)[0] as usize;
    let pairs = client.empty(2 * range_count * key_bytes);
    // SAFETY: the scanned tile counts give each range a distinct number below `range_count`,
    // and `pairs` holds two keys per range.
    unsafe {
        emit_ranges::launch_unchecked::<K>(
            &client,
            CubeCount::Static(tiles, 1, 1),
            CubeDim::new_1d(UNITS),
            BufferArg::from_raw_parts(current, len_usize),
            len,
            BufferArg::from_raw_parts(params, 3),
            has_gap,
            BufferArg::from_raw_parts(tile_counts, tiles as usize),
            BufferArg::from_raw_parts(pairs.clone(), 2 * range_count),
        );
    }
    let pairs = download::<K>(gpu, &client, &pairs, 2 * range_count);
    pairs
        .as_chunks::<2>()
        .0
        .iter()
        .map(|&pair| pair.into())
        .collect()
}

/// Exclusive scan of the first `len` values of `data` in place, on the GPU. Returns a one-value
/// buffer holding the grand total.
fn scan(client: &Client, data: &Handle, len: u32) -> Handle {
    let slices = len.div_ceil(TILE);
    // The slice totals are scanned by one cube, so there can be at most `TILE` of them. That
    // covers `TILE * TILE` values: more than `DIGITS * tiles` for any `u32` key count.
    assert!(slices <= TILE, "scan supports at most TILE * TILE values");
    let totals = client.empty(slices as usize * size_of::<u32>());
    let grand_total = client.empty(size_of::<u32>());
    // SAFETY: each kernel bounds-checks against the length it's given; `totals` has one entry
    // per slice and `grand_total` one entry.
    unsafe {
        scan_slices::launch_unchecked(
            client,
            CubeCount::Static(slices, 1, 1),
            CubeDim::new_1d(UNITS),
            BufferArg::from_raw_parts(data.clone(), len as usize),
            len,
            BufferArg::from_raw_parts(totals.clone(), slices as usize),
        );
        scan_slices::launch_unchecked(
            client,
            CubeCount::Static(1, 1, 1),
            CubeDim::new_1d(UNITS),
            BufferArg::from_raw_parts(totals.clone(), slices as usize),
            slices,
            BufferArg::from_raw_parts(grand_total.clone(), 1),
        );
        add_slice_offsets::launch_unchecked(
            client,
            CubeCount::Static(len.div_ceil(UNITS), 1, 1),
            CubeDim::new_1d(UNITS),
            BufferArg::from_raw_parts(data.clone(), len as usize),
            len,
            BufferArg::from_raw_parts(totals, slices as usize),
        );
    }
    grand_total
}

/// Encodes `values` with `encode` into the GPU buffer `handle` and returns the smallest and
/// largest key. Each chunk is encoded in parallel straight into a pinned buffer while the chunk
/// before it copies, so there is no separate encoding pass, key vector, or min/max pass.
fn upload<V, K>(
    gpu: &Gpu,
    client: &Client,
    handle: &Handle,
    values: &[V],
    encode: impl Fn(V) -> K + Sync,
) -> (K, K)
where
    V: Copy + Sync,
    K: Key,
{
    /// Values each Rayon task encodes.
    const PIECE: usize = 1 << 16;
    let chunk_values = CHUNK_BYTES / size_of::<K>();
    let device_pointer = gpu.backend.device_pointer(client, handle);
    let staging = PinnedStaging::lock(gpu.backend);
    let mut min_max: Option<(K, K)> = None;
    for (chunk_index, chunk) in values.chunks(chunk_values).enumerate() {
        let slot = chunk_index % 2;
        staging.wait(slot);
        // SAFETY: `wait` finished the buffer's previous copy, so nothing else reads or writes
        // it; it holds CHUNK_BYTES, room for `chunk.len()` keys; pinned allocations are
        // page-aligned, so aligned for `K`; any bytes are a valid `u32`/`u64` (`Pod`).
        let keys = unsafe {
            core::slice::from_raw_parts_mut(staging.buffers[slot].cast::<K>(), chunk.len())
        };
        let chunk_min_max = keys
            .par_chunks_mut(PIECE)
            .zip(chunk.par_chunks(PIECE))
            .map(|(keys, values)| {
                let mut min_max: Option<(K, K)> = None;
                for (key, &value) in keys.iter_mut().zip(values) {
                    *key = encode(value);
                    min_max = Some(
                        min_max.map_or((*key, *key), |(min, max)| (min.min(*key), max.max(*key))),
                    );
                }
                min_max
            })
            .reduce(|| None, merge_min_max);
        min_max = merge_min_max(min_max, chunk_min_max);
        // SAFETY: the buffer stays untouched until `wait(slot)` again; the destination range
        // lies inside CubeCL's buffer, which `device_pointer`'s resource keeps in place.
        unsafe {
            staging.copy_to_device(
                slot,
                device_pointer.address + (chunk_index * CHUNK_BYTES) as u64,
                size_of_val(keys),
            );
        }
    }
    // Finish the copies before CubeCL's kernels, which run on CubeCL's own streams, read the
    // buffer.
    staging.finish();
    drop(staging);
    drop(device_pointer);
    min_max.expect("Engine::CubeCl: upload needs at least one value")
}

fn merge_min_max<K: Key>(left: Option<(K, K)>, right: Option<(K, K)>) -> Option<(K, K)> {
    match (left, right) {
        (Some((left_min, left_max)), Some((right_min, right_max))) => {
            Some((left_min.min(right_min), left_max.max(right_max)))
        }
        (one, None) | (None, one) => one,
    }
}

/// Copies `len` keys out of the GPU buffer `handle`.
fn download<K: Key>(gpu: &Gpu, client: &Client, handle: &Handle, len: usize) -> Vec<K> {
    let mut keys = std::vec![K::zeroed(); len];
    if len == 0 {
        return keys;
    }
    // These copies run on our own stream, so CubeCL's kernels that write `handle` must finish
    // first.
    pollster::block_on(client.sync()).expect("Engine::CubeCl: waiting for the GPU");
    let device_pointer = gpu.backend.device_pointer(client, handle);
    let staging = PinnedStaging::lock(gpu.backend);
    let mut pending: [Option<&mut [u8]>; 2] = [None, None];
    for (chunk_index, chunk) in bytemuck::cast_slice_mut::<K, u8>(&mut keys)
        .chunks_mut(CHUNK_BYTES)
        .enumerate()
    {
        let slot = chunk_index % 2;
        staging.wait(slot);
        if let Some(previous) = pending[slot].take() {
            staging.read_out(slot, previous);
        }
        // SAFETY: `wait` finished the buffer's previous copy and its bytes were read out; the
        // source range lies inside CubeCL's buffer, which `device_pointer`'s resource keeps in
        // place; the buffer holds CHUNK_BYTES, at least `chunk.len()`.
        unsafe {
            staging.copy_to_host(
                slot,
                device_pointer.address + (chunk_index * CHUNK_BYTES) as u64,
                chunk.len(),
            );
        }
        pending[slot] = Some(chunk);
    }
    for (slot, pending) in pending.iter_mut().enumerate() {
        staging.wait(slot);
        if let Some(previous) = pending.take() {
            staging.read_out(slot, previous);
        }
    }
    drop(staging);
    drop(device_pointer);
    keys
}

/// Which vendor API `CubeCL` runs on, and so which driver calls copy memory.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Backend {
    Cuda,
    Hip,
}

/// A `CubeCL` buffer's device address, and the resource that keeps the buffer in place while
/// it's held.
struct DevicePointer {
    address: u64,
    _resource: Box<dyn Send>,
}

impl Backend {
    fn device_pointer(self, client: &Client, handle: &Handle) -> DevicePointer {
        use cubecl_runtime::runtime::Runtime;
        match self {
            Self::Cuda => {
                let resource = client
                    .get_resource::<<cubecl::cuda::CudaRuntime as Runtime>::Server>(handle.clone())
                    .expect("Engine::CubeCl: CubeCL's CUDA buffer");
                DevicePointer {
                    address: resource.resource().ptr,
                    _resource: Box::new(resource),
                }
            }
            Self::Hip => {
                let resource = client
                    .get_resource::<<cubecl::hip::HipRuntime as Runtime>::Server>(handle.clone())
                    .expect("Engine::CubeCl: CubeCL's HIP buffer");
                DevicePointer {
                    address: resource.resource().ptr as u64,
                    _resource: Box::new(resource),
                }
            }
        }
    }
}

/// A stream and two reused pinned host buffers (with an event each), for chunked copies that
/// overlap filling one buffer with copying the other, as the CUDA-Oxide and HIP engines do.
/// `CubeCL`'s own copies are slow for large buffers (about 1.5 GB/s under WSL2) and sometimes
/// pin fresh memory, which costs about 0.6 ms per MB, so these buffers are allocated once and
/// kept.
struct PinnedStaging {
    backend: Backend,
    stream: *mut c_void,
    buffers: [*mut u8; 2],
    events: [*mut c_void; 2],
}

// SAFETY: the raw stream, buffers, and events are used only through `PinnedStaging::lock`,
// which holds the mutex, so no two threads use them at once. CUDA and HIP allow these handles to
// be used from any thread whose current device (context) is theirs, which `lock` sets.
unsafe impl Send for PinnedStaging {}

fn hip_check(status: cubecl_hip_sys::hipError_t, what: &str) {
    assert!(
        status == cubecl_hip_sys::HIP_SUCCESS,
        "Engine::CubeCl: {what} failed with HIP error {status}"
    );
}

impl PinnedStaging {
    /// The process's staging buffers, created on first use, with this thread set to use device
    /// 0 (for CUDA, its primary context, the one `CubeCL` uses).
    fn lock(backend: Backend) -> MutexGuard<'static, Self> {
        use cudarc::driver::result;
        static STAGING: OnceLock<Mutex<PinnedStaging>> = OnceLock::new();
        match backend {
            // SAFETY: `CubeCL` already opened device 0, so the driver is loaded and the device
            // exists; its primary context is valid for the life of the process.
            Backend::Cuda => unsafe {
                result::init().expect("Engine::CubeCl: cuInit");
                let device = result::device::get(0).expect("Engine::CubeCl: CUDA device 0");
                let context =
                    result::primary_ctx::retain(device).expect("Engine::CubeCl: primary context");
                result::ctx::set_current(context).expect("Engine::CubeCl: setting the context");
            },
            // SAFETY: `CubeCL` already opened HIP device 0, so the runtime is loaded.
            Backend::Hip => hip_check(unsafe { cubecl_hip_sys::hipSetDevice(0) }, "hipSetDevice"),
        }
        let staging = STAGING
            .get_or_init(|| Mutex::new(Self::new(backend)))
            .lock()
            .unwrap_or_else(PoisonError::into_inner);
        assert_eq!(
            staging.backend, backend,
            "Engine::CubeCl: one GPU per process"
        );
        staging
    }

    fn new(backend: Backend) -> Self {
        use cudarc::driver::{result, sys};
        match backend {
            Backend::Cuda => {
                let stream = result::stream::create(result::stream::StreamKind::NonBlocking)
                    .expect("Engine::CubeCl: creating a CUDA stream");
                // SAFETY: the context is current (set by `lock`); each buffer is CHUNK_BYTES of
                // pinned host memory, never freed, so it outlives every copy.
                let buffers = [0, 1].map(|_| unsafe {
                    result::malloc_host(CHUNK_BYTES, 0)
                        .expect("Engine::CubeCl: allocating pinned memory")
                        .cast::<u8>()
                });
                let events = [0, 1].map(|_| {
                    result::event::create(sys::CUevent_flags::CU_EVENT_DISABLE_TIMING)
                        .expect("Engine::CubeCl: creating a CUDA event")
                        .cast::<c_void>()
                });
                Self {
                    backend,
                    stream: stream.cast(),
                    buffers,
                    events,
                }
            }
            Backend::Hip => {
                let mut stream: cubecl_hip_sys::hipStream_t = core::ptr::null_mut();
                // SAFETY: each call writes its output through a valid pointer; the buffers are
                // CHUNK_BYTES of pinned host memory, never freed, so they outlive every copy.
                unsafe {
                    hip_check(
                        cubecl_hip_sys::hipStreamCreateWithFlags(
                            &raw mut stream,
                            cubecl_hip_sys::hipStreamNonBlocking,
                        ),
                        "hipStreamCreateWithFlags",
                    );
                }
                let buffers = [0, 1].map(|_| {
                    let mut buffer: *mut c_void = core::ptr::null_mut();
                    // SAFETY: as above.
                    hip_check(
                        unsafe {
                            cubecl_hip_sys::hipHostMalloc(
                                &raw mut buffer,
                                CHUNK_BYTES,
                                cubecl_hip_sys::hipHostMallocDefault,
                            )
                        },
                        "hipHostMalloc",
                    );
                    buffer.cast::<u8>()
                });
                let events = [0, 1].map(|_| {
                    let mut event: cubecl_hip_sys::hipEvent_t = core::ptr::null_mut();
                    // SAFETY: as above.
                    hip_check(
                        unsafe {
                            cubecl_hip_sys::hipEventCreateWithFlags(
                                &raw mut event,
                                cubecl_hip_sys::hipEventDisableTiming,
                            )
                        },
                        "hipEventCreateWithFlags",
                    );
                    event.cast::<c_void>()
                });
                Self {
                    backend,
                    stream: stream.cast(),
                    buffers,
                    events,
                }
            }
        }
    }

    /// Waits until buffer `slot`'s last copy (if any) has finished.
    fn wait(&self, slot: usize) {
        match self.backend {
            // SAFETY: the event is valid for the life of the process.
            Backend::Cuda => {
                unsafe { cudarc::driver::result::event::synchronize(self.events[slot].cast()) }
                    .expect("Engine::CubeCl: waiting for a copy");
            }
            // SAFETY: as above.
            Backend::Hip => hip_check(
                unsafe { cubecl_hip_sys::hipEventSynchronize(self.events[slot].cast()) },
                "hipEventSynchronize",
            ),
        }
    }

    /// Copies buffer `slot`'s first `destination.len()` bytes into `destination`. Call it only
    /// after `wait(slot)`.
    const fn read_out(&self, slot: usize, destination: &mut [u8]) {
        // SAFETY: after `wait(slot)` no copy touches the buffer, which holds CHUNK_BYTES, at
        // least `destination.len()` (a chunk).
        destination.copy_from_slice(unsafe {
            core::slice::from_raw_parts(self.buffers[slot], destination.len())
        });
    }

    /// Records buffer `slot`'s event after the copies queued so far.
    fn record(&self, slot: usize) {
        match self.backend {
            // SAFETY: the event and stream are valid for the life of the process.
            Backend::Cuda => unsafe {
                cudarc::driver::result::event::record(self.events[slot].cast(), self.stream.cast())
            }
            .expect("Engine::CubeCl: recording a copy"),
            // SAFETY: as above.
            Backend::Hip => hip_check(
                unsafe {
                    cubecl_hip_sys::hipEventRecord(self.events[slot].cast(), self.stream.cast())
                },
                "hipEventRecord",
            ),
        }
    }

    /// Queues a copy of buffer `slot`'s first `bytes` bytes to device address `destination`.
    ///
    /// # Safety
    ///
    /// The buffer must not be touched until `wait(slot)`, and `destination .. destination +
    /// bytes` must lie inside a live device allocation.
    unsafe fn copy_to_device(&self, slot: usize, destination: u64, bytes: usize) {
        match self.backend {
            // SAFETY: the caller upholds the contract; the stream is valid.
            Backend::Cuda => unsafe {
                cudarc::driver::sys::cuMemcpyHtoDAsync_v2(
                    destination,
                    self.buffers[slot].cast(),
                    bytes,
                    self.stream.cast(),
                )
                .result()
                .expect("Engine::CubeCl: copying keys to the GPU");
            },
            // SAFETY: as above. HIP device addresses are pointers.
            Backend::Hip => hip_check(
                unsafe {
                    cubecl_hip_sys::hipMemcpyHtoDAsync(
                        destination as cubecl_hip_sys::hipDeviceptr_t,
                        self.buffers[slot].cast(),
                        bytes,
                        self.stream.cast(),
                    )
                },
                "hipMemcpyHtoDAsync",
            ),
        }
        self.record(slot);
    }

    /// Queues a copy of `bytes` bytes from device address `source` into buffer `slot`.
    ///
    /// # Safety
    ///
    /// The buffer must not be touched until `wait(slot)`, and `source .. source + bytes` must
    /// lie inside a live device allocation.
    unsafe fn copy_to_host(&self, slot: usize, source: u64, bytes: usize) {
        match self.backend {
            // SAFETY: the caller upholds the contract; the stream is valid.
            Backend::Cuda => unsafe {
                cudarc::driver::sys::cuMemcpyDtoHAsync_v2(
                    self.buffers[slot].cast(),
                    source,
                    bytes,
                    self.stream.cast(),
                )
                .result()
                .expect("Engine::CubeCl: copying ranges from the GPU");
            },
            // SAFETY: as above. HIP device addresses are pointers.
            Backend::Hip => hip_check(
                unsafe {
                    cubecl_hip_sys::hipMemcpyDtoHAsync(
                        self.buffers[slot].cast(),
                        source as cubecl_hip_sys::hipDeviceptr_t,
                        bytes,
                        self.stream.cast(),
                    )
                },
                "hipMemcpyDtoHAsync",
            ),
        }
        self.record(slot);
    }

    /// Waits for every queued copy.
    fn finish(&self) {
        match self.backend {
            // SAFETY: the stream is valid for the life of the process.
            Backend::Cuda => {
                unsafe { cudarc::driver::result::stream::synchronize(self.stream.cast()) }
                    .expect("Engine::CubeCl: finishing the copies");
            }
            // SAFETY: as above.
            Backend::Hip => hip_check(
                unsafe { cubecl_hip_sys::hipStreamSynchronize(self.stream.cast()) },
                "hipStreamSynchronize",
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cpu_ranges<K: Key>(keys: &[K], gap: Option<(K, K)>) -> Vec<(K, K)> {
        let mut sorted = keys.to_vec();
        sorted.sort_unstable();
        let mut ranges: Vec<(K, K)> = Vec::new();
        for key in sorted {
            match ranges.last_mut() {
                Some((_, last))
                    if *last == key
                        || last.into_u64().checked_add(1) == Some(key.into_u64())
                        || gap == Some((*last, key)) =>
                {
                    *last = key;
                }
                _ => ranges.push((key, key)),
            }
        }
        ranges
    }

    fn splitmix(state: &mut u64) -> u64 {
        *state = state.wrapping_add(0x9e37_79b9_7f4a_7c15);
        let mut mixed = *state;
        mixed = (mixed ^ (mixed >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        mixed = (mixed ^ (mixed >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
        mixed ^ (mixed >> 31)
    }

    /// Every length class around the tile size, key spreads from one value to the full width
    /// (including keys at `u64::MAX`), with and without a gap.
    #[test]
    fn ranges_match_cpu() {
        if !available() {
            return;
        }
        let mut state = 7_u64;
        for len in [
            0_usize, 1, 2, 3, 255, 256, 4095, 4096, 4097, 10_000, 100_003,
        ] {
            for limit in [1_u64, 2, 10, 1000, 1 << 20, 1 << 33, u64::MAX] {
                let keys64: Vec<u64> = (0..len)
                    .map(|_| {
                        let key = splitmix(&mut state) % limit;
                        if limit == u64::MAX {
                            key | (u64::MAX - 5)
                        } else {
                            key
                        }
                    })
                    .collect();
                for gap in [None, Some((3_u64, 7_u64))] {
                    assert_eq!(
                        ranges(&keys64, |key| key, gap),
                        cpu_ranges(&keys64, gap),
                        "u64, len {len}, limit {limit}, gap {gap:?}"
                    );
                }
                // The low 32 bits make the u32 keys.
                let keys32: Vec<u32> = keys64
                    .iter()
                    .map(|&key| {
                        u32::try_from(key & u64::from(u32::MAX)).expect("masked to 32 bits")
                    })
                    .collect();
                for gap in [None, Some((3_u32, 7_u32))] {
                    assert_eq!(
                        ranges(&keys32, |key| key, gap),
                        cpu_ranges(&keys32, gap),
                        "u32, len {len}, limit {limit}, gap {gap:?}"
                    );
                }
            }
        }
    }
}
