use core::iter::FusedIterator;

impl<'a, T: Copy + Ord, B: BuildHasher> Iterator for Elements<'a, T, B> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        // implementation
    }
}

impl<'a, T: Copy + Ord, B: BuildHasher> FusedIterator for Elements<'a, T, B> {}

impl<'a, T: Copy + Ord, B: BuildHasher> ExactSizeIterator for Elements<'a, T, B> {
    fn len(&self) -> usize {
        self.remaining
    }
}

// DoubleEndedIterator isn't easily implementable for Elements because
// it requires random access to elements or a reverse iterator that
// can navigate from the end efficiently
