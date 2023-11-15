// #![feature(portable_simd)]
#![allow(dead_code)]

use syntactic_for::syntactic_for;

// use std::simd::f32x16;
// use std::simd::f32x4;

// fn sample1() {
//     let a = f32x4::splat(10.0);
//     let b = f32x4::from_array([1.0, 2.0, 3.0, 4.0]);
//     println!("{:?}", a + b);
// }

fn sample2() {
    println!("feature\tcould\tare");
    syntactic_for! { feature in [
        "aes",
        "pclmulqdq",
        "rdrand",
        "rdseed",
        "tsc",
        "mmx",
        "sse",
        "sse2",
        "sse3",
        "ssse3",
        "sse4.1",
        "sse4.2",
        "sse4a",
        "sha",
        "avx",
        "avx2",
        "avx512f",
        "avx512cd",
        "avx512er",
        "avx512pf",
        "avx512bw",
        "avx512dq",
        "avx512vl",
        "avx512ifma",
        "avx512vbmi",
        "avx512vpopcntdq",
        "fma",
        "bmi1",
        "bmi2",
        "abm",
        "lzcnt",
        "tbm",
        "popcnt",
        "fxsr",
        "xsave",
        "xsaveopt",
        "xsaves",
        "xsavec",
        ] {$(
            println!("{}\t{}\t{}",$feature,is_x86_feature_detected!($feature),cfg!(target_feature = $feature));

    )*}};
}

// fn sample_avx512f() {
//     sample2();
//     let a = f32x16::splat(10.0);
//     let b = f32x16::from([
//         1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 11.0, 12.0, 13.0, 14.0, 15.0, 16.0,
//     ]);
//     println!("{:?}", a + b);
// }

fn main() {
    sample2();
}
