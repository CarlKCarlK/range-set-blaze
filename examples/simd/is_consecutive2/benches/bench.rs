#![feature(portable_simd)]
#![feature(array_chunks)]

use core::any::type_name;
use core::mem;
use criterion::{
    black_box, criterion_group, criterion_main, AxisScale, BenchmarkId, Criterion,
    PlotConfiguration,
};
use is_consecutive2::*;
use std::simd::SimdElement;

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

fn create_benchmark_id<T>(name: &str, lanes: usize, parameter: usize) -> BenchmarkId
where
    T: SimdElement,
{
    BenchmarkId::new(
        format!(
            "{},{},{},{},{}",
            name,
            SIMD_SUFFIX,
            type_name::<T>(),
            mem::size_of::<T>() * 8,
            lanes,
        ),
        parameter,
    )
}

#[inline(never)]
fn vector(c: &mut Criterion) {
    let parameters = [1_024_000, 102_400, 10_240, 1024];

    let mut group = c.benchmark_group("vector");
    group.plot_config(PlotConfiguration::default().summary_scale(AxisScale::Logarithmic));

    group.sample_size(1000);

    for parameter in parameters.iter() {
        let v = (100..parameter + 100)
            .map(|i| (i % (Integer::MAX as usize)) as Integer)
            .collect::<Vec<Integer>>();
        let (prefix_s, s, reminder_s) = v.as_simd::<LANES>();

        // Everyone ignores the prefix and remainder. Everything is aligned.

        let v = &v[prefix_s.len()..v.len() - reminder_s.len()];
        group.bench_function(
            create_benchmark_id::<Integer>("regular", LANES, *parameter),
            |b| {
                b.iter(|| {
                    let _: usize = black_box(
                        // would be better to move array chunking out of the loop
                        v.array_chunks::<LANES>()
                            .map(|chunk| IsConsecutive::is_consecutive_regular(chunk) as usize)
                            .sum(),
                    );
                });
            },
        );

        group.bench_function(
            create_benchmark_id::<Integer>("splat0", LANES, *parameter),
            |b| {
                b.iter(|| {
                    let _: usize = black_box(
                        s.iter()
                            .map(|chunk| IsConsecutive::is_consecutive_splat0(*chunk) as usize)
                            .sum(),
                    );
                });
            },
        );

        group.bench_function(
            create_benchmark_id::<Integer>("splat1", LANES, *parameter),
            |b| {
                b.iter(|| {
                    let _: usize = black_box(
                        s.iter()
                            .map(|chunk| IsConsecutive::is_consecutive_splat1(*chunk) as usize)
                            .sum(),
                    );
                });
            },
        );

        group.bench_function(
            create_benchmark_id::<Integer>("splat2", LANES, *parameter),
            |b| {
                b.iter(|| {
                    let _: usize = black_box(
                        s.iter()
                            .map(|chunk| IsConsecutive::is_consecutive_splat2(*chunk) as usize)
                            .sum(),
                    );
                });
            },
        );

        group.bench_function(
            create_benchmark_id::<Integer>("rotate", LANES, *parameter),
            |b| {
                b.iter(|| {
                    let _: usize = black_box(
                        s.iter()
                            .map(|chunk| IsConsecutive::is_consecutive_rotate(*chunk) as usize)
                            .sum(),
                    );
                });
            },
        );
    }
    group.finish();
}

criterion_group!(benches, vector);
criterion_main!(benches);
