//! Portable GPU-assisted construction for large unsorted slices.

use core::cell::RefCell;
use core::mem::size_of;
use core::net::{Ipv4Addr, Ipv6Addr};
use core::num::NonZeroUsize;
use std::sync::{Arc, Mutex, OnceLock, mpsc};
use std::vec::Vec;

use lampshade::{Compactor, Context, KeyValueCompactor, KeyValueSoaSorter, Sorter};
use rayon::slice::ParallelSliceMut;

#[cfg(doc)]
use crate::RangeMapBlaze;
#[cfg(feature = "float_nightly_experimental")]
use crate::float::{gpu_not_nan_f16_from_ordered, gpu_not_nan_f16_to_ordered};
use crate::float::{
    gpu_not_nan_f32_from_ordered, gpu_not_nan_f32_to_ordered, gpu_not_nan_f64_from_ordered,
    gpu_not_nan_f64_to_ordered,
};
use crate::{CheckSortedDisjoint, NotNanF32, NotNanF64, RangeSetBlaze, TotalF32, TotalF64};
#[cfg(feature = "float_nightly_experimental")]
use crate::{NotNanF16, NotNanF128, TotalF16, TotalF128};

#[cfg(feature = "cuda")]
mod cuda;
#[cfg(feature = "cutile")]
mod cutile;
mod map_ingest;

const WORKGROUP_SIZE: u32 = 256;
const BOUNDARY_32_SHADER: &str = r"
struct Parameters {
    len: u32,
    groups_x: u32,
    gap_left: u32,
    gap_right: u32,
}

@group(0) @binding(0) var<storage, read> values: array<u32>;
@group(0) @binding(1) var<storage, read_write> start_mask: array<u32>;
@group(0) @binding(2) var<storage, read_write> end_mask: array<u32>;
@group(0) @binding(3) var<uniform> parameters: Parameters;

fn adjacent(left: u32, right: u32) -> bool {
    let ordinary = left != 0xffffffffu && right == left + 1u;
    let exceptional = left == parameters.gap_left && right == parameters.gap_right;
    return ordinary || exceptional;
}

@compute @workgroup_size(256)
fn main(
    @builtin(workgroup_id) workgroup: vec3<u32>,
    @builtin(local_invocation_id) local: vec3<u32>,
) {
    let group = workgroup.y * parameters.groups_x + workgroup.x;
    let index = group * 256u + local.x;
    if index >= parameters.len {
        return;
    }

    let value = values[index];
    var is_start = index == 0u;
    if !is_start {
        let previous = values[index - 1u];
        is_start = value != previous && !adjacent(previous, value);
    }

    var is_end = index + 1u == parameters.len;
    if !is_end {
        let next = values[index + 1u];
        is_end = next != value && !adjacent(value, next);
    }

    start_mask[index] = select(0u, 1u, is_start);
    end_mask[index] = select(0u, 1u, is_end);
}
";

const BOUNDARY_64_SHADER: &str = r"
struct Parameters {
    len: u32,
    groups_x: u32,
    gap_left_hi: u32,
    gap_left_lo: u32,
    gap_right_hi: u32,
    gap_right_lo: u32,
    _padding0: u32,
    _padding1: u32,
}

struct Key {
    hi: u32,
    lo: u32,
}

@group(0) @binding(0) var<storage, read> hi: array<u32>;
@group(0) @binding(1) var<storage, read> lo: array<u32>;
@group(0) @binding(2) var<storage, read_write> pairs: array<Key>;
@group(0) @binding(3) var<storage, read_write> start_mask: array<u32>;
@group(0) @binding(4) var<storage, read_write> end_mask: array<u32>;
@group(0) @binding(5) var<uniform> parameters: Parameters;

fn same(left: Key, right: Key) -> bool {
    return left.hi == right.hi && left.lo == right.lo;
}

fn adjacent(left: Key, right: Key) -> bool {
    let ordinary = left.hi == right.hi && left.lo != 0xffffffffu && right.lo == left.lo + 1u;
    let carry = left.lo == 0xffffffffu && right.lo == 0u &&
        left.hi != 0xffffffffu && right.hi == left.hi + 1u;
    let exceptional = left.hi == parameters.gap_left_hi &&
        left.lo == parameters.gap_left_lo &&
        right.hi == parameters.gap_right_hi &&
        right.lo == parameters.gap_right_lo;
    return ordinary || carry || exceptional;
}

@compute @workgroup_size(256)
fn main(
    @builtin(workgroup_id) workgroup: vec3<u32>,
    @builtin(local_invocation_id) local: vec3<u32>,
) {
    let group = workgroup.y * parameters.groups_x + workgroup.x;
    let index = group * 256u + local.x;
    if index >= parameters.len {
        return;
    }

    let value = Key(hi[index], lo[index]);
    pairs[index] = value;
    var is_start = index == 0u;
    if !is_start {
        let previous = Key(hi[index - 1u], lo[index - 1u]);
        is_start = !same(previous, value) && !adjacent(previous, value);
    }

    var is_end = index + 1u == parameters.len;
    if !is_end {
        let next = Key(hi[index + 1u], lo[index + 1u]);
        is_end = !same(value, next) && !adjacent(value, next);
    }

    start_mask[index] = select(0u, 1u, is_start);
    end_mask[index] = select(0u, 1u, is_end);
}
";

// Maintenance: every crate-owned `Integer` type must have a policy below and
// appear in `gpu_api!`. This stays separate from the public `Integer` trait so
// enabling `shuffled_slices` does not break downstream `Integer` implementations.
trait GpuElement: crate::Integer {
    const KEY_WORDS: u8;
    const GAP: Option<(u64, u64)> = None;

    fn encode(self) -> u64;
    fn decode(key: u64) -> Self;
}

/// How [`RangeSetBlaze::from_shuffled_slice`] and [`RangeMapBlaze::from_shuffled_slices`] do
/// their work.
///
/// A GPU engine named on its own ([`Engine::Wgpu`], [`Engine::Cuda`]) panics if it can't run;
/// its `…OrCpu` form ([`Engine::WgpuOrCpu`], [`Engine::CudaOrCpu`]) uses [`Engine::Cpu`] with
/// every available thread instead.
///
/// *Experimental; requires the `shuffled_slices` feature.*
#[cfg_attr(docsrs, doc(cfg(feature = "shuffled_slices")))]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Engine {
    /// Sort on this many CPU threads (Rayon's `par_sort_unstable`), then find the ranges. For a
    /// set, the sort is in place, and `Cpu(1)` is plain `sort_unstable` followed by
    /// [`RangeSetBlaze::from_slice`].
    Cpu(NonZeroUsize),
    /// Sort and find the ranges on the GPU with portable wgpu compute shaders (Vulkan, Metal,
    /// DirectX 12), in one pass; only the ranges come back.
    ///
    /// For sets, [`Engine::Cpu`] with every thread was faster on the machines measured; for maps
    /// whose keys form few ranges, this engine was faster. Either way, it leaves the CPU free for
    /// other work.
    ///
    /// # Panics
    ///
    /// Panics if there is no hardware GPU, if the key type has no GPU representation (`u128`,
    /// `i128`, `Ipv6Addr`, `NotNanF128`, and the `TotalF*` types), or if the input exceeds the
    /// GPU's limits or memory. For a map, it also panics for key and value types the GPU doesn't
    /// handle; [`RangeMapBlaze::from_shuffled_slices`] lists the ones it does.
    Wgpu,
    /// [`Engine::Wgpu`] when it can run; otherwise [`Engine::Cpu`] with every available
    /// thread.
    WgpuOrCpu,
    /// Like [`Engine::Wgpu`], with the same types, but with CUDA kernels written in Rust
    /// (cuda-oxide) instead of portable shaders. NVIDIA GPUs on Linux only. On large shuffled
    /// inputs (10 million keys or more) it was faster than [`Engine::Cpu`] with every thread for
    /// every key type measured.
    ///
    /// *Experimental; requires the `cuda` feature, which builds with `cargo oxide`.*
    ///
    /// # Panics
    ///
    /// Panics if there is no CUDA GPU, for the types [`Engine::Wgpu`] panics for, or if the
    /// input exceeds the GPU's memory.
    #[cfg(feature = "cuda")]
    Cuda,
    /// [`Engine::Cuda`] when it can run; otherwise [`Engine::Cpu`] with every available thread.
    ///
    /// *Experimental; requires the `cuda` feature, which builds with `cargo oxide`.*
    #[cfg(feature = "cuda")]
    CudaOrCpu,
}

fn from_shuffled_slice<T: GpuElement>(values: &mut [T], engine: Engine) -> RangeSetBlaze<T> {
    match engine {
        Engine::Cpu(threads) => from_slice_cpu(values, threads),
        Engine::Wgpu => {
            assert!(
                T::KEY_WORDS != 0,
                "Engine::Wgpu: this key type has no GPU representation"
            );
            let context = gpu_context().expect("Engine::Wgpu: no hardware GPU is available");
            from_slice_gpu_with_context(values, context)
                .expect("Engine::Wgpu: the input exceeds the GPU's limits or memory")
        }
        Engine::WgpuOrCpu => (T::KEY_WORDS != 0)
            .then(|| try_gpu(values))
            .flatten()
            .unwrap_or_else(|| from_slice_cpu(values, all_threads())),
        #[cfg(feature = "cuda")]
        Engine::Cuda => {
            try_cuda(values).unwrap_or_else(|unavailable| panic!("Engine::Cuda: {unavailable}"))
        }
        #[cfg(feature = "cuda")]
        Engine::CudaOrCpu => {
            try_cuda(values).unwrap_or_else(|_| from_slice_cpu(values, all_threads()))
        }
    }
}

/// Why [`Engine::Cuda`] couldn't run.
#[cfg(feature = "cuda")]
#[derive(Debug)]
enum CudaUnavailable {
    NoGpu,
    UnsupportedType,
    OutOfMemory,
}

#[cfg(feature = "cuda")]
impl core::fmt::Display for CudaUnavailable {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(match self {
            Self::NoGpu => "no CUDA GPU is available",
            Self::UnsupportedType => "the key or value type has no GPU representation",
            Self::OutOfMemory => "the input exceeds the GPU's memory",
        })
    }
}

#[cfg(feature = "cuda")]
fn try_cuda<T: GpuElement>(values: &[T]) -> Result<RangeSetBlaze<T>, CudaUnavailable> {
    if T::KEY_WORDS == 0 {
        return Err(CudaUnavailable::UnsupportedType);
    }
    let cuda = cuda::cuda_context().ok_or(CudaUnavailable::NoGpu)?;
    let ranges = if T::KEY_WORDS == 1 {
        let gap = T::GAP.map(|(left, right)| {
            (
                u32::try_from(left).expect("32-bit gap exceeded u32"),
                u32::try_from(right).expect("32-bit gap exceeded u32"),
            )
        });
        let encode = |value: T| u32::try_from(value.encode()).expect("32-bit GPU key exceeded u32");
        cuda::ranges_32(cuda, values, encode, gap).map(|ranges| {
            ranges
                .into_iter()
                .map(|(start, end)| (u64::from(start), u64::from(end)))
                .collect()
        })
    } else {
        cuda::ranges_64(cuda, values, T::encode, T::GAP)
    }
    .ok_or(CudaUnavailable::OutOfMemory)?;
    let ranges = ranges
        .into_iter()
        .map(|(start, end)| T::decode(start)..=T::decode(end));
    Ok(RangeSetBlaze::from_sorted_disjoint(
        CheckSortedDisjoint::new(ranges),
    ))
}

fn all_threads() -> NonZeroUsize {
    std::thread::available_parallelism().unwrap_or(NonZeroUsize::MIN)
}

/// Sorts the slice in place on `threads` threads (Rayon), then finds the ranges with
/// [`RangeSetBlaze::from_slice`], which needs only one fast pass over sorted input.
fn from_slice_cpu<T: crate::Integer>(values: &mut [T], threads: NonZeroUsize) -> RangeSetBlaze<T> {
    sort_in_place(values, threads);
    RangeSetBlaze::from_slice(&*values)
}

fn sort_in_place<T: Ord + Send>(values: &mut [T], threads: NonZeroUsize) {
    if threads.get() == 1 {
        values.sort_unstable();
    } else {
        thread_pool(threads).install(|| values.par_sort_unstable());
    }
}

/// A Rayon pool with exactly `threads` threads, created on first use and then reused, so
/// repeated calls don't pay to start threads.
fn thread_pool(threads: NonZeroUsize) -> Arc<rayon::ThreadPool> {
    static POOLS: Mutex<Vec<(NonZeroUsize, Arc<rayon::ThreadPool>)>> = Mutex::new(Vec::new());
    let mut pools = POOLS
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    if let Some((_, pool)) = pools.iter().find(|(count, _)| *count == threads) {
        return Arc::clone(pool);
    }
    let pool = Arc::new(
        rayon::ThreadPoolBuilder::new()
            .num_threads(threads.get())
            .build()
            .expect("could not start the CPU sort threads"),
    );
    pools.push((threads, Arc::clone(&pool)));
    pool
}

/// Runs on the GPU, or returns `None` if there is no hardware GPU, the input exceeds the GPU's
/// limits, or the GPU runs out of memory. Any other GPU error panics.
fn try_gpu<T: GpuElement>(values: &[T]) -> Option<RangeSetBlaze<T>> {
    from_slice_gpu_with_context(values, gpu_context()?)
}

fn from_slice_gpu_with_context<T: GpuElement>(
    values: &[T],
    context: &Context,
) -> Option<RangeSetBlaze<T>> {
    if values.is_empty() {
        return Some(RangeSetBlaze::new());
    }
    let len = u32::try_from(values.len()).ok()?;
    let endpoints = with_error_scopes(context, || match T::KEY_WORDS {
        1 => normalize_32(context, values, len, T::GAP),
        2 => normalize_64(context, values, len, T::GAP),
        _ => None,
    });
    let ranges = endpoints?
        .into_iter()
        .map(|(start, end)| T::decode(start)..=T::decode(end));
    Some(RangeSetBlaze::from_sorted_disjoint(
        CheckSortedDisjoint::new(ranges),
    ))
}

/// Runs `run`, which records and submits GPU work. Returns `None` if `run` does, or if the GPU
/// ran out of memory; panics on any other GPU error.
fn with_error_scopes<R>(context: &Context, run: impl FnOnce() -> Option<R>) -> Option<R> {
    // wgpu reports a failed allocation later, as an error on this thread's error scopes, not
    // at the call. So capture out-of-memory and validation errors for the whole run.
    GPU_FAILURE.with_borrow_mut(Option::take);
    let out_of_memory = context
        .device
        .push_error_scope(wgpu::ErrorFilter::OutOfMemory);
    let validation = context
        .device
        .push_error_scope(wgpu::ErrorFilter::Validation);
    let result = run();
    let validation_error = pollster::block_on(validation.pop());
    if pollster::block_on(out_of_memory.pop()).is_some() {
        return None; // Out of GPU memory: the caller falls back to the CPU.
    }
    if let Some(error) = validation_error {
        panic!("GPU range implementation failure: {error}");
    }
    if let Some(failure) = GPU_FAILURE.with_borrow_mut(Option::take) {
        panic!("GPU range execution failed: {failure}");
    }
    result
}

fn gpu_context() -> Option<&'static Context> {
    static CONTEXT: OnceLock<Option<Context>> = OnceLock::new();
    CONTEXT
        .get_or_init(|| pollster::block_on(init_gpu_context(false)))
        .as_ref()
}

/// A context for tests, which may also use a software Vulkan adapter (such as Mesa's lavapipe),
/// so CI can run the GPU pipeline on machines without a GPU. See [`accepts_adapter`].
#[cfg(test)]
fn test_context() -> Option<&'static Context> {
    static CONTEXT: OnceLock<Option<Context>> = OnceLock::new();
    CONTEXT
        .get_or_init(|| pollster::block_on(init_gpu_context(true)))
        .as_ref()
}

/// Whether to create a device on this adapter. Real use never takes a software adapter: the CPU
/// is faster, and creating a device on Windows' software adapter (WARP) crashed in CI. Tests may
/// take a software *Vulkan* adapter, which is how CI exercises the pipeline without a GPU.
fn accepts_adapter(adapter_info: &wgpu::AdapterInfo, allow_software_vulkan: bool) -> bool {
    adapter_info.device_type != wgpu::DeviceType::Cpu
        || (allow_software_vulkan && adapter_info.backend == wgpu::Backend::Vulkan)
}

async fn init_gpu_context(allow_software_vulkan: bool) -> Option<Context> {
    let descriptor = wgpu::InstanceDescriptor {
        backends: wgpu::Backends::PRIMARY,
        ..wgpu::InstanceDescriptor::new_without_display_handle()
    }
    .with_env();
    let instance = wgpu::Instance::new(descriptor);
    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            compatible_surface: None,
            force_fallback_adapter: false,
            apply_limit_buckets: false,
        })
        .await
        .ok()?;
    let adapter_info = adapter.get_info();
    if !accepts_adapter(&adapter_info, allow_software_vulkan) {
        return None;
    }
    let supported_features = adapter.features();
    let required_features = gpu_context_features(&adapter_info, supported_features);
    let (device, queue) = adapter
        .request_device(&wgpu::DeviceDescriptor {
            label: Some("RangeSetBlaze GPU context"),
            required_features,
            required_limits: adapter.limits(),
            memory_hints: wgpu::MemoryHints::Performance,
            ..Default::default()
        })
        .await
        .ok()?;
    Some(Context {
        adapter_info,
        device,
        queue,
    })
}

fn gpu_context_features(
    adapter_info: &wgpu::AdapterInfo,
    supported_features: wgpu::Features,
) -> wgpu::Features {
    // Keep Lampshade's optional compute-feature policy while additionally
    // enabling mapped primary buffers only for a shared-memory adapter.
    let unreliable_optional_compute = adapter_info.backend == wgpu::Backend::Vulkan
        && adapter_info.vendor == 0x10de
        && adapter_info.device_type == wgpu::DeviceType::IntegratedGpu;
    let apple_metal = adapter_info.backend == wgpu::Backend::Metal
        && (adapter_info.vendor == 0x106b || adapter_info.name.starts_with("Apple "));
    let mut requested = wgpu::Features::empty();
    if !unreliable_optional_compute {
        requested |= wgpu::Features::SUBGROUP;
        if !apple_metal {
            requested |= wgpu::Features::TIMESTAMP_QUERY;
        }
    }
    if should_use_mappable_primary_buffers(adapter_info.device_type, supported_features) {
        requested |= wgpu::Features::MAPPABLE_PRIMARY_BUFFERS;
    }
    supported_features & requested
}

fn should_use_mappable_primary_buffers(
    device_type: wgpu::DeviceType,
    features: wgpu::Features,
) -> bool {
    device_type == wgpu::DeviceType::IntegratedGpu
        && features.contains(wgpu::Features::MAPPABLE_PRIMARY_BUFFERS)
}

fn uses_mappable_primary_buffers(context: &Context) -> bool {
    should_use_mappable_primary_buffers(context.adapter_info.device_type, context.device.features())
}

fn normalize_32<T: GpuElement>(
    context: &Context,
    values: &[T],
    len: u32,
    gap: Option<(u64, u64)>,
) -> Option<Vec<(u64, u64)>> {
    let byte_len = u64::from(len) * size_of::<u32>() as u64;
    if !buffers_fit(context, byte_len, byte_len * 2) {
        return None;
    }
    let input = input_storage_buffer(
        context,
        "GPU range input",
        byte_len,
        values
            .iter()
            .copied()
            .map(|value| u32::try_from(value.encode()).expect("32-bit GPU key exceeded u32")),
    )?;
    let sorted = storage_buffer(
        context,
        "GPU range sorted",
        byte_len,
        wgpu::BufferUsages::empty(),
    );
    let buffers = BoundaryBuffers::new(context, byte_len);
    let parameters = match gap {
        Some((left, right)) => [
            len,
            groups_x(context, len)?,
            u32::try_from(left).expect("32-bit gap exceeded u32"),
            u32::try_from(right).expect("32-bit gap exceeded u32"),
        ],
        None => [len, groups_x(context, len)?, 0, 0],
    };
    let boundary = BoundaryKernel::new_32(context, &sorted, &buffers, &parameters);
    let mut radix_sorter = Sorter::from_context(context);
    let mut compactor = Compactor::from_context(context);
    let mut encoder = encoder(context, "GPU range normalization");
    resource_or_panic(radix_sorter.record_sort(&mut encoder, &input, &sorted, len))?;
    boundary.record(&mut encoder);
    record_compactions_32(&mut compactor, &mut encoder, &sorted, &buffers, len)?;
    submit(context, encoder.finish())?;
    read_endpoints_32(context, &buffers, len)
}

fn normalize_64<T: GpuElement>(
    context: &Context,
    values: &[T],
    len: u32,
    gap: Option<(u64, u64)>,
) -> Option<Vec<(u64, u64)>> {
    let word_bytes = u64::from(len) * size_of::<u32>() as u64;
    let pair_bytes = u64::from(len) * 2 * size_of::<u32>() as u64;
    if !buffers_fit(context, pair_bytes, pair_bytes * 2) {
        return None;
    }
    let lo = input_storage_buffer(
        context,
        "GPU range low words",
        word_bytes,
        values.iter().copied().map(|value| low_word(value.encode())),
    )?;
    let hi = input_storage_buffer(
        context,
        "GPU range high words",
        word_bytes,
        values
            .iter()
            .copied()
            .map(|value| high_word(value.encode())),
    )?;
    let pairs = storage_buffer(
        context,
        "GPU range sorted pairs",
        pair_bytes,
        wgpu::BufferUsages::empty(),
    );
    let buffers = BoundaryBuffers::new_with_endpoint_size(context, word_bytes, pair_bytes);
    let (left, right) = gap.unwrap_or((0, 0));
    let parameters = [
        len,
        groups_x(context, len)?,
        high_word(left),
        low_word(left),
        high_word(right),
        low_word(right),
        0,
        0,
    ];
    let boundary = BoundaryKernel::new_64(context, &hi, &lo, &pairs, &buffers, &parameters);
    // Two independently prepared stable sorters avoid rebuilding bindings when
    // the key/value roles swap for the second lexicographic pass.
    let mut low_sorter = KeyValueSoaSorter::from_context(context);
    let mut high_sorter = KeyValueSoaSorter::from_context(context);
    resource_or_panic(low_sorter.prepare_sort(&lo, &hi, len))?;
    resource_or_panic(high_sorter.prepare_sort(&hi, &lo, len))?;
    let mut compactor = KeyValueCompactor::from_context(context);
    let mut encoder = encoder(context, "GPU 64-bit range normalization");
    resource_or_panic(low_sorter.record_reserved_sort(&mut encoder, &lo, &hi, len))?;
    resource_or_panic(high_sorter.record_reserved_sort(&mut encoder, &hi, &lo, len))?;
    boundary.record(&mut encoder);
    resource_or_panic(compactor.record_compact(
        &mut encoder,
        &pairs,
        &buffers.start_mask,
        &buffers.starts,
        &buffers.start_count,
        len,
    ))?;
    resource_or_panic(compactor.record_compact(
        &mut encoder,
        &pairs,
        &buffers.end_mask,
        &buffers.ends,
        &buffers.end_count,
        len,
    ))?;
    submit(context, encoder.finish())?;
    read_endpoints_64(context, &buffers, len)
}

fn record_compactions_32(
    compactor: &mut Compactor,
    encoder: &mut wgpu::CommandEncoder,
    sorted: &wgpu::Buffer,
    buffers: &BoundaryBuffers,
    len: u32,
) -> Option<()> {
    resource_or_panic(compactor.record_compact(
        encoder,
        sorted,
        &buffers.start_mask,
        &buffers.starts,
        &buffers.start_count,
        len,
    ))?;
    resource_or_panic(compactor.record_compact(
        encoder,
        sorted,
        &buffers.end_mask,
        &buffers.ends,
        &buffers.end_count,
        len,
    ))
}

struct BoundaryBuffers {
    start_mask: wgpu::Buffer,
    end_mask: wgpu::Buffer,
    starts: wgpu::Buffer,
    ends: wgpu::Buffer,
    start_count: wgpu::Buffer,
    end_count: wgpu::Buffer,
    count_readback: Option<wgpu::Buffer>,
}

impl BoundaryBuffers {
    fn new(context: &Context, byte_len: u64) -> Self {
        Self::new_with_endpoint_size(context, byte_len, byte_len)
    }

    fn new_with_endpoint_size(context: &Context, mask_bytes: u64, endpoint_bytes: u64) -> Self {
        let start_mask = storage_buffer(
            context,
            "GPU range start mask",
            mask_bytes,
            wgpu::BufferUsages::COPY_SRC,
        );
        let end_mask = storage_buffer(
            context,
            "GPU range end mask",
            mask_bytes,
            wgpu::BufferUsages::COPY_SRC,
        );
        let readback_usage = if uses_mappable_primary_buffers(context) {
            wgpu::BufferUsages::MAP_READ
        } else {
            wgpu::BufferUsages::COPY_SRC
        };
        let starts = storage_buffer(context, "GPU range starts", endpoint_bytes, readback_usage);
        let ends = storage_buffer(context, "GPU range ends", endpoint_bytes, readback_usage);
        let count_usage = readback_usage | wgpu::BufferUsages::COPY_DST;
        let start_count = storage_buffer(context, "GPU range start count", 4, count_usage);
        let end_count = storage_buffer(context, "GPU range end count", 4, count_usage);
        let count_readback = (!uses_mappable_primary_buffers(context))
            .then(|| readback_buffer(context, "GPU range count readback", 8));
        Self {
            start_mask,
            end_mask,
            starts,
            ends,
            start_count,
            end_count,
            count_readback,
        }
    }
}

struct BoundaryKernel {
    pipeline: wgpu::ComputePipeline,
    bind_group: wgpu::BindGroup,
    groups_x: u32,
    groups_y: u32,
}

impl BoundaryKernel {
    fn new_32(
        context: &Context,
        values: &wgpu::Buffer,
        buffers: &BoundaryBuffers,
        parameters: &[u32; 4],
    ) -> Self {
        Self::new(
            context,
            BOUNDARY_32_SHADER,
            &[
                (values, true),
                (&buffers.start_mask, false),
                (&buffers.end_mask, false),
            ],
            parameters,
        )
    }

    fn new_64(
        context: &Context,
        hi: &wgpu::Buffer,
        lo: &wgpu::Buffer,
        pairs: &wgpu::Buffer,
        buffers: &BoundaryBuffers,
        parameters: &[u32; 8],
    ) -> Self {
        Self::new(
            context,
            BOUNDARY_64_SHADER,
            &[
                (hi, true),
                (lo, true),
                (pairs, false),
                (&buffers.start_mask, false),
                (&buffers.end_mask, false),
            ],
            parameters,
        )
    }

    fn new(
        context: &Context,
        source: &'static str,
        storage: &[(&wgpu::Buffer, bool)],
        parameters: &[u32],
    ) -> Self {
        let parameter_buffer = context.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("GPU range boundary parameters"),
            size: size_of_val(parameters) as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        context
            .queue
            .write_buffer(&parameter_buffer, 0, bytemuck::cast_slice(parameters));
        let mut layout_entries: Vec<_> = storage
            .iter()
            .enumerate()
            .map(|(binding, (_, read_only))| {
                storage_layout_entry(
                    u32::try_from(binding).expect("too many bindings"),
                    *read_only,
                )
            })
            .collect();
        let uniform_binding = u32::try_from(storage.len()).expect("too many bindings");
        layout_entries.push(uniform_layout_entry(uniform_binding));
        let layout = context
            .device
            .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("GPU range boundary layout"),
                entries: &layout_entries,
            });
        let shader = context
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("GPU range boundary shader"),
                source: wgpu::ShaderSource::Wgsl(source.into()),
            });
        let pipeline_layout =
            context
                .device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("GPU range boundary pipeline layout"),
                    bind_group_layouts: &[Some(&layout)],
                    immediate_size: 0,
                });
        let pipeline = context
            .device
            .create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: Some("GPU range boundary pipeline"),
                layout: Some(&pipeline_layout),
                module: &shader,
                entry_point: Some("main"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                cache: None,
            });
        let mut entries: Vec<_> = storage
            .iter()
            .enumerate()
            .map(|(binding, (buffer, _))| {
                entire_buffer_entry(u32::try_from(binding).expect("too many bindings"), buffer)
            })
            .collect();
        entries.push(entire_buffer_entry(uniform_binding, &parameter_buffer));
        let bind_group = context
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("GPU range boundary bind group"),
                layout: &layout,
                entries: &entries,
            });
        let len = parameters[0];
        let groups_x = parameters[1];
        let groups_y = len.div_ceil(WORKGROUP_SIZE).div_ceil(groups_x);
        Self {
            pipeline,
            bind_group,
            groups_x,
            groups_y,
        }
    }

    fn record(&self, encoder: &mut wgpu::CommandEncoder) {
        let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("Mark GPU range boundaries"),
            timestamp_writes: None,
        });
        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, &self.bind_group, &[]);
        pass.dispatch_workgroups(self.groups_x, self.groups_y, 1);
    }
}

fn groups_x(context: &Context, len: u32) -> Option<u32> {
    let total = len.div_ceil(WORKGROUP_SIZE);
    let maximum = context.device.limits().max_compute_workgroups_per_dimension;
    let x = total.min(maximum);
    (total.div_ceil(maximum) <= maximum).then_some(x)
}

fn read_counts(context: &Context, buffers: &BoundaryBuffers) -> Option<usize> {
    let counts = buffers.count_readback.as_ref().map_or_else(
        || {
            let (starts, ends) =
                map_two_u32(context, &buffers.start_count, 1, &buffers.end_count, 1)?;
            Some([starts[0], ends[0]].to_vec())
        },
        |readback| {
            let mut encoder = encoder(context, "GPU range count copy");
            encoder.copy_buffer_to_buffer(&buffers.start_count, 0, readback, 0, 4);
            encoder.copy_buffer_to_buffer(&buffers.end_count, 0, readback, 4, 4);
            submit(context, encoder.finish())?;
            map_u32(context, readback, 2)
        },
    )?;
    assert_eq!(
        counts[0], counts[1],
        "GPU emitted unpaired range boundaries"
    );
    Some(counts[0] as usize)
}

fn read_endpoints_32(
    context: &Context,
    buffers: &BoundaryBuffers,
    len: u32,
) -> Option<Vec<(u64, u64)>> {
    let count = read_counts(context, buffers)?;
    assert!(count <= len as usize, "GPU emitted too many ranges");
    if count == 0 {
        return Some(Vec::new());
    }
    let endpoint_bytes = count as u64 * 4;
    let (starts, ends) = if buffers.count_readback.is_none() {
        map_two_u32(context, &buffers.starts, count, &buffers.ends, count)?
    } else {
        let readback = readback_buffer(context, "GPU range endpoint readback", endpoint_bytes * 2);
        let mut encoder = encoder(context, "GPU range endpoint copy");
        encoder.copy_buffer_to_buffer(&buffers.starts, 0, &readback, 0, endpoint_bytes);
        encoder.copy_buffer_to_buffer(&buffers.ends, 0, &readback, endpoint_bytes, endpoint_bytes);
        submit(context, encoder.finish())?;
        let words = map_u32(context, &readback, count * 2)?;
        (words[..count].to_vec(), words[count..].to_vec())
    };
    Some(
        starts
            .iter()
            .zip(&ends)
            .map(|(&start, &end)| (u64::from(start), u64::from(end)))
            .collect(),
    )
}

fn read_endpoints_64(
    context: &Context,
    buffers: &BoundaryBuffers,
    len: u32,
) -> Option<Vec<(u64, u64)>> {
    let count = read_counts(context, buffers)?;
    assert!(count <= len as usize, "GPU emitted too many ranges");
    if count == 0 {
        return Some(Vec::new());
    }
    let endpoint_bytes = count as u64 * 8;
    let (starts, ends) = if buffers.count_readback.is_none() {
        map_two_u32(
            context,
            &buffers.starts,
            count * 2,
            &buffers.ends,
            count * 2,
        )?
    } else {
        let readback = readback_buffer(context, "GPU range endpoint readback", endpoint_bytes * 2);
        let mut encoder = encoder(context, "GPU range endpoint copy");
        encoder.copy_buffer_to_buffer(&buffers.starts, 0, &readback, 0, endpoint_bytes);
        encoder.copy_buffer_to_buffer(&buffers.ends, 0, &readback, endpoint_bytes, endpoint_bytes);
        submit(context, encoder.finish())?;
        let words = map_u32(context, &readback, count * 4)?;
        (words[..count * 2].to_vec(), words[count * 2..].to_vec())
    };
    Some(
        starts
            .as_chunks::<2>()
            .0
            .iter()
            .zip(ends.as_chunks::<2>().0)
            .map(|(&[start_hi, start_lo], &[end_hi, end_lo])| {
                (join_words(start_hi, start_lo), join_words(end_hi, end_lo))
            })
            .collect(),
    )
}

fn join_words(hi: u32, lo: u32) -> u64 {
    (u64::from(hi) << 32) | u64::from(lo)
}

fn high_word(value: u64) -> u32 {
    u32::try_from(value >> 32).expect("shifted 64-bit key exceeded u32")
}

fn low_word(value: u64) -> u32 {
    u32::try_from(value & u64::from(u32::MAX)).expect("masked 64-bit key exceeded u32")
}

fn buffers_fit(context: &Context, largest_storage: u64, largest_readback: u64) -> bool {
    let limits = context.device.limits();
    let largest_buffer = if uses_mappable_primary_buffers(context) {
        largest_storage
    } else {
        largest_readback
    };
    largest_storage <= limits.max_storage_buffer_binding_size
        && largest_storage <= limits.max_buffer_size
        && largest_buffer <= limits.max_buffer_size
}

fn resource_or_panic(result: Result<(), lampshade::Error>) -> Option<()> {
    match result {
        Ok(()) => Some(()),
        Err(
            lampshade::Error::ElementCountTooLarge { .. }
            | lampshade::Error::RadixElementCountLimitExceeded { .. }
            | lampshade::Error::SizeOverflow
            | lampshade::Error::BufferLimitExceeded { .. },
        ) => None,
        Err(error) => panic!("GPU range implementation failure: {error}"),
    }
}

std::thread_local! {
    /// The first GPU failure seen on this thread during the current call, if any.
    static GPU_FAILURE: RefCell<Option<std::string::String>> = const { RefCell::new(None) };
}

fn record_gpu_failure(what: &str, error: &impl core::fmt::Display) {
    GPU_FAILURE.with_borrow_mut(|failure| {
        failure.get_or_insert_with(|| std::format!("{what}: {error}"));
    });
}

fn submit(context: &Context, command: wgpu::CommandBuffer) -> Option<()> {
    let submission = context.queue.submit(Some(command));
    context
        .device
        .poll(wgpu::PollType::Wait {
            submission_index: Some(submission),
            timeout: None,
        })
        .map_err(|error| record_gpu_failure("GPU range execution failed while waiting", &error))
        .ok()?;
    Some(())
}

fn map_u32(context: &Context, buffer: &wgpu::Buffer, count: usize) -> Option<Vec<u32>> {
    let slice = buffer.slice(..count as u64 * 4);
    let (sender, receiver) = mpsc::channel();
    slice.map_async(wgpu::MapMode::Read, move |result| {
        drop(sender.send(result));
    });
    context
        .device
        .poll(wgpu::PollType::Wait {
            submission_index: None,
            timeout: None,
        })
        .map_err(|error| record_gpu_failure("GPU range execution failed while mapping", &error))
        .ok()?;
    receiver
        .recv()
        .map_err(|error| record_gpu_failure("GPU range readback channel failed", &error))
        .ok()?
        .map_err(|error| record_gpu_failure("GPU range readback mapping failed", &error))
        .ok()?;
    let result = {
        let mapped = slice
            .get_mapped_range()
            .map_err(|error| record_gpu_failure("GPU range readback access failed", &error))
            .ok()?;
        bytemuck::cast_slice::<u8, u32>(&mapped).to_vec()
    };
    buffer.unmap();
    Some(result)
}

fn map_two_u32(
    context: &Context,
    first: &wgpu::Buffer,
    first_count: usize,
    second: &wgpu::Buffer,
    second_count: usize,
) -> Option<(Vec<u32>, Vec<u32>)> {
    let first_slice = first.slice(..first_count as u64 * 4);
    let second_slice = second.slice(..second_count as u64 * 4);
    let (first_sender, first_receiver) = mpsc::channel();
    let (second_sender, second_receiver) = mpsc::channel();
    first_slice.map_async(wgpu::MapMode::Read, move |result| {
        drop(first_sender.send(result));
    });
    second_slice.map_async(wgpu::MapMode::Read, move |result| {
        drop(second_sender.send(result));
    });
    context
        .device
        .poll(wgpu::PollType::Wait {
            submission_index: None,
            timeout: None,
        })
        .map_err(|error| record_gpu_failure("GPU range execution failed while mapping", &error))
        .ok()?;
    first_receiver
        .recv()
        .map_err(|error| record_gpu_failure("GPU range readback channel failed", &error))
        .ok()?
        .map_err(|error| record_gpu_failure("GPU range readback mapping failed", &error))
        .ok()?;
    second_receiver
        .recv()
        .map_err(|error| record_gpu_failure("GPU range readback channel failed", &error))
        .ok()?
        .map_err(|error| record_gpu_failure("GPU range readback mapping failed", &error))
        .ok()?;
    let first_words = {
        let mapped = first_slice
            .get_mapped_range()
            .map_err(|error| record_gpu_failure("GPU range readback access failed", &error))
            .ok()?;
        bytemuck::cast_slice::<u8, u32>(&mapped).to_vec()
    };
    let second_words = {
        let mapped = second_slice
            .get_mapped_range()
            .map_err(|error| record_gpu_failure("GPU range readback access failed", &error))
            .ok()?;
        bytemuck::cast_slice::<u8, u32>(&mapped).to_vec()
    };
    first.unmap();
    second.unmap();
    Some((first_words, second_words))
}

fn input_storage_buffer(
    context: &Context,
    label: &'static str,
    size: u64,
    words: impl ExactSizeIterator<Item = u32>,
) -> Option<wgpu::Buffer> {
    if uses_mappable_primary_buffers(context) {
        assert_eq!(words.len() as u64 * 4, size, "GPU input size mismatch");
        let buffer = context.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some(label),
            size,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::MAP_WRITE,
            mapped_at_creation: true,
        });
        {
            let mut mapped = buffer
                .get_mapped_range_mut(..)
                .map_err(|error| record_gpu_failure("GPU input mapping failed", &error))
                .ok()?;
            let (destinations, remainder) = mapped.slice(..).into_chunks::<4>();
            assert!(remainder.is_empty(), "GPU input was not word aligned");
            destinations.write_iter(words.map(u32::to_ne_bytes));
        }
        buffer.unmap();
        Some(buffer)
    } else {
        let words: Vec<_> = words.collect();
        let buffer = storage_buffer(context, label, size, wgpu::BufferUsages::COPY_DST);
        context
            .queue
            .write_buffer(&buffer, 0, bytemuck::cast_slice(&words));
        Some(buffer)
    }
}

/// Like [`input_storage_buffer`], but from a slice, which the queue can copy from directly.
fn slice_storage_buffer(
    context: &Context,
    label: &'static str,
    words: &[u32],
) -> Option<wgpu::Buffer> {
    let size = size_of_val(words) as u64;
    if uses_mappable_primary_buffers(context) {
        return input_storage_buffer(context, label, size, words.iter().copied());
    }
    let buffer = storage_buffer(context, label, size, wgpu::BufferUsages::COPY_DST);
    context
        .queue
        .write_buffer(&buffer, 0, bytemuck::cast_slice(words));
    Some(buffer)
}

fn storage_buffer(
    context: &Context,
    label: &'static str,
    size: u64,
    extra: wgpu::BufferUsages,
) -> wgpu::Buffer {
    context.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some(label),
        size,
        usage: wgpu::BufferUsages::STORAGE | extra,
        mapped_at_creation: false,
    })
}

fn readback_buffer(context: &Context, label: &'static str, size: u64) -> wgpu::Buffer {
    context.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some(label),
        size,
        usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    })
}

fn encoder(context: &Context, label: &'static str) -> wgpu::CommandEncoder {
    context
        .device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some(label) })
}

const fn storage_layout_entry(binding: u32, read_only: bool) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
        binding,
        visibility: wgpu::ShaderStages::COMPUTE,
        ty: wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Storage { read_only },
            has_dynamic_offset: false,
            min_binding_size: None,
        },
        count: None,
    }
}

const fn uniform_layout_entry(binding: u32) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
        binding,
        visibility: wgpu::ShaderStages::COMPUTE,
        ty: wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Uniform,
            has_dynamic_offset: false,
            min_binding_size: None,
        },
        count: None,
    }
}

fn entire_buffer_entry(binding: u32, buffer: &wgpu::Buffer) -> wgpu::BindGroupEntry<'_> {
    wgpu::BindGroupEntry {
        binding,
        resource: buffer.as_entire_binding(),
    }
}

macro_rules! gpu_api {
    ($($type:ty),+ $(,)?) => {$($crate::gpu::gpu_api_one!($type);)+};
}

macro_rules! gpu_api_one {
    ($type:ty) => {
        impl RangeSetBlaze<$type> {
            /// Creates a [`RangeSetBlaze`] from a slice, using several CPU threads or the GPU,
            /// as `engine` says. It may reorder the slice; the order afterward is unspecified.
            ///
            /// *Experimental; requires the `shuffled_slices` feature.* It pays off for large slices
            /// whose values are in no useful order, such as shuffled data. If nearby values are
            /// already next to each other (sorted, or clumpy data in order), use
            /// [`RangeSetBlaze::from_slice`], which will be much faster.
            ///
            /// See [`Engine`] for the choices, including when [`Engine::Wgpu`] panics.
            #[cfg_attr(docsrs, doc(cfg(feature = "shuffled_slices")))]
            #[must_use]
            pub fn from_shuffled_slice(slice: &mut [$type], engine: Engine) -> Self {
                from_shuffled_slice(slice, engine)
            }
        }
    };
}

pub(crate) use gpu_api_one;

macro_rules! unsigned_gpu {
    ($type:ty, $words:expr) => {
        impl GpuElement for $type {
            const KEY_WORDS: u8 = $words;
            fn encode(self) -> u64 {
                u64::try_from(self).expect("GPU key exceeded u64")
            }
            fn decode(key: u64) -> Self {
                Self::try_from(key).expect("GPU key did not fit its source type")
            }
        }
    };
}

macro_rules! signed_gpu {
    ($type:ty, $unsigned:ty, $sign:expr, $words:expr) => {
        impl GpuElement for $type {
            const KEY_WORDS: u8 = $words;
            fn encode(self) -> u64 {
                u64::try_from(self.cast_unsigned() ^ $sign).expect("GPU key exceeded u64")
            }
            fn decode(key: u64) -> Self {
                (<$unsigned>::try_from(key).expect("GPU key did not fit its source type") ^ $sign)
                    .cast_signed()
            }
        }
    };
}

macro_rules! cpu_only {
    ($($type:ty),+ $(,)?) => {$(
        impl GpuElement for $type {
            const KEY_WORDS: u8 = 0;
            fn encode(self) -> u64 { unreachable!("CPU-only GPU element") }
            fn decode(_key: u64) -> Self { unreachable!("CPU-only GPU element") }
        }
    )+};
}

unsigned_gpu!(u8, 1);
unsigned_gpu!(u16, 1);
unsigned_gpu!(u32, 1);
unsigned_gpu!(u64, 2);
signed_gpu!(i8, u8, 0x80, 1);
signed_gpu!(i16, u16, 0x8000, 1);
signed_gpu!(i32, u32, 0x8000_0000, 1);
signed_gpu!(i64, u64, 0x8000_0000_0000_0000, 2);

#[cfg(target_pointer_width = "32")]
unsigned_gpu!(usize, 1);
#[cfg(target_pointer_width = "64")]
unsigned_gpu!(usize, 2);
#[cfg(target_pointer_width = "32")]
signed_gpu!(isize, usize, 0x8000_0000, 1);
#[cfg(target_pointer_width = "64")]
signed_gpu!(isize, usize, 0x8000_0000_0000_0000, 2);

impl GpuElement for char {
    const KEY_WORDS: u8 = 1;
    const GAP: Option<(u64, u64)> = Some((0xd7ff, 0xe000));
    fn encode(self) -> u64 {
        u64::from(u32::from(self))
    }
    fn decode(key: u64) -> Self {
        Self::from_u32(u32::try_from(key).expect("char GPU key exceeded u32"))
            .expect("GPU returned an invalid char")
    }
}

impl GpuElement for Ipv4Addr {
    const KEY_WORDS: u8 = 1;
    fn encode(self) -> u64 {
        u64::from(u32::from(self))
    }
    fn decode(key: u64) -> Self {
        Self::from(u32::try_from(key).expect("IPv4 GPU key exceeded u32"))
    }
}

impl GpuElement for NotNanF32 {
    const KEY_WORDS: u8 = 1;
    const GAP: Option<(u64, u64)> = Some((0x7fff_fffe, 0x8000_0000));
    fn encode(self) -> u64 {
        u64::from(gpu_not_nan_f32_to_ordered(self.into_inner()).cast_unsigned() ^ 0x8000_0000)
    }
    fn decode(key: u64) -> Self {
        Self::new(gpu_not_nan_f32_from_ordered(
            (u32::try_from(key).expect("f32 GPU key exceeded u32") ^ 0x8000_0000).cast_signed(),
        ))
    }
}

impl GpuElement for NotNanF64 {
    const KEY_WORDS: u8 = 2;
    const GAP: Option<(u64, u64)> = Some((0x7fff_ffff_ffff_fffe, 0x8000_0000_0000_0000));
    fn encode(self) -> u64 {
        gpu_not_nan_f64_to_ordered(self.into_inner()).cast_unsigned() ^ 0x8000_0000_0000_0000
    }
    fn decode(key: u64) -> Self {
        Self::new(gpu_not_nan_f64_from_ordered(
            (key ^ 0x8000_0000_0000_0000).cast_signed(),
        ))
    }
}

cpu_only!(u128, i128, Ipv6Addr, TotalF32, TotalF64);
#[cfg(feature = "float_nightly_experimental")]
cpu_only!(NotNanF128, TotalF16, TotalF128);

#[cfg(feature = "float_nightly_experimental")]
impl GpuElement for NotNanF16 {
    const KEY_WORDS: u8 = 1;
    const GAP: Option<(u64, u64)> = Some((0x7ffe, 0x8000));
    fn encode(self) -> u64 {
        u64::from(gpu_not_nan_f16_to_ordered(self.into_inner()).cast_unsigned() ^ 0x8000)
    }
    fn decode(key: u64) -> Self {
        Self::new(gpu_not_nan_f16_from_ordered(
            (u16::try_from(key).expect("f16 GPU key exceeded u16") ^ 0x8000).cast_signed(),
        ))
    }
}

gpu_api!(
    u8, u16, u32, u64, u128, i8, i16, i32, i64, i128, usize, isize, char, Ipv4Addr, Ipv6Addr,
    NotNanF32, NotNanF64, TotalF32, TotalF64
);
#[cfg(feature = "float_nightly_experimental")]
gpu_api!(NotNanF16, NotNanF128, TotalF16, TotalF128);

#[cfg(test)]
mod tests {
    use super::*;
    #[cfg(feature = "float_nightly_experimental")]
    use crate::not_nan::nnf16;
    use crate::not_nan::{nnf32, nnf64};

    #[test]
    fn mapped_primary_policy_requires_shared_memory_and_feature_support() {
        let feature = wgpu::Features::MAPPABLE_PRIMARY_BUFFERS;
        assert!(should_use_mappable_primary_buffers(
            wgpu::DeviceType::IntegratedGpu,
            feature,
        ));
        assert!(!should_use_mappable_primary_buffers(
            wgpu::DeviceType::DiscreteGpu,
            feature,
        ));
        assert!(!should_use_mappable_primary_buffers(
            wgpu::DeviceType::IntegratedGpu,
            wgpu::Features::empty(),
        ));
    }

    fn check<T: GpuElement>(values: &[T]) {
        let expected: RangeSetBlaze<T> = values.iter().copied().collect();
        let Some(context) = test_context() else {
            return;
        };
        let Some(actual) = from_slice_gpu_with_context(values, context) else {
            return;
        };
        assert_eq!(
            actual.ranges().collect::<Vec<_>>(),
            expected.ranges().collect::<Vec<_>>()
        );
        assert_eq!(actual, expected);
    }

    #[test]
    fn exact_32_bit_families() {
        check(&[3u8, 1, 2, 2, 0, u8::MAX]);
        check(&[u16::MAX, 1, 0, 2, 2]);
        check(&[u32::MAX, 1, 0, 2, 2]);
        check(&[i8::MAX, 0, -1, i8::MIN, 1]);
        check(&[i16::MAX, 0, -1, i16::MIN, 1]);
        check(&[i32::MAX, 0, -1, i32::MIN, 1]);
        check(&[usize::MAX, 0, 1, 2, 2]);
        check(&[isize::MAX, 0, -1, isize::MIN, 1]);
        check(&['\u{e001}', '\u{d7ff}', '\u{e000}', '\u{0}', '\u{10ffff}']);
        check(&[
            Ipv4Addr::BROADCAST,
            Ipv4Addr::new(10, 0, 0, 2),
            Ipv4Addr::new(10, 0, 0, 1),
            Ipv4Addr::new(10, 0, 0, 255),
            Ipv4Addr::new(10, 0, 1, 0),
            Ipv4Addr::UNSPECIFIED,
        ]);
        check(&[
            nnf32(f32::INFINITY),
            nnf32(1.0),
            nnf32(0.0),
            nnf32(-f32::from_bits(1)),
            nnf32(-1.0),
            nnf32(f32::NEG_INFINITY),
        ]);
    }

    #[test]
    fn exact_64_bit_families() {
        check(&[
            0u64,
            u64::from(u32::MAX),
            u64::from(u32::MAX) + 1,
            u64::MAX,
            2,
            2,
        ]);
        check(&[i64::MIN, -1, 0, 1, i64::MAX]);
        check(&[
            nnf64(f64::INFINITY),
            nnf64(1.0),
            nnf64(0.0),
            nnf64(-f64::from_bits(1)),
            nnf64(-1.0),
            nnf64(f64::NEG_INFINITY),
        ]);
    }

    #[cfg(feature = "float_nightly_experimental")]
    #[test]
    fn exact_f16_family() {
        check(&[
            nnf16(f16::INFINITY),
            nnf16(1.0),
            nnf16(0.0),
            nnf16(-f16::from_bits(1)),
            nnf16(-1.0),
            nnf16(f16::NEG_INFINITY),
        ]);
    }

    #[test]
    fn randomized_exact_comparisons() {
        let mut state = 0x9e37_79b9_7f4a_7c15u64;
        let mut values = Vec::with_capacity(4_096);
        for _ in 0..4_096 {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            values.push(state);
        }
        check_random_small_integers(&values);
        check_random_large_integers(&values);
        check_random_special_types(&values);
    }

    fn check_random_small_integers(values: &[u64]) {
        check(
            &values
                .iter()
                .map(|&value| u8::try_from(value & u64::from(u8::MAX)).expect("masked to u8"))
                .collect::<Vec<_>>(),
        );
        check(
            &values
                .iter()
                .map(|&value| u16::try_from(value & u64::from(u16::MAX)).expect("masked to u16"))
                .collect::<Vec<_>>(),
        );
        check(
            &values
                .iter()
                .map(|&value| low_word(value))
                .collect::<Vec<_>>(),
        );
        check(
            &values
                .iter()
                .map(|&value| {
                    u8::try_from(value & u64::from(u8::MAX))
                        .expect("masked to u8")
                        .cast_signed()
                })
                .collect::<Vec<_>>(),
        );
        check(
            &values
                .iter()
                .map(|&value| {
                    u16::try_from(value & u64::from(u16::MAX))
                        .expect("masked to u16")
                        .cast_signed()
                })
                .collect::<Vec<_>>(),
        );
        check(
            &values
                .iter()
                .map(|&value| low_word(value).cast_signed())
                .collect::<Vec<_>>(),
        );
    }

    fn check_random_large_integers(values: &[u64]) {
        check(values);
        check(
            &values
                .iter()
                .map(|&value| value.cast_signed())
                .collect::<Vec<_>>(),
        );
    }

    fn check_random_special_types(values: &[u64]) {
        check(
            &values
                .iter()
                .map(|&value| {
                    usize::try_from(value).unwrap_or_else(|_| {
                        usize::try_from(low_word(value)).expect("u32 should fit usize")
                    })
                })
                .collect::<Vec<_>>(),
        );
        check(
            &values
                .iter()
                .map(|&value| {
                    usize::try_from(value)
                        .unwrap_or_else(|_| {
                            usize::try_from(low_word(value)).expect("u32 should fit usize")
                        })
                        .cast_signed()
                })
                .collect::<Vec<_>>(),
        );
        check(
            &values
                .iter()
                .filter_map(|&value| char::from_u32(low_word(value) % 0x11_0000))
                .collect::<Vec<_>>(),
        );
        check(
            &values
                .iter()
                .map(|&value| Ipv4Addr::from(low_word(value)))
                .collect::<Vec<_>>(),
        );
        check(
            &values
                .iter()
                .filter_map(|&value| NotNanF32::try_new(f32::from_bits(low_word(value))))
                .collect::<Vec<_>>(),
        );
        check(
            &values
                .iter()
                .filter_map(|&value| NotNanF64::try_new(f64::from_bits(value)))
                .collect::<Vec<_>>(),
        );
    }

    /// CI sets `RANGE_SET_BLAZE_REQUIRE_TEST_GPU` where a software Vulkan driver is installed, so
    /// the GPU tests can't pass there by silently skipping.
    #[test]
    fn test_gpu_is_available_when_required() {
        if std::env::var_os("RANGE_SET_BLAZE_REQUIRE_TEST_GPU").is_some() {
            let context = test_context()
                .expect("RANGE_SET_BLAZE_REQUIRE_TEST_GPU is set, but no test GPU was found");
            std::println!(
                "test GPU: {:?} ({:?}, {:?})",
                context.adapter_info.name,
                context.adapter_info.backend,
                context.adapter_info.device_type
            );
        }
    }

    /// Real use never takes a software adapter, even where tests do.
    #[test]
    fn real_use_rejects_software_adapters() {
        if test_context()
            .is_some_and(|context| context.adapter_info.device_type == wgpu::DeviceType::Cpu)
        {
            assert!(gpu_context().is_none());
        }
    }

    fn has_hardware_gpu() -> bool {
        gpu_context()
            .is_some_and(|context| context.adapter_info.device_type != wgpu::DeviceType::Cpu)
    }

    /// Every engine that can run here, for the type `T`.
    fn engines(gpu_capable: bool) -> Vec<Engine> {
        let mut engines = std::vec![
            Engine::Cpu(NonZeroUsize::MIN),
            Engine::Cpu(NonZeroUsize::new(3).expect("3 is nonzero")),
            Engine::WgpuOrCpu,
        ];
        if gpu_capable && has_hardware_gpu() {
            engines.push(Engine::Wgpu);
        }
        #[cfg(feature = "cuda")]
        if gpu_capable && cuda::cuda_context().is_some() {
            engines.push(Engine::Cuda);
        }
        #[cfg(feature = "cuda")]
        engines.push(Engine::CudaOrCpu);
        engines
    }

    #[test]
    fn shuffled_handles_empty_and_small_inputs() {
        for engine in engines(true) {
            assert!(RangeSetBlaze::<u32>::from_shuffled_slice(&mut [], engine).is_empty());
            assert_eq!(
                RangeSetBlaze::<u32>::from_shuffled_slice(&mut [3, 1, 2, 2], engine),
                RangeSetBlaze::from_iter([1..=3]),
                "{engine:?}"
            );
        }
        // u128 has no GPU representation, so Wgpu is excluded.
        for engine in engines(false) {
            assert_eq!(
                RangeSetBlaze::<u128>::from_shuffled_slice(&mut [3, 1, 2, 2], engine),
                RangeSetBlaze::from_iter([1..=3]),
                "{engine:?}"
            );
        }
    }

    #[test]
    #[should_panic(expected = "no GPU representation")]
    fn wgpu_panics_for_types_without_a_gpu_representation() {
        let _ = RangeSetBlaze::<u128>::from_shuffled_slice(&mut [1, 2], Engine::Wgpu);
    }

    /// The CPU engine on float keys: negatives, subnormals on both sides of the `-0.0` gap, zero,
    /// and the infinities. Afterward the slice must still hold the same values.
    #[test]
    fn cpu_float_keys_match_from_slice() {
        let f32s: Vec<f32> = std::vec![
            f32::INFINITY,
            -f32::from_bits(1),
            3.0,
            -0.0,
            f32::from_bits(1),
            f32::NEG_INFINITY,
            -2.0,
            -f32::from_bits(2),
            0.0,
            2.0,
            f32::from_bits(2),
            -1.0,
            f32::MAX,
            -f32::MAX,
            1.0,
        ];
        let f64s: Vec<f64> = f32s.iter().map(|&x| f64::from(x)).collect();
        // Under Miri, only one thread: Rayon's `crossbeam-epoch` trips Miri's aliasing model on
        // its own.
        let thread_counts: &[usize] = if cfg!(miri) { &[1] } else { &[1, 3] };
        for &threads in thread_counts {
            let threads = NonZeroUsize::new(threads).expect("nonzero");
            let mut values: Vec<NotNanF32> = f32s.iter().map(|&x| nnf32(x)).collect();
            let expected = RangeSetBlaze::from_slice(&values);
            let mut sorted_before = values.clone();
            sorted_before.sort_unstable();
            assert_eq!(
                RangeSetBlaze::<NotNanF32>::from_shuffled_slice(&mut values, Engine::Cpu(threads)),
                expected
            );
            values.sort_unstable();
            assert_eq!(values, sorted_before, "the slice must hold the same values");

            let mut values: Vec<NotNanF64> = f64s.iter().map(|&x| nnf64(x)).collect();
            let expected = RangeSetBlaze::from_slice(&values);
            let mut sorted_before = values.clone();
            sorted_before.sort_unstable();
            assert_eq!(
                RangeSetBlaze::<NotNanF64>::from_shuffled_slice(&mut values, Engine::Cpu(threads)),
                expected
            );
            values.sort_unstable();
            assert_eq!(values, sorted_before, "the slice must hold the same values");
        }
        // Subnormals either side of the gap join across it: {-min, 0, +min} is one range.
        let mut values = std::vec![
            nnf32(f32::from_bits(1)),
            nnf32(-f32::from_bits(1)),
            nnf32(0.0)
        ];
        let set = RangeSetBlaze::<NotNanF32>::from_shuffled_slice(
            &mut values,
            Engine::Cpu(NonZeroUsize::MIN),
        );
        assert_eq!(set.ranges_len(), 1);
    }

    /// Every engine matches `from_slice` on shuffled clumpy data, for sizes around the GPU
    /// work-group size (256).
    #[test]
    fn shuffled_matches_from_slice_at_many_sizes() {
        let seed = 0x2545_f491_4f6c_dd1du64;
        for len in [1usize, 2, 3, 255, 256, 257, 1_023, 4_097, 65_537] {
            let mut state = seed ^ len as u64;
            let mut values: Vec<u32> = (0..len)
                .map(|index| u32::try_from(index + index / 7).expect("small test index"))
                .collect();
            for index in (1..values.len()).rev() {
                state ^= state << 13;
                state ^= state >> 7;
                state ^= state << 17;
                let swap = usize::try_from(state % (index as u64 + 1)).expect("small swap index");
                values.swap(index, swap);
            }
            let expected = RangeSetBlaze::from_slice(&values);
            for engine in engines(true) {
                let mut copy = values.clone();
                assert_eq!(
                    RangeSetBlaze::<u32>::from_shuffled_slice(&mut copy, engine),
                    expected,
                    "len={len} {engine:?}"
                );
            }
        }
    }
}
