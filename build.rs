//! This build script reads the `BUILDFEATURES` environment variable and sets
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

const OFFLOAD_SOURCES: [&str; 6] = [
    "native.rs",
    "resident.rs",
    "radix.rs",
    "ranges.rs",
    "device.rs",
    "ptx.rs",
];

struct DeviceTarget {
    triple: &'static str,
    extension: &'static str,
    configuration: &'static str,
}

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
    println!("cargo:rustc-check-cfg=cfg(rsb_offload_amdgpu)");
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
    let target = offload_target(&architecture);
    if target.triple == "amdgcn-amd-amdhsa" {
        println!("cargo:rustc-cfg=rsb_offload_amdgpu");
    }
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
        target.configuration,
    ];
    run(Command::new(&rustc)
        .args(common)
        .arg("--crate-type=staticlib")
        .arg(format!("-Zoffload=HostMetadata={}", manifest.display()))
        .arg(&source)
        .arg("-o")
        .arg(output.join("metadata.a")));

    let device = compile_device(&output, &manifest, &architecture, &target);
    run(Command::new(&rustc)
        .args(common)
        .args(["--crate-type=staticlib", "-Clto=fat", "-Cembed-bitcode=yes"])
        .arg(format!("-Zoffload=Host={}", device.display()))
        .arg(&source)
        .arg("-o")
        .arg(output.join("librsb_offload_native.a")));
    link_offload(&output, &runtime);
}

fn offload_target(architecture: &str) -> DeviceTarget {
    if architecture.starts_with("sm_")
        && architecture.len() > 3
        && architecture[3..].chars().all(|c| c.is_ascii_digit())
    {
        DeviceTarget {
            triple: "nvptx64-nvidia-cuda",
            extension: "ptx",
            configuration: "--cfg=rsb_offload_nvidia",
        }
    } else {
        assert!(
            architecture.starts_with("gfx")
                && architecture.len() > 3
                && architecture[3..].chars().all(|c| c.is_ascii_hexdigit()),
            "RANGE_SET_BLAZE_OFFLOAD_ARCH must name an NVIDIA (sm_86) or AMD (gfx1151) architecture"
        );
        DeviceTarget {
            triple: "amdgcn-amd-amdhsa",
            extension: "elf",
            configuration: "--cfg=rsb_offload_amdgpu",
        }
    }
}

fn compile_device(
    output: &Path,
    manifest: &Path,
    architecture: &str,
    target: &DeviceTarget,
) -> PathBuf {
    // Rebuild core with the GPU's target CPU too: mixing a prebuilt core with a different
    // target CPU is rejected by rustc's ABI checks. Use an isolated Cargo project/target
    // directory so the nested build never waits on the outer Cargo target-directory lock.
    fs::write(output.join("Cargo.toml"), "[package]\nname = \"rsb-offload-native\"\nversion = \"0.0.0\"\nedition = \"2024\"\n[workspace]\n[lib]\npath = \"native.rs\"\ncrate-type = [\"cdylib\"]\n").expect("could not write GPU build manifest");
    let device_target = output.join("device-target");
    let mut cargo = Command::new(env::var_os("CARGO").expect("Cargo must set CARGO"));
    // Cargo gives this build script the host's `CARGO_CFG_*` and `CARGO_FEATURE_*`
    // variables, and the nested Cargo does not clear ones its own target lacks. Leaked
    // host cfgs mislead `compiler_builtins`: x86_64's `CARGO_CFG_TARGET_HAS_RELIABLE_F128`
    // enables f128 routines that LLVM cannot lower for amdgcn.
    for (key, _) in env::vars_os() {
        if key
            .to_str()
            .is_some_and(|key| key.starts_with("CARGO_CFG_") || key.starts_with("CARGO_FEATURE_"))
        {
            cargo.env_remove(key);
        }
    }
    run(cargo
        .args([
            "build",
            "--release",
            "--lib",
            "--target",
            target.triple,
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
                "-Ctarget-cpu={architecture} -Cpanic=abort -Zunstable-options {} -Zoffload=Device={}",
                target.configuration, manifest.display()
            ),
        ));
    device_target.join(format!(
        "{}/release/rsb_offload_native.{}",
        target.triple, target.extension
    ))
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
