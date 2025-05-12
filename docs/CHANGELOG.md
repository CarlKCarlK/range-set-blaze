# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.2.0] - 2024-cmk

- Added support for maps, `RangeMapBlaze`.
- Some breaking changes in the rest of the package.

## [0.1.16] - 2024-cmk

- Added `RangeSetBlaze::from_sorted_starts`
- Added documentation for `SortedStarts` and `AssumeSortedStarts`
- Changed `CheckSortedDisjoint::new` to support `IntoIterator`
- Changed `AssumeSortedStarts::new` to support `IntoIterator`

## [0.1.15] - 2024-cmk

- Added DoubleEndedIterator when iterating integer elements. Thanks to enh.

## [0.1.14] - 2023-12-5

### Changed

- Added optional `from_slice` cargo feature and support
  for a new `RangeSetBlaze::from_slice` constructor.
  The feature uses SIMD and requires Rust nightly.
