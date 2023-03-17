use std::ops::RangeInclusive;

use itertools::{Itertools, KMergeBy, MergeBy};

use crate::{Integer, SortedDisjoint, SortedStarts};

#[derive(Clone)]
#[must_use = "iterators are lazy and do nothing unless consumed"]
pub struct Merge<T, L, R>
where
    T: Integer,
    L: Iterator<Item = RangeInclusive<T>> + SortedDisjoint,
    R: Iterator<Item = RangeInclusive<T>> + SortedDisjoint,
{
    #[allow(clippy::type_complexity)]
    iter: MergeBy<L, R, fn(&RangeInclusive<T>, &RangeInclusive<T>) -> bool>,
}

impl<T, L, R> Merge<T, L, R>
where
    T: Integer,
    L: Iterator<Item = RangeInclusive<T>> + SortedDisjoint,
    R: Iterator<Item = RangeInclusive<T>> + SortedDisjoint,
{
    pub fn new(left: L, right: R) -> Self {
        Self {
            iter: left.merge_by(right, |a, b| a.start() < b.start()),
        }
    }
}

impl<T, L, R> Iterator for Merge<T, L, R>
where
    T: Integer,
    L: Iterator<Item = RangeInclusive<T>> + SortedDisjoint,
    R: Iterator<Item = RangeInclusive<T>> + SortedDisjoint,
{
    type Item = RangeInclusive<T>;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

impl<T, L, R> SortedStarts for Merge<T, L, R>
where
    T: Integer,
    L: Iterator<Item = RangeInclusive<T>> + SortedDisjoint,
    R: Iterator<Item = RangeInclusive<T>> + SortedDisjoint,
{
}

#[derive(Clone)]
#[must_use = "iterators are lazy and do nothing unless consumed"]
pub struct KMerge<T, I>
where
    T: Integer,
    I: Iterator<Item = RangeInclusive<T>> + SortedDisjoint,
{
    #[allow(clippy::type_complexity)]
    iter: KMergeBy<I, fn(&RangeInclusive<T>, &RangeInclusive<T>) -> bool>,
}

impl<T, I> KMerge<T, I>
where
    T: Integer,
    I: Iterator<Item = RangeInclusive<T>> + SortedDisjoint,
{
    pub fn new<J>(iter: J) -> Self
    where
        J: IntoIterator<Item = I>,
    {
        Self {
            iter: iter.into_iter().kmerge_by(|a, b| a.start() < b.start()),
        }
    }
}

impl<T, I> Iterator for KMerge<T, I>
where
    T: Integer,
    I: Iterator<Item = RangeInclusive<T>> + SortedDisjoint,
{
    type Item = RangeInclusive<T>;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

impl<T, I> SortedStarts for KMerge<T, I>
where
    T: Integer,
    I: Iterator<Item = RangeInclusive<T>> + SortedDisjoint,
{
}
