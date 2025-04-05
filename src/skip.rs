use core::iter::{FusedIterator, ExactSizeIterator};

pub struct Skip<I> {
    iter: I,
    n: usize,
}

impl<I> Skip<I> {
    pub fn new(iter: I, n: usize) -> Skip<I> {
        Skip { iter, n }
    }
}

impl<I: Iterator> Iterator for Skip<I> {
    fn next(&mut self) -> Option<I::Item> {
        while self.n > 0 {
            self.iter.next()?;
            self.n -= 1;
        }
        self.iter.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let (_, upper) = self.iter.size_hint();
        (0, upper)
    }
}

// Only implement FusedIterator when the inner iterator is fused
impl<I: Iterator + FusedIterator> FusedIterator for Skip<I> {}

// Only implement ExactSizeIterator when the inner iterator has an exact size
impl<I: Iterator + ExactSizeIterator> ExactSizeIterator for Skip<I> {
    fn len(&self) -> usize {
        let inner_len = self.iter.len();
        inner_len.saturating_sub(self.n)
    }
}

// Only implement DoubleEndedIterator when the inner iterator supports bidirectional iteration
impl<I: Iterator + DoubleEndedIterator + ExactSizeIterator> DoubleEndedIterator for Skip<I> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.iter.next_back()
    }
}
