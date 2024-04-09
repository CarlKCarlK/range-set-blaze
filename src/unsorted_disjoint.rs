// use crate::{map::EndValue, Integer};
// use crate::{SortedDisjoint, SortedStarts};
// use core::cmp::{max, min};
// use core::iter::FusedIterator;
// use core::ops::RangeInclusive;
// use num_traits::Zero;

// #[must_use = "iterators are lazy and do nothing unless consumed"]
// pub(crate) struct UnsortedDisjoint<T, I>
// where
//     T: Integer,
//     I: Iterator<Item = RangeInclusive<T>>,
// {
//     iter: I,
//     option_range: Option<RangeInclusive<T>>,
//     min_value_plus_2: T,
//     two: T,
// }

// impl<T, I> UnsortedDisjoint<T, I>
// where
//     T: Integer,

//     I: Iterator<Item = RangeInclusive<T>>, // Any iterator is fine
// {
//     pub fn new(into_iter: I) -> Self {
//         UnsortedDisjoint {
//             iter: into_iter,
//             option_range: None,
//             min_value_plus_2: T::min_value() + T::one() + T::one(),
//             two: T::one() + T::one(),
//         }
//     }
// }

// impl<T, I> Iterator for UnsortedDisjoint<T, I>
// where
//     T: Integer,
//     I: Iterator<Item = RangeInclusive<T>>,
// {
//     type Item = RangeInclusive<T>;

//     fn next(&mut self) -> Option<Self::Item> {
//         loop {
//             // get the next range_value, if none, return the current range_value
//             // cmk create a new range_value instead of modifying the existing one????
//             let Some(next_range) = self.iter.next() else {
//                 return self.option_range.take();
//             };
//             let next_range = next_range;
//             // check the next range is valid and non-empty
//             let (next_start, next_end) = next_range.clone().into_inner();
//             assert!(
//                 next_end <= T::safe_max_value(),
//                 "end must be <= T::safe_max_value()"
//             );
//             if next_start > next_end {
//                 continue;
//             }

//             // get the current range (if none, set the current range to the next range and loop)
//             let Some(mut current_range) = self.option_range.take() else {
//                 self.option_range = Some(next_range);
//                 continue;
//             };

//             // if the ranges do not touch or overlap, return the current range and set the current range to the next range
//             let (current_start, current_end) = current_range.clone().into_inner();

//             // cmk0000 simplify
//             if (next_start >= self.min_value_plus_2 && current_end <= next_start - self.two)
//                 || (current_start >= self.min_value_plus_2 && next_end <= current_start - self.two)
//             {
//                 self.option_range = Some(next_range);
//                 return Some(current_range);
//             }

//             // So, they touch or overlap.

//             // they touch or overlap and have the same value, so merge
//             current_range = min(current_start, next_start)..=max(current_end, next_end);
//             // continue;
//         }
//     }

//     // As few as one (or zero if iter is empty) and as many as iter.len()
//     // There could be one extra if option_range is Some.
//     fn size_hint(&self) -> (usize, Option<usize>) {
//         let (lower, upper) = self.iter.size_hint();
//         let lower = if lower == 0 { 0 } else { 1 };
//         if self.option_range.is_some() {
//             (lower, upper.map(|x| x + 1))
//         } else {
//             (lower, upper)
//         }
//     }
// }

// #[derive(Clone, Debug)]
// #[must_use = "iterators are lazy and do nothing unless consumed"]

// /// Gives any iterator of ranges the [`SortedStartsMap`] trait without any checking.
// #[doc(hidden)]
// pub struct AssumeSortedStarts<T, I>
// where
//     T: Integer,
//     I: Iterator<Item = RangeInclusive<T>> + FusedIterator,
// {
//     iter: I,
// }

// impl<T, I> SortedStarts<T> for AssumeSortedStarts<T, I>
// where
//     T: Integer,
//     I: Iterator<Item = RangeInclusive<T>> + FusedIterator,
// {
// }

// impl<T, I> AssumeSortedStarts<T, I>
// where
//     T: Integer,
//     I: Iterator<Item = RangeInclusive<T>> + FusedIterator,
// {
//     pub fn new(iter: I) -> Self {
//         AssumeSortedStarts { iter }
//     }
// }

// impl<T, I> FusedIterator for AssumeSortedStarts<T, I>
// where
//     T: Integer,
//     I: Iterator<Item = RangeInclusive<T>> + FusedIterator,
// {
// }

// impl<T, I> Iterator for AssumeSortedStarts<T, I>
// where
//     T: Integer,
//     I: Iterator<Item = RangeInclusive<T>> + FusedIterator,
// {
//     type Item = RangeInclusive<T>;

//     fn next(&mut self) -> Option<Self::Item> {
//         self.iter.next()
//     }

//     fn size_hint(&self) -> (usize, Option<usize>) {
//         self.iter.size_hint()
//     }
// }

// #[must_use = "iterators are lazy and do nothing unless consumed"]
// pub(crate) struct SortedDisjointWithLenSoFar<T, I>
// where
//     T: Integer,
//     I: SortedDisjoint<T>,
// {
//     iter: I,
//     len: <T as Integer>::SafeLen,
// }
// impl<T, I> SortedDisjointWithLenSoFar<T, I>
// where
//     T: Integer,
//     I: SortedDisjoint<T>,
// {
//     pub fn len_so_far(&self) -> <T as Integer>::SafeLen {
//         self.len
//     }
// }

// // cmk
// // impl<T: Integer, V: PartialEqClone, I> FusedIterator for SortedDisjointWithLenSoFar<T, V, I> where
// //     I: SortedDisjoint<T, V> + FusedIterator
// // {
// // }

// impl<T, I> Iterator for SortedDisjointWithLenSoFar<T, I>
// where
//     T: Integer,
//     I: SortedDisjoint<T>,
// {
//     type Item = (T, EndValue<T, ()>);

//     fn next(&mut self) -> Option<Self::Item> {
//         if let Some(range) = self.iter.next() {
//             let (start, end) = range.clone().into_inner();
//             debug_assert!(start <= end && end <= T::safe_max_value());
//             self.len += T::safe_len(&range);
//             let end_value = EndValue { end, value: () };
//             Some((start, end_value))
//         } else {
//             None
//         }
//     }
//     fn size_hint(&self) -> (usize, Option<usize>) {
//         self.iter.size_hint()
//     }
// }

// impl<T: Integer, I> From<I> for SortedDisjointWithLenSoFar<T, I::IntoIter>
// where
//     I: IntoIterator<Item = RangeInclusive<T>>,
//     I::IntoIter: SortedDisjoint<T>,
// {
//     fn from(into_iter: I) -> Self {
//         SortedDisjointWithLenSoFar {
//             iter: into_iter.into_iter(),
//             len: <T as Integer>::SafeLen::zero(),
//         }
//     }
// }
use crate::{Integer, RangeSetBlaze, SortedDisjoint, SortedStarts};
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

// cmk000 rename "From" to "new"
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
            let range = match self.iter.next() {
                Some(r) => r,
                None => return self.option_range.take(),
            };

            let (next_start, next_end) = range.into_inner();
            if next_start > next_end {
                continue;
            }
            assert!(
                next_end <= T::safe_max_value(),
                "end must be <= T::safe_max_value()"
            );

            let Some(self_range) = self.option_range.clone() else {
                self.option_range = Some(next_start..=next_end);
                continue;
            };

            let (self_start, self_end) = self_range.into_inner();
            if (next_start >= self.min_value_plus_2 && self_end <= next_start - self.two)
                || (self_start >= self.min_value_plus_2 && next_end <= self_start - self.two)
            {
                let result = Some(self_start..=self_end);
                self.option_range = Some(next_start..=next_end);
                return result;
            } else {
                self.option_range = Some(min(self_start, next_start)..=max(self_end, next_end));
                continue;
            }
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

impl<T, I> FusedIterator for AssumeSortedStarts<T, I>
where
    T: Integer,
    I: Iterator<Item = RangeInclusive<T>> + FusedIterator,
{
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
        RangeSetBlaze::from_sorted_starts(self)
    }
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
