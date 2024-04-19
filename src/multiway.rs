impl<T, I> MultiwayRangeSetBlazeRef<T> for I
where
    T: Integer,
    I: IntoIterator<Item = RangeSetBlaze<T>>,
{
}

/// The trait used to provide methods on multiple [`RangeSetBlaze`] references,
/// specifically [`union`] and [`intersection`].
///
/// Also see [`MultiwayRangeSetBlaze`].
///
/// [`union`]: MultiwayRangeSetBlazeRef::union
/// [`intersection`]: MultiwayRangeSetBlazeRef::intersection
pub trait MultiwayRangeSetBlazeRef<T: Integer>:
    IntoIterator<Item = RangeSetBlaze<T>> + Sized
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
    /// let union = vec![a, b, c].into_iter().union();
    ///
    /// assert_eq!(union, RangeSetBlaze::from_iter([1..=15, 18..=100]));
    /// ```
    fn union(self) -> RangeSetBlaze<T> {
        RangeSetBlaze::from_sorted_disjoint(self.into_iter().map(|x| x.into_ranges()).union())
    }

    /// Intersects the given [`RangeSetBlaze`] references, creating a new [`RangeSetBlaze`].
    /// Any number of input can be given.
    ///
    /// For exactly two inputs, you can also use the '&' operator.
    /// Also see [`MultiwayRangeSetBlaze::intersection`].
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
    /// let intersection = vec![a, b, c].into_iter().intersection();
    ///
    /// assert_eq!(intersection, RangeSetBlaze::from_iter([5..=6, 8..=9, 11..=13]));
    /// ```
    fn intersection(self) -> RangeSetBlaze<T> {
        self.into_iter()
            .map(RangeSetBlaze::into_ranges)
            .intersection()
            .into_range_set_blaze()
    }
}
impl<'a, T, I> MultiwayRangeSetBlaze<'a, T> for I
where
    T: Integer + 'a,
    I: IntoIterator<Item = &'a RangeSetBlaze<T>>,
{
}
/// The trait used to provide methods on multiple [`RangeSetBlaze`]'s,
/// specifically [`union`] and [`intersection`].
///
/// Also see [`MultiwayRangeSetBlazeRef`].
///
/// [`union`]: MultiwayRangeSetBlaze::union
/// [`intersection`]: MultiwayRangeSetBlaze::intersection
pub trait MultiwayRangeSetBlaze<'a, T: Integer + 'a>:
    IntoIterator<Item = &'a RangeSetBlaze<T>> + Sized
{
    /// Unions the given [`RangeSetBlaze`]'s, creating a new [`RangeSetBlaze`].
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
            .map(RangeSetBlaze::ranges)
            .union()
            .into_range_set_blaze()
    }

    /// Intersects the given [`RangeSetBlaze`]'s, creating a new [`RangeSetBlaze`].
    /// Any number of input can be given.
    ///
    /// For exactly two inputs, you can also use the '&' operator.
    /// Also see [`MultiwayRangeSetBlazeRef::intersection`].
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
            .map(RangeSetBlaze::ranges)
            .intersection()
            .into_range_set_blaze()
    }
}

use crate::{BitAndKMerge, BitOrKMerge, Integer, RangeSetBlaze, SortedDisjoint, UnionIter};

impl<T, II, I> MultiwaySortedDisjoint<T, I> for II
where
    T: Integer,
    I: SortedDisjoint<T>,
    II: IntoIterator<Item = I>,
{
}

/// The trait used to define methods on multiple [`SortedDisjoint`] iterators,
/// specifically [`union`] and [`intersection`].
///
/// [`union`]: crate::MultiwaySortedDisjoint::union
/// [`intersection`]: crate::MultiwaySortedDisjoint::intersection
pub trait MultiwaySortedDisjoint<T: Integer, I>: IntoIterator<Item = I> + Sized
where
    I: SortedDisjoint<T>,
{
    /// Unions the given [`SortedDisjoint`] iterators, creating a new [`SortedDisjoint`] iterator.
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
    /// assert_eq!(union.to_string(), "1..=15, 18..=100");
    /// ```
    fn union(self) -> BitOrKMerge<T, I> {
        UnionIter::new_k(self)
    }

    /// Intersects the given [`SortedDisjoint`] iterators, creating a new [`SortedDisjoint`] iterator.
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
    /// assert_eq!(intersection.to_string(), "5..=6, 8..=9, 11..=13");
    /// ```
    fn intersection(self) -> BitAndKMerge<T, I> {
        // We define set intersection in terms of complement and (set/map) union.
        // Elsewhere, map intersection is defined -- in part -- in terms of set intersection.
        self.into_iter()
            .map(|seq| seq.into_iter().complement())
            .union()
            .complement()
    }

    // cmk000 add sym diff and add to tests

    // cmk0 can we now implement xor on any number of iterators?
}
