# AMD laptop portability measurements, November 13, 2026

These measurements use the HP ZBook Ultra G1a 14 with a Ryzen AI Max+ PRO 395,
Radeon 8060S, and 128 GB unified LPDDR5X. Each GPU result is compared against the
same laptop's CPU in the same OS. Native Windows and WSL2 runs were managed
over SSH from the desktop and ran sequentially.

Source `8cb9dd213771e3f81bea776fbe592faf417b66bf` merges desktop implementation
`aa4c21a76d1263d3d31008b8109b73bcf4b7c211` into the sister branch. The only
local source edit during measurement was the handoff document. The wgpu path
includes the new sorter/compactor resource cache. These measurements predate
any AMD-specific offload integration.

## Set construction

One million `u32` values, portable release build with Rust 1.99.1 and
`shuffled_slices`. The harness warms each constructor for at least one second,
reports the median of five samples, and compares its result against the CPU.
Times include encoding, transfers, sorting, range assembly, and result
destruction. Input cloning is outside the timer.

| Environment | Workload | Order | CPU1 ms | CPU16 ms | CPU32 ms | wgpu ms |
| --- | --- | --- | ---: | ---: | ---: | ---: |
| Windows Vulkan | distinct | shuffled |11.080|3.811|3.643|6.104|
| Windows Vulkan | distinct | in order |0.736|0.801|0.675|5.134|
| Windows Vulkan | duplicates | shuffled |19.283|4.440|4.765|5.885|
| Windows Vulkan | duplicates | in order |1.769|2.050|2.026|5.231|
| WSL2 Dozen | distinct | shuffled |9.397|3.943|6.172|19.491|
| WSL2 Dozen | distinct | in order |0.280|0.390|0.386|19.776|
| WSL2 Dozen | duplicates | shuffled |8.045|3.604|6.014|19.194|
| WSL2 Dozen | duplicates | in order |0.672|0.856|0.771|19.641|

Distinct input produces 3,907 ranges; duplicate input contains 62,500 distinct
values and produces 245 ranges. `from_slice` on ordered distinct input took
0.099 ms natively and 0.094 ms in WSL2. Sorting is unnecessary for that case.

The native GPU took 1.68 times the best CPU time on shuffled distinct input.
The WSL2 GPU path took 3.19 times the native GPU time in these runs. CPU times
also differ between environments; this is an end-to-end environment comparison,
not an isolated driver or kernel comparison. Results are from one complete
run per environment and do not establish variation across runs or large sizes.

- [Native Windows CSV](results/wgpu_cached_sets_radeon8060s_windows_vulkan_20261113.csv)
  and [metadata](results/wgpu_cached_sets_radeon8060s_windows_vulkan_20261113.log).
- [WSL2 Dozen CSV](results/wgpu_cached_sets_radeon8060s_wsl_dozen_20261113.csv)
  and [metadata](results/wgpu_cached_sets_radeon8060s_wsl_dozen_20261113.log).

## SIMD instruction levels

The separate existing Criterion group uses one million `u32` values with
average clump width 1,000. It has a different distribution from the GPU
construction harness. Do not combine these times into one engine ranking.
The command uses 20 samples, one second of warmup, and two seconds of measurement.
The table reports Criterion's regression slope estimate in microseconds.

| Series | Estimate us | 95 percent confidence interval us |
| --- | ---: | ---: |
| from_iter |801.901|798.606 to 805.462|
| scalar fallback |141.603|140.945 to 142.424|
| SSE2 |94.231|93.606 to 94.841|
| SSE4.2 |92.220|91.695 to 92.869|
| AVX2 |70.756|70.307 to 71.127|
| AVX-512 |79.091|78.365 to 79.953|
| runtime dispatch, AVX-512 |79.294|78.883 to 79.704|

AVX2 was faster than AVX-512 for this workload on this laptop. The scalar
fallback still permits ordinary LLVM optimization; it does not disable all
compiler auto-vectorization.

[CSV](results/simd_clumps1000_u32_radeon8060s_wsl_20261113.csv),
[full estimates](results/simd_clumps1000_u32_radeon8060s_wsl_20261113.estimates.json),
and [log](results/simd_clumps1000_u32_radeon8060s_wsl_20261113.log) preserve the
mean, median, slope, and confidence information.

## Environment and tool changes

Windows 11 Pro build 26200 uses Radeon driver 32.0.22050.4002. The active plan
was HP Optimized (Modern Standby); AC power was online and battery charge 99%
at readiness inspection. Power settings, OS, and GPU drivers were unchanged.

Ubuntu 24.04.3 runs under WSL2 kernel 6.6.87.2-microsoft-standard-WSL2. The
existing Mesa Dozen 25.2.8 installation was preserved. Its adapter reports
`Microsoft Direct3D12 (AMD Radeon(TM) 8060S Graphics)`, integrated GPU,
vendor `0x1002`, device `0x1586`, and driver`DRIVER_ID_MESA_DOZEN`.

Rust 1.99.1 was installed for Windows and WSL. Missing WSL ripgrep 15.2.0 was
installed in the user's Cargo directory; `just` already existed. The initial
native Windows optional CUDA Git dependency fetch stalled. Reusing the WSL
Git database, registry index, and crate archives allowed an offline locked
build, without manifest, lockfile, or dependency-version changes.

The GPU benchmark constructor assertions passed for all eight rows. The
initial broad WSL GPU test run was interrupted during map randomization to
prioritize the requested measurements; it is not recorded as a full-suite pass.
Further regression checks and the required full gate are recorded with their
actual outcomes before the branch is pushed.

## AMD offload launch proof

A separate core-only probe compiled with `nightly-2026-11-12` for
`amdgcn-amd-amdhsa`, `gfx1151`. Host metadata, device ELF, and final host
embedding used the [compiler's three-pass procedure](https://rustc-dev-guide.rust-lang.org/offload/usage.html).
The OpenMP runtime reported one GPU device. With mandatory GPU execution,
one workgroup of 256 threads wrote distinct values into a 256-element buffer;
all host readbacks passed. This is a launch/correctness proof, not a sort timing.

The necessary runtime was extracted into the user-owned prefix
`~/.local/opt/rsb-amd-runtime-7.2.1`; no global package or driver installation
was performed. Official ROCm 7.2.1 packages supplied `hsa-rocr` 1.18.0,
`rocm-core` 7.2.1, `rocminfo` 1.0.0, and `rocprofiler-register` 0.6.0.
ROCDXG 1.2.0 supplied the WSL bridge. AMD package checksums were verified
against repository metadata. ROCDXG SHA256:
`3ed9526719290cd8f590150dad8ea0f234fa779bea6a4c9a8449d7ae6b8cfb6e`.

The probe adds the extracted `opt/rocm/lib` and `opt/rocm-7.2.1/lib`, plus
`/usr/lib/wsl/lib`, to its own `LD_LIBRARY_PATH` and sets
`HSA_ENABLE_DXG_DETECTION=1`. It does not alter the global shell environment.
`rocminfo` confirms the GPU ISA is `gfx1151`. Pinned nightly components are
`offload`, `llvm-tools`, `llvm-bitcode-linker`, `rust-src`, `rustfmt`, and `clippy`.

The first probe omitted the explicit thread dimension and launched 256 groups
of one thread; the host assertion failed. The corrected launch explicitly
sets one group and 256 threads and passes. The standalone probe files and logs
are retained on the laptop in `/tmp/rsb-amd-offload-probe/`, with a copy on the
desktop in `/tmp/rsb-amd-probe-artifacts/`.
