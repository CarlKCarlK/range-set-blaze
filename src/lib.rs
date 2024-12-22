#![cfg_attr(feature = "from_slice", feature(portable_simd))]
#![doc = include_str!("../README.md")]
// cmk0 move these to Cargo.toml
#![warn(
    clippy::use_self,
    unused_lifetimes,
    missing_docs,
    single_use_lifetimes,
    clippy::pedantic,
    unreachable_pub,
    //cmk need to add stuff to examples clippy::cargo,
    //cmk clippy::cargo_common_metadata
    clippy::perf,
    clippy::style,
    clippy::complexity,
    clippy::correctness,
    clippy::nursery,
    clippy::must_use_candidate,
    clippy::unwrap_used,
    clippy::panic_in_result_fn,

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
pub use crate::map::{
    BitAndRangesMap, BitAndRangesMap2, BitSubRangesMap, BitSubRangesMap2, RangeMapBlaze,
    SortedStartsInVec, SortedStartsInVecMap, ValueRef,
};

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
pub use crate::set::{demo_read_ranges_from_file, IntoIter, Iter, RangeSetBlaze};

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

mod unsorted_disjoint_map;
pub use crate::unsorted_disjoint_map::AssumePrioritySortedStartsMap;

mod values;
pub use crate::values::Values;

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
