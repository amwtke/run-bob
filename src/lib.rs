//! Library facade so integration tests can access internal modules,
//! and so the binary and library share the print helpers.

use colored::*;
use std::path::Path;

pub mod assets;
pub mod commands;

/// True if the target looks like a Java/Maven project (has `src/main/java/`).
/// Used by `status` and `upgrade` to skip the optional Java skeleton when the
/// target opted out at `init` time. Mirrors the `/bob-survey` Stage 0 sentinel.
pub fn is_java_target(target: &Path) -> bool {
    target.join("src").join("main").join("java").is_dir()
}

/// Print a success message with a green checkmark.
pub fn success(msg: &str) {
    println!("{} {}", "✓".green().bold(), msg);
}

/// Print an info message.
pub fn info(msg: &str) {
    println!("{} {}", "ℹ".blue().bold(), msg);
}

/// Print a warning message.
pub fn warn(msg: &str) {
    println!("{} {}", "⚠".yellow().bold(), msg);
}

/// Print a skipped message.
pub fn skip(msg: &str) {
    println!("{} {}", "↷".bright_black().bold(), msg.bright_black());
}
