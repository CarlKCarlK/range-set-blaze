use crate::integer::is_consecutive;
use crate::Integer;
use core::simd::{LaneCount, Simd, SimdElement, SupportedLaneCount};
use core::{iter::FusedIterator, ops::RangeInclusive};

#[derive(Clone, Debug)]
#[must_use = "iterators are lazy and do nothing unless consumed"]
pub(crate) struct FromSliceIter<'a, T, const N: usize>
where
    T: Integer + SimdElement,
    LaneCount<N>: SupportedLaneCount,
{
    prefix_iter: core::slice::Iter<'a, T>,
    previous_range: Option<RangeInclusive<T>>,
    chunks: core::slice::Iter<'a, Simd<T, N>>,
    suffix: &'a [T],
    slice_len: usize,
    reference: Simd<T, N>,
}

impl<'a, T: 'a, const N: usize> FromSliceIter<'a, T, N>
where
    T: Integer + SimdElement,
    LaneCount<N>: SupportedLaneCount,
{
    pub(crate) fn new(slice: &'a [T], reference: &Simd<T, N>) -> Self {
        let (prefix, middle, suffix) = slice.as_simd();
        FromSliceIter {
            prefix_iter: prefix.iter(),
            previous_range: None,
            chunks: middle.iter(),
            suffix,
            slice_len: slice.len(),
            reference: *reference,
        }
    }
}

impl<'a, T, const N: usize> FusedIterator for FromSliceIter<'a, T, N>
where
    T: Integer + SimdElement,
    LaneCount<N>: SupportedLaneCount,
    Simd<T, N>: std::ops::Sub<Output = Simd<T, N>>,
{
}

impl<'a, T: 'a, const N: usize> Iterator for FromSliceIter<'a, T, N>
where
    T: Integer + SimdElement,
    LaneCount<N>: SupportedLaneCount,
    Simd<T, N>: std::ops::Sub<Output = Simd<T, N>>,
{
    type Item = RangeInclusive<T>;

    #[inline]
    fn next(&mut self) -> Option<RangeInclusive<T>> {
        if let Some(before) = self.prefix_iter.next() {
            return Some(*before..=*before);
        }
        let reference = self.reference;
        for chunk in self.chunks.by_ref() {
            if is_consecutive(chunk, reference) {
                let this_start = chunk[0];
                let this_end = chunk[N - 1];

                if let Some(inner_previous_range) = self.previous_range.as_mut() {
                    // if some and previous is some and adjacent, combine
                    if *inner_previous_range.end() + T::one() == this_start {
                        *inner_previous_range = *(inner_previous_range.start())..=this_end;
                    } else {
                        // if some and previous is some but not adjacent, flush previous, set previous to this range.
                        let result = Some(inner_previous_range.clone());
                        *inner_previous_range = this_start..=this_end;
                        return result;
                    }
                } else {
                    // if some and previous is None, set previous to this range.
                    self.previous_range = Some(this_start..=this_end);
                }
            } else {
                // If none, flush previous range, set it to none, output this chunk as a bunch of singletons.
                self.prefix_iter = chunk.as_array().iter();
                if let Some(previous) = self.previous_range.take() {
                    debug_assert!(self.previous_range.is_none());
                    return Some(previous);
                }
                if let Some(before) = self.prefix_iter.next() {
                    return Some(*before..=*before);
                }
            }
        }

        // at the very, very end, flush previous.
        if let Some(previous) = &self.previous_range.take() {
            debug_assert!(self.previous_range.is_none());
            return Some(previous.clone());
        }

        self.prefix_iter = self.suffix.iter();
        self.suffix = &[];

        self.prefix_iter.next().map(|before| *before..=*before)
    }

    // We could have one less or one more than the iter.
    fn size_hint(&self) -> (usize, Option<usize>) {
        let low = if self.slice_len > 0 {
            self.slice_len - 1
        } else {
            0
        };
        let high = if self.slice_len < usize::MAX {
            self.slice_len + 1
        } else {
            usize::MAX
        };
        (low, Some(high))
    }
}
