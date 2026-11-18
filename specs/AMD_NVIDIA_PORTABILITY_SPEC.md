<!-- todo​0 consider deleting this spec once the work below is implemented and released. -->

# AMD laptop and NVIDIA desktop portability experiment

## Goal and systems

Test wgpu, nightly Rust offload, and CPU SIMD portability. Benchmark each
machine's GPU against its own CPU, and report absolute times across machines
separately. Do not use the desktop CPU as the laptop GPU's baseline.

- Laptop: HP ZBook Ultra G1a 14-inch Mobile Workstation, Ryzen AI Max+ PRO 395
  (16 Zen 5 cores, 32 threads), Radeon 8060S, 128 GB unified LPDDR5X,
  Windows 11 and Ubuntu 24.04 under WSL2. Native Linux is unconfirmed.
- Desktop: Ryzen 9 7950X, NVIDIA RTX 3060, Windows and WSL2. Record installed
  memory and current drivers with each run rather than inferring them.

## Execution order

1. Run native Windows correctness tests, CPU/SIMD, and wgpu on both machines,
   managed from WSL2. Use the same source snapshot, lockfile, Rust version,
   workload seeds, types, and sizes. Include uncommitted source in the snapshot
   identifier; a commit hash alone does not identify this working tree.
2. Repeat CPU and wgpu under WSL2 where the adapter probe confirms hardware
   acceleration. Label the backend and translation layer explicitly. A software
   adapter is an unavailable GPU result, not a GPU performance measurement.
3. Probe AMD offload under WSL2 independently: runtime device enumeration,
   launch, readback, and correctness of a minimal kernel before any sort port.
4. If the probe succeeds, port shared storage and warp operations, then sort.
   Preserve stable ordering and validate partial groups and scheduling progress.
   Record any vendor-specific code needed to achieve portability.

The existing offload build targets NVPTX and its radix implementation has five
PTX helpers and 32-lane assumptions. It cannot run unchanged on the Radeon.
ROCm availability does not establish that Rust offload's compiler, linker,
OpenMP device plugin, and kernels all work on this system.

AMD documents a Strix Halo WSL path through ROCDXG:
[AMD WSL guide](https://rocmdocs.amd.com/projects/radeon-ryzen/en/latest/docs/install/installryz/wsl/howto_wsl.html).
Confirm the installed driver/runtime and exact processor against its linked
compatibility matrix before changing the laptop environment.

## Initial commands

Run from the repository on either OS with a native toolchain. For native Windows
launched from WSL2, use PowerShell, the WSL UNC manifest path, and a native Windows
`CARGO_TARGET_DIR`; see the [previous runner notes](../experiments/OFFLOAD_AND_WGPU_BENCHMARK.md#reproduction).

```sh
cargo +1.99.1 test --release --locked --features shuffled_slices --lib gpu -- --test-threads=1
cargo +1.99.1 run --release --locked --features shuffled_slices --bin gpu_ingest -- --cpu 16 --cpu 32 --type u32 10000 100000 1000000
cargo +1.99.1 bench --locked --bench bench -- ingest_clumps_simd_levels
```

The set harness includes `from_slice`, CPU(1), CPU(16), CPU(32), and a strict
hardware wgpu column. The Criterion SIMD group separately compares scalar
fallback, supported instruction levels, and runtime dispatch; CPU engine timings
alone do not isolate SIMD performance. Keep Criterion results separate from the
GPU harness because their distributions and measurement methods differ.

After correctness and limits are confirmed, repeat the set command with sizes
`10000000`, then `100000000` where all working buffers fit. Add `--type u64`
as a separate run to expose differences in key width. Do not silently fall back
to CPU on GPU allocation failure or omit a failed size without recording it.

For compression-sensitive workloads and memory-path diagnostics:

```sh
cargo +1.99.1 run --release --locked --features shuffled_slices --bin gpu_normalize_wgpu -- 1000000
cargo +1.99.1 run --release --locked --features shuffled_slices --bin gpu_normalize_wgpu -- 1000000 --staged
cargo +1.99.1 run --release --locked --features shuffled_slices --bin gpu_map_ingest -- 1000000
```

The normalization experiment reports stages and mapped-versus-staged behavior;
do not describe its timings as the library engine's timings. Shared physical
memory does not itself eliminate API copies or adapter binding limits.

## Recording and comparison

Save CSV and stderr logs under `experiments/results/` with machine, OS, backend,
and date in each name. Record source snapshot, toolchain, build flags, adapter,
driver, buffer limits, selected memory path, CPU thread counts, and runtime
environment. Keep baseline builds portable; label any `target-cpu=native`
experiment separately and never move that executable between machines.

Run the laptop plugged in, record its power mode, warm it to steady state, and
avoid concurrent benchmarks. Run CPU and GPU sequentially, including on shared
memory hardware. Retain each harness's existing warmup, medians, and correctness
checks. Repeat complete runs to distinguish improvements from run variability.

Report CPU time divided by GPU time within the same machine and OS, alongside
absolute times and output range counts. Also report unsupported configurations,
cold initialization separately, and the source changes needed for each backend.
GPU API portability, algorithm portability, and performance portability are
separate findings.

## Existing evidence

- [Radeon WSL2 Dozen and Windows controls](gpu-wgpu-strix-halo-wsl2-dozen-2026-09-22.md)
  include direct mapping and a concrete 128 MiB Dozen binding limit. Do not
  assume that limit persists with today's drivers, or install Dozen again
  without first checking the laptop's existing configuration.
- [Radeon map data](../experiments/results/gpu_map_ingest_radeon_8060s.csv)
  provide historical results, not a matched run of the current source.
- [Current desktop offload and wgpu measurements](../experiments/OFFLOAD_AND_WGPU_BENCHMARK.md)
  establish the recent baseline and its limitations.

No new laptop measurements have been collected for this experiment yet.
