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
    match try_ranges_32(context, values, encode, gap) {
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
    let len = u32::try_from(values.len()).expect("CUDA input exceeds u32 positions");
    if len == 0 {
        return Ok(Vec::new());
    }
    let kernels = Kernels::load(context)?;
    let stream = context.default_stream();
    let tiles = len.div_ceil(TILE);

    // Sort: four stable 8-bit passes, alternating between two buffers.
    let mut sorted = scratch(&stream, values.len())?;
    upload(context, &stream, values, encode, &sorted)?;
    let mut spare = scratch(&stream, values.len())?;
    let counts = scratch(&stream, (RADIX * tiles) as usize)?;
    for shift in [0_u32, 8, 16, 24] {
        launch(
            &stream,
            &kernels.radix_histogram,
            tiles,
            &[
                pointer(&sorted),
                u64::from(len),
                u64::from(shift),
                u64::from(tiles),
                pointer(&counts),
            ],
        )?;
        let total = kernels.exclusive_scan(&stream, pointer(&counts), RADIX * tiles)?;
        // SAFETY: as in `release`, everything runs on `stream`.
        unsafe { total.drop_async(&stream)? };
        launch(
            &stream,
            &kernels.radix_scatter,
            tiles,
            &[
                pointer(&sorted),
                u64::from(len),
                pointer(&sorted), // no values: an empty slice
                0,
                u64::from(shift),
                u64::from(tiles),
                pointer(&counts),
                u64::from(RADIX * tiles),
                pointer(&spare),
                0,
            ],
        )?;
        core::mem::swap(&mut sorted, &mut spare);
    }

    // Ranges: mark each range's first key, scan the marks, then emit each range's ends.
    let (gap_left, gap_right) = gap.unwrap_or((0, 0));
    let has_gap = u64::from(gap.is_some());
    let marks = spare;
    let elements = len.div_ceil(BLOCK);
    launch(
        &stream,
        &kernels.mark_starts,
        elements,
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
    let count = total.to_host_vec(&stream)?[0] as usize;
    let starts = scratch(&stream, count)?;
    let ends = scratch(&stream, count)?;
    launch(
        &stream,
        &kernels.emit_ranges,
        elements,
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
    release(&stream, [sorted, marks, counts, total, starts, ends])?;
    Ok(starts_host.into_iter().zip(ends_host).collect())
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
    scan_tiles: CudaFunction,
    add_tile_offsets: CudaFunction,
    mark_starts: CudaFunction,
    emit_ranges: CudaFunction,
}

impl Kernels {
    fn load(context: &Arc<CudaContext>) -> Result<Self, DriverError> {
        let module = context.load_module_from_ptx_src(PTX)?;
        Ok(Self {
            radix_histogram: module.load_function("radix_histogram")?,
            radix_scatter: module.load_function("radix_scatter")?,
            scan_tiles: module.load_function("scan_tiles")?,
            add_tile_offsets: module.load_function("add_tile_offsets")?,
            mark_starts: module.load_function("mark_starts")?,
            emit_ranges: module.load_function("emit_ranges")?,
            _module: module,
        })
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

    use crate::{Engine, RangeSetBlaze};

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
    #[should_panic(expected = "only 32-bit key types")]
    fn wider_keys_panic() {
        let _ = RangeSetBlaze::<u64>::from_shuffled_slice(&mut [1, 2], Engine::CudaOnly);
    }
}
