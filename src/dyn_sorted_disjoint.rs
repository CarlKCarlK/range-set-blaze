use core::ops::RangeInclusive;

use alloc::boxed::Box;

use crate::{Integer, SortedDisjoint, SortedStarts};

#[must_use = "iterators are lazy and do nothing unless consumed"]
/// Gives [`SortedDisjoint`] iterators a uniform type. Used by the [`union_dyn`] and [`intersection_dyn`] macros to give all
/// their input iterators the same type.
///
/// [`union_dyn`]: crate::union_dyn
/// [`intersection_dyn`]: crate::intersection_dyn
///
/// # Example
/// ```
/// use range_set_blaze::prelude::*;
///
/// let a = RangeSetBlaze::from_iter([1u8..=6, 8..=9, 11..=15]);
/// let b = RangeSetBlaze::from_iter([5..=13, 18..=29]);
/// let c = RangeSetBlaze::from_iter([38..=42]);
/// let union = [
///     DynSortedDisjoint::new(a.ranges()),
///     DynSortedDisjoint::new(!b.ranges()),
///     DynSortedDisjoint::new(c.ranges()),
/// ]
/// .union();
/// assert_eq!(union.to_string(), "0..=6, 8..=9, 11..=17, 30..=255");
/// ```

pub struct DynSortedDisjoint<'a, T: Integer> {
    iter: Box<dyn SortedDisjoint<T> + 'a>,
}

impl<'a, T: Integer> DynSortedDisjoint<'a, T> {
    /// Create a [`DynSortedDisjoint`] from any [`SortedDisjoint`] iterator. See [`DynSortedDisjoint`] for an example.
    pub fn new<I>(iter: I) -> Self
    where
        I: SortedDisjoint<T> + 'a,
    {
        Self {
            iter: Box::new(iter),
        }
    }
}

// All DynSortedDisjoint's are SortedDisjoint's
impl<'a, T: Integer> SortedStarts<T> for DynSortedDisjoint<'a, T> {}
impl<'a, T: Integer> SortedDisjoint<T> for DynSortedDisjoint<'a, T> {}

impl<'a, T: Integer> Iterator for DynSortedDisjoint<'a, T> {
    type Item = RangeInclusive<T>;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

/// Intersects one or more [`SortedDisjoint`] iterators, creating a new [`SortedDisjoint`] iterator.
/// The input iterators need not to be of the same type.
///
/// For input iterators of the same type, [`intersection`] may be slightly faster.
///
/// # Performance
///   All work is done on demand, in one pass through the input iterators. Minimal memory is used.
///
/// # Example: 3-Input Parity
///
/// Find the integers that appear an odd number of times in the [`SortedDisjoint`] iterators.
///
/// [`intersection`]: crate::MultiwaySortedDisjoint::intersection
/// ```
/// use range_set_blaze::prelude::*;
///
/// let a = RangeSetBlaze::from_iter([1..=6, 8..=9, 11..=15]);
/// let b = RangeSetBlaze::from_iter([5..=13, 18..=29]);
/// let c = RangeSetBlaze::from_iter([38..=42]);
///
/// let parity = union_dyn!(
///     intersection_dyn!(a.ranges(), !b.ranges(), !c.ranges()),
///     intersection_dyn!(!a.ranges(), b.ranges(), !c.ranges()),
///     intersection_dyn!(!a.ranges(), !b.ranges(), c.ranges()),
///     intersection_dyn!(a.ranges(), b.ranges(), c.ranges())
/// );
/// assert_eq!(
///     parity.to_string(),
///     "1..=4, 7..=7, 10..=10, 14..=15, 18..=29, 38..=42"
/// );
/// ```
#[macro_export]
macro_rules! intersection_dyn {
    ($($val:expr),*) => {$crate::MultiwaySortedDisjoint::intersection([$($crate::DynSortedDisjoint::new($val)),*])}
}

/// Unions one or more [`SortedDisjoint`] iterators, creating a new [`SortedDisjoint`] iterator.
/// The input iterators need not to be of the same type.
///
/// For input iterators of the same type, [`union`] may be slightly faster.
///
/// # Performance
///   All work is done on demand, in one pass through the input iterators. Minimal memory is used.
///
/// # Example: 3-Input Parity
///
/// Find the integers that appear an odd number of times in the [`SortedDisjoint`] iterators.
///
/// [`union`]: crate::MultiwaySortedDisjoint::union
/// ```
/// use range_set_blaze::prelude::*;
///
/// let a = RangeSetBlaze::from_iter([1..=6, 8..=9, 11..=15]);
/// let b = RangeSetBlaze::from_iter([5..=13, 18..=29]);
/// let c = RangeSetBlaze::from_iter([38..=42]);
///
/// let parity = union_dyn!(
///     intersection_dyn!(a.ranges(), !b.ranges(), !c.ranges()),
///     intersection_dyn!(!a.ranges(), b.ranges(), !c.ranges()),
///     intersection_dyn!(!a.ranges(), !b.ranges(), c.ranges()),
///     intersection_dyn!(a.ranges(), b.ranges(), c.ranges())
/// );
/// assert_eq!(
///     parity.to_string(),
///     "1..=4, 7..=7, 10..=10, 14..=15, 18..=29, 38..=42"
/// );
/// ```
#[macro_export]
macro_rules! union_dyn {
    ($($val:expr),*) => {
                        $crate::MultiwaySortedDisjoint::union([$($crate::DynSortedDisjoint::new($val)),*])
                        }
}
