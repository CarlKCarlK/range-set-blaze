//! [`Engine::CudaOnly`]: the GPU pipeline as CUDA kernels written in Rust with cuda-oxide.
//!
//! The kernels live in `cuda-kernels/` and are compiled ahead of time to `cuda_kernels.ptx`, so
//! building this crate needs only `cuda-core`, not the cuda-oxide compiler. Each call is one-shot:
//! only the CUDA context is kept between calls; the module (JIT-compiled from the PTX by the
//! driver) and every buffer are created per call, as the wgpu engine creates its pipelines and
//! buffers per call.

use core::ffi::c_void;
use std::sync::{Arc, OnceLock};
use std::vec::Vec;

use cuda_core::simt::launch_kernel_on_stream;
use cuda_core::simt::memory::memcpy_htod_async;
use cuda_core::sys::cudaError_enum_CUDA_ERROR_OUT_OF_MEMORY;
use cuda_core::{
    CudaContext, CudaFunction, CudaModule, CudaStream, DeviceBuffer, DriverError, PinnedHostBuffer,
};

use super::{high_word, join_words, low_word};

#[cfg(doc)]
use super::Engine;

const PTX: &str = include_str!("cuda_kernels.ptx");
/// Threads per block and items per block in the kernels; they must match `cuda-kernels/`.
const BLOCK: u32 = 256;
const TILE: u32 = 4096;
const RADIX: u32 = 256;
/// Keys per upload chunk: each is encoded into pinned memory while the one before it copies.
const CHUNK: usize = 1 << 20;

pub(super) fn cuda_context() -> Option<&'static Arc<CudaContext>> {
    static CONTEXT: OnceLock<Option<Arc<CudaContext>>> = OnceLock::new();
    CONTEXT.get_or_init(|| CudaContext::new(0).ok()).as_ref()
}

/// The ranges of `values` encoded as `u32` keys, as `(start, end)` key pairs, or `None` if the GPU
/// ran out of memory. `gap` names two keys that count as adjacent (as for `char`, across the
/// surrogates).
///
/// # Panics
///
/// Panics on any other CUDA error.
pub(super) fn ranges_32<T: Copy>(
    context: &Arc<CudaContext>,
    values: &[T],
    encode: impl Fn(T) -> u32,
    gap: Option<(u32, u32)>,
) -> Option<Vec<(u32, u32)>> {
    finish(try_ranges_32(context, values, encode, gap))
}

/// As [`ranges_32`], for values encoded as `u64` keys.
pub(super) fn ranges_64<T: Copy>(
    context: &Arc<CudaContext>,
    values: &[T],
    encode: impl Fn(T) -> u64,
    gap: Option<(u64, u64)>,
) -> Option<Vec<(u64, u64)>> {
    finish(try_ranges_64(context, values, encode, gap))
}

/// A map's ranges from its `u32` key words and its values' one or two `u32` words: each range's
/// `(start, end)` key words, and the input position of its first entry (whose value is the
/// range's). Where a key repeats, its last entry wins. `None` if the GPU ran out of memory.
///
/// # Panics
///
/// Panics on any other CUDA error.
pub(super) fn map_ranges(
    context: &Arc<CudaContext>,
    keys: &[u32],
    gap: Option<(u32, u32)>,
    value_lo: &[u32],
    value_hi: Option<&[u32]>,
) -> Option<MapRanges> {
    finish(try_map_ranges(context, keys, gap, value_lo, value_hi))
}

/// A map's ranges as `(start, end)` key words, and each range's first input position.
type MapRanges = (Vec<(u32, u32)>, Vec<u32>);

fn finish<R>(result: Result<R, DriverError>) -> Option<R> {
    match result {
        Ok(ranges) => Some(ranges),
        Err(error) if error.0 == cudaError_enum_CUDA_ERROR_OUT_OF_MEMORY => None,
        Err(error) => panic!("CUDA range execution failed: {error:?}"),
    }
}

fn try_ranges_32<T: Copy>(
    context: &Arc<CudaContext>,
    values: &[T],
    encode: impl Fn(T) -> u32,
    gap: Option<(u32, u32)>,
) -> Result<Vec<(u32, u32)>, DriverError> {
    let Some(len) = length(values.len()) else {
        return Ok(Vec::new());
    };
    let kernels = Kernels::load(context)?;
    let stream = context.default_stream();
    let mut sorted = scratch(&stream, values.len())?;
    upload(context, &stream, values, encode, &sorted)?;
    let mut spare = scratch(&stream, values.len())?;
    kernels.radix_sort(&stream, len, &mut sorted, &mut spare, None)?;

    // Ranges: mark each range's first key, scan the marks, then emit each range's ends.
    let (gap_left, gap_right) = gap.unwrap_or((0, 0));
    let has_gap = u64::from(gap.is_some());
    let marks = spare;
    let blocks = len.div_ceil(BLOCK);
    launch(
        &stream,
        &kernels.mark_starts,
        blocks,
        &[
            pointer(&sorted),
            u64::from(len),
            u64::from(gap_left),
            u64::from(gap_right),
            has_gap,
            pointer(&marks),
        ],
    )?;
    let total = kernels.exclusive_scan(&stream, pointer(&marks), len)?;
    let count = read_count(&stream, &total)?;
    let starts = scratch(&stream, count)?;
    let ends = scratch(&stream, count)?;
    launch(
        &stream,
        &kernels.emit_ranges,
        blocks,
        &[
            pointer(&sorted),
            u64::from(len),
            pointer(&marks),
            u64::from(len),
            u64::from(gap_left),
            u64::from(gap_right),
            has_gap,
            pointer(&starts),
            pointer(&ends),
        ],
    )?;
    let starts_host = starts.to_host_vec(&stream)?;
    let ends_host = ends.to_host_vec(&stream)?;
    release(&stream, [sorted, marks, total, starts, ends])?;
    Ok(starts_host.into_iter().zip(ends_host).collect())
}

fn try_ranges_64<T: Copy>(
    context: &Arc<CudaContext>,
    values: &[T],
    encode: impl Fn(T) -> u64,
    gap: Option<(u64, u64)>,
) -> Result<Vec<(u64, u64)>, DriverError> {
    let Some(len) = length(values.len()) else {
        return Ok(Vec::new());
    };
    let kernels = Kernels::load(context)?;
    let stream = context.default_stream();
    let mut lo = scratch(&stream, values.len())?;
    upload(
        context,
        &stream,
        values,
        |value| low_word(encode(value)),
        &lo,
    )?;
    let mut hi = scratch(&stream, values.len())?;
    upload(
        context,
        &stream,
        values,
        |value| high_word(encode(value)),
        &hi,
    )?;

    // Sort by the low word carrying the high word, then stably by the high word carrying the
    // low word: the keys end up in `(hi, lo)` order.
    let mut lo_spare = scratch(&stream, values.len())?;
    let mut hi_spare = scratch(&stream, values.len())?;
    kernels.radix_sort(
        &stream,
        len,
        &mut lo,
        &mut lo_spare,
        Some((&mut hi, &mut hi_spare)),
    )?;
    kernels.radix_sort(
        &stream,
        len,
        &mut hi,
        &mut hi_spare,
        Some((&mut lo, &mut lo_spare)),
    )?;

    let (gap_left, gap_right) = gap.unwrap_or((0, 0));
    let gap_words = [
        u64::from(high_word(gap_left)),
        u64::from(low_word(gap_left)),
        u64::from(high_word(gap_right)),
        u64::from(low_word(gap_right)),
        u64::from(gap.is_some()),
    ];
    let marks = lo_spare;
    let blocks = len.div_ceil(BLOCK);
    let mut arguments = std::vec![pointer(&hi), u64::from(len), pointer(&lo), u64::from(len)];
    arguments.extend(gap_words);
    arguments.push(pointer(&marks));
    launch(&stream, &kernels.mark_starts_64, blocks, &arguments)?;
    let total = kernels.exclusive_scan(&stream, pointer(&marks), len)?;
    let count = read_count(&stream, &total)?;
    let ends = [
        scratch(&stream, count)?,
        scratch(&stream, count)?,
        scratch(&stream, count)?,
        scratch(&stream, count)?,
    ];
    let mut arguments = std::vec![
        pointer(&hi),
        u64::from(len),
        pointer(&lo),
        u64::from(len),
        pointer(&marks),
        u64::from(len),
    ];
    arguments.extend(gap_words);
    arguments.extend(ends.iter().map(pointer));
    launch(&stream, &kernels.emit_ranges_64, blocks, &arguments)?;
    let [starts_hi, starts_lo, ends_hi, ends_lo] =
        ends.each_ref().map(|buffer| buffer.to_host_vec(&stream));
    let (starts_hi, starts_lo, ends_hi, ends_lo) = (starts_hi?, starts_lo?, ends_hi?, ends_lo?);
    let [a, b, c, d] = ends;
    release(&stream, [hi, lo, hi_spare, marks, total, a, b, c, d])?;
    Ok((0..count)
        .map(|range| {
            (
                join_words(starts_hi[range], starts_lo[range]),
                join_words(ends_hi[range], ends_lo[range]),
            )
        })
        .collect())
}

fn try_map_ranges(
    context: &Arc<CudaContext>,
    keys: &[u32],
    gap: Option<(u32, u32)>,
    value_lo: &[u32],
    value_hi: Option<&[u32]>,
) -> Result<MapRanges, DriverError> {
    let Some(len) = length(keys.len()) else {
        return Ok((Vec::new(), Vec::new()));
    };
    let kernels = Kernels::load(context)?;
    let stream = context.default_stream();
    let Survivors {
        keys: survivor_keys,
        positions: survivor_positions,
        marks,
        count: survivors,
        spent,
    } = survivors(context, &kernels, &stream, keys, len)?;

    // Ranges among the survivors: keys adjacent and values equal.
    let value_lo_device = scratch(&stream, value_lo.len())?;
    upload(context, &stream, value_lo, |word| word, &value_lo_device)?;
    let value_hi_device = value_hi
        .map(|value_hi| {
            let device = scratch(&stream, value_hi.len())?;
            upload(context, &stream, value_hi, |word| word, &device)?;
            Ok(device)
        })
        .transpose()?;
    let (hi_pointer, hi_len) = value_hi_device.as_ref().map_or_else(
        || (pointer(&value_lo_device), 0),
        |device| (pointer(device), u64::from(len)),
    );
    let (gap_left, gap_right) = gap.unwrap_or((0, 0));
    let values_and_gap = [
        pointer(&value_lo_device),
        u64::from(len),
        hi_pointer,
        hi_len,
    ];
    let gap_words = [
        u64::from(gap_left),
        u64::from(gap_right),
        u64::from(gap.is_some()),
    ];
    let survivor_blocks = survivors.div_ceil(BLOCK);
    let mut arguments = std::vec![
        pointer(&survivor_keys),
        u64::from(survivors),
        pointer(&survivor_positions),
        u64::from(survivors),
    ];
    arguments.extend(values_and_gap);
    arguments.extend(gap_words);
    arguments.push(pointer(&marks));
    launch(
        &stream,
        &kernels.mark_map_starts,
        survivor_blocks,
        &arguments,
    )?;
    let total = kernels.exclusive_scan(&stream, pointer(&marks), survivors)?;
    let count = read_count(&stream, &total)?;
    let outputs = [
        scratch(&stream, count)?,
        scratch(&stream, count)?,
        scratch(&stream, count)?,
    ];
    let mut arguments = std::vec![
        pointer(&survivor_keys),
        u64::from(survivors),
        pointer(&survivor_positions),
        u64::from(survivors),
    ];
    arguments.extend(values_and_gap);
    arguments.extend([pointer(&marks), u64::from(survivors)]);
    arguments.extend(gap_words);
    arguments.extend(outputs.iter().map(pointer));
    launch(
        &stream,
        &kernels.emit_map_ranges,
        survivor_blocks,
        &arguments,
    )?;
    let [starts, ends, firsts] = outputs.each_ref().map(|buffer| buffer.to_host_vec(&stream));
    let (starts, ends, firsts) = (starts?, ends?, firsts?);
    let [a, b, c] = outputs;
    release(
        &stream,
        spent
            .into_iter()
            .chain([
                survivor_keys,
                survivor_positions,
                marks,
                value_lo_device,
                total,
                a,
                b,
                c,
            ])
            .chain(value_hi_device),
    )?;
    Ok((starts.into_iter().zip(ends).collect(), firsts))
}

/// A map's surviving entries (the last of each key), sorted by key, on the device.
struct Survivors {
    keys: DeviceBuffer<u32>,
    positions: DeviceBuffer<u32>,
    /// Scratch, one word per input entry.
    marks: DeviceBuffer<u32>,
    count: u32,
    /// Buffers to [`release`] at the end.
    spent: [DeviceBuffer<u32>; 3],
}

/// Sorts the keys carrying their input positions, then keeps the last entry of each key: the
/// sort is stable, so that entry has the key's last value.
fn survivors(
    context: &Arc<CudaContext>,
    kernels: &Kernels,
    stream: &Arc<CudaStream>,
    keys: &[u32],
    len: u32,
) -> Result<Survivors, DriverError> {
    let blocks = len.div_ceil(BLOCK);
    let mut sorted = scratch(stream, keys.len())?;
    upload(context, stream, keys, |key| key, &sorted)?;
    let mut positions = scratch(stream, keys.len())?;
    launch(
        stream,
        &kernels.fill_positions,
        blocks,
        &[pointer(&positions), u64::from(len)],
    )?;
    let mut sorted_spare = scratch(stream, keys.len())?;
    let mut positions_spare = scratch(stream, keys.len())?;
    kernels.radix_sort(
        stream,
        len,
        &mut sorted,
        &mut sorted_spare,
        Some((&mut positions, &mut positions_spare)),
    )?;

    let marks = scratch(stream, keys.len())?;
    launch(
        stream,
        &kernels.mark_survivors,
        blocks,
        &[pointer(&sorted), u64::from(len), pointer(&marks)],
    )?;
    let total = kernels.exclusive_scan(stream, pointer(&marks), len)?;
    let count =
        u32::try_from(read_count(stream, &total)?).expect("survivors are at most the input");
    launch(
        stream,
        &kernels.compact_survivors,
        blocks,
        &[
            pointer(&sorted),
            u64::from(len),
            pointer(&positions),
            u64::from(len),
            pointer(&marks),
            u64::from(len),
            pointer(&sorted_spare),
            pointer(&positions_spare),
        ],
    )?;
    Ok(Survivors {
        keys: sorted_spare,
        positions: positions_spare,
        marks,
        count,
        spent: [sorted, positions, total],
    })
}

/// The input length as the kernels' `u32`, or `None` if the input is empty.
fn length(len: usize) -> Option<u32> {
    (len != 0).then(|| u32::try_from(len).expect("CUDA input exceeds u32 positions"))
}

/// Reads a scan's one-word total.
fn read_count(stream: &CudaStream, total: &DeviceBuffer<u32>) -> Result<usize, DriverError> {
    Ok(total.to_host_vec(stream)?[0] as usize)
}

/// Encodes `values` into `keys` on the device, a chunk at a time through two pinned host
/// buffers, so encoding one chunk overlaps copying the one before it.
fn upload<T: Copy>(
    context: &Arc<CudaContext>,
    stream: &CudaStream,
    values: &[T],
    encode: impl Fn(T) -> u32,
    keys: &DeviceBuffer<u32>,
) -> Result<(), DriverError> {
    let chunk = CHUNK.min(values.len());
    let mut staging = [
        PinnedHostBuffer::<u32>::zeroed(context, chunk)?,
        PinnedHostBuffer::<u32>::zeroed(context, chunk)?,
    ];
    for (index, values) in values.chunks(chunk).enumerate() {
        let buffer = &mut staging[index % 2];
        for (key, &value) in buffer.iter_mut().zip(values) {
            *key = encode(value);
        }
        // The copy issued two chunks ago read this buffer; wait for it (and the one after it).
        stream.synchronize()?;
        let offset = (index * chunk * size_of::<u32>()) as u64;
        // SAFETY: the destination range lies inside `keys`, the source is pinned and stays
        // untouched until the next synchronize, and both outlive the copy (we synchronize below).
        unsafe {
            memcpy_htod_async(
                keys.cu_deviceptr() + offset,
                buffer.as_ptr(),
                values.len() * size_of::<u32>(),
                stream.cu_stream(),
            )?;
        }
    }
    stream.synchronize()
}

/// Launches `function` on `blocks` blocks of [`BLOCK`] threads. Each argument is passed
/// from the low bytes of a `u64`, which suits the kernels' `u32` and 64-bit parameters
/// (a slice is two: its pointer and its length).
fn launch(
    stream: &CudaStream,
    function: &CudaFunction,
    blocks: u32,
    arguments: &[u64],
) -> Result<(), DriverError> {
    let mut storage = arguments.to_vec();
    let mut parameters: Vec<*mut c_void> = storage
        .iter_mut()
        .map(|argument| core::ptr::from_mut(argument).cast())
        .collect();
    // SAFETY: the arguments match the kernel's PTX parameters in order and kind (see
    // `cuda-kernels/`), every pointer is a live device buffer of the length passed with it,
    // and each kernel bounds-checks its own grid against those lengths.
    unsafe {
        launch_kernel_on_stream(
            function,
            (blocks, 1, 1),
            (BLOCK, 1, 1),
            0,
            stream,
            &mut parameters,
        )
    }
}

fn pointer(buffer: &DeviceBuffer<u32>) -> u64 {
    buffer.cu_deviceptr()
}

/// A stream-ordered (`cuMemAllocAsync`) buffer that a kernel or copy fills before anything reads
/// it. Plain `cuMemFree` of large buffers is slow (about 95 ms per 100M-key call under WSL).
/// Release it with [`release`].
fn scratch(stream: &Arc<CudaStream>, len: usize) -> Result<DeviceBuffer<u32>, DriverError> {
    // SAFETY: every caller writes the whole buffer (by copy or kernel) before reading it.
    unsafe { DeviceBuffer::uninitialized_async(stream, len) }
}

/// Frees `buffers` in stream order, then waits once, so the memory is back with the driver when
/// the call returns. (Dropping each buffer instead synchronizes the context every time.)
fn release(
    stream: &CudaStream,
    buffers: impl IntoIterator<Item = DeviceBuffer<u32>>,
) -> Result<(), DriverError> {
    for buffer in buffers {
        // SAFETY: all of this module's work runs on `stream`, so nothing else can still be
        // using the buffer when the free runs.
        unsafe { buffer.drop_async(stream)? };
    }
    stream.synchronize()
}

struct Kernels {
    // Keeps the functions' module loaded.
    _module: Arc<CudaModule>,
    radix_histogram: CudaFunction,
    radix_scatter: CudaFunction,
    digit_counts: CudaFunction,
    scan_tiles: CudaFunction,
    add_tile_offsets: CudaFunction,
    mark_starts: CudaFunction,
    emit_ranges: CudaFunction,
    mark_starts_64: CudaFunction,
    emit_ranges_64: CudaFunction,
    fill_positions: CudaFunction,
    mark_survivors: CudaFunction,
    compact_survivors: CudaFunction,
    mark_map_starts: CudaFunction,
    emit_map_ranges: CudaFunction,
}

impl Kernels {
    fn load(context: &Arc<CudaContext>) -> Result<Self, DriverError> {
        let module = context.load_module_from_ptx_src(PTX)?;
        Ok(Self {
            radix_histogram: module.load_function("radix_histogram")?,
            radix_scatter: module.load_function("radix_scatter")?,
            digit_counts: module.load_function("digit_counts")?,
            scan_tiles: module.load_function("scan_tiles")?,
            add_tile_offsets: module.load_function("add_tile_offsets")?,
            mark_starts: module.load_function("mark_starts")?,
            emit_ranges: module.load_function("emit_ranges")?,
            mark_starts_64: module.load_function("mark_starts_64")?,
            emit_ranges_64: module.load_function("emit_ranges_64")?,
            fill_positions: module.load_function("fill_positions")?,
            mark_survivors: module.load_function("mark_survivors")?,
            compact_survivors: module.load_function("compact_survivors")?,
            mark_map_starts: module.load_function("mark_map_starts")?,
            emit_map_ranges: module.load_function("emit_map_ranges")?,
            _module: module,
        })
    }

    /// Stable LSD radix sort of `len` keys: up to four 8-bit passes between `keys` and
    /// `keys_spare`, carrying `values` (and their spare) along if given. The result ends up back
    /// in `keys` (and `values`). A pass where every key has the same digit would change nothing,
    /// so it is skipped.
    fn radix_sort(
        &self,
        stream: &Arc<CudaStream>,
        len: u32,
        keys: &mut DeviceBuffer<u32>,
        keys_spare: &mut DeviceBuffer<u32>,
        mut values: Option<(&mut DeviceBuffer<u32>, &mut DeviceBuffer<u32>)>,
    ) -> Result<(), DriverError> {
        let tiles = len.div_ceil(TILE);
        let mut digit_counts = scratch(stream, (4 * RADIX) as usize)?;
        digit_counts.zero_async(stream)?;
        launch(
            stream,
            &self.digit_counts,
            tiles,
            &[pointer(keys), u64::from(len), pointer(&digit_counts)],
        )?;
        let digit_counts_host = digit_counts.to_host_vec(stream)?;
        // SAFETY: as in `release`, everything runs on `stream`.
        unsafe { digit_counts.drop_async(stream)? };
        let counts = scratch(stream, (RADIX * tiles) as usize)?;
        for (pass, shift) in [0_u32, 8, 16, 24].into_iter().enumerate() {
            let pass_counts = &digit_counts_host[pass * RADIX as usize..][..RADIX as usize];
            if pass_counts.contains(&len) {
                continue;
            }
            launch(
                stream,
                &self.radix_histogram,
                tiles,
                &[
                    pointer(keys),
                    u64::from(len),
                    u64::from(shift),
                    u64::from(tiles),
                    pointer(&counts),
                ],
            )?;
            let total = self.exclusive_scan(stream, pointer(&counts), RADIX * tiles)?;
            // SAFETY: as in `release`, everything runs on `stream`.
            unsafe { total.drop_async(stream)? };
            // No values: an empty slice, and an output nothing writes.
            let (values_in, values_len, values_out) = values.as_ref().map_or_else(
                || (pointer(keys), 0, 0),
                |(values, values_spare)| (pointer(values), u64::from(len), pointer(values_spare)),
            );
            launch(
                stream,
                &self.radix_scatter,
                tiles,
                &[
                    pointer(keys),
                    u64::from(len),
                    values_in,
                    values_len,
                    u64::from(shift),
                    u64::from(tiles),
                    pointer(&counts),
                    u64::from(RADIX * tiles),
                    pointer(keys_spare),
                    values_out,
                ],
            )?;
            core::mem::swap(keys, keys_spare);
            if let Some((values, values_spare)) = &mut values {
                core::mem::swap(*values, *values_spare);
            }
        }
        // SAFETY: as in `release`, everything runs on `stream`.
        unsafe { counts.drop_async(stream) }
    }

    /// Exclusive scan of the `len` words at `data`, in place. Returns a one-word buffer
    /// holding their total.
    fn exclusive_scan(
        &self,
        stream: &Arc<CudaStream>,
        data: u64,
        len: u32,
    ) -> Result<DeviceBuffer<u32>, DriverError> {
        let tiles = len.div_ceil(TILE);
        let tile_sums = scratch(stream, tiles as usize)?;
        launch(
            stream,
            &self.scan_tiles,
            tiles,
            &[data, u64::from(len), pointer(&tile_sums)],
        )?;
        if tiles == 1 {
            return Ok(tile_sums);
        }
        let total = self.exclusive_scan(stream, pointer(&tile_sums), tiles)?;
        launch(
            stream,
            &self.add_tile_offsets,
            tiles,
            &[data, u64::from(len), pointer(&tile_sums), u64::from(tiles)],
        )?;
        // SAFETY: as in `release`, everything runs on `stream`.
        unsafe { tile_sums.drop_async(stream)? };
        Ok(total)
    }
}

#[cfg(test)]
mod tests {
    use std::vec::Vec;

    use crate::{Engine, RangeMapBlaze, RangeSetBlaze};

    macro_rules! check {
        ($values:expr) => {{
            let values = $values;
            let expected = RangeSetBlaze::from_iter(values.iter().copied());
            let found = super::super::from_shuffled_slice(&mut values.to_vec(), Engine::CudaOnly);
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
        let found = RangeMapBlaze::from_shuffled_slices(keys, values, Engine::CudaOnly);
        assert_eq!(found, expected, "len {}", keys.len());
    }

    #[test]
    #[should_panic(expected = "no GPU representation")]
    fn unsupported_keys_panic() {
        let _ = RangeSetBlaze::<u128>::from_shuffled_slice(&mut [1, 2], Engine::CudaOnly);
    }

    #[test]
    #[should_panic(expected = "no GPU representation")]
    fn unsupported_map_values_panic() {
        let _ = RangeMapBlaze::from_shuffled_slices(&[1_u32], &["a"], Engine::CudaOnly);
    }
}
