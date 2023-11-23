#![cfg(feature = "from_slice")]

use crate::Integer;
use core::simd::{LaneCount, Simd, SimdElement, SupportedLaneCount};
use core::{iter::FusedIterator, ops::RangeInclusive};

#[macro_export]
#[doc(hidden)]
macro_rules! from_slice {
    ($reference:ident) => {
        #[inline]
        fn from_slice(slice: impl AsRef<[Self]>) -> RangeSetBlaze<Self> {
            FromSliceIter::<Self, { SIMD_REGISTER_BYTES / core::mem::size_of::<Self>() }>::new(
                slice.as_ref(),
                $reference(),
            )
            .collect()
        }
    };
}

#[inline]
pub(crate) fn is_consecutive<T, const N: usize>(chunk: Simd<T, N>, reference: Simd<T, N>) -> bool
where
    T: Integer + SimdElement,
    LaneCount<N>: SupportedLaneCount,
    Simd<T, N>: core::ops::Sub<Output = Simd<T, N>>,
{
    // let b = chunk.rotate_lanes_right::<1>();
    // chunk - b == reference
    let subtracted = chunk - reference;
    Simd::<T, N>::splat(chunk[0]) == subtracted
    // const SWIZZLE_CONST: [usize; N] = [0; N];
    // simd_swizzle!(subtracted, SWIZZLE_CONST) == subtracted
}

macro_rules! reference_t {
    ($function:ident, $type:ty) => {
        pub(crate) const fn $function<const N: usize>() -> Simd<$type, N>
        where
            LaneCount<N>: SupportedLaneCount,
        {
            // let mut arr: [$type; N] = [1; N];
            // arr[0] = (1 as $type).wrapping_sub(N as $type); // is -(N-1) for signed & unsigned

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

// avx512 (512 bits) or scalar
#[cfg(any(target_feature = "avx512f", not(target_feature = "avx2")))]
pub(crate) const SIMD_REGISTER_BYTES: usize = 512 / 8;
// avx2 (256 bits)
#[cfg(all(target_feature = "avx2", not(target_feature = "avx512f")))]
pub(crate) const SIMD_REGISTER_BYTES: usize = 256 / 8;

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
    pub(crate) fn new(slice: &'a [T], reference: Simd<T, N>) -> Self {
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

impl<'a, T, const N: usize> FusedIterator for FromSliceIter<'a, T, N>
where
    T: Integer + SimdElement,
    LaneCount<N>: SupportedLaneCount,
    Simd<T, N>: core::ops::Sub<Output = Simd<T, N>>,
{
}

impl<'a, T: 'a, const N: usize> Iterator for FromSliceIter<'a, T, N>
where
    T: Integer + SimdElement,
    LaneCount<N>: SupportedLaneCount,
    Simd<T, N>: core::ops::Sub<Output = Simd<T, N>>,
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
