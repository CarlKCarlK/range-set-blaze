# CPU and GPU engines for set construction: results, November 2026

This report compares every engine that builds a `RangeSetBlaze` from a shuffled slice,
on two machines, under one set of rules. It is written as material for a RustConf '27
workshop on choosing between SIMD, threads, and GPUs in Rust.

## Machines

| | Desktop | Laptop |
| --- | --- | --- |
| CPU | AMD Ryzen 9 7950X, 16 cores / 32 threads | AMD Ryzen AI Max+ PRO 395, 16 cores / 32 threads |
| GPU | NVIDIA RTX 3060, 12 GB, PCIe | AMD Radeon 8060S (gfx1151), shares system memory |
| Memory | 62 GB visible to WSL2 | 128 GB total; 46 GB visible to WSL2 |
| OS | Windows with WSL2 (Ubuntu) | Windows with WSL2 (Ubuntu) |

## Engines

| Engine | Where it runs | What it is |
| --- | --- | --- |
| `Cpu(n)` | CPU, n threads | Rust's comparison sort (`par_sort_unstable`), then `from_slice` (SIMD) |
| `CpuRadix(n)` | CPU, n threads | Safe parallel radix sort (`src/gpu/cpu_radix.rs`), then `from_slice` |
| `Cuda` | NVIDIA | CUDA-Oxide: Rust kernels compiled by NVIDIA's `cargo oxide` |
| `CuTile` | NVIDIA | NVIDIA cuTile tile kernels |
| `Wgpu` | any (Vulkan, DX12, Metal) | Portable compute shaders through wgpu |
| `Offload` | NVIDIA or AMD RDNA | Rust kernels through Rust's nightly `std::offload` (OpenMP runtime) |
| `Hip` | AMD RDNA | The same Rust kernels as `Offload`, launched directly through AMD's HIP runtime |

`Offload` and `Hip` share one kernel source. The GPU engines all radix-sort on the GPU,
find the ranges there, and download only the ranges.

## Rules

- **Fairness.** Each timed call starts from the input slice and allocates its own
  working memory. Only one-time setup (driver contexts, compiled kernels, thread pools)
  persists between calls. wgpu's cross-call scratch cache was removed for this reason.
- **Workload.** The headline workload is *duplicates, shuffled*: few ranges, values in
  random order. At 100M values that is 6.25 million distinct values, each repeated 16
  times in clumps, forming 24,415 ranges.
- **Measurement.** `experiments/gpu_ingest.rs` checks every engine's result against a
  CPU reference, warms up for at least a second, and reports the median of timed runs
  (5, or 3 with `--quick`). Times are end to end: encoding, transfers, sorting, range
  finding, and freeing the result. Copying the input is outside the timer.
- **Environment.** CPU, CUDA-Oxide, cuTile, offload, and HIP run under WSL2. wgpu runs as
  a native Windows build, because under WSL2 wgpu falls back to a CPU renderer.

## 100M values (duplicates, shuffled)

Each engine ran 3 times (each run the median of 3 samples); ranges show the spread.
Fastest first on each machine.

**Desktop (Ryzen 9 7950X + RTX 3060)**

| Rank | Engine | ms |
| ---: | --- | ---: |
| 1 | `Cuda` (CUDA-Oxide) | 130–134 |
| 2 | `Offload` | 164–168 |
| 3 | `CpuRadix(16)` | 213–225 |
| 3 | `CuTile` | 213–225 |
| 5 | `Cpu(16)` | 252–278 |
| 6 | `Wgpu` (native Windows) | 295–392 |

**Laptop (Ryzen AI Max+ PRO 395 + Radeon 8060S)**

| Rank | Engine | ms |
| ---: | --- | ---: |
| 1 | `CpuRadix(16)` | 205–214 |
| 2 | `Hip` | 222–238 |
| 3 | `Offload` | 273–275 |
| 4 | `Cpu(16)` | 334–365 |
| 5 | `Wgpu` (native Windows) | 571–602 |

- **On the desktop, two GPU engines beat the best CPU sort:** CUDA-Oxide by about 1.6×
  and offload by about 1.3×. cuTile ties the CPU radix sort.
- **On the laptop, the CPU radix sort wins.** The best GPU engine, HIP, is about 10%
  slower. The laptop's CPU shares the same fast memory as its GPU.
- **HIP and offload run identical kernels.** The difference between them on the laptop
  is the runtime layer: direct HIP launches with chunked pinned uploads against the
  OpenMP runtime.
- **Comparing against a fair CPU baseline matters.** Against the comparison sort
  (`Cpu(16)`), every GPU engine except wgpu looks like a clear win on both machines.
- **Windows CPU timings differ from WSL2.** The native Windows wgpu runs also time the
  CPU engines: on the desktop `CpuRadix(16)` took 225–288 ms and `Cpu(16)` 254–310 ms; on
  the laptop 282–304 ms and 443–485 ms. wgpu loses to the same-OS CPU on both machines.

## 2³⁰ values (1.07 billion `u64` keys, duplicates, shuffled)

At this size the GPU sort needs about 16 GB of working memory, more than the RTX 3060
has. Windows' graphics driver model pages the excess to system RAM, so the desktop still
runs it, slowly.

| Rank | Machine | Engine | Seconds |
| ---: | --- | --- | ---: |
| 1 | Laptop | `Cpu(16)` | 4.42–4.68 |
| 2 | Laptop | `Hip` | 5.30–5.54 |
| 3 | Laptop | `CpuRadix(16)` | 5.96 |
| 4 | Desktop | `CpuRadix(16)` | 6.10 |
| 5 | Desktop | `Cpu(16)` | 6.47–6.71 |
| 6 | Desktop | `Offload` | 11.61 |

The laptop's shared memory helps both its CPU and its GPU; its GPU is about 2.2× faster
than the desktop's here. No GPU engine beats the best CPU sort at this size.

## What changed along the way

Each step below was found by measuring, usually with Nsight Systems or Nsight Compute,
rather than by guessing. Times are 100M shuffled distinct `u32` on the RTX 3060.

| Step | Offload time |
| --- | ---: |
| First radix version: GPU sort only; host encodes, decodes, and builds ranges on one thread | 838 ms |
| Parallel encode; find ranges on the GPU; download only the ranges | 424 ms |
| 32-bit keys when the type fits (half the transfer and memory traffic) | 256 ms |
| Plain shared atomics for digit counts; scatter shared memory sized by key width | 203 ms |
| GPU-scope release/acquire in the scatter's look-back | ~165–200 ms (noisy) |

- **Most of the first version's time was not on the GPU.** About 600 ms of 880 ms was
  single-threaded host work around the sort.
- **A "clever" optimization cost 10×.** Grouping equal digits with `match_any` before
  updating shared counters made digit counting compute-bound: 13.8 ms per launch against
  1.35 ms with plain shared atomics.
- **Rust's atomics default to system scope on NVIDIA.** `Ordering::Release`/`Acquire`
  compile to fences that also order against the CPU. GPU-scope PTX (`st.release.gpu`,
  `ld.acquire.gpu`) cut the scatter from 14.6 to about 9.2 ms per pass.
- **Not every idea helped.** Staging offload uploads through pinned memory was slower
  (59 ms against 36 ms for 400 MB). Zero-copy reads on the laptop's shared memory tied
  with chunked pinned uploads.
- **Baselines need the same care.** The CPU radix sort first lost badly at a billion
  keys because the benchmark's `u64` keys all sit near 2³². Its top-digit split put
  nearly every key in a few buckets. Splitting on `key − min` fixed it (16.7 s to 6.1 s).

## Portability

One Rust kernel source runs on both vendors. Everything vendor-specific is in
`src/gpu/offload/device.rs`: block-shared memory, warp matching and shuffles, barriers,
and a shared atomic add. On NVIDIA, five of these are small inline PTX snippets that
`core::arch::nvptx` doesn't yet expose. The kernels assume 32-lane warps, which rules out
AMD's GCN and CDNA GPUs; the build rejects those.

The costs are in the toolchain and runtime: a pinned nightly with extra components,
one GPU architecture per build, vendor runtimes (`ptxas` and CUDA; or ROCm's HSA and HIP
libraries), Linux only, and an extra linker flag in downstream binaries. See
[the offload guide](../docs/offload.md).

## Raw data

All CSV and log files are in [`results/`](results/). The 100M table uses the
`table100m_*` files; the 2³⁰ table uses `big_1g_u64_*`, `cpu_radix_fixed_1g_*`, and
`big_1g_u64_hip_radix_fixed_*`.
