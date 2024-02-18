use core::iter::FusedIterator;

use itertools::{Itertools, KMergeBy, MergeBy};

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
pub struct MergeMap<'a, T, V, L, R>
where
    T: Integer,
    V: PartialEq + 'a,
    L: SortedDisjointMap<'a, T, V>,
    R: SortedDisjointMap<'a, T, V>,
{
    #[allow(clippy::type_complexity)]
    iter: MergeBy<L, R, fn(&RangeValue<T, &'a V>, &RangeValue<T, &'a V>) -> bool>,
}

impl<'a, T, V, L, R> MergeMap<'a, T, V, L, R>
where
    T: Integer,
    V: PartialEq + 'a,
    L: SortedDisjointMap<'a, T, V>,
    R: SortedDisjointMap<'a, T, V>,
{
    /// Creates a new [`MergeMap`] iterator from two [`SortedDisjointMap`] iterators. See [`MergeMap`] for more details and examples.
    pub fn new(left: L, right: R) -> Self {
        Self {
            iter: left.merge_by(right, |a, b| a.range.start() < b.range.start()),
        }
    }
}

impl<'a, T, V, L, R> FusedIterator for MergeMap<'a, T, V, L, R>
where
    T: Integer,
    V: PartialEq + 'a,
    L: SortedDisjointMap<'a, T, V>,
    R: SortedDisjointMap<'a, T, V>,
{
}

impl<'a, T, V, L, R> Iterator for MergeMap<'a, T, V, L, R>
where
    T: Integer,
    V: PartialEq + 'a,
    L: SortedDisjointMap<'a, T, V>,
    R: SortedDisjointMap<'a, T, V>,
{
    type Item = RangeValue<T, &'a V>;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

impl<'a, T, V, L, R> SortedStartsMap<'a, T, V> for MergeMap<'a, T, V, L, R>
where
    T: Integer,
    V: PartialEq + 'a,
    L: SortedDisjointMap<'a, T, V>,
    R: SortedDisjointMap<'a, T, V>,
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
//     V: PartialEq,
//     I: SortedDisjointMap<'a, T, V>,
// {
//     #[allow(clippy::type_complexity)]
//     iter: KMergeBy<I, fn(&RangeValue<T, V>, &RangeValue<T, V>) -> bool>,
// }

// impl<T, V, I> KMergeMap<T, V, I>
// where
//     T: Integer,
//     V: PartialEq,
//     I: SortedDisjointMap<'a, T, V>,
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
//     V: PartialEq,
//     I: SortedDisjointMap<'a, T, V>,
// {
// }

// impl<T, V, I> Iterator for KMergeMap<T, V, I>
// where
//     T: Integer,
//     V: PartialEq,
//     I: SortedDisjointMap<'a, T, V>,
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
//     V: PartialEq,
//     I: SortedDisjointMap<'a, T, V>,
// {
// }
