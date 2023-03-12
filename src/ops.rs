use std::ops::{self, RangeInclusive};

use itertools::Itertools;

use crate::{
    not_iter::NotIter, sorted_disjoint_iter::SortedDisjointIter, BitAndMerge, BitOrMerge,
    BitSubMerge, BitXOr, BitXOrTee, Integer, Ranges, SortedDisjoint, SortedDisjointIterator,
    SortedStarts,
};

// Not: Ranges, NotIter, BitOrMerge
impl<T: Integer> ops::Not for Ranges<'_, T> {
    type Output = NotIter<T, Self>;

    fn not(self) -> Self::Output {
        NotIter::new(self)
    }
}

impl<T: Integer, I> ops::Not for NotIter<T, I>
where
    I: Iterator<Item = RangeInclusive<T>> + SortedDisjoint,
{
    type Output = NotIter<T, Self>;

    fn not(self) -> Self::Output {
        // It would be fun to optimize to self.iter, but that would require
        // also considering fields 'start_not' and 'next_time_return_none'.
        NotIter::new(self)
    }
}

impl<T: Integer, I> ops::Not for SortedDisjointIter<T, I>
where
    I: Iterator<Item = RangeInclusive<T>> + SortedStarts,
{
    type Output = NotIter<T, Self>;

    fn not(self) -> Self::Output {
        NotIter::new(self)
    }
}

// BitOr: Ranges, NotIter, BitOrMerge
impl<T: Integer, I> ops::BitOr<I> for Ranges<'_, T>
where
    I: Iterator<Item = RangeInclusive<T>> + SortedDisjoint,
{
    type Output = BitOrMerge<T, Self, I>;

    fn bitor(self, rhs: I) -> Self::Output {
        SortedDisjointIterator::bitor(self, rhs)
    }
}

impl<T: Integer, R, L> ops::BitOr<R> for NotIter<T, L>
where
    L: Iterator<Item = RangeInclusive<T>> + SortedDisjoint,
    R: Iterator<Item = RangeInclusive<T>> + SortedDisjoint,
{
    type Output = BitOrMerge<T, Self, R>;

    fn bitor(self, rhs: R) -> Self::Output {
        SortedDisjointIterator::bitor(self, rhs)
    }
}

impl<T: Integer, R, L> ops::BitOr<R> for SortedDisjointIter<T, L>
where
    L: Iterator<Item = RangeInclusive<T>> + SortedStarts,
    R: Iterator<Item = RangeInclusive<T>> + SortedDisjoint,
{
    type Output = BitOrMerge<T, Self, R>;

    fn bitor(self, rhs: R) -> Self::Output {
        // It might be fine to optimize to self.iter, but that would require
        // also considering field 'range'
        SortedDisjointIterator::bitor(self, rhs)
    }
}

// Sub: Ranges, NotIter, BitOrMerge

impl<T: Integer, I> ops::Sub<I> for Ranges<'_, T>
where
    I: Iterator<Item = RangeInclusive<T>> + SortedDisjoint,
{
    type Output = BitSubMerge<T, Self, I>;

    fn sub(self, rhs: I) -> Self::Output {
        !(!self | rhs)
    }
}

impl<T: Integer, R, L> ops::Sub<R> for NotIter<T, L>
where
    L: Iterator<Item = RangeInclusive<T>> + SortedDisjoint,
    R: Iterator<Item = RangeInclusive<T>> + SortedDisjoint,
{
    type Output = BitSubMerge<T, Self, R>;

    fn sub(self, rhs: R) -> Self::Output {
        // It would be fun to optimize !!self.iter into self.iter
        // but that would require also considering fields 'start_not' and 'next_time_return_none'.
        !(!self | rhs)
    }
}

impl<T: Integer, R, L> ops::Sub<R> for SortedDisjointIter<T, L>
where
    L: Iterator<Item = RangeInclusive<T>> + SortedStarts,
    R: Iterator<Item = RangeInclusive<T>> + SortedDisjoint,
{
    type Output = BitSubMerge<T, Self, R>;

    fn sub(self, rhs: R) -> Self::Output {
        !(!self | rhs)
    }
}

// BitXor: Ranges, NotIter, BitOrMerge

impl<T: Integer, I> ops::BitXor<I> for Ranges<'_, T>
where
    I: Iterator<Item = RangeInclusive<T>> + SortedDisjoint,
{
    type Output = BitXOr<T, Self, I>;

    #[allow(clippy::suspicious_arithmetic_impl)]
    fn bitxor(self, rhs: I) -> Self::Output {
        // We optimize by using self.clone() instead of tee
        let lhs1 = self.clone();
        let (rhs0, rhs1) = rhs.tee();
        (self - rhs0) | (rhs1.sub(lhs1))
    }
}

impl<T: Integer, R, L> ops::BitXor<R> for NotIter<T, L>
where
    L: Iterator<Item = RangeInclusive<T>> + SortedDisjoint,
    R: Iterator<Item = RangeInclusive<T>> + SortedDisjoint,
{
    type Output = BitXOrTee<T, Self, R>;

    #[allow(clippy::suspicious_arithmetic_impl)]
    fn bitxor(self, rhs: R) -> Self::Output {
        // It would be fine optimize !!self.iter into self.iter, ala
        // ¬(¬n ∨ ¬r) ∨ ¬(n ∨ r) // https://www.wolframalpha.com/input?i=%28not+n%29+xor+r
        // but that would require also considering fields 'start_not' and 'next_time_return_none'.
        let (lhs0, lhs1) = self.tee();
        let (rhs0, rhs1) = rhs.tee();
        lhs0.sub(rhs0) | rhs1.sub(lhs1)
    }
}

impl<T: Integer, R, L> ops::BitXor<R> for SortedDisjointIter<T, L>
where
    L: Iterator<Item = RangeInclusive<T>> + SortedStarts,
    R: Iterator<Item = RangeInclusive<T>> + SortedDisjoint,
{
    type Output = BitXOrTee<T, Self, R>;

    #[allow(clippy::suspicious_arithmetic_impl)]
    fn bitxor(self, rhs: R) -> Self::Output {
        let (lhs0, lhs1) = self.tee();
        let (rhs0, rhs1) = rhs.tee();
        lhs0.sub(rhs0) | rhs1.sub(lhs1)
    }
}

// BitAnd: Ranges, NotIter, BitOrMerge

impl<T: Integer, I> ops::BitAnd<I> for Ranges<'_, T>
where
    I: Iterator<Item = RangeInclusive<T>> + SortedDisjoint,
{
    type Output = BitAndMerge<T, Self, I>;

    #[allow(clippy::suspicious_arithmetic_impl)]
    fn bitand(self, rhs: I) -> Self::Output {
        !(!self | rhs.not())
    }
}

impl<T: Integer, R, L> ops::BitAnd<R> for NotIter<T, L>
where
    L: Iterator<Item = RangeInclusive<T>> + SortedDisjoint,
    R: Iterator<Item = RangeInclusive<T>> + SortedDisjoint,
{
    type Output = BitAndMerge<T, Self, R>;

    fn bitand(self, rhs: R) -> Self::Output {
        // It would be fun to optimize !!self.iter into self.iter
        // but that would require also considering fields 'start_not' and 'next_time_return_none'.
        !(!self | rhs.not())
    }
}

// cmk name all generics in a sensible way
impl<T: Integer, R, L> ops::BitAnd<R> for SortedDisjointIter<T, L>
where
    L: Iterator<Item = RangeInclusive<T>> + SortedStarts,
    R: Iterator<Item = RangeInclusive<T>> + SortedDisjoint,
{
    type Output = BitAndMerge<T, Self, R>;

    fn bitand(self, rhs: R) -> Self::Output {
        !(!self | rhs.not())
    }
}
