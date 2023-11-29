use std::env;

fn main() {
    if let Ok(simd_lanes) = env::var("SIMD_LANES") {
        println!("cargo:rustc-cfg=simd_lanes=\"{}\"", simd_lanes);
        println!("cargo:rerun-if-env-changed=SIMD_LANES");
    }
    if let Ok(simd_integer) = env::var("SIMD_INTEGER") {
        println!("cargo:rustc-cfg=simd_integer=\"{}\"", simd_integer);
        println!("cargo:rerun-if-env-changed=SIMD_INTEGER");
    }
}
