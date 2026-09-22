<!-- todo​0 consider deleting this report once the experiment is either productized or rejected. -->

# GPU normalization experiment

## Result

On an NVIDIA GeForce RTX 3060, the complete unsorted-input pipeline answers the
main question positively: one upload, CUB radix sort, Rust cuda-oxide
normalization, compact range readback, and trusted `RangeSetBlaze`
materialization beats the existing CPU constructors at every measured size and
distribution. The advantage is largest when the final range list is compact.

The sorted-input-only result is more limited. GPU boundary detection is useful
when the output contains many ranges, but CPU `from_slice` is already extremely
fast for sorted long runs. A GPU post-sort API is not attractive by itself for
highly compressed sorted inputs.

Full measurements are in:

- [`gpu_unsorted_results.csv`](gpu_unsorted_results.csv)
- [`gpu_post_sort_results.csv`](gpu_post_sort_results.csv)

## Environment

- GPU: NVIDIA GeForce RTX 3060, 12 GiB, compute capability 8.6
- CUDA toolkit: 13.3
- cuda-oxide: commit `b0f961df3af0ff140b3b006fa2b6750b71f43f62`
- cuTile: commit `e04245bdcf1f5bfc602a2078168eff90ee40bebb`
- cuda-oxide toolchain: `nightly-2026-08-28`
- cuTile toolchain: `nightly-2026-09-21`

Times are medians in milliseconds. Allocation, CUDA context creation, module
loading, and JIT compilation are warm-up/setup costs outside the timed path.
Persistent device allocations and CUB temporary storage are reused across
iterations.

The current native build recipe is intentionally machine-oriented: its default
architecture is `sm_86` and its CCCL include suffix is
`targets/x86_64-linux/include/cccl`. `CUDA_HOME` and
`RANGE_SET_BLAZE_CUDA_ARCH` are configurable, but a productized feature would
also need target-aware include discovery.

## Complete unsorted pipeline

The implemented data path is:

```text
host shuffled &[u32]
  -> one H2D upload
  -> CUB DeviceRadixSort::SortKeys (device to device)
  -> Rust cuda-oxide boundary detection, count scan, and compaction
  -> compact range D2H readback
  -> trusted SortedDisjoint iterator
  -> RangeSetBlaze::from_sorted_disjoint
```

The CUB wrapper contains only the radix-sort call, pointer conversions, selected
output-buffer reporting, and CUDA error propagation. Rust owns both key
buffers and the temporary-storage buffer. CUB borrows their device pointers on
the existing cuda-oxide stream. The selected sorted buffer is consumed directly
by the Rust kernel; sorted values never return to the host.

### 100M-item results

| Distribution | Ranges | Ratio | CPU slice | Upload | Sort | Normalize | Readback | Build | GPU total |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| random gaps | 96,775,149 | 1.03 | 7,595.36 | 30.06 | 31.73 | 339.58 | 553.15 | 1,330.83 | 2,298.04 |
| duplicate heavy | 10,714,037 | 9.33 | 4,521.84 | 25.72 | 12.85 | 86.99 | 15.34 | 65.66 | 206.31 |
| clumps 16 | 6,250,000 | 16.00 | 4,996.32 | 25.43 | 13.42 | 75.37 | 14.16 | 33.48 | 161.89 |
| clumps 256 | 390,625 | 256.00 | 4,726.85 | 26.54 | 13.14 | 63.94 | 1.05 | 2.06 | 105.82 |
| clumps 4096 | 24,415 | 4,095.84 | 4,669.34 | 25.59 | 13.11 | 61.71 | 0.25 | 0.13 | 101.45 |

At 100M values, output range count explains most of the remaining variation.
With 96.8M ranges, range readback plus B-tree construction takes about 1.88
seconds. With 24K ranges, those stages take about 0.38 ms combined; upload,
radix sort, and Rust normalization dominate instead.

The 10M run encountered intermittent whole-GPU slow intervals: upload, sort,
and normalization rose together for duplicate-heavy and clumps-16 inputs.
Those rows are retained rather than selectively discarded. End-to-end GPU time
was still 8.5x to 67x faster than CPU `from_slice` for those two rows. Repeated
benchmarking under controlled GPU clocks would be appropriate before claiming
precise throughput numbers.

## Output materialization

The benchmark uses a private `TrustedGpuRanges` iterator implementing
`Iterator`, `FusedIterator`, `SortedStarts<u32>`, and `SortedDisjoint<u32>`.
Timed construction does not use `CheckSortedDisjoint` and does not rescan the
ranges for ordering, overlap, adjacency, or emptiness.

This makes output-size effects explicit. At 100M shuffled inputs:

- 96.8M ranges require 1,330.83 ms to materialize the B-tree;
- 10.7M ranges require 65.66 ms;
- 6.25M ranges require 33.48 ms;
- 390K ranges require 2.06 ms;
- 24K ranges require 0.13 ms.

Thus B-tree construction is material only when GPU output itself is large. It
is not a limiting cost in the high-compression regime.

## Correctness

For every measured size and distribution, correctness checks run outside all
timed regions:

1. CPU `RangeSetBlaze::from_iter` must equal CPU `from_slice`.
2. The complete GPU range vector must equal the CPU reference range vector
   element for element.
3. Materializing the trusted GPU iterator must produce a `RangeSetBlaze` equal
   to the CPU reference set.

The post-sort cuda-oxide and cuTile experiments also compare their complete
range vectors with the CPU-normalized vector.

## Post-sort backend comparison

cuTile's tile scan is substantially faster than the current cuda-oxide
prototype's deliberately simple one-thread-per-1024-values boundary kernel and
serial cross-chunk scan. At 100M random-gap values, cuTile's device pipeline was
15.72 ms versus cuda-oxide's 782.55 ms in the recorded run.

That does not make sorted compact inputs a good GPU target. At 100M values and
24K ranges, CPU `from_slice` took 12.79 ms, cuTile end-to-end took 55.46 ms, and
cuda-oxide took 193.69 ms. The existing CPU SIMD path benefits strongly from
long sorted runs and avoids transfer overhead.

The complete unsorted pipeline changes the conclusion because CPU sorting
dominates. For the same 100M/24K-range shape after element-level shuffling, CPU
`from_slice` took 4,669.34 ms and the CUB plus Rust GPU pipeline took 101.45 ms.

## Current API findings

Neither pinned Rust API exposes reusable device-wide radix sort, compaction, or
deduplication. cuda-oxide provides Rust warp/block scans and reductions; cuTile
provides tile-local scans and reductions. The cuda-oxide CCCL example uses CUDA
C++ wrappers and does not expose a host Rust `DeviceRadixSort` API.

The experiment therefore uses the newly permitted minimal native wrapper for
CUB keys-only radix sort. All normalization stages remain Rust implementations.

## Commands

```bash
RUSTUP_TOOLCHAIN=nightly-2026-08-28 \
  cargo oxide run gpu_normalize_oxide \
  --bin gpu_normalize_oxide --features gpu-cub -- \
  --unsorted 100000 1000000 10000000 100000000

RUSTUP_TOOLCHAIN=nightly-2026-08-28 \
  cargo oxide run gpu_normalize_oxide \
  --bin gpu_normalize_oxide --features gpu-oxide -- \
  100000 1000000 10000000 100000000

RUSTUP_TOOLCHAIN=nightly-2026-09-21 \
  cargo run --release --bin gpu_normalize_cutile \
  --features gpu-cutile -- \
  100000 1000000 10000000 100000000
```
