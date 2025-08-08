# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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
