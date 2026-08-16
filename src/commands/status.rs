//! `run-bob status` — check whether the harness is properly installed.

use anyhow::{bail, Context, Result};
use colored::*;
use std::path::{Path, PathBuf};

use crate::assets::{Category, HARNESS_ASSETS, HARNESS_DIRS};
use crate::{inspect_managed_path, ExpectedPathKind, ManagedPathState};

pub fn run(target_dir: &str) -> Result<()> {
    let target = PathBuf::from(target_dir)
        .canonicalize()
        .with_context(|| format!("Failed to resolve target directory: {}", target_dir))?;

    println!();
    println!("{} {}", "📋".bold(), "run-bob harness status".bold().cyan());
    println!("  {} {}", "→ target:".dimmed(), target.display());
    println!();

    let mut all_ok = true;
    let mut current_cat: Option<Category> = None;
    let java_target = crate::is_java_target(&target);

    for asset in HARNESS_ASSETS {
        // The Java/Maven skeleton is optional (init --with-java). Skip it on
        // non-Java targets so status doesn't flag absent files as missing.
        if asset.category.is_java_skeleton() && !java_target {
            continue;
        }
        if Some(asset.category) != current_cat {
            if current_cat.is_some() {
                println!();
            }
            println!("{}", asset.category.status_header().bold());
            current_cat = Some(asset.category);
        }
        all_ok &= check_file(&target, asset.rel_path);
    }

    println!();
    println!("{}", "Working directories".bold());
    for dir in HARNESS_DIRS {
        all_ok &= check_dir(&target, dir.rel_path);
    }

    println!();
    if all_ok {
        println!("{} {}", "✓".green().bold(), "harness is complete.".green());
    } else {
        println!(
            "{} {}",
            "✗".red().bold(),
            "some assets are missing or invalid. Run `run-bob upgrade` for an existing harness or `run-bob init` for a new target."
                .red()
        );
    }
    println!();

    if all_ok {
        Ok(())
    } else {
        bail!(
            "harness is incomplete; use `run-bob upgrade` for an existing harness or `run-bob init` for a new target"
        )
    }
}

fn check_file(target: &Path, segments: &[&str]) -> bool {
    check_path(target, segments, ExpectedPathKind::File, false)
}

fn check_dir(target: &Path, segments: &[&str]) -> bool {
    check_path(target, segments, ExpectedPathKind::Directory, true)
}

fn check_path(
    target: &Path,
    segments: &[&str],
    expected: ExpectedPathKind,
    display_as_directory: bool,
) -> bool {
    let relative = segments.join("/");
    let display = if display_as_directory {
        format!("{relative}/")
    } else {
        relative
    };

    match inspect_managed_path(target, segments, expected) {
        Ok(ManagedPathState::Present) => {
            println!("  {} {}", "✓".green(), display);
            true
        }
        Ok(ManagedPathState::Missing) => {
            println!("  {} {} {}", "✗".red(), display.red(), "(missing)".red());
            false
        }
        Err(error) => {
            println!(
                "  {} {} ({})",
                "✗".red(),
                display.red(),
                error.to_string().red()
            );
            false
        }
    }
}
