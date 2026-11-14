//! Device operations used by the shared sorting algorithms.
//!
//! Keep vendor intrinsics here so the algorithms can use the same operations on
//! different GPU targets. Bitonic sort uses coordinates and block barriers on both
//! vendors. Radix sort currently requires NVIDIA's 32-lane warp operations.

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
use core::arch::amdgpu::s_barrier;
#[cfg(target_arch = "amdgpu")]
pub(super) use core::arch::amdgpu::{workgroup_id_x as block_idx_x, workitem_id_x as thread_idx_x};
#[cfg(target_arch = "amdgpu")]
use core::sync::atomic::{Ordering, fence};

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
