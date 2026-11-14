//! Crate root that `build.rs` compiles for AMD GPUs when `hip_nightly_experimental` is on.
//! It exports the same radix-sort and range kernel bodies that offload uses, as plain
//! GPU kernels with fixed names, so [`Engine::Hip`] launches identical device code through
//! AMD's HIP runtime instead of through OpenMP offload. Depends only on `core`.

#![no_std]
#![feature(
    gpu_offload,
    stdarch_amdgpu,
    abi_gpu_kernel,
    core_intrinsics,
    gpu_intrinsics
)]
#![allow(internal_features)]

use core::panic::PanicInfo;

mod device;
mod key;
mod radix;
mod ranges;

const BLOCK_SIZE: u32 = 256;

#[panic_handler]
fn panic(_: &PanicInfo<'_>) -> ! {
    loop {}
}

// One exported kernel per key width, each forwarding to the shared body.
#[unsafe(no_mangle)]
pub extern "gpu-kernel" fn rsb_hip_digit_counts_u32(
    keys: *mut u32,
    len: u32,
    passes: u32,
    counts: *mut [u64; 2048],
) {
    radix::digit_counts_body::<u32>(keys, len, passes, counts);
}

#[unsafe(no_mangle)]
pub extern "gpu-kernel" fn rsb_hip_digit_counts_u64(
    keys: *mut u64,
    len: u32,
    passes: u32,
    counts: *mut [u64; 2048],
) {
    radix::digit_counts_body::<u64>(keys, len, passes, counts);
}

#[unsafe(no_mangle)]
pub extern "gpu-kernel" fn rsb_hip_radix_scatter_u32(
    keys: *mut u32,
    len: u32,
    shift: u32,
    digit_bases: *mut u64,
    bases_offset: u32,
    status: *mut u64,
    keys_out: *mut u32,
) {
    radix::radix_scatter_body::<u32>(
        keys,
        len,
        shift,
        digit_bases,
        bases_offset,
        status,
        keys_out,
    );
}

#[unsafe(no_mangle)]
pub extern "gpu-kernel" fn rsb_hip_radix_scatter_u64(
    keys: *mut u64,
    len: u32,
    shift: u32,
    digit_bases: *mut u64,
    bases_offset: u32,
    status: *mut u64,
    keys_out: *mut u64,
) {
    radix::radix_scatter_body::<u64>(
        keys,
        len,
        shift,
        digit_bases,
        bases_offset,
        status,
        keys_out,
    );
}

#[unsafe(no_mangle)]
pub extern "gpu-kernel" fn rsb_hip_count_starts_u32(
    keys: *mut u32,
    len: u32,
    gap_left: u64,
    gap_right: u64,
    has_gap: u32,
    counts: *mut u64,
) {
    ranges::count_starts_body::<u32>(keys, len, gap_left, gap_right, has_gap, counts);
}

#[unsafe(no_mangle)]
pub extern "gpu-kernel" fn rsb_hip_count_starts_u64(
    keys: *mut u64,
    len: u32,
    gap_left: u64,
    gap_right: u64,
    has_gap: u32,
    counts: *mut u64,
) {
    ranges::count_starts_body::<u64>(keys, len, gap_left, gap_right, has_gap, counts);
}

#[unsafe(no_mangle)]
pub extern "gpu-kernel" fn rsb_hip_emit_ranges_u32(
    keys: *mut u32,
    len: u32,
    gap_left: u64,
    gap_right: u64,
    has_gap: u32,
    tile_offsets: *mut u64,
    ranges: *mut u32,
) {
    ranges::emit_ranges_body::<u32>(
        keys,
        len,
        gap_left,
        gap_right,
        has_gap,
        tile_offsets,
        ranges,
    );
}

#[unsafe(no_mangle)]
pub extern "gpu-kernel" fn rsb_hip_emit_ranges_u64(
    keys: *mut u64,
    len: u32,
    gap_left: u64,
    gap_right: u64,
    has_gap: u32,
    tile_offsets: *mut u64,
    ranges: *mut u64,
) {
    ranges::emit_ranges_body::<u64>(
        keys,
        len,
        gap_left,
        gap_right,
        has_gap,
        tile_offsets,
        ranges,
    );
}

#[unsafe(no_mangle)]
pub extern "gpu-kernel" fn rsb_hip_clear_workspace(workspace: *mut u64, len: u32) {
    radix::clear_workspace_body(workspace, len);
}
