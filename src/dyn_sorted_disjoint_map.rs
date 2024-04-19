use core::{iter::FusedIterator, ops::RangeInclusive};

use crate::{
    map::{CloneBorrow, ValueOwned},
    Integer, SortedDisjointMap,
};
use alloc::boxed::Box;

#[must_use = "iterators are lazy and do nothing unless consumed"]
/// Gives [`SortedDisjointMap`] iterators a uniform type. Used by the [`union_map_dyn`] and [`intersection_map_dyn`] macros to give all
/// their input iterators the same type.
///
/// [`union_map_dyn`]: crate::union_map_dyn
/// [`intersection_map_dyn`]: crate::intersection_map_dyn
///
/// # Example
/// ```
/// use range_set_blaze::prelude::*;
///
/// let a = RangeSetBlaze::from_iter([1u8..=6, 8..=9, 11..=15]);
/// let b = RangeSetBlaze::from_iter([5..=13, 18..=29]);
/// let c = RangeSetBlaze::from_iter([38..=42]);
/// let union = [
///     DynSortedDisjointMap::new(a.ranges()),
///     DynSortedDisjointMap::new(!b.ranges()),
///     DynSortedDisjointMap::new(c.ranges()),
/// ]
/// .union();
/// assert_eq!(union.to_string(), "0..=6, 8..=9, 11..=17, 30..=255");
/// ```

pub struct DynSortedDisjointMap<'a, T, V, VR>
where
    T: Integer,
    V: ValueOwned,
    VR: CloneBorrow<V>,
{
    iter: Box<dyn SortedDisjointMap<T, V, VR> + 'a>,
}

// Constructs a `DynSortedDisjointMap` encapsulating a `SortedDisjointMap` iterator.
// The lifetime `'a` ensures that any references held by the iterator are valid
// for the duration of the `DynSortedDisjointMap`'s existence. This is crucial for
// preventing dangling references and ensuring memory safety when the iterator
// contains references to data outside of itself.
impl<'a, T, V, VR> DynSortedDisjointMap<'a, T, V, VR>
where
    T: Integer,
    V: ValueOwned,
    VR: CloneBorrow<V>,
{
    /// Create a [`DynSortedDisjointMap`] from any [`SortedDisjointMap`] iterator. See [`DynSortedDisjointMap`] for an example.
    pub fn new<I>(iter: I) -> Self
    where
        I: SortedDisjointMap<T, V, VR> + 'a,
    {
        Self {
            iter: Box::new(iter),
        }
    }
}

impl<T, V, VR> FusedIterator for DynSortedDisjointMap<'_, T, V, VR>
where
    T: Integer,
    V: ValueOwned,
    VR: CloneBorrow<V>,
{
}

impl<T, V, VR> Iterator for DynSortedDisjointMap<'_, T, V, VR>
where
    T: Integer,
    V: ValueOwned,
    VR: CloneBorrow<V>,
{
    type Item = (RangeInclusive<T>, VR);

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

/// Intersects one or more [`SortedDisjointMap`] iterators, creating a new [`SortedDisjointMap`] iterator.
/// The input iterators need not to be of the same type.
///
/// For input iterators of the same type, [`intersection`] may be slightly faster.
///
/// # Performance
///   All work is done on demand, in one pass through the input iterators. Minimal memory is used.
///
/// # Example: 3-Input Parity
///
/// Find the integers that appear an odd number of times in the [`SortedDisjointMap`] iterators.
///
/// [`intersection`]: crate::MultiwaySortedDisjointMap::intersection
/// ```
/// use range_set_blaze::prelude::*;
///
/// let a = RangeSetBlaze::from_iter([1..=6, 8..=9, 11..=15]);
/// let b = RangeSetBlaze::from_iter([5..=13, 18..=29]);
/// let c = RangeSetBlaze::from_iter([38..=42]);
///
/// let parity = union_map_dyn!(
///     intersection_map_dyn!(a.ranges(), !b.ranges(), !c.ranges()),
///     intersection_map_dyn!(!a.ranges(), b.ranges(), !c.ranges()),
///     intersection_map_dyn!(!a.ranges(), !b.ranges(), c.ranges()),
///     intersection_map_dyn!(a.ranges(), b.ranges(), c.ranges())
/// );
/// assert_eq!(
///     parity.to_string(),
///     "1..=4, 7..=7, 10..=10, 14..=15, 18..=29, 38..=42"
/// );
/// ```
#[macro_export]
macro_rules! intersection_map_dyn {
    ($($val:expr),*) => {$crate::MultiwaySortedDisjointMap::intersection([$($crate::DynSortedDisjointMap::new($val)),*])}
}

/// Unions one or more [`SortedDisjointMap`] iterators, creating a new [`SortedDisjointMap`] iterator.
/// The input iterators need not to be of the same type.
///
/// For input iterators of the same type, [`union`] may be slightly faster.
///
/// # Performance
///   All work is done on demand, in one pass through the input iterators. Minimal memory is used.
///
/// # Example: 3-Input Parity
///
/// Find the integers that appear an odd number of times in the [`SortedDisjointMap`] iterators.
///
/// [`union`]: crate::MultiwaySortedDisjointMap::union
/// ```
/// use range_set_blaze::prelude::*;
///
/// let a = RangeSetBlaze::from_iter([1..=6, 8..=9, 11..=15]);
/// let b = RangeSetBlaze::from_iter([5..=13, 18..=29]);
/// let c = RangeSetBlaze::from_iter([38..=42]);
///
/// let parity = union_map_dyn!(
///     intersection_map_dyn!(a.ranges(), !b.ranges(), !c.ranges()),
///     intersection_map_dyn!(!a.ranges(), b.ranges(), !c.ranges()),
///     intersection_map_dyn!(!a.ranges(), !b.ranges(), c.ranges()),
///     intersection_map_dyn!(a.ranges(), b.ranges(), c.ranges())
/// );
/// assert_eq!(
///     parity.to_string(),
///     "1..=4, 7..=7, 10..=10, 14..=15, 18..=29, 38..=42"
/// );
/// ```
#[macro_export]
macro_rules! union_map_dyn {
    ($($val:expr),*) => {
                        $crate::MultiwaySortedDisjointMap::union([$($crate::DynSortedDisjointMap::new($val)),*])
                        }
}
