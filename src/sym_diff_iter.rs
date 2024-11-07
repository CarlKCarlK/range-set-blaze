use crate::{
    merge::KMerge, BitXorKMerge, BitXorMerge, Integer, Merge, SortedDisjoint, SortedStarts,
};
use alloc::collections::BinaryHeap;
use core::{cmp::Reverse, iter::FusedIterator, ops::RangeInclusive};

/// The output of cmk
#[derive(Clone, Debug)]
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

impl<T, I> Iterator for SymDiffIter<T, I>
where
    T: Integer,
    I: SortedStarts<T>,
{
    type Item = RangeInclusive<T>;

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
                    self.start_or_min_value = end.add_one(); // The 'if' prevents overflow.
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
                let result = self.start_or_min_value..=next_start.sub_one();
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
            self.start_or_min_value = end.add_one();
            self.next_again = Some(next_start..=next_end);
            if let Some(result) = self.process(count % 2 == 1, result) {
                return result;
            }
            // continue;
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
    #[allow(clippy::option_option)]
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
        if gather_end.add_one() == next_start {
            self.gather = Some(gather_start..=next_end);
            return None;
        }

        // Next is disjoint from gather, so return gather and set gather to next.
        self.gather = Some(next_start..=next_end);
        Some(Some(gather_start..=gather_end))
    }

    /// Creates a new [`SymDiffIter`] from zero or more [`SortedDisjoint`] iterators.
    /// See [`SymDiffIter`] for more details and examples.
    #[inline]
    pub fn new(iter: I) -> Self {
        Self {
            iter,
            start_or_min_value: T::min_value(),
            end_heap: BinaryHeap::with_capacity(10),
            next_again: None,
            gather: None,
        }
    }
}

impl<T, L, R> BitXorMerge<T, L, R>
where
    T: Integer,
    L: SortedDisjoint<T>,
    R: SortedDisjoint<T>,
{
    // cmk fix the comment on the set size. It should say inputs are SortedStarts not SortedDisjoint.
    /// Creates a new [`SymDiffIter`] from zero or more [`SortedDisjoint`] iterators. See [`SymDiffIter`] for more details and examples.
    ///
    /// [SortedDisjoint]: crate::SortedDisjoint.html#table-of-contents
    #[inline]
    pub fn new2(left: L, right: R) -> Self {
        let iter = Merge::new(left, right);
        Self::new(iter)
    }
}

/// cmk doc
impl<T, J> BitXorKMerge<T, J>
where
    T: Integer,
    J: SortedDisjoint<T>,
{
    // cmk fix the comment on the set size. It should say inputs are SortedStarts not SortedDisjoint.
    /// Creates a new [`SymDiffIter`] from zero or more [`SortedDisjoint`] iterators. See [`SymDiffIter`] for more details and examples.
    #[inline]
    pub fn new_k<K>(k: K) -> Self
    where
        K: IntoIterator<Item = J>,
    {
        let iter = KMerge::new(k);
        Self::new(iter)
    }
}
