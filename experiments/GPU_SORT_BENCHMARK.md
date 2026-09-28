# CPU versus GPU sorting benchmark

This experiment isolates sorting random `u32` values. It does not run range detection, compaction, range readback, or `RangeSetBlaze` construction.

## Measured result

The CPU was faster through 30,000 values (0.1960 ms versus 0.2443 ms end-to-end). The GPU was faster at the next measured size, 100,000 values (0.2950 ms versus 0.8195 ms). The measured crossover is therefore bracketed between **30,000 and 100,000 values**; 100,000 is the first sampled GPU win. At 10 million values, GPU end-to-end was 8.54 times faster. No production crossover was changed based on this experiment.

- [CSV data](results/gpu_sort_cpu_vs_gpu.csv)
- [SVG chart](results/gpu_sort_cpu_vs_gpu.svg)
- [Raw run log](results/gpu_sort_cpu_vs_gpu.log)

## Hardware and software

The measurement ran as a Windows x86-64 process on Windows 11 Pro 10.0.26200 (build 26200), an AMD Ryzen 9 7950X (16 cores, 32 logical processors), and 67,873,632,256 bytes of RAM. wgpu selected an NVIDIA GeForce RTX 3060 12 GB through the **Vulkan** backend using NVIDIA driver 591.86. The compiler was `rustc 1.97.0 (2d8144b78 2026-07-07)`, targeting `x86_64-pc-windows-msvc`. Dependency versions were wgpu 30.0.1 and Lampshade 0.13.0.

The exact benchmark command, issued from the repository root in WSL so that the Windows toolchain could reach the hardware adapter, was:

```bash
/mnt/c/Users/carlk/.cargo/bin/cargo.exe run --release --features gpu --bin gpu_sort_benchmark > experiments/results/gpu_sort_cpu_vs_gpu.csv 2> experiments/results/gpu_sort_cpu_vs_gpu.log
```

Regenerate the dependency-free SVG chart with:

```bash
python3 experiments/plot_gpu_sort.py \
  experiments/results/gpu_sort_cpu_vs_gpu.csv \
  experiments/results/gpu_sort_cpu_vs_gpu.svg
```

## Method and caveats

The CPU measurement uses Rust's standard slice **`sort_unstable()`**. It is the appropriate standard sort here because equal `u32` values have no relative identity to preserve, and it avoids the allocation required by stable sorting. Copying the reproducible input into a preallocated CPU work slice happens outside the timed region, so the CPU timing starts with input resident in host memory and measures sorting only.

The GPU path uses the same `lampshade::Sorter` and wgpu context feature policy as `src/gpu.rs`. GPU end-to-end timing starts from that same host-resident input and includes `Queue::write_buffer`, the radix sort, copying the complete sorted array to a staging buffer, mapping it, and copying it into a host `Vec<u32>`. Adapter/device creation, shader/pipeline setup, and reusable GPU buffer allocation are warm setup costs and are excluded. The separately reported upload, radix-sort, and readback medians are synchronized wall-clock phases. End-to-end is measured independently as a fused submission, so its median should not be reconstructed by adding the three phase medians. “GPU radix sort only” still includes CPU command recording, queue submission, and waiting for completion; it is not a shader timestamp.

Inputs use a fixed xorshift seed (`0x6a09e667f3bcc909`, mixed with each size). Each case is warmed first and uses 31 repetitions through 10,000 values, 21 through 100,000, 11 through 1 million, 5 through 10 million, and 3 above that. After timing, the full GPU output is compared element-for-element with the CPU-sorted output. The full sweep through 30 million values fit the adapter limits.
