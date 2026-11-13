//! This build script reads the `BUILDFEATURES` environment variable and sets
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

const OFFLOAD_SOURCES: [&str; 5] = [
    "native.rs",
    "resident.rs",
    "radix.rs",
    "device.rs",
    "ptx.rs",
];

fn main() {
    if let Ok(features) = env::var("BUILDFEATURES") {
        for feature in features.split(',') {
            println!("cargo:rustc-cfg=feature=\"{}\"", feature.trim());
            println!("cargo:rerun-if-env-changed=BUILDFEATURES");
        }
    }
    if env::var_os("CARGO_FEATURE_OFFLOAD_NIGHTLY_EXPERIMENTAL").is_some() {
        build_offload();
    }
}

fn run(command: &mut Command) {
    assert!(
        command
            .status()
            .expect("could not start offload compiler")
            .success(),
        "offload compilation failed; see docs/offload.md for setup instructions"
    );
}

fn build_offload() {
    // Documentation builds expose the API without compiling or linking native kernels.
    println!("cargo:rerun-if-env-changed=DOCS_RS");
    if env::var_os("DOCS_RS").is_some() {
        return;
    }
    assert_eq!(
        env::var("HOST").expect("Cargo must set HOST"),
        env::var("TARGET").expect("Cargo must set TARGET"),
        "the offload backend currently requires a native host build"
    );
    assert_eq!(
        env::var("CARGO_CFG_TARGET_OS").expect("Cargo must set CARGO_CFG_TARGET_OS"),
        "linux",
        "the offload backend currently supports Linux hosts only"
    );
    println!("cargo:rerun-if-env-changed=RANGE_SET_BLAZE_OFFLOAD_ARCH");
    let architecture = env::var("RANGE_SET_BLAZE_OFFLOAD_ARCH").unwrap_or_else(|_| "sm_86".into());
    assert!(
        architecture.starts_with("sm_")
            && architecture.len() > 3
            && architecture[3..].chars().all(|c| c.is_ascii_digit()),
        "RANGE_SET_BLAZE_OFFLOAD_ARCH must be an NVIDIA architecture such as sm_86"
    );
    let output = PathBuf::from(env::var_os("OUT_DIR").expect("Cargo must set OUT_DIR"));
    let rustc = env::var_os("RUSTC").expect("Cargo must set RUSTC");
    let sysroot = Command::new(&rustc)
        .args(["--print", "sysroot"])
        .output()
        .expect("could not query Rust sysroot");
    assert!(
        sysroot.status.success(),
        "could not locate the Rust sysroot"
    );
    let sysroot = PathBuf::from(
        String::from_utf8(sysroot.stdout)
            .expect("sysroot must be UTF-8")
            .trim(),
    );
    let runtime = sysroot
        .join("lib/rustlib")
        .join(env::var("HOST").expect("Cargo must set HOST"))
        .join("lib");
    assert!(
        runtime.join("libomptarget.so").exists(),
        "missing nightly offload component; run rustup component add offload llvm-tools llvm-bitcode-linker rust-src"
    );
    let source = output.join("native.rs");
    for name in OFFLOAD_SOURCES {
        println!("cargo:rerun-if-changed=src/gpu/offload/{name}");
        fs::copy(format!("src/gpu/offload/{name}"), output.join(name))
            .expect("could not copy offload source");
    }
    let manifest = output.join("offload.manifest");
    let common = [
        "--edition=2024",
        "--crate-name=rsb_offload_native",
        "-Cpanic=abort",
        "-O",
        "-Zunstable-options",
    ];
    run(Command::new(&rustc)
        .args(common)
        .arg("--crate-type=staticlib")
        .arg(format!("-Zoffload=HostMetadata={}", manifest.display()))
        .arg(&source)
        .arg("-o")
        .arg(output.join("metadata.a")));

    // Rebuild core with the GPU's target CPU too: mixing a prebuilt core with a different
    // target CPU is rejected by rustc's ABI checks. Use an isolated Cargo project/target
    // directory so the nested build never waits on the outer Cargo target-directory lock.
    fs::write(output.join("Cargo.toml"), "[package]\nname = \"rsb-offload-native\"\nversion = \"0.0.0\"\nedition = \"2024\"\n[workspace]\n[lib]\npath = \"native.rs\"\ncrate-type = [\"cdylib\"]\n").expect("could not write GPU build manifest");
    let device_target = output.join("device-target");
    run(Command::new(env::var_os("CARGO").expect("Cargo must set CARGO"))
        .args([
            "build",
            "--release",
            "--lib",
            "--target",
            "nvptx64-nvidia-cuda",
            "-Zbuild-std=core",
            "--manifest-path",
        ])
        .arg(output.join("Cargo.toml"))
        .env("CARGO_TARGET_DIR", &device_target)
        .env_remove("CARGO_ENCODED_RUSTFLAGS")
        .env_remove("RUSTC_WRAPPER")
        .env_remove("RUSTC_WORKSPACE_WRAPPER")
        .env(
            "RUSTFLAGS",
            format!(
                "-Ctarget-cpu={architecture} -Cpanic=abort -Zunstable-options -Zoffload=Device={}",
                manifest.display()
            ),
        ));
    let device = device_target.join("nvptx64-nvidia-cuda/release/rsb_offload_native.ptx");
    run(Command::new(&rustc)
        .args(common)
        .args(["--crate-type=staticlib", "-Clto=fat", "-Cembed-bitcode=yes"])
        .arg(format!("-Zoffload=Host={}", device.display()))
        .arg(&source)
        .arg("-o")
        .arg(output.join("librsb_offload_native.a")));
    link_offload(&output, &runtime);
}

fn link_offload(output: &Path, runtime: &Path) {
    println!("cargo:rustc-link-search=native={}", output.display());
    println!("cargo:rustc-link-search=native={}", runtime.display());
    println!("cargo:rustc-link-lib=static=rsb_offload_native");
    println!("cargo:rustc-link-lib=dylib=omptarget");
    // Keep every kernel registration entry. Downstream final binaries need this flag
    // too; Cargo does not propagate rustc-link-arg from dependencies (docs/offload.md).
    println!("cargo:rustc-link-arg=-Wl,-z,nostart-stop-gc");
    println!("cargo:rustc-link-arg=-Wl,-rpath,{}", runtime.display());
}
