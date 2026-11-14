//! A host allocation associated with GPU storage for a sequence of synchronous kernels.

use core::ffi::c_void;
use core::mem::size_of_val;

unsafe extern "C" {
    fn omp_get_initial_device() -> i32;
    fn omp_target_alloc(size: usize, device: i32) -> *mut c_void;
    fn omp_target_free(pointer: *mut c_void, device: i32);
    fn omp_target_memcpy(
        destination: *mut c_void,
        source: *const c_void,
        bytes: usize,
        destination_offset: usize,
        source_offset: usize,
        destination_device: i32,
        source_device: i32,
    ) -> i32;
    fn omp_target_associate_ptr(
        host: *const c_void,
        device: *const c_void,
        bytes: usize,
        offset: usize,
        device_number: i32,
    ) -> i32;
    fn omp_target_disassociate_ptr(host: *const c_void, device: i32) -> i32;
}

/// The mutable borrow prevents host access until the final download completes.
pub(super) struct DeviceBuffer<'a, T: Copy = u64> {
    host: &'a mut [T],
    device: *mut c_void,
    associated: bool,
}

impl<'a, T: Copy> DeviceBuffer<'a, T> {
    pub(super) fn upload(host: &'a mut [T]) -> Self {
        Self::allocate(host, true)
    }

    /// # Safety
    /// The caller must initialize every device element before it is read by a kernel
    /// or downloaded. The host slice itself must remain initialized throughout.
    pub(super) unsafe fn scratch(host: &'a mut [T]) -> Self {
        Self::allocate(host, false)
    }

    fn allocate(host: &'a mut [T], upload: bool) -> Self {
        assert!(!host.is_empty(), "offload buffer must not be empty");
        // SAFETY: device 0 has been checked by the caller. The initialized host slice
        // stays exclusively borrowed until Drop, and the allocation has its exact size.
        // Copy before association: an associated device pointer must not be used directly.
        let device = unsafe {
            let device = omp_target_alloc(size_of_val(host), 0);
            assert!(!device.is_null(), "offload allocation failed");
            if upload {
                assert_eq!(
                    omp_target_memcpy(
                        device,
                        host.as_ptr().cast(),
                        size_of_val(host),
                        0,
                        0,
                        0,
                        omp_get_initial_device()
                    ),
                    0,
                    "offload upload failed",
                );
            }
            assert_eq!(
                omp_target_associate_ptr(host.as_ptr().cast(), device, size_of_val(host), 0, 0),
                0,
                "offload pointer association failed",
            );
            device
        };
        Self {
            host,
            device,
            associated: true,
        }
    }

    pub(super) fn kernel_argument<const N: usize>(&mut self) -> *mut [T; N] {
        assert_eq!(
            self.host.len(),
            N,
            "offload array size must match its mapping"
        );
        self.host.as_mut_ptr().cast()
    }

    /// The whole allocation is already mapped, so a kernel's raw pointer argument
    /// may look up its base while indexing the full explicitly supplied length.
    pub(super) fn pointer(&mut self) -> *mut T {
        self.host.as_mut_ptr()
    }

    pub(super) fn download(mut self) {
        self.disassociate();
        // SAFETY: all launches are synchronous and have finished. Disassociation makes
        // the still-allocated device pointer usable for this one final copy. The host
        // slice is initialized, writable, exclusively borrowed, and has the exact size.
        unsafe {
            assert_eq!(
                omp_target_memcpy(
                    self.host.as_mut_ptr().cast(),
                    self.device,
                    size_of_val(self.host),
                    0,
                    0,
                    omp_get_initial_device(),
                    0
                ),
                0,
                "offload download failed",
            );
        }
    }

    /// Downloads only the first `len` words, for results shorter than the allocation.
    pub(super) fn download_prefix(mut self, len: usize) {
        assert!(len <= self.host.len(), "offload prefix exceeds its buffer");
        self.disassociate();
        // SAFETY: as in download, for the first len words of the same allocation, which
        // the caller's synchronous kernels have finished initializing.
        unsafe {
            assert_eq!(
                omp_target_memcpy(
                    self.host.as_mut_ptr().cast(),
                    self.device,
                    len * size_of::<T>(),
                    0,
                    0,
                    omp_get_initial_device(),
                    0
                ),
                0,
                "offload download failed",
            );
        }
    }

    pub(super) fn download_from(mut self, mut source: Self) {
        assert_eq!(
            self.host.len(),
            source.host.len(),
            "offload copy lengths differ"
        );
        self.disassociate();
        source.disassociate();
        // SAFETY: both synchronous kernel buffers are exclusively borrowed, disassociated,
        // equally sized, and still allocated. The scatter initialized every source key.
        unsafe {
            assert_eq!(
                omp_target_memcpy(
                    self.host.as_mut_ptr().cast(),
                    source.device,
                    size_of_val(self.host),
                    0,
                    0,
                    omp_get_initial_device(),
                    0
                ),
                0,
                "offload download failed"
            );
        }
    }

    fn disassociate(&mut self) {
        // SAFETY: this exact host address was associated once in upload and is still
        // exclusively borrowed. No kernel is running when the association is removed.
        unsafe {
            assert_eq!(
                omp_target_disassociate_ptr(self.host.as_ptr().cast(), 0),
                0,
                "offload pointer disassociation failed"
            );
        }
        self.associated = false;
    }
}

impl<T: Copy> Drop for DeviceBuffer<'_, T> {
    fn drop(&mut self) {
        if self.associated {
            self.disassociate();
        }
        // SAFETY: this allocation was returned by omp_target_alloc for device 0,
        // is no longer associated, and is freed exactly once after all launches finish.
        unsafe { omp_target_free(self.device, 0) };
    }
}
