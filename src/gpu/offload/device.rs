//! Device operations used by the shared sorting algorithms.
//!
//! Keep vendor intrinsics here so the algorithms can use the same operations on
//! different GPU targets. Both sorts run on both vendors. Radix sort's warp operations
//! assume 32 lanes: NVIDIA warps, and AMD RDNA wave32 (the LLVM default for gfx10+).

#[cfg(target_arch = "nvptx64")]
pub(super) use super::ptx::{add_shared, match_any, shared, shuffle_up, sync_warp};
#[cfg(target_arch = "nvptx64")]
pub(super) use core::arch::nvptx::{
    _block_dim_x as block_dim_x, _block_idx_x as block_idx_x, _syncthreads as sync_block,
    _thread_idx_x as thread_idx_x,
};

#[cfg(target_arch = "amdgpu")]
use super::BLOCK_SIZE;
#[cfg(target_arch = "amdgpu")]
use core::arch::amdgpu::{ballot, ds_bpermute, mbcnt_lo, s_barrier, wave_barrier};
#[cfg(target_arch = "amdgpu")]
pub(super) use core::arch::amdgpu::{workgroup_id_x as block_idx_x, workitem_id_x as thread_idx_x};
#[cfg(target_arch = "amdgpu")]
use core::intrinsics::gpu::gpu_launch_sized_workgroup_mem;
#[cfg(target_arch = "amdgpu")]
use core::sync::atomic::{AtomicU32, Ordering, fence};

#[cfg(target_arch = "amdgpu")]
pub(super) const fn block_dim_x() -> u32 {
    // All bitonic launches explicitly request BLOCK_SIZE threads on both vendors.
    BLOCK_SIZE
}

#[cfg(target_arch = "amdgpu")]
pub(super) fn sync_block() {
    // Bitonic partners communicate through global memory, including across waves.
    // Publish writes before the workgroup barrier and acquire peers' writes after it.
    fence(Ordering::Release);
    s_barrier();
    fence(Ordering::Acquire);
}

// Radix sort's lane masks are u32, so a 64-lane wavefront would silently break them.
#[cfg(all(target_arch = "amdgpu", target_feature = "wavefrontsize64"))]
compile_error!("offload radix sort requires wave32; do not enable wavefrontsize64");

/// Block-shared (LDS) storage allocated at launch time.
///
/// # Safety
/// Use only within a kernel launched with `dyn_cache` of at least `BYTES`. Initialize
/// the storage before reading it, synchronize accesses between threads, and never
/// retain the pointer after the kernel completes.
#[cfg(target_arch = "amdgpu")]
#[inline(always)]
pub(super) unsafe fn shared<const BYTES: usize>() -> *mut u8 {
    // u64 alignment covers every u64/u32 access the kernels make. Every call in a
    // workgroup returns the same base address.
    gpu_launch_sized_workgroup_mem::<u64>().cast()
}

/// The lanes of this wave whose `value` equals this lane's.
///
/// # Safety
/// Every lane of the caller's 32-lane wave must execute this operation, and `value`
/// must be below 512. (Radix digits are at most 256, the out-of-range tail marker.)
#[cfg(target_arch = "amdgpu")]
#[inline(always)]
pub(super) unsafe fn match_any(value: u32) -> u32 {
    // One ballot per bit: keep the lanes that agree with this lane on every bit.
    let mut peers = u32::MAX;
    let mut bit = 0;
    while bit < 9 {
        let set = value & (1 << bit) != 0;
        let lanes = ballot(set) as u32;
        peers &= if set { lanes } else { !lanes };
        bit += 1;
    }
    peers
}

/// The value held by the lane `delta` below this one; lanes below `delta` get their own.
///
/// # Safety
/// Every lane of the caller's 32-lane wave must execute with the same delta, in 1..32.
#[cfg(target_arch = "amdgpu")]
#[inline(always)]
pub(super) unsafe fn shuffle_up(value: u32, delta: u32) -> u32 {
    let lane = mbcnt_lo(u32::MAX, 0);
    let source = if lane >= delta { lane - delta } else { lane };
    // SAFETY: the caller guarantees full-wave participation. ds_bpermute addresses
    // lanes in bytes, four per lane.
    unsafe { ds_bpermute(source << 2, value) }
}

/// Orders this wave's shared-memory accesses before the following round.
///
/// # Safety
/// Every lane in the caller's wave must reach this barrier before another round.
#[cfg(target_arch = "amdgpu")]
#[inline(always)]
pub(super) unsafe fn sync_warp() {
    // A wave already runs in lockstep; the fences keep the compiler and memory system
    // from moving one round's shared reads and writes past another's.
    fence(Ordering::Release);
    wave_barrier();
    fence(Ordering::Acquire);
}

/// # Safety
/// pointer must name an initialized, aligned shared-memory u32 counter. A block
/// barrier must separate updates from ordinary reads of the completed histogram.
#[cfg(target_arch = "amdgpu")]
#[inline(always)]
pub(super) unsafe fn add_shared(pointer: *mut u32, increment: u32) {
    // SAFETY: the caller provides an aligned counter that only atomics update until
    // the following block barrier.
    unsafe { AtomicU32::from_ptr(pointer) }.fetch_add(increment, Ordering::Relaxed);
}
