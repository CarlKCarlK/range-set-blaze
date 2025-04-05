use core::iter::FusedIterator;

// ...existing code...

impl<'a, T: Copy + Ord, B: BuildHasher> Iterator for Intersection<'a, T, B> {
    // ...existing code...
}

impl<'a, T: Copy + Ord, B: BuildHasher> FusedIterator for Intersection<'a, T, B> {}