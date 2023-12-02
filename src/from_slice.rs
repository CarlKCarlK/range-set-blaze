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
            FromSliceIter::<Self>::new(slice.as_ref(), $reference()).collect()
        }
    };
}

pub(crate) const LANES: usize = 8;

#[inline]
pub(crate) fn is_consecutive<T>(chunk: Simd<T, LANES>, reference: Simd<T, LANES>) -> bool
where
    T: Integer + SimdElement,
    Simd<T, LANES>: Sub<Output = Simd<T, LANES>>,
{
    // let b = chunk.rotate_lanes_right::<1>(); // cmk
    // chunk - b == reference
    let subtracted = chunk - reference;
    Simd::splat(subtracted[0]) == subtracted
    // const SWIZZLE_CONST: [usize; N] = [0; N]; // cmk
    // simd_swizzle!(subtracted, SWIZZLE_CONST) == subtracted
}

macro_rules! reference_t {
    ($function:ident, $type:ty) => {
        pub(crate) const fn $function() -> Simd<$type, LANES> {
            // let mut arr: [$type; N] = [1; N]; // cmk
            // arr[0] = (1 as $type).wrapping_sub(N as $type); // is -(N-1) for signed & unsigned

            let mut arr: [$type; LANES] = [0; LANES];
            let mut i = 0;
            while i < LANES {
                arr[i] = i as $type;
                i += 1;
            }
            Simd::from_array(arr)
        }
    };
}

reference_t!(reference_i8, i8);
reference_t!(reference_u8, u8);
reference_t!(reference_i16, i16);
reference_t!(reference_u16, u16);
reference_t!(reference_i32, i32);
reference_t!(reference_u32, u32);
reference_t!(reference_i64, i64);
reference_t!(reference_u64, u64);
reference_t!(reference_isize, isize);
reference_t!(reference_usize, usize);

// cmk

#[derive(Clone, Debug)]
#[must_use = "iterators are lazy and do nothing unless consumed"]
pub(crate) struct FromSliceIter<'a, T>
where
    T: Integer + SimdElement,
{
    prefix_iter: core::slice::Iter<'a, T>,
    previous_range: Option<RangeInclusive<T>>,
    chunks: slice::Iter<'a, Simd<T, LANES>>,
    suffix: &'a [T],
    slice_len: usize,
    reference: Simd<T, LANES>,
}

impl<'a, T: 'a> FromSliceIter<'a, T>
where
    T: Integer + SimdElement,
{
    pub(crate) fn new(slice: &'a [T], reference: Simd<T, LANES>) -> Self {
        let (prefix, middle, suffix) = slice.as_simd();
        FromSliceIter {
            prefix_iter: prefix.iter(),
            previous_range: None,
            chunks: middle.iter(),
            suffix,
            slice_len: slice.len(),
            reference,
        }
    }
}

impl<'a, T> FusedIterator for FromSliceIter<'a, T>
where
    T: Integer + SimdElement,
    Simd<T, LANES>: core::ops::Sub<Output = Simd<T, LANES>>,
{
}

impl<'a, T: 'a> Iterator for FromSliceIter<'a, T>
where
    T: Integer + SimdElement,
    Simd<T, LANES>: Sub<Output = Simd<T, LANES>>,
{
    type Item = RangeInclusive<T>;

    #[inline]
    fn next(&mut self) -> Option<RangeInclusive<T>> {
        if let Some(before) = self.prefix_iter.next() {
            return Some(*before..=*before);
        }
        let reference = self.reference;
        for chunk in self.chunks.by_ref() {
            if is_consecutive(*chunk, reference) {
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
