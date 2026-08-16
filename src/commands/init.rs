//! `run-bob init` command — installs harness assets into the target directory.

use anyhow::{Context, Result};
use colored::*;
use std::fs;
use std::path::{Path, PathBuf};

use crate::assets::{Asset, Category, HARNESS_ASSETS, HARNESS_DIRS};
use crate::{inspect_managed_path, ExpectedPathKind};

pub fn run(
    target_dir: &str,
    force: bool,
    minimal: bool,
    no_gitignore: bool,
    with_java: bool,
) -> Result<()> {
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
    if with_java {
        println!("  {} {}", "→ mode:".dimmed(), "--with-java (install Maven/ArchUnit skeleton)".yellow());
    }
    if no_gitignore {
        println!("  {} {}", "→ mode:".dimmed(), "--no-gitignore (skip .gitignore)".yellow());
    }
    println!();

    let applicable_assets = HARNESS_ASSETS
        .iter()
        .filter(|asset| !minimal || asset.included_in_minimal)
        .filter(|asset| !asset.category.is_java_skeleton() || with_java)
        .collect::<Vec<_>>();
    preflight(&target, &applicable_assets, minimal, no_gitignore)?;

    let mut current_cat: Option<Category> = None;
    for asset in applicable_assets {
        if Some(asset.category) != current_cat {
            if current_cat.is_some() {
                println!();
            }
            println!("{}", asset.category.install_header().bold());
            current_cat = Some(asset.category);
        }
        install_asset(&target, asset, force)?;
    }

    if !minimal {
        println!();
        println!("{}", "Creating working directories...".bold());
        for dir in HARNESS_DIRS {
            ensure_dir_at(&target, dir.rel_path)?;
            crate::success(&format!("{} {}", dir.display(), dir.note));
        }
    }

    println!();
    println!("{}", "Updating .gitignore...".bold());
    let report = crate::commands::gitignore::apply(&target, no_gitignore)?;
    crate::commands::gitignore::print_report(&report);

    print_next_steps(minimal);

    Ok(())
}

fn preflight(
    target: &Path,
    applicable_assets: &[&Asset],
    minimal: bool,
    no_gitignore: bool,
) -> Result<()> {
    for asset in applicable_assets {
        inspect_managed_path(target, asset.rel_path, ExpectedPathKind::File)?;
    }
    if !minimal {
        for dir in HARNESS_DIRS {
            inspect_managed_path(target, dir.rel_path, ExpectedPathKind::Directory)?;
        }
    }
    if !no_gitignore {
        inspect_managed_path(target, &[".gitignore"], ExpectedPathKind::File)?;
    }
    Ok(())
}

fn install_asset(target: &Path, asset: &Asset, force: bool) -> Result<()> {
    let mut path = target.to_path_buf();
    for seg in asset.rel_path {
        path = path.join(seg);
    }
    write_file(&path, asset.content, force, &asset.display())
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
    crate::set_executable_if_shell(path)?;
    crate::success(display);
    Ok(())
}

fn ensure_dir_at(target: &Path, segments: &[&str]) -> Result<()> {
    let mut path = target.to_path_buf();
    for seg in segments {
        path = path.join(seg);
    }
    fs::create_dir_all(&path)
        .with_context(|| format!("Failed to create directory {}", path.display()))
}

fn print_next_steps(minimal: bool) {
    println!();
    println!("{}", "━".repeat(60).bright_black());
    println!("{}", "Next steps".bold().green());
    println!("{}", "━".repeat(60).bright_black());

    if minimal {
        println!("  Skills installed. Try either host:");
        println!("    Claude Code:");
        println!(
            "      {} /bob-survey <your business description>   (optional sanity check)",
            "•".cyan()
        );
        println!(
            "      {} /bob-model <doc-path>                     (mandatory — extracts the domain SSoT)",
            "•".cyan()
        );
        println!("    Codex:");
        println!(
            "      {} $bob-survey <your business description>   (optional sanity check)",
            "•".cyan()
        );
        println!(
            "      {} $bob-model <doc-path>                     (mandatory — extracts the domain SSoT)",
            "•".cyan()
        );
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
    println!("  2. {} open Claude Code or Codex in this directory.", "Launch".cyan());
    println!();
    println!(
        "  3. {} start the workflow (survey is optional; model and downstream are mandatory):",
        "Run".cyan()
    );
    println!("       Claude Code:");
    println!(
        "         {}    {}",
        "/bob-survey <your business description>".bold().green(),
        "(optional)".bright_black()
    );
    println!(
        "         {}    {}",
        "/bob-model <doc-path>".bold().green(),
        "(mandatory — produces the SSoT for downstream skills)".bright_black()
    );
    println!(
        "         {}",
        "/bob-stories <your business description>".bold().green()
    );
    println!("         {}", "/bob-spec <use case>".bold().green());
    println!("       Codex:");
    println!(
        "         {}    {}",
        "$bob-survey <your business description>".bold().green(),
        "(optional)".bright_black()
    );
    println!(
        "         {}    {}",
        "$bob-model <doc-path>".bold().green(),
        "(mandatory — produces the SSoT for downstream skills)".bright_black()
    );
    println!(
        "         {}",
        "$bob-stories <your business description>".bold().green()
    );
    println!("         {}", "$bob-spec <use case>".bold().green());
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
