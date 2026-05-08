//! `run-bob init` command — installs harness assets into the target directory.

use anyhow::{Context, Result};
use colored::*;
use std::fs;
use std::path::{Path, PathBuf};

const ROOT_CLAUDE_MD: &str = include_str!("../templates/root/CLAUDE.md");
const ROOT_ARCHITECTURE: &str = include_str!("../templates/root/ARCHITECTURE.md");
const ROOT_README: &str = include_str!("../templates/root/README-RUN-BOB.md");
const ROOT_ARCHUNIT_TEST: &str = include_str!("../templates/root/CleanArchitectureTest.java");

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

    if !minimal {
        println!("{}", "Installing harness documents...".bold());
        install_root_file(&target, "CLAUDE.md", ROOT_CLAUDE_MD, force)?;
        install_root_file(&target, "ARCHITECTURE.md", ROOT_ARCHITECTURE, force)?;
        install_root_file(&target, "README-RUN-BOB.md", ROOT_README, force)?;
        install_archunit_test(&target, force)?;
    }

    print_next_steps(minimal);

    Ok(())
}

/// Install a skill as `.claude/skills/<name>/SKILL.md`.
#[allow(dead_code)]
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

/// Install a Java file at an arbitrary path under target.
#[allow(dead_code)]
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
#[allow(dead_code)]
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
