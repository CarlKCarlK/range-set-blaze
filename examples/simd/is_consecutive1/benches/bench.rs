#![feature(portable_simd)]
#![feature(array_chunks)]
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use is_consecutive1::*;

// create a string from the SIMD extension used
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

// compare against this
#[inline]
pub fn is_consecutive_regular(chunk: &[Integer; LANES]) -> bool {
    for i in 1..LANES {
        if chunk[i - 1].checked_add(1) != Some(chunk[i]) {
            return false;
        }
    }
    true
}

// define a benchmark called "simple"
fn simple(c: &mut Criterion) {
    let mut group = c.benchmark_group("simple");
    group.sample_size(1000);

    // generate about 1 million aligned elements
    let parameter: Integer = 1_024_000;
    let v = (100..parameter + 100).collect::<Vec<_>>();
    let (prefix, simd_chunks, reminder) = v.as_simd::<LANES>(); // keep aligned part
    let v = &v[prefix.len()..v.len() - reminder.len()]; // keep aligned part

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
                simd_chunks
                    .iter()
                    .map(|chunk| IsConsecutive::is_consecutive(*chunk) as usize)
                    .sum(),
            );
        });
    });

    group.finish();
}

criterion_group!(benches, simple);
criterion_main!(benches);
