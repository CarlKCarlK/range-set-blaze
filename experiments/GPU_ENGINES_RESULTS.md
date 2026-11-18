# CPU and GPU engines for set construction: results, November 2026

This report compares the measured CPU and GPU engines that build a `RangeSetBlaze`
from a shuffled slice on two machines. It is written as material for a RustConf '27
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
| `Cpu(n)` | CPU, n threads | Rust's comparison sort (`par_sort_unstable`), then parallel range finding (SIMD) |
| `CpuRadix(n)` | CPU, n threads | Safe parallel radix sort (`src/gpu/cpu_radix.rs`), then parallel range finding |
| `Cuda` | NVIDIA | CUDA-Oxide: Rust kernels compiled by NVIDIA's `cargo oxide` |
| `CuTile` | NVIDIA | NVIDIA cuTile tile kernels |
| `Wgpu` | any (Vulkan, DX12, Metal) | Portable compute shaders through wgpu |
| `Offload` | NVIDIA or AMD RDNA | Rust kernels through Rust's nightly `std::offload` (OpenMP runtime) |
| `Hip` | AMD RDNA | The same Rust kernels as `Offload`, launched directly through AMD's HIP runtime |
| `CubeCl` | NVIDIA or AMD | Rust kernels compiled at run time by CubeCL for CUDA or HIP (added November 15, committed in `db4e68a6`) |

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
  CPU reference, warms up for at least a second of wall time (including input cloning),
  and reports the median of timed samples (5, or 3 with `--quick`). Times include
  encoding, transfers, sorting, range finding, and freeing both the result and the
  cloned input. Copying the input is outside the timer. Cold initialization is excluded
  by the correctness check and warmup before sampling.
- **Environment.** CPU, CUDA-Oxide, cuTile, offload, HIP, and `CubeCl` run under WSL2. wgpu runs as
  a native Windows build; the WSL2 configurations used for these runs selected a CPU
  renderer for wgpu. This is not a general limitation of wgpu under WSL2: earlier
  [AMD and NVIDIA portability measurements](AMD_NVIDIA_PORTABILITY_BENCHMARK.md)
  include hardware-accelerated wgpu through Dozen on the laptop.

## 100M `u32` values: same-session comparisons (November 15–16)

The article and plot use the measurements in this section. Desktop CPU, CUDA-Oxide,
cuTile, and CubeCL ran in one November 15 session. The laptop reran CPU, CubeCL, HIP,
offload, and native Windows wgpu sequentially on November 16, from source `56041ee2`.
The desktop was left available for other work; its offload and wgpu measurements
remain from November 14 and are marked †. Desktop GPU activity from another program
was about 40% during the November 15 session, so these observations do not establish
idle-machine performance.

Each GPU entry spans three run medians, each from three timed samples after warmup.
The WSL2 CPU entries pool six desktop or nine laptop run medians across the engine
builds. CPU range finding is parallel in these sessions.

| Engine | Desktop, WSL2 ms | Laptop, WSL2 ms |
| --- | ---: | ---: |
| `CpuRadix(16)` | 169–182 | 127–136 |
| `Cpu(16)` | 216–251 | 216–242 |
| `Cuda` (CUDA-Oxide) | 130–133 | unavailable |
| `CubeCl` | 138–146 | 230–235 |
| `CuTile` | 215–224 | unavailable |
| `Hip` | unavailable | 201–209 |
| `Offload` | 164–168 † | 194–198 |

**Native Windows controls**

| Machine | `CpuRadix(16)` ms | `Cpu(16)` ms | `Wgpu` ms |
| --- | ---: | ---: | ---: |
| Desktop, November 14 † | 225–288 | 254–310 | 295–392 |
| Laptop, November 16 | 188–194 | 370–378 | 526–545 |

Windows controls come from the same three runs as wgpu on each machine. Laptop wgpu
selected the Radeon 8060S through Vulkan. It loses to the CPU engines in those same
Windows runs; comparisons with WSL2 also include OS, compiler, and driver differences.

- **CPU radix still wins on the laptop.** Using range midpoints, it is about 1.5× as
  fast as the fastest measured GPU engine, offload.
- **The fresh laptop runs change the HIP/offload ranking.** Offload took 194–198 ms,
  versus HIP's 201–209 ms. The November 14 runs below had HIP ahead. These timings
  do not isolate the reason for the change.
- **CubeCL trades some speed for less repository code.** At range midpoints it takes
  about 8% longer than CUDA-Oxide on the desktop and 13% longer than HIP on the laptop,
  from 788 code LOC for both vendors versus about 2,700 LOC for CUDA-Oxide plus HIP.
  Laptop offload is faster still. CubeCL, wgpu, and offload all share kernels across
  the two vendors, with different compiler and runtime paths.
- **The CPU baselines now find ranges in parallel.** The historical November 14
  results below predate that change and should not be mixed into current rankings.
- **Copies dominated the first CubeCL version.** Its built-in transfers took
  250–570 ms for 100M keys. Reused pinned buffers with keys encoded directly into
  them brought the desktop total from about 380 ms to 138–146 ms.

Raw data: [desktop CubeCL](results/table100m_desktop_cubecl_wsl_20261115.csv),
[desktop CUDA-Oxide and cuTile](results/table100m_desktop_cuda_cutile_wsl_20261115.csv),
[laptop WSL2](results/table100m_laptop_wsl_20261116.csv),
[laptop Windows](results/table100m_laptop_windows_20261116.csv).
The [WSL2 run log](results/table100m_laptop_wsl_20261116.log) and
[Windows run log](results/table100m_laptop_windows_20261116.log) record source hashes,
compiler versions, build output, and adapter selection; the Windows log also records
its driver and power scheme. WSL2 adapter lines describe the unused wgpu adapter
(llvmpipe), not the HIP, CubeCL, or offload devices: those runs explicitly selected
the GPU-only engines. The HIP build retained the previously observed nightly
`OnceLock<…>: Sync` recursion warning; all builds and benchmark checks completed. The previous
[laptop CubeCL/HIP session](results/table100m_laptop_cubecl_hip_wsl_20261115.csv)
remains archived, but supplies no headline values here.

CubeCL builds with `--features cubecl_experimental` on stable Rust; set
`RANGE_SET_BLAZE_REQUIRE_CUBECL_GPU=1` to require a GPU. On the laptop it also needs
the HIP runtime paths from the [offload guide](../docs/offload.md).

## Historical 100M `u32` measurements (November 14, before parallel range finding)

These tables retain the earlier revision for comparison. Use the same-session
measurements above for the current article and plot.

Each GPU engine ran 3 times (each run the median of 3 samples). WSL2 CPU ranges pool
6 run medians per machine across two engine builds; Windows CPU ranges use 3.
Ranges show the observed minimum and maximum, not confidence intervals. The following
rankings use WSL2 results only; native Windows results are separate below.

**Desktop (Ryzen 9 7950X + RTX 3060)**

| Rank | Engine | ms |
| ---: | --- | ---: |
| 1 | `Cuda` (CUDA-Oxide) | 130–134 |
| 2 | `Offload` | 164–168 |
| 3 | `CpuRadix(16)` | 213–225 |
| 3 | `CuTile` | 213–225 |
| 5 | `Cpu(16)` | 252–278 |

**Laptop (Ryzen AI Max+ PRO 395 + Radeon 8060S)**

| Rank | Engine | ms |
| ---: | --- | ---: |
| 1 | `CpuRadix(16)` | 205–214 |
| 2 | `Hip` | 222–238 |
| 3 | `Offload` | 273–275 |
| 4 | `Cpu(16)` | 334–365 |

**Native Windows controls**

| Machine | `CpuRadix(16)` ms | `Cpu(16)` ms | `Wgpu` ms |
| --- | ---: | ---: | ---: |
| Desktop | 225–288 | 254–310 | 295–392 |
| Laptop | 282–304 | 443–485 | 571–602 |

Windows CPU and wgpu columns come from the same runs. The CSVs also include
32-thread CPU results; these tables focus on 16 threads and do not establish an
optimal thread count. Cross-OS comparisons also include compiler and driver differences.

- **On the desktop, two GPU engines beat the measured 16-thread CPU baselines:**
  CUDA-Oxide by about 1.6× and offload by about 1.3× against `CpuRadix(16)`.
  cuTile's observed range overlaps the CPU radix sort's.
- **On the laptop, the CPU radix sort wins.** The best GPU engine, HIP, is about 10%
  slower. The laptop's CPU shares the same fast memory as its GPU.
- **HIP and offload share kernel source.** HIP took 222–238 ms against offload's
  273–275 ms on the laptop. Their host paths differ: direct HIP launches with chunked
  pinned uploads versus the OpenMP runtime. These end-to-end runs do not isolate
  launch overhead or establish that the generated GPU code is identical.
- **Comparing against a fair CPU baseline matters.** Against the comparison sort
  (`Cpu(16)`), every GPU engine except wgpu looks like a clear win on both machines.
- **Windows CPU timings differ from WSL2.** wgpu loses to `CpuRadix(16)` in the
  same-OS controls on both machines; desktop wgpu and comparison-sort ranges overlap.

## 2³⁰ values (1.07 billion `u64` keys, duplicates, shuffled)

At this size, two full-size `u64` key buffers alone need 16 GiB, before auxiliary
workspace, more than the RTX 3060's 12 GB. The saved run note attributes the desktop's
slow result to Windows GPU-memory paging, but contains no residency or paging trace.
Treat that explanation as a hypothesis requiring profiling.

These are exploratory WSL2 results. Each populated engine/CSV cell is one run median
of 3 samples; ranges pool run medians from before and after the CPU-radix fix. The
corrected radix timings have only one run median per machine. They are not a repeat
of the 100M protocol, and CUDA-Oxide, cuTile, wgpu, desktop HIP, and laptop offload
have no results in this table.

| Rank | Machine | Engine | Seconds |
| ---: | --- | --- | ---: |
| 1 | Laptop | `Cpu(16)` | 4.42–4.68 |
| 2 | Laptop | `Hip` | 5.30–5.54 |
| 3 | Laptop | `CpuRadix(16)` | 5.96 |
| 4 | Desktop | `CpuRadix(16)` | 6.10 |
| 5 | Desktop | `Cpu(16)` | 6.47–6.66 |
| 6 | Desktop | `Offload` | 11.61 |

Desktop offload took about 2.1–2.2 times as long as laptop HIP here. This comparison
changes the machine, GPU, memory system, and runtime together; it does not isolate a
benefit from shared memory. Neither measured GPU configuration beats its own machine's
fastest measured CPU baseline. No conclusion follows for the unmeasured engines.

## What changed along the way

The development notes below report profiling with Nsight Systems or Nsight Compute.
Times are 100M shuffled distinct `u32` on the RTX 3060, a different distribution from
the headline duplicates workload. These are historical observations across revisions,
not a controlled ablation on one revision. The raw-data manifest below does not
identify profiler captures for these steps, so the stage-level numbers remain
reported observations rather than independently verified results in this review.

| Step | Offload time |
| --- | ---: |
| First radix version: GPU sort only; host encodes, decodes, and builds ranges on one thread | 838 ms |
| Parallel encode; find ranges on the GPU; download only the ranges | 424 ms |
| 32-bit keys when the type fits (half the transfer and memory traffic) | 256 ms |
| Plain shared atomics for digit counts; scatter shared memory sized by key width | 203 ms |
| GPU-scope release/acquire in the scatter's look-back | ~165–200 ms (noisy) |

- **Host work dominated an early profiling observation.** Development notes attribute
  about 600 ms of an approximately 880 ms run to single-threaded host work around the
  sort. That total differs from the table's 838 ms; the saved manifest does not identify
  the captures needed to reconcile them.
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

Offload and HIP share Rust kernel bodies. Device operations are selected in
`src/gpu/offload/device.rs`: block-shared memory, warp matching and shuffles, barriers,
shared atomic addition, and memory ordering. NVIDIA PTX implementations live in
`src/gpu/offload/ptx.rs`; host compilation and runtime integration are also
vendor-specific. The radix kernels assume 32-lane warps, which rules out AMD's GCN
and CDNA GPUs; the build rejects those, and HIP checks warp size at startup.

The costs are in the toolchain and runtime: a pinned nightly with extra components,
one GPU architecture per build, vendor runtimes (`ptxas` and CUDA; or ROCm's HSA and HIP
libraries), Linux only, and an extra linker flag in downstream binaries. See
[the offload guide](../docs/offload.md).

## Raw data

The committed summary CSVs and run notes are in [`results/`](results/):

| Measurement | Files |
| --- | --- |
| 100M laptop rerun, WSL2 | [CSV](results/table100m_laptop_wsl_20261116.csv), [log](results/table100m_laptop_wsl_20261116.log) |
| 100M laptop rerun, Windows | [CSV](results/table100m_laptop_windows_20261116.csv), [log](results/table100m_laptop_windows_20261116.log) |
| 100M desktop, WSL2 | [CSV](results/table100m_desktop_wsl_20261114.csv) |
| 100M laptop, WSL2 | [CSV](results/table100m_laptop_wsl_20261114.csv) |
| 100M desktop, Windows | [CSV](results/table100m_desktop_windows_20261114.csv) |
| 100M laptop, Windows | [CSV](results/table100m_laptop_windows_20261114.csv) |
| 100M source and build notes | [Log](results/table100m_20261114.log) |
| 2³⁰ desktop, before radix fix | [CSV](results/big_1g_u64_offload_rtx3060_wsl_20261114.csv), [log](results/big_1g_u64_offload_rtx3060_wsl_20261114.log) |
| 2³⁰ desktop, corrected radix | [CSV](results/cpu_radix_fixed_1g_u64_7950x_wsl_20261114.csv), [log](results/cpu_radix_fixed_1g_u64_7950x_wsl_20261114.log) |
| 2³⁰ laptop, before radix fix | [CSV](results/big_1g_u64_hip_radeon8060s_wsl_20261114.csv), [log](results/big_1g_u64_hip_radeon8060s_wsl_20261114.log) |
| 2³⁰ laptop, corrected radix | [CSV](results/big_1g_u64_hip_radix_fixed_radeon8060s_wsl_20261114.csv), [log](results/big_1g_u64_hip_radix_fixed_radeon8060s_wsl_20261114.log) |

The 100M notes record source `d874b306`; the local desktop source marker agrees.
Local runner scripts select offload/HIP nightly `nightly-2026-11-12` and native
Windows Rust `1.99.1`. The corrected laptop billion-key note identifies
`eceb53f7`; other billion-key notes do not identify exact source revisions. The
harness uses fixed shuffle seed `0xd1b5_4a32_d192_ed03`, clumps of 256 consecutive
distinct values separated by one missing value, and 16 repetitions per distinct key
for these inputs. Its `u64` generator starts at `0xffff_ff00`, so the billion-key
workload is not uniformly distributed over the full `u64` domain.

The common harness arguments are:

```sh
gpu_ingest --quick --cpu 16 --type u32 100000000
gpu_ingest --quick --cpu 16 --type u64 1073741824
```

Build separately for each engine; see the [offload guide](../docs/offload.md) for
the pinned toolchain, runtime paths, architecture selection, and hardware-required
settings. Windows runs also pass `--cpu 32`. The November 16 laptop rerun uses Rust `1.99.1`
for CubeCL and Windows wgpu, and `nightly-2026-11-12` for HIP and offload. Its WSL2
builds set `RANGE_SET_BLAZE_ROCM_LIB` to the laptop's ROCm 7.2.1 library directory,
`RANGE_SET_BLAZE_OFFLOAD_ARCH=gfx1151`, and `OMP_TARGET_OFFLOAD=MANDATORY`, with
the GPU-required variables set so missing hardware cannot silently use a CPU fallback.
The original runner scripts and extra output are local to `/mnt/c/deldir/rsb-gpu-bench/`, not part of this repository.

For a fully reproducible rerun, archive the exact source and lockfile identity,
per-build commands and compiler versions (including CUDA-Oxide), driver and runtime
versions, adapter/backend selection, power mode, and individual sample durations.
The committed notes do not contain all of that metadata for every run. These
tables describe the saved observations, not a guarantee of the same ranking on
other workloads or configurations.
