//! Integration tests for run-bob.
//! Each test creates a tempdir, runs init/status, and verifies filesystem state.

use std::collections::BTreeMap;
use std::path::Path;
use std::process::{Command, Output};

const GENERATED_SKILLS: &[&str] = &[
    "bob-survey",
    "bob-model",
    "bob-stories",
    "bob-identify",
    "bob-onion",
    "bob-spec",
    "bob-compliance",
    "bob-nfr",
    "visual-md",
];

const SKILL_ROOTS: &[&str] = &[".claude/skills", ".agents/skills"];

fn files_below(root: &Path) -> BTreeMap<String, (Vec<u8>, Option<u32>)> {
    fn walk(root: &Path, dir: &Path, files: &mut BTreeMap<String, (Vec<u8>, Option<u32>)>) {
        for entry in std::fs::read_dir(dir).expect("read skill directory") {
            let entry = entry.expect("read skill directory entry");
            let file_type = entry.file_type().expect("read skill entry type");
            let path = entry.path();

            if file_type.is_dir() {
                walk(root, &path, files);
            } else if file_type.is_file() {
                let relative = path
                    .strip_prefix(root)
                    .expect("skill file below root")
                    .to_string_lossy()
                    .replace('\\', "/");
                let content = std::fs::read(&path).expect("read skill file");

                #[cfg(unix)]
                let unix_mode = {
                    use std::os::unix::fs::PermissionsExt;
                    Some(
                        std::fs::metadata(&path)
                            .expect("read skill file metadata")
                            .permissions()
                            .mode()
                            & 0o7777,
                    )
                };

                #[cfg(not(unix))]
                let unix_mode = None;

                files.insert(relative, (content, unix_mode));
            }
        }
    }

    let mut files = BTreeMap::new();
    walk(root, root, &mut files);
    files
}

/// Path to the cargo-built binary under test.
fn run_bob_bin() -> std::path::PathBuf {
    // CARGO_BIN_EXE_<name> is set by Cargo for integration tests.
    std::path::PathBuf::from(env!("CARGO_BIN_EXE_run-bob"))
}

fn assert_command_succeeded(output: &Output, operation: &str) {
    assert!(
        output.status.success(),
        "{operation} failed with {}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

fn assert_dual_host_next_steps(output: &Output, mode: &str) {
    let stdout = String::from_utf8_lossy(&output.stdout);
    let Some((_, next_steps)) = stdout.split_once("Next steps") else {
        panic!("{mode} init output missing the Next steps marker\nstdout:\n{stdout}");
    };
    for expected in [
        "Claude Code:",
        "Codex:",
        "/bob-survey <your business description>",
        "/bob-model <doc-path>",
        "$bob-survey <your business description>",
        "$bob-model <doc-path>",
        "optional",
        "mandatory",
    ] {
        assert!(
            next_steps.contains(expected),
            "{mode} init next steps missing {expected:?}\nstdout:\n{stdout}"
        );
    }
}

fn legacy_claude_only_fixture(target: &Path) -> BTreeMap<String, (Vec<u8>, Option<u32>)> {
    let output = Command::new(run_bob_bin())
        .args(["init", "--dir"])
        .arg(target)
        .output()
        .expect("initialize legacy fixture");
    assert_command_succeeded(&output, "legacy fixture init");

    let claude_snapshot = files_below(&target.join(SKILL_ROOTS[0]));
    std::fs::remove_dir_all(target.join(SKILL_ROOTS[1]))
        .expect("remove only the Codex skill tree from legacy fixture");
    assert!(
        target.join(".agents").is_dir(),
        "legacy fixture keeps .agents"
    );
    assert!(
        !target.join(SKILL_ROOTS[1]).exists(),
        "legacy fixture must be Claude-only"
    );
    claude_snapshot
}

fn command_output_text(output: &Output) -> String {
    format!(
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )
}

#[cfg(unix)]
fn assert_symlink_unchanged(path: &Path, expected_target: &Path) {
    let metadata = std::fs::symlink_metadata(path).expect("stat preserved symlink");
    assert!(
        metadata.file_type().is_symlink(),
        "{} is no longer a symlink",
        path.display()
    );
    assert_eq!(
        std::fs::read_link(path).expect("read preserved symlink"),
        expected_target,
        "{} now points to a different target",
        path.display()
    );
}

#[test]
fn asset_registry_has_identical_claude_and_codex_skill_entries() {
    use std::collections::BTreeSet;

    use run_bob::assets::HARNESS_ASSETS;

    let expected_tails = [
        "bob-identify/SKILL.md",
        "bob-onion/SKILL.md",
        "bob-spec/SKILL.md",
        "bob-survey/SKILL.md",
        "bob-stories/SKILL.md",
        "bob-nfr/SKILL.md",
        "bob-compliance/SKILL.md",
        "bob-model/SKILL.md",
        "bob-model/scripts/server.cjs",
        "bob-model/scripts/helper.js",
        "bob-model/scripts/start-server.sh",
        "bob-model/scripts/stop-server.sh",
        "bob-model/scripts/frame-template.html",
        "visual-md/SKILL.md",
        "visual-md/scripts/server.cjs",
        "visual-md/scripts/client.js",
        "visual-md/scripts/md2html.cjs",
        "visual-md/scripts/slugify.cjs",
        "visual-md/scripts/frame-template.html",
        "visual-md/scripts/start-server.sh",
        "visual-md/scripts/stop-server.sh",
        "visual-md/scripts/package.json",
    ];

    let entries_for = |host: &str| {
        HARNESS_ASSETS
            .iter()
            .filter(|asset| asset.rel_path.starts_with(&[host, "skills"]))
            .map(|asset| {
                let tail = asset.rel_path[2..].join("/");
                let shell_file = asset.rel_path.last().is_some_and(|name| name.ends_with(".sh"));
                (
                    tail,
                    (
                        asset.content.as_bytes().to_vec(),
                        asset.included_in_minimal,
                        asset.upgrade_safe,
                        shell_file,
                    ),
                )
            })
            .collect::<BTreeMap<_, _>>()
    };

    let claude = entries_for(".claude");
    let codex = entries_for(".agents");
    let expected_tails = expected_tails.into_iter().collect::<BTreeSet<_>>();

    assert_eq!(
        claude.keys().map(String::as_str).collect::<BTreeSet<_>>(),
        expected_tails,
        "Claude registry tails drifted"
    );
    assert_eq!(
        codex.keys().map(String::as_str).collect::<BTreeSet<_>>(),
        expected_tails
    );
    assert_eq!(claude, codex, "Claude and Codex registry entries must match");
}

#[test]
fn init_installs_byte_identical_dual_skill_trees() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let target = tmp.path();

    let output = Command::new(run_bob_bin())
        .args(["init", "--dir"])
        .arg(target)
        .output()
        .expect("init");
    assert_command_succeeded(&output, "full init");
    assert_dual_host_next_steps(&output, "full");

    let claude = files_below(&target.join(SKILL_ROOTS[0]));
    let codex = files_below(&target.join(SKILL_ROOTS[1]));
    assert_eq!(
        claude, codex,
        "installed skill trees must be byte- and mode-identical"
    );
}

#[test]
fn minimal_init_installs_all_nine_skills_for_both_hosts_only() {
    use std::collections::BTreeSet;

    let tmp = tempfile::tempdir().expect("tempdir");
    let target = tmp.path();

    let output = Command::new(run_bob_bin())
        .args(["init", "--minimal", "--no-gitignore", "--dir"])
        .arg(target)
        .output()
        .expect("minimal init");
    assert_command_succeeded(&output, "minimal init");
    assert_dual_host_next_steps(&output, "minimal");

    let expected_skills = GENERATED_SKILLS.iter().copied().collect::<BTreeSet<_>>();
    for root in SKILL_ROOTS {
        let actual_skills = std::fs::read_dir(target.join(root))
            .expect("read generated skill root")
            .map(|entry| {
                entry
                    .expect("read generated skill")
                    .file_name()
                    .into_string()
                    .expect("UTF-8 skill name")
            })
            .collect::<BTreeSet<_>>();
        assert_eq!(
            actual_skills.iter().map(String::as_str).collect::<BTreeSet<_>>(),
            expected_skills,
            "minimal init generated the wrong skill set under {root}"
        );
        for skill in GENERATED_SKILLS {
            assert!(
                target.join(root).join(skill).join("SKILL.md").is_file(),
                "minimal init did not install {root}/{skill}/SKILL.md"
            );
        }
    }

    let top_level = std::fs::read_dir(target)
        .expect("read target")
        .map(|entry| {
            entry
                .expect("read target entry")
                .file_name()
                .into_string()
                .expect("UTF-8 target entry")
        })
        .collect::<BTreeSet<_>>();
    assert_eq!(
        top_level,
        BTreeSet::from([".agents".to_string(), ".claude".to_string()])
    );
}

#[test]
fn init_without_force_preserves_existing_claude_skill_and_adds_codex() {
    use run_bob::assets::HARNESS_ASSETS;

    let tmp = tempfile::tempdir().expect("tempdir");
    let target = tmp.path();
    let claude_skill = target.join(".claude/skills/bob-identify/SKILL.md");
    std::fs::create_dir_all(claude_skill.parent().expect("Claude skill parent"))
        .expect("create existing Claude skill directory");
    let sentinel = b"user-owned Claude skill\n";
    std::fs::write(&claude_skill, sentinel).expect("write Claude skill sentinel");

    let output = Command::new(run_bob_bin())
        .args(["init", "--no-gitignore", "--dir"])
        .arg(target)
        .output()
        .expect("init");
    assert_command_succeeded(&output, "init without force");

    let embedded = HARNESS_ASSETS
        .iter()
        .find(|asset| asset.rel_path == [".claude", "skills", "bob-identify", "SKILL.md"])
        .expect("embedded bob-identify asset")
        .content
        .as_bytes();
    assert_eq!(
        std::fs::read(&claude_skill).expect("read Claude sentinel"),
        sentinel
    );
    assert_eq!(
        std::fs::read(target.join(".agents/skills/bob-identify/SKILL.md"))
            .expect("read installed Codex skill"),
        embedded
    );
}

#[test]
fn binary_prints_version() {
    let output = Command::new(run_bob_bin())
        .arg("--version")
        .output()
        .expect("run run-bob --version");
    assert!(output.status.success(), "run-bob --version failed");
    let stdout = String::from_utf8(output.stdout).unwrap();
    let expected = format!("run-bob {}", env!("CARGO_PKG_VERSION"));
    assert!(
        stdout.contains(&expected),
        "expected '{}' in output, got: {}",
        expected,
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
    for flag in &["--force", "--minimal", "--dir", "--with-java"] {
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

    Command::new(run_bob_bin())
        .args(["init", "--with-java", "--dir"])
        .arg(target)
        .status()
        .expect("init");

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

    Command::new(run_bob_bin())
        .args(["init", "--with-java", "--dir"])
        .arg(target)
        .status()
        .expect("init");

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

    Command::new(run_bob_bin())
        .args(["init", "--with-java", "--dir"])
        .arg(target)
        .status()
        .expect("init");

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
fn init_default_skips_java_skeleton() {
    // Plain `run-bob init` (no flags) must NOT create the Java/Maven skeleton.
    // Java is opt-in via --with-java now.
    let tmp = tempfile::tempdir().expect("tempdir");
    let target = tmp.path();

    Command::new(run_bob_bin())
        .args(["init", "--dir"])
        .arg(target)
        .status()
        .expect("init");

    // No ArchUnit test
    assert!(
        !target.join("src/test/java/architecture/CleanArchitectureTest.java").exists(),
        "default init must NOT install ArchUnit test"
    );
    // No shared Java skeleton
    assert!(
        !target.join("src/main/java/com/example/shared/usecase/UseCase.java").exists(),
        "default init must NOT install UseCase.java"
    );
    assert!(
        !target.join("src/main/java/com/example/shared/framework/transaction/TransactionalUseCaseDecorator.java").exists(),
        "default init must NOT install TransactionalUseCaseDecorator.java"
    );
    // No src/ at all, because nothing under src/ was written
    assert!(!target.join("src").exists(), "default init must NOT create src/ at all");

    // But docs/skills are still installed
    assert!(target.join("CLAUDE.md").is_file(), "default init must install CLAUDE.md");
    assert!(
        target.join(".claude/skills/bob-identify/SKILL.md").is_file(),
        "default init must install skills"
    );
}

#[test]
fn init_with_java_installs_skeleton() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let target = tmp.path();

    Command::new(run_bob_bin())
        .args(["init", "--with-java", "--dir"])
        .arg(target)
        .status()
        .expect("init --with-java");

    assert!(
        target.join("src/test/java/architecture/CleanArchitectureTest.java").is_file(),
        "--with-java must install ArchUnit test"
    );
    assert!(
        target.join("src/main/java/com/example/shared/usecase/UseCase.java").is_file(),
        "--with-java must install UseCase.java"
    );
    assert!(
        target.join("src/main/java/com/example/shared/framework/transaction/TransactionalUseCaseDecorator.java").is_file(),
        "--with-java must install TransactionalUseCaseDecorator.java"
    );
}

#[test]
fn status_complete_on_non_java_target_after_default_init() {
    // After plain `init`, status must still report "harness is complete" —
    // Java assets are silently skipped because the target has no src/main/java.
    let tmp = tempfile::tempdir().expect("tempdir");
    let target = tmp.path();
    Command::new(run_bob_bin())
        .args(["init", "--dir"])
        .arg(target)
        .status()
        .expect("init");

    let output = Command::new(run_bob_bin())
        .args(["status", "--dir"])
        .arg(target)
        .output()
        .expect("status");
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert!(
        stdout.contains("harness is complete"),
        "default init + status must report complete on non-Java target; got:\n{}",
        stdout
    );
    // Java tokens must NOT appear in output (not checked at all)
    for absent in &["CleanArchitectureTest.java", "UseCase.java", "TransactionalUseCaseDecorator.java"] {
        assert!(
            !stdout.contains(absent),
            "status must not mention {} on non-Java target; got:\n{}",
            absent,
            stdout
        );
    }
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
    Command::new(run_bob_bin())
        .args(["init", "--with-java", "--dir"])
        .arg(target)
        .status()
        .expect("init");

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
///   - Claude/Codex skills + SharedJava → always upgrade-safe (pure generated content)
///   - ArchUnit          → never upgrade-safe (FORBIDDEN_IN_INNER is user-edited)
///   - HarnessDoc        → mixed (README is safe; CLAUDE / ARCHITECTURE are not)
#[test]
fn upgrade_safe_field_matches_category_policy() {
    use run_bob::assets::{Category, HARNESS_ASSETS};

    for asset in HARNESS_ASSETS {
        let display = asset.rel_path.join("/");
        if asset.category.is_skill() {
            assert!(
                asset.upgrade_safe,
                "{} is a skill but upgrade_safe=false",
                display
            );
            continue;
        }

        match asset.category {
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
            Category::ClaudeSkill | Category::CodexSkill => {
                unreachable!("skill categories handled above")
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

    // --with-java needed so the ArchUnit user-owned file actually exists.
    std::process::Command::new(run_bob_bin())
        .args(["init", "--with-java", "--dir"])
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
fn upgrade_dry_run_from_claude_only_writes_nothing() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let target = tmp.path();
    let claude_before = legacy_claude_only_fixture(target);

    let output = Command::new(run_bob_bin())
        .args(["upgrade", "--dry-run", "--dir"])
        .arg(target)
        .output()
        .expect("upgrade legacy fixture in dry-run mode");
    assert_command_succeeded(&output, "legacy dry-run upgrade");

    assert!(
        !target.join(SKILL_ROOTS[1]).exists(),
        "dry-run must not create the Codex skill tree"
    );
    assert_eq!(
        files_below(&target.join(SKILL_ROOTS[0])),
        claude_before,
        "dry-run must preserve all Claude bytes and executable bits"
    );
    assert!(
        !target.join(".run-bob-backup").exists(),
        "dry-run must not create backups"
    );
}

#[test]
fn upgrade_from_claude_only_adds_codex_without_touching_claude() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let target = tmp.path();
    let claude_before = legacy_claude_only_fixture(target);

    let output = Command::new(run_bob_bin())
        .args(["upgrade", "--dir"])
        .arg(target)
        .output()
        .expect("upgrade legacy fixture");
    assert_command_succeeded(&output, "legacy upgrade");

    let claude_after = files_below(&target.join(SKILL_ROOTS[0]));
    let codex_after = files_below(&target.join(SKILL_ROOTS[1]));
    assert_eq!(
        claude_after, claude_before,
        "upgrade changed the Claude tree"
    );
    assert_eq!(
        codex_after.len(),
        22,
        "upgrade must restore all 22 Codex files"
    );
    assert_eq!(
        codex_after, claude_before,
        "restored Codex files must match the existing Claude files byte-for-byte and mode-for-mode"
    );
    assert!(
        !target.join(".run-bob-backup").exists(),
        "an all-MISSING migration must not create a backup"
    );
}

#[test]
fn upgrade_backs_up_stale_files_for_both_skill_hosts() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let target = tmp.path();
    let claude_path = target.join(".claude/skills/bob-identify/SKILL.md");
    let codex_path = target.join(".agents/skills/bob-identify/SKILL.md");

    let output = Command::new(run_bob_bin())
        .args(["init", "--dir"])
        .arg(target)
        .output()
        .expect("init");
    assert_command_succeeded(&output, "dual-host init");
    let expected = std::fs::read(&claude_path).expect("read embedded skill");
    let claude_sentinel = b"STALE-CLAUDE\n";
    let codex_sentinel = b"STALE-CODEX\n";
    std::fs::write(&claude_path, claude_sentinel).expect("stale Claude skill");
    std::fs::write(&codex_path, codex_sentinel).expect("stale Codex skill");

    let output = Command::new(run_bob_bin())
        .args(["upgrade", "--dir"])
        .arg(target)
        .output()
        .expect("upgrade stale dual-host fixture");
    assert_command_succeeded(&output, "dual-host stale upgrade");

    let timestamp_dirs = std::fs::read_dir(target.join(".run-bob-backup"))
        .expect("read backup root")
        .map(|entry| entry.expect("backup timestamp entry").path())
        .collect::<Vec<_>>();
    assert_eq!(
        timestamp_dirs.len(),
        1,
        "one shared timestamp directory expected"
    );
    let backup_root = &timestamp_dirs[0];
    assert_eq!(
        std::fs::read(backup_root.join(".claude/skills/bob-identify/SKILL.md"))
            .expect("read Claude backup"),
        claude_sentinel
    );
    assert_eq!(
        std::fs::read(backup_root.join(".agents/skills/bob-identify/SKILL.md"))
            .expect("read Codex backup"),
        codex_sentinel
    );
    assert_eq!(
        std::fs::read(&claude_path).expect("read restored Claude"),
        expected
    );
    assert_eq!(
        std::fs::read(&codex_path).expect("read restored Codex"),
        expected
    );
}

#[cfg(unix)]
#[test]
fn upgrade_refuses_symlinked_backup_root_without_following() {
    use std::os::unix::fs::symlink;

    for dangling in [false, true] {
        let tmp = tempfile::tempdir().expect("backup symlink tempdir");
        let target = tmp.path().join("project");
        std::fs::create_dir(&target).expect("create project target");
        let output = Command::new(run_bob_bin())
            .args(["init", "--dir"])
            .arg(&target)
            .output()
            .expect("init backup fixture");
        assert_command_succeeded(&output, "backup fixture init");

        let stale_skill = target.join(".claude/skills/bob-identify/SKILL.md");
        std::fs::write(&stale_skill, b"STALE-BEFORE-BACKUP\n").expect("write stale skill");

        let external = tmp.path().join(if dangling {
            "missing-external-backup-root"
        } else {
            "external-backup-root"
        });
        let external_before = if dangling {
            None
        } else {
            std::fs::create_dir(&external).expect("create external backup root");
            std::fs::write(external.join("sentinel"), b"EXTERNAL-BACKUP\n")
                .expect("write external backup sentinel");
            Some(files_below(&external))
        };
        let backup_link = target.join(".run-bob-backup");
        symlink(&external, &backup_link).expect("create backup-root symlink");
        let original_link_target = std::fs::read_link(&backup_link).expect("read backup symlink");

        let output = Command::new(run_bob_bin())
            .args(["upgrade", "--no-gitignore", "--dir"])
            .arg(&target)
            .output()
            .expect("upgrade with symlinked backup root");
        assert!(
            !output.status.success(),
            "upgrade followed the backup-root symlink: {}",
            command_output_text(&output)
        );
        assert!(
            command_output_text(&output).contains(".run-bob-backup"),
            "upgrade error omitted the backup-root conflict: {}",
            command_output_text(&output)
        );
        assert_symlink_unchanged(&backup_link, &original_link_target);
        if let Some(external_before) = external_before {
            assert_eq!(
                files_below(&external),
                external_before,
                "upgrade wrote through the backup-root symlink"
            );
        } else {
            assert!(
                !external.exists(),
                "upgrade created the dangling backup-root target"
            );
        }
        assert_eq!(
            std::fs::read(&stale_skill).expect("read stale skill after failed upgrade"),
            b"STALE-BEFORE-BACKUP\n",
            "upgrade overwrote a managed file after backup preflight failed"
        );
    }
}

#[test]
fn upgrade_missing_shell_scripts_are_executable_for_both_hosts() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let target = tmp.path();
    let output = Command::new(run_bob_bin())
        .args(["init", "--dir"])
        .arg(target)
        .output()
        .expect("init");
    assert_command_succeeded(&output, "dual-host init");

    let scripts = SKILL_ROOTS
        .iter()
        .flat_map(|root| {
            ["bob-model", "visual-md"].map(move |skill| {
                target
                    .join(root)
                    .join(skill)
                    .join("scripts/start-server.sh")
            })
        })
        .collect::<Vec<_>>();
    for script in &scripts {
        std::fs::remove_file(script).expect("remove generated shell script");
    }

    let output = Command::new(run_bob_bin())
        .args(["upgrade", "--dir"])
        .arg(target)
        .output()
        .expect("upgrade missing shell scripts");
    assert_command_succeeded(&output, "upgrade missing shell scripts");

    for script in scripts {
        assert!(
            script.is_file(),
            "upgrade did not restore {}",
            script.display()
        );
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mode = std::fs::metadata(&script)
                .expect("stat restored shell script")
                .permissions()
                .mode();
            assert_ne!(
                mode & 0o111,
                0,
                "restored shell script is not executable: {} ({mode:o})",
                script.display()
            );
        }
    }
}

#[test]
fn init_and_upgrade_reject_wrong_kind_destinations_before_writing() {
    let init_tmp = tempfile::tempdir().expect("init tempdir");
    let init_target = init_tmp.path();
    let conflict = init_target.join(".agents/skills/bob-identify/SKILL.md");
    std::fs::create_dir_all(&conflict).expect("create directory at managed file path");

    let output = Command::new(run_bob_bin())
        .args(["init", "--force", "--no-gitignore", "--dir"])
        .arg(init_target)
        .output()
        .expect("forced init with wrong-kind destination");
    assert!(
        !output.status.success(),
        "wrong-kind forced init unexpectedly succeeded"
    );
    assert!(
        command_output_text(&output).contains(".agents/skills/bob-identify/SKILL.md"),
        "init error omitted exact conflict path: {}",
        command_output_text(&output)
    );
    assert!(
        !init_target
            .join(".claude/skills/bob-identify/SKILL.md")
            .exists(),
        "init wrote an earlier managed asset before rejecting a later conflict"
    );

    let upgrade_tmp = tempfile::tempdir().expect("upgrade tempdir");
    let upgrade_target = upgrade_tmp.path();
    let claude_before = legacy_claude_only_fixture(upgrade_target);
    std::fs::create_dir_all(upgrade_target.join(".agents/skills"))
        .expect("create Codex skill root");
    let ancestor_conflict = upgrade_target.join(".agents/skills/bob-model");
    std::fs::write(&ancestor_conflict, b"not a directory\n")
        .expect("create file at required parent path");

    let output = Command::new(run_bob_bin())
        .args(["upgrade", "--no-gitignore", "--dir"])
        .arg(upgrade_target)
        .output()
        .expect("upgrade with wrong-kind ancestor");
    assert!(
        !output.status.success(),
        "wrong-kind upgrade unexpectedly succeeded"
    );
    assert!(
        command_output_text(&output).contains(".agents/skills/bob-model"),
        "upgrade error omitted exact ancestor conflict path: {}",
        command_output_text(&output)
    );
    assert!(
        !upgrade_target
            .join(".agents/skills/bob-identify/SKILL.md")
            .exists(),
        "upgrade installed an earlier missing asset before rejecting a later conflict"
    );
    assert_eq!(
        files_below(&upgrade_target.join(SKILL_ROOTS[0])),
        claude_before,
        "failed upgrade changed the Claude tree"
    );
}

#[cfg(unix)]
#[test]
fn init_force_and_upgrade_refuse_symlinked_destinations_without_following() {
    use std::os::unix::fs::symlink;

    for dangling in [false, true] {
        let tmp = tempfile::tempdir().expect("init symlink tempdir");
        let target = tmp.path().join("project");
        std::fs::create_dir(&target).expect("create init target");
        let external = tmp.path().join(if dangling {
            "missing-init-target"
        } else {
            "external-init-target"
        });
        if !dangling {
            std::fs::write(&external, b"EXTERNAL-INIT\n").expect("write external init target");
        }
        let conflict = target.join(".agents/skills/bob-identify/SKILL.md");
        std::fs::create_dir_all(conflict.parent().expect("conflict parent"))
            .expect("create managed parent");
        symlink(&external, &conflict).expect("create init symlink conflict");
        let original_link_target = std::fs::read_link(&conflict).expect("read init symlink");

        let output = Command::new(run_bob_bin())
            .args(["init", "--force", "--no-gitignore", "--dir"])
            .arg(&target)
            .output()
            .expect("forced init with symlink conflict");
        assert!(
            !output.status.success(),
            "forced init followed a symlink: {}",
            command_output_text(&output)
        );
        assert!(
            command_output_text(&output).contains(".agents/skills/bob-identify/SKILL.md"),
            "forced init error omitted symlink path: {}",
            command_output_text(&output)
        );
        assert_symlink_unchanged(&conflict, &original_link_target);
        if dangling {
            assert!(
                !external.exists(),
                "forced init created a dangling link target"
            );
        } else {
            assert_eq!(
                std::fs::read(&external).expect("read external init target"),
                b"EXTERNAL-INIT\n"
            );
        }
        assert!(
            !target.join(".claude/skills/bob-identify/SKILL.md").exists(),
            "forced init wrote another host before rejecting the symlink"
        );
    }

    for dangling in [false, true] {
        let tmp = tempfile::tempdir().expect("upgrade symlink tempdir");
        let target = tmp.path().join("project");
        std::fs::create_dir(&target).expect("create upgrade target");
        let claude_before = legacy_claude_only_fixture(&target);
        let external = tmp.path().join(if dangling {
            "missing-upgrade-target"
        } else {
            "external-upgrade-target"
        });
        if !dangling {
            std::fs::write(&external, b"EXTERNAL-UPGRADE\n")
                .expect("write external upgrade target");
        }
        let conflict = target.join(".claude/skills/bob-identify/SKILL.md");
        std::fs::remove_file(&conflict).expect("remove managed file for symlink");
        symlink(&external, &conflict).expect("create upgrade symlink conflict");
        let original_link_target = std::fs::read_link(&conflict).expect("read upgrade symlink");

        let output = Command::new(run_bob_bin())
            .args(["upgrade", "--no-gitignore", "--dir"])
            .arg(&target)
            .output()
            .expect("upgrade with symlink conflict");
        assert!(
            !output.status.success(),
            "upgrade followed a symlink: {}",
            command_output_text(&output)
        );
        assert!(
            command_output_text(&output).contains(".claude/skills/bob-identify/SKILL.md"),
            "upgrade error omitted symlink path: {}",
            command_output_text(&output)
        );
        assert_symlink_unchanged(&conflict, &original_link_target);
        if dangling {
            assert!(!external.exists(), "upgrade created a dangling link target");
        } else {
            assert_eq!(
                std::fs::read(&external).expect("read external upgrade target"),
                b"EXTERNAL-UPGRADE\n"
            );
        }
        assert!(
            !target.join(SKILL_ROOTS[1]).exists(),
            "upgrade installed missing Codex assets before rejecting the symlink"
        );
        let mut expected_claude = claude_before.clone();
        expected_claude.remove("bob-identify/SKILL.md");
        let mut actual_claude = files_below(&target.join(SKILL_ROOTS[0]));
        actual_claude.remove("bob-identify/SKILL.md");
        assert_eq!(
            actual_claude, expected_claude,
            "failed upgrade changed other Claude files"
        );
    }
}

#[cfg(unix)]
#[test]
fn init_force_and_upgrade_refuse_symlinked_ancestor_directories() {
    use std::os::unix::fs::symlink;

    for host in [".claude", ".agents"] {
        for dangling in [false, true] {
            let tmp = tempfile::tempdir().expect("init ancestor-symlink tempdir");
            let target = tmp.path().join("project");
            std::fs::create_dir(&target).expect("create init target");
            let external = tmp.path().join(format!(
                "{}-{}-init-ancestor",
                host.trim_start_matches('.'),
                if dangling { "missing" } else { "external" }
            ));
            let external_before = if dangling {
                None
            } else {
                std::fs::create_dir(&external).expect("create external init directory");
                std::fs::write(external.join("sentinel"), b"EXTERNAL-INIT-DIR\n")
                    .expect("write external init sentinel");
                Some(files_below(&external))
            };
            let conflict = target.join(host).join("skills/bob-identify");
            std::fs::create_dir_all(conflict.parent().expect("ancestor conflict parent"))
                .expect("create managed init parent");
            symlink(&external, &conflict).expect("create init ancestor symlink");
            let original_link_target =
                std::fs::read_link(&conflict).expect("read init ancestor symlink");

            let output = Command::new(run_bob_bin())
                .args(["init", "--force", "--no-gitignore", "--dir"])
                .arg(&target)
                .output()
                .expect("forced init with ancestor symlink");
            assert!(
                !output.status.success(),
                "forced init accepted {host} ancestor symlink: {}",
                command_output_text(&output)
            );
            assert!(
                command_output_text(&output).contains(&format!("{host}/skills/bob-identify")),
                "forced init error omitted ancestor symlink: {}",
                command_output_text(&output)
            );
            assert_symlink_unchanged(&conflict, &original_link_target);
            if let Some(external_before) = external_before {
                assert_eq!(files_below(&external), external_before);
            } else {
                assert!(
                    !external.exists(),
                    "forced init created dangling directory target"
                );
            }
            for root in SKILL_ROOTS {
                assert!(
                    !target.join(root).join("bob-onion/SKILL.md").exists(),
                    "forced init partially wrote {root} before rejecting {host} ancestor symlink"
                );
            }
        }
    }

    for host in [".claude", ".agents"] {
        for dangling in [false, true] {
            let tmp = tempfile::tempdir().expect("upgrade ancestor-symlink tempdir");
            let target = tmp.path().join("project");
            std::fs::create_dir(&target).expect("create upgrade target");
            let claude_before = legacy_claude_only_fixture(&target);
            let external = tmp.path().join(format!(
                "{}-{}-upgrade-ancestor",
                host.trim_start_matches('.'),
                if dangling { "missing" } else { "external" }
            ));
            let external_before = if dangling {
                None
            } else {
                std::fs::create_dir(&external).expect("create external upgrade directory");
                std::fs::write(external.join("sentinel"), b"EXTERNAL-UPGRADE-DIR\n")
                    .expect("write external upgrade sentinel");
                Some(files_below(&external))
            };
            let conflict = target.join(host).join("skills/bob-identify");
            if conflict.is_dir() {
                std::fs::remove_dir_all(&conflict).expect("remove managed skill directory");
            } else {
                std::fs::create_dir_all(conflict.parent().expect("ancestor conflict parent"))
                    .expect("create managed upgrade parent");
            }
            symlink(&external, &conflict).expect("create upgrade ancestor symlink");
            let original_link_target =
                std::fs::read_link(&conflict).expect("read upgrade ancestor symlink");

            let output = Command::new(run_bob_bin())
                .args(["upgrade", "--no-gitignore", "--dir"])
                .arg(&target)
                .output()
                .expect("upgrade with ancestor symlink");
            assert!(
                !output.status.success(),
                "upgrade accepted {host} ancestor symlink: {}",
                command_output_text(&output)
            );
            assert!(
                command_output_text(&output).contains(&format!("{host}/skills/bob-identify")),
                "upgrade error omitted ancestor symlink: {}",
                command_output_text(&output)
            );
            assert_symlink_unchanged(&conflict, &original_link_target);
            if let Some(external_before) = external_before {
                assert_eq!(files_below(&external), external_before);
            } else {
                assert!(
                    !external.exists(),
                    "upgrade created dangling directory target"
                );
            }
            assert!(
                !target.join(".agents/skills/bob-onion/SKILL.md").exists(),
                "upgrade installed a missing Codex asset before rejecting {host} ancestor symlink"
            );
            let mut expected_claude = claude_before.clone();
            if host == ".claude" {
                expected_claude.remove("bob-identify/SKILL.md");
            }
            assert_eq!(
                files_below(&target.join(SKILL_ROOTS[0])),
                expected_claude,
                "failed upgrade changed another Claude file"
            );
        }
    }
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

#[test]
fn bob_spec_mentions_compliance_in_all_three_templates() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let target = tmp.path();
    std::process::Command::new(run_bob_bin())
        .args(["init", "--dir"])
        .arg(target)
        .status()
        .expect("init");

    let p = target.join(".claude").join("skills").join("bob-spec").join("SKILL.md");
    let content = std::fs::read_to_string(&p).expect("read bob-spec");

    // bob-spec must mention /bob-compliance at least 3 times (one per template's 下一步)
    let count = content.matches("/bob-compliance").count();
    assert!(
        count >= 3,
        "bob-spec must mention /bob-compliance at least 3 times (Template A/B/C reminder); found {}",
        count
    );

    // And the reminder text should hint at "先 /bob-compliance 再 /bob-nfr" ordering
    assert!(
        content.contains("docs/compliance/sources/"),
        "bob-spec compliance reminder must reference the sources/ directory"
    );
}

#[test]
fn init_creates_compliance_dir_and_sources() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let target = tmp.path();
    std::process::Command::new(run_bob_bin())
        .args(["init", "--dir"])
        .arg(target)
        .status()
        .expect("init");

    assert!(
        target.join("docs").join("compliance").is_dir(),
        "docs/compliance/ must be created"
    );
    assert!(
        target.join("docs").join("compliance").join("sources").is_dir(),
        "docs/compliance/sources/ must be created"
    );
    assert!(
        target.join("docs").join("compliance").join("README.md").is_file(),
        "docs/compliance/README.md must be installed"
    );

    // sources/ must be empty (user populates it)
    let entries: Vec<_> = std::fs::read_dir(target.join("docs").join("compliance").join("sources"))
        .expect("read_dir sources")
        .filter_map(|e| e.ok())
        .collect();
    assert!(
        entries.is_empty(),
        "docs/compliance/sources/ must be empty after init; got {} entries",
        entries.len()
    );

    // No generated files / lock yet (those are runtime products of /bob-compliance)
    assert!(
        !target.join("docs").join("compliance").join(".compliance.lock").exists(),
        ".compliance.lock must NOT exist after init"
    );
}

#[test]
fn init_minimal_skips_compliance_dir_but_installs_skill() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let target = tmp.path();
    std::process::Command::new(run_bob_bin())
        .args(["init", "--minimal", "--dir"])
        .arg(target)
        .status()
        .expect("init --minimal");

    // Skill must still be installed (skills always included in minimal)
    let skill = target
        .join(".claude")
        .join("skills")
        .join("bob-compliance")
        .join("SKILL.md");
    assert!(
        skill.is_file(),
        "minimal must install bob-compliance skill"
    );

    // But the compliance/ dir and README must NOT be created
    assert!(
        !target.join("docs").join("compliance").exists(),
        "minimal must NOT create docs/compliance/"
    );
}

#[test]
fn upgrade_preserves_user_compliance_sources() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let target = tmp.path();

    // Fresh init
    std::process::Command::new(run_bob_bin())
        .args(["init", "--dir"])
        .arg(target)
        .status()
        .expect("init");

    // User drops a "compliance source file" into sources/
    let user_source = target
        .join("docs")
        .join("compliance")
        .join("sources")
        .join("my-team-rules.md");
    let sentinel = "USER COMPLIANCE SOURCE — must survive upgrade\n";
    std::fs::write(&user_source, sentinel).expect("write user source");

    // User simulates a runtime product: a generated md and a lock file
    let generated = target
        .join("docs")
        .join("compliance")
        .join("my-team-rules.md");
    let generated_content = "# Generated structured md (would normally be produced by /bob-compliance)\n";
    std::fs::write(&generated, generated_content).expect("write generated");
    let lock = target.join("docs").join("compliance").join(".compliance.lock");
    let lock_content = "generated_at: 2026-05-14T00:00:00Z\nsources: []\n";
    std::fs::write(&lock, lock_content).expect("write lock");

    // Run upgrade
    let status = std::process::Command::new(run_bob_bin())
        .args(["upgrade", "--dir"])
        .arg(target)
        .status()
        .expect("upgrade");
    assert!(status.success(), "upgrade failed");

    // All three user/runtime artifacts must be byte-identical after upgrade
    assert_eq!(
        std::fs::read_to_string(&user_source).expect("read"),
        sentinel,
        "sources/my-team-rules.md must NOT be touched by upgrade"
    );
    assert_eq!(
        std::fs::read_to_string(&generated).expect("read"),
        generated_content,
        "generated my-team-rules.md must NOT be touched by upgrade"
    );
    assert_eq!(
        std::fs::read_to_string(&lock).expect("read"),
        lock_content,
        ".compliance.lock must NOT be touched by upgrade"
    );
}

#[test]
fn init_creates_bob_model_skill() {
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
        .join("bob-model")
        .join("SKILL.md");
    assert!(p.is_file(), "bob-model SKILL.md missing at {}", p.display());
    let content = std::fs::read_to_string(&p).expect("read skill");

    // Frontmatter
    assert!(content.starts_with("---"), "must start with YAML frontmatter");
    assert!(content.contains("name: bob-model"), "frontmatter name");
    assert!(content.contains("description:"), "frontmatter description");

    // Stage 0 + meta tokens (will be extended in Tasks 3 and 4)
    for token in &[
        // CLI / invocation
        "/bob-model",
        "--story",
        // 三段式
        "三段式",
        "推测",
        "推荐选择",
        // Workflow
        "Stage 0",
        "入口体检",
        "短路",
        "缺输入",
        "极小需求",
        "常规",
        // Output target hint (the skill must explain where its output goes)
        "docs/bob/03-model-",
        // Stage 1 — extraction
        "Stage 1",
        "术语表",
        "Entity 草图",
        "状态机种子",
        "不变量",
        "业务规则",
        "BR-",
        "UseCase 初步清单",
        "开放问题",
        // Source-doc handling (mirrors compliance Stage 1)
        "Read 工具",
        "pages",
        // Stage 2 — md SSoT
        "Stage 2",
        "SSoT",
        "frontmatter",
        "source_doc_sha256",
        // Stage 3 — html view
        "Stage 3",
        "Mermaid",
        "classDiagram",
        "stateDiagram-v2",
        "flowchart",
        "粘性 TOC",
        // Stage 4 — closing
        "Stage 4",
        "/bob-stories",
        "/bob-identify",
        // 不变量
        "不变量",
        "md 是 SSoT",
        // bob-review (post-internalization, v0.5.0)
        "window.bobReview",
        ".claude/skills/bob-model/scripts/start-server.sh",
    ] {
        assert!(content.contains(token), "bob-model must mention {}", token);
    }
}

#[test]
fn bob_model_skill_mentions_dual_output() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let target = tmp.path();
    std::process::Command::new(run_bob_bin())
        .args(["init", "--dir"])
        .arg(target)
        .status()
        .expect("init");

    let p = target.join(".claude").join("skills").join("bob-model").join("SKILL.md");
    let content = std::fs::read_to_string(&p).expect("read skill");

    // The skill must instruct Claude to produce BOTH md and html
    assert!(content.contains(".md"), "skill must mention .md output");
    assert!(content.contains(".html"), "skill must mention .html output");
    assert!(
        content.contains("docs/bob/03-model-"),
        "skill must reference the output path prefix"
    );
}

#[test]
fn bob_model_skill_explains_cdn_strategy() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let target = tmp.path();
    std::process::Command::new(run_bob_bin())
        .args(["init", "--dir"])
        .arg(target)
        .status()
        .expect("init");

    let p = target.join(".claude").join("skills").join("bob-model").join("SKILL.md");
    let content = std::fs::read_to_string(&p).expect("read skill");

    // The skill must explain the CDN strategy + offline degradation
    assert!(content.contains("CDN"), "skill must mention CDN");
    assert!(
        content.contains("降级") || content.contains("离线"),
        "skill must mention offline / degradation behavior"
    );
    assert!(
        content.contains("cdn.jsdelivr.net") || content.contains("mermaid"),
        "skill must reference the specific Mermaid CDN url or library name"
    );
}

#[test]
fn bob_survey_declares_model_mandatory_across_difficulties() {
    // survey is optional; /bob-model and downstream are mandatory regardless of
    // Easy/Medium/Hard. The skill must say so explicitly and must NOT contain
    // any "Easy → /bob-identify" pattern that bypasses model.
    let tmp = tempfile::tempdir().expect("tempdir");
    let target = tmp.path();
    std::process::Command::new(run_bob_bin())
        .args(["init", "--dir"])
        .arg(target)
        .status()
        .expect("init");

    let p = target.join(".claude").join("skills").join("bob-survey").join("SKILL.md");
    let content = std::fs::read_to_string(&p).expect("read bob-survey");

    // Survey-optional / model-mandatory invariant must be stated.
    assert!(
        content.contains("survey 是可选阶段") || content.contains("survey 可选"),
        "bob-survey must declare survey is optional"
    );
    assert!(
        content.contains("强制阶段") && content.contains("不论"),
        "bob-survey must declare /bob-model is mandatory across difficulties"
    );

    // The old "Easy → /bob-identify (G 模式)" routing rows must be gone.
    assert!(
        !content.contains("直接 `/bob-identify <需求>`(G 模式)"),
        "bob-survey must no longer route Easy directly to /bob-identify (it bypasses /bob-model)"
    );

    // Every greenfield difficulty row in Stage 3 must route to /bob-model now.
    for difficulty_row in &[
        "Easy | 🟢 直接 `/bob-model",
        "Medium | 🟢 直接 `/bob-model",
        "Hard | 🟢 直接 `/bob-model",
    ] {
        assert!(
            content.contains(difficulty_row),
            "bob-survey 绿地 row must route through /bob-model: {}",
            difficulty_row
        );
    }

    // Anti-threshold guardrail (the regression we're fixing — TL must not write
    // "超过 N 个新端口才升 model" in the report).
    assert!(
        content.contains("超过 N 个新端口才升 model") || content.contains("阈值短路"),
        "bob-survey must explicitly forbid threshold-shortcut TL phrasing"
    );
}

#[test]
fn bob_survey_mentions_model_soft_prompt() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let target = tmp.path();
    std::process::Command::new(run_bob_bin())
        .args(["init", "--dir"])
        .arg(target)
        .status()
        .expect("init");

    let p = target.join(".claude").join("skills").join("bob-survey").join("SKILL.md");
    let content = std::fs::read_to_string(&p).expect("read bob-survey");

    // bob-survey's 下一步 must mention /bob-model and the source-doc requirement
    assert!(
        content.contains("/bob-model"),
        "bob-survey must mention /bob-model in 下一步; got:\n{}",
        content
    );
    // And the prompt should clarify it's gated on having a source requirement doc
    assert!(
        content.contains("Medium") || content.contains("Hard") || content.contains("源需求文档"),
        "bob-survey /bob-model prompt should hint at difficulty or source-doc gating"
    );
}

#[test]
fn init_makes_sh_scripts_executable() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let target = tmp.path();
    run_bob::commands::init::run(target.to_str().unwrap(), false, false, true, false)
        .expect("init failed");

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let script_path = target.join(".claude/skills/bob-model/scripts/start-server.sh");
        let meta = std::fs::metadata(&script_path).expect("script must exist");
        let mode = meta.permissions().mode();
        assert!(
            mode & 0o111 != 0,
            "start-server.sh should be executable; got mode {:o}",
            mode
        );
    }
}

#[test]
fn init_adds_bob_dir_to_gitignore() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let target = tmp.path();
    let status = Command::new(run_bob_bin())
        .args(["init", "--dir"])
        .arg(target)
        .status()
        .expect("run run-bob init");
    assert!(status.success(), "run-bob init failed");

    let gitignore_content = std::fs::read_to_string(target.join(".gitignore"))
        .expect("gitignore should exist after init");
    assert!(
        gitignore_content.contains(".bob/"),
        "gitignore should contain .bob/ entry; got: {}",
        gitignore_content
    );
}

#[test]
fn init_installs_bob_model_scripts() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let target = tmp.path();
    run_bob::commands::init::run(target.to_str().unwrap(), false, false, true, false)
        .expect("init failed");

    let scripts_dir = target.join(".claude/skills/bob-model/scripts");
    for fname in ["server.cjs", "helper.js", "start-server.sh", "stop-server.sh", "frame-template.html"] {
        let path = scripts_dir.join(fname);
        assert!(path.is_file(), "{} should be installed; not found", path.display());
    }

    // Verify server.cjs has new namespace
    let server_content = std::fs::read_to_string(scripts_dir.join("server.cjs"))
        .expect("server.cjs readable");
    assert!(
        server_content.contains("BOB_REVIEW_PORT"),
        "server.cjs should contain BOB_REVIEW_PORT (got namespace-renamed); head: {}",
        &server_content[..server_content.len().min(200)]
    );
    assert!(
        !server_content.contains("BRAINSTORM_PORT"),
        "server.cjs must NOT contain old BRAINSTORM_PORT after namespace rename"
    );

    // Verify helper.js has new namespace
    let helper_content = std::fs::read_to_string(scripts_dir.join("helper.js"))
        .expect("helper.js readable");
    assert!(
        helper_content.contains("window.bobReview"),
        "helper.js should expose window.bobReview"
    );
    assert!(
        !helper_content.contains("window.brainstorm"),
        "helper.js must NOT contain old window.brainstorm"
    );
}

#[test]
fn start_server_script_has_node_detection() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let target = tmp.path();
    run_bob::commands::init::run(target.to_str().unwrap(), false, false, true, false)
        .expect("init failed");

    let content = std::fs::read_to_string(
        target.join(".claude/skills/bob-model/scripts/start-server.sh")
    ).expect("start-server.sh readable");

    assert!(
        content.contains("command -v node"),
        "start-server.sh should detect missing node"
    );
    assert!(
        content.contains("NODE_MAJOR") && (content.contains("-lt 14") || content.contains("< 14")),
        "start-server.sh should reject Node < 14"
    );
}

#[test]
fn upgrade_replaces_stale_bob_model_scripts() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let target = tmp.path();

    // First init
    run_bob::commands::init::run(target.to_str().unwrap(), false, false, true, false)
        .expect("init failed");

    // Tamper with server.cjs to simulate a stale version
    let server_path = target.join(".claude/skills/bob-model/scripts/server.cjs");
    std::fs::write(&server_path, "// stale version, should be replaced\n").unwrap();
    let after_tamper = std::fs::read_to_string(&server_path).unwrap();
    assert_eq!(after_tamper, "// stale version, should be replaced\n");

    // Run upgrade — signature: (target_dir, dry_run, no_backup, no_gitignore)
    run_bob::commands::upgrade::run(target.to_str().unwrap(), false, true, true)
        .expect("upgrade failed");

    // Verify content restored
    let after_upgrade = std::fs::read_to_string(&server_path).unwrap();
    assert!(
        after_upgrade.contains("BOB_REVIEW_PORT"),
        "upgrade should restore stale server.cjs with bob-review namespace"
    );
    assert!(
        after_upgrade.len() > 100,
        "upgrade should restore full content (got {} bytes)",
        after_upgrade.len()
    );
}

#[test]
fn bob_spec_has_pattern_section() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let target = tmp.path();
    Command::new(run_bob_bin()).args(["init", "--dir"]).arg(target).status().expect("init");

    let p = target.join(".claude").join("skills").join("bob-spec").join("SKILL.md");
    let content = std::fs::read_to_string(&p).expect("read bob-spec");

    for token in &[
        "## 9.5 涉及设计模式",
        "PAT-1-1",
        "可观察痕迹",
        "机检锚点",
        "角色映射",
    ] {
        assert!(content.contains(token), "bob-spec must mention {} (§9.5)", token);
    }

    // 不破坏既有契约:/bob-compliance 仍 ≥3 次提及
    assert!(
        content.matches("/bob-compliance").count() >= 3,
        "bob-spec must still mention /bob-compliance >=3 times"
    );
}

#[test]
fn bob_spec_has_pattern_probe() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let target = tmp.path();
    Command::new(run_bob_bin()).args(["init", "--dir"]).arg(target).status().expect("init");

    let p = target.join(".claude").join("skills").join("bob-spec").join("SKILL.md");
    let content = std::fs::read_to_string(&p).expect("read bob-spec");

    for token in &[
        "Step S5",
        "要不要为本用例显式立 GoF 设计模式",
        "跳过(本 spec 不约束设计模式)",
        "信号 → 模式",
        "不立的代价",
        "Strategy",
        "状态多、迁移复杂",
        "Template Method",
        "横切包裹",
    ] {
        assert!(content.contains(token), "bob-spec must mention {} (Step S5)", token);
    }
}

#[test]
fn bob_compliance_loads_spec_patterns() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let target = tmp.path();
    Command::new(run_bob_bin()).args(["init", "--dir"]).arg(target).status().expect("init");

    let p = target.join(".claude").join("skills").join("bob-compliance").join("SKILL.md");
    let content = std::fs::read_to_string(&p).expect("read bob-compliance");

    for token in &[
        "空仓但有模式",      // Stage 0 carve-out
        "docs/specs/spec-",  // Stage 2 第二类规则源
        "## 9.5 涉及设计模式",
        "PAT-",
    ] {
        assert!(content.contains(token), "bob-compliance must mention {} (load patterns)", token);
    }
}

#[test]
fn bob_compliance_checks_pattern_conformance() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let target = tmp.path();
    Command::new(run_bob_bin()).args(["init", "--dir"]).arg(target).status().expect("init");

    let p = target.join(".claude").join("skills").join("bob-compliance").join("SKILL.md");
    let content = std::fs::read_to_string(&p).expect("read bob-compliance");

    for token in &[
        "## 模式符合度",       // Stage 4 报告段
        "收敛相关 spec",       // Stage 3.1 story→spec 收敛(Task 4 独有)
        "三项全过记",          // Stage 3.2 PASS/FAIL 判定逻辑(Task 4 独有)
        "PASS",               // 逐 PAT 判定
    ] {
        assert!(content.contains(token), "bob-compliance must mention {} (conformance)", token);
    }
}
