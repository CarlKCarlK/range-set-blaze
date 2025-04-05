use core::iter::FusedIterator;

// Note: ExactSizeIterator cannot be implemented for Union
// because we don't know the exact size until we've fully processed both sets
// and eliminated duplicates

impl<'a, T: Copy + Ord, B: BuildHasher> Iterator for Union<'a, T, B> {
    // ...existing code...
}

// FusedIterator is safe for Union only if we can guarantee that once next() returns None,
// subsequent calls will continue to return None. This needs to be verified by examining
// the implementation details.
//
// The implementation should be checked to ensure that once both input iterators
// are exhausted, the Union iterator will consistently return None.
// impl<'a, T: Copy + Ord, B: BuildHasher> FusedIterator for Union<'a, T, B> {}
