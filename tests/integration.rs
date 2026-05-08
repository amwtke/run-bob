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

#[test]
fn init_creates_claude_md_with_r0() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let target = tmp.path();

    Command::new(run_bob_bin())
        .args(["init", "--dir"])
        .arg(target)
        .status()
        .expect("init");

    let p = target.join("CLAUDE.md");
    assert!(p.is_file(), "CLAUDE.md missing");
    let content = std::fs::read_to_string(&p).unwrap();

    // Must declare itself a run-bob harness
    assert!(content.contains("run-bob"), "CLAUDE.md must reference run-bob");
    // R0 meta-rule must be present
    assert!(content.contains("R0"), "CLAUDE.md must have R0 meta-rule");
    assert!(
        content.contains("通用判定优先于具体清单") || content.contains("5 问决策树"),
        "R0 must reference the decision tree"
    );
    // R12 must be present (B2 mode)
    assert!(content.contains("R12"), "CLAUDE.md must have R12 (B2 clean island)");
    // 4-ring package names use "entity" (not "domain")
    assert!(content.contains("entity"), "CLAUDE.md must use 'entity' not 'domain' as Ring 1");
    // Must have technology-stack-pending warning
    assert!(
        content.contains("## 技术栈约定"),
        "CLAUDE.md must have ## 技术栈约定 section"
    );
}

#[test]
fn init_creates_readme_run_bob_with_3_modes() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let target = tmp.path();
    Command::new(run_bob_bin()).args(["init", "--dir"]).arg(target).status().expect("init");

    let p = target.join("README-RUN-BOB.md");
    assert!(p.is_file(), "README-RUN-BOB.md missing");
    let content = std::fs::read_to_string(&p).unwrap();
    for token in &[
        "/bob-identify",
        "/bob-onion",
        "/bob-spec",
        "ARCHITECTURE.md",
        "G(",     // G mode
        "B1",     // B1 mode
        "B2",     // B2 mode
    ] {
        assert!(content.contains(token), "README-RUN-BOB.md must contain {}", token);
    }
}
