# cuda-oxide vs cuTile: the same pipeline, written two ways

range-set-blaze has two NVIDIA engines that do the same job: sort a shuffled slice on the GPU,
then find its ranges (and, for maps, the value of each range).

- `Engine::Cuda` ([`src/gpu/cuda.rs`](../src/gpu/cuda.rs)) is written with **cuda-oxide**:
  thread-level kernels in Rust, much like CUDA C++. Each kernel says what one thread does, with
  thread and block indices, shared memory, warp intrinsics, and atomics.
- `Engine::CuTile` ([`src/gpu/cutile.rs`](../src/gpu/cutile.rs)) is written with **cuTile**:
  tile-level kernels in Rust. Each kernel says what one program does to a whole tile of 1,024
  values, with operations such as `scan_sum`, `reduce_sum`, `select`, and gather/scatter through
  tiles of pointers. The compiler decides how threads and warps share the work.

Both come from NVIDIA's [cuda-rust](https://github.com/NVIDIA/cuda-rust) repository, pinned here at
`ad0a42c`. Both run on Linux only (here, WSL2), and both are behind experimental features.

## Step 1: mark where each range starts

Given sorted keys, a key starts a new range unless it equals, or is one more than, the key before
it.

**cuda-oxide**: one thread per key. Each thread writes only its own element, so the output is a
`DisjointSlice` and the launch is safe:

```rust
#[kernel]
#[launch_bounds(256)]
#[launch_contract(domain = 1, block = (256, 1, 1))]
pub(super) fn mark_starts(
    sorted: &[u32],
    gap_left: u32,
    gap_right: u32,
    has_gap: u32,
    mut starts: DisjointSlice<'_, u32>,
) {
    if let Some((start, index)) = starts.get_mut_indexed() {
        let i = index.get();
        *start =
            u32::from(i == 0 || !joins(sorted[i - 1], sorted[i], gap_left, gap_right, has_gap));
    }
}
```

**cuTile**: one program per tile. It loads its 1,024 keys and the 1,024 keys just before them,
compares the two tiles, and sums the result. The scan that turns these marks into range numbers is
the built-in `scan_sum`. In cuda-oxide, the scan is 100 lines of hand-written kernels.

```rust
#[cutile::entry()]
unsafe fn count_ranges<K: ElementType>(
    counts: *mut i32,
    sorted: *mut K,
    len: i32,
    /* gap and key-constant arguments */
) {
    let tile: i32 = get_tile_block_id().0;
    let index: Tile<i32, { [1024] }> = tile_indices(tile);
    let one: Tile<i32, { [1024] }> = constant(1i32, shape![1024]);
    let previous_index: Tile<i32, { [1024] }> = index - one;
    // SAFETY: `sorted` points to `len` keys.
    let key: Tile<K, { [1024] }> = unsafe { gather(sorted, index, len, key_zero) };
    let previous: Tile<K, { [1024] }> = unsafe { gather(sorted, previous_index, len, key_zero) };
    let joined: Tile<i32, { [1024] }> = joins(previous, key, gap_left, gap_right, has_gap, key_one);
    let starts: Tile<i32, { [1024] }> = range_starts(index, joined, len);
    let count: Tile<i32, { [] }> = reduce_sum(starts, 0i32);
    // SAFETY: `counts` has one word per tile.
    unsafe { store_count(counts, tile, count) };
}
```

## Step 2: rank each key within its digit (the heart of the radix sort)

Each pass of a least-significant-digit radix sort moves every key to its digit's bucket while
keeping keys in order within a bucket. For that, each key needs its **rank**: how many keys before
it in its tile have the same digit.

**cuda-oxide**: 8-bit digits, 4 passes. A warp finds each key's peers (lanes with the same digit)
with one instruction, `match_any_sync`, and the rank is a population count. Tiles find their global
offsets by decoupled look-back (a tile waits on the tiles before it), so each pass is one kernel.

```rust
let peers = warp::match_any_sync(0xffff_ffff, digit);
let rank = (peers & lane_below).count_ones();
```

**cuTile**: 3-bit digits, 11 passes for 32-bit keys. Tiles don't expose warps, so the rank comes
from one tile-wide scan per digit value:

```rust
for value in 0i32..8i32 {
    let matches: Tile<bool, { [1024] }> = eq_tile(digit, wanted);
    let flags: Tile<i32, { [1024] }> = select(matches, one, zero);
    let through: Tile<i32, { [1024] }> = scan_sum(flags, 0i32, reverse::Forward, 0i32);
    let before: Tile<i32, { [1024] }> = through - flags;
    // ... load this tile's base for `value` ...
    destination = destination + flags * (base_all + before);
    wanted = wanted + step;
}
```

That loop is where cuTile's time goes. At 100M keys, `scatter_digits` spent about 21 ms per pass
on its scans, while moving the keys took only about 2 ms. 3-bit digits were about 15% faster than
4-bit or 2-bit ones, trading cheaper passes against more of them. cuTile has no warp-level
look-back either, so each pass takes two launches with a host scan of the digit counts between
them.

## Size and safety

Lines of code, not counting blanks and comments:

| Part | cuda-oxide | cuTile |
| --- | ---: | ---: |
| Sort kernels | 230 | 139 |
| Host side of the sort | 69 | 73 |
| Finding ranges, including the scan | 147 | 140 |
| Uses of `unsafe` in the file | 57 | 66 |

cuTile isn't shorter for range finding: its built-in scan saves a hundred lines, but every tile
value needs its own typed `let`. It isn't safer here either. Gathering and scattering through raw
pointers is `unsafe` in both, and both engines document each `unsafe` with a `SAFETY` comment. What
cuTile removes is the thread-level machinery: indices, shared memory, `sync_threads`, warp
intrinsics, and atomics.

## Speed (RTX 3060, WSL2, one run)

Sets, 100M shuffled keys:

| Type | `Cpu(16)` | cuda-oxide | cuTile |
| --- | ---: | ---: | ---: |
| `u32` | 269 ms | **154 ms** | 264 ms |
| `NotNanF32` | 362 ms | **148 ms** | 249 ms |
| `u64` | 484 ms | **316 ms** | 473 ms |
| `NotNanF64` | 579 ms | **342 ms** | 443 ms |

Maps, 100M shuffled pairs:

| Ranges | CPU builder | cuda-oxide | cuTile |
| ---: | ---: | ---: | ---: |
| 1 | 791 ms | **352 ms** | 464 ms |
| 12.5M | 689 ms | **426 ms** | 548 ms |
| 100M | **3,143 ms** | 4,782 ms | 4,851 ms |

The CPU numbers in this run are noisier than in earlier runs (100M `i32` took 521 ms here, against
276 ms before). Details and other sizes are in [CUDA_ENGINE_BENCHMARK.md](CUDA_ENGINE_BENCHMARK.md).

## What got in the way with cuTile

All of these are in cuda-rust `ad0a42c`; the first two are also on `main` as of 2026-10-25.

1. **`constant(false, ..)` compiles as true**, and **any unsigned constant above the signed maximum
   compiles as 0** (`u32` above `i32::MAX`). Both are silent: the first made a bounds mask let
   out-of-range lanes load, which crashed with an illegal-address error. Reported as
   [cuda-rust#1469](https://github.com/NVIDIA/cuda-rust/issues/1469), with a fix in
   [#1470](https://github.com/NVIDIA/cuda-rust/pull/1470). The workarounds: flags are `i32` 0 or 1,
   and "is `x` the maximum?" is tested as `x + 1 == 0`.
2. **No conversions between integer types.** `convert_scalar` and `convert_tile` handle only
   floats: `i32` to `u32` and `u32` to `u64` are rejected. A kernel generic over its key type
   therefore can't make a key-typed constant, so the host passes `key_zero`, `key_one`, and the
   digit mask as arguments.
3. **No arithmetic on generic scalars.** In Rust's type checker, `ElementType` doesn't imply `Add`
   or `Sub`, so a generic `K` scalar can't be added to; a `Tile<K, _>` can. Derived constants are
   computed on tiles.
4. **Every tile value needs a typed `let`.** The compiler can't infer the type of a bare
   `constant(..)` argument, and paths such as `u32::MAX` aren't accepted as constants.
5. **`no_std`.** `#[cutile::module]` expands to code that uses the std prelude, so this
   `no_std` crate imports `Vec`, `String`, and `format!` inside the module.
6. **Kernels compile on first use** (about 3 s here), with `tileiras` from the CUDA toolkit, which
   must therefore be installed wherever the program runs. cuda-oxide compiles kernels with the
   program instead.

## For the workshop

- **cuTile's tiles are the easier model to explain.** "Load a tile, compare it with its
  neighbor, scan, scatter" fits on a slide. cuda-oxide's sort needs warps, shared memory, and a
  look-back protocol before it makes sense.
- **cuda-oxide is about twice as fast here**, because sorting rewards exactly the warp-level tricks
  tiles hide. cuTile still beats a 16-thread CPU sort for 32-bit keys at 100M.
- **cuTile is younger.** Two silent miscompiles and several missing conversions turned up in a few
  hundred lines of kernel code.
