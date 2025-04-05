use core::iter::FusedIterator;

// ...existing code...

impl<'a, T: Copy + Ord, B: BuildHasher> Iterator for From<'a, T, B> {
    // ...existing code...
}

impl<'a, T: Copy + Ord, B: BuildHasher> FusedIterator for From<'a, T, B> {}

impl<'a, T: Copy + Ord, B: BuildHasher> Iterator for Into<'a, T, B> {
    // ...existing code...
}

impl<'a, T: Copy + Ord, B: BuildHasher> FusedIterator for Into<'a, T, B> {}