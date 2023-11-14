use core::{
    iter::FusedIterator,
    ops::{self, RangeInclusive},
    slice::ChunksExact,
};

use crate::{is_good, BitAndMerge, BitOrMerge, BitSubMerge, BitXOrTee, Integer, SortedDisjoint};

/// cmk
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
pub struct FromSliceIter<'a, T>
where
    T: Integer,
{
    slice: &'a [T],
    before_start: usize,
    before_end_exclusive: usize,
    chunk_size: usize, //  = 16;
    previous_range: Option<RangeInclusive<T>>,
    chunks_start: usize,
    chunks_end_exclusive: usize,
    remainder_start: usize,
    remainder_end_exclusive: usize,
}

impl<'a, T: 'a> FromSliceIter<'a, T>
where
    T: Integer,
{
    /// cmk Create a new [`NotIter`] from a [`SortedDisjoint`] iterator. See [`NotIter`] for an example.
    pub fn new(slice: &'a [T]) -> Self {
        FromSliceIter {
            slice,
            before_start: 0,
            before_end_exclusive: 0,
            chunk_size: 16, // cmk const
            previous_range: None,
            chunks_start: 0,
            chunks_end_exclusive: slice.len() / 16 * 16,
            remainder_start: slice.len() / 16 * 16,
            remainder_end_exclusive: slice.len(),
        }
    }
}

// cmk what's this about?
// impl<T, I> FusedIterator for FromSliceIter<'a, T> where T: Integer {}

impl<'a, T: 'a> Iterator for FromSliceIter<'a, T>
where
    T: Integer,
{
    type Item = RangeInclusive<T>;

    fn next(&mut self) -> Option<RangeInclusive<T>> {
        if self.before_start < self.before_end_exclusive {
            let element = self.slice[self.before_start];
            self.before_start += 1;
            return Some(element..=element);
        }
        while self.chunks_start < self.chunks_end_exclusive {
            // cmk
            let chunk_end_exclusive = self.chunks_start + self.chunk_size;
            let chunk = &self.slice[self.chunks_start..chunk_end_exclusive];
            if is_good(chunk) {
                let this_start = self.slice[self.chunks_start];
                let this_end = self.slice[chunk_end_exclusive - 1];

                if let Some(inner_previous_range) = self.previous_range.as_mut() {
                    // if some and previous is some and adjacent, combine
                    if *inner_previous_range.end() + T::one() == this_start {
                        *inner_previous_range = *(inner_previous_range.start())..=this_end;
                    } else {
                        // if some and previous is some but not adjacent, flush previous, set previous to this range.
                        let result = Some(inner_previous_range.clone());
                        *inner_previous_range = this_start..=this_end;
                        self.chunks_start += self.chunk_size;
                        return result;
                    }
                } else {
                    // if some and previous is None, set previous to this range.
                    self.previous_range = Some(this_start..=this_end);
                }
            } else {
                // If none, flush previous range, set it to none, output this chunk as a bunch of singletons.
                self.before_start = self.chunks_start;
                self.before_end_exclusive = chunk_end_exclusive;
                if let Some(previous) = self.previous_range.take() {
                    debug_assert!(self.previous_range.is_none());
                    self.chunks_start += self.chunk_size;
                    return Some(previous);
                }
                // cmk similar code elsewhere
                if self.before_start < self.before_end_exclusive {
                    let element = self.slice[self.before_start];
                    self.before_start += 1;
                    self.chunks_start += self.chunk_size;
                    return Some(element..=element);
                }
            }
            self.chunks_start += self.chunk_size;
        }

        // at the very, very end, flush previous.
        if let Some(previous) = &self.previous_range.take() {
            debug_assert!(self.previous_range.is_none());
            return Some(previous.clone());
        }

        self.before_start = self.remainder_start;
        self.before_end_exclusive = self.remainder_end_exclusive;
        self.remainder_start = self.remainder_end_exclusive;
        // cmk similar code elsewhere
        if self.before_start < self.before_end_exclusive {
            let element = self.slice[self.before_start];
            self.before_start += 1;
            Some(element..=element)
        } else {
            None
        }
    }

    // cmk
    // // We could have one less or one more than the iter.
    // fn size_hint(&self) -> (usize, Option<usize>) {
    //     let (low, high) = self.iter.size_hint();
    //     let low = if low > 0 { low - 1 } else { 0 };
    //     let high = high.map(|high| {
    //         if high < usize::MAX {
    //             high + 1
    //         } else {
    //             usize::MAX
    //         }
    //     });
    //     (low, high)
    // }
}

// impl<T: Integer, I> ops::Not for NotIter<T, I>
// where
//     I: SortedDisjoint<T>,
// {
//     type Output = NotIter<T, Self>;

//     fn not(self) -> Self::Output {
//         // It would be fun to optimize to self.iter, but that would require
//         // also considering fields 'start_not' and 'next_time_return_none'.
//         self.complement()
//     }
// }

// impl<T: Integer, R, L> ops::BitOr<R> for NotIter<T, L>
// where
//     L: SortedDisjoint<T>,
//     R: SortedDisjoint<T>,
// {
//     type Output = BitOrMerge<T, Self, R>;

//     fn bitor(self, other: R) -> Self::Output {
//         SortedDisjoint::union(self, other)
//     }
// }

// impl<T: Integer, R, L> ops::Sub<R> for NotIter<T, L>
// where
//     L: SortedDisjoint<T>,
//     R: SortedDisjoint<T>,
// {
//     type Output = BitSubMerge<T, Self, R>;

//     fn sub(self, other: R) -> Self::Output {
//         // It would be fun to optimize !!self.iter into self.iter
//         // but that would require also considering fields 'start_not' and 'next_time_return_none'.
//         SortedDisjoint::difference(self, other)
//     }
// }

// impl<T: Integer, R, L> ops::BitXor<R> for NotIter<T, L>
// where
//     L: SortedDisjoint<T>,
//     R: SortedDisjoint<T>,
// {
//     type Output = BitXOrTee<T, Self, R>;

//     #[allow(clippy::suspicious_arithmetic_impl)]
//     fn bitxor(self, other: R) -> Self::Output {
//         // It would be fine optimize !!self.iter into self.iter, ala
//         // ¬(¬n ∨ ¬r) ∨ ¬(n ∨ r) // https://www.wolframalpha.com/input?i=%28not+n%29+xor+r
//         // but that would require also considering fields 'start_not' and 'next_time_return_none'.
//         SortedDisjoint::symmetric_difference(self, other)
//     }
// }

// impl<T: Integer, R, L> ops::BitAnd<R> for NotIter<T, L>
// where
//     L: SortedDisjoint<T>,
//     R: SortedDisjoint<T>,
// {
//     type Output = BitAndMerge<T, Self, R>;

//     fn bitand(self, other: R) -> Self::Output {
//         // It would be fun to optimize !!self.iter into self.iter
//         // but that would require also considering fields 'start_not' and 'next_time_return_none'.
//         SortedDisjoint::intersection(self, other)
//     }
// }

// // FUTURE define Not, etc on DynSortedDisjoint
