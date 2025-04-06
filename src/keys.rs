use core::iter::FusedIterator;
use std::collections::btree_map;

use crate::{
    Integer, SortedDisjointMap,
    iter_map::{IntoIterMap, IterMap},
    map::{EndValue, ValueRef},
};

/// An iterator over the integer elements of a [`RangeMapBlaze`]. Double-ended.
///
/// This `struct` is created by the [`keys`] method on [`RangeMapBlaze`]. See its
/// documentation for more.
///
/// [`keys`]: crate::RangeMapBlaze::keys
/// [`RangeMapBlaze`]: crate::RangeMapBlaze
#[must_use = "iterators are lazy and do nothing unless consumed"]
#[derive(Clone, Debug)]
pub struct Keys<T, VR, I>
where
    T: Integer,
    VR: ValueRef,
    I: SortedDisjointMap<T, VR>,
{
    iter: IterMap<T, VR, I>,
}

impl<T, VR, I> Keys<T, VR, I>
where
    T: Integer,
    VR: ValueRef,
    I: SortedDisjointMap<T, VR>,
{
    pub(crate) const fn new(iter: I) -> Self {
        Self {
            iter: IterMap::new(iter),
        }
    }
}

impl<T, VR, I> FusedIterator for Keys<T, VR, I>
where
    T: Integer,
    VR: ValueRef,
    I: SortedDisjointMap<T, VR> + FusedIterator,
{
}

impl<T, VR, I> Iterator for Keys<T, VR, I>
where
    T: Integer,
    VR: ValueRef,
    I: SortedDisjointMap<T, VR>,
{
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|(key, _value)| key)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

impl<T, VR, I> DoubleEndedIterator for Keys<T, VR, I>
where
    T: Integer,
    VR: ValueRef,
    I: SortedDisjointMap<T, VR> + DoubleEndedIterator,
{
    fn next_back(&mut self) -> Option<Self::Item> {
        self.iter.next_back().map(|(key, _value)| key)
    }
}

/// An iterator over the integer elements of a [`RangeMapBlaze`]. Double-ended.
///
/// This `struct` is created by the [`into_keys`] method on [`RangeMapBlaze`]. See its
/// documentation for more.
///
/// [`into_keys`]: crate::RangeMapBlaze::into_keys
/// [`RangeMapBlaze`]: crate::RangeMapBlaze
#[must_use = "iterators are lazy and do nothing unless consumed"]
#[derive(Debug)]
pub struct IntoKeys<T, V>
where
    T: Integer,
    V: Eq + Clone,
{
    into_iter: IntoIterMap<T, V>,
}

impl<T, V> IntoKeys<T, V>
where
    T: Integer,
    V: Eq + Clone,
{
    pub(crate) const fn new(btree_map_into_iter: btree_map::IntoIter<T, EndValue<T, V>>) -> Self {
        let into_iter = IntoIterMap::new(btree_map_into_iter);
        Self { into_iter }
    }
}

impl<T, V> Iterator for IntoKeys<T, V>
where
    T: Integer,
    V: Eq + Clone,
{
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        self.into_iter.next().map(|(key, _value)| key)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.into_iter.size_hint()
    }
}

impl<T, V> FusedIterator for IntoKeys<T, V>
where
    T: Integer,
    V: Eq + Clone,
{
}

impl<T, V> DoubleEndedIterator for IntoKeys<T, V>
where
    T: Integer,
    V: Eq + Clone,
{
    fn next_back(&mut self) -> Option<Self::Item> {
        self.into_iter.next_back().map(|(key, _value)| key)
    }
}
