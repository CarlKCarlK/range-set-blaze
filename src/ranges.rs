use alloc::collections::btree_map;
use core::{
    iter::FusedIterator,
    ops::{self, RangeInclusive},
};

use itertools::Itertools;

use crate::{
    BitAndMerge, BitOrMerge, BitSubMerge, BitXOr, BitXOrTee, Integer, NotIter, SortedDisjoint,
    SortedStarts,
};

/// An iterator that visits the ranges in the [`RangeSetBlaze`],
/// i.e., the integers as sorted & disjoint ranges.
///
/// This `struct` is created by the [`ranges`] method on [`RangeSetBlaze`]. See [`ranges`]'s
/// documentation for more.
///
/// [`RangeSetBlaze`]: crate::RangeSetBlaze
/// [`ranges`]: crate::RangeSetBlaze::ranges
#[derive(Clone, Debug)]
#[must_use = "iterators are lazy and do nothing unless consumed"]
pub struct RangesIter<'a, T: Integer> {
    pub(crate) iter: btree_map::Iter<'a, T, T>,
}

impl<'a, T: Integer> AsRef<RangesIter<'a, T>> for RangesIter<'a, T> {
    fn as_ref(&self) -> &Self {
        // Self is RangesIter<'a>, the type for which we impl AsRef
        self
    }
}

// RangesIter (one of the iterators from RangeSetBlaze) is SortedDisjoint
impl<T: Integer> SortedStarts<T> for RangesIter<'_, T> {}
impl<T: Integer> SortedDisjoint<T> for RangesIter<'_, T> {}

impl<T: Integer> ExactSizeIterator for RangesIter<'_, T> {
    #[must_use]
    fn len(&self) -> usize {
        self.iter.len()
    }
}

impl<'a, T: Integer> FusedIterator for RangesIter<'a, T> {}

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

#[must_use = "iterators are lazy and do nothing unless consumed"]
/// An iterator that moves out the ranges in the [`RangeSetBlaze`],
/// i.e., the integers as sorted & disjoint ranges.
///
/// This `struct` is created by the [`into_ranges`] method on [`RangeSetBlaze`]. See [`into_ranges`]'s
/// documentation for more.
///
/// [`RangeSetBlaze`]: crate::RangeSetBlaze
/// [`into_ranges`]: crate::RangeSetBlaze::into_ranges
#[derive(Debug)]
pub struct IntoRangesIter<T: Integer> {
    pub(crate) iter: alloc::collections::btree_map::IntoIter<T, T>,
}

impl<T: Integer> SortedStarts<T> for IntoRangesIter<T> {}
impl<T: Integer> SortedDisjoint<T> for IntoRangesIter<T> {}

impl<T: Integer> ExactSizeIterator for IntoRangesIter<T> {
    #[must_use]
    fn len(&self) -> usize {
        self.iter.len()
    }
}

impl<T: Integer> FusedIterator for IntoRangesIter<T> {}

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
        self.complement()
    }
}

impl<T: Integer> ops::Not for IntoRangesIter<T> {
    type Output = NotIter<T, Self>;

    fn not(self) -> Self::Output {
        self.complement()
    }
}

impl<T: Integer, I> ops::BitOr<I> for RangesIter<'_, T>
where
    I: SortedDisjoint<T>,
{
    type Output = BitOrMerge<T, Self, I>;

    fn bitor(self, other: I) -> Self::Output {
        SortedDisjoint::union(self, other)
    }
}

impl<T: Integer, I> ops::BitOr<I> for IntoRangesIter<T>
where
    I: SortedDisjoint<T>,
{
    type Output = BitOrMerge<T, Self, I>;

    fn bitor(self, other: I) -> Self::Output {
        SortedDisjoint::union(self, other)
    }
}

impl<T: Integer, I> ops::Sub<I> for RangesIter<'_, T>
where
    I: SortedDisjoint<T>,
{
    type Output = BitSubMerge<T, Self, I>;

    fn sub(self, other: I) -> Self::Output {
        SortedDisjoint::difference(self, other)
    }
}

impl<T: Integer, I> ops::Sub<I> for IntoRangesIter<T>
where
    I: SortedDisjoint<T>,
{
    type Output = BitSubMerge<T, Self, I>;

    fn sub(self, other: I) -> Self::Output {
        SortedDisjoint::difference(self, other)
    }
}

impl<T: Integer, I> ops::BitXor<I> for RangesIter<'_, T>
where
    I: SortedDisjoint<T>,
{
    type Output = BitXOr<T, Self, I>;

    #[allow(clippy::suspicious_arithmetic_impl)]
    fn bitxor(self, other: I) -> Self::Output {
        // We optimize by using self.clone() instead of tee
        let lhs1 = self.clone();
        let (rhs0, rhs1) = other.tee();
        (self - rhs0) | (rhs1.difference(lhs1))
    }
}

impl<T: Integer, I> ops::BitXor<I> for IntoRangesIter<T>
where
    I: SortedDisjoint<T>,
{
    type Output = BitXOrTee<T, Self, I>;

    #[allow(clippy::suspicious_arithmetic_impl)]
    fn bitxor(self, other: I) -> Self::Output {
        SortedDisjoint::symmetric_difference(self, other)
    }
}

impl<T: Integer, I> ops::BitAnd<I> for RangesIter<'_, T>
where
    I: SortedDisjoint<T>,
{
    type Output = BitAndMerge<T, Self, I>;

    #[allow(clippy::suspicious_arithmetic_impl)]
    fn bitand(self, other: I) -> Self::Output {
        SortedDisjoint::intersection(self, other)
    }
}

impl<T: Integer, I> ops::BitAnd<I> for IntoRangesIter<T>
where
    I: SortedDisjoint<T>,
{
    type Output = BitAndMerge<T, Self, I>;

    #[allow(clippy::suspicious_arithmetic_impl)]
    fn bitand(self, other: I) -> Self::Output {
        SortedDisjoint::intersection(self, other)
    }
}
