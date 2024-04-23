#![allow(missing_docs)]
use crate::{
    map::CloneBorrow,
    sorted_disjoint_map::{Priority, PrioritySortedDisjointMap, PrioritySortedStartsMap},
    Integer,
};
use alloc::collections::btree_map;
use core::{iter::FusedIterator, marker::PhantomData, ops::RangeInclusive};

use crate::{
    map::{EndValue, ValueOwned},
    sorted_disjoint_map::SortedDisjointMap,
};

/// An iterator that visits the ranges in the [`RangeSetBlaze`],
/// i.e., the integers as sorted & disjoint ranges.
///
/// This `struct` is created by the [`ranges`] method on [`RangeSetBlaze`]. See [`ranges`]'s
/// documentation for more.
///
/// [`RangeSetBlaze`]: crate::RangeSetBlaze
/// [`ranges`]: crate::RangeSetBlaze::ranges
#[derive(Clone)]
#[must_use = "iterators are lazy and do nothing unless consumed"]
pub struct RangeValuesIter<'a, T: Integer, V: ValueOwned> {
    // cmk00 define a new
    pub(crate) iter: btree_map::Iter<'a, T, EndValue<T, V>>,
}

// cmk00 what is this for?
impl<'a, T: Integer, V: ValueOwned> AsRef<RangeValuesIter<'a, T, V>> for RangeValuesIter<'a, T, V> {
    fn as_ref(&self) -> &Self {
        // Self is RangeValuesIter<'a>, the type for which we impl AsRef
        self
    }
}

impl<T: Integer, V: ValueOwned> ExactSizeIterator for RangeValuesIter<'_, T, V> {
    #[must_use]
    fn len(&self) -> usize {
        self.iter.len()
    }
}

impl<T: Integer, V: ValueOwned> FusedIterator for RangeValuesIter<'_, T, V> {}

// Range's iterator is just the inside BTreeMap iterator as values
impl<'a, T, V> Iterator for RangeValuesIter<'a, T, V>
where
    T: Integer,
    V: ValueOwned + 'a,
{
    type Item = (RangeInclusive<T>, &'a V); // Assuming VR is always &'a V for next

    fn next(&mut self) -> Option<Self::Item> {
        self.iter
            .next()
            .map(|(start, end_value)| (*start..=end_value.end, &end_value.value))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

impl<'a, T, V> DoubleEndedIterator for RangeValuesIter<'a, T, V>
where
    T: Integer,
    V: ValueOwned + 'a,
{
    fn next_back(&mut self) -> Option<Self::Item> {
        self.iter
            .next_back()
            .map(|(start, end_value)| (*start..=end_value.end, &end_value.value))
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
pub struct IntoRangeValuesIter<T: Integer, V: ValueOwned> {
    // cmk00 define a new
    pub(crate) iter: btree_map::IntoIter<T, EndValue<T, V>>,
}

impl<T: Integer, V: ValueOwned> ExactSizeIterator for IntoRangeValuesIter<T, V> {
    #[must_use]
    fn len(&self) -> usize {
        self.iter.len()
    }
}

impl<T: Integer, V: ValueOwned> FusedIterator for IntoRangeValuesIter<T, V> {}

impl<'a, T: Integer, V: ValueOwned + 'a> Iterator for IntoRangeValuesIter<T, V> {
    type Item = (RangeInclusive<T>, V);

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|(start, end_value)| {
            let range = start..=end_value.end;
            // cmk don't use RangeValue here
            (range, end_value.value)
        })
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

/// An iterator that visits the ranges in the [`RangeSetBlaze`],
/// i.e., the integers as sorted & disjoint ranges.
///
/// This `struct` is created by the [`ranges`] method on [`RangeSetBlaze`]. See [`ranges`]'s
/// documentation for more.
///
/// [`RangeSetBlaze`]: crate::RangeSetBlaze
/// [`ranges`]: crate::RangeSetBlaze::ranges
#[derive(Clone)]
#[must_use = "iterators are lazy and do nothing unless consumed"]
pub struct MapRangesIter<'a, T: Integer, V: ValueOwned> {
    iter: btree_map::Iter<'a, T, EndValue<T, V>>,
    gather: Option<RangeInclusive<T>>,
}

impl<'a, T: Integer, V: ValueOwned> MapRangesIter<'a, T, V> {
    pub fn new(iter: btree_map::Iter<'a, T, EndValue<T, V>>) -> Self {
        MapRangesIter { iter, gather: None }
    }
}

// cmk00 what is this for?
impl<'a, T: Integer, V: ValueOwned> AsRef<MapRangesIter<'a, T, V>> for MapRangesIter<'a, T, V> {
    fn as_ref(&self) -> &Self {
        // Self is MapRangesIter<'a>, the type for which we impl AsRef
        self
    }
}

impl<T: Integer, V: ValueOwned> FusedIterator for MapRangesIter<'_, T, V> {}

// Range's iterator is just the inside BTreeMap iterator as values
impl<'a, T, V> Iterator for MapRangesIter<'a, T, V>
where
    T: Integer,
    V: ValueOwned + 'a,
{
    type Item = RangeInclusive<T>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            // If no next, return gather, if any.
            let Some((start, end_value)) = self.iter.next() else {
                return self.gather.take();
            };

            let (start_next, end_next) = (*start, end_value.end);
            debug_assert!(start_next <= end_next); // real assert

            // if not gather, start a new gather.
            let Some(gather) = self.gather.take() else {
                self.gather = Some(start_next..=end_next);
                continue;
            };

            let (gather_start, gather_end) = gather.into_inner();

            // if next is just touching gather, extend gather.
            if gather_end + T::one() == start_next {
                self.gather = Some(gather_start..=end_next);
                continue;
            }

            // they are disjoint, return gather and start a new gather.
            self.gather = Some(start_next..=end_next);
            return Some(gather_start..=gather_end);
        }
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
pub struct MapIntoRangesIter<T: Integer, V: ValueOwned> {
    iter: btree_map::IntoIter<T, EndValue<T, V>>,
    gather: Option<RangeInclusive<T>>,
}

impl<T: Integer, V: ValueOwned> MapIntoRangesIter<T, V> {
    pub fn new(iter: btree_map::IntoIter<T, EndValue<T, V>>) -> Self {
        MapIntoRangesIter { iter, gather: None }
    }
}

impl<T: Integer, V: ValueOwned> FusedIterator for MapIntoRangesIter<T, V> {}

impl<'a, T: Integer, V: ValueOwned + 'a> Iterator for MapIntoRangesIter<T, V> {
    type Item = RangeInclusive<T>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            // If no next, return gather, if any.
            let Some((start_next, end_value)) = self.iter.next() else {
                return self.gather.take();
            };

            let end_next = end_value.end;
            debug_assert!(start_next <= end_next); // real assert

            // if not gather, start a new gather.
            let Some(gather) = self.gather.take() else {
                self.gather = Some(start_next..=end_next);
                continue;
            };

            let (gather_start, gather_end) = gather.into_inner();

            // if next is just touching gather, extend gather.
            if gather_end + T::one() == start_next {
                self.gather = Some(gather_start..=end_next);
                continue;
            }

            // they are disjoint, return gather and start a new gather.
            self.gather = Some(start_next..=end_next);
            return Some(gather_start..=gather_end);
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

// cmk00 define a double ended iterator??

/// cmk
#[derive(Clone)]
#[must_use = "iterators are lazy and do nothing unless consumed"]
pub struct RangeValuesToRangesIter<T, V, VR, I>
where
    T: Integer,
    V: ValueOwned,
    VR: CloneBorrow<V>,
    I: SortedDisjointMap<T, V, VR>,
{
    iter: I,
    gather: Option<RangeInclusive<T>>,
    phantom: PhantomData<(V, VR)>,
}

// implement exact size iterator for one special case
impl<'a, T> ExactSizeIterator for RangeValuesToRangesIter<T, (), &'a (), RangeValuesIter<'a, T, ()>>
where
    T: Integer,
{
    #[must_use]
    fn len(&self) -> usize {
        self.iter.len()
    }
}

impl<T, V, VR, I> FusedIterator for RangeValuesToRangesIter<T, V, VR, I>
where
    T: Integer,
    V: ValueOwned,
    VR: CloneBorrow<V>,
    I: SortedDisjointMap<T, V, VR>,
{
}

impl<T, V, VR, I> RangeValuesToRangesIter<T, V, VR, I>
where
    T: Integer,
    V: ValueOwned,
    VR: CloneBorrow<V>,
    I: SortedDisjointMap<T, V, VR>,
{
    /// Creates a new `RangeValuesToRangesIter` from an existing sorted disjoint map iterator.
    /// `option_ranges` is initialized as `None` by default.
    pub fn new(iter: I) -> Self {
        Self {
            iter,
            gather: None,         // cmk rename "gather"?
            phantom: PhantomData, // cmk needed?
        }
    }
}

impl<T, V, VR, I> Iterator for RangeValuesToRangesIter<T, V, VR, I>
where
    T: Integer,
    V: ValueOwned,
    VR: CloneBorrow<V>,
    I: SortedDisjointMap<T, V, VR>,
{
    type Item = RangeInclusive<T>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            // If no next value, return gather, if any.
            let Some(next_range_value) = self.iter.next() else {
                return self.gather.take();
            };
            let (next_start, next_end) = next_range_value.0.into_inner();

            // If there is no gather, start a new gather.
            let Some(gather) = self.gather.take() else {
                self.gather = Some(next_start..=next_end);
                continue;
            };
            let (gather_start, gather_end) = gather.into_inner();

            // If next is just touching gather, extend gather.
            if gather_end + T::one() == next_start {
                self.gather = Some(gather_start..=next_end);
                continue;
            }

            // They are disjoint, return gather and start a new gather.
            self.gather = Some(next_start..=next_end);
            return Some(gather_start..=gather_end);
        }
    }
}

pub(crate) trait ExpectDebugUnwrapRelease<T> {
    fn expect_debug_unwrap_release(self, msg: &str) -> T;
}

#[allow(unused_variables)]
impl<T> ExpectDebugUnwrapRelease<T> for Option<T> {
    fn expect_debug_unwrap_release(self, msg: &str) -> T {
        #[cfg(debug_assertions)]
        {
            self.expect(msg)
        }
        #[cfg(not(debug_assertions))]
        {
            self.unwrap()
        }
    }
}
#[derive(Clone, Debug)]
pub struct SetPriorityMap<T, V, VR, I>
where
    T: Integer,
    V: ValueOwned,
    VR: CloneBorrow<V>,
    I: SortedDisjointMap<T, V, VR>,
{
    iter: I,
    priority_number: usize,
    phantom_data: PhantomData<(T, V, VR)>,
}

impl<T, V, VR, I> FusedIterator for SetPriorityMap<T, V, VR, I>
where
    T: Integer,
    V: ValueOwned,
    VR: CloneBorrow<V>,
    I: SortedDisjointMap<T, V, VR>,
{
}

impl<T, V, VR, I> Iterator for SetPriorityMap<T, V, VR, I>
where
    T: Integer,
    V: ValueOwned,
    VR: CloneBorrow<V>,
    I: SortedDisjointMap<T, V, VR>,
{
    type Item = Priority<T, V, VR>;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter
            .next()
            .map(|range_value| Priority::new(range_value, self.priority_number))
    }
}

impl<T, V, VR, I> SetPriorityMap<T, V, VR, I>
where
    T: Integer,
    V: ValueOwned,
    VR: CloneBorrow<V>,
    I: SortedDisjointMap<T, V, VR>,
{
    pub fn new(iter: I, priority: usize) -> Self {
        SetPriorityMap {
            iter,
            priority_number: priority,
            phantom_data: PhantomData,
        }
    }
}

impl<T, V, VR, I> PrioritySortedStartsMap<T, V, VR> for SetPriorityMap<T, V, VR, I>
where
    T: Integer,
    V: ValueOwned,
    VR: CloneBorrow<V>,
    I: SortedDisjointMap<T, V, VR>,
{
}
impl<T, V, VR, I> PrioritySortedDisjointMap<T, V, VR> for SetPriorityMap<T, V, VR, I>
where
    T: Integer,
    V: ValueOwned,
    VR: CloneBorrow<V>,
    I: SortedDisjointMap<T, V, VR>,
{
}
