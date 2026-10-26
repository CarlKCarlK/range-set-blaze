//! Experimental cuTile kernels: set ingestion for keys encoded as `u32` or `u64`, and map
//! ingestion for keys encoded as `u32`.
//!
//! A cuTile program works on whole tiles of values rather than on single threads: the compiler
//! decides how threads and warps share each tile. Each tile here is `TILE` keys.
//!
//! **Sort** (least-significant digit first, 3 bits per pass): each pass takes two launches with a
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
use cutile::prelude::{DType, DeviceOp, IntoDeviceOp};
use cutile::tensor::{Tensor, ToHostVec};
use cutile::tile_kernel::TileKernel;

/// Keys per tile. The tile shapes in [`kernels`] must match.
const TILE: usize = 1024;
/// Bits per sort digit, and the number of digit values. `kernels` must match.
const DIGIT_BITS: u32 = 3;
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

    // Kernels are generic over the key type `K` (`u32` or `u64`); the host names it with
    // `.generics(..)`. cuTile converts only to and from floats, not between integer types, so a
    // generic kernel can't make a `K` constant from a literal: the host passes `key_zero`,
    // `key_one`, and `digit_mask` (7), and the kernels derive the rest with tile arithmetic.
    //
    // The cuTile compiler can't infer the type of a bare `constant(..)` argument, so constants
    // get annotated `let`s; the other intermediate values follow suit for readability.
    //
    // Flags are `i32` 0 or 1 rather than `bool` tiles, because this cuTile version compiles
    // `constant(false, ..)` as true (https://github.com/NVIDIA/cuda-rust/issues/1469; fixed by
    // https://github.com/NVIDIA/cuda-rust/pull/1470, not yet merged).

    /// Writes how many of each tile's keys have each digit (the 3 bits at `shift`) to
    /// `counts[digit * tiles + tile]`: the column sums of the tile's digit flags.
    ///
    /// # Safety
    ///
    /// `keys` must point to `len` keys and `counts` to 8 words per tile.
    #[cutile::entry()]
    unsafe fn digit_counts<K: ElementType>(
        counts: *mut i32,
        keys: *mut K,
        len: i32,
        shift: K,
        key_zero: K,
        key_one: K,
        digit_mask: K,
        tiles: i32,
    ) {
        let tile: i32 = get_tile_block_id().0;
        let index: Tile<i32, { [1024] }> = tile_indices(tile);
        let zero_key: K = key_zero;
        // SAFETY: `keys` points to `len` keys.
        let key: Tile<K, { [1024] }> = unsafe { gather(keys, index, len, zero_key) };
        let digit: Tile<K, { [1024] }> = digits(key, shift, key_one, digit_mask, index, len);
        let flags: Tile<i32, { [1024, 8] }> = digit_flags(digit);
        let count: Tile<i32, { [8] }> = reduce_sum(flags, 0i32);
        // `counts[value * tiles + tile]` for each digit value.
        let values: Tile<i32, { [8] }> = iota(shape![8]);
        let tile_counts: Tile<i32, { [8] }> = broadcast_scalar(tiles, shape![8]);
        let this_tile: Tile<i32, { [8] }> = broadcast_scalar(tile, shape![8]);
        let offsets: Tile<i32, { [8] }> = values * tile_counts + this_tile;
        let count_slots: PointerTile<*mut i32, { [8] }> = slots_8(counts, offsets);
        // SAFETY: `counts` has 8 words per tile.
        unsafe {
            store_ptr_tko(
                count_slots,
                count,
                ordering::Weak,
                None::<scope::TileBlock>,
                None,
                None,
                Latency::<0>,
            );
        }
    }

    /// Moves each key to `sorted`, ordered by its digit (the 3 bits at `shift`), keeping keys
    /// with the same digit in order. `bases[digit * tiles + tile]` is where this tile's first key
    /// with that digit goes. When `carry` is 1, each key's position in `positions` moves with
    /// it to `sorted_positions`.
    ///
    /// # Safety
    ///
    /// `keys` and `sorted` must each point to `len` keys, and `bases` to 8 words per tile that
    /// place every key in `0..len` exactly once. When `carry` is 1, `positions` and
    /// `sorted_positions` must each point to `len` positions.
    #[cutile::entry()]
    unsafe fn scatter_digits<K: ElementType>(
        sorted: *mut K,
        sorted_positions: *mut i32,
        keys: *mut K,
        positions: *mut i32,
        carry: i32,
        bases: *mut i32,
        len: i32,
        shift: K,
        key_zero: K,
        key_one: K,
        digit_mask: K,
        tiles: i32,
    ) {
        let tile: i32 = get_tile_block_id().0;
        let index: Tile<i32, { [1024] }> = tile_indices(tile);
        let zero_key: K = key_zero;
        // SAFETY: `keys` points to `len` keys.
        let key: Tile<K, { [1024] }> = unsafe { gather(keys, index, len, zero_key) };
        let digit: Tile<K, { [1024] }> = digits(key, shift, key_one, digit_mask, index, len);
        let zero: Tile<i32, { [1024] }> = constant(0i32, shape![1024]);
        let one: Tile<i32, { [1024] }> = constant(1i32, shape![1024]);
        let bases_base: PointerTile<*mut i32, { [] }> = pointer_to_tile(bases);

        // Each key's destination: its digit's base for this tile, plus its rank among this
        // tile's keys with the same digit, one digit value at a time. (With 4-bit digits,
        // scanning all 16 columns of `digit_flags` at once was about 40% slower.)
        let mut destination: Tile<i32, { [1024] }> = constant(0i32, shape![1024]);
        let step: Tile<K, { [1024] }> = broadcast_scalar(key_one, shape![1024]);
        let mut wanted: Tile<K, { [1024] }> = broadcast_scalar(zero_key, shape![1024]);
        for value in 0i32..8i32 {
            let matches: Tile<bool, { [1024] }> = eq_tile(digit, wanted);
            let flags: Tile<i32, { [1024] }> = select(matches, one, zero);
            let through: Tile<i32, { [1024] }> = scan_sum(flags, 0i32, reverse::Forward, 0i32);
            let before: Tile<i32, { [1024] }> = through - flags;
            let base_slot: PointerTile<*mut i32, { [] }> = addptr(bases_base, value * tiles + tile);
            // SAFETY: `bases` has 8 words per tile.
            let (base, _): (Tile<i32, { [] }>, Token) = unsafe {
                load_ptr_tko(
                    base_slot,
                    ordering::Weak,
                    None::<scope::TileBlock>,
                    None,
                    None,
                    None,
                    Latency::<0>,
                )
            };
            let base_1: Tile<i32, { [1] }> = base.reshape(shape![1]);
            let base_all: Tile<i32, { [1024] }> = base_1.broadcast(shape![1024]);
            destination = destination + flags * (base_all + before);
            wanted = wanted + step;
        }

        let valid: Tile<i32, { [1024] }> = in_range(index, len);
        let valid_mask: Tile<bool, { [1024] }> = eq_tile(valid, one);
        let sorted_slots: PointerTile<*mut K, { [1024] }> = slots(sorted, destination);
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
        if carry == 1i32 {
            // SAFETY: `positions` points to `len` positions.
            let position: Tile<i32, { [1024] }> = unsafe { gather(positions, index, len, 0i32) };
            let position_slots: PointerTile<*mut i32, { [1024] }> =
                slots(sorted_positions, destination);
            // SAFETY: as for the keys.
            unsafe {
                store_ptr_tko(
                    position_slots,
                    position,
                    ordering::Weak,
                    None::<scope::TileBlock>,
                    Some(valid_mask),
                    None,
                    Latency::<0>,
                );
            }
        }
    }

    /// Writes how many ranges start in each tile of sorted keys to `counts[tile]`. When
    /// `has_gap` is 1, keys `gap_left` and `gap_right` count as adjacent.
    ///
    /// # Safety
    ///
    /// `sorted` must point to `len` keys and `counts` to one word per tile.
    #[cutile::entry()]
    unsafe fn count_ranges<K: ElementType>(
        counts: *mut i32,
        sorted: *mut K,
        len: i32,
        gap_left: K,
        gap_right: K,
        has_gap: i32,
        key_zero: K,
        key_one: K,
    ) {
        let tile: i32 = get_tile_block_id().0;
        let index: Tile<i32, { [1024] }> = tile_indices(tile);
        let zero_key: K = key_zero;
        let one: Tile<i32, { [1024] }> = constant(1i32, shape![1024]);
        let previous_index: Tile<i32, { [1024] }> = index - one;
        // SAFETY: `sorted` points to `len` keys.
        let key: Tile<K, { [1024] }> = unsafe { gather(sorted, index, len, zero_key) };
        let previous: Tile<K, { [1024] }> =
            unsafe { gather(sorted, previous_index, len, zero_key) };
        let joined: Tile<i32, { [1024] }> =
            joins(previous, key, gap_left, gap_right, has_gap, key_one);
        let starts: Tile<i32, { [1024] }> = range_starts(index, joined, len);
        let count: Tile<i32, { [] }> = reduce_sum(starts, 0i32);
        // SAFETY: `counts` has one word per tile.
        unsafe { store_count(counts, tile, count) };
    }

    /// Writes each range's first key to `starts` and its last key to `ends`. `first_range[tile]`
    /// is the number of ranges that start before this tile. The gap is as in `count_ranges`.
    ///
    /// # Safety
    ///
    /// `sorted` must point to `len` keys, `first_range` to one word per tile, and `starts` and
    /// `ends` to one word per range.
    #[cutile::entry()]
    unsafe fn emit_ranges<K: ElementType>(
        starts: *mut K,
        ends: *mut K,
        sorted: *mut K,
        first_range: *mut i32,
        len: i32,
        gap_left: K,
        gap_right: K,
        has_gap: i32,
        key_zero: K,
        key_one: K,
    ) {
        let tile: i32 = get_tile_block_id().0;
        let index: Tile<i32, { [1024] }> = tile_indices(tile);
        let zero_key: K = key_zero;
        let one: Tile<i32, { [1024] }> = constant(1i32, shape![1024]);
        let previous_index: Tile<i32, { [1024] }> = index - one;
        let next_index: Tile<i32, { [1024] }> = index + one;
        // SAFETY: `sorted` points to `len` keys.
        let key: Tile<K, { [1024] }> = unsafe { gather(sorted, index, len, zero_key) };
        let previous: Tile<K, { [1024] }> =
            unsafe { gather(sorted, previous_index, len, zero_key) };
        let next: Tile<K, { [1024] }> = unsafe { gather(sorted, next_index, len, zero_key) };
        let joined_before: Tile<i32, { [1024] }> =
            joins(previous, key, gap_left, gap_right, has_gap, key_one);
        let joined_after: Tile<i32, { [1024] }> =
            joins(key, next, gap_left, gap_right, has_gap, key_one);
        let is_start: Tile<i32, { [1024] }> = range_starts(index, joined_before, len);
        let is_end: Tile<i32, { [1024] }> = range_ends(index, joined_after, len);
        // SAFETY: `first_range` has one word per tile.
        let range: Tile<i32, { [1024] }> = unsafe { numbers(first_range, tile, is_start) };
        // SAFETY: `starts` and `ends` have one word per range, and each range has exactly one
        // start and one end.
        unsafe {
            scatter_where(starts, range, key, is_start);
            scatter_where(ends, range, key, is_end);
        }
    }

    /// Writes how many of each tile's sorted keys are the last of their run of equal keys to
    /// `counts[tile]`: the entries that survive, since a map's last entry for a key wins.
    ///
    /// # Safety
    ///
    /// `sorted` must point to `len` keys and `counts` to one word per tile.
    #[cutile::entry()]
    unsafe fn count_survivors(counts: *mut i32, sorted: *mut u32, len: i32) {
        let tile: i32 = get_tile_block_id().0;
        let index: Tile<i32, { [1024] }> = tile_indices(tile);
        // SAFETY: `sorted` points to `len` keys.
        let survives: Tile<i32, { [1024] }> = unsafe { survivors(sorted, index, len) };
        let count: Tile<i32, { [] }> = reduce_sum(survives, 0i32);
        // SAFETY: `counts` has one word per tile.
        unsafe { store_count(counts, tile, count) };
    }

    /// Packs the surviving keys and their positions. `first_survivor[tile]` is the number of
    /// survivors before this tile.
    ///
    /// # Safety
    ///
    /// `sorted` and `positions` must point to `len` entries, `first_survivor` to one word per
    /// tile, and `survivor_keys` and `survivor_positions` to one word per survivor.
    #[cutile::entry()]
    unsafe fn emit_survivors(
        survivor_keys: *mut u32,
        survivor_positions: *mut i32,
        sorted: *mut u32,
        positions: *mut i32,
        first_survivor: *mut i32,
        len: i32,
    ) {
        let tile: i32 = get_tile_block_id().0;
        let index: Tile<i32, { [1024] }> = tile_indices(tile);
        // SAFETY: `sorted` and `positions` point to `len` entries.
        let survives: Tile<i32, { [1024] }> = unsafe { survivors(sorted, index, len) };
        let key: Tile<u32, { [1024] }> = unsafe { gather(sorted, index, len, 0u32) };
        let position: Tile<i32, { [1024] }> = unsafe { gather(positions, index, len, 0i32) };
        // SAFETY: `first_survivor` has one word per tile.
        let slot: Tile<i32, { [1024] }> = unsafe { numbers(first_survivor, tile, survives) };
        // SAFETY: the outputs have one word per survivor.
        unsafe {
            scatter_where(survivor_keys, slot, key, survives);
            scatter_where(survivor_positions, slot, position, survives);
        }
    }

    /// Writes how many map ranges start in each tile of survivors to `counts[tile]`: where the
    /// keys aren't adjacent, or the values differ. Each survivor's value is
    /// `(value_lo[position], value_hi[position])`, or just the low word when `has_hi` is 0;
    /// there are `value_count` values. The gap is as in `count_ranges`.
    ///
    /// # Safety
    ///
    /// `keys` and `positions` must point to `len` survivors whose positions are below
    /// `value_count`, `value_lo` (and, when `has_hi` is 1, `value_hi`) to `value_count` words,
    /// and `counts` to one word per tile.
    #[cutile::entry()]
    unsafe fn count_map_ranges(
        counts: *mut i32,
        keys: *mut u32,
        positions: *mut i32,
        value_lo: *mut u32,
        value_hi: *mut u32,
        has_hi: i32,
        value_count: i32,
        len: i32,
        gap_left: u32,
        gap_right: u32,
        has_gap: i32,
    ) {
        let tile: i32 = get_tile_block_id().0;
        let index: Tile<i32, { [1024] }> = tile_indices(tile);
        let one: Tile<i32, { [1024] }> = constant(1i32, shape![1024]);
        let previous_index: Tile<i32, { [1024] }> = index - one;
        // SAFETY: as documented above.
        let joined: Tile<i32, { [1024] }> = unsafe {
            map_joins(
                keys,
                positions,
                value_lo,
                value_hi,
                has_hi,
                value_count,
                previous_index,
                index,
                len,
                gap_left,
                gap_right,
                has_gap,
            )
        };
        let starts: Tile<i32, { [1024] }> = range_starts(index, joined, len);
        let count: Tile<i32, { [] }> = reduce_sum(starts, 0i32);
        // SAFETY: `counts` has one word per tile.
        unsafe { store_count(counts, tile, count) };
    }

    /// Writes each map range's first key to `starts`, its last key to `ends`, and its first
    /// survivor's position to `firsts`. `first_range[tile]` is the number of ranges that start
    /// before this tile. The rest is as in `count_map_ranges`.
    ///
    /// # Safety
    ///
    /// As for `count_map_ranges`, with `first_range` pointing to one word per tile, and
    /// `starts`, `ends`, and `firsts` to one word per range.
    #[cutile::entry()]
    unsafe fn emit_map_ranges(
        starts: *mut u32,
        ends: *mut u32,
        firsts: *mut i32,
        keys: *mut u32,
        positions: *mut i32,
        value_lo: *mut u32,
        value_hi: *mut u32,
        has_hi: i32,
        value_count: i32,
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
        // SAFETY: as documented above.
        let joined_before: Tile<i32, { [1024] }> = unsafe {
            map_joins(
                keys,
                positions,
                value_lo,
                value_hi,
                has_hi,
                value_count,
                previous_index,
                index,
                len,
                gap_left,
                gap_right,
                has_gap,
            )
        };
        let joined_after: Tile<i32, { [1024] }> = unsafe {
            map_joins(
                keys,
                positions,
                value_lo,
                value_hi,
                has_hi,
                value_count,
                index,
                next_index,
                len,
                gap_left,
                gap_right,
                has_gap,
            )
        };
        let key: Tile<u32, { [1024] }> = unsafe { gather(keys, index, len, 0u32) };
        let position: Tile<i32, { [1024] }> = unsafe { gather(positions, index, len, 0i32) };
        let is_start: Tile<i32, { [1024] }> = range_starts(index, joined_before, len);
        let is_end: Tile<i32, { [1024] }> = range_ends(index, joined_after, len);
        // SAFETY: `first_range` has one word per tile.
        let range: Tile<i32, { [1024] }> = unsafe { numbers(first_range, tile, is_start) };
        // SAFETY: the outputs have one word per range, and each range has exactly one start
        // and one end.
        unsafe {
            scatter_where(starts, range, key, is_start);
            scatter_where(firsts, range, position, is_start);
            scatter_where(ends, range, key, is_end);
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

    /// Pointers to `base[index]` for 8 indices.
    fn slots_8(base: *mut i32, index: Tile<i32, { [8] }>) -> PointerTile<*mut i32, { [8] }> {
        let base: PointerTile<*mut i32, { [] }> = pointer_to_tile(base);
        let base_1: PointerTile<*mut i32, { [1] }> = base.reshape(shape![1]);
        let bases: PointerTile<*mut i32, { [8] }> = base_1.broadcast(shape![8]);
        bases.offset_tile(index)
    }

    /// The values at `index`, or `fill` where `index` is outside `0..len`.
    ///
    /// # Safety
    ///
    /// `base` must point to `len` values.
    unsafe fn gather<E: ElementType>(
        base: *mut E,
        index: Tile<i32, { [1024] }>,
        len: i32,
        fill: E,
    ) -> Tile<E, { [1024] }> {
        let one: Tile<i32, { [1024] }> = constant(1i32, shape![1024]);
        let in_bounds: Tile<bool, { [1024] }> = eq_tile(in_range(index, len), one);
        let value_slots: PointerTile<*mut E, { [1024] }> = slots(base, index);
        // SAFETY: the mask skips every index outside `0..len`.
        let (values, _): (Tile<E, { [1024] }>, Token) = unsafe {
            load_ptr_tko(
                value_slots,
                ordering::Weak,
                None::<scope::TileBlock>,
                Some(in_bounds),
                Some(fill),
                None,
                Latency::<0>,
            )
        };
        values
    }

    /// Stores `count` to `counts[tile]`.
    ///
    /// # Safety
    ///
    /// `counts` must have a word for `tile`.
    unsafe fn store_count(counts: *mut i32, tile: i32, count: Tile<i32, { [] }>) {
        let counts_base: PointerTile<*mut i32, { [] }> = pointer_to_tile(counts);
        let slot: PointerTile<*mut i32, { [] }> = addptr(counts_base, tile);
        // SAFETY: `counts` has a word for `tile`.
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

    /// Numbers the marked entries from `first[tile]` up: each entry gets the number of the last
    /// marked entry at or before it.
    ///
    /// # Safety
    ///
    /// `first` must have a word for `tile`.
    unsafe fn numbers(
        first: *mut i32,
        tile: i32,
        marks: Tile<i32, { [1024] }>,
    ) -> Tile<i32, { [1024] }> {
        let one: Tile<i32, { [1024] }> = constant(1i32, shape![1024]);
        let first_base: PointerTile<*mut i32, { [] }> = pointer_to_tile(first);
        let first_slot: PointerTile<*mut i32, { [] }> = addptr(first_base, tile);
        // SAFETY: `first` has a word for `tile`.
        let (before, _): (Tile<i32, { [] }>, Token) = unsafe {
            load_ptr_tko(
                first_slot,
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
        let marked: Tile<i32, { [1024] }> = scan_sum(marks, 0i32, reverse::Forward, 0i32);
        before_all + marked - one
    }

    /// Stores `values` to `base[slot]` where `marks` is 1.
    ///
    /// # Safety
    ///
    /// Every marked slot must be in `base` and written by no other entry.
    unsafe fn scatter_where<E: ElementType>(
        base: *mut E,
        slot: Tile<i32, { [1024] }>,
        values: Tile<E, { [1024] }>,
        marks: Tile<i32, { [1024] }>,
    ) {
        let one: Tile<i32, { [1024] }> = constant(1i32, shape![1024]);
        let mask: Tile<bool, { [1024] }> = eq_tile(marks, one);
        let value_slots: PointerTile<*mut E, { [1024] }> = slots(base, slot);
        // SAFETY: every marked slot is in `base` and written once.
        unsafe {
            store_ptr_tko(
                value_slots,
                values,
                ordering::Weak,
                None::<scope::TileBlock>,
                Some(mask),
                None,
                Latency::<0>,
            );
        }
    }

    /// Each key's digit, the bits of `digit_mask` (7) at `shift`; 8, which matches no digit,
    /// where `index` is outside `0..len`.
    fn digits<K: ElementType>(
        key: Tile<K, { [1024] }>,
        shift: K,
        key_one: K,
        digit_mask: K,
        index: Tile<i32, { [1024] }>,
        len: i32,
    ) -> Tile<K, { [1024] }> {
        let one: Tile<i32, { [1024] }> = constant(1i32, shape![1024]);
        let low_bits: Tile<K, { [1024] }> = broadcast_scalar(digit_mask, shape![1024]);
        let ones: Tile<K, { [1024] }> = broadcast_scalar(key_one, shape![1024]);
        let none: Tile<K, { [1024] }> = low_bits + ones;
        let shifts: Tile<K, { [1024] }> = broadcast_scalar(shift, shape![1024]);
        let shifted: Tile<K, { [1024] }> = shri(key, shifts);
        let digit: Tile<K, { [1024] }> = andi(shifted, low_bits);
        let valid: Tile<bool, { [1024] }> = eq_tile(in_range(index, len), one);
        select(valid, digit, none)
    }

    /// A 1024 × 8 tile with a 1 in each key's row at its digit's column, and 0 elsewhere (an
    /// all-0 row where the digit is 8).
    fn digit_flags<K: ElementType>(digit: Tile<K, { [1024] }>) -> Tile<i32, { [1024, 8] }> {
        let zero: Tile<i32, { [1024, 8] }> = constant(0i32, shape![1024, 8]);
        let one: Tile<i32, { [1024, 8] }> = constant(1i32, shape![1024, 8]);
        let digit_column: Tile<K, { [1024, 1] }> = digit.reshape(shape![1024, 1]);
        let digits: Tile<K, { [1024, 8] }> = digit_column.broadcast(shape![1024, 8]);
        let values: Tile<K, { [8] }> = iota(shape![8]);
        let values_row: Tile<K, { [1, 8] }> = values.reshape(shape![1, 8]);
        let all_values: Tile<K, { [1024, 8] }> = values_row.broadcast(shape![1024, 8]);
        let matches: Tile<bool, { [1024, 8] }> = eq_tile(digits, all_values);
        select(matches, one, zero)
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

    /// 1 where the entry at `index` starts a range (it's first, or doesn't join the one
    /// before), else 0.
    fn range_starts(
        index: Tile<i32, { [1024] }>,
        joined_before: Tile<i32, { [1024] }>,
        len: i32,
    ) -> Tile<i32, { [1024] }> {
        let zero: Tile<i32, { [1024] }> = constant(0i32, shape![1024]);
        let one: Tile<i32, { [1024] }> = constant(1i32, shape![1024]);
        let first: Tile<bool, { [1024] }> = eq_tile(index, zero);
        let starts: Tile<i32, { [1024] }> = select(first, one, one - joined_before);
        starts * in_range(index, len)
    }

    /// 1 where the entry at `index` ends a range (it's last, or the one after doesn't join
    /// it), else 0.
    fn range_ends(
        index: Tile<i32, { [1024] }>,
        joined_after: Tile<i32, { [1024] }>,
        len: i32,
    ) -> Tile<i32, { [1024] }> {
        let one: Tile<i32, { [1024] }> = constant(1i32, shape![1024]);
        let lasts: Tile<i32, { [1024] }> = broadcast_scalar(len - 1i32, shape![1024]);
        let last: Tile<bool, { [1024] }> = eq_tile(index, lasts);
        let ends: Tile<i32, { [1024] }> = select(last, one, one - joined_after);
        ends * in_range(index, len)
    }

    /// 1 where sorted keys `left` then `right` belong to the same range, else 0: equal, or
    /// [`adjacent`].
    fn joins<K: ElementType>(
        left: Tile<K, { [1024] }>,
        right: Tile<K, { [1024] }>,
        gap_left: K,
        gap_right: K,
        has_gap: i32,
        key_one: K,
    ) -> Tile<i32, { [1024] }> {
        let one: Tile<i32, { [1024] }> = constant(1i32, shape![1024]);
        let equal: Tile<bool, { [1024] }> = eq_tile(left, right);
        let near: Tile<i32, { [1024] }> =
            adjacent(left, right, gap_left, gap_right, has_gap, key_one);
        select(equal, one, near)
    }

    /// 1 where `right` follows `left` (without wrapping from the maximum key to 0), or the two
    /// are the gap pair when `has_gap` is 1; else 0.
    fn adjacent<K: ElementType>(
        left: Tile<K, { [1024] }>,
        right: Tile<K, { [1024] }>,
        gap_left: K,
        gap_right: K,
        has_gap: i32,
        key_one: K,
    ) -> Tile<i32, { [1024] }> {
        let zero: Tile<i32, { [1024] }> = constant(0i32, shape![1024]);
        let one: Tile<i32, { [1024] }> = constant(1i32, shape![1024]);
        let one_key: Tile<K, { [1024] }> = broadcast_scalar(key_one, shape![1024]);
        let zero_key: Tile<K, { [1024] }> = one_key - one_key;
        // `left + 1` wraps to 0 exactly when `left` is the maximum key. (This avoids a maximum
        // constant: cuTile compiles any unsigned constant above the signed maximum as 0, issue
        // #1469.)
        let successor: Tile<K, { [1024] }> = left + one_key;
        let wrapped: Tile<bool, { [1024] }> = eq_tile(successor, zero_key);
        let follows: Tile<bool, { [1024] }> = eq_tile(right, successor);
        let next: Tile<i32, { [1024] }> = select(wrapped, zero, select(follows, one, zero));

        let gap_lefts: Tile<K, { [1024] }> = broadcast_scalar(gap_left, shape![1024]);
        let gap_rights: Tile<K, { [1024] }> = broadcast_scalar(gap_right, shape![1024]);
        let has_gaps: Tile<i32, { [1024] }> = broadcast_scalar(has_gap, shape![1024]);
        let at_gap_left: Tile<i32, { [1024] }> = select(eq_tile(left, gap_lefts), one, zero);
        let at_gap_right: Tile<i32, { [1024] }> = select(eq_tile(right, gap_rights), one, zero);
        let across_gap: Tile<i32, { [1024] }> = has_gaps * at_gap_left * at_gap_right;
        max_tile(next, across_gap)
    }

    /// 1 where the sorted entry at `index` is the last of its run of equal keys, else 0.
    ///
    /// # Safety
    ///
    /// `sorted` must point to `len` keys.
    unsafe fn survivors(
        sorted: *mut u32,
        index: Tile<i32, { [1024] }>,
        len: i32,
    ) -> Tile<i32, { [1024] }> {
        let zero: Tile<i32, { [1024] }> = constant(0i32, shape![1024]);
        let one: Tile<i32, { [1024] }> = constant(1i32, shape![1024]);
        let next_index: Tile<i32, { [1024] }> = index + one;
        // SAFETY: `sorted` points to `len` keys.
        let key: Tile<u32, { [1024] }> = unsafe { gather(sorted, index, len, 0u32) };
        let next: Tile<u32, { [1024] }> = unsafe { gather(sorted, next_index, len, 0u32) };
        let lasts: Tile<i32, { [1024] }> = broadcast_scalar(len - 1i32, shape![1024]);
        let last: Tile<bool, { [1024] }> = eq_tile(index, lasts);
        let differs: Tile<bool, { [1024] }> = ne_tile(key, next);
        let survives: Tile<i32, { [1024] }> = select(last, one, select(differs, one, zero));
        survives * in_range(index, len)
    }

    /// 1 where survivors `left` and `right` belong to the same map range (adjacent keys and
    /// equal values), else 0.
    ///
    /// # Safety
    ///
    /// As for `count_map_ranges`.
    unsafe fn map_joins(
        keys: *mut u32,
        positions: *mut i32,
        value_lo: *mut u32,
        value_hi: *mut u32,
        has_hi: i32,
        value_count: i32,
        left: Tile<i32, { [1024] }>,
        right: Tile<i32, { [1024] }>,
        len: i32,
        gap_left: u32,
        gap_right: u32,
        has_gap: i32,
    ) -> Tile<i32, { [1024] }> {
        let zero: Tile<i32, { [1024] }> = constant(0i32, shape![1024]);
        let one: Tile<i32, { [1024] }> = constant(1i32, shape![1024]);
        // SAFETY: as for `count_map_ranges`; out-of-range entries read as 0 and are masked by
        // the callers.
        let left_key: Tile<u32, { [1024] }> = unsafe { gather(keys, left, len, 0u32) };
        let right_key: Tile<u32, { [1024] }> = unsafe { gather(keys, right, len, 0u32) };
        let left_position: Tile<i32, { [1024] }> = unsafe { gather(positions, left, len, 0i32) };
        let right_position: Tile<i32, { [1024] }> = unsafe { gather(positions, right, len, 0i32) };
        let near: Tile<i32, { [1024] }> =
            adjacent(left_key, right_key, gap_left, gap_right, has_gap, 1u32);
        // SAFETY: every position is below `value_count`.
        let left_lo: Tile<u32, { [1024] }> =
            unsafe { gather(value_lo, left_position, value_count, 0u32) };
        let right_lo: Tile<u32, { [1024] }> =
            unsafe { gather(value_lo, right_position, value_count, 0u32) };
        let same_lo: Tile<i32, { [1024] }> = select(eq_tile(left_lo, right_lo), one, zero);
        let same: Tile<i32, { [1024] }> = if has_hi == 1i32 {
            // SAFETY: `value_hi` has a word for every position when `has_hi` is 1.
            let left_hi: Tile<u32, { [1024] }> =
                unsafe { gather(value_hi, left_position, value_count, 0u32) };
            let right_hi: Tile<u32, { [1024] }> =
                unsafe { gather(value_hi, right_position, value_count, 0u32) };
            same_lo * select(eq_tile(left_hi, right_hi), one, zero)
        } else {
            same_lo
        };
        near * same
    }
}

use kernels::{
    count_map_ranges, count_ranges, count_survivors, digit_counts, emit_map_ranges, emit_ranges,
    emit_survivors, scatter_digits,
};

/// A key type the kernels handle: `u32` or `u64`.
pub(super) trait Key: DType + Copy + IntoDeviceOp<Self> {
    /// The type's name, which instantiates the generic kernels.
    const NAME: &'static str;
    /// `bits` as a key, for shift amounts.
    fn from_bits(bits: u32) -> Self;
}

impl Key for u32 {
    const NAME: &'static str = "u32";
    fn from_bits(bits: u32) -> Self {
        bits
    }
}

impl Key for u64 {
    const NAME: &'static str = "u64";
    fn from_bits(bits: u32) -> Self {
        Self::from(bits)
    }
}

/// The ranges of `keys`, as `(start, end)` pairs. `gap` names two keys that count as adjacent
/// (as for `char`, across the surrogates).
///
/// # Panics
///
/// Panics if there are more than `i32::MAX` keys.
pub(super) fn ranges<K: Key>(keys: Vec<K>, gap: Option<(K, K)>) -> Result<Vec<(K, K)>, Error> {
    if keys.is_empty() {
        return Ok(Vec::new());
    }
    let len = i32::try_from(keys.len()).expect("cuTile input exceeds i32 positions");
    let keys = copy_host_vec_to_device(&Arc::new(keys)).sync()?;
    let (sorted, _) = sort(keys, None, len)?;
    sorted_ranges(&sorted, len, gap)
}

/// A map's ranges as `(start, end)` keys, and each range's first input position.
pub(super) type MapRanges = (Vec<(u32, u32)>, Vec<u32>);

/// A map's ranges from its keys and its values' one or two words: each range's `(start, end)`
/// keys, and the input position of its first entry (whose value is the range's). Where a key
/// repeats, its last entry wins.
///
/// # Panics
///
/// Panics if there are more than `i32::MAX` entries.
pub(super) fn map_ranges(
    keys: Vec<u32>,
    gap: Option<(u32, u32)>,
    value_lo: Vec<u32>,
    value_hi: Option<Vec<u32>>,
) -> Result<MapRanges, Error> {
    if keys.is_empty() {
        return Ok((Vec::new(), Vec::new()));
    }
    let len = i32::try_from(keys.len()).expect("cuTile input exceeds i32 positions");
    let n = keys.len();
    let keys = copy_host_vec_to_device(&Arc::new(keys)).sync()?;
    let positions = copy_host_vec_to_device(&Arc::new((0..len).collect::<Vec<i32>>())).sync()?;
    let (sorted, positions) = sort(keys, Some(positions), len)?;
    let positions = positions.expect("the sort carries positions when given them");

    // Survivors: the last entry for each key.
    let (tile_len, _, grid) = tiles(len);
    let counts = zeros::<i32>(&[tile_len]).sync()?;
    // SAFETY: `sorted` holds `len` keys and `counts` one word per tile.
    unsafe { count_survivors(counts.device_pointer(), sorted.device_pointer(), len) }
        .grid(grid)
        .sync()?;
    let (first_survivor, survivor_count) = exclusive_scan(counts.to_host_vec().sync()?);
    let first_survivor = copy_host_vec_to_device(&Arc::new(first_survivor)).sync()?;
    let survivor_len = usize::try_from(survivor_count).expect("counts are never negative");
    let survivor_keys = zeros::<u32>(&[survivor_len]).sync()?;
    let survivor_positions = zeros::<i32>(&[survivor_len]).sync()?;
    // SAFETY: `sorted` and `positions` hold `len` entries, `first_survivor` one word per tile,
    // and the outputs one word per survivor.
    unsafe {
        emit_survivors(
            survivor_keys.device_pointer(),
            survivor_positions.device_pointer(),
            sorted.device_pointer(),
            positions.device_pointer(),
            first_survivor.device_pointer(),
            len,
        )
    }
    .grid(grid)
    .sync()?;

    // Ranges among the survivors: keys adjacent and values equal.
    let value_count = i32::try_from(n).expect("checked above");
    let value_lo = copy_host_vec_to_device(&Arc::new(value_lo)).sync()?;
    let has_hi = i32::from(value_hi.is_some());
    // Without high words, the low words stand in; `has_hi` 0 means they're never read.
    let value_hi = match value_hi {
        Some(value_hi) => Some(copy_host_vec_to_device(&Arc::new(value_hi)).sync()?),
        None => None,
    };
    let value_hi_pointer = value_hi.as_ref().unwrap_or(&value_lo).device_pointer();
    let (gap_left, gap_right) = gap.unwrap_or((0, 0));
    let has_gap = i32::from(gap.is_some());
    let (survivor_tiles, _, survivor_grid) = tiles(survivor_count);
    let counts = zeros::<i32>(&[survivor_tiles]).sync()?;
    // SAFETY: the survivors' positions are input positions, below `value_count`, the number of
    // words in each value array; `counts` has one word per tile.
    unsafe {
        count_map_ranges(
            counts.device_pointer(),
            survivor_keys.device_pointer(),
            survivor_positions.device_pointer(),
            value_lo.device_pointer(),
            value_hi_pointer,
            has_hi,
            value_count,
            survivor_count,
            gap_left,
            gap_right,
            has_gap,
        )
    }
    .grid(survivor_grid)
    .sync()?;
    let (first_range, range_count) = exclusive_scan(counts.to_host_vec().sync()?);
    let first_range = copy_host_vec_to_device(&Arc::new(first_range)).sync()?;
    let range_len = usize::try_from(range_count).expect("counts are never negative");
    let starts = zeros::<u32>(&[range_len]).sync()?;
    let ends = zeros::<u32>(&[range_len]).sync()?;
    let firsts = zeros::<i32>(&[range_len]).sync()?;
    // SAFETY: as for `count_map_ranges`, and the outputs hold one word per range.
    unsafe {
        emit_map_ranges(
            starts.device_pointer(),
            ends.device_pointer(),
            firsts.device_pointer(),
            survivor_keys.device_pointer(),
            survivor_positions.device_pointer(),
            value_lo.device_pointer(),
            value_hi_pointer,
            has_hi,
            value_count,
            first_range.device_pointer(),
            survivor_count,
            gap_left,
            gap_right,
            has_gap,
        )
    }
    .grid(survivor_grid)
    .sync()?;

    let starts = starts.to_host_vec().sync()?;
    let ends = ends.to_host_vec().sync()?;
    let firsts = firsts.to_host_vec().sync()?;
    Ok((
        starts.into_iter().zip(ends).collect(),
        firsts
            .into_iter()
            .map(|first| u32::try_from(first).expect("positions are never negative"))
            .collect(),
    ))
}

/// Whether there's a CUDA device to run on.
#[cfg(test)]
pub(super) fn available() -> bool {
    cutile::prelude::Device::new(0).is_ok()
}

/// Launch shape: one tile program per `TILE` entries.
fn tiles(len: i32) -> (usize, i32, (u32, u32, u32)) {
    let len = usize::try_from(len).expect("lengths are never negative");
    let tiles = len.div_ceil(TILE);
    let count = i32::try_from(tiles).expect("tile count fits i32");
    let grid = (u32::try_from(tiles).expect("tile count fits u32"), 1, 1);
    (tiles, count, grid)
}

/// Each count's exclusive prefix sum, and the total.
fn exclusive_scan(mut counts: Vec<i32>) -> (Vec<i32>, i32) {
    let mut total = 0;
    for count in &mut counts {
        (*count, total) = (total, total + *count);
    }
    (counts, total)
}

/// Sorts `len` keys, one 3-bit digit per pass, from the least significant, carrying
/// `positions` along if given. (3 bits took about 15% less time than 4 or 2: each pass's
/// ranking does one scan per digit value, so narrower digits mean cheaper passes, but more of
/// them.)
fn sort<K: Key>(
    keys: Tensor<K>,
    positions: Option<Tensor<i32>>,
    len: i32,
) -> Result<(Tensor<K>, Option<Tensor<i32>>), Error> {
    let (tiles, tile_count, grid) = tiles(len);
    let n = usize::try_from(len).expect("lengths are never negative");
    let mut keys = keys;
    let mut spare = zeros::<K>(&[n]).sync()?;
    let mut positions = positions;
    let mut spare_positions = match positions {
        Some(_) => Some(zeros::<i32>(&[n]).sync()?),
        None => None,
    };
    let carry = i32::from(positions.is_some());
    // Reused every pass, so shared: reading a plain `Tensor` back to the host consumes it.
    let counts = Arc::new(zeros::<i32>(&[DIGITS * tiles]).sync()?);
    let key_bits = u32::try_from(size_of::<K>() * 8).expect("keys are 32 or 64 bits");
    for pass in 0..key_bits.div_ceil(DIGIT_BITS) {
        let shift = K::from_bits(pass * DIGIT_BITS);
        // SAFETY: `keys` holds `len` keys and `counts` 8 words per tile.
        unsafe {
            digit_counts(
                counts.device_pointer(),
                keys.device_pointer(),
                len,
                shift,
                K::from_bits(0),
                K::from_bits(1),
                K::from_bits((1 << DIGIT_BITS) - 1),
                tile_count,
            )
        }
        .generics(std::vec![K::NAME.into()])
        .grid(grid)
        .sync()?;

        // Where each tile's keys of each digit go: an exclusive scan, digit-major.
        let digit_counts = (&counts).to_host_vec().sync()?;
        // A pass where every key has the same digit would leave the keys in place.
        if digit_counts
            .chunks(tiles)
            .any(|digit| digit.iter().sum::<i32>() == len)
        {
            continue;
        }
        let (bases, _) = exclusive_scan(digit_counts);
        let bases = copy_host_vec_to_device(&Arc::new(bases)).sync()?;
        // Without positions, `counts` stands in for them; `carry` 0 means it's never touched.
        let position_pointer = positions
            .as_ref()
            .map_or_else(|| counts.device_pointer(), Tensor::device_pointer);
        let spare_position_pointer = spare_positions
            .as_ref()
            .map_or_else(|| counts.device_pointer(), Tensor::device_pointer);
        // SAFETY: `keys` and `spare` (and, when carried, the positions) hold `len` entries, and
        // `bases` (an exclusive scan of every tile's digit counts) places each in `0..len`
        // exactly once.
        unsafe {
            scatter_digits(
                spare.device_pointer(),
                spare_position_pointer,
                keys.device_pointer(),
                position_pointer,
                carry,
                bases.device_pointer(),
                len,
                shift,
                K::from_bits(0),
                K::from_bits(1),
                K::from_bits((1 << DIGIT_BITS) - 1),
                tile_count,
            )
        }
        .generics(std::vec![K::NAME.into()])
        .grid(grid)
        .sync()?;
        core::mem::swap(&mut keys, &mut spare);
        core::mem::swap(&mut positions, &mut spare_positions);
    }
    Ok((keys, positions))
}

/// The ranges of `len` sorted keys, as `(start, end)` pairs.
fn sorted_ranges<K: Key>(
    sorted: &Tensor<K>,
    len: i32,
    gap: Option<(K, K)>,
) -> Result<Vec<(K, K)>, Error> {
    let (tiles, _, grid) = tiles(len);
    let zero = K::from_bits(0);
    let (gap_left, gap_right) = gap.unwrap_or((zero, zero));
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
            K::from_bits(0),
            K::from_bits(1),
        )
    }
    .generics(std::vec![K::NAME.into()])
    .grid(grid)
    .sync()?;

    let (first_range, range_count) = exclusive_scan(counts.to_host_vec().sync()?);
    let range_len = usize::try_from(range_count).expect("counts are never negative");
    let first_range = copy_host_vec_to_device(&Arc::new(first_range)).sync()?;
    let starts = zeros::<K>(&[range_len]).sync()?;
    let ends = zeros::<K>(&[range_len]).sync()?;
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
            K::from_bits(0),
            K::from_bits(1),
        )
    }
    .generics(std::vec![K::NAME.into()])
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
    fn ranges_matches_cpu() {
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
                ranges(keys.clone(), None).expect("cuTile launch failed"),
                cpu_ranges(&keys, None),
                "on {} keys",
                keys.len()
            );
        }
    }

    #[test]
    fn ranges_matches_cpu_for_u64() {
        let mut state = 0x9e37_79b9_7f4a_7c15_u64;
        let mut next = move || {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            state
        };
        let cases: Vec<Vec<u64>> = vec![
            vec![u64::MAX, 0, u64::MAX - 1, 1],
            // Runs that cross from the low 32 bits into the high ones.
            (0..5000).map(|i| 0xffff_f000 + i).rev().collect(),
            // Every digit of every pass.
            (0..70_001).map(|_| next()).collect(),
            // Duplicates within a narrow band near the top.
            (0..50_000).map(|_| u64::MAX - next() % 100_000).collect(),
        ];
        for keys in cases {
            let mut sorted = keys.clone();
            sorted.sort_unstable();
            let mut expected: Vec<(u64, u64)> = Vec::new();
            for key in sorted {
                match expected.last_mut() {
                    Some((_, end)) if key == *end || (*end != u64::MAX && key == *end + 1) => {
                        *end = key;
                    }
                    _ => expected.push((key, key)),
                }
            }
            assert_eq!(
                ranges(keys.clone(), None).expect("cuTile launch failed"),
                expected,
                "on {} keys",
                keys.len()
            );
        }
    }

    #[test]
    fn engine_cutile_matches_from_iter_for_64_bit_keys() {
        use crate::not_nan::nnf64;
        use crate::{Engine, NotNanF64, RangeSetBlaze};

        let keys = random_keys(50_000, 200_000, 0x8765_4321);
        let mut u64s: Vec<u64> = keys.iter().map(|&key| u64::from(key) << 20).collect();
        let mut i64s: Vec<i64> = keys.iter().map(|&key| i64::from(key) - 100_000).collect();
        let mut floats: Vec<NotNanF64> = keys
            .iter()
            .map(|&key| nnf64(f64::from(key) - 100_000.0))
            .collect();
        assert_eq!(
            RangeSetBlaze::from_iter(u64s.clone()),
            RangeSetBlaze::<u64>::from_shuffled_slice(&mut u64s, Engine::CuTile)
        );
        assert_eq!(
            RangeSetBlaze::from_iter(i64s.clone()),
            RangeSetBlaze::<i64>::from_shuffled_slice(&mut i64s, Engine::CuTile)
        );
        assert_eq!(
            RangeSetBlaze::from_iter(floats.clone()),
            RangeSetBlaze::<NotNanF64>::from_shuffled_slice(&mut floats, Engine::CuTile)
        );
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
    fn ranges_joins_across_the_gap() {
        let gap = Some((0xd7ff, 0xe000));
        let keys = vec![0xe001, 0xd7fe, 0xe000, 0xd7ff, 0xe003];
        assert_eq!(
            ranges(keys.clone(), gap).expect("cuTile launch failed"),
            cpu_ranges(&keys, gap)
        );
        assert_eq!(
            cpu_ranges(&keys, gap),
            vec![(0xd7fe, 0xe001), (0xe003, 0xe003)]
        );
    }
}
