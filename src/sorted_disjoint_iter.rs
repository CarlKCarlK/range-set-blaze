use std::{cmp::max, ops::RangeInclusive};

use itertools::Itertools;

use crate::{
    unsorted_disjoint::{AssumeSortedStarts, UnsortedDisjoint},
    Integer, SortedStarts,
};

// cmk00 maybe not the best name
#[derive(Clone)]
#[must_use = "iterators are lazy and do nothing unless consumed"]
pub struct SortedDisjointIter<T, I>
where
    T: Integer,
    I: Iterator<Item = RangeInclusive<T>> + SortedStarts,
{
    iter: I,
    option_range: Option<RangeInclusive<T>>,
}

impl<T, I> SortedDisjointIter<T, I>
where
    T: Integer,
    I: Iterator<Item = RangeInclusive<T>> + SortedStarts,
{
    pub fn new(iter: I) -> Self {
        Self {
            iter,
            option_range: None,
        }
    }
}

impl<T: Integer, const N: usize> From<[T; N]>
    for SortedDisjointIter<T, SortedRangeInclusiveVec<T>>
{
    fn from(arr: [T; N]) -> Self {
        arr.as_slice().into()
    }
}

impl<T: Integer> From<&[T]> for SortedDisjointIter<T, SortedRangeInclusiveVec<T>> {
    fn from(slice: &[T]) -> Self {
        slice.iter().cloned().collect()
    }
}

impl<T: Integer, const N: usize> From<[RangeInclusive<T>; N]>
    for SortedDisjointIter<T, SortedRangeInclusiveVec<T>>
{
    fn from(arr: [RangeInclusive<T>; N]) -> Self {
        arr.as_slice().into()
    }
}

impl<T: Integer> From<&[RangeInclusive<T>]> for SortedDisjointIter<T, SortedRangeInclusiveVec<T>> {
    fn from(slice: &[RangeInclusive<T>]) -> Self {
        slice.iter().cloned().collect()
    }
}

type SortedRangeInclusiveVec<T> = AssumeSortedStarts<T, std::vec::IntoIter<RangeInclusive<T>>>;

impl<T: Integer> FromIterator<T> for SortedDisjointIter<T, SortedRangeInclusiveVec<T>> {
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = T>,
    {
        iter.into_iter().map(|x| x..=x).collect()
    }
}

impl<T: Integer> FromIterator<RangeInclusive<T>>
    for SortedDisjointIter<T, SortedRangeInclusiveVec<T>>
{
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = RangeInclusive<T>>,
    {
        UnsortedDisjoint::from(iter.into_iter()).into()
    }
}

impl<T, I> From<UnsortedDisjoint<T, I>> for SortedDisjointIter<T, SortedRangeInclusiveVec<T>>
where
    T: Integer,
    I: Iterator<Item = RangeInclusive<T>>, // Any iterator is OK, because we will sort
{
    fn from(unsorted_disjoint: UnsortedDisjoint<T, I>) -> Self {
        let iter = AssumeSortedStarts {
            iter: unsorted_disjoint.sorted_by_key(|range| *range.start()),
        };
        Self {
            iter,
            option_range: None,
        }
    }
}

// cmk00 be sure that every function that accepts a val isn't max_value u128, i128 and returns an error
impl<T: Integer, I> Iterator for SortedDisjointIter<T, I>
where
    I: Iterator<Item = RangeInclusive<T>> + SortedStarts,
{
    type Item = RangeInclusive<T>;

    fn next(&mut self) -> Option<RangeInclusive<T>> {
        loop {
            if let Some(range) = self.iter.next() {
                let (start, end) = range.into_inner();
                if end < start {
                    return self.next(); // !!!cmk00 test this
                }
                if let Some(current_range) = self.option_range.clone() {
                    let (current_start, current_end) = current_range.into_inner();
                    debug_assert!(current_start <= start); // real assert
                    if start <= current_end
                        || (current_end < T::max_value2() && start <= current_end + T::one())
                    {
                        self.option_range = Some(current_start..=max(current_end, end));
                        continue;
                    } else {
                        self.option_range = Some(start..=end);
                        return Some(current_start..=current_end);
                    }
                } else {
                    self.option_range = Some(start..=end);
                    continue;
                }
            } else {
                let result = self.option_range.clone();
                self.option_range = None;
                return result;
            }
        }
    }

    // There could be a few as 1 (or 0 if the iter is empty) or as many as the iter.
    // Plus, possibly one more if we have a range is in progress.
    fn size_hint(&self) -> (usize, Option<usize>) {
        let (low, high) = self.iter.size_hint();
        let low = low.min(1);
        if self.option_range.is_some() {
            (low, high.map(|x| x + 1))
        } else {
            (low, high)
        }
    }
}
