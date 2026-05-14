//! Manage the `# run-bob` block inside the target directory's `.gitignore`.
//!
//! See `docs/superpowers/specs/2026-05-14-gitignore-management-design.md`.

use anyhow::Result;
use colored::*;
use std::path::Path;

pub const GITIGNORE_BLOCK_HEADER: &str = "# run-bob";
pub const GITIGNORE_ENTRIES: &[&str] = &[".run-bob-backup/"];

#[derive(Debug, PartialEq, Eq)]
pub enum GitignoreReport {
    Skipped,
    Created { entries: usize },
    Updated { added: usize },
    UpToDate,
}

/// Read `<target>/.gitignore`, compute the needed update, write if any, return a report.
pub fn apply(_target: &Path, skip: bool) -> Result<GitignoreReport> {
    if skip {
        return Ok(GitignoreReport::Skipped);
    }
    // Real logic added in Task 2 onward.
    Ok(GitignoreReport::Skipped)
}

/// Print a human-readable line summarising what `apply` did.
pub fn print_report(report: &GitignoreReport) {
    match report {
        GitignoreReport::Skipped => {
            println!("  {} {}", "→".bright_black(), "skipped: --no-gitignore".bright_black());
        }
        GitignoreReport::Created { entries } => {
            println!(
                "  {} {} ({})",
                "+".green(),
                ".gitignore",
                format!("created, {} entr{}", entries, if *entries == 1 { "y" } else { "ies" }).green()
            );
        }
        GitignoreReport::Updated { added } => {
            println!(
                "  {} {} ({})",
                "↑".yellow(),
                ".gitignore",
                format!("added {} entr{}", added, if *added == 1 { "y" } else { "ies" }).yellow()
            );
        }
        GitignoreReport::UpToDate => {
            println!("  {} {} ({})", "✓".green(), ".gitignore", "up to date".dimmed());
        }
    }
}
