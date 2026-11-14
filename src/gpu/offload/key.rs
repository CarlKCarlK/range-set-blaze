//! The key widths the sort and range kernels handle: `u32` when the element type
//! fits (half the memory traffic), else `u64`.

pub(super) trait Key: Copy + Eq {
    const ZERO: Self;
    const MAX: Self;

    /// The 8-bit radix digit starting at bit `shift`.
    fn digit(self, shift: u32) -> u32;

    /// `self + 1`; callers check `self != MAX` first.
    fn next(self) -> Self;

    /// A gap key, passed to kernels as u64. For u32 keys it must fit.
    fn from_u64(value: u64) -> Self;
}

impl Key for u32 {
    const ZERO: Self = 0;
    const MAX: Self = u32::MAX;

    #[inline(always)]
    fn digit(self, shift: u32) -> u32 {
        (self >> shift) & 255
    }

    #[inline(always)]
    fn next(self) -> Self {
        self + 1
    }

    #[inline(always)]
    fn from_u64(value: u64) -> Self {
        value as u32
    }
}

impl Key for u64 {
    const ZERO: Self = 0;
    const MAX: Self = u64::MAX;

    #[inline(always)]
    fn digit(self, shift: u32) -> u32 {
        ((self >> shift) & 255) as u32
    }

    #[inline(always)]
    fn next(self) -> Self {
        self + 1
    }

    #[inline(always)]
    fn from_u64(value: u64) -> Self {
        value
    }
}
