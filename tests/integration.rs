//! Integration tests for run-bob.
//! Each test creates a tempdir, runs init/status, and verifies filesystem state.

use std::process::Command;

/// Path to the cargo-built binary under test.
fn run_bob_bin() -> std::path::PathBuf {
    // CARGO_BIN_EXE_<name> is set by Cargo for integration tests.
    std::path::PathBuf::from(env!("CARGO_BIN_EXE_run-bob"))
}

#[test]
fn binary_prints_version() {
    let output = Command::new(run_bob_bin())
        .arg("--version")
        .output()
        .expect("run run-bob --version");
    assert!(output.status.success(), "run-bob --version failed");
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(
        stdout.contains("run-bob 0.1.0"),
        "expected version in output, got: {}",
        stdout
    );
}

#[test]
fn init_help_lists_flags() {
    let output = Command::new(run_bob_bin())
        .args(["init", "--help"])
        .output()
        .expect("run run-bob init --help");
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    for flag in &["--force", "--minimal", "--dir"] {
        assert!(stdout.contains(flag), "expected {} flag in help: {}", flag, stdout);
    }
}

#[test]
fn init_creates_architecture_md() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let target = tmp.path();

    let status = Command::new(run_bob_bin())
        .args(["init", "--dir"])
        .arg(target)
        .status()
        .expect("run run-bob init");
    assert!(status.success(), "run-bob init failed");

    let p = target.join("ARCHITECTURE.md");
    assert!(p.is_file(), "missing {}", p.display());
    let content = std::fs::read_to_string(&p).unwrap();
    assert!(
        content.contains("# 架构(Bob 4 环)"),
        "ARCHITECTURE.md must have Bob 4-ring header"
    );
    assert!(
        content.contains("Single Source of Truth"),
        "ARCHITECTURE.md must declare itself as SSoT"
    );
}
