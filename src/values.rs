use core::iter::FusedIterator;
use std::collections::btree_map;

use crate::{
    iter_map::{IntoIterMap, IterMap},
    map::{EndValue, ValueRef},
    Integer, SortedDisjointMap,
};

/// A (double-ended) iterator over the values of a [`RangeMapBlaze`].
///
/// This `struct` is created by the [`values`] method on [`RangeMapBlaze`]. See its
/// documentation for more.
///
/// [`values`]: RangeMapBlaze::values
#[must_use = "iterators are lazy and do nothing unless consumed"]
#[derive(Clone, Debug)]
pub struct Values<T, VR, I>
where
    T: Integer,
    VR: ValueRef,
    I: SortedDisjointMap<T, VR>,
{
    iter: IterMap<T, VR, I>,
}

impl<T, VR, I> Values<T, VR, I>
where
    T: Integer,
    VR: ValueRef,
    I: SortedDisjointMap<T, VR>,
{
    pub const fn new(iter: I) -> Self {
        Self {
            iter: IterMap::new(iter),
        }
    }
}

impl<T, VR, I> FusedIterator for Values<T, VR, I>
where
    T: Integer,
    VR: ValueRef,
    I: SortedDisjointMap<T, VR> + FusedIterator,
{
}

impl<T, VR, I> Iterator for Values<T, VR, I>
where
    T: Integer,
    VR: ValueRef,
    I: SortedDisjointMap<T, VR>,
{
    type Item = VR;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|(_key, value)| value)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

impl<T, VR, I> DoubleEndedIterator for Values<T, VR, I>
where
    T: Integer,
    VR: ValueRef,
    I: SortedDisjointMap<T, VR> + DoubleEndedIterator,
{
    fn next_back(&mut self) -> Option<Self::Item> {
        self.iter.next_back().map(|(_key, value)| value)
    }
}

/// A (double-ended) iterator over the values of a [`RangeMapBlaze`].
///
/// This `struct` is created by the [`into_values`] method on [`RangeMapBlaze`]. See its
/// documentation for more.
///
/// [`into_values`]: RangeMapBlaze::into_values
#[must_use = "iterators are lazy and do nothing unless consumed"]
pub struct IntoValues<T, V>
where
    T: Integer,
    V: Eq + Clone,
{
    into_iter: IntoIterMap<T, V>,
}

impl<T, V> IntoValues<T, V>
where
    T: Integer,
    V: Eq + Clone,
{
    pub(crate) const fn new(btree_map_into_iter: btree_map::IntoIter<T, EndValue<T, V>>) -> Self {
        let into_iter = IntoIterMap::new(btree_map_into_iter);
        Self { into_iter }
    }
}

impl<T, V> Iterator for IntoValues<T, V>
where
    T: Integer,
    V: Eq + Clone,
{
    type Item = V;

    fn next(&mut self) -> Option<Self::Item> {
        self.into_iter.next().map(|(_key, value)| value)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.into_iter.size_hint()
    }
}

impl<T, V> FusedIterator for IntoValues<T, V>
where
    T: Integer,
    V: Eq + Clone,
{
}

impl<T, V> DoubleEndedIterator for IntoValues<T, V>
where
    T: Integer,
    V: Eq + Clone,
{
    fn next_back(&mut self) -> Option<Self::Item> {
        self.into_iter.next_back().map(|(_key, value)| value)
    }
}
