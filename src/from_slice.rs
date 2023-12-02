#![cfg(feature = "from_slice")]

use alloc::slice;

use crate::Integer;
use core::simd::{Simd, SimdElement};
use core::{iter::FusedIterator, ops::RangeInclusive, ops::Sub};

// cmk delete
// pub(crate) const fn const_min(a: usize, b: usize) -> usize {
//     if a < b {
//         a
//     } else {
//         b
//     }
// }

#[macro_export]
#[doc(hidden)]
macro_rules! from_slice {
    ($reference:ident) => {
        #[inline]
        fn from_slice(slice: impl AsRef<[Self]>) -> RangeSetBlaze<Self> {
            FromSliceIter::<Self>::new(slice.as_ref()).collect()
        }
    };
}

pub(crate) const LANES: usize = 16;

#[derive(Clone, Debug)]
#[must_use = "iterators are lazy and do nothing unless consumed"]
pub(crate) struct FromSliceIter<'a, T>
where
    T: Integer + SimdElement + IsConsecutive,
{
    prefix_iter: core::slice::Iter<'a, T>,
    previous_range: Option<RangeInclusive<T>>,
    chunks: slice::Iter<'a, Simd<T, LANES>>,
    suffix: &'a [T],
    slice_len: usize,
}

impl<'a, T: 'a> FromSliceIter<'a, T>
where
    T: Integer + SimdElement + IsConsecutive,
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

impl<'a, T> FusedIterator for FromSliceIter<'a, T>
where
    T: Integer + SimdElement + IsConsecutive,
    Simd<T, LANES>: core::ops::Sub<Output = Simd<T, LANES>>,
{
}

impl<'a, T: 'a> Iterator for FromSliceIter<'a, T>
where
    T: Integer + SimdElement + IsConsecutive,
    Simd<T, LANES>: Sub<Output = Simd<T, LANES>>,
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
                let this_end = chunk[LANES - 1];

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
    fn is_consecutive(chunk: Simd<Self, LANES>) -> bool
    where
        Self: SimdElement;
}

macro_rules! impl_is_consecutive_for {
    ($type:ty, $lanes:expr) => {
        impl IsConsecutive for $type {
            #[inline] // cmk00 make this it's own macro for better readability
            fn is_consecutive(chunk: Simd<Self, $lanes>) -> bool {
                #[inline] // cmk rule this #inline is important
                pub const fn reference() -> Simd<$type, $lanes> {
                    let mut arr: [$type; $lanes] = [0; $lanes];
                    let mut i = 0;
                    while i < $lanes {
                        arr[i] = i as $type;
                        i += 1;
                    }
                    Simd::from_array(arr)
                }

                let subtracted = chunk - reference();
                Simd::splat(chunk[0]) == subtracted
            }
        }
    };
}

// Apply the macro to each integer type
impl_is_consecutive_for!(i8, LANES);
impl_is_consecutive_for!(i16, LANES);
impl_is_consecutive_for!(i32, LANES);
impl_is_consecutive_for!(i64, LANES);
impl_is_consecutive_for!(isize, LANES);
impl_is_consecutive_for!(u8, LANES);
impl_is_consecutive_for!(u16, LANES);
impl_is_consecutive_for!(u32, LANES);
impl_is_consecutive_for!(u64, LANES);
impl_is_consecutive_for!(usize, LANES);
