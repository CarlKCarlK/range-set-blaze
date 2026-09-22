//! This build script reads the `BUILDFEATURES` environment variable and sets
use std::env;

fn main() {
    if let Ok(features) = env::var("BUILDFEATURES") {
        for feature in features.split(',') {
            println!("cargo:rustc-cfg=feature=\"{}\"", feature.trim());
            println!("cargo:rerun-if-env-changed=BUILDFEATURES");
        }
    }

    println!("cargo:rerun-if-env-changed=RANGE_SET_BLAZE_CUDA_ARCH");
    println!("cargo:rerun-if-env-changed=CUDA_HOME");
    println!("cargo:rerun-if-env-changed=DOCS_RS");
    println!("cargo:rerun-if-changed=native/cub_sort.cu");
    if env::var_os("CARGO_FEATURE_GPU_CUB").is_some() && env::var_os("DOCS_RS").is_none() {
        build_cub_sort();
    }
}

fn build_cub_sort() {
    let cuda_home = env::var("CUDA_HOME").unwrap_or_else(|_| "/usr/local/cuda".to_owned());
    let architecture = env::var("RANGE_SET_BLAZE_CUDA_ARCH").unwrap_or_else(|_| "sm_86".to_owned());
    let cccl_include = format!("{cuda_home}/targets/x86_64-linux/include/cccl");

    cc::Build::new()
        .cuda(true)
        .cudart("shared")
        .cpp(true)
        .file("native/cub_sort.cu")
        .include(cccl_include)
        .flag("-std=c++17")
        .flag(format!("-arch={architecture}"))
        .compile("range_set_blaze_cub_sort");
}
