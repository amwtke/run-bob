//! Library facade so integration tests can access internal modules,
//! and so the binary and library share the print helpers.

use colored::*;

pub mod assets;
pub mod commands;

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
