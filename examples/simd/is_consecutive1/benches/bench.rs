#![feature(portable_simd)]
#![feature(array_chunks)]

use core::array;
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
    0
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

type IsConsecutiveFn = fn(Simd<Integer, LANES>, Simd<Integer, LANES>) -> bool;
const FUNCTIONS: [(&str, IsConsecutiveFn); 2] = [
    // cmk ("splat0", is_consecutive_splat0 as IsConsecutiveFn),
    ("splat1", is_consecutive_splat1 as IsConsecutiveFn),
    ("splat2", is_consecutive_splat2 as IsConsecutiveFn),
    // cmk ("sizzle", is_consecutive_sizzle as IsConsecutiveFn),
    // cmk ("rotate", is_consecutive_rotate as IsConsecutiveFn),
];

reference_splat!(reference_splat, Integer);

fn compare_is_consecutive(c: &mut Criterion) {
    let a_array: [Integer; LANES] = black_box(array::from_fn(|i| 100 + i as Integer));
    let a_simd: Simd<Integer, LANES> = Simd::from_array(a_array);

    let mut group = c.benchmark_group("compare_is_consecutive");
    group.sample_size(1000);

    group.bench_function(
        format!(
            "regular,{},{},{},{}",
            SIMD_SUFFIX,
            env!("SIMD_INTEGER"),
            Integer::BITS,
            LANES,
        ),
        |b| {
            b.iter(|| {
                assert!(black_box(is_consecutive_regular(&a_array, 1, Integer::MAX)));
            });
        },
    );

    for (name, func) in FUNCTIONS {
        let name = format!(
            "{},{},{},{},{}",
            name,
            SIMD_SUFFIX,
            env!("SIMD_INTEGER"),
            Integer::BITS,
            LANES,
        );
        group.bench_function(name, |b| {
            b.iter(|| {
                assert!(black_box(func(a_simd, reference_splat())));
            });
        });
    }

    group.finish();
}

fn vector(c: &mut Criterion) {
    let ns = [100usize, 1000, 10_000, 100_000, 1_000_000];

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

        group.bench_function(
            BenchmarkId::new(
                format!(
                    "regular,{},{},{},{}",
                    SIMD_SUFFIX,
                    env!("SIMD_INTEGER"),
                    Integer::BITS,
                    LANES,
                ),
                n,
            ),
            |b| {
                b.iter(|| {
                    black_box(
                        v.array_chunks::<LANES>()
                            .all(|chunk| is_consecutive_regular(&chunk, 1, Integer::MAX)),
                    );
                });
            },
        );

        for (name, func) in FUNCTIONS {
            let id = BenchmarkId::new(
                format!(
                    "{},{},{},{},{}",
                    name,
                    SIMD_SUFFIX,
                    env!("SIMD_INTEGER"),
                    Integer::BITS,
                    LANES,
                ),
                n,
            );
            group.bench_function(id, |b| {
                b.iter(|| {
                    black_box(s.iter().all(|chunk| func(*chunk, reference_splat())));
                    // cmk we ignore the remainder
                });
            });
        }
    }

    group.finish();
}

criterion_group!(benches, compare_is_consecutive, vector);
criterion_main!(benches);
