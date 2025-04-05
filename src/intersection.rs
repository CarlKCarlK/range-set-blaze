use core::iter::FusedIterator;

// ...existing code...

impl<'a, T: Copy + Ord, B: BuildHasher> Iterator for Intersection<'a, T, B> {
    // ...existing code...
}

impl<'a, T: Copy + Ord, B: BuildHasher> FusedIterator for Intersection<'a, T, B> {}

// Note: DoubleEndedIterator cannot be easily implemented for Intersection
// because it would require backwards iteration through both underlying iterators
// while maintaining the intersection logic
