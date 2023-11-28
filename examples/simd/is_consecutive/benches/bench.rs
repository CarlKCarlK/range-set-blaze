#![feature(portable_simd)]

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use is_consecutive::*;
use std::simd::prelude::*;

// Define the function type signature
type IsConsecutiveFn = fn(Simd<u32, LANES>) -> bool;

// List of functions and their names
const FUNCTIONS: [(&str, IsConsecutiveFn); 5] = [
    ("splat0", is_consecutive_splat0 as IsConsecutiveFn),
    ("splat1", is_consecutive_splat1 as IsConsecutiveFn),
    ("splat2", is_consecutive_splat2 as IsConsecutiveFn),
    ("sizzle", is_consecutive_sizzle as IsConsecutiveFn),
    ("rotate", is_consecutive_rotate as IsConsecutiveFn),
];

const SIMD_SUFFIX: &str = if cfg!(target_feature = "avx512f") {
    "avx512f"
} else if cfg!(target_feature = "avx2") {
    "avx2"
} else if cfg!(target_feature = "sse2") {
    "sse2"
} else {
    "error"
};

fn compare_is_consecutive(c: &mut Criterion) {
    use core::array;

    let mut group = c.benchmark_group("compare_is_consecutive");
    group.sample_size(1000);

    let a: [u32; LANES] = black_box(array::from_fn(|i| 100 + i as u32));
    let ninety_nines: [u32; LANES] = black_box([99; LANES]);

    group.bench_function(format!("regular,{}", SIMD_SUFFIX), |b| {
        b.iter(|| {
            assert!(black_box(is_consecutive_regular(&a)));
            // black_box(is_consecutive_regular(&ninety_nines));
        });
    });

    use std::simd::prelude::*;
    let a = Simd::from_array(a);
    let ninety_nines = Simd::from_array(ninety_nines);

    for (name, func) in FUNCTIONS {
        let name = format!("{},{}", name, SIMD_SUFFIX);
        group.bench_function(name, |b| {
            b.iter(|| {
                assert!(black_box(func(a)));
            });
        });
    }

    group.finish();
}

criterion_group!(benches, compare_is_consecutive);
criterion_main!(benches);
