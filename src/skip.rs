use core::iter::FusedIterator;

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

impl<I: Iterator> FusedIterator for Skip<I> where I: FusedIterator {}

impl<I: Iterator> ExactSizeIterator for Skip<I>
where
    I: ExactSizeIterator,
{
    fn len(&self) -> usize {
        let inner_len = self.iter.len();
        inner_len.saturating_sub(self.n)
    }
}

impl<I: Iterator> DoubleEndedIterator for Skip<I>
where
    I: DoubleEndedIterator + ExactSizeIterator,
{
    fn next_back(&mut self) -> Option<Self::Item> {
        self.iter.next_back()
    }
}
