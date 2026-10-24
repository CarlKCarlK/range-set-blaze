# GPU engines benchmark: CPU vs wgpu vs CUDA (preliminary)

Preliminary numbers for building a `RangeSetBlaze` or `RangeMapBlaze` from large shuffled slices
with each `Engine`, on one machine. Every call is one-shot: each engine pays for its own buffers,
pipelines or module, and staging. The only things kept between calls are the process-wide wgpu
context, the CUDA context, and the Rayon thread pools.

- **CPU** (`Engine::Cpu(n)`): Rayon sort on `n` threads, then range finding.
- **wgpu** (`Engine::GpuOnly`): portable compute shaders (Lampshade radix sort) through Vulkan.
- **CUDA** (`Engine::CudaOnly`, feature `cuda`): CUDA kernels written in Rust with cuda-oxide,
  single-source with the host code in `src/gpu/cuda.rs` (`#[cuda_module]`, typed launches with
  launch contracts): a Onesweep radix sort with warp-ranked, coalesced scatter, then range marking,
  scan, and emit. These numbers were measured with the earlier build, which compiled the same
  kernels to a separate PTX file.

## No single environment runs all three

CUDA from Rust runs only on Linux (here, WSL2). wgpu sees the GPU only natively on Windows: in
WSL2 it finds only llvmpipe, a software rasterizer that the engine rejects. (Mesa's Dozen driver
can reach the GPU from WSL2, but it caps storage buffers at 128 MiB, about 33M keys.) So these are
two runs on the same machine, and the CPU columns tie them together:

| Run | Engines | CPU threads |
| --- | --- | --- |
| WSL2 (Ubuntu) | CPU, CUDA | 32 (16 cores with SMT) |
| Windows 11 | CPU, wgpu | 32 |

At 100M shuffled `u32`, `Cpu(16)` took 254 ms in WSL and 296 ms on Windows, and `Cpu(32)` took
245 ms on Windows. The two environments are close, but not identical.

## Sets: 100M shuffled keys

| Type | Workload | CPU best | wgpu (Windows) | CUDA (WSL) |
| --- | --- | ---: | ---: | ---: |
| `u32` | distinct | 245 ms | 398 ms | **146 ms** |
| `i32` | distinct | 277 ms | 458 ms | **133 ms** |
| `NotNanF32` | distinct | 332 ms | 330 ms | **135 ms** |
| `u64` | distinct | 400 ms | 901 ms | **255 ms** |
| `NotNanF64` | distinct | 455 ms | 897 ms | **249 ms** |
| `u32` | duplicates | 271 ms | 443 ms | **130 ms** |
| `NotNanF32` | duplicates | 394 ms | 440 ms | **126 ms** |
| `NotNanF64` | duplicates | 524 ms | 838 ms | **238 ms** |

"CPU best" is the faster of `Cpu(16)` in WSL and `Cpu(32)` on Windows.

- **CUDA** beats the CPU by 1.5–3.1× for every type at 100M, and by about 1.5–2.6× at 10M for
  keys of 32 bits or fewer. At 10M, 64-bit keys are mixed: `u64` and `i64` tie, and `NotNanF64` is
  1.3–1.5× faster. At 1M, CUDA and the CPU are within about 30% of each other either way.
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
| 100M | 1 | 645 / 600 ms | 469 ms | **345 ms** |
| 100M | 390K | 714 / 600 ms | 490 ms | **346 ms** |
| 100M | 12.5M | 616 / 921 ms | 949 ms | **501 ms** |
| 100M | 25M | 1,028 / 1,460 ms | 1,666 ms | 1,420 ms |
| 100M | 100M | 3,447 / 4,713 ms | 5,939 ms | **3,100 ms** |
| 10M | 39K | 58 / 49 ms | **38 ms** | 40 ms |
| 10M | 10M | **279** / 345 ms | 509 ms | 406 ms |

Maps favor the GPUs more than sets do, because the CPU builder sorts key-position pairs. When
nearly every key is its own range, building the map on the CPU dominates for every engine. The
25M-range row varies by run: an earlier run measured 727 ms for CUDA, and the old and current CUDA
builds take the same time (about 1.25 s) on identical data, so this is not a code regression.

## Caveats

- **GPU clocks.** Idle, the RTX 3060 sits in P8 (210 MHz graphics, 405 MHz memory, against a
  2,100 MHz peak). Bursts of work between idle periods keep it there, so the same kernels can run
  up to 10× slower. Each timed operation warms up for at least a second first (every engine
  alike). In real one-shot use with idle gaps, a GPU call can therefore be much slower than these
  numbers.
- **First call.** The first CUDA call in a process costs about 170 ms (context and module setup),
  and the first wgpu call about 450 ms (context creation, measured on Windows).
- **Uploads under WSL.** Getting bytes from host memory to the GPU ran at about 8–12 GB/s however
  it was done (ordinary memory, pinned staging, or pinning the caller's slice). Uploading takes
  about 46 of the 125 ms for 100M `u32` with CUDA.

## Hardware and commands

AMD Ryzen 9 7950X (16 cores, 32 threads), NVIDIA GeForce RTX 3060 12 GB, driver 610.88. WSL2 has
32 processors; CUDA 13.3; Rust 1.99. wgpu 30 with Lampshade 0.13, Vulkan backend on Windows.

```bash
# WSL2: CPU and CUDA (the cuda feature builds with cargo oxide)
cargo oxide run -- --release --features cuda --bin gpu_ingest -- --cpu 16 1000000 10000000 100000000
cargo oxide run -- --release --features cuda --bin gpu_map_ingest -- 1000000 10000000 100000000
# Windows: CPU and wgpu
cargo run --release --features shuffled_slices --bin gpu_ingest -- --cpu 16 --cpu 32 1000000 10000000 100000000
cargo run --release --features shuffled_slices --bin gpu_map_ingest -- 1000000 10000000 100000000
```

- WSL: [sets](results/cuda_sets_rtx3060_wsl.csv) ([log](results/cuda_sets_rtx3060_wsl.log)),
  [maps](results/cuda_maps_rtx3060_wsl.csv) ([log](results/cuda_maps_rtx3060_wsl.log))
- Windows: [sets](results/wgpu_sets_rtx3060_windows.csv)
  ([log](results/wgpu_sets_rtx3060_windows.log)), [maps](results/wgpu_maps_rtx3060_windows.csv)
  ([log](results/wgpu_maps_rtx3060_windows.log))
