use alloc::collections::BinaryHeap;
#[cfg(test)]
use rand::{rngs::StdRng, Rng, SeedableRng};

use crate::{
    merge::KMerge, AssumeSortedStarts, CheckSortedDisjoint, Integer, Merge, SortedDisjoint,
    SortedDisjointMap, SortedStarts, SymDiffIterKMerge, SymDiffIterMerge,
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
    gather: Option<RangeInclusive<T>>,
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
        loop {
            let count = self.end_heap.len();
            let Some(next_range) = self.next_again.take().or_else(|| self.iter.next()) else {
                // The workspace is empty and next is empty, so return everything gathered.
                if count == 0 {
                    return self.gather.take();
                };

                // The workspace is not empty (but next is empty) is process the next chunk of the workspace.
                let end = self.end_heap.pop().unwrap().0;
                self.remove_same_end(end);
                let result = self.start_or_min_value..=end;
                if !self.end_heap.is_empty() {
                    self.start_or_min_value = end + T::one(); // The 'if' prevents overflow.
                }
                if let Some(result) = self.process(count % 2 == 1, result) {
                    return result;
                }
                continue;
            };

            // Next has the same start as the workspace, so add it to the workspace.
            // (or the workspace is empty, so add it to the workspace.)
            let (next_start, next_end) = next_range.into_inner();
            if count == 0 || self.start_or_min_value == next_start {
                self.start_or_min_value = next_start;
                self.end_heap.push(Reverse(next_end));
                continue;
            }

            // Next start inside the workspace's first chunk, so process up to next_start.
            let end = self.end_heap.peek().unwrap().0;
            if next_start <= end {
                let result = self.start_or_min_value..=next_start - T::one(); // cmk000 check for overflow
                self.start_or_min_value = next_start;
                self.end_heap.push(Reverse(next_end));
                if let Some(result) = self.process(count % 2 == 1, result) {
                    return result;
                }
                continue;
            }

            // Next start is after the workspaces end, but the workspace contains only one chuck,
            // so process the workspace and set the workspace to next.
            self.remove_same_end(end);
            let result = self.start_or_min_value..=end;
            if self.end_heap.is_empty() {
                self.start_or_min_value = next_start;
                self.end_heap.push(Reverse(next_end));
                if let Some(result) = self.process(count % 2 == 1, result) {
                    return result;
                }
                continue;
            }

            // Next start is after the workspaces end, and the workspace contains more than one chuck,
            // so process one chunk and then process next
            self.start_or_min_value = end + T::one(); // cmk000 check for overflow
            self.next_again = Some(next_start..=next_end);
            if let Some(result) = self.process(count % 2 == 1, result) {
                return result;
            }
            continue;
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
    fn process(
        &mut self,
        keep: bool,
        next: RangeInclusive<T>,
    ) -> Option<Option<RangeInclusive<T>>> {
        if !keep {
            return None;
        }
        let Some(gather) = self.gather.take() else {
            self.gather = Some(next);
            return None;
        };
        // If there is no "next" then return gather if it exists.

        // Take both next and gather apart.
        let (next_start, next_end) = next.into_inner();
        let (gather_start, gather_end) = gather.into_inner();

        // We can assume gather_end < next_start.
        debug_assert!(gather_end < next_start); // real assert

        // If they touch, set gather to the union and loop.
        if gather_end + T::one() == next_start {
            self.gather = Some(gather_start..=next_end);
            return None;
        }

        // Next is disjoint from gather, so return gather and set gather to next.
        self.gather = Some(next_start..=next_end);
        return Some(Some(gather_start..=gather_end));
    }

    /// Creates a new [`SymDiffIter`] from zero or more [`SortedDisjointMap`] iterators.
    /// See [`SymDiffIter`] for more details and examples.
    pub fn new(mut iter: I) -> Self {
        Self {
            iter,
            start_or_min_value: T::min_value(),
            end_heap: BinaryHeap::with_capacity(10),
            next_again: None,
            gather: None,
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

#[test]
fn set_random_symmetric_difference() {
    use crate::range_set_blaze::MultiwayRangeSetBlaze;
    use crate::CheckSortedDisjointMap;
    use crate::RangeSetBlaze;

    for seed in 0..20 {
        println!("seed: {seed}");
        let mut rng = StdRng::seed_from_u64(seed);

        let mut set0 = RangeSetBlaze::new();
        let mut set1 = RangeSetBlaze::new();

        for _ in 0..500 {
            let key = rng.gen_range(0..=255u8); // cmk0000 u8
            set0.insert(key);
            print!("l{key} ");
            let key = rng.gen_range(0..=255u8);
            set1.insert(key);
            print!("r{key} ");

            let symmetric_difference = SymDiffIter::new2(set0.ranges(), set1.ranges());

            // println!(
            //     "left ^ right = {}",
            //     SymDiffIter::new2(set0.ranges(), set1.ranges()).to_string()
            // );

            let map0 = CheckSortedDisjointMap::new(set0.ranges().map(|range| (range.clone(), &())))
                .into_range_map_blaze();
            let map1 = CheckSortedDisjointMap::new(set1.ranges().map(|range| (range.clone(), &())))
                .into_range_map_blaze();
            let mut expected_map = &map0 ^ &map1;

            println!();
            println!("set0: {set0}");
            println!("set1: {set1}");

            for range in symmetric_difference {
                // println!();
                // print!("removing ");
                for k in range {
                    let get0 = set0.get(k);
                    let get1 = set1.get(k);
                    match (get0, get1) {
                        (Some(_k0), Some(_k1)) => {
                            println!();
                            println!("left: {}", set0);
                            println!("right: {}", set1);
                            let s_d = SymDiffIter::new2(set0.ranges(), set1.ranges())
                                .into_range_set_blaze();
                            panic!("left ^ right = {s_d}");
                        }
                        (Some(_k0), None) => {}
                        (None, Some(_k1)) => {}
                        (None, None) => {
                            panic!("should not happen 1");
                        }
                    }
                    assert!(expected_map.remove(k).is_some());
                }
                // println!();
            }
            if !expected_map.is_empty() {
                println!();
                println!("left: {}", set0);
                println!("right: {}", set1);
                let s_d = SymDiffIter::new2(set0.ranges(), set1.ranges()).into_range_set_blaze();
                println!("left ^ right = {s_d}");
                panic!("expected_keys should be empty: {expected_map}");
            }
        }
    }
}

#[test]
fn set_sym_diff_repro1() {
    use crate::RangeSetBlaze;

    let l = RangeSetBlaze::from_iter([157..=158]);
    let r = RangeSetBlaze::from_iter([158..=158]);
    let iter = SymDiffIter::new2(l.ranges(), r.ranges());
    let v = iter.collect::<Vec<_>>();
    println!("{v:?}");
}

#[test]
fn sdi1() {
    let a = [157..=158, 158..=158].into_iter();
    let a = AssumeSortedStarts::new(a);
    let mut iter = SymDiffIter::new(a);
    assert_eq!(iter.next(), Some(157..=157));
    assert_eq!(iter.next(), None);

    let a = [0..=0, 0..=0, 0..=1, 2..=100].into_iter();
    let a = AssumeSortedStarts::new(a);
    let mut iter = SymDiffIter::new(a);
    assert_eq!(iter.next(), Some(0..=100));
    assert_eq!(iter.next(), None);

    let a = [0..=0, 0..=1, 2..=100].into_iter();
    let a = AssumeSortedStarts::new(a);
    let mut iter = SymDiffIter::new(a);
    assert_eq!(iter.next(), Some(1..=100));
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
        assert_eq!(iter.next(), Some(0..=1));
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
        assert_eq!(iter.next(), Some(0..=1));
        assert_eq!(iter.next(), None);

        let a = [0..=0, 1..=1].into_iter();
        let a = AssumeSortedStarts::new(a);
        let mut iter = SymDiffIter::new(a);
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
