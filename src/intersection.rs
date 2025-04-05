use core::iter::FusedIterator;

// ...existing code...

impl<'a, T: Copy + Ord, B: BuildHasher> Iterator for Intersection<'a, T, B> {
    // ...existing code...
}

// FusedIterator is safe for Intersection only if we can guarantee that once next() returns None,
// subsequent calls will continue to return None. This needs to be verified by examining
// the implementation details.
//
// The implementation should be checked to ensure that once either input iterator
// is exhausted, the Intersection iterator will consistently return None.
// impl<'a, T: Copy + Ord, B: BuildHasher> FusedIterator for Intersection<'a, T, B> {}

// Note: DoubleEndedIterator cannot be easily implemented for Intersection
// because it would require backwards iteration through both underlying iterators
// while maintaining the intersection logic
