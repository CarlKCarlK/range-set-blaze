//! Sorting-only CPU versus wgpu/Lampshade benchmark.

use std::hint::black_box;
use std::mem::size_of;
use std::sync::mpsc;
use std::time::{Duration, Instant};

use lampshade::{Context, Sorter};

const DEFAULT_SIZES: [usize; 12] = [
    100, 300, 1_000, 3_000, 10_000, 30_000, 100_000, 300_000, 1_000_000, 3_000_000, 10_000_000,
    30_000_000,
];
const RANDOM_SEED: u64 = 0x6a09_e667_f3bc_c909;

fn main() -> Result<(), String> {
    let context = pollster::block_on(init_context())?;
    if context.adapter_info.device_type == wgpu::DeviceType::Cpu {
        return Err("wgpu selected a CPU adapter; a hardware GPU is required".to_owned());
    }
    eprintln!(
        "adapter={:?} backend={:?} device_type={:?} driver={:?} driver_info={:?}",
        context.adapter_info.name,
        context.adapter_info.backend,
        context.adapter_info.device_type,
        context.adapter_info.driver,
        context.adapter_info.driver_info,
    );
    eprintln!("random_seed=0x{RANDOM_SEED:016x} cpu_sort=sort_unstable");
    println!(
        "n,repetitions,cpu_sort_median_ms,gpu_upload_median_ms,gpu_radix_sort_median_ms,gpu_readback_median_ms,gpu_end_to_end_median_ms,cpu_over_gpu_end_to_end"
    );

    for size in requested_sizes()? {
        let values = random_values(size);
        match benchmark_size(&context, &values) {
            Ok(row) => println!(
                "{},{},{:.6},{:.6},{:.6},{:.6},{:.6},{:.6}",
                size,
                row.repetitions,
                milliseconds(row.cpu),
                milliseconds(row.upload),
                milliseconds(row.sort),
                milliseconds(row.readback),
                milliseconds(row.end_to_end),
                row.cpu.as_secs_f64() / row.end_to_end.as_secs_f64(),
            ),
            Err(error) => {
                eprintln!("stopping before n={size}: {error}");
                break;
            }
        }
    }
    Ok(())
}

struct BenchmarkRow {
    repetitions: usize,
    cpu: Duration,
    upload: Duration,
    sort: Duration,
    readback: Duration,
    end_to_end: Duration,
}

fn benchmark_size(context: &Context, values: &[u32]) -> Result<BenchmarkRow, String> {
    let repetitions = repetitions(values.len());
    let mut pipeline = SortPipeline::new(context, values.len())?;

    let mut cpu_work = values.to_vec();
    cpu_work.sort_unstable();
    let expected = cpu_work.clone();
    pipeline.upload(values)?;
    pipeline.radix_sort()?;
    let warm_result = pipeline.readback()?;
    if warm_result != expected {
        return Err("GPU warm-up sort differed from CPU sort".to_owned());
    }
    drop(pipeline.end_to_end(values)?);

    let mut cpu_samples = Vec::with_capacity(repetitions);
    for _ in 0..repetitions {
        cpu_work.clone_from_slice(values);
        let start = Instant::now();
        black_box(&mut cpu_work).sort_unstable();
        cpu_samples.push(start.elapsed());
    }

    let mut upload_samples = Vec::with_capacity(repetitions);
    let mut sort_samples = Vec::with_capacity(repetitions);
    let mut readback_samples = Vec::with_capacity(repetitions);
    for _ in 0..repetitions {
        let start = Instant::now();
        pipeline.upload(black_box(values))?;
        upload_samples.push(start.elapsed());

        let start = Instant::now();
        pipeline.radix_sort()?;
        sort_samples.push(start.elapsed());

        let start = Instant::now();
        black_box(pipeline.readback()?);
        readback_samples.push(start.elapsed());
    }

    let mut end_to_end_samples = Vec::with_capacity(repetitions);
    for _ in 0..repetitions {
        let start = Instant::now();
        let result = pipeline.end_to_end(black_box(values))?;
        end_to_end_samples.push(start.elapsed());
        black_box(result);
    }

    let actual = pipeline.end_to_end(values)?;
    if actual != expected {
        return Err("GPU sort differed from CPU sort".to_owned());
    }

    Ok(BenchmarkRow {
        repetitions,
        cpu: median(&mut cpu_samples),
        upload: median(&mut upload_samples),
        sort: median(&mut sort_samples),
        readback: median(&mut readback_samples),
        end_to_end: median(&mut end_to_end_samples),
    })
}

struct SortPipeline<'a> {
    context: &'a Context,
    sorter: Sorter,
    input: wgpu::Buffer,
    sorted: wgpu::Buffer,
    readback: Option<wgpu::Buffer>,
    len: u32,
    byte_len: u64,
    mapped_primary: bool,
}

impl<'a> SortPipeline<'a> {
    fn new(context: &'a Context, len: usize) -> Result<Self, String> {
        let len = u32::try_from(len).map_err(|_| "input exceeds u32::MAX items")?;
        let byte_len = u64::from(len) * size_of::<u32>() as u64;
        let limits = context.device.limits();
        if byte_len > limits.max_storage_buffer_binding_size || byte_len > limits.max_buffer_size {
            return Err(format!(
                "{byte_len} bytes exceeds the adapter's {}-byte storage binding or {}-byte buffer limit",
                limits.max_storage_buffer_binding_size, limits.max_buffer_size,
            ));
        }
        let mapped_primary = uses_mappable_primary_buffers(context);
        let input_extra = if mapped_primary {
            wgpu::BufferUsages::MAP_WRITE
        } else {
            wgpu::BufferUsages::COPY_DST
        };
        let sorted_extra = if mapped_primary {
            wgpu::BufferUsages::MAP_READ
        } else {
            wgpu::BufferUsages::COPY_SRC
        };
        let input = storage_buffer(
            &context.device,
            "Sort benchmark input",
            byte_len,
            input_extra,
        );
        let sorted = storage_buffer(
            &context.device,
            "Sort benchmark output",
            byte_len,
            sorted_extra,
        );
        let readback = (!mapped_primary).then(|| {
            context.device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Sort benchmark readback"),
                size: byte_len,
                usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            })
        });
        Ok(Self {
            context,
            sorter: Sorter::from_context(context),
            input,
            sorted,
            readback,
            len,
            byte_len,
            mapped_primary,
        })
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
        let mut encoder = self.encoder("Sorting-only radix sort");
        self.record_sort(&mut encoder)?;
        self.submit_and_wait(Some(encoder.finish()))
    }

    fn readback(&self) -> Result<Vec<u32>, String> {
        self.readback.as_ref().map_or_else(
            || map_read_u32(&self.context.device, &self.sorted, self.len as usize),
            |readback| {
                let mut encoder = self.encoder("Sorting-only readback");
                encoder.copy_buffer_to_buffer(&self.sorted, 0, readback, 0, self.byte_len);
                self.context.queue.submit(Some(encoder.finish()));
                map_read_u32(&self.context.device, readback, self.len as usize)
            },
        )
    }

    fn end_to_end(&mut self, values: &[u32]) -> Result<Vec<u32>, String> {
        if self.mapped_primary {
            map_write_u32(&self.context.device, &self.input, values)?;
        } else {
            self.enqueue_upload(values);
        }
        let mut encoder = self.encoder("Sorting-only end to end");
        self.record_sort(&mut encoder)?;
        if let Some(readback) = &self.readback {
            encoder.copy_buffer_to_buffer(&self.sorted, 0, readback, 0, self.byte_len);
        }
        self.context.queue.submit(Some(encoder.finish()));
        let result_buffer = self.readback.as_ref().unwrap_or(&self.sorted);
        map_read_u32(&self.context.device, result_buffer, self.len as usize)
    }

    fn record_sort(&mut self, encoder: &mut wgpu::CommandEncoder) -> Result<(), String> {
        self.sorter
            .record_sort(encoder, &self.input, &self.sorted, self.len)
            .map_err(|error| error.to_string())
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

async fn init_context() -> Result<Context, String> {
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
    let required_features = context_features(&adapter_info, supported_features);
    let (device, queue) = adapter
        .request_device(&wgpu::DeviceDescriptor {
            label: Some("sorting-only benchmark"),
            required_features,
            required_limits: adapter.limits(),
            memory_hints: wgpu::MemoryHints::Performance,
            ..Default::default()
        })
        .await
        .map_err(|error| format!("failed to request a GPU device: {error}"))?;
    Ok(Context {
        adapter_info,
        device,
        queue,
    })
}

fn context_features(
    adapter_info: &wgpu::AdapterInfo,
    supported_features: wgpu::Features,
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
    if adapter_info.device_type == wgpu::DeviceType::IntegratedGpu {
        requested |= wgpu::Features::MAPPABLE_PRIMARY_BUFFERS;
    }
    supported_features & requested
}

fn uses_mappable_primary_buffers(context: &Context) -> bool {
    context.adapter_info.device_type == wgpu::DeviceType::IntegratedGpu
        && context
            .device
            .features()
            .contains(wgpu::Features::MAPPABLE_PRIMARY_BUFFERS)
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

fn map_write_u32(
    device: &wgpu::Device,
    buffer: &wgpu::Buffer,
    values: &[u32],
) -> Result<(), String> {
    let slice = buffer.slice(..values.len() as u64 * size_of::<u32>() as u64);
    wait_for_map(device, &slice, wgpu::MapMode::Write)?;
    {
        let mut mapped = slice
            .get_mapped_range_mut()
            .map_err(|error| error.to_string())?;
        mapped.copy_from_slice(bytemuck::cast_slice(values));
    }
    buffer.unmap();
    Ok(())
}

fn map_read_u32(
    device: &wgpu::Device,
    buffer: &wgpu::Buffer,
    count: usize,
) -> Result<Vec<u32>, String> {
    let slice = buffer.slice(..count as u64 * size_of::<u32>() as u64);
    wait_for_map(device, &slice, wgpu::MapMode::Read)?;
    let result = {
        let mapped = slice
            .get_mapped_range()
            .map_err(|error| error.to_string())?;
        bytemuck::cast_slice::<u8, u32>(&mapped).to_vec()
    };
    buffer.unmap();
    Ok(result)
}

fn wait_for_map(
    device: &wgpu::Device,
    slice: &wgpu::BufferSlice<'_>,
    mode: wgpu::MapMode,
) -> Result<(), String> {
    let (sender, receiver) = mpsc::channel();
    slice.map_async(mode, move |result| {
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
        .map_err(|error| error.to_string())
}

fn random_values(len: usize) -> Vec<u32> {
    let mut state = RANDOM_SEED ^ len as u64;
    (0..len)
        .map(|_| {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            let bytes = state.to_le_bytes();
            u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]])
        })
        .collect()
}

fn requested_sizes() -> Result<Vec<usize>, String> {
    let sizes: Result<Vec<_>, _> = std::env::args()
        .skip(1)
        .map(|value| value.parse::<usize>())
        .collect();
    let sizes = sizes.map_err(|error| format!("sizes must be positive integers: {error}"))?;
    if sizes.is_empty() {
        Ok(DEFAULT_SIZES.to_vec())
    } else if sizes.contains(&0) {
        Err("sizes must be positive integers".to_owned())
    } else {
        Ok(sizes)
    }
}

const fn repetitions(len: usize) -> usize {
    match len {
        0..=10_000 => 31,
        10_001..=100_000 => 21,
        100_001..=1_000_000 => 11,
        1_000_001..=10_000_000 => 5,
        _ => 3,
    }
}

fn median(samples: &mut [Duration]) -> Duration {
    samples.sort_unstable();
    samples[samples.len() / 2]
}

fn milliseconds(duration: Duration) -> f64 {
    duration.as_secs_f64() * 1_000.0
}
