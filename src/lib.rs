#![cfg_attr(feature = "from_slice", feature(portable_simd))]
#![doc = include_str!("../README.md")]
#![warn(
    clippy::use_self,
    unused_lifetimes,
    missing_docs,
    single_use_lifetimes,
    clippy::pedantic,
    // cmk0 unreachable_pub,
    // cmk0 clippy::cargo,
    clippy::perf,
    clippy::style,
    clippy::complexity,
    clippy::correctness,
    clippy::nursery,
    // cmk0 clippy::cargo_common_metadata
    // cmk clippy::result_unwrap_used and clippy::option_unwrap_used: Warns if you're using .unwrap() or .expect(), which can be a sign of inadequate error handling.
    // cmk	clippy::panic_in_result_fn: Ensures functions that return Result do not contain panic!, which could be inappropriate in production code.

)]
#![no_std]
extern crate alloc;
#[cfg(feature = "std")]
extern crate std;

// cmk consider having .len() always returning the smallest type that fits the length, never usize. This would make 32-bit and 64-bit systems more consistent.

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

// FUTURE: Support serde via optional feature
mod uint_plus_one;
pub use uint_plus_one::UIntPlusOne;
mod dyn_sorted_disjoint;
mod dyn_sorted_disjoint_map;
mod from_slice;
mod integer;
pub use crate::integer::Integer;
mod intersection_iter_map;
/// cmk doc
mod iter_map;
mod map;
mod ranges_iter;
/// cmk doc
mod set;
pub use crate::range_values::{IntoRangeValuesIter, MapIntoRangesIter, MapRangesIter};
pub use crate::ranges_iter::{IntoRangesIter, RangesIter};
pub use crate::set::{IntoIter, Iter, RangeSetBlaze};
pub use crate::sorted_disjoint_map::Priority;

mod not_iter;
pub mod prelude;
mod range_values;
pub use crate::range_values::RangeValuesIter;
#[cfg(feature = "rog-experimental")]
mod rog;
mod sorted_disjoint;
// use alloc::collections::btree_map;
// use gen_ops::gen_ops_ex;
pub use crate::multiway::MultiwayRangeSetBlaze;
pub use crate::multiway::MultiwayRangeSetBlazeRef;
pub use intersection_iter_map::IntersectionIterMap;
mod map_from_iter;
mod sym_diff_iter;
mod sym_diff_iter_map;
pub use map::ValueRef;
pub use merge::{KMerge, Merge}; // cmk make public???
pub use merge_map::{KMergeMap, MergeMap}; // cmk make public???
pub use multiway::MultiwaySortedDisjoint;
pub use multiway_map::MultiwayRangeMapBlaze;
pub use multiway_map::MultiwayRangeMapBlazeRef;
pub use multiway_map::MultiwaySortedDisjointMap;
use range_values::RangeValuesToRangesIter;
pub use sym_diff_iter::SymDiffIter;
pub use sym_diff_iter_map::SymDiffIterMap;
mod multiway;
mod multiway_map;
mod sorted_disjoint_map;
mod tests_map;
mod tests_set;
mod union_iter;
mod union_iter_map;
pub(crate) mod unsorted_disjoint;
pub(crate) mod unsorted_disjoint_map;
pub use crate::map::RangeMapBlaze;
pub use crate::sorted_disjoint_map::IntoString;
pub use crate::unsorted_disjoint::AssumeSortedStarts; // cmk make public???
pub use dyn_sorted_disjoint::DynSortedDisjoint;
pub use dyn_sorted_disjoint_map::DynSortedDisjointMap;
mod merge;
mod merge_map;
pub use not_iter::NotIter;
#[cfg(feature = "rog-experimental")]
#[allow(deprecated)]
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

// #[doc(hidden)]
// pub type BitOrMerge<T, L, R> = UnionIter<T, Merge<T, L, R>>;

// cmk rename to Union...
#[doc(hidden)]
pub type BitOrMapMerge<T, VR, L, R> = UnionIterMap<T, VR, MergeMap<T, VR, L, R>>;
#[doc(hidden)]
pub type BitOrMerge<T, L, R> = UnionIter<T, merge::Merge<T, L, R>>;

#[doc(hidden)]
pub type BitXorMapMerge<T, VR, L, R> = SymDiffIterMap<T, VR, MergeMap<T, VR, L, R>>;
#[doc(hidden)]
pub type BitXorMapKMerge<T, VR, II> = SymDiffIterMap<T, VR, KMergeMap<T, VR, II>>;

#[doc(hidden)]
pub type BitXorMerge<T, L, R> = SymDiffIter<T, Merge<T, L, R>>;
#[doc(hidden)]
pub type BitXorKMerge<T, II> = SymDiffIter<T, KMerge<T, II>>;

#[doc(hidden)]
pub type BitOrMapKMerge<T, VR, I> = UnionIterMap<T, VR, KMergeMap<T, VR, I>>;
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
#[doc(hidden)]
pub type BitAndMapWithRanges<'a, T, V, VR, I> =
    IntersectionIterMap<T, VR, I, BitAndKMerge<T, MapRangesIter<'a, T, V>>>;
#[doc(hidden)]
pub type BitAndMapWithRangeValues<'a, T, VR, I> =
    IntersectionIterMap<T, VR, I, BitAndKMerge<T, RangeValuesToRangesIter<T, VR, I>>>;
#[doc(hidden)]
pub type BitNorMerge<T, L, R> = NotIter<T, BitOrMerge<T, L, R>>;
#[doc(hidden)]
pub type BitSubMerge<T, L, R> = NotIter<T, BitOrMerge<T, NotIter<T, L>, R>>;

#[cfg(feature = "std")]
#[cfg(test)]
mod tests2 {
    use alloc::vec;

    #[test]
    fn test_multiway() {
        use crate::prelude::*;

        let a = RangeSetBlaze::from_iter([1..=6, 8..=9, 11..=15]);
        let b = RangeSetBlaze::from_iter([5..=13, 18..=29]);
        let c = RangeSetBlaze::from_iter([25..=100]);
        // use crate::multiway::MultiwayRangeSetBlaze;
        let iter = vec![a, b, c].into_iter();
        let union = iter.union();
        assert_eq!(union, RangeSetBlaze::from_iter([1..=15, 18..=100]));

        let a = RangeSetBlaze::from_iter([1..=6, 8..=9, 11..=15]);
        let b = RangeSetBlaze::from_iter([5..=13, 18..=29]);
        let c = RangeSetBlaze::from_iter([25..=100]);
        // use crate::multiway::MultiwayRangeSetBlazeRef;
        let union = [a, b, c].union();
        assert_eq!(union, RangeSetBlaze::from_iter([1..=15, 18..=100]));
    }
}

// cmk update 9 rules on data structures to mention implementing into_iter() on reference.
