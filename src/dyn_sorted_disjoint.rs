use core::{iter::FusedIterator, ops::RangeInclusive};

use crate::{Integer, SortedDisjoint};
use alloc::boxed::Box;

/// Gives [`SortedDisjoint`] iterators a uniform type. Used by the [`union_dyn`], etc. macros to give all
/// their input iterators the same type.
///
/// [`SortedDisjoint`]: crate::SortedDisjoint.html#table-of-contents
/// [`union_dyn`]: crate::union_dyn
/// [`intersection_dyn`]: crate::intersection_dyn
///
/// # Example
/// ```
/// use range_set_blaze::prelude::*;
///
/// let a = RangeSetBlaze::from_iter([1..=6, 8..=9, 11..=15]);
/// let b = CheckSortedDisjoint::new([5..=13, 18..=29]);
/// let c = RangeSetBlaze::from_iter([38..=42]);
/// let union = [
///     DynSortedDisjoint::new(a.ranges()),
///     DynSortedDisjoint::new(b),
///     DynSortedDisjoint::new(c.ranges()),
/// ]
/// .union();
/// assert_eq!(union.into_string(), "1..=15, 18..=29, 38..=42");
/// ```
#[must_use = "iterators are lazy and do nothing unless consumed"]
pub struct DynSortedDisjoint<'a, T: Integer> {
    iter: Box<dyn SortedDisjoint<T> + 'a>,
}

impl<'a, T: Integer> DynSortedDisjoint<'a, T> {
    /// Create a [`DynSortedDisjoint`] from any [`SortedDisjoint`] iterator. See [`DynSortedDisjoint`] for an example.
    ///
    /// [`SortedDisjoint`]: crate::SortedDisjoint.html#table-of-contents
    #[inline]
    pub fn new<I>(iter: I) -> Self
    where
        I: SortedDisjoint<T> + 'a,
    {
        Self {
            iter: Box::new(iter),
        }
    }
}

impl<T: Integer> FusedIterator for DynSortedDisjoint<'_, T> {}

impl<T: Integer> Iterator for DynSortedDisjoint<'_, T> {
    type Item = RangeInclusive<T>;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

/// Intersects zero or more [`SortedDisjoint`] iterators, creating a new [`SortedDisjoint`] iterator.
/// The input iterators need not be of the same type.
///
/// For input iterators of the same type, [`intersection`] may be slightly faster.
///
/// # Performance
///   All work is done on demand, in one pass through the input iterators. Minimal memory is used.
///
/// [`SortedDisjoint`]: crate::SortedDisjoint.html#table-of-contents
/// [`intersection`]: crate::MultiwaySortedDisjoint::intersection
/// # Examples
/// ```
/// use range_set_blaze::prelude::*;
///
/// let a = RangeSetBlaze::from_iter([1u8..=6, 8..=9, 11..=15]);
/// let b = CheckSortedDisjoint::new([5..=13, 18..=29]);
/// let c = RangeSetBlaze::from_iter([38..=42]);
/// let not_c = !c.ranges();
///
/// let intersection = intersection_dyn!(a.ranges(), b, not_c);
/// assert_eq!(intersection.into_string(), "5..=6, 8..=9, 11..=13");
/// ```
#[macro_export]
macro_rules! intersection_dyn {
    ($($val:expr),*) => {$crate::MultiwaySortedDisjoint::intersection([$($crate::DynSortedDisjoint::new($val)),*])}
}

/// Unions zero or more [`SortedDisjoint`] iterators, creating a new [`SortedDisjoint`] iterator.
/// The input iterators need not be of the same type.
///
/// For input iterators of the same type, [`union`] may be slightly faster.
///
/// # Performance
///   All work is done on demand, in one pass through the input iterators. Minimal memory is used.
///
/// [`SortedDisjoint`]: crate::SortedDisjoint.html#table-of-contents
/// [`union`]: crate::MultiwaySortedDisjoint::union
/// # Examples
/// ```
/// use range_set_blaze::prelude::*;
///
/// let a = RangeSetBlaze::from_iter([1..=6, 8..=9, 11..=15]);
/// let b = CheckSortedDisjoint::new([5..=13, 18..=29]);
/// let c = RangeSetBlaze::from_iter([38..=42]);
/// let union = union_dyn!(a.ranges(), b, c.ranges());
/// assert_eq!(union.into_string(), "1..=15, 18..=29, 38..=42");
/// ```
#[macro_export]
macro_rules! union_dyn {
    ($($val:expr),*) => {
                        $crate::MultiwaySortedDisjoint::union([$($crate::DynSortedDisjoint::new($val)),*])
                        }
}

/// Computes the symmetric difference of zero or more [`SortedDisjoint`] iterators, creating a new [`SortedDisjoint`] iterator.
/// The input iterators need not be of the same type.
///
/// For input iterators of the same type, [`symmetric_difference`] may be slightly faster.
///
/// # Performance
///   All work is done on demand, in one pass through the input iterators. Minimal memory is used.
///
/// [`SortedDisjoint`]: crate::SortedDisjoint.html#table-of-contents
/// [`symmetric_difference`]: crate::MultiwaySortedDisjoint::symmetric_difference
/// # Examples
/// ```
/// use range_set_blaze::prelude::*;
///
/// let a = RangeSetBlaze::from_iter([1..=6, 8..=9, 11..=15]);
/// let b = CheckSortedDisjoint::new([5..=13, 18..=29]);
/// let c = RangeSetBlaze::from_iter([38..=42]);
/// let sym_diff = symmetric_difference_dyn!(a.ranges(), b, c.ranges());
/// assert_eq!(sym_diff.into_string(), "1..=4, 7..=7, 10..=10, 14..=15, 18..=29, 38..=42");
/// ```
#[macro_export]
macro_rules! symmetric_difference_dyn {
    ($($val:expr),*) => {
                        $crate::MultiwaySortedDisjoint::symmetric_difference([$($crate::DynSortedDisjoint::new($val)),*])
                        }
}
