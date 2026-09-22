//! Portable GPU-assisted construction for large unsorted slices.

use core::mem::size_of;
use core::net::{Ipv4Addr, Ipv6Addr};
use std::sync::{OnceLock, mpsc};
use std::vec::Vec;

use lampshade::{Compactor, Context, KeyValueCompactor, KeyValueSoaSorter, Sorter};

#[cfg(feature = "float_nightly_experimental")]
use crate::float::{gpu_not_nan_f16_from_ordered, gpu_not_nan_f16_to_ordered};
use crate::float::{
    gpu_not_nan_f32_from_ordered, gpu_not_nan_f32_to_ordered, gpu_not_nan_f64_from_ordered,
    gpu_not_nan_f64_to_ordered,
};
use crate::{CheckSortedDisjoint, NotNanF32, NotNanF64, RangeSetBlaze, TotalF32, TotalF64};
#[cfg(feature = "float_nightly_experimental")]
use crate::{NotNanF16, NotNanF128, TotalF16, TotalF128};

const WORKGROUP_SIZE: u32 = 256;
// The existing measurements show a decisive end-to-end win at one million
// values across the useful compression distributions. Keep the initial policy
// conservative while adapter-to-adapter data accumulates.
const GPU_CROSSOVER: usize = 1_000_000;

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
// enabling `gpu` does not break downstream `Integer` implementations.
trait GpuElement: crate::Integer {
    const KEY_WORDS: u8;
    const GAP: Option<(u64, u64)> = None;

    fn encode(self) -> u64;
    fn decode(key: u64) -> Self;
}

fn from_slice<T: GpuElement>(values: &[T]) -> RangeSetBlaze<T> {
    if values.len() < GPU_CROSSOVER || T::KEY_WORDS == 0 {
        return values.iter().copied().collect();
    }
    from_slice_gpu(values).unwrap_or_else(|| values.iter().copied().collect())
}

fn from_slice_gpu<T: GpuElement>(values: &[T]) -> Option<RangeSetBlaze<T>> {
    let context = gpu_context()?;
    (context.adapter_info.device_type != wgpu::DeviceType::Cpu)
        .then(|| from_slice_gpu_with_context(values, context))?
}

fn from_slice_gpu_with_context<T: GpuElement>(
    values: &[T],
    context: &Context,
) -> Option<RangeSetBlaze<T>> {
    if values.is_empty() {
        return Some(RangeSetBlaze::new());
    }
    let len = u32::try_from(values.len()).ok()?;
    let endpoints = match T::KEY_WORDS {
        1 => normalize_32(context, values, len, T::GAP)?,
        2 => normalize_64(context, values, len, T::GAP)?,
        _ => return None,
    };
    let ranges = endpoints
        .into_iter()
        .map(|(start, end)| T::decode(start)..=T::decode(end));
    Some(RangeSetBlaze::from_sorted_disjoint(
        CheckSortedDisjoint::new(ranges),
    ))
}

fn gpu_context() -> Option<&'static Context> {
    static CONTEXT: OnceLock<Option<Context>> = OnceLock::new();
    CONTEXT
        .get_or_init(|| pollster::block_on(Context::init()).ok())
        .as_ref()
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
    let input_values: Vec<u32> = values
        .iter()
        .copied()
        .map(|value| u32::try_from(value.encode()).expect("32-bit GPU key exceeded u32"))
        .collect();
    let input = storage_buffer(
        context,
        "GPU range input",
        byte_len,
        wgpu::BufferUsages::COPY_DST,
    );
    let sorted = storage_buffer(
        context,
        "GPU range sorted",
        byte_len,
        wgpu::BufferUsages::empty(),
    );
    context
        .queue
        .write_buffer(&input, 0, bytemuck::cast_slice(&input_values));
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
    submit(context, encoder.finish());
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
    let low_values: Vec<u32> = values
        .iter()
        .copied()
        .map(|value| low_word(value.encode()))
        .collect();
    let high_values: Vec<u32> = values
        .iter()
        .copied()
        .map(|value| high_word(value.encode()))
        .collect();
    let lo = storage_buffer(
        context,
        "GPU range low words",
        word_bytes,
        wgpu::BufferUsages::COPY_DST,
    );
    let hi = storage_buffer(
        context,
        "GPU range high words",
        word_bytes,
        wgpu::BufferUsages::COPY_DST,
    );
    context
        .queue
        .write_buffer(&lo, 0, bytemuck::cast_slice(&low_values));
    context
        .queue
        .write_buffer(&hi, 0, bytemuck::cast_slice(&high_values));
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
    submit(context, encoder.finish());
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
    count_readback: wgpu::Buffer,
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
        let starts = storage_buffer(
            context,
            "GPU range starts",
            endpoint_bytes,
            wgpu::BufferUsages::COPY_SRC,
        );
        let ends = storage_buffer(
            context,
            "GPU range ends",
            endpoint_bytes,
            wgpu::BufferUsages::COPY_SRC,
        );
        let count_usage = wgpu::BufferUsages::COPY_SRC | wgpu::BufferUsages::COPY_DST;
        let start_count = storage_buffer(context, "GPU range start count", 4, count_usage);
        let end_count = storage_buffer(context, "GPU range end count", 4, count_usage);
        let count_readback = readback_buffer(context, "GPU range count readback", 8);
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
    let mut encoder = encoder(context, "GPU range count copy");
    encoder.copy_buffer_to_buffer(&buffers.start_count, 0, &buffers.count_readback, 0, 4);
    encoder.copy_buffer_to_buffer(&buffers.end_count, 0, &buffers.count_readback, 4, 4);
    submit(context, encoder.finish());
    let counts = map_u32(context, &buffers.count_readback, 2);
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
    let readback = readback_buffer(context, "GPU range endpoint readback", endpoint_bytes * 2);
    let mut encoder = encoder(context, "GPU range endpoint copy");
    encoder.copy_buffer_to_buffer(&buffers.starts, 0, &readback, 0, endpoint_bytes);
    encoder.copy_buffer_to_buffer(&buffers.ends, 0, &readback, endpoint_bytes, endpoint_bytes);
    submit(context, encoder.finish());
    let words = map_u32(context, &readback, count * 2);
    Some(
        words[..count]
            .iter()
            .zip(&words[count..])
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
    let readback = readback_buffer(context, "GPU range endpoint readback", endpoint_bytes * 2);
    let mut encoder = encoder(context, "GPU range endpoint copy");
    encoder.copy_buffer_to_buffer(&buffers.starts, 0, &readback, 0, endpoint_bytes);
    encoder.copy_buffer_to_buffer(&buffers.ends, 0, &readback, endpoint_bytes, endpoint_bytes);
    submit(context, encoder.finish());
    let words = map_u32(context, &readback, count * 4);
    let (starts, ends) = words.split_at(count * 2);
    Some(
        starts
            .chunks_exact(2)
            .zip(ends.chunks_exact(2))
            .map(|(start, end)| (join_words(start[0], start[1]), join_words(end[0], end[1])))
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
    largest_storage <= limits.max_storage_buffer_binding_size
        && largest_storage <= limits.max_buffer_size
        && largest_readback <= limits.max_buffer_size
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

fn submit(context: &Context, command: wgpu::CommandBuffer) {
    let submission = context.queue.submit(Some(command));
    context
        .device
        .poll(wgpu::PollType::Wait {
            submission_index: Some(submission),
            timeout: None,
        })
        .unwrap_or_else(|error| panic!("GPU range execution failed while waiting: {error}"));
}

fn map_u32(context: &Context, buffer: &wgpu::Buffer, count: usize) -> Vec<u32> {
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
        .unwrap_or_else(|error| panic!("GPU range execution failed while mapping: {error}"));
    receiver
        .recv()
        .unwrap_or_else(|error| panic!("GPU range readback channel failed: {error}"))
        .unwrap_or_else(|error| panic!("GPU range readback mapping failed: {error}"));
    let result = {
        let mapped = slice
            .get_mapped_range()
            .unwrap_or_else(|error| panic!("GPU range readback access failed: {error}"));
        bytemuck::cast_slice::<u8, u32>(&mapped).to_vec()
    };
    buffer.unmap();
    result
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
            /// Constructs from the slice using portable GPU acceleration when supported and
            /// worthwhile, otherwise using the normal CPU implementation.
            ///
            /// This experimental, opt-in policy is available only with the `gpu` feature. It
            /// falls back to the CPU for inputs below its crossover threshold, types without a
            /// GPU key representation, systems without a suitable hardware adapter, and inputs
            /// that exceed adapter buffer or dispatch limits. Other expected environmental or
            /// resource constraints can also make GPU execution unavailable.
            ///
            /// The GPU-capable 32-bit family is `u8`, `u16`, `u32`, `i8`, `i16`, `i32`, `char`,
            /// [`Ipv4Addr`], [`NotNanF32`], and, with `float_nightly_experimental`, `NotNanF16`.
            /// The 64-bit family is `u64`, `i64`, [`NotNanF64`], and `usize`/`isize` according to
            /// the target width. `u128`, `i128`, [`Ipv6Addr`], `NotNanF128`, and the `TotalF*`
            /// types always use the CPU.
            ///
            /// Unexpected GPU execution or implementation failures are not treated as ordinary
            /// policy fallback. Existing CPU constructors are unaffected by this method.
            #[cfg_attr(docsrs, doc(cfg(feature = "gpu")))]
            #[must_use]
            pub fn from_slice_gpu(slice: &[$type]) -> Self {
                from_slice(slice)
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

    fn check<T: GpuElement>(values: &[T]) {
        let expected: RangeSetBlaze<T> = values.iter().copied().collect();
        let Some(context) = gpu_context() else {
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

    #[test]
    fn public_policy_handles_empty_and_small_inputs() {
        assert!(RangeSetBlaze::<u32>::from_slice_gpu(&[]).is_empty());
        assert_eq!(
            RangeSetBlaze::<u32>::from_slice_gpu(&[3, 1, 2, 2]),
            RangeSetBlaze::from_iter([1..=3]),
        );
        assert_eq!(
            RangeSetBlaze::<u128>::from_slice_gpu(&[3, 1, 2, 2]),
            RangeSetBlaze::from_iter([1..=3]),
        );
    }
}
