use core::iter::FusedIterator;

impl<'a, T: Copy + Ord, B: BuildHasher> Iterator for Union<'a, T, B> {
    // ...existing code...
}

impl<'a, T: Copy + Ord, B: BuildHasher> FusedIterator for Union<'a, T, B> {}
