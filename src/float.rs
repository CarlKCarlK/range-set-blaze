//! Experimental support for floating point ranges
//! Enable with `total_float_experimental` (stable, `f32`/`f64`) and
//! `total_float_nightly_experimental` (nightly, adds `f16`/`f128`).

pub mod total;
pub use total::{Total, TotalF32, TotalF64};
#[cfg(feature = "total_float_nightly_experimental")]
pub use total::TotalF16;

pub mod finite;
