range-set-blaze
==========

[![github](https://img.shields.io/badge/github-range-set-blaze-8da0cb?style=flat&labelColor=555555&logo=github)](https://github.com/CarlKCarlK/range-set-blaze)
[![crates.io](https://img.shields.io/crates/v/range-set-blaze.svg?flat&color=fc8d62&logo=rust")](https://crates.io/crates/range-set-blaze)
[![docs.rs](https://img.shields.io/badge/docs.rs-range-set-blaze-66c2a5?flat&labelColor=555555&logoColor=white&logo=core:image/svg+xml;base64,PHN2ZyByb2xlPSJpbWciIHhtbG5zPSJodHRwOi8vd3d3LnczLm9yZy8yMDAwL3N2ZyIgdmlld0JveD0iMCAwIDUxMiA1MTIiPjxwYXRoIGZpbGw9IiNmNWY1ZjUiIGQ9Ik00ODguNiAyNTAuMkwzOTIgMjE0VjEwNS41YzAtMTUtOS4zLTI4LjQtMjMuNC0zMy43bC0xMDAtMzcuNWMtOC4xLTMuMS0xNy4xLTMuMS0yNS4zIDBsLTEwMCAzNy41Yy0xNC4xIDUuMy0yMy40IDE4LjctMjMuNCAzMy43VjIxNGwtOTYuNiAzNi4yQzkuMyAyNTUuNSAwIDI2OC45IDAgMjgzLjlWMzk0YzAgMTMuNiA3LjcgMjYuMSAxOS45IDMyLjJsMTAwIDUwYzEwLjEgNS4xIDIyLjEgNS4xIDMyLjIgMGwxMDMuOS01MiAxMDMuOSA1MmMxMC4xIDUuMSAyMi4xIDUuMSAzMi4yIDBsMTAwLTUwYzEyLjItNi4xIDE5LjktMTguNiAxOS45LTMyLjJWMjgzLjljMC0xNS05LjMtMjguNC0yMy40LTMzLjd6TTM1OCAyMTQuOGwtODUgMzEuOXYtNjguMmw4NS0zN3Y3My4zek0xNTQgMTA0LjFsMTAyLTM4LjIgMTAyIDM4LjJ2LjZsLTEwMiA0MS40LTEwMi00MS40di0uNnptODQgMjkxLjFsLTg1IDQyLjV2LTc5LjFsODUtMzguOHY3NS40em0wLTExMmwtMTAyIDQxLjQtMTAyLTQxLjR2LS42bDEwMi0zOC4yIDEwMiAzOC4ydi42em0yNDAgMTEybC04NSA0Mi41di03OS4xbDg1LTM4Ljh2NzUuNHptMC0xMTJsLTEwMiA0MS40LTEwMi00MS40di0uNmwxMDItMzguMiAxMDIgMzguMnYuNnoiPjwvcGF0aD48L3N2Zz4K)](https://docs.rs/range-set-blaze)[![CI](https://github.com/CarlKCarlK/range-set-blaze/actions/workflows/ci.yml/badge.svg)](https://github.com/CarlKCarlK/range-set-blaze/actions/workflows/ci.yml)

<!-- FUTURE: Add coverage badge? -->

Integer sets as fast, sorted, integer ranges with full set operations

The integers can be any size ([`u8`] to [`u128`]) and may be signed ([`i8`] to [`i128`]). The [set operations] include `union`, `intersection`, `difference`, `symmetric difference`, and `complement`.

The crate's main struct is [`RangeSetBlaze`], a set of integers. See the [`RangeSetBlaze` documentation] for details.

> Unlike the standard [`BTreeSet`] and [`HashSet`], [`RangeSetBlaze`] does not store every integer in the set. Rather, it stores sorted & disjoint ranges of integers in a cache-efficient [`BTreeMap`]. It differs from [other interval libraries](https://github.com/CarlKCarlK/range-set-blaze/blob/main/docs/bench.md) -- that we know of -- by
offering full set operations and by being optimized for sets of [clumpy](1) integers.
>
> We can construct a [`RangeSetBlaze`] from unsorted & redundant integers (or ranges). When the inputs are clumpy, construction will be [linear][1] in the number of inputs and set operations will be sped up [quadratically][1].

The crate's main trait is [`SortedDisjoint`]. It is implemented by iterators of sorted & disjoint ranges of integers. See the [`SortedDisjoint` documentation] for details.

> With any [`SortedDisjoint`] iterator we can perform set operations in one pass through the ranges and with minimal (constant) memory. It enforces the "sorted & disjoint" constraint at compile time. This trait is inspired by the `SortedIterator` trait from the [sorted_iter](https://crates.io/crates/sorted_iter) crate. [`SortedDisjoint`] differs from its inspiration by specializing on disjoint integer ranges.

[`RangeSetBlaze`]: struct.RangeSetBlaze.html
[`SortedDisjoint`]: trait.SortedDisjoint.html
[`u8`]: https://doc.rust-lang.org/std/primitive.u8.html
[`u128`]: https://doc.rust-lang.org/std/primitive.u128.html
[`i8`]: https://doc.rust-lang.org/std/primitive.i8.html
[`i128`]: https://doc.rust-lang.org/std/primitive.i128.html
[`RangeSetBlaze` documentation]: struct.RangeSetBlaze.html
[`SortedDisjoint` documentation]: trait.SortedDisjoint.html
[`BTreeSet`]: std::collections::BTreeSet
[`HashSet`]: std::collections::HashSet
[`BTreeMap`]: std::collections::BTreeMap
[set operations]: rangesetblaze-set-operations
[1]: struct.RangeSetBlaze.html#constructor-performance

Benchmarks
-----------

See the [benchmarks](https://github.com/CarlKCarlK/range-set-blaze/blob/main/docs/bench.md) for performance comparisons with other range-related crates.

Generally, for many tasks involving clumpy integers and ranges, `RangeSetBlaze` is much faster than alternatives.

The benchmarks are in the `benches` directory. To run them, use `cargo bench`.

Examples
-----------

Example 1
- - - - - -

Here we take the union (operator “|”) of two [`RangeSetBlaze`]'s:

![Example 1](https://raw.githubusercontent.com/CarlKCarlK/range-set-blaze/main/docs/rust_example1.png "Example 1")

```rust
use range_set_blaze::RangeSetBlaze;

 // a is the set of integers from 100 to 499 (inclusive) and 501 to 1000 (inclusive)
let a = RangeSetBlaze::from_iter([100..=499,501..=999]);
 // b is the set of integers -20 and the range 400 to 599 (inclusive)
let b = RangeSetBlaze::from_iter([-20..=-20,400..=599]);
// c is the union of a and b, namely -20 and 100 to 999 (inclusive)
let c = a | b;
assert_eq!(c, RangeSetBlaze::from_iter([-20..=-20,100..=999]));
```

Example 2
- - - - - -

In biology, suppose we want to find the intron regions of a gene but we are given only the transcription region and the exon regions.

![Example 2](https://raw.githubusercontent.com/CarlKCarlK/range-set-blaze/main/docs/rust_example2.png "Example 2")

We create a [`RangeSetBlaze`] for the transcription region and a [`RangeSetBlaze`] for all the exon regions.
Then we take the difference between the transcription region and exon regions to find the intron regions.

```rust
use range_set_blaze::RangeSetBlaze;

let line = "chr15   29370   37380   29370,32358,36715   30817,32561,37380";

// split the line on white space
let mut iter = line.split_whitespace();
let chr = iter.next().unwrap();

// Parse the start and end of the transcription region into a RangeSetBlaze
let trans_start: i32 = iter.next().unwrap().parse().unwrap();
let trans_end: i32 = iter.next().unwrap().parse().unwrap();
let trans = RangeSetBlaze::from_iter([trans_start..=trans_end]);
assert_eq!(trans, RangeSetBlaze::from_iter([29370..=37380]));

// Parse the start and end of the exons into a RangeSetBlaze
let exon_starts = iter.next().unwrap().split(',').map(|s| s.parse::<i32>());
let exon_ends = iter.next().unwrap().split(',').map(|s| s.parse::<i32>());
let exon_ranges = exon_starts
    .zip(exon_ends)
    .map(|(s, e)| s.unwrap()..=e.unwrap());
let exons = RangeSetBlaze::from_iter(exon_ranges);
assert_eq!(
    exons,
    RangeSetBlaze::from_iter([29370..=30817, 32358..=32561, 36715..=37380])
);

// Use 'set difference' to find the introns
let intron = trans - exons;
assert_eq!(intron, RangeSetBlaze::from_iter([30818..=32357, 32562..=36714]));
for range in intron.ranges() {
    let (start, end) = range.into_inner();
    println!("{chr}\t{start}\t{end}");
}
```
