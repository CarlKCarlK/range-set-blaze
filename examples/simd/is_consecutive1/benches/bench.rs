#![feature(portable_simd)]
#![feature(array_chunks)]

use core::simd::prelude::*;
use criterion::{
    black_box, criterion_group, criterion_main, AxisScale, BenchmarkId, Criterion,
    PlotConfiguration,
};
use is_consecutive1::*;

const SIMD_SUFFIX: &str = if cfg!(target_feature = "avx512f") {
    "avx512f,512"
} else if cfg!(target_feature = "avx2") {
    "avx2,256"
} else if cfg!(target_feature = "sse2") {
    "sse2,128"
} else {
    "error"
};

#[cfg(simd_integer = "i8")]
type Integer = i8;
#[cfg(simd_integer = "i16")]
type Integer = i16;
#[cfg(simd_integer = "i32")]
type Integer = i32;
#[cfg(simd_integer = "i64")]
type Integer = i64;
#[cfg(simd_integer = "isize")]
type Integer = isize;
#[cfg(simd_integer = "u8")]
type Integer = u8;
#[cfg(simd_integer = "u16")]
type Integer = u16;
#[cfg(simd_integer = "u32")]
type Integer = u32;
#[cfg(simd_integer = "u64")]
type Integer = u64;
#[cfg(simd_integer = "usize")]
type Integer = usize;
#[cfg(not(any(
    simd_integer = "i8",
    simd_integer = "i16",
    simd_integer = "i32",
    simd_integer = "i64",
    simd_integer = "isize",
    simd_integer = "u8",
    simd_integer = "u16",
    simd_integer = "u32",
    simd_integer = "u64",
    simd_integer = "usize"
)))]
type Integer = i32;

const LANES: usize = if cfg!(simd_lanes = "2") {
    2
} else if cfg!(simd_lanes = "4") {
    4
} else if cfg!(simd_lanes = "8") {
    8
} else if cfg!(simd_lanes = "16") {
    16
} else if cfg!(simd_lanes = "32") {
    32
} else {
    64
};

define_is_consecutive_regular!(is_consecutive_regular, Integer, LANES);
define_is_consecutive_splat0!(is_consecutive_splat0, Integer, LANES);
define_is_consecutive_splat1!(is_consecutive_splat1, Integer, LANES);
define_is_consecutive_splat2!(is_consecutive_splat2, Integer, LANES);
define_is_consecutive_rotate!(is_consecutive_rotate, Integer, LANES);
define_is_consecutive_swizzle!(is_consecutive_swizzle, Integer, LANES);

fn create_benchmark_id(name: &str, n: usize) -> BenchmarkId {
    BenchmarkId::new(
        format!(
            "{},{},{},{},{}",
            name,
            SIMD_SUFFIX,
            option_env!("SIMD_INTEGER").unwrap_or("i32"),
            Integer::BITS,
            LANES,
        ),
        n,
    )
}

#[inline(never)]
fn vector(c: &mut Criterion) {
    let ns = [1_024_000, 102_400, 10_240, 1024];

    let mut group = c.benchmark_group("vector");
    group.plot_config(PlotConfiguration::default().summary_scale(AxisScale::Logarithmic));

    group.sample_size(1000);

    for n in ns.iter() {
        let v = (100..n + 100)
            .map(|i| (i % (Integer::MAX as usize)) as Integer)
            .collect::<Vec<Integer>>();
        let (prefix_s, s, reminder_s) = v.as_simd::<LANES>();

        // Everyone ignores the prefix and remainder. Everything is aligned.

        let v = &v[prefix_s.len()..v.len() - reminder_s.len()];
        group.bench_function(create_benchmark_id("regular", *n), |b| {
            b.iter(|| {
                let _: usize = black_box(
                    v.array_chunks::<LANES>()
                        .map(|chunk| is_consecutive_regular(chunk) as usize)
                        .sum(),
                );
            });
        });

        group.bench_function(create_benchmark_id("splat0", *n), |b| {
            b.iter(|| {
                let _: usize = black_box(
                    s.iter()
                        .map(|chunk| is_consecutive_splat0(*chunk) as usize)
                        .sum(),
                );
            });
        });

        group.bench_function(create_benchmark_id("splat1", *n), |b| {
            b.iter(|| {
                let _: usize = black_box(
                    s.iter()
                        .map(|chunk| is_consecutive_splat1(*chunk) as usize)
                        .sum(),
                );
            });
        });

        group.bench_function(create_benchmark_id("splat2", *n), |b| {
            b.iter(|| {
                let _: usize = black_box(
                    s.iter()
                        .map(|chunk| is_consecutive_splat2(*chunk) as usize)
                        .sum(),
                );
            });
        });

        group.bench_function(create_benchmark_id("rotate", *n), |b| {
            b.iter(|| {
                let _: usize = black_box(
                    s.iter()
                        .map(|chunk| is_consecutive_rotate(*chunk) as usize)
                        .sum(),
                );
            });
        });

        group.bench_function(create_benchmark_id("swizzle", *n), |b| {
            b.iter(|| {
                let _: usize = black_box(
                    s.iter()
                        .map(|chunk| is_consecutive_swizzle(*chunk) as usize)
                        .sum(),
                );
            });
        });
    }

    group.finish();
}

criterion_group!(benches, vector);
criterion_main!(benches);