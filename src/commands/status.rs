//! `run-bob status` — check whether the harness is properly installed.

use anyhow::{Context, Result};
use colored::*;
use std::path::{Path, PathBuf};

pub fn run(target_dir: &str) -> Result<()> {
    let target = PathBuf::from(target_dir)
        .canonicalize()
        .with_context(|| format!("Failed to resolve target directory: {}", target_dir))?;

    println!();
    println!(
        "{} {}",
        "📋".bold(),
        "run-bob harness status".bold().cyan()
    );
    println!("  {} {}", "→ target:".dimmed(), target.display());
    println!();

    let mut all_ok = true;

    println!("{}", "Skills".bold());
    all_ok &= check(&target, ".claude/skills/bob-identify/SKILL.md");
    all_ok &= check(&target, ".claude/skills/bob-onion/SKILL.md");
    all_ok &= check(&target, ".claude/skills/bob-spec/SKILL.md");

    println!();
    println!("{}", "Harness documents".bold());
    all_ok &= check(&target, "CLAUDE.md");
    all_ok &= check(&target, "ARCHITECTURE.md");
    all_ok &= check(&target, "README-RUN-BOB.md");

    println!();
    println!("{}", "ArchUnit guards".bold());
    all_ok &= check(&target, "src/test/java/architecture/CleanArchitectureTest.java");

    println!();
    println!("{}", "Shared Java skeletons".bold());
    all_ok &= check(&target, "src/main/java/com/example/shared/usecase/UseCase.java");
    all_ok &= check(&target, "src/main/java/com/example/shared/framework/transaction/TransactionalUseCaseDecorator.java");

    println!();
    println!("{}", "Working directories".bold());
    all_ok &= check_dir(&target, "docs/bob");
    all_ok &= check_dir(&target, "docs/specs");

    println!();
    if all_ok {
        println!("{} {}", "✓".green().bold(), "harness is complete.".green());
    } else {
        println!(
            "{} {}",
            "✗".red().bold(),
            "some assets are missing. Run `run-bob init` to install.".red()
        );
    }
    println!();

    Ok(())
}

fn check(target: &Path, rel: &str) -> bool {
    let p = target.join(rel);
    if p.is_file() {
        println!("  {} {}", "✓".green(), rel);
        true
    } else {
        println!("  {} {}", "✗".red(), rel.red());
        false
    }
}

fn check_dir(target: &Path, rel: &str) -> bool {
    let p = target.join(rel);
    if p.is_dir() {
        println!("  {} {}/", "✓".green(), rel);
        true
    } else {
        println!("  {} {}/", "✗".red(), rel.red());
        false
    }
}
