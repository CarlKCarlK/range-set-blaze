use crate::range_values::ExpectDebugUnwrapRelease;
use crate::sorted_disjoint_map::{Priority, PrioritySortedStartsMap};
use crate::{
    map::{CloneBorrow, EndValue, ValueOwned},
    sorted_disjoint_map::{RangeValue, SortedDisjointMap},
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
pub(crate) struct UnsortedDisjointMap<T, V, VR, I>
where
    T: Integer,
    V: ValueOwned,
    VR: CloneBorrow<V>,
    I: Iterator<Item = RangeValue<T, V, VR>>,
{
    iter: I,
    option_priority: Option<Priority<T, V, VR>>,
    min_value_plus_2: T,
    two: T,
    priority: usize,
}

impl<T, V, VR, I> UnsortedDisjointMap<T, V, VR, I>
where
    T: Integer,
    V: ValueOwned,
    VR: CloneBorrow<V>,
    I: Iterator<Item = RangeValue<T, V, VR>>, // Any iterator is fine
{
    pub fn new(into_iter: I) -> Self {
        UnsortedDisjointMap {
            iter: into_iter,
            option_priority: None,
            min_value_plus_2: T::min_value() + T::one() + T::one(),
            two: T::one() + T::one(),
            priority: usize::MAX,
        }
    }
}

// cmk
// impl<'a, T, V, VR, I> FusedIterator for UnsortedDisjointMap<'a, T, V, VR, I>
// where
//     T: Integer,
//     V: PartialEqClone + 'a,
//     I: Iterator<Item = RangeValue<T, V, VR>> + FusedIterator,
// {
// }

impl<T, V, VR, I> Iterator for UnsortedDisjointMap<T, V, VR, I>
where
    T: Integer,
    V: ValueOwned,
    VR: CloneBorrow<V>,
    I: Iterator<Item = RangeValue<T, V, VR>>,
{
    type Item = Priority<T, V, VR>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            // get the next range_value, if none, return the current range_value
            // cmk create a new range_value instead of modifying the existing one????
            let Some(next_range_value) = self.iter.next() else {
                return self.option_priority.take();
            };
            let next_priority = Priority::new(next_range_value, self.priority);
            self.priority = self
                .priority
                .checked_sub(1)
                .expect_debug_unwrap_release("overflow");

            // check the next range is valid and non-empty
            let (next_start, next_end) = next_priority.range_value.range.clone().into_inner();
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
            let (current_start, current_end) =
                current_priority.range_value.range.clone().into_inner();
            if (next_start >= self.min_value_plus_2 && current_end <= next_start - self.two)
                || (current_start >= self.min_value_plus_2 && next_end <= current_start - self.two)
            {
                self.option_priority = Some(next_priority);
                return Some(current_priority);
            }

            // So, they touch or overlap.

            // cmk think about combining this with the previous if
            // if values are different, return the current range and set the current range to the next range
            if current_priority.range_value.value.borrow()
                != next_priority.range_value.value.borrow()
            {
                self.option_priority = Some(next_priority);
                return Some(current_priority);
            }

            // they touch or overlap and have the same value, so merge
            current_priority.range_value.range =
                min(current_start, next_start)..=max(current_end, next_end);
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
pub(crate) struct SortedDisjointWithLenSoFarMap<T, V, VR, I>
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

impl<T, V, VR, I> SortedDisjointWithLenSoFarMap<T, V, VR, I>
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

impl<T, V, VR, I> Iterator for SortedDisjointWithLenSoFarMap<T, V, VR, I>
where
    T: Integer,
    VR: CloneBorrow<V>,
    V: ValueOwned,
    I: SortedDisjointMap<T, V, VR>,
{
    type Item = (T, EndValue<T, V>);

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(range_value) = self.iter.next() {
            let (start, end) = range_value.range.clone().into_inner();
            debug_assert!(start <= end && end <= T::safe_max_value());
            self.len += T::safe_len(&range_value.range);
            let end_value = EndValue {
                end,
                value: range_value.value.borrow_clone(),
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
    for SortedDisjointWithLenSoFarMap<T, V, VR, I::IntoIter>
where
    VR: CloneBorrow<V>,
    I: IntoIterator<Item = RangeValue<T, V, VR>>,
    I::IntoIter: SortedDisjointMap<T, V, VR>,
{
    fn from(into_iter: I) -> Self {
        SortedDisjointWithLenSoFarMap {
            iter: into_iter.into_iter(),
            len: <T as Integer>::SafeLen::zero(),
            phantom_data: PhantomData,
        }
    }
}

/// cmk doc mostly for internal use. Does not check that the ranges are disjoint and sorted.
pub struct TupleToRangeValueIter<'a, T, V, I>
where
    T: Integer,
    V: ValueOwned + 'a,
    I: Iterator<Item = (RangeInclusive<T>, &'a V)>,
{
    iter: I,
}

impl<'a, T, V, I> Iterator for TupleToRangeValueIter<'a, T, V, I>
where
    T: Integer,
    V: ValueOwned + 'a,
    I: Iterator<Item = (RangeInclusive<T>, &'a V)>,
{
    type Item = RangeValue<T, V, &'a V>;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter
            .next()
            .map(|(range, value)| RangeValue::new(range, value))
    }
}

/// Gives any iterator of cmk implements the [`SortedDisjointMap`] trait without any checking.
// cmk0 why was this hidden? check for others#[doc(hidden)]
/// doc
pub struct CheckSortedDisjointMap<T, V, VR, I>
where
    T: Integer,
    V: ValueOwned,
    VR: CloneBorrow<V>,
    I: Iterator<Item = RangeValue<T, V, VR>>,
{
    iter: I,
    seen_none: bool,
    previous: Option<RangeValue<T, V, VR>>,
}

// define new
impl<T, V, VR, I> CheckSortedDisjointMap<T, V, VR, I>
where
    T: Integer,
    V: ValueOwned,
    VR: CloneBorrow<V>,
    I: Iterator<Item = RangeValue<T, V, VR>>,
{
    pub fn new(iter: I) -> Self {
        CheckSortedDisjointMap {
            iter,
            seen_none: false,
            previous: None,
        }
    }
}

impl<'a, T, V, J> CheckSortedDisjointMap<T, V, &'a V, TupleToRangeValueIter<'a, T, V, J>>
where
    T: Integer,
    V: ValueOwned,
    J: Iterator<Item = (RangeInclusive<T>, &'a V)>,
{
    pub fn from_tuples(iter: J) -> Self {
        let iter = TupleToRangeValueIter { iter };
        CheckSortedDisjointMap::new(iter)
    }
}

// implement fused
impl<T, V, VR, I> FusedIterator for CheckSortedDisjointMap<T, V, VR, I>
where
    T: Integer,
    V: ValueOwned,
    VR: CloneBorrow<V>,
    I: Iterator<Item = RangeValue<T, V, VR>>,
{
}

// implement iterator
impl<T, V, VR, I> Iterator for CheckSortedDisjointMap<T, V, VR, I>
where
    T: Integer,
    V: ValueOwned,
    VR: CloneBorrow<V>,
    I: Iterator<Item = RangeValue<T, V, VR>>,
{
    type Item = RangeValue<T, V, VR>;

    fn next(&mut self) -> Option<Self::Item> {
        let range_value = self.iter.next();
        let Some(range_value) = range_value else {
            self.seen_none = true;
            return None;
        };
        // cmk should test all these
        assert!(!self.seen_none, "A value must not be returned after None");
        let Some(previous) = self.previous.take() else {
            self.previous = Some(range_value.clone());
            return Some(range_value);
        };

        let (previous_start, previous_end) = previous.range.clone().into_inner();
        let (start, end) = range_value.range.clone().into_inner();
        assert!(start <= end, "Start must be <= end.",);
        assert!(
            end <= T::safe_max_value(),
            "End must be <= T::safe_max_value()"
        );
        assert!(previous_end < start, "Ranges must be disjoint and sorted");
        if previous_end + T::one() == start {
            assert!(
                previous.value.borrow() != range_value.value.borrow(),
                "Touching ranges must have different values"
            );
        }
        self.previous = Some(range_value.clone());
        Some(range_value.clone())
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

// // cmk00 check
// // cmk00 make Fused but don't require it

// cmk000 CheckSortedDisjointMap should have a Default
