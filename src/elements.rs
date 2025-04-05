use core::iter::{ExactSizeIterator, FusedIterator};

impl<'a, T: Copy + Ord, B: BuildHasher> Iterator for Elements<'a, T, B> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        // implementation
    }
}

// Safe because once remaining reaches 0, it will consistently return None
impl<'a, T: Copy + Ord, B: BuildHasher> FusedIterator for Elements<'a, T, B> {}

// Safe because we track the exact number of remaining elements
impl<'a, T: Copy + Ord, B: BuildHasher> ExactSizeIterator for Elements<'a, T, B> {
    fn len(&self) -> usize {
        self.remaining
    }
}

// DoubleEndedIterator isn't easily implementable for Elements because
// it requires random access to elements or a reverse iterator that
// can navigate from the end efficiently
