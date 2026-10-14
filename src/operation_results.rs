//! The types returned by set and map operations on sorted, disjoint streams.
//!
//! Each is a struct with a private field holding its current implementation, so the
//! implementation can change without changing the type. They are named after the operation, as
//! std names `btree_set::Union`; `Map` marks a map stream and `Multiway` marks k inputs.

use core::{iter::FusedIterator, ops::RangeInclusive};

use crate::{
    Integer, IntersectionIterMap, MultiwayUnionInner, NotIter, RangeValuesToRangesIter,
    SortedDisjoint, SortedDisjointMap, SymmetricDifferenceInner, UnionInner, map::ValueCarrier,
};

// Implementations. Where an operation is composed from other operations (for example,
// intersection as the complement of the union of complements), the composition uses the public
// result types, so the operation's body can be written with ordinary operators.
type IntersectionInner<T, L, R> = NotIter<T, Union<T, NotIter<T, L>, NotIter<T, R>>>;
type DifferenceInner<T, L, R> = NotIter<T, Union<T, NotIter<T, L>, R>>;
type MultiwayIntersectionInner<T, I> = NotIter<T, MultiwayUnion<T, NotIter<T, I>>>;
type NotMapInner<T, VC, I> = NotIter<T, RangeValuesToRangesIter<T, VC, I>>;
type IntersectionMapInner<T, VC, L, R> =
    IntersectionIterMap<T, VC, R, RangeValuesToRangesIter<T, VC, L>>;
type DifferenceMapInner<T, VC, L, R> = IntersectionIterMap<T, VC, L, NotMap<T, VC, R>>;
type MultiwayIntersectionMapInner<T, VC, I> =
    IntersectionIterMap<T, VC, I, MultiwayIntersection<T, RangeValuesToRangesIter<T, VC, I>>>;

/// Defines an operation result type: an iterator struct wrapping its implementation in a private
/// field, with a crate-private constructor.
macro_rules! operation_result {
    (
        $(#[$attr:meta])*
        $Name:ident<$($G:ident),+> wraps $Inner:ty, yields $Item:ty, where $($bound:tt)+
    ) => {
        $(#[$attr])*
        ///
        /// Its fields are private, so its implementation can change without changing its type.
        #[must_use = "iterators are lazy and do nothing unless consumed"]
        #[derive(Clone, Debug)]
        pub struct $Name<$($G),+>
        where
            $($bound)+
        {
            inner: $Inner,
        }

        impl<$($G),+> $Name<$($G),+>
        where
            $($bound)+
        {
            #[inline]
            pub(crate) const fn new(inner: $Inner) -> Self {
                Self { inner }
            }
        }

        impl<$($G),+> Iterator for $Name<$($G),+>
        where
            $($bound)+
        {
            type Item = $Item;

            #[inline]
            fn next(&mut self) -> Option<Self::Item> {
                self.inner.next()
            }

            #[inline]
            fn size_hint(&self) -> (usize, Option<usize>) {
                self.inner.size_hint()
            }
        }

        impl<$($G),+> FusedIterator for $Name<$($G),+> where $($bound)+ {}
    };
}

operation_result! {
    /// This `struct` is created by the [`union`] method on [`SortedDisjoint`] and the `|`
    /// operator. See [`union`] for details.
    ///
    /// [`SortedDisjoint`]: crate::SortedDisjoint
    /// [`union`]: crate::SortedDisjoint::union
    Union<T, L, R> wraps UnionInner<T, L, R>, yields RangeInclusive<T>,
    where T: Integer, L: SortedDisjoint<T>, R: SortedDisjoint<T>
}

operation_result! {
    /// This `struct` is created by the [`intersection`] method on [`SortedDisjoint`] and the `&`
    /// operator. See [`intersection`] for details.
    ///
    /// [`SortedDisjoint`]: crate::SortedDisjoint
    /// [`intersection`]: crate::SortedDisjoint::intersection
    Intersection<T, L, R> wraps IntersectionInner<T, L, R>, yields RangeInclusive<T>,
    where T: Integer, L: SortedDisjoint<T>, R: SortedDisjoint<T>
}

operation_result! {
    /// This `struct` is created by the [`difference`] method on [`SortedDisjoint`] and the `-`
    /// operator. See [`difference`] for details.
    ///
    /// [`SortedDisjoint`]: crate::SortedDisjoint
    /// [`difference`]: crate::SortedDisjoint::difference
    Difference<T, L, R> wraps DifferenceInner<T, L, R>, yields RangeInclusive<T>,
    where T: Integer, L: SortedDisjoint<T>, R: SortedDisjoint<T>
}

operation_result! {
    /// This `struct` is created by the [`symmetric_difference`] method on [`SortedDisjoint`] and
    /// the `^` operator. See [`symmetric_difference`] for details.
    ///
    /// [`SortedDisjoint`]: crate::SortedDisjoint
    /// [`symmetric_difference`]: crate::SortedDisjoint::symmetric_difference
    SymmetricDifference<T, L, R> wraps SymmetricDifferenceInner<T, L, R>, yields RangeInclusive<T>,
    where T: Integer, L: SortedDisjoint<T>, R: SortedDisjoint<T>
}

operation_result! {
    /// This `struct` is created by the [`union`] method on [`MultiwaySortedDisjoint`]. See
    /// [`union`] for details.
    ///
    /// [`MultiwaySortedDisjoint`]: crate::MultiwaySortedDisjoint
    /// [`union`]: crate::MultiwaySortedDisjoint::union
    MultiwayUnion<T, I> wraps MultiwayUnionInner<T, I>, yields RangeInclusive<T>,
    where T: Integer, I: SortedDisjoint<T>
}

operation_result! {
    /// This `struct` is created by the [`intersection`] method on [`MultiwaySortedDisjoint`]. See
    /// [`intersection`] for details.
    ///
    /// [`MultiwaySortedDisjoint`]: crate::MultiwaySortedDisjoint
    /// [`intersection`]: crate::MultiwaySortedDisjoint::intersection
    MultiwayIntersection<T, I> wraps MultiwayIntersectionInner<T, I>, yields RangeInclusive<T>,
    where T: Integer, I: SortedDisjoint<T>
}

operation_result! {
    /// This `struct` is created by the [`complement`] method on [`SortedDisjointMap`] and the `!`
    /// operator: the keys not in the map, as a set stream. See [`complement`] for details.
    ///
    /// [`SortedDisjointMap`]: crate::SortedDisjointMap
    /// [`complement`]: crate::SortedDisjointMap::complement
    NotMap<T, VC, I> wraps NotMapInner<T, VC, I>, yields RangeInclusive<T>,
    where T: Integer, VC: ValueCarrier, I: SortedDisjointMap<T, VC>
}

operation_result! {
    /// This `struct` is created by the [`intersection`] method on [`SortedDisjointMap`] and the
    /// `&` operator. See [`intersection`] for details.
    ///
    /// [`SortedDisjointMap`]: crate::SortedDisjointMap
    /// [`intersection`]: crate::SortedDisjointMap::intersection
    IntersectionMap<T, VC, L, R> wraps IntersectionMapInner<T, VC, L, R>,
    yields (RangeInclusive<T>, VC),
    where T: Integer, VC: ValueCarrier, L: SortedDisjointMap<T, VC>, R: SortedDisjointMap<T, VC>
}

operation_result! {
    /// This `struct` is created by the [`difference`] method on [`SortedDisjointMap`] and the `-`
    /// operator. See [`difference`] for details.
    ///
    /// [`SortedDisjointMap`]: crate::SortedDisjointMap
    /// [`difference`]: crate::SortedDisjointMap::difference
    DifferenceMap<T, VC, L, R> wraps DifferenceMapInner<T, VC, L, R>,
    yields (RangeInclusive<T>, VC),
    where T: Integer, VC: ValueCarrier, L: SortedDisjointMap<T, VC>, R: SortedDisjointMap<T, VC>
}

operation_result! {
    /// This `struct` is created by the [`intersection`] method on [`MultiwaySortedDisjointMap`].
    /// See [`intersection`] for details.
    ///
    /// [`MultiwaySortedDisjointMap`]: crate::MultiwaySortedDisjointMap
    /// [`intersection`]: crate::MultiwaySortedDisjointMap::intersection
    MultiwayIntersectionMap<T, VC, I> wraps MultiwayIntersectionMapInner<T, VC, I>,
    yields (RangeInclusive<T>, VC),
    where T: Integer, VC: ValueCarrier, I: SortedDisjointMap<T, VC>
}
