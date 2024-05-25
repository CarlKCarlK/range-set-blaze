range-set-blaze
==========

[![github](https://img.shields.io/badge/github-range--set--blaze-8da0cb?style=flat&labelColor=555555&logo=github)](https://github.com/CarlKCarlK/range-set-blaze)
[![crates.io](https://img.shields.io/crates/v/range-set-blaze.svg?flat&color=fc8d62&logo=rust")](https://crates.io/crates/range-set-blaze)
[![docs.rs](https://img.shields.io/badge/docs.rs-range--set--blaze-66c2a5?flat&labelColor=555555&logoColor=white&logo=core:image/svg+xml;base64,PHN2ZyByb2xlPSJpbWciIHhtbG5zPSJodHRwOi8vd3d3LnczLm9yZy8yMDAwL3N2ZyIgdmlld0JveD0iMCAwIDUxMiA1MTIiPjxwYXRoIGZpbGw9IiNmNWY1ZjUiIGQ9Ik00ODguNiAyNTAuMkwzOTIgMjE0VjEwNS41YzAtMTUtOS4zLTI4LjQtMjMuNC0zMy43bC0xMDAtMzcuNWMtOC4xLTMuMS0xNy4xLTMuMS0yNS4zIDBsLTEwMCAzNy41Yy0xNC4xIDUuMy0yMy40IDE4LjctMjMuNCAzMy43VjIxNGwtOTYuNiAzNi4yQzkuMyAyNTUuNSAwIDI2OC45IDAgMjgzLjlWMzk0YzAgMTMuNiA3LjcgMjYuMSAxOS45IDMyLjJsMTAwIDUwYzEwLjEgNS4xIDIyLjEgNS4xIDMyLjIgMGwxMDMuOS01MiAxMDMuOSA1MmMxMC4xIDUuMSAyMi4xIDUuMSAzMi4yIDBsMTAwLTUwYzEyLjItNi4xIDE5LjktMTguNiAxOS45LTMyLjJWMjgzLjljMC0xNS05LjMtMjguNC0yMy40LTMzLjd6TTM1OCAyMTQuOGwtODUgMzEuOXYtNjguMmw4NS0zN3Y3My4zek0xNTQgMTA0LjFsMTAyLTM4LjIgMTAyIDM4LjJ2LjZsLTEwMiA0MS40LTEwMi00MS40di0uNnptODQgMjkxLjFsLTg1IDQyLjV2LTc5LjFsODUtMzguOHY3NS40em0wLTExMmwtMTAyIDQxLjQtMTAyLTQxLjR2LS42bDEwMi0zOC4yIDEwMiAzOC4ydi42em0yNDAgMTEybC04NSA0Mi41di03OS4xbDg1LTM4Ljh2NzUuNHptMC0xMTJsLTEwMiA0MS40LTEwMi00MS40di0uNmwxMDItMzguMiAxMDIgMzguMnYuNnoiPjwvcGF0aD48L3N2Zz4K)](https://docs.rs/range-set-blaze)

<!-- FUTURE: Add coverage badge? -->

Integer sets as fast, sorted integer ranges -- Maps with integer-range keys -- Full set operations

Supports all of Rust's integer-like types, [`u8`] to [`u128`], [`i8`] to [`i128`], `char` (Unicode characters), `IpvAddr`, and `Ipv6Addr`.
 The [set operations] and [map operations]
include `union`, `intersection`, `difference`, `symmetric difference`, and `complement`.

The crate's main structs are:

* [`RangeSetBlaze`], a set of integers. See the [set documentation] for details
* [`RangeMapBlaze`], a map from integers to values. See the [map documentation] for details

> Unlike the standard [`BTreeSet/BTreeMap`] and [`HashSet/HashMap`], `RangeSetBlaze` does not store every integer in the set. Rather, it stores sorted & disjoint ranges of integers in a cache-efficient [`BTreeMap`]. It differs from [other interval libraries](https://github.com/CarlKCarlK/range-set-blaze/blob/main/docs/bench.md) -- that we know of -- by
offering full set operations and by being optimized for sets of [clumpy][1] integers.
>
> We can construct a `RangeSetBlaze` or `RangeMapBlaze` from unsorted & redundant integers (or ranges). When the inputs are clumpy, construction will be [linear][1] in the number of inputs and set operations will be sped up [quadratically][1].

The crate's main traits are

* [`SortedDisjoint`], implemented by iterators of sorted & disjoint ranges of integers. See [documentation][2] for details.
* [`SortedDisjointMap`], implemented by iterators of pairs, where the first item is a sorted & disjoint range of integers. The second item
is a value. See [documentation][3] for details.

> With any `SortedDisjoint` or `SortedDisjointMap` iterator we can perform set operations in one pass through the ranges and with minimal (constant) memory.
The package enforces the "sorted & disjoint" constraint at compile time
(making invalid states unrepresentable).

[`RangeSetBlaze`]: https://docs.rs/range-set-blaze/latest/range_set_blaze/struct.RangeSetBlaze.html
[`SortedDisjoint`]: https://docs.rs/range-set-blaze/latest/range_set_blaze/trait.SortedDisjoint.html#table-of-contents
[`SortedDisjointMap`]: https://docs.rs/range-set-blaze/latest/range_set_blaze/trait.SortedDisjointMap.html#table-of-contents
[`u8`]: https://doc.rust-lang.org/std/primitive.u8.html
[`u128`]: https://doc.rust-lang.org/std/primitive.u128.html
[`i8`]: https://doc.rust-lang.org/std/primitive.i8.html
[`i128`]: https://doc.rust-lang.org/std/primitive.i128.html
[set documentation]: https://docs.rs/range-set-blaze/latest/range_set_blaze/struct.RangeSetBlaze.html
[map documentation]: https://docs.rs/range-set-blaze/latest/range_set_blaze/struct.RangeMapBlaze.html
[`BTreeSet/BTreeMap`]: https://doc.rust-lang.org/std/collections/struct.BTreeSet.html
[`HashSet/HashMap`]: https://doc.rust-lang.org/std/collections/struct.HashSet.html
[`BTreeMap`]: https://doc.rust-lang.org/std/collections/struct.BTreeMap.html
[set operations]: https://docs.rs/range-set-blaze/latest/range_set_blaze/struct.RangeSetBlaze.html#rangesetblaze-set-operations
[map operations]: https://docs.rs/range-set-blaze/latest/range_set_blaze/struct.RangeSetBlaze.html#rangesetblaze-map-operations
[1]: https://docs.rs/range-set-blaze/latest/range_set_blaze/struct.RangeSetBlaze.html#constructor-performance
[2]:(https://docs.rs/range-set-blaze/latest/range_set_blaze/trait.SortedDisjoint.html#table-of-contents)
[3]:(https://docs.rs/range-set-blaze/latest/range_set_blaze/trait.SortedDisjointMaps.html#table-of-contents)

The crate supports `no_std`, WASM, and embedded projects. For `no_std`, etc., Use the command:

```bash
cargo add range-set-blaze --features "alloc" --no-default-features
```

Benchmarks
-----------

See the [benchmarks](https://github.com/CarlKCarlK/range-set-blaze/blob/main/docs/bench.md) for performance comparisons with other range-related crates.

Generally, for many tasks involving clumpy integers and ranges, `RangeSetBlaze` is much faster than alternatives.

The benchmarks are in the `benches` directory. To run them, use `cargo bench`.

Articles
-----------

* [Nine Rules for Creating Fast, Safe, and Compatible Data Structures in Rust:
Lessons from RangeSetBlaze](https://medium.com/towards-data-science/nine-rules-for-creating-fast-safe-and-compatible-data-structures-in-rust-part-1-c0973092e0a3) in *Towards Data Science*. It provides a high-level overview of the crate and its design.

* [Nine Rules for Running Rust on the Web and on Embedded: Practical Lessons from Porting range-set-blaze to no_std and WASM](https://medium.com/towards-data-science/nine-rules-for-running-rust-on-the-web-and-on-embedded-94462ef249a2) in *Towards Data Science*. It covers porting to "no_std".

* [Check AI-Generated Code Perfectly and Automatically
My Experience Applying Kani’s Formal Verification to ChatGPT-Suggested Rust Code](https://medium.com/@carlmkadie/check-ai-generated-code-perfectly-and-automatically-d5b61acff741). Shows how to prove overflow safety.

* [Nine Rules to Formally Validate Rust Algorithms with Dafny](https://medium.com/towards-data-science/nine-rules-to-formally-validate-rust-algorithms-with-dafny-part-1-5cb8c8a0bb92) in *Towards Data Science*. It shows how to formally validate one of the crate's algorithms.

* [Nine Rules for SIMD Acceleration of your Rust Code:
General Lessons from Boosting Data Ingestion in the range-set-blaze Crate by 7x](https://medium.com/towards-data-science/nine-rules-for-simd-acceleration-of-your-rust-code-part-1-c16fe639ce21) in *Towards Data Science*

* *Also see:* [CHANGELOG](https://github.com/CarlKCarlK/range-set-blaze/blob/main/CHANGELOG.md)

Examples
-----------

**Example 1**: Set Operations
- - - - - -

Here we take the union (operator “|”) of two `RangeSetBlaze`'s:

![Example 1](https://raw.githubusercontent.com/CarlKCarlK/range-set-blaze/main/docs/rust_example1.png "Example 1")

```rust
use range_set_blaze::prelude::*;

 // a is the set of integers from 100 to 499 (inclusive) and 501 to 1000 (inclusive)
let a = RangeSetBlaze::from_iter([100..=499, 501..=999]);
 // b is the set of integers -20 and the range 400 to 599 (inclusive)
let b = RangeSetBlaze::from_iter([-20..=-20, 400..=599]);
// c is the union of a and b, namely -20 and 100 to 999 (inclusive)
let c = a | b;
assert_eq!(c, RangeSetBlaze::from_iter([-20..=-20, 100..=999]));
```

**Example 2**: Maps (and Network Addresses)
- - - - - -

In networking, suppose we want to simplify a routing table. Here [`RangeMapBlaze`] merges identical routes
-- if adjacent or overlapping. It also remove all overlaps (respecting priority) and sorts.
The result is a fast BTree from regions to the next hop.
Similar code can simplify font tables.

```rust
use range_set_blaze::prelude::*;
use std::net::Ipv4Addr;

// A routing table, sorted by priority
let routing = [
    // destination, prefix, next hop, interface
    ("10.0.1.8", 30, "10.1.1.0", "eth2"),
    ("10.0.1.12", 30, "10.1.1.0", "eth2"),
    ("10.0.1.7", 32, "10.1.1.0", "eth2"),
    ("10.0.0.0", 8, "10.3.4.2", "eth1"),
    ("0.0.0.0", 0, "152.10.0.0", "eth0"),
];

// Create a RangeMapBlaze from the routing table
let range_map = routing
    .iter()
    .map(|(dest, prefix_len, next_hop, interface)| {
        let dest: Ipv4Addr = dest.parse().unwrap();
        let next_hop: Ipv4Addr = next_hop.parse().unwrap();
        let mask = u32::MAX.checked_shr(*prefix_len).unwrap_or(0);
        let range_start = Ipv4Addr::from(u32::from(dest) & !mask);
        let range_end = Ipv4Addr::from(u32::from(dest) | mask);
        (range_start..=range_end, (next_hop, interface))
    })
    .collect::<RangeMapBlaze<_, _>>();

// Print the now disjoint, sorted ranges and their associated values
for (range, (next_hop, interface)) in range_map.range_values() {
    println!("{range:?} -> ({next_hop}, {interface})");
}

// Look up an address
assert_eq!(
    range_map.get(Ipv4Addr::new(10, 0, 1, 6)),
    Some(&(Ipv4Addr::new(10, 3, 4, 2), &"eth1"))
);
```

Output:

```text
0.0.0.0..=9.255.255.255 -> (152.10.0.0, eth0)
10.0.0.0..=10.0.1.6 -> (10.3.4.2, eth1)
10.0.1.7..=10.0.1.15 -> (10.1.1.0, eth2)
10.0.1.16..=10.255.255.255 -> (10.3.4.2, eth1)
11.0.0.0..=255.255.255.255 -> (152.10.0.0, eth0)
```

**Example 3**: Biology
- - - - - -

In biology, suppose we want to find the intron regions of a gene but we are given only the transcription region and the exon regions.

![Example 3](https://raw.githubusercontent.com/CarlKCarlK/range-set-blaze/main/docs/rust_example2.png "Example 3")

We create a `RangeSetBlaze` for the transcription region and a `RangeSetBlaze` for all the exon regions.
Then we take the difference between the transcription region and exon regions to find the intron regions.

```rust
use range_set_blaze::prelude::*;

let line = "chr15   29370   37380   29370,32358,36715   30817,32561,37380";

// split the line on white space
let mut iter = line.split_whitespace();
let chrom = iter.next().unwrap();

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
assert_eq!(exons, RangeSetBlaze::from_iter([29370..=30817, 32358..=32561, 36715..=37380]));

// Use 'set difference' to find the introns
let intron = trans - exons;
assert_eq!(intron, RangeSetBlaze::from_iter([30818..=32357, 32562..=36714]));
for range in intron.ranges() {
    let (start, end) = range.into_inner();
    println!("{chrom}\t{start}\t{end}");
}
```

cmk need doc for things (see "cmk doc")
