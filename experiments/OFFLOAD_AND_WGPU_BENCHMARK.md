# Offload and wgpu experiments, November 13, 2026

Hardware: NVIDIA GeForce RTX 3060 (`sm_86`), Ryzen 9 7950X, 32 logical processors.
Windows measurements were launched from WSL2 but used native Windows Rust binaries.
Both Windows wgpu runs selected the RTX 3060 through Vulkan. WSL2 wgpu selected
llvmpipe, so its hardware timing columns are empty; CUDA engines used the real GPU.

## Method

All numbers below are end-to-end set construction for one million shuffled distinct
`u32` keys, producing 3,907 ranges. The existing harness validates each engine against
the CPU result, warms each operation for at least one second, and reports the median
of five timed samples. Input cloning is outside the timer; encoding, transfers,
sorting, range assembly, and destruction of the result are included.

| Implementation | Environment | Time | CPU comparison in that run |
| --- | --- | ---: | ---: |
| Initial offload bitonic, transfers every pass | WSL2 | 553.085 ms | 3.930 ms, 16 threads |
| Resident offload bitonic, fused block-local passes | WSL2 | 16.647 ms | 3.268 ms, 16 threads |
| Offload radix prototype | WSL2 | 10.366 ms | 3.355 ms, 16 threads |
| Radix with warp-aggregated histogram and tested allocation-cache setting | WSL2 | 10.542 ms | 3.095 ms, 16 threads |
| wgpu before sorter/workspace reuse | Windows | 6.864 ms | 2.455 ms, 32 threads |
| wgpu with sorter/workspace reuse | Windows | 3.992 ms | 2.614 ms, 32 threads |

These are different runs, not a controlled compiler-only comparison. cuTile and CPU
timings also varied between runs. A 3.454 ms offload sample with allocation caching
did not reproduce reliably and should not be used as a promised speedup. GPU-scoped
atomics were also tried and did not demonstrate a repeatable benefit; the retained
radix implementation uses standard global atomics and explicit block-scoped shared
histogram updates.

The earlier CUDA-Oxide result was 2.454 ms at this size, with a 3.229 ms CPU result;
that engine was not rerun in this experiment. See its [saved data](results/cuda_sets_rtx3060_wsl.csv).
Larger sizes and different output compression can change the comparison.

## What changed

Offload now defaults to stable 8-bit LSD radix sort above 1,024 keys, retaining fused
bitonic sort for smaller inputs and explicit comparisons. It ports CUDA-Oxide's
warp ranking, shared-memory scatter, and ticket-ordered look-back. Five small PTX
helpers supply shared storage, warp matching/shuffling/synchronization, and shared
histogram updates. Global status and histogram counters use standard Rust atomics.
Constant byte passes are skipped. Keys stay resident throughout sorting, but are
still stored as `u64` words and downloaded for CPU range assembly.

The 32-bit wgpu set path now reuses its Lampshade sorter and compactor, including
their pipelines and scratch workspace. Input, boundary, and endpoint buffers are
still per-call. A mutex protects workspace until GPU completion/readback. The cache
uses the actual process-wide context address, whose static lifetime prevents address
reuse; device IDs alone proved insufficient across distinct wgpu instances.
Retained scratch capacity can keep GPU memory allocated between calls. It is cleared
after allocation failure. The 64-bit set and map paths are unchanged.

## Reproduction

WSL2 offload and cuTile:

```sh
cargo +nightly build --release --locked --features offload_nightly_experimental,cutile --bin gpu_ingest
RANGE_SET_BLAZE_REQUIRE_OFFLOAD_GPU=1 target/release/gpu_ingest --cpu 16 --cpu 32 --type u32 10000 100000 1000000
```

Use `RANGE_SET_BLAZE_OFFLOAD_SORT=bitonic` or `radix` to force an algorithm. The
allocation-cache experiment also set `LIBOMPTARGET_MEMORY_MANAGER_THRESHOLD=16777216`
before starting the process. That setting is optional and is not changed by the crate.

Native Windows, using the same source and a Windows target directory:

```powershell
cargo +1.99.1 run --release --locked --features shuffled_slices --bin gpu_ingest -- --cpu 16 --cpu 32 --type u32 10000 100000 1000000
```

For source under a WSL UNC path, pass its absolute `Cargo.toml` through
`--manifest-path` and set `CARGO_TARGET_DIR` to a native Windows directory. The runs
here were managed with PowerShell from WSL2; no interactive Windows session was needed.

## Data and checks

- [Initial offload](results/offload_sets_rtx3060_wsl.csv)
- [Resident/fused bitonic](results/offload_resident_sets_rtx3060_wsl.csv)
- [First radix prototype](results/offload_radix_sets_rtx3060_wsl.csv)
- [GPU-scoped atomic experiment](results/offload_radix_scoped_sets_rtx3060_wsl.csv)
- [Retained radix with the tested cache setting](results/offload_radix_pooled_sets_rtx3060_wsl.csv)
- [Windows wgpu baseline](results/wgpu_sets_rtx3060_windows_20261113.csv)
- [Windows wgpu reuse](results/wgpu_cached_sets_rtx3060_windows_20261113.csv)

Adjacent `.log` files record the adapter, environment, and commands. Radix correctness
checks include full-width random keys, constant bytes, partial tiles, duplicate keys,
and inputs larger than the GPU's resident block capacity. wgpu checks include
growing/shrinking capacity and alternating hardware/test contexts.
