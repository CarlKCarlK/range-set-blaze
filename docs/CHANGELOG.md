# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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
  - `AssumePrioritySortedStartsMap<T, VR, I>` -> `AssumePrioritySortedStartsMap<I>`
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
