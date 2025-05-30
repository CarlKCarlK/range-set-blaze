// impl<T, I> MultiwayRangeMapBlazeRef<T> for I
// where
//     T: Integer,
//     I: IntoIterator<Item = RangeMapBlaze<T, V>>,
// {
// }

use crate::{
    Integer, IntersectionKMap, RangeMapBlaze, SortedDisjointMap, SymDiffIterMap, SymDiffKMergeMap,
    UnionIterMap, UnionKMergeMap, intersection_iter_map::IntersectionIterMap, map::ValueRef,
    range_values::RangeValuesToRangesIter,
};
use alloc::vec::Vec;

impl<T, V, I> MultiwayRangeMapBlaze<T, V> for I
where
    T: Integer,
    V: Eq + Clone,
    I: IntoIterator<Item = RangeMapBlaze<T, V>>,
{
}
/// Provides methods on zero or more [`RangeMapBlaze`]'s,
/// specifically [`union`], [`intersection`] and [`symmetric_difference`].
///
/// Also see [`MultiwayRangeMapBlazeRef`].
///
/// [`union`]: MultiwayRangeMapBlaze::union
/// [`intersection`]: MultiwayRangeMapBlaze::intersection
/// [`symmetric_difference`]: MultiwayRangeMapBlaze::symmetric_difference
pub trait MultiwayRangeMapBlaze<T: Integer, V: Eq + Clone>:
    IntoIterator<Item = RangeMapBlaze<T, V>>
{
    /// Unions the given [`RangeMapBlaze`]'s, creating a new [`RangeMapBlaze`].
    /// Any number of input can be given.
    ///
    /// For exactly two inputs, you can also use the '|' operator.
    /// Also see [`MultiwayRangeMapBlazeRef::union`].
    ///
    /// # Performance
    ///
    ///  All work is done on demand, in one pass through the inputs. Minimal memory is used.
    ///
    /// # Example
    ///
    /// Find the integers that appear in any of the [`RangeMapBlaze`]'s.
    ///
    /// ```
    /// use range_set_blaze::prelude::*;
    ///
    /// let a = RangeMapBlaze::from_iter([(2..=2, "a"), (6..=200, "a")]);
    /// let b = RangeMapBlaze::from_iter([(2..=6, "b")]);
    /// let c = RangeMapBlaze::from_iter([(1..=2, "c"), (5..=100, "c")]);
    ///
    /// let union = [a, b, c].union();
    ///
    /// assert_eq!(union.to_string(), r#"(1..=2, "c"), (3..=4, "b"), (5..=100, "c"), (101..=200, "a")"#);
    /// ```
    fn union(self) -> RangeMapBlaze<T, V>
    where
        Self: Sized,
    {
        self.into_iter()
            .map(RangeMapBlaze::into_range_values)
            .union()
            .into_range_map_blaze()
    }

    /// Intersects the given [`RangeMapBlaze`]'s, creating a new [`RangeMapBlaze`].
    /// Any number of input can be given.
    ///
    /// For exactly two inputs, you can also use the '&' operator.
    /// Also see [`MultiwayRangeMapBlazeRef::intersection`].
    ///
    ///
    /// # Panics
    ///
    /// The intersection of zero maps causes a panic. Mathematically, it could be
    /// a mapping from all integers to some fill-in value but we don't implement that.
    ///
    /// # Performance
    ///
    ///  All work is done on demand, in one pass through the inputs. Minimal memory is used.
    ///
    /// # Example
    ///
    /// Find the integers that appear in all the [`RangeMapBlaze`]'s.
    ///
    /// ```
    /// use range_set_blaze::prelude::*;
    ///
    /// let a = RangeMapBlaze::from_iter([(2..=2, "a"), (6..=200, "a")]);
    /// let b = RangeMapBlaze::from_iter([(2..=6, "b")]);
    /// let c = RangeMapBlaze::from_iter([(1..=2, "c"), (5..=100, "c")]);
    ///
    /// let intersection = [a, b, c].intersection();
    ///
    /// assert_eq!(intersection.to_string(), r#"(2..=2, "c"), (6..=6, "c")"#);
    /// ```
    fn intersection(self) -> RangeMapBlaze<T, V>
    where
        Self: Sized,
    {
        self.into_iter()
            .map(RangeMapBlaze::into_range_values)
            .intersection()
            .into_range_map_blaze()
    }

    /// Symmetric difference on the given [`RangeMapBlaze`]'s, creating a new [`RangeMapBlaze`].
    /// ```
    /// use range_set_blaze::prelude::*;
    ///
    /// let a = RangeMapBlaze::from_iter([(2..=2, "a"), (6..=200, "a")]);
    /// let b = RangeMapBlaze::from_iter([(2..=6, "b")]);
    /// let c = RangeMapBlaze::from_iter([(1..=2, "c"), (5..=100, "c")]);
    ///
    /// let symmetric_difference = [a, b, c].symmetric_difference();
    ///
    /// assert_eq!(symmetric_difference.to_string(), r#"(1..=2, "c"), (3..=4, "b"), (6..=6, "c"), (101..=200, "a")"#);
    /// ```
    fn symmetric_difference(self) -> RangeMapBlaze<T, V>
    where
        Self: Sized,
    {
        self.into_iter()
            .map(RangeMapBlaze::into_range_values)
            .symmetric_difference()
            .into_range_map_blaze()
    }
}

impl<'a, T, V, I> MultiwayRangeMapBlazeRef<'a, T, V> for I
where
    T: Integer + 'a,
    V: Eq + Clone + 'a,
    I: IntoIterator<Item = &'a RangeMapBlaze<T, V>>,
{
}
/// Provides methods on zero or more [`RangeMapBlaze`] references,
/// specifically [`union`], [`intersection`] and [`symmetric_difference`].
///
/// Also see [`MultiwayRangeMapBlaze`].
///
/// [`union`]: MultiwayRangeMapBlazeRef::union
/// [`intersection`]: MultiwayRangeMapBlazeRef::intersection
/// [`symmetric_difference`]: MultiwayRangeMapBlazeRef::symmetric_difference
pub trait MultiwayRangeMapBlazeRef<'a, T: Integer + 'a, V: Eq + Clone + 'a>:
    IntoIterator<Item = &'a RangeMapBlaze<T, V>> + Sized
{
    /// Unions the given [`RangeMapBlaze`] references, creating a new [`RangeMapBlaze`].
    /// Any number of input can be given.
    ///
    /// For exactly two inputs, you can also use the '|' operator.
    /// Also see [`MultiwayRangeMapBlaze::union`].
    ///
    /// # Performance
    ///
    ///  All work is done on demand, in one pass through the inputs. Minimal memory is used.
    ///
    /// # Example
    ///
    /// Find the integers that appear in any of the [`RangeMapBlaze`] references.
    ///
    /// ```
    /// use range_set_blaze::prelude::*;
    ///
    /// let a = RangeMapBlaze::from_iter([(2..=2, "a"), (6..=200, "a")]);
    /// let b = RangeMapBlaze::from_iter([(2..=6, "b")]);
    /// let c = RangeMapBlaze::from_iter([(1..=2, "c"), (5..=100, "c")]);
    ///
    /// let union = [a, b, c].union();
    ///
    /// assert_eq!(union.to_string(), r#"(1..=2, "c"), (3..=4, "b"), (5..=100, "c"), (101..=200, "a")"#);
    /// ```
    fn union(self) -> RangeMapBlaze<T, V> {
        self.into_iter()
            .map(RangeMapBlaze::range_values)
            .union()
            .into_range_map_blaze()
    }

    /// Intersects the given [`RangeMapBlaze`] references, creating a new [`RangeMapBlaze`].
    /// Any number of input can be given.
    ///
    /// For exactly two inputs, you can also use the '&' operator.
    /// Also see [`MultiwayRangeMapBlaze::intersection`].
    ///
    /// # Panics
    ///
    /// The intersection of zero maps causes a panic. Mathematically, it could be
    /// a mapping from all integers to some fill-in value but we don't implement that.
    ///
    /// # Performance
    ///
    ///  All work is done on demand, in one pass through the inputs. Minimal memory is used.
    ///
    /// # Example
    ///
    /// Find the integers that appear in all the [`RangeMapBlaze`] references.
    ///
    /// ```
    /// use range_set_blaze::prelude::*;
    ///
    /// let a = RangeMapBlaze::from_iter([(2..=2, "a"), (6..=200, "a")]);
    /// let b = RangeMapBlaze::from_iter([(2..=6, "b")]);
    /// let c = RangeMapBlaze::from_iter([(1..=2, "c"), (5..=100, "c")]);
    ///
    /// let intersection = [a, b, c].intersection();
    ///
    /// assert_eq!(intersection.to_string(), r#"(2..=2, "c"), (6..=6, "c")"#);
    /// ```
    fn intersection(self) -> RangeMapBlaze<T, V> {
        self.into_iter()
            .map(RangeMapBlaze::range_values)
            .intersection()
            .into_range_map_blaze()
    }

    /// Symmetric difference on the given [`RangeMapBlaze`] references, creating a new [`RangeMapBlaze`].
    /// ```
    /// use range_set_blaze::prelude::*;
    ///
    /// let a = RangeMapBlaze::from_iter([(1..=2, "a"), (5..=100, "a")]);
    /// let b = RangeMapBlaze::from_iter([(2..=6, "b")]);
    /// let c = RangeMapBlaze::from_iter([(2..=2, "c"), (6..=200, "c")]);
    ///
    /// let symmetric_difference = [a, b, c].symmetric_difference();
    ///
    /// assert_eq!(symmetric_difference.to_string(), r#"(1..=1, "a"), (2..=2, "c"), (3..=4, "b"), (6..=6, "c"), (101..=200, "c")"#);
    /// ```
    fn symmetric_difference(self) -> RangeMapBlaze<T, V> {
        self.into_iter()
            .map(RangeMapBlaze::range_values)
            .symmetric_difference()
            .into_range_map_blaze()
    }
}

impl<T, VR, II, I> MultiwaySortedDisjointMap<T, VR, I> for II
where
    T: Integer,
    VR: ValueRef,
    I: SortedDisjointMap<T, VR>,
    II: IntoIterator<Item = I>,
{
}

/// Provides methods on zero or more [`SortedDisjointMap`] iterators,
/// specifically [`union`], [`intersection`], and [`symmetric_difference`].
///
/// [`SortedDisjointMap`]: trait.SortedDisjointMap.html#table-of-contents
/// [`union`]: crate::MultiwaySortedDisjointMap::union
/// [`intersection`]: crate::MultiwaySortedDisjointMap::intersection
/// [`symmetric_difference`]: crate::MultiwaySortedDisjointMap::symmetric_difference
pub trait MultiwaySortedDisjointMap<T, VR, I>: IntoIterator<Item = I> + Sized
where
    T: Integer,
    VR: ValueRef,
    I: SortedDisjointMap<T, VR>,
{
    /// Unions the given [`SortedDisjointMap`] iterators, creating a new [`SortedDisjointMap`] iterator.
    /// The input iterators must be of the same type. Any number of input iterators can be given.
    ///
    /// For input iterators of different types, use the [`union_dyn!`] macro.
    ///
    /// [`SortedDisjointMap`]: trait.SortedDisjointMap.html#table-of-contents
    /// [`union_dyn!`]: crate::union_dyn
    ///
    /// For exactly two inputs, you can also use the `|` operator.
    ///
    ///
    /// # Performance
    ///
    ///  All work is done on demand, in one pass through the input iterators. Minimal memory is used.
    ///
    /// # Example
    ///
    /// Find the integers that appear in any of the [`SortedDisjointMap`] iterators.
    ///
    /// ```
    /// use range_set_blaze::prelude::*;
    ///
    /// let a = CheckSortedDisjointMap::new(vec![(2..=2, &"a"), (6..=200, &"a")]);
    /// let b = CheckSortedDisjointMap::new(vec![(2..=6, &"b")]);
    /// let c = CheckSortedDisjointMap::new(vec![(1..=2, &"c"), (5..=100, &"c")]);
    ///
    /// let union = [a, b, c].union();
    ///
    /// assert_eq!(union.into_string(), r#"(1..=2, "c"), (3..=4, "b"), (5..=100, "c"), (101..=200, "a")"#);
    /// ```
    fn union(self) -> UnionKMergeMap<T, VR, I> {
        UnionIterMap::new_k(self)
    }

    /// Intersects the given [`SortedDisjointMap`] iterators, creating a new [`SortedDisjointMap`] iterator.
    /// The input iterators must be of the same type. Any number of input iterators can be given.
    ///
    /// For input iterators of different types, use the [`intersection_dyn!`] macro.
    ///
    /// [`SortedDisjointMap`]: trait.SortedDisjointMap.html#table-of-contents
    /// [`intersection_dyn!`]: crate::intersection_dyn
    ///
    /// For exactly two inputs, you can also use the `&` operator.
    ///
    /// # Panics
    ///
    /// The intersection of zero maps causes a panic. Mathematically, it could be
    /// a mapping from all integers to some fill-in value but we don't implement that.
    ///
    /// # Performance
    ///
    ///  All work is done on demand, in one pass through the input iterators. Minimal memory is used.
    ///
    /// # Example
    ///
    /// Find the integers that appear in all the [`SortedDisjointMap`] iterators.
    ///
    /// ```
    /// use range_set_blaze::prelude::*;
    ///
    /// let a = CheckSortedDisjointMap::new(vec![(2..=2, &"a"), (6..=200, &"a")]);
    /// let b = CheckSortedDisjointMap::new(vec![(2..=6, &"b")]);
    /// let c = CheckSortedDisjointMap::new(vec![(1..=2, &"c"), (5..=100, &"c")]);
    ///
    /// let intersection = [a, b, c].intersection();
    ///
    /// assert_eq!(intersection.into_string(), r#"(2..=2, "c"), (6..=6, "c")"#);
    /// ```
    fn intersection<'a>(self) -> IntersectionKMap<'a, T, VR, I> {
        // We define map intersection -- in part -- in terms of set intersection.
        // Elsewhere, we define set intersection in terms of complement and (set/map) union.
        use crate::MultiwaySortedDisjoint;
        let mut iter = self.into_iter().collect::<Vec<_>>().into_iter().rev();
        let iter_map = iter
            .next()
            .expect("The intersection of 0 maps is undefined.");
        let iter_set = iter.map(RangeValuesToRangesIter::new).intersection();
        IntersectionIterMap::new(iter_map, iter_set)
    }

    /// Symmetric difference on the given [`SortedDisjointMap`] iterators, creating a new [`SortedDisjointMap`] iterator.
    ///
    /// For input iterators of different types, use the [`symmetric_difference_dyn!`] macro.
    ///
    /// [`SortedDisjointMap`]: trait.SortedDisjointMap.html#table-of-contents
    /// [`symmetric_difference_dyn!`]: crate::symmetric_difference_dyn
    ///
    /// For exactly two inputs, you can also use the `^` operator.
    ///
    ///
    /// ```
    /// use range_set_blaze::prelude::*;
    ///
    /// let a = CheckSortedDisjointMap::new(vec![(2..=2, &"a"), (6..=200, &"a")]);
    /// let b = CheckSortedDisjointMap::new(vec![(2..=6, &"b")]);
    /// let c = CheckSortedDisjointMap::new(vec![(1..=2, &"c"), (5..=100, &"c")]);
    ///
    /// let symmetric_difference = [a, b, c].symmetric_difference();
    ///
    /// assert_eq!(symmetric_difference.into_string(), r#"(1..=2, "c"), (3..=4, "b"), (6..=6, "c"), (101..=200, "a")"#);
    /// ```
    fn symmetric_difference(self) -> SymDiffKMergeMap<T, VR, I> {
        SymDiffIterMap::new_k(self)
    }
}
