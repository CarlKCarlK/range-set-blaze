#![feature(portable_simd)]

use core::array;
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use is_consecutive::*;
use std::simd::prelude::*;

const SIMD_SUFFIX: &str = if cfg!(target_feature = "avx512f") {
    "512-avx512f"
} else if cfg!(target_feature = "avx2") {
    "256-avx2"
} else if cfg!(target_feature = "sse2") {
    "128-sse2"
} else {
    "error"
};

type IsConsecutiveFn = fn(Simd<u32, LANES>) -> bool;
const FUNCTIONS: [(&str, IsConsecutiveFn); 5] = [
    ("splat0", is_consecutive_splat0 as IsConsecutiveFn),
    ("splat1", is_consecutive_splat1 as IsConsecutiveFn),
    ("splat2", is_consecutive_splat2 as IsConsecutiveFn),
    ("sizzle", is_consecutive_sizzle as IsConsecutiveFn),
    ("rotate", is_consecutive_rotate as IsConsecutiveFn),
];

fn compare_is_consecutive(c: &mut Criterion) {
    let a_array: [u32; LANES] = black_box(array::from_fn(|i| 100 + i as u32));
    let a_simd: Simd<u32, 16> = Simd::from_array(a_array);

    let mut group = c.benchmark_group("compare_is_consecutive");
    group.sample_size(1000);

    group.bench_function(format!("regular,{}", SIMD_SUFFIX), |b| {
        b.iter(|| {
            assert!(black_box(is_consecutive_regular(&a_array)));
        });
    });

    for (name, func) in FUNCTIONS {
        let name = format!("{},{}", name, SIMD_SUFFIX);
        group.bench_function(name, |b| {
            b.iter(|| {
                assert!(black_box(func(a_simd)));
            });
        });
    }

    group.finish();
}

criterion_group!(benches, compare_is_consecutive);
criterion_main!(benches);
