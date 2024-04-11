use alloc::collections::{btree_map::Range, BinaryHeap};

use crate::{
    merge::KMerge, AssumeSortedStarts, Integer, Merge, SortedDisjoint, SortedStarts,
    SymDiffIterKMerge, SymDiffIterMerge,
};
use core::{
    array,
    cmp::{max, Reverse},
    iter::FusedIterator,
    ops::RangeInclusive,
};

/// Turns any number of [`SortedDisjointMap`] iterators into a [`SortedDisjointMap`] iterator of their union,
/// i.e., all the integers in any input iterator, as sorted & disjoint ranges. Uses [`Merge`]
/// or [`KMerge`].
///
/// [`SortedDisjointMap`]: crate::SortedDisjointMap
/// [`Merge`]: crate::Merge
/// [`KMerge`]: crate::KMerge
///
/// # Examples
///
/// ```
/// use itertools::Itertools;
/// use range_set_blaze::{SymDiffIter, Merge, SortedDisjointMap, CheckSortedDisjoint};
///
/// let a = CheckSortedDisjoint::new(vec![1..=2, 5..=100].into_iter());
/// let b = CheckSortedDisjoint::from([2..=6]);
/// let union = SymDiffIter::new(Merge::new(a, b));
/// assert_eq!(union.to_string(), "1..=100");
///
/// // Or, equivalently:
/// let a = CheckSortedDisjoint::new(vec![1..=2, 5..=100].into_iter());
/// let b = CheckSortedDisjoint::from([2..=6]);
/// let union = a | b;
/// assert_eq!(union.to_string(), "1..=100")
/// ```
// cmk #[derive(Clone, Debug)]
#[must_use = "iterators are lazy and do nothing unless consumed"]
pub struct SymDiffIter<T, I>
where
    T: Integer,
    I: SortedStarts<T>,
{
    iter: I,
    start_or_min_value: T,
    end_heap: BinaryHeap<Reverse<T>>,
    next_again: Option<RangeInclusive<T>>,
}

impl<T, I> FusedIterator for SymDiffIter<T, I>
where
    T: Integer,
    I: SortedStarts<T>,
{
}

// cmk0000 review this for simplifications
impl<T, I> Iterator for SymDiffIter<T, I>
where
    T: Integer,
    I: SortedStarts<T>,
{
    type Item = RangeInclusive<T>;
    // cmk0 does this and UnionIter do the right thing on non-fused input?

    fn next(&mut self) -> Option<RangeInclusive<T>> {
        'read_fresh: loop {
            let Some(next_range) = self.next_again.take().or_else(|| self.iter.next()) else {
                loop {
                    let count = self.end_heap.len();
                    if count == 0 {
                        return None;
                    };
                    let end = self.end_heap.pop().unwrap().0;
                    self.remove_same_end(end);
                    if self.end_heap.is_empty() {
                        if count % 2 == 0 {
                            return None;
                        } else {
                            return Some(self.start_or_min_value..=end);
                        }
                    }
                    if count % 2 == 1 {
                        let result = Some(self.start_or_min_value..=end);
                        self.start_or_min_value = end + T::one(); // cmk000 check for overflow
                        return result;
                    }
                    self.start_or_min_value = end + T::one(); // cmk000 check for overflow
                }
            };

            let (next_start, next_end) = next_range.into_inner();
            if self.end_heap.is_empty() {
                self.start_or_min_value = next_start;
                self.end_heap.push(Reverse(next_end));
                continue 'read_fresh;
            }

            if self.start_or_min_value != next_start {
                'process_again: loop {
                    let count = self.end_heap.len();
                    let end = self.end_heap.pop().unwrap().0;
                    self.remove_same_end(end);
                    if self.end_heap.is_empty() {
                        if count % 2 == 1 {
                            let result = Some(self.start_or_min_value..=end);
                            self.start_or_min_value = next_start;
                            self.end_heap.push(Reverse(next_end));
                            return result;
                        } else {
                            self.start_or_min_value = next_start;
                            self.end_heap.push(Reverse(next_end));
                            continue 'read_fresh;
                        }
                    }
                    if count % 2 == 1 {
                        let result = Some(self.start_or_min_value..=end);
                        self.start_or_min_value = end + T::one(); // cmk000 check for overflow
                        self.next_again = Some(next_start..=next_end);
                        return result;
                    } else {
                        self.start_or_min_value = end + T::one(); // cmk000 check for overflow
                        continue 'process_again;
                    }
                }
            }

            self.end_heap.push(Reverse(next_end));
            continue 'read_fresh;
        }
    }
}

impl<T, I> SymDiffIter<T, I>
where
    T: Integer,
    I: SortedStarts<T>,
{
    #[inline]
    fn remove_same_end(&mut self, end: T) {
        while let Some(end2) = self.end_heap.peek() {
            if end2.0 == end {
                self.end_heap.pop();
            } else {
                break;
            }
        }
    }

    #[inline]
    fn dump_some(&mut self, dump_to: T) -> RangeInclusive<T> {
        todo!()
        // debug_assert!(!self.end_heap.is_empty()); // real assert
        // debug_assert!(self.start_or_min_value < dump_to); // real assert

        // // Count how many items have the same value as the top item.
        // let end = self.end_heap.pop().unwrap().0;
        // let mut count = self.count_same_end(end);

        // if end < dump_to {
        //     let result = self.start_or_min_value..=end;
        //     self.start_or_min_value = end + T::one();
        //     if !self.end_heap.is_empty() {
        //     self.dump_to = Some(dump_to);
        //     }
        //     return result;
        // } else {
        //     let result = self.start_or_min_value..=dump_to - T::one();
        //     self.start_or_min_value = ;
        // }
    }

    #[inline]
    fn dump_all(&mut self) -> Option<RangeInclusive<T>> {
        todo!()
        // loop {
        //     // If the workspace is empty, return None.
        //     let Some(reverse_end) = self.end_heap.pop() else {
        //         return None;
        //     };

        //     // Count how many items have the same value as the top item.
        //     let end = reverse_end.0;
        //     self.remove_same_end(end);

        //     // Find the possible result
        //     let result = self.start_or_min_value..=end;

        //     // move up the start (but avoid overflow)
        //     if end < T::safe_max_value() {
        //         self.start_or_min_value = end + T::one();
        //     } else {
        //         debug_assert!(self.end_heap.is_empty()); // real assert
        //     }

        //     // If the count is odd, return the result, otherwise loop.
        //     if count % 2 == 1 {
        //         return Some(result);
        //     }
        // }
    }
    // cmk could split this into two functions
    #[inline]
    fn dump(&mut self, iter_start: Option<T>) {
        todo!()
        // // Count how many items have the same value as the top item.
        // let mut count = 1usize;
        // let end = self.end_heap.pop().unwrap().0;
        // while let Some(end2) = self.end_heap.peek() {
        //     if end2.0 == end {
        //         self.end_heap.pop();
        //         count += 1;
        //     } else {
        //         break;
        //     }
        // }

        // let end_end = match (iter_start) {
        //     Some(iter_start) => {
        //         if end <= iter_start {
        //             self.keep_it(self.start_or_min_value..=end);
        //             return;
        //         }
        //         end
        //     }
        //     None => end,
        // };
        // let Some(iter_start) = iter_start else {
        //     self.keep_it(self.start_or_min_value..=end);
        //     return;
        // };

        // if count % 2 == 1 {
        //     self.keep_it(self.start_or_min_value..=end);
        // }
    }

    #[inline]
    fn keep_it(&mut self, keeper: RangeInclusive<T>) {}

    /// Creates a new [`SymDiffIter`] from zero or more [`SortedDisjointMap`] iterators.
    /// See [`SymDiffIter`] for more details and examples.
    pub fn new(mut iter: I) -> Self {
        Self {
            iter,
            start_or_min_value: T::min_value(),
            end_heap: BinaryHeap::new(),
            next_again: None,
        }
    }
}

impl<T, L, R> SymDiffIterMerge<T, L, R>
where
    T: Integer,
    L: SortedDisjoint<T>,
    R: SortedDisjoint<T>,
{
    // cmk fix the comment on the set size. It should say inputs are SortedStarts not SortedDisjoint.
    /// Creates a new [`SymDiffIter`] from zero or more [`SortedDisjointMap`] iterators. See [`SymDiffIter`] for more details and examples.
    pub fn new2(left: L, right: R) -> Self {
        let iter = Merge::new(left, right);
        Self::new(iter)
    }
}

/// cmk doc
impl<T, J> SymDiffIterKMerge<T, J>
where
    T: Integer,
    J: SortedDisjoint<T>,
{
    // cmk fix the comment on the set size. It should say inputs are SortedStarts not SortedDisjoint.
    /// Creates a new [`SymDiffIter`] from zero or more [`SortedDisjointMap`] iterators. See [`SymDiffIter`] for more details and examples.
    pub fn new_k<K>(k: K) -> Self
    where
        K: IntoIterator<Item = J>,
    {
        let iter = KMerge::new(k);
        Self::new(iter)
    }
}

pub struct SymDiffIter2<T, I>
where
    T: Integer,
    I: Iterator<Item = RangeInclusive<T>> + FusedIterator,
{
    iter: I,
    gather: Option<RangeInclusive<T>>,
}

impl<T, I> FusedIterator for SymDiffIter2<T, I>
where
    T: Integer,
    I: Iterator<Item = RangeInclusive<T>> + FusedIterator,
{
}

// cmk0000 review this for simplifications
impl<T, I> Iterator for SymDiffIter2<T, I>
where
    T: Integer,
    I: Iterator<Item = RangeInclusive<T>> + FusedIterator,
{
    type Item = RangeInclusive<T>;
    // cmk0 does this and UnionIter do the right thing on non-fused input?

    #[inline]
    fn next(&mut self) -> Option<RangeInclusive<T>> {
        loop {
            // If there is no "next" then return gather if it exists.
            let Some(next) = self.iter.next() else {
                return self.gather.take();
            };

            // If there is no gather, set it to next and loop.
            let Some(gather) = self.gather.take() else {
                self.gather = Some(next);
                continue;
            };

            // Take both next and gather apart.
            let (next_start, next_end) = next.into_inner();
            let (gather_start, gather_end) = gather.into_inner();

            // If only used with SymDiffIter2, we can assume gather_end < next_start.
            debug_assert!(gather_end < next_start); // real assert

            // If they touch, set gather to the union and loop.
            if gather_end + T::one() == next_start {
                self.gather = Some(gather_start..=next_end);
                continue;
            }

            // Next is disjoint from gather, so return gather and set gather to next.
            self.gather = Some(next_start..=next_end);
            return Some(gather_start..=gather_end);
        }
    }
}

impl<T, I> SymDiffIter2<T, I>
where
    T: Integer,
    I: Iterator<Item = RangeInclusive<T>> + FusedIterator,
{
    /// Creates a new [`SymDiffIter`] from zero or more [`SortedDisjointMap`] iterators.
    /// See [`SymDiffIter`] for more details and examples.
    pub fn new(mut iter: I) -> Self {
        Self { iter, gather: None }
    }
}

#[test]
fn sdi2() {
    let a = [0..=0].into_iter();
    let iter = SymDiffIter2::new(a);
    let v = iter.collect::<Vec<_>>();
    assert_eq!(v, vec![0..=0]);

    let a = [0..=0, 2..=100].into_iter();
    let iter = SymDiffIter2::new(a);
    let v = iter.collect::<Vec<_>>();
    assert_eq!(v, vec![0..=0, 2..=100]);

    let a = [0..=0, 1..=100].into_iter();
    let iter = SymDiffIter2::new(a);
    let v = iter.collect::<Vec<_>>();
    assert_eq!(v, vec![0..=100]);

    // this should debug fail
    // let mut a = [0..=0, 0..=100].into_iter();
    // let iter = SymDiffIter2::new(a);
    // let v = iter.collect::<Vec<_>>();
    // assert_eq!(v, vec![0..=100]);
}

#[test]
fn sdi1() {
    let a = [0..=0, 0..=0, 0..=1, 2..=100].into_iter();
    let a = AssumeSortedStarts::new(a);
    let mut iter = SymDiffIter::new(a);
    assert_eq!(iter.next(), Some(0..=0));
    assert_eq!(iter.next(), Some(1..=1));
    assert_eq!(iter.next(), Some(2..=100));
    assert_eq!(iter.next(), None);

    let a = [0..=0, 0..=1, 2..=100].into_iter();
    let a = AssumeSortedStarts::new(a);
    let mut iter = SymDiffIter::new(a);
    assert_eq!(iter.next(), Some(1..=1));
    assert_eq!(iter.next(), Some(2..=100));
    assert_eq!(iter.next(), None);

    let a = [0..=0, 0..=0, 2..=100].into_iter();
    let a = AssumeSortedStarts::new(a);
    let mut iter = SymDiffIter::new(a);
    assert_eq!(iter.next(), Some(2..=100));
    assert_eq!(iter.next(), None);

    let a = [0..=0, 0..=0, 0..=0, 2..=100].into_iter();
    let a = AssumeSortedStarts::new(a);
    let mut iter = SymDiffIter::new(a);
    assert_eq!(iter.next(), Some(0..=0));
    assert_eq!(iter.next(), Some(2..=100));
    assert_eq!(iter.next(), None);
    {
        let a = [0..=1, 0..=0].into_iter();
        let a = AssumeSortedStarts::new(a);
        let mut iter = SymDiffIter::new(a);
        assert_eq!(iter.next(), Some(1..=1));
        assert_eq!(iter.next(), None);

        let a = [0..=1, 0..=0, 0..=0].into_iter();
        let a = AssumeSortedStarts::new(a);
        let mut iter = SymDiffIter::new(a);
        assert_eq!(iter.next(), Some(0..=0));
        assert_eq!(iter.next(), Some(1..=1));
        assert_eq!(iter.next(), None);

        let a = [0..=0, 0..=0, 0..=0].into_iter();
        let a = AssumeSortedStarts::new(a);
        let mut iter = SymDiffIter::new(a);
        assert_eq!(iter.next(), Some(0..=0));
        assert_eq!(iter.next(), None);

        let a = [0..=0, 0..=0].into_iter();
        let a = AssumeSortedStarts::new(a);
        let mut iter = SymDiffIter::new(a);
        assert_eq!(iter.next(), None);

        let a = [0..=0, 1..=1].into_iter();
        let a = AssumeSortedStarts::new(a);
        let mut iter = SymDiffIter::new(a);
        assert_eq!(iter.next(), Some(0..=0));
        assert_eq!(iter.next(), Some(1..=1));
        assert_eq!(iter.next(), None);

        let a = [0..=0, 1..=1].into_iter();
        let a = AssumeSortedStarts::new(a);
        let iter = SymDiffIter::new(a);
        let mut iter = SymDiffIter2::new(iter);
        assert_eq!(iter.next(), Some(0..=1));
        assert_eq!(iter.next(), None);

        let a = [0..=0, 2..=2].into_iter();
        let a = AssumeSortedStarts::new(a);
        let mut iter = SymDiffIter::new(a);
        assert_eq!(iter.next(), Some(0..=0));
        assert_eq!(iter.next(), Some(2..=2));
        assert_eq!(iter.next(), None);

        let a = [0..=0].into_iter();
        let a = AssumeSortedStarts::new(a);
        let mut iter = SymDiffIter::new(a);
        assert_eq!(iter.next(), Some(0..=0));
        assert_eq!(iter.next(), None);

        let a: array::IntoIter<RangeInclusive<i32>, 0> = [].into_iter();
        let a = AssumeSortedStarts::new(a);
        let iter = SymDiffIter::new(a);
        let v = iter.collect::<Vec<_>>();
        assert_eq!(v, vec![]);
    }
}
