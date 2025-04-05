use core::iter::{DoubleEndedIterator, FusedIterator};

// ...existing code...

impl<'a, T: Copy + Ord, B: BuildHasher> Iterator for From<'a, T, B> {
    // ...existing code...
}

impl<'a, T: Copy + Ord, B: BuildHasher> FusedIterator for From<'a, T, B> {}

// Only if From uses a double-ended inner collection
impl<'a, T: Copy + Ord, B: BuildHasher> DoubleEndedIterator for From<'a, T, B> {
    fn next_back(&mut self) -> Option<Self::Item> {
        // Assuming that From holds something like a btree_set::Iter that supports DoubleEndedIterator
        self.inner.next_back()
    }
}

impl<'a, T: Copy + Ord, B: BuildHasher> Iterator for Into<'a, T, B> {
    // ...existing code...
}

impl<'a, T: Copy + Ord, B: BuildHasher> FusedIterator for Into<'a, T, B> {}

// Only if Into uses a double-ended inner collection
impl<'a, T: Copy + Ord, B: BuildHasher> DoubleEndedIterator for Into<'a, T, B> {
    fn next_back(&mut self) -> Option<Self::Item> {
        // Assuming that Into holds something like a btree_set::Iter that supports DoubleEndedIterator
        self.inner.next_back()
    }
}
