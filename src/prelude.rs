//! This prelude module provides a convenient way to import the most commonly used
//! types, traits, and functions.
//!
//! ```
//! use range_set_blaze::prelude::*;
//! ```
pub use crate::{
    intersection_dyn, union_dyn, CheckSortedDisjoint, DynSortedDisjoint, MultiwayRangeMapBlaze,
    MultiwayRangeSetBlaze, MultiwayRangeSetBlazeRef, MultiwaySortedDisjoint, RangeMapBlaze,
    RangeSetBlaze, SortedDisjoint, SortedDisjointMap,
};
