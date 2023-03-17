use std::{
    collections::btree_map,
    ops::{self, RangeInclusive},
};

use itertools::Itertools;

use crate::{
    BitAndMerge, BitOrMerge, BitSubMerge, BitXOr, BitXOrTee, Integer, NotIter, SortedDisjoint,
    SortedDisjointIterator, SortedStarts,
};

#[derive(Clone)]
#[must_use = "iterators are lazy and do nothing unless consumed"]
/// An iterator that visits the ranges in the [`RangeSetInt`],
/// i.e., the integers as sorted & disjoint ranges.
///
/// This `struct` is created by the [`ranges`] method on [`RangeSetInt`]. See [`ranges`]'s
/// documentation for more.
///
/// [`RangeSetInt`]: crate::RangeSetInt
/// [`ranges`]: crate::RangeSetInt::ranges
pub struct RangesIter<'a, T: Integer> {
    pub(crate) iter: btree_map::Iter<'a, T, T>,
}

impl<'a, T: Integer> AsRef<RangesIter<'a, T>> for RangesIter<'a, T> {
    fn as_ref(&self) -> &Self {
        // Self is RangesIter<'a>, the type for which we impl AsRef
        self
    }
}

// RangesIter (one of the iterators from RangeSetInt) is SortedDisjoint
impl<T: Integer> SortedStarts for RangesIter<'_, T> {}
impl<T: Integer> SortedDisjoint for RangesIter<'_, T> {}

impl<T: Integer> ExactSizeIterator for RangesIter<'_, T> {
    #[must_use]
    fn len(&self) -> usize {
        self.iter.len()
    }
}

// Range's iterator is just the inside BTreeMap iterator as values
impl<'a, T: Integer> Iterator for RangesIter<'a, T> {
    type Item = RangeInclusive<T>;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|(start, end)| *start..=*end)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

//cmk0000 #[derive(Clone)]
#[must_use = "iterators are lazy and do nothing unless consumed"]
/// An iterator that visits the ranges in the [`RangeSetInt`],
/// i.e., the integers as sorted & disjoint ranges.
///
/// This `struct` is created by the [`into_ranges`] method on [`RangeSetInt`]. See [`into_ranges`]'s
/// documentation for more.
///
/// [`RangeSetInt`]: crate::RangeSetInt
/// [`into_ranges`]: crate::RangeSetInt::into_ranges
pub struct IntoRangesIter<T: Integer> {
    pub(crate) iter: std::collections::btree_map::IntoIter<T, T>,
}

impl<T: Integer> SortedStarts for IntoRangesIter<T> {}
impl<T: Integer> SortedDisjoint for IntoRangesIter<T> {}

impl<T: Integer> ExactSizeIterator for IntoRangesIter<T> {
    #[must_use]
    fn len(&self) -> usize {
        self.iter.len()
    }
}

// Range's iterator is just the inside BTreeMap iterator as values
impl<T: Integer> Iterator for IntoRangesIter<T> {
    type Item = RangeInclusive<T>;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|(start, end)| start..=end)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

impl<T: Integer> ops::Not for RangesIter<'_, T> {
    type Output = NotIter<T, Self>;

    fn not(self) -> Self::Output {
        SortedDisjointIterator::not(self)
    }
}

impl<T: Integer> ops::Not for IntoRangesIter<T> {
    type Output = NotIter<T, Self>;

    fn not(self) -> Self::Output {
        SortedDisjointIterator::not(self)
    }
}

impl<T: Integer, I> ops::BitOr<I> for RangesIter<'_, T>
where
    I: Iterator<Item = RangeInclusive<T>> + SortedDisjoint,
{
    type Output = BitOrMerge<T, Self, I>;

    fn bitor(self, other: I) -> Self::Output {
        SortedDisjointIterator::bitor(self, other)
    }
}

impl<T: Integer, I> ops::BitOr<I> for IntoRangesIter<T>
where
    I: Iterator<Item = RangeInclusive<T>> + SortedDisjoint,
{
    type Output = BitOrMerge<T, Self, I>;

    fn bitor(self, other: I) -> Self::Output {
        SortedDisjointIterator::bitor(self, other)
    }
}

impl<T: Integer, I> ops::Sub<I> for RangesIter<'_, T>
where
    I: Iterator<Item = RangeInclusive<T>> + SortedDisjoint,
{
    type Output = BitSubMerge<T, Self, I>;

    fn sub(self, other: I) -> Self::Output {
        SortedDisjointIterator::sub(self, other)
    }
}

impl<T: Integer, I> ops::Sub<I> for IntoRangesIter<T>
where
    I: Iterator<Item = RangeInclusive<T>> + SortedDisjoint,
{
    type Output = BitSubMerge<T, Self, I>;

    fn sub(self, other: I) -> Self::Output {
        SortedDisjointIterator::sub(self, other)
    }
}

impl<T: Integer, I> ops::BitXor<I> for RangesIter<'_, T>
where
    I: Iterator<Item = RangeInclusive<T>> + SortedDisjoint,
{
    type Output = BitXOr<T, Self, I>;

    #[allow(clippy::suspicious_arithmetic_impl)]
    fn bitxor(self, other: I) -> Self::Output {
        // We optimize by using self.clone() instead of tee
        let lhs1 = self.clone();
        let (rhs0, rhs1) = other.tee();
        (self - rhs0) | (rhs1.sub(lhs1))
    }
}

impl<T: Integer, I> ops::BitXor<I> for IntoRangesIter<T>
where
    I: Iterator<Item = RangeInclusive<T>> + SortedDisjoint,
{
    type Output = BitXOrTee<T, Self, I>;

    #[allow(clippy::suspicious_arithmetic_impl)]
    fn bitxor(self, other: I) -> Self::Output {
        // cmk000000000 we could like to optimize by using self.clone() instead of tee
        SortedDisjointIterator::bitxor(self, other)
    }
}

impl<T: Integer, I> ops::BitAnd<I> for RangesIter<'_, T>
where
    I: Iterator<Item = RangeInclusive<T>> + SortedDisjoint,
{
    type Output = BitAndMerge<T, Self, I>;

    #[allow(clippy::suspicious_arithmetic_impl)]
    fn bitand(self, other: I) -> Self::Output {
        SortedDisjointIterator::bitand(self, other)
    }
}

impl<T: Integer, I> ops::BitAnd<I> for IntoRangesIter<T>
where
    I: Iterator<Item = RangeInclusive<T>> + SortedDisjoint,
{
    type Output = BitAndMerge<T, Self, I>;

    #[allow(clippy::suspicious_arithmetic_impl)]
    fn bitand(self, other: I) -> Self::Output {
        SortedDisjointIterator::bitand(self, other)
    }
}
