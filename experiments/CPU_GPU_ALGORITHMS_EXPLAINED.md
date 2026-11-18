# From one CPU thread to GPU kernels: how the set constructors work

This walkthrough explains the implementations behind `RangeSetBlaze::from_shuffled_slice`,
from the simplest integration to the most machinery. The examples focus on sets of `u32`
values. CPU comparison and CPU radix each have separate one-thread and 16-thread sections;
the thread counts are configurations of an engine, not entirely separate implementations.

The source snapshot is `db4e68a6`, including the parallel range finding that both CPU
engines use after their sort (section 2) and the `CubeCL` engine (section 5).
Excerpt line numbers refer to that snapshot. The order follows the amount of repository
code and the concepts a reader must learn, not performance rank. A short library call can hide a
complicated algorithm in a dependency.

All implementations solve the same problem. For example:

```text
Input:          [8, 3, 2, 8, 5, 4]
Sorted values:  [2, 3, 4, 5, 8, 8]
Output ranges:  [2..=5, 8..=8]
```

Duplicates disappear. Consecutive distinct values join a range. A gap starts another
range. Every engine pays for some combination of sorting, finding these boundaries, and
building the final set. The GPU engines also pay for host/device coordination and transfers.

**How to read the timings.** Unless stated otherwise, times below are for **100 million
shuffled duplicate `u32` values**, producing 24,415 ranges. Desktop means Ryzen 9 7950X
with RTX 3060; laptop means Ryzen AI Max+ PRO 395 with Radeon 8060S. CPU, CUDA-Oxide,
cuTile, offload, HIP, and `CubeCL` measurements used WSL2; wgpu used native Windows. The older
one-thread numbers might be out of date. The billion-key results are a different workload:
2³⁰ shuffled duplicate `u64` values, with limited runs across revisions. Missing results
are not estimates. See [CPU and GPU engine results](GPU_ENGINES_RESULTS.md) for the
measurement protocol, same-OS controls, and raw-data links.

The headline multithreaded CPU and GPU times come from the same `gpu_ingest` sessions:
November 15 on the desktop and November 16 on the laptop. Each engine has three run
medians; WSL2 CPU ranges pool six desktop or nine laptop run medians across engine builds.
Desktop offload and wgpu retain November 14 measurements, explicitly marked below.
Native Windows wgpu has its own same-OS CPU controls in the results report. The separate
`parallel_ranges::benchmark` comparisons below explain the range-finding optimization;
they are not the headline timings.

**How to read the omissions.** The Rust blocks are short, verbatim source excerpts with
surrounding context removed; they are not standalone programs. “Shown” counts their
nonblank, non-comment source lines. “Omitted” is the rest of the measured repository
backend footprint, including shared backend/build code where applicable. It does **not**
include the dependency, compiler, runtime, test, or common-library code excluded from
that footprint. Instead, each engine's direct dependencies are counted at the end.

## 1. CPU comparison sort, one thread

**Speed:** desktop **1,249 ms**; laptop **1,709 ms**. These are older measurements and
might be out of date. There is no one-thread result in the cited billion-key table.

The application-level algorithm is two operations: sort the mutable slice, then construct
ranges from the sorted values. Rust's standard-library `sort_unstable` uses comparisons
to establish order. “Unstable” means equal values need not retain their original positions;
that is harmless for a set because equal values become one member.

After sorting, `from_sorted_slice` finds the ranges with the library's SIMD range finder,
the same one `RangeSetBlaze::from_slice` uses. “One thread” therefore does not mean
“one scalar comparison at a time.” With one thread the whole slice is a single chunk, so
none of section 2's parallel machinery starts. This path does not transfer data to a GPU.

The two essential operations are shown in execution order, although they come from
different places in [the CPU adapter](../src/gpu.rs):

<!-- excerpt: src/gpu.rs:507-507; src/gpu.rs:502-502 -->

```rust
values.sort_unstable();
T::from_sorted_slice(values, threads)
```

**Shown: 2 code LOC. Omitted from the 154-LOC CPU-family footprint: 152 code LOC.**

That omitted family code contains the helper functions, the thread-count branch, the
15-LOC cached thread-pool helper, and the 128-LOC parallel range finder described in
section 2. The one-thread execution itself does not enter the pool. The standard-library
sorting implementation and the shared SIMD range finder are outside these counts and are
not printed. The two-line excerpt is the integration, not a two-line implementation of
sorting.

## 2. CPU comparison sort, 16 threads

**Speed, with parallel range finding:** desktop **216–251 ms**; laptop **216–242 ms**.
In a separate in-crate optimization benchmark, parallel range finding took 219 ms and
263 ms overall, versus 268 ms and 318 ms with one-thread range finding. Those paired
measurements describe the optimization, not the headline sessions. At 2³⁰ `u64` values, measured
before parallel range finding: desktop **6.47–6.66 seconds**; laptop **4.42–4.68 seconds**.

The overall pipeline is unchanged: sort, then build ranges. The sorting work runs
inside a cached Rayon pool configured for 16 workers. Rayon partitions the comparison
sorting problem into smaller problems, processes sufficiently large pieces in parallel,
and handles small pieces sequentially. Its implementation also has specialized paths
and fallbacks; this document does not reproduce them.

This is more than launching 16 independent sorts: the pieces must form one globally
sorted slice. Rayon supplies that coordination.

The essential operations from [the CPU adapter](../src/gpu.rs) are:

<!-- excerpt: src/gpu.rs:509-509; src/gpu.rs:502-502 -->

```rust
thread_pool(threads).install(|| values.par_sort_unstable());
T::from_sorted_slice(values, threads)
```

The omitted repository code selects and caches the pool, chooses the one-thread or
multithreaded path, and wraps the operations. Rayon's actual parallel sorting algorithm
and work scheduler are dependency code, not included in the footprint. Most of the
speedup comes from calling that existing implementation, rather than writing new GPU code.

### Finding the ranges in parallel

Once the slice is globally sorted, finding its ranges in parallel needs no general
merge. Split the sorted slice into contiguous chunks. Every value in one chunk is at most
every value in the next, so each worker can run the SIMD range finder on its own chunk,
and the ranges it finds are already sorted and disjoint. Only the **seams** can need
repair. The last range of one chunk and the first range of the next can share a value,
because duplicates straddle the cut, or they can be adjacent (`41` ends one chunk, `42`
starts the next).

```text
Sorted:   [2, 3, 4, 4 | 4, 5, 8, 9 | 10, 20]
Chunks:   [2..=4]       [4..=5, 8..=9]  [10..=10, 20..=20]
Seams:          4 = 4: merge      9, 10 adjacent: merge
Output:   [2..=5, 8..=10, 20..=20]
```

The workers run in [the parallel range finder](../src/gpu/parallel_ranges.rs):

<!-- excerpt: src/gpu/parallel_ranges.rs:77-80 -->

```rust
sorted
    .par_chunks(chunk_len)
    .map(|chunk| sorted_slice_ranges(level, chunk))
    .collect()
```

A lazy iterator then walks the per-chunk vectors in order, holding back one pending
range. An interior range is emitted unchanged. Only the first range of a chunk is
compared with the pending range, and when they share a value or touch, it extends the
pending range instead. The pending range can keep growing across several seams, for
example when a whole chunk is one range:

<!-- excerpt: src/gpu/parallel_ranges.rs:150-150; src/gpu/parallel_ranges.rs:154-156 -->

```rust
if at_seam && continues(*pending, range) {
    pending.1 = range.1;
    continue;
}
```

The iterator's output feeds the set's tree directly, with no concatenated vector and no
second sort. The crate marks iterators known to yield sorted, disjoint ranges with the
`SortedDisjoint` trait, and the seam iterator implements it itself:

<!-- excerpt: src/gpu/parallel_ranges.rs:180-180 -->

```rust
impl<T: Integer> SortedDisjoint<T> for SeamRanges<T> {}
```

That one line matters for speed. Wrapping the iterator in the crate's runtime checker
(`CheckSortedDisjoint`) instead raised this step from about 112 ms to 158 ms on 10 million
ranges. The guarantee holds by construction: each chunk's ranges come out sorted, and
the seams are asserted in order even in release builds.

Two details keep the parallel step from costing more than it saves. Chunks hold at
least 2¹⁸ values, so short slices stay on one thread. Building the set's tree from the
ranges still runs on one thread. With few ranges that's negligible (0.3 ms for 24,415
ranges), but for sparse data, where nearly every value is its own range, it dominates:
about 1.7–3.6 seconds for 100 million ranges.

Parallel range finding helps most when the one-thread range finder is slow. For the
headline workload, runs of 16 equal values defeat the SIMD fast path. One thread took
65 ms to find the ranges in 100 million sorted values; 16 threads took about 10 ms. For
distinct values in clumps of 256, one thread already took only 14 ms, so there was
little to gain.

**Shown: 2 + 9 code LOC. Omitted from the same 154-LOC CPU-family footprint: 143 code LOC.**

The omitted code includes the chunk-count and chunk-length choice, the seam iterator's
other branches, the seam assertion, and the dispatch that sends key types without the
SIMD range finder (the nightly `f16`/`f128` floats) to the one-thread `from_slice`.

## 3. CPU radix sort, one thread

**Speed:** desktop **673 ms**; laptop **643 ms**, measured before parallel range finding.
There is no one-thread result in the cited billion-key table.

Comparison sorting asks which of two values comes first. Radix sorting instead examines
groups of bits, called digits, and distributes values into buckets for those digits.
A counting pass records bucket sizes; an exclusive prefix sum turns sizes into starting
positions; a scatter writes each value into its next available bucket position. Each pass
preserves order among values with the same digit, which lets passes over successively
more significant digits build a correctly sorted result.

This engine runs the repository's own radix sorter, [`cpu_radix.rs`](../src/gpu/cpu_radix.rs),
inside the cached Rayon pool; a one-thread pool runs the same code with one worker. It
sorts each value by its order-preserving GPU key (`T::encode`), the same key the GPU
engines sort. Then the ranges are found as in sections 1 and 2.

The essential operations from [the CPU adapter](../src/gpu.rs) are:

<!-- excerpt: src/gpu.rs:291-292 -->

```rust
thread_pool(threads).install(|| cpu_radix::sort_by_key(values, T::encode));
T::from_sorted_slice(values, threads)
```

**Shown: 2 code LOC. Omitted from the 306-LOC family footprint: 304 code LOC.**

The omissions are the 163-LOC radix sorter, the shared 15-LOC thread-pool helper, and the
128-LOC parallel range finder from section 2.

## 4. CPU radix sort, 16 threads

**Speed, with parallel range finding:** desktop **169–182 ms**; laptop **127–136 ms**.
In a separate in-crate optimization benchmark, parallel range finding took 144 ms and
119 ms overall, versus 195 ms and 177 ms with one-thread range finding. At 2³⁰
`u64` values, measured before parallel range finding: desktop **6.10 seconds**; laptop
**5.96 seconds**.

With more workers, the sorter works in two steps:

1. It subtracts the smallest key from every key, so digits cover only the occupied range,
   and splits the keys by their highest 8 to 16 significant bits. The split is sized so
   that each bucket holds about 2¹⁸ keys, small enough to stay in cache. Each worker
   scatters its own slice of the input, and the buckets are disjoint, so no locking is
   needed.
2. Each bucket is sorted independently, in parallel, by its remaining bits, using digits
   up to 11 bits wide. One read counts every pass's digits, and passes whose digit is the
   same for every key are skipped.

The repository code is the same as in section 3; only the pool size changes. After the
sort, the ranges are found in parallel exactly as in section 2.

**Shown: the same 2 code LOC as section 3, from the same 306-LOC family footprint.**

The 163-LOC sorter, 15-LOC pool helper, and 128-LOC range finder are shared by the
one-thread and 16-thread versions; do not add their footprints together as if they were
two independent codebases.

### Why not use a radix-sort crate?

A crate would remove the 163 lines from the footprint, so two crates were tried in place of
the custom sorter: `rdst` and `voracious_radix_sort`. On the laptop, sorting 100 million
shuffled keys with 16 threads took (median of 5, milliseconds; sort only):

| Keys | Custom | `rdst` | `voracious_radix_sort` | Rayon comparison sort |
| --- | ---: | ---: | ---: | ---: |
| Narrow duplicates, `u32` | **109** | 116 | 111 | 262 |
| Narrow duplicates, `u64` | **181** | 257 | 192 | 287 |
| Narrow distinct, `u32` | **113** | 116 | 114 | 316 |
| Narrow distinct, `u64` | **186** | 268 | 308 | 330 |
| Full-width random, `u32` | 123 | **92** | 140 | 303 |
| Full-width random, `u64` | 305 | **180** | 206 | 318 |

“Narrow” keys are this walkthrough's workloads: a 100-million-value set fits in about 27
bits. Range sets are usually built from data like this. A range set pays off when values
come in runs, and values that come in runs span a small part of their type. In effect the
values are `u23` or `u36` numbers that Rust must store in a `u32` or `u64`.

Radix-sort cost depends on how many bits the sort processes. The custom sorter processes
only the occupied bits. `rdst` starts at the type's top byte and reads the whole input to
discover that the byte is the same for every key before moving to the next byte. For
27-bit keys in a `u64`, that is four wasted reads of 800 MB. Its first useful byte then
splits the keys into only a handful of large buckets. On the desktop, `rdst` made the
workload about 40% slower in that separate sort-comparison experiment (205 ms versus
144 ms). On full-width random keys, where
every bit matters, `rdst` is the fastest, so the custom sorter's advantage comes
specifically from narrow keys.

## 5. CubeCL: one set of Rust kernels for NVIDIA and AMD

**Speed:** desktop **138–146 ms**; laptop **230–235 ms**. In the same sessions, CUDA-Oxide
took 130–133 ms on the desktop; HIP took 201–209 ms and offload 194–198 ms on the laptop.
There is no billion-key result yet.

[CubeCL](https://github.com/tracel-ai/cubecl) compiles kernels written as ordinary Rust
functions (marked `#[cube]`) for whichever GPU is present at run time: here NVIDIA through
CUDA, or AMD through HIP. Like wgpu and offload, it runs on both GPUs from one set of
kernels. The vendor-specific alternative takes CUDA-Oxide plus HIP.

The sort is a least-significant-digit radix sort with four-bit digits, taken from
`key - min`. Like the CPU radix sorter, it runs only as many passes as `max - min` has
bits: six passes, not eight, for this workload's 23-bit spread. Each pass has three
steps. Each tile of 4,096 keys counts its keys of each digit. A GPU scan turns the counts
into each tile's first output position for each digit. Then each tile scatters its keys.
To keep equal digits in order, each of a tile's 256 units owns 16 consecutive keys, and a
cube-wide scan of the units' digit counts gives each unit its first slot for each digit.
No cube waits for another cube, so unlike the Onesweep engines (CUDA-Oxide, HIP), this
sort doesn't depend on the GPU running every cube at once.

The heart of the scatter, from [the CubeCL engine](../src/gpu/cubecl.rs), is plain Rust
that runs on the GPU:

<!-- excerpt: src/gpu/cubecl.rs:191-197 -->

```rust
let key = keys[index as usize];
let digit = digit_of::<K>(key, min, shift);
let slot = (digit * UNITS + UNIT_POS) as usize;
let position = offsets[(digit * tiles + tile) as usize] + columns[slot]
    - digit_starts[digit as usize];
columns[slot] += 1u32;
out[position as usize] = key;
```

`columns[slot]` starts as the scanned count of keys before this unit's keys with this
digit, so `position` is the key's place in the output. `UNIT_POS`, shared memory, and the
cube-wide scan (built from `CubeCL`'s `plane_inclusive_sum`) are the only GPU-specific
ideas. Range finding then counts range starts per tile, scans the counts, and emits each
range's first and last key, so only the ranges come back.

The surprise was the copies, not the kernels. `CubeCL`'s own transfers moved only about
1.5 GB/s under WSL2 and sometimes pinned fresh memory, which costs about 0.6 ms per MB.
At first, copying 100 million keys took 250–570 ms, while the sort took about 60 ms. The
engine therefore bypasses `CubeCL` for copies. It asks `CubeCL` for each buffer's device
address, then has the CUDA or HIP driver copy through two reused pinned buffers. Each chunk
is encoded straight into one pinned buffer, in parallel, while the other one copies, and
the minimum and maximum are tracked along the way. That is what the CUDA-Oxide engine
does, and it took the desktop from about 380 ms to 138–146 ms. The copies are the
engine's only `unsafe` code.

**Shown: 7 code LOC. Omitted from the 788-LOC footprint: 781 code LOC.**

The footprint is 253 kernel code LOC, 174 host code LOC (device setup, sort passes,
scan, and range finding), 332 code LOC of pinned CUDA/HIP copies, and 29 code LOC of
dispatch in `src/gpu.rs`. The copies are 42% of the engine: the price of working around
the framework's transfers. `CubeCL`'s compiler (with a bundled LLVM), runtimes, and
vendor drivers are dependencies outside the count. `CubeCL` is alpha, with breaking
changes between minor versions, so it is pinned exactly, and it needs Rust 1.95 or newer.

## 6. cuTile: express the GPU work as operations on tiles

**Speed:** desktop **215–224 ms**. This NVIDIA-only engine is unavailable on the
Radeon laptop. No cuTile result is present in the cited billion-key table.

cuTile works on a tile of 1,024 keys at once. For each three-bit digit, a tile counts
how many keys belong to each of eight digit values. The host reads the tile counts and
computes the starting position of every tile/digit combination. A second kernel ranks
each key within its digit and scatters it into the corresponding output position.

Inside a tile, ranking is expressed as a scan over a vector of zero/one flags. For
flags `[1, 0, 1, 1]`, the inclusive prefix sum is `[1, 1, 2, 3]`; subtracting the flags
gives `[0, 1, 1, 2]`, the number of matching predecessors at each position.

These lines from [the cuTile kernels](../src/gpu/cutile.rs) express that computation:

<!-- excerpt: src/gpu/cutile.rs:162-165 -->

```rust
let matches: Tile<bool, { [1024] }> = eq_tile(digit, wanted);
let flags: Tile<i32, { [1024] }> = select(matches, one, zero);
let through: Tile<i32, { [1024] }> = scan_sum(flags, 0i32, reverse::Forward, 0i32);
let before: Tile<i32, { [1024] }> = through - flags;
```

Adding the tile/digit's global base gives the destination for each matching key. The
compiler decides how threads implement these tile operations; the source does not
manually implement warp shuffles for this scan. Passes alternate between buffers,
preserve equal-digit order, and skip digits that are constant across the input.

Range finding uses two more kernels: count range starts per tile, prefix-sum those
counts on the host, then emit the range endpoints on the GPU. Only the endpoints
are downloaded for this path, rather than the entire sorted input.

**Shown: 4 code LOC. Omitted from the 850-LOC footprint: 846 code LOC.**

The footprint contains 281 host/support code LOC and 569 kernel/module code LOC.
The excerpt omits tile loads, digit extraction, base loading and broadcasting, masked
stores, tail handling, the host scans and launches, buffer management, range emission,
64-bit support, and map-specific kernels. The latter capabilities enlarge the full
backend footprint even though this example is a `u32` set. cuTile's tensor operations,
compiler, runtime, and toolkit setup are dependencies outside these 850 LOC.

## 7. wgpu: assemble a portable compute pipeline

**Speed:** desktop **295–392 ms** (November 14, not rerun); laptop **526–545 ms**
(November 16), both on **native Windows**. Laptop same-run Windows CPU controls were
188–194 ms for radix sort and 370–378 ms for comparison sort, both with 16 threads.
No wgpu result is present in the cited billion-key table. These times should be compared
with Windows CPU controls, not treated as a pure API comparison with the WSL2 engines.

The host uploads encoded keys and asks Lampshade to radix-sort them. A small shader then
marks each sorted value that starts or ends a range. Lampshade's compactor packs the
marked endpoints into dense output arrays. The host downloads those endpoints and
constructs the final set.

Boundary detection is local: a value starts a range if it is the first value, or differs
from its predecessor and is not consecutive with it. The end test is symmetric. Duplicate
values must not create extra boundaries, and the first/last elements need special handling.

The central command sequence in [the wgpu adapter](../src/gpu.rs) is:

<!-- excerpt: src/gpu.rs:710-717 -->

```rust
let mut radix_sorter = Sorter::from_context(context);
let mut compactor = Compactor::from_context(context);
let mut encoder = encoder(context, "GPU range normalization");
resource_or_panic(radix_sorter.record_sort(&mut encoder, &input, &sorted, len))?;
boundary.record(&mut encoder);
record_compactions_32(&mut compactor, &mut encoder, &sorted, &buffers, len)?;
submit(context, encoder.finish())?;
read_endpoints_32(context, &buffers, len)
```

The compact host sequence hides substantial algorithmic work in Lampshade, including
portable sorting and adapter-specific optimized paths. The repository code still has to
describe buffers and shader bindings, honor device limits, submit commands, and wait for
mapped readback correctly. Portability moves some complexity into the API and dependency.

**Shown: 8 code LOC. Omitted from the 891-LOC footprint: 883 code LOC.**

The footprint contains 790 host/support code LOC and 101 embedded shader code LOC.
The omissions include input encoding/upload, adapter selection, limits and error scopes,
binding layouts, the boundary shaders, compaction bindings, asynchronous readback,
mapped-memory choices, and the wider-key path. Lampshade's sorting, compaction, and scan
implementations are outside these 891 LOC; a call to `record_sort` is not the sort itself.

## 8. CUDA-Oxide: implement the GPU radix algorithm explicitly

**Speed:** desktop **130–133 ms**. This NVIDIA-only engine is unavailable on the
Radeon laptop. No CUDA-Oxide result is present in the cited billion-key table.

This engine implements an eight-bit-digit radix sort with 256 threads per block and
4,096 keys per tile. It counts all digit positions, computes digit bases, and performs
stable scatter passes. Within each warp, matching-digit lane masks provide ranks;
shared-memory arrays combine warp counts and arrange writes into mostly adjacent addresses.

A hard part is finding the position of each tile's digit group without running a
separate global prefix-sum kernel for every scatter pass. The kernel uses **decoupled
look-back**: each tile publishes its count, then reads predecessor counts until it finds
one already containing a cumulative prefix. A ticket assigns tile indices as blocks
start, so a tile waits only for predecessor tiles that have already started.

This excerpt from [the CUDA-Oxide scatter kernel](../src/gpu/cuda.rs) shows the
publication and predecessor walk, inside an already-established nonzero-tile branch:

<!-- excerpt: src/gpu/cuda.rs:261-274 -->

```rust
slot(tile).store(AGGREGATE | u64::from(total), AtomicOrdering::Release);
let mut prefix = 0;
let mut previous = tile;
loop {
    previous -= 1;
    let mut word = slot(previous).load(AtomicOrdering::Acquire);
    while word & READY == 0 {
        word = slot(previous).load(AtomicOrdering::Acquire);
    }
    prefix += word as u32;
    if word & INCLUSIVE != 0 {
        break;
    }
}
```

The status word carries both a count and flags describing whether it is a tile count
or a cumulative count. Release/acquire ordering makes publication observable in the
required order. Correctness also depends on the tile-ticket rule, initialized status
storage, full-warp participation, and barriers around shared-memory reuse. Those are
part of the algorithm, not optional launch boilerplate.

After sorting, marking, scanning, and emission kernels produce the range endpoints.
Typed launch contracts and device-buffer types express some safety conditions, but
data-dependent scatter writes and asynchronous allocation/free lifetimes still require
explicit reasoning.

**Shown: 14 code LOC. Omitted from the 1,182-LOC footprint: 1,168 code LOC.**

The footprint contains 599 host/support code LOC and 583 kernel/module code LOC.
The omissions include digit histograms, warp ranking, the rest of look-back, shared
scattering, prefix-sum kernels, range marking/emission, launch validation, pinned/chunked
uploads, stream-ordered cleanup, and wider-key/map support. CUDA-Oxide's compiler,
generated launchers, buffer implementation, and CUDA runtime are outside the count.

## 9. HIP: shared Rust kernels, direct AMD runtime calls

**Speed:** laptop **201–209 ms**. This AMD engine is unavailable on the RTX 3060
desktop. At 2³⁰ `u64` values, laptop HIP took **5.30–5.54 seconds**.

HIP shares its radix-sort and range-finding kernel bodies with offload. The sort is
the same family of eight-bit stable scatter algorithm described above: digit counts,
host-computed bases, tile tickets, warp ranking, and decoupled look-back. It is a
separate implementation from CUDA-Oxide, not that engine's source compiled for AMD.

The distinctive part is the host adapter. It loads an embedded AMD code object, looks
up named kernels, allocates device memory, and passes kernel arguments through HIP's
C interface. Large uploads use two pinned buffers and events so a staging buffer is
not overwritten while a transfer still reads it.

One scatter launch from [the HIP adapter](../src/gpu/hip.rs) looks like this:

<!-- excerpt: src/gpu/hip.rs:472-489 -->

```rust
let (mut from, mut to, mut bases_pointer) = (input.pointer, output.pointer, bases.pointer);
let pass = u32::try_from(pass).expect("at most 8 passes");
let (mut shift, mut offset) = (pass * 8, pass * 256);
launch(
    kernels.radix_scatter,
    blocks,
    scatter_shared(key_bytes::<K>()),
    &mut [
        argument(&mut from),
        argument(&mut len_argument),
        argument(&mut shift),
        argument(&mut bases_pointer),
        argument(&mut offset),
        argument(&mut status_pointer),
        argument(&mut to),
    ],
);
swap(&mut input, &mut output);
```

The argument order, representation, shared-memory size, and launch geometry must
match the compiled kernel. The raw FFI boundary does not give the reader the same
typed argument list as an ordinary Rust function call. After sorting, per-tile range
counts are scanned on the host, then the GPU emits endpoints for download. If the
endpoint representation would outgrow its working buffer, the implementation can
download sorted keys and finish range construction on the host instead.

**Shown: 18 code LOC. Omitted from the 1,550-LOC footprint: 1,532 code LOC.**

The footprint is 471 host code LOC, 143 kernel-entry/glue code LOC, 663 shared
kernel/support code LOC, and 273 shared build-script code LOC. The excerpt leaves out
nearly all of the kernel algorithm, FFI declarations, device/module discovery, warp-size
checks, resource lifetimes, upload synchronization, range finding, and code-object
compilation. HIP/ROCm internals are dependencies outside the count.

## 10. Offload: shared kernels plus compiler/runtime mapping machinery

**Speed:** desktop **164–168 ms** (November 14, not rerun); laptop **194–198 ms**
(November 16, alongside HIP and CubeCL). At 2³⁰ `u64` values:
desktop offload took **11.61 seconds**; laptop offload was not measured in that table.

The large-input algorithm shares the Rust radix and range kernel bodies used by HIP.
Instead of loading kernel symbols and launching through HIP directly, it uses Rust's
nightly `offload!` interface and the OpenMP target runtime. Its device layer selects
NVIDIA or AMD operations for shared memory, warp communication, atomics, and barriers.
The radix algorithm requires 32-lane warps/wavefronts.

The GPU counts digits; the host reads those counts, computes and uploads bases, and
alternates two device buffers across active passes. Here is the buffer selection and
scatter launch from
[the offload radix adapter](../src/gpu/offload/radix.rs):

<!-- excerpt: src/gpu/offload/radix.rs:477-486 -->

```rust
let (from, to) = if in_original {
    (input.pointer(), output.pointer())
} else {
    (output.pointer(), input.pointer())
};
offload! { kernel = radix_scatter::<K>, workgroup_dim = [blocks, 1, 1],
thread_dim = [BLOCK, 1, 1], dyn_cache = launch_shared(scatter_shared(size_of::<K>())), device = 0,
args = (from, len as u32, pass as u32 * 8, bases.pointer(),
    pass as u32 * 256, status.pointer(), to), }
in_original = !in_original;
```

The launch looks shorter than the HIP call, but more machinery sits behind it.
The [resident-buffer adapter](../src/gpu/offload/resident.rs) explicitly allocates
device storage and associates it with host addresses so subsequent launches reuse
the allocation. Downloading and freeing require correct disassociation and lifetimes.
The [build script](../build.rs) coordinates host metadata, GPU compilation, final
embedding, runtime linking, and architecture selection.

There is also a different small-input algorithm. By default, inputs of at most 1,024
keys use **bitonic sorting**: a fixed sequence of compare/exchange stages arranges
pairs in ascending or descending order until the whole padded sequence is ordered.
That path downloads the sorted keys and finds ranges on the host. Larger inputs use
radix sorting by default; the environment can force either choice. The large radix
path finds ranges on the GPU when its endpoint representation fits, with a sorted-key
download/host-range fallback when it does not. The headline 100M measurements exercise
the large radix path, not the small-input bitonic path.

**Shown: 10 code LOC. Omitted from the 1,717-LOC footprint: 1,707 code LOC.**

The footprint is 615 host/support code LOC, 166 entry/launch-glue code LOC, the same
663 shared kernel/support code LOC used by HIP, and the same 273 build-script code LOC.
The omissions include pointer association, allocation/copy/free, histogram preparation,
workspace clearing, the warp-level scatter algorithm, range finding and fallback,
bitonic kernels, vendor-specific intrinsics/PTX, and the multistage build. Compiler
and OpenMP runtime implementations are outside the count. Sharing kernel source does
not imply identical generated GPU code or identical end-to-end runtime costs.

## What the line counts cover

The table makes the size of the excerpts explicit. A “shown” line can call a large
helper that is counted among the omitted lines; the number is not a measure of how
much behavior the excerpt explains.

| Engine/configuration | Repository code LOC | Shown here | Omitted here |
| --- | ---: | ---: | ---: |
| CPU comparison, 1 thread | 154 | 2 | 152 |
| CPU comparison, 16 threads | 154 | 11 | 143 |
| CPU radix, 1 thread | 306 | 2 | 304 |
| CPU radix, 16 threads | 306 | 2 | 304 |
| `CubeCL` | 788 | 7 | 781 |
| cuTile | 850 | 4 | 846 |
| wgpu | 891 | 8 | 883 |
| CUDA-Oxide | 1,182 | 14 | 1,168 |
| HIP | 1,550 | 18 | 1,532 |
| Offload | 1,717 | 10 | 1,707 |

Counts are nonblank physical source lines excluding comment-only lines, including
attributes, imports, and delimiter-only lines in the selected regions. They describe
backend footprints, including wider-key and map support where it is in those regions,
not minimal extracted `u32` programs. The host/device split is a reading aid: some files
contain declarations or launch glue for both sides.

The inventory below is pinned to `b97205e7`, except the parallel range finder and the
`CubeCL` engine, which are counted in the uncommitted working tree; source line intervals are inclusive. Shared
components are listed once and included in each applicable engine's footprint.

| Component | Source regions | Code LOC |
| --- | --- | ---: |
| CPU comparison wrapper | `src/gpu.rs` 488–499 | 11 |
| CPU pool, shared by both CPU families | `src/gpu.rs` 503–517 | 15 |
| CPU radix sorter | `src/gpu/cpu_radix.rs` 1–202 | 163 |
| CPU parallel range finder, shared by both CPU families | `src/gpu/parallel_ranges.rs` 1–194; `src/from_slice.rs` 128–133; `GpuElement` dispatch in `src/gpu.rs` | 128 |
| `CubeCL` kernels | `src/gpu/cubecl.rs` 1–317 | 253 |
| `CubeCL` host | `src/gpu/cubecl.rs` 318–522 | 174 |
| `CubeCL` pinned CUDA/HIP copies | `src/gpu/cubecl.rs` 523–939 | 332 |
| `CubeCL` dispatch | `from_slice_cubecl` in `src/gpu.rs` | 29 |
| wgpu host/support | `src/gpu.rs` 519–578 and 589–1376 | 790 |
| wgpu shaders | `src/gpu.rs` 39–154 | 101 |
| CUDA-Oxide host/support | `src/gpu/cuda.rs` 1–65 and 753–1434 | 599 |
| CUDA-Oxide kernel/module | `src/gpu/cuda.rs` 66–751 | 583 |
| cuTile host/support | `src/gpu/cutile.rs` 1–40, 808–976, and 983–1145 | 281 |
| cuTile kernel/module | `src/gpu/cutile.rs` 41–806 | 569 |
| HIP host | `src/gpu/hip.rs` 1–568 | 471 |
| HIP entry/glue | `src/gpu/offload/hip_native.rs`, whole file | 143 |
| Offload host/support | `src/gpu/offload.rs` 1–246; `resident.rs`, whole file; `radix.rs` 406–694 | 615 |
| Offload entry/glue | `src/gpu/offload/native.rs`, whole file | 166 |
| HIP/offload shared kernels/support | `src/gpu/offload/{key,device,ptx,ranges}.rs`, whole files; `radix.rs` 1–405 | 663 |
| HIP/offload shared build | `build.rs`, whole file | 273 |

The shortened `resident.rs` and `radix.rs` paths in this table are under
`src/gpu/offload/`. These counts exclude the common engine dispatch, general key
encoding/decoding implementations, shared `from_slice`/set-building internals, tests,
benchmark harnesses, manifests, setup commands, and external dependencies. The
one-thread comparison path does not use the counted pool; its family footprint retains
that code to match the earlier engine-size comparison.

Dependencies are counted rather than measured in lines. The dependency column lists the
direct crates each engine's code uses beyond the crate's base dependencies, plus anything
outside Cargo that it needs to build or run:

| Engine/configuration | Crates | Outside Cargo |
| --- | --- | --- |
| CPU comparison | 1: `rayon` | none |
| CPU radix | 1: `rayon` | none |
| `CubeCL` | 4: `cubecl`, `cubecl-runtime`, `cubecl-hip-sys`, `cudarc` | CUDA driver or AMD HIP runtime; Rust 1.95 or newer |
| cuTile | 1: `cutile` | CUDA toolkit where the program runs (kernels compile on first use) |
| wgpu | 3: `wgpu`, `lampshade`, `pollster` | a Vulkan, Metal, or DirectX 12 driver |
| CUDA-Oxide | 3: `cuda-core`, `cuda-device`, `cuda-host` | `cargo oxide` toolchain, CUDA driver |
| HIP | none | nightly Rust with offload support, AMD HIP runtime |
| Offload | none | nightly Rust with offload support, OpenMP target runtime, vendor toolkit |

These are per-engine needs, not what a build compiles: the experimental `shuffled_slices`
feature currently enables `rayon`, `wgpu`, `lampshade`, and `pollster` together for
every engine.

The practical comparison is therefore twofold: how much code the project owns, and
how much mechanism someone must understand when modifying it. The CPU baseline is
competitive because it combines a strong sort (Rayon's comparison sort, or a radix sort
tuned for narrow keys) with threads, SIMD, and parallel range finding. On the headline
workload, the desktop CPU radix engine (169–182 ms) trails CUDA-Oxide (130–133 ms) and
CubeCL (138–146 ms), and beats cuTile (215–224 ms). Desktop offload's older 164–168 ms
measurement is from a different session. On the laptop, CPU radix (127–136 ms) is about
1.5× as fast as the fastest measured GPU engine, offload (194–198 ms). The GPU
implementations show how much additional work it takes to approach or exceed that baseline.

`CubeCL` changes the portability trade-off. Comparing range midpoints in the same sessions,
it takes about 8% longer than CUDA-Oxide on the desktop (138–146 ms against 130–133 ms)
and about 13% longer than HIP on the laptop (230–235 ms against 201–209 ms), from one
788-LOC code base. Laptop offload is faster still (194–198 ms). The two vendor-specific
engines together are about 2,700 LOC. Its cost is a young framework and a copy path, 42% of
its code, that works around the framework's slow transfers.
