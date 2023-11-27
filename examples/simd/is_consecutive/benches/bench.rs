#![feature(portable_simd)]

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use is_consecutive::*;

fn compare_is_consecutive(c: &mut Criterion) {
    use core::array;

    let mut group = c.benchmark_group("compare_is_consecutive");
    group.sample_size(1000);

    let a: [u32; LANES] = black_box(array::from_fn(|i| 100 + i as u32));
    let ninety_nines: [u32; LANES] = black_box([99; LANES]);

    group.bench_function("regular", |b| {
        b.iter(|| {
            black_box(is_consecutive_regular(&a));
            black_box(is_consecutive_regular(&ninety_nines));
        });
    });

    use std::simd::prelude::*;
    let a = Simd::from_array(a);
    let ninety_nines = Simd::from_array(ninety_nines);

    group.bench_function("splat0", |b| {
        b.iter(|| {
            black_box(is_consecutive_splat0(a));
            black_box(is_consecutive_splat0(ninety_nines));
        });
    });

    group.bench_function("splat1", |b| {
        b.iter(|| {
            black_box(is_consecutive_splat1(a));
            black_box(is_consecutive_splat1(ninety_nines));
        });
    });

    group.bench_function("splat2", |b| {
        b.iter(|| {
            black_box(is_consecutive_splat2(a));
            black_box(is_consecutive_splat2(ninety_nines));
        });
    });

    group.bench_function("sizzle", |b| {
        b.iter(|| {
            black_box(is_consecutive_sizzle(a));
            black_box(is_consecutive_sizzle(ninety_nines));
        });
    });

    group.bench_function("rotate", |b| {
        b.iter(|| {
            black_box(is_consecutive_rotate(a));
            black_box(is_consecutive_rotate(ninety_nines));
        });
    });

    group.finish();
}

criterion_group!(benches, compare_is_consecutive);
criterion_main!(benches);
