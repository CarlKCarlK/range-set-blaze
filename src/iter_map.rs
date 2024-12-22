// cmk check list
// main and into_
// iter, values, keys
// exact size iterator, double ended iterator, fused iterator, size_hint
// document the exact size and double ended

use core::{iter::FusedIterator, ops::RangeInclusive};

use alloc::collections::btree_map;

use crate::{
    map::{EndValue, ValueRef},
    Integer, SortedDisjointMap,
};

/// An iterator over the integer elements of a [`RangeMapBlaze`]. Double-ended.
///
/// This `struct` is created by the [`iter`] method on [`RangeMapBlaze`]. See its
/// documentation for more.
///
/// [`RangeMapBlaze`]: crate::map::RangeMapBlaze
/// [`iter`]: crate::RangeMapBlaze::iter
#[must_use = "iterators are lazy and do nothing unless consumed"]
#[derive(Clone, Debug)]
pub struct IterMap<T, VR, I>
where
    T: Integer,
    VR: ValueRef,
    I: SortedDisjointMap<T, VR>,
{
    iter: I,
    option_range_value_front: Option<(RangeInclusive<T>, VR)>,
    option_range_value_back: Option<(RangeInclusive<T>, VR)>,
}

impl<T, VR, I> IterMap<T, VR, I>
where
    T: Integer,
    VR: ValueRef,
    I: SortedDisjointMap<T, VR>,
{
    pub(crate) const fn new(iter: I) -> Self {
        Self {
            iter,
            option_range_value_front: None,
            option_range_value_back: None,
        }
    }
}

impl<T, VR, I> FusedIterator for IterMap<T, VR, I>
where
    T: Integer,
    VR: ValueRef,
    I: SortedDisjointMap<T, VR> + FusedIterator,
{
}

impl<T, VR, I> Iterator for IterMap<T, VR, I>
where
    T: Integer,
    VR: ValueRef,
    I: SortedDisjointMap<T, VR>,
{
    type Item = (T, VR);

    fn next(&mut self) -> Option<Self::Item> {
        let mut range_value = self
            .option_range_value_front
            .take()
            .or_else(|| self.iter.next())
            .or_else(|| self.option_range_value_back.take())?;

        let (start, end) = range_value.0.into_inner();
        debug_assert!(start <= end);
        let value = range_value.1.clone();
        if start < end {
            range_value.0 = start.add_one()..=end;
            self.option_range_value_front = Some(range_value);
        }
        Some((start, value))
    }

    // We'll have at least as many integers as intervals. There could be more that usize MAX
    // The option_range field could increase the number of integers, but we can ignore that.
    fn size_hint(&self) -> (usize, Option<usize>) {
        let (low, _high) = self.iter.size_hint();
        (low, None)
    }
}

impl<T, VR, I> DoubleEndedIterator for IterMap<T, VR, I>
where
    T: Integer,
    VR: ValueRef,
    I: SortedDisjointMap<T, VR> + DoubleEndedIterator,
{
    fn next_back(&mut self) -> Option<Self::Item> {
        let mut range_value = self
            .option_range_value_back
            .take()
            .or_else(|| self.iter.next_back())
            .or_else(|| self.option_range_value_front.take())?;
        let (start, end) = range_value.0.into_inner();
        debug_assert!(start <= end);
        let value = range_value.1.clone();
        if start < end {
            range_value.0 = start..=end.sub_one();
            self.option_range_value_back = Some(range_value);
        }

        Some((end, value))
    }
}

/// An iterator over the integer elements of a [`RangeMapBlaze`]. Double-ended.
///
/// This `struct` is created by the [`into_iter`] method on [`RangeMapBlaze`]. See its
/// documentation for more.
///
/// [`RangeMapBlaze`]: crate::map::RangeMapBlaze
/// [`into_iter`]: crate::RangeMapBlaze::into_iter
#[derive(Debug)]
#[must_use = "iterators are lazy and do nothing unless consumed"]
pub struct IntoIterMap<T, V>
where
    T: Integer,
    V: Eq + Clone,
{
    option_start_end_value_front: Option<(T, EndValue<T, V>)>,
    option_start_end_value_back: Option<(T, EndValue<T, V>)>,
    into_iter: btree_map::IntoIter<T, EndValue<T, V>>,
}

impl<T, V> IntoIterMap<T, V>
where
    T: Integer,
    V: Eq + Clone,
{
    pub(crate) const fn new(into_iter: btree_map::IntoIter<T, EndValue<T, V>>) -> Self {
        Self {
            option_start_end_value_front: None,
            option_start_end_value_back: None,
            into_iter,
        }
    }
}

impl<T, V> FusedIterator for IntoIterMap<T, V>
where
    T: Integer,
    V: Eq + Clone,
{
}

impl<T, V> Iterator for IntoIterMap<T, V>
where
    T: Integer,
    V: Eq + Clone,
{
    type Item = (T, V);

    fn next(&mut self) -> Option<Self::Item> {
        let (start, end_value) = self
            .option_start_end_value_front
            .take()
            .or_else(|| self.into_iter.next())
            .or_else(|| self.option_start_end_value_back.take())?;

        let end = end_value.end;
        let value = end_value.value.clone();
        debug_assert!(start <= end);
        if start < end {
            let start_plus1_end_value = (start.add_one(), end_value);
            self.option_start_end_value_front = Some(start_plus1_end_value);
        }
        Some((start, value))
    }

    // We'll have at least as many integers as intervals. There could be more that usize MAX
    // the option_range field could increase the number of integers, but we can ignore that.
    fn size_hint(&self) -> (usize, Option<usize>) {
        let (low, _high) = self.into_iter.size_hint();
        (low, None)
    }
}

impl<T, V> DoubleEndedIterator for IntoIterMap<T, V>
where
    T: Integer,
    V: Eq + Clone,
{
    fn next_back(&mut self) -> Option<Self::Item> {
        let (start, mut end_value) = self
            .option_start_end_value_back
            .take()
            .or_else(|| self.into_iter.next_back())
            .or_else(|| self.option_start_end_value_front.take())?;

        let end = end_value.end;
        let value = end_value.value.clone();
        debug_assert!(start <= end);

        if start < end {
            end_value.end.assign_sub_one();
            let start_end_less1_value = (start, end_value);
            self.option_start_end_value_back = Some(start_end_less1_value);
        }

        Some((end, value))
    }
}
