//! `run-bob status` — check whether the harness is properly installed.

use anyhow::{Context, Result};
use colored::*;
use std::path::{Path, PathBuf};

use crate::assets::{Category, HARNESS_ASSETS, HARNESS_DIRS};

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
    let mut current_cat: Option<Category> = None;

    for asset in HARNESS_ASSETS {
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
            "some assets are missing. Run `run-bob init` to install.".red()
        );
    }
    println!();

    Ok(())
}

fn check_file(target: &Path, segments: &[&str]) -> bool {
    let mut p = target.to_path_buf();
    for s in segments {
        p = p.join(s);
    }
    let display = segments.join("/");
    if p.is_file() {
        println!("  {} {}", "✓".green(), display);
        true
    } else {
        println!("  {} {}", "✗".red(), display.red());
        false
    }
}

fn check_dir(target: &Path, segments: &[&str]) -> bool {
    let mut p = target.to_path_buf();
    for s in segments {
        p = p.join(s);
    }
    let display = format!("{}/", segments.join("/"));
    if p.is_dir() {
        println!("  {} {}", "✓".green(), display);
        true
    } else {
        println!("  {} {}", "✗".red(), display.red());
        false
    }
}
