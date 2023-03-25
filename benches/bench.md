# Benchmarks

## 'worst': Worst case for RangeSetBlaze

* **Measure**: intake speed
* **Candidates**: HashSet, BTreeSet, RangeSetBlaze
* **Vary**: *n* from 1 to 10,000, number of random integers
* **Details**: Select *n* random integers randomly and uniformly from the range 0..=999 (with replacement).

### Results

RangeSetBlaze is consistently 2.5 times slower than HashSet. On small sets, BTreeSet is competitive with HashSet, but gets almost as slow as RangeSetBlaze as the sets grow.

### Conclusion

RangeSetBlaze is not a good choice non-clumpy integers. However, it is not catastrophically worse. It is just 2.5 times worse.

![worst lines](https://raw.githubusercontent.com/fastlmm/PySnpTools/master/doc/source/lines.svg "worst lines")

## 'ingest_clumps_base': Measure RangeSetBlaze on increasing clumpy integers

* **Measure**: intake speed
* **Candidates**: HashSet, BTreeSet, RangeSetBlaze
* **Vary**: *average clump size* from 1 (no clumps) to 1M (one big clump)
* **Details**: We generate 1M integers in clumps. Each clump has size chosen uniformly random from roughly 1 to double *average clump size*. The clumps are random uniform in a range of roughly 1 to 10M. The exact range is sized so that the union of the 1M integers will cover 10% of the range.

### 'ingest_clumps_base' Results

With no clumps, RangeSetBlaze is about 2.6 times slower than HashSet. As the average clump size reaches 3, it becomes the best performer. As the average clump size goes past 100, it is a steady 30 times faster than HashTable and 10 times faster than BTreeSet.

If we are allowed to input the clumps as ranges (instead of as individual integers), then RangeSetBlaze is 200 times faster than HashTable and 60 times faster than BTreeSet.

### ingest_clumps_base' Conclusion

RangeSetBlaze is a great choice for clumpy integers.

![ingest_clumps_base](../target/criterion/ingest_clumps_base/report/lines.svg "ingest_clumps_base")

## 'ingest_clumps_ranges': Measure Various range set crates on the clumpy integers

* **Measure**: intake speed
* **Candidates**: RangeSetBlaze, rangemap
* **Vary**: *average clump size* from 100 to 1000 (a subset of the 'ingest_clumps_base' case)
* **Details**: We generate 1M integers in clumps. Each clump has size chosen uniformly random from roughly 1 to double *average clump size*. The clumps are random uniform in a range of roughly 1 to 10M. The exact range is sized so that the union of the 1M integers will cover 10% of the range.

We give each crate the clumps as ranges (instead of as individual integers).

### 'ingest_clumps_ranges' Results & Conclusion

RangeSetBlaze is the only crate that batches its input. This lets it ingest ranges 2 to 4 times faster than the other crates. (The other crates could add batching.)

![ingest_clumps_ranges](../target/criterion/ingest_clumps_ranges/report/lines.svg "ingest_clumps_ranges")

## 'ingest_clumps_integers': Measure Various range set crates on the clumpy integers

* **Measure**: intake speed
* **Candidates**: RangeSetBlaze, rangemap, BTreeSet, HashSet
* **Vary**: *average clump size* from 100 to 1000 (a subset of the 'ingest_clumps_base' case)
* **Details**: We generate 1M integers in clumps. Each clump has size chosen uniformly random from roughly 1 to double *average clump size*. The clumps are random uniform in a range of roughly 1 to 10M. The exact range is sized so that the union of the 1M integers will cover 10% of the range.

We give each crate the clumps as individual integers.

### 'ingest_clumps_integers' Results & Conclusion

Over this range of clumpiness, RangeSetBlaze is 11 times faster than BTreeSet and HashSet and about 100 times faster than the other range creates.

Again, we can attribute this speedup to RangeSetBlaze's input batching, which the other crates could add.

![ingest_clumps_integers](../target/criterion/ingest_clumps_integers/report/lines.svg "ingest_clumps_integers")

## 'union_two_sets': Compare 'union' vs 'insert'

* **Measure**: adding ranges to an existing set
* **Candidates**: RangeSetBlaze::BitOrAssign, rangemap extend
* **Vary**: number of clumps in the second set, from 1 to 100K.
* **Details**: We first create two clump iterators, each with the desired number clumps and a coverage of 10%. Their range is 0..=99_999_999.
We, next, turn these two iterators into two sets. The first set is made from 1000 clumps. Finally, we measure the
time it takes to add the second set to the first set.

RangeSetBlaze uses a hybrid approach. When adding a few clumps, it adds them one at a time. When adding many clumps, it unions the two sets all at once.

### 'union_two_sets' Results

When adding one clump to the first set, RangeSetBlaze is about 30% faster than the other crates. The one-at-a-time methods are about 4 times faster than than the all-at-once method.

As the number-of-clumps-to-add grows, RangeSetBlaze automatically switches from one-at-a-time to all-at-once. This allows it to be 6 times faster than the one-at-a-time methods.

### union_two_sets' Conclusion

Over the whole range of clumpiness, RangeSetBlaze is faster. Compared to non-hybrid methods it is many times faster at the extremes.

![stream_vs_adhoc](../target/criterion/stream_vs_adhoc/report/lines.svg "stream_vs_adhoc")

## 'every_op': Compare the set operations

* **Measure**: set operation speed
* **Candidates**: union, intersection, difference, symmetric_difference, complement
* **Vary**: number of ranges in the set, from 1 to 100K.
* **Details**: We create two clump iterators, each with the desired number of clumps and a coverage of 0.5. Their range is 0..=99_999_999. We, next, turn these two iterators into two sets. Finally, we measure the time it takes to operate on the two sets.

### 'every_op' Results and Conclusion

Complement (which works on just once set) is twice as fast as union, intersection, and difference. Symmetric difference is 2.9 times slower.

![every_op](../target/criterion/every_op/report/lines.svg "every_op")

## 'intersection_vary_k_w_2_at_a_time': Multiway vs 2-way intersection

* **Measure**: intersection speed
* **Candidates**: multiway intersection, 2-way intersection
* **Vary**: number of sets, from 2 to 100.
* **Details**: We create *n* clump iterators, each with 1,000 clumps. The iterators are designed such that the coverage of the final intersection is about 25%. The range of integers in the clumps is 0..=99_999_999. We, next, turn these *n* iterators into *n* sets. Finally, we measure the time it takes to operate on the sets.

### 'intersection_vary_k_w_2_at_a_time' Results and Conclusion

2-way is 16% fast on two items but beyond that it gets slower and slower. For 100 sets, it must create about 100 intermediate sets and is about 100 times slower than multiway.

Not shown, when the final coverage is a sparse 1%, 2-way is 2.75 times slower than multiway.

Dynamic multiway is usually a percent or two slower than static multiway.

![intersection_vary_k_w_2_at_a_time](../target/criterion/intersection_vary_k_w_2_at_a_time/report/lines.svg "intersection_vary_k_w_2_at_a_time")
