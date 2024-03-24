// cmk check list
// main and into_
// iter, values, keys
// exact size iterator, double ended iterator, fused iterator, size_hint
// document the exact size and double ended

use core::{fmt, iter::FusedIterator};

use alloc::collections::btree_map;

use crate::{
    map::{CloneBorrow, EndValue, ValueOwned},
    sorted_disjoint_map::RangeValue,
    Integer, SortedDisjointMap,
};

/// A (double-ended) iterator over the integer elements of a [`RangeMapBlaze`].
///
/// This `struct` is created by the [`iter`] method on [`RangeMapBlaze`]. See its
/// documentation for more.
///
/// [`iter`]: RangeMapBlaze::iter
#[must_use = "iterators are lazy and do nothing unless consumed"]
#[derive(Clone, Debug)]
pub struct IterMap<T, V, VR, I>
where
    T: Integer,
    V: ValueOwned,
    VR: CloneBorrow<V>,
    I: SortedDisjointMap<T, V, VR>,
{
    iter: I,
    option_range_value_front: Option<RangeValue<T, V, VR>>,
    option_range_value_back: Option<RangeValue<T, V, VR>>,
}

impl<'a, T, V, VR, I> IterMap<T, V, VR, I>
where
    T: Integer,
    V: ValueOwned,
    VR: CloneBorrow<V>,
    I: SortedDisjointMap<T, V, VR>,
{
    pub fn new(iter: I) -> Self {
        IterMap {
            iter,
            option_range_value_front: None,
            option_range_value_back: None,
        }
    }
}

impl<'a, T, V, VR, I> FusedIterator for IterMap<T, V, VR, I>
where
    T: Integer,
    V: ValueOwned,
    VR: CloneBorrow<V>,
    I: SortedDisjointMap<T, V, VR> + FusedIterator,
{
}

impl<'a, T, V, VR, I> Iterator for IterMap<T, V, VR, I>
where
    T: Integer,
    V: ValueOwned,
    VR: CloneBorrow<V>,
    I: SortedDisjointMap<T, V, VR>,
{
    type Item = (T, VR);

    fn next(&mut self) -> Option<Self::Item> {
        let mut range_value = self
            .option_range_value_front
            .take()
            .or_else(|| self.iter.next())
            .or_else(|| self.option_range_value_back.take())?;

        let (start, end) = range_value.range.into_inner();
        debug_assert!(start <= end && end <= T::safe_max_value());
        let value = range_value.value.clone_borrow();
        if start < end {
            range_value.range = start + T::one()..=end;
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

impl<'a, T, V, VR, I> DoubleEndedIterator for IterMap<T, V, VR, I>
where
    T: Integer,
    V: ValueOwned,
    VR: CloneBorrow<V>,
    I: SortedDisjointMap<T, V, VR> + DoubleEndedIterator,
{
    fn next_back(&mut self) -> Option<Self::Item> {
        let mut range_value = self
            .option_range_value_back
            .take()
            .or_else(|| self.iter.next_back())
            .or_else(|| self.option_range_value_front.take())?;
        let (start, end) = range_value.range.into_inner();
        debug_assert!(start <= end && end <= T::safe_max_value());
        let value = range_value.value.clone_borrow();
        if start < end {
            range_value.range = start..=end - T::one();
            self.option_range_value_back = Some(range_value);
        }

        Some((end, value))
    }
}

#[must_use = "iterators are lazy and do nothing unless consumed"]
/// A (double-ended) iterator over the integer elements of a [`RangeMapBlaze`].
///
/// This `struct` is created by the [`into_iter`] method on [`RangeMapBlaze`]. See its
/// documentation for more.
///
/// [`into_iter`]: RangeMapBlaze::into_iter
pub struct IntoIterMap<T, V>
where
    T: Integer,
    V: ValueOwned,
{
    option_start_end_value_front: Option<(T, EndValue<T, V>)>,
    option_start_end_value_back: Option<(T, EndValue<T, V>)>,
    into_iter: btree_map::IntoIter<T, EndValue<T, V>>,
}

impl<T, V> IntoIterMap<T, V>
where
    T: Integer,
    V: ValueOwned,
{
    pub fn new(into_iter: btree_map::IntoIter<T, EndValue<T, V>>) -> Self {
        IntoIterMap {
            option_start_end_value_front: None,
            option_start_end_value_back: None,
            into_iter,
        }
    }
}

impl<T, V> FusedIterator for IntoIterMap<T, V>
where
    T: Integer,
    V: ValueOwned,
{
}

impl<T, V> Iterator for IntoIterMap<T, V>
where
    T: Integer,
    V: ValueOwned,
{
    type Item = (T, V);

    fn next(&mut self) -> Option<Self::Item> {
        let start_end_value = self
            .option_start_end_value_front
            .take()
            .or_else(|| self.into_iter.next())
            .or_else(|| self.option_start_end_value_back.take())?;

        let start = start_end_value.0;
        let end = start_end_value.1.end;
        let value = start_end_value.1.value.borrow_clone();
        debug_assert!(start <= end && end <= T::safe_max_value());
        if start < end {
            let end_value = start_end_value.1;
            let start_end_value = (start + T::one(), end_value);
            self.option_start_end_value_front = Some(start_end_value);
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
    V: ValueOwned,
{
    fn next_back(&mut self) -> Option<Self::Item> {
        let start_end_value = self
            .option_start_end_value_back
            .take()
            .or_else(|| self.into_iter.next_back())
            .or_else(|| self.option_start_end_value_front.take())?;

        let start = start_end_value.0;
        let end = start_end_value.1.end;
        let value = start_end_value.1.value.borrow_clone();
        debug_assert!(start <= end && end <= T::safe_max_value());

        if start < end {
            let mut end_value = start_end_value.1;
            end_value.end -= T::one();
            let start_end_value = (start, end_value);
            self.option_start_end_value_back = Some(start_end_value);
        }

        Some((end, value))
    }
}

impl<T> fmt::Debug for IntoIterMap<T, ()>
where
    T: Integer,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // General case, possibly minimal info since we can't conditionally compile based on V being ()
        f.debug_struct("IntoIterMap")
            .field(
                "option_start_end_value_front",
                &self.option_start_end_value_front,
            )
            .field(
                "option_start_end_value_back",
                &self.option_start_end_value_back,
            )
            // .field("into_iter", &self.into_iter) // IntoIter might not be Debug, so consider how to handle this
            .finish()
    }
}
/// A (double-ended) iterator over the integer elements of a [`RangeMapBlaze`].
///
/// This `struct` is created by the [`iter`] method on [`RangeMapBlaze`]. See its
/// documentation for more.
///
/// [`iter`]: RangeMapBlaze::iter
#[must_use = "iterators are lazy and do nothing unless consumed"]
#[derive(Clone, Debug)]
pub struct KeysMap<T, V, VR, I>
where
    T: Integer,
    V: ValueOwned,
    VR: CloneBorrow<V>,
    I: SortedDisjointMap<T, V, VR>,
{
    iter: IterMap<T, V, VR, I>,
}

impl<'a, T, V, VR, I> KeysMap<T, V, VR, I>
where
    T: Integer,
    V: ValueOwned,
    VR: CloneBorrow<V>,
    I: SortedDisjointMap<T, V, VR>,
{
    pub fn new(iter: I) -> Self {
        KeysMap {
            iter: IterMap::new(iter),
        }
    }
}

impl<'a, T, V, VR, I> FusedIterator for KeysMap<T, V, VR, I>
where
    T: Integer,
    V: ValueOwned,
    VR: CloneBorrow<V>,
    I: SortedDisjointMap<T, V, VR>,
{
}

impl<'a, T, V, VR, I> Iterator for KeysMap<T, V, VR, I>
where
    T: Integer,
    V: ValueOwned,
    VR: CloneBorrow<V>,
    I: SortedDisjointMap<T, V, VR>,
{
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|(key, _value)| key)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

impl<'a, T, V, VR, I> DoubleEndedIterator for KeysMap<T, V, VR, I>
where
    T: Integer,
    V: ValueOwned,
    VR: CloneBorrow<V>,
    I: SortedDisjointMap<T, V, VR> + DoubleEndedIterator,
{
    fn next_back(&mut self) -> Option<Self::Item> {
        self.iter.next_back().map(|(key, _value)| key)
    }
}
