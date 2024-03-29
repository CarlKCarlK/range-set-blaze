use crate::{
    range_set_blaze::SortedStartsToUnitMap, Integer, RangeMapBlaze, RangeSetBlaze, SortedDisjoint,
    SortedStarts,
};
use core::{
    cmp::{max, min},
    iter::FusedIterator,
    ops::RangeInclusive,
};
use num_traits::Zero;

#[must_use = "iterators are lazy and do nothing unless consumed"]
pub(crate) struct UnsortedDisjoint<T, I>
where
    T: Integer,
    I: Iterator<Item = RangeInclusive<T>>,
{
    iter: I,
    option_range: Option<RangeInclusive<T>>,
    min_value_plus_2: T,
    two: T,
}

impl<T, I> From<I> for UnsortedDisjoint<T, I::IntoIter>
where
    T: Integer,
    I: IntoIterator<Item = RangeInclusive<T>>, // Any iterator is fine
{
    fn from(into_iter: I) -> Self {
        UnsortedDisjoint {
            iter: into_iter.into_iter(),
            option_range: None,
            min_value_plus_2: T::min_value() + T::one() + T::one(),
            two: T::one() + T::one(),
        }
    }
}

impl<T, I> FusedIterator for UnsortedDisjoint<T, I>
where
    T: Integer,
    I: Iterator<Item = RangeInclusive<T>> + FusedIterator,
{
}

impl<T, I> Iterator for UnsortedDisjoint<T, I>
where
    T: Integer,
    I: Iterator<Item = RangeInclusive<T>>,
{
    type Item = RangeInclusive<T>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            // get the next range, if none, return the current range
            let Some(next_range) = self.iter.next() else {
                return self.option_range.take();
            };

            // check the next range is valid and non-empty
            let (next_start, next_end) = next_range.clone().into_inner();
            assert!(
                next_end <= T::safe_max_value(),
                "end must be <= T::safe_max_value()"
            );
            if next_start > next_end {
                continue;
            }

            // get the current range (if none, set the current range to the next range and loop)
            let Some(current_range) = self.option_range.take() else {
                self.option_range = Some(next_range);
                continue;
            };

            // if the ranges do not touch or overlap, return the current range and set the current range to the next range
            let (current_start, current_end) = current_range.clone().into_inner();
            if (next_start >= self.min_value_plus_2 && current_end <= next_start - self.two)
                || (current_start >= self.min_value_plus_2 && next_end <= current_start - self.two)
            {
                self.option_range = Some(next_range);
                return Some(current_range);
            }

            // they touch or overlap, so merge them and loop
            self.option_range = Some(min(current_start, next_start)..=max(current_end, next_end));
        }
    }

    // As few as one (or zero if iter is empty) and as many as iter.len()
    // There could be one extra if option_range is Some.
    fn size_hint(&self) -> (usize, Option<usize>) {
        let (lower, upper) = self.iter.size_hint();
        let lower = if lower == 0 { 0 } else { 1 };
        if self.option_range.is_some() {
            (lower, upper.map(|x| x + 1))
        } else {
            (lower, upper)
        }
    }
}

#[must_use = "iterators are lazy and do nothing unless consumed"]
pub(crate) struct SortedDisjointWithLenSoFar<T, I>
where
    T: Integer,
    I: SortedDisjoint<T>,
{
    iter: I,
    len: <T as Integer>::SafeLen,
}

impl<T: Integer, I> From<I> for SortedDisjointWithLenSoFar<T, I::IntoIter>
where
    I: IntoIterator<Item = RangeInclusive<T>>,
    I::IntoIter: SortedDisjoint<T>,
{
    fn from(into_iter: I) -> Self {
        SortedDisjointWithLenSoFar {
            iter: into_iter.into_iter(),
            len: <T as Integer>::SafeLen::zero(),
        }
    }
}

impl<T: Integer, I> SortedDisjointWithLenSoFar<T, I>
where
    I: SortedDisjoint<T>,
{
    pub fn len_so_far(&self) -> <T as Integer>::SafeLen {
        self.len
    }
}

impl<T: Integer, I> FusedIterator for SortedDisjointWithLenSoFar<T, I> where
    I: SortedDisjoint<T> + FusedIterator
{
}

impl<T: Integer, I> Iterator for SortedDisjointWithLenSoFar<T, I>
where
    I: SortedDisjoint<T>,
{
    type Item = (T, T);

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(range) = self.iter.next() {
            let (start, end) = range.clone().into_inner();
            debug_assert!(start <= end && end <= T::safe_max_value());
            self.len += T::safe_len(&range);
            Some((start, end))
        } else {
            None
        }
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

#[derive(Clone, Debug)]
#[must_use = "iterators are lazy and do nothing unless consumed"]

/// Gives any iterator of ranges the [`SortedStarts`] trait without any checking.
pub struct AssumeSortedStarts<T, I>
where
    T: Integer,
    I: Iterator<Item = RangeInclusive<T>> + FusedIterator,
{
    pub(crate) iter: I,
}

impl<T: Integer, I> SortedStarts<T> for AssumeSortedStarts<T, I> where
    I: Iterator<Item = RangeInclusive<T>> + FusedIterator
{
}

impl<T, I> AssumeSortedStarts<T, I>
where
    T: Integer,
    I: Iterator<Item = RangeInclusive<T>> + FusedIterator,
{
    /// Construct [`AssumeSortedStarts`] from a range iterator.
    pub fn new<J: IntoIterator<IntoIter = I>>(iter: J) -> Self {
        AssumeSortedStarts {
            iter: iter.into_iter(),
        }
    }

    /// Create a [`RangeSetBlaze`] from an [`AssumeSortedStarts`] iterator.
    ///
    /// *For more about constructors and performance, see [`RangeSetBlaze` Constructors](struct.RangeSetBlaze.html#constructors).*
    ///
    /// # Examples
    ///
    /// ```
    /// use range_set_blaze::prelude::*;
    ///
    /// let a0 = RangeSetBlaze::from_sorted_starts(AssumeSortedStarts::new([-10..=-5, -7..=2]));
    /// let a1: RangeSetBlaze<i32> = AssumeSortedStarts::new([-10..=-5, -7..=2]).into_range_set_blaze();
    /// assert!(a0 == a1 && a0.to_string() == "-10..=2");
    /// ```
    pub fn into_range_set_blaze(self) -> RangeSetBlaze<T>
    where
        Self: Sized,
    {
        let map = SortedStartsToUnitMap::new(self);
        let range_map_blaze = RangeMapBlaze::from_priority_sorted_starts_map(map);
        RangeSetBlaze(range_map_blaze)
    }
}

impl<T, I> FusedIterator for AssumeSortedStarts<T, I>
where
    T: Integer,
    I: Iterator<Item = RangeInclusive<T>> + FusedIterator,
{
}

impl<T, I> Iterator for AssumeSortedStarts<T, I>
where
    T: Integer,
    I: Iterator<Item = RangeInclusive<T>> + FusedIterator,
{
    type Item = RangeInclusive<T>;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}
