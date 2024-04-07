use crate::range_values::ExpectDebugUnwrapRelease;
use crate::sorted_disjoint_map::{Priority, PrioritySortedStartsMap};
use crate::{
    map::{CloneBorrow, EndValue, ValueOwned},
    sorted_disjoint_map::SortedDisjointMap,
    Integer,
};
use core::ops::RangeInclusive;
use core::{
    cmp::{max, min},
    iter::FusedIterator,
    marker::PhantomData,
};
use num_traits::Zero;

#[must_use = "iterators are lazy and do nothing unless consumed"]
pub(crate) struct UnsortedPriorityDisjointMap<T, V, VR, I>
where
    T: Integer,
    V: ValueOwned,
    VR: CloneBorrow<V>,
    I: Iterator<Item = (RangeInclusive<T>, VR)>,
{
    iter: I,
    option_priority: Option<Priority<T, V, VR>>,
    min_value_plus_2: T,
    two: T,
    priority_number: usize,
}

impl<T, V, VR, I> UnsortedPriorityDisjointMap<T, V, VR, I>
where
    T: Integer,
    V: ValueOwned,
    VR: CloneBorrow<V>,
    I: Iterator<Item = (RangeInclusive<T>, VR)>, // Any iterator is fine
{
    pub fn new(into_iter: I) -> Self {
        UnsortedPriorityDisjointMap {
            iter: into_iter,
            option_priority: None,
            min_value_plus_2: T::min_value() + T::one() + T::one(),
            two: T::one() + T::one(),
            priority_number: 0,
        }
    }
}

// cmk
// impl<'a, T, V, VR, I> FusedIterator for UnsortedDisjointMap<'a, T, V, VR, I>
// where
//     T: Integer,
//     V: PartialEqClone + 'a,
//     I: Iterator<Item = (T,  VR)> + FusedIterator,
// {
// }

impl<T, V, VR, I> Iterator for UnsortedPriorityDisjointMap<T, V, VR, I>
where
    T: Integer,
    V: ValueOwned,
    VR: CloneBorrow<V>,
    I: Iterator<Item = (RangeInclusive<T>, VR)>,
{
    type Item = Priority<T, V, VR>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            // get the next range_value, if none, return the current range_value
            // cmk create a new range_value instead of modifying the existing one????
            let Some(next_range_value) = self.iter.next() else {
                return self.option_priority.take();
            };
            let next_priority = Priority::new(next_range_value, self.priority_number);
            self.priority_number = self
                .priority_number
                .checked_add(1)
                .expect_debug_unwrap_release("overflow");

            // check the next range is valid and non-empty
            let (next_start, next_end) = next_priority.start_and_end();
            assert!(
                next_end <= T::safe_max_value(),
                "end must be <= T::safe_max_value()"
            );
            if next_start > next_end {
                continue;
            }

            // get the current range (if none, set the current range to the next range and loop)
            let Some(mut current_priority) = self.option_priority.take() else {
                self.option_priority = Some(next_priority);
                continue;
            };

            // if the ranges do not touch or overlap, return the current range and set the current range to the next range
            let (current_start, current_end) = current_priority.start_and_end();
            if (next_start >= self.min_value_plus_2 && current_end <= next_start - self.two)
                || (current_start >= self.min_value_plus_2 && next_end <= current_start - self.two)
            {
                self.option_priority = Some(next_priority);
                return Some(current_priority);
            }

            // So, they touch or overlap.

            // cmk think about combining this with the previous if
            // if values are different, return the current range and set the current range to the next range
            if current_priority.value().borrow() != next_priority.value().borrow() {
                self.option_priority = Some(next_priority);
                return Some(current_priority);
            }

            // they touch or overlap and have the same value, so merge
            current_priority.set_range(min(current_start, next_start)..=max(current_end, next_end));
            self.option_priority = Some(current_priority);
            // continue;
        }
    }

    // As few as one (or zero if iter is empty) and as many as iter.len()
    // There could be one extra if option_range is Some.
    fn size_hint(&self) -> (usize, Option<usize>) {
        let (lower, upper) = self.iter.size_hint();
        let lower = if lower == 0 { 0 } else { 1 };
        if self.option_priority.is_some() {
            (lower, upper.map(|x| x + 1))
        } else {
            (lower, upper)
        }
    }
}

#[must_use = "iterators are lazy and do nothing unless consumed"]
pub(crate) struct SortedDisjointMapWithLenSoFar<T, V, VR, I>
where
    T: Integer,
    V: ValueOwned,
    VR: CloneBorrow<V>,
    I: SortedDisjointMap<T, V, VR>,
{
    iter: I,
    len: <T as Integer>::SafeLen,
    phantom_data: PhantomData<(V, VR)>,
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

impl<T, V, VR, I> SortedDisjointMapWithLenSoFar<T, V, VR, I>
where
    T: Integer,
    V: ValueOwned,
    VR: CloneBorrow<V>,
    I: SortedDisjointMap<T, V, VR>,
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

impl<T, V, VR, I> Iterator for SortedDisjointMapWithLenSoFar<T, V, VR, I>
where
    T: Integer,
    VR: CloneBorrow<V>,
    V: ValueOwned,
    I: SortedDisjointMap<T, V, VR>,
{
    type Item = (T, EndValue<T, V>);

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(range_value) = self.iter.next() {
            let (range, value) = range_value;
            let (start, end) = range.clone().into_inner();
            debug_assert!(start <= end && end <= T::safe_max_value());
            self.len += T::safe_len(&range);
            let end_value = EndValue {
                end,
                value: value.borrow_clone(),
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
pub struct AssumePrioritySortedStartsMap<T, V, VR, I>
where
    T: Integer,
    V: ValueOwned,
    VR: CloneBorrow<V>,
    I: Iterator<Item = Priority<T, V, VR>> + FusedIterator,
{
    iter: I,
}

impl<T, V, VR, I> PrioritySortedStartsMap<T, V, VR> for AssumePrioritySortedStartsMap<T, V, VR, I>
where
    T: Integer,
    V: ValueOwned,
    VR: CloneBorrow<V>,
    I: Iterator<Item = Priority<T, V, VR>> + FusedIterator,
{
}

impl<T, V, VR, I> AssumePrioritySortedStartsMap<T, V, VR, I>
where
    T: Integer,
    V: ValueOwned,
    VR: CloneBorrow<V>,
    I: Iterator<Item = Priority<T, V, VR>> + FusedIterator,
{
    pub fn new(iter: I) -> Self {
        AssumePrioritySortedStartsMap { iter }
    }
}

impl<T, V, VR, I> FusedIterator for AssumePrioritySortedStartsMap<T, V, VR, I>
where
    T: Integer,
    V: ValueOwned,
    VR: CloneBorrow<V>,
    I: Iterator<Item = Priority<T, V, VR>> + FusedIterator,
{
}

impl<T, V, VR, I> Iterator for AssumePrioritySortedStartsMap<T, V, VR, I>
where
    T: Integer,
    V: ValueOwned,
    VR: CloneBorrow<V>,
    I: Iterator<Item = Priority<T, V, VR>> + FusedIterator,
{
    type Item = Priority<T, V, VR>;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

impl<T: Integer, V: ValueOwned, VR, I> From<I>
    for SortedDisjointMapWithLenSoFar<T, V, VR, I::IntoIter>
where
    VR: CloneBorrow<V>,
    I: IntoIterator<Item = (RangeInclusive<T>, VR)>,
    I::IntoIter: SortedDisjointMap<T, V, VR>,
{
    fn from(into_iter: I) -> Self {
        SortedDisjointMapWithLenSoFar {
            iter: into_iter.into_iter(),
            len: <T as Integer>::SafeLen::zero(),
            phantom_data: PhantomData,
        }
    }
}
