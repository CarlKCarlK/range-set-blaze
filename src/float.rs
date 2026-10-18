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
//! ## Converting Native Floats
//!
//! Choose a wrapper by the meaning you want:
//!
//! * [`NotNan`] (e.g. [`NotNanF64`]) is the usual choice: ordinary numeric order, NaN rejected, and
//!   `-0.0` and `+0.0` treated as the same value (stored as `+0.0`).
//! * [`Total`] (e.g. [`TotalF64`]) keeps every bit pattern as its own value, ordered by
//!   `total_cmp`: NaNs are kept, and `-0.0` comes just before `+0.0`.
//!
//! **Into a wrapper** (shown for `f64`; `f32`, `f16`, and `f128` work the same way):
//!
//! | You have              | `NotNan`                                                    | `Total`                                     |
//! |-----------------------|-------------------------------------------------------------|---------------------------------------------|
//! | one value             | [`NotNanF64::new`], [`try_new`] (returns `Option`), [`nnf64`] (`const`) | [`TotalF64::new`], [`tf64`] (`const`) |
//! | one range `a..=b`     | [`NotNanF64::from_primitive_range`]                         | [`TotalF64::from_primitive_range`]          |
//! | an iterator of ranges | [`NotNanF64::from_primitive_ranges`]                        | [`TotalF64::from_primitive_ranges`]         |
//! | an iterator of values | [`NotNanF64::values`]                                       | [`TotalF64::values`]                        |
//! | `&[f64]` (borrowed)   | [`NotNanF64::from_primitive_slice`]: a view; also panics on `-0.0` | [`TotalF64::from_primitive_slice`]: a view |
//! | `Vec<f64>` (owned)    | [`NotNanF64::from_primitive_vec`]: reuses the allocation    | `v.into_iter().map(TotalF64::new).collect()` (also reuses it) |
//!
//! Range conversions wrap just the two endpoints; they don't list the floats in between.
//!
//! **NaN and `-0.0` with `NotNan`:** every `NotNan` conversion panics on NaN, except
//! [`try_new`], which returns `None`. Every one turns `-0.0` into `+0.0`, except
//! [`NotNanF64::from_primitive_slice`]: a borrowed view can't rewrite memory, so it panics on
//! `-0.0` instead. If your data may contain `-0.0`, use [`NotNanF64::from_primitive_vec`] or
//! [`NotNanF64::values`]. (Advanced: the `unsafe` [`NotNanF64::new_unchecked`] and
//! [`NotNanF64::from_primitive_slice_unchecked`] skip these checks when you can guarantee there is
//! no NaN and no `-0.0`.)
//!
//! **Building a set or map:** pass wrapped slices to [`RangeSetBlaze::from_slice`] or
//! [`RangeMapBlaze::from_slices`], and wrapped values or ranges to `from_iter`/`collect`.
//!
//! **Back to native floats** (both wrappers):
//!
//! | You have  | Use                                                                                     |
//! |-----------|-----------------------------------------------------------------------------------------|
//! | one value | `into_inner` ([`NotNan::into_inner`], [`Total::into_inner`])                            |
//! | a range   | `into_primitive_range` (gives `a..=b`) or `into_primitive_inner` (gives `(a, b)`), from [`NotNanRangeExt`] or [`TotalRangeExt`] |
//! | a slice   | `as_primitive_slice`, a view, from [`NotNanSliceExt`] or [`TotalSliceExt`]              |
//!
//! ```
//! use range_set_blaze::{NotNanF64, RangeMapBlaze, RangeSetBlaze, not_nan::NotNanRangeExt};
//!
//! // A borrowed slice: a view, no copy. (`-0.0`, like NaN, would panic.)
//! let data = [1.0, 2.0, 5.0];
//! let set = RangeSetBlaze::from_slice(NotNanF64::from_primitive_slice(&data));
//! assert!(set.contains(NotNanF64::new(5.0)));
//!
//! // An owned vector: reuses its allocation, and turns `-0.0` into `+0.0`.
//! let keys = NotNanF64::from_primitive_vec(vec![-0.0, 1.0, 2.0]);
//! let map = RangeMapBlaze::from_slices(&keys, ["a", "b", "b"]);
//! assert_eq!(map.get(NotNanF64::new(0.0)), Some(&"a"));
//!
//! // Ranges in, ranges out.
//! let set = RangeSetBlaze::from_iter(NotNanF64::from_primitive_ranges([3.0..=4.0, 1.0..=2.0]));
//! let first = set.ranges().next().map(NotNanRangeExt::into_primitive_range);
//! assert_eq!(first, Some(1.0..=2.0));
//! ```
//!
//! ## Floats as Map Values
//!
//! A [`RangeMapBlaze`]'s values must implement `Eq`, which native floats don't (NaN isn't equal to
//! itself), so `RangeMapBlaze<u32, f64>` doesn't compile. Use a wrapper as the value type
//! instead, such as `RangeMapBlaze<u32, NotNanF64>`. Neighboring keys with equal values merge into
//! one range, using the wrapper's meaning of "equal": with `NotNan`, `-0.0` and `+0.0` are the same
//! value; with `Total`, they're different, and NaNs with the same bits are equal.
//!
//! ```
//! use range_set_blaze::{NotNanF64, RangeMapBlaze, TotalF64};
//!
//! // Price per unit by quantity tier.
//! let prices = RangeMapBlaze::from_iter([
//!     (1..=9, NotNanF64::new(2.50)),
//!     (10..=99, NotNanF64::new(2.00)),
//!     (100..=u32::MAX, NotNanF64::new(1.75)),
//! ]);
//! assert_eq!(prices.get(42).map(|price| price.into_inner()), Some(2.00));
//!
//! // From columns: wrap the float column in bulk, then use `from_slices`.
//! let sensor_ids = [7_u32, 8, 9, 10];
//! let readings = [0.5, 0.5, 0.5, f64::NAN]; // NaN means "no reading"
//! let map = RangeMapBlaze::from_slices(sensor_ids, TotalF64::from_primitive_slice(&readings));
//! assert_eq!(map.range_values_len(), 2); // 7..=9 -> 0.5, 10..=10 -> NaN
//! ```
//!
//! [`NotNan`]: crate::NotNan
//! [`NotNanF64`]: crate::NotNanF64
//! [`Total`]: crate::Total
//! [`TotalF64`]: crate::TotalF64
//! [`NotNanF64::new`]: crate::NotNan::new
//! [`try_new`]: crate::NotNan::try_new
//! [`nnf64`]: crate::not_nan::nnf64
//! [`TotalF64::new`]: crate::Total::new
//! [`tf64`]: crate::total::tf64
//! [`NotNanF64::from_primitive_range`]: crate::NotNan::from_primitive_range
//! [`TotalF64::from_primitive_range`]: crate::Total::from_primitive_range
//! [`NotNanF64::from_primitive_ranges`]: crate::NotNan::from_primitive_ranges
//! [`TotalF64::from_primitive_ranges`]: crate::Total::from_primitive_ranges
//! [`NotNanF64::values`]: crate::NotNan::values
//! [`TotalF64::values`]: crate::Total::values
//! [`NotNanF64::from_primitive_slice`]: crate::NotNan::from_primitive_slice
//! [`TotalF64::from_primitive_slice`]: crate::Total::from_primitive_slice
//! [`NotNanF64::from_primitive_vec`]: crate::NotNan::from_primitive_vec
//! [`NotNanF64::new_unchecked`]: crate::NotNan::new_unchecked
//! [`NotNanF64::from_primitive_slice_unchecked`]: crate::NotNan::from_primitive_slice_unchecked
//! [`RangeSetBlaze::from_slice`]: crate::RangeSetBlaze::from_slice
//! [`RangeMapBlaze::from_slices`]: crate::RangeMapBlaze::from_slices
//! [`RangeMapBlaze`]: crate::RangeMapBlaze
//! [`NotNan::into_inner`]: crate::NotNan::into_inner
//! [`Total::into_inner`]: crate::Total::into_inner
//! [`NotNanRangeExt`]: crate::not_nan::NotNanRangeExt
//! [`TotalRangeExt`]: crate::total::TotalRangeExt
//! [`NotNanSliceExt`]: crate::not_nan::NotNanSliceExt
//! [`TotalSliceExt`]: crate::total::TotalSliceExt
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

#[cfg(feature = "from_slice_in_place")]
pub(crate) fn gpu_not_nan_f32_to_ordered(value: f32) -> i32 {
    <f32 as not_nan_float::NotNanFloatImpl>::to_ordered(value)
}

#[cfg(feature = "from_slice_in_place")]
pub(crate) fn gpu_not_nan_f32_from_ordered(value: i32) -> f32 {
    <f32 as not_nan_float::NotNanFloatImpl>::from_ordered(value)
}

#[cfg(feature = "from_slice_in_place")]
pub(crate) fn gpu_not_nan_f64_to_ordered(value: f64) -> i64 {
    <f64 as not_nan_float::NotNanFloatImpl>::to_ordered(value)
}

#[cfg(feature = "from_slice_in_place")]
pub(crate) fn gpu_not_nan_f64_from_ordered(value: i64) -> f64 {
    <f64 as not_nan_float::NotNanFloatImpl>::from_ordered(value)
}

#[cfg(all(
    feature = "from_slice_in_place",
    feature = "float_nightly_experimental"
))]
pub(crate) fn gpu_not_nan_f16_to_ordered(value: f16) -> i16 {
    <f16 as not_nan_float::NotNanFloatImpl>::to_ordered(value)
}

#[cfg(all(
    feature = "from_slice_in_place",
    feature = "float_nightly_experimental"
))]
pub(crate) fn gpu_not_nan_f16_from_ordered(value: i16) -> f16 {
    <f16 as not_nan_float::NotNanFloatImpl>::from_ordered(value)
}

pub mod total;
pub use total::{Total, TotalF32, TotalF64};
#[cfg(feature = "float_nightly_experimental")]
pub use total::{TotalF16, TotalF128};

pub mod not_nan;
pub use not_nan::{NotNan, NotNanF32, NotNanF64};
#[cfg(feature = "float_nightly_experimental")]
pub use not_nan::{NotNanF16, NotNanF128};
