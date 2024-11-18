use core::{iter::FusedIterator, ops::RangeInclusive};

use crate::{map::ValueRef, Integer, SortedDisjointMap};
use alloc::boxed::Box;

/// Gives [`SortedDisjointMap`] iterators a uniform type. Used by the [`union_map_dyn`], etc. macros to give all
/// their input iterators the same type.
///
/// [`SortedDisjointMap`]: trait.SortedDisjointMap.html#table-of-contents
/// [`union_map_dyn`]: crate::union_map_dyn
/// [`intersection_map_dyn`]: crate::intersection_map_dyn
///
/// # Example
/// ```
/// use range_set_blaze::prelude::*;
///
/// let a = RangeMapBlaze::from_iter([(1..=6, "a"), (8..=9, "a"), (11..=15, "a")]);
/// let b = CheckSortedDisjointMap::new([(5..=13, &"b"), (18..=29, &"b")]);
/// let c = RangeMapBlaze::from_iter([(38..=42, "c")]);
/// let union = [
///     DynSortedDisjointMap::new(a.range_values()),
///     DynSortedDisjointMap::new(b),
///     DynSortedDisjointMap::new(c.range_values()),
/// ].union();
/// assert_eq!(union.into_string(), r#"(1..=6, "a"), (7..=7, "b"), (8..=9, "a"), (10..=10, "b"), (11..=15, "a"), (18..=29, "b"), (38..=42, "c")"#);
/// ```
#[must_use = "iterators are lazy and do nothing unless consumed"]
pub struct DynSortedDisjointMap<'a, T, VR>
where
    T: Integer,
    VR: ValueRef,
{
    iter: Box<dyn SortedDisjointMap<T, VR> + 'a>,
}

// Constructs a `DynSortedDisjointMap` encapsulating a `SortedDisjointMap` iterator.
// The lifetime `'a` ensures that any references held by the iterator are valid
// for the duration of the `DynSortedDisjointMap`'s existence. This is crucial for
// preventing dangling references and ensuring memory safety when the iterator
// contains references to data outside of itself.
impl<'a, T, VR> DynSortedDisjointMap<'a, T, VR>
where
    T: Integer,
    VR: ValueRef,
{
    /// Create a [`DynSortedDisjointMap`] from any [`SortedDisjointMap`] iterator. See [`DynSortedDisjointMap`] for an example.
    ///
    /// [`SortedDisjointMap`]: trait.SortedDisjointMap.html#table-of-contents
    #[inline]
    pub fn new<I>(iter: I) -> Self
    where
        I: SortedDisjointMap<T, VR> + 'a,
    {
        Self {
            iter: Box::new(iter),
        }
    }
}

impl<T, VR> FusedIterator for DynSortedDisjointMap<'_, T, VR>
where
    T: Integer,
    VR: ValueRef,
{
}

impl<T, VR> Iterator for DynSortedDisjointMap<'_, T, VR>
where
    T: Integer,
    VR: ValueRef,
{
    type Item = (RangeInclusive<T>, VR);

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

/// Intersects zero or more [`SortedDisjointMap`] iterators,
/// creating a new [`SortedDisjointMap`] iterator.
/// The input iterators need not to be of the same type.
///
/// For input iterators of the same type, [`intersection`] may be slightly faster.
///
/// # Performance
///   All work is done on demand, in one pass through the input iterators. Minimal memory is used.
///
/// [`SortedDisjointMap`]: trait.SortedDisjointMap.html#table-of-contents
/// [`intersection`]: crate::MultiwaySortedDisjointMap::intersection
/// # Examples
///
/// ```
/// use range_set_blaze::prelude::*;
///
/// let a = RangeMapBlaze::from_iter([(1..=2, "a"), (5..=100, "a")]);
/// let b = CheckSortedDisjointMap::new([(2..=6, &"b")]);
/// let c = RangeMapBlaze::from_iter([(2..=2, "c"), (6..=200, "c")]);
/// let intersection = intersection_map_dyn!(a.range_values(), b, c.range_values());
/// assert_eq!(intersection.into_string(), r#"(2..=2, "a"), (6..=6, "a")"#);
/// ```
#[macro_export]
macro_rules! intersection_map_dyn {
    ($($val:expr),*) => {$crate::MultiwaySortedDisjointMap::intersection([$($crate::DynSortedDisjointMap::new($val)),*])}
}

/// Unions zero or more [`SortedDisjointMap`] iterators,
/// creating a new [`SortedDisjointMap`] iterator.
/// The input iterators need not to be of the same type.
///
/// For input iterators of the same type, [`union`] may be slightly faster.
///
/// # Performance
///   All work is done on demand, in one pass through the input iterators. Minimal memory is used.
///
/// [`SortedDisjointMap`]: trait.SortedDisjointMap.html#table-of-contents
/// [`union`]: crate::MultiwaySortedDisjointMap::union
/// # Examples
///
/// ```
/// use range_set_blaze::prelude::*;
///
/// let a = RangeMapBlaze::from_iter([(1..=2, "a"), (5..=100, "a")]);
/// let b = CheckSortedDisjointMap::new([(2..=6, &"b")]);
/// let c = RangeMapBlaze::from_iter([(2..=2, "c"), (6..=200, "c")]);
/// let union = union_map_dyn!(a.range_values(), b, c.range_values());
/// assert_eq!(union.into_string(), r#"(1..=2, "a"), (3..=4, "b"), (5..=100, "a"), (101..=200, "c")"#);
/// ```
#[macro_export]
macro_rules! union_map_dyn {
    ($($val:expr),*) => {
                        $crate::MultiwaySortedDisjointMap::union([$($crate::DynSortedDisjointMap::new($val)),*])
                        }
}

/// Find the symmetric difference of zero or more [`SortedDisjointMap`] iterators,
/// creating a new [`SortedDisjointMap`] iterator.
/// The input iterators need not to be of the same type.
///
/// For input iterators of the same type, [`symmetric_difference`] may be slightly faster.
///
/// # Performance
///   All work is done on demand, in one pass through the input iterators. Minimal memory is used.
///
/// [`SortedDisjointMap`]: trait.SortedDisjointMap.html#table-of-contents
/// [`symmetric_difference`]: crate::MultiwaySortedDisjointMap::symmetric_difference
/// # Examples
///
/// ```
/// use range_set_blaze::prelude::*;
///
/// let a = RangeMapBlaze::from_iter([(1..=2, "a"), (5..=100, "a")]);
/// let b = CheckSortedDisjointMap::new([(2..=6, &"b")]);
/// let c = RangeMapBlaze::from_iter([(2..=2, "c"), (6..=200, "c")]);
/// let sym_diff = symmetric_difference_map_dyn!(a.range_values(), b, c.range_values());
/// assert_eq!(sym_diff.into_string(), r#"(1..=2, "a"), (3..=4, "b"), (6..=6, "a"), (101..=200, "c")"#);
/// ```
#[macro_export]
macro_rules! symmetric_difference_map_dyn {
    ($($val:expr),*) => {
                        $crate::MultiwaySortedDisjointMap::symmetric_difference([$($crate::DynSortedDisjointMap::new($val)),*])
                        }
}
