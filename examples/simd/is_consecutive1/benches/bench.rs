#![feature(portable_simd)]
#![feature(array_chunks)]

use criterion::{black_box, criterion_group, criterion_main, Criterion};
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

type Integer = i32;
const LANES: usize = 64;

pub fn is_consecutive_regular(chunk: &[Integer; LANES]) -> bool {
    for i in 1..LANES {
        if chunk[i - 1].checked_add(1) != Some(chunk[i]) {
            return false;
        }
    }
    true
}

fn simple(c: &mut Criterion) {
    let mut group = c.benchmark_group("simple");
    group.sample_size(1000);

    let v = (100..1_024_000 + 100)
        .map(|i| (i % (Integer::MAX as usize)) as Integer)
        .collect::<Vec<Integer>>();
    let (prefix_s, s, reminder_s) = v.as_simd::<LANES>();
    let v = &v[prefix_s.len()..v.len() - reminder_s.len()];

    group.bench_function(format!("regular,{}", SIMD_SUFFIX), |b| {
        b.iter(|| {
            let _: usize = black_box(
                v.array_chunks::<LANES>()
                    .map(|chunk| is_consecutive_regular(chunk) as usize)
                    .sum(),
            );
        });
    });

    group.bench_function(format!("splat1,{}", SIMD_SUFFIX), |b| {
        b.iter(|| {
            let _: usize = black_box(
                s.iter()
                    .map(|chunk| IsConsecutive::is_consecutive(*chunk) as usize)
                    .sum(),
            );
        });
    });

    group.finish();
}

criterion_group!(benches, simple);
criterion_main!(benches);
