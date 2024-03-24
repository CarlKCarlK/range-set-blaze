//! This prelude module provides a convenient way to import the most commonly used
//! types, traits, and functions.
//!
//! ```
//! use range_set_blaze::prelude::*;
//! ```
pub use crate::{
    intersection_dyn, intersection_map_dyn, lib2::MultiwayRangeSetBlaze, union_dyn, union_map_dyn,
    AssumeSortedStarts, CheckSortedDisjoint, DynSortedDisjoint, DynSortedDisjointMap,
    MultiwayRangeMapBlaze, MultiwaySortedDisjoint, MultiwaySortedDisjointMap, RangeMapBlaze,
    RangeSetBlaze2, SortedDisjoint, SortedDisjointMap, SortedStarts,
};
