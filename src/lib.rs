#![cfg_attr(feature = "from_slice", feature(portable_simd))]
#![doc = include_str!("../README.md")]
#![warn(missing_docs)]
#![cfg_attr(not(feature = "std"), no_std)]
#![warn(clippy::use_self)]

// cmk #![feature(step_trait)] // cmk use unstable feature???
// cmk #![feature(const_option)]

// Developer notes:
//
// To run tests with different settings, environment variables are recommended.
// For example, the Windows steps to run one of the SIMD-related benchmark is:
// ```bash
// rustup override set nightly # use nightly compiler
// set RUSTFLAGS=-C target-cpu=native # use current CPUs full instruction set
// set BUILDFEATURES=from_slice # enable the from_slice feature via build.rs
// cargo bench ingest_clumps_iter_v_slice
// ```

// #[cfg(all(feature = "std", feature = "alloc"))]
// compile_error!("feature \"std\" and feature \"alloc\" cannot be enabled at the same time");
// #[cfg(feature = "std")]
// compile_error!("The 'std' feature is active");
// #[cfg(feature = "alloc")]
// compile_error!("The 'alloc' feature is active");
extern crate alloc;

// FUTURE: Support serde via optional feature
mod dyn_sorted_disjoint;
mod dyn_sorted_disjoint_map;
mod from_slice;
mod integer;
mod intersection_iter_map;
/// cmk doc
mod iter_map;
mod map;
/// cmk doc
pub mod range_set_blaze;
mod ranges;
pub use crate::range_set_blaze::RangeSetBlaze;
pub use crate::range_values::IntoRangeValuesIter;
pub use crate::ranges::IntoRangesIter;
mod not_iter;
pub mod prelude;
pub use crate::map::UniqueValue;
pub mod range_values;
#[cfg(feature = "rog-experimental")]
mod rog;
mod sorted_disjoint;
// use alloc::collections::btree_map;
// use gen_ops::gen_ops_ex;
pub use crate::multiway::MultiwayRangeSetBlaze;
pub use intersection_iter_map::IntersectionIterMap;
mod sym_diff_iter;
mod sym_diff_iter_map;
pub use map::CloneBorrow;
pub use map::ValueOwned;
use merge::KMerge;
use merge_map::KMergeMap;
pub use multiway::MultiwaySortedDisjoint;
pub use multiway_map::MultiwayRangeMapBlaze;
pub use multiway_map::MultiwaySortedDisjointMap;
use range_values::RangeValuesToRangesIter;
pub use sym_diff_iter::SymDiffIter;
pub use sym_diff_iter_map::SymDiffIterMap;
mod multiway;
mod multiway_map;
mod sorted_disjoint_map;
mod tests;
mod tests_map;
mod union_iter;
mod union_iter_map;
mod unsorted_disjoint;
mod unsorted_disjoint_map;
pub use crate::map::RangeMapBlaze;
pub use crate::sorted_disjoint_map::Priority;
pub use crate::unsorted_disjoint::AssumeSortedStarts;
pub use crate::unsorted_disjoint_map::AssumePrioritySortedStartsMap;
// use alloc::{collections::BTreeMap, vec::Vec};
use core::{
    // cmp::{max, Ordering},
    // convert::From,
    fmt,
    // iter::FusedIterator,
    ops::RangeInclusive,
    str::FromStr,
};
pub use dyn_sorted_disjoint::DynSortedDisjoint;
pub use dyn_sorted_disjoint_map::DynSortedDisjointMap;
// use itertools::Tee;
pub use merge::Merge;
pub use merge_map::MergeMap; // cmk KMergeMap
mod merge;
mod merge_map;
pub use not_iter::NotIter;
use num_traits::{ops::overflowing::OverflowingSub, CheckedAdd, WrappingSub};
#[cfg(feature = "rog-experimental")]
pub use rog::{Rog, RogsIter};
pub use sorted_disjoint::{CheckSortedDisjoint, SortedDisjoint, SortedStarts};
// cmk use sorted_disjoint_map::SortedDisjointMapWithLenSoFar;
pub use crate::sorted_disjoint_map::CheckSortedDisjointMap;
pub use sorted_disjoint_map::{SortedDisjointMap, SortedStartsMap};
// pub use union_iter::UnionIter;
pub use union_iter::UnionIter;
pub use union_iter_map::UnionIterMap;
// use unsorted_disjoint::SortedDisjointWithLenSoFar;
// use unsorted_disjoint::UnsortedDisjoint;
// cmk pub use unsorted_disjoint_map::UnsortedDisjointMap;
// cmk use unsorted_disjoint_map::UnsortedDisjointMap;

/// The element trait of the [`RangeSetBlaze`] and [`SortedDisjoint`], specifically `u8` to `u128` (including `usize`) and `i8` to `i128` (including `isize`).
pub trait Integer:
    num_integer::Integer
    + FromStr
    + Copy
    + fmt::Display
    + fmt::Debug
    + core::iter::Sum
    + num_traits::NumAssignOps
    + num_traits::Bounded
    + num_traits::NumCast
    + Send
    + Sync
    + OverflowingSub
    + CheckedAdd
    + WrappingSub
{
    #[cfg(feature = "from_slice")]
    /// A definition of [`RangeSetBlaze::from_slice()`] specific to this integer type.
    fn from_slice(slice: impl AsRef<[Self]>) -> RangeSetBlaze<Self>;

    /// The type of the length of a [`RangeSetBlaze`]. For example, the length of a `RangeSetBlaze<u8>` is `usize`. Note
    /// that it can't be `u8` because the length ranges from 0 to 256, which is one too large for `u8`.
    ///
    /// In general, `SafeLen` will be `usize` if `usize` is always large enough. If not, `SafeLen` will be the smallest unsigned integer
    /// type that is always large enough. However, for `u128` and `i128`, nothing is always large enough so
    ///  `SafeLen` will be `u128` and we prohibit the largest value from being used in [`Integer`].
    ///
    /// # Examples
    /// ```
    /// use range_set_blaze::{RangeSetBlaze, Integer};
    ///
    /// let len: <u8 as Integer>::SafeLen = RangeSetBlaze::from_iter([0u8..=255]).len();
    /// assert_eq!(len, 256);
    /// ```
    type SafeLen: core::hash::Hash
        + num_integer::Integer
        + num_traits::NumAssignOps
        + num_traits::Bounded
        + num_traits::NumCast
        + num_traits::One
        + core::ops::AddAssign
        + core::ops::SubAssign
        + Copy
        + PartialEq
        + Eq
        + PartialOrd
        + Ord
        + Send
        + Default
        + fmt::Debug
        + fmt::Display;

    /// Returns the length of a range without any overflow.
    ///
    /// # Example
    /// ```
    /// use range_set_blaze::Integer;
    ///
    /// assert_eq!(<u8 as Integer>::safe_len(&(0..=255)), 256);
    /// ```
    fn safe_len(range: &RangeInclusive<Self>) -> <Self as Integer>::SafeLen;

    /// For a given `Integer` type, returns the largest value that can be used. For all types other than `u128` and `i128`,
    /// this is the same as `Self::MAX`. For `u128` and `i128`, this is one less than `Self::MAX`.
    ///
    /// # Example
    /// ```
    /// use range_set_blaze::{Integer, RangeSetBlaze};
    ///
    /// // for i8, we can use up to 127
    /// let a = RangeSetBlaze::from_iter([i8::MAX]);
    /// // for i128, we can use up to 170141183460469231731687303715884105726
    /// let a = RangeSetBlaze::from_iter([<i128 as Integer>::safe_max_value()]);
    /// ```
    /// # Panics
    /// ```should_panic
    /// use range_set_blaze::{Integer, RangeSetBlaze};
    ///
    /// // for i128, using 170141183460469231731687303715884105727 throws a panic.
    /// let a = RangeSetBlaze::from_iter([i128::MAX]);
    /// ```
    fn safe_max_value() -> Self {
        Self::max_value()
    }

    // FUTURE define .len() SortedDisjoint

    /// Converts a `f64` to [`Integer::SafeLen`] using the formula `f as Self::SafeLen`. For large integer types, this will result in a loss of precision.
    fn f64_to_safe_len(f: f64) -> Self::SafeLen;

    /// Converts [`Integer::SafeLen`] to `f64` using the formula `len as f64`. For large integer types, this will result in a loss of precision.
    fn safe_len_to_f64(len: Self::SafeLen) -> f64;

    /// Computes `a + (b - 1) as Self`
    fn add_len_less_one(a: Self, b: Self::SafeLen) -> Self;

    /// Computes `a - (b - 1) as Self`
    fn sub_len_less_one(a: Self, b: Self::SafeLen) -> Self;
}

// #[doc(hidden)]
// pub type BitOrMerge<T, L, R> = UnionIter<T, Merge<T, L, R>>;

// cmk rename to Union...
#[doc(hidden)]
pub type UnionIterMapMerge<T, V, VR, L, R> = UnionIterMap<T, V, VR, MergeMap<T, V, VR, L, R>>;
#[doc(hidden)]
pub type UnionIterMerge<T, L, R> = UnionIter<T, Merge<T, L, R>>;

#[doc(hidden)]
pub type SymDiffIterMapMerge<T, V, VR, L, R> = SymDiffIterMap<T, V, VR, MergeMap<T, V, VR, L, R>>;
#[doc(hidden)]
pub type SymDiffIterMapKMerge<T, V, VR, II> = SymDiffIterMap<T, V, VR, KMergeMap<T, V, VR, II>>;

#[doc(hidden)]
pub type SymDiffIterMerge<T, L, R> = SymDiffIter<T, Merge<T, L, R>>;
#[doc(hidden)]
pub type SymDiffIterKMerge<T, II> = SymDiffIter<T, KMerge<T, II>>;

#[doc(hidden)]
pub type UnionIterMapKMerge<T, V, VR, I> = UnionIterMap<T, V, VR, KMergeMap<T, V, VR, I>>;
#[doc(hidden)]
pub type UnionIterKMerge<T, I> = UnionIter<T, KMerge<T, I>>;
#[doc(hidden)]
pub type BitAndMerge<T, L, R> = NotIter<T, BitNandMerge<T, L, R>>;
#[doc(hidden)]
pub type BitAndKMerge<T, I> = NotIter<T, BitNandKMerge<T, I>>;

// cmk000 'UnionIterMerge' used to be called 'BitOrMerge', put it back???
#[doc(hidden)]
pub type BitNandMerge<T, L, R> = UnionIterMerge<T, NotIter<T, L>, NotIter<T, R>>;
#[doc(hidden)]
pub type BitNandKMerge<T, I> = UnionIterKMerge<T, NotIter<T, I>>;
#[doc(hidden)]
pub type IntersectionMap<T, V, VR, I> =
    IntersectionIterMap<T, V, VR, I, BitAndKMerge<T, RangeValuesToRangesIter<T, V, VR, I>>>;
#[doc(hidden)]
pub type BitNorMerge<T, L, R> = NotIter<T, UnionIterMerge<T, L, R>>;
#[doc(hidden)]
pub type BitSubMerge<T, L, R> = NotIter<T, UnionIterMerge<T, NotIter<T, L>, R>>;
#[doc(hidden)]
// pub type BitXOrTee<T, L, R> =
//     BitOrMerge<T, BitSubMerge<T, Tee<L>, Tee<R>>, BitSubMerge<T, Tee<R>, Tee<L>>>;
// #[doc(hidden)]
// pub type BitXOr<T, L, R> = BitOrMerge<T, BitSubMerge<T, L, Tee<R>>, BitSubMerge<T, Tee<R>, L>>;
// #[doc(hidden)]
// pub type BitEq<T, L, R> = BitOrMerge<
//     T,
//     NotIter<T, BitOrMerge<T, NotIter<T, Tee<L>>, NotIter<T, Tee<R>>>>,
//     NotIter<T, BitOrMerge<T, Tee<L>, Tee<R>>>,
// >;

// // FUTURE: use fn range to implement one-at-a-time intersection, difference, etc. and then add more inplace ops.
// cmk00 Can we/should we hide MergeMapIter and KMergeMapIter and SymDiffMapIter::new and UnionMapIter::new?
#[test]
// cmk0000 challenge: convert from every level to sorted disjoint* for both map and set.
pub fn convert_challenge() {
    use itertools::Itertools;
    use unsorted_disjoint_map::UnsortedPriorityDisjointMap;

    fn is_sorted_disjoint_map<T, V, VR, S>(_iter: S)
    where
        T: Integer,
        V: ValueOwned,
        VR: CloneBorrow<V>,
        S: SortedDisjointMap<T, V, VR>,
    {
    }

    //===========================
    // Map - ranges
    //===========================

    // * from sorted_disjoint
    let a = CheckSortedDisjointMap::new([(1..=2, &"a"), (5..=100, &"a")]);
    assert!(a.equal(CheckSortedDisjointMap::new([
        (1..=2, &"a"),
        (5..=100, &"a")
    ])));

    // cmk00 should "to_string" be "into_string" ???

    // * from (priority) sorted_starts
    let a = [(1..=4, &"a"), (5..=100, &"a"), (5..=5, &"b")].into_iter();
    // cmk00 should we reverse the sense of priority_number so lower is better?
    let a = a
        .enumerate()
        .map(|(i, range_value)| Priority::new(range_value, i));
    let a = AssumePrioritySortedStartsMap::new(a);
    let a = UnionIterMap::new(a);
    // is_sorted_disjoint_map::<_, _, _, _>(a);
    assert!(a.equal(CheckSortedDisjointMap::new([(1..=100, &"a"),])));

    // * from unsorted_disjoint
    let iter = [(5..=100, &"a"), (5..=5, &"b"), (1..=4, &"a")].into_iter();
    let iter = iter
        .enumerate()
        .map(|(i, range_value)| Priority::new(range_value, i));
    let iter = iter.into_iter().sorted_by(|a, b| {
        // We sort only by start -- priority is not used until later.
        a.start().cmp(&b.start())
    });
    let iter = AssumePrioritySortedStartsMap::new(iter);
    let iter = UnionIterMap::new(iter);
    assert!(iter.equal(CheckSortedDisjointMap::new([(1..=100, &"a"),])));

    // * anything
    let iter = [(5, &"a"), (5, &"b"), (1, &"a")]
        .into_iter()
        .map(|(x, y)| (x..=x, y));
    let iter = UnsortedPriorityDisjointMap::new(iter.into_iter());
    let iter = iter.into_iter().sorted_by(|a, b| {
        // We sort only by start -- priority is not used until later.
        a.start().cmp(&b.start())
    });
    let iter = AssumePrioritySortedStartsMap::new(iter);
    let iter = UnionIterMap::new(iter);
    assert!(iter.equal(CheckSortedDisjointMap::new([(1..=1, &"a"), (5..=5, &"a"),])));

    //===========================
    // Map - points
    //===========================

    // * from sorted_disjoint
    let a = [(1, &"a"), (5, &"a")].into_iter().map(|(x, y)| (x..=x, y));
    let a = CheckSortedDisjointMap::new(a);
    assert!(a.equal(CheckSortedDisjointMap::new([(1..=1, &"a"), (5..=5, &"a")])));

    // cmk00 should "to_string" be "into_string" ???

    // * from (priority) sorted_starts
    let a = [(1, &"a"), (5, &"a"), (5, &"b")].into_iter();
    // cmk00 should we reverse the sense of priority_number so lower is better?
    let a = a
        .enumerate()
        .map(|(i, (k, v))| Priority::new((k..=k, v), i));
    let a = AssumePrioritySortedStartsMap::new(a);
    let a = UnionIterMap::new(a);
    // is_sorted_disjoint_map::<_, _, _, _>(a);
    assert!(a.equal(CheckSortedDisjointMap::new([(1..=1, &"a"), (5..=5, &"a")])));

    // * from unsorted_disjoint
    let iter = [(5, &"a"), (5, &"b"), (1, &"a")].into_iter();
    let iter = iter
        .enumerate()
        .map(|(i, (k, v))| Priority::new((k..=k, v), i));
    let iter = iter.into_iter().sorted_by(|a, b| {
        // We sort only by start -- priority is not used until later.
        a.start().cmp(&b.start())
    });
    let iter = AssumePrioritySortedStartsMap::new(iter);
    let iter = UnionIterMap::new(iter);
    assert!(iter.equal(CheckSortedDisjointMap::new([(1..=1, &"a"), (5..=5, &"a")])));

    // * anything
    let iter = [(5..=100, &"a"), (5..=5, &"b"), (1..=4, &"a")].into_iter();
    let iter = UnsortedPriorityDisjointMap::new(iter.into_iter());
    let iter = iter.into_iter().sorted_by(|a, b| {
        // We sort only by start -- priority is not used until later.
        a.start().cmp(&b.start())
    });
    let iter = AssumePrioritySortedStartsMap::new(iter);
    let iter = UnionIterMap::new(iter);
    assert!(iter.equal(CheckSortedDisjointMap::new([(1..=100, &"a"),])));

    //===========================
    // Set - ranges
    //===========================

    // * from sorted_disjoint
    let a = CheckSortedDisjoint::new([1..=2, 5..=100]);
    assert!(a.equal(CheckSortedDisjoint::new([1..=2, 5..=100])));

    // cmk00 should "to_string" be "into_string" ???

    // * from (priority) sorted_starts
    let a = [1..=4, 5..=100, 5..=5].into_iter();
    // cmk00 should we reverse the sense of priority_number so lower is better?
    let a = AssumeSortedStarts::new(a);
    let a = UnionIter::new(a);
    assert!(a.equal(CheckSortedDisjoint::new([1..=100])));

    // * from unsorted_disjoint
    let iter = [5..=100, 5..=5, 1..=4].into_iter();
    let iter = iter.into_iter().sorted_by(|a, b| {
        // We sort only by start -- priority is not used until later.
        a.start().cmp(&b.start())
    });
    let iter = AssumeSortedStarts::new(iter);
    let iter = UnionIter::new(iter);
    assert!(iter.equal(CheckSortedDisjoint::new([1..=100])));

    // * anything
    let iter = [5..=100, 5..=5, 1..=5].into_iter();
    let iter = iter.sorted_by(|a, b| {
        // We sort only by start -- priority is not used until later.
        a.start().cmp(&b.start())
    });
    let iter = AssumeSortedStarts::new(iter);
    let iter = UnionIter::new(iter);
    assert!(iter.equal(CheckSortedDisjoint::new([1..=100])));
    // Set - points

    // what about multiple inputs?
}

// pub fn values_to_sorted_disjoint<T, I>(iter: I) -> SortedDisjointToUnitMap<T, I>
// where
//     T: Integer,
//     I: Iterator<Item = T>, // cmk00 into_iter??? cmk00
// {
//     let iter = iter.map(|x| (x..=x, &()));
//     let iter = UnsortedDisjointMap::new(iter);
//     let sorted_disjoint_map = UnionIterMap::new(iter);
//     let sorted_disjoint = UnitMapToSortedDisjoint::new(sorted_disjoint_map);
//     sorted_disjoint
// }

// // cmk0000 rename and move
// pub fn sorted_starts_to_sorted_disjoint<T, I>(
//     sorted_starts: I,
// ) -> UnitMapToSortedDisjoint<T, UnionIterMap<T, (), &'static (), SortedStartsToUnitMap<T, I>>>
// where
//     T: Integer,
//     I: SortedStarts<T>, // into_iter??? cmk00
// {
//     let sorted_starts_map = SortedStartsToUnitMap::new(sorted_starts);
//     let sorted_disjoint_map = UnionIterMap::new(sorted_starts_map);
//     let sorted_disjoint: UnitMapToSortedDisjoint<
//         T,
//         UnionIterMap<T, (), &(), SortedStartsToUnitMap<T, I>>,
//     > = UnitMapToSortedDisjoint::new(sorted_disjoint_map);
//     sorted_disjoint
// }

// fn union_test() -> Result<(), Box<dyn std::error::Error>> {
//     // RangeSetBlaze, RangesIter, NotIter, UnionIter, Tee, UnionIter(g)
//     let a0 = RangeSetBlaze::from_iter([1..=6]);
//     let a1 = RangeSetBlaze::from_iter([8..=9]);
//     let a2 = RangeSetBlaze::from_iter([11..=15]);
//     let a12 = &a1 | &a2;
//     let not_a0 = !&a0;
//     let a = &a0 | &a1 | &a2;
//     let b = a0.ranges() | a1.ranges() | a2.ranges();
//     let c = !not_a0.ranges() | a12.ranges();
//     let d = a0.ranges() | a1.ranges() | a2.ranges();

//     // cmk000
//     let f = sorted_starts_to_sorted_disjoint(a0.iter())
//         | sorted_starts_to_sorted_disjoint(a1.iter())
//         | sorted_starts_to_sorted_disjoint(a2.iter());
//     assert!(a.ranges().equal(b));
//     assert!(a.ranges().equal(c));
//     assert!(a.ranges().equal(d));
//     assert!(a.ranges().equal(f));
//     Ok(())
// }

/// Test every function in the library that does a union like thing.
#[test]
fn test_every_union() {
    use crate::range_set_blaze::MultiwayRangeSetBlaze;

    // cmk000000
    // bitor x 4
    let a = RangeSetBlaze::from_iter([1..=6, 8..=9, 11..=15]);
    let b = RangeSetBlaze::from_iter([5..=13, 18..=29]);
    let c = &a | &b;
    assert_eq!(c, RangeSetBlaze::from_iter([1..=15, 18..=29]));
    let c = a | &b;
    let a = RangeSetBlaze::from_iter([1..=6, 8..=9, 11..=15]);
    assert_eq!(c, RangeSetBlaze::from_iter([1..=15, 18..=29]));
    let c = &a | b;
    assert_eq!(c, RangeSetBlaze::from_iter([1..=15, 18..=29]));
    let b = RangeSetBlaze::from_iter([5..=13, 18..=29]);
    let c = a | b;
    assert_eq!(c, RangeSetBlaze::from_iter([1..=15, 18..=29]));

    // bitor_assign x 2
    let mut a = RangeSetBlaze::from_iter([1..=6, 8..=9, 11..=15]);
    let b = RangeSetBlaze::from_iter([5..=13, 18..=29]);
    a |= &b;
    assert_eq!(a, RangeSetBlaze::from_iter([1..=15, 18..=29]));
    let mut a = RangeSetBlaze::from_iter([1..=6, 8..=9, 11..=15]);
    a |= b;
    assert_eq!(a, RangeSetBlaze::from_iter([1..=15, 18..=29]));

    // extend x 2
    let mut a = RangeSetBlaze::from_iter([1..=6, 8..=9, 11..=15]);
    let b = RangeSetBlaze::from_iter([5..=13, 18..=29]);
    a.extend(b.ranges());
    assert_eq!(a, RangeSetBlaze::from_iter([1..=15, 18..=29]));
    let mut a = RangeSetBlaze::from_iter([1..=6, 8..=9, 11..=15]);
    a.extend(b.iter());
    assert_eq!(a, RangeSetBlaze::from_iter([1..=15, 18..=29]));

    // append
    let mut a = RangeSetBlaze::from_iter([1..=6, 8..=9, 11..=15]);
    let mut b = RangeSetBlaze::from_iter([5..=13, 18..=29]);
    a.append(&mut b);
    assert_eq!(a, RangeSetBlaze::from_iter([1..=15, 18..=29]));
    assert!(b.is_empty());

    // // .union()
    let a = RangeSetBlaze::from_iter([1..=6, 8..=9, 11..=15]);
    let b = RangeSetBlaze::from_iter([5..=13, 18..=29]);
    let c = [&a, &b].union();
    assert_eq!(c, RangeSetBlaze::from_iter([1..=15, 18..=29]));

    // union_dyn!
    let c = union_dyn!(a.ranges(), b.ranges());
    assert!(c.equal(RangeSetBlaze::from_iter([1..=15, 18..=29]).ranges()));

    // [sorted disjoints].union()
    let c = [a.ranges(), b.ranges()].union();
    assert!(c.equal(RangeSetBlaze::from_iter([1..=15, 18..=29]).ranges()));
}
