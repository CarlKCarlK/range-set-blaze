# GPU engines benchmark: CPU vs wgpu vs CUDA (preliminary)

Preliminary numbers for building a `RangeSetBlaze` or `RangeMapBlaze` from large shuffled slices
with each `Engine`, on one machine. Every call is one-shot: each engine pays for its own buffers
and staging. The only things kept between calls are the process-wide wgpu context, the CUDA
context and its loaded kernels, and the Rayon thread pools.

- **CPU** (`Engine::Cpu(n)`): Rayon sort on `n` threads, then range finding.
- **wgpu** (`Engine::Wgpu`): portable compute shaders (Lampshade radix sort) through Vulkan.
- **CUDA** (`Engine::Cuda`, feature `cuda`): CUDA kernels written in Rust with cuda-oxide,
  single-source with the host code in `src/gpu/cuda.rs` (`#[cuda_module]`, typed launches with
  launch contracts): a Onesweep radix sort with warp-ranked, coalesced scatter, then range marking,
  scan, and emit.

The CUDA numbers come from a run on 2026-10-24 of the current single-source build, with kernels
loaded once per process. The wgpu numbers come from the earlier Windows run; the wgpu engine has not
changed since.

## No single environment runs all three

CUDA from Rust runs only on Linux (here, WSL2). wgpu sees the GPU only natively on Windows: in
WSL2 it finds only llvmpipe, a software rasterizer that the engine rejects. (Mesa's Dozen driver
can reach the GPU from WSL2, but it caps storage buffers at 128 MiB, about 33M keys.) So these are
two runs on the same machine, and the CPU columns tie them together:

| Run | Engines | CPU threads |
| --- | --- | --- |
| WSL2 (Ubuntu) | CPU, CUDA | 32 (16 cores with SMT) |
| Windows 11 | CPU, wgpu | 32 |

At 100M shuffled `u32`, `Cpu(16)` took 269 ms in WSL and 296 ms on Windows, and `Cpu(32)` took
245 ms on Windows. The two environments are close, but not identical.

## Sets: 100M shuffled keys

| Type | Workload | CPU best | wgpu (Windows) | CUDA (WSL) |
| --- | --- | ---: | ---: | ---: |
| `u32` | distinct | 245 ms | 397 ms | **151 ms** |
| `i32` | distinct | 272 ms | 458 ms | **139 ms** |
| `NotNanF32` | distinct | 332 ms | 330 ms | **141 ms** |
| `u64` | distinct | 402 ms | 901 ms | **255 ms** |
| `NotNanF64` | distinct | 462 ms | 897 ms | **257 ms** |
| `u32` | duplicates | 278 ms | 442 ms | **141 ms** |
| `NotNanF32` | duplicates | 416 ms | 440 ms | **136 ms** |
| `NotNanF64` | duplicates | 499 ms | 838 ms | **236 ms** |

"CPU best" is the faster of `Cpu(16)` in WSL and `Cpu(32)` on Windows.

- **CUDA** beats the CPU by 1.4–3.1× for every type at 100M, and by 1.5–2.3× at 10M for keys of
  32 bits or fewer. At 10M, 64-bit keys are mixed: `u64` and `i64` tie (0.9–1.1×), and `NotNanF64`
  is 1.3–1.4× faster. At 1M, CUDA ties or beats the CPU for keys of 32 bits or fewer (1.0–1.7×), and
  the CPU is up to about 1.3× faster for 64-bit `u64`, `i64` and distinct `NotNanF64` keys.
- **wgpu** loses to the CPU for sets on this machine or, in a few cases, ties (such as 100M
  distinct `NotNanF32`). It is about 2× slower for 64-bit keys, which it sorts as two 32-bit passes
  of a key-value sort.
- On **in-order** input, `from_slice` beats every engine at every size (about 20 ms for 100M
  `u32`).

## Maps: shuffled key-value pairs

`cpu_parallel` is the experiment's multithreaded CPU builder (faster than the library's CPU
engine); `lib_cpu` is `Engine::Cpu` with every thread.

| Pairs | Ranges | `cpu_parallel` (WSL / Windows) | wgpu (Windows) | CUDA (WSL) |
| --- | ---: | ---: | ---: | ---: |
| 100M | 1 | 616 / 600 ms | 469 ms | **342 ms** |
| 100M | 390K | 628 / 600 ms | 490 ms | **351 ms** |
| 100M | 12.5M | 606 / 921 ms | 949 ms | **424 ms** |
| 100M | 25M | **995** / 1,460 ms | 1,666 ms | 1,350 ms |
| 100M | 100M | 3,070 / 4,713 ms | 5,939 ms | **2,962 ms** |
| 10M | 39K | 57 / 49 ms | **38 ms** | 40 ms |
| 10M | 10M | 279 / 345 ms | 509 ms | 306 ms |

Maps favor the GPUs more than sets do, because the CPU builder sorts key-position pairs. When
nearly every key is its own range, building the map on the CPU dominates for every engine, and
CUDA roughly ties `cpu_parallel`. The 25M-range row varies by run (one earlier run measured 727 ms
for CUDA), so treat it as noisy.

## cuTile

`Engine::CuTile` (feature `cutile`, `src/gpu/cutile.rs`) runs the same pipeline as cuTile
kernels: NVIDIA's tile-based Rust kernels, where each program works on a tile of 1,024 keys and
the compiler maps tiles onto threads and warps. It handles the same types as `Engine::Wgpu` (sets
with keys up to 64 bits; maps with keys up to 32 bits and values of one or two words). It builds
with plain `cargo`, but cuTile compiles the kernels on first use (about 3 s), so the CUDA toolkit
must be installed where it runs. The sort is a least-significant-digit radix sort with 3-bit
digits (11 passes for 32-bit keys, 22 for 64-bit; passes where every key has the same digit are
skipped), and each pass ranks keys within their digit with 8 tile-wide scans. The code-level
comparison with the CUDA engine is in [CUDA_VS_CUTILE.md](CUDA_VS_CUTILE.md).

### Sets: 100M shuffled keys

One WSL run with all three engines:

| Type | Workload | `Cpu(16)` | CUDA | cuTile |
| --- | --- | ---: | ---: | ---: |
| `u32` | distinct | 269 ms | **154 ms** | 264 ms |
| `NotNanF32` | distinct | 362 ms | **148 ms** | 249 ms |
| `char` | duplicates | 291 ms | **149 ms** | 218 ms |
| `u8` | duplicates | 194 ms | **98 ms** | 166 ms |
| `u64` | distinct | 484 ms | **316 ms** | 473 ms |
| `i64` | distinct | 568 ms | **382 ms** | 607 ms |
| `NotNanF64` | distinct | 579 ms | **342 ms** | 443 ms |

- For keys of 32 bits or fewer, cuTile beats `Cpu(16)` and takes about 1.6× as long as CUDA. For
  64-bit keys (up to 22 passes) it roughly ties the CPU and takes about 1.5× as long as CUDA.
- At 10M and 1M this run was noisy for every engine (CUDA took 39 ms for 10M `u32`, against about
  17 ms in other runs), so compare small sizes only within a run. In this run, cuTile was the
  slowest engine at 1M (9–12 ms) and 10M.
- Where the time goes, for 100M shuffled `u32` with the earlier 4-bit digits (one warm call, about
  250 ms): `scatter_digits` 147 ms (about 21 ms per pass), upload 45 ms, encoding on the CPU about
  28 ms, `digit_counts` 12 ms, the host round trips of the digit counts about 11 ms, and finding
  ranges 5 ms. Moving the keys costs only about 2 ms per pass, so the scatter is bound by its
  scans. The CUDA engine ranks keys with warp instructions, which cuTile's tiles don't expose.
- Digit width trades scans per pass against passes: on 100M `u32`, 3-bit digits took 208–224 ms,
  4-bit 250–266 ms, and 2-bit about 268 ms (alternating runs of one call).
- Ranking with one 1,024 × 16 one-hot tile and a single scan was about 40% slower than 16 scans of
  1,024, so only `digit_counts` uses a one-hot tile (one `reduce_sum`).
- cuTile encodes keys on every CPU thread (as the map path does); the CUDA engine encodes on one
  thread, overlapped with its upload.

### Maps: shuffled key-value pairs

| Pairs | Ranges | `cpu_parallel` | CUDA | cuTile |
| --- | ---: | ---: | ---: | ---: |
| 100M | 1 | 791 ms | **352 ms** | 464 ms |
| 100M | 390K | 630 ms | **356 ms** | 492 ms |
| 100M | 12.5M | 689 ms | **426 ms** | 548 ms |
| 100M | 25M | **1,081 ms** | 1,510 ms | 1,517 ms |
| 100M | 100M | **3,143 ms** | 4,782 ms | 4,851 ms |
| 10M | 39K | 52 ms | **44 ms** | 52 ms |
| 10M | 10M | **310 ms** | 391 ms | 421 ms |

- With few ranges, cuTile beats the CPU builder at 100M and ties it at 10M; CUDA is fastest.
  When most keys are their own range, building the map on the CPU dominates every engine.
- Two host-side fixes cut a warm 100M one-range call from 816 to 415 ms: positions come from
  `arange` on the GPU (about 159 ms on the host before), and the key and value words are copied
  for upload on every thread (about 300 ms on one thread before; cuTile uploads only from owned
  vectors).

### Compiler bugs worked around

Two cuTile compiler bugs, confirmed on the GPU at cuda-rust `ad0a42c` and at `main` as of
2026-10-25 (`9df9e7c`). Both are in `encode_literal_bytes`
(`cutile-compiler/src/compiler/compile_expression.rs`). `constant(false, ..)` compiles as true:
the bool literal arrives as the string `"false"`, and the encoder treats anything but `"0"` as
true. Any unsigned constant above the signed maximum (for `u32`, above `i32::MAX`) compiles as 0:
integers are signless, so it is parsed as signed, and a failed parse silently becomes 0. Both are
reported in [cuda-rust#1469](https://github.com/NVIDIA/cuda-rust/issues/1469), with a fix in
[cuda-rust#1470](https://github.com/NVIDIA/cuda-rust/pull/1470) (open as of 2026-10-25); once it
merges, the workarounds can go.

```bash
cargo oxide run --features cuda,cutile --bin gpu_ingest -- --cpu 16 1000000 10000000 100000000
cargo oxide run --features cuda,cutile --bin gpu_map_ingest -- 1000000 10000000 100000000
```

[Sets](results/cutile_sets_rtx3060_wsl.csv) ([log](results/cutile_sets_rtx3060_wsl.log)),
[maps](results/cutile_maps_rtx3060_wsl.csv) ([log](results/cutile_maps_rtx3060_wsl.log)).

## Caveats

- **What persists between calls.** Each call allocates and frees its own buffers. Kept
  process-wide: the wgpu device, the CUDA context and loaded CUDA kernels (compiled code, loaded
  once as CUDA programs normally do), and the Rayon thread pools. The wgpu engine still creates its
  compute pipelines on every call, because Lampshade's sorters own their pipelines and their
  buffers together; that costs it about a millisecond per call, which matters mainly at 1M keys.

- **GPU clocks.** Idle, the RTX 3060 sits in P8 (210 MHz graphics, 405 MHz memory, against a
  2,100 MHz peak). Bursts of work between idle periods keep it there, so the same kernels can run
  up to 10× slower. Each timed operation warms up for at least a second first (every engine
  alike). In real one-shot use with idle gaps, a GPU call can therefore be much slower than these
  numbers.
- **First call.** The first CUDA call in a process costs about 170 ms (context and module setup,
  measured with an earlier build), and the first wgpu call about 450 ms (context creation,
  measured on Windows).
- **Uploads under WSL.** Getting bytes from host memory to the GPU ran at about 8–12 GB/s however
  it was done (ordinary memory, pinned staging, or pinning the caller's slice). In an earlier
  probe, uploading took about 46 of the 125 ms for 100M `u32` with CUDA.

## Hardware and commands

AMD Ryzen 9 7950X (16 cores, 32 threads), NVIDIA GeForce RTX 3060 12 GB, driver 610.88. WSL2 has
32 processors; CUDA 13.3; Rust 1.99. wgpu 30 with Lampshade 0.13, Vulkan backend on Windows.

```bash
# WSL2: CPU and CUDA (the cuda feature builds with cargo oxide, which builds release by default
# and prints a build banner on stdout before the CSV)
cargo oxide run --features cuda --bin gpu_ingest -- --cpu 16 1000000 10000000 100000000
cargo oxide run --features cuda --bin gpu_map_ingest -- 1000000 10000000 100000000
# Windows: CPU and wgpu
cargo run --release --features shuffled_slices --bin gpu_ingest -- --cpu 16 --cpu 32 1000000 10000000 100000000
cargo run --release --features shuffled_slices --bin gpu_map_ingest -- 1000000 10000000 100000000
```

- WSL: [sets](results/cuda_sets_rtx3060_wsl.csv) ([log](results/cuda_sets_rtx3060_wsl.log)),
  [maps](results/cuda_maps_rtx3060_wsl.csv) ([log](results/cuda_maps_rtx3060_wsl.log))
- Windows: [sets](results/wgpu_sets_rtx3060_windows.csv)
  ([log](results/wgpu_sets_rtx3060_windows.log)), [maps](results/wgpu_maps_rtx3060_windows.csv)
  ([log](results/wgpu_maps_rtx3060_windows.log))
