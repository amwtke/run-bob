//! `run-bob init` command — installs harness assets into the target directory.

use anyhow::{Context, Result};
use colored::*;
use std::fs;
use std::path::{Path, PathBuf};

const ROOT_CLAUDE_MD: &str = include_str!("../templates/root/CLAUDE.md");
const ROOT_ARCHITECTURE: &str = include_str!("../templates/root/ARCHITECTURE.md");
const ROOT_README: &str = include_str!("../templates/root/README-RUN-BOB.md");
const ROOT_ARCHUNIT_TEST: &str = include_str!("../templates/root/CleanArchitectureTest.java");
const SHARED_USECASE: &str = include_str!("../templates/root/UseCase.java");
const SHARED_DECORATOR: &str = include_str!("../templates/root/TransactionalUseCaseDecorator.java");
const SKILL_BOB_IDENTIFY: &str = include_str!("../templates/skills/bob-identify.md");
const SKILL_BOB_ONION: &str = include_str!("../templates/skills/bob-onion.md");
const SKILL_BOB_SPEC: &str = include_str!("../templates/skills/bob-spec.md");

pub fn run(target_dir: &str, force: bool, minimal: bool) -> Result<()> {
    let target = PathBuf::from(target_dir)
        .canonicalize()
        .with_context(|| format!("Failed to resolve target directory: {}", target_dir))?;

    println!();
    println!(
        "{} {}",
        "🛠 ".bold(),
        "run-bob: installing Bob 4-ring Clean Architecture + Superpowers harness".bold().cyan()
    );
    println!("  {} {}", "→ target:".dimmed(), target.display());
    if force {
        println!("  {} {}", "→ mode:".dimmed(), "--force (will overwrite)".yellow());
    }
    if minimal {
        println!("  {} {}", "→ mode:".dimmed(), "--minimal (skills only)".yellow());
    }
    println!();

    println!("{}", "Installing skills...".bold());
    install_skill(&target, "bob-identify", SKILL_BOB_IDENTIFY, force)?;
    install_skill(&target, "bob-onion", SKILL_BOB_ONION, force)?;
    install_skill(&target, "bob-spec", SKILL_BOB_SPEC, force)?;

    if !minimal {
        println!("{}", "Installing harness documents...".bold());
        install_root_file(&target, "CLAUDE.md", ROOT_CLAUDE_MD, force)?;
        install_root_file(&target, "ARCHITECTURE.md", ROOT_ARCHITECTURE, force)?;
        install_root_file(&target, "README-RUN-BOB.md", ROOT_README, force)?;
        install_archunit_test(&target, force)?;

        println!();
        println!("{}", "Installing shared Java skeletons...".bold());
        install_shared_usecase(&target, force)?;
        install_shared_decorator(&target, force)?;

        println!();
        println!("{}", "Creating working directories...".bold());
        ensure_dir(&target.join("docs").join("bob"))?;
        ensure_dir(&target.join("docs").join("specs"))?;
        crate::success("docs/bob/   (identify & onion intermediate notes)");
        crate::success("docs/specs/ (bob-spec outputs → Superpowers inputs)");
    }

    print_next_steps(minimal);

    Ok(())
}

/// Install a skill as `.claude/skills/<name>/SKILL.md`.
fn install_skill(target: &Path, name: &str, content: &str, force: bool) -> Result<()> {
    let skill_dir = target.join(".claude").join("skills").join(name);
    ensure_dir(&skill_dir)?;
    let path = skill_dir.join("SKILL.md");
    write_file(&path, content, force, &format!(".claude/skills/{}/SKILL.md", name))
}

/// Install a root-level file.
fn install_root_file(target: &Path, name: &str, content: &str, force: bool) -> Result<()> {
    let path = target.join(name);
    write_file(&path, content, force, name)
}

fn install_archunit_test(target: &Path, force: bool) -> Result<()> {
    let path = target
        .join("src").join("test").join("java")
        .join("architecture").join("CleanArchitectureTest.java");
    write_file(
        &path,
        ROOT_ARCHUNIT_TEST,
        force,
        "src/test/java/architecture/CleanArchitectureTest.java",
    )
}

fn install_shared_usecase(target: &Path, force: bool) -> Result<()> {
    install_java_file(
        target,
        &["src", "main", "java", "com", "example", "shared", "usecase", "UseCase.java"],
        SHARED_USECASE,
        force,
    )
}

fn install_shared_decorator(target: &Path, force: bool) -> Result<()> {
    install_java_file(
        target,
        &[
            "src", "main", "java", "com", "example", "shared",
            "framework", "transaction", "TransactionalUseCaseDecorator.java",
        ],
        SHARED_DECORATOR,
        force,
    )
}

/// Install a Java file at an arbitrary path under target.
fn install_java_file(
    target: &Path,
    rel_path: &[&str],
    content: &str,
    force: bool,
) -> Result<()> {
    let mut path = target.to_path_buf();
    for seg in rel_path {
        path = path.join(seg);
    }
    let display = rel_path.join("/");
    write_file(&path, content, force, &display)
}

/// Write a file, respecting the --force flag.
fn write_file(path: &Path, content: &str, force: bool, display: &str) -> Result<()> {
    if path.exists() && !force {
        crate::skip(&format!("{} already exists (use --force to overwrite)", display));
        return Ok(());
    }
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create parent dir for {}", path.display()))?;
    }
    fs::write(path, content)
        .with_context(|| format!("Failed to write {}", path.display()))?;
    crate::success(display);
    Ok(())
}

/// Make sure a directory exists.
fn ensure_dir(path: &Path) -> Result<()> {
    fs::create_dir_all(path)
        .with_context(|| format!("Failed to create directory {}", path.display()))
}

fn print_next_steps(minimal: bool) {
    println!();
    println!("{}", "━".repeat(60).bright_black());
    println!("{}", "Next steps".bold().green());
    println!("{}", "━".repeat(60).bright_black());

    if minimal {
        println!("  Skills installed. Open Claude Code and try:");
        println!("    {} /bob-identify <your business description>", "•".cyan());
        println!();
        println!(
            "  {} for full project integration, run without --minimal.",
            "tip:".yellow().bold()
        );
        return;
    }

    println!();
    println!("  1. {} open the generated files:", "Review".cyan());
    println!("       - CLAUDE.md               (project-level AI rules)");
    println!("       - ARCHITECTURE.md         (4-ring SSoT, starts empty)");
    println!("       - README-RUN-BOB.md       (how to use this harness)");
    println!();
    println!("  2. {} open Claude Code in this directory.", "Launch".cyan());
    println!();
    println!("  3. {} start the workflow:", "Run".cyan());
    println!(
        "       {}",
        "/bob-identify <your business description>".bold().green()
    );
    println!("       {}", "/bob-onion".bold().green());
    println!(
        "       {}",
        "/bob-spec <use case>".bold().green()
    );
    println!();
    println!(
        "  4. {} hand off the generated spec to Superpowers for TDD.",
        "Implement".cyan()
    );
    println!();
    println!(
        "  {} see README-RUN-BOB.md for the complete workflow.",
        "→".bright_black()
    );
    println!();
}
