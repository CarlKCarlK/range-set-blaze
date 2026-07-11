//! Experimental support for floating point ranges.\
//! Enable with `float_experimental` (stable, `f32`/`f64`) and
//! `float_nightly_experimental` (nightly, adds `f16`/`f128`).
//!
//! Exports two types of floating point range types.\
//! Total: Every bit pattern is valid and distinct.\
//! Finite: Only finite floating point values are valid, e.g. `f64::MIN..=f64::MAX`. Also, -0.0 is treated as 0.0
//!
//! Each of those is available in four sizes: 16, 32, 64 and 128.
//!
//! ## Example: Merging Overlapping Intervals
//!
//! Turn a list of unsorted, overlapping f64 intervals (inclusive) into
//! a sorted list of disjoint intervals (still inclusive).
//! ```
//! use range_set_blaze::{RangeSetBlaze, FiniteF64, finite::FiniteRangeExt};
//!
//! let overlapping_intervals = vec![
//!     (11.0, 13.0),
//!     (1.00000001, 2.99999999),
//!     (30.0, 31.0),
//!     (5.50000002, 9.00000006),
//!     (-4.0, 1.99999999),
//!     (15.0, 15.0),
//!     (7.00000003, 7.49999997),
//!     (21.00000004, 28.0),
//!     (2.50000005, 5.99999996),
//!     (19.0, 21.99999998),
//! ];
//!
//! let disjoint_intervals: Vec<_> = overlapping_intervals
//!     .into_iter()
//!     .map(|(s, e)| FiniteF64::from_primitive_range(s..=e)) // Wrap each range
//!     .collect::<RangeSetBlaze<_>>() // Coalesce into disjoint ranges
//!     .ranges() // Convert back
//!     .map(FiniteRangeExt::into_primitive_inner)
//!     .collect();
//!
//! assert_eq!(
//!     disjoint_intervals,
//!     vec![
//!         (-4.0, 9.00000006),
//!         (11.0, 13.0),
//!         (15.0, 15.0),
//!         (19.0, 28.0),
//!         (30.0, 31.0)
//!     ]
//! );
//! ```
//!
//! ## Example: Wrapping Primitive (e.g. f32, f64) Ranges
//!
//! ```
//! use range_set_blaze::{finite::ff64, total::{tf32, tf64}, RangeSetBlaze};
//!
//! // Alternatively, use `TotalF64::new`, `TotalF32::new`, and `FiniteF64::new`.
//! let set = RangeSetBlaze::from_iter([tf64(3.0)..=tf64(5.0)]);
//! assert!(set.contains(tf64(3.1)));
//! assert!(!set.contains(tf64(2.9)));
//!
//! let set = RangeSetBlaze::from(ff64(3.0)..=ff64(5.0));
//! assert!(set.contains(ff64(4.9)));
//! assert!(!set.contains(ff64(5.1)));
//!
//! let set = RangeSetBlaze::from_iter([tf32(3.0)..=tf32(5.0), tf32(7.0)..=tf32(9.0)]);
//! assert!(set.contains(tf32(4.0)));
//! assert!(!set.contains(tf32(6.0)));
//! ```
//!
//! ## Example: Coalescing Two Billion f32's
#![doc = concat!(
    "````rust,no_run\n",
    include_str!("../examples/float_maps.rs"),
    "\n````"
)]
//! ## Example: Total Floats
//!
//! Unlike `Finite`, `Total` uses `total_cmp` order, so it can also represent
//! NaN, the infinities, and -0.0 as distinct, orderable values. This example
//! builds a category map over all of `f64` from nothing but
//! `TotalF64::MIN`/`MAX`, the two infinities, and `-0.0`: start with NaN
//! everywhere, carve out the finite range between the infinities as Normal,
//! then punch in the infinities and -0.0 as exceptions (overlapping ranges
//! have right-to-left precedence, so later entries win).
//!
//! ```
//! use range_set_blaze::{RangeMapBlaze, TotalF64, total::{tf64, TotalRangeExt}};
//!
//! #[derive(Debug, Clone, Copy, PartialEq, Eq)]
//! enum Category {
//!     NaN,
//!     NegInfinity,
//!     PosInfinity,
//!     MinusZero,
//!     Normal,
//! }
//!
//! let category_map = RangeMapBlaze::from_iter([
//!     (TotalF64::MIN..=TotalF64::MAX, Category::NaN),
//!     (tf64(f64::NEG_INFINITY)..=tf64(f64::INFINITY), Category::Normal),
//!     (tf64(f64::NEG_INFINITY)..=tf64(f64::NEG_INFINITY), Category::NegInfinity),
//!     (tf64(f64::INFINITY)..=tf64(f64::INFINITY), Category::PosInfinity),
//!     (tf64(-0.0)..=tf64(-0.0), Category::MinusZero),
//! ]);
//!
//! # assert_eq!(category_map.get(tf64(f64::NAN)), Some(&Category::NaN));
//! # assert_eq!(category_map.get(tf64(f64::NEG_INFINITY)), Some(&Category::NegInfinity));
//! # assert_eq!(category_map.get(tf64(f64::INFINITY)), Some(&Category::PosInfinity));
//! # assert_eq!(category_map.get(tf64(-0.0)), Some(&Category::MinusZero));
//! # assert_eq!(category_map.get(tf64(0.0)), Some(&Category::Normal));
//!
//! for (range, category) in category_map.range_values() {
//!     let (start, end) = range.into_primitive_inner();
//!     println!(
//!         "{start:e} (0x{:016x}) ..= {end:e} (0x{:016x}) -> {category:?}",
//!         start.to_bits(),
//!         end.to_bits(),
//!     );
//! }
//! // Output:
//! // NaN (0xffffffffffffffff) ..= NaN (0xfff0000000000001) -> NaN
//! // -inf (0xfff0000000000000) ..= -inf (0xfff0000000000000) -> NegInfinity
//! // -1.7976931348623157e308 (0xffefffffffffffff) ..= -5e-324 (0x8000000000000001) -> Normal
//! // -0e0 (0x8000000000000000) ..= -0e0 (0x8000000000000000) -> MinusZero
//! // 0e0 (0x0000000000000000) ..= 1.7976931348623157e308 (0x7fefffffffffffff) -> Normal
//! // inf (0x7ff0000000000000) ..= inf (0x7ff0000000000000) -> PosInfinity
//! // NaN (0x7ff0000000000001) ..= NaN (0x7fffffffffffffff) -> NaN
//! ```

mod finite_float;
mod total_float;

pub mod total;
pub use total::{Total, TotalF32, TotalF64};
#[cfg(feature = "float_nightly_experimental")]
pub use total::{TotalF16, TotalF128};

pub mod finite;
pub use finite::{Finite, FiniteF32, FiniteF64};
#[cfg(feature = "float_nightly_experimental")]
pub use finite::{FiniteF16, FiniteF128};
