use std::env;
use std::process::{Command, ExitCode, Stdio};

fn run_step(name: &str, args: &[&str]) -> bool {
    println!("\n==> {name}");
    let status = Command::new("cargo")
        .args(args)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status();

    match status {
        Ok(s) if s.success() => true,
        Ok(s) => {
            eprintln!("step failed: {name} (status: {s})");
            false
        }
        Err(err) => {
            eprintln!("step failed: {name} (error: {err})");
            false
        }
    }
}

fn check_all() -> ExitCode {
    // If nextest is unavailable, fall back to cargo test while preserving flow.
    let has_nextest = Command::new("cargo")
        .args(["nextest", "--version"])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .is_ok_and(|s| s.success());

    let mut steps: Vec<(&str, Vec<&str>)> = vec![
        (
            "clippy (std + rog_experimental)",
            vec![
                "clippy",
                "--all-targets",
                "--features",
                "std rog_experimental",
                "--",
                "-D",
                "clippy::all",
                "-A",
                "deprecated",
            ],
        ),
        ("test --release", vec!["test", "--release"]),
        (
            "test --no-default-features --features rog_experimental",
            vec![
                "test",
                "--no-default-features",
                "--features",
                "rog_experimental",
            ],
        ),
        (
            "nightly test --features rog_experimental from_slice",
            vec![
                "+nightly",
                "test",
                "--features",
                "rog_experimental from_slice",
            ],
        ),
        (
            "nightly test --all-features",
            vec!["+nightly", "test", "--all-features"],
        ),
    ];

    if has_nextest {
        steps.insert(
            1,
            (
                "nextest -p range-set-blaze --all-targets",
                vec!["xtest"],
            ),
        );
    } else {
        eprintln!("note: cargo-nextest not found; using cargo test -p range-set-blaze --all-targets");
        steps.insert(
            1,
            (
                "test -p range-set-blaze --all-targets",
                vec!["test", "-p", "range-set-blaze", "--all-targets"],
            ),
        );
    }

    for (name, args) in steps {
        if !run_step(name, &args) {
            eprintln!("\ncheck-all FAILED");
            return ExitCode::from(1);
        }
    }

    println!("\ncheck-all PASSED");
    ExitCode::SUCCESS
}

fn main() -> ExitCode {
    let mut args = env::args().skip(1);
    match args.next().as_deref() {
        Some("check-all") => check_all(),
        _ => {
            eprintln!("usage: cargo check-all");
            ExitCode::from(2)
        }
    }
}
