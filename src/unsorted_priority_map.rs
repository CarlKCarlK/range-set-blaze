use crate::map::ValueRef;
use crate::range_values::ExpectDebugUnwrapRelease;
use crate::sorted_disjoint_map::{Priority, PrioritySortedStartsMap};
use crate::{Integer, map::EndValue, sorted_disjoint_map::SortedDisjointMap};
use core::ops::RangeInclusive;
use core::{
    cmp::{max, min},
    iter::FusedIterator,
    marker::PhantomData,
};
use num_traits::Zero;

#[must_use = "iterators are lazy and do nothing unless consumed"]
#[allow(clippy::redundant_pub_crate)]
pub(crate) struct UnsortedPriorityMap<T, VR, I>
where
    T: Integer,
    VR: ValueRef,
    I: Iterator<Item = (RangeInclusive<T>, VR)>,
{
    iter: I,
    option_priority: Option<Priority<T, VR>>,
    min_value_plus_2: T,
    priority_number: usize,
}

impl<T, VR, I> UnsortedPriorityMap<T, VR, I>
where
    T: Integer,
    VR: ValueRef,
    I: Iterator<Item = (RangeInclusive<T>, VR)>, // Any iterator is fine
{
    #[inline]
    pub(crate) fn new(into_iter: I) -> Self {
        Self {
            iter: into_iter,
            option_priority: None,
            min_value_plus_2: T::min_value().add_one().add_one(),
            priority_number: 0,
        }
    }
}

impl<T, VR, I> FusedIterator for UnsortedPriorityMap<T, VR, I>
where
    T: Integer,
    VR: ValueRef,
    I: Iterator<Item = (RangeInclusive<T>, VR)>,
{
}

impl<T, VR, I> Iterator for UnsortedPriorityMap<T, VR, I>
where
    T: Integer,
    VR: ValueRef,
    I: Iterator<Item = (RangeInclusive<T>, VR)>,
{
    type Item = Priority<T, VR>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            // Get the next range_value, if none, return the current range_value
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
            if next_start > next_end {
                continue;
            }

            // get the current range (if none, set the current range to the next range and loop)
            let Some(mut current_priority) = self.option_priority.take() else {
                self.option_priority = Some(next_priority);
                continue;
            };

            // If the values are different or the ranges do not touch or overlap,
            // return the current range and set the current range to the next range
            let (current_start, current_end) = current_priority.start_and_end();
            if current_priority.value().borrow() != next_priority.value().borrow()
                || (next_start >= self.min_value_plus_2
                    && current_end <= next_start.sub_one().sub_one())
                || (current_start >= self.min_value_plus_2
                    && next_end <= current_start.sub_one().sub_one())
            {
                self.option_priority = Some(next_priority);
                return Some(current_priority);
            }

            // They touch or overlap and have the same value, so merge
            current_priority.set_range(min(current_start, next_start)..=max(current_end, next_end));
            self.option_priority = Some(current_priority);

            // return to the top of the loop
        }
    }

    // As few as one (or zero if iter is empty) and as many as iter.len()
    // There could be one extra if option_range is Some.
    fn size_hint(&self) -> (usize, Option<usize>) {
        let (lower, upper) = self.iter.size_hint();
        let lower = min(lower, 1);
        if self.option_priority.is_some() {
            (lower, upper.map(|x| x + 1))
        } else {
            (lower, upper)
        }
    }
}

#[must_use = "iterators are lazy and do nothing unless consumed"]
#[allow(clippy::redundant_pub_crate)]
pub(crate) struct SortedDisjointMapWithLenSoFar<T, VR, I>
where
    T: Integer,
    VR: ValueRef,
    I: SortedDisjointMap<T, VR>,
{
    iter: I,
    len: <T as Integer>::SafeLen,
    phantom: PhantomData<VR>,
}

impl<T, VR, I> SortedDisjointMapWithLenSoFar<T, VR, I>
where
    T: Integer,
    VR: ValueRef,
    I: SortedDisjointMap<T, VR>,
{
    pub(crate) const fn len_so_far(&self) -> <T as Integer>::SafeLen {
        self.len
    }

    pub(crate) fn new(iter: I) -> Self {
        Self {
            iter,
            len: <T as Integer>::SafeLen::zero(),
            phantom: PhantomData,
        }
    }
}

impl<T, VR, I> FusedIterator for SortedDisjointMapWithLenSoFar<T, VR, I>
where
    T: Integer,
    VR: ValueRef,
    I: SortedDisjointMap<T, VR>,
{
}

impl<T, VR, I> Iterator for SortedDisjointMapWithLenSoFar<T, VR, I>
where
    T: Integer,
    VR: ValueRef,
    I: SortedDisjointMap<T, VR>,
{
    type Item = (T, EndValue<T, VR::Target>);

    fn next(&mut self) -> Option<Self::Item> {
        if let Some((range, value)) = self.iter.next() {
            let (start, end) = range.clone().into_inner();
            debug_assert!(start <= end);
            self.len += T::safe_len(&range);
            let end_value = EndValue {
                end,
                value: value.to_owned(),
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
/// Used internally by [`UnionIterMap`] and [`SymDiffIterMap`].
///
/// [`UnionIterMap`]: crate::UnionIterMap
/// [`SymDiffIterMap`]: crate::SymDiffIterMap
pub struct AssumePrioritySortedStartsMap<T, VR, I>
where
    T: Integer,
    VR: ValueRef,
    I: Iterator<Item = Priority<T, VR>> + FusedIterator,
{
    iter: I,
}

impl<T, VR, I> PrioritySortedStartsMap<T, VR> for AssumePrioritySortedStartsMap<T, VR, I>
where
    T: Integer,
    VR: ValueRef,
    I: Iterator<Item = Priority<T, VR>> + FusedIterator,
{
}

impl<T, VR, I> AssumePrioritySortedStartsMap<T, VR, I>
where
    T: Integer,
    VR: ValueRef,
    I: Iterator<Item = Priority<T, VR>> + FusedIterator,
{
    pub(crate) const fn new(iter: I) -> Self {
        Self { iter }
    }
}

impl<T, VR, I> FusedIterator for AssumePrioritySortedStartsMap<T, VR, I>
where
    T: Integer,
    VR: ValueRef,
    I: Iterator<Item = Priority<T, VR>> + FusedIterator,
{
}

impl<T, VR, I> Iterator for AssumePrioritySortedStartsMap<T, VR, I>
where
    T: Integer,
    VR: ValueRef,
    I: Iterator<Item = Priority<T, VR>> + FusedIterator,
{
    type Item = Priority<T, VR>;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}
