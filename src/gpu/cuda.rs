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
use cuda_core::simt::memory::{memcpy_dtoh_async, memcpy_htod_async, memset_d8_async};
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
const CHUNK: usize = 1 << 18;
/// Inputs up to this many keys are encoded into ordinary memory and copied at once: allocating
/// pinned memory costs more than it saves (about 0.6 ms per MB, plus freeing it), while beyond
/// about 4 MB a fresh ordinary buffer pays more in page faults.
const PAGEABLE_UPLOAD: usize = 1 << 20;
/// Each buffer carved from an [`Arena`] starts on a multiple of this many words (256 bytes).
const ALIGN_WORDS: usize = 64;

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
    let n = values.len();
    let kernels = Kernels::load(context)?;
    let stream = context.default_stream();
    let mut arena = Arena::new(
        &stream,
        2 * aligned(n) + sort_scratch(len) + scan_scratch(len),
    )?;
    let mut sorted = arena.take(n);
    let mut spare = arena.take(n);
    let sort_space = arena.take(sort_scratch(len));
    let scan_space = arena.take(scan_scratch(len));
    upload(context, &stream, values, encode, sorted)?;
    kernels.radix_sort(&stream, &mut sorted, &mut spare, None, sort_space)?;

    // Ranges: mark each range's first key, scan the marks, then emit each range's ends.
    let (gap_left, gap_right) = gap.unwrap_or((0, 0));
    let gap_words = [
        u64::from(gap_left),
        u64::from(gap_right),
        u64::from(gap.is_some()),
    ];
    let marks = spare;
    let blocks = len.div_ceil(BLOCK);
    let mut arguments = sorted.argument().to_vec();
    arguments.extend(gap_words);
    arguments.push(marks.pointer);
    launch(&stream, &kernels.mark_starts, blocks, &arguments)?;
    let count = read_count(&stream, kernels.exclusive_scan(&stream, marks, scan_space)?)?;
    let mut outputs = Arena::new(&stream, 2 * aligned(count))?;
    let (starts, ends) = (outputs.take(count), outputs.take(count));
    let mut arguments = [sorted.argument(), marks.argument()].concat();
    arguments.extend(gap_words);
    arguments.extend([starts.pointer, ends.pointer]);
    launch(&stream, &kernels.emit_ranges, blocks, &arguments)?;
    let host = read(&stream, outputs.used())?;
    let (starts, ends) = (&host[..count], &host[aligned(count)..][..count]);
    release(&stream, [arena, outputs])?;
    Ok(starts.iter().copied().zip(ends.iter().copied()).collect())
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
    let n = values.len();
    let kernels = Kernels::load(context)?;
    let stream = context.default_stream();
    let mut arena = Arena::new(
        &stream,
        4 * aligned(n) + sort_scratch(len) + scan_scratch(len),
    )?;
    let (mut lo, mut hi) = (arena.take(n), arena.take(n));
    let (mut lo_spare, mut hi_spare) = (arena.take(n), arena.take(n));
    let sort_space = arena.take(sort_scratch(len));
    let scan_space = arena.take(scan_scratch(len));
    upload(
        context,
        &stream,
        values,
        |value| low_word(encode(value)),
        lo,
    )?;
    upload(
        context,
        &stream,
        values,
        |value| high_word(encode(value)),
        hi,
    )?;

    // Sort by the low word carrying the high word, then stably by the high word carrying the
    // low word: the keys end up in `(hi, lo)` order.
    kernels.radix_sort(
        &stream,
        &mut lo,
        &mut lo_spare,
        Some((&mut hi, &mut hi_spare)),
        sort_space,
    )?;
    kernels.radix_sort(
        &stream,
        &mut hi,
        &mut hi_spare,
        Some((&mut lo, &mut lo_spare)),
        sort_space,
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
    let mut arguments = [hi.argument(), lo.argument()].concat();
    arguments.extend(gap_words);
    arguments.push(marks.pointer);
    launch(&stream, &kernels.mark_starts_64, blocks, &arguments)?;
    let count = read_count(&stream, kernels.exclusive_scan(&stream, marks, scan_space)?)?;
    let mut outputs = Arena::new(&stream, 4 * aligned(count))?;
    let ends = [(); 4].map(|()| outputs.take(count));
    let mut arguments = [hi.argument(), lo.argument(), marks.argument()].concat();
    arguments.extend(gap_words);
    arguments.extend(ends.iter().map(|words| words.pointer));
    launch(&stream, &kernels.emit_ranges_64, blocks, &arguments)?;
    let host = read(&stream, outputs.used())?;
    let column = |index: usize| &host[index * aligned(count)..][..count];
    let (starts_hi, starts_lo, ends_hi, ends_lo) = (column(0), column(1), column(2), column(3));
    let ranges = (0..count)
        .map(|range| {
            (
                join_words(starts_hi[range], starts_lo[range]),
                join_words(ends_hi[range], ends_lo[range]),
            )
        })
        .collect();
    release(&stream, [arena, outputs])?;
    Ok(ranges)
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
    let n = keys.len();
    let kernels = Kernels::load(context)?;
    let stream = context.default_stream();
    let value_columns = if value_hi.is_some() { 2 } else { 1 };
    let mut arena = Arena::new(
        &stream,
        (5 + value_columns) * aligned(n) + sort_scratch(len) + scan_scratch(len),
    )?;
    let survivors = survivors(context, &kernels, &stream, &mut arena, keys)?;

    // Ranges among the survivors: keys adjacent and values equal.
    let lo_words = arena.take(n);
    upload(context, &stream, value_lo, |word| word, lo_words)?;
    let hi_words = match value_hi {
        Some(value_hi) => {
            let words = arena.take(n);
            upload(context, &stream, value_hi, |word| word, words)?;
            words
        }
        // An empty slice: the kernels then compare one word.
        None => Words {
            pointer: lo_words.pointer,
            len: 0,
        },
    };
    let (gap_left, gap_right) = gap.unwrap_or((0, 0));
    let gap_words = [
        u64::from(gap_left),
        u64::from(gap_right),
        u64::from(gap.is_some()),
    ];
    let Survivors {
        keys: survivor_keys,
        positions: survivor_positions,
        marks,
        scan_space,
    } = survivors;
    let survivor_count = u32::try_from(survivor_keys.len).expect("survivors fit in u32");
    let survivor_blocks = survivor_count.div_ceil(BLOCK);
    let entries = [
        survivor_keys.argument(),
        survivor_positions.argument(),
        lo_words.argument(),
        hi_words.argument(),
    ]
    .concat();
    let mut arguments = entries.clone();
    arguments.extend(gap_words);
    arguments.push(marks.pointer);
    launch(
        &stream,
        &kernels.mark_map_starts,
        survivor_blocks,
        &arguments,
    )?;
    let count = read_count(&stream, kernels.exclusive_scan(&stream, marks, scan_space)?)?;
    let mut outputs = Arena::new(&stream, 3 * aligned(count))?;
    let columns = [(); 3].map(|()| outputs.take(count));
    let mut arguments = entries;
    arguments.extend(marks.argument());
    arguments.extend(gap_words);
    arguments.extend(columns.iter().map(|words| words.pointer));
    launch(
        &stream,
        &kernels.emit_map_ranges,
        survivor_blocks,
        &arguments,
    )?;
    let host = read(&stream, outputs.used())?;
    let column = |index: usize| &host[index * aligned(count)..][..count];
    let ranges = column(0)
        .iter()
        .copied()
        .zip(column(1).iter().copied())
        .collect();
    let firsts = column(2).to_vec();
    release(&stream, [arena, outputs])?;
    Ok((ranges, firsts))
}

/// A map's surviving entries (the last of each key), sorted by key, on the device.
struct Survivors {
    keys: Words,
    positions: Words,
    /// Scratch with room for one word per survivor, and for scanning them.
    marks: Words,
    scan_space: Words,
}

/// Sorts the keys carrying their input positions, then keeps the last entry of each key: the
/// sort is stable, so that entry has the key's last value. Takes five columns, the sort's
/// scratch, and the scan's scratch from `arena`.
fn survivors(
    context: &Arc<CudaContext>,
    kernels: &Kernels,
    stream: &Arc<CudaStream>,
    arena: &mut Arena,
    keys: &[u32],
) -> Result<Survivors, DriverError> {
    let n = keys.len();
    let len = u32::try_from(n).expect("CUDA input exceeds u32 positions");
    let blocks = len.div_ceil(BLOCK);
    let (mut sorted, mut positions) = (arena.take(n), arena.take(n));
    let (mut sorted_spare, mut positions_spare) = (arena.take(n), arena.take(n));
    let marks = arena.take(n);
    let sort_space = arena.take(sort_scratch(len));
    let scan_space = arena.take(scan_scratch(len));
    upload(context, stream, keys, |key| key, sorted)?;
    launch(
        stream,
        &kernels.fill_positions,
        blocks,
        &[positions.pointer, u64::from(len)],
    )?;
    kernels.radix_sort(
        stream,
        &mut sorted,
        &mut sorted_spare,
        Some((&mut positions, &mut positions_spare)),
        sort_space,
    )?;

    launch(
        stream,
        &kernels.mark_survivors,
        blocks,
        &[sorted.pointer, u64::from(len), marks.pointer],
    )?;
    let count = read_count(stream, kernels.exclusive_scan(stream, marks, scan_space)?)?;
    let mut arguments = [sorted.argument(), positions.argument(), marks.argument()].concat();
    arguments.extend([sorted_spare.pointer, positions_spare.pointer]);
    launch(stream, &kernels.compact_survivors, blocks, &arguments)?;
    Ok(Survivors {
        keys: sorted_spare.prefix(count),
        positions: positions_spare.prefix(count),
        marks: marks.prefix(count),
        scan_space,
    })
}

/// The input length as the kernels' `u32`, or `None` if the input is empty.
fn length(len: usize) -> Option<u32> {
    (len != 0).then(|| u32::try_from(len).expect("CUDA input exceeds u32 positions"))
}

/// Encodes `values` into `keys` on the device. Large inputs go a chunk at a time through two
/// pinned host buffers, so encoding one chunk overlaps copying the one before it. (Encoding all
/// of a large input into fresh memory first is slow: its page faults cost more than the copy.)
fn upload<T: Copy>(
    context: &Arc<CudaContext>,
    stream: &CudaStream,
    values: &[T],
    encode: impl Fn(T) -> u32,
    keys: Words,
) -> Result<(), DriverError> {
    debug_assert_eq!(
        values.len(),
        keys.len,
        "upload into a column of another length"
    );
    if values.len() <= PAGEABLE_UPLOAD {
        let encoded: Vec<u32> = values.iter().map(|&value| encode(value)).collect();
        // SAFETY: the destination is `keys`, and the copy finishes before `encoded` is dropped
        // (we synchronize below).
        unsafe {
            memcpy_htod_async(
                keys.pointer,
                encoded.as_ptr(),
                encoded.len() * size_of::<u32>(),
                stream.cu_stream(),
            )?;
        }
        return stream.synchronize();
    }
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
                keys.pointer + offset,
                buffer.as_ptr(),
                values.len() * size_of::<u32>(),
                stream.cu_stream(),
            )?;
        }
    }
    stream.synchronize()
}

/// Copies `words` from the device.
fn read(stream: &CudaStream, words: Words) -> Result<Vec<u32>, DriverError> {
    let mut host = Vec::with_capacity(words.len);
    // SAFETY: `host` has room for `words.len` words, which the copy fills before `set_len`
    // (we synchronize first).
    unsafe {
        memcpy_dtoh_async(
            host.as_mut_ptr(),
            words.pointer,
            words.len * size_of::<u32>(),
            stream.cu_stream(),
        )?;
        stream.synchronize()?;
        host.set_len(words.len);
    }
    Ok(host)
}

/// Reads a scan's one-word total.
fn read_count(stream: &CudaStream, total: Words) -> Result<usize, DriverError> {
    Ok(read(stream, total)?[0] as usize)
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
    // `cuda-kernels/`), every pointer is live device memory of the length passed with it,
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

/// `len` rounded up to whole [`ALIGN_WORDS`].
const fn aligned(len: usize) -> usize {
    len.next_multiple_of(ALIGN_WORDS)
}

/// A view of `len` words of device memory, inside an [`Arena`].
#[derive(Clone, Copy)]
struct Words {
    pointer: u64,
    len: usize,
}

impl Words {
    /// The kernel arguments for this view as a slice: its pointer and its length.
    fn argument(self) -> [u64; 2] {
        [self.pointer, self.len as u64]
    }

    /// The first `len` words.
    fn prefix(self, len: usize) -> Self {
        assert!(len <= self.len, "prefix past the end of a device view");
        Self {
            pointer: self.pointer,
            len,
        }
    }

    /// The first `len` words, and the rest after rounding `len` up to [`ALIGN_WORDS`].
    fn split(self, len: usize) -> (Self, Self) {
        let taken = aligned(len);
        assert!(taken <= self.len, "device scratch is smaller than planned");
        (
            self.prefix(len),
            Self {
                pointer: self.pointer + (taken * size_of::<u32>()) as u64,
                len: self.len - taken,
            },
        )
    }
}

/// One stream-ordered (`cuMemAllocAsync`) allocation, carved into [`Words`] views, so a call
/// allocates once per phase rather than once per buffer. Every view is written (by copy or
/// kernel) before it is read. Plain `cuMemFree` of large buffers is slow (about 95 ms per
/// 100M-key call under WSL), so free arenas with [`release`].
struct Arena {
    buffer: DeviceBuffer<u32>,
    rest: Words,
}

impl Arena {
    /// An arena of `len` words; [`Arena::take`] rounds each view up to [`ALIGN_WORDS`].
    fn new(stream: &Arc<CudaStream>, len: usize) -> Result<Self, DriverError> {
        // SAFETY: every view is written before it is read (see above).
        let buffer = unsafe { DeviceBuffer::uninitialized_async(stream, len.max(1))? };
        let rest = Words {
            pointer: buffer.cu_deviceptr(),
            len,
        };
        Ok(Self { buffer, rest })
    }

    fn take(&mut self, len: usize) -> Words {
        let (taken, rest) = self.rest.split(len);
        self.rest = rest;
        taken
    }

    /// Everything taken so far, as one view.
    fn used(&self) -> Words {
        Words {
            pointer: self.buffer.cu_deviceptr(),
            len: usize::try_from(self.rest.pointer - self.buffer.cu_deviceptr())
                .expect("arena offsets fit in usize")
                / size_of::<u32>(),
        }
    }
}

/// Frees `arenas` in stream order, then waits once, so the memory is back with the driver when
/// the call returns. (Dropping a buffer instead synchronizes the context every time.)
fn release(
    stream: &CudaStream,
    arenas: impl IntoIterator<Item = Arena>,
) -> Result<(), DriverError> {
    for arena in arenas {
        // SAFETY: all of this module's work runs on `stream`, so nothing else can still be
        // using the memory when the free runs.
        unsafe { arena.buffer.drop_async(stream)? };
    }
    stream.synchronize()
}

/// Words of scratch for [`Kernels::exclusive_scan`] of `len` words.
fn scan_scratch(len: u32) -> usize {
    let tiles = len.div_ceil(TILE);
    if tiles <= 1 {
        aligned(1)
    } else {
        aligned(tiles as usize) + scan_scratch(tiles)
    }
}

/// Words of scratch for [`Kernels::radix_sort`] of `len` keys.
fn sort_scratch(len: u32) -> usize {
    let counts = RADIX * len.div_ceil(TILE);
    aligned(4 * RADIX as usize) + aligned(counts as usize) + scan_scratch(counts)
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

    /// Stable LSD radix sort of the keys: up to four 8-bit passes between `keys` and
    /// `keys_spare`, carrying `values` (and their spare) along if given. The result ends up back
    /// in `keys` (and `values`). A pass where every key has the same digit would change nothing,
    /// so it is skipped. `scratch` must hold [`sort_scratch`] words.
    fn radix_sort(
        &self,
        stream: &Arc<CudaStream>,
        keys: &mut Words,
        keys_spare: &mut Words,
        mut values: Option<(&mut Words, &mut Words)>,
        scratch: Words,
    ) -> Result<(), DriverError> {
        let len = u32::try_from(keys.len).expect("CUDA input exceeds u32 positions");
        let tiles = len.div_ceil(TILE);
        let (digit_counts, rest) = scratch.split(4 * RADIX as usize);
        let (counts, scan_space) = rest.split((RADIX * tiles) as usize);
        // SAFETY: `digit_counts` is device memory of this many bytes, and everything runs on
        // `stream`.
        unsafe {
            memset_d8_async(
                digit_counts.pointer,
                0,
                digit_counts.len * size_of::<u32>(),
                stream.cu_stream(),
            )?;
        }
        launch(
            stream,
            &self.digit_counts,
            tiles,
            &[keys.pointer, keys.len as u64, digit_counts.pointer],
        )?;
        let digit_counts = read(stream, digit_counts)?;
        for (pass, shift) in [0_u32, 8, 16, 24].into_iter().enumerate() {
            let pass_counts = &digit_counts[pass * RADIX as usize..][..RADIX as usize];
            if pass_counts.contains(&len) {
                continue;
            }
            launch(
                stream,
                &self.radix_histogram,
                tiles,
                &[
                    keys.pointer,
                    keys.len as u64,
                    u64::from(shift),
                    u64::from(tiles),
                    counts.pointer,
                ],
            )?;
            self.exclusive_scan(stream, counts, scan_space)?;
            // No values: an empty slice, and an output nothing writes.
            let (values_in, values_out) = values.as_ref().map_or_else(
                || ([keys.pointer, 0], 0),
                |(values, values_spare)| (values.argument(), values_spare.pointer),
            );
            let mut arguments = keys.argument().to_vec();
            arguments.extend(values_in);
            arguments.extend([u64::from(shift), u64::from(tiles)]);
            arguments.extend(counts.argument());
            arguments.extend([keys_spare.pointer, values_out]);
            launch(stream, &self.radix_scatter, tiles, &arguments)?;
            core::mem::swap(keys, keys_spare);
            if let Some((values, values_spare)) = &mut values {
                core::mem::swap(*values, *values_spare);
            }
        }
        Ok(())
    }

    /// Exclusive scan of `data`, in place, using `scratch` ([`scan_scratch`] words). Returns a
    /// one-word view of their total.
    fn exclusive_scan(
        &self,
        stream: &Arc<CudaStream>,
        data: Words,
        scratch: Words,
    ) -> Result<Words, DriverError> {
        let len = u32::try_from(data.len).expect("CUDA scan exceeds u32 words");
        let tiles = len.div_ceil(TILE).max(1);
        let (tile_sums, rest) = scratch.split(tiles as usize);
        launch(
            stream,
            &self.scan_tiles,
            tiles,
            &[data.pointer, u64::from(len), tile_sums.pointer],
        )?;
        if tiles == 1 {
            return Ok(tile_sums);
        }
        let total = self.exclusive_scan(stream, tile_sums, rest)?;
        let mut arguments = std::vec![data.pointer, u64::from(len)];
        arguments.extend(tile_sums.argument());
        launch(stream, &self.add_tile_offsets, tiles, &arguments)?;
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
