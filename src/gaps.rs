use core::iter::FusedIterator;

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

impl<'a, T: Copy + Ord, B: BuildHasher> FusedIterator for Gaps<'a, T, B> {}

impl<'a, T: Copy + Ord, B: BuildHasher> ExactSizeIterator for Gaps<'a, T, B> {
    fn len(&self) -> usize {
        // Gaps iterator always returns exactly slice.len() + 1 items
        // (one before the first item, all gaps between items, and one after the last item)
        self.slice.len() + 1
    }
}

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