use std::env;
use std::path::PathBuf;
use std::process::{Command, ExitCode, Stdio};
use std::sync::{Arc, Mutex};
use std::thread;

fn slugify(name: &str) -> String {
    let mut slug = String::with_capacity(name.len());
    let mut last_was_dash = false;
    for ch in name.chars() {
        if ch.is_ascii_alphanumeric() {
            slug.push(ch.to_ascii_lowercase());
            last_was_dash = false;
        } else if !last_was_dash {
            slug.push('-');
            last_was_dash = true;
        }
    }
    slug.trim_matches('-').to_string()
}

fn run_step(name: &str, program: &str, args: &[String], target_dir: &PathBuf) -> bool {
    println!("\n==> {name} [target-dir: {}]", target_dir.display());
    let status = Command::new(program)
        .args(args)
        .env("CARGO_TARGET_DIR", target_dir)
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

    // `just clippy` / `just test-stable` / `just test-nightly` are the single source of
    // truth for which feature-flag combinations get tested — see `justfile`. Delegating
    // to them here (instead of re-listing `cargo` invocations) keeps `cargo check-all`
    // and `just check-all` from drifting apart the way they did before.
    let mut steps: Vec<(String, String, Vec<String>)> = vec![
        (
            "just clippy".to_string(),
            "just".to_string(),
            vec!["clippy".to_string()],
        ),
        (
            "just test-stable".to_string(),
            "just".to_string(),
            vec!["test-stable".to_string()],
        ),
        (
            "just test-nightly".to_string(),
            "just".to_string(),
            vec!["test-nightly".to_string()],
        ),
    ];

    if has_nextest {
        steps.insert(
            1,
            (
                "nextest -p range-set-blaze --lib --tests --examples".to_string(),
                "cargo".to_string(),
                vec!["xtest".to_string()],
            ),
        );
    } else {
        eprintln!(
            "note: cargo-nextest not found; using cargo test -p range-set-blaze --lib --tests --examples"
        );
        steps.insert(
            1,
            (
                "test -p range-set-blaze --lib --tests --examples".to_string(),
                "cargo".to_string(),
                vec![
                    "test".to_string(),
                    "-p".to_string(),
                    "range-set-blaze".to_string(),
                    "--lib".to_string(),
                    "--tests".to_string(),
                    "--examples".to_string(),
                ],
            ),
        );
    }

    println!("\nRunning {} steps in parallel...", steps.len());
    let failures = Arc::new(Mutex::new(Vec::<String>::new()));
    let handles: Vec<_> = steps
        .into_iter()
        .enumerate()
        .map(|(idx, (name, program, args))| {
            let failures = Arc::clone(&failures);
            thread::spawn(move || {
                let target_dir = PathBuf::from("target").join("check-all").join(format!(
                    "{:02}-{}",
                    idx + 1,
                    slugify(&name)
                ));
                if !run_step(&name, &program, &args, &target_dir) {
                    failures.lock().expect("poisoned mutex").push(name);
                }
            })
        })
        .collect();

    for handle in handles {
        if handle.join().is_err() {
            failures
                .lock()
                .expect("poisoned mutex")
                .push("internal thread panic".to_string());
        }
    }

    let failures = failures.lock().expect("poisoned mutex");
    if !failures.is_empty() {
        eprintln!("\ncheck-all FAILED");
        for failure in failures.iter() {
            eprintln!(" - {failure}");
        }
        return ExitCode::from(1);
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
