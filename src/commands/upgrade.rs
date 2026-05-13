//! `run-bob upgrade` — re-sync upgrade-safe harness assets with the embedded version.
//!
//! See `docs/superpowers/specs/2026-05-13-run-bob-upgrade-design.md` for design.

use anyhow::{Context, Result};
use colored::*;
use std::fs;
use std::path::{Path, PathBuf};

use crate::assets::{Asset, HARNESS_ASSETS};

pub fn run(target_dir: &str, dry_run: bool, no_backup: bool) -> Result<()> {
    let target = PathBuf::from(target_dir)
        .canonicalize()
        .with_context(|| format!("Failed to resolve target directory: {}", target_dir))?;

    print_header(&target, dry_run, no_backup);

    // Classify every upgrade-safe asset.
    let mut up_to_date: Vec<&Asset> = Vec::new();
    let mut outdated: Vec<(&Asset, String)> = Vec::new(); // (asset, current on-disk content)
    let mut missing: Vec<&Asset> = Vec::new();

    println!("{}", "Checking upgrade-safe assets...".bold());
    for asset in HARNESS_ASSETS.iter().filter(|a| a.upgrade_safe) {
        let path = asset_path(&target, asset);
        if !path.is_file() {
            println!("  {} {} ({})", "+".green(), asset.display(), "missing — will install".yellow());
            missing.push(asset);
        } else {
            let current = fs::read_to_string(&path)
                .with_context(|| format!("Failed to read {} for diff", path.display()))?;
            if current == asset.content {
                println!("  {} {} ({})", "✓".green(), asset.display(), "up to date".dimmed());
                up_to_date.push(asset);
            } else {
                println!("  {} {} ({})", "↑".yellow(), asset.display(), "outdated".yellow());
                outdated.push((asset, current));
            }
        }
    }

    let user_owned: Vec<&Asset> = HARNESS_ASSETS.iter().filter(|a| !a.upgrade_safe).collect();

    // Zero-change short circuit.
    if outdated.is_empty() && missing.is_empty() {
        println!();
        print_user_owned_skip_note(&user_owned);
        println!();
        println!(
            "{} {}",
            "✓".green().bold(),
            "All upgrade-safe assets are up to date.".green()
        );
        return Ok(());
    }

    // Dry-run short circuit.
    if dry_run {
        println!();
        println!(
            "{} dry-run: no files would be written. Run without --dry-run to apply.",
            "→".cyan().bold()
        );
        println!();
        print_user_owned_skip_note(&user_owned);
        println!();
        println!(
            "{} would update {}, would install {}, {} up to date.",
            "✓".green().bold(),
            outdated.len(),
            missing.len(),
            up_to_date.len()
        );
        return Ok(());
    }

    // Apply step lands in Task 4. For now, bail to keep behavior honest.
    let _ = no_backup;
    anyhow::bail!("upgrade apply step not yet implemented (Task 4)");
}

fn print_header(target: &Path, dry_run: bool, no_backup: bool) {
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
}

fn print_user_owned_skip_note(user_owned: &[&Asset]) {
    if user_owned.is_empty() {
        return;
    }
    let names: Vec<String> = user_owned.iter().map(|a| a.display()).collect();
    println!(
        "{} {} user-owned files skipped ({}).",
        "ℹ".blue().bold(),
        user_owned.len(),
        names.join(", ")
    );
    println!(
        "  Use {} if you need to overwrite them.",
        "`run-bob init --force`".bold()
    );
}

fn asset_path(target: &Path, asset: &Asset) -> PathBuf {
    let mut path = target.to_path_buf();
    for seg in asset.rel_path {
        path = path.join(seg);
    }
    path
}
