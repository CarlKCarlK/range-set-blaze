use core::{
    iter::FusedIterator,
    ops::{self, RangeInclusive},
};

use crate::{BitAndMerge, BitOrMerge, BitSubMerge, BitXOrTee, Integer, SortedDisjoint};

/// Turns a [`SortedDisjoint`] iterator into a [`SortedDisjoint`] iterator of its complement,
/// i.e., all the integers not in the original iterator, as sorted & disjoint ranges.
///
/// # Example
///
/// ```
/// use range_set_blaze::{NotIter, SortedDisjoint, CheckSortedDisjoint};
///
/// let a = CheckSortedDisjoint::from([1u8..=2, 5..=100]);
/// let b = NotIter::new(a);
/// assert_eq!(b.to_string(), "0..=0, 3..=4, 101..=255");
///
/// // Or, equivalently:
/// let b = !CheckSortedDisjoint::from([1u8..=2, 5..=100]);
/// assert_eq!(b.to_string(), "0..=0, 3..=4, 101..=255");
/// ```
#[derive(Clone, Debug)]
#[must_use = "iterators are lazy and do nothing unless consumed"]
pub struct NotIter<T, I>
where
    T: Integer,
    I: SortedDisjoint<T>,
{
    iter: I,
    start_not: T,
    next_time_return_none: bool,
}

impl<T, I> NotIter<T, I>
where
    T: Integer,
    I: SortedDisjoint<T>,
{
    /// Create a new [`NotIter`] from a [`SortedDisjoint`] iterator. See [`NotIter`] for an example.
    pub fn new<J>(iter: J) -> Self
    where
        J: IntoIterator<Item = RangeInclusive<T>, IntoIter = I>,
    {
        NotIter {
            iter: iter.into_iter(),
            start_not: T::min_value(),
            next_time_return_none: false,
        }
    }
}

impl<T, I> FusedIterator for NotIter<T, I>
where
    T: Integer,
    I: SortedDisjoint<T> + FusedIterator,
{
}

impl<T, I> Iterator for NotIter<T, I>
where
    T: Integer,
    I: SortedDisjoint<T>,
{
    type Item = RangeInclusive<T>;
    fn next(&mut self) -> Option<RangeInclusive<T>> {
        debug_assert!(T::min_value() <= T::safe_max_value()); // real assert
        if self.next_time_return_none {
            return None;
        }
        let next_item = self.iter.next();
        if let Some(range) = next_item {
            let (start, end) = range.into_inner();
            debug_assert!(start <= end && end <= T::safe_max_value());
            if self.start_not < start {
                // We can subtract with underflow worry because
                // we know that start > start_not and so not min_value
                let result = Some(self.start_not..=start - T::one());
                if end < T::safe_max_value() {
                    self.start_not = end + T::one();
                } else {
                    self.next_time_return_none = true;
                }
                result
            } else if end < T::safe_max_value() {
                self.start_not = end + T::one();
                self.next() // will recurse at most once
            } else {
                self.next_time_return_none = true;
                None
            }
        } else {
            self.next_time_return_none = true;
            Some(self.start_not..=T::safe_max_value())
        }
    }

    // We could have one less or one more than the iter.
    fn size_hint(&self) -> (usize, Option<usize>) {
        let (low, high) = self.iter.size_hint();
        let low = if low > 0 { low - 1 } else { 0 };
        let high = high.map(|high| {
            if high < usize::MAX {
                high + 1
            } else {
                usize::MAX
            }
        });
        (low, high)
    }
}

impl<T: Integer, I> ops::Not for NotIter<T, I>
where
    I: SortedDisjoint<T>,
{
    type Output = NotIter<T, Self>;

    fn not(self) -> Self::Output {
        // It would be fun to optimize to self.iter, but that would require
        // also considering fields 'start_not' and 'next_time_return_none'.
        self.complement()
    }
}

impl<T: Integer, R, L> ops::BitOr<R> for NotIter<T, L>
where
    L: SortedDisjoint<T>,
    R: SortedDisjoint<T>,
{
    type Output = BitOrMerge<T, Self, R>;

    fn bitor(self, other: R) -> Self::Output {
        SortedDisjoint::union(self, other)
    }
}

impl<T: Integer, R, L> ops::Sub<R> for NotIter<T, L>
where
    L: SortedDisjoint<T>,
    R: SortedDisjoint<T>,
{
    type Output = BitSubMerge<T, Self, R>;

    fn sub(self, other: R) -> Self::Output {
        // It would be fun to optimize !!self.iter into self.iter
        // but that would require also considering fields 'start_not' and 'next_time_return_none'.
        SortedDisjoint::difference(self, other)
    }
}

impl<T: Integer, R, L> ops::BitXor<R> for NotIter<T, L>
where
    L: SortedDisjoint<T>,
    R: SortedDisjoint<T>,
{
    type Output = BitXOrTee<T, Self, R>;

    #[allow(clippy::suspicious_arithmetic_impl)]
    fn bitxor(self, other: R) -> Self::Output {
        // It would be fine optimize !!self.iter into self.iter, ala
        // ¬(¬n ∨ ¬r) ∨ ¬(n ∨ r) // https://www.wolframalpha.com/input?i=%28not+n%29+xor+r
        // but that would require also considering fields 'start_not' and 'next_time_return_none'.
        SortedDisjoint::symmetric_difference(self, other)
    }
}

impl<T: Integer, R, L> ops::BitAnd<R> for NotIter<T, L>
where
    L: SortedDisjoint<T>,
    R: SortedDisjoint<T>,
{
    type Output = BitAndMerge<T, Self, R>;

    fn bitand(self, other: R) -> Self::Output {
        // It would be fun to optimize !!self.iter into self.iter
        // but that would require also considering fields 'start_not' and 'next_time_return_none'.
        SortedDisjoint::intersection(self, other)
    }
}

// FUTURE define Not, etc on DynSortedDisjoint
