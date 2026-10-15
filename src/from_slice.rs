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

use crate::{AssumeSortedStarts, Integer, RangeMapBlaze, RangeSetBlaze, UnionIter};
use alloc::vec::Vec;
use fearless_simd::{Level, dispatch, prelude::*};

/// Builds a [`RangeSetBlaze`] from a slice, using SIMD to find runs of consecutive integers.
#[inline]
#[allow(clippy::redundant_pub_crate)]
pub(crate) fn from_slice<T: SimdInteger>(slice: &[T]) -> RangeSetBlaze<T> {
    // Without `std`, fearless_simd can't detect CPU features at runtime and uses the features
    // enabled at compile time (for example, via `-C target-cpu`).
    let level = Level::try_detect().unwrap_or(Level::baseline());
    from_slice_at_level(level, slice)
}

/// How many SIMD chunks the run-extending loop checks per iteration.
///
/// On clumpy data, 4 was about 1.2-1.6x faster than 1 with AVX2 and AVX-512 (8 added nothing).
/// It costs a few percent when runs are short (around 100 values), since a block rarely fits.
const UNROLL: usize = 4;

/// Like [`from_slice`], but at a given SIMD `level` instead of the detected one.
#[inline]
#[allow(clippy::redundant_pub_crate)]
pub(crate) fn from_slice_at_level<T: SimdInteger>(level: Level, slice: &[T]) -> RangeSetBlaze<T> {
    from_slice_at_level_unrolled::<T, UNROLL>(level, slice)
}

/// Like [`from_slice_at_level`], but with a given unroll factor (see [`UNROLL`]).
#[inline]
#[allow(clippy::redundant_pub_crate)]
pub(crate) fn from_slice_at_level_unrolled<T: SimdInteger, const N: usize>(
    level: Level,
    slice: &[T],
) -> RangeSetBlaze<T> {
    let mut ranges = from_slice_ranges_unrolled::<T, N>(level, slice);
    ranges.sort_unstable_by_key(|&(start, _)| start);
    RangeSetBlaze::from_sorted_disjoint(UnionIter::new(AssumeSortedStarts::new(
        ranges.into_iter().map(|(start, end)| start..=end),
    )))
}

/// Returns the ranges of `slice`, in slice order, with touching neighbors merged.
///
/// The ranges are nonempty but may overlap and are not sorted.
#[cfg(test)]
#[inline]
#[allow(clippy::redundant_pub_crate)]
pub(crate) fn from_slice_ranges<T: SimdInteger>(
    level: Level,
    slice: &[T],
) -> Vec<core::ops::RangeInclusive<T>> {
    from_slice_ranges_unrolled::<T, UNROLL>(level, slice)
        .into_iter()
        .map(|(start, end)| start..=end)
        .collect()
}

/// Like [`from_slice_ranges`], but with a given unroll factor (see [`UNROLL`]), and returning
/// `(start, end)` pairs, which are smaller than `RangeInclusive` and so faster to collect and sort.
// TODO When the MSRV supports the new `Copy` range types (`core::range::RangeInclusive`, which,
// unlike `core::ops::RangeInclusive`, has no iterator state), use them here instead of pairs.
#[inline]
fn from_slice_ranges_unrolled<T: SimdInteger, const N: usize>(
    level: Level,
    slice: &[T],
) -> Vec<(T, T)> {
    dispatch!(level, simd => collect_ranges::<T, _, N>(simd, slice))
}

/// Builds a [`RangeMapBlaze`] from a slice of keys and a same-length slice of values, using SIMD
/// to find runs of consecutive keys. Values are compared with scalar code.
#[inline]
#[allow(clippy::redundant_pub_crate)]
pub(crate) fn from_slices<T: SimdInteger, V: Eq + Clone>(
    keys: &[T],
    values: &[V],
) -> RangeMapBlaze<T, V> {
    let level = Level::try_detect().unwrap_or(Level::baseline());
    from_slices_at_level(level, keys, values)
}

/// Like [`from_slices`], but at a given SIMD `level` instead of the detected one.
#[inline]
#[allow(clippy::redundant_pub_crate)]
pub(crate) fn from_slices_at_level<T: SimdInteger, V: Eq + Clone>(
    level: Level,
    keys: &[T],
    values: &[V],
) -> RangeMapBlaze<T, V> {
    assert_eq!(
        keys.len(),
        values.len(),
        "keys and values must have the same length"
    );
    let runs = dispatch!(level, simd => collect_runs::<T, V, _, UNROLL>(simd, keys, values));
    // The runs are in slice order, so `from_iter` gives later runs precedence, as usual.
    runs.into_iter()
        .map(|(start, end, value)| (start..=end, value))
        .collect()
}

/// An integer type whose slices can be scanned with SIMD.
///
/// Every implementor maps each value to one lane (or, for 128-bit types, two 64-bit lanes) of a
/// type `fearless_simd` supports natively: `Self` itself for the fixed-width integers up to 64
/// bits; a fixed-width integer of the same size for `isize`/`usize`, `char`, and `Ipv4Addr`; low
/// and high `u64` halves for `u128`, `i128`, and `Ipv6Addr`. The mapping preserves equality, and
/// consecutive lane values (lane-wise `splat(first) + offsets`) are consecutive `Self` values. The
/// converse may fail, which only means SIMD misses a run that the scalar merging then catches:
/// `char`'s consecutive `'\u{D7FF}'` and `'\u{E000}'` aren't consecutive lanes, and a 128-bit run
/// whose low half carries into its high half doesn't match `splat(first) + offsets`.
#[allow(clippy::redundant_pub_crate)]
pub(crate) trait SimdInteger: Integer {
    /// The SIMD lane type backing `Self`.
    type Lane: SimdIntElement + TryFrom<usize>;

    /// The SIMD vector used to scan a chunk of the slice. Its lane count, divided by
    /// [`Self::LANES_PER_VALUE`], sets the chunk size (see [`chunk_len`]).
    type Vector<S: Simd>: SimdInt<S, Element = Self::Lane>;

    /// How many lanes hold one value: 1, or 2 for 128-bit values split into 64-bit halves.
    const LANES_PER_VALUE: usize = 1;

    /// Returns a vector holding `value` in every value slot.
    fn splat<S: Simd>(simd: S, value: Self) -> Self::Vector<S>;

    /// Returns the vector holding `0, 1, 2, ...` in its value slots, so that
    /// `splat(first) + offsets()` is the chunk of consecutive values starting at `first`.
    #[expect(
        clippy::inline_always,
        reason = "must inline into the fearless_simd dispatch target-feature closure"
    )]
    #[inline(always)]
    fn offsets<S: Simd>(simd: S) -> Self::Vector<S> {
        Self::Vector::<S>::from_fn(simd, lane_from_usize)
    }

    /// Returns the vector that, added to a chunk of consecutive values, gives the next chunk.
    #[expect(
        clippy::inline_always,
        reason = "must inline into the fearless_simd dispatch target-feature closure"
    )]
    #[inline(always)]
    fn step<S: Simd>(simd: S) -> Self::Vector<S> {
        Self::Vector::<S>::splat(simd, lane_from_usize(chunk_len::<Self, S>()))
    }

    /// Loads `chunk`, which must have exactly [`chunk_len`] values, into a vector.
    fn load<S: Simd>(simd: S, chunk: &[Self]) -> Self::Vector<S>;
}

/// How many values one SIMD chunk holds.
#[expect(
    clippy::inline_always,
    reason = "must inline into the fearless_simd dispatch target-feature closure"
)]
#[inline(always)]
const fn chunk_len<T: SimdInteger, S: Simd>() -> usize {
    T::Vector::<S>::LEN / T::LANES_PER_VALUE
}

/// Converts a lane index or count, which always fits, to a lane value.
#[expect(
    clippy::inline_always,
    reason = "must inline into the fearless_simd dispatch target-feature closure"
)]
#[inline(always)]
fn lane_from_usize<L: TryFrom<usize>>(index: usize) -> L {
    L::try_from(index)
        .unwrap_or_else(|_| unreachable!("a SIMD lane index or count always fits its lane type"))
}

/// Implements [`SimdInteger::load`] by reinterpreting the chunk's bytes as the lane type via
/// [`bytemuck::cast_slice`] (a no-op cast when `Lane == Self`), for types whose layout allows it.
macro_rules! load_by_cast {
    () => {
        #[inline(always)]
        fn load<S: Simd>(simd: S, chunk: &[Self]) -> Self::Vector<S> {
            Self::Vector::<S>::from_slice(simd, bytemuck::cast_slice(chunk))
        }
    };
}

// Every type uses fearless_simd's widest vector, 512 bits (64, 32, 16, or 8 lanes).
// On narrower hardware, fearless_simd splits wide vectors into several native operations.
macro_rules! impl_simd_integer {
    ($($type:ty => $vector:ident),+ $(,)?) => {
        $(
            impl SimdInteger for $type {
                type Lane = Self;
                type Vector<S: Simd> = fearless_simd::$vector<S>;

                #[inline(always)]
                fn splat<S: Simd>(simd: S, value: Self) -> Self::Vector<S> {
                    Self::Vector::<S>::splat(simd, value)
                }

                load_by_cast!();
            }
        )+
    };
}

impl_simd_integer!(
    i8 => i8x64,
    i16 => i16x32,
    i32 => i32x16,
    i64 => i64x8,
    u8 => u8x64,
    u16 => u16x32,
    u32 => u32x16,
    u64 => u64x8,
);

// `isize`/`usize` are just pointer-width integers: each one maps to the fixed-width lane type of
// the same size for the current `target_pointer_width`, and `load_by_cast!`'s `bytemuck::cast_slice`
// reinterprets the chunk as that type with no copy and no `unsafe` in this crate.
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
                fn splat<S: Simd>(simd: S, value: Self) -> Self::Vector<S> {
                    Self::Vector::<S>::splat(simd, value as $isize_lane)
                }

                load_by_cast!();
            }

            #[cfg(target_pointer_width = $width)]
            impl SimdInteger for usize {
                type Lane = $usize_lane;
                type Vector<S: Simd> = fearless_simd::$usize_vector<S>;

                #[inline(always)]
                fn splat<S: Simd>(simd: S, value: Self) -> Self::Vector<S> {
                    Self::Vector::<S>::splat(simd, value as $usize_lane)
                }

                load_by_cast!();
            }
        )+
    };
}

impl_simd_integer_pointer_sized!(
    "16": i16 => i16x32, u16 => u16x32;
    "32": i32 => i32x16, u32 => u32x16;
    "64": i64 => i64x8, u64 => u64x8;
);

// `char` is a `u32` that is a valid Unicode scalar value, so its bytes cast to `u32` lanes.
impl SimdInteger for char {
    type Lane = u32;
    type Vector<S: Simd> = fearless_simd::u32x16<S>;

    #[expect(
        clippy::inline_always,
        reason = "must inline into the fearless_simd dispatch target-feature closure"
    )]
    #[inline(always)]
    fn splat<S: Simd>(simd: S, value: Self) -> Self::Vector<S> {
        Self::Vector::<S>::splat(simd, value.into())
    }

    load_by_cast!();
}

// `Ipv4Addr`'s memory layout isn't guaranteed, so instead of casting, each lane is converted
// with `u32::from`, which orders addresses as `Ipv4Addr` does.
impl SimdInteger for core::net::Ipv4Addr {
    type Lane = u32;
    type Vector<S: Simd> = fearless_simd::u32x16<S>;

    #[expect(
        clippy::inline_always,
        reason = "must inline into the fearless_simd dispatch target-feature closure"
    )]
    #[inline(always)]
    fn splat<S: Simd>(simd: S, value: Self) -> Self::Vector<S> {
        Self::Vector::<S>::splat(simd, value.into())
    }

    #[expect(
        clippy::inline_always,
        reason = "must inline into the fearless_simd dispatch target-feature closure"
    )]
    #[inline(always)]
    fn load<S: Simd>(simd: S, chunk: &[Self]) -> Self::Vector<S> {
        Self::Vector::<S>::from_fn(simd, |index| chunk[index].into())
    }
}

// 128-bit values have no SIMD lane type, so each one spans two `u64` lanes, its low and high
// halves, in memory order: a `u128` slice then casts to `u64` lanes directly.

/// Which of a 128-bit value's two `u64` lanes holds its low half (memory order).
#[cfg(target_endian = "little")]
const LOW_HALF: usize = 0;
/// Which of a 128-bit value's two `u64` lanes holds its low half (memory order).
#[cfg(target_endian = "big")]
const LOW_HALF: usize = 1;

/// Splits `bits` into the `u64` lane for lane `index` of a 128-bit-values vector.
#[expect(
    clippy::inline_always,
    reason = "must inline into the fearless_simd dispatch target-feature closure"
)]
#[inline(always)]
#[expect(
    clippy::cast_possible_truncation,
    reason = "takes one 64-bit half on purpose"
)]
const fn half(bits: u128, index: usize) -> u64 {
    if index % 2 == LOW_HALF {
        bits as u64
    } else {
        (bits >> 64) as u64
    }
}

/// Implements [`SimdInteger`]'s methods for a 128-bit type, given how a value becomes `u128` bits.
macro_rules! impl_simd_integer_128_methods {
    ($to_bits:expr) => {
        type Lane = u64;
        type Vector<S: Simd> = fearless_simd::u64x8<S>;
        const LANES_PER_VALUE: usize = 2;

        #[inline(always)]
        fn splat<S: Simd>(simd: S, value: Self) -> Self::Vector<S> {
            let bits: u128 = $to_bits(value);
            Self::Vector::<S>::from_fn(simd, |index| half(bits, index))
        }

        // A value's offset goes in its low half; its high half gets 0.
        #[inline(always)]
        fn offsets<S: Simd>(simd: S) -> Self::Vector<S> {
            Self::Vector::<S>::from_fn(simd, |index| {
                half(lane_from_usize::<u128>(index / 2), index)
            })
        }

        #[inline(always)]
        fn step<S: Simd>(simd: S) -> Self::Vector<S> {
            let len = lane_from_usize::<u128>(chunk_len::<Self, S>());
            Self::Vector::<S>::from_fn(simd, |index| half(len, index))
        }
    };
}

impl SimdInteger for u128 {
    impl_simd_integer_128_methods!(|value: Self| value);
    load_by_cast!();
}

impl SimdInteger for i128 {
    impl_simd_integer_128_methods!(|value: Self| value.cast_unsigned());
    load_by_cast!();
}

// Like `Ipv4Addr`, `Ipv6Addr`'s layout isn't guaranteed, so each lane is a half of `u128::from`.
impl SimdInteger for core::net::Ipv6Addr {
    impl_simd_integer_128_methods!(u128::from);

    #[expect(
        clippy::inline_always,
        reason = "must inline into the fearless_simd dispatch target-feature closure"
    )]
    #[inline(always)]
    fn load<S: Simd>(simd: S, chunk: &[Self]) -> Self::Vector<S> {
        Self::Vector::<S>::from_fn(simd, |index| half(chunk[index / 2].into(), index))
    }
}

/// The SIMD kernel. Must stay `#[inline(always)]` and eager; see the module docs.
#[expect(
    clippy::inline_always,
    reason = "must inline into the fearless_simd dispatch target-feature closure"
)]
#[inline(always)]
fn collect_ranges<T, S, const N: usize>(simd: S, slice: &[T]) -> Vec<(T, T)>
where
    T: SimdInteger,
    S: Simd,
{
    let lanes = chunk_len::<T, S>();
    let block = lanes * N;
    let offsets = T::offsets(simd);
    let step = T::step(simd);
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
        // Unrolled: check `N` chunks per iteration with a single branch.
        while N > 1 && rest.len() >= block {
            let (next, after) = rest.split_at(block);
            if !block_continues(simd, offsets, step, end, next) {
                break;
            }
            end = next[block - 1];
            rest = after;
        }
        while rest.len() >= lanes {
            let (next, after) = rest.split_at(lanes);
            if !chunk_continues(simd, offsets, end, next) {
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

/// The map SIMD kernel: like [`collect_ranges`], but finds runs of consecutive keys, then splits
/// each run wherever the (scalar-compared) value changes. Must stay `#[inline(always)]` and eager;
/// see the module docs.
#[expect(
    clippy::inline_always,
    reason = "must inline into the fearless_simd dispatch target-feature closure"
)]
#[inline(always)]
fn collect_runs<'a, T, V, S, const N: usize>(
    simd: S,
    keys: &[T],
    values: &'a [V],
) -> Vec<(T, T, &'a V)>
where
    T: SimdInteger,
    V: Eq,
    S: Simd,
{
    let lanes = chunk_len::<T, S>();
    let block = lanes * N;
    let offsets = T::offsets(simd);
    let step = T::step(simd);
    let mut runs = RunCollector::new();
    let len = keys.len();
    let mut index = 0;
    while len - index >= lanes {
        let chunk = &keys[index..index + lanes];
        if !is_consecutive_with_offsets(simd, offsets, chunk) {
            for (&key, value) in chunk.iter().zip(&values[index..index + lanes]) {
                runs.push(key, key, value);
            }
            index += lanes;
            continue;
        }
        // Tight inner loop: extend the run of consecutive keys while each next chunk continues it.
        let start_index = index;
        let mut end = chunk[lanes - 1];
        index += lanes;
        // Unrolled: check `N` chunks per iteration with a single branch.
        while N > 1 && len - index >= block {
            let next = &keys[index..index + block];
            if !block_continues(simd, offsets, step, end, next) {
                break;
            }
            end = next[block - 1];
            index += block;
        }
        while len - index >= lanes {
            let next = &keys[index..index + lanes];
            if !chunk_continues(simd, offsets, end, next) {
                break;
            }
            end = next[lanes - 1];
            index += lanes;
        }
        runs.push_consecutive_keys(&keys[start_index..index], &values[start_index..index]);
    }
    for (&key, value) in keys[index..].iter().zip(&values[index..]) {
        runs.push(key, key, value);
    }
    runs.finish()
}

/// Like [`RangeCollector`], but for `(start, end, value)` runs: a run merges into the previous one
/// only when their values are equal.
struct RunCollector<'a, T: Integer, V> {
    runs: Vec<(T, T, &'a V)>,
    current: Option<(T, T, &'a V)>,
}

impl<'a, T: Integer, V: Eq> RunCollector<'a, T, V> {
    #[expect(
        clippy::inline_always,
        reason = "must inline into the fearless_simd dispatch target-feature closure"
    )]
    #[inline(always)]
    const fn new() -> Self {
        Self {
            runs: Vec::new(),
            current: None,
        }
    }

    #[expect(
        clippy::inline_always,
        reason = "must inline into the fearless_simd dispatch target-feature closure"
    )]
    #[inline(always)]
    fn push(&mut self, start: T, end: T, value: &'a V) {
        debug_assert!(start <= end, "runs from a slice are never empty");
        if let Some((current_start, current_end, current_value)) = &mut self.current {
            // When `start > *current_end`, `start > T::min_value()`, so `sub_one` can't overflow.
            if *current_start <= start
                && (start <= *current_end || start.sub_one() == *current_end)
                && *current_value == value
            {
                if end > *current_end {
                    *current_end = end;
                }
                return;
            }
            self.runs
                .push((*current_start, *current_end, *current_value));
        }
        self.current = Some((start, end, value));
    }

    /// Pushes consecutive, increasing `keys`, split wherever the value changes.
    #[expect(
        clippy::inline_always,
        reason = "must inline into the fearless_simd dispatch target-feature closure"
    )]
    #[inline(always)]
    fn push_consecutive_keys(&mut self, keys: &[T], values: &'a [V]) {
        let mut rest = values;
        let mut start_index = 0;
        while let Some((value, after)) = rest.split_first() {
            let same = count_equal_prefix(after, value);
            let end_index = start_index + same;
            self.push(keys[start_index], keys[end_index], value);
            start_index = end_index + 1;
            rest = &after[same..];
        }
    }

    #[expect(
        clippy::inline_always,
        reason = "must inline into the fearless_simd dispatch target-feature closure"
    )]
    #[inline(always)]
    fn finish(mut self) -> Vec<(T, T, &'a V)> {
        if let Some(run) = self.current {
            self.runs.push(run);
        }
        self.runs
    }
}

/// How many values [`count_equal_prefix`] compares per branch, for cheap-to-compare values.
const VALUE_BLOCK: usize = 64;

/// Returns how many leading `values` equal `value`.
///
/// There's no SIMD on values (`V` is any `Eq` type). But when `V` looks cheap to compare (no drop
/// glue and at most 8 bytes, such as a primitive integer), each block of [`VALUE_BLOCK`] values is
/// compared without an early exit, so LLVM can auto-vectorize the comparison with the target
/// features of the enclosing `dispatch!` closure. A block is first checked by its last value, so
/// when values change often, a block rarely costs more than one extra comparison.
///
/// Other types, such as `String`, compare one value at a time: blocks made them up to 2x slower.
#[expect(
    clippy::inline_always,
    reason = "must inline into the fearless_simd dispatch target-feature closure"
)]
#[inline(always)]
fn count_equal_prefix<V: Eq>(values: &[V], value: &V) -> usize {
    let mut count = 0;
    if !core::mem::needs_drop::<V>() && size_of::<V>() <= size_of::<u64>() {
        for block in values.as_chunks::<VALUE_BLOCK>().0 {
            if block[VALUE_BLOCK - 1] != *value {
                break;
            }
            #[expect(
                clippy::needless_bitwise_bool,
                reason = "`&&` would exit early, which blocks auto-vectorization"
            )]
            let all_equal = block.iter().fold(true, |all, next| all & (next == value));
            if !all_equal {
                break;
            }
            count += VALUE_BLOCK;
        }
    }
    // Finish within the block that broke (or the short remainder), one value at a time.
    count
        + values[count..]
            .iter()
            .take_while(|&next| next == value)
            .count()
}

/// Collects ranges in slice order, merging each into the previous one when they touch or overlap.
///
/// The range being extended lives in a local (in registers) rather than at the end of the
/// vector, which keeps the hot all-consecutive path free of memory writes.
struct RangeCollector<T: Integer> {
    ranges: Vec<(T, T)>,
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
            self.ranges.push((*current_start, *current_end));
        }
        self.current = Some((start, end));
    }

    #[expect(
        clippy::inline_always,
        reason = "must inline into the fearless_simd dispatch target-feature closure"
    )]
    #[inline(always)]
    fn finish(mut self) -> Vec<(T, T)> {
        if let Some((start, end)) = self.current {
            self.ranges.push((start, end));
        }
        self.ranges
    }
}

/// Returns `true` if `block` (a whole number of chunks) holds the consecutive integers that come
/// right after `end`.
#[expect(
    clippy::inline_always,
    reason = "must inline into the fearless_simd dispatch target-feature closure"
)]
#[inline(always)]
fn block_continues<T, S>(
    simd: S,
    offsets: T::Vector<S>,
    step: T::Vector<S>,
    end: T,
    block: &[T],
) -> bool
where
    T: SimdInteger,
    S: Simd,
{
    // `block[0] > end`, so `block[0] > T::min_value()` and `sub_one` can't overflow.
    if !(block[0] > end && block[0].sub_one() == end) {
        return false;
    }
    // Lane arithmetic wraps, so also require the block's last value to be at least its
    // first, which rejects wrapped blocks (see `is_consecutive_with_offsets`).
    let mut expected = T::splat(simd, block[0]) + offsets;
    let mut all = block[0] <= block[block.len() - 1];
    for chunk in block.chunks_exact(chunk_len::<T, S>()) {
        all &= T::load(simd, chunk).simd_eq(expected).all_true();
        expected += step;
    }
    all
}

/// Returns `true` if `chunk` holds the consecutive integers that come right after `end`.
#[expect(
    clippy::inline_always,
    reason = "must inline into the fearless_simd dispatch target-feature closure"
)]
#[inline(always)]
fn chunk_continues<T, S>(simd: S, offsets: T::Vector<S>, end: T, chunk: &[T]) -> bool
where
    T: SimdInteger,
    S: Simd,
{
    // `chunk[0] > end`, so `chunk[0] > T::min_value()` and `sub_one` can't overflow.
    chunk[0] > end && chunk[0].sub_one() == end && is_consecutive_with_offsets(simd, offsets, chunk)
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
    (values - offsets).simd_eq(T::splat(simd, first)).all_true() && first <= chunk[chunk.len() - 1]
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
    is_consecutive_with_offsets(simd, T::offsets(simd), chunk)
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
