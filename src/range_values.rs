use crate::{sorted_disjoint_map::RangeValue, Integer};
use alloc::collections::btree_map;
use core::{iter::FusedIterator, marker::PhantomData, ops::RangeInclusive};

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
}

impl<'a, T: Integer, V: ValueOwned> AsRef<RangeValuesIter<'a, T, V>> for RangeValuesIter<'a, T, V> {
    fn as_ref(&self) -> &Self {
        // Self is RangeValuesIter<'a>, the type for which we impl AsRef
        self
    }
}

// RangeValuesIter (one of the iterators from RangeSetBlaze) is SortedDisjoint
impl<'a, T: Integer, V: ValueOwned> SortedStartsMap<'a, T, V> for RangeValuesIter<'a, T, V> {}
impl<'a, T: Integer, V: ValueOwned> SortedDisjointMap<'a, T, V> for RangeValuesIter<'a, T, V> {}

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
    type Item = RangeValue<'a, T, V>; // Assuming VR is always &'a V for next

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|(start, end_value)| RangeValue {
            range: *start..=end_value.end,
            value: &end_value.value,
            priority: 0, // cmk don't use RangeValue here
        })
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

// cmk
// impl<T: Integer, V: ValueOwned> DoubleEndedIterator for RangeValuesIter<'_, T, V> {
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

// impl<'a, T: Integer, V: ValueOwned + 'a> SortedStartsMap<'a, T, V>
//     for IntoRangeValuesIter<'a, T, V>
// {
// }
// impl<'a, T: Integer, V: ValueOwned + 'a> SortedDisjointMap<'a, T, V>
//     for IntoRangeValuesIter<'a, T, V>
// {
// }

impl<'a, T: Integer, V: ValueOwned> ExactSizeIterator for IntoRangeValuesIter<'a, T, V> {
    #[must_use]
    fn len(&self) -> usize {
        self.iter.len()
    }
}

impl<'a, T: Integer, V: ValueOwned> FusedIterator for IntoRangeValuesIter<'a, T, V> {}

impl<'a, T: Integer, V: ValueOwned + 'a> Iterator for IntoRangeValuesIter<'a, T, V> {
    type Item = (RangeInclusive<T>, V);

    fn next(&mut self) -> Option<Self::Item> {
        self.iter
            .next()
            .map(|(start, end_value)| (start..=end_value.end, end_value.value))
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

// cmk
// impl<T: Integer, V: ValueOwned> ops::Not for RangeValuesIter<'_, T, V> {
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

// impl<T: Integer, V: ValueOwned, I> ops::BitOr<I> for RangeValuesIter<'_, T, V>
// where
//     I: SortedDisjointMap<'a, T, V>,
// {
//     type Output = BitOrMerge<T, Self, I>;

//     fn bitor(self, other: I) -> Self::Output {
//         SortedDisjoint::union(self, other)
//     }
// }

// impl<T: Integer, V: ValueOwned, I> ops::BitOr<I> for IntoRangeValuesIter<'a, T, V>
// where
//     I: SortedDisjointMap<'a, T, V>,
// {
//     type Output = BitOrMerge<T, Self, I>;

//     fn bitor(self, other: I) -> Self::Output {
//         SortedDisjoint::union(self, other)
//     }
// }

// impl<T: Integer, V: ValueOwned, I> ops::Sub<I> for RangeValuesIter<'_, T, V>
// where
//     I: SortedDisjointMap<'a, T, V>,
// {
//     type Output = BitSubMerge<T, Self, I>;

//     fn sub(self, other: I) -> Self::Output {
//         SortedDisjoint::difference(self, other)
//     }
// }

// impl<T: Integer, V: ValueOwned, I> ops::Sub<I> for IntoRangeValuesIter<'a, T, V>
// where
//     I: SortedDisjointMap<'a, T, V>,
// {
//     type Output = BitSubMerge<T, Self, I>;

//     fn sub(self, other: I) -> Self::Output {
//         SortedDisjoint::difference(self, other)
//     }
// }

// impl<T: Integer, V: ValueOwned, I> ops::BitXor<I> for RangeValuesIter<'_, T, V>
// where
//     I: SortedDisjointMap<'a, T, V>,
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
//     I: SortedDisjointMap<'a, T, V>,
// {
//     type Output = BitXOrTee<T, Self, I>;

//     #[allow(clippy::suspicious_arithmetic_impl)]
//     fn bitxor(self, other: I) -> Self::Output {
//         SortedDisjoint::symmetric_difference(self, other)
//     }
// }

// impl<T: Integer, V: ValueOwned, I> ops::BitAnd<I> for RangeValuesIter<'_, T, V>
// where
//     I: SortedDisjointMap<'a, T, V>,
// {
//     type Output = BitAndMerge<T, Self, I>;

//     #[allow(clippy::suspicious_arithmetic_impl)]
//     fn bitand(self, other: I) -> Self::Output {
//         SortedDisjoint::intersection(self, other)
//     }
// }

// impl<T: Integer, V: ValueOwned, I> ops::BitAnd<I> for IntoRangeValuesIter<'a, T, V>
// where
//     I: SortedDisjointMap<'a, T, V>,
// {
//     type Output = BitAndMerge<T, Self, I>;

//     #[allow(clippy::suspicious_arithmetic_impl)]
//     fn bitand(self, other: I) -> Self::Output {
//         SortedDisjoint::intersection(self, other)
//     }
// }
