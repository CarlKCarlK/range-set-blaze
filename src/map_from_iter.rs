use crate::union_iter_map::UnionIterMap;
use crate::{Integer, RangeMapBlaze, UniqueValue, ValueOwned};
use core::ops::RangeInclusive;

// We create a RangeMapBlaze from an iterator of integers or integer ranges by
// 1. turning them into a UnionIterMap (internally, it collects into intervals and sorts by start).
// 2. Turning the SortedDisjointMap into a BTreeMap.
impl<'a, T, V> FromIterator<(T, &'a V)> for RangeMapBlaze<T, V>
where
    T: Integer,
    V: ValueOwned + 'a,
{
    /// Create a [`RangeMapBlaze`] from an iterator of integers. Duplicates and out-of-order elements are fine.
    ///
    /// *For more about constructors and performance, see [`RangeMapBlaze` Constructors](struct.RangeMapBlaze.html#RangeMapBlaze-constructors).*
    ///
    /// # Examples
    ///
    /// ```
    /// use range_set_blaze::RangeMapBlaze;
    ///
    /// let a0 = RangeMapBlaze::from_iter([(3, &"a"), (2, &"a"), (1, &"a"), (100, &"b"), (1, &"c")]);
    /// let a1: RangeMapBlaze<i32, &str> = [(3, &"a"), (2, &"a"), (1, &"a"), (100, &"b"), (1, &"c")].into_iter().collect();
    /// assert!(a0 == a1 && a0.to_string() == r#"(1..=3, "a"), (100..=100, "b")"#);
    /// ```
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = (T, &'a V)>,
    {
        let iter = iter.into_iter().map(|(x, r)| (x..=x, r));
        Self::from_iter(iter)
    }
}

impl<'a, T, V> FromIterator<(RangeInclusive<T>, &'a V)> for RangeMapBlaze<T, V>
where
    T: Integer + 'a,    // cmk 'a needed? everywhere
    V: ValueOwned + 'a, // cmk 'a needed? everywhere
{
    /// Create a [`RangeMapBlaze`] from an iterator of inclusive ranges, `start..=end`.
    /// Overlapping, out-of-order, and empty ranges are fine.
    ///
    /// *For more about constructors and performance, see [`RangeMapBlaze` Constructors](struct.RangeMapBlaze.html#RangeMapBlaze-constructors).*
    ///
    /// # Examples
    ///
    /// ```
    /// use range_set_blaze::prelude::*;
    ///
    /// #[allow(clippy::reversed_empty_ranges)]
    /// let a0 = RangeMapBlaze::from_iter([(1..=2, &"a"), (2..=2, &"b"), (-10..=-5, &"c"), (1..=0, &"d")]);
    /// #[allow(clippy::reversed_empty_ranges)]
    /// let a1: RangeMapBlaze<i32, &str> = [(1..=2, &"a"), (2..=2, &"b"), (-10..=-5, &"c"), (1..=0, &"d")].into_iter().collect();
    /// assert!(a0 == a1 && a0.to_string() == r#"(-10..=-5, "c"), (1..=2, "a")"#);
    /// ```
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = (RangeInclusive<T>, &'a V)>,
    {
        let iter = iter.into_iter();
        let union_iter_map = UnionIterMap::<T, V, &V, _>::from_iter(iter);
        Self::from_sorted_disjoint_map(union_iter_map)
    }
}

impl<T: Integer, V: ValueOwned> FromIterator<(RangeInclusive<T>, V)> for RangeMapBlaze<T, V> {
    /// Create a [`RangeMapBlaze`] from an iterator of inclusive ranges, `start..=end`.
    /// Overlapping, out-of-order, and empty ranges are fine.
    ///
    /// *For more about constructors and performance, see [`RangeMapBlaze` Constructors](struct.RangeMapBlaze.html#RangeMapBlaze-constructors).*
    ///
    /// # Examples
    ///
    /// ```
    /// use range_set_blaze::RangeMapBlaze;
    ///
    /// #[allow(clippy::reversed_empty_ranges)]
    /// let a0 = RangeMapBlaze::from_iter([(1..=2, "a"), (2..=2, "b"), (-10..=-5, "c"), (1..=0, "d")]);
    /// #[allow(clippy::reversed_empty_ranges)]
    /// let a1: RangeMapBlaze<i32, &str> = [(1..=2, "a"), (2..=2, "b"), (-10..=-5, "c"), (1..=0, "d")].into_iter().collect();
    /// assert!(a0 == a1 && a0.to_string() == r#"(-10..=-5, "c"), (1..=2, "a")"#);
    /// ```
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = (RangeInclusive<T>, V)>,
    {
        let iter = iter
            .into_iter()
            .map(|(r, v)| (r.clone(), UniqueValue::new(v)));
        let union_iter_map = UnionIterMap::<T, V, UniqueValue<V>, _>::from_iter(iter);
        Self::from_sorted_disjoint_map(union_iter_map)
    }
}

impl<T: Integer, V: ValueOwned> FromIterator<(T, V)> for RangeMapBlaze<T, V> {
    /// Create a [`RangeMapBlaze`] from an iterator of inclusive ranges, `start..=end`.
    /// Overlapping, out-of-order, and empty ranges are fine.
    ///
    /// *For more about constructors and performance, see [`RangeMapBlaze` Constructors](struct.RangeMapBlaze.html#RangeMapBlaze-constructors).*
    ///
    /// # Examples
    ///
    /// ```
    /// use range_set_blaze::prelude::*;
    ///
    /// #[allow(clippy::reversed_empty_ranges)]
    /// let a0 = RangeMapBlaze::from_iter([(3, "a"), (2, "a"), (1, "a"), (100, "b"), (1, "c")]);
    /// let a1: RangeMapBlaze<i32, &str> = [(3, "a"), (2, "a"), (1, "a"), (100, "b"), (1, "c")].into_iter().collect();
    /// assert!(a0 == a1 && a0.to_string() == r#"(1..=3, "a"), (100..=100, "b")"#);
    /// ```
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = (T, V)>,
    {
        let iter = iter.into_iter().map(|(k, v)| (k..=k, v));
        Self::from_iter(iter)
    }
}

impl<'a, T, V> FromIterator<&'a (T, &'a V)> for RangeMapBlaze<T, V>
where
    T: Integer,
    V: ValueOwned + 'a,
{
    /// Create a [`RangeMapBlaze`] from an iterator of integers. Duplicates and out-of-order elements are fine.
    ///
    /// *For more about constructors and performance, see [`RangeMapBlaze` Constructors](struct.RangeMapBlaze.html#RangeMapBlaze-constructors).*
    ///
    /// # Examples
    ///
    /// ```
    /// use range_set_blaze::RangeMapBlaze;
    ///
    /// let v = vec![(3, &"a"), (2, &"a"), (1, &"a"), (100, &"b"), (1, &"c")];
    /// let a0 = RangeMapBlaze::from_iter(&v);
    /// let a1: RangeMapBlaze<i32, &str> = (&v).into_iter().collect();
    /// assert!(a0 == a1 && a0.to_string() == r#"(1..=3, "a"), (100..=100, "b")"#);
    /// ```
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = &'a (T, &'a V)>,
    {
        let iter = iter.into_iter().map(|&(x, r)| (x..=x, r));
        Self::from_iter(iter)
    }
}

impl<'a, T, V> FromIterator<&'a (RangeInclusive<T>, &'a V)> for RangeMapBlaze<T, V>
where
    T: Integer + 'a,    // cmk 'a needed? everywhere
    V: ValueOwned + 'a, // cmk 'a needed? everywhere
{
    /// Create a [`RangeMapBlaze`] from an iterator of inclusive ranges, `start..=end`.
    /// Overlapping, out-of-order, and empty ranges are fine.
    ///
    /// *For more about constructors and performance, see [`RangeMapBlaze` Constructors](struct.RangeMapBlaze.html#RangeMapBlaze-constructors).*
    ///
    /// # Examples
    ///
    /// ```
    /// use range_set_blaze::prelude::*;
    ///
    /// let v = vec![(1..=2, &"a"), (2..=2, &"b"), (-10..=-5, &"c"), (1..=0, &"d")];
    /// let a0: RangeMapBlaze<i32, &str> = RangeMapBlaze::from_iter(&v);
    /// let a1: RangeMapBlaze<i32, &str> = (&v).into_iter().collect();
    /// assert!(a0 == a1 && a0.to_string() == r#"(-10..=-5, "c"), (1..=2, "a")"#);
    /// ```
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = &'a (RangeInclusive<T>, &'a V)>,
    {
        let iter = iter.into_iter();
        let iter = iter.map(|(r, v)| (r.clone(), *v));
        Self::from_iter(iter)
    }
}

impl<'a, T: Integer, V: ValueOwned> FromIterator<&'a (RangeInclusive<T>, V)>
    for RangeMapBlaze<T, V>
{
    /// Create a [`RangeMapBlaze`] from an iterator of inclusive ranges, `start..=end`.
    /// Overlapping, out-of-order, and empty ranges are fine.
    ///
    /// *For more about constructors and performance, see [`RangeMapBlaze` Constructors](struct.RangeMapBlaze.html#RangeMapBlaze-constructors).*
    ///
    /// # Examples
    ///
    /// ```
    /// use range_set_blaze::prelude::*;
    ///
    /// #[allow(clippy::reversed_empty_ranges)]
    /// let vec_range = vec![(1..=2, "a"), (2..=2, "b"), (-10..=-5, "c"), (1..=0, "d")];
    /// let a0 = RangeMapBlaze::from_iter(vec_range.iter());
    /// let a1: RangeMapBlaze<i32, &str> = vec_range.iter().collect();
    /// assert!(a0 == a1 && a0.to_string() == r#"(-10..=-5, "c"), (1..=2, "a")"#);
    /// ```
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = &'a (RangeInclusive<T>, V)>,
    {
        let iter = iter.into_iter();
        let iter = iter.map(|(r, v)| (r.clone(), v));
        Self::from_iter(iter)
    }
}

impl<'a, T: Integer, V: ValueOwned> FromIterator<&'a (T, V)> for RangeMapBlaze<T, V> {
    /// Create a [`RangeMapBlaze`] from an iterator of inclusive ranges, `start..=end`.
    /// Overlapping, out-of-order, and empty ranges are fine.
    ///
    /// *For more about constructors and performance, see [`RangeMapBlaze` Constructors](struct.RangeMapBlaze.html#RangeMapBlaze-constructors).*
    ///
    /// # Examples
    ///
    /// ```
    /// use range_set_blaze::prelude::*;
    ///
    /// let v = vec![(1, "a"), (2, "a"), (2, "b")];
    /// let a0 = RangeMapBlaze::from_iter(&v);
    /// let a1: RangeMapBlaze<i32, &str> = (&v).iter().collect();
    /// assert!(a0 == a1 && a0.to_string() == r#"(1..=2, "a")"#);
    /// ```
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = &'a (T, V)>,
    {
        let iter = iter.into_iter().map(|(k, v)| {
            let k = *k;
            (k..=k, v)
        });
        Self::from_iter(iter)
    }
}

#[test]
fn test_cmk_delete_me3() {
    use crate::prelude::*;

    #[allow(clippy::reversed_empty_ranges)]
    let arr = [(1..=2, "a"), (2..=2, "b"), (-10..=-5, "c"), (1..=0, "d")];
    let a0 = RangeMapBlaze::from_iter(&arr);
    let a1: RangeMapBlaze<i32, &str> = arr.iter().collect();
    assert!(a0 == a1 && a0.to_string() == r#"(-10..=-5, "c"), (1..=2, "a")"#);
}
