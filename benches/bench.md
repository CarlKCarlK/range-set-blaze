# Benchmarks

## 'worst': Worst case for RangeSetInt

* **Measure**: intake speed
* **Candidates**: HashSet, BTreeSet, RangeSetInt
* **Vary**: *n* from 1 to 10,000, number of random integers
* **Details**: Select *n* random integers randomly and uniformly from the range 0..=999 (with replacement).

### Results

RangeSetInt is consistently 2.5 times slower than HashSet. On small sets, BTreeSet is competitive with HashSet, but gets almost as slow as RangeSetInt as the sets grow.

### Conclusion

RangeSetInt is not a good choice non-clumpy integers. However, it is not catastrophically worse. It is just 2.5 times worse.

![worst lines](https://raw.githubusercontent.com/fastlmm/PySnpTools/master/doc/source/lines.svg "worst lines")

## 'vs_btree_set': Measure RangeSetInt on increasing clumpy integers
<!-- cmk000 rename case  -->

* **Measure**: intake speed
* **Candidates**: HashSet, BTreeSet, RangeSetInt
* **Vary**: *average clump size* from 1 (no clumps) to 1M (one big clump)
* **Details**: We generate 1M integers in clumps. Each clump has size chosen uniformly random from roughly 1 to double *average clump size*. The clumps are random uniform in a range of roughly 1 to 10M. The exact range is sized so that the union of the 1M integers will cover 10% of the range.

### 'vs_btree_set' Results

With no clumps, RangeSetInt is about 2.6 times slower than HashSet. As the average clump size reaches 3, it becomes the best performer. As the average clump size goes past 100, it is a steady 30 times faster than HashTable and 10 times faster than BTreeSet.

If we are allowed to input the clumps as ranges (instead of as individual integers), then RangeSetInt is 200 times faster than HashTable and 60 times faster than BTreeSet.

### vs_btree_set' Conclusion

RangeSetInt is a great choice for clumpy integers.

![vs_btree_set](../target/criterion/vs_btree_set/report/lines.svg "vs_btree_set")

## 'stream_vs_ad_hoc': Compare 'union' vs 'insert'
<!-- cmk000 rename case  -->

* **Measure**: intake/union speed
* **Candidates**: RangeSetInt::union, RangeSetInt::insert
* **Vary**: number of clumps in the set, from 1 to 100K.
* **Details**: We first create two clump iterators, each with the desired number clumps and a coverage of 10%. Their range is 0..=99_999_999.
We, next, turn these two iterators into two sets. Finally, we measure the time it takes to union the two sets.

We union two different ways: one-at-a-time (using RangeSetInt::insert) vs all-at-once (using RangeSetInt::union).

### 'stream_vs_ad_hoc' Results

All-at-once goes from being about 5 times slower to being 5 times faster. The crossover point is around 200 clumps.
(Not shown, but for coverage of 50%, the crossover point is around 10 clumps.)

### stream_vs_ad_hoc' Conclusion

For sets with many ranges, use RangeSetInt::union. For sets with few ranges, use RangeSetInt::insert.

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
