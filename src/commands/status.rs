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

    println!("{}", "Harness documents".bold());
    all_ok &= check(&target, "CLAUDE.md");
    all_ok &= check(&target, "ARCHITECTURE.md");
    all_ok &= check(&target, "README-RUN-BOB.md");

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

#[allow(dead_code)]
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
