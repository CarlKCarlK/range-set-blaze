# AMD and NVIDIA portability measurements, November 13, 2026

These are fresh runs of the published GPU implementation at
`aa4c21a76d1263d3d31008b8109b73bcf4b7c211`. The AMD branch merged that
implementation as `8cb9dd213771e3f81bea776fbe592faf417b66bf`; its additional
changes at measurement time were documentation only. Each GPU is compared
with its own machine's CPU in the same OS.

The desktop has a Ryzen 9 7950X and RTX 3060. The laptop is an HP ZBook Ultra
G1a with Ryzen AI Max+ PRO 395, Radeon 8060S, and 128 GB unified LPDDR5X.
The laptop was plugged in, with its HP Optimized power plan preserved.

## End-to-end construction

One million `u32` input values, fixed generator seeds. Each row is the median
of five samples after at least one second of warmup per operation. Encoding,
transfers, sorting, range assembly, and result destruction are included; input
cloning is outside the timer. The harness checks each result against the CPU.
CPU and GPU operations run sequentially within each machine.

The CPU column is the fastest of the measured `Engine::Cpu(1)`, `Cpu(16)`,
and `Cpu(32)` choices, not an exhaustive search of thread counts. Ratios are
CPU time divided by GPU time; values above 1 mean the GPU was faster.

| Machine and environment | Workload | Backend | CPU ms (threads) | GPU ms | CPU/GPU |
| --- | --- | --- | ---: | ---: | ---: |
| RTX 3060 desktop, Windows Vulkan | Distinct, shuffled | wgpu | 2.287 (32) | 2.771 | 0.83 |
| RTX 3060 desktop, Windows Vulkan | Duplicates, shuffled | wgpu | 3.009 (16) | 2.525 | 1.19 |
| Radeon 8060S laptop, WSL2 Dozen | Distinct, shuffled | wgpu | 3.943 (16) | 19.491 | 0.20 |
| Radeon 8060S laptop, WSL2 Dozen | Duplicates, shuffled | wgpu | 3.604 (16) | 19.194 | 0.19 |
| RTX 3060 desktop, WSL2 | Distinct, shuffled | offload radix | 3.164 (16) | 11.261 | 0.28 |
| RTX 3060 desktop, WSL2 | Duplicates, shuffled | offload radix | 3.515 (16) | 17.429 | 0.20 |

The native Windows Radeon run is in progress. Its result is needed before
attributing the laptop's WSL2 numbers to the GPU rather than the Dozen
translation path. The WSL2 adapter was verified as hardware, not llvmpipe.
Its Windows driver is `32.0.22050.4002`; Dozen is the existing Mesa 25.2.8 build.

Sorted inputs are retained in the CSV files. Direct `from_slice` is much
faster on those inputs: for distinct keys it took 0.083 ms on Windows desktop,
0.094 ms on WSL2 laptop, and 0.072 ms in the desktop offload run. A sort-based
GPU constructor is unnecessary for that workload.

Native Windows wgpu uses Rust 1.99.1 on both systems; the desktop offload run
uses nightly-2026-11-12. The offload run uses its automatic radix selection
and no allocation-cache override. Comparisons across APIs or OSes therefore
also include compiler and driver differences. A single run does not establish
a stable ranking; older saved desktop timings varied materially.

## SIMD isolation

The existing Criterion group separately tests one million `u32` values with
average clump width 1,000. This distribution differs from the ingestion table.
Runs use 20 samples, one-second warmup, and two-second measurement time per
series. Values below are Criterion slope estimates with 95 percent confidence
intervals, in microseconds.

| Series | Desktop 7950X, WSL2 | 95 percent interval |
| --- | ---: | ---: |
| `from_iter` | 853.625 | 845.593–859.614 |
| Scalar fallback | 179.534 | 178.358–181.094 |
| SSE2 | 111.484 | 108.661–116.920 |
| SSE4.2 | 110.096 | 106.783–113.961 |
| AVX2 | 91.960 | 90.979–92.978 |
| AVX-512 | 95.740 | 95.032–96.687 |
| Automatic dispatch (AVX-512) | 94.751 | 94.528–94.957 |

The scalar series disables explicit SIMD dispatch; LLVM may still
auto-vectorize it. AMD SIMD measurements are pending.

```sh
cargo +1.99.1 bench --locked --bench bench -- '^ingest_clumps_simd_levels_u32/.*/1000$' --warm-up-time 1 --measurement-time 2
```

## Data and implementation status

- [Desktop Windows ingestion CSV](results/wgpu_matched_sets_rtx3060_windows_20261113.csv)
- [Desktop WSL2 offload CSV](results/offload_matched_sets_rtx3060_wsl_20261113.csv)
- [Desktop SIMD CSV](results/simd_clumps1000_u32_7950x_wsl_20261113.csv)
- [Desktop SIMD estimate JSON](results/simd_clumps1000_u32_7950x_wsl_20261113.json)
- [AMD coordination and results](https://github.com/CarlKCarlK/range-set-blaze/pull/39)

Adjacent logs record commands, source revisions, adapters, and toolchains.
These measurements precede the device-operation refactor. That refactor moves
thread coordinates, barriers, and radix operations behind `device.rs`, leaving
the existing NVIDIA implementation unchanged; all 26 NVIDIA hardware tests
passed afterward. It is a foundation for shared algorithms, not a claim that
the AMD offload backend already runs. The AMD compiler/runtime probe precedes
target selection and vendor-operation implementations. The existing wgpu path
already runs on both vendors.
