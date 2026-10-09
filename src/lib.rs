#![cfg_attr(docsrs, feature(doc_cfg))]
#![cfg_attr(feature = "from_slice", feature(portable_simd))]
#![doc = include_str!("../README.md")]
#![no_std]
#![cfg_attr(feature = "float_nightly_experimental", feature(f16))]
#![cfg_attr(feature = "float_nightly_experimental", feature(f128))]
#![cfg_attr(feature = "cursor_nightly_experimental", feature(btree_cursors))]

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

pub mod gaps;
pub use gaps::{FillGapsIter, FillGapsIterMap};

pub mod joins;

pub mod float;
pub use float::*;

mod integer;
pub use crate::integer::Integer;

mod inner_join_iter_map;
pub use inner_join_iter_map::InnerJoinMap;

mod intersection_iter_map;
pub use intersection_iter_map::IntersectionIterMap;

mod multiway_join_iter_map;
mod multiway_select_map;
pub use multiway_select_map::{
    MultiwaySymmetricDifferenceMap, MultiwayUnionMap, SymmetricDifferenceMap, UnionMap,
};
mod multiway_sweep;
mod multiway_sym_diff_set;
pub use multiway_join_iter_map::{MultiwayFullJoinMap, MultiwayInnerJoinMap};
pub use multiway_sweep::{MultiwaySweep, SweepEvent};
pub use multiway_sym_diff_set::MultiwaySymmetricDifference;

mod left_join_iter_map;
pub use left_join_iter_map::LeftJoinMap;

mod full_join_iter_map;
pub use full_join_iter_map::FullJoinMap;

mod transform_values_iter_map;
pub use transform_values_iter_map::TransformValuesMap;

mod iter_map;
pub use crate::iter_map::{IntoIterMap, IterMap};

mod keys;
pub use crate::keys::{IntoKeys, Keys};

mod map;
pub use crate::map::{Owned, RangeMapBlaze, ValueCarrier};

mod map_op;

mod merge;
pub use merge::{KMerge, Merge};

mod multiway;
pub use multiway::{MultiwayRangeSetBlaze, MultiwayRangeSetBlazeRef, MultiwaySortedDisjoint};

mod multiway_map;
pub use multiway_map::{
    MultiwayRangeMapBlaze, MultiwayRangeMapBlazeRef, MultiwaySortedDisjointMap,
};

mod not_iter;
mod operation_results;
pub use not_iter::NotIter;
pub use operation_results::{
    Difference, DifferenceMap, Intersection, IntersectionMap, MultiwayIntersection,
    MultiwayIntersectionMap, MultiwayUnion, NotMap, SymmetricDifference, Union,
};

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
pub use sorted_disjoint::{CheckSortedDisjoint, RangeOnce, SortedDisjoint, SortedStarts};

mod sorted_disjoint_map;
pub use sorted_disjoint_map::{
    CheckSortedDisjointMap, IntoString, SortedDisjointMap, SortedStartsMap,
};

mod sym_diff_iter;
pub use sym_diff_iter::SymDiffIter;

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

#[cfg(any(test, feature = "test_util"))]
#[doc(hidden)]
pub mod test_util;

// Internal modules
pub(crate) mod from_slice;
pub(crate) mod map_from_iter;
#[cfg(all(test, feature = "cursor_nightly_experimental"))]
mod tests_cursor_lookup;
pub(crate) mod tests_map;
pub(crate) mod tests_set;

// Helpers
type DifferenceMapInternal<T, VC, L, R> = IntersectionIterMap<T, VC, L, NotIter<T, R>>;
// Implementations of the set union and symmetric difference result types (see
// `operation_results`), also used directly by the code that builds them.
type UnionInner<T, L, R> = UnionIter<T, Merge<T, L, R>>;
type MultiwayUnionInner<T, I> = UnionIter<T, KMerge<T, I>>;
type SymmetricDifferenceInner<T, L, R> = SymDiffIter<T, Merge<T, L, R>>;

// Former names of the operation result types, kept so code that named them gets a deprecation
// warning (with the new name) instead of an error.
#[doc(hidden)]
#[deprecated(note = "renamed to `Union`")]
pub type UnionMerge<T, L, R> = Union<T, L, R>;
#[doc(hidden)]
#[deprecated(note = "renamed to `Intersection`")]
pub type IntersectionMerge<T, L, R> = Intersection<T, L, R>;
#[doc(hidden)]
#[deprecated(note = "renamed to `Difference`")]
pub type DifferenceMerge<T, L, R> = Difference<T, L, R>;
#[doc(hidden)]
#[deprecated(note = "renamed to `SymmetricDifference`")]
pub type SymDiffMerge<T, L, R> = SymmetricDifference<T, L, R>;
#[doc(hidden)]
#[deprecated(note = "renamed to `MultiwayUnion`")]
pub type UnionKMerge<T, I> = MultiwayUnion<T, I>;
#[doc(hidden)]
#[deprecated(note = "renamed to `MultiwaySymmetricDifference`")]
pub type SymDiffKMerge<T, I> = MultiwaySymmetricDifference<T, I>;
#[doc(hidden)]
#[deprecated(note = "renamed to `UnionMap`")]
pub type UnionMergeMap<T, VC, L, R> = UnionMap<T, VC, L, R>;
#[doc(hidden)]
#[deprecated(note = "renamed to `SymmetricDifferenceMap`")]
pub type SymDiffMergeMap<T, VC, L, R> = SymmetricDifferenceMap<T, VC, L, R>;
#[doc(hidden)]
#[deprecated(note = "renamed to `MultiwayUnionMap`")]
pub type UnionKMergeMap<T, VC, I> = MultiwayUnionMap<T, VC, I>;
#[doc(hidden)]
#[deprecated(note = "renamed to `MultiwaySymmetricDifferenceMap`")]
pub type SymDiffKMergeMap<T, VC, I> = MultiwaySymmetricDifferenceMap<T, VC, I>;
#[doc(hidden)]
#[deprecated(note = "renamed to `MultiwayIntersectionMap` (without the lifetime parameter)")]
pub type IntersectionKMap<'a, T, VC, I> = MultiwayIntersectionMap<T, VC, I>;
