#![cfg(feature = "from_slice")]

use alloc::slice;

use crate::Integer;
use core::simd::{LaneCount, Simd, SimdElement, SupportedLaneCount};
use core::{iter::FusedIterator, ops::RangeInclusive, ops::Sub};

#[derive(Clone, Debug)]
#[must_use = "iterators are lazy and do nothing unless consumed"]
pub(crate) struct FromSliceIter<'a, T, const N: usize>
where
    T: Integer + SimdElement + IsConsecutive,
    LaneCount<N>: SupportedLaneCount,
{
    prefix_iter: core::slice::Iter<'a, T>,
    previous_range: Option<RangeInclusive<T>>,
    chunks: slice::Iter<'a, Simd<T, N>>,
    suffix: &'a [T],
    slice_len: usize,
}

impl<'a, T: 'a, const N: usize> FromSliceIter<'a, T, N>
where
    T: Integer + SimdElement + IsConsecutive,
    LaneCount<N>: SupportedLaneCount,
{
    pub(crate) fn new(slice: &'a [T]) -> Self {
        let (prefix, middle, suffix) = slice.as_simd();
        FromSliceIter {
            prefix_iter: prefix.iter(),
            previous_range: None,
            chunks: middle.iter(),
            suffix,
            slice_len: slice.len(),
        }
    }
}

impl<'a, T, const N: usize> FusedIterator for FromSliceIter<'a, T, N>
where
    T: Integer + SimdElement + IsConsecutive,
    Simd<T, N>: core::ops::Sub<Output = Simd<T, N>>,
    LaneCount<N>: SupportedLaneCount,
{
}

impl<'a, T: 'a, const N: usize> Iterator for FromSliceIter<'a, T, N>
where
    T: Integer + SimdElement + IsConsecutive,
    Simd<T, N>: Sub<Output = Simd<T, N>>,
    LaneCount<N>: SupportedLaneCount,
{
    type Item = RangeInclusive<T>;

    #[inline]
    fn next(&mut self) -> Option<RangeInclusive<T>> {
        if let Some(before) = self.prefix_iter.next() {
            return Some(*before..=*before);
        }
        for chunk in self.chunks.by_ref() {
            if T::is_consecutive(*chunk) {
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

pub trait IsConsecutive {
    fn is_consecutive<const N: usize>(chunk: Simd<Self, N>) -> bool
    where
        Self: SimdElement,
        Simd<Self, N>: Sub<Simd<Self, N>, Output = Simd<Self, N>>,
        LaneCount<N>: SupportedLaneCount;
}

macro_rules! define_const_reference {
    ($type:ty) => {
        const fn reference<const N: usize>() -> Simd<$type, N>
        where
            LaneCount<N>: SupportedLaneCount,
        {
            let mut arr: [$type; N] = [0; N];
            let mut i = 0;
            while i < N {
                arr[i] = i as $type;
                i += 1;
            }
            Simd::from_array(arr)
        }
    };
}

macro_rules! impl_is_consecutive {
    ($type:ty) => {
        // Repeat for each integer type (i8, i16, i32, i64, isize, u8, u16, u32, u64, usize)

        impl IsConsecutive for $type {
            #[inline] // cmk00 make this it's own macro for better readability
            fn is_consecutive<const N: usize>(chunk: Simd<Self, N>) -> bool
            where
                Self: SimdElement,
                Simd<Self, N>: Sub<Simd<Self, N>, Output = Simd<Self, N>>,
                LaneCount<N>: SupportedLaneCount,
            {
                define_const_reference!($type);
                let subtracted = chunk - reference();
                Simd::splat(chunk[0]) == subtracted
            }
        }
    };
}

// Apply the macro to each integer type
impl_is_consecutive!(i8);
impl_is_consecutive!(i16);
impl_is_consecutive!(i32);
impl_is_consecutive!(i64);
impl_is_consecutive!(isize);
impl_is_consecutive!(u8);
impl_is_consecutive!(u16);
impl_is_consecutive!(u32);
impl_is_consecutive!(u64);
impl_is_consecutive!(usize);

#[test]
fn test_is_consecutive() {
    let mut arr: [i8; 16] = [0; 16];
    let mut i = 0;
    while i < 16 {
        arr[i] = i as i8;
        i += 1;
    }
    let simd = Simd::from_array(arr);
    assert!(i8::is_consecutive(simd));
}
