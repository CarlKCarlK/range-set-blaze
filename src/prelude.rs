//! This prelude module provides a convenient way to import the most commonly used
//! types, traits, and functions.
//!
//! ```
//! use range_set_blaze::prelude::*;
//! ```
pub use crate::{
    intersection_dyn, intersection_map_dyn, union_dyn, union_map_dyn, CheckSortedDisjoint,
    DynSortedDisjoint, DynSortedDisjointMap, MultiwayRangeMapBlaze, MultiwayRangeSetBlaze,
    MultiwaySortedDisjoint, MultiwaySortedDisjointMap, RangeMapBlaze, RangeSetBlaze,
    SortedDisjoint, SortedDisjointMap, SortedStarts,
};
