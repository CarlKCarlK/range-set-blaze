# Rust GPU offload backend

The `offload_nightly_experimental` feature adds `Engine::Offload` and
`Engine::OffloadOrCpu` to the shuffled-slice constructors for sets and maps.
It currently targets NVIDIA GPUs on Linux, including WSL with GPU access.
Defaults and the stable/no_std configurations do not enable this feature.

## Setup and use

Use the pinned offload nightly and install its compiler components:

```sh
rustup toolchain install nightly-2026-11-12 --component offload,llvm-tools,llvm-bitcode-linker,rust-src,clippy,rustfmt
```

Install the NVIDIA CUDA toolkit with `ptxas` on `PATH`. The build defaults to
`sm_86` (RTX 3060). For another GPU, set `RANGE_SET_BLAZE_OFFLOAD_ARCH`, for example
`RANGE_SET_BLAZE_OFFLOAD_ARCH=sm_89`. The build compiles kernels for that architecture;
this is not a binary containing kernels for every GPU.

```sh
just test-offload
RANGE_SET_BLAZE_REQUIRE_OFFLOAD_GPU=1 just test-offload
```

The second command requires actual offload GPU execution. Without the variable,
tests exercise CPU fallback and run hardware cases only when a GPU is available.
CI builds the real device kernels and tests fallback on its CPU runners.

```rust,no_run
use range_set_blaze::{Engine, RangeMapBlaze, RangeSetBlaze};

let set = RangeSetBlaze::<u32>::from_shuffled_slice(
    &mut [9, 3, 1, 2, 2], Engine::Offload,
);
assert_eq!(set, RangeSetBlaze::from_iter([1..=3, 9..=9]));

let map = RangeMapBlaze::from_shuffled_slices(
    &[2_u32, 1, 2], &["old", "a", "a"], Engine::Offload,
);
assert_eq!(map, RangeMapBlaze::from_slices(&[1_u32, 2], &["a", "a"]));
```

Set key support matches `Engine::Wgpu`. Map keys use the existing 32-bit encoding;
map values can have any `Eq + Clone` type, including borrowed values. Unsupported
key types or unavailable devices panic for `Offload` and use the CPU for
`OffloadOrCpu`. Allocation or launch failures do not silently fall back. Native
allocation, mapping, or launch failures can terminate the process rather than produce
a catchable Rust panic.

LLVM's `libomptarget.so`, `libomp.so`, and their dependencies must be available at
runtime. Repository executables have a build-time runtime search path. Downstream
applications must also keep LLVM's kernel registration entries when linking:

```sh
RUSTFLAGS="-Clink-arg=-Wl,-z,nostart-stop-gc" cargo +nightly-2026-11-12 build
```

Cargo does not propagate dependency build scripts' linker arguments to the final
application. Without this flag, the linker can discard GPU registration entries.
For a downstream application or a relocated executable, set the loader path to the
matching nightly's runtime directory, or install those libraries in its loader path:

```sh
export LD_LIBRARY_PATH="$(rustc +nightly-2026-11-12 --print sysroot)/lib/rustlib/x86_64-unknown-linux-gnu/lib${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}"
```

## Algorithm and build

The host encodes keys as order-preserving `u64` words. Above 1,024 keys the default
algorithm is a stable 8-bit LSD radix sort, ported from the CUDA-Oxide engine's Onesweep
algorithm. One histogram kernel counts every byte; the host downloads those small
counts once, computes digit bases, and skips constant-byte passes. Each active byte
then takes a workspace-reset launch and a stable-scatter launch. The scatter uses
warp ranking, shared memory, and ticket-ordered decoupled look-back. Sets with 32-bit
key encodings examine four bytes; 64-bit keys and packed map entries examine eight.
The radix path does not pad the input.

The NVIDIA warp operations and shared-memory declaration currently use small private
PTX helpers in `src/gpu/offload/ptx.rs`; the rest of the algorithm is Rust. The port
still stores encoded keys in `u64` words, including 32-bit set keys, and assembles
ranges on the CPU. These differences matter when comparing it with CUDA-Oxide.

Smaller inputs retain bitonic sort. They are padded to the next power of two with
`u64::MAX`. The first kernel sorts 256-key blocks, combining
36 passes with block barriers. Each subsequent global merge uses separate launches
for strides of at least 256, then combines its final eight block-local passes in one
kernel. Cross-block passes still need separate launches for grid-wide synchronization.
For a padded size of 2²⁰ keys this needs 91 launches instead of the original 210.
Padding is removed before decoding and range assembly on the CPU. To compare the
algorithms explicitly, set `RANGE_SET_BLAZE_OFFLOAD_SORT=radix` or `bitonic`; other
values panic. This is an experimental benchmark control.
Inputs are limited to 2³⁰ keys, including padding.

For maps, a word is `key << 32 | input_position`. Sorting these words preserves
input order within duplicate keys, so the existing range assembly correctly keeps
the last value. Values stay on the host.

The key buffer stays on the GPU across all sorting passes: one upload and one final
download. Radix sort additionally transfers a small histogram/prefix table. A private
wrapper holds the host slice's mutable borrow, allocates device
storage, uploads once, and associates the host address with that allocation using
[OpenMP pointer association](https://www.openmp.org/spec-html/5.1/openmpsu176.html).
Its infinite mapping reference count lets `offload!` reuse the allocation without
transfers between kernels. After the last synchronous launch, the wrapper removes
the association, downloads once, and frees the allocation. Values remain on the CPU
for maps, and range assembly still runs on the CPU for both sets and maps.

This prototype is intended for evaluating the compiler/backend. The LLVM runtime
[documents an allocation-cache threshold](https://openmp.llvm.org/design/Runtimes.html#libomptarget-memory-manager-threshold),
configured before process startup with `LIBOMPTARGET_MEMORY_MANAGER_THRESHOLD` in
bytes. A 16 MiB setting was tested, but its speedup did not reproduce consistently;
it is not set by this library. It may retain freed device allocations for reuse.
The ingestion experiments include `offload_ms` and `lib_offload_ms` columns when built
with this feature; start with small inputs.

`build.rs` follows Rust's three-pass offload build: host metadata, GPU code, then
host code embedding the GPU artifact. The core-only kernel source is compiled
independently, so the device never needs wgpu, Rayon, or the rest of this crate.
Generated projects, manifests, and archives live under Cargo's build output directory;
the workspace lockfile and dependencies are unchanged. GPU `core` is rebuilt with
the same target CPU as the kernels to satisfy rustc's ABI checks.

The unsafe operations include the private native interface, OpenMP memory management,
GPU element pointers, barriers, and the PTX helpers. Radix scatter initializes every
output key before readback or the next pass. Ticket order prevents look-back from
waiting for blocks that have not started. Each bitonic compare/exchange pair has
exactly one thread accessing it. All threads reach every block barrier, including
threads beyond a small input's end. Block barriers synchronize block-local passes;
synchronous launches separate cross-block passes. The host cannot access the borrowed
buffer until the final download completes.

See [Rust's offload usage guide](https://rustc-dev-guide.rust-lang.org/offload/usage.html)
and the [GPU offload tracking issue](https://github.com/rust-lang/rust/issues/131513)
for upstream status. The API and build process remain experimental.

## Early bitonic measurements

Measured on November 13, 2026, on an RTX 3060 under WSL2 with a Ryzen 9 7950X.
These are end-to-end construction times in milliseconds, with five timed samples
after warm-up. For shuffled distinct `u32` set keys:

| Input keys | Initial offload | Resident buffer and fused passes | CPU, 16 threads | cuTile |
| ---: | ---: | ---: | ---: | ---: |
| 10,000 | 112.928 | 2.045 | 0.396 | 5.270 |
| 100,000 | 180.914 | 5.263 | 1.017 | 11.506 |
| 1,000,000 | 553.085 | 16.647 | 3.268 | 18.910 |

The million-key case is about 33 times faster than the initial offload prototype.
CPU sorting is still faster on these inputs; the full results also include one-thread
and 32-thread CPU timings. cuTile timings vary across runs, so these measurements do
not establish a universal advantage over that engine.

For one million shuffled map pairs collapsing to one range, the optimized offload
engine took 18.415 ms, cuTile took 18.983 ms, and the 32-thread CPU engine took
13.215 ms. Other map patterns vary with output size; range assembly remains on the CPU.

Data and commands are in the [initial set CSV](../experiments/results/offload_sets_rtx3060_wsl.csv),
[optimized set CSV](../experiments/results/offload_resident_sets_rtx3060_wsl.csv), and
[optimized map CSV](../experiments/results/offload_resident_maps_rtx3060_wsl.csv), with
metadata in their adjacent `.log` files.

The GPU correctness suite passes with hardware execution required. A bitonic runtime trace
of four sorts reusing the same host allocation showed four uploads, four downloads,
and 24 kernel launches; intervening mapping updates were suppressed. The tests also
cover map precedence, floating-point key encodings, and input sizes around block
boundaries.

## Radix and native Windows follow-up

The radix prototype measured about 10 ms for one million shuffled `u32` keys in WSL2,
while the CPU was about 3 ms. An early 3.45 ms allocation-cache result did not reproduce
in subsequent runs and is not a reliable performance claim.

Native Windows wgpu measurements, launched from WSL2, showed 6.864 ms before reusing
sorter pipelines and scratch workspace, and 3.992 ms afterward. The Windows CPU was
about 2.6 ms. The wgpu reuse currently covers the 32-bit set path; a context-identity
cache key prevents resources from crossing between separate wgpu instances.

See [Offload and wgpu benchmarks](../experiments/OFFLOAD_AND_WGPU_BENCHMARK.md) for
commands, raw data, validation, and the limits of comparisons across environments.
