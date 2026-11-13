//! Compiled independently for the host and NVIDIA GPU by `build.rs`.
//! This file intentionally depends only on `core`, so the GPU never builds wgpu or Rayon.

#![no_std]
#![feature(gpu_offload)]
#![cfg_attr(target_arch = "nvptx64", feature(stdarch_nvptx, abi_gpu_kernel))]
#![cfg_attr(target_arch = "nvptx64", feature(asm_experimental_arch))]

#[cfg(target_arch = "nvptx64")]
use core::arch::nvptx::{_block_dim_x, _block_idx_x, _syncthreads, _thread_idx_x};
#[cfg(not(target_arch = "nvptx64"))]
use core::offload::offload;
use core::offload::offload_kernel;
use core::panic::PanicInfo;
#[cfg(target_arch = "nvptx64")]
use core::ptr::swap;
#[cfg(not(target_arch = "nvptx64"))]
use core::slice::from_raw_parts_mut;

#[cfg(not(target_arch = "nvptx64"))]
mod resident;
#[cfg(not(target_arch = "nvptx64"))]
use resident::DeviceBuffer;
#[cfg(target_arch = "nvptx64")]
mod ptx;
mod radix;

const BLOCK_SIZE: u32 = 256;

#[panic_handler]
fn panic(_: &PanicInfo<'_>) -> ! {
    #[cfg(not(target_arch = "nvptx64"))]
    {
        unsafe extern "C" {
            fn abort() -> !;
        }
        // SAFETY: abort has no preconditions and prevents unwinding across the C interface.
        unsafe { abort() }
    }
    #[cfg(target_arch = "nvptx64")]
    loop {}
}

/// One global bitonic pass, whose partners may belong to different thread blocks.
#[offload_kernel]
fn compare_exchange<const N: usize>(keys: *mut [u64; N], stride: usize, width: usize) {
    let index = thread_index();
    // SAFETY: this launch handles one pass over disjoint pairs in the resident allocation.
    unsafe { compare_pair(keys, index, stride, width) };
}

/// Build alternating ascending/descending blocks without returning to the host between
/// their 36 sorting passes. Threads outside N still participate in every block barrier.
#[offload_kernel]
fn sort_blocks<const N: usize>(keys: *mut [u64; N]) {
    let index = thread_index();
    let mut width = 2;
    while width <= N && width <= BLOCK_SIZE as usize {
        let mut stride = width / 2;
        while stride != 0 {
            // SAFETY: each pair is disjoint, and stride < BLOCK_SIZE keeps both partners
            // in this block. Every thread reaches the barrier before the next pass.
            unsafe {
                compare_pair(keys, index, stride, width);
                _syncthreads();
            }
            stride /= 2;
        }
        width *= 2;
    }
}

/// Finish a global merge's eight block-local passes in a single launch.
#[offload_kernel]
fn finish_blocks<const N: usize>(keys: *mut [u64; N], width: usize) {
    let index = thread_index();
    let mut stride = BLOCK_SIZE as usize / 2;
    while stride != 0 {
        // SAFETY: these strides keep both partners in the same block. All threads
        // participate, and each barrier makes the previous pass's writes visible.
        unsafe {
            compare_pair(keys, index, stride, width);
            _syncthreads();
        }
        stride /= 2;
    }
}

#[cfg(target_arch = "nvptx64")]
fn thread_index() -> usize {
    // SAFETY: these intrinsics only read this GPU thread's launch coordinates.
    unsafe { (_thread_idx_x() + _block_idx_x() * _block_dim_x()) as usize }
}

/// # Safety
/// `keys` points to the mapped, initialized N-element allocation. Threads must handle
/// exactly one pass over disjoint XOR pairs, with synchronization between passes.
#[cfg(target_arch = "nvptx64")]
unsafe fn compare_pair<const N: usize>(
    keys: *mut [u64; N],
    index: usize,
    stride: usize,
    width: usize,
) {
    let other = index ^ stride;
    if other <= index || other >= N {
        return;
    }
    // SAFETY: the host maps the complete initialized slice for the duration of this
    // synchronous launch. XOR is an involution, and only the smaller index handles each
    // pair, so no two threads read or write the same element in this pass. Raw element
    // pointers avoid manufacturing overlapping mutable references to the whole slice.
    unsafe {
        let left = keys.cast::<u64>().add(index);
        let right = keys.cast::<u64>().add(other);
        if (*left > *right) == (index & width == 0) {
            swap(left, right);
        }
    }
}

#[cfg(not(target_arch = "nvptx64"))]
unsafe extern "C" {
    fn omp_get_num_devices() -> i32;
}

#[cfg(not(target_arch = "nvptx64"))]
#[unsafe(no_mangle)]
pub extern "C" fn rsb_offload_devices() -> i32 {
    // SAFETY: OpenMP's runtime query takes no arguments and initializes its plugins.
    unsafe { omp_get_num_devices() }
}

/// # Safety
/// `keys` must be exclusively owned, initialized, and valid for `len` elements, where
/// `len` is a power of two in 2..=2^30. An offload GPU must be available.
#[cfg(not(target_arch = "nvptx64"))]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rsb_offload_sort(keys: *mut u64, len: usize) {
    // SAFETY: the caller guarantees initialized, exclusively owned storage for len keys.
    let host = unsafe { from_raw_parts_mut(keys, len) };
    let mut resident = DeviceBuffer::upload(host);
    // Fixed-array pointers let rustc map the exact allocation size while keeping the
    // generated gpu-kernel ABI free of fat pointers. No array is placed on the stack.
    macro_rules! sizes {
        ($($size:expr),* $(,)?) => {
            match len {
                $($size => launch::<$size>(resident.kernel_argument()),)*
                _ => panic!("invalid offload buffer length"),
            }
        };
    }
    sizes!(
        2, 4, 8, 16, 32, 64, 128, 256, 512, 1024, 2048, 4096, 8192, 16384, 32768, 65536, 131072,
        262144, 524288, 1048576, 2097152, 4194304, 8388608, 16777216, 33554432, 67108864,
        134217728, 268435456, 536870912, 1073741824
    );
    resident.download();
}

#[cfg(not(target_arch = "nvptx64"))]
fn launch<const N: usize>(keys: *mut [u64; N]) {
    let groups = (N as u32).div_ceil(BLOCK_SIZE);
    offload! {
        kernel = sort_blocks::<N>,
        workgroup_dim = [groups, 1, 1],
        thread_dim = [BLOCK_SIZE, 1, 1],
        device = 0,
        args = (keys,),
    }
    let mut width = BLOCK_SIZE as usize * 2;
    while width <= N {
        let mut stride = width / 2;
        // A block barrier cannot synchronize different blocks: large-stride passes
        // must still be separate synchronous launches, providing a grid-wide barrier.
        while stride >= BLOCK_SIZE as usize {
            offload! {
                kernel = compare_exchange::<N>,
                workgroup_dim = [groups, 1, 1],
                thread_dim = [BLOCK_SIZE, 1, 1],
                device = 0,
                args = (keys, stride, width),
            }
            stride /= 2;
        }
        offload! {
            kernel = finish_blocks::<N>,
            workgroup_dim = [groups, 1, 1],
            thread_dim = [BLOCK_SIZE, 1, 1],
            device = 0,
            args = (keys, width),
        }
        width *= 2;
    }
}
