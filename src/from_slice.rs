#![cfg(feature = "from_slice")]

use crate::{Integer, RangeSetBlaze};
use alloc::slice;
use core::{iter::FusedIterator, ops::RangeInclusive, slice::ChunksExact};
use fearless_simd::{Level, dispatch, prelude::*};

#[allow(clippy::redundant_pub_crate)]
pub(crate) trait FromSliceInteger: Integer {
    fn from_slice(slice: &[Self]) -> RangeSetBlaze<Self>;
}

macro_rules! impl_from_slice_integer {
    ($($type:ty),+ $(,)?) => {
        $(
            impl FromSliceInteger for $type {
                #[inline]
                fn from_slice(slice: &[Self]) -> RangeSetBlaze<Self> {
                    from_slice_simd(slice)
                }
            }
        )+
    };
}

impl_from_slice_integer!(i8, i16, i32, i64, u8, u16, u32, u64);

#[allow(clippy::redundant_pub_crate)]
pub(crate) trait SimdInteger: Integer + SimdIntElement {
    type Vector<S: Simd>: SimdInt<S, Element = Self>;
}

macro_rules! impl_simd_integer {
    ($($type:ty => $vector:ident),+ $(,)?) => {
        $(
            impl SimdInteger for $type {
                type Vector<S: Simd> = fearless_simd::$vector<S>;
            }
        )+
    };
}

impl_simd_integer!(
    i8 => i8x16,
    i16 => i16x16,
    i32 => i32x16,
    i64 => i64x8,
    u8 => u8x16,
    u16 => u16x16,
    u32 => u32x16,
    u64 => u64x8,
);

macro_rules! impl_from_slice_integer_scalar {
    ($($type:ty),+ $(,)?) => {
        $(
            impl FromSliceInteger for $type {
                #[inline]
                fn from_slice(slice: &[Self]) -> RangeSetBlaze<Self> {
                    RangeSetBlaze::from_iter(slice.iter().copied())
                }
            }
        )+
    };
}

// Fearless SIMD intentionally supports fixed-width integers rather than pointer-sized types.
impl_from_slice_integer_scalar!(isize, usize);

#[inline]
fn from_slice_simd<T>(slice: &[T]) -> RangeSetBlaze<T>
where
    T: SimdInteger,
{
    let level = Level::try_detect().unwrap_or(Level::baseline());
    dispatch!(level, simd => FromSliceIter::new(simd, slice).collect())
}

#[allow(clippy::redundant_pub_crate)]
#[allow(clippy::module_name_repetitions)]
#[derive(Clone, Debug)]
#[must_use = "iterators are lazy and do nothing unless consumed"]
pub(crate) struct FromSliceIter<'a, T, S>
where
    T: SimdInteger,
    S: Simd,
{
    simd: S,
    offsets: T::Vector<S>,
    singleton_iter: slice::Iter<'a, T>,
    previous_range: Option<RangeInclusive<T>>,
    chunks: ChunksExact<'a, T>,
    suffix: &'a [T],
    slice_len: usize,
}

impl<'a, T, S> FromSliceIter<'a, T, S>
where
    T: SimdInteger,
    S: Simd,
{
    #[inline(always)]
    pub(crate) fn new(simd: S, slice: &'a [T]) -> Self {
        let chunks = slice.chunks_exact(T::Vector::<S>::LEN);
        let suffix = chunks.remainder();
        Self {
            simd,
            offsets: comparison_value::<T, S>(simd),
            singleton_iter: slice[..0].iter(),
            previous_range: None,
            chunks,
            suffix,
            slice_len: slice.len(),
        }
    }
}

impl<T, S> FusedIterator for FromSliceIter<'_, T, S>
where
    T: SimdInteger,
    S: Simd,
{
}

impl<T, S> Iterator for FromSliceIter<'_, T, S>
where
    T: SimdInteger,
    S: Simd,
{
    type Item = RangeInclusive<T>;

    #[inline(always)]
    fn next(&mut self) -> Option<RangeInclusive<T>> {
        if let Some(before) = self.singleton_iter.next() {
            return Some(*before..=*before);
        }
        for chunk in self.chunks.by_ref() {
            if is_consecutive_with_offsets(self.simd, self.offsets, chunk) {
                let this_start = chunk[0];
                let this_end = chunk[chunk.len() - 1];

                if let Some(inner_previous_range) = self.previous_range.as_mut() {
                    if (*inner_previous_range.end()).add_one() == this_start {
                        *inner_previous_range = *(inner_previous_range.start())..=this_end;
                    } else {
                        let result = Some(inner_previous_range.clone());
                        *inner_previous_range = this_start..=this_end;
                        return result;
                    }
                } else {
                    self.previous_range = Some(this_start..=this_end);
                }
            } else {
                self.singleton_iter = chunk.iter();
                if let Some(previous) = self.previous_range.take() {
                    debug_assert!(self.previous_range.is_none());
                    return Some(previous);
                }
                let before = self
                    .singleton_iter
                    .next()
                    .expect(".next() is always Some() because the SIMD chunk has non-zero length");
                return Some(*before..=*before);
            }
        }

        if let Some(previous) = &self.previous_range.take() {
            debug_assert!(self.previous_range.is_none());
            return Some(previous.clone());
        }

        self.singleton_iter = self.suffix.iter();
        self.suffix = &[];

        self.singleton_iter.next().map(|before| *before..=*before)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let low = self.slice_len.min(1);
        let high = self.slice_len;
        (low, Some(high))
    }
}

#[cfg(test)]
#[inline(always)]
pub(crate) fn is_consecutive<T, S>(simd: S, chunk: &[T]) -> bool
where
    T: SimdInteger,
    S: Simd,
{
    is_consecutive_with_offsets(simd, comparison_value::<T, S>(simd), chunk)
}

#[inline(always)]
fn is_consecutive_with_offsets<T, S>(simd: S, offsets: T::Vector<S>, chunk: &[T]) -> bool
where
    T: SimdInteger,
    S: Simd,
{
    let values = T::Vector::<S>::from_slice(simd, chunk);
    (values - offsets)
        .simd_eq(T::Vector::<S>::splat(simd, chunk[0]))
        .all_true()
}

#[inline(always)]
fn comparison_value<T, S>(simd: S) -> T::Vector<S>
where
    T: SimdInteger,
    S: Simd,
{
    T::Vector::<S>::from_fn(simd, |index| match T::try_from(index) {
        Ok(value) => value,
        Err(_) => unreachable!("a SIMD lane index always fits its integer element type"),
    })
}
