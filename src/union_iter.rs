use crate::map::SortedStartsInVec;
use crate::merge::KMerge;
use crate::unsorted_disjoint::UnsortedDisjoint;
use crate::{AssumeSortedStarts, Merge, SortedDisjoint, SortedStarts, UnionKMerge};
use crate::{Integer, UnionMerge};
use core::cmp::max;
use core::iter::FusedIterator;
use core::ops::RangeInclusive;
use itertools::Itertools;

/// This `struct` is created by the [`union`] method on [`SortedDisjoint`]. See [`union`]'s
/// documentation for more.
///
/// [`SortedDisjoint`]: crate::SortedDisjoint
/// [`union`]: crate::SortedDisjoint::union
#[derive(Clone, Debug)]
#[must_use = "iterators are lazy and do nothing unless consumed"]
pub struct UnionIter<T, SS>
where
    T: Integer,
    SS: SortedStarts<T>,
{
    iter: SS,
    option_range: Option<RangeInclusive<T>>,
}

impl<T, I> Iterator for UnionIter<T, I>
where
    T: Integer,
    I: SortedStarts<T>,
{
    type Item = RangeInclusive<T>;

    fn next(&mut self) -> Option<RangeInclusive<T>> {
        loop {
            let Some(range) = self.iter.next() else {
                return self.option_range.take();
            };

            let (start, end) = range.into_inner();
            debug_assert!(start <= end); // real assert

            let Some(current_range) = self.option_range.take() else {
                self.option_range = Some(start..=end);
                continue;
            };

            let (current_start, current_end) = current_range.into_inner();
            debug_assert!(current_start <= start); // real assert
            if start <= current_end
                || (current_end < T::max_value() && start <= current_end.add_one())
            {
                self.option_range = Some(current_start..=max(current_end, end));
                continue;
            }

            self.option_range = Some(start..=end);
            return Some(current_start..=current_end);
        }
    }
}

impl<T, I> UnionIter<T, I>
where
    T: Integer,
    I: SortedStarts<T>,
{
    // cmk fix the comment on the set size. It should say inputs are SortedStarts not SortedDisjoint.
    /// Creates a new [`UnionIter`] from zero or more [`SortedStarts`] iterators. See [`UnionIter`] for more details and examples.
    pub(crate) const fn new(iter: I) -> Self {
        Self {
            iter,
            option_range: None,
        }
    }
}

impl<T, L, R> UnionMerge<T, L, R>
where
    T: Integer,
    L: SortedDisjoint<T>,
    R: SortedDisjoint<T>,
{
    #[inline]
    pub(crate) fn new2(left: L, right: R) -> Self {
        let iter: Merge<T, L, R> = Merge::new(left, right);
        Self::new(iter)
    }
}

impl<T, J> UnionKMerge<T, J>
where
    T: Integer,
    J: SortedDisjoint<T>,
{
    #[inline]
    pub(crate) fn new_k<K>(k: K) -> Self
    where
        K: IntoIterator<Item = J>,
    {
        let iter = KMerge::new(k);
        Self::new(iter)
    }
}

// cmk simplify the long types
// from iter (T, VR) to UnionIter
impl<T> FromIterator<RangeInclusive<T>> for UnionIter<T, SortedStartsInVec<T>>
where
    T: Integer,
{
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = RangeInclusive<T>>,
    {
        let iter = iter.into_iter();
        let iter = UnsortedDisjoint::new(iter);
        let iter = iter.sorted_by(|a, b| a.start().cmp(b.start()));
        let iter = AssumeSortedStarts::new(iter);
        Self::new(iter)
    }
}

impl<T, I> FusedIterator for UnionIter<T, I>
where
    T: Integer,
    I: SortedStarts<T> + FusedIterator,
{
}
