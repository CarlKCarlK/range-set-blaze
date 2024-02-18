use crate::{
    map::{EndValue, ValueOwned},
    sorted_disjoint_map::{RangeValue, SortedDisjointMap, SortedStartsMap},
    Integer,
};
use core::{
    cmp::{max, min},
    iter::FusedIterator,
    marker::PhantomData,
    ops::Deref,
};
use num_traits::Zero;

#[must_use = "iterators are lazy and do nothing unless consumed"]
pub(crate) struct UnsortedDisjointMap<'a, T, V, VR, I>
where
    T: Integer,
    V: ValueOwned + 'a,
    VR: Deref<Target = V> + 'a,
    I: Iterator<Item = RangeValue<'a, T, V, VR>>,
{
    iter: I,
    option_range_value: Option<RangeValue<'a, T, V, VR>>,
    min_value_plus_2: T,
    two: T,
}

impl<'a, T, V, VR, I> From<I> for UnsortedDisjointMap<'a, T, V, VR, I::IntoIter>
where
    T: Integer,
    V: ValueOwned + 'a,
    VR: Deref<Target = V> + 'a,
    I: IntoIterator<Item = RangeValue<'a, T, V, VR>>, // Any iterator is fine
{
    fn from(into_iter: I) -> Self {
        UnsortedDisjointMap {
            iter: into_iter.into_iter(),
            option_range_value: None,
            min_value_plus_2: T::min_value() + T::one() + T::one(),
            two: T::one() + T::one(),
        }
    }
}

// cmk
// impl<'a, T, V, VR, I> FusedIterator for UnsortedDisjointMap<'a, T, V, VR, I>
// where
//     T: Integer,
//     V: PartialEqClone + 'a,
//     I: Iterator<Item = RangeValue<'a, T, V, VR>> + FusedIterator,
// {
// }

impl<'a, T, V, VR, I> Iterator for UnsortedDisjointMap<'a, T, V, VR, I>
where
    T: Integer,
    V: ValueOwned + 'a,
    VR: Deref<Target = V> + 'a,
    I: Iterator<Item = RangeValue<'a, T, V, VR>>,
{
    type Item = RangeValue<'a, T, V, VR>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let range_value = match self.iter.next() {
                Some(r) => r,
                None => return self.option_range_value.take(),
            };

            let (next_start, next_end) = range_value.range.into_inner();
            if next_start > next_end {
                continue;
            }
            assert!(
                next_end <= T::safe_max_value(),
                "end must be <= T::safe_max_value()"
            );

            let Some(self_range_value) = &self.option_range_value else {
                let ncr = RangeValue {
                    range: next_start..=next_end,
                    value: range_value.value,
                    phantom_data: PhantomData,
                };
                self.option_range_value = Some(ncr);
                continue;
            };

            let (self_start, self_end) = self_range_value.range.clone().into_inner();
            if (next_start >= self.min_value_plus_2 && self_end <= next_start - self.two)
                || (self_start >= self.min_value_plus_2 && next_end <= self_start - self.two)
            {
                let scr = RangeValue {
                    range: self_start..=self_end,
                    value: self_range_value.value,
                    phantom_data: PhantomData,
                };
                let result = Some(scr);
                let ncr = RangeValue {
                    range: next_start..=next_end,
                    value: range_value.value,
                    phantom_data: PhantomData,
                };
                self.option_range_value = Some(ncr);
                return result;
            } else {
                let xcr = RangeValue {
                    range: min(self_start, next_start)..=max(self_end, next_end),
                    value: self_range_value.value,
                    phantom_data: PhantomData,
                };
                self.option_range_value = Some(xcr);
                continue;
            }
        }
    }

    // As few as one (or zero if iter is empty) and as many as iter.len()
    // There could be one extra if option_range is Some.
    fn size_hint(&self) -> (usize, Option<usize>) {
        let (lower, upper) = self.iter.size_hint();
        let lower = if lower == 0 { 0 } else { 1 };
        if self.option_range_value.is_some() {
            (lower, upper.map(|x| x + 1))
        } else {
            (lower, upper)
        }
    }
}

#[must_use = "iterators are lazy and do nothing unless consumed"]
pub(crate) struct SortedDisjointWithLenSoFarMap<'a, T, V, VR, I>
where
    T: Integer,
    V: ValueOwned + 'a,
    VR: Deref<Target = V> + 'a,
    I: SortedDisjointMap<'a, T, V, VR>,
    <V as ToOwned>::Owned: PartialEq,
{
    iter: I,
    len: <T as Integer>::SafeLen,
    _phantom_data: PhantomData<&'a VR>,
}

// cmk
// impl<T: Integer, V: PartialEqClone, I> From<I> for SortedDisjointWithLenSoFarMap<T, V, I::IntoIterMap>
// where
//     I: IntoIterator<Item = RangeValue<T, V>>,
//     I::IntoIter: SortedDisjointMap<T, V>,
// {
//     fn from(into_iter: I) -> Self {
//         SortedDisjointWithLenSoFarMap {
//             iter: into_iter.into_iter(),
//             len: <T as Integer>::SafeLen::zero(),
//             _phantom_data: PhantomData,
//         }
//     }
// }

impl<'a, T: Integer, V: ValueOwned + 'a, VR: Deref<Target = V> + 'a, I>
    SortedDisjointWithLenSoFarMap<'a, T, V, VR, I>
where
    I: SortedDisjointMap<'a, T, V, VR>,
    <V as ToOwned>::Owned: PartialEq,
{
    pub fn len_so_far(&self) -> <T as Integer>::SafeLen {
        self.len
    }
}

// cmk
// impl<T: Integer, V: PartialEqClone, I> FusedIterator for SortedDisjointWithLenSoFarMap<T, V, I> where
//     I: SortedDisjointMap<T, V> + FusedIterator
// {
// }

impl<'a, T, V, VR, I> Iterator for SortedDisjointWithLenSoFarMap<'a, T, V, VR, I>
where
    T: Integer,
    V: ValueOwned + 'a,
    VR: Deref<Target = V> + 'a,
    <V as ToOwned>::Owned: PartialEq,
    I: SortedDisjointMap<'a, T, V, VR>,
{
    type Item = (T, EndValue<T, V>);

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(range_value) = self.iter.next() {
            let (start, end) = range_value.range.clone().into_inner();
            debug_assert!(start <= end && end <= T::safe_max_value());
            self.len += T::safe_len(&range_value.range);
            let end_value = EndValue {
                end,
                value: range_value.value.to_owned(),
            };
            Some((start, end_value))
        } else {
            None
        }
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

#[derive(Clone, Debug)]
#[must_use = "iterators are lazy and do nothing unless consumed"]

/// Gives any iterator of ranges the [`SortedStartsMap`] trait without any checking.
#[doc(hidden)]
pub struct AssumeSortedStartsMap<'a, T, V, VR, I>
where
    T: Integer,
    V: ValueOwned + 'a,
    VR: Deref<Target = V> + 'a,
    I: Iterator<Item = RangeValue<'a, T, V, VR>>,
{
    pub(crate) iter: I,
}

impl<'a, T: Integer, V: ValueOwned + 'a, VR: Deref<Target = V> + 'a, I>
    SortedStartsMap<'a, T, V, VR> for AssumeSortedStartsMap<'a, T, V, VR, I>
where
    I: Iterator<Item = RangeValue<'a, T, V, VR>>,
{
}

impl<'a, T, V, VR, I> AssumeSortedStartsMap<'a, T, V, VR, I>
where
    T: Integer,
    V: ValueOwned + 'a,
    VR: Deref<Target = V> + 'a,
    I: Iterator<Item = RangeValue<'a, T, V, VR>>,
{
    pub fn new(iter: I) -> Self {
        AssumeSortedStartsMap { iter }
    }
}

impl<'a, T, V, VR, I> FusedIterator for AssumeSortedStartsMap<'a, T, V, VR, I>
where
    T: Integer,
    V: ValueOwned + 'a,
    VR: Deref<Target = V> + 'a,
    I: Iterator<Item = RangeValue<'a, T, V, VR>> + FusedIterator,
{
}

impl<'a, T, V, VR, I> Iterator for AssumeSortedStartsMap<'a, T, V, VR, I>
where
    T: Integer,
    V: ValueOwned + 'a,
    VR: Deref<Target = V> + 'a,
    I: Iterator<Item = RangeValue<'a, T, V, VR>>,
{
    type Item = RangeValue<'a, T, V, VR>;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

impl<'a, T: Integer, V: ValueOwned + 'a, VR: Deref<Target = V> + 'a, I> From<I>
    for SortedDisjointWithLenSoFarMap<'a, T, V, VR, I::IntoIter>
where
    I: IntoIterator<Item = RangeValue<'a, T, V, VR>>,
    I::IntoIter: SortedDisjointMap<'a, T, V, VR>,
    <V as ToOwned>::Owned: PartialEq,
{
    fn from(into_iter: I) -> Self {
        SortedDisjointWithLenSoFarMap {
            iter: into_iter.into_iter(),
            len: <T as Integer>::SafeLen::zero(),
            _phantom_data: PhantomData,
        }
    }
}
