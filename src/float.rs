//! Experimental support for floating point ranges
//! Enable with `total_float_experimental` and `total_float_nightly_experimental`

pub mod total_f32;
pub use total_f32::TotalF32;
pub mod total_f32_int;
pub mod total_f64;
pub use total_f64::TotalF64;
pub mod total_f64_int;

#[cfg(feature = "total_float_nightly_experimental")]
pub mod total_f16;
#[cfg(feature = "total_float_nightly_experimental")]
pub use total_f16::TotalF16;
#[cfg(feature = "total_float_nightly_experimental")]
pub mod total_f16_int;

#[cfg(feature = "total_float_nightly_experimental")]
pub mod total_f128;
#[cfg(feature = "total_float_nightly_experimental")]
pub use total_f128::TotalF128;
#[cfg(feature = "total_float_nightly_experimental")]
pub mod total_f128_int;
