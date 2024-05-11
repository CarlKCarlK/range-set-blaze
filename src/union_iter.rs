use crate::map::SortedStartsInVec;
use crate::merge::KMerge;
use crate::unsorted_disjoint::UnsortedDisjoint;
use crate::{AssumeSortedStarts, BitOrKMerge, Merge, SortedDisjoint, SortedStarts};
use crate::{BitOrMerge, Integer};
use core::cmp::max;
use core::iter::FusedIterator;
use core::ops::RangeInclusive;
use itertools::Itertools;

/// Turns any number of [`SortedDisjoint`] iterators into a [`SortedDisjoint`] iterator of their union,
/// i.e., all the integers in any input iterator, as sorted & disjoint ranges. Uses [`Merge`]
/// or [`KMerge`].
///
/// [`SortedDisjoint`]: crate::SortedDisjoint
/// [`Merge`]: crate::merge::Merge
/// [`KMerge`]: crate::merge::KMerge
///
/// # Examples
///
/// ```
/// use itertools::Itertools;
/// use range_set_blaze::{prelude::*,UnionIter};
///
/// let a = CheckSortedDisjoint::new([1..=2, 5..=100]);
/// let b = CheckSortedDisjoint::new([2..=6]);
/// let union = UnionIter::new2(a, b);
/// assert_eq!(union.into_string(), "1..=100");
///
/// // Or, equivalently:
/// let a = CheckSortedDisjoint::new([1..=2, 5..=100]);
/// let b = CheckSortedDisjoint::new([2..=6]);
/// let union = a | b;
/// assert_eq!(union.into_string(), "1..=100")
/// ```
// cmk #[derive(Clone, Debug)]
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
            if end < start {
                continue;
            }

            let Some(current_range) = self.option_range.take() else {
                self.option_range = Some(start..=end);
                continue;
            };

            let (current_start, current_end) = current_range.into_inner();
            debug_assert!(current_start <= start); // real assert
            if start <= current_end
                || (current_end < T::max_value() && start <= current_end + T::one())
            {
                self.option_range = Some(current_start..=max(current_end, end));
                continue;
            }

            self.option_range = Some(start..=end);
            return Some(current_start..=current_end);
        }
    }
}

// #[allow(dead_code)]
// fn cmk_debug_string<'a, T>(item: &Option<RangeInclusive<T>>) -> String
// where
//     T: Integer,
// {
//     if let Some(item) = item {
//         format!("Some({:?})", item.0)
//     } else {
//         "None".to_string()
//     }
// }

impl<T, I> UnionIter<T, I>
where
    T: Integer,
    I: SortedStarts<T>,
{
    // cmk fix the comment on the set size. It should say inputs are SortedStarts not SortedDisjoint.
    /// Creates a new [`UnionIter`] from zero or more [`SortedStarts`] iterators. See [`UnionIter`] for more details and examples.
    pub fn new(iter: I) -> Self {
        Self {
            iter,
            option_range: None,
        }
    }
}

impl<T, L, R> BitOrMerge<T, L, R>
where
    T: Integer,
    L: SortedDisjoint<T>,
    R: SortedDisjoint<T>,
{
    // cmk fix the comment on the set size. It should say inputs are SortedStarts not SortedDisjoint.
    /// Creates a new [`crate::sym_diff_iter_map::SymDiffIter`] from zero or more [`SortedDisjoint`] iterators. See [`crate::sym_diff_iter_map::SymDiffIter`] for more details and examples.
    pub fn new2(left: L, right: R) -> Self {
        let iter: Merge<T, L, R> = Merge::new(left, right);
        Self::new(iter)
    }
}

/// cmk doc
impl<T, J> BitOrKMerge<T, J>
where
    T: Integer,
    J: SortedDisjoint<T>,
{
    // cmk fix the comment on the set size. It should say inputs are SortedStarts not SortedDisjoint.
    /// Creates a new [`crate::sym_diff_iter_map::SymDiffIter`] from zero or more [`SortedDisjoint`] iterators. See [`crate::sym_diff_iter_map::SymDiffIter`] for more details and examples.
    pub fn new_k<K>(k: K) -> Self
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
        let iter = iter.sorted_by(|a, b| a.start().cmp(&b.start()));
        let iter = AssumeSortedStarts::new(iter);
        UnionIter::new(iter)
    }
}

// cmk0 test that every iterator (that can be) is FusedIterator
impl<T, I> FusedIterator for UnionIter<T, I>
where
    T: Integer,
    I: SortedStarts<T> + FusedIterator,
{
}
