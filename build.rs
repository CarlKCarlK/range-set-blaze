//! This build script reads the `BUILDFEATURES` environment variable and sets
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

const OFFLOAD_SOURCES: [&str; 7] = [
    "native.rs",
    "key.rs",
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
    if env::var_os("CARGO_FEATURE_HIP_NIGHTLY_EXPERIMENTAL").is_some() {
        build_hip();
    }
}

/// The kernel sources the HIP engine shares with offload, plus its own crate root.
const HIP_SOURCES: [&str; 6] = [
    "hip_native.rs",
    "key.rs",
    "radix.rs",
    "ranges.rs",
    "device.rs",
    "ptx.rs",
];

/// Compiles the shared radix and range kernels for an AMD GPU as a plain code object that
/// the HIP runtime loads, and links the host against HIP (`libamdhip64`).
fn build_hip() {
    println!("cargo:rerun-if-env-changed=DOCS_RS");
    println!("cargo:rerun-if-env-changed=RANGE_SET_BLAZE_HIP_ARCH");
    println!("cargo:rerun-if-env-changed=RANGE_SET_BLAZE_ROCM_LIB");
    let output = PathBuf::from(env::var_os("OUT_DIR").expect("Cargo must set OUT_DIR")).join("hip");
    fs::create_dir_all(&output).expect("could not create the HIP build directory");
    let code_object = output.join("amdgcn-amd-amdhsa/release/rsb_hip_native.elf");
    println!(
        "cargo:rustc-env=RSB_HIP_CODE_OBJECT={}",
        code_object.display()
    );
    if env::var_os("DOCS_RS").is_some() {
        // Documentation builds expose the API; an empty code object keeps include_bytes! working.
        fs::create_dir_all(code_object.parent().expect("has a parent")).expect("could not create");
        fs::write(&code_object, []).expect("could not write a placeholder code object");
        return;
    }
    let architecture = env::var("RANGE_SET_BLAZE_HIP_ARCH").unwrap_or_else(|_| "gfx1151".into());
    assert!(
        architecture.starts_with("gfx")
            && architecture.len() > 3
            && architecture[3..].chars().all(|c| c.is_ascii_hexdigit()),
        "RANGE_SET_BLAZE_HIP_ARCH must name an AMD architecture such as gfx1151"
    );
    assert!(
        amd_has_wave32(&architecture),
        "RANGE_SET_BLAZE_HIP_ARCH={architecture}: the radix kernels need 32-lane wavefronts (RDNA, gfx10 or newer)"
    );
    for name in HIP_SOURCES {
        println!("cargo:rerun-if-changed=src/gpu/offload/{name}");
        fs::copy(format!("src/gpu/offload/{name}"), output.join(name))
            .expect("could not copy a HIP kernel source");
    }
    fs::write(output.join("Cargo.toml"), "[package]\nname = \"rsb-hip-native\"\nversion = \"0.0.0\"\nedition = \"2024\"\n[workspace]\n[lib]\npath = \"hip_native.rs\"\ncrate-type = [\"cdylib\"]\n").expect("could not write the HIP build manifest");
    nested_gpu_build(
        &output.join("Cargo.toml"),
        &output,
        "amdgcn-amd-amdhsa",
        &format!(
            "-Ctarget-cpu={architecture} -Cpanic=abort -Zunstable-options --cfg=rsb_offload_amdgpu"
        ),
        "HIP kernel compilation failed; see docs/offload.md for the nightly toolchain setup",
    );
    let library = env::var("RANGE_SET_BLAZE_ROCM_LIB").unwrap_or_else(|_| "/opt/rocm/lib".into());
    println!("cargo:rustc-link-search=native={library}");
    println!("cargo:rustc-link-lib=dylib=amdhip64");
    println!("cargo:rustc-link-arg=-Wl,-rpath,{library}");
}

/// Whether an AMD `gfx` architecture runs the 32-lane wavefronts the radix kernels assume:
/// RDNA (gfx10 and newer) does; GCN and CDNA (gfx9 and older) have only 64-lane wavefronts.
fn amd_has_wave32(architecture: &str) -> bool {
    let digits = &architecture[3..];
    digits.len() >= 4 && digits[..2].parse::<u32>().is_ok_and(|major| major >= 10)
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
        assert!(
            amd_has_wave32(architecture),
            "RANGE_SET_BLAZE_OFFLOAD_ARCH={architecture}: the radix kernels need 32-lane wavefronts (RDNA, gfx10 or newer)"
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
    nested_gpu_build(
        &output.join("Cargo.toml"),
        &device_target,
        target.triple,
        &format!(
            "-Ctarget-cpu={architecture} -Cpanic=abort -Zunstable-options {} -Zoffload=Device={}",
            target.configuration,
            manifest.display()
        ),
        "offload compilation failed; see docs/offload.md for setup instructions",
    );
    device_target.join(format!(
        "{}/release/rsb_offload_native.{}",
        target.triple, target.extension
    ))
}

/// Builds the GPU crate at `manifest` for `triple` into `target_dir`, rebuilding `core`
/// for the GPU.
fn nested_gpu_build(
    manifest: &Path,
    target_dir: &Path,
    triple: &str,
    rustflags: &str,
    failure: &str,
) {
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
    let status = cargo
        .args([
            "build",
            "--release",
            "--lib",
            "--target",
            triple,
            "-Zbuild-std=core",
            "--manifest-path",
        ])
        .arg(manifest)
        .env("CARGO_TARGET_DIR", target_dir)
        .env_remove("CARGO_ENCODED_RUSTFLAGS")
        .env_remove("RUSTC_WRAPPER")
        .env_remove("RUSTC_WORKSPACE_WRAPPER")
        .env("RUSTFLAGS", rustflags)
        .status()
        .expect("could not start the GPU compiler");
    assert!(status.success(), "{failure}");
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
