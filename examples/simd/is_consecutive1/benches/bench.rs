#![feature(portable_simd)]
#![feature(array_chunks)]

use core::simd::{prelude::*, LaneCount, SupportedLaneCount};
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
} else if cfg!(simd_lanes = "64") {
    64
} else {
    64
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

reference_splat0!(reference_splat0_integer, Integer);
reference_splat!(reference_splat_integer, Integer);
reference_rotate!(reference_rotate_integer, Integer);

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
        let v = &v[prefix_s.len()..v.len() - reminder_s.len()];

        // Everyone ignores the prefix and remainder. Everything is aligned.

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
                        .map(|chunk| {
                            is_consecutive_splat0(
                                *chunk,
                                reference_splat0_integer(),
                                (LANES - 1) as Integer,
                            ) as usize
                        })
                        .sum(),
                );
            });
        });

        group.bench_function(create_benchmark_id("splat1", *n), |b| {
            b.iter(|| {
                let _: usize = black_box(
                    s.iter()
                        .map(|chunk| {
                            is_consecutive_splat1(*chunk, reference_splat_integer()) as usize
                        })
                        .sum(),
                );
            });
        });

        group.bench_function(create_benchmark_id("splat2", *n), |b| {
            b.iter(|| {
                let _: usize = black_box(
                    s.iter()
                        .map(|chunk| {
                            is_consecutive_splat2(*chunk, reference_splat_integer()) as usize
                        })
                        .sum(),
                );
            });
        });

        group.bench_function(create_benchmark_id("rotate", *n), |b| {
            b.iter(|| {
                let _: usize = black_box(
                    s.iter()
                        .map(|chunk| {
                            is_consecutive_rotate(*chunk, reference_rotate_integer()) as usize
                        })
                        .sum(),
                );
            });
        });
    }

    group.finish();
}

criterion_group!(benches, vector);
criterion_main!(benches);
