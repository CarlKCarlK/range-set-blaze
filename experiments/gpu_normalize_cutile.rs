//! Rust-only cuTile experiment for post-sort `u32` normalization.

#[path = "gpu_normalize/common.rs"]
mod common;

use std::hint::black_box;
use std::ops::RangeInclusive;
use std::sync::Arc;
use std::time::{Duration, Instant};

use common::{Distribution, benchmark_cpu, durations, iterations, median, milliseconds};
use cutile::api;
use cutile::cuda_async::device_operation::DeviceOp;
use cutile::cuda_core::{Device, Stream, memcpy_dtoh_async, memcpy_htod_async};
use cutile::tensor::Tensor;
use cutile::tile_kernel::TileKernel;

const CHUNK: usize = 1024;

/// cuTile kernels for sorted-input range normalization.
#[cutile::module]
mod normalization_module {
    use cutile::core::*;

    /// Marks range boundaries and compacts them within each tile.
    #[cutile::entry]
    unsafe fn mark_boundaries<const CHUNK: i32>(
        input: *mut u32,
        len: i32,
        staged_starts: *mut u32,
        staged_ends: *mut u32,
        start_counts: *mut i32,
        end_counts: *mut i32,
    ) {
        // SAFETY: the host supplies live allocations covering `len`, padded
        // CHUNK-wide staging regions, and one count slot per launched tile.
        unsafe {
            let program = get_tile_block_id().0;
            let lanes: Tile<i32, { [CHUNK] }> = iota(shape![CHUNK]);
            let indices = lanes + broadcast_scalar(program * CHUNK, shape![CHUNK]);
            let len_tile = broadcast_scalar(len, shape![CHUNK]);
            let valid = lt_tile(indices, len_tile);

            let input_scalar: PointerTile<*mut u32, { [] }> = pointer_to_tile(input);
            let input_one: PointerTile<*mut u32, { [1] }> = input_scalar.reshape(shape![1]);
            let input_base: PointerTile<*mut u32, { [CHUNK] }> = input_one.broadcast(shape![CHUNK]);
            let current_ptrs = input_base.offset_tile(indices);
            let (current, _): (Tile<u32, { [CHUNK] }>, Token) = load_ptr_tko(
                current_ptrs,
                ordering::Weak,
                None::<scope::TileBlock>,
                Some(valid),
                Some(0u32),
                None,
                Latency::<0>,
            );

            let previous_indices = indices - broadcast_scalar(1i32, shape![CHUNK]);
            let has_previous = valid & gt_tile(indices, broadcast_scalar(0i32, shape![CHUNK]));
            let previous_ptrs = input_base.offset_tile(previous_indices);
            let (previous, _): (Tile<u32, { [CHUNK] }>, Token) = load_ptr_tko(
                previous_ptrs,
                ordering::Weak,
                None::<scope::TileBlock>,
                Some(has_previous),
                Some(0u32),
                None,
                Latency::<0>,
            );

            let next_indices = indices + broadcast_scalar(1i32, shape![CHUNK]);
            let has_next = lt_tile(next_indices, len_tile);
            let next_ptrs = input_base.offset_tile(next_indices);
            let (next, _): (Tile<u32, { [CHUNK] }>, Token) = load_ptr_tko(
                next_ptrs,
                ordering::Weak,
                None::<scope::TileBlock>,
                Some(has_next),
                Some(0u32),
                None,
                Latency::<0>,
            );

            let one_u32 = broadcast_scalar(1u32, shape![CHUNK]);
            let starts_after_gap =
                ne_tile(current, previous) & ne_tile(current, previous + one_u32);
            let ends_before_gap = ne_tile(next, current) & ne_tile(next, current + one_u32);
            let no_previous = eq_tile(indices, broadcast_scalar(0i32, shape![CHUNK]));
            let no_next = eq_tile(next_indices, len_tile);
            let is_start = valid & (no_previous | starts_after_gap);
            let is_end = valid & (no_next | ends_before_gap);
            let one_i32 = broadcast_scalar(1i32, shape![CHUNK]);
            let zero_i32 = broadcast_scalar(0i32, shape![CHUNK]);
            let start_flags = select(is_start, one_i32, zero_i32);
            let end_flags = select(is_end, one_i32, zero_i32);
            let start_prefix = scan_sum(start_flags, 0i32, reverse::Forward, 0i32);
            let end_prefix = scan_sum(end_flags, 0i32, reverse::Forward, 0i32);
            let start_count: Tile<i32, { [] }> = reduce_sum(start_flags, 0i32);
            let end_count: Tile<i32, { [] }> = reduce_sum(end_flags, 0i32);

            let staged_start_scalar: PointerTile<*mut u32, { [] }> = pointer_to_tile(staged_starts);
            let staged_start_one: PointerTile<*mut u32, { [1] }> =
                staged_start_scalar.reshape(shape![1]);
            let staged_start_base: PointerTile<*mut u32, { [CHUNK] }> =
                staged_start_one.broadcast(shape![CHUNK]);
            let staged_end_scalar: PointerTile<*mut u32, { [] }> = pointer_to_tile(staged_ends);
            let staged_end_one: PointerTile<*mut u32, { [1] }> =
                staged_end_scalar.reshape(shape![1]);
            let staged_end_base: PointerTile<*mut u32, { [CHUNK] }> =
                staged_end_one.broadcast(shape![CHUNK]);
            let chunk_base = broadcast_scalar(program * CHUNK, shape![CHUNK]);
            let start_destinations =
                staged_start_base.offset_tile(chunk_base + start_prefix - one_i32);
            let end_destinations = staged_end_base.offset_tile(chunk_base + end_prefix - one_i32);
            store_ptr_tko(
                start_destinations,
                current,
                ordering::Weak,
                None::<scope::TileBlock>,
                Some(is_start),
                None,
                Latency::<0>,
            );
            store_ptr_tko(
                end_destinations,
                current,
                ordering::Weak,
                None::<scope::TileBlock>,
                Some(is_end),
                None,
                Latency::<0>,
            );

            let program_tile: Tile<i32, { [] }> = broadcast_scalar(program, shape![]);
            let start_count_ptr = pointer_to_tile(start_counts).offset_tile(program_tile);
            let end_count_ptr = pointer_to_tile(end_counts).offset_tile(program_tile);
            store_ptr_tko(
                start_count_ptr,
                start_count,
                ordering::Weak,
                None::<scope::TileBlock>,
                None,
                None,
                Latency::<0>,
            );
            store_ptr_tko(
                end_count_ptr,
                end_count,
                ordering::Weak,
                None::<scope::TileBlock>,
                None,
                None,
                Latency::<0>,
            );
        }
    }

    /// Exclusively scans per-tile boundary counts and records totals.
    #[cutile::entry]
    unsafe fn scan_chunk_counts(
        start_counts: *mut i32,
        end_counts: *mut i32,
        start_offsets: *mut i32,
        end_offsets: *mut i32,
        totals: *mut i32,
        chunk_count: i32,
    ) {
        // SAFETY: one tile executes this kernel, and all count/offset arrays
        // contain `chunk_count` elements while `totals` contains two.
        unsafe {
            let start_count_base: PointerTile<*mut i32, { [] }> = pointer_to_tile(start_counts);
            let end_count_base: PointerTile<*mut i32, { [] }> = pointer_to_tile(end_counts);
            let start_offset_base: PointerTile<*mut i32, { [] }> = pointer_to_tile(start_offsets);
            let end_offset_base: PointerTile<*mut i32, { [] }> = pointer_to_tile(end_offsets);
            let mut start_total: Tile<i32, { [] }> = constant(0i32, shape![]);
            let mut end_total: Tile<i32, { [] }> = constant(0i32, shape![]);
            for index in 0i32..chunk_count {
                let index_tile: Tile<i32, { [] }> = broadcast_scalar(index, shape![]);
                let start_count_ptr: PointerTile<*mut i32, { [] }> =
                    start_count_base.offset_tile(index_tile);
                let (start_count, _): (Tile<i32, { [] }>, Token) = load_ptr_tko(
                    start_count_ptr,
                    ordering::Weak,
                    None::<scope::TileBlock>,
                    None,
                    None,
                    None,
                    Latency::<0>,
                );
                let end_count_ptr: PointerTile<*mut i32, { [] }> =
                    end_count_base.offset_tile(index_tile);
                let (end_count, _): (Tile<i32, { [] }>, Token) = load_ptr_tko(
                    end_count_ptr,
                    ordering::Weak,
                    None::<scope::TileBlock>,
                    None,
                    None,
                    None,
                    Latency::<0>,
                );
                let start_offset_ptr: PointerTile<*mut i32, { [] }> =
                    start_offset_base.offset_tile(index_tile);
                store_ptr_tko(
                    start_offset_ptr,
                    start_total,
                    ordering::Weak,
                    None::<scope::TileBlock>,
                    None,
                    None,
                    Latency::<0>,
                );
                let end_offset_ptr: PointerTile<*mut i32, { [] }> =
                    end_offset_base.offset_tile(index_tile);
                store_ptr_tko(
                    end_offset_ptr,
                    end_total,
                    ordering::Weak,
                    None::<scope::TileBlock>,
                    None,
                    None,
                    Latency::<0>,
                );
                start_total = start_total + start_count;
                end_total = end_total + end_count;
            }
            let totals_base: PointerTile<*mut i32, { [] }> = pointer_to_tile(totals);
            store_ptr_tko(
                totals_base,
                start_total,
                ordering::Weak,
                None::<scope::TileBlock>,
                None,
                None,
                Latency::<0>,
            );
            let end_total_ptr: PointerTile<*mut i32, { [] }> = addptr(totals_base, 1i32);
            store_ptr_tko(
                end_total_ptr,
                end_total,
                ordering::Weak,
                None::<scope::TileBlock>,
                None,
                None,
                Latency::<0>,
            );
        }
    }

    /// Compacts padded per-tile staging regions into global boundary arrays.
    #[cutile::entry]
    unsafe fn compact_boundaries<const CHUNK: i32>(
        staged_starts: *mut u32,
        staged_ends: *mut u32,
        start_counts: *mut i32,
        end_counts: *mut i32,
        start_offsets: *mut i32,
        end_offsets: *mut i32,
        compact_starts: *mut u32,
        compact_ends: *mut u32,
    ) {
        // SAFETY: the host keeps every allocation alive, staging is padded to
        // whole CHUNKs, and scanned offsets assign disjoint output segments.
        unsafe {
            let program = get_tile_block_id().0;
            let program_tile: Tile<i32, { [] }> = broadcast_scalar(program, shape![]);
            let start_count_base: PointerTile<*mut i32, { [] }> = pointer_to_tile(start_counts);
            let start_count_ptr: PointerTile<*mut i32, { [] }> =
                start_count_base.offset_tile(program_tile);
            let (start_count, _): (Tile<i32, { [] }>, Token) = load_ptr_tko(
                start_count_ptr,
                ordering::Weak,
                None::<scope::TileBlock>,
                None,
                None,
                None,
                Latency::<0>,
            );
            let end_count_base: PointerTile<*mut i32, { [] }> = pointer_to_tile(end_counts);
            let end_count_ptr: PointerTile<*mut i32, { [] }> =
                end_count_base.offset_tile(program_tile);
            let (end_count, _): (Tile<i32, { [] }>, Token) = load_ptr_tko(
                end_count_ptr,
                ordering::Weak,
                None::<scope::TileBlock>,
                None,
                None,
                None,
                Latency::<0>,
            );
            let start_offset_base: PointerTile<*mut i32, { [] }> = pointer_to_tile(start_offsets);
            let start_offset_ptr: PointerTile<*mut i32, { [] }> =
                start_offset_base.offset_tile(program_tile);
            let (start_offset, _): (Tile<i32, { [] }>, Token) = load_ptr_tko(
                start_offset_ptr,
                ordering::Weak,
                None::<scope::TileBlock>,
                None,
                None,
                None,
                Latency::<0>,
            );
            let end_offset_base: PointerTile<*mut i32, { [] }> = pointer_to_tile(end_offsets);
            let end_offset_ptr: PointerTile<*mut i32, { [] }> =
                end_offset_base.offset_tile(program_tile);
            let (end_offset, _): (Tile<i32, { [] }>, Token) = load_ptr_tko(
                end_offset_ptr,
                ordering::Weak,
                None::<scope::TileBlock>,
                None,
                None,
                None,
                Latency::<0>,
            );

            let lanes: Tile<i32, { [CHUNK] }> = iota(shape![CHUNK]);
            let chunk_indices = lanes + broadcast_scalar(program * CHUNK, shape![CHUNK]);
            let staged_start_scalar: PointerTile<*mut u32, { [] }> = pointer_to_tile(staged_starts);
            let staged_start_one: PointerTile<*mut u32, { [1] }> =
                staged_start_scalar.reshape(shape![1]);
            let staged_start_base: PointerTile<*mut u32, { [CHUNK] }> =
                staged_start_one.broadcast(shape![CHUNK]);
            let staged_end_scalar: PointerTile<*mut u32, { [] }> = pointer_to_tile(staged_ends);
            let staged_end_one: PointerTile<*mut u32, { [1] }> =
                staged_end_scalar.reshape(shape![1]);
            let staged_end_base: PointerTile<*mut u32, { [CHUNK] }> =
                staged_end_one.broadcast(shape![CHUNK]);
            let staged_start_ptrs: PointerTile<*mut u32, { [CHUNK] }> =
                staged_start_base.offset_tile(chunk_indices);
            let (starts, _): (Tile<u32, { [CHUNK] }>, Token) = load_ptr_tko(
                staged_start_ptrs,
                ordering::Weak,
                None::<scope::TileBlock>,
                None,
                None,
                None,
                Latency::<0>,
            );
            let staged_end_ptrs: PointerTile<*mut u32, { [CHUNK] }> =
                staged_end_base.offset_tile(chunk_indices);
            let (ends, _): (Tile<u32, { [CHUNK] }>, Token) = load_ptr_tko(
                staged_end_ptrs,
                ordering::Weak,
                None::<scope::TileBlock>,
                None,
                None,
                None,
                Latency::<0>,
            );
            let start_count_one: Tile<i32, { [1] }> = start_count.reshape(shape![1]);
            let start_count_tile: Tile<i32, { [CHUNK] }> = start_count_one.broadcast(shape![CHUNK]);
            let start_mask = lt_tile(lanes, start_count_tile);
            let end_count_one: Tile<i32, { [1] }> = end_count.reshape(shape![1]);
            let end_count_tile: Tile<i32, { [CHUNK] }> = end_count_one.broadcast(shape![CHUNK]);
            let end_mask = lt_tile(lanes, end_count_tile);
            let compact_start_scalar: PointerTile<*mut u32, { [] }> =
                pointer_to_tile(compact_starts);
            let compact_start_one: PointerTile<*mut u32, { [1] }> =
                compact_start_scalar.reshape(shape![1]);
            let compact_start_base: PointerTile<*mut u32, { [CHUNK] }> =
                compact_start_one.broadcast(shape![CHUNK]);
            let compact_end_scalar: PointerTile<*mut u32, { [] }> = pointer_to_tile(compact_ends);
            let compact_end_one: PointerTile<*mut u32, { [1] }> =
                compact_end_scalar.reshape(shape![1]);
            let compact_end_base: PointerTile<*mut u32, { [CHUNK] }> =
                compact_end_one.broadcast(shape![CHUNK]);
            let start_offset_one: Tile<i32, { [1] }> = start_offset.reshape(shape![1]);
            let start_offset_tile: Tile<i32, { [CHUNK] }> =
                start_offset_one.broadcast(shape![CHUNK]);
            let compact_start_ptrs: PointerTile<*mut u32, { [CHUNK] }> =
                compact_start_base.offset_tile(lanes + start_offset_tile);
            store_ptr_tko(
                compact_start_ptrs,
                starts,
                ordering::Weak,
                None::<scope::TileBlock>,
                Some(start_mask),
                None,
                Latency::<0>,
            );
            let end_offset_one: Tile<i32, { [1] }> = end_offset.reshape(shape![1]);
            let end_offset_tile: Tile<i32, { [CHUNK] }> = end_offset_one.broadcast(shape![CHUNK]);
            let compact_end_ptrs: PointerTile<*mut u32, { [CHUNK] }> =
                compact_end_base.offset_tile(lanes + end_offset_tile);
            store_ptr_tko(
                compact_end_ptrs,
                ends,
                ordering::Weak,
                None::<scope::TileBlock>,
                Some(end_mask),
                None,
                Latency::<0>,
            );
        }
    }
}

struct CutileNormalizer {
    stream: Arc<Stream>,
    input: Tensor<u32>,
    staged_starts: Tensor<u32>,
    staged_ends: Tensor<u32>,
    start_counts: Tensor<i32>,
    end_counts: Tensor<i32>,
    start_offsets: Tensor<i32>,
    end_offsets: Tensor<i32>,
    totals: Tensor<i32>,
    compact_starts: Tensor<u32>,
    compact_ends: Tensor<u32>,
    len: usize,
    chunk_count: usize,
}

impl CutileNormalizer {
    fn new(values: &[u32]) -> Result<Self, String> {
        assert!(!values.is_empty());
        assert!(values.len() <= i32::MAX as usize);
        let device = Device::new(0).map_err(|error| error.to_string())?;
        let stream = device.new_stream().map_err(|error| error.to_string())?;
        let input = api::copy_host_vec_to_device(&Arc::new(values.to_vec()))
            .sync_on(&stream)
            .map_err(|error| error.to_string())?;
        let chunk_count = values.len().div_ceil(CHUNK);
        let staging_len = chunk_count * CHUNK;
        let staged_starts = api::zeros(&[staging_len])
            .sync_on(&stream)
            .map_err(|error| error.to_string())?;
        let staged_ends = api::zeros(&[staging_len])
            .sync_on(&stream)
            .map_err(|error| error.to_string())?;
        let start_counts = api::zeros(&[chunk_count])
            .sync_on(&stream)
            .map_err(|error| error.to_string())?;
        let end_counts = api::zeros(&[chunk_count])
            .sync_on(&stream)
            .map_err(|error| error.to_string())?;
        let start_offsets = api::zeros(&[chunk_count])
            .sync_on(&stream)
            .map_err(|error| error.to_string())?;
        let end_offsets = api::zeros(&[chunk_count])
            .sync_on(&stream)
            .map_err(|error| error.to_string())?;
        let totals = api::zeros(&[2])
            .sync_on(&stream)
            .map_err(|error| error.to_string())?;
        let compact_starts = api::zeros(&[values.len()])
            .sync_on(&stream)
            .map_err(|error| error.to_string())?;
        let compact_ends = api::zeros(&[values.len()])
            .sync_on(&stream)
            .map_err(|error| error.to_string())?;
        Ok(Self {
            stream,
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
            len: values.len(),
            chunk_count,
        })
    }

    fn upload(&self, values: &[u32]) -> Result<(), String> {
        assert_eq!(values.len(), self.len);
        // SAFETY: the tensor owns `self.len` u32 elements and remains alive
        // until the copy on this stream has completed.
        unsafe {
            memcpy_htod_async(
                self.input.device_pointer().cu_deviceptr(),
                values.as_ptr(),
                values.len(),
                &self.stream,
            )
        }
        .map_err(|error| error.to_string())?;
        // SAFETY: this owned stream is valid, and `values` remains alive until
        // the asynchronous copy has completed.
        unsafe { self.stream.synchronize() }.map_err(|error| error.to_string())
    }

    fn run_device_pipeline(&self) -> Result<(), String> {
        let generics = vec![CHUNK.to_string()];
        unsafe {
            normalization_module::mark_boundaries(
                self.input.device_pointer(),
                self.len as i32,
                self.staged_starts.device_pointer(),
                self.staged_ends.device_pointer(),
                self.start_counts.device_pointer(),
                self.end_counts.device_pointer(),
            )
        }
        .generics(generics.clone())
        .grid((self.chunk_count as u32, 1, 1))
        .sync_on(&self.stream)
        .map_err(|error| error.to_string())?;
        unsafe {
            normalization_module::scan_chunk_counts(
                self.start_counts.device_pointer(),
                self.end_counts.device_pointer(),
                self.start_offsets.device_pointer(),
                self.end_offsets.device_pointer(),
                self.totals.device_pointer(),
                self.chunk_count as i32,
            )
        }
        .grid((1, 1, 1))
        .sync_on(&self.stream)
        .map_err(|error| error.to_string())?;
        unsafe {
            normalization_module::compact_boundaries(
                self.staged_starts.device_pointer(),
                self.staged_ends.device_pointer(),
                self.start_counts.device_pointer(),
                self.end_counts.device_pointer(),
                self.start_offsets.device_pointer(),
                self.end_offsets.device_pointer(),
                self.compact_starts.device_pointer(),
                self.compact_ends.device_pointer(),
            )
        }
        .generics(generics)
        .grid((self.chunk_count as u32, 1, 1))
        .sync_on(&self.stream)
        .map_err(|error| error.to_string())?;
        Ok(())
    }

    fn readback_ranges(&self) -> Result<Vec<RangeInclusive<u32>>, String> {
        let mut totals = [0i32; 2];
        // SAFETY: totals has room for two i32 values and the source tensor owns two.
        unsafe {
            memcpy_dtoh_async(
                totals.as_mut_ptr(),
                self.totals.device_pointer().cu_deviceptr(),
                2,
                &self.stream,
            )
        }
        .map_err(|error| error.to_string())?;
        // SAFETY: the stream is owned by this normalizer and the host totals
        // buffer remains alive through synchronization.
        unsafe { self.stream.synchronize() }.map_err(|error| error.to_string())?;
        assert_eq!(totals[0], totals[1], "start/end boundary counts differ");
        let count = totals[0] as usize;
        let mut starts = Vec::<u32>::with_capacity(count);
        let mut ends = Vec::<u32>::with_capacity(count);
        // SAFETY: the compact tensors contain `count` initialized values and
        // both host vectors reserve exactly `count` writable elements.
        unsafe {
            memcpy_dtoh_async(
                starts.as_mut_ptr(),
                self.compact_starts.device_pointer().cu_deviceptr(),
                count,
                &self.stream,
            )
            .map_err(|error| error.to_string())?;
            memcpy_dtoh_async(
                ends.as_mut_ptr(),
                self.compact_ends.device_pointer().cu_deviceptr(),
                count,
                &self.stream,
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

fn main() -> Result<(), String> {
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

            let gpu = CutileNormalizer::new(&values)?;
            gpu.run_device_pipeline()?;
            let actual = gpu.readback_ranges()?;
            assert_eq!(actual, expected);

            let device = durations(iteration_count, || {
                gpu.run_device_pipeline().expect("cuTile pipeline failed");
            });
            let readback = durations(iteration_count, || {
                black_box(gpu.readback_ranges().expect("cuTile readback failed"));
            });
            let host_to_result = durations(iteration_count, || {
                let start = Instant::now();
                gpu.upload(black_box(&values))
                    .expect("cuTile upload failed");
                gpu.run_device_pipeline().expect("cuTile pipeline failed");
                let ranges = gpu.readback_ranges().expect("cuTile readback failed");
                black_box(common::set_from_gpu_ranges(ranges));
                black_box(start.elapsed());
            });
            let host_to_result: Duration = median(&host_to_result);
            println!(
                "cutile,{},{},{},{:.6},{:.6},{:.6},{:.6},{:.6},{:.6},{:.6}",
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
