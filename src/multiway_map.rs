// impl<T, I> MultiwayRangeMapBlazeRef<T> for I
// where
//     T: Integer,
//     I: IntoIterator<Item = RangeMapBlaze<T, V>>,
// {
// }

use crate::{
    Integer, IntersectionKMap, MultiwayInnerJoinIterMap, MultiwayOuterJoinIncrementalIterMap,
    MultiwayOuterJoinIterMap, MultiwaySweep, RangeMapBlaze, SortedDisjointMap, SymDiffIterMap,
    SymDiffKMergeMap, UnionIterMap, UnionKMergeMap, intersection_iter_map::IntersectionIterMap,
    map::ValueCarrier, range_values::RangeValuesToRangesIter,
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

    // TODO0(api-change): New public multiway join.
    /// Joins the given [`RangeMapBlaze`] references on the keys covered by **all** of them,
    /// creating a new [`RangeMapBlaze`] whose values are `f` applied to every input's value there.
    ///
    /// `f` receives one `&V` per input, in input order. This is a thin wrapper over
    /// [`MultiwaySortedDisjointMap::inner_join`]; see it for details (call order, merging, zero
    /// inputs, and performance).
    ///
    /// [`MultiwaySortedDisjointMap::inner_join`]: crate::MultiwaySortedDisjointMap::inner_join
    ///
    /// # Examples
    ///
    /// ```
    /// use range_set_blaze::prelude::*;
    ///
    /// let a = RangeMapBlaze::from_iter([(0..=9, 'x'), (10..=19, 'y')]);
    /// let b = RangeMapBlaze::from_iter([(5..=14, 'p')]);
    /// let c = RangeMapBlaze::from_iter([(0..=99, 'q')]);
    ///
    /// let labels = [&a, &b, &c].inner_join(|values| values.iter().copied().collect::<String>());
    /// assert_eq!(labels.to_string(), r#"(5..=9, "xpq"), (10..=14, "ypq")"#);
    /// ```
    fn inner_join<F, W>(self, f: F) -> RangeMapBlaze<T, W>
    where
        F: FnMut(&[&'a V]) -> W,
        W: Eq + Clone,
    {
        self.into_iter()
            .map(RangeMapBlaze::range_values)
            .inner_join(f)
            .into_range_map_blaze()
    }

    // TODO0(api-change): New public multiway join.
    /// Joins the given [`RangeMapBlaze`] references on the keys covered by **at least one** of
    /// them, creating a new [`RangeMapBlaze`] whose values are `f` applied to each input's value
    /// there (or `None`).
    ///
    /// `f` receives one slot per input, in input order, and is never called with every slot
    /// `None`. This is a thin wrapper over [`MultiwaySortedDisjointMap::outer_join`]; see it for
    /// details (call order, merging, zero inputs, and performance).
    ///
    /// [`MultiwaySortedDisjointMap::outer_join`]: crate::MultiwaySortedDisjointMap::outer_join
    ///
    /// # Examples
    ///
    /// ```
    /// use range_set_blaze::prelude::*;
    ///
    /// let a = RangeMapBlaze::from_iter([(0..=9, 1)]);
    /// let b = RangeMapBlaze::from_iter([(5..=14, 10)]);
    ///
    /// // Sum the values present, counting a missing input as 0.
    /// let sums = [&a, &b].outer_join(|slots| slots.iter().flatten().copied().sum::<i32>());
    /// assert_eq!(sums.to_string(), "(0..=4, 1), (5..=9, 11), (10..=14, 10)");
    /// ```
    fn outer_join<F, W>(self, f: F) -> RangeMapBlaze<T, W>
    where
        F: FnMut(&[Option<&'a V>]) -> W,
        W: Eq + Clone,
    {
        self.into_iter()
            .map(RangeMapBlaze::range_values)
            .outer_join(f)
            .into_range_map_blaze()
    }

    // TODO0(api-change): New public multiway join.
    /// Like [`outer_join`](MultiwayRangeMapBlazeRef::outer_join), but `f` also receives
    /// `changed_from`: the inputs that changed since its previous call, with their previous
    /// values. This is a thin wrapper over
    /// [`MultiwaySortedDisjointMap::outer_join_incremental`]; see it for details.
    ///
    /// [`MultiwaySortedDisjointMap::outer_join_incremental`]: crate::MultiwaySortedDisjointMap::outer_join_incremental
    ///
    /// # Examples
    ///
    /// ```
    /// use range_set_blaze::prelude::*;
    ///
    /// let a = RangeMapBlaze::from_iter([(0..=9, 1)]);
    /// let b = RangeMapBlaze::from_iter([(5..=14, 10)]);
    ///
    /// let mut sum = 0;
    /// let sums = [&a, &b].outer_join_incremental(|values, changed_from| {
    ///     for (index, previous) in changed_from {
    ///         sum -= previous.copied().unwrap_or(0);
    ///         sum += values[*index].copied().unwrap_or(0);
    ///     }
    ///     sum
    /// });
    /// assert_eq!(sums.to_string(), "(0..=4, 1), (5..=9, 11), (10..=14, 10)");
    /// ```
    fn outer_join_incremental<F, W>(self, f: F) -> RangeMapBlaze<T, W>
    where
        F: FnMut(&[Option<&'a V>], &[(usize, Option<&'a V>)]) -> W,
        W: Eq + Clone,
    {
        self.into_iter()
            .map(RangeMapBlaze::range_values)
            .outer_join_incremental(f)
            .into_range_map_blaze()
    }
}

impl<T, VC, II, I> MultiwaySortedDisjointMap<T, VC, I> for II
where
    T: Integer,
    VC: ValueCarrier,
    I: SortedDisjointMap<T, VC>,
    II: IntoIterator<Item = I>,
{
}

/// Provides methods on zero or more [`SortedDisjointMap`] iterators,
/// specifically [`union`], [`intersection`], and [`symmetric_difference`].
///
/// [`SortedDisjointMap`]: crate::SortedDisjointMap.html#table-of-contents
/// [`union`]: crate::MultiwaySortedDisjointMap::union
/// [`intersection`]: crate::MultiwaySortedDisjointMap::intersection
/// [`symmetric_difference`]: crate::MultiwaySortedDisjointMap::symmetric_difference
pub trait MultiwaySortedDisjointMap<T, VC, I>: IntoIterator<Item = I> + Sized
where
    T: Integer,
    VC: ValueCarrier,
    I: SortedDisjointMap<T, VC>,
{
    /// Unions the given [`SortedDisjointMap`] iterators, creating a new [`SortedDisjointMap`] iterator.
    /// The input iterators must be of the same type. Any number of input iterators can be given.
    ///
    /// For input iterators of different types, use the [`union_dyn!`] macro.
    ///
    /// [`SortedDisjointMap`]: crate::SortedDisjointMap.html#table-of-contents
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
    fn union(self) -> UnionKMergeMap<T, VC, I> {
        UnionIterMap::new_k(self)
    }

    /// Intersects the given [`SortedDisjointMap`] iterators, creating a new [`SortedDisjointMap`] iterator.
    /// The input iterators must be of the same type. Any number of input iterators can be given.
    ///
    /// For input iterators of different types, use the [`intersection_dyn!`] macro.
    ///
    /// [`SortedDisjointMap`]: crate::SortedDisjointMap.html#table-of-contents
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
    fn intersection<'a>(self) -> IntersectionKMap<'a, T, VC, I> {
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
    /// [`SortedDisjointMap`]: crate::SortedDisjointMap.html#table-of-contents
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
    fn symmetric_difference(self) -> SymDiffKMergeMap<T, VC, I> {
        SymDiffIterMap::new_k(self)
    }

    // TODO0(api-change): New public multiway join.
    /// Joins the given [`SortedDisjointMap`] iterators on the ranges covered by **all** of them,
    /// calling `f` with every input's value there and yielding its result.
    ///
    /// `f` receives one value carrier per input, in input order (for
    /// [`RangeMapBlaze::range_values`], a `&V`). It is called once per maximal range over which
    /// the inputs' values are constant, in ascending key order, so it may be `FnMut` and keep
    /// state. Results are carried by [`Owned`]; touching ranges with equal results are merged.
    /// Any number of inputs can be given; with zero inputs, "all inputs present" holds
    /// everywhere, so the result is the universal range with the value `f(&[])`.
    ///
    /// Unlike [`intersection`], which keeps one input's value, this combines all of them. For
    /// exactly two inputs, also see [`SortedDisjointMap::inner_join`], which yields pairs.
    ///
    /// [`SortedDisjointMap`]: crate::SortedDisjointMap.html#table-of-contents
    /// [`SortedDisjointMap::inner_join`]: crate::SortedDisjointMap::inner_join
    /// [`RangeMapBlaze::range_values`]: crate::RangeMapBlaze::range_values
    /// [`Owned`]: crate::Owned
    /// [`intersection`]: crate::MultiwaySortedDisjointMap::intersection
    ///
    /// # Performance
    ///
    /// One pass through the inputs. Each input range costs O(log k) for k inputs, as with
    /// [`union`](crate::MultiwaySortedDisjointMap::union); the slice passed to `f` is updated in
    /// place, not rebuilt.
    ///
    /// # Examples
    ///
    /// ```
    /// use range_set_blaze::prelude::*;
    ///
    /// let a = RangeMapBlaze::from_iter([(0..=9, 1), (10..=19, 2)]);
    /// let b = RangeMapBlaze::from_iter([(5..=14, 10)]);
    /// let c = RangeMapBlaze::from_iter([(0..=99, 100)]);
    ///
    /// let sums = [a.range_values(), b.range_values(), c.range_values()]
    ///     .inner_join(|values| values.iter().copied().sum::<i32>());
    /// assert_eq!(sums.into_string(), "(5..=9, 111), (10..=14, 112)");
    /// ```
    fn inner_join<F, W>(self, f: F) -> MultiwayInnerJoinIterMap<T, VC, I, F, W>
    where
        F: FnMut(&[VC]) -> W,
        W: Eq + Clone,
    {
        MultiwayInnerJoinIterMap::new(self, f)
    }

    // TODO0(api-change): New public multiway join.
    /// Joins the given [`SortedDisjointMap`] iterators on the ranges covered by **at least one**
    /// of them, calling `f` with each input's value there (or `None`) and yielding its result.
    ///
    /// `f` receives one slot per input, in input order: `Some(value)` if that input covers the
    /// range, `None` if not. It is never called with every slot `None`. It is called once per
    /// maximal range over which the slots are constant, in ascending key order, so it may be
    /// `FnMut` and keep state. Results are carried by [`Owned`]; touching ranges with equal
    /// results are merged. With zero inputs, the result is empty.
    ///
    /// For exactly two inputs, also see [`SortedDisjointMap::outer_join`], which yields pairs.
    ///
    /// [`SortedDisjointMap`]: crate::SortedDisjointMap.html#table-of-contents
    /// [`SortedDisjointMap::outer_join`]: crate::SortedDisjointMap::outer_join
    /// [`Owned`]: crate::Owned
    ///
    /// # Performance
    ///
    /// One pass through the inputs. Each input range costs O(log k) for k inputs, as with
    /// [`union`](crate::MultiwaySortedDisjointMap::union); the slice passed to `f` is updated in
    /// place, not rebuilt.
    ///
    /// # Examples
    ///
    /// ```
    /// use range_set_blaze::prelude::*;
    ///
    /// let a = RangeMapBlaze::from_iter([(0..=9, "a")]);
    /// let b = RangeMapBlaze::from_iter([(5..=14, "b")]);
    ///
    /// // Which inputs cover each range?
    /// let coverage = [a.range_values(), b.range_values()].outer_join(|slots| {
    ///     slots.iter().flatten().copied().copied().collect::<Vec<_>>().join("+")
    /// });
    /// assert_eq!(
    ///     coverage.into_string(),
    ///     r#"(0..=4, "a"), (5..=9, "a+b"), (10..=14, "b")"#
    /// );
    /// ```
    fn outer_join<F, W>(self, f: F) -> MultiwayOuterJoinIterMap<T, VC, I, F, W>
    where
        F: FnMut(&[Option<VC>]) -> W,
        W: Eq + Clone,
    {
        MultiwayOuterJoinIterMap::new(self, f)
    }

    // TODO0(api-change): New public multiway join.
    /// Like [`outer_join`], but `f` is also told which inputs changed since its previous call, so
    /// it can maintain a running result in time proportional to the changes rather than to the
    /// number of inputs present.
    ///
    /// `f` receives `(values, changed_from)`:
    ///
    /// - `values`: one slot per input, in input order, exactly as for [`outer_join`].
    /// - `changed_from`: for each input whose slot differs from `f`'s previous call, its position
    ///   and its value at that previous call (`None` if it was absent). Its current value is
    ///   `values[position]`. Each input appears at most once. On the first call, every present
    ///   input is listed as changed from `None`. Gaps (no input present) are skipped, as for
    ///   [`outer_join`], so a call after a gap lists everything that left and entered across it.
    ///
    /// Use this when `f` aggregates over many inputs (a sum, a union, a count of distinct values)
    /// and only a few change from one range to the next. Otherwise, [`outer_join`] is simpler.
    /// Previous values are moved into `changed_from`, not cloned.
    ///
    /// [`outer_join`]: crate::MultiwaySortedDisjointMap::outer_join
    ///
    /// # Examples
    ///
    /// ```
    /// use range_set_blaze::prelude::*;
    ///
    /// let a = RangeMapBlaze::from_iter([(0..=9, 1), (10..=19, 2)]);
    /// let b = RangeMapBlaze::from_iter([(5..=14, 10)]);
    ///
    /// // Keep a running sum, adjusting it only for inputs that changed.
    /// let mut sum = 0;
    /// let sums = [a.range_values(), b.range_values()].outer_join_incremental(
    ///     |values, changed_from| {
    ///         for (index, previous) in changed_from {
    ///             sum -= previous.copied().unwrap_or(0);
    ///             sum += values[*index].copied().unwrap_or(0);
    ///         }
    ///         sum
    ///     },
    /// );
    /// assert_eq!(sums.into_string(), "(0..=4, 1), (5..=9, 11), (10..=14, 12), (15..=19, 2)");
    /// ```
    fn outer_join_incremental<F, W>(
        self,
        f: F,
    ) -> MultiwayOuterJoinIncrementalIterMap<T, VC, I, F, W>
    where
        F: FnMut(&[Option<VC>], &[(usize, Option<VC>)]) -> W,
        W: Eq + Clone,
    {
        MultiwayOuterJoinIncrementalIterMap::new(self, f)
    }

    // TODO0(api-change): New public multiway primitive.
    /// Sweeps the given [`SortedDisjointMap`] iterators, yielding each input range's start and
    /// end, in key order, as [`SweepEvent`]s.
    ///
    /// This is the low-level operation behind the multiway joins, for computations they do not
    /// cover (overlap depth, running aggregates, custom precedence). Every input range yields one
    /// [`SweepEvent::Start`] (with its range, input position, and value, moved rather than cloned)
    /// and later one [`SweepEvent::End`] (with its last key and input position). Events are in key
    /// order: a range ending at `p - 1` ends before a range starting at `p` starts, so between
    /// consecutive events the set of started-but-not-ended ranges is constant. The order of
    /// events of the same kind at the same key is unspecified.
    ///
    /// [`SortedDisjointMap`]: crate::SortedDisjointMap.html#table-of-contents
    /// [`SweepEvent`]: crate::SweepEvent
    /// [`SweepEvent::Start`]: crate::SweepEvent::Start
    /// [`SweepEvent::End`]: crate::SweepEvent::End
    ///
    /// # Performance
    ///
    /// One pass through the inputs, O(log k) per input range for k inputs. No values are cloned or
    /// stored.
    ///
    /// # Examples
    ///
    /// ```
    /// use range_set_blaze::{SweepEvent, prelude::*};
    ///
    /// let a = RangeMapBlaze::from_iter([(0..=9, "a")]);
    /// let b = RangeMapBlaze::from_iter([(5..=14, "b")]);
    /// let c = RangeMapBlaze::from_iter([(7..=8, "c")]);
    ///
    /// // The most inputs that overlap any key.
    /// let mut depth = 0;
    /// let mut max_depth = 0;
    /// for event in [a.range_values(), b.range_values(), c.range_values()].sweep() {
    ///     match event {
    ///         SweepEvent::Start { .. } => {
    ///             depth += 1;
    ///             max_depth = max_depth.max(depth);
    ///         }
    ///         SweepEvent::End { .. } => depth -= 1,
    ///     }
    /// }
    /// assert_eq!(max_depth, 3);
    /// ```
    fn sweep(self) -> MultiwaySweep<T, VC, I> {
        MultiwaySweep::new(self)
    }
}
