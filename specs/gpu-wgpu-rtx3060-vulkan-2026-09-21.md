<!-- todo​0 consider deleting this note once the wgpu experiment is either productized or rejected. -->

# wgpu/Vulkan benchmark run: RTX 3060, 2026-09-21

Provenance for
[`gpu-wgpu-rtx3060-vulkan-2026-09-21.csv`](gpu-wgpu-rtx3060-vulkan-2026-09-21.csv).

This was a complete real-hardware sweep: 100K, 1M, 10M, and 100M inputs all
completed across random gaps, duplicate-heavy input, and clumps of 16, 256,
and 4096 values. It is separate from automated sandbox validation, where only
the llvmpipe software adapter was exposed. The public GPU policy rejects that
software adapter and uses its CPU fallback; the measurements below came from
the RTX 3060 through wgpu's Vulkan backend.

## Headline result

At 100M `u32` values, the saved end-to-end medians were:

| Distribution | Output ranges | CPU `from_slice` | wgpu GPU | Speedup |
| --- | ---: | ---: | ---: | ---: |
| random gaps | 96.8M | 7,099.3 ms | 2,048.0 ms | 3.5x |
| duplicate heavy | 10.7M | 5,284.0 ms | 331.8 ms | 15.9x |
| clumps 16 | 6.25M | 5,212.3 ms | 243.6 ms | 21.4x |
| clumps 256 | 390K | 5,128.5 ms | 149.4 ms | 34.3x |
| clumps 4096 | 24.4K | 5,188.5 ms | 147.0 ms | 35.3x |

Portable GPU ingestion can substantially accelerate construction from very
large unsorted slices, especially when the resulting set compresses into
relatively few ranges. This does not imply a universal GPU advantage: existing
CPU construction remains appropriate for small inputs, CPU algorithms already
handle sorted or clumpy inputs efficiently, and poorly compressed output adds
substantial range-readback and final `BTreeMap` materialization costs.

## Experimental status and follow-ups

The public policy remains experimental, opt-in, and off by default. It adds an
explicit GPU-enabled constructor without changing any existing CPU constructor.
The implementation is portable wgpu/Lampshade code; earlier CUDA, cuda-oxide,
cuTile, and CUB results remain in this directory only as historical evidence.

Known follow-ups deliberately left out of the initial policy include:

- Reuse buffers, pipelines, sorters, and compactors across public calls; only
  the GPU context is currently cached.
- Tune the deliberately conservative crossover threshold using broader
  hardware data.
- Benchmark more adapters before treating the policy as production-tuned.
- Investigate 128-bit keys separately; they remain CPU-only, as do
  `RangeMapBlaze` operations.

## Environment

- GPU: NVIDIA GeForce RTX 3060 (12 GiB)
- wgpu backend: Vulkan (adapter-reported; wgpu picked this automatically on
  Windows)
- OS: Windows 11 Pro, target `x86_64-pc-windows-msvc`
- Rust: `rustc 1.100.0-nightly (5a2be9f5f 2026-09-06)`, toolchain
  `nightly-x86_64-pc-windows-msvc`, active via a `rustup` directory override
  for this repo (the repo's own `rust-toolchain.toml` pins stable `1.97.0`;
  the override took precedence for this run, so this is not the pinned CI
  toolchain)
- `wgpu` 30.0.1, `lampshade` 0.13.0, `pollster` 0.4.0, `bytemuck` 1.25.2 (all
  from crates.io, exact versions as locked in `Cargo.lock` at the commit
  below)
- Commit: `81ccb880e61f82fb375dd9c72a08be55f4c9dc0e` ("Add wgpu-based
  normalization experiment and enhance GPU backend support")

## Command

```bash
cargo run --release --bin gpu_normalize_wgpu --features gpu-wgpu
```

That command records the feature name at the measured commit. The finalized
branch renames the portable feature to `gpu`.

No positional size arguments were passed, so `common::requested_sizes()`
used its default sweep: 100,000 / 1,000,000 / 10,000,000 / 100,000,000,
across all `UNSORTED_DISTRIBUTIONS` (random gaps, duplicate heavy, clumps-16,
clumps-256, clumps-4096).

## What the numbers are

Every timing column is a **median**, not a mean or single sample:
`experiments/gpu_normalize/common.rs::median` takes the middle of a
sorted sample vector. Sample counts follow
`common::iterations(len)`: 9 iterations at 100K, 7 at 1M, 5 at 10M, 3 at
100M.

- `cpu_from_slice_ms` / `cpu_from_iter_ms`: median CPU
  `RangeSetBlaze::from_slice` / `from_iter` time over that many iterations.
- `gpu_upload_ms` / `gpu_radix_sort_ms` / `gpu_normalization_ms` /
  `gpu_range_readback_ms` / `gpu_build_from_ranges_ms`: median per-stage GPU
  pipeline time (upload → CUB-style radix sort on device → boundary
  normalization → range readback → CPU-side `RangeSetBlaze` construction
  from the returned ranges), each stage timed and sorted independently.
- `gpu_end_to_end_ms`: median of the summed per-iteration staged pipeline
  (upload+sort+normalize+readback+build), timed as one `Instant` span per
  iteration, not a sum of the per-stage medians.
- `gpu_fused_end_to_end_ms`: median of a **separate** timing loop that calls
  `gpu.fused_ranges(&values)`, an internally-fused path timed end-to-end as
  one span per iteration. It is a distinct benchmark loop from
  `gpu_end_to_end_ms`, not derived from the same samples.

Correctness (GPU ranges equal the CPU-derived reference `RangeSetBlaze`, for
both the staged and fused paths) is asserted outside all timed regions
before each row's numbers are recorded.
