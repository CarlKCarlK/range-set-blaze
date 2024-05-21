#![allow(missing_docs)]
use crate::{
    map::{CloneRef, ValueRef},
    sorted_disjoint_map::{Priority, PrioritySortedStartsMap},
    Integer,
};
use alloc::collections::btree_map;
use core::{iter::FusedIterator, marker::PhantomData, ops::RangeInclusive};

use crate::{
    map::{EndValue, PartialEqClone},
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
#[allow(clippy::module_name_repetitions)]
pub struct RangeValuesIter<'a, T: Integer, V: PartialEqClone> {
    // cmk00 define a new
    pub(crate) iter: btree_map::Iter<'a, T, EndValue<T, V>>,
}

// cmk00 what is this for?
impl<T: Integer, V: PartialEqClone> AsRef<Self> for RangeValuesIter<'_, T, V> {
    fn as_ref(&self) -> &Self {
        // Self is RangeValuesIter<'a>, the type for which we impl AsRef
        self
    }
}

impl<T: Integer, V: PartialEqClone> ExactSizeIterator for RangeValuesIter<'_, T, V> {
    #[must_use]
    fn len(&self) -> usize {
        self.iter.len()
    }
}

impl<T: Integer, V: PartialEqClone> FusedIterator for RangeValuesIter<'_, T, V> {}

// Range's iterator is just the inside BTreeMap iterator as values
impl<'a, T, V> Iterator for RangeValuesIter<'a, T, V>
where
    T: Integer,
    V: PartialEqClone + 'a,
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
    V: PartialEqClone + 'a,
{
    fn next_back(&mut self) -> Option<Self::Item> {
        self.iter
            .next_back()
            .map(|(start, end_value)| (*start..=end_value.end, &end_value.value))
    }
}

// cmk must explain that this is not a SortedDisjointMap because it gives a value rather than a reference
#[must_use = "iterators are lazy and do nothing unless consumed"]
/// An iterator that moves out the ranges in the [`RangeSetBlaze`],
/// i.e., the integers as sorted & disjoint ranges.
///
/// This `struct` is created by the [`into_ranges`] method on [`RangeSetBlaze`]. See [`into_ranges`]'s
/// documentation for more.
///
/// [`RangeSetBlaze`]: crate::RangeSetBlaze
/// [`into_ranges`]: crate::RangeSetBlaze::into_ranges
pub struct IntoRangeValuesIter<T: Integer, V: PartialEqClone> {
    // cmk00 define a new
    pub(crate) iter: btree_map::IntoIter<T, EndValue<T, V>>,
}

impl<T: Integer, V: PartialEqClone> ExactSizeIterator for IntoRangeValuesIter<T, V> {
    #[must_use]
    fn len(&self) -> usize {
        self.iter.len()
    }
}

impl<T: Integer, V: PartialEqClone> FusedIterator for IntoRangeValuesIter<T, V> {}

impl<'a, T: Integer, V: PartialEqClone + 'a> Iterator for IntoRangeValuesIter<T, V> {
    type Item = (RangeInclusive<T>, V);

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|(start, end_value)| {
            let range = start..=end_value.end;
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
pub struct MapRangesIter<'a, T: Integer, V: PartialEqClone> {
    iter: btree_map::Iter<'a, T, EndValue<T, V>>,
    gather: Option<RangeInclusive<T>>,
}

impl<'a, T: Integer, V: PartialEqClone> MapRangesIter<'a, T, V> {
    pub const fn new(iter: btree_map::Iter<'a, T, EndValue<T, V>>) -> Self {
        MapRangesIter { iter, gather: None }
    }
}

// cmk00 what is this for?
impl<T: Integer, V: PartialEqClone> AsRef<Self> for MapRangesIter<'_, T, V> {
    fn as_ref(&self) -> &Self {
        // Self is MapRangesIter<'a>, the type for which we impl AsRef
        self
    }
}

impl<T: Integer, V: PartialEqClone> FusedIterator for MapRangesIter<'_, T, V> {}

// Range's iterator is just the inside BTreeMap iterator as values
impl<'a, T, V> Iterator for MapRangesIter<'a, T, V>
where
    T: Integer,
    V: PartialEqClone + 'a,
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
            if gather_end.add_one() == start_next {
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
pub struct MapIntoRangesIter<T: Integer, V: PartialEqClone> {
    iter: btree_map::IntoIter<T, EndValue<T, V>>,
    gather: Option<RangeInclusive<T>>,
}

impl<T: Integer, V: PartialEqClone> MapIntoRangesIter<T, V> {
    pub fn new(iter: btree_map::IntoIter<T, EndValue<T, V>>) -> Self {
        Self { iter, gather: None }
    }
}

impl<T: Integer, V: PartialEqClone> FusedIterator for MapIntoRangesIter<T, V> {}

impl<'a, T: Integer, V: PartialEqClone + 'a> Iterator for MapIntoRangesIter<T, V> {
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
            if gather_end.add_one() == start_next {
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
#[allow(clippy::module_name_repetitions)]
pub struct RangeValuesToRangesIter<T, VR, I>
where
    T: Integer,
    VR: CloneRef<VR::Value> + ValueRef,
    I: SortedDisjointMap<T, VR>,
{
    iter: I,
    gather: Option<RangeInclusive<T>>,
    phantom: PhantomData<VR>,
}

// implement exact size iterator for one special case
impl<'a, T> ExactSizeIterator for RangeValuesToRangesIter<T, &'a (), RangeValuesIter<'a, T, ()>>
where
    T: Integer,
{
    #[must_use]
    fn len(&self) -> usize {
        self.iter.len()
    }
}

impl<T, VR, I> FusedIterator for RangeValuesToRangesIter<T, VR, I>
where
    T: Integer,
    VR: CloneRef<VR::Value> + ValueRef,
    I: SortedDisjointMap<T, VR>,
{
}

impl<T, VR, I> RangeValuesToRangesIter<T, VR, I>
where
    T: Integer,
    VR: CloneRef<VR::Value> + ValueRef,
    I: SortedDisjointMap<T, VR>,
{
    /// Creates a new `RangeValuesToRangesIter` from an existing sorted disjoint map iterator.
    /// `option_ranges` is initialized as `None` by default.
    pub const fn new(iter: I) -> Self {
        Self {
            iter,
            gather: None,         // cmk rename "gather"?
            phantom: PhantomData, // cmk needed?
        }
    }
}

impl<T, VR, I> Iterator for RangeValuesToRangesIter<T, VR, I>
where
    T: Integer,
    VR: CloneRef<VR::Value> + ValueRef,
    I: SortedDisjointMap<T, VR>,
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
            if gather_end.add_one() == next_start {
                self.gather = Some(gather_start..=next_end);
                continue;
            }

            // They are disjoint, return gather and start a new gather.
            self.gather = Some(next_start..=next_end);
            return Some(gather_start..=gather_end);
        }
    }
}

#[allow(clippy::redundant_pub_crate)]
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
pub struct SetPriorityMap<T, VR, I>
where
    T: Integer,
    VR: CloneRef<VR::Value> + ValueRef,
    I: SortedDisjointMap<T, VR>,
{
    iter: I,
    priority_number: usize,
    phantom_data: PhantomData<(T, VR)>,
}

impl<T, VR, I> FusedIterator for SetPriorityMap<T, VR, I>
where
    T: Integer,
    VR: CloneRef<VR::Value> + ValueRef,
    I: SortedDisjointMap<T, VR>,
{
}

impl<T, VR, I> Iterator for SetPriorityMap<T, VR, I>
where
    T: Integer,
    VR: CloneRef<VR::Value> + ValueRef,
    I: SortedDisjointMap<T, VR>,
{
    type Item = Priority<T, VR>;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter
            .next()
            .map(|range_value| Priority::new(range_value, self.priority_number))
    }
}

impl<T, VR, I> SetPriorityMap<T, VR, I>
where
    T: Integer,
    VR: CloneRef<VR::Value> + ValueRef,
    I: SortedDisjointMap<T, VR>,
{
    pub const fn new(iter: I, priority: usize) -> Self {
        Self {
            iter,
            priority_number: priority,
            phantom_data: PhantomData,
        }
    }
}

impl<T, VR, I> PrioritySortedStartsMap<T, VR> for SetPriorityMap<T, VR, I>
where
    T: Integer,
    VR: CloneRef<VR::Value> + ValueRef,
    I: SortedDisjointMap<T, VR>,
{
}
