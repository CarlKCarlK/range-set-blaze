//! NVIDIA operations needed by radix sort that stdarch does not yet expose.
//! All warp operations require all 32 lanes to participate with the same mask.

use core::arch::asm;

/// # Safety
/// Use only within a kernel. Initialize the storage before reading it, synchronize
/// accesses between threads, and never retain the pointer after the kernel completes.
#[inline(always)]
pub(super) unsafe fn shared<const BYTES: usize>() -> *mut u8 {
    let pointer: u64;
    // SAFETY: this declares block-shared storage, aligned for all u64/u32 accesses.
    // The PTX scope gives each inlined call its own symbol; every thread in a block
    // receives that block's same generic-address-space pointer.
    unsafe {
        asm!(
            "{{ .shared .align 8 .b8 storage[{bytes}]; cvta.shared.u64 {pointer}, storage; }}",
            bytes = const BYTES,
            pointer = out(reg64) pointer,
            options(nostack),
        );
    }
    pointer as *mut u8
}

/// # Safety
/// Every lane of the caller's full warp must execute this operation.
#[inline(always)]
pub(super) unsafe fn match_any(value: u32) -> u32 {
    let peers: u32;
    // SAFETY: the caller guarantees full-warp participation.
    unsafe {
        asm!("match.any.sync.b32 {peers}, {value}, 0xffffffff;",
            peers = out(reg32) peers, value = in(reg32) value, options(nostack));
    }
    peers
}

/// # Safety
/// Every lane must execute with the same delta, in 1..32.
#[inline(always)]
pub(super) unsafe fn shuffle_up(value: u32, delta: u32) -> u32 {
    let previous: u32;
    // SAFETY: the caller guarantees full-warp participation and a valid delta.
    unsafe {
        asm!("shfl.sync.up.b32 {previous}, {value}, {delta}, 0, 0xffffffff;",
            previous = out(reg32) previous, value = in(reg32) value,
            delta = in(reg32) delta, options(nostack));
    }
    previous
}

/// # Safety
/// Every lane in the caller's warp must reach this barrier before another round.
#[inline(always)]
pub(super) unsafe fn sync_warp() {
    // SAFETY: the caller guarantees full-warp participation; the barrier also
    // orders the warp's shared-memory accesses before the following round.
    unsafe { asm!("bar.warp.sync 0xffffffff;", options(nostack)) };
}

/// # Safety
/// pointer must name an initialized, aligned shared-memory u32 counter. A block
/// barrier must separate updates from ordinary reads of the completed histogram.
#[inline(always)]
pub(super) unsafe fn add_shared(pointer: *mut u32, increment: u32) {
    // SAFETY: the caller provides block-shared storage. Converting its generic
    // pointer back to shared space lets the instruction use the appropriate CTA scope.
    unsafe {
        asm!("{{ .reg .u64 address; cvta.to.shared.u64 address, {pointer}; red.relaxed.cta.shared.add.u32 [address], {increment}; }}",
            pointer = in(reg64) pointer as u64, increment = in(reg32) increment, options(nostack));
    }
}

/// A release store visible to every thread on this GPU. Rust's `Ordering::Release`
/// compiles to a system-scope fence on NVPTX, which also orders against the host and
/// was radix_scatter's largest stall (Nsight Compute `membar`); look-back only needs GPU scope.
///
/// # Safety
/// pointer must name an aligned u64 in global memory that other threads access only
/// through these GPU-scope operations or atomics.
#[inline(always)]
pub(super) unsafe fn store_release_gpu(pointer: *mut u64, value: u64) {
    // SAFETY: the caller provides an aligned global word; sm_70+ supports st.release.
    unsafe {
        asm!("st.release.gpu.u64 [{pointer}], {value};",
            pointer = in(reg64) pointer as u64, value = in(reg64) value, options(nostack));
    }
}

/// An acquire load paired with [`store_release_gpu`].
///
/// # Safety
/// As for [`store_release_gpu`].
#[inline(always)]
pub(super) unsafe fn load_acquire_gpu(pointer: *const u64) -> u64 {
    let value: u64;
    // SAFETY: the caller provides an aligned global word; sm_70+ supports ld.acquire.
    unsafe {
        asm!("ld.acquire.gpu.u64 {value}, [{pointer}];",
            value = out(reg64) value, pointer = in(reg64) pointer as u64, options(nostack));
    }
    value
}
