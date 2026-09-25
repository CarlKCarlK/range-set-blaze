//! SIMD-accelerated [`RangeSetBlaze::from_slice`], on stable Rust via [`fearless_simd`].
//!
//! # How the SIMD code is structured
//!
//! `fearless_simd` enables CPU features (AVX2, AVX-512, NEON, ...) only for the code that is
//! inlined into the closure passed to [`dispatch!`]. Any SIMD operation reached through a
//! non-inlined function runs without those features and is dramatically slower (5-10x here).
//! So:
//!
//! - All SIMD work happens in [`collect_ranges`], an eager, `#[inline(always)]` kernel called
//!   directly inside [`dispatch!`].
//! - The kernel must not hand SIMD work to out-of-line code, such as a lazy iterator consumed by
//!   `collect()`, `Iterator` adapters whose `next` isn't `#[inline(always)]`, or
//!   `RangeSetBlaze::from_iter`.
//! - Anything that doesn't need SIMD (sorting, union, building the B-tree) runs after
//!   [`dispatch!`] returns.

use crate::{AssumeSortedStarts, Integer, RangeSetBlaze, UnionIter};
use alloc::vec::Vec;
use core::ops::RangeInclusive;
use fearless_simd::{Level, dispatch, prelude::*};

/// Builds a [`RangeSetBlaze`] from a slice, using SIMD to find runs of consecutive integers.
#[inline]
#[allow(clippy::redundant_pub_crate)]
pub(crate) fn from_slice<T: SimdInteger>(slice: &[T]) -> RangeSetBlaze<T> {
    // Without `std`, fearless_simd can't detect CPU features at runtime and uses the features
    // enabled at compile time (for example, via `-C target-cpu`).
    let level = Level::try_detect().unwrap_or(Level::baseline());
    let mut ranges = from_slice_ranges(level, slice);
    ranges.sort_unstable_by(|a, b| a.start().cmp(b.start()));
    RangeSetBlaze::from_sorted_disjoint(UnionIter::new(AssumeSortedStarts::new(ranges)))
}

/// Returns the ranges of `slice`, in slice order, with touching neighbors merged.
///
/// The ranges are nonempty but may overlap and are not sorted.
#[inline]
#[allow(clippy::redundant_pub_crate)]
pub(crate) fn from_slice_ranges<T: SimdInteger>(
    level: Level,
    slice: &[T],
) -> Vec<RangeInclusive<T>> {
    dispatch!(level, simd => collect_ranges(simd, slice))
}

/// An integer type whose slices can be scanned with SIMD.
#[allow(clippy::redundant_pub_crate)]
pub(crate) trait SimdInteger: Integer {
    /// The fixed-width integer with the same size and signedness as `Self`: `Self` itself,
    /// except for `isize`/`usize`, which `fearless_simd` doesn't support as lane types.
    type Lane: SimdIntElement + TryFrom<usize>;

    /// The SIMD vector used to scan a chunk of the slice. Its lane count sets the chunk size.
    type Vector<S: Simd>: SimdInt<S, Element = Self::Lane>;

    /// Converts `self` to its same-width lane value (a no-op).
    fn to_lane(self) -> Self::Lane;

    /// Loads `chunk`, which must have exactly `Self::Vector::<S>::LEN` elements, into a vector.
    fn load<S: Simd>(simd: S, chunk: &[Self]) -> Self::Vector<S>;
}

// Lane counts: 16 lanes, except 8 for 64-bit types (fearless_simd's widest vector is 512 bits).
// On narrower hardware, fearless_simd splits wide vectors into several native operations.
macro_rules! impl_simd_integer {
    ($($type:ty => $vector:ident),+ $(,)?) => {
        $(
            impl SimdInteger for $type {
                type Lane = Self;
                type Vector<S: Simd> = fearless_simd::$vector<S>;

                #[inline(always)]
                fn to_lane(self) -> Self::Lane {
                    self
                }

                #[inline(always)]
                fn load<S: Simd>(simd: S, chunk: &[Self]) -> Self::Vector<S> {
                    Self::Vector::<S>::from_slice(simd, chunk)
                }
            }
        )+
    };
}

impl_simd_integer!(
    i8 => i8x16,
    i16 => i16x16,
    i32 => i32x16,
    i64 => i64x8,
    u8 => u8x16,
    u16 => u16x16,
    u32 => u32x16,
    u64 => u64x8,
);

// `isize`/`usize` borrow the vector type of the fixed-width integer with the same size. The
// `as` casts in `to_lane` are same-width reinterpretations (guaranteed by the `cfg`); `load`
// reinterprets the whole chunk at once via `bytemuck::cast_slice`, avoiding an element-by-element
// copy, with no `unsafe` needed in this crate (`bytemuck::cast_slice` is itself implemented with
// `unsafe`, upstream).
macro_rules! impl_simd_integer_pointer_sized {
    ($($width:literal: $isize_lane:ty => $isize_vector:ident, $usize_lane:ty => $usize_vector:ident);+ $(;)?) => {
        $(
            #[cfg(target_pointer_width = $width)]
            const _: () = assert!(
                size_of::<isize>() == size_of::<$isize_lane>()
                    && size_of::<usize>() == size_of::<$usize_lane>()
            );

            #[cfg(target_pointer_width = $width)]
            impl SimdInteger for isize {
                type Lane = $isize_lane;
                type Vector<S: Simd> = fearless_simd::$isize_vector<S>;

                #[inline(always)]
                fn to_lane(self) -> Self::Lane {
                    self as $isize_lane
                }

                #[inline(always)]
                fn load<S: Simd>(simd: S, chunk: &[Self]) -> Self::Vector<S> {
                    Self::Vector::<S>::from_slice(simd, bytemuck::cast_slice(chunk))
                }
            }

            #[cfg(target_pointer_width = $width)]
            impl SimdInteger for usize {
                type Lane = $usize_lane;
                type Vector<S: Simd> = fearless_simd::$usize_vector<S>;

                #[inline(always)]
                fn to_lane(self) -> Self::Lane {
                    self as $usize_lane
                }

                #[inline(always)]
                fn load<S: Simd>(simd: S, chunk: &[Self]) -> Self::Vector<S> {
                    Self::Vector::<S>::from_slice(simd, bytemuck::cast_slice(chunk))
                }
            }
        )+
    };
}

impl_simd_integer_pointer_sized!(
    "16": i16 => i16x16, u16 => u16x16;
    "32": i32 => i32x16, u32 => u32x16;
    "64": i64 => i64x8, u64 => u64x8;
);

/// The SIMD kernel. Must stay `#[inline(always)]` and eager; see the module docs.
#[expect(
    clippy::inline_always,
    reason = "must inline into the fearless_simd dispatch target-feature closure"
)]
#[inline(always)]
fn collect_ranges<T, S>(simd: S, slice: &[T]) -> Vec<RangeInclusive<T>>
where
    T: SimdInteger,
    S: Simd,
{
    let lanes = T::Vector::<S>::LEN;
    let offsets = lane_offsets::<T, S>(simd);
    let mut ranges = RangeCollector::new();
    let mut rest = slice;
    while rest.len() >= lanes {
        let (chunk, after) = rest.split_at(lanes);
        rest = after;
        if !is_consecutive_with_offsets(simd, offsets, chunk) {
            for &value in chunk {
                ranges.push(value, value);
            }
            continue;
        }
        // Tight inner loop: extend the run while each next chunk continues it.
        let start = chunk[0];
        let mut end = chunk[lanes - 1];
        while rest.len() >= lanes {
            let (next, after) = rest.split_at(lanes);
            // `next[0] > end`, so `next[0] > T::min_value()` and `sub_one` can't overflow.
            if !(next[0] > end
                && next[0].sub_one() == end
                && is_consecutive_with_offsets(simd, offsets, next))
            {
                break;
            }
            end = next[lanes - 1];
            rest = after;
        }
        ranges.push(start, end);
    }
    for &value in rest {
        ranges.push(value, value);
    }
    ranges.finish()
}

/// Collects ranges in slice order, merging each into the previous one when they touch or overlap.
///
/// The range being extended lives in a local (in registers) rather than at the end of the
/// vector, which keeps the hot all-consecutive path free of memory writes.
struct RangeCollector<T: Integer> {
    ranges: Vec<RangeInclusive<T>>,
    current: Option<(T, T)>,
}

impl<T: Integer> RangeCollector<T> {
    #[expect(
        clippy::inline_always,
        reason = "must inline into the fearless_simd dispatch target-feature closure"
    )]
    #[inline(always)]
    const fn new() -> Self {
        Self {
            ranges: Vec::new(),
            current: None,
        }
    }

    #[expect(
        clippy::inline_always,
        reason = "must inline into the fearless_simd dispatch target-feature closure"
    )]
    #[inline(always)]
    fn push(&mut self, start: T, end: T) {
        debug_assert!(start <= end, "ranges from a slice are never empty");
        if let Some((current_start, current_end)) = &mut self.current {
            // When `start > *current_end`, `start > T::min_value()`, so `sub_one` can't overflow.
            if *current_start <= start && (start <= *current_end || start.sub_one() == *current_end)
            {
                if end > *current_end {
                    *current_end = end;
                }
                return;
            }
            self.ranges.push(*current_start..=*current_end);
        }
        self.current = Some((start, end));
    }

    #[expect(
        clippy::inline_always,
        reason = "must inline into the fearless_simd dispatch target-feature closure"
    )]
    #[inline(always)]
    fn finish(mut self) -> Vec<RangeInclusive<T>> {
        if let Some((start, end)) = self.current {
            self.ranges.push(start..=end);
        }
        self.ranges
    }
}

/// Returns `true` if `chunk` holds strictly consecutive, increasing integers.
#[expect(
    clippy::inline_always,
    reason = "must inline into the fearless_simd dispatch target-feature closure"
)]
#[inline(always)]
fn is_consecutive_with_offsets<T, S>(simd: S, offsets: T::Vector<S>, chunk: &[T]) -> bool
where
    T: SimdInteger,
    S: Simd,
{
    let first = chunk[0];
    let values = T::load(simd, chunk);
    // Lane subtraction wraps, so a chunk such as `[254u8, 255, 0, 1, ...]` also passes the SIMD
    // test. Genuinely consecutive values never wrap, so their last value is at least their
    // first; checking that rejects the wrapped chunks.
    (values - offsets)
        .simd_eq(T::Vector::<S>::splat(simd, first.to_lane()))
        .all_true()
        && first <= chunk[chunk.len() - 1]
}

/// Returns the vector `[0, 1, 2, ..., LEN - 1]`.
#[expect(
    clippy::inline_always,
    reason = "must inline into the fearless_simd dispatch target-feature closure"
)]
#[inline(always)]
fn lane_offsets<T, S>(simd: S) -> T::Vector<S>
where
    T: SimdInteger,
    S: Simd,
{
    T::Vector::<S>::from_fn(simd, |index| {
        T::Lane::try_from(index)
            .unwrap_or_else(|_| unreachable!("a SIMD lane index always fits its lane type"))
    })
}

#[cfg(test)]
#[allow(clippy::redundant_pub_crate)]
#[expect(
    clippy::inline_always,
    reason = "must inline into the fearless_simd dispatch target-feature closure"
)]
#[inline(always)]
pub(crate) fn is_consecutive<T, S>(simd: S, chunk: &[T]) -> bool
where
    T: SimdInteger,
    S: Simd,
{
    is_consecutive_with_offsets(simd, lane_offsets::<T, S>(simd), chunk)
}

/// Every SIMD level this machine and build can run, for testing each backend.
#[cfg(test)]
#[allow(clippy::redundant_pub_crate)]
pub(crate) fn testable_levels() -> Vec<Level> {
    let mut levels = alloc::vec![Level::baseline(), Level::fallback()];
    if let Some(detected) = Level::try_detect() {
        levels.push(detected);
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        levels.extend(
            [
                detected.as_sse2().map(Level::Sse2),
                detected.as_sse4_2().map(Level::Sse4_2),
                detected.as_avx2().map(Level::Avx2),
                detected.as_avx512().map(Level::Avx512),
            ]
            .into_iter()
            .flatten(),
        );
    }
    levels
}
