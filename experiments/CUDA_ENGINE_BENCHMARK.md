# CUDA (cuda-oxide) engine benchmark

`Engine::CudaOnly` (feature `cuda`) runs the same one-shot ingestion pipelines as `Engine::GpuOnly`,
as CUDA kernels written in Rust with cuda-oxide (`cuda-kernels/`). Every call pays for its own
module load, buffers, and pinned staging; only the CUDA context is kept between calls.

## Measured result

On 100 million shuffled keys, CUDA was about 1.6–3× faster than every CPU thread, for every key
type, and maps were about 1.3–1.8× faster than the parallel CPU builder (`cpu_parallel_ms`), except where nearly
every key is its own range ("gaps"). At 1 million keys the CPU wins, at 10 million results are
mixed and noisy, and on in-order input `from_slice` wins at every size.

| 100M shuffled | `Cpu(16)` | CUDA |
| --- | ---: | ---: |
| set `u32` | 244 ms | 142 ms |
| set `NotNanF32` (duplicates) | 386 ms | 122 ms |
| set `u64` | 392 ms | 254 ms |
| set `NotNanF64` | 448 ms | 251 ms |
| map, 390K ranges | 557 ms (`cpu_parallel`) | 327 ms |
| map, 12.5M ranges | 706 ms (`cpu_parallel`) | 428 ms |

For the portable engine on the same machine (Windows, where wgpu reaches the GPU), `GpuOnly`
took 401 ms for 100M shuffled `u32` against 245 ms for `Cpu(32)`.

- [Set CSV](results/cuda_sets_rtx3060_wsl.csv) and [log](results/cuda_sets_rtx3060_wsl.log)
- [Map CSV](results/cuda_maps_rtx3060_wsl.csv) and [log](results/cuda_maps_rtx3060_wsl.log)

## Hardware and method

AMD Ryzen 9 7950X (16 cores, 32 threads; `Cpu(16)` is its best CPU setting, since the sort is
memory-bound) and an NVIDIA GeForce RTX 3060 12 GB, driver 610.88, CUDA 13.3, under WSL2 with
32 processors. In WSL, wgpu sees only llvmpipe, so the wgpu columns are empty.

```bash
cargo run --release --features cuda --bin gpu_ingest -- --cpu 16 1000000 10000000 100000000
cargo run --release --features cuda --bin gpu_map_ingest -- 1000000 10000000 100000000
```

Where the time goes for 100M `u32` (about 123 ms): upload about 46 ms (bound by the
single-threaded encode into pinned buffers), sort about 56 ms (up to four passes of an 8.4 ms
scatter plus histograms and scans), ranges about 13 ms, and freeing buffers about 6 ms.
