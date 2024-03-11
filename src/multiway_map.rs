// impl<T, I> MultiwayRangeMapBlazeRef<T> for I
// where
//     T: Integer,
//     I: IntoIterator<Item = RangeMapBlaze<T, V>>,
// {
// }

use crate::{
    intersection_iter_map::IntersectionIterMap,
    map::{CloneBorrow, ValueOwned},
    merge_map::KMergeMap,
    range_values::RangeValuesToRangesIter,
    BitOrKMergeMap, Integer, IntersectionMap, RangeMapBlaze, SortedDisjointMap, UnionIterMap,
};

// /// The trait used to provide methods on multiple [`RangeMapBlaze`] references,
// /// specifically [`union`] and [`intersection`].
// ///
// /// Also see [`MultiwayRangeMapBlaze`].
// ///
// /// [`union`]: MultiwayRangeMapBlazeRef::union
// /// [`intersection`]: MultiwayRangeMapBlazeRef::intersection
// pub trait MultiwayRangeMapBlazeRef<T: Integer, V: ValueOwned>:
//     IntoIterator<Item = RangeMapBlaze<T, V>> + Sized
// {
//     /// Unions the given [`RangeMapBlaze`] references, creating a new [`RangeMapBlaze`].
//     /// Any number of input can be given.
//     ///
//     /// For exactly two inputs, you can also use the '|' operator.
//     /// Also see [`MultiwayRangeMapBlaze::union`].
//     ///
//     /// # Performance
//     ///
//     ///  All work is done on demand, in one pass through the inputs. Minimal memory is used.
//     ///
//     /// # Example
//     ///
//     /// Find the integers that appear in any of the [`RangeMapBlaze`]'s.
//     ///
//     /// ```
//     /// use range_set_blaze::prelude::*;
//     ///
//     /// let a = RangeMapBlaze::from_iter([1..=6, 8..=9, 11..=15]);
//     /// let b = RangeMapBlaze::from_iter([5..=13, 18..=29]);
//     /// let c = RangeMapBlaze::from_iter([25..=100]);
//     ///
//     /// let union = vec![a, b, c].into_iter().union();
//     ///
//     /// assert_eq!(union, RangeMapBlaze::from_iter([1..=15, 18..=100]));
//     /// ```
//     fn union(self) -> RangeMapBlaze<T, V> {
//         todo!("cmk not tested");
//         // let vec: Vec<_> = self.into_iter().collect(); // cmk OK to hold the RangeValueIters in memory?
//         // let iter = vec.iter().map(|x| x.range_values());
//         // let iter = iter.union();
//         // RangeMapBlaze::from_sorted_disjoint_map(iter)
//     }

//     /// Intersects the given [`RangeMapBlaze`] references, creating a new [`RangeMapBlaze`].
//     /// Any number of input can be given.
//     ///
//     /// For exactly two inputs, you can also use the '&' operator.
//     /// Also see [`MultiwayRangeMapBlaze::intersection`].
//     ///
//     /// # Performance
//     ///
//     ///  All work is done on demand, in one pass through the inputs. Minimal memory is used.
//     ///
//     /// # Example
//     ///
//     /// Find the integers that appear in all the [`RangeMapBlaze`]'s.
//     ///
//     /// ```
//     /// use range_set_blaze::prelude::*;
//     ///
//     /// let a = RangeMapBlaze::from_iter([1..=6, 8..=9, 11..=15]);
//     /// let b = RangeMapBlaze::from_iter([5..=13, 18..=29]);
//     /// let c = RangeMapBlaze::from_iter([-100..=100]);
//     ///
//     /// let intersection = vec![a, b, c].into_iter().intersection();
//     ///
//     /// assert_eq!(intersection, RangeMapBlaze::from_iter([5..=6, 8..=9, 11..=13]));
//     /// ```
//     fn intersection(self) -> RangeMapBlaze<T, V> {
//         todo!("cmk")
//         // self.into_iter()
//         //     .map(RangeMapBlaze::into_ranges)
//         //     .intersection()
//         //     .into_range_map_blaze()
//     }
// }

impl<'a, T, V, I> MultiwayRangeMapBlaze<'a, T, V> for I
where
    T: Integer + 'a,
    V: ValueOwned + 'a,
    I: IntoIterator<Item = &'a RangeMapBlaze<T, V>>,
{
}
/// The trait used to provide methods on multiple [`RangeMapBlaze`]'s,
/// specifically [`union`] and [`intersection`].
///
/// Also see [`MultiwayRangeMapBlazeRef`].
///
/// [`union`]: MultiwayRangeMapBlaze::union
/// [`intersection`]: MultiwayRangeMapBlaze::intersection
pub trait MultiwayRangeMapBlaze<'a, T: Integer + 'a, V: ValueOwned + 'a>:
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
    /// let a = RangeMapBlaze::from_iter([1..=6, 8..=9, 11..=15]);
    /// let b = RangeMapBlaze::from_iter([5..=13, 18..=29]);
    /// let c = RangeMapBlaze::from_iter([25..=100]);
    ///
    /// let union = [a, b, c].union();
    ///
    /// assert_eq!(union, RangeMapBlaze::from_iter([1..=15, 18..=100]));
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
    /// let a = RangeMapBlaze::from_iter([1..=6, 8..=9, 11..=15]);
    /// let b = RangeMapBlaze::from_iter([5..=13, 18..=29]);
    /// let c = RangeMapBlaze::from_iter([-100..=100]);
    ///
    /// let intersection = [a, b, c].intersection();
    ///
    /// assert_eq!(intersection, RangeMapBlaze::from_iter([5..=6, 8..=9, 11..=13]));
    /// ```
    fn intersection(self) -> RangeMapBlaze<T, V> {
        self.into_iter()
            .map(|x| RangeMapBlaze::range_values(x))
            .intersection()
            .into_range_map_blaze()
    }
}

impl<'a, T, V, VR, II, I> MultiwaySortedDisjointMap<'a, T, V, VR, I> for II
where
    T: Integer + 'a,
    V: ValueOwned + 'a,
    VR: CloneBorrow<V> + 'a,
    I: SortedDisjointMap<'a, T, V, VR>,
    II: IntoIterator<Item = I>,
{
}

/// The trait used to define methods on multiple [`SortedDisjointMap`] iterators,
/// specifically [`union`] and [`intersection`].
///
/// [`union`]: crate::MultiwaySortedDisjointMap::union
/// [`intersection`]: crate::MultiwaySortedDisjointMap::intersection
pub trait MultiwaySortedDisjointMap<'a, T, V, VR, I>: IntoIterator<Item = I> + Sized
where
    T: Integer + 'a,
    V: ValueOwned + 'a,
    VR: CloneBorrow<V> + 'a,
    I: SortedDisjointMap<'a, T, V, VR>,
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
    /// let a = RangeMapBlaze::from_iter([1..=6, 8..=9, 11..=15]).into_ranges();
    /// let b = RangeMapBlaze::from_iter([5..=13, 18..=29]).into_ranges();
    /// let c = RangeMapBlaze::from_iter([25..=100]).into_ranges();
    ///
    /// let union = [a, b, c].union();
    ///
    /// assert_eq!(union.to_string(), "1..=15, 18..=100");
    /// ```
    fn union(self) -> BitOrKMergeMap<'a, T, V, VR, I> {
        UnionIterMap::new(KMergeMap::new(self))
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
    /// let a = RangeMapBlaze::from_iter([1..=6, 8..=9, 11..=15]).into_ranges();
    /// let b = RangeMapBlaze::from_iter([5..=13, 18..=29]).into_ranges();
    /// let c = RangeMapBlaze::from_iter([-100..=100]).into_ranges();
    ///
    /// let intersection = [a, b, c].intersection();
    ///
    /// assert_eq!(intersection.to_string(), "5..=6, 8..=9, 11..=13");
    /// ```
    fn intersection(self) -> IntersectionMap<'a, T, V, VR, I> {
        use crate::MultiwaySortedDisjoint;
        let mut iter = self.into_iter();
        let iter_map = iter
            .next()
            .expect("The intersection of 0 maps is undefined.");
        // cmk100000000000000
        let iter_set = iter
            .map(|x| RangeValuesToRangesIter::new(x))
            .intersection2();

        IntersectionIterMap::new(iter_map, iter_set)
    }
}
// cmk confirm that on ranges the union of 0 sets 0 empty and the intersection of 0 sets is the universal set.
// cmk on maps, the union is still empty, but the intersection is undefined because we can't give a value to T.s
