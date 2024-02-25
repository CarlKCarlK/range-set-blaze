use core::iter::FusedIterator;
use core::marker::PhantomData;

use itertools::{Itertools, MergeBy};

use crate::map::{CloneBorrow, ValueOwned};
use crate::Integer;

use crate::sorted_disjoint_map::{RangeValue, SortedDisjointMap, SortedStartsMap};

/// Works with [`UnionIter`] to turn any number of [`SortedDisjointMap`] iterators into a [`SortedDisjointMap`] iterator of their union,
/// i.e., all the integers in any input iterator, as sorted & disjoint ranges.
///
/// Also see [`KMergeMap`].
///
/// [`SortedDisjointMap`]: crate::SortedDisjointMap
/// [`UnionIter`]: crate::UnionIter
///
/// # Examples
///
/// ```
/// use itertools::Itertools;
/// use range_set_blaze::{UnionIter, MergeMap, SortedDisjointMap, CheckSortedDisjoint};
///
/// let a = CheckSortedDisjoint::new(vec![1..=2, 5..=100].into_iter());
/// let b = CheckSortedDisjoint::from([2..=6]);
/// let union = UnionIter::new(MergeMap::new(a, b));
/// assert_eq!(union.to_string(), "1..=100");
///
/// // Or, equivalently:
/// let a = CheckSortedDisjoint::new(vec![1..=2, 5..=100].into_iter());
/// let b = CheckSortedDisjoint::from([2..=6]);
/// let c = a | b;
/// assert_eq!(c.to_string(), "1..=100")
/// ```
#[must_use = "iterators are lazy and do nothing unless consumed"]
pub struct MergeMap<'a, T, V, VR, L, R>
where
    T: Integer + 'a,
    V: ValueOwned + 'a,
    VR: CloneBorrow<V> + 'a,
    L: SortedDisjointMap<'a, T, V, VR>,
    R: SortedDisjointMap<'a, T, V, VR>,
{
    #[allow(clippy::type_complexity)]
    iter: MergeBy<L, R, fn(&RangeValue<'a, T, V, VR>, &RangeValue<'a, T, V, VR>) -> bool>,
    phantom: PhantomData<VR>,
}

impl<'a, T, V, VR, L, R> MergeMap<'a, T, V, VR, L, R>
where
    T: Integer,
    V: ValueOwned + 'a,
    VR: CloneBorrow<V> + 'a,
    L: SortedDisjointMap<'a, T, V, VR>,
    R: SortedDisjointMap<'a, T, V, VR>,
    <V as ToOwned>::Owned: PartialEq,
{
    /// Creates a new [`MergeMap`] iterator from two [`SortedDisjointMap`] iterators. See [`MergeMap`] for more details and examples.
    pub fn new(left: L, right: R) -> Self {
        Self {
            iter: left.merge_by(right, |a, b| a.range.start() < b.range.start()),
            phantom: PhantomData,
        }
    }
}

impl<'a, T, V, VR, L, R> FusedIterator for MergeMap<'a, T, V, VR, L, R>
where
    T: Integer,
    V: ValueOwned + 'a,
    VR: CloneBorrow<V> + 'a,
    L: SortedDisjointMap<'a, T, V, VR>,
    R: SortedDisjointMap<'a, T, V, VR>,
    <V as ToOwned>::Owned: PartialEq,
{
}

impl<'a, T, V, VR, L, R> Iterator for MergeMap<'a, T, V, VR, L, R>
where
    T: Integer,
    V: ValueOwned + 'a,
    VR: CloneBorrow<V> + 'a,
    L: SortedDisjointMap<'a, T, V, VR>,
    R: SortedDisjointMap<'a, T, V, VR>,
    <V as ToOwned>::Owned: PartialEq,
{
    type Item = RangeValue<'a, T, V, VR>;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

impl<'a, T, V, VR, L, R> SortedStartsMap<'a, T, V, VR> for MergeMap<'a, T, V, VR, L, R>
where
    T: Integer,
    V: ValueOwned + 'a,
    VR: CloneBorrow<V> + 'a,
    L: SortedDisjointMap<'a, T, V, VR>,
    R: SortedDisjointMap<'a, T, V, VR>,
    <V as ToOwned>::Owned: PartialEq,
{
}

// /// Works with [`UnionIter`] to turn two [`SortedDisjointMap`] iterators into a [`SortedDisjointMap`] iterator of their union,
// /// i.e., all the integers in any input iterator, as sorted & disjoint ranges.
// ///
// /// Also see [`MergeMap`].
// ///
// /// [`SortedDisjointMap`]: crate::SortedDisjointMap
// /// [`UnionIter`]: crate::UnionIter
// ///
// /// # Examples
// ///
// /// ```
// /// use itertools::Itertools;
// /// use range_set_blaze::{UnionIter, KMergeMap, MultiwaySortedDisjoint, SortedDisjointMap, CheckSortedDisjoint};
// ///
// /// let a = CheckSortedDisjoint::new(vec![1..=2, 5..=100].into_iter());
// /// let b = CheckSortedDisjoint::new(vec![2..=6].into_iter());
// /// let c = CheckSortedDisjoint::new(vec![-1..=-1].into_iter());
// /// let union = UnionIter::new(KMergeMap::new([a, b, c]));
// /// assert_eq!(union.to_string(), "-1..=-1, 1..=100");
// ///
// /// // Or, equivalently:
// /// let a = CheckSortedDisjoint::new(vec![1..=2, 5..=100].into_iter());
// /// let b = CheckSortedDisjoint::new(vec![2..=6].into_iter());
// /// let c = CheckSortedDisjoint::new(vec![-1..=-1].into_iter());
// /// let union = [a, b, c].union();
// /// assert_eq!(union.to_string(), "-1..=-1, 1..=100");
// /// ```
// #[derive(Clone, Debug)]
// #[must_use = "iterators are lazy and do nothing unless consumed"]
// pub struct KMergeMap<T, V, I>
// where
//     T: Integer,
//     V: PartialEqClone,
//     I: SortedDisjointMap<'a, T, V, VR>,
// {
//     #[allow(clippy::type_complexity)]
//     iter: KMergeBy<I, fn(&RangeValue<T, V>, &RangeValue<T, V>) -> bool>,
// }

// impl<T, V, I> KMergeMap<T, V, I>
// where
//     T: Integer,
//     V: PartialEqClone,
//     I: SortedDisjointMap<'a, T, V, VR>,
// {
//     /// Creates a new [`KMergeMap`] iterator from zero or more [`SortedDisjointMap`] iterators. See [`KMergeMap`] for more details and examples.
//     pub fn new<J>(iter: J) -> Self
//     where
//         J: IntoIterator<Item = I>,
//     {
//         Self {
//             iter: iter.into_iter().kmerge_by(|a, b| a.start() < b.start()),
//         }
//     }
// }

// impl<T, V, I> FusedIterator for KMergeMap<T, V, I>
// where
//     T: Integer,
//     V: PartialEqClone,
//     I: SortedDisjointMap<'a, T, V, VR>,
// {
// }

// impl<T, V, I> Iterator for KMergeMap<T, V, I>
// where
//     T: Integer,
//     V: PartialEqClone,
//     I: SortedDisjointMap<'a, T, V, VR>,
// {
//     type Item = RangeValue<T, V>;

//     fn next(&mut self) -> Option<Self::Item> {
//         self.iter.next()
//     }

//     fn size_hint(&self) -> (usize, Option<usize>) {
//         self.iter.size_hint()
//     }
// }

// impl<T, V, I> SortedStartsMap<T, V> for KMergeMap<T, V, I>
// where
//     T: Integer,
//     V: PartialEqClone,
//     I: SortedDisjointMap<'a, T, V, VR>,
// {
// }
