use core::{iter::FusedIterator, ops::RangeInclusive};

use alloc::collections::btree_map;

use crate::{
    Integer, SortedDisjointMap,
    map::{EndValue, ValueCarrier},
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
pub struct IterMap<T, VC, I> {
    iter: I,
    option_range_value_front: Option<(RangeInclusive<T>, VC)>,
    option_range_value_back: Option<(RangeInclusive<T>, VC)>,
}

impl<T, VC, I> IterMap<T, VC, I>
where
    T: Integer,
    VC: ValueCarrier,
    I: SortedDisjointMap<T, VC>,
{
    pub(crate) const fn new(iter: I) -> Self {
        Self {
            iter,
            option_range_value_front: None,
            option_range_value_back: None,
        }
    }
}

impl<T, VC, I> FusedIterator for IterMap<T, VC, I>
where
    T: Integer,
    VC: ValueCarrier,
    I: SortedDisjointMap<T, VC> + FusedIterator,
{
}

impl<T, VC, I> Iterator for IterMap<T, VC, I>
where
    T: Integer,
    VC: ValueCarrier,
    I: SortedDisjointMap<T, VC>,
{
    type Item = (T, VC);

    fn next(&mut self) -> Option<Self::Item> {
        let range_value = self
            .option_range_value_front
            .take()
            .or_else(|| self.iter.next())
            .or_else(|| self.option_range_value_back.take())?;

        let (mut range, value) = range_value;
        let (start, end) = range.into_inner();
        debug_assert!(start <= end);
        if start < end {
            range = start.add_one()..=end;
            self.option_range_value_front = Some((range, value.clone()));
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

impl<T, VC, I> DoubleEndedIterator for IterMap<T, VC, I>
where
    T: Integer,
    VC: ValueCarrier,
    I: SortedDisjointMap<T, VC> + DoubleEndedIterator,
{
    fn next_back(&mut self) -> Option<Self::Item> {
        let range_value = self
            .option_range_value_back
            .take()
            .or_else(|| self.iter.next_back())
            .or_else(|| self.option_range_value_front.take())?;
        let (mut range, value) = range_value;
        let (start, end) = range.into_inner();
        debug_assert!(start <= end);
        if start < end {
            range = start..=end.sub_one();
            self.option_range_value_back = Some((range, value.clone()));
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
pub struct IntoIterMap<T, V> {
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
