use std::ops::RangeInclusive;

use crate::{Integer, SortedDisjoint};

// cmk rule: Make structs clonable when possible.
/// An iterator that visits the ranges representing the complement,
/// i.e., all the integers not in the original iterator, as sorted & disjoint ranges.
#[derive(Clone)]
#[must_use = "iterators are lazy and do nothing unless consumed"]
pub struct NotIter<T, I>
where
    T: Integer,
    I: Iterator<Item = RangeInclusive<T>> + SortedDisjoint,
{
    iter: I,
    start_not: T,
    next_time_return_none: bool,
}

// cmk rule: Create a new function when setup is complicated and the function is used in multiple places.
impl<T, I> NotIter<T, I>
where
    T: Integer,
    I: Iterator<Item = RangeInclusive<T>> + SortedDisjoint,
{
    pub fn new<J>(into_iter: J) -> Self
    where
        J: IntoIterator<Item = RangeInclusive<T>, IntoIter = I>,
    {
        NotIter {
            iter: into_iter.into_iter(),
            start_not: T::min_value(),
            next_time_return_none: false,
        }
    }
}

// !!!cmk0 create coverage tests
impl<T, I> Iterator for NotIter<T, I>
where
    T: Integer,
    I: Iterator<Item = RangeInclusive<T>> + SortedDisjoint,
{
    type Item = RangeInclusive<T>;
    fn next(&mut self) -> Option<RangeInclusive<T>> {
        debug_assert!(T::min_value() <= T::max_value2()); // real assert
        if self.next_time_return_none {
            return None;
        }
        let next_item = self.iter.next();
        if let Some(range) = next_item {
            let (start, end) = range.into_inner();
            debug_assert!(start <= end && end <= T::max_value2());
            if self.start_not < start {
                // We can subtract with underflow worry because
                // we know that start > start_not and so not min_value
                let result = Some(self.start_not..=start - T::one());
                if end < T::max_value2() {
                    self.start_not = end + T::one();
                } else {
                    self.next_time_return_none = true;
                }
                result
            } else if end < T::max_value2() {
                self.start_not = end + T::one();
                self.next() // will recurse at most once
            } else {
                self.next_time_return_none = true;
                None
            }
        } else {
            self.next_time_return_none = true;
            Some(self.start_not..=T::max_value2())
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
