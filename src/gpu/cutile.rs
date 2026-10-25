//! Experimental cuTile kernels: set ingestion for keys encoded as `u32`.
//!
//! A cuTile program works on whole tiles of values rather than on single threads: the compiler
//! decides how threads and warps share each tile. Each tile here is `TILE` keys.
//!
//! **Sort** (least-significant digit first, 4 bits per pass): each pass takes two launches with a
//! host scan between them.
//!
//! 1. `digit_counts`: each tile counts how many of its keys have each digit.
//! 2. The host turns those counts into where each tile's keys of each digit go (an exclusive
//!    scan, digit by digit, tile by tile), and skips the pass if every key has the same digit.
//! 3. `scatter_digits`: each tile ranks its keys within their digit (one `scan_sum` per digit
//!    value) and scatters them to their places. Keys keep their order within a digit, so the sort
//!    is stable, as a least-significant-digit sort must be.
//!
//! **Ranges**: two more launches with a host scan between them.
//!
//! 1. `count_ranges`: each tile marks the keys that start a range and writes how many it has.
//! 2. The host turns those counts into each tile's first range number.
//! 3. `emit_ranges`: each tile scans its marks to number its ranges, then scatters each range's
//!    first and last key.
//!
//! cuTile compiles the kernels on first use with `tileiras` from the CUDA toolkit.

use std::sync::Arc;
use std::vec::Vec;

use cutile::api::{copy_host_vec_to_device, zeros};
use cutile::error::Error;
use cutile::prelude::DeviceOp;
use cutile::tensor::{Tensor, ToHostVec};
use cutile::tile_kernel::TileKernel;

/// Keys per tile. The tile shapes in [`kernels`] must match.
const TILE: usize = 1024;
/// Bits per sort digit, and the number of digit values. `kernels` must match.
const DIGIT_BITS: u32 = 4;
const DIGITS: usize = 1 << DIGIT_BITS;

#[cutile::module]
mod kernels {
    // The crate is `no_std`, but `#[cutile::module]` expands to code here that names the std
    // prelude (`Vec`, `String`, `format!`).
    use std::string::{String, ToString};
    use std::vec::Vec;
    use std::{format, vec};

    #[cfg_attr(
        not(test),
        expect(
            clippy::wildcard_imports,
            reason = "cuTile kernels use the DSL as a glob"
        )
    )]
    use cutile::core::*;

    // The cuTile compiler can't infer the type of a bare `constant(..)` argument, so constants
    // get annotated `let`s; the other intermediate values follow suit for readability.
    //
    // Flags are `i32` 0 or 1 rather than `bool` tiles, because this cuTile version compiles
    // `constant(false, ..)` as true.

    /// Writes how many of each tile's keys have each digit (the 4 bits at `shift`) to
    /// `counts[digit * tiles + tile]`: the column sums of the tile's digit flags.
    ///
    /// # Safety
    ///
    /// `keys` must point to `len` keys and `counts` to 16 words per tile.
    #[cutile::entry()]
    unsafe fn digit_counts(counts: *mut i32, keys: *mut u32, len: i32, shift: u32, tiles: i32) {
        let tile: i32 = get_tile_block_id().0;
        let index: Tile<i32, { [1024] }> = tile_indices(tile);
        // SAFETY: `keys` points to `len` keys.
        let key: Tile<u32, { [1024] }> = unsafe { gather(keys, index, len) };
        let digit: Tile<i32, { [1024] }> = digits(key, shift, index, len);
        let flags: Tile<i32, { [1024, 16] }> = digit_flags(digit);
        let count: Tile<i32, { [16] }> = reduce_sum(flags, 0i32);
        // `counts[value * tiles + tile]` for each digit value.
        let values: Tile<i32, { [16] }> = iota(shape![16]);
        let tile_counts: Tile<i32, { [16] }> = broadcast_scalar(tiles, shape![16]);
        let this_tile: Tile<i32, { [16] }> = broadcast_scalar(tile, shape![16]);
        let offsets: Tile<i32, { [16] }> = values * tile_counts + this_tile;
        let counts_base: PointerTile<*mut i32, { [] }> = pointer_to_tile(counts);
        let counts_1: PointerTile<*mut i32, { [1] }> = counts_base.reshape(shape![1]);
        let counts_all: PointerTile<*mut i32, { [16] }> = counts_1.broadcast(shape![16]);
        let slots: PointerTile<*mut i32, { [16] }> = counts_all.offset_tile(offsets);
        // SAFETY: `counts` has 16 words per tile.
        unsafe {
            store_ptr_tko(
                slots,
                count,
                ordering::Weak,
                None::<scope::TileBlock>,
                None,
                None,
                Latency::<0>,
            );
        }
    }

    /// Moves each key to `sorted`, ordered by its digit (the 4 bits at `shift`), keeping keys
    /// with the same digit in order. `bases[digit * tiles + tile]` is where this tile's first key
    /// with that digit goes.
    ///
    /// # Safety
    ///
    /// `keys` and `sorted` must each point to `len` keys, and `bases` to 16 words per tile that
    /// place every key in `0..len` exactly once.
    #[cutile::entry()]
    unsafe fn scatter_digits(
        sorted: *mut u32,
        keys: *mut u32,
        bases: *mut i32,
        len: i32,
        shift: u32,
        tiles: i32,
    ) {
        let tile: i32 = get_tile_block_id().0;
        let index: Tile<i32, { [1024] }> = tile_indices(tile);
        // SAFETY: `keys` points to `len` keys.
        let key: Tile<u32, { [1024] }> = unsafe { gather(keys, index, len) };
        let digit: Tile<i32, { [1024] }> = digits(key, shift, index, len);
        let zero: Tile<i32, { [1024] }> = constant(0i32, shape![1024]);
        let one: Tile<i32, { [1024] }> = constant(1i32, shape![1024]);

        // Each key's rank among this tile's keys with the same digit, one digit value at a
        // time. (Scanning all 16 columns of `digit_flags` at once was about 40% slower.)
        let mut rank: Tile<i32, { [1024] }> = constant(0i32, shape![1024]);
        for value in 0i32..16i32 {
            let wanted: Tile<i32, { [1024] }> = broadcast_scalar(value, shape![1024]);
            let matches: Tile<bool, { [1024] }> = eq_tile(digit, wanted);
            let flags: Tile<i32, { [1024] }> = select(matches, one, zero);
            let through: Tile<i32, { [1024] }> = scan_sum(flags, 0i32, reverse::Forward, 0i32);
            let before: Tile<i32, { [1024] }> = through - flags;
            rank = rank + flags * before;
        }

        let valid: Tile<i32, { [1024] }> = in_range(index, len);
        let valid_mask: Tile<bool, { [1024] }> = eq_tile(valid, one);
        let tile_counts: Tile<i32, { [1024] }> = broadcast_scalar(tiles, shape![1024]);
        let this_tile: Tile<i32, { [1024] }> = broadcast_scalar(tile, shape![1024]);
        let base_index: Tile<i32, { [1024] }> = digit * tile_counts + this_tile;
        let base_slots: PointerTile<*mut i32, { [1024] }> = slots(bases, base_index);
        // SAFETY: the mask skips keys outside `0..len`, the only ones whose digit (16) has no
        // base.
        let (base, _): (Tile<i32, { [1024] }>, Token) = unsafe {
            load_ptr_tko(
                base_slots,
                ordering::Weak,
                None::<scope::TileBlock>,
                Some(valid_mask),
                Some(0i32),
                None,
                Latency::<0>,
            )
        };
        let destination: Tile<i32, { [1024] }> = base + rank;
        let sorted_slots: PointerTile<*mut u32, { [1024] }> = slots(sorted, destination);
        // SAFETY: `bases` places every key in `0..len` exactly once.
        unsafe {
            store_ptr_tko(
                sorted_slots,
                key,
                ordering::Weak,
                None::<scope::TileBlock>,
                Some(valid_mask),
                None,
                Latency::<0>,
            );
        }
    }

    /// Writes how many ranges start in each tile of sorted keys to `counts[tile]`. When
    /// `has_gap` is 1, keys `gap_left` and `gap_right` count as adjacent.
    ///
    /// # Safety
    ///
    /// `sorted` must point to `len` keys and `counts` to one word per tile.
    #[cutile::entry()]
    unsafe fn count_ranges(
        counts: *mut i32,
        sorted: *mut u32,
        len: i32,
        gap_left: u32,
        gap_right: u32,
        has_gap: i32,
    ) {
        let tile: i32 = get_tile_block_id().0;
        let index: Tile<i32, { [1024] }> = tile_indices(tile);
        let one: Tile<i32, { [1024] }> = constant(1i32, shape![1024]);
        let previous_index: Tile<i32, { [1024] }> = index - one;
        // SAFETY: `sorted` points to `len` keys.
        let key: Tile<u32, { [1024] }> = unsafe { gather(sorted, index, len) };
        let previous: Tile<u32, { [1024] }> = unsafe { gather(sorted, previous_index, len) };
        let starts: Tile<i32, { [1024] }> =
            range_starts(index, previous, key, len, gap_left, gap_right, has_gap);
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
    /// is the number of ranges that start before this tile. The gap is as in `count_ranges`.
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
        gap_left: u32,
        gap_right: u32,
        has_gap: i32,
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
        let is_start: Tile<i32, { [1024] }> =
            range_starts(index, previous, key, len, gap_left, gap_right, has_gap);
        let is_end: Tile<i32, { [1024] }> =
            range_ends(index, key, next, len, gap_left, gap_right, has_gap);

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

    /// A 1024 × 16 tile with a 1 in each key's row at its digit's column, and 0 elsewhere (an
    /// all-0 row where the digit is 16).
    fn digit_flags(digit: Tile<i32, { [1024] }>) -> Tile<i32, { [1024, 16] }> {
        let zero: Tile<i32, { [1024, 16] }> = constant(0i32, shape![1024, 16]);
        let one: Tile<i32, { [1024, 16] }> = constant(1i32, shape![1024, 16]);
        let digit_column: Tile<i32, { [1024, 1] }> = digit.reshape(shape![1024, 1]);
        let digits: Tile<i32, { [1024, 16] }> = digit_column.broadcast(shape![1024, 16]);
        let values: Tile<i32, { [16] }> = iota(shape![16]);
        let values_row: Tile<i32, { [1, 16] }> = values.reshape(shape![1, 16]);
        let all_values: Tile<i32, { [1024, 16] }> = values_row.broadcast(shape![1024, 16]);
        let matches: Tile<bool, { [1024, 16] }> = eq_tile(digits, all_values);
        select(matches, one, zero)
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

    /// Each key's digit, the 4 bits at `shift`; 16, which matches no digit, where `index` is
    /// outside `0..len`.
    fn digits(
        key: Tile<u32, { [1024] }>,
        shift: u32,
        index: Tile<i32, { [1024] }>,
        len: i32,
    ) -> Tile<i32, { [1024] }> {
        let one: Tile<i32, { [1024] }> = constant(1i32, shape![1024]);
        let none: Tile<i32, { [1024] }> = constant(16i32, shape![1024]);
        let low_bits: Tile<u32, { [1024] }> = constant(15u32, shape![1024]);
        let shifts: Tile<u32, { [1024] }> = broadcast_scalar(shift, shape![1024]);
        let shifted: Tile<u32, { [1024] }> = shri(key, shifts);
        let digit_bits: Tile<u32, { [1024] }> = andi(shifted, low_bits);
        let digit: Tile<i32, { [1024] }> = bitcast(digit_bits);
        let valid: Tile<bool, { [1024] }> = eq_tile(in_range(index, len), one);
        select(valid, digit, none)
    }

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
        gap_left: u32,
        gap_right: u32,
        has_gap: i32,
    ) -> Tile<i32, { [1024] }> {
        let zero: Tile<i32, { [1024] }> = constant(0i32, shape![1024]);
        let one: Tile<i32, { [1024] }> = constant(1i32, shape![1024]);
        let first: Tile<bool, { [1024] }> = eq_tile(index, zero);
        let joined: Tile<i32, { [1024] }> = joins(previous, key, gap_left, gap_right, has_gap);
        let starts: Tile<i32, { [1024] }> = select(first, one, one - joined);
        starts * in_range(index, len)
    }

    /// 1 where `key` (at `index`, before `next`) ends a range, else 0.
    fn range_ends(
        index: Tile<i32, { [1024] }>,
        key: Tile<u32, { [1024] }>,
        next: Tile<u32, { [1024] }>,
        len: i32,
        gap_left: u32,
        gap_right: u32,
        has_gap: i32,
    ) -> Tile<i32, { [1024] }> {
        let one: Tile<i32, { [1024] }> = constant(1i32, shape![1024]);
        let lasts: Tile<i32, { [1024] }> = broadcast_scalar(len - 1i32, shape![1024]);
        let last: Tile<bool, { [1024] }> = eq_tile(index, lasts);
        let joined: Tile<i32, { [1024] }> = joins(key, next, gap_left, gap_right, has_gap);
        let ends: Tile<i32, { [1024] }> = select(last, one, one - joined);
        ends * in_range(index, len)
    }

    /// 1 where sorted keys `left` then `right` belong to the same range, else 0: equal,
    /// adjacent without wrapping from `u32::MAX` to 0, or the gap pair when `has_gap` is 1.
    fn joins(
        left: Tile<u32, { [1024] }>,
        right: Tile<u32, { [1024] }>,
        gap_left: u32,
        gap_right: u32,
        has_gap: i32,
    ) -> Tile<i32, { [1024] }> {
        let zero: Tile<i32, { [1024] }> = constant(0i32, shape![1024]);
        let one: Tile<i32, { [1024] }> = constant(1i32, shape![1024]);
        let zero_key: Tile<u32, { [1024] }> = constant(0u32, shape![1024]);
        let one_key: Tile<u32, { [1024] }> = constant(1u32, shape![1024]);
        // `left + 1` wraps to 0 exactly when `left` is `u32::MAX`. (This avoids a `u32::MAX`
        // constant: cuTile compiles any `u32` constant above `i32::MAX` as 0.)
        let successor: Tile<u32, { [1024] }> = left + one_key;
        let wrapped: Tile<bool, { [1024] }> = eq_tile(successor, zero_key);
        let follows: Tile<bool, { [1024] }> = eq_tile(right, successor);
        let equal: Tile<bool, { [1024] }> = eq_tile(left, right);
        let adjacent: Tile<i32, { [1024] }> = select(wrapped, zero, select(follows, one, zero));

        let gap_lefts: Tile<u32, { [1024] }> = broadcast_scalar(gap_left, shape![1024]);
        let gap_rights: Tile<u32, { [1024] }> = broadcast_scalar(gap_right, shape![1024]);
        let has_gaps: Tile<i32, { [1024] }> = broadcast_scalar(has_gap, shape![1024]);
        let at_gap_left: Tile<i32, { [1024] }> = select(eq_tile(left, gap_lefts), one, zero);
        let at_gap_right: Tile<i32, { [1024] }> = select(eq_tile(right, gap_rights), one, zero);
        let across_gap: Tile<i32, { [1024] }> = has_gaps * at_gap_left * at_gap_right;

        let near: Tile<i32, { [1024] }> = max_tile(adjacent, across_gap);
        select(equal, one, near)
    }
}

use kernels::{count_ranges, digit_counts, emit_ranges, scatter_digits};

/// The ranges of `keys`, as `(start, end)` pairs. `gap` names two keys that count as adjacent
/// (as for `char`, across the surrogates).
///
/// # Panics
///
/// Panics if there are more than `i32::MAX` keys.
pub(super) fn ranges_32(keys: Vec<u32>, gap: Option<(u32, u32)>) -> Result<Vec<(u32, u32)>, Error> {
    if keys.is_empty() {
        return Ok(Vec::new());
    }
    let len = i32::try_from(keys.len()).expect("cuTile input exceeds i32 positions");
    let keys = copy_host_vec_to_device(&Arc::new(keys)).sync()?;
    let sorted = sort(keys, len)?;
    ranges(&sorted, len, gap)
}

/// Launch shape: one tile program per `TILE` keys.
fn tiles(len: i32) -> (usize, i32, (u32, u32, u32)) {
    let len = usize::try_from(len).expect("lengths are never negative");
    let tiles = len.div_ceil(TILE);
    let count = i32::try_from(tiles).expect("tile count fits i32");
    let grid = (u32::try_from(tiles).expect("tile count fits u32"), 1, 1);
    (tiles, count, grid)
}

/// Sorts `len` keys, one 4-bit digit per pass, from the least significant.
fn sort(keys: Tensor<u32>, len: i32) -> Result<Tensor<u32>, Error> {
    let (tiles, tile_count, grid) = tiles(len);
    let mut keys = keys;
    let n = usize::try_from(len).expect("lengths are never negative");
    let mut spare = zeros::<u32>(&[n]).sync()?;
    // Reused every pass, so shared: reading a plain `Tensor` back to the host consumes it.
    let counts = Arc::new(zeros::<i32>(&[DIGITS * tiles]).sync()?);
    for pass in 0..u32::BITS / DIGIT_BITS {
        let shift = pass * DIGIT_BITS;
        // SAFETY: `keys` holds `len` keys and `counts` 16 words per tile.
        unsafe {
            digit_counts(
                counts.device_pointer(),
                keys.device_pointer(),
                len,
                shift,
                tile_count,
            )
        }
        .grid(grid)
        .sync()?;

        // Where each tile's keys of each digit go: an exclusive scan, digit-major.
        let mut bases = (&counts).to_host_vec().sync()?;
        // A pass where every key has the same digit would leave the keys in place.
        if bases
            .chunks(tiles)
            .any(|digit| digit.iter().sum::<i32>() == len)
        {
            continue;
        }
        let mut placed = 0;
        for base in &mut bases {
            (*base, placed) = (placed, placed + *base);
        }
        let bases = copy_host_vec_to_device(&Arc::new(bases)).sync()?;
        // SAFETY: `keys` and `spare` hold `len` keys, and `bases` (an exclusive scan of every
        // tile's digit counts) places each key in `0..len` exactly once.
        unsafe {
            scatter_digits(
                spare.device_pointer(),
                keys.device_pointer(),
                bases.device_pointer(),
                len,
                shift,
                tile_count,
            )
        }
        .grid(grid)
        .sync()?;
        core::mem::swap(&mut keys, &mut spare);
    }
    Ok(keys)
}

/// The ranges of `len` sorted keys, as `(start, end)` pairs.
fn ranges(
    sorted: &Tensor<u32>,
    len: i32,
    gap: Option<(u32, u32)>,
) -> Result<Vec<(u32, u32)>, Error> {
    let (tiles, _, grid) = tiles(len);
    let (gap_left, gap_right) = gap.unwrap_or((0, 0));
    let has_gap = i32::from(gap.is_some());
    let counts = zeros::<i32>(&[tiles]).sync()?;
    // SAFETY: `sorted` holds `len` keys and `counts` one word per tile.
    unsafe {
        count_ranges(
            counts.device_pointer(),
            sorted.device_pointer(),
            len,
            gap_left,
            gap_right,
            has_gap,
        )
    }
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
    // SAFETY: `sorted` holds `len` keys, `first_range` one word per tile, and `starts` and `ends`
    // one word per range (the counts' total).
    unsafe {
        emit_ranges(
            starts.device_pointer(),
            ends.device_pointer(),
            sorted.device_pointer(),
            first_range.device_pointer(),
            len,
            gap_left,
            gap_right,
            has_gap,
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

    /// The ranges of keys, found on the CPU.
    fn cpu_ranges(keys: &[u32], gap: Option<(u32, u32)>) -> Vec<(u32, u32)> {
        let mut sorted = keys.to_vec();
        sorted.sort_unstable();
        let mut ranges: Vec<(u32, u32)> = Vec::new();
        for key in sorted {
            match ranges.last_mut() {
                Some((_, end))
                    if key == *end
                        || (*end != u32::MAX && key == *end + 1)
                        || gap == Some((*end, key)) =>
                {
                    *end = key;
                }
                _ => ranges.push((key, key)),
            }
        }
        ranges
    }

    /// Pseudo-random keys below `limit`.
    fn random_keys(count: usize, limit: u32, mut state: u32) -> Vec<u32> {
        (0..count)
            .map(|_| {
                state ^= state << 13;
                state ^= state >> 17;
                state ^= state << 5;
                state % limit
            })
            .collect()
    }

    #[test]
    fn ranges_32_matches_cpu() {
        let mut cases: Vec<Vec<u32>> = vec![
            vec![],
            vec![7],
            vec![u32::MAX, 3, 1, 10, 8, 3, 2, 7, u32::MAX - 1],
            vec![u32::MAX, 0],
            (0..5000).rev().collect(),
            (0..3000).map(|i| i * 2).collect(),
            // Duplicates across many tiles, and every digit of every pass.
            random_keys(100_000, 300_000, 0x2545_f491),
            random_keys(70_001, u32::MAX, 0x9e37_79b9),
        ];
        // Already sorted, which skips no passes but must stay sorted.
        cases.push((0..10_000).collect());
        for keys in cases {
            assert_eq!(
                ranges_32(keys.clone(), None).expect("cuTile launch failed"),
                cpu_ranges(&keys, None),
                "on {} keys",
                keys.len()
            );
        }
    }

    #[test]
    fn engine_cutile_matches_from_iter() {
        use crate::not_nan::nnf32;
        use crate::{Engine, NotNanF32, RangeSetBlaze};

        let keys = random_keys(50_000, 200_000, 0x1234_5678);
        let mut u32s = keys.clone();
        let mut i32s: Vec<i32> = keys
            .iter()
            .map(|&key| key.cast_signed() - 100_000)
            .collect();
        let mut chars: Vec<char> = keys
            .iter()
            .map(|&key| char::from_u32(0xd000 + key % 0x2000).unwrap_or('\u{e000}'))
            .collect();
        #[expect(
            clippy::cast_precision_loss,
            reason = "the keys are below 2^24, so exact"
        )]
        let mut floats: Vec<NotNanF32> = keys
            .iter()
            .map(|&key| nnf32(key as f32 - 100_000.0))
            .collect();
        assert_eq!(
            RangeSetBlaze::from_iter(u32s.clone()),
            RangeSetBlaze::<u32>::from_shuffled_slice(&mut u32s, Engine::CuTile)
        );
        assert_eq!(
            RangeSetBlaze::from_iter(i32s.clone()),
            RangeSetBlaze::<i32>::from_shuffled_slice(&mut i32s, Engine::CuTile)
        );
        assert_eq!(
            RangeSetBlaze::from_iter(chars.clone()),
            RangeSetBlaze::<char>::from_shuffled_slice(&mut chars, Engine::CuTile)
        );
        assert_eq!(
            RangeSetBlaze::from_iter(floats.clone()),
            RangeSetBlaze::<NotNanF32>::from_shuffled_slice(&mut floats, Engine::CuTile)
        );
    }

    #[test]
    fn ranges_32_joins_across_the_gap() {
        let gap = Some((0xd7ff, 0xe000));
        let keys = vec![0xe001, 0xd7fe, 0xe000, 0xd7ff, 0xe003];
        assert_eq!(
            ranges_32(keys.clone(), gap).expect("cuTile launch failed"),
            cpu_ranges(&keys, gap)
        );
        assert_eq!(
            cpu_ranges(&keys, gap),
            vec![(0xd7fe, 0xe001), (0xe003, 0xe003)]
        );
    }
}
