//! `run-bob upgrade` — re-sync upgrade-safe harness assets with the embedded version.
//!
//! See `docs/superpowers/specs/2026-05-13-run-bob-upgrade-design.md` for design.

use anyhow::{Context, Result};
use colored::*;
use std::path::PathBuf;

pub fn run(target_dir: &str, dry_run: bool, no_backup: bool) -> Result<()> {
    let target = PathBuf::from(target_dir)
        .canonicalize()
        .with_context(|| format!("Failed to resolve target directory: {}", target_dir))?;

    println!();
    println!(
        "{} {}",
        "🛠 ".bold(),
        "run-bob upgrade".bold().cyan()
    );
    println!("  {} {}", "→ target:".dimmed(), target.display());
    let mode = match (dry_run, no_backup) {
        (true, _) => "--dry-run (no files will be written)",
        (false, true) => "--no-backup (backup disabled)",
        (false, false) => "default (backup enabled)",
    };
    println!("  {} {}", "→ mode:".dimmed(), mode);
    println!();

    // Detection + apply land in Task 3+. This stub keeps the wiring honest.
    crate::info("upgrade implementation pending — this is a wiring stub.");
    Ok(())
}
