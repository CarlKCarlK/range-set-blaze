use std::{collections::btree_map, ops::RangeInclusive};

use crate::{Integer, SortedDisjoint, SortedStarts};

#[derive(Clone)]
#[must_use = "iterators are lazy and do nothing unless consumed"]
/// An iterator that visits the ranges in the [`RangeSetInt`],
/// i.e., the integers as sorted & disjoint ranges.
///
/// This `struct` is created by the [`ranges`] method on [`RangeSetInt`]. See its
/// documentation for more.
///
/// [`ranges`]: RangeSetInt::ranges
pub struct RangesIter<'a, T: Integer> {
    pub(crate) iter: btree_map::Iter<'a, T, T>,
}

impl<'a, T: Integer> AsRef<RangesIter<'a, T>> for RangesIter<'a, T> {
    fn as_ref(&self) -> &Self {
        // Self is RangesIter<'a>, the type for which we impl AsRef
        self
    }
}

// RangesIter (one of the iterators from RangeSetInt) is SortedDisjoint
impl<T: Integer> SortedStarts for RangesIter<'_, T> {}
impl<T: Integer> SortedDisjoint for RangesIter<'_, T> {}

impl<T: Integer> ExactSizeIterator for RangesIter<'_, T> {
    #[must_use]
    fn len(&self) -> usize {
        self.iter.len()
    }
}

// Range's iterator is just the inside BTreeMap iterator as values
impl<'a, T: Integer> Iterator for RangesIter<'a, T> {
    type Item = RangeInclusive<T>;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|(start, end)| *start..=*end)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

//cmk0000 #[derive(Clone)]
#[must_use = "iterators are lazy and do nothing unless consumed"]
/// An iterator that visits the ranges in the [`RangeSetInt`],
/// i.e., the integers as sorted & disjoint ranges.
///
/// This `struct` is created by the [`into_ranges`] method on [`RangeSetInt`]. See its
/// documentation for more.
///
/// [`into_ranges`]: RangeSetInt::into_ranges
pub struct IntoRangesIter<T: Integer> {
    pub(crate) iter: std::collections::btree_map::IntoIter<T, T>,
}

impl<T: Integer> SortedStarts for IntoRangesIter<T> {}
impl<T: Integer> SortedDisjoint for IntoRangesIter<T> {}

impl<T: Integer> ExactSizeIterator for IntoRangesIter<T> {
    #[must_use]
    fn len(&self) -> usize {
        self.iter.len()
    }
}

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
