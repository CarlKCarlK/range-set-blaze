//! Experiment: build a `RangeMapBlaze<u32, u32>` from two `u32` columns (keys, values) with the
//! GPU doing the ingestion: sort, duplicate resolution, range detection, and compaction. Only the
//! finished `(start, end, value)` ranges come back to the CPU, which builds the map from them.
//!
//! Usage: `cargo run --release --features in_place_slices --bin gpu_map_ingest -- [SIZES...]`
//! (default: 1,000,000 and 10,000,000). Prints CSV. Tests:
//! `cargo test --release --features in_place_slices --bin gpu_map_ingest`.
//!
//! Pipeline, all on the GPU after the upload:
//!
//! 1. Stable sort of the keys, carrying the values (`KeyValueSoaSorter`). Equal keys keep their
//!    input order.
//! 2. Survivors: keep an element only if the next key differs, i.e. the last of its key. With the
//!    stable sort, that is the right-most input value, which is the precedence `from_iter` and
//!    `from_slices` use (checked in `precedence_is_last_value_wins`).
//! 3. Compact the survivors' keys and values. Their count stays on the GPU.
//! 4. Boundaries: survivors `i - 1` and `i` share a range if `key[i - 1] != u32::MAX`,
//!    `key[i - 1] + 1 == key[i]`, and their values are equal. The `!= u32::MAX` test comes first,
//!    so `+ 1` never overflows. Mark starts and ends.
//! 5. Compact start keys, end keys, and start values.
//! 6. Read back the range count, then the ranges.
//!
//! `cpu_parallel_from_slices` runs the same pipeline on every CPU thread (the `cpu_parallel_ms`
//! column), as the fair CPU comparison. `lib_cpu_ms` and `lib_gpu_ms` time the library's
//! `RangeMapBlaze::from_in_place_slices` with `Engine::Cpu` (every thread) and `Engine::GpuOnly`.
//!
//! Results on an AMD Radeon 8060S (Vulkan) with a 32-thread CPU are in
//! `experiments/results/gpu_map_ingest_radeon_8060s.csv` (cold start in the `.log` beside it).

use std::hint::black_box;
use std::mem::size_of;
use std::sync::mpsc;
use std::time::{Duration, Instant};

use lampshade::{Compactor, Context, KeyValueSoaSorter};
use range_set_blaze::{CheckSortedDisjointMap, Engine, RangeMapBlaze};
use rayon::iter::{IndexedParallelIterator, IntoParallelRefIterator, ParallelIterator};
use rayon::slice::{ParallelSlice, ParallelSliceMut};

/// Seed for generated data; fixed so every machine benchmarks the same inputs.
const SEED: u64 = 0x5eed_ba5e_d00d_f00d;
const WORKGROUP_SIZE: u32 = 256;

fn main() -> Result<(), String> {
    let sizes: Vec<usize> = std::env::args()
        .skip(1)
        .map(|value| {
            value
                .parse()
                .map_err(|error| format!("bad size {value:?}: {error}"))
        })
        .collect::<Result<_, _>>()?;
    let sizes = if sizes.is_empty() {
        vec![1_000_000, 10_000_000]
    } else {
        sizes
    };

    // Cold start: context and pipelines, then the first (smallest) run, timed separately.
    let start = Instant::now();
    let context = pollster::block_on(Context::init()).map_err(|error| error.to_string())?;
    let context_time = start.elapsed();
    eprintln!(
        "adapter={:?} backend={:?} device_type={:?}",
        context.adapter_info.name, context.adapter_info.backend, context.adapter_info.device_type
    );
    let start = Instant::now();
    let kernels = Kernels::new(&context);
    let kernel_time = start.elapsed();
    eprintln!(
        "cold: context {:.1} ms, shader pipelines {:.1} ms",
        milliseconds(context_time),
        milliseconds(kernel_time)
    );

    println!(
        "pattern,order,input_pairs,output_ranges,gpu_first_call_ms,gpu_steady_ms,cpu_from_slices_ms,cpu_from_iter_ms,cpu_rayon_sort_ms,cpu_parallel_ms,lib_cpu_ms,lib_gpu_ms"
    );
    for &n in &sizes {
        for pattern in Pattern::ALL {
            for shuffled in [true, false] {
                benchmark_case(&context, &kernels, pattern, n, shuffled)?;
            }
        }
    }
    Ok(())
}

fn all_threads() -> std::num::NonZeroUsize {
    std::thread::available_parallelism().unwrap_or(std::num::NonZeroUsize::MIN)
}

/// Times one pattern, size, and order, and prints its CSV row.
fn benchmark_case(
    context: &Context,
    kernels: &Kernels,
    pattern: Pattern,
    n: usize,
    shuffled: bool,
) -> Result<(), String> {
    let (keys, values) = pattern.generate(n, shuffled, SEED);
    let expected = RangeMapBlaze::from_slices(&keys, &values);

    // First call at this size: allocates buffers and prepares the sorter.
    let start = Instant::now();
    let mut ingest = MapIngest::new(context, kernels, n)?;
    let first = ingest.run(&keys, &values)?;
    let first_time = start.elapsed();
    if first != expected {
        return Err(format!(
            "GPU result differs from from_slices: {} {n}",
            pattern.name()
        ));
    }

    // Steady state: the same prepared buffers and pipelines, reused.
    let gpu = median(5, || ingest.run(black_box(&keys), black_box(&values)))?;
    let cpu_slices = median(5, || {
        Ok(RangeMapBlaze::from_slices(
            black_box(&keys),
            black_box(&values),
        ))
    })?;
    let cpu_iter = median(5, || {
        Ok(black_box(&keys)
            .iter()
            .copied()
            .zip(black_box(&values).iter().copied())
            .collect::<RangeMapBlaze<u32, u32>>())
    })?;
    let cpu_rayon = median(5, || {
        Ok(rayon_sort_then_from_slices(
            black_box(&keys),
            black_box(&values),
        ))
    })?;
    if cpu_parallel_from_slices(&keys, &values) != expected {
        return Err(format!(
            "CPU parallel result differs from from_slices: {} {n}",
            pattern.name()
        ));
    }
    let cpu_parallel = median(5, || {
        Ok(cpu_parallel_from_slices(
            black_box(&keys),
            black_box(&values),
        ))
    })?;
    // The library's `from_in_place_slices`. It doesn't reorder these slices, so one copy serves
    // every run.
    let (mut lib_keys, mut lib_values) = (keys.clone(), values.clone());
    let mut library =
        |engine| RangeMapBlaze::from_in_place_slices(&mut lib_keys, &mut lib_values, engine);
    for engine in [Engine::Cpu(all_threads()), Engine::GpuOnly] {
        if library(engine) != expected {
            return Err(format!(
                "from_in_place_slices({engine:?}) differs from from_slices: {} {n}",
                pattern.name()
            ));
        }
    }
    let library_threads = median(5, || Ok(library(Engine::Cpu(all_threads()))))?;
    let library_gpu = median(5, || Ok(library(Engine::GpuOnly)))?;
    println!(
        "{},{},{n},{},{:.3},{:.3},{:.3},{:.3},{:.3},{:.3},{:.3},{:.3}",
        pattern.name(),
        if shuffled { "shuffled" } else { "in_order" },
        expected.ranges_len(),
        milliseconds(first_time),
        milliseconds(gpu),
        milliseconds(cpu_slices),
        milliseconds(cpu_iter),
        milliseconds(cpu_rayon),
        milliseconds(cpu_parallel),
        milliseconds(library_threads),
        milliseconds(library_gpu),
    );
    Ok(())
}

/// The CPU counterpart of the GPU pipeline, on every Rayon thread:
///
/// 1. Pack each pair's key and input position into a `u64` (`key << 32 | position`) and sort
///    those with Rayon's unstable sort. Each packed value is distinct, so equal keys still end
///    up in input order, and the last of each key is the right-most input value.
/// 2. Split the sorted array into chunks that start at key boundaries, so no key spans two
///    chunks, and in each chunk keep the last of each key and merge consecutive keys with
///    equal values into ranges.
/// 3. Join the chunks' ranges, merging across chunk seams, and build the map from them.
fn cpu_parallel_from_slices(keys: &[u32], values: &[u32]) -> RangeMapBlaze<u32, u32> {
    assert_eq!(keys.len(), values.len(), "keys and values differ in length");
    assert!(
        u32::try_from(keys.len().saturating_sub(1)).is_ok(),
        "more than 2^32 pairs"
    );
    let mut packed: Vec<u64> = keys
        .par_iter()
        .enumerate()
        .map(|(position, &key)| u64::from(key) << 32 | position as u64)
        .collect();
    packed.par_sort_unstable();

    let key = |index: usize| (packed[index] >> 32) as u32;
    let chunk_count = (rayon::current_num_threads() * 4).min(packed.len()).max(1);
    let mut bounds = vec![0];
    for chunk in 1..chunk_count {
        let mut bound = (chunk * packed.len() / chunk_count).max(*bounds.last().expect("nonempty"));
        while bound < packed.len() && bound > 0 && key(bound) == key(bound - 1) {
            bound += 1;
        }
        bounds.push(bound);
    }
    bounds.push(packed.len());

    let chunks: Vec<Vec<(u32, u32, u32)>> = bounds
        .par_windows(2)
        .map(|window| sorted_chunk_ranges(&packed[window[0]..window[1]], values))
        .collect();

    let mut ranges: Vec<(u32, u32, u32)> = Vec::with_capacity(chunks.iter().map(Vec::len).sum());
    for (start, end, value) in chunks.into_iter().flatten() {
        match ranges.last_mut() {
            Some((_, last_end, last_value))
                if *last_value == value && last_end.checked_add(1) == Some(start) =>
            {
                *last_end = end;
            }
            _ => ranges.push((start, end, value)),
        }
    }
    RangeMapBlaze::from_sorted_disjoint_map(CheckSortedDisjointMap::new(
        ranges
            .iter()
            .map(|(start, end, value)| (*start..=*end, value)),
    ))
}

/// The `(start, end, value)` ranges of a sorted run of packed pairs that no key crosses out of.
fn sorted_chunk_ranges(packed: &[u64], values: &[u32]) -> Vec<(u32, u32, u32)> {
    let mut ranges: Vec<(u32, u32, u32)> = Vec::new();
    for (index, &pair) in packed.iter().enumerate() {
        let key = (pair >> 32) as u32;
        // Only the last of each key survives.
        if packed
            .get(index + 1)
            .is_some_and(|&next| (next >> 32) as u32 == key)
        {
            continue;
        }
        let position = usize::try_from(pair & u64::from(u32::MAX)).expect("position fits in usize");
        let value = values[position];
        match ranges.last_mut() {
            Some((_, end, last_value))
                if *last_value == value && end.checked_add(1) == Some(key) =>
            {
                *end = key;
            }
            _ => ranges.push((key, key, value)),
        }
    }
    ranges
}

/// The multithreaded CPU baseline: pair up the columns, sort the pairs by key with Rayon's
/// *stable* parallel sort (so equal keys keep their input order and the last value still wins),
/// split them back into columns, and build the map with `from_slices`.
fn rayon_sort_then_from_slices(keys: &[u32], values: &[u32]) -> RangeMapBlaze<u32, u32> {
    let mut pairs: Vec<(u32, u32)> = keys.iter().copied().zip(values.iter().copied()).collect();
    pairs.par_sort_by_key(|&(key, _)| key);
    let (sorted_keys, sorted_values): (Vec<u32>, Vec<u32>) = pairs.into_iter().unzip();
    RangeMapBlaze::from_slices(&sorted_keys, &sorted_values)
}

/// The compiled survivor and boundary shaders; built once per context.
struct Kernels {
    survivors: Kernel,
    boundaries: Kernel,
}

impl Kernels {
    fn new(context: &Context) -> Self {
        Self {
            survivors: Kernel::new(context, "survivors", SURVIVOR_SHADER, &[true, false, false]),
            boundaries: Kernel::new(
                context,
                "boundaries",
                BOUNDARY_SHADER,
                &[true, true, true, false, false, false],
            ),
        }
    }
}

/// Keep element `i` of the sorted keys if it is the last of its key.
const SURVIVOR_SHADER: &str = r"
@group(0) @binding(0) var<storage, read> keys: array<u32>;
@group(0) @binding(1) var<storage, read_write> keep: array<u32>;
@group(0) @binding(2) var<storage, read_write> parameters: array<u32>; // [len, groups_x]

@compute @workgroup_size(256)
fn main(@builtin(workgroup_id) workgroup: vec3<u32>, @builtin(local_invocation_id) local: vec3<u32>) {
    let index = (workgroup.y * parameters[1] + workgroup.x) * 256u + local.x;
    let len = parameters[0];
    if index >= len {
        return;
    }
    let last_of_key = index + 1u == len || keys[index + 1u] != keys[index];
    keep[index] = select(0u, 1u, last_of_key);
}
";

/// Mark range starts and ends among the `count` survivors (`count` is read from the GPU, so the
/// survivor compaction never round-trips to the CPU). Indices at or past `count` get no marks.
const BOUNDARY_SHADER: &str = r"
@group(0) @binding(0) var<storage, read> keys: array<u32>;
@group(0) @binding(1) var<storage, read> values: array<u32>;
@group(0) @binding(2) var<storage, read> count: array<u32>; // [survivor count]
@group(0) @binding(3) var<storage, read_write> start_mask: array<u32>;
@group(0) @binding(4) var<storage, read_write> end_mask: array<u32>;
@group(0) @binding(5) var<storage, read_write> parameters: array<u32>; // [len, groups_x]

// Survivors `left` and `left + 1` share a range. Test `!= MAX` first so `+ 1u` can't overflow.
fn joined(left: u32) -> bool {
    let key = keys[left];
    return key != 0xffffffffu && keys[left + 1u] == key + 1u && values[left + 1u] == values[left];
}

@compute @workgroup_size(256)
fn main(@builtin(workgroup_id) workgroup: vec3<u32>, @builtin(local_invocation_id) local: vec3<u32>) {
    let index = (workgroup.y * parameters[1] + workgroup.x) * 256u + local.x;
    if index >= parameters[0] {
        return;
    }
    let survivors = count[0];
    if index >= survivors {
        start_mask[index] = 0u;
        end_mask[index] = 0u;
        return;
    }
    let is_start = index == 0u || !joined(index - 1u);
    let is_end = index + 1u == survivors || !joined(index);
    start_mask[index] = select(0u, 1u, is_start);
    end_mask[index] = select(0u, 1u, is_end);
}
";

/// One compute shader with storage-buffer bindings (`read_only` per binding).
struct Kernel {
    pipeline: wgpu::ComputePipeline,
    layout: wgpu::BindGroupLayout,
}

impl Kernel {
    fn new(context: &Context, label: &'static str, source: &str, read_only: &[bool]) -> Self {
        let entries: Vec<_> = read_only
            .iter()
            .enumerate()
            .map(|(binding, &read_only)| wgpu::BindGroupLayoutEntry {
                binding: u32::try_from(binding).expect("few bindings"),
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            })
            .collect();
        let device = &context.device;
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some(label),
            entries: &entries,
        });
        let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some(label),
            source: wgpu::ShaderSource::Wgsl(source.into()),
        });
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some(label),
            bind_group_layouts: &[Some(&layout)],
            immediate_size: 0,
        });
        let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some(label),
            layout: Some(&pipeline_layout),
            module: &module,
            entry_point: Some("main"),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            cache: None,
        });
        Self { pipeline, layout }
    }

    fn bind(&self, context: &Context, buffers: &[&wgpu::Buffer]) -> wgpu::BindGroup {
        let entries: Vec<_> = buffers
            .iter()
            .enumerate()
            .map(|(binding, buffer)| wgpu::BindGroupEntry {
                binding: u32::try_from(binding).expect("few bindings"),
                resource: buffer.as_entire_binding(),
            })
            .collect();
        context
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                label: None,
                layout: &self.layout,
                entries: &entries,
            })
    }

    fn record(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        bind_group: &wgpu::BindGroup,
        groups: (u32, u32),
    ) {
        let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: None,
            timestamp_writes: None,
        });
        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, bind_group, &[]);
        pass.dispatch_workgroups(groups.0, groups.1, 1);
    }
}

/// GPU buffers and prepared sorter for maps of up to `capacity` pairs. Reused across runs of the
/// same size; [`MapIngest::run`] re-uploads the columns each time.
struct MapIngest<'a> {
    context: &'a Context,
    len: u32,
    groups: (u32, u32),
    keys: wgpu::Buffer,
    values: wgpu::Buffer,
    survivor_keys: wgpu::Buffer,
    survivor_values: wgpu::Buffer,
    start_keys: wgpu::Buffer,
    end_keys: wgpu::Buffer,
    start_values: wgpu::Buffer,
    survivor_count: wgpu::Buffer,
    survivor_value_count: wgpu::Buffer,
    range_count: wgpu::Buffer,
    end_count: wgpu::Buffer,
    start_value_count: wgpu::Buffer,
    keep: wgpu::Buffer,
    start_mask: wgpu::Buffer,
    end_mask: wgpu::Buffer,
    survivor_bind: wgpu::BindGroup,
    boundary_bind: wgpu::BindGroup,
    sorter: KeyValueSoaSorter,
    compactor: Compactor,
    kernels: &'a Kernels,
}

impl<'a> MapIngest<'a> {
    fn new(context: &'a Context, kernels: &'a Kernels, capacity: usize) -> Result<Self, String> {
        let len =
            u32::try_from(capacity.max(1)).map_err(|_| "more than u32::MAX pairs".to_string())?;
        let bytes = u64::from(len) * size_of::<u32>() as u64;
        let limits = context.device.limits();
        if bytes > limits.max_storage_buffer_binding_size || bytes > limits.max_buffer_size {
            return Err(format!(
                "{len} pairs need {bytes}-byte buffers; this adapter allows {}",
                limits
                    .max_storage_buffer_binding_size
                    .min(limits.max_buffer_size)
            ));
        }
        let device = &context.device;
        let column = |label| {
            storage(
                device,
                label,
                bytes,
                wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::COPY_SRC,
            )
        };
        let mask = |label| storage(device, label, bytes, wgpu::BufferUsages::COPY_SRC);
        let counter = |label| {
            storage(
                device,
                label,
                4,
                wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::COPY_SRC,
            )
        };
        let keys = column("keys");
        let values = column("values");
        let survivor_keys = column("survivor keys");
        let survivor_values = column("survivor values");
        let start_keys = column("start keys");
        let end_keys = column("end keys");
        let start_values = column("start values");
        let survivor_count = counter("survivor count");
        let survivor_value_count = counter("survivor value count");
        let range_count = counter("range count");
        let end_count = counter("end count");
        let start_value_count = counter("start value count");
        let keep = mask("keep mask");
        let start_mask = mask("start mask");
        let end_mask = mask("end mask");

        let total_groups = len.div_ceil(WORKGROUP_SIZE);
        let maximum = limits.max_compute_workgroups_per_dimension;
        let groups = (total_groups.min(maximum), total_groups.div_ceil(maximum));
        if groups.1 > maximum {
            return Err("too many workgroups for this adapter".to_string());
        }
        let parameters = storage(device, "parameters", 8, wgpu::BufferUsages::COPY_DST);
        context
            .queue
            .write_buffer(&parameters, 0, bytemuck::cast_slice(&[len, groups.0]));
        let survivor_bind = kernels
            .survivors
            .bind(context, &[&keys, &keep, &parameters]);
        let boundary_bind = kernels.boundaries.bind(
            context,
            &[
                &survivor_keys,
                &survivor_values,
                &survivor_count,
                &start_mask,
                &end_mask,
                &parameters,
            ],
        );

        let mut sorter = KeyValueSoaSorter::from_context(context);
        sorter
            .prepare_sort(&keys, &values, len)
            .map_err(|error| error.to_string())?;
        Ok(Self {
            context,
            len,
            groups,
            keys,
            values,
            survivor_keys,
            survivor_values,
            start_keys,
            end_keys,
            start_values,
            survivor_count,
            survivor_value_count,
            range_count,
            end_count,
            start_value_count,
            keep,
            start_mask,
            end_mask,
            survivor_bind,
            boundary_bind,
            sorter,
            compactor: Compactor::from_context(context),
            kernels,
        })
    }

    /// Builds the map from two same-length columns of exactly this ingest's capacity.
    fn run(&mut self, keys: &[u32], values: &[u32]) -> Result<RangeMapBlaze<u32, u32>, String> {
        if keys.len() != values.len() {
            return Err("keys and values must have the same length".to_string());
        }
        if keys.is_empty() {
            return Ok(RangeMapBlaze::new());
        }
        if keys.len() != self.len as usize {
            return Err(format!(
                "prepared for {} pairs, given {}",
                self.len,
                keys.len()
            ));
        }
        let context = self.context;
        let error_scope = context
            .device
            .push_error_scope(wgpu::ErrorFilter::Validation);
        context
            .queue
            .write_buffer(&self.keys, 0, bytemuck::cast_slice(keys));
        context
            .queue
            .write_buffer(&self.values, 0, bytemuck::cast_slice(values));

        let mut encoder = context
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("map ingest"),
            });
        self.record(&mut encoder)?;
        let count_readback = readback(&context.device, 4);
        encoder.copy_buffer_to_buffer(&self.range_count, 0, &count_readback, 0, 4);
        context.queue.submit(Some(encoder.finish()));
        let ranges = map_u32(&context.device, &count_readback, 1)?[0] as usize;

        // Second trip: copy just the finished ranges.
        let bytes = ranges as u64 * 4;
        let readback = readback(&context.device, bytes.max(4) * 3);
        let mut encoder = context
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("map ingest readback"),
            });
        for (index, buffer) in [&self.start_keys, &self.end_keys, &self.start_values]
            .iter()
            .enumerate()
        {
            encoder.copy_buffer_to_buffer(buffer, 0, &readback, index as u64 * bytes, bytes);
        }
        context.queue.submit(Some(encoder.finish()));
        let words = map_u32(&context.device, &readback, ranges * 3)?;
        if let Some(error) = pollster::block_on(error_scope.pop()) {
            return Err(format!("GPU validation error: {error}"));
        }
        let (starts, rest) = words.split_at(ranges);
        let (ends, range_values) = rest.split_at(ranges);
        Ok(RangeMapBlaze::from_sorted_disjoint_map(
            CheckSortedDisjointMap::new(
                starts
                    .iter()
                    .zip(ends)
                    .zip(range_values)
                    .map(|((&start, &end), value)| (start..=end, value)),
            ),
        ))
    }
}

impl MapIngest<'_> {
    /// Records the sort, survivor marking and compaction, boundary marking, and range compaction.
    fn record(&mut self, encoder: &mut wgpu::CommandEncoder) -> Result<(), String> {
        let gpu_error = |error: lampshade::Error| error.to_string();
        self.sorter
            .record_reserved_sort(encoder, &self.keys, &self.values, self.len)
            .map_err(gpu_error)?;
        self.kernels
            .survivors
            .record(encoder, &self.survivor_bind, self.groups);
        for (input, output, count) in [
            (&self.keys, &self.survivor_keys, &self.survivor_count),
            (
                &self.values,
                &self.survivor_values,
                &self.survivor_value_count,
            ),
        ] {
            self.compactor
                .record_compact(encoder, input, &self.keep, output, count, self.len)
                .map_err(gpu_error)?;
        }
        self.kernels
            .boundaries
            .record(encoder, &self.boundary_bind, self.groups);
        for (input, mask, output, count) in [
            (
                &self.survivor_keys,
                &self.start_mask,
                &self.start_keys,
                &self.range_count,
            ),
            (
                &self.survivor_values,
                &self.start_mask,
                &self.start_values,
                &self.start_value_count,
            ),
            (
                &self.survivor_keys,
                &self.end_mask,
                &self.end_keys,
                &self.end_count,
            ),
        ] {
            self.compactor
                .record_compact(encoder, input, mask, output, count, self.len)
                .map_err(gpu_error)?;
        }
        Ok(())
    }
}

fn storage(
    device: &wgpu::Device,
    label: &'static str,
    size: u64,
    extra: wgpu::BufferUsages,
) -> wgpu::Buffer {
    device.create_buffer(&wgpu::BufferDescriptor {
        label: Some(label),
        size,
        usage: wgpu::BufferUsages::STORAGE | extra,
        mapped_at_creation: false,
    })
}

fn readback(device: &wgpu::Device, size: u64) -> wgpu::Buffer {
    device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("readback"),
        size,
        usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    })
}

fn map_u32(device: &wgpu::Device, buffer: &wgpu::Buffer, count: usize) -> Result<Vec<u32>, String> {
    let slice = buffer.slice(..(count.max(1) as u64 * 4));
    let (sender, receiver) = mpsc::channel();
    slice.map_async(wgpu::MapMode::Read, move |result| drop(sender.send(result)));
    device
        .poll(wgpu::PollType::Wait {
            submission_index: None,
            timeout: None,
        })
        .map_err(|error| error.to_string())?;
    receiver
        .recv()
        .map_err(|error| error.to_string())?
        .map_err(|error| error.to_string())?;
    let words = {
        let mapped = slice
            .get_mapped_range()
            .map_err(|error| error.to_string())?;
        bytemuck::cast_slice::<u8, u32>(&mapped)[..count].to_vec()
    };
    buffer.unmap();
    Ok(words)
}

/// Input shapes that vary how much the output compresses.
#[derive(Clone, Copy)]
enum Pattern {
    /// Consecutive keys, one value: the whole input is one range.
    OneValue,
    /// Consecutive keys, a new value every 256 keys: `n / 256` ranges.
    ValueEvery256,
    /// Consecutive keys, a new value every 4 keys: `n / 4` ranges.
    ValueEvery4,
    /// Keys two apart (never adjacent): `n` ranges, no compression.
    Gaps,
    /// Each key appears 8 times with conflicting values; the last one wins.
    Duplicates,
}

impl Pattern {
    const ALL: [Self; 5] = [
        Self::OneValue,
        Self::ValueEvery256,
        Self::ValueEvery4,
        Self::Gaps,
        Self::Duplicates,
    ];

    const fn name(self) -> &'static str {
        match self {
            Self::OneValue => "one_value",
            Self::ValueEvery256 => "value_every_256",
            Self::ValueEvery4 => "value_every_4",
            Self::Gaps => "gaps",
            Self::Duplicates => "duplicates_x8",
        }
    }

    /// `n` (key, value) pairs; shuffled with xorshift from `seed` if `shuffled`.
    fn generate(self, n: usize, shuffled: bool, seed: u64) -> (Vec<u32>, Vec<u32>) {
        let index = |i: usize| u32::try_from(i).expect("benchmark sizes fit in u32");
        let mut pairs: Vec<(u32, u32)> = (0..n)
            .map(|i| match self {
                Self::OneValue => (index(i), 7),
                Self::ValueEvery256 => (index(i), index(i / 256)),
                Self::ValueEvery4 => (index(i), index(i / 4)),
                Self::Gaps => (index(i) * 2, index(i)),
                Self::Duplicates => (index(i / 8), index(i)),
            })
            .collect();
        if shuffled {
            shuffle(&mut pairs, seed);
        }
        pairs.into_iter().unzip()
    }
}

fn shuffle<T>(items: &mut [T], seed: u64) {
    let mut state = seed;
    for i in (1..items.len()).rev() {
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        let modulus = u64::try_from(i + 1).expect("index fits in u64");
        items.swap(
            i,
            usize::try_from(state % modulus).expect("swap index fits in usize"),
        );
    }
}

/// Median of `runs` timed calls after one warm-up.
fn median<R>(
    runs: usize,
    mut operation: impl FnMut() -> Result<R, String>,
) -> Result<Duration, String> {
    drop(operation()?);
    let mut samples = Vec::with_capacity(runs);
    for _ in 0..runs {
        let start = Instant::now();
        drop(black_box(operation()?));
        samples.push(start.elapsed());
    }
    samples.sort_unstable();
    Ok(samples[runs / 2])
}

fn milliseconds(duration: Duration) -> f64 {
    duration.as_secs_f64() * 1_000.0
}

#[cfg(test)]
mod tests {
    use super::*;

    /// One GPU context for all tests (creating devices is slow); `None` if there is no adapter.
    fn context() -> Option<&'static (Context, Kernels)> {
        static CONTEXT: std::sync::OnceLock<Option<(Context, Kernels)>> =
            std::sync::OnceLock::new();
        CONTEXT
            .get_or_init(|| {
                let context = pollster::block_on(Context::init()).ok()?;
                let kernels = Kernels::new(&context);
                Some((context, kernels))
            })
            .as_ref()
    }

    /// The GPU result must equal both CPU constructors.
    fn check(keys: &[u32], values: &[u32]) {
        let expected = RangeMapBlaze::from_slices(keys, values);
        let from_iter: RangeMapBlaze<u32, u32> =
            keys.iter().copied().zip(values.iter().copied()).collect();
        assert_eq!(expected, from_iter);
        assert_eq!(rayon_sort_then_from_slices(keys, values), expected);
        assert_eq!(cpu_parallel_from_slices(keys, values), expected);
        let Some((context, kernels)) = context() else {
            assert!(
                std::env::var_os("RANGE_SET_BLAZE_REQUIRE_TEST_GPU").is_none(),
                "no test GPU"
            );
            return;
        };
        let actual = MapIngest::new(context, kernels, keys.len())
            .and_then(|mut ingest| ingest.run(keys, values))
            .expect("GPU ingest");
        assert_eq!(actual, expected, "keys={keys:?} values={values:?}");
    }

    /// Verify first: with duplicate keys, the last (right-most) value wins, for both CPU
    /// constructors. The GPU pipeline relies on a stable sort to match this.
    #[test]
    fn precedence_is_last_value_wins() {
        let keys = [5_u32, 3, 5, 4, 5];
        let values = [1_u32, 9, 2, 9, 3];
        let from_slices = RangeMapBlaze::from_slices(keys, values);
        let from_iter: RangeMapBlaze<u32, u32> = keys.into_iter().zip(values).collect();
        assert_eq!(from_slices, from_iter);
        assert_eq!(from_slices.get(5), Some(&3));
        assert_eq!(from_slices.to_string(), "(3..=4, 9), (5..=5, 3)");
    }

    #[test]
    fn small_cases() {
        check(&[], &[]);
        check(&[7], &[1]);
        // Unsorted, equal values merge.
        check(&[3, 1, 2], &[5, 5, 5]);
        // Changing values split a run of consecutive keys.
        check(&[1, 2, 3, 4], &[5, 5, 6, 6]);
        // Gaps split equal values.
        check(&[1, 3, 5], &[5, 5, 5]);
        // Duplicates with conflicting values: the last wins, and can join or split ranges.
        check(&[2, 1, 2, 3, 2], &[8, 7, 9, 7, 7]);
        check(&[1, 1, 1], &[1, 2, 3]);
    }

    /// `u32::MAX` must not be treated as adjacent to `0` (that would need `MAX + 1`).
    #[test]
    fn u32_boundaries() {
        check(&[u32::MAX, 0], &[4, 4]);
        check(&[u32::MAX - 1, u32::MAX, 0, 1], &[4, 4, 4, 4]);
        check(&[u32::MAX, u32::MAX - 1, u32::MAX], &[1, 2, 2]);
        check(&[0, u32::MAX, u32::MAX], &[0, 1, 0]);
    }

    /// Larger seeded random cases, around the 256-wide workgroup and many workgroups, mixing
    /// duplicates, conflicting values, runs, and gaps.
    #[test]
    fn random_cases() {
        let mut state = 0x0dd5_eed5_1234_5678_u64;
        let mut next = move || {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            state
        };
        for len in [1_usize, 2, 255, 256, 257, 1_000, 65_537] {
            for spread in [4_u64, 64, 1 << 20] {
                let keys: Vec<u32> = (0..len)
                    .map(|_| u32::try_from(next() % spread).expect("small"))
                    .collect();
                let values: Vec<u32> = (0..len)
                    .map(|_| u32::try_from(next() % 3).expect("small"))
                    .collect();
                check(&keys, &values);
            }
        }
    }

    /// The benchmark patterns, small, both orders.
    #[test]
    fn benchmark_patterns() {
        for pattern in Pattern::ALL {
            for shuffled in [false, true] {
                let (keys, values) = pattern.generate(5_000, shuffled, SEED);
                check(&keys, &values);
            }
        }
    }
}
