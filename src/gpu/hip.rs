//! [`Engine::Hip`](super::Engine::Hip): the same Rust radix-sort and range kernels as
//! [`Engine::Offload`](super::Engine::Offload) on AMD, launched directly through AMD's HIP
//! runtime instead of OpenMP offload. This is the AMD counterpart of the CUDA-Oxide
//! engine: explicit device buffers, explicit launches, kernels written in Rust.
//!
//! `build.rs` compiles the kernels (`gpu/offload/hip_native.rs`) into an AMD code object
//! that is embedded here and loaded once per process.
//!
//! Every HIP call crosses a C interface, so this module needs `unsafe`. Each block states
//! its invariant; device pointers never escape [`DeviceBuffer`].

use core::ffi::{CStr, c_char, c_void};
use core::mem::swap;
use core::ptr::{from_mut, null_mut};
use std::env::var_os;
use std::sync::OnceLock;
use std::vec;
use std::vec::Vec;

use super::host_ranges;

// Must match gpu/offload/radix.rs and gpu/offload/ranges.rs.
const BLOCK: u32 = 256;
const TILE_KEYS: u32 = 4096;
const TILE: usize = TILE_KEYS as usize;
const RADIX: usize = 256;
const COUNTS_SHARED: u32 = 8192;
const EMIT_SHARED: u32 = 32;
/// `radix_scatter`'s shared bytes for keys of `key_bytes` bytes (see radix.rs).
const fn scatter_shared(key_bytes: u32) -> u32 {
    TILE_KEYS * key_bytes + 10_280
}

const HIP_MEMCPY_HOST_TO_DEVICE: i32 = 1;
const HIP_MEMCPY_DEVICE_TO_HOST: i32 = 2;

unsafe extern "C" {
    fn hipInit(flags: u32) -> i32;
    fn hipGetDeviceCount(count: *mut i32) -> i32;
    fn hipModuleLoadData(module: *mut *mut c_void, image: *const c_void) -> i32;
    fn hipModuleGetFunction(
        function: *mut *mut c_void,
        module: *mut c_void,
        name: *const c_char,
    ) -> i32;
    fn hipMalloc(pointer: *mut *mut c_void, bytes: usize) -> i32;
    fn hipFree(pointer: *mut c_void) -> i32;
    fn hipMemcpy(destination: *mut c_void, source: *const c_void, bytes: usize, kind: i32) -> i32;
    fn hipMemset(destination: *mut c_void, value: i32, bytes: usize) -> i32;
    fn hipModuleLaunchKernel(
        function: *mut c_void,
        grid_x: u32,
        grid_y: u32,
        grid_z: u32,
        block_x: u32,
        block_y: u32,
        block_z: u32,
        shared_bytes: u32,
        stream: *mut c_void,
        parameters: *mut *mut c_void,
        extra: *mut *mut c_void,
    ) -> i32;
    fn hipDeviceSynchronize() -> i32;
    fn hipHostMalloc(pointer: *mut *mut c_void, bytes: usize, flags: u32) -> i32;
    fn hipHostFree(pointer: *mut c_void) -> i32;
    fn hipStreamCreate(stream: *mut *mut c_void) -> i32;
    fn hipStreamDestroy(stream: *mut c_void) -> i32;
    fn hipStreamSynchronize(stream: *mut c_void) -> i32;
    fn hipEventCreate(event: *mut *mut c_void) -> i32;
    fn hipEventDestroy(event: *mut c_void) -> i32;
    fn hipEventRecord(event: *mut c_void, stream: *mut c_void) -> i32;
    fn hipEventSynchronize(event: *mut c_void) -> i32;
    fn hipMemcpyAsync(
        destination: *mut c_void,
        source: *const c_void,
        bytes: usize,
        kind: i32,
        stream: *mut c_void,
    ) -> i32;
}

/// Bytes per upload chunk: each is copied into pinned memory while the one before it
/// transfers. Under WSL2, HIP copies from ordinary (pageable) memory at about 3 GB/s.
const CHUNK_BYTES: usize = 1 << 23;

/// The embedded AMD code object, 8-byte aligned for the loader.
#[repr(C, align(8))]
struct CodeObject<T: ?Sized>(T);
static CODE_OBJECT: &CodeObject<[u8]> = &CodeObject(*include_bytes!(env!("RSB_HIP_CODE_OBJECT")));

fn check(status: i32, what: &str) {
    assert_eq!(
        status, 0,
        "Engine::Hip: {what} failed with HIP error {status}"
    );
}

/// A loaded kernel's handle.
#[derive(Clone, Copy)]
struct Handle(*mut c_void);

// SAFETY: HIP function handles are process-wide and usable from any host thread; the
// module that owns them is never unloaded while the process runs.
unsafe impl Send for Handle {}
// SAFETY: as for Send; handles are only read after initialization.
unsafe impl Sync for Handle {}

/// The loaded kernels for one key width.
pub(super) struct Kernels {
    digit_counts: Handle,
    radix_scatter: Handle,
    count_starts: Handle,
    emit_ranges: Handle,
}

/// One-time process setup, which the benchmark fairness rule allows to persist.
pub(super) struct Hip {
    clear_workspace: Handle,
    narrow: Kernels,
    wide: Kernels,
}

fn hip() -> Option<&'static Hip> {
    static HIP: OnceLock<Option<Hip>> = OnceLock::new();
    HIP.get_or_init(|| {
        let mut count = 0;
        // SAFETY: hipInit takes no pointers; hipGetDeviceCount writes one i32.
        if unsafe { hipInit(0) } != 0
            || unsafe { hipGetDeviceCount(&raw mut count) } != 0
            || count < 1
        {
            return None;
        }
        let mut module = null_mut();
        // SAFETY: the code object is a complete, aligned ELF that lives for the whole process.
        check(
            unsafe { hipModuleLoadData(&raw mut module, CODE_OBJECT.0.as_ptr().cast()) },
            "loading the kernels",
        );
        let function = |name: &CStr| {
            let mut function = null_mut();
            // SAFETY: module is loaded, and name is a NUL-terminated kernel name.
            check(
                unsafe { hipModuleGetFunction(&raw mut function, module, name.as_ptr()) },
                "finding a kernel",
            );
            Handle(function)
        };
        Some(Hip {
            clear_workspace: function(c"rsb_hip_clear_workspace"),
            narrow: Kernels {
                digit_counts: function(c"rsb_hip_digit_counts_u32"),
                radix_scatter: function(c"rsb_hip_radix_scatter_u32"),
                count_starts: function(c"rsb_hip_count_starts_u32"),
                emit_ranges: function(c"rsb_hip_emit_ranges_u32"),
            },
            wide: Kernels {
                digit_counts: function(c"rsb_hip_digit_counts_u64"),
                radix_scatter: function(c"rsb_hip_radix_scatter_u64"),
                count_starts: function(c"rsb_hip_count_starts_u64"),
                emit_ranges: function(c"rsb_hip_emit_ranges_u64"),
            },
        })
    })
    .as_ref()
}

pub(super) fn available() -> bool {
    let available = hip().is_some();
    if var_os("RANGE_SET_BLAZE_REQUIRE_HIP_GPU").is_some() {
        assert!(available, "the required HIP GPU is unavailable");
    }
    available
}

/// GPU memory owned by one call.
struct DeviceBuffer {
    pointer: *mut c_void,
    bytes: usize,
}

impl DeviceBuffer {
    fn new(bytes: usize) -> Self {
        let mut pointer = null_mut();
        // SAFETY: hipMalloc writes one device pointer for an allocation of `bytes`.
        check(
            unsafe { hipMalloc(&raw mut pointer, bytes.max(1)) },
            "allocating GPU memory",
        );
        Self { pointer, bytes }
    }

    /// Uploads `host` a chunk at a time through two pinned buffers allocated for this call,
    /// so copying one chunk into pinned memory overlaps transferring the one before it.
    fn upload_chunked<T: Copy>(host: &[T]) -> Self {
        let buffer = Self::new(size_of_val(host));
        // SAFETY: host is initialized for its whole length, and u8 has no invalid values.
        let bytes: &[u8] =
            unsafe { core::slice::from_raw_parts(host.as_ptr().cast(), size_of_val(host)) };
        let mut pinned = [null_mut(); 2];
        let mut events = [null_mut(); 2];
        let mut stream = null_mut();
        // SAFETY: each call writes one handle; the pinned buffers hold CHUNK_BYTES.
        unsafe {
            check(hipStreamCreate(&raw mut stream), "creating a stream");
            for index in 0..2 {
                check(
                    hipHostMalloc(&raw mut pinned[index], CHUNK_BYTES, 0),
                    "allocating pinned memory",
                );
                check(hipEventCreate(&raw mut events[index]), "creating an event");
            }
        }
        for (index, chunk) in bytes.chunks(CHUNK_BYTES).enumerate() {
            let slot = index % 2;
            // SAFETY: the event marks the last copy out of this pinned buffer; after it
            // completes, the buffer may be overwritten. The destination range lies within
            // the device allocation, and the pinned buffer holds at least chunk.len() bytes.
            unsafe {
                if index >= 2 {
                    check(hipEventSynchronize(events[slot]), "waiting for an upload");
                }
                core::ptr::copy_nonoverlapping(
                    chunk.as_ptr(),
                    pinned[slot].cast::<u8>(),
                    chunk.len(),
                );
                check(
                    hipMemcpyAsync(
                        buffer.pointer.cast::<u8>().add(index * CHUNK_BYTES).cast(),
                        pinned[slot],
                        chunk.len(),
                        HIP_MEMCPY_HOST_TO_DEVICE,
                        stream,
                    ),
                    "uploading",
                );
                check(hipEventRecord(events[slot], stream), "recording an upload");
            }
        }
        // SAFETY: waits for every copy before freeing the pinned buffers, events, and stream.
        unsafe {
            check(hipStreamSynchronize(stream), "finishing the upload");
            for index in 0..2 {
                hipEventDestroy(events[index]);
                hipHostFree(pinned[index]);
            }
            hipStreamDestroy(stream);
        }
        buffer
    }

    fn upload<T: Copy>(host: &[T]) -> Self {
        let buffer = Self::new(size_of_val(host));
        // SAFETY: both regions hold size_of_val(host) bytes; the copy is synchronous.
        check(
            unsafe {
                hipMemcpy(
                    buffer.pointer,
                    host.as_ptr().cast(),
                    buffer.bytes,
                    HIP_MEMCPY_HOST_TO_DEVICE,
                )
            },
            "uploading",
        );
        buffer
    }

    fn zero(&self, bytes: usize) {
        assert!(bytes <= self.bytes);
        // SAFETY: bytes is within this allocation.
        check(
            unsafe { hipMemset(self.pointer, 0, bytes) },
            "clearing GPU memory",
        );
    }

    fn download<T: Copy>(&self, host: &mut [T]) {
        assert!(size_of_val(host) <= self.bytes);
        // SAFETY: host has room for the copied bytes, which lie within this allocation.
        check(
            unsafe {
                hipMemcpy(
                    host.as_mut_ptr().cast(),
                    self.pointer,
                    size_of_val(host),
                    HIP_MEMCPY_DEVICE_TO_HOST,
                )
            },
            "downloading",
        );
    }
}

impl Drop for DeviceBuffer {
    fn drop(&mut self) {
        // SAFETY: pointer came from hipMalloc and is freed once; launches are synchronous.
        unsafe { hipFree(self.pointer) };
    }
}

/// Launches `function` with one 256-thread block per entry of `blocks` and waits for it.
fn launch(function: Handle, blocks: usize, shared_bytes: u32, arguments: &mut [*mut c_void]) {
    let blocks = u32::try_from(blocks).expect("Engine::Hip: too many blocks");
    // SAFETY: function is a loaded kernel whose parameters match `arguments`, each of which
    // points to a live value of the kernel's parameter type; device pointers are live.
    check(
        unsafe {
            hipModuleLaunchKernel(
                function.0,
                blocks,
                1,
                1,
                BLOCK,
                1,
                1,
                shared_bytes,
                null_mut(),
                arguments.as_mut_ptr(),
                null_mut(),
            )
        },
        "launching a kernel",
    );
    // SAFETY: waits for the launch; takes no pointers.
    check(unsafe { hipDeviceSynchronize() }, "running a kernel");
}

/// Erases a kernel argument's type for `hipModuleLaunchKernel`.
const fn argument<T>(value: &mut T) -> *mut c_void {
    from_mut(value).cast()
}

/// A key width the kernels support.
pub(super) trait HipKey: Copy + Default + Into<u64> {
    fn kernels(hip: &Hip) -> &Kernels;
}

impl HipKey for u32 {
    fn kernels(hip: &Hip) -> &Kernels {
        &hip.narrow
    }
}

impl HipKey for u64 {
    fn kernels(hip: &Hip) -> &Kernels {
        &hip.wide
    }
}

/// The size of `K` in bytes, as the `u32` kernel launches take.
fn key_bytes<K>() -> u32 {
    u32::try_from(size_of::<K>()).expect("keys are 4 or 8 bytes")
}

/// Sorts `keys` on the GPU and returns their ranges as `(first, last)` key pairs; `bits`
/// (32 or 64) must cover every key. Keys `gap.0` and `gap.1` also count as adjacent.
/// All GPU memory is allocated within the call; only the ranges come back.
pub(super) fn ranges<K: HipKey>(keys: &[K], bits: u32, gap: Option<(u64, u64)>) -> Vec<(u64, u64)> {
    let hip = hip().expect("Engine::Hip: no HIP GPU is available");
    let len = keys.len();
    if len < 2 {
        return host_ranges(keys, gap);
    }
    assert!(len <= 1 << 30, "Engine::Hip: input exceeds 2^30 keys");
    assert!(
        matches!(bits, 32 | 64),
        "Engine::Hip: keys must have 32 or 64 bits"
    );
    match sort(hip, keys, bits) {
        Sorted::AllEqual => {
            let key = keys[0].into();
            vec![(key, key)]
        }
        Sorted::OnDevice { sorted, free } => find_ranges::<K>(hip, &sorted, &free, len, gap),
    }
}

/// Where `sort` left the keys.
enum Sorted {
    /// Every byte was constant, so all keys are equal.
    AllEqual,
    /// The sorted keys are in `sorted`; `free` is a same-sized buffer no longer needed.
    OnDevice {
        sorted: DeviceBuffer,
        free: DeviceBuffer,
    },
}

/// Uploads `keys` and radix-sorts them on the GPU, 8 bits per pass, skipping passes
/// whose digit is the same for every key.
fn sort<K: HipKey>(hip: &Hip, keys: &[K], bits: u32) -> Sorted {
    let kernels = K::kernels(hip);
    let len = keys.len();
    let mut len_argument = u32::try_from(len).expect("checked by the caller");
    let blocks = len.div_ceil(TILE);
    let workspace_len = 1 + blocks * RADIX;

    // Count every digit position's bytes in one read.
    let mut input = DeviceBuffer::upload_chunked(keys);
    let counts = DeviceBuffer::new(2048 * size_of::<u64>());
    counts.zero(counts.bytes);
    let mut passes = bits / 8;
    let (mut keys_pointer, mut counts_pointer) = (input.pointer, counts.pointer);
    launch(
        kernels.digit_counts,
        blocks,
        COUNTS_SHARED,
        &mut [
            argument(&mut keys_pointer),
            argument(&mut len_argument),
            argument(&mut passes),
            argument(&mut counts_pointer),
        ],
    );
    let mut histogram = [0_u64; 2048];
    counts.download(&mut histogram);
    // Turn each pass's counts into digit start positions; note which passes do anything.
    let mut active = [false; 8];
    for (pass, is_active) in active.iter_mut().enumerate().take(passes as usize) {
        let digits = &mut histogram[pass * RADIX..(pass + 1) * RADIX];
        let largest = digits.iter().copied().max().unwrap_or(0);
        let mut total = 0;
        for start in digits.iter_mut() {
            let count = *start;
            *start = total;
            total += count;
        }
        assert_eq!(
            total, len as u64,
            "Engine::Hip: digit histogram is inconsistent"
        );
        *is_active = largest != len as u64;
    }
    if !active.contains(&true) {
        return Sorted::AllEqual;
    }

    // Radix passes, alternating between two buffers.
    let bases = DeviceBuffer::upload(&histogram);
    let mut output = DeviceBuffer::new(size_of_val(keys));
    let status = DeviceBuffer::new(workspace_len * size_of::<u64>());
    let mut workspace_len_argument = u32::try_from(workspace_len).expect("at most 2^26 words");
    for (pass, _) in active
        .iter()
        .enumerate()
        .filter(|&(_, &is_active)| is_active)
    {
        let mut status_pointer = status.pointer;
        launch(
            hip.clear_workspace,
            workspace_len.div_ceil(BLOCK as usize),
            0,
            &mut [
                argument(&mut status_pointer),
                argument(&mut workspace_len_argument),
            ],
        );
        let (mut from, mut to, mut bases_pointer) = (input.pointer, output.pointer, bases.pointer);
        let pass = u32::try_from(pass).expect("at most 8 passes");
        let (mut shift, mut offset) = (pass * 8, pass * 256);
        launch(
            kernels.radix_scatter,
            blocks,
            scatter_shared(key_bytes::<K>()),
            &mut [
                argument(&mut from),
                argument(&mut len_argument),
                argument(&mut shift),
                argument(&mut bases_pointer),
                argument(&mut offset),
                argument(&mut status_pointer),
                argument(&mut to),
            ],
        );
        swap(&mut input, &mut output);
    }
    Sorted::OnDevice {
        sorted: input,
        free: output,
    }
}

/// Finds the ranges of the `len` sorted keys in `sorted`: counts starts per tile on the
/// GPU, scans those counts on the host, then writes `(first, last)` pairs into `free`
/// and downloads only them.
fn find_ranges<K: HipKey>(
    hip: &Hip,
    sorted: &DeviceBuffer,
    free: &DeviceBuffer,
    len: usize,
    gap: Option<(u64, u64)>,
) -> Vec<(u64, u64)> {
    let kernels = K::kernels(hip);
    let blocks = len.div_ceil(TILE);
    let mut len_argument = u32::try_from(len).expect("checked by the caller");
    let (gap_left, gap_right) = gap.unwrap_or((0, 0));
    let (mut gap_left, mut gap_right, mut has_gap) =
        (gap_left, gap_right, u32::from(gap.is_some()));
    let tile_counts = DeviceBuffer::new(blocks * size_of::<u64>());
    tile_counts.zero(tile_counts.bytes);
    let (mut sorted_pointer, mut counts_pointer) = (sorted.pointer, tile_counts.pointer);
    launch(
        kernels.count_starts,
        blocks,
        0,
        &mut [
            argument(&mut sorted_pointer),
            argument(&mut len_argument),
            argument(&mut gap_left),
            argument(&mut gap_right),
            argument(&mut has_gap),
            argument(&mut counts_pointer),
        ],
    );
    let mut offsets = vec![0_u64; blocks];
    tile_counts.download(&mut offsets);
    let mut total = 0_usize;
    for word in &mut offsets {
        let tile_count = usize::try_from(*word).expect("at most 2^30 ranges");
        *word = total as u64;
        total += tile_count;
    }
    if 2 * total > len {
        // The ranges are no smaller than the keys; bring back the sorted keys instead.
        let mut sorted_keys = vec![K::default(); len];
        sorted.download(&mut sorted_keys);
        return host_ranges(&sorted_keys, gap);
    }
    let tile_offsets = DeviceBuffer::upload(&offsets);
    let (mut offsets_pointer, mut ranges_pointer) = (tile_offsets.pointer, free.pointer);
    launch(
        kernels.emit_ranges,
        blocks,
        EMIT_SHARED,
        &mut [
            argument(&mut sorted_pointer),
            argument(&mut len_argument),
            argument(&mut gap_left),
            argument(&mut gap_right),
            argument(&mut has_gap),
            argument(&mut offsets_pointer),
            argument(&mut ranges_pointer),
        ],
    );
    let mut pairs = vec![K::default(); 2 * total];
    free.download(&mut pairs);
    pairs
        .as_chunks::<2>()
        .0
        .iter()
        .map(|&[first, last]| (first.into(), last.into()))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gpu_ranges_match_host_ranges() {
        if !available() {
            return;
        }
        let gap = Some((0xd7ff, 0xe000));
        for len in [2_usize, 3, 1025, 4095, 4096, 4097, 65537, 1_000_003] {
            for pattern in 0..7 {
                let keys: Vec<u64> = (0..len as u64)
                    .map(|index| {
                        let random = index.wrapping_mul(0x9e37_79b9_7f4a_7c15);
                        match pattern {
                            0 => (random % len as u64) / 256 * 257 + random % 256,
                            1 => random & u64::from(u32::MAX),
                            2 => random % 1000 + (random % 7) * 5000,
                            3 => 0xd7f0 + random % 16 + (random % 2) * 0x0810,
                            4 => 42,
                            5 => u64::MAX - random % 300,
                            _ => index,
                        }
                    })
                    .collect();
                let mut sorted = keys.clone();
                sorted.sort_unstable();
                let expected = host_ranges(&sorted, gap);
                if keys.iter().all(|&key| u32::try_from(key).is_ok()) {
                    let narrow: Vec<u32> = keys
                        .iter()
                        .map(|&key| u32::try_from(key).expect("checked above"))
                        .collect();
                    assert_eq!(
                        ranges(&narrow, 32, gap),
                        expected,
                        "u32: len={len}, pattern={pattern}"
                    );
                }
                assert_eq!(
                    ranges(&keys, 64, gap),
                    expected,
                    "u64: len={len}, pattern={pattern}"
                );
            }
        }
    }
}
