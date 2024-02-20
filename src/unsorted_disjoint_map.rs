use crate::{
    map::{EndValue, ValueOwned},
    sorted_disjoint_map::{RangeValue, SortedDisjointMap, SortedStartsMap},
    Integer,
};
use core::{
    cmp::{max, min},
    iter::FusedIterator,
    marker::PhantomData,
};
use num_traits::Zero;

#[must_use = "iterators are lazy and do nothing unless consumed"]
pub(crate) struct UnsortedDisjointMap<'a, T, V, I>
where
    T: Integer,
    V: ValueOwned + 'a,
    I: Iterator<Item = RangeValue<'a, T, V>>,
{
    iter: I,
    option_range_value: Option<RangeValue<'a, T, V>>,
    min_value_plus_1: T,
    min_value_plus_2: T,
    two: T,
    priority: usize,
}

impl<'a, T, V, I> From<I> for UnsortedDisjointMap<'a, T, V, I::IntoIter>
where
    T: Integer,
    V: ValueOwned + 'a,
    I: IntoIterator<Item = RangeValue<'a, T, V>>, // Any iterator is fine
{
    fn from(into_iter: I) -> Self {
        UnsortedDisjointMap {
            iter: into_iter.into_iter(),
            option_range_value: None,
            min_value_plus_1: T::min_value() + T::one(),
            min_value_plus_2: T::min_value() + T::one() + T::one(),
            two: T::one() + T::one(),
            priority: 0,
        }
    }
}

// cmk
// impl<'a, T, V, VR, I> FusedIterator for UnsortedDisjointMap<'a, T, V, VR, I>
// where
//     T: Integer,
//     V: PartialEqClone + 'a,
//     I: Iterator<Item = RangeValue<'a, T, V>> + FusedIterator,
// {
// }

impl<'a, T, V, I> Iterator for UnsortedDisjointMap<'a, T, V, I>
where
    T: Integer,
    V: ValueOwned + 'a,
    I: Iterator<Item = RangeValue<'a, T, V>>,
{
    type Item = RangeValue<'a, T, V>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            // get the next range_value, if none, return the current range_value
            let Some(mut next_range_value) = self.iter.next() else {
                return self.option_range_value.take();
            };
            next_range_value.priority = self.priority;
            self.priority += 1;

            // check the next range is valid and non-empty
            let (next_start, next_end) = next_range_value.range.clone().into_inner();
            assert!(
                next_end <= T::safe_max_value(),
                "end must be <= T::safe_max_value()"
            );
            if next_start > next_end {
                continue;
            }

            // get the current range (if none, set the current range to the next range and loop)
            let Some(mut current_range_value) = self.option_range_value.take() else {
                self.option_range_value = Some(next_range_value);
                continue;
            };

            // if the ranges do not touch or overlap, return the current range and set the current range to the next range
            let (current_start, current_end) = current_range_value.range.clone().into_inner();
            if (next_start >= self.min_value_plus_2 && current_end <= next_start - self.two)
                || (current_start >= self.min_value_plus_2 && next_end <= current_start - self.two)
            {
                self.option_range_value = Some(next_range_value);
                return Some(current_range_value);
            }

            // So, they touch or overlap.

            // cmk think about combining this with the previous if
            // if values are different, return the current range and set the current range to the next range
            if current_range_value.value != next_range_value.value {
                self.option_range_value = Some(next_range_value);
                return Some(current_range_value);
            }

            // they touch or overlap and have the same value, so merge
            current_range_value.range = min(current_start, next_start)..=max(current_end, next_end);
            self.option_range_value = Some(current_range_value);
            // continue;
        }
    }

    // As few as one (or zero if iter is empty) and as many as iter.len()
    // There could be one extra if option_range is Some.
    fn size_hint(&self) -> (usize, Option<usize>) {
        let (lower, upper) = self.iter.size_hint();
        let lower = if lower == 0 { 0 } else { 1 };
        if self.option_range_value.is_some() {
            (lower, upper.map(|x| x + 1))
        } else {
            (lower, upper)
        }
    }
}

#[must_use = "iterators are lazy and do nothing unless consumed"]
pub(crate) struct SortedDisjointWithLenSoFarMap<'a, T, V, I>
where
    T: Integer,
    V: ValueOwned + 'a,
    I: SortedDisjointMap<'a, T, V>,
    <V as ToOwned>::Owned: PartialEq,
{
    iter: I,
    len: <T as Integer>::SafeLen,
    _phantom_data: PhantomData<&'a V>,
}

// cmk
// impl<T: Integer, V: PartialEqClone, I> From<I> for SortedDisjointWithLenSoFarMap<T, V, I::IntoIterMap>
// where
//     I: IntoIterator<Item = RangeValue<T, V>>,
//     I::IntoIter: SortedDisjointMap<T, V>,
// {
//     fn from(into_iter: I) -> Self {
//         SortedDisjointWithLenSoFarMap {
//             iter: into_iter.into_iter(),
//             len: <T as Integer>::SafeLen::zero(),
//             _phantom_data: PhantomData,
//         }
//     }
// }

impl<'a, T: Integer, V: ValueOwned + 'a, I> SortedDisjointWithLenSoFarMap<'a, T, V, I>
where
    I: SortedDisjointMap<'a, T, V>,
    <V as ToOwned>::Owned: PartialEq,
{
    pub fn len_so_far(&self) -> <T as Integer>::SafeLen {
        self.len
    }
}

// cmk
// impl<T: Integer, V: PartialEqClone, I> FusedIterator for SortedDisjointWithLenSoFarMap<T, V, I> where
//     I: SortedDisjointMap<T, V> + FusedIterator
// {
// }

impl<'a, T, V, I> Iterator for SortedDisjointWithLenSoFarMap<'a, T, V, I>
where
    T: Integer,
    V: ValueOwned + 'a,
    <V as ToOwned>::Owned: PartialEq,
    I: SortedDisjointMap<'a, T, V>,
{
    type Item = (T, EndValue<T, V>);

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(range_value) = self.iter.next() {
            let (start, end) = range_value.range.clone().into_inner();
            debug_assert!(start <= end && end <= T::safe_max_value());
            self.len += T::safe_len(&range_value.range);
            let end_value = EndValue {
                end,
                value: range_value.value.to_owned(),
            };
            Some((start, end_value))
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

/// Gives any iterator of ranges the [`SortedStartsMap`] trait without any checking.
#[doc(hidden)]
pub struct AssumeSortedStartsMap<'a, T, V, I>
where
    T: Integer,
    V: ValueOwned + 'a,
    I: Iterator<Item = RangeValue<'a, T, V>>,
{
    pub(crate) iter: I,
}

impl<'a, T: Integer, V: ValueOwned + 'a, I> SortedStartsMap<'a, T, V>
    for AssumeSortedStartsMap<'a, T, V, I>
where
    I: Iterator<Item = RangeValue<'a, T, V>>,
{
}

impl<'a, T, V, I> AssumeSortedStartsMap<'a, T, V, I>
where
    T: Integer,
    V: ValueOwned + 'a,
    I: Iterator<Item = RangeValue<'a, T, V>>,
{
    pub fn new(iter: I) -> Self {
        AssumeSortedStartsMap { iter }
    }
}

impl<'a, T, V, I> FusedIterator for AssumeSortedStartsMap<'a, T, V, I>
where
    T: Integer,
    V: ValueOwned + 'a,
    I: Iterator<Item = RangeValue<'a, T, V>> + FusedIterator,
{
}

impl<'a, T, V, I> Iterator for AssumeSortedStartsMap<'a, T, V, I>
where
    T: Integer,
    V: ValueOwned + 'a,
    I: Iterator<Item = RangeValue<'a, T, V>>,
{
    type Item = RangeValue<'a, T, V>;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

impl<'a, T: Integer, V: ValueOwned + 'a, I> From<I>
    for SortedDisjointWithLenSoFarMap<'a, T, V, I::IntoIter>
where
    I: IntoIterator<Item = RangeValue<'a, T, V>>,
    I::IntoIter: SortedDisjointMap<'a, T, V>,
    <V as ToOwned>::Owned: PartialEq,
{
    fn from(into_iter: I) -> Self {
        SortedDisjointWithLenSoFarMap {
            iter: into_iter.into_iter(),
            len: <T as Integer>::SafeLen::zero(),
            _phantom_data: PhantomData,
        }
    }
}
