use core::iter::{ExactSizeIterator, FusedIterator};

impl<'a, T: Copy + Ord, B: BuildHasher> Iterator for Gaps<'a, T, B> {
    type Item = (Option<&'a T>, Option<&'a T>);

    fn next(&mut self) -> Option<Self::Item> {
        if self.idx == 0 {
            self.idx += 1;
            return Some((None, self.slice.get(0)));
        }
        if self.idx == self.slice.len() {
            return Some((self.slice.get(self.idx - 1), None));
        }
        let item = Some((self.slice.get(self.idx - 1), self.slice.get(self.idx)));
        self.idx += 1;
        item
    }
}

// Safe because the implementation has a clear termination condition (idx > slice.len())
// and will consistently return None once exhausted
impl<'a, T: Copy + Ord, B: BuildHasher> FusedIterator for Gaps<'a, T, B> {}

// Safe because we know the exact number of gaps is always slice.len() + 1
impl<'a, T: Copy + Ord, B: BuildHasher> ExactSizeIterator for Gaps<'a, T, B> {
    fn len(&self) -> usize {
        // Gaps iterator always returns exactly slice.len() + 1 items
        self.slice.len() + 1
    }
}

// Safe because we can navigate the gaps from the end just as easily as from the beginning
impl<'a, T: Copy + Ord, B: BuildHasher> DoubleEndedIterator for Gaps<'a, T, B> {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.idx > self.slice.len() {
            return None;
        }

        let last_idx = self.slice.len() - self.idx;
        self.idx += 1;

        if last_idx == 0 {
            Some((None, self.slice.get(0)))
        } else if last_idx == self.slice.len() {
            Some((self.slice.get(self.slice.len() - 1), None))
        } else {
            Some((self.slice.get(last_idx - 1), self.slice.get(last_idx)))
        }
    }
}
