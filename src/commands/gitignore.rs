//! Manage the `# run-bob` block inside the target directory's `.gitignore`.
//!
//! See `docs/superpowers/specs/2026-05-14-gitignore-management-design.md`.

use anyhow::{Context, Result};
use colored::*;
use std::fs;
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
pub fn apply(target: &Path, skip: bool) -> Result<GitignoreReport> {
    if skip {
        return Ok(GitignoreReport::Skipped);
    }
    let path = target.join(".gitignore");
    let current: Option<String> = if path.is_file() {
        Some(fs::read_to_string(&path)
            .with_context(|| format!("Failed to read {}", path.display()))?)
    } else {
        None
    };
    let (new_content, report) =
        compute_update(current.as_deref(), GITIGNORE_BLOCK_HEADER, GITIGNORE_ENTRIES);
    if let Some(content) = new_content {
        fs::write(&path, content)
            .with_context(|| format!("Failed to write {}", path.display()))?;
    }
    Ok(report)
}

fn build_block(header: &str, entries: &[&str]) -> String {
    let mut s = String::new();
    s.push_str(header);
    s.push('\n');
    for e in entries {
        s.push_str(e);
        s.push('\n');
    }
    s
}

/// Pure: given current `.gitignore` content (None if missing), return
/// (new_content_to_write_or_None, action_taken).
fn compute_update(
    current: Option<&str>,
    header: &str,
    entries: &[&str],
) -> (Option<String>, GitignoreReport) {
    match current {
        None => {
            let body = build_block(header, entries);
            (Some(body), GitignoreReport::Created { entries: entries.len() })
        }
        Some(existing) => {
            let lines: Vec<&str> = existing.split('\n').collect();
            let block_start = lines.iter().position(|l| l.trim() == header);

            match block_start {
                None => {
                    // Case B: append block. Ensure existing ends with "\n\n".
                    let mut out = existing.to_string();
                    if !out.is_empty() {
                        while !out.ends_with("\n\n") {
                            out.push('\n');
                        }
                    }
                    out.push_str(&build_block(header, entries));
                    (Some(out), GitignoreReport::Updated { added: entries.len() })
                }
                Some(_start) => {
                    // Cases C/D handled in later tasks.
                    unimplemented!("cases C/D not yet implemented")
                }
            }
        }
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn case_a_creates_when_missing() {
        let (new_content, action) = compute_update(
            None,
            GITIGNORE_BLOCK_HEADER,
            GITIGNORE_ENTRIES,
        );
        assert_eq!(
            new_content.as_deref(),
            Some("# run-bob\n.run-bob-backup/\n")
        );
        assert_eq!(action, GitignoreReport::Created { entries: 1 });
    }

    #[test]
    fn case_b_appends_block_with_trailing_newline() {
        // file ends with '\n'  → exactly one blank line should appear before the block.
        let existing = "target/\n*.log\n";
        let (new_content, action) =
            compute_update(Some(existing), GITIGNORE_BLOCK_HEADER, GITIGNORE_ENTRIES);
        assert_eq!(
            new_content.as_deref(),
            Some("target/\n*.log\n\n# run-bob\n.run-bob-backup/\n")
        );
        assert_eq!(action, GitignoreReport::Updated { added: 1 });
    }

    #[test]
    fn case_b_appends_block_no_trailing_newline() {
        // file does NOT end with '\n' → add newline + blank line + block.
        let existing = "target/\n*.log";
        let (new_content, _) =
            compute_update(Some(existing), GITIGNORE_BLOCK_HEADER, GITIGNORE_ENTRIES);
        assert_eq!(
            new_content.as_deref(),
            Some("target/\n*.log\n\n# run-bob\n.run-bob-backup/\n")
        );
    }

    #[test]
    fn case_b_appends_block_already_has_blank_line() {
        // file already ends with "\n\n" → no extra newline needed.
        let existing = "target/\n*.log\n\n";
        let (new_content, _) =
            compute_update(Some(existing), GITIGNORE_BLOCK_HEADER, GITIGNORE_ENTRIES);
        assert_eq!(
            new_content.as_deref(),
            Some("target/\n*.log\n\n# run-bob\n.run-bob-backup/\n")
        );
    }
}
