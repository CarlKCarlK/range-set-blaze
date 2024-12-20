use crate::Integer;
use alloc::collections::btree_map;
use core::{iter::FusedIterator, ops::RangeInclusive};

/// This `struct` is created by the [`ranges`] method on [`RangeSetBlaze`]. See [`ranges`]'s
/// documentation for more.
///
/// [`RangeSetBlaze`]: crate::RangeSetBlaze
/// [`ranges`]: crate::RangeSetBlaze::ranges
#[derive(Clone, Debug)]
#[must_use = "iterators are lazy and do nothing unless consumed"]
pub struct RangesIter<'a, T: Integer> {
    pub(crate) iter: btree_map::Iter<'a, T, T>,
}

impl<T: Integer> ExactSizeIterator for RangesIter<'_, T> {
    #[must_use]
    fn len(&self) -> usize {
        self.iter.len()
    }
}

impl<T: Integer> FusedIterator for RangesIter<'_, T> {}

// Range's iterator is just the inside BTreeMap iterator as values
impl<T: Integer> Iterator for RangesIter<'_, T> {
    type Item = RangeInclusive<T>;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|(start, end)| *start..=*end)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

impl<T: Integer> DoubleEndedIterator for RangesIter<'_, T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.iter.next_back().map(|(start, end)| *start..=*end)
    }
}

/// This `struct` is created by the [`into_ranges`] method on [`RangeSetBlaze`]. See [`into_ranges`]'s
/// documentation for more.
///
/// [`RangeSetBlaze`]: crate::RangeSetBlaze
/// [`into_ranges`]: crate::RangeSetBlaze::into_ranges
#[must_use = "iterators are lazy and do nothing unless consumed"]
#[derive(Debug)]
pub struct IntoRangesIter<T: Integer> {
    pub(crate) iter: alloc::collections::btree_map::IntoIter<T, T>,
}

impl<T: Integer> ExactSizeIterator for IntoRangesIter<T> {
    #[must_use]
    fn len(&self) -> usize {
        self.iter.len()
    }
}

impl<T: Integer> FusedIterator for IntoRangesIter<T> {}

// Range's iterator is just the inside BTreeMap iterator as values
impl<T: Integer> Iterator for IntoRangesIter<T> {
    type Item = RangeInclusive<T>;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|(start, end)| start..=end)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

impl<T: Integer> DoubleEndedIterator for IntoRangesIter<T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.iter.next_back().map(|(start, end)| start..=end)
    }
}
