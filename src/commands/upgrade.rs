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

    // Apply: optional backup → overwrite OUTDATED → install MISSING.
    println!();
    println!("{}", "Applying changes...".bold());

    if !no_backup && !outdated.is_empty() {
        let ts = utc_timestamp();
        let backup_root = target.join(".run-bob-backup").join(&ts);
        // Step A: back up every OUTDATED file FIRST.
        for (asset, original_content) in &outdated {
            let backup_path = asset_path(&backup_root, asset);
            if let Some(parent) = backup_path.parent() {
                fs::create_dir_all(parent)
                    .with_context(|| format!("Failed to create backup dir {}", parent.display()))?;
            }
            fs::write(&backup_path, original_content)
                .with_context(|| format!("Failed to write backup {}", backup_path.display()))?;
        }
        println!(
            "  {} backup: {}/ ({} files)",
            "📦".bold(),
            format!(".run-bob-backup/{}", ts).bold(),
            outdated.len()
        );
    }

    // Step B: overwrite OUTDATED with embedded content.
    for (asset, _) in &outdated {
        let path = asset_path(&target, asset);
        fs::write(&path, asset.content)
            .with_context(|| format!("Failed to write {}", path.display()))?;
        println!("  {} {} ({})", "✓".green(), asset.display(), "updated".cyan());
    }

    // Install MISSING files (no backup — they didn't exist).
    for asset in &missing {
        let path = asset_path(&target, asset);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create dir {}", parent.display()))?;
        }
        fs::write(&path, asset.content)
            .with_context(|| format!("Failed to write {}", path.display()))?;
        println!("  {} {} ({})", "✓".green(), asset.display(), "installed".green());
    }

    println!();
    print_user_owned_skip_note(&user_owned);
    println!();
    println!(
        "{} upgrade complete. {} updated, {} installed, {} up to date.",
        "✓".green().bold(),
        outdated.len(),
        missing.len(),
        up_to_date.len()
    );
    Ok(())
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

/// Format `SystemTime::now()` as `YYYYMMDDTHHMMSSZ` (UTC).
/// Uses the Hinnant civil_from_days algorithm — do not "simplify" it.
fn utc_timestamp() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};

    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time before UNIX_EPOCH")
        .as_secs() as i64;

    let days = secs.div_euclid(86_400);
    let sod = secs.rem_euclid(86_400);
    let hour = sod / 3600;
    let minute = (sod % 3600) / 60;
    let second = sod % 60;

    let (year, month, day) = days_to_ymd(days);
    format!(
        "{:04}{:02}{:02}T{:02}{:02}{:02}Z",
        year, month, day, hour, minute, second
    )
}

/// Howard Hinnant's `civil_from_days`. Input: days since 1970-01-01.
/// Output: (year, month [1..=12], day [1..=31]). Correct for any year.
fn days_to_ymd(days: i64) -> (i64, u32, u32) {
    let z = days + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = (z - era * 146_097) as u64; // [0, 146096]
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365; // [0, 399]
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100); // [0, 365]
    let mp = (5 * doy + 2) / 153; // [0, 11]
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32; // [1, 31]
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32; // [1, 12]
    let y = if m <= 2 { y + 1 } else { y };
    (y, m, d)
}
