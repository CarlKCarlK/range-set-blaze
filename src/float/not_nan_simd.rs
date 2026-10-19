//! SIMD kernels that check (and normalize) primitive `f32`/`f64` slices for [`NotNan`], on stable
//! Rust via [`fearless_simd`].
//!
//! They work on the floats' bits rather than with float comparisons: a value is NaN exactly when
//! its bits without the sign exceed infinity's, and `-0.0` is a single bit pattern. (Written with
//! float operations, LLVM merges the two checks into a floating-point classification that it then
//! doesn't vectorize.) As in [`crate::from_slice`], all SIMD work is `#[inline(always)]` code
//! inside the [`dispatch!`] closure.
//!
//! [`NotNan`]: super::not_nan::NotNan

/// How many SIMD vectors the kernels check per branch.
const UNROLL: usize = 4;

macro_rules! impl_not_nan_simd {
    ($module:ident: $float:ty, $bits:ty, $vector:ident) => {
    #[doc = concat!("Kernels for `", stringify!($float), "`.")]
    pub(super) mod $module {
        use fearless_simd::{Level, dispatch, prelude::*};
        use super::UNROLL;

        /// Returns `true` if no value is NaN or `-0.0`.
        pub(in crate::float) fn all_valid(values: &[$float]) -> bool {
            all_valid_at_level(Level::try_detect().unwrap_or(Level::baseline()), values)
        }

        /// Like [`all_valid`], but at a given SIMD `level`.
        pub(in crate::float) fn all_valid_at_level(level: Level, values: &[$float]) -> bool {
            /// The bits other than the sign bit.
            const ABS_MASK: $bits = <$bits>::MAX >> 1;
            const INFINITY: $bits = <$float>::INFINITY.to_bits();
            const NEG_ZERO: $bits = (-0.0 as $float).to_bits();

            #[inline(always)]
            fn kernel<S: Simd>(simd: S, bits: &[$bits]) -> bool {
                let lanes = fearless_simd::$vector::<S>::LEN;
                let abs_mask = fearless_simd::$vector::<S>::splat(simd, ABS_MASK);
                let infinity = fearless_simd::$vector::<S>::splat(simd, INFINITY);
                let neg_zero = fearless_simd::$vector::<S>::splat(simd, NEG_ZERO);
                let mut blocks = bits.chunks_exact(lanes * UNROLL);
                for block in &mut blocks {
                    let mut valid = true;
                    for chunk in block.chunks_exact(lanes) {
                        let values = fearless_simd::$vector::<S>::from_slice(simd, chunk);
                        valid &= (values & abs_mask).simd_le(infinity).all_true()
                            & !values.simd_eq(neg_zero).any_true();
                    }
                    if !valid {
                        return false;
                    }
                }
                blocks
                    .remainder()
                    .iter()
                    .all(|&bits| bits & ABS_MASK <= INFINITY && bits != NEG_ZERO)
            }

            let bits: &[$bits] = bytemuck::cast_slice(values);
            dispatch!(level, simd => kernel(simd, bits))
        }

        /// Replaces every `-0.0` with `+0.0`, in place, and returns `true` if no value is NaN.
        /// (When it returns `false`, values after the first NaN may not have been normalized.)
        pub(in crate::float) fn normalize(values: &mut [$float]) -> bool {
            normalize_at_level(Level::try_detect().unwrap_or(Level::baseline()), values)
        }

        /// Like [`normalize`], but at a given SIMD `level`.
        pub(in crate::float) fn normalize_at_level(level: Level, values: &mut [$float]) -> bool {
            const ABS_MASK: $bits = <$bits>::MAX >> 1;
            const INFINITY: $bits = <$float>::INFINITY.to_bits();
            const NEG_ZERO: $bits = (-0.0 as $float).to_bits();

            #[inline(always)]
            fn kernel<S: Simd>(simd: S, bits: &mut [$bits]) -> bool {
                let lanes = fearless_simd::$vector::<S>::LEN;
                let abs_mask = fearless_simd::$vector::<S>::splat(simd, ABS_MASK);
                let infinity = fearless_simd::$vector::<S>::splat(simd, INFINITY);
                let neg_zero = fearless_simd::$vector::<S>::splat(simd, NEG_ZERO);
                let zero = fearless_simd::$vector::<S>::splat(simd, 0);
                let mut blocks = bits.chunks_exact_mut(lanes * UNROLL);
                for block in &mut blocks {
                    let mut not_nan = true;
                    for chunk in block.chunks_exact_mut(lanes) {
                        let values = fearless_simd::$vector::<S>::from_slice(simd, chunk);
                        not_nan &= (values & abs_mask).simd_le(infinity).all_true();
                        // Write back only chunks holding a `-0.0`, so clean data is just read.
                        let is_neg_zero = values.simd_eq(neg_zero);
                        if is_neg_zero.any_true() {
                            is_neg_zero.select(zero, values).store_slice(chunk);
                        }
                    }
                    if !not_nan {
                        return false;
                    }
                }
                for bits in blocks.into_remainder() {
                    if *bits & ABS_MASK > INFINITY {
                        return false;
                    }
                    if *bits == NEG_ZERO {
                        *bits = 0;
                    }
                }
                true
            }

            let bits: &mut [$bits] = bytemuck::cast_slice_mut(values);
            dispatch!(level, simd => kernel(simd, bits))
        }
    }
    };
}

impl_not_nan_simd!(f64: f64, u64, u64x8);
impl_not_nan_simd!(f32: f32, u32, u32x16);

#[cfg(test)]
mod tests {
    use crate::from_slice::testable_levels;
    use alloc::vec::Vec;

    macro_rules! test_type {
        ($test:ident, $module:ident: $float:ty, $bits:ty) => {
            #[test]
            fn $test() {
                use super::$module::{all_valid_at_level, normalize_at_level};

                let nans: [$float; 4] = [
                    <$float>::NAN,
                    -<$float>::NAN,
                    // Smallest NaN payloads, positive and negative.
                    <$float>::from_bits(<$float>::INFINITY.to_bits() + 1),
                    <$float>::from_bits(<$float>::NEG_INFINITY.to_bits() + 1),
                ];
                let valid: [$float; 9] = [
                    0.0,
                    1.5,
                    -2.25,
                    <$float>::INFINITY,
                    <$float>::NEG_INFINITY,
                    <$float>::MAX,
                    <$float>::MIN,
                    <$float>::from_bits(1),  // smallest subnormal
                    -<$float>::from_bits(1), // smallest negative subnormal
                ];
                for level in testable_levels() {
                    for len in (0..=140).chain([1000]) {
                        let values: Vec<$float> =
                            (0..len).map(|index| valid[index % valid.len()]).collect();
                        assert!(all_valid_at_level(level, &values), "{level:?} {len}");
                        let mut normalized = values.clone();
                        assert!(normalize_at_level(level, &mut normalized));
                        assert_eq!(
                            normalized
                                .iter()
                                .map(|x| x.to_bits())
                                .collect::<Vec<$bits>>(),
                            values.iter().map(|x| x.to_bits()).collect::<Vec<$bits>>()
                        );
                        for position in 0..len {
                            // `-0.0` fails validation, and normalizes to `+0.0`.
                            let mut with_neg_zero = values.clone();
                            with_neg_zero[position] = -0.0;
                            assert!(
                                !all_valid_at_level(level, &with_neg_zero),
                                "{level:?} {len} {position}"
                            );
                            assert!(normalize_at_level(level, &mut with_neg_zero));
                            assert_eq!(
                                with_neg_zero[position].to_bits(),
                                (0.0 as $float).to_bits()
                            );
                            for (index, (x, y)) in with_neg_zero.iter().zip(&values).enumerate() {
                                if index != position {
                                    assert_eq!(x.to_bits(), y.to_bits());
                                }
                            }
                            // Every kind of NaN fails both.
                            for nan in nans {
                                let mut with_nan = values.clone();
                                with_nan[position] = nan;
                                assert!(
                                    !all_valid_at_level(level, &with_nan),
                                    "{level:?} {len} {position} {nan}"
                                );
                                assert!(
                                    !normalize_at_level(level, &mut with_nan),
                                    "{level:?} {len} {position} {nan}"
                                );
                            }
                        }
                    }
                }
            }
        };
    }

    test_type!(f64_kernels_match_scalar, f64: f64, u64);
    test_type!(f32_kernels_match_scalar, f32: f32, u32);
}
