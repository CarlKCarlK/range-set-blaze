//! Portable wgpu/WGSL experiment for unsorted `u32` normalization.

include!("gpu_normalize/common.rs");

use std::hint::black_box;
use std::iter::FusedIterator;
use std::mem::size_of;
use std::ops::RangeInclusive;
use std::sync::mpsc;
use std::time::{Duration, Instant};

use lampshade::{Compactor, Context, Sorter};
use num_traits::ToPrimitive;
use range_set_blaze::{RangeSetBlaze, SortedDisjoint, SortedStarts};

const WORKGROUP_SIZE: u32 = 256;

const BOUNDARY_SHADER_TEMPLATE: &str = r"
struct Parameters {
    len: u32,
    groups_x: u32,
    _padding0: u32,
    _padding1: u32,
}

@group(0) @binding(0) var<storage, read> values: array<u32>;
@group(0) @binding(1) var<storage, read_write> start_mask: array<u32>;
@group(0) @binding(2) var<storage, read_write> end_mask: array<u32>;
@group(0) @binding(3) var<uniform> parameters: Parameters;

@compute @workgroup_size({{WORKGROUP_SIZE}})
fn main(
    @builtin(workgroup_id) workgroup: vec3<u32>,
    @builtin(local_invocation_id) local: vec3<u32>,
) {
    let group = workgroup.y * parameters.groups_x + workgroup.x;
    let index = group * {{WORKGROUP_SIZE}}u + local.x;
    if index >= parameters.len {
        return;
    }

    let value = values[index];
    var is_start = index == 0u;
    if !is_start {
        let previous = values[index - 1u];
        is_start = value != previous &&
            (previous == 0xffffffffu || value != previous + 1u);
    }

    var is_end = index + 1u == parameters.len;
    if !is_end {
        let next = values[index + 1u];
        is_end = next != value &&
            (value == 0xffffffffu || next != value + 1u);
    }

    start_mask[index] = select(0u, 1u, is_start);
    end_mask[index] = select(0u, 1u, is_end);
}
";

struct BoundaryKernel {
    pipeline: wgpu::ComputePipeline,
    bind_group: wgpu::BindGroup,
    groups_x: u32,
    groups_y: u32,
}

impl BoundaryKernel {
    fn new(
        context: &Context,
        values: &wgpu::Buffer,
        start_mask: &wgpu::Buffer,
        end_mask: &wgpu::Buffer,
        len: u32,
    ) -> Self {
        let total_groups = len.div_ceil(WORKGROUP_SIZE);
        let maximum_groups = context.device.limits().max_compute_workgroups_per_dimension;
        let groups_x = total_groups.min(maximum_groups);
        let groups_y = total_groups.div_ceil(maximum_groups);
        assert!(
            groups_y <= maximum_groups,
            "input needs more two-dimensional workgroups than this adapter supports"
        );

        let parameters = [len, groups_x, 0, 0];
        let parameter_buffer = context.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Range boundary parameters"),
            size: size_of::<[u32; 4]>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        context
            .queue
            .write_buffer(&parameter_buffer, 0, bytemuck::cast_slice(&parameters));

        let layout = context
            .device
            .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Range boundary layout"),
                entries: &[
                    storage_layout_entry(0, true),
                    storage_layout_entry(1, false),
                    storage_layout_entry(2, false),
                    wgpu::BindGroupLayoutEntry {
                        binding: 3,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                ],
            });
        let shader_source =
            BOUNDARY_SHADER_TEMPLATE.replace("{{WORKGROUP_SIZE}}", &WORKGROUP_SIZE.to_string());
        let shader = context
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("Range boundary shader"),
                source: wgpu::ShaderSource::Wgsl(shader_source.into()),
            });
        let pipeline_layout =
            context
                .device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("Range boundary pipeline layout"),
                    bind_group_layouts: &[Some(&layout)],
                    immediate_size: 0,
                });
        let pipeline = context
            .device
            .create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: Some("Range boundary pipeline"),
                layout: Some(&pipeline_layout),
                module: &shader,
                entry_point: Some("main"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                cache: None,
            });
        let bind_group = context
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("Range boundary bind group"),
                layout: &layout,
                entries: &[
                    entire_buffer_entry(0, values),
                    entire_buffer_entry(1, start_mask),
                    entire_buffer_entry(2, end_mask),
                    entire_buffer_entry(3, &parameter_buffer),
                ],
            });
        Self {
            pipeline,
            bind_group,
            groups_x,
            groups_y,
        }
    }

    fn record(&self, encoder: &mut wgpu::CommandEncoder) {
        let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("Mark range boundaries"),
            timestamp_writes: None,
        });
        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, &self.bind_group, &[]);
        pass.dispatch_workgroups(self.groups_x, self.groups_y, 1);
    }
}

struct WgpuPipeline {
    context: Context,
    mappable_primary_available: bool,
    mapped_primary: bool,
    sorter: Sorter,
    compactor: Compactor,
    boundaries: BoundaryKernel,
    input: wgpu::Buffer,
    sorted: wgpu::Buffer,
    start_mask: wgpu::Buffer,
    end_mask: wgpu::Buffer,
    starts: wgpu::Buffer,
    ends: wgpu::Buffer,
    start_count: wgpu::Buffer,
    end_count: wgpu::Buffer,
    count_readback: Option<wgpu::Buffer>,
    range_readback: Option<wgpu::Buffer>,
    len: u32,
}

impl WgpuPipeline {
    fn new(values: &[u32]) -> Result<Self, String> {
        if values.is_empty() {
            return Err("input must not be empty".to_owned());
        }
        let len = u32::try_from(values.len()).map_err(|_| "input exceeds u32::MAX items")?;
        let force_staged = std::env::args().any(|argument| argument == "--staged");
        let (context, mappable_primary_available, mapped_primary) =
            pollster::block_on(init_context(!force_staged))?;
        let byte_len = u64::from(len) * size_of::<u32>() as u64;
        validate_buffer_size(&context, len, byte_len)?;

        let input_usage = if mapped_primary {
            wgpu::BufferUsages::MAP_WRITE
        } else {
            wgpu::BufferUsages::COPY_DST
        };
        let input = storage_buffer(&context.device, "Unsorted input", byte_len, input_usage);
        let sorted = storage_buffer(
            &context.device,
            "Sorted values",
            byte_len,
            wgpu::BufferUsages::empty(),
        );
        let start_mask = storage_buffer(
            &context.device,
            "Range start mask",
            byte_len,
            wgpu::BufferUsages::COPY_SRC,
        );
        let end_mask = storage_buffer(
            &context.device,
            "Range end mask",
            byte_len,
            wgpu::BufferUsages::COPY_SRC,
        );
        let readback_usage = if mapped_primary {
            wgpu::BufferUsages::MAP_READ
        } else {
            wgpu::BufferUsages::COPY_SRC
        };
        let starts = storage_buffer(
            &context.device,
            "Compacted range starts",
            byte_len,
            readback_usage,
        );
        let ends = storage_buffer(
            &context.device,
            "Compacted range ends",
            byte_len,
            readback_usage,
        );
        let count_usage = wgpu::BufferUsages::COPY_DST | readback_usage;
        let start_count = storage_buffer(
            &context.device,
            "Range start count",
            size_of::<u32>() as u64,
            count_usage,
        );
        let end_count = storage_buffer(
            &context.device,
            "Range end count",
            size_of::<u32>() as u64,
            count_usage,
        );
        let count_readback = (!mapped_primary).then(|| {
            readback_buffer(
                &context.device,
                "Range count readback",
                2 * size_of::<u32>() as u64,
            )
        });
        let boundaries = BoundaryKernel::new(&context, &sorted, &start_mask, &end_mask, len);
        let radix_sorter = Sorter::from_context(&context);
        let compactor = Compactor::from_context(&context);
        let pipeline = Self {
            context,
            mappable_primary_available,
            mapped_primary,
            sorter: radix_sorter,
            compactor,
            boundaries,
            input,
            sorted,
            start_mask,
            end_mask,
            starts,
            ends,
            start_count,
            end_count,
            count_readback,
            range_readback: None,
            len,
        };
        pipeline.upload(values)?;
        Ok(pipeline)
    }

    fn adapter_label(&self) -> String {
        let adapter = self
            .context
            .adapter_info
            .name
            .chars()
            .map(|character| {
                if character.is_ascii_alphanumeric() {
                    character.to_ascii_lowercase()
                } else {
                    '-'
                }
            })
            .collect::<String>();
        format!(
            "wgpu-lampshade-{}-{:?}-{adapter}",
            if self.mapped_primary {
                "mapped-primary"
            } else {
                "staged"
            },
            self.context.adapter_info.backend,
        )
        .to_ascii_lowercase()
    }

    fn upload(&self, values: &[u32]) -> Result<(), String> {
        if self.mapped_primary {
            map_write_u32(&self.context.device, &self.input, values)
        } else {
            self.enqueue_upload(values);
            self.submit_and_wait(None)
        }
    }

    fn enqueue_upload(&self, values: &[u32]) {
        assert_eq!(values.len(), self.len as usize);
        self.context
            .queue
            .write_buffer(&self.input, 0, bytemuck::cast_slice(values));
    }

    fn radix_sort(&mut self) -> Result<(), String> {
        let mut encoder = self.encoder("Lampshade radix sort");
        self.record_radix_sort(&mut encoder)?;
        self.submit_and_wait(Some(encoder.finish()))
    }

    fn record_radix_sort(&mut self, encoder: &mut wgpu::CommandEncoder) -> Result<(), String> {
        self.sorter
            .record_sort(encoder, &self.input, &self.sorted, self.len)
            .map_err(|error| error.to_string())
    }

    fn normalize(&mut self) -> Result<(), String> {
        let mut encoder = self.encoder("Portable range normalization");
        self.record_normalization(&mut encoder)?;
        self.submit_and_wait(Some(encoder.finish()))
    }

    fn record_normalization(&mut self, encoder: &mut wgpu::CommandEncoder) -> Result<(), String> {
        self.boundaries.record(encoder);
        self.compactor
            .record_compact(
                encoder,
                &self.sorted,
                &self.start_mask,
                &self.starts,
                &self.start_count,
                self.len,
            )
            .map_err(|error| error.to_string())?;
        self.compactor
            .record_compact(
                encoder,
                &self.sorted,
                &self.end_mask,
                &self.ends,
                &self.end_count,
                self.len,
            )
            .map_err(|error| error.to_string())
    }

    fn readback_ranges(&mut self) -> Result<Vec<RangeInclusive<u32>>, String> {
        if !self.mapped_primary {
            let mut encoder = self.encoder("Range count readback copy");
            self.record_count_copy(&mut encoder);
            self.submit_and_wait(Some(encoder.finish()))?;
        }
        let count = self.read_range_count()?;
        self.readback_endpoints(count)
    }

    fn fused_ranges(&mut self, values: &[u32]) -> Result<Vec<RangeInclusive<u32>>, String> {
        if self.mapped_primary {
            self.upload(values)?;
        } else {
            self.enqueue_upload(values);
        }
        let mut encoder = self.encoder("Fused portable normalization");
        self.record_radix_sort(&mut encoder)?;
        self.record_normalization(&mut encoder)?;
        self.record_count_copy(&mut encoder);
        self.submit_and_wait(Some(encoder.finish()))?;
        let count = self.read_range_count()?;
        self.readback_endpoints(count)
    }

    fn record_count_copy(&self, encoder: &mut wgpu::CommandEncoder) {
        let Some(readback) = &self.count_readback else {
            return;
        };
        encoder.copy_buffer_to_buffer(&self.start_count, 0, readback, 0, size_of::<u32>() as u64);
        encoder.copy_buffer_to_buffer(
            &self.end_count,
            0,
            readback,
            size_of::<u32>() as u64,
            size_of::<u32>() as u64,
        );
    }

    fn read_range_count(&self) -> Result<usize, String> {
        let counts = if let Some(readback) = &self.count_readback {
            map_u32(&self.context.device, readback, 2)?
        } else {
            let (starts, ends) = map_two_u32(
                &self.context.device,
                &self.start_count,
                1,
                &self.end_count,
                1,
            )?;
            [starts[0], ends[0]].to_vec()
        };
        Ok(validate_range_count(&counts, self.len))
    }

    fn readback_endpoints(&mut self, count: usize) -> Result<Vec<RangeInclusive<u32>>, String> {
        if count == 0 {
            return Ok(Vec::new());
        }

        let endpoint_bytes = count as u64 * size_of::<u32>() as u64;
        let required_bytes = endpoint_bytes * 2;
        let largest_buffer = if self.mapped_primary {
            endpoint_bytes
        } else {
            required_bytes
        };
        if largest_buffer > self.context.device.limits().max_buffer_size {
            return Err(format!(
                "{count} ranges need a {largest_buffer}-byte readback buffer, exceeding adapter '{}' limit of {} bytes",
                self.context.adapter_info.name,
                self.context.device.limits().max_buffer_size,
            ));
        }
        let (starts, ends) = if self.mapped_primary {
            map_two_u32(&self.context.device, &self.starts, count, &self.ends, count)?
        } else {
            let recreate = self
                .range_readback
                .as_ref()
                .is_none_or(|buffer| buffer.size() < required_bytes);
            if recreate {
                self.range_readback = Some(readback_buffer(
                    &self.context.device,
                    "Compacted range readback",
                    required_bytes,
                ));
            }
            let readback = self
                .range_readback
                .as_ref()
                .expect("readback buffer initialized");
            let mut encoder = self.encoder("Compacted range readback copy");
            encoder.copy_buffer_to_buffer(&self.starts, 0, readback, 0, endpoint_bytes);
            encoder.copy_buffer_to_buffer(&self.ends, 0, readback, endpoint_bytes, endpoint_bytes);
            self.submit_and_wait(Some(encoder.finish()))?;
            let endpoints = map_u32(&self.context.device, readback, count * 2)?;
            (endpoints[..count].to_vec(), endpoints[count..].to_vec())
        };
        Ok(starts
            .iter()
            .copied()
            .zip(ends.iter().copied())
            .map(|(start, end)| start..=end)
            .collect())
    }

    fn encoder(&self, label: &'static str) -> wgpu::CommandEncoder {
        self.context
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some(label) })
    }

    fn submit_and_wait(&self, command: Option<wgpu::CommandBuffer>) -> Result<(), String> {
        let submission = self.context.queue.submit(command);
        self.context
            .device
            .poll(wgpu::PollType::Wait {
                submission_index: Some(submission),
                timeout: None,
            })
            .map_err(|error| error.to_string())?;
        Ok(())
    }
}

fn main() -> Result<(), String> {
    println!(
        "backend,distribution,input_values,output_ranges,compression_ratio,cpu_from_iter_ms,gpu_upload_ms,gpu_radix_sort_ms,gpu_normalization_ms,gpu_range_readback_ms,gpu_build_from_ranges_ms,gpu_end_to_end_ms,gpu_fused_end_to_end_ms"
    );
    for len in requested_sizes() {
        for distribution in UNSORTED_DISTRIBUTIONS {
            benchmark_case(len, distribution)?;
        }
    }
    Ok(())
}

fn benchmark_case(len: usize, distribution: Distribution) -> Result<(), String> {
    let values = generate_shuffled(len, distribution);
    let iteration_count = iterations(len);
    // Warm the CPU path, then release its result before measurement. This
    // avoids first-distribution allocator/frequency bias without leaving large
    // reference structures live during either benchmark.
    drop(black_box(values.iter().copied()).collect::<RangeSetBlaze<_>>());
    // Measure CPU construction before allocating validation structures or GPU
    // state, so the input is the only large live allocation.
    let cpu = benchmark_unsorted_cpu(&values, iteration_count);

    let mut gpu = WgpuPipeline::new(&values)?;
    eprintln!(
        "adapter={:?} backend={:?} mappable_primary_available={} memory_path={}",
        gpu.context.adapter_info.name,
        gpu.context.adapter_info.backend,
        gpu.mappable_primary_available,
        if gpu.mapped_primary {
            "mapped-primary"
        } else {
            "staged"
        },
    );
    gpu.radix_sort()?;
    gpu.normalize()?;
    drop(gpu.readback_ranges()?);
    drop(gpu.fused_ranges(&values)?);

    let mut upload = Vec::with_capacity(iteration_count);
    let mut radix_sort = Vec::with_capacity(iteration_count);
    let mut normalization = Vec::with_capacity(iteration_count);
    let mut readback = Vec::with_capacity(iteration_count);
    let mut build = Vec::with_capacity(iteration_count);
    let mut end_to_end = Vec::with_capacity(iteration_count);
    let mut fused_end_to_end = Vec::with_capacity(iteration_count);
    for _ in 0..iteration_count {
        let total_start = Instant::now();

        let stage_start = Instant::now();
        gpu.upload(black_box(&values))?;
        upload.push(stage_start.elapsed());

        let stage_start = Instant::now();
        gpu.radix_sort()?;
        radix_sort.push(stage_start.elapsed());

        let stage_start = Instant::now();
        gpu.normalize()?;
        normalization.push(stage_start.elapsed());

        let stage_start = Instant::now();
        let ranges = gpu.readback_ranges()?;
        readback.push(stage_start.elapsed());

        let stage_start = Instant::now();
        black_box(set_from_gpu_ranges(ranges));
        build.push(stage_start.elapsed());

        end_to_end.push(total_start.elapsed());
    }
    for _ in 0..iteration_count {
        let total_start = Instant::now();
        let ranges = gpu.fused_ranges(black_box(&values))?;
        black_box(set_from_gpu_ranges(ranges));
        fused_end_to_end.push(total_start.elapsed());
    }
    for samples in [
        &mut upload,
        &mut radix_sort,
        &mut normalization,
        &mut readback,
        &mut build,
        &mut end_to_end,
        &mut fused_end_to_end,
    ] {
        samples.sort_unstable();
    }

    // Keep the large validation structures out of both the CPU and GPU timing
    // windows. The timed paths above are warmed but otherwise use only the
    // original input and the state required by the implementation under test.
    let expected = values.iter().copied().collect::<RangeSetBlaze<_>>();
    let expected_ranges: Vec<_> = expected.ranges().collect();
    let range_count = expected_ranges.len();

    gpu.upload(&values)?;
    gpu.radix_sort()?;
    gpu.normalize()?;
    let actual_ranges = gpu.readback_ranges()?;
    let fused_ranges = gpu.fused_ranges(&values)?;
    validate_results(actual_ranges, &fused_ranges, &expected_ranges, &expected);

    println!(
        "{},shuffled-{},{},{},{:.6},{:.6},{:.6},{:.6},{:.6},{:.6},{:.6},{:.6},{:.6}",
        gpu.adapter_label(),
        distribution.name(),
        len,
        range_count,
        len.to_f64().expect("input length should fit f64")
            / range_count.to_f64().expect("range count should fit f64"),
        milliseconds(cpu.from_iter_total),
        milliseconds(median(&upload)),
        milliseconds(median(&radix_sort)),
        milliseconds(median(&normalization)),
        milliseconds(median(&readback)),
        milliseconds(median(&build)),
        milliseconds(median(&end_to_end)),
        milliseconds(median(&fused_end_to_end)),
    );
    Ok(())
}

fn validate_range_count(counts: &[u32], len: u32) -> usize {
    assert_eq!(
        counts[0], counts[1],
        "GPU emitted unpaired range boundaries"
    );
    let count = counts[0] as usize;
    assert!(count <= len as usize);
    count
}

fn validate_results(
    actual_ranges: Vec<RangeInclusive<u32>>,
    fused_ranges: &[RangeInclusive<u32>],
    expected_ranges: &[RangeInclusive<u32>],
    expected: &RangeSetBlaze<u32>,
) {
    assert_eq!(actual_ranges, expected_ranges);
    assert_eq!(set_from_gpu_ranges(actual_ranges), *expected);
    assert_eq!(fused_ranges, expected_ranges);
}

fn validate_buffer_size(context: &Context, len: u32, byte_len: u64) -> Result<(), String> {
    let limits = context.device.limits();
    if byte_len <= limits.max_storage_buffer_binding_size && byte_len <= limits.max_buffer_size {
        return Ok(());
    }
    Err(format!(
        "{len} items need a {byte_len}-byte storage buffer, but adapter '{}' allows a {}-byte binding and a {}-byte buffer",
        context.adapter_info.name, limits.max_storage_buffer_binding_size, limits.max_buffer_size,
    ))
}

async fn init_context(enable_mappable_primary: bool) -> Result<(Context, bool, bool), String> {
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
        .map_err(|error| format!("failed to request a GPU adapter: {error}"))?;
    let adapter_info = adapter.get_info();
    let supported_features = adapter.features();
    let mappable_primary_available =
        supported_features.contains(wgpu::Features::MAPPABLE_PRIMARY_BUFFERS);
    let mapped_primary = enable_mappable_primary
        && adapter_info.device_type == wgpu::DeviceType::IntegratedGpu
        && mappable_primary_available;
    let required_features = context_features(&adapter_info, supported_features, mapped_primary);
    let (device, queue) = adapter
        .request_device(&wgpu::DeviceDescriptor {
            label: Some("wgpu normalization experiment"),
            required_features,
            required_limits: adapter.limits(),
            memory_hints: wgpu::MemoryHints::Performance,
            ..Default::default()
        })
        .await
        .map_err(|error| format!("failed to request a GPU device: {error}"))?;
    Ok((
        Context {
            adapter_info,
            device,
            queue,
        },
        mappable_primary_available,
        mapped_primary,
    ))
}

fn context_features(
    adapter_info: &wgpu::AdapterInfo,
    supported_features: wgpu::Features,
    mapped_primary: bool,
) -> wgpu::Features {
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
    if mapped_primary {
        requested |= wgpu::Features::MAPPABLE_PRIMARY_BUFFERS;
    }
    supported_features & requested
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

fn entire_buffer_entry(binding: u32, buffer: &wgpu::Buffer) -> wgpu::BindGroupEntry<'_> {
    wgpu::BindGroupEntry {
        binding,
        resource: buffer.as_entire_binding(),
    }
}

fn storage_buffer(
    device: &wgpu::Device,
    label: &'static str,
    size: u64,
    extra_usage: wgpu::BufferUsages,
) -> wgpu::Buffer {
    device.create_buffer(&wgpu::BufferDescriptor {
        label: Some(label),
        size,
        usage: wgpu::BufferUsages::STORAGE | extra_usage,
        mapped_at_creation: false,
    })
}

fn readback_buffer(device: &wgpu::Device, label: &'static str, size: u64) -> wgpu::Buffer {
    device.create_buffer(&wgpu::BufferDescriptor {
        label: Some(label),
        size,
        usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    })
}

fn map_u32(device: &wgpu::Device, buffer: &wgpu::Buffer, count: usize) -> Result<Vec<u32>, String> {
    let byte_len = count as u64 * size_of::<u32>() as u64;
    let slice = buffer.slice(..byte_len);
    let (sender, receiver) = mpsc::channel();
    slice.map_async(wgpu::MapMode::Read, move |result| {
        drop(sender.send(result));
    });
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
    let result = {
        let mapped = slice
            .get_mapped_range()
            .map_err(|error| error.to_string())?;
        bytemuck::cast_slice::<u8, u32>(&mapped).to_vec()
    };
    buffer.unmap();
    Ok(result)
}

fn map_two_u32(
    device: &wgpu::Device,
    first: &wgpu::Buffer,
    first_count: usize,
    second: &wgpu::Buffer,
    second_count: usize,
) -> Result<(Vec<u32>, Vec<u32>), String> {
    let first_slice = first.slice(..first_count as u64 * size_of::<u32>() as u64);
    let second_slice = second.slice(..second_count as u64 * size_of::<u32>() as u64);
    let (first_sender, first_receiver) = mpsc::channel();
    let (second_sender, second_receiver) = mpsc::channel();
    first_slice.map_async(wgpu::MapMode::Read, move |result| {
        drop(first_sender.send(result));
    });
    second_slice.map_async(wgpu::MapMode::Read, move |result| {
        drop(second_sender.send(result));
    });
    device
        .poll(wgpu::PollType::Wait {
            submission_index: None,
            timeout: None,
        })
        .map_err(|error| error.to_string())?;
    first_receiver
        .recv()
        .map_err(|error| error.to_string())?
        .map_err(|error| error.to_string())?;
    second_receiver
        .recv()
        .map_err(|error| error.to_string())?
        .map_err(|error| error.to_string())?;
    let first_words = {
        let mapped = first_slice
            .get_mapped_range()
            .map_err(|error| error.to_string())?;
        bytemuck::cast_slice::<u8, u32>(&mapped).to_vec()
    };
    let second_words = {
        let mapped = second_slice
            .get_mapped_range()
            .map_err(|error| error.to_string())?;
        bytemuck::cast_slice::<u8, u32>(&mapped).to_vec()
    };
    first.unmap();
    second.unmap();
    Ok((first_words, second_words))
}

fn map_write_u32(
    device: &wgpu::Device,
    buffer: &wgpu::Buffer,
    values: &[u32],
) -> Result<(), String> {
    let slice = buffer.slice(..values.len() as u64 * size_of::<u32>() as u64);
    let (sender, receiver) = mpsc::channel();
    slice.map_async(wgpu::MapMode::Write, move |result| {
        drop(sender.send(result));
    });
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
    {
        let mut mapped = slice
            .get_mapped_range_mut()
            .map_err(|error| error.to_string())?;
        mapped.copy_from_slice(bytemuck::cast_slice(values));
    }
    buffer.unmap();
    Ok(())
}
