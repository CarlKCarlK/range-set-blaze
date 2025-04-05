use core::iter::FusedIterator;

use core::hash::BuildHasher;
use core::ops::RangeBounds;
use std::collections::BTreeSet;

pub struct RangeSet<T, B: BuildHasher> {
    set: BTreeSet<T>,
    hasher: B,
}

pub struct RangeSetIter<'a, T: Copy + Ord, B: BuildHasher> {
    inner: std::collections::btree_set::Iter<'a, T>,
    hasher: &'a B,
}

impl<'a, T: Copy + Ord, B: BuildHasher> Iterator for RangeSetIter<'a, T, B> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
    }
}

// Safe because BTreeSet iterators are already fused
impl<'a, T: Copy + Ord, B: BuildHasher> FusedIterator for RangeSetIter<'a, T, B> {}

// Safe because BTreeSet iterators already implement ExactSizeIterator
impl<'a, T: Copy + Ord, B: BuildHasher> ExactSizeIterator for RangeSetIter<'a, T, B> {
    fn len(&self) -> usize {
        self.inner.len()
    }
}

// Safe because BTreeSet iterators already implement DoubleEndedIterator
impl<'a, T: Copy + Ord, B: BuildHasher> DoubleEndedIterator for RangeSetIter<'a, T, B> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.inner.next_back()
    }
}
