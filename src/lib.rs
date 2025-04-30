#![cfg_attr(feature = "from_slice", feature(portable_simd))]
#![feature(btree_cursors)]
#![doc = include_str!("../README.md")]
// cmk move these to Cargo.toml
#![warn(
    clippy::use_self,
    unused_lifetimes,
    missing_docs,
    single_use_lifetimes,
    clippy::pedantic,
    unreachable_pub,
    clippy::cargo,
    clippy::cargo_common_metadata,
    clippy::perf,
    clippy::style,
    clippy::complexity,
    clippy::correctness,
    clippy::nursery,
    clippy::must_use_candidate,
    clippy::unwrap_used,
    clippy::panic_in_result_fn
)]
#![no_std]
extern crate alloc;
#[cfg(feature = "std")]
extern crate std;

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

// Prelude: Simplified imports for common use
pub mod prelude;

// General Imports
mod dyn_sorted_disjoint;

pub use dyn_sorted_disjoint::DynSortedDisjoint;

mod dyn_sorted_disjoint_map;
pub use dyn_sorted_disjoint_map::DynSortedDisjointMap;

mod integer;
pub use crate::integer::Integer;

mod intersection_iter_map;
pub use intersection_iter_map::IntersectionIterMap;

mod iter_map;
pub use crate::iter_map::{IntoIterMap, IterMap};

mod keys;
pub use crate::keys::{IntoKeys, Keys};

mod map;
pub use crate::map::{RangeMapBlaze, ValueRef};

mod merge;
pub use merge::{KMerge, Merge};

mod merge_map;
pub use merge_map::{KMergeMap, MergeMap};

mod multiway;
pub use multiway::{MultiwayRangeSetBlaze, MultiwayRangeSetBlazeRef, MultiwaySortedDisjoint};

mod multiway_map;
pub use multiway_map::{
    MultiwayRangeMapBlaze, MultiwayRangeMapBlazeRef, MultiwaySortedDisjointMap,
};

mod not_iter;
pub use not_iter::NotIter;

mod range_values;
pub use crate::range_values::{
    IntoRangeValuesIter, MapIntoRangesIter, MapRangesIter, RangeValuesIter, RangeValuesToRangesIter,
};

mod ranges_iter;
pub use crate::ranges_iter::{IntoRangesIter, RangesIter};

mod set;
#[cfg(all(not(coverage), feature = "std"))]
pub use crate::set::demo_read_ranges_from_file;
pub use crate::set::{IntoIter, Iter, RangeSetBlaze};

mod sorted_disjoint;
pub use sorted_disjoint::{CheckSortedDisjoint, SortedDisjoint, SortedStarts};

mod sorted_disjoint_map;
pub use sorted_disjoint_map::{
    CheckSortedDisjointMap, IntoString, SortedDisjointMap, SortedStartsMap,
};

mod sym_diff_iter;
pub use sym_diff_iter::SymDiffIter;

mod sym_diff_iter_map;
pub use sym_diff_iter_map::SymDiffIterMap;

mod union_iter;
pub use union_iter::UnionIter;

mod union_iter_map;
pub use union_iter_map::UnionIterMap;

mod unsorted_disjoint;
pub use crate::unsorted_disjoint::AssumeSortedStarts;

mod unsorted_priority_map;
pub use crate::unsorted_priority_map::AssumePrioritySortedStartsMap;

mod values;
pub use crate::values::{IntoValues, Values};

mod uint_plus_one;
pub use uint_plus_one::UIntPlusOne;

#[cfg(feature = "rog-experimental")]
mod rog;
#[cfg(feature = "rog-experimental")]
#[allow(deprecated)]
pub use rog::{Rog, RogsIter};

// Internal modules
pub(crate) mod from_slice;
pub(crate) mod map_from_iter;
pub(crate) mod tests_map;
pub(crate) mod tests_set;

// Helpers
type NandMerge<T, L, R> = UnionMerge<T, NotIter<T, L>, NotIter<T, R>>;
type NandKMerge<T, I> = UnionKMerge<T, NotIter<T, I>>;
type DifferenceMapInternal<T, VR, L, R> = IntersectionIterMap<T, VR, L, NotIter<T, R>>;
type IntersectionMapInternal<T, I> = NotIter<T, NandKMerge<T, I>>;

// Public Types
#[doc(hidden)]
pub type DifferenceMap<T, VR, L, R> =
    DifferenceMapInternal<T, VR, L, RangeValuesToRangesIter<T, VR, R>>;
#[doc(hidden)]
pub type DifferenceMerge<T, L, R> = NotIter<T, UnionMerge<T, NotIter<T, L>, R>>;

#[doc(hidden)]
pub type IntersectionKMap<'a, T, VR, I> =
    IntersectionIterMap<T, VR, I, IntersectionMapInternal<T, RangeValuesToRangesIter<T, VR, I>>>;
#[doc(hidden)]
pub type IntersectionMap<T, VR, L, R> =
    IntersectionIterMap<T, VR, L, RangeValuesToRangesIter<T, VR, R>>;
#[doc(hidden)]
pub type IntersectionMerge<T, L, R> = NotIter<T, NandMerge<T, L, R>>;

#[doc(hidden)]
pub type NotMap<T, VR, I> = NotIter<T, RangeValuesToRangesIter<T, VR, I>>;

#[doc(hidden)]
pub type SymDiffKMerge<T, II> = SymDiffIter<T, KMerge<T, II>>;
#[doc(hidden)]
pub type SymDiffKMergeMap<T, VR, II> = SymDiffIterMap<T, VR, KMergeMap<T, VR, II>>;
#[doc(hidden)]
pub type SymDiffMerge<T, L, R> = SymDiffIter<T, Merge<T, L, R>>;
#[doc(hidden)]
pub type SymDiffMergeMap<T, VR, L, R> = SymDiffIterMap<T, VR, MergeMap<T, VR, L, R>>;

#[doc(hidden)]
pub type UnionKMerge<T, I> = UnionIter<T, KMerge<T, I>>;
#[doc(hidden)]
pub type UnionKMergeMap<T, VR, I> = UnionIterMap<T, VR, KMergeMap<T, VR, I>>;
#[doc(hidden)]
pub type UnionMerge<T, L, R> = UnionIter<T, merge::Merge<T, L, R>>;
#[doc(hidden)]
pub type UnionMergeMap<T, VR, L, R> = UnionIterMap<T, VR, MergeMap<T, VR, L, R>>;
