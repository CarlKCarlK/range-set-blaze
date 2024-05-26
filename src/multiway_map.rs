// impl<T, I> MultiwayRangeMapBlazeRef<T> for I
// where
//     T: Integer,
//     I: IntoIterator<Item = RangeMapBlaze<T, V>>,
// {
// }

use crate::{
    intersection_iter_map::IntersectionIterMap,
    map::{EqClone, ValueRef},
    range_values::RangeValuesToRangesIter,
    BitAndMapWithRangeValues, BitOrMapKMerge, BitXorMapKMerge, Integer, RangeMapBlaze,
    SortedDisjointMap, SymDiffIterMap, UnionIterMap,
};

impl<T, V, I> MultiwayRangeMapBlaze<T, V> for I
where
    T: Integer,
    V: EqClone,
    I: IntoIterator<Item = RangeMapBlaze<T, V>>,
{
}
/// Provides methods on multiple [`RangeMapBlaze`]'s,
/// specifically [`union`], [`intersection`] and [`symmetric_difference`].
///
/// Also see [`MultiwayRangeMapBlazeRef`].
///
/// [`union`]: MultiwayRangeMapBlaze::union
/// [`intersection`]: MultiwayRangeMapBlaze::intersection
/// [`symmetric_difference`]: MultiwayRangeMapBlaze::symmetric_difference
pub trait MultiwayRangeMapBlaze<T: Integer, V: EqClone>:
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
    /// let a = RangeMapBlaze::from_iter([(1..=2, "a"), (5..=100, "a")]);
    /// let b = RangeMapBlaze::from_iter([(2..=6, "b")]);
    /// let c = RangeMapBlaze::from_iter([(2..=2, "c"), (6..=200, "c")]);
    ///
    /// let union = [a, b, c].into_iter().union();
    ///
    /// assert_eq!(union.to_string(), r#"(1..=2, "a"), (3..=4, "b"), (5..=100, "a"), (101..=200, "c")"#);
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
    /// let intersection = [a, b, c].into_iter().intersection();
    ///
    /// assert_eq!(intersection.to_string(), r#"(2..=2, "a"), (6..=6, "a")"#);
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
    /// let a = RangeMapBlaze::from_iter([(1..=2, "a"), (5..=100, "a")]);
    /// let b = RangeMapBlaze::from_iter([(2..=6, "b")]);
    /// let c = RangeMapBlaze::from_iter([(2..=2, "c"), (6..=200, "c")]);
    ///
    /// let symmetric_difference = [a, b, c].into_iter().symmetric_difference();
    ///
    /// assert_eq!(symmetric_difference.to_string(), r#"(1..=2, "a"), (3..=4, "b"), (6..=6, "a"), (101..=200, "c")"#);
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
    V: EqClone + 'a,
    I: IntoIterator<Item = &'a RangeMapBlaze<T, V>>,
{
}
/// Provide methods on multiple [`RangeMapBlaze`] references,
/// specifically [`union`], [`intersection`] and [`symmetric_difference`].
///
/// Also see [`MultiwayRangeMapBlaze`].
///
/// [`union`]: MultiwayRangeMapBlazeRef::union
/// [`intersection`]: MultiwayRangeMapBlazeRef::intersection
/// [`symmetric_difference`]: MultiwayRangeMapBlazeRef::symmetric_difference
pub trait MultiwayRangeMapBlazeRef<'a, T: Integer + 'a, V: EqClone + 'a>:
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
    /// The intersection of 0 maps is undefined. (We can create a universal set of integers, but we don't know that value to use.)
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
    /// assert_eq!(symmetric_difference.to_string(), r#"(1..=2, "a"), (3..=4, "b"), (6..=6, "a"), (101..=200, "c")"#);
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

/// Provides methods on multiple [`SortedDisjointMap`] iterators,
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
    /// For input iterators of different types, use the [`intersection_dyn!`] macro.
    ///
    /// [`SortedDisjointMap`]: trait.SortedDisjointMap.html#table-of-contents
    /// [`intersection_dyn!`]: crate::intersection_dyn
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
        let iter_set = iter.map(RangeValuesToRangesIter::new).intersection();
        IntersectionIterMap::new(iter_map, iter_set)
    }

    /// Symmetric difference on the given [`SortedDisjointMap`] iterators, creating a new [`SortedDisjointMap`] iterator.
    ///
    /// For input iterators of different types, use the [`symmetric_difference_dyn!`] macro.
    ///
    /// [`SortedDisjointMap`]: trait.SortedDisjointMap.html#table-of-contents
    /// [`symmetric_difference_dyn!`]: crate::symmetric_difference_dyn

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
        SymDiffIterMap::new_k(self)
    }
}
// cmk confirm that on ranges the union of 0 sets 0 empty and the intersection of 0 sets is the universal set.
// cmk why does the multiway.rs have a MultiwayRangeSetBlaze but not a MultiwayRangeMapBlazeRef?
// cmk on maps, the union is still empty, but the intersection is undefined because we can't give a value to T.s

#[cfg(feature = "std")]
#[test]
fn test_ref_union() {
    use crate::prelude::*;

    let a = RangeSetBlaze::from_iter([1..=2, 5..=100]);
    let b = RangeSetBlaze::from_iter([2..=6]);
    let c = RangeSetBlaze::from_iter([2..=2, 6..=200]);
    let d = [&a, &b, &c].union();
    println!("{d}");
    assert_eq!(d, RangeSetBlaze::from_iter([1..=200]));

    let a = RangeSetBlaze::from_iter([1..=2, 5..=100]);
    let b = RangeSetBlaze::from_iter([2..=6]);
    let c = RangeSetBlaze::from_iter([2..=2, 6..=200]);
    let d = [a, b, c].union();
    println!("{d}");
    assert_eq!(d, RangeSetBlaze::from_iter([1..=200]));

    let a = RangeMapBlaze::from_iter([(1..=2, "a"), (5..=100, "a")]);
    let b = RangeMapBlaze::from_iter([(2..=6, "b")]);
    let c = RangeMapBlaze::from_iter([(2..=2, "c"), (6..=200, "c")]);
    let d = [&a, &b, &c].union();
    println!("{d}");
    assert_eq!(
        d,
        RangeMapBlaze::from_iter([(1..=2, "a"), (3..=4, "b"), (5..=100, "a"), (101..=200, "c")])
    );

    let a = RangeMapBlaze::from_iter([(1..=2, "a"), (5..=100, "a")]);
    let b = RangeMapBlaze::from_iter([(2..=6, "b")]);
    let c = RangeMapBlaze::from_iter([(2..=2, "c"), (6..=200, "c")]);
    let d: RangeMapBlaze<i32, &str> = [a, b, c].union();
    println!("{d}");
    assert_eq!(
        d,
        RangeMapBlaze::from_iter([(1..=2, "a"), (3..=4, "b"), (5..=100, "a"), (101..=200, "c")])
    );
}
