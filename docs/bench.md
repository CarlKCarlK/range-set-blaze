# Benchmarks for (some) Range-Related Rust Crates

## Range-Related Rust Crates

| Crate | # Downloads | Ranges | Element Type | Set Operations? | Internal | Maps, too? |
| --- | --- | --- | --- | --- | --- | --- |
[range-set-blaze](https://github.com/CarlKCarlK/range-set-blaze) | zero | Disjoint | Integer | Set Ops | BTreeMap | Only Sets |
[rangemap](https://crates.io/crates/rangemap) | 243,612 | Disjoint | Ord | No Set Ops | BTreeMap | Sets/Maps |
[iset](https://crates.io/crates/iset) | 128,824 | Overlapping | PartialOrd | No Set Ops | Red Black | Sets/Maps |
[theban_interval_tree](https://crates.io/crates/theban_interval_tree) |72,000 | Overlapping(?) | ? | No Set Ops | interval tree | Sets/Maps ||
[Range-collections](https://crates.io/crates/range-collections) | 44,974 | Disjoint | Ord | Set Ops | SmallVec | Sets/Maps |
[range-set](https://crates.io/crates/range-set) | 19,526 | Disjoint | Ord? | No Set Ops | SmallVec | Only Sets |
[sorted-iter](https://crates.io/crates/sorted-iter) | 19,183 | No | Ord | Set Ops | *n/a* | Sets/Maps |
[ranges](https://crates.io/crates/ranges) | 14,964 | Disjoint | 'Domain' | Set Ops | Vec | Only Sets |
[unbounded-interval-tree](https://crates.io/crates/unbounded-interval-tree) | 3,780 | Overlapping | Ord | No Set Ops | Red Black | Only Sets ||
[ranged_set](https://crates.io/crates/ranged_set) | 2,116 | Disjoint | Ord | No Set Ops | ? | Only Sets |
[btree-range-map](https://crates.io/crates/btree-range-map) |  1,897 | Yes | Measure | No Set Ops | BTreeMap | Only Sets |
[nonoverlapping_interval_tree](https://crates.io/crates/nonoverlapping_interval_tree)|  <1000 | Yes | ? | No Set Ops | BTreeMap | Only Sets | |
[segmap](https://crates.io/crates/segmap)|  <1000 | Yes | Ord | not tested | BTreeMap | Sets/Maps |
[interval-map](https://crates.io/crates/interval-map)|  <1000 | Yes | Ord | No Set Ops | sorted vec | Sets/Maps |
[range_bounds_map](https://crates.io/crates/range_bounds_map)|  <1000 | Yes | Ord | No Set Ops | ? | Sets/Maps |
[int_range_set](https://crates.io/crates/int_range_set)|  <1000 | Yes | u64 | No Set Ops | tinyvec | Only Sets |

> *The # of downloads as of March 2023*

## Benchmark Selection Criteria

I ended up evaluating:

* `BTreeSet`, `HashSet`, from the standard library
* `range_map`, the most popular crate that works with ranges in a tree
* `Range-collections` and `range-set`, the most popular crates that store ranges in a vector

These crates store disjoint ranges. I eliminated crates for overlapping ranges, a different data structure (`iset`, `theban_interval_tree`, and `unbounded-interval-tree`).

The disjoint ranges can be stored in a tree or a vector. With a tree, we expect inserts to be much faster than with a vector, O(log *n*) vs O(*n*). Benchmark `ingest_clumps_easy` below showed this to be true. Because I care about such inserts, after the benchmark, I remove vector-based crates from further consideration.

Finally, I looked for crates that supported set operations (for example, union, intersection, set difference). None of the remaining crates offered tested set operations. (The inspirational `sorted-iter` does, but it is designed to work on sorted values, not ranges, and so is not included.)

If I misunderstood any of the crates, please let me know. If you'd like to benchmark a crate, the benchmarking code is in the `benches` directory of this repository.

## Benchmark Results

These benchmarks that allow us to understand the `range-set-blaze::RangeSetBlaze` data structure and to compare it to similar data structures from other crates.

## Benchmark #1: 'worst': Worst case for RangeSetBlaze

* **Measure**: intake speed
* **Candidates**: `HashSet`, `BTreeSet`, `RangeSetBlaze`
* **Vary**: *n* from 1 to 10,000, number of random integers
* **Details**: Select *n* integers randomly and uniformly from the range 0..=999 (with replacement).

### 'worst' Results

`RangeSetBlaze` is consistently about 2.5 times slower than `HashSet`. On small sets, `BTreeSet` is competitive with `HashSet`, but gets almost as slow as `RangeSetBlaze` as the sets grow.

### 'worst' Conclusion

`HashSet`, not `RangeSetBlaze` is a good choice for sets of non-clumpy integers. However, `RangeSetBlaze` is not catastrophically bad; it is just 2.5 times worse.

![worst lines](https://carlkcarlk.github.io/range-set-blaze/criterion/worst/report/lines.svg "worst lines")

## Benchmark #2: 'ingest_clumps_base': Measure `RangeSetBlaze` on increasingly clumpy integers

* **Measure**: integer intake speed
* **Candidates**: `HashSet`, `BTreeSet`, `RangeSetBlaze`
* **Vary**: *average clump size* from 1 (no clumps) to 100K (ten big clumps)
* **Details**: We generate 1M integers with clumps. We ingest the integers one at a time.
Each clump has size chosen uniformly random from roughly 1 to double *average clump size*. (The integer clumps are random uniform, with-replacement, in a span from 0 to roughly 10M. The exact span is sized so that the union of the 1M integers will cover about 10% of the span. In other words, a given integer in the span will have a 10% chance of being one of the 1M integers generated.)

### 'ingest_clumps_base' Results

As before, with no clumps, `RangeSetBlaze` is more than 2.5 times slower than `HashSet`. Somewhere around clump size 3, `RangeSetBlaze` becomes the best performer. As the average clump size goes past 100, `RangeSetBlaze` is a steady 30 times faster than HashTable and 15 times faster than BTreeSet.

If we are allowed to input the clumps as ranges (instead of as individual integers), then when the average clump size is 1000 `RangeSetBlaze` is 700
times faster than `HashSet` and `BTreeSet`.

### ingest_clumps_base' Conclusion

Range-based methods such as `RangeSetBlaze` are a great choice for clumpy integers.
When the input is given as ranges, they are the only sensible choice.

![ingest_clumps_base](https://carlkcarlk.github.io/range-set-blaze/criterion/ingest_clumps_base/report/lines.svg "ingest_clumps_base")

## Benchmark #3: 'ingest_clumps_integers': Measure the `rangemap` crate on clumpy integers

* **Measure**: integer intake speed
* **Candidates**: `base` + rangemap,
* **Vary**: *average clump size* from 1 (no clumps) to 100K (ten big clumps)
* **Details**: As with `base`.

We give each crate the clumps as individual integers.

### 'ingest_clumps_integers' Results & Conclusion

`rangemap` is typically three times slower than `HashSet` and 75 times slower than `RangeSetBlaze`. However ...

`RangeSetBlaze` batches its integer input by noticing when consecutive integers fit in a clump. This batching is not implemented in `rangemap` but could easily be added to it or any other range-based crate.

![ingest_clumps_integers](https://carlkcarlk.github.io/range-set-blaze/criterion/ingest_clumps_integers/report/lines.svg "ingest_clumps_integers")

## Benchmark #4: 'ingest_clumps_ranges': Measure rangemap on ranges of clumpy integers

* **Measure**: range intake speed
* **Candidates**: RangeSetBlaze + rangemap
* **Vary**: *average clump size* from 1 (no clumps) to 100K (ten big clumps)
* **Details**: As with `base`.

We give each crate the clumps as ranges (instead of as individual integers).

### 'ingest_clumps_ranges' Results & Conclusion

Over most clump sizes, `RangeSetBlaze` is about 4 times faster than `rangemap`. However ...

`RangeSetBlaze` batches range inputs by sorting them and then merging adjacent ranges. This batching is not implemented in `rangemap` but could easily be added to it or any other range-based crate.

![ingest_clumps_ranges](https://carlkcarlk.github.io/range-set-blaze/criterion/ingest_clumps_ranges/report/lines.svg "ingest_clumps_ranges")

## Benchmark #5: 'ingest_clumps_easy': Measure various crates on (easier) ranges of clumpy integers

* **Measure**: range intake speed
* **Candidates**: Tree based (RangeSetBlaze rangemap), Vector based (`range_collections`, `range_set`)
* **Vary**: *average clump size* from 1 (100K ranges) to 10 (10K ranges)
* **Details**: We generate 100K integers with clumps (down from 1M)

We give each crate the clumps as ranges (instead of as individual integers).

### 'ingest_clumps_easy' Results & Conclusion

The fastest vector-based method is 14 times slower than the slowest tree-based method. It is 50 times slower than `RangeSetBlaze`. This is expected because vector-based methods are not designed for a large numbers of inserts.

![ingest_clumps_easy](https://carlkcarlk.github.io/range-set-blaze/criterion/ingest_clumps_easy/report/lines.svg "ingest_clumps_easy")

## Benchmark #6: 'union_two_sets': Union two sets of clumpy integers

* **Measure**: adding ranges to an existing set
* **Candidates**: RangeSetBlaze, rangemap
* **Vary**: Number of clumps in the second set, from 1 to about 90K.
* **Details**: We first create two clump iterators, each with the desired number clumps. Their integer span is 0..=99_999_999.
Each clump iterator is designed to cover about 10% of this span. We, next, turn these two iterators into two sets. The first set is made from 1000 clumps. Finally, we measure the time it takes to add the second set to the first set.

RangeSetBlaze unions with a hybrid algorithm. When adding a few ranges, it adds them one at a time. When adding many ranges, it
merges the two sets of ranges by iterating over them in sorted order.

### 'union_two_sets' Results

When adding one clump to the first set, RangeSetBlaze is about 30% faster than the other crate.

As the number-of-clumps-to-add grows, RangeSetBlaze automatically switches algorithms. This allows it to be 6 times faster than the one-at-a-time method.

### union_two_sets' Conclusion

Over the whole range of clumpiness, RangeSetBlaze is faster. Compared to non-hybrid methods, it is many times faster as the size of the second set grows.

![union_two_sets](https://carlkcarlk.github.io/range-set-blaze/criterion/union_two_sets/report/lines.svg "union_two_sets")

## Benchmark #7: 'every_op': Compare the set operations

* **Measure**: set operation speed
* **Candidates**: union, intersection, difference, symmetric_difference, complement
* **Vary**: number of ranges in the set, from 1 to about 50K.
* **Details**: We create two clump iterators, each with the desired number of clumps and a coverage of 0.5. Their span is 0..=99_999_999. We, next, turn these two iterators into two sets. Finally, we measure the time it takes to operate on the two sets.

### 'every_op' Results and Conclusion

Complement (which works on just once set) is twice as fast as union, intersection, and difference. Symmetric difference is 2.9 times slower.

![every_op](https://carlkcarlk.github.io/range-set-blaze/criterion/every_op/report/lines.svg "every_op")

## Benchmark #8: 'intersect_k_sets': Multiway vs 2-way intersection

* **Measure**: intersection speed
* **Candidates**: 2-at-a-time intersection multiway intersection (static and dynamic)
* **Vary**: number of sets, from 2 to 100.
* **Details**: We create *n* iterators. Each iterator generates 1,000 clumps. The iterators are designed such that the coverage of the final intersection is about 25%. The span of integers in the clumps is 0..=99_999_999. We, next, turn these *n* iterators into *n* sets. Finally, we measure the time it takes to operate on the sets.

### 'intersect_k_sets' Results and Conclusion

On two sets, all methods are similar but beyond that two-at-a-time gets slower and slower. For 100 sets, it must create about 100 intermediate sets and is about 14 times slower than multiway.

Dynamic multiway is not needed with `RangeSetBlaze` but is sometimes needed on `SortedDisjoint` iterators
(also available from the `range-set-blaze` crate). It is 5% to 10% slower than static multiway.

![intersect_k_sets](https://carlkcarlk.github.io/range-set-blaze/criterion/intersect_k_sets/report/lines.svg "intersect_k_sets")
