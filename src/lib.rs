#![cfg_attr(feature = "from_slice", feature(portable_simd))]
#![doc = include_str!("../README.md")]
#![cfg_attr(not(feature = "std"), no_std)]
#![warn(
    clippy::use_self,
    unused_lifetimes,
    missing_docs,
    single_use_lifetimes,
    clippy::pedantic,
    // cmk00 unreachable_pub,
    clippy::cargo,
    clippy::perf,
    clippy::style,
    clippy::complexity,
    clippy::correctness,
    clippy::nursery,
    clippy::cargo_common_metadata
)]

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
mod ranges;
/// cmk doc
pub mod set;
pub use crate::range_values::IntoRangeValuesIter;
pub use crate::range_values::{MapIntoRangesIter, MapRangesIter};
pub use crate::ranges::IntoRangesIter;
pub use crate::ranges::RangesIter;
pub use crate::set::RangeSetBlaze;

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
pub use crate::multiway::MultiwayRangeSetBlazeRef;
pub use intersection_iter_map::IntersectionIterMap;
mod sym_diff_iter;
mod sym_diff_iter_map;
pub use map::CloneBorrow;
pub use map::ValueOwned;
use merge::{KMerge, Merge};
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
pub use crate::sorted_disjoint_map::IntoString;
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
pub type BitOrMapMerge<T, V, VR, L, R> = UnionIterMap<T, V, VR, MergeMap<T, V, VR, L, R>>;
#[doc(hidden)]
pub type BitOrMerge<T, L, R> = UnionIter<T, merge::Merge<T, L, R>>;

#[doc(hidden)]
pub type BitXorMapMerge<T, V, VR, L, R> = SymDiffIterMap<T, V, VR, MergeMap<T, V, VR, L, R>>;
#[doc(hidden)]
pub type BitXorMapKMerge<T, V, VR, II> = SymDiffIterMap<T, V, VR, KMergeMap<T, V, VR, II>>;

#[doc(hidden)]
pub type BitXorMerge<T, L, R> = SymDiffIter<T, Merge<T, L, R>>;
#[doc(hidden)]
pub type BitXorKMerge<T, II> = SymDiffIter<T, KMerge<T, II>>;

#[doc(hidden)]
pub type BitOrMapKMerge<T, V, VR, I> = UnionIterMap<T, V, VR, KMergeMap<T, V, VR, I>>;
#[doc(hidden)]
pub type BitOrKMerge<T, I> = UnionIter<T, KMerge<T, I>>;
#[doc(hidden)]
pub type BitAndMerge<T, L, R> = NotIter<T, BitNandMerge<T, L, R>>;
#[doc(hidden)]
pub type BitAndKMerge<T, I> = NotIter<T, BitNandKMerge<T, I>>;
#[doc(hidden)]
pub type BitNandMerge<T, L, R> = BitOrMerge<T, NotIter<T, L>, NotIter<T, R>>;
#[doc(hidden)]
pub type BitNandKMerge<T, I> = BitOrKMerge<T, NotIter<T, I>>;
#[doc(hidden)] // cmk00 create better name
pub type BitAndMapWithRanges<'a, T, V, VR, I> =
    IntersectionIterMap<T, V, VR, I, BitAndKMerge<T, MapRangesIter<'a, T, V>>>;
#[doc(hidden)]
pub type BitAndMapWithRangeValues<'a, T, V, VR, I> =
    IntersectionIterMap<T, V, VR, I, BitAndKMerge<T, RangeValuesToRangesIter<T, V, VR, I>>>;
#[doc(hidden)]
pub type BitNorMerge<T, L, R> = NotIter<T, BitOrMerge<T, L, R>>;
#[doc(hidden)]
pub type BitSubMerge<T, L, R> = NotIter<T, BitOrMerge<T, NotIter<T, L>, R>>;
