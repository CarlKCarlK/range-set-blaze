//! Device operations used by the shared sorting algorithms.
//!
//! Keep vendor intrinsics here so the algorithms can use the same operations on
//! different GPU targets. The initial implementation uses NVIDIA's 32-lane warps;
//! an AMD implementation must honor the participation and memory-ordering contracts
//! documented on these operations before enabling radix sort there.

pub(super) use super::ptx::{add_shared, match_any, shared, shuffle_up, sync_warp};
pub(super) use core::arch::nvptx::{
    _block_dim_x as block_dim_x, _block_idx_x as block_idx_x, _syncthreads as sync_block,
    _thread_idx_x as thread_idx_x,
};
