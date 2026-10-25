//! Experimental cuTile kernels: so far, the range half of set ingestion for sorted `u32` keys.
//!
//! A cuTile program works on whole tiles of values rather than on single threads: the compiler
//! decides how threads and warps share each tile. Each tile here is `TILE` sorted keys. Finding
//! ranges takes two launches with a small host scan between them:
//!
//! 1. `count_ranges`: each tile marks the keys that start a range and writes how many it has.
//! 2. The host turns those counts into each tile's first range number (an exclusive scan).
//! 3. `emit_ranges`: each tile scans its marks to number its ranges, then scatters each range's
//!    first and last key.
//!
//! cuTile compiles the kernels on first use with `tileiras` from the CUDA toolkit.

use std::sync::Arc;
use std::vec::Vec;

use cutile::api::{copy_host_vec_to_device, zeros};
use cutile::error::Error;
use cutile::prelude::DeviceOp;
use cutile::tensor::ToHostVec;
use cutile::tile_kernel::TileKernel;

/// Keys per tile. The tile shapes in [`kernels`] must match.
#[cfg_attr(
    not(test),
    expect(dead_code, reason = "no engine uses the cuTile kernels yet")
)]
const TILE: usize = 1024;

#[cutile::module]
mod kernels {
    // The crate is `no_std`, but `#[cutile::module]` expands to code here that names the std
    // prelude (`Vec`, `String`, `format!`).
    use std::string::{String, ToString};
    use std::vec::Vec;
    use std::{format, vec};

    #[expect(clippy::wildcard_imports, reason = "cuTile kernels use the DSL as a glob")]
    use cutile::core::*;

    // The cuTile compiler can't infer the type of a bare `constant(..)` argument, so constants
    // get annotated `let`s; the other intermediate values follow suit for readability.

    /// Writes how many ranges start in each tile of sorted keys to `counts[tile]`.
    ///
    /// # Safety
    ///
    /// `sorted` must point to `len` keys and `counts` to one word per tile.
    #[cutile::entry()]
    unsafe fn count_ranges(counts: *mut i32, sorted: *mut u32, len: i32) {
        let tile: i32 = get_tile_block_id().0;
        let index: Tile<i32, { [1024] }> = tile_indices(tile);
        let one: Tile<i32, { [1024] }> = constant(1i32, shape![1024]);
        let previous_index: Tile<i32, { [1024] }> = index - one;
        // SAFETY: `sorted` points to `len` keys.
        let key: Tile<u32, { [1024] }> = unsafe { gather(sorted, index, len) };
        let previous: Tile<u32, { [1024] }> = unsafe { gather(sorted, previous_index, len) };
        let starts: Tile<i32, { [1024] }> = range_starts(index, previous, key, len);
        let count: Tile<i32, { [] }> = reduce_sum(starts, 0i32);
        let counts_base: PointerTile<*mut i32, { [] }> = pointer_to_tile(counts);
        let slot: PointerTile<*mut i32, { [] }> = addptr(counts_base, tile);
        // SAFETY: `counts` has one word per tile.
        unsafe {
            store_ptr_tko(
                slot,
                count,
                ordering::Weak,
                None::<scope::TileBlock>,
                None,
                None,
                Latency::<0>,
            );
        }
    }

    /// Writes each range's first key to `starts` and its last key to `ends`. `first_range[tile]`
    /// is the number of ranges that start before this tile.
    ///
    /// # Safety
    ///
    /// `sorted` must point to `len` keys, `first_range` to one word per tile, and `starts` and
    /// `ends` to one word per range.
    #[cutile::entry()]
    unsafe fn emit_ranges(
        starts: *mut u32,
        ends: *mut u32,
        sorted: *mut u32,
        first_range: *mut i32,
        len: i32,
    ) {
        let tile: i32 = get_tile_block_id().0;
        let index: Tile<i32, { [1024] }> = tile_indices(tile);
        let one: Tile<i32, { [1024] }> = constant(1i32, shape![1024]);
        let previous_index: Tile<i32, { [1024] }> = index - one;
        let next_index: Tile<i32, { [1024] }> = index + one;
        // SAFETY: `sorted` points to `len` keys.
        let key: Tile<u32, { [1024] }> = unsafe { gather(sorted, index, len) };
        let previous: Tile<u32, { [1024] }> = unsafe { gather(sorted, previous_index, len) };
        let next: Tile<u32, { [1024] }> = unsafe { gather(sorted, next_index, len) };
        let is_start: Tile<i32, { [1024] }> = range_starts(index, previous, key, len);
        let is_end: Tile<i32, { [1024] }> = range_ends(index, key, next, len);

        // Each key's range number: the ranges before this tile, plus this tile's starts so far,
        // minus one.
        let first_range_base: PointerTile<*mut i32, { [] }> = pointer_to_tile(first_range);
        let first_range_slot: PointerTile<*mut i32, { [] }> = addptr(first_range_base, tile);
        // SAFETY: `first_range` has one word per tile.
        let (before, _): (Tile<i32, { [] }>, Token) = unsafe {
            load_ptr_tko(
                first_range_slot,
                ordering::Weak,
                None::<scope::TileBlock>,
                None,
                None,
                None,
                Latency::<0>,
            )
        };
        let before_1: Tile<i32, { [1] }> = before.reshape(shape![1]);
        let before_all: Tile<i32, { [1024] }> = before_1.broadcast(shape![1024]);
        let started: Tile<i32, { [1024] }> = scan_sum(is_start, 0i32, reverse::Forward, 0i32);
        let started_before: Tile<i32, { [1024] }> = before_all + started;
        let range: Tile<i32, { [1024] }> = started_before - one;

        let start_mask: Tile<bool, { [1024] }> = eq_tile(is_start, one);
        let start_slots: PointerTile<*mut u32, { [1024] }> = slots(starts, range);
        // SAFETY: `starts` has one word per range, and each range has exactly one start.
        unsafe {
            store_ptr_tko(
                start_slots,
                key,
                ordering::Weak,
                None::<scope::TileBlock>,
                Some(start_mask),
                None,
                Latency::<0>,
            );
        }
        let end_mask: Tile<bool, { [1024] }> = eq_tile(is_end, one);
        let end_slots: PointerTile<*mut u32, { [1024] }> = slots(ends, range);
        // SAFETY: `ends` has one word per range, and each range has exactly one end.
        unsafe {
            store_ptr_tko(
                end_slots,
                key,
                ordering::Weak,
                None::<scope::TileBlock>,
                Some(end_mask),
                None,
                Latency::<0>,
            );
        }
    }

    /// The positions of tile `tile`'s keys.
    fn tile_indices(tile: i32) -> Tile<i32, { [1024] }> {
        let offsets: Tile<i32, { [1024] }> = iota(shape![1024]);
        let first: Tile<i32, { [1024] }> = broadcast_scalar(tile * 1024i32, shape![1024]);
        offsets + first
    }

    /// Pointers to `base[index]` for each index.
    fn slots<E: ElementType>(
        base: *mut E,
        index: Tile<i32, { [1024] }>,
    ) -> PointerTile<*mut E, { [1024] }> {
        let base: PointerTile<*mut E, { [] }> = pointer_to_tile(base);
        let base_1: PointerTile<*mut E, { [1] }> = base.reshape(shape![1]);
        let bases: PointerTile<*mut E, { [1024] }> = base_1.broadcast(shape![1024]);
        bases.offset_tile(index)
    }

    /// The keys at `index`, or 0 where `index` is outside `0..len`.
    ///
    /// # Safety
    ///
    /// `sorted` must point to `len` keys.
    unsafe fn gather(
        sorted: *mut u32,
        index: Tile<i32, { [1024] }>,
        len: i32,
    ) -> Tile<u32, { [1024] }> {
        let one: Tile<i32, { [1024] }> = constant(1i32, shape![1024]);
        let in_bounds: Tile<bool, { [1024] }> = eq_tile(in_range(index, len), one);
        let key_slots: PointerTile<*mut u32, { [1024] }> = slots(sorted, index);
        // SAFETY: the mask skips every index outside `0..len`.
        let (keys, _): (Tile<u32, { [1024] }>, Token) = unsafe {
            load_ptr_tko(
                key_slots,
                ordering::Weak,
                None::<scope::TileBlock>,
                Some(in_bounds),
                Some(0u32),
                None,
                Latency::<0>,
            )
        };
        keys
    }

    // Flags are `i32` 0 or 1 rather than `bool` tiles, because this cuTile version compiles
    // `constant(false, ..)` as true.

    /// 1 where an index is in `0..len`, else 0.
    fn in_range(index: Tile<i32, { [1024] }>, len: i32) -> Tile<i32, { [1024] }> {
        let zero: Tile<i32, { [1024] }> = constant(0i32, shape![1024]);
        let one: Tile<i32, { [1024] }> = constant(1i32, shape![1024]);
        let lens: Tile<i32, { [1024] }> = broadcast_scalar(len, shape![1024]);
        let not_negative: Tile<bool, { [1024] }> = ge_tile(index, zero);
        let below_len: Tile<bool, { [1024] }> = lt_tile(index, lens);
        let below: Tile<i32, { [1024] }> = select(below_len, one, zero);
        select(not_negative, below, zero)
    }

    /// 1 where `key` (at `index`, after `previous`) starts a range, else 0.
    fn range_starts(
        index: Tile<i32, { [1024] }>,
        previous: Tile<u32, { [1024] }>,
        key: Tile<u32, { [1024] }>,
        len: i32,
    ) -> Tile<i32, { [1024] }> {
        let zero: Tile<i32, { [1024] }> = constant(0i32, shape![1024]);
        let one: Tile<i32, { [1024] }> = constant(1i32, shape![1024]);
        let first: Tile<bool, { [1024] }> = eq_tile(index, zero);
        let joined: Tile<i32, { [1024] }> = joins(previous, key);
        let starts: Tile<i32, { [1024] }> = select(first, one, one - joined);
        starts * in_range(index, len)
    }

    /// 1 where `key` (at `index`, before `next`) ends a range, else 0.
    fn range_ends(
        index: Tile<i32, { [1024] }>,
        key: Tile<u32, { [1024] }>,
        next: Tile<u32, { [1024] }>,
        len: i32,
    ) -> Tile<i32, { [1024] }> {
        let one: Tile<i32, { [1024] }> = constant(1i32, shape![1024]);
        let lasts: Tile<i32, { [1024] }> = broadcast_scalar(len - 1i32, shape![1024]);
        let last: Tile<bool, { [1024] }> = eq_tile(index, lasts);
        let joined: Tile<i32, { [1024] }> = joins(key, next);
        let ends: Tile<i32, { [1024] }> = select(last, one, one - joined);
        ends * in_range(index, len)
    }

    /// 1 where sorted keys `left` then `right` belong to the same range, else 0: equal, or
    /// adjacent without wrapping from `u32::MAX` to 0.
    fn joins(left: Tile<u32, { [1024] }>, right: Tile<u32, { [1024] }>) -> Tile<i32, { [1024] }> {
        let zero: Tile<i32, { [1024] }> = constant(0i32, shape![1024]);
        let one: Tile<i32, { [1024] }> = constant(1i32, shape![1024]);
        let zero_key: Tile<u32, { [1024] }> = constant(0u32, shape![1024]);
        let one_key: Tile<u32, { [1024] }> = constant(1u32, shape![1024]);
        // `left + 1` wraps to 0 exactly when `left` is `u32::MAX`. (This avoids a `u32::MAX`
        // constant: this cuTile version compiled `constant(4_294_967_295u32, ..)` as 0.)
        let successor: Tile<u32, { [1024] }> = left + one_key;
        let wrapped: Tile<bool, { [1024] }> = eq_tile(successor, zero_key);
        let follows: Tile<bool, { [1024] }> = eq_tile(right, successor);
        let equal: Tile<bool, { [1024] }> = eq_tile(left, right);
        let adjacent: Tile<i32, { [1024] }> = select(wrapped, zero, select(follows, one, zero));
        select(equal, one, adjacent)
    }
}

use kernels::{count_ranges, emit_ranges};

/// The ranges of sorted `u32` keys, as `(start, end)` pairs.
///
/// # Panics
///
/// Panics if there are more than `i32::MAX` keys.
#[cfg_attr(
    not(test),
    expect(dead_code, reason = "no engine uses the cuTile kernels yet")
)]
pub(super) fn sorted_ranges_32(sorted: &[u32]) -> Result<Vec<(u32, u32)>, Error> {
    if sorted.is_empty() {
        return Ok(Vec::new());
    }
    let len = i32::try_from(sorted.len()).expect("cuTile input exceeds i32 positions");
    let tiles = sorted.len().div_ceil(TILE);
    let grid = (u32::try_from(tiles).expect("tile count fits u32"), 1, 1);

    let keys = copy_host_vec_to_device(&Arc::new(sorted.to_vec())).sync()?;
    let counts = zeros::<i32>(&[tiles]).sync()?;
    // SAFETY: `keys` holds `len` keys and `counts` one word per tile.
    unsafe { count_ranges(counts.device_pointer(), keys.device_pointer(), len) }
        .grid(grid)
        .sync()?;

    let mut first_range = counts.to_host_vec().sync()?;
    let mut ranges = 0;
    for count in &mut first_range {
        (*count, ranges) = (ranges, ranges + *count);
    }
    let range_count = usize::try_from(ranges).expect("range count is never negative");
    let first_range = copy_host_vec_to_device(&Arc::new(first_range)).sync()?;
    let starts = zeros::<u32>(&[range_count]).sync()?;
    let ends = zeros::<u32>(&[range_count]).sync()?;
    // SAFETY: `keys` holds `len` keys, `first_range` one word per tile, and `starts` and `ends`
    // one word per range (the counts' total).
    unsafe {
        emit_ranges(
            starts.device_pointer(),
            ends.device_pointer(),
            keys.device_pointer(),
            first_range.device_pointer(),
            len,
        )
    }
    .grid(grid)
    .sync()?;

    let starts = starts.to_host_vec().sync()?;
    let ends = ends.to_host_vec().sync()?;
    Ok(starts.into_iter().zip(ends).collect())
}

#[cfg(test)]
mod tests {
    use std::vec;

    use super::*;

    /// The ranges of sorted keys, found on the CPU.
    fn cpu_ranges(sorted: &[u32]) -> Vec<(u32, u32)> {
        let mut ranges: Vec<(u32, u32)> = Vec::new();
        for &key in sorted {
            match ranges.last_mut() {
                Some((_, end)) if key == *end || (*end != u32::MAX && key == *end + 1) => {
                    *end = key;
                }
                _ => ranges.push((key, key)),
            }
        }
        ranges
    }

    #[test]
    fn sorted_ranges_32_matches_cpu() {
        let mut cases: Vec<Vec<u32>> = vec![
            vec![],
            vec![7],
            vec![1, 2, 3, 3, 7, 8, 10, u32::MAX - 1, u32::MAX],
            vec![0, u32::MAX],
            (0..5000).collect(),
            (0..3000).map(|i| i * 2).collect(),
        ];
        // Pseudo-random sorted keys with runs and duplicates, straddling tile boundaries.
        let mut state = 0x2545_f491_u32;
        let mut keys: Vec<u32> = (0..100_000)
            .map(|_| {
                state ^= state << 13;
                state ^= state >> 17;
                state ^= state << 5;
                state % 300_000
            })
            .collect();
        keys.sort_unstable();
        cases.push(keys);

        for sorted in cases {
            assert_eq!(
                sorted_ranges_32(&sorted).expect("cuTile launch failed"),
                cpu_ranges(&sorted),
                "on {} keys",
                sorted.len()
            );
        }
    }
}
