//! Floating-point range support.\
//! The `f32`/`f64` wrappers are available by default. Enable
//! `float_nightly_experimental` on nightly to add the `f16`/`f128` wrappers.
//!
//! Exports two types of floating point range types.\
//! Total: Every bit pattern is valid and distinct, including NaNs, infinities, and signed zero.\
//! `NotNan`: Every value except NaN is valid, e.g. `f64::NEG_INFINITY..=f64::INFINITY`. Also, -0.0 is treated as 0.0
//!
//! Each of those is available in four sizes: 16, 32, 64 and 128.
//!
//! ## Example: Merging Overlapping Intervals
//!
//! Turn a list of unsorted, overlapping f64 intervals (inclusive) into
//! a sorted list of disjoint intervals (still inclusive).
//! ```
//! use range_set_blaze::{RangeSetBlaze, NotNanF64, not_nan::NotNanRangeExt};
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
//!     .map(|(s, e)| NotNanF64::from_primitive_range(s..=e)) // Wrap each range
//!     .collect::<RangeSetBlaze<_>>() // Coalesce into disjoint ranges
//!     .ranges() // Convert back
//!     .map(NotNanRangeExt::into_primitive_inner)
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
//! use range_set_blaze::{not_nan::nnf64, total::{tf32, tf64}, RangeSetBlaze};
//!
//! // Alternatively, use `TotalF64::new`, `TotalF32::new`, and `NotNanF64::new`.
//! let set = RangeSetBlaze::from_iter([tf64(3.0)..=tf64(5.0)]);
//! assert!(set.contains(tf64(3.1)));
//! assert!(!set.contains(tf64(2.9)));
//!
//! let set = RangeSetBlaze::from(nnf64(3.0)..=nnf64(5.0));
//! assert!(set.contains(nnf64(4.9)));
//! assert!(!set.contains(nnf64(5.1)));
//!
//! let set = RangeSetBlaze::from_iter([tf32(3.0)..=tf32(5.0), tf32(7.0)..=tf32(9.0)]);
//! assert!(set.contains(tf32(4.0)));
//! assert!(!set.contains(tf32(6.0)));
//! ```
//!
//! ## From Native Float Slices and Vectors
//!
//! To build a set or map from many native floats (e.g. `f64`), first convert them in bulk, then
//! pass the result to [`RangeSetBlaze::from_slice`] or [`RangeMapBlaze::from_slices`]:
//!
//! | You have   | Convert with                          | Copies? | NaN    | `-0.0`               |
//! |------------|---------------------------------------|---------|--------|----------------------|
//! | `&[f64]`   | [`NotNanF64::from_primitive_slice`]   | No (a view) | panics | panics (a view can't rewrite it) |
//! | `Vec<f64>` | [`NotNanF64::from_primitive_vec`]     | No (reuses the allocation) | panics | normalized to `+0.0` |
//! | `&[f64]`   | [`TotalF64::from_primitive_slice`]    | No (a view) | kept   | kept, distinct from `+0.0` |
//!
//! The `NotNan` conversions check every value with SIMD for `f32`/`f64`, so they cost about as
//! much as reading the data once. The same methods exist for the other sizes.
//!
//! ```
//! use range_set_blaze::{NotNanF64, RangeMapBlaze, RangeSetBlaze};
//!
//! // A view: no copy, but `-0.0` (like NaN) panics.
//! let data = [1.0, 2.0, 5.0];
//! let set = RangeSetBlaze::from_slice(NotNanF64::from_primitive_slice(&data));
//! assert!(set.contains(NotNanF64::new(5.0)));
//!
//! // From an owned vector: reuses its allocation, and turns `-0.0` into `+0.0`.
//! let keys = NotNanF64::from_primitive_vec(vec![-0.0, 1.0, 2.0]);
//! let map = RangeMapBlaze::from_slices(&keys, ["a", "b", "b"]);
//! assert_eq!(map.get(NotNanF64::new(0.0)), Some(&"a"));
//! ```
//!
//! [`RangeSetBlaze::from_slice`]: crate::RangeSetBlaze::from_slice
//! [`RangeMapBlaze::from_slices`]: crate::RangeMapBlaze::from_slices
//! [`NotNanF64::from_primitive_slice`]: crate::NotNan::from_primitive_slice
//! [`NotNanF64::from_primitive_vec`]: crate::NotNan::from_primitive_vec
//! [`TotalF64::from_primitive_slice`]: crate::Total::from_primitive_slice
//!
//! ## Example: Coalescing Two Billion f32's
#![doc = concat!(
    "````rust,no_run\n",
    include_str!("../examples/float_maps.rs"),
    "\n````"
)]
//! ## Example: Total Floats
//!
//! Unlike `NotNan`, `Total` also admits NaN and keeps -0.0 as a value distinct
//! from +0.0, all ordered via `total_cmp`. This example
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

mod not_nan_float;
mod not_nan_simd;
mod total_float;

pub mod total;
pub use total::{Total, TotalF32, TotalF64};
#[cfg(feature = "float_nightly_experimental")]
pub use total::{TotalF16, TotalF128};

pub mod not_nan;
pub use not_nan::{NotNan, NotNanF32, NotNanF64};
#[cfg(feature = "float_nightly_experimental")]
pub use not_nan::{NotNanF16, NotNanF128};
