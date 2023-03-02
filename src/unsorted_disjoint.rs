// !!!cmk make the names consistent, start/lower vs stop/upper/end/...
// !!!cmk replace OptionRange with Option<RangeInclusive<T>>

use num_traits::Zero;
use std::{
    cmp::{max, min},
    ops::RangeInclusive,
};

use crate::{Integer, SortedDisjoint, SortedStarts};

pub struct UnsortedDisjoint<T, I>
where
    T: Integer,
    I: Iterator<Item = RangeInclusive<T>>,
{
    iter: I,
    range: Option<RangeInclusive<T>>,
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
            range: None,
            two: T::one() + T::one(),
        }
    }
}

impl<T, I> Iterator for UnsortedDisjoint<T, I>
where
    T: Integer,
    I: Iterator<Item = RangeInclusive<T>>,
{
    type Item = RangeInclusive<T>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(range_inclusive) = self.iter.next() {
            let (lower, upper) = range_inclusive.into_inner();
            if lower > upper {
                return self.next();
            }
            assert!(upper <= T::max_value2()); // !!!cmk0 raise error on panic?
            if let Some(self_range_inclusive) = self.range.clone() {
                let (self_lower, self_upper) = self_range_inclusive.into_inner();
                if (lower >= self.two && lower - self.two >= self_upper)
                    || (self_lower >= self.two && self_lower - self.two >= upper)
                {
                    let result = Some(self_lower..=self_upper);
                    self.range = Some(lower..=upper);
                    result
                } else {
                    self.range = Some(min(self_lower, lower)..=max(self_upper, upper));
                    self.next()
                }
            } else {
                self.range = Some(lower..=upper);
                self.next()
            }
        } else if let Some(range_inclusive) = self.range.clone() {
            // cmk0 clone?
            self.range = None;
            Some(range_inclusive)
        } else {
            None
        }
    }

    // As few as one (or zero if iter is empty) and as many as iter.len()
    fn size_hint(&self) -> (usize, Option<usize>) {
        let (lower, upper) = self.iter.size_hint();
        let lower = if lower == 0 { 0 } else { 1 };
        (lower, upper)
    }
}

pub struct SortedDisjointWithLenSoFar<T, I>
where
    T: Integer,
    I: Iterator<Item = RangeInclusive<T>> + SortedDisjoint,
{
    iter: I,
    len: <T as Integer>::SafeLen,
}

// cmk Rule there is no reason From's should be into iterators
impl<T: Integer, I> From<I> for SortedDisjointWithLenSoFar<T, I::IntoIter>
where
    I: IntoIterator<Item = RangeInclusive<T>>,
    I::IntoIter: SortedDisjoint,
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
    I: Iterator<Item = RangeInclusive<T>> + SortedDisjoint,
{
    pub fn len_so_far(&self) -> <T as Integer>::SafeLen {
        self.len.clone()
    }
}

impl<T: Integer, I> Iterator for SortedDisjointWithLenSoFar<T, I>
where
    I: Iterator<Item = RangeInclusive<T>> + SortedDisjoint,
{
    type Item = (T, T);

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(range_inclusive) = self.iter.next() {
            let (start, stop) = range_inclusive.clone().into_inner();
            debug_assert!(start <= stop && stop <= T::max_value2());
            self.len += T::safe_inclusive_len(&range_inclusive);
            Some((start, stop))
        } else {
            None
        }
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}
impl<T: Integer, I> SortedDisjoint for SortedDisjointWithLenSoFar<T, I> where
    I: Iterator<Item = RangeInclusive<T>> + SortedDisjoint
{
}
impl<T: Integer, I> SortedStarts for SortedDisjointWithLenSoFar<T, I> where
    I: Iterator<Item = RangeInclusive<T>> + SortedDisjoint
{
}

#[derive(Clone)]
pub struct AssumeSortedStarts<T, I>
where
    T: Integer,
    I: Iterator<Item = RangeInclusive<T>>,
{
    pub(crate) iter: I,
}

impl<T: Integer, I> SortedStarts for AssumeSortedStarts<T, I> where
    I: Iterator<Item = RangeInclusive<T>>
{
}

impl<T, I> Iterator for AssumeSortedStarts<T, I>
where
    T: Integer,
    I: Iterator<Item = RangeInclusive<T>>,
{
    type Item = RangeInclusive<T>;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}
