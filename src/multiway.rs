impl<T, I> MultiwayRangeSetBlaze<T> for I
where
    T: Integer,
    I: IntoIterator<Item = RangeSetBlaze<T>>,
{
}

/// Provides methods on zero or more [`RangeSetBlaze`]'s,
/// specifically [`union`], [`intersection`] and [`symmetric_difference`].
///
/// Also see [`MultiwayRangeSetBlazeRef`].
///
/// [`union`]: MultiwayRangeSetBlaze::union
/// [`intersection`]: MultiwayRangeSetBlaze::intersection
/// [`symmetric_difference`]: MultiwayRangeSetBlaze::symmetric_difference
#[allow(clippy::module_name_repetitions)]
pub trait MultiwayRangeSetBlaze<T: Integer>: IntoIterator<Item = RangeSetBlaze<T>> + Sized {
    /// Unions the given [`RangeSetBlaze`]'s', creating a new [`RangeSetBlaze`].
    /// Any number of input can be given.
    ///
    /// For exactly two inputs, you can also use the '|' operator.
    /// Also see [`MultiwayRangeSetBlazeRef::union`].
    ///
    /// # Performance
    ///
    ///  All work is done on demand, in one pass through the inputs. Minimal memory is used.
    ///
    /// # Example
    ///
    /// Find the integers that appear in any of the [`RangeSetBlaze`]'s.
    ///
    /// ```
    /// use range_set_blaze::prelude::*;
    ///
    /// let a = RangeSetBlaze::from_iter([1..=6, 8..=9, 11..=15]);
    /// let b = RangeSetBlaze::from_iter([5..=13, 18..=29]);
    /// let c = RangeSetBlaze::from_iter([25..=100]);
    ///
    /// let union = [a, b, c].union();
    ///
    /// assert_eq!(union, RangeSetBlaze::from_iter([1..=15, 18..=100]));
    /// ```
    fn union(self) -> RangeSetBlaze<T> {
        self.into_iter()
            .map(RangeSetBlaze::into_ranges)
            .union()
            .into_range_set_blaze()
    }

    /// Intersects the given [`RangeSetBlaze`] references, creating a new [`RangeSetBlaze`].
    /// Any number of input can be given.
    ///
    /// For exactly two inputs, you can also use the '&' operator.
    /// Also see [`MultiwayRangeSetBlazeRef::intersection`].
    ///
    /// When given zero inputs, it returns the universal set.
    ///
    /// # Performance
    ///
    ///  All work is done on demand, in one pass through the inputs. Minimal memory is used.
    ///
    /// # Example
    ///
    /// Find the integers that appear in all the [`RangeSetBlaze`]'s.
    ///
    /// ```
    /// use range_set_blaze::prelude::*;
    ///
    /// let a = RangeSetBlaze::from_iter([1..=6, 8..=9, 11..=15]);
    /// let b = RangeSetBlaze::from_iter([5..=13, 18..=29]);
    /// let c = RangeSetBlaze::from_iter([-100..=100]);
    ///
    /// let intersection = [a, b, c].intersection();
    ///
    /// assert_eq!(intersection, RangeSetBlaze::from_iter([5..=6, 8..=9, 11..=13]));
    /// ```
    fn intersection(self) -> RangeSetBlaze<T> {
        self.into_iter()
            .map(RangeSetBlaze::into_ranges)
            .intersection()
            .into_range_set_blaze()
    }

    /// Computes the symmetric difference of the given [`RangeSetBlaze`] references,
    /// creating a new [`RangeSetBlaze`].
    /// The symmetric difference is the set of elements that appear in an odd number
    /// of the input sets.
    ///
    /// Any number of inputs can be given.
    ///
    /// For exactly two inputs, you can also use the '^' operator.
    /// Also see [`MultiwayRangeSetBlazeRef::symmetric_difference`].
    ///
    /// # Performance
    ///
    /// All work is done on demand, in one pass through the inputs. Minimal memory is used.
    ///
    /// # Example
    ///
    /// Find the integers that appear in an odd number of the [`RangeSetBlaze`]s.
    ///
    /// ```
    /// use range_set_blaze::prelude::*;
    ///
    /// let a = RangeSetBlaze::from_iter([1..=6, 8..=9, 11..=15]);
    /// let b = RangeSetBlaze::from_iter([5..=13, 18..=29]);
    /// let c = RangeSetBlaze::from_iter([-100..=100]);
    ///
    /// let symmetric_difference = [a, b, c].symmetric_difference();
    ///
    /// assert_eq!(
    ///     symmetric_difference,
    ///     RangeSetBlaze::from_iter([
    ///         -100..=0, 5..=6, 8..=9, 11..=13, 16..=17, 30..=100
    ///     ])
    /// );
    /// ```
    fn symmetric_difference(self) -> RangeSetBlaze<T> {
        self.into_iter()
            .map(RangeSetBlaze::into_ranges)
            .symmetric_difference()
            .into_range_set_blaze()
    }
}
impl<'a, T, I> MultiwayRangeSetBlazeRef<'a, T> for I
where
    T: Integer + 'a,
    I: IntoIterator<Item = &'a RangeSetBlaze<T>>,
{
}
/// Provides methods on zero or more [`RangeSetBlaze`] references,
/// specifically [`union`], [`intersection`], and [`symmetric_difference`].
///
/// Also see [`MultiwayRangeSetBlaze`].
///
/// [`union`]: MultiwayRangeSetBlazeRef::union
/// [`intersection`]: MultiwayRangeSetBlazeRef::intersection
/// [`symmetric_difference`]: MultiwayRangeSetBlazeRef::symmetric_difference
#[allow(clippy::module_name_repetitions)]
pub trait MultiwayRangeSetBlazeRef<'a, T: Integer + 'a>:
    IntoIterator<Item = &'a RangeSetBlaze<T>> + Sized
{
    /// Unions the given [`RangeSetBlaze`] references, creating a new [`RangeSetBlaze`].
    /// Any number of input can be given.
    ///
    /// For exactly two inputs, you can also use the '|' operator.
    /// Also see [`MultiwayRangeSetBlaze::union`].
    ///
    /// # Performance
    ///
    ///  All work is done on demand, in one pass through the inputs. Minimal memory is used.
    ///
    /// # Example
    ///
    /// Find the integers that appear in any of the [`RangeSetBlaze`]'s.
    ///
    /// ```
    /// use range_set_blaze::prelude::*;
    ///
    /// let a = RangeSetBlaze::from_iter([1..=6, 8..=9, 11..=15]);
    /// let b = RangeSetBlaze::from_iter([5..=13, 18..=29]);
    /// let c = RangeSetBlaze::from_iter([25..=100]);
    ///
    /// let union = [&a, &b, &c].union();
    ///
    /// assert_eq!(union, RangeSetBlaze::from_iter([1..=15, 18..=100]));
    /// ```
    fn union(self) -> RangeSetBlaze<T> {
        self.into_iter()
            .map(RangeSetBlaze::ranges)
            .union()
            .into_range_set_blaze()
    }

    /// Intersects the given [`RangeSetBlaze`]'s, creating a new [`RangeSetBlaze`].
    /// Any number of input can be given.
    ///
    /// For exactly two inputs, you can also use the '&' operator.
    /// Also see [`MultiwayRangeSetBlaze::intersection`].
    ///
    /// When given zero inputs, it returns the universal set.
    ///
    /// # Performance
    ///
    ///  All work is done on demand, in one pass through the inputs. Minimal memory is used.
    ///
    /// # Example
    ///
    /// Find the integers that appear in all the [`RangeSetBlaze`]'s.
    ///
    /// ```
    /// use range_set_blaze::prelude::*;
    ///
    /// let a = RangeSetBlaze::from_iter([1..=6, 8..=9, 11..=15]);
    /// let b = RangeSetBlaze::from_iter([5..=13, 18..=29]);
    /// let c = RangeSetBlaze::from_iter([-100..=100]);
    ///
    /// let intersection = [&a, &b, &c].intersection();
    ///
    /// assert_eq!(intersection, RangeSetBlaze::from_iter([5..=6, 8..=9, 11..=13]));
    /// ```
    fn intersection(self) -> RangeSetBlaze<T> {
        self.into_iter()
            .map(RangeSetBlaze::ranges)
            .intersection()
            .into_range_set_blaze()
    }

    /// Computes the symmetric difference of the given [`RangeSetBlaze`],
    /// creating a new [`RangeSetBlaze`].
    /// The symmetric difference is the set of elements that appear in an odd number
    /// of the input sets.
    ///
    /// Any number of inputs can be given.
    ///
    /// For exactly two inputs, you can also use the '^' operator.
    /// Also see [`MultiwayRangeSetBlaze::symmetric_difference`].
    ///
    /// # Performance
    ///
    /// All work is done on demand, in one pass through the inputs. Minimal memory is used.
    ///
    /// # Example
    ///
    /// Find the integers that appear in an odd number of the [`RangeSetBlaze`]s.
    ///
    /// ```
    /// use range_set_blaze::prelude::*;
    ///
    /// let a = RangeSetBlaze::from_iter([1..=6, 8..=9, 11..=15]);
    /// let b = RangeSetBlaze::from_iter([5..=13, 18..=29]);
    /// let c = RangeSetBlaze::from_iter([-100..=100]);
    ///
    /// let symmetric_difference = [&a, &b, &c].symmetric_difference();
    ///
    /// assert_eq!(
    ///     symmetric_difference,
    ///     RangeSetBlaze::from_iter([
    ///         -100..=0, 5..=6, 8..=9, 11..=13, 16..=17, 30..=100
    ///     ])
    /// );
    /// ```
    fn symmetric_difference(self) -> RangeSetBlaze<T> {
        self.into_iter()
            .map(RangeSetBlaze::ranges)
            .symmetric_difference()
            .into_range_set_blaze()
    }
}

use crate::{
    Integer, IntersectionMapInternal, RangeSetBlaze, SortedDisjoint, SymDiffIter, SymDiffKMerge,
    UnionIter, UnionKMerge,
};

impl<T, II, I> MultiwaySortedDisjoint<T, I> for II
where
    T: Integer,
    I: SortedDisjoint<T>,
    II: IntoIterator<Item = I>,
{
}

/// Provides methods on zero or more [`SortedDisjoint`] iterators,
/// specifically [`union`], [`intersection`], and [`symmetric_difference`].
///
/// [SortedDisjoint]: crate::SortedDisjoint.html#table-of-contents
/// [`union`]: crate::MultiwaySortedDisjoint::union
/// [`intersection`]: crate::MultiwaySortedDisjoint::intersection
/// [`symmetric_difference`]: crate::MultiwaySortedDisjoint::symmetric_difference
#[allow(clippy::module_name_repetitions)]
pub trait MultiwaySortedDisjoint<T: Integer, I>: IntoIterator<Item = I> + Sized
where
    I: SortedDisjoint<T>,
{
    /// Unions the given [`SortedDisjoint`] iterators, creating a new [`SortedDisjoint`] iterator.
    /// The input iterators must be of the same type. Any number of input iterators can be given.
    ///
    /// For input iterators of different types, use the [`union_map_dyn!`] macro.
    ///
    /// [`union_map_dyn!`]: crate::union_map_dyn
    /// [SortedDisjoint]: crate::SortedDisjoint.html#table-of-contents
    ///
    /// # Performance
    ///
    ///  All work is done on demand, in one pass through the input iterators. Minimal memory is used.
    ///
    /// # Example
    ///
    /// Find the integers that appear in any of the [`SortedDisjoint`] iterators.
    ///
    /// ```
    /// use range_set_blaze::prelude::*;
    ///
    /// let a = RangeSetBlaze::from_iter([1..=6, 8..=9, 11..=15]).into_ranges();
    /// let b = RangeSetBlaze::from_iter([5..=13, 18..=29]).into_ranges();
    /// let c = RangeSetBlaze::from_iter([25..=100]).into_ranges();
    ///
    /// let union = [a, b, c].union();
    ///
    /// assert_eq!(union.into_string(), "1..=15, 18..=100");
    /// ```
    fn union(self) -> UnionKMerge<T, I> {
        UnionIter::new_k(self)
    }

    /// Intersects the given [`SortedDisjoint`] iterators, creating a new [`SortedDisjoint`] iterator.
    /// The input iterators must be of the same type. Any number of input iterators can be given.
    ///
    /// For input iterators of different types, use the [`intersection_map_dyn!`] macro.
    ///
    /// [`intersection_map_dyn!`]: crate::intersection_map_dyn
    /// [SortedDisjoint]: crate::SortedDisjoint.html#table-of-contents
    ///
    /// For exactly two inputs, you can also use the `&` operator.
    /// When given zero inputs, it returns the universal set.
    ///
    /// # Performance
    ///
    ///  All work is done on demand, in one pass through the input iterators. Minimal memory is used.
    ///
    /// # Example
    ///
    /// Find the integers that appear in all the [`SortedDisjoint`] iterators.
    ///
    /// ```
    /// use range_set_blaze::prelude::*;
    ///
    /// let a = RangeSetBlaze::from_iter([1..=6, 8..=9, 11..=15]).into_ranges();
    /// let b = RangeSetBlaze::from_iter([5..=13, 18..=29]).into_ranges();
    /// let c = RangeSetBlaze::from_iter([-100..=100]).into_ranges();
    ///
    /// let intersection = [a, b, c].intersection();
    ///
    /// assert_eq!(intersection.into_string(), "5..=6, 8..=9, 11..=13");
    /// ```
    fn intersection(self) -> IntersectionMapInternal<T, I> {
        // We define set intersection in terms of complement and (set/map) union.
        // Elsewhere, map intersection is defined -- in part -- in terms of set intersection.
        self.into_iter()
            .map(|seq| seq.into_iter().complement())
            .union()
            .complement()
    }

    /// Computes the symmetric difference of the given [`SortedDisjoint`] iterators, creating a new [`SortedDisjoint`] iterator.
    /// The symmetric difference is the set of elements that appear in an odd number of the input iterators.
    /// The input iterators must be of the same type. Any number of input iterators can be given.
    ///
    /// For input iterators of different types, use the [`symmetric_difference_map_dyn!`] macro.
    ///
    /// [`symmetric_difference_map_dyn!`]: crate::symmetric_difference_map_dyn
    /// [SortedDisjoint]: crate::SortedDisjoint.html#table-of-contents
    ///
    /// # Performance
    ///
    /// All work is done on demand, in one pass through the input iterators. Minimal memory is used.
    ///
    /// # Example
    ///
    /// Find the integers that appear in an odd number of the [`SortedDisjoint`] iterators.
    ///
    /// ```
    /// use range_set_blaze::prelude::*;
    ///
    /// let a = RangeSetBlaze::from_iter([1..=6, 8..=9, 11..=15]).into_ranges();
    /// let b = RangeSetBlaze::from_iter([5..=13, 18..=29]).into_ranges();
    /// let c = RangeSetBlaze::from_iter([-100..=100]).into_ranges();
    ///
    /// let symmetric_difference = [a, b, c].symmetric_difference();
    ///
    /// assert_eq!(
    ///     symmetric_difference.into_string(),
    ///     "-100..=0, 5..=6, 8..=9, 11..=13, 16..=17, 30..=100"
    /// );
    /// ```
    fn symmetric_difference(self) -> SymDiffKMerge<T, I> {
        SymDiffIter::new_k(self)
    }
}
