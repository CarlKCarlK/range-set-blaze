// impl<T, I> MultiwayRangeMapBlazeRef<T> for I
// where
//     T: Integer,
//     I: IntoIterator<Item = RangeMapBlaze<T, V>>,
// {
// }

use crate::{
    intersection_iter_map::IntersectionIterMap,
    map::{PartialEqClone, ValueRef},
    range_values::RangeValuesToRangesIter,
    BitAndMapWithRangeValues, BitOrMapKMerge, BitXorMapKMerge, Integer, RangeMapBlaze,
    SortedDisjointMap, UnionIterMap,
};

impl<'a, T, V, I> MultiwayRangeMapBlaze<'a, T, V> for I
where
    T: Integer + 'a,
    V: PartialEqClone + 'a,
    I: IntoIterator<Item = &'a RangeMapBlaze<T, V>>,
{
}
/// The trait used to provide methods on multiple [`RangeMapBlaze`]'s,
/// specifically [`union`], [`intersection`] and [`symmetric_difference`].
///
/// Also see [`MultiwayRangeMapBlazeRef`].
///
/// [`union`]: MultiwayRangeMapBlaze::union
/// [`intersection`]: MultiwayRangeMapBlaze::intersection
/// [`symmetric_difference`]: MultiwayRangeMapBlaze::symmetric_difference
pub trait MultiwayRangeMapBlaze<'a, T: Integer + 'a, V: PartialEqClone + 'a>:
    IntoIterator<Item = &'a RangeMapBlaze<T, V>> + Sized
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
    /// let a = RangeMapBlaze::from_iter([(1..=2, "a"), (5..=100, "a")]);
    /// let b = RangeMapBlaze::from_iter([(2..=6, "b")]);
    /// let c = RangeMapBlaze::from_iter([(2..=2, "c"), (6..=200, "c")]);
    ///
    /// let union = [a, b, c].union();
    ///
    /// assert_eq!(union.to_string(), r#"(1..=2, "a"), (3..=4, "b"), (5..=100, "a"), (101..=200, "c")"#);
    /// ```
    fn union(self) -> RangeMapBlaze<T, V> {
        self.into_iter()
            .map(|x| RangeMapBlaze::range_values(x))
            .union()
            .into_range_map_blaze()
    }

    /// Intersects the given [`RangeMapBlaze`]'s, creating a new [`RangeMapBlaze`].
    /// Any number of input can be given.
    ///
    /// For exactly two inputs, you can also use the '&' operator.
    /// Also see [`MultiwayRangeMapBlazeRef::intersection`].
    ///
    /// The intersection of 0 maps is undefined. (We can create a universal set of integers, but we don't know that value to use.)
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
    /// let a = RangeMapBlaze::from_iter([(1..=2, "a"), (5..=100, "a")]);
    /// let b = RangeMapBlaze::from_iter([(2..=6, "b")]);
    /// let c = RangeMapBlaze::from_iter([(2..=2, "c"), (6..=200, "c")]);
    ///
    /// let intersection = [a, b, c].intersection();
    ///
    /// assert_eq!(intersection.to_string(), r#"(2..=2, "a"), (6..=6, "a")"#);
    /// ```
    fn intersection(self) -> RangeMapBlaze<T, V> {
        self.into_iter()
            .map(|x| RangeMapBlaze::range_values(x))
            .intersection()
            .into_range_map_blaze()
    }

    /// Symmetric difference on the given [`RangeMapBlaze`]'s, creating a new [`RangeMapBlaze`].
    /// ```
    /// use range_set_blaze::prelude::*;
    ///
    /// let a = RangeMapBlaze::from_iter([(1..=2, "a"), (5..=100, "a")]);
    /// let b = RangeMapBlaze::from_iter([(2..=6, "b")]);
    /// let c = RangeMapBlaze::from_iter([(2..=2, "c"), (6..=200, "c")]);
    ///
    /// let symmetric_difference = [a, b, c].symmetric_difference();
    ///
    /// assert_eq!(symmetric_difference.to_string(), r#"(1..=2, "a"), (3..=4, "b"), (6..=6, "a"), (101..=200, "c")"#);
    /// ```
    fn symmetric_difference(self) -> RangeMapBlaze<T, V> {
        self.into_iter()
            .map(|x| RangeMapBlaze::range_values(x))
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

/// The trait used to define methods on multiple [`SortedDisjointMap`] iterators,
/// specifically [`union`] and [`intersection`].
///
/// [`union`]: crate::MultiwaySortedDisjointMap::union
/// [`intersection`]: crate::MultiwaySortedDisjointMap::intersection
pub trait MultiwaySortedDisjointMap<T, VR, I>: IntoIterator<Item = I> + Sized
where
    T: Integer,
    VR: ValueRef,
    I: SortedDisjointMap<T, VR>,
{
    /// Unions the given [`SortedDisjointMap`] iterators, creating a new [`SortedDisjointMap`] iterator.
    /// The input iterators must be of the same type. Any number of input iterators can be given.
    ///
    /// For input iterators of different types, use the [`union_dyn`] macro.
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
    /// let a = CheckSortedDisjointMap::new(vec![(1..=2, &"a"), (5..=100, &"a")]);
    /// let b = CheckSortedDisjointMap::new(vec![(2..=6, &"b")]);
    /// let c = CheckSortedDisjointMap::new(vec![(2..=2, &"c"), (6..=200, &"c")]);
    ///
    /// let union = [a, b, c].union();
    ///
    /// assert_eq!(union.into_string(), r#"(1..=2, "a"), (3..=4, "b"), (5..=100, "a"), (101..=200, "c")"#);
    /// ```
    fn union(self) -> BitOrMapKMerge<T, VR, I> {
        // cmk0 why does this not have .into_iter() but intersection does?
        UnionIterMap::new_k(self)
    }

    /// Intersects the given [`SortedDisjointMap`] iterators, creating a new [`SortedDisjointMap`] iterator.
    /// The input iterators must be of the same type. Any number of input iterators can be given.
    ///
    /// For input iterators of different types, use the [`intersection_dyn`] macro.
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
    /// let a = CheckSortedDisjointMap::new(vec![(1..=2, &"a"), (5..=100, &"a")]);
    /// let b = CheckSortedDisjointMap::new(vec![(2..=6, &"b")]);
    /// let c = CheckSortedDisjointMap::new(vec![(2..=2, &"c"), (6..=200, &"c")]);
    ///
    /// let intersection = [a, b, c].intersection();
    ///
    /// assert_eq!(intersection.into_string(), r#"(2..=2, "a"), (6..=6, "a")"#);
    /// ```
    fn intersection<'a>(self) -> BitAndMapWithRangeValues<'a, T, VR, I> {
        // We define map intersection -- in part -- in terms of set intersection.
        // Elsewhere, we define set intersection in terms of complement and (set/map) union.
        use crate::MultiwaySortedDisjoint;
        let mut iter = self.into_iter();
        let iter_map = iter
            .next()
            .expect("The intersection of 0 maps is undefined.");
        let iter_set = iter.map(|x| RangeValuesToRangesIter::new(x)).intersection();
        IntersectionIterMap::new(iter_map, iter_set)
    }

    /// Symmetric difference on the given [`SortedDisjointMap`] iterators, creating a new [`SortedDisjointMap`] iterator.
    /// ```
    /// use range_set_blaze::prelude::*;
    ///
    /// let a = CheckSortedDisjointMap::new(vec![(1..=2, &"a"), (5..=100, &"a")]);
    /// let b = CheckSortedDisjointMap::new(vec![(2..=6, &"b")]);
    /// let c = CheckSortedDisjointMap::new(vec![(2..=2, &"c"), (6..=200, &"c")]);
    ///
    /// let symmetric_difference = [a, b, c].symmetric_difference();
    ///
    /// assert_eq!(symmetric_difference.into_string(), r#"(1..=2, "a"), (3..=4, "b"), (6..=6, "a"), (101..=200, "c")"#);
    /// ```
    fn symmetric_difference(self) -> BitXorMapKMerge<T, VR, I> {
        BitXorMapKMerge::new_k(self)
    }
}
// cmk confirm that on ranges the union of 0 sets 0 empty and the intersection of 0 sets is the universal set.
// cmk why does the multiway.rs have a MultiwayRangeSetBlazeRef but not a MultiwayRangeMapBlazeRef?
// cmk on maps, the union is still empty, but the intersection is undefined because we can't give a value to T.s
