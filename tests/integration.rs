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
        stdout.contains("run-bob 0.2.0"),
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

#[test]
fn init_installs_archunit_test_at_correct_path() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let target = tmp.path();

    Command::new(run_bob_bin()).args(["init", "--dir"]).arg(target).status().expect("init");

    let archunit = target
        .join("src").join("test").join("java")
        .join("architecture").join("CleanArchitectureTest.java");
    assert!(archunit.is_file(), "expected ArchUnit template at {}", archunit.display());

    let content = std::fs::read_to_string(&archunit).unwrap();
    assert!(content.contains("@ArchTest"), "must contain @ArchTest");
    assert!(content.contains("layered_dependencies"), "must include layered_dependencies");
    assert!(
        content.contains("entity_pure_of_frameworks"),
        "must include entity_pure_of_frameworks rule"
    );
    assert!(
        content.contains("usecase_pure_of_frameworks"),
        "must include usecase_pure_of_frameworks rule"
    );
    assert!(
        content.contains("FORBIDDEN_IN_INNER"),
        "must use parameterized FORBIDDEN_IN_INNER array"
    );
    assert!(
        content.contains("transactional_methods_only_in_decorator"),
        "must include transactional decorator rule"
    );
    assert!(
        content.contains("no_event_listener_unless_decided"),
        "must include R10-bob no_event_listener_unless_decided rule"
    );
    assert!(
        content.contains("packages = \"com.example\""),
        "default @AnalyzeClasses must use com.example to cover shared + business"
    );
}

#[test]
fn init_installs_shared_usecase_interface() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let target = tmp.path();

    Command::new(run_bob_bin()).args(["init", "--dir"]).arg(target).status().expect("init");

    let p = target
        .join("src").join("main").join("java")
        .join("com").join("example").join("shared")
        .join("usecase").join("UseCase.java");
    assert!(p.is_file(), "expected UseCase.java at {}", p.display());

    let content = std::fs::read_to_string(&p).unwrap();
    assert!(
        content.contains("package com.example.shared.usecase;"),
        "must declare correct package"
    );
    assert!(
        content.contains("public interface UseCase<C, R>"),
        "must declare generic UseCase<C, R> interface"
    );
    assert!(content.contains("R execute(C cmd)"), "must declare execute method");
    // Must NOT contain Spring or any framework import
    assert!(!content.contains("org.springframework"), "UseCase.java must not import Spring");
    assert!(!content.contains("@Transactional"), "UseCase.java must not have @Transactional");
}

#[test]
fn init_installs_transactional_decorator() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let target = tmp.path();

    Command::new(run_bob_bin()).args(["init", "--dir"]).arg(target).status().expect("init");

    let p = target
        .join("src").join("main").join("java")
        .join("com").join("example").join("shared")
        .join("framework").join("transaction").join("TransactionalUseCaseDecorator.java");
    assert!(p.is_file(), "expected decorator at {}", p.display());

    let content = std::fs::read_to_string(&p).unwrap();
    assert!(
        content.contains("package com.example.shared.framework.transaction;"),
        "must declare correct package"
    );
    assert!(content.contains("@Transactional"), "decorator must have @Transactional");
    assert!(
        content.contains("implements UseCase<C, R>"),
        "decorator must implement UseCase<C, R>"
    );
    assert!(
        content.contains("import org.springframework.transaction.annotation.Transactional;"),
        "decorator must import Spring's @Transactional"
    );
}

#[test]
fn init_creates_bob_identify_skill() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let target = tmp.path();
    Command::new(run_bob_bin()).args(["init", "--dir"]).arg(target).status().expect("init");

    let p = target.join(".claude").join("skills").join("bob-identify").join("SKILL.md");
    assert!(p.is_file(), "bob-identify SKILL.md missing");
    let content = std::fs::read_to_string(&p).unwrap();

    // YAML frontmatter
    assert!(content.starts_with("---"), "must start with YAML frontmatter");
    assert!(content.contains("name: bob-identify"), "frontmatter name");
    assert!(content.contains("description:"), "frontmatter description");

    // Required sections
    for token in &[
        "5 问决策树",
        "Q1",
        "Q2",
        "Q3",
        "Q4",
        "Q5",
        "G",
        "B1",
        "B2",
        "推测",
        "推荐",
        "清洁孤岛",
    ] {
        assert!(content.contains(token), "bob-identify must mention {}", token);
    }
}

#[test]
fn init_creates_bob_onion_skill() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let target = tmp.path();
    Command::new(run_bob_bin()).args(["init", "--dir"]).arg(target).status().expect("init");

    let p = target.join(".claude").join("skills").join("bob-onion").join("SKILL.md");
    assert!(p.is_file(), "bob-onion SKILL.md missing");
    let content = std::fs::read_to_string(&p).unwrap();

    assert!(content.starts_with("---"));
    assert!(content.contains("name: bob-onion"));

    for token in &[
        "ARCHITECTURE.md",
        "4 环",
        "端口清单",
        "状态机",
        "TransactionalUseCaseDecorator",
        "FORBIDDEN_IN_INNER",
        "ADR",
        "推测",
        "G",
        "B1",
        "B2",
    ] {
        assert!(content.contains(token), "bob-onion must mention {}", token);
    }
}

#[test]
fn init_creates_bob_spec_skill() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let target = tmp.path();
    Command::new(run_bob_bin()).args(["init", "--dir"]).arg(target).status().expect("init");

    let p = target.join(".claude").join("skills").join("bob-spec").join("SKILL.md");
    assert!(p.is_file(), "bob-spec SKILL.md missing");
    let content = std::fs::read_to_string(&p).unwrap();

    assert!(content.starts_with("---"));
    assert!(content.contains("name: bob-spec"));

    for token in &[
        "Given-When-Then",
        "ARCHITECTURE.md",
        "TransactionalUseCaseDecorator",
        "Superpowers",
        "命令型",
        "查询型",
        "重构型",
        "交给 Superpowers 的开放问题",
        "技术栈",
        "5 问决策树",
        "推测",
    ] {
        assert!(content.contains(token), "bob-spec must mention {}", token);
    }
}

#[test]
fn init_creates_working_directories() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let target = tmp.path();
    Command::new(run_bob_bin()).args(["init", "--dir"]).arg(target).status().expect("init");

    assert!(target.join("docs").join("bob").is_dir(), "docs/bob/ missing");
    assert!(target.join("docs").join("specs").is_dir(), "docs/specs/ missing");
}

#[test]
fn init_minimal_skips_archunit_and_shared_and_anchors() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let target = tmp.path();

    Command::new(run_bob_bin())
        .args(["init", "--minimal", "--dir"])
        .arg(target)
        .status()
        .expect("init --minimal");

    // Skills must still be installed
    for skill in &["bob-identify", "bob-onion", "bob-spec", "bob-survey", "bob-stories", "bob-nfr"] {
        let p = target.join(".claude").join("skills").join(skill).join("SKILL.md");
        assert!(p.is_file(), "minimal must still install skill {}", skill);
    }

    // Anchor docs must NOT be installed
    for f in &["CLAUDE.md", "ARCHITECTURE.md", "README-RUN-BOB.md"] {
        assert!(
            !target.join(f).exists(),
            "minimal must not install {}",
            f
        );
    }

    // ArchUnit must NOT be installed
    assert!(
        !target.join("src").join("test").join("java").join("architecture")
            .join("CleanArchitectureTest.java").exists(),
        "minimal must not install ArchUnit"
    );

    // Shared骨架 must NOT be installed
    assert!(
        !target.join("src").join("main").join("java").join("com").join("example")
            .join("shared").join("usecase").join("UseCase.java").exists(),
        "minimal must not install UseCase.java"
    );
    assert!(
        !target.join("src").join("main").join("java").join("com").join("example")
            .join("shared").join("framework").join("transaction")
            .join("TransactionalUseCaseDecorator.java").exists(),
        "minimal must not install TransactionalUseCaseDecorator.java"
    );

    // Working directories must NOT be created in --minimal
    assert!(!target.join("docs").join("bob").exists(), "minimal must not create docs/bob/");
    assert!(!target.join("docs").join("specs").exists(), "minimal must not create docs/specs/");
}

#[test]
fn status_reports_complete_after_full_init() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let target = tmp.path();
    Command::new(run_bob_bin()).args(["init", "--dir"]).arg(target).status().expect("init");

    let output = Command::new(run_bob_bin())
        .args(["status", "--dir"])
        .arg(target)
        .output()
        .expect("status");
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert!(stdout.contains("harness is complete"), "got:\n{}", stdout);
    for token in &[
        "bob-identify",
        "bob-onion",
        "bob-spec",
        "CLAUDE.md",
        "ARCHITECTURE.md",
        "README-RUN-BOB.md",
        "CleanArchitectureTest.java",
        "UseCase.java",
        "TransactionalUseCaseDecorator.java",
        "docs/bob",
        "docs/specs",
    ] {
        assert!(stdout.contains(token), "status must list {}; got:\n{}", token, stdout);
    }
}

/// Drift guard: every file `init` writes must be checked by `status`.
/// If a new template is added to the asset registry but somehow only one
/// side observes it, this test fails.
#[test]
fn status_checks_every_file_init_writes() {
    use std::collections::HashSet;

    let tmp = tempfile::tempdir().expect("tempdir");
    let target = tmp.path();
    Command::new(run_bob_bin())
        .args(["init", "--dir"])
        .arg(target)
        .status()
        .expect("init");

    // Collect every file init produced, as forward-slash relative paths.
    let mut init_wrote: HashSet<String> = HashSet::new();
    fn walk(root: &std::path::Path, dir: &std::path::Path, acc: &mut HashSet<String>) {
        for entry in std::fs::read_dir(dir).expect("read_dir") {
            let path = entry.expect("entry").path();
            if path.is_file() {
                let rel = path
                    .strip_prefix(root)
                    .expect("strip_prefix")
                    .to_string_lossy()
                    .replace('\\', "/");
                acc.insert(rel);
            } else if path.is_dir() {
                walk(root, &path, acc);
            }
        }
    }
    walk(target, target, &mut init_wrote);

    // .gitignore is managed by the gitignore module (not a harness asset),
    // so status intentionally does not track it. Exclude it from the drift check.
    init_wrote.remove(".gitignore");

    // Run status and pull out the relative paths it printed.
    let output = Command::new(run_bob_bin())
        .args(["status", "--dir"])
        .arg(target)
        .output()
        .expect("status");
    let stdout = String::from_utf8(output.stdout).unwrap();

    let status_checks: HashSet<String> = stdout
        .lines()
        .filter_map(|line| {
            let t = line.trim();
            t.strip_prefix("✓ ")
                .or_else(|| t.strip_prefix("✗ "))
                .map(|s| s.to_string())
        })
        // exclude directory entries (they end with '/')
        .filter(|s| !s.ends_with('/'))
        .collect();

    let missing: Vec<&String> = init_wrote.difference(&status_checks).collect();
    assert!(
        missing.is_empty(),
        "drift detected: init wrote files that status does not check: {:?}\nstatus stdout:\n{}",
        missing,
        stdout
    );
}

#[test]
fn status_flags_missing_after_minimal_init() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let target = tmp.path();
    Command::new(run_bob_bin()).args(["init", "--minimal", "--dir"]).arg(target).status().expect("init");

    let output = Command::new(run_bob_bin())
        .args(["status", "--dir"])
        .arg(target)
        .output()
        .expect("status");
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("some assets are missing"), "got:\n{}", stdout);
}

/// SSoT drift guard: every entry in HARNESS_ASSETS must have a deliberate
/// `upgrade_safe` value matching its policy.
///   - Skill + SharedJava → always upgrade-safe (pure generated content)
///   - ArchUnit          → never upgrade-safe (FORBIDDEN_IN_INNER is user-edited)
///   - HarnessDoc        → mixed (README is safe; CLAUDE / ARCHITECTURE are not)
#[test]
fn upgrade_safe_field_matches_category_policy() {
    use run_bob::assets::{Category, HARNESS_ASSETS};

    for asset in HARNESS_ASSETS {
        let display = asset.rel_path.join("/");
        match asset.category {
            Category::Skill => assert!(
                asset.upgrade_safe,
                "{} is a Skill but upgrade_safe=false",
                display
            ),
            Category::SharedJava => assert!(
                asset.upgrade_safe,
                "{} is a SharedJava but upgrade_safe=false",
                display
            ),
            Category::ArchUnit => assert!(
                !asset.upgrade_safe,
                "{} is an ArchUnit but upgrade_safe=true",
                display
            ),
            Category::HarnessDoc => {
                let upgrade_safe_docs: &[&[&str]] = &[
                    &["README-RUN-BOB.md"],
                    &["docs", "compliance", "README.md"],
                ];
                let is_upgrade_safe_doc = upgrade_safe_docs
                    .iter()
                    .any(|s| asset.rel_path == *s);
                if is_upgrade_safe_doc {
                    assert!(
                        asset.upgrade_safe,
                        "{} is a machine-managed HarnessDoc, must be upgrade_safe=true",
                        display
                    );
                } else {
                    assert!(
                        !asset.upgrade_safe,
                        "{} is a user-owned HarnessDoc but upgrade_safe=true",
                        display
                    );
                }
            }
        }
    }
}

#[test]
fn upgrade_help_lists_flags() {
    let output = std::process::Command::new(run_bob_bin())
        .args(["upgrade", "--help"])
        .output()
        .expect("run run-bob upgrade --help");
    assert!(
        output.status.success(),
        "run-bob upgrade --help failed: {:?}",
        output
    );
    let stdout = String::from_utf8(output.stdout).unwrap();
    for flag in &["--dir", "--dry-run", "--no-backup"] {
        assert!(
            stdout.contains(flag),
            "expected {} in `upgrade --help` output, got:\n{}",
            flag,
            stdout
        );
    }
}

#[test]
fn upgrade_on_fresh_init_is_noop() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let target = tmp.path();

    // Fresh init lays down everything.
    let status = std::process::Command::new(run_bob_bin())
        .args(["init", "--dir"])
        .arg(target)
        .status()
        .expect("init");
    assert!(status.success());

    let output = std::process::Command::new(run_bob_bin())
        .args(["upgrade", "--dir"])
        .arg(target)
        .output()
        .expect("upgrade");
    assert!(output.status.success(), "upgrade failed: {:?}", output);
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert!(
        stdout.contains("All upgrade-safe assets are up to date"),
        "expected no-op message, got:\n{}",
        stdout
    );
    // No backup dir created on no-op.
    assert!(
        !target.join(".run-bob-backup").exists(),
        ".run-bob-backup must not exist after a no-op upgrade"
    );
}

#[test]
fn upgrade_skips_user_owned() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let target = tmp.path();

    std::process::Command::new(run_bob_bin())
        .args(["init", "--dir"])
        .arg(target)
        .status()
        .expect("init");

    // Hand-edit the 3 user-owned files. Upgrade must NOT touch them.
    let sentinel = "USER-EDIT-DO-NOT-OVERWRITE\n";
    let user_owned = [
        target.join("CLAUDE.md"),
        target.join("ARCHITECTURE.md"),
        target.join("src/test/java/architecture/CleanArchitectureTest.java"),
    ];
    for p in &user_owned {
        std::fs::write(p, sentinel).expect("write sentinel");
    }

    let output = std::process::Command::new(run_bob_bin())
        .args(["upgrade", "--dir"])
        .arg(target)
        .output()
        .expect("upgrade");
    assert!(output.status.success(), "upgrade failed: {:?}", output);

    for p in &user_owned {
        let actual = std::fs::read_to_string(p).expect("read user-owned");
        assert_eq!(
            actual, sentinel,
            "upgrade must not touch user-owned file {}",
            p.display()
        );
    }

    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(
        stdout.contains("user-owned files skipped"),
        "expected user-owned skip note, got:\n{}",
        stdout
    );
}

#[test]
fn upgrade_dry_run_writes_nothing() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let target = tmp.path();

    std::process::Command::new(run_bob_bin())
        .args(["init", "--dir"])
        .arg(target)
        .status()
        .expect("init");

    // Corrupt one skill file so detection sees OUTDATED.
    let skill = target.join(".claude/skills/bob-identify/SKILL.md");
    std::fs::write(&skill, "STALE\n").expect("write stale");

    let output = std::process::Command::new(run_bob_bin())
        .args(["upgrade", "--dry-run", "--dir"])
        .arg(target)
        .output()
        .expect("upgrade --dry-run");
    assert!(output.status.success(), "upgrade --dry-run failed: {:?}", output);

    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(
        stdout.contains("dry-run"),
        "expected dry-run note, got:\n{}",
        stdout
    );

    // File on disk is still the stale version.
    let actual = std::fs::read_to_string(&skill).expect("read");
    assert_eq!(actual, "STALE\n", "dry-run must not write files");
    assert!(
        !target.join(".run-bob-backup").exists(),
        ".run-bob-backup must not exist after dry-run"
    );
}

#[test]
fn upgrade_replaces_stale_skill() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let target = tmp.path();

    std::process::Command::new(run_bob_bin())
        .args(["init", "--dir"])
        .arg(target)
        .status()
        .expect("init");

    let skill = target.join(".claude/skills/bob-identify/SKILL.md");
    let original = std::fs::read_to_string(&skill).expect("read original");
    std::fs::write(&skill, "STALE-CONTENT\n").expect("write stale");

    let output = std::process::Command::new(run_bob_bin())
        .args(["upgrade", "--dir"])
        .arg(target)
        .output()
        .expect("upgrade");
    assert!(output.status.success(), "upgrade failed: {:?}", output);

    // File is restored to embedded content.
    let after = std::fs::read_to_string(&skill).expect("read after");
    assert_eq!(
        after, original,
        "skill must be restored to embedded content after upgrade"
    );

    // Backup directory exists; find its single timestamp subdir.
    let backup_root = target.join(".run-bob-backup");
    assert!(backup_root.is_dir(), ".run-bob-backup must exist");
    let entries: Vec<_> = std::fs::read_dir(&backup_root)
        .expect("read_dir backup")
        .filter_map(|e| e.ok())
        .collect();
    assert_eq!(
        entries.len(),
        1,
        "exactly one timestamp dir expected under .run-bob-backup"
    );
    let ts_dir = entries[0].path();
    let backed_up = ts_dir.join(".claude/skills/bob-identify/SKILL.md");
    assert!(
        backed_up.is_file(),
        "backed-up file expected at {}",
        backed_up.display()
    );
    let backup_content = std::fs::read_to_string(&backed_up).expect("read backup");
    assert_eq!(
        backup_content, "STALE-CONTENT\n",
        "backup must preserve the original (stale) content"
    );
}

#[test]
fn upgrade_no_backup_skips_backup() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let target = tmp.path();

    std::process::Command::new(run_bob_bin())
        .args(["init", "--dir"])
        .arg(target)
        .status()
        .expect("init");

    let skill = target.join(".claude/skills/bob-onion/SKILL.md");
    let embedded = std::fs::read_to_string(&skill).expect("read embedded");
    std::fs::write(&skill, "STALE\n").expect("write stale");

    let output = std::process::Command::new(run_bob_bin())
        .args(["upgrade", "--no-backup", "--dir"])
        .arg(target)
        .output()
        .expect("upgrade --no-backup");
    assert!(output.status.success(), "upgrade --no-backup failed: {:?}", output);

    // File was overwritten.
    let after = std::fs::read_to_string(&skill).expect("read after");
    assert_eq!(after, embedded, "skill must be overwritten with --no-backup");

    // But no backup directory was created.
    assert!(
        !target.join(".run-bob-backup").exists(),
        ".run-bob-backup must not exist with --no-backup"
    );
}

#[test]
fn upgrade_installs_missing_skill() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let target = tmp.path();

    std::process::Command::new(run_bob_bin())
        .args(["init", "--dir"])
        .arg(target)
        .status()
        .expect("init");

    // Delete a skill so the upgrade sees MISSING.
    let skill = target.join(".claude/skills/bob-spec/SKILL.md");
    std::fs::remove_file(&skill).expect("remove skill");
    assert!(!skill.exists());

    let output = std::process::Command::new(run_bob_bin())
        .args(["upgrade", "--dir"])
        .arg(target)
        .output()
        .expect("upgrade");
    assert!(output.status.success(), "upgrade failed: {:?}", output);

    // Skill was re-installed.
    assert!(skill.is_file(), "bob-spec/SKILL.md must be installed back");
    let content = std::fs::read_to_string(&skill).expect("read installed");
    assert!(
        content.contains("name: bob-spec"),
        "installed content must match the embedded template"
    );

    // MISSING does not create a backup directory.
    assert!(
        !target.join(".run-bob-backup").exists(),
        ".run-bob-backup must NOT exist when only MISSING was applied"
    );

    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(
        stdout.contains("installed"),
        "expected 'installed' in summary, got:\n{}",
        stdout
    );
}

#[test]
fn init_creates_bob_survey_skill() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let target = tmp.path();
    std::process::Command::new(run_bob_bin())
        .args(["init", "--dir"])
        .arg(target)
        .status()
        .expect("init");

    let p = target.join(".claude").join("skills").join("bob-survey").join("SKILL.md");
    assert!(p.is_file(), "bob-survey SKILL.md missing at {}", p.display());
    let content = std::fs::read_to_string(&p).unwrap();

    // Frontmatter contract
    assert!(content.starts_with("---"), "must start with YAML frontmatter");
    assert!(content.contains("name: bob-survey"), "frontmatter name");
    assert!(content.contains("description:"), "frontmatter description");

    // Load-bearing tokens — the conversation contract
    for token in &[
        // Workflow
        "/bob-survey",
        "三段式",
        "推测",
        "推荐选择",
        // Three repo states
        "G(绿地)",
        "β(棕地未跑过 bob)",
        "γ(成熟 bob)",
        // 6 scoring dimensions
        "Entity 纯度",
        "UseCase 纯度",
        "端口位置",
        "状态机位置",
        "@Transactional 唯一",
        "FORBIDDEN_IN_INNER",
        // Difficulty rubric
        "跨环数",
        "状态机增量",
        "legacy 复用",
        "Easy",
        "Medium",
        "Hard",
        // v2 — 4th factor + stories routing
        "前置重构量",
        "Q4",
        "/bob-stories",
        // Recommendation matrix
        "🟢",
        "🟡",
        "🔴",
        // Output schema
        "docs/bob/00-survey-",
        "ARCHITECTURE.md",
        "§12",
        // Soft handoff to identify
        "/bob-identify",
    ] {
        assert!(content.contains(token), "bob-survey must mention {}", token);
    }
}

#[test]
fn init_creates_architecture_md_with_section_12() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let target = tmp.path();
    std::process::Command::new(run_bob_bin())
        .args(["init", "--dir"])
        .arg(target)
        .status()
        .expect("init");

    let p = target.join("ARCHITECTURE.md");
    assert!(p.is_file());
    let content = std::fs::read_to_string(&p).unwrap();

    // §12 header + empty table header must be shipped so /bob-survey
    // can append rows deterministically.
    assert!(
        content.contains("## 12. 架构体检记录"),
        "ARCHITECTURE.md must ship empty §12 header"
    );
    for col in &["日期", "状态", "总分", "需求", "难度", "推荐", "详报"] {
        assert!(
            content.contains(col),
            "ARCHITECTURE.md §12 must have column header {}",
            col
        );
    }
}

#[test]
fn bob_identify_mentions_survey_soft_prompt() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let target = tmp.path();
    std::process::Command::new(run_bob_bin())
        .args(["init", "--dir"])
        .arg(target)
        .status()
        .expect("init");

    let p = target.join(".claude").join("skills").join("bob-identify").join("SKILL.md");
    let content = std::fs::read_to_string(&p).unwrap();

    // The soft prompt must mention survey and the 7-day threshold.
    for token in &[
        "/bob-survey",
        "docs/bob/00-survey",
        "7 天",
        "soft",  // marker we'll include in the new section header
    ] {
        assert!(
            content.contains(token),
            "bob-identify must mention {} for survey integration",
            token
        );
    }
}

#[test]
fn init_creates_bob_stories_skill() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let target = tmp.path();
    std::process::Command::new(run_bob_bin())
        .args(["init", "--dir"])
        .arg(target)
        .status()
        .expect("init");

    let p = target.join(".claude").join("skills").join("bob-stories").join("SKILL.md");
    assert!(p.is_file(), "bob-stories SKILL.md missing at {}", p.display());
    let content = std::fs::read_to_string(&p).unwrap();

    // Frontmatter
    assert!(content.starts_with("---"));
    assert!(content.contains("name: bob-stories"));
    assert!(content.contains("description:"));

    // Load-bearing tokens
    for token in &[
        // CLI
        "/bob-stories",
        "--refactor",
        "--from-survey",
        "--refresh",
        // 三段式 conventions
        "三段式",
        "推测",
        "推荐选择",
        // Stages
        "Stage 0",
        "Stage 1",
        "Stage 2",
        "Stage 3",
        "Stage 4",
        // Mode detection
        "feature",
        "refactor",
        "混合",
        // Output paths
        "docs/bob/02-stories-",
        "docs/bob/02-stories/",
        // Story types in index
        "前置重构 stories",
        "新功能 stories",
        // Identify handoff
        "/bob-identify",
        "--story",
        // Survey input
        "00-survey-",
    ] {
        assert!(content.contains(token), "bob-stories must mention {}", token);
    }
}

#[test]
fn bob_identify_mentions_stories_soft_prompt() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let target = tmp.path();
    std::process::Command::new(run_bob_bin())
        .args(["init", "--dir"])
        .arg(target)
        .status()
        .expect("init");

    let p = target.join(".claude").join("skills").join("bob-identify").join("SKILL.md");
    let content = std::fs::read_to_string(&p).unwrap();

    for token in &[
        "/bob-stories",
        "02-stories-",
        "--story",
    ] {
        assert!(
            content.contains(token),
            "bob-identify must mention {} for stories integration",
            token
        );
    }
}

#[test]
fn bob_stories_mentions_test_coverage_stage() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let target = tmp.path();
    std::process::Command::new(run_bob_bin())
        .args(["init", "--dir"])
        .arg(target)
        .status()
        .expect("init");

    let p = target.join(".claude").join("skills").join("bob-stories").join("SKILL.md");
    let content = std::fs::read_to_string(&p).unwrap();

    for token in &[
        "Stage 2.5",
        "测试覆盖体检",
        "R0.",
        "characterize",
        "全分支覆盖",
        "未覆盖分支",
    ] {
        assert!(
            content.contains(token),
            "bob-stories must mention {} for safety net integration",
            token
        );
    }
}

#[test]
fn bob_identify_refactor_mentions_test_coverage_check() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let target = tmp.path();
    std::process::Command::new(run_bob_bin())
        .args(["init", "--dir"])
        .arg(target)
        .status()
        .expect("init");

    let p = target.join(".claude").join("skills").join("bob-identify").join("SKILL.md");
    let content = std::fs::read_to_string(&p).unwrap();

    for token in &[
        "Step B1.0",
        "测试覆盖现状",
        "分支",
        "测试覆盖警告",
    ] {
        assert!(
            content.contains(token),
            "bob-identify --refactor must mention {} for B1 safety gate",
            token
        );
    }
}

#[test]
fn bob_spec_template_c_mentions_step_0_with_stories_interlock() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let target = tmp.path();
    std::process::Command::new(run_bob_bin())
        .args(["init", "--dir"])
        .arg(target)
        .status()
        .expect("init");

    let p = target.join(".claude").join("skills").join("bob-spec").join("SKILL.md");
    let content = std::fs::read_to_string(&p).unwrap();

    for token in &[
        "Step 0",
        "全分支级",
        "docs/bob/02-stories",
        "characterize",
    ] {
        assert!(
            content.contains(token),
            "bob-spec Template C must mention {} for Step 0 stories interlock",
            token
        );
    }
}

#[test]
fn init_creates_bob_nfr_skill() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let target = tmp.path();
    std::process::Command::new(run_bob_bin())
        .args(["init", "--dir"])
        .arg(target)
        .status()
        .expect("init");

    let p = target.join(".claude").join("skills").join("bob-nfr").join("SKILL.md");
    assert!(p.is_file(), "bob-nfr SKILL.md missing at {}", p.display());
    let content = std::fs::read_to_string(&p).unwrap();

    // Frontmatter
    assert!(content.starts_with("---"));
    assert!(content.contains("name: bob-nfr"));
    assert!(content.contains("description:"));

    // Load-bearing tokens
    for token in &[
        // CLI
        "/bob-nfr",
        "--story",
        "--refresh",
        // 三段式 conventions
        "三段式",
        "推测",
        "推荐选择",
        // Stages (5)
        "Stage 0",
        "Stage 1",
        "Stage 2",
        "Stage 3",
        "Stage 4",
        // 13 NFR cards (English names — Chinese names embedded inline)
        "Performance",
        "Scalability",
        "Capacity",
        "Reliability",
        "Monitoring",
        "Authentication",
        "Authorisation",
        "Security",
        "Data Privacy",
        "Configurability",
        "Extensibility",
        "Portability",
        "Compatibility",
        // Quantification policy
        "量化优先",
        "待定",
        "待量化",
        // Output schema
        "docs/bob/04-nfr-",
        "建议新增 story 清单",
        // Spec / story input
        "<spec-path>",
        "spec",
    ] {
        assert!(content.contains(token), "bob-nfr must mention {}", token);
    }
}

#[test]
fn bob_spec_mentions_nfr_reminder() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let target = tmp.path();
    std::process::Command::new(run_bob_bin())
        .args(["init", "--dir"])
        .arg(target)
        .status()
        .expect("init");

    let p = target.join(".claude").join("skills").join("bob-spec").join("SKILL.md");
    let content = std::fs::read_to_string(&p).unwrap();

    // bob-spec should mention /bob-nfr at least 3 times (one per template's 下一步)
    let count = content.matches("/bob-nfr").count();
    assert!(
        count >= 3,
        "bob-spec must mention /bob-nfr at least 3 times (Template A/B/C reminder); found {}",
        count
    );

    // And the reminder text should hint at "Superpowers TDD ... UT" follow-up
    assert!(
        content.contains("Superpowers TDD") && content.contains("UT"),
        "bob-spec NFR reminder must reference Superpowers TDD + UT completion as the trigger"
    );
}

#[test]
fn init_creates_gitignore_with_run_bob_block() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let target = tmp.path();

    let status = std::process::Command::new(run_bob_bin())
        .args(["init", "--dir"])
        .arg(target)
        .status()
        .expect("init");
    assert!(status.success(), "init failed");

    let gitignore = target.join(".gitignore");
    assert!(gitignore.is_file(), ".gitignore must be created by init");
    let content = std::fs::read_to_string(&gitignore).expect("read .gitignore");
    assert!(
        content.contains("# run-bob"),
        ".gitignore must contain run-bob block header; got:\n{}",
        content
    );
    assert!(
        content.contains(".run-bob-backup/"),
        ".gitignore must contain .run-bob-backup/; got:\n{}",
        content
    );
}

#[test]
fn init_no_gitignore_flag_skips_gitignore() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let target = tmp.path();

    let status = std::process::Command::new(run_bob_bin())
        .args(["init", "--no-gitignore", "--dir"])
        .arg(target)
        .status()
        .expect("init --no-gitignore");
    assert!(status.success());

    assert!(
        !target.join(".gitignore").exists(),
        "--no-gitignore must skip .gitignore creation"
    );
}

#[test]
fn init_help_lists_no_gitignore_flag() {
    let output = std::process::Command::new(run_bob_bin())
        .args(["init", "--help"])
        .output()
        .expect("init --help");
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(
        stdout.contains("--no-gitignore"),
        "init --help must list --no-gitignore; got:\n{}",
        stdout
    );
}

#[test]
fn init_run_twice_keeps_gitignore_byte_identical() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let target = tmp.path();

    std::process::Command::new(run_bob_bin())
        .args(["init", "--dir"])
        .arg(target)
        .status()
        .expect("first init");

    let first = std::fs::read(target.join(".gitignore")).expect("read first .gitignore");

    std::process::Command::new(run_bob_bin())
        .args(["init", "--force", "--dir"])
        .arg(target)
        .status()
        .expect("second init");

    let second = std::fs::read(target.join(".gitignore")).expect("read second .gitignore");
    assert_eq!(first, second, "second init must leave .gitignore byte-identical");
}

#[test]
fn upgrade_creates_gitignore_when_missing() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let target = tmp.path();

    // init first so we have a valid harness layout
    std::process::Command::new(run_bob_bin())
        .args(["init", "--dir"])
        .arg(target)
        .status()
        .expect("init");

    // Then delete .gitignore to simulate an older run-bob install that
    // never wrote one. upgrade must re-create it.
    let gitignore = target.join(".gitignore");
    assert!(gitignore.is_file(), "init should have created .gitignore");
    std::fs::remove_file(&gitignore).expect("remove .gitignore");

    let status = std::process::Command::new(run_bob_bin())
        .args(["upgrade", "--dir"])
        .arg(target)
        .status()
        .expect("upgrade");
    assert!(status.success());

    assert!(gitignore.is_file(), "upgrade must re-create .gitignore");
    let content = std::fs::read_to_string(&gitignore).expect("read");
    assert!(content.contains("# run-bob"));
    assert!(content.contains(".run-bob-backup/"));
}

#[test]
fn upgrade_no_gitignore_flag_leaves_gitignore_alone() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let target = tmp.path();

    std::process::Command::new(run_bob_bin())
        .args(["init", "--dir"])
        .arg(target)
        .status()
        .expect("init");

    let gitignore = target.join(".gitignore");
    std::fs::remove_file(&gitignore).expect("remove for test");
    assert!(!gitignore.exists());

    let status = std::process::Command::new(run_bob_bin())
        .args(["upgrade", "--no-gitignore", "--dir"])
        .arg(target)
        .status()
        .expect("upgrade --no-gitignore");
    assert!(status.success());

    assert!(
        !gitignore.exists(),
        "--no-gitignore must not re-create .gitignore"
    );
}

#[test]
fn upgrade_help_lists_no_gitignore_flag() {
    let output = std::process::Command::new(run_bob_bin())
        .args(["upgrade", "--help"])
        .output()
        .expect("upgrade --help");
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(
        stdout.contains("--no-gitignore"),
        "upgrade --help must list --no-gitignore; got:\n{}",
        stdout
    );
}

#[test]
fn upgrade_dry_run_does_not_create_gitignore() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let target = tmp.path();

    std::process::Command::new(run_bob_bin())
        .args(["init", "--dir"])
        .arg(target)
        .status()
        .expect("init");

    std::fs::remove_file(target.join(".gitignore")).expect("remove");

    let status = std::process::Command::new(run_bob_bin())
        .args(["upgrade", "--dry-run", "--dir"])
        .arg(target)
        .status()
        .expect("upgrade --dry-run");
    assert!(status.success());

    assert!(
        !target.join(".gitignore").exists(),
        "dry-run must not touch .gitignore"
    );
}

#[test]
fn init_appends_to_existing_gitignore_preserving_content() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let target = tmp.path();

    // Pre-existing .gitignore (e.g. user's Maven project setup).
    std::fs::write(target.join(".gitignore"), "target/\n*.log\n").expect("write pre-existing");

    let status = std::process::Command::new(run_bob_bin())
        .args(["init", "--dir"])
        .arg(target)
        .status()
        .expect("init");
    assert!(status.success());

    let content = std::fs::read_to_string(target.join(".gitignore")).expect("read");
    // User content preserved.
    assert!(content.contains("target/\n"), "user line target/ must survive");
    assert!(content.contains("*.log\n"), "user line *.log must survive");
    // Run-bob block added.
    assert!(content.contains("# run-bob\n.run-bob-backup/"), "run-bob block must be present");
    // Block is separated from user content by a blank line.
    assert!(
        content.contains("*.log\n\n# run-bob"),
        "must have blank line separator between user content and block; got:\n{}",
        content
    );
}

#[test]
fn init_creates_compliance_readme_with_pmd_note() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let target = tmp.path();
    std::process::Command::new(run_bob_bin())
        .args(["init", "--dir"])
        .arg(target)
        .status()
        .expect("init");

    let p = target.join("docs").join("compliance").join("README.md");
    assert!(p.is_file(), "compliance/README.md missing at {}", p.display());
    let content = std::fs::read_to_string(&p).expect("read README");

    // Load-bearing tokens — the 3-section contract
    for token in &[
        // Section 1: 用法
        "项目级合规检查",
        "sources/",
        "/bob-compliance",
        "PDF",
        "docx",
        // Section 2: PMD/SonarQube 例外
        "PMD",
        "SonarQube",
        "保持",
        "为空",
        // Section 3: 缓存
        ".compliance.lock",
        "sha256",
        "filename",
    ] {
        assert!(
            content.contains(token),
            "compliance/README.md must mention {}; got:\n{}",
            token,
            content
        );
    }
}

#[test]
fn init_creates_bob_compliance_skill() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let target = tmp.path();
    std::process::Command::new(run_bob_bin())
        .args(["init", "--dir"])
        .arg(target)
        .status()
        .expect("init");

    let p = target
        .join(".claude")
        .join("skills")
        .join("bob-compliance")
        .join("SKILL.md");
    assert!(p.is_file(), "bob-compliance SKILL.md missing at {}", p.display());
    let content = std::fs::read_to_string(&p).expect("read skill");

    // Frontmatter
    assert!(content.starts_with("---"), "must start with YAML frontmatter");
    assert!(content.contains("name: bob-compliance"), "frontmatter name");
    assert!(content.contains("description:"), "frontmatter description");

    // Stage 0 tokens (will be extended in Tasks 4 and 5 with additional stage tokens)
    for token in &[
        // CLI / invocation
        "/bob-compliance",
        "--story",
        "--refresh",
        "--all-branch",
        // 三段式
        "三段式",
        "推测",
        "推荐选择",
        // Stage 0
        "Stage 0",
        "状态探测",
        "空仓",
        "首次",
        "漂移",
        "冷藏",
        "docs/compliance/sources/",
        ".compliance.lock",
        "sha256",
        // Stage 1 — generation
        "Stage 1",
        "结构化",
        "规则 ID",
        "强制",
        "推荐",
        "参考",
        "frontmatter",
        "alibaba-songshan",
        "ALI-1.1.2",  // sample rule ID from the schema example
        "generated_to",
        // Stage 2 — load
        "Stage 2",
        "装载",
        "规则索引",
        "Severity",
        // Stage 3 — diff check
        "Stage 3",
        "diff",
        "违反",
        "待量化",
        "豁免",
        // Diff scope priority
        "git diff",
        "master..HEAD",
        // Stage 4 — report + handoff
        "Stage 4",
        "docs/bob/05-compliance-",
        "建议新增 story 清单",
        "/bob-stories",
        // Cross-skill handoff
        "/bob-nfr",
    ] {
        assert!(content.contains(token), "bob-compliance must mention {}", token);
    }
}

#[test]
fn claude_md_has_r13_compliance() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let target = tmp.path();
    std::process::Command::new(run_bob_bin())
        .args(["init", "--dir"])
        .arg(target)
        .status()
        .expect("init");

    let p = target.join("CLAUDE.md");
    assert!(p.is_file(), "CLAUDE.md missing");
    let content = std::fs::read_to_string(&p).expect("read CLAUDE.md");

    for token in &[
        "R13",
        "合规规则前置",
        "docs/compliance/",
        "/bob-compliance",
        "【强制】",
        "规则 ID",
        "豁免",
    ] {
        assert!(
            content.contains(token),
            "CLAUDE.md must contain {} for R13; got:\n{}",
            token,
            content
        );
    }
}
