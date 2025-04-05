use core::iter::FusedIterator;

use core::hash::BuildHasher;
use core::ops::RangeBounds;
use std::collections::BTreeSet;
use std::collections::HashMap;
use std::collections::hash_map::Entry;

pub struct Ranges<'a, T: 'a + Copy + Ord, B: BuildHasher> {
    map: &'a HashMap<T, BTreeSet<T>, B>,
    iter: Option<btree_set::Iter<'a, T>>,
    range: Option<(&'a T, &'a T)>,
}

impl<'a, T: Copy + Ord, B: BuildHasher> Ranges<'a, T, B> {
    pub fn new(map: &'a HashMap<T, BTreeSet<T>, B>) -> Self {
        Ranges {
            map,
            iter: None,
            range: None,
        }
    }

    pub fn range<R>(&mut self, range: R) -> &mut Self
    where
        R: RangeBounds<T>,
    {
        self.range = Some((range.start_bound().cloned(), range.end_bound().cloned()));
        self
    }
}

impl<'a, T: Copy + Ord, B: BuildHasher> Iterator for Ranges<'a, T, B> {
    type Item = (&'a T, &'a T);

    fn next(&mut self) -> Option<Self::Item> {
        // ...existing code...
    }
}

impl<'a, T: Copy + Ord, B: BuildHasher> FusedIterator for Ranges<'a, T, B> {}

// Note: Implementing ExactSizeIterator would require tracking the remaining elements,
// which is complex for Ranges due to its lazy nature and range filtering.
// It's not trivial to implement without significant changes.

// Note: DoubleEndedIterator isn't easily implementable for Ranges because
// it depends on the map keys and nested iteration of BTreeSet entries.
// We would need to significantly restructure how ranges are tracked to support
// efficient bidirectional iteration.
