use core::cmp::Ordering;
use core::iter::FusedIterator;
use core::num::NonZeroUsize;

use itertools::{Itertools, KMergeBy, MergeBy};

use crate::map::{CloneBorrow, ValueOwned};
use crate::range_values::{non_zero_checked_sub, AdjustPriorityMap, NonZeroConstant};
use crate::{Integer, RangeValue};
use alloc::borrow::ToOwned;

use crate::sorted_disjoint_map::{Priority, SortedDisjointMap, SortedStartsMap};

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
pub struct MergeMap<T, V, VR, L, R>
where
    T: Integer,
    V: ValueOwned,
    VR: CloneBorrow<V>,
    L: SortedDisjointMap<T, V, VR>,
    R: SortedDisjointMap<T, V, VR>,
{
    #[allow(clippy::type_complexity)]
    iter: MergeBy<
        AdjustPriorityMap<T, V, VR, L, NonZeroConstant>,
        AdjustPriorityMap<T, V, VR, R, NonZeroConstant>,
        fn(&Priority<T, V, VR>, &Priority<T, V, VR>) -> bool,
    >,
}

impl<T, V, VR, L, R> MergeMap<T, V, VR, L, R>
where
    T: Integer,
    V: ValueOwned,
    VR: CloneBorrow<V>,
    L: SortedDisjointMap<T, V, VR>,
    R: SortedDisjointMap<T, V, VR>,
    <V as ToOwned>::Owned: PartialEq, // cmk is this needed?
{
    /// Creates a new [`MergeMap`] iterator from two [`SortedDisjointMap`] iterators. See [`MergeMap`] for more details and examples.
    pub fn new(left: L, right: R) -> Self {
        let left = AdjustPriorityMap::new(
            left,
            NonZeroConstant {
                priority_number: NonZeroUsize::MAX,
            },
        );
        let right = AdjustPriorityMap::new(
            right,
            NonZeroConstant {
                priority_number: NonZeroUsize::Min,
            },
        );
        Self {
            iter: left.merge_by(right, |a, b| {
                a.range_value.range.start() < b.range_value.range.start()
            }),
        }
    }
}

impl<T, V, VR, L, R> FusedIterator for MergeMap<T, V, VR, L, R>
where
    T: Integer,
    V: ValueOwned,
    VR: CloneBorrow<V>,
    L: SortedDisjointMap<T, V, VR>,
    R: SortedDisjointMap<T, V, VR>,
    <V as ToOwned>::Owned: PartialEq, // cmk is this needed?
{
}

impl<T, V, VR, L, R> Iterator for MergeMap<T, V, VR, L, R>
where
    T: Integer,
    V: ValueOwned,
    VR: CloneBorrow<V>,
    L: SortedDisjointMap<T, V, VR>,
    R: SortedDisjointMap<T, V, VR>,
    <V as ToOwned>::Owned: PartialEq, // cmk is this needed?
{
    type Item = RangeValue<T, V, VR>;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|x| x.range_value)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

impl<T, V, VR, L, R> SortedStartsMap<T, V, VR> for MergeMap<T, V, VR, L, R>
where
    T: Integer,
    V: ValueOwned,
    VR: CloneBorrow<V>,
    L: SortedDisjointMap<T, V, VR>,
    R: SortedDisjointMap<T, V, VR>,
    <V as ToOwned>::Owned: PartialEq, // cmk is this needed?
{
}

/// Works with [`UnionIter`] to turn two [`SortedDisjointMap`] iterators into a [`SortedDisjointMap`] iterator of their union,
/// i.e., all the integers in any input iterator, as sorted & disjoint ranges.
///
/// Also see [`MergeMap`].
///
/// [`SortedDisjointMap`]: crate::SortedDisjointMap
/// [`UnionIter`]: crate::UnionIter
///
/// # Examples
///
/// ```
/// use itertools::Itertools;
/// use range_set_blaze::{UnionIter, KMergeMap, MultiwaySortedDisjoint, SortedDisjointMap, CheckSortedDisjoint};
///
/// let a = CheckSortedDisjoint::new(vec![1..=2, 5..=100].into_iter());
/// let b = CheckSortedDisjoint::new(vec![2..=6].into_iter());
/// let c = CheckSortedDisjoint::new(vec![-1..=-1].into_iter());
/// let union = UnionIter::new(KMergeMap::new([a, b, c]));
/// assert_eq!(union.to_string(), "-1..=-1, 1..=100");
///
/// // Or, equivalently:
/// let a = CheckSortedDisjoint::new(vec![1..=2, 5..=100].into_iter());
/// let b = CheckSortedDisjoint::new(vec![2..=6].into_iter());
/// let c = CheckSortedDisjoint::new(vec![-1..=-1].into_iter());
/// let union = [a, b, c].union();
/// assert_eq!(union.to_string(), "-1..=-1, 1..=100");
/// ```
#[derive(Debug)]
#[must_use = "iterators are lazy and do nothing unless consumed"]
pub struct KMergeMap<T, V, VR, I>
where
    T: Integer,
    V: ValueOwned,
    VR: CloneBorrow<V>,
    I: SortedDisjointMap<T, V, VR>,
{
    #[allow(clippy::type_complexity)]
    iter: KMergeBy<
        AdjustPriorityMap<T, V, VR, I, NonZeroConstant>,
        fn(&Priority<T, V, VR>, &Priority<T, V, VR>) -> bool,
    >,
}

impl<T, V, VR, I> KMergeMap<T, V, VR, I>
where
    T: Integer,
    V: ValueOwned,
    VR: CloneBorrow<V>,
    I: SortedDisjointMap<T, V, VR>,
{
    /// Creates a new [`KMergeMap`] iterator from zero or more [`SortedDisjointMap`] iterators. See [`KMergeMap`] for more details and examples.
    pub fn new<J>(iter: J) -> Self
    where
        J: IntoIterator<Item = I>,
    {
        // Prioritize from left to right
        let iter = iter.into_iter().enumerate().map(|(i, x)| {
            let priority_number = non_zero_checked_sub(NonZeroUsize::MAX, i).unwrap();
            AdjustPriorityMap::new(x, NonZeroConstant { priority_number })
        });
        // Merge RangeValues by start with ties broken by priority
        let iter: KMergeBy<
            AdjustPriorityMap<T, V, VR, I, NonZeroConstant>,
            fn(&Priority<T, V, VR>, &Priority<T, V, VR>) -> bool,
        > = iter.kmerge_by(|a, b| {
            match a
                .range_value
                .range
                .start()
                .cmp(&b.range_value.range.start())
            {
                Ordering::Less => true,
                Ordering::Equal => a.priority < b.priority,
                Ordering::Greater => false,
            }
        });
        Self { iter }
    }
}

impl<'a, T, V, VR, I> FusedIterator for KMergeMap<T, V, VR, I>
where
    T: Integer,
    V: ValueOwned,
    VR: CloneBorrow<V>,
    I: SortedDisjointMap<T, V, VR>,
{
}

impl<T, V, VR, I> Iterator for KMergeMap<T, V, VR, I>
where
    T: Integer,
    V: ValueOwned,
    VR: CloneBorrow<V>,
    I: SortedDisjointMap<T, V, VR>,
{
    type Item = Priority<T, V, VR>;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

// cmk0 remove
// impl<T, V, VR, I> SortedStartsMap<T, V, VR> for KMergeMap<T, V, VR, I>
// where
//     T: Integer,
//     V: ValueOwned,
//     VR: CloneBorrow<V>,
//     I: SortedDisjointMap<T, V, VR>,
// {
// }
