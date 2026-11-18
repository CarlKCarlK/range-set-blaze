# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

This release is not breaking.

### Added

- **`RangeMapBlaze::from_slices(keys, values)`** builds a map from a slice of keys and a
  same-length slice of values (two columns of a table), using SIMD to find runs of consecutive
  keys. Values may be any `V: Eq + Clone`. On clumpy keys it is typically 6 to 16 times faster
  than `from_iter`. It panics if the lengths differ.
- **Experimental `shuffled_slices` feature, with `RangeSetBlaze::from_shuffled_slice(&mut
  slice, engine)`.** It sorts the slice in place, then finds the ranges, so it suits large slices
  in no useful order (for example, shuffled). `Engine::Cpu(n)` sorts with Rayon on `n` threads;
  `Engine::Wgpu` sorts and finds the ranges on the GPU via `wgpu`, panicking if it can't run;
  `Engine::WgpuOrCpu` falls back to every thread. On 100 million shuffled `u32` values on an AMD Ryzen AI MAX+ 395 laptop, `Cpu(32)` took
  463 ms and its integrated GPU 644 ms, against 4,044 ms for `from_slice`; on data already in
  order, `from_slice` is fastest. The feature is off by default and adds no dependencies unless
  enabled; it is mainly a demonstration of GPU programming and a fair test of it.
- **`RangeMapBlaze::from_shuffled_slices(&keys, &values, engine)`** (same experimental
  `shuffled_slices` feature) builds a map from key and value slices whose keys are in no useful
  order, on several CPU threads or the GPU. It leaves the slices unchanged. The last value wins for a repeated key, as with `from_slices`. Any
  `Integer` keys and any `V: Eq + Clone` values work on the CPU, including borrowed and non-`Sync`
  values; the GPU handles keys of at most 32 bits with the crate's integer values of at most 64
  bits, and `WgpuOrCpu` silently uses the CPU for other types. On 10 million shuffled `u32` pairs
  on an AMD Ryzen 9 7950X with an NVIDIA RTX 3060, `Cpu(32)` took 50 to 130 ms and the GPU 37 to
  123 ms when the map had at most 2.5 million ranges, against 1.8 to 2.0 seconds for
  `from_slices`.
- **`NotNan::from_primitive_vec`** converts a `Vec<f64>` (or `f32`, ...) into `NotNan` values in
  place, reusing the allocation, normalizing `-0.0` to `+0.0`, and panicking on NaN.
- The float docs have a new "Converting Native Floats" section that lays out every conversion
  into and out of `NotNan` and `Total` (values, ranges, iterators, slices, vectors) and their NaN
  and `-0.0` rules.

### Changed

- **`RangeSetBlaze::from_slice` now works on stable Rust and is no longer behind a Cargo
  feature.** The constructor, and its `fearless_simd` and `bytemuck` dependencies, are now
  unconditional parts of the crate, available in every build (including `no_std`). The
  `from_slice` feature still exists so that `features = ["from_slice"]` keeps resolving, but it
  does nothing and is deprecated; drop it at your convenience.
- Types outside this crate that implement `Integer` need no changes: there are no new required
  methods.
- The SIMD implementation moved off nightly-only `portable_simd` onto [`fearless_simd`], which
  runs on stable Rust:
  - With the `std` feature (the default), the best SIMD instructions supported by the CPU running
    the program are detected and dispatched to at runtime.
  - Without `std` (`no_std`), it uses whatever SIMD instructions the build target and compiler
    flags enable. On targets without SIMD, such as many microcontrollers, it runs portable scalar
    code, which is still faster than `from_iter` on clumpy integers.
- `isize` and `usize` slices are now SIMD-accelerated as well, mapped to the fixed-width integer
  lanes matching `target_pointer_width` (`i16`/`u16`, `i32`/`u32`, or `i64`/`u64`). Their SIMD load
  path uses [`bytemuck`] to reinterpret the input slice in place, rather than copying it
  element-by-element, closing most of the previous performance gap between `isize`/`usize` and
  the corresponding fixed-width integer types.
- **`from_slice` is faster.** It now checks four SIMD chunks per loop iteration, collects
  smaller intermediate ranges, and uses 512-bit vectors for every integer type (previously 128 or
  256 bits for 8- and 16-bit types). On clumpy data it is typically 10 to 20 times faster than
  `from_iter`/`collect`; checked on AMD, Intel, and ARM (including Apple) CPUs.
- `from_slice` now uses SIMD for `char`, `Ipv4Addr`, `u128`, `i128`, and `Ipv6Addr` too. 128-bit
  values span two 64-bit lanes; the IP address types, whose memory layout Rust doesn't guarantee,
  are converted lane by lane rather than reinterpreted. On clumpy IP addresses they are about
  17 to 33 times faster than `from_iter`.
- `from_slice` and `from_slices` now use SIMD for float keys too: `NotNanF32`, `NotNanF64`,
  `TotalF32`, and `TotalF64`. Each lane holds the float's order-preserving integer key (the
  `total_cmp` order). On 10 million clumpy `NotNanF32` values in order, `from_slice` dropped from
  about 53 ms to 2 ms, about the speed of `u32`.
- `NotNan::from_primitive_slice` now validates `f32`/`f64` with SIMD (about 11 times faster).
- The docs, README, and examples now recommend `from_slice` whenever the integers are in a slice,
  array, or `Vec`.
- The benchmark docs report `RangeMapBlaze::from_slices` (in `bench_map.md`) and a new IP address
  benchmark comparing `from_slice` and `from_iter` (in `bench.md`).
- The pinned nightly toolchain is now `nightly-2026-10-12` (was `nightly-2026-09-15`). This
  affects only local builds and CI.

### Removed

- The `build-simd` and `test-simd` just recipes.
- The unused `num-integer` dependency.

[`fearless_simd`]: https://crates.io/crates/fearless_simd
[`bytemuck`]: https://crates.io/crates/bytemuck

Note: 0.8.0 already includes the 0.7.1 `from_slice` wraparound correctness fix described below;
this release keeps that fix and adds a regression test for runs that wrap from MAX to MIN.

## [0.8.0] - 2026-10-09

This is a breaking release (see **Changed** and **Removed**). Code that only uses
`RangeSetBlaze`, `RangeMapBlaze`, and their operators and methods is unaffected
apart from the new minimum Rust version. It also includes the `from_slice`
fix described under 0.7.1, which was never published to crates.io.

### Added

- **Map joins.** `SortedDisjointMap::inner_join`, `left_join`, and `full_join`
  pair up two map streams by key, yielding `(range, (left, right))` with
  `Option` for a side that may be absent. `RangeMapBlaze::inner_join`,
  `left_join`, and `full_join` take a function of the two values (for
  example, `a.inner_join(&b, |l, r| ...)`) and return a map of its results.
  `left_join` stops as soon as the left input ends, without reading the rest
  of the right input.
- **A joins guide** (`range_set_blaze::joins`) explains the joins,
  `transform_values`, and `sweep`, and when to use each.
- **Multiway joins.** `MultiwaySortedDisjointMap::inner_join` and `full_join`
  (and the same on collections of `&RangeMapBlaze`) call a closure once per
  stretch of keys with every input's value there (`&[V]`, or
  `&[Option<V>]` for the full join) and collect its results.
- **`transform_values`** on map streams and on `RangeMapBlaze`: same keys,
  new values, merging touching ranges whose new values are equal.
- **`MultiwaySortedDisjointMap::sweep`**, a low-level stream of every input
  range's start and end in key order (`SweepEvent`), for computations the
  joins do not cover. The multiway joins are built on it.
- **`Owned<V>`**, a `ValueCarrier` that holds its value directly, so stream
  operations can produce new values without allocating. In the prelude.
- `RangeMapBlaze::universe_with(&value)` and `RangeSetBlaze::universe()`.
- `ValueCarrier` for pairs of carriers, so joined streams compose with other
  map operations.
- New iterator types: `InnerJoinMap`, `LeftJoinMap`, `FullJoinMap`,
  `TransformValuesMap`, `MultiwayInnerJoinMap`, `MultiwayFullJoinMap`,
  `MultiwaySweep`, and `SweepEvent`.

### Changed

- **Minimum supported Rust version is now 1.89** (was 1.87).
- **Faster map unions and symmetric differences.** The two-way map `|` and
  `^` operators are about 1.6-2.9x faster; multiway map `union` and
  `symmetric_difference` are about 1.4-4x faster. Multiway set
  `symmetric_difference` is also faster in most cases (up to about 1.23x
  slower with 2-4 sparse inputs). Results are unchanged.
- **Breaking: set and map operation result types are now opaque structs named
  after their operation** (as std names `btree_set::Union`): `Union`,
  `Intersection`, `Difference`, `SymmetricDifference`, `MultiwayUnion`,
  `MultiwayIntersection`, `MultiwaySymmetricDifference`, `UnionMap`,
  `IntersectionMap`, `DifferenceMap`, `SymmetricDifferenceMap`, `NotMap`,
  `MultiwayUnionMap`, `MultiwayIntersectionMap`, and
  `MultiwaySymmetricDifferenceMap`. They were hidden type aliases exposing
  internal types, so any change of implementation changed them; now their
  fields are private. Code that only iterates or chains these results is
  unaffected. The old names (`UnionMerge`, `IntersectionMerge`,
  `DifferenceMerge`, `SymDiffMerge`, `UnionKMerge`, `SymDiffKMerge`,
  `UnionMergeMap`, `SymDiffMergeMap`, `UnionKMergeMap`, `SymDiffKMergeMap`,
  `IntersectionKMap`) remain as deprecated aliases. Code that named the old
  internal types directly (for example `UnionIter<...>` for `a | b`) must use
  the new names.
- **Breaking:** `SortedDisjointMap::complement` now returns `NotMap` (was the
  equivalent `NotIter<..., RangeValuesToRangesIter<...>>`), `complement_with`
  returns `RangeToRangeValueIter<..., NotMap<...>>`, and
  `MultiwaySortedDisjointMap::intersection` no longer has an unused lifetime
  parameter.
- The `RangeMapBlaze` docs now state that values should be cheap to clone,
  because operations clone a value whenever they split its range; wrap large
  values in `Rc` or `Arc`.
- `just check-all` now also runs the MSRV check, matching CI.
- The pinned development toolchain is now Rust 1.99.0 (was 1.97.0). This
  affects only local builds and CI, not the minimum supported Rust version.

### Removed

- **Breaking:** `KMergeMap`, `MergeMap`, and `SymDiffIterMap`. They had no
  public constructors and are no longer used by the crate.

## [0.7.1] - 2026-09-25

This version was never published to crates.io. Its fix first ships in 0.8.0.

### Fixed

- **Correctness fix for the opt-in, nightly-only `from_slice` feature.** With
  that feature enabled, `RangeSetBlaze::from_slice` (and `RangeSetBlaze::from`
  on an array, which uses `from_slice` when the feature is on) could silently
  drop elements, or return an empty set, when the input contained a run of
  consecutive values that wrapped from the type's maximum to its minimum, such as
  `[.., 254u8, 255, 0, 1, ..]` or `[.., i64::MAX, i64::MIN, ..]`. The SIMD check
  for "this chunk is consecutive" used wrapping lane arithmetic, so a wrapped
  chunk passed; a similar wrap at chunk boundaries could merge a run ending at
  `MAX` with one starting at `MIN`. Both cases now fall back to treating the
  values individually, and regression tests compare `from_slice` against
  `from_iter` on wrapping runs for every supported integer type and alignment.
- Builds without the `from_slice` feature, including the default feature set
  and all stable-Rust builds, were not affected: they never use this code path.

## [0.7.0] - 2026-09-15

### Added

- Added the experimental `cursor_nightly_experimental` feature, which uses Rust's
  nightly-only B-tree cursor API to speed up `insert`/`ranges_insert` on both
  `RangeSetBlaze` (~2x) and `RangeMapBlaze` (~1.7x), with the same public API.
- Floating-point range support for `f32` and `f64`, plus `range_at`,
  `range_or_gap_at`, and `fill_gaps`, are now unconditional public APIs.
- `RangeSetBlaze::fill_gaps` returns a `RangeMapBlaze<T, bool>` and
  `RangeMapBlaze::fill_gaps` returns a `RangeMapBlaze<T, Option<V>>`, following
  the crate's usual two-layer design: collection methods return gathered
  collections, while `SortedDisjoint::fill_gaps` and
  `SortedDisjointMap::fill_gaps` return the lazy streams
  (`FillGapsIter` and `FillGapsIterMap`) for advanced composition. Both cover
  the complete integer domain from `T::min_value()` through `T::max_value()`.
- `FillGapsIterMap` and `FillGapsIter` now implement `SortedDisjointMap` and can be
  collected directly into `RangeMapBlaze<T, Option<V>>` and
  `RangeMapBlaze<T, bool>`, respectively. `bool` now implements `ValueCarrier` as
  the canonical by-value carrier for totalized sets.
- Added `docs/release_checklist.md`, the canonical release procedure, and a
  "Policy on AI-assisted development and contributions" section in the README.

### Fixed

- Fixed doc links on `FillGapsIter`/`FillGapsIterMap` that resolved to the wrong
  path because the linked-to trait docs used a bare relative `.html` path instead
  of an intra-doc `crate::` path; this broke once a type implementing
  `SortedDisjoint`/`SortedDisjointMap` first existed in a nested module (`gaps`).

### Changed

- CI's nightly toolchain (used for the `cursor_nightly_experimental`/
  `float_nightly_experimental` checks, Clippy, and docs) is now pinned to a fixed
  dated nightly instead of floating `nightly`, mirroring the already-pinned
  stable toolchain in `rust-toolchain.toml`. A floating nightly previously broke
  CI when upstream Clippy added the `single_range_in_vec_init` lint against this
  crate's idiomatic `from_iter([a..=b])` single-range construction; that lint is
  now explicitly allowed for the nightly Clippy run. Bump the pinned nightly
  deliberately, the same way the stable toolchain is bumped.
- The 32-bit Ubuntu CI job now retries its 32-bit `apt-get` package install
  with a fresh package list, since `awalsh128/cache-apt-pkgs-action` reports
  success even when its internal install fails (e.g. a transient mirror 404),
  which previously surfaced later as a confusing linker error instead of a
  clear package-install failure.
- Qualified test-only `min_value()`/`max_value()` calls on concrete primitive
  integer types to the crate's `Integer` trait, removing `deprecated` warnings
  that arose because Rust resolves those names to std's deprecated inherent
  methods before the trait method.

### Breaking

- Removed the `rog_experimental` and `float_experimental` Cargo features. Remove
  them from dependency feature lists. `float_nightly_experimental` remains for
  nightly-only `f16` and `f128` support.
- Removed `Rog`, `RogsIter`, and the old `rogs_*` methods. Use `range_at`,
  `range_or_gap_at`, or `fill_gaps` instead.
- Renamed `ValueRef` to `ValueCarrier` and redesigned it with a `Value`
  associated type and `value_eq` method; generic code now uses explicit
  borrowing only where borrowed access to the logical value is required.
- Renamed `Finite`/`FiniteF32`/`FiniteF64`/`FiniteF16`/`FiniteF128` to
  `NotNan`/`NotNanF32`/`NotNanF64`/`NotNanF16`/`NotNanF128` (module `finite` ->
  `not_nan`, shorthand constructors `ff64`/... -> `nnf64`/...,
  `FiniteSliceExt`/`FiniteRangeExt` -> `NotNanSliceExt`/`NotNanRangeExt`). The
  type's legal domain grew to match: it now excludes only NaN, so `+infinity`
  and `-infinity` are legal values and `MIN`/`MAX` are the two infinities
  (previously `f64::MIN`/`f64::MAX`); `MAX_SIZE` grew by 2 to account for them.

## [0.6.1] - 2026-07-10

### Added

- Added experimental floating-point range support through the `float_experimental` feature
  for `f32`/`f64`, and the `float_nightly_experimental` feature for `f16`/`f128`.
  `Finite` supports finite values with zero canonicalized to `+0.0`, while `Total` supports
  all floating-point values in total order, including NaNs, infinities, and signed zero.
  Both provide constructors, range operations, and `after`/`before` navigation APIs.

## [0.6.0] - 2026-06-26

### Added

- `AssumeSortedStarts` now verifies, in debug builds only, that the ranges it is given
  actually have sorted starts, panicking with a clear message otherwise. Release builds are
  unaffected (zero runtime cost). (PR #29)

### Breaking

- `AssumeSortedStarts` regains a leading type parameter: `AssumeSortedStarts<I>` ->
  `AssumeSortedStarts<T, I>`. Most `AssumeSortedStarts::new(iter)` calls are unchanged due to
  inference; explicit type aliases/annotations may need updating.

## [0.5.1] - 2026-06-26

### Added

- Implemented `SortedStarts`/`SortedDisjoint` (and the `*Map` equivalents) for applicable
  `core::iter`/`std::iter` adapter types whose iterators cannot break the sorted-disjoint
  invariants, so these standard iterators can be used directly with the library's operators
  (PR #27).
- Added a combinator to turn an `Option<SortedDisjoint>` into a `SortedDisjoint` via
  `FlatMap`/`Flatten`.

### Notes

- `core::iter::StepBy` is not yet supported because it does not implement `FusedIterator`
  upstream; see the TODO in `src/sorted_disjoint.rs` and
  <https://internals.rust-lang.org/t/implement-fusediterator-for-core-stepby/24074>

## [0.5.0] - 2026-03-06

### Changed

- Removed redundant struct-level trait bounds across iterator/map/set wrapper structs and kept
  constraints on impls/usage sites where needed.

### Breaking

- `AssumeSortedStarts` and `AssumePrioritySortedStartsMap` now take fewer generic parameters.
  - `AssumeSortedStarts<T, I>` -> `AssumeSortedStarts<I>`
  - `AssumePrioritySortedStartsMap<T, VC, I>` -> `AssumePrioritySortedStartsMap<I>`
- Migration note: calls like `AssumeSortedStarts::new(iter)` and
  `AssumePrioritySortedStartsMap::new(iter)` are typically unchanged due to inference; explicit
  type aliases/annotations may need updates.

## [0.4.4] - 2026-02-25

### Changed

- `RangeSetBlaze::ranges_insert` and `RangeMapBlaze::ranges_insert` now accept any `RangeBounds<T>`, not just `RangeInclusive<T>` (issue #24).

## [0.4.3] - 2026-02-25

### Changed

- Improved local CI ergonomics and speed:
  - `cargo check-all` now runs independent checks in parallel
  - local `check-all` test target now uses `--lib --tests --examples` to skip slow benches
- Updated `from_slice` SIMD imports/bounds for compatibility with current Rust nightly APIs.
- Updated `examples/nine_rules_maps` assertions to match right-to-left precedence semantics.

## [0.4.2] - 2026-02-01

### Changed

- Updated `from_slice` feature for compatibility with Rust nightly (post-2026-01-28)
  - Removed obsolete `LaneCount` and `SupportedLaneCount` trait bounds from SIMD code
  - Lane count constraints now compiler-enforced (max 64 lanes, power-of-two only)
  - Requires recent Rust nightly for `from_slice` feature
  - No functional changes; purely a compatibility update

## [0.4.1] - 2025-10-26

- Implemented `RangeOnce<T>`, a zero-allocation adapter that yields **0 or 1**
  non-empty inclusive ranges.
  - `RangeOnce` implements `SortedStarts<T>` and `SortedDisjoint<T>`,
    providing a sound and ergonomic way to work with single ranges.
  - Example:

    ```rust
    &a & RangeOnce::new(15, 35);
    &a | RangeOnce::new(22, 25);  
    ```

  - Empty ranges (`start > end`) now produce an empty iterator,
    preserving all invariants.
- Added `From<RangeInclusive<T>>` for `RangeSetBlaze`, allowing direct
  conversion of single ranges:

  ```rust
  RangeSetBlaze::from(5..=10);
  RangeSetBlaze::from(5..=4); // yields empty set
  ```

## [0.4.0] - 2025-8-9

- Added `.is_universal()` method.
- Breaking change: Renamed the `ValueRef` trait method from `to_owned()` to `into_value()`  to avoid conflict with the standard library’s `ToOwned` trait.

## [0.3.0] - 2025-5-29

- Changed precedence of `RangeMapBlaze` to always be right-to-left.

## [0.2.0] - 2025-5-13

- Added support for maps, `RangeMapBlaze`.
- Add support for `char`, `IpAddV4`, and `IpAddV6` integer-like types.
- Some breaking changes in the rest of the package to improve
  the API and performance.

## [0.1.16] - 2024-3-9

- Added `RangeSetBlaze::from_sorted_starts`
- Added documentation for `SortedStarts` and `AssumeSortedStarts`
- Changed `CheckSortedDisjoint::new` to support `IntoIterator`
- Changed `AssumeSortedStarts::new` to support `IntoIterator`

## [0.1.15] - 2024-2-9

- Added DoubleEndedIterator when iterating integer elements. Thanks to enh.

## [0.1.14] - 2023-12-5

### Changed

- Added optional `from_slice` cargo feature and support
  for a new `RangeSetBlaze::from_slice` constructor.
  The feature uses SIMD and requires Rust nightly.
