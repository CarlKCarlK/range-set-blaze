#![feature(portable_simd)]

use core::array;
use core::simd::{prelude::*, LaneCount, SupportedLaneCount};
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use is_consecutive1::*;

const SIMD_SUFFIX: &str = if cfg!(target_feature = "avx512f") {
    "512-avx512f"
} else if cfg!(target_feature = "avx2") {
    "256-avx2"
} else if cfg!(target_feature = "sse2") {
    "128-sse2"
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

type IsConsecutiveFn = fn(Simd<u32, LANES>, Simd<u32, LANES>) -> bool;
const FUNCTIONS: [(&str, IsConsecutiveFn); 2] = [
    // cmk ("splat0", is_consecutive_splat0 as IsConsecutiveFn),
    ("splat1", is_consecutive_splat1 as IsConsecutiveFn),
    ("splat2", is_consecutive_splat2 as IsConsecutiveFn),
    // cmk ("sizzle", is_consecutive_sizzle as IsConsecutiveFn),
    // cmk ("rotate", is_consecutive_rotate as IsConsecutiveFn),
];

reference_splat!(reference_splat_u32, u32);

fn compare_is_consecutive(c: &mut Criterion) {
    let a_array: [u32; LANES] = black_box(array::from_fn(|i| 100 + i as u32));
    let a_simd: Simd<u32, LANES> = Simd::from_array(a_array);

    let mut group = c.benchmark_group("compare_is_consecutive");
    group.sample_size(1000);

    group.bench_function(format!("regular,{},{}", SIMD_SUFFIX, LANES), |b| {
        b.iter(|| {
            assert!(black_box(is_consecutive_regular(&a_array, 1, u32::MAX)));
        });
    });

    for (name, func) in FUNCTIONS {
        let name = format!("{},{},{}", name, SIMD_SUFFIX, LANES);
        group.bench_function(name, |b| {
            b.iter(|| {
                assert!(black_box(func(a_simd, reference_splat_u32())));
            });
        });
    }

    group.finish();
}

criterion_group!(benches, compare_is_consecutive);
criterion_main!(benches);
