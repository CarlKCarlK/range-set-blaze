use crate::{map::BitOrMergeMap, sorted_disjoint_map::RangeValue, Integer};
use alloc::{collections::btree_map, rc::Rc};
use core::{
    iter::FusedIterator,
    marker::PhantomData,
    ops::{self, RangeInclusive},
};

use crate::{
    map::{EndValue, ValueOwned},
    sorted_disjoint_map::{SortedDisjointMap, SortedStartsMap},
};

/// An iterator that visits the ranges in the [`RangeSetBlaze`],
/// i.e., the integers as sorted & disjoint ranges.
///
/// This `struct` is created by the [`ranges`] method on [`RangeSetBlaze`]. See [`ranges`]'s
/// documentation for more.
///
/// [`RangeSetBlaze`]: crate::RangeSetBlaze
/// [`ranges`]: crate::RangeSetBlaze::ranges
#[derive(Clone)]
#[must_use = "iterators are lazy and do nothing unless consumed"]
pub struct RangeValuesIter<'a, T: Integer, V: ValueOwned> {
    pub(crate) iter: btree_map::Iter<'a, T, EndValue<T, V>>,
    pub(crate) priority: usize,
}

impl<'a, T: Integer, V: ValueOwned> AsRef<RangeValuesIter<'a, T, V>> for RangeValuesIter<'a, T, V> {
    fn as_ref(&self) -> &Self {
        // Self is RangeValuesIter<'a>, the type for which we impl AsRef
        self
    }
}

// RangeValuesIter (one of the iterators from RangeSetBlaze) is SortedDisjoint
impl<'a, T: Integer, V: ValueOwned> SortedStartsMap<'a, T, V, &'a V> for RangeValuesIter<'a, T, V> {}
impl<'a, T: Integer, V: ValueOwned> SortedDisjointMap<'a, T, V, &'a V>
    for RangeValuesIter<'a, T, V>
{
}

impl<T: Integer, V: ValueOwned> ExactSizeIterator for RangeValuesIter<'_, T, V> {
    #[must_use]
    fn len(&self) -> usize {
        self.iter.len()
    }
}

impl<'a, T: Integer, V: ValueOwned> FusedIterator for RangeValuesIter<'a, T, V> {}

// Range's iterator is just the inside BTreeMap iterator as values
impl<'a, T, V> Iterator for RangeValuesIter<'a, T, V>
where
    T: Integer,
    V: ValueOwned + 'a,
{
    type Item = RangeValue<'a, T, V, &'a V>; // Assuming VR is always &'a V for next

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|(start, end_value)| RangeValue {
            range: *start..=end_value.end,
            value: &end_value.value,
            priority: self.priority, // cmk??? don't use RangeValue here
            phantom: PhantomData,
        })
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

// cmk
// impl<T: Integer, V: ValueOwned> DoubleEndedIterator for RangeValuesIter<'_, T, V, VR> {
//     fn next_back(&mut self) -> Option<Self::Item> {
//         self.iter.next_back().map(|(start, end)| *start..=*end)
//     }
// }

#[must_use = "iterators are lazy and do nothing unless consumed"]
/// An iterator that moves out the ranges in the [`RangeSetBlaze`],
/// i.e., the integers as sorted & disjoint ranges.
///
/// This `struct` is created by the [`into_ranges`] method on [`RangeSetBlaze`]. See [`into_ranges`]'s
/// documentation for more.
///
/// [`RangeSetBlaze`]: crate::RangeSetBlaze
/// [`into_ranges`]: crate::RangeSetBlaze::into_ranges
pub struct IntoRangeValuesIter<'a, T: Integer + 'a, V: ValueOwned + 'a> {
    pub(crate) iter: btree_map::IntoIter<T, EndValue<T, V>>,
    phantom: PhantomData<&'a V>,
}

impl<'a, T: Integer, V: ValueOwned + 'a> SortedStartsMap<'a, T, V, Rc<V>>
    for IntoRangeValuesIter<'a, T, V>
{
}
impl<'a, T: Integer, V: ValueOwned + 'a> SortedDisjointMap<'a, T, V, Rc<V>>
    for IntoRangeValuesIter<'a, T, V>
{
}

impl<'a, T: Integer, V: ValueOwned> ExactSizeIterator for IntoRangeValuesIter<'a, T, V> {
    #[must_use]
    fn len(&self) -> usize {
        self.iter.len()
    }
}

impl<'a, T: Integer, V: ValueOwned> FusedIterator for IntoRangeValuesIter<'a, T, V> {}

impl<'a, T: Integer, V: ValueOwned + 'a> Iterator for IntoRangeValuesIter<'a, T, V> {
    type Item = RangeValue<'a, T, V, Rc<V>>;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|(start, end_value)| {
            let range = start..=end_value.end;
            RangeValue {
                range,
                value: Rc::new(end_value.value),
                priority: 0, // cmk don't use RangeValue here
                phantom: PhantomData,
            }
        })
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

// cmk
// impl<'a, T: Integer, V: ValueOwned> DoubleEndedIterator for IntoRangeValuesIter<'a, T, V> {
//     fn next_back(&mut self) -> Option<Self::Item> {
//         self.iter.next_back().map(|(start, end)| start..=end)
//     }
// }

/// cmk
#[derive(Clone)]
#[must_use = "iterators are lazy and do nothing unless consumed"]
pub struct RangesFromMapIter<'a, T: Integer, V: ValueOwned> {
    pub(crate) iter: btree_map::Iter<'a, T, EndValue<T, V>>,
    pub(crate) option_ranges: Option<RangeInclusive<T>>,
}

// RangesFromMapIter (one of the iterators from RangeSetBlaze) is SortedDisjoint
impl<'a, T: Integer, V: ValueOwned> crate::SortedStarts<T> for RangesFromMapIter<'a, T, V> {}
impl<'a, T: Integer, V: ValueOwned> crate::SortedDisjoint<T> for RangesFromMapIter<'a, T, V> {}

impl<'a, T: Integer, V: ValueOwned> FusedIterator for RangesFromMapIter<'a, T, V> {}

// Range's iterator is just the inside BTreeMap iterator as values
impl<'a, T, V> Iterator for RangesFromMapIter<'a, T, V>
where
    T: Integer,
    V: ValueOwned + 'a,
{
    type Item = RangeInclusive<T>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            // If no next value, return whatever is current (could be None)
            let Some((next_start, next_end_value)) = self.iter.next() else {
                return self.option_ranges.take();
            };
            let (next_start, next_end) = (*next_start, next_end_value.end);

            // If no current value, set current to next and loop
            let Some(current_range) = self.option_ranges.take() else {
                self.option_ranges = Some(next_start..=next_end);
                continue;
            };
            let (current_start, current_end) = current_range.into_inner();

            // If current range and next range are adjacent, merge them and loop
            if current_end + T::one() == next_start {
                self.option_ranges = Some(current_start..=next_end);
                continue;
            }

            self.option_ranges = Some(next_start..=next_end);
            return Some(current_start..=current_end);
        }
    }
}

// cmk
// impl<T: Integer, V: ValueOwned> DoubleEndedIterator for RangesFromMapIter<'_, T, V> {
//     fn next_back(&mut self) -> Option<Self::Item> {
//         self.iter.next_back().map(|(start, end)| *start..=*end)
//     }
// }

// cmk
// impl<T: Integer, V: ValueOwned> ops::Not for RangeValuesIter<'_, T, V, VR> {
//     type Output = NotIter<T, Self>;

//     fn not(self) -> Self::Output {
//         self.complement()
//     }
// }

// impl<T: Integer, V: ValueOwned> ops::Not for IntoRangeValuesIter<'a, T, V> {
//     type Output = NotIter<T, Self>;

//     fn not(self) -> Self::Output {
//         self.complement()
//     }
// }

impl<'a, T: Integer, V: ValueOwned, I> ops::BitOr<I> for RangeValuesIter<'a, T, V>
where
    I: SortedDisjointMap<'a, T, V, &'a V>,
{
    type Output = BitOrMergeMap<'a, T, V, &'a V, Self, I>;

    fn bitor(self, other: I) -> Self::Output {
        SortedDisjointMap::union(self, other)
    }
}

impl<'a, T: Integer, V: ValueOwned, I> ops::BitOr<I> for IntoRangeValuesIter<'a, T, V>
where
    I: SortedDisjointMap<'a, T, V, Rc<V>>,
{
    type Output = BitOrMergeMap<'a, T, V, Rc<V>, Self, I>;

    fn bitor(self, other: I) -> Self::Output {
        SortedDisjointMap::union(self, other)
    }
}

// impl<T: Integer, V: ValueOwned, I> ops::Sub<I> for RangeValuesIter<'_, T, V, VR>
// where
//     I: SortedDisjointMap<'a, T, V, VR>,
// {
//     type Output = BitSubMerge<T, Self, I>;

//     fn sub(self, other: I) -> Self::Output {
//         SortedDisjoint::difference(self, other)
//     }
// }

// impl<T: Integer, V: ValueOwned, I> ops::Sub<I> for IntoRangeValuesIter<'a, T, V>
// where
//     I: SortedDisjointMap<'a, T, V, VR>,
// {
//     type Output = BitSubMerge<T, Self, I>;

//     fn sub(self, other: I) -> Self::Output {
//         SortedDisjoint::difference(self, other)
//     }
// }

// impl<T: Integer, V: ValueOwned, I> ops::BitXor<I> for RangeValuesIter<'_, T, V, VR>
// where
//     I: SortedDisjointMap<'a, T, V, VR>,
// {
//     type Output = BitXOr<T, Self, I>;

//     #[allow(clippy::suspicious_arithmetic_impl)]
//     fn bitxor(self, other: I) -> Self::Output {
//         // We optimize by using self.clone() instead of tee
//         let lhs1 = self.clone();
//         let (rhs0, rhs1) = other.tee();
//         (self - rhs0) | (rhs1.difference(lhs1))
//     }
// }

// impl<T: Integer, V: ValueOwned, I> ops::BitXor<I> for IntoRangeValuesIter<'a, T, V>
// where
//     I: SortedDisjointMap<'a, T, V, VR>,
// {
//     type Output = BitXOrTee<T, Self, I>;

//     #[allow(clippy::suspicious_arithmetic_impl)]
//     fn bitxor(self, other: I) -> Self::Output {
//         SortedDisjoint::symmetric_difference(self, other)
//     }
// }

// impl<T: Integer, V: ValueOwned, I> ops::BitAnd<I> for RangeValuesIter<'_, T, V, VR>
// where
//     I: SortedDisjointMap<'a, T, V, VR>,
// {
//     type Output = BitAndMerge<T, Self, I>;

//     #[allow(clippy::suspicious_arithmetic_impl)]
//     fn bitand(self, other: I) -> Self::Output {
//         SortedDisjoint::intersection(self, other)
//     }
// }

// impl<T: Integer, V: ValueOwned, I> ops::BitAnd<I> for IntoRangeValuesIter<'a, T, V>
// where
//     I: SortedDisjointMap<'a, T, V, VR>,
// {
//     type Output = BitAndMerge<T, Self, I>;

//     #[allow(clippy::suspicious_arithmetic_impl)]
//     fn bitand(self, other: I) -> Self::Output {
//         SortedDisjoint::intersection(self, other)
//     }
// }
