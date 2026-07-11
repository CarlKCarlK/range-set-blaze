//! One can approximate cosine with the Taylor series
//! `cos(x) ≈ 1 - x^2/2! + x^4/4! - x^6/6! + x^8/8! - ...`.
//!
//! But how many terms are actually needed? This example sweeps every finite
//! `f32` value in `[-pi, pi]` (~2.16 billion values), computes the smallest
//! number of Taylor terms needed to reach
//! `TARGET_ERROR` (1e-7), and tabulates the results in a
//! `RangeMapBlaze<FiniteF32, u8>`.
//!
//! Surprisingly, most representable `f32` values in this interval need only
//! one term (`1`) or two terms (`1 - x*x/2`). The resulting `RangeMapBlaze`
//! acts as a compact dispatch table: a lookup tells you how many Taylor terms to
//! evaluate for each input range. Averaged uniformly over representable `f32`
//! values in `[-pi, pi]`, only about 1.2 terms are needed.
//!
//! Aside: Real math libraries go even further. They first reduce the input to a
//! smaller interval (using cosine's periodicity and symmetry) and then use
//! carefully optimized minimax polynomials rather than a Taylor series.
//! This example intentionally uses the simpler Taylor series so the focus
//! remains on how `RangeMapBlaze` can discover and represent an compact
//! dispatch table.

use core::f32::consts::PI;
use range_set_blaze::{
    FiniteF32, Integer, RangeMapBlaze, RangeSetBlaze,
    finite::{FiniteRangeExt, ff32},
};
use rayon::prelude::*;
use std::{num::NonZero, ops::BitOr, thread::available_parallelism};

const TARGET_ERROR: f64 = 1e-7;

/// Expected output:
///
/// ```text
///      Running `target/release/examples/float_maps`
/// Taylor-series terms needed for cos(x) to guarantee an error under one f32 epsilon, for f32 in [-pi, pi]:
/// (target absolute error: 0.0000001; taylor/std may still disagree in the last printed digit -- display-rounding, not a bug)
///   [  -3.1415927e0,   -3.0849006e0] -> 10 term(s)  (cos(mid): taylor=-0.9995982, std=-0.9995983)
///   [  -3.0849004e0,   -2.4833546e0] -> 9 term(s)  (cos(mid): taylor=-0.9367867, std=-0.9367868)
///   [  -2.4833543e0,   -1.9118674e0] -> 8 term(s)  (cos(mid): taylor=-0.5865679, std=-0.5865678)
///   [  -1.9118673e0,   -1.3804736e0] -> 7 term(s)  (cos(mid): taylor=-0.0753027, std=-0.0753027)
///   [  -1.3804735e0,   -9.036002e-1] -> 6 term(s)  (cos(mid): taylor=0.4157428, std=0.4157428)
///   [ -9.0360016e-1,  -5.0198424e-1] -> 5 term(s)  (cos(mid): taylor=0.7630404, std=0.7630404)
///   [  -5.019842e-1,   -2.039649e-1] -> 4 term(s)  (cos(mid): taylor=0.9383486, std=0.9383486)
///   [ -2.0396489e-1,  -3.9359797e-2] -> 3 term(s)  (cos(mid): taylor=0.9926082, std=0.9926082)
///   [ -3.9359793e-2,  -4.4721362e-4] -> 2 term(s)  (cos(mid): taylor=0.9998019, std=0.9998019)
///   [  -4.472136e-4,    4.472136e-4] -> 1 term(s)  (cos(mid): taylor=1.0000000, std=1.0000000)
///   [  4.4721362e-4,   3.9359793e-2] -> 2 term(s)  (cos(mid): taylor=0.9998019, std=0.9998019)
///   [  3.9359797e-2,   2.0396489e-1] -> 3 term(s)  (cos(mid): taylor=0.9926082, std=0.9926082)
///   [   2.039649e-1,    5.019842e-1] -> 4 term(s)  (cos(mid): taylor=0.9383486, std=0.9383486)
///   [  5.0198424e-1,   9.0360016e-1] -> 5 term(s)  (cos(mid): taylor=0.7630404, std=0.7630404)
///   [   9.036002e-1,    1.3804735e0] -> 6 term(s)  (cos(mid): taylor=0.4157428, std=0.4157428)
///   [   1.3804736e0,    1.9118673e0] -> 7 term(s)  (cos(mid): taylor=-0.0753027, std=-0.0753027)
///   [   1.9118674e0,    2.4833543e0] -> 8 term(s)  (cos(mid): taylor=-0.5865679, std=-0.5865678)
///   [   2.4833546e0,    3.0849004e0] -> 9 term(s)  (cos(mid): taylor=-0.9367867, std=-0.9367868)
///   [   3.0849006e0,    3.1415927e0] -> 10 term(s)  (cos(mid): taylor=-0.9995982, std=-0.9995983)
///
/// 19 disjoint ranges cover all 2_157_060_023 in-scope f32 values.
/// Mean terms per in-scope f32 value (each value weighted equally): 1.23
/// ```
fn main() {
    let scope = RangeSetBlaze::from_iter([ff32(-PI)..=ff32(PI)]);

    // Split scope across all available cores; each thread sweeps its own
    // chunk into a local RangeMapBlaze, then union (also known as `|` and `bitor`)
    // merges them --this is cheap because the # of ranges in each chunk turns out to be small.
    let num_chunks = available_parallelism().map_or(1, NonZero::get);
    let term_map: RangeMapBlaze<FiniteF32, u8> = chunks(&scope, num_chunks)
        .into_par_iter()
        .map(|chunk| chunk.iter().map(|x| (x, terms_needed(x))).collect())
        .reduce(RangeMapBlaze::new, BitOr::bitor); // `bitor` is a very efficient union that exploits ownership.

    println!(
        "Taylor-series terms needed for cos(x) to guarantee an error under one f32 epsilon, for f32 in [-pi, pi]:\n\
         (target absolute error: {TARGET_ERROR}; taylor/std may still disagree in the last printed digit -- display-rounding, not a bug)"
    );
    for (range, n) in term_map.range_values() {
        let (start, end) = range.into_primitive_inner();
        let mid = f32::midpoint(start, end);
        let taylor = taylor_cos(mid, *n);
        let std = mid.cos();
        println!(
            "  [{start:>14e}, {end:>14e}] -> {n} term(s)  (cos(mid): taylor={taylor:.7}, std={std:.7})",
        );
    }
    println!(
        "\n{} disjoint ranges cover all {} in-scope f32 values.",
        separate_with_underscores(&term_map.range_values().count()),
        separate_with_underscores(&term_map.len())
    );
    println!(
        "Mean terms per in-scope f32 value (each value weighted equally): {:.2}",
        mean_terms(&term_map)
    );
}

fn separate_with_underscores(value: &impl ToString) -> String {
    let mut value = value.to_string();
    let mut index = value.len();
    while index > 3 {
        index -= 3;
        value.insert(index, '_');
    }
    value
}

/// Smallest `N` (1..=`u8::MAX`) for which the remainder bound
/// `|x|^(2N) / (2N)!` guarantees error under `TARGET_ERROR`. Assumes `x`
/// is already in scope.
fn terms_needed(x: FiniteF32) -> u8 {
    let x = f64::from(x.into_inner());
    let xx = x * x;
    let mut term_magnitude = 1.0_f64; // |x|^0 / 0!
    for n in 1..=u8::MAX {
        term_magnitude *= xx / f64::from(2 * u32::from(n) * (2 * u32::from(n) - 1));
        if term_magnitude <= TARGET_ERROR {
            return n;
        }
    }
    panic!("u8::MAX terms not enough for x={x}")
}

/// Splits `scope` into `n` nearly equal contiguous chunks.
fn chunks(scope: &RangeSetBlaze<FiniteF32>, n: usize) -> Vec<RangeSetBlaze<FiniteF32>> {
    let Some(mut start) = scope.first() else {
        return Vec::new();
    };
    let n = u32::try_from(n).unwrap_or(u32::MAX).clamp(1, scope.len());
    let (base_len, remainder) = (scope.len() / n, scope.len() % n);

    (0..n)
        .map(|i| {
            let len = base_len + u32::from(i < remainder);
            let end = start.inclusive_end_from_start(len);
            let chunk = RangeSetBlaze::from_iter([start..=end]);
            start = end.after();
            chunk
        })
        .collect()
}

/// `cos(x)` via `terms` terms of its Taylor series, for display purposes
/// only -- `terms_needed` never evaluates the series itself.
// `2 * k * (2 * k - 1)` is at most `2 * 255 * 509 = 259_590` (`k` is `u8`), well within
// `f32`'s 24-bit mantissa, so the `as f32` below is exact, not lossy.
#[allow(clippy::cast_precision_loss)]
fn taylor_cos(x: f32, terms: u8) -> f32 {
    let xx = x * x;
    let mut term = 1.0_f32;
    let mut sum = 1.0_f32;
    for k in 1..terms {
        term *= -xx / (2 * u32::from(k) * (2 * u32::from(k) - 1)) as f32;
        sum += term;
    }
    sum
}

/// Mean term count across in-scope `f32` values, each counted once (not an
/// average over the reals, which would over-weight sparse magnitudes).
/// Exact: weights each range's term count by its `Integer::safe_len`.
fn mean_terms(term_map: &RangeMapBlaze<FiniteF32, u8>) -> f64 {
    let mut weighted_sum = 0.0;
    let mut total_count = 0.0;
    for (range, n) in term_map.range_values() {
        let len = FiniteF32::safe_len_to_f64_lossy(FiniteF32::safe_len(&range));
        weighted_sum = len.mul_add(f64::from(*n), weighted_sum);
        total_count += len;
    }
    weighted_sum / total_count
}
