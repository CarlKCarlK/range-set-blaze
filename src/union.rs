use core::iter::FusedIterator;

// Note: ExactSizeIterator cannot be implemented for Union
// because we don't know the exact size until we've fully processed both sets
// and eliminated duplicates

impl<'a, T: Copy + Ord, B: BuildHasher> Iterator for Union<'a, T, B> {
    // ...existing code...
}

impl<'a, T: Copy + Ord, B: BuildHasher> FusedIterator for Union<'a, T, B> {}
