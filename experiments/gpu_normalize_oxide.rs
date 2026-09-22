//! Rust-only cuda-oxide experiment for post-sort `u32` normalization.

#[path = "gpu_normalize/common.rs"]
mod common;

use std::hint::black_box;
use std::mem::size_of;
use std::ops::RangeInclusive;
use std::sync::Arc;
use std::time::{Duration, Instant};

use common::{Distribution, benchmark_cpu, durations, iterations, median, milliseconds};
use cuda_core::simt::{CudaContext, CudaStream, DeviceBuffer, LaunchConfig};
use cuda_device::{DisjointSlice, kernel, thread};
use cuda_host::cuda_module;
#[cfg(feature = "gpu-cub")]
use range_set_blaze::RangeSetBlaze;

const CHUNK: usize = 1024;

#[cfg(feature = "gpu-cub")]
unsafe extern "C" {
    fn rsb_cub_sort_keys_u32(
        temp_storage: u64,
        temp_storage_bytes: *mut usize,
        keys_in: u64,
        keys_out: u64,
        num_items: usize,
        stream: u64,
        sorted_keys: *mut u64,
    ) -> i32;
}

#[cuda_module]
mod normalization_module {
    use super::*;

    #[kernel]
    pub(crate) unsafe fn mark_boundaries(
        input: &[u32],
        mut staged_starts: DisjointSlice<'_, u32>,
        mut staged_ends: DisjointSlice<'_, u32>,
        mut start_counts: DisjointSlice<'_, u32>,
        mut end_counts: DisjointSlice<'_, u32>,
    ) {
        let chunk_index = thread::index_1d().get();
        let chunk_start = chunk_index * CHUNK;
        if chunk_start >= input.len() {
            return;
        }
        let chunk_end = core::cmp::min(chunk_start + CHUNK, input.len());
        let mut start_count = 0usize;
        let mut end_count = 0usize;
        let mut index = chunk_start;
        while index < chunk_end {
            let value = input[index];
            let is_start = if index == 0 {
                true
            } else {
                let previous = input[index - 1];
                value != previous && (previous == u32::MAX || value != previous + 1)
            };
            if is_start {
                // SAFETY: each thread owns the disjoint CHUNK-wide staging region
                // beginning at `chunk_start`, and emits at most CHUNK values.
                unsafe {
                    *staged_starts.get_unchecked_mut(chunk_start + start_count) = value;
                }
                start_count += 1;
            }

            let is_end = if index + 1 == input.len() {
                true
            } else {
                let next = input[index + 1];
                next != value && (value == u32::MAX || next != value + 1)
            };
            if is_end {
                // SAFETY: the same per-thread staging ownership argument applies.
                unsafe {
                    *staged_ends.get_unchecked_mut(chunk_start + end_count) = value;
                }
                end_count += 1;
            }
            index += 1;
        }

        if let Some(slot) = start_counts.get_mut(thread::index_1d()) {
            *slot = start_count as u32;
        }
        if let Some(slot) = end_counts.get_mut(thread::index_1d()) {
            *slot = end_count as u32;
        }
    }

    #[kernel]
    pub(crate) unsafe fn scan_chunk_counts(
        start_counts: &[u32],
        end_counts: &[u32],
        mut start_offsets: DisjointSlice<'_, u32>,
        mut end_offsets: DisjointSlice<'_, u32>,
        mut totals: DisjointSlice<'_, u32>,
    ) {
        if thread::index_1d().get() != 0 {
            return;
        }
        let mut start_total = 0u32;
        let mut end_total = 0u32;
        let mut index = 0usize;
        while index < start_counts.len() {
            // SAFETY: exactly one GPU thread executes this loop, all indices are
            // bounded by the equally sized count and offset allocations.
            unsafe {
                *start_offsets.get_unchecked_mut(index) = start_total;
                *end_offsets.get_unchecked_mut(index) = end_total;
            }
            start_total += start_counts[index];
            end_total += end_counts[index];
            index += 1;
        }
        // SAFETY: this is the sole executing thread and totals has length two.
        unsafe {
            *totals.get_unchecked_mut(0) = start_total;
            *totals.get_unchecked_mut(1) = end_total;
        }
    }

    #[kernel]
    pub(crate) unsafe fn compact_boundaries(
        staged_starts: &[u32],
        staged_ends: &[u32],
        start_counts: &[u32],
        end_counts: &[u32],
        start_offsets: &[u32],
        end_offsets: &[u32],
        mut compact_starts: DisjointSlice<'_, u32>,
        mut compact_ends: DisjointSlice<'_, u32>,
    ) {
        let chunk_index = thread::index_1d().get();
        if chunk_index >= start_counts.len() {
            return;
        }
        let chunk_start = chunk_index * CHUNK;
        let start_count = start_counts[chunk_index] as usize;
        let end_count = end_counts[chunk_index] as usize;
        let start_offset = start_offsets[chunk_index] as usize;
        let end_offset = end_offsets[chunk_index] as usize;

        let mut local = 0usize;
        while local < start_count {
            // SAFETY: exclusive-prefix offsets assign every chunk a disjoint
            // compact output segment, and `local` is below this chunk's count.
            unsafe {
                *compact_starts.get_unchecked_mut(start_offset + local) =
                    staged_starts[chunk_start + local];
            }
            local += 1;
        }
        local = 0;
        while local < end_count {
            // SAFETY: the equivalent end-offset proof applies here.
            unsafe {
                *compact_ends.get_unchecked_mut(end_offset + local) =
                    staged_ends[chunk_start + local];
            }
            local += 1;
        }
    }
}

struct OxideNormalizer {
    context: Arc<CudaContext>,
    stream: Arc<CudaStream>,
    module: normalization_module::LoadedModule,
    input: DeviceBuffer<u32>,
    staged_starts: DeviceBuffer<u32>,
    staged_ends: DeviceBuffer<u32>,
    start_counts: DeviceBuffer<u32>,
    end_counts: DeviceBuffer<u32>,
    start_offsets: DeviceBuffer<u32>,
    end_offsets: DeviceBuffer<u32>,
    totals: DeviceBuffer<u32>,
    compact_starts: DeviceBuffer<u32>,
    compact_ends: DeviceBuffer<u32>,
    chunk_count: usize,
}

impl OxideNormalizer {
    fn new(values: &[u32]) -> Result<Self, String> {
        assert!(!values.is_empty());
        assert!(values.len() <= u32::MAX as usize);
        let context = CudaContext::new(0).map_err(|error| error.to_string())?;
        let stream = context.default_stream();
        let module = normalization_module::load(&context).map_err(|error| error.to_string())?;
        let chunk_count = values.len().div_ceil(CHUNK);
        let input = DeviceBuffer::from_host(&stream, values).map_err(|error| error.to_string())?;
        let staged_starts =
            DeviceBuffer::zeroed(&stream, values.len()).map_err(|error| error.to_string())?;
        let staged_ends =
            DeviceBuffer::zeroed(&stream, values.len()).map_err(|error| error.to_string())?;
        let start_counts =
            DeviceBuffer::zeroed(&stream, chunk_count).map_err(|error| error.to_string())?;
        let end_counts =
            DeviceBuffer::zeroed(&stream, chunk_count).map_err(|error| error.to_string())?;
        let start_offsets =
            DeviceBuffer::zeroed(&stream, chunk_count).map_err(|error| error.to_string())?;
        let end_offsets =
            DeviceBuffer::zeroed(&stream, chunk_count).map_err(|error| error.to_string())?;
        let totals = DeviceBuffer::zeroed(&stream, 2).map_err(|error| error.to_string())?;
        let compact_starts =
            DeviceBuffer::zeroed(&stream, values.len()).map_err(|error| error.to_string())?;
        let compact_ends =
            DeviceBuffer::zeroed(&stream, values.len()).map_err(|error| error.to_string())?;
        Ok(Self {
            context,
            stream,
            module,
            input,
            staged_starts,
            staged_ends,
            start_counts,
            end_counts,
            start_offsets,
            end_offsets,
            totals,
            compact_starts,
            compact_ends,
            chunk_count,
        })
    }

    fn upload(&mut self, values: &[u32]) -> Result<(), String> {
        self.input
            .copy_from_host(&self.stream, values)
            .map_err(|error| error.to_string())
    }

    fn run_device_pipeline(&mut self) -> Result<(), String> {
        let launch = LaunchConfig::for_num_elems(self.chunk_count as u32);
        // SAFETY: allocations cover every range documented by the kernels;
        // chunk staging regions and scanned output segments are disjoint.
        unsafe {
            self.module
                .mark_boundaries(
                    &self.stream,
                    launch,
                    &self.input,
                    &mut self.staged_starts,
                    &mut self.staged_ends,
                    &mut self.start_counts,
                    &mut self.end_counts,
                )
                .map_err(|error| error.to_string())?;
            self.module
                .scan_chunk_counts(
                    &self.stream,
                    LaunchConfig::for_num_elems(1),
                    &self.start_counts,
                    &self.end_counts,
                    &mut self.start_offsets,
                    &mut self.end_offsets,
                    &mut self.totals,
                )
                .map_err(|error| error.to_string())?;
            self.module
                .compact_boundaries(
                    &self.stream,
                    launch,
                    &self.staged_starts,
                    &self.staged_ends,
                    &self.start_counts,
                    &self.end_counts,
                    &self.start_offsets,
                    &self.end_offsets,
                    &mut self.compact_starts,
                    &mut self.compact_ends,
                )
                .map_err(|error| error.to_string())?;
        }
        self.stream.synchronize().map_err(|error| error.to_string())
    }

    #[cfg(feature = "gpu-cub")]
    fn run_device_pipeline_from(&mut self, input: &DeviceBuffer<u32>) -> Result<(), String> {
        let launch = LaunchConfig::for_num_elems(self.chunk_count as u32);
        // SAFETY: `input` has the same length as the persistent allocations;
        // chunk staging regions and scanned output segments are disjoint.
        unsafe {
            self.module
                .mark_boundaries(
                    &self.stream,
                    launch,
                    input,
                    &mut self.staged_starts,
                    &mut self.staged_ends,
                    &mut self.start_counts,
                    &mut self.end_counts,
                )
                .map_err(|error| error.to_string())?;
            self.module
                .scan_chunk_counts(
                    &self.stream,
                    LaunchConfig::for_num_elems(1),
                    &self.start_counts,
                    &self.end_counts,
                    &mut self.start_offsets,
                    &mut self.end_offsets,
                    &mut self.totals,
                )
                .map_err(|error| error.to_string())?;
            self.module
                .compact_boundaries(
                    &self.stream,
                    launch,
                    &self.staged_starts,
                    &self.staged_ends,
                    &self.start_counts,
                    &self.end_counts,
                    &self.start_offsets,
                    &self.end_offsets,
                    &mut self.compact_starts,
                    &mut self.compact_ends,
                )
                .map_err(|error| error.to_string())?;
        }
        self.stream.synchronize().map_err(|error| error.to_string())
    }

    fn readback_ranges(&self) -> Result<Vec<RangeInclusive<u32>>, String> {
        let totals = self
            .totals
            .to_host_vec(&self.stream)
            .map_err(|error| error.to_string())?;
        assert_eq!(totals[0], totals[1], "start/end boundary counts differ");
        let count = totals[0] as usize;
        let byte_count = count
            .checked_mul(size_of::<u32>())
            .expect("range readback byte count overflowed usize");
        let mut starts = Vec::<u32>::with_capacity(count);
        let mut ends = Vec::<u32>::with_capacity(count);
        // SAFETY: both device allocations contain `count` initialized compact
        // values, and the vectors reserve exactly that many writable elements.
        unsafe {
            cuda_core::simt::memory::memcpy_dtoh_async(
                starts.as_mut_ptr(),
                self.compact_starts.cu_deviceptr(),
                byte_count,
                self.stream.cu_stream(),
            )
            .map_err(|error| error.to_string())?;
            cuda_core::simt::memory::memcpy_dtoh_async(
                ends.as_mut_ptr(),
                self.compact_ends.cu_deviceptr(),
                byte_count,
                self.stream.cu_stream(),
            )
            .map_err(|error| error.to_string())?;
            self.stream
                .synchronize()
                .map_err(|error| error.to_string())?;
            starts.set_len(count);
            ends.set_len(count);
        }
        Ok(starts
            .into_iter()
            .zip(ends)
            .map(|(start, end)| start..=end)
            .collect())
    }
}

#[cfg(feature = "gpu-cub")]
#[derive(Clone, Copy)]
enum SortedBuffer {
    Input,
    Alternate,
}

#[cfg(feature = "gpu-cub")]
struct CubOxidePipeline {
    normalizer: OxideNormalizer,
    alternate: DeviceBuffer<u32>,
    temporary: DeviceBuffer<u8>,
    temporary_bytes: usize,
    sorted_buffer: SortedBuffer,
}

#[cfg(feature = "gpu-cub")]
impl CubOxidePipeline {
    fn new(values: &[u32]) -> Result<Self, String> {
        let normalizer = OxideNormalizer::new(values)?;
        let alternate = DeviceBuffer::zeroed(&normalizer.stream, values.len())
            .map_err(|error| error.to_string())?;
        normalizer
            .context
            .bind_to_thread()
            .map_err(|error| error.to_string())?;
        let mut temporary_bytes = 0usize;
        let mut sorted_pointer = 0u64;
        // SAFETY: the query does not dereference key or temporary-storage
        // pointers. Both key allocations and the stream remain owned by Rust.
        let status = unsafe {
            rsb_cub_sort_keys_u32(
                0,
                &mut temporary_bytes,
                normalizer.input.cu_deviceptr(),
                alternate.cu_deviceptr(),
                values.len(),
                normalizer.stream.cu_stream() as usize as u64,
                &mut sorted_pointer,
            )
        };
        check_cub(status, "temporary-storage query")?;
        let temporary = DeviceBuffer::zeroed(&normalizer.stream, temporary_bytes)
            .map_err(|error| error.to_string())?;
        Ok(Self {
            normalizer,
            alternate,
            temporary,
            temporary_bytes,
            sorted_buffer: SortedBuffer::Input,
        })
    }

    fn upload(&mut self, values: &[u32]) -> Result<(), String> {
        self.normalizer.upload(values)
    }

    fn radix_sort(&mut self) -> Result<(), String> {
        self.normalizer
            .context
            .bind_to_thread()
            .map_err(|error| error.to_string())?;
        let mut temporary_bytes = self.temporary_bytes;
        let mut sorted_pointer = 0u64;
        // SAFETY: all device allocations cover the supplied item/storage
        // counts and stay alive through synchronization on this same stream.
        let status = unsafe {
            rsb_cub_sort_keys_u32(
                self.temporary.cu_deviceptr(),
                &mut temporary_bytes,
                self.normalizer.input.cu_deviceptr(),
                self.alternate.cu_deviceptr(),
                self.normalizer.input.len(),
                self.normalizer.stream.cu_stream() as usize as u64,
                &mut sorted_pointer,
            )
        };
        check_cub(status, "radix sort")?;
        assert!(
            temporary_bytes <= self.temporary_bytes,
            "CUB requested more temporary storage than its query reported"
        );
        self.sorted_buffer = if sorted_pointer == self.normalizer.input.cu_deviceptr() {
            SortedBuffer::Input
        } else if sorted_pointer == self.alternate.cu_deviceptr() {
            SortedBuffer::Alternate
        } else {
            return Err("CUB returned an unknown sorted-key device pointer".to_owned());
        };
        self.normalizer
            .stream
            .synchronize()
            .map_err(|error| error.to_string())
    }

    fn normalize(&mut self) -> Result<(), String> {
        match self.sorted_buffer {
            SortedBuffer::Input => self.normalizer.run_device_pipeline(),
            SortedBuffer::Alternate => {
                let Self {
                    normalizer,
                    alternate,
                    ..
                } = self;
                normalizer.run_device_pipeline_from(alternate)
            }
        }
    }

    fn readback_ranges(&self) -> Result<Vec<RangeInclusive<u32>>, String> {
        self.normalizer.readback_ranges()
    }
}

#[cfg(feature = "gpu-cub")]
fn check_cub(status: i32, operation: &str) -> Result<(), String> {
    if status == 0 {
        Ok(())
    } else {
        Err(format!("CUB {operation} failed with CUDA error {status}"))
    }
}

fn main() -> Result<(), String> {
    if std::env::args().any(|argument| argument == "--unsorted") {
        #[cfg(feature = "gpu-cub")]
        return benchmark_unsorted();
        #[cfg(not(feature = "gpu-cub"))]
        return Err("--unsorted requires --features gpu-cub".to_owned());
    }
    benchmark_post_sort()
}

fn benchmark_post_sort() -> Result<(), String> {
    println!(
        "backend,distribution,input_values,output_ranges,compression_ratio,cpu_total_ms,cpu_normalize_ms,cpu_build_from_ranges_ms,gpu_device_pipeline_ms,gpu_range_readback_ms,gpu_end_to_end_ms"
    );
    for len in common::requested_sizes() {
        for distribution in Distribution::ALL {
            let values = common::generate_sorted(len, distribution);
            let expected = common::cpu_sorted_ranges(&values);
            let range_count = expected.len();
            let iteration_count = iterations(len);
            let cpu = benchmark_cpu(&values, &expected, iteration_count);

            let mut gpu = OxideNormalizer::new(&values)?;
            gpu.run_device_pipeline()?;
            let actual = gpu.readback_ranges()?;
            assert_eq!(actual, expected);

            let device = durations(iteration_count, || {
                gpu.run_device_pipeline()
                    .expect("cuda-oxide pipeline failed");
            });
            let readback = durations(iteration_count, || {
                black_box(gpu.readback_ranges().expect("cuda-oxide readback failed"));
            });
            let host_to_result = durations(iteration_count, || {
                let start = Instant::now();
                gpu.upload(black_box(&values))
                    .expect("cuda-oxide upload failed");
                gpu.run_device_pipeline()
                    .expect("cuda-oxide pipeline failed");
                let ranges = gpu.readback_ranges().expect("cuda-oxide readback failed");
                black_box(common::set_from_gpu_ranges(ranges));
                black_box(start.elapsed());
            });
            let host_to_result: Duration = median(&host_to_result);
            println!(
                "cuda-oxide,{},{},{},{:.6},{:.6},{:.6},{:.6},{:.6},{:.6},{:.6}",
                distribution.name(),
                len,
                range_count,
                len as f64 / range_count as f64,
                milliseconds(cpu.from_slice_total),
                milliseconds(cpu.normalize_ranges),
                milliseconds(cpu.build_from_ranges),
                milliseconds(median(&device)),
                milliseconds(median(&readback)),
                milliseconds(host_to_result),
            );
        }
    }
    Ok(())
}

#[cfg(feature = "gpu-cub")]
fn benchmark_unsorted() -> Result<(), String> {
    println!(
        "backend,distribution,input_values,output_ranges,compression_ratio,cpu_from_slice_ms,cpu_from_iter_ms,gpu_upload_ms,gpu_radix_sort_ms,gpu_normalization_ms,gpu_range_readback_ms,gpu_build_from_ranges_ms,gpu_end_to_end_ms"
    );
    for len in common::requested_sizes() {
        for distribution in common::UNSORTED_DISTRIBUTIONS {
            let values = common::generate_shuffled(len, distribution);
            let iteration_count = iterations(len);
            let expected = RangeSetBlaze::from_slice(&values);
            let expected_from_iter = RangeSetBlaze::from_iter(values.iter().copied());
            assert_eq!(expected_from_iter, expected);
            let expected_ranges: Vec<_> = expected.ranges().collect();
            let range_count = expected_ranges.len();
            let cpu = common::benchmark_unsorted_cpu(&values, iteration_count);

            let mut gpu = CubOxidePipeline::new(&values)?;
            gpu.radix_sort()?;
            gpu.normalize()?;
            let actual_ranges = gpu.readback_ranges()?;
            assert_eq!(actual_ranges, expected_ranges);
            let actual_set = common::set_from_gpu_ranges(actual_ranges);
            assert_eq!(actual_set, expected);

            let mut upload = Vec::with_capacity(iteration_count);
            let mut radix_sort = Vec::with_capacity(iteration_count);
            let mut normalization = Vec::with_capacity(iteration_count);
            let mut readback = Vec::with_capacity(iteration_count);
            let mut build = Vec::with_capacity(iteration_count);
            let mut end_to_end = Vec::with_capacity(iteration_count);
            for _ in 0..iteration_count {
                let total_start = Instant::now();

                let stage_start = Instant::now();
                gpu.upload(black_box(&values)).expect("GPU upload failed");
                upload.push(stage_start.elapsed());

                let stage_start = Instant::now();
                gpu.radix_sort().expect("CUB radix sort failed");
                radix_sort.push(stage_start.elapsed());

                let stage_start = Instant::now();
                gpu.normalize().expect("cuda-oxide normalization failed");
                normalization.push(stage_start.elapsed());

                let stage_start = Instant::now();
                let ranges = gpu.readback_ranges().expect("GPU range readback failed");
                readback.push(stage_start.elapsed());

                let stage_start = Instant::now();
                black_box(common::set_from_gpu_ranges(ranges));
                build.push(stage_start.elapsed());

                end_to_end.push(total_start.elapsed());
            }
            upload.sort_unstable();
            radix_sort.sort_unstable();
            normalization.sort_unstable();
            readback.sort_unstable();
            build.sort_unstable();
            end_to_end.sort_unstable();

            println!(
                "cub+cuda-oxide,shuffled-{},{},{},{:.6},{:.6},{:.6},{:.6},{:.6},{:.6},{:.6},{:.6},{:.6}",
                distribution.name(),
                len,
                range_count,
                len as f64 / range_count as f64,
                milliseconds(cpu.from_slice_total),
                milliseconds(cpu.from_iter_total),
                milliseconds(median(&upload)),
                milliseconds(median(&radix_sort)),
                milliseconds(median(&normalization)),
                milliseconds(median(&readback)),
                milliseconds(median(&build)),
                milliseconds(median(&end_to_end)),
            );
        }
    }
    Ok(())
}
