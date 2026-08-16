//! Library facade so integration tests can access internal modules,
//! and so the binary and library share the print helpers.

use anyhow::{bail, Context, Result};
use colored::*;
use std::fs;
use std::path::{Path, PathBuf};

pub mod assets;
pub mod commands;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ExpectedPathKind {
    File,
    Directory,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ManagedPathState {
    Missing,
    Present,
}

/// Inspect a managed path without following any component below `target`.
///
/// `target` must already be canonical. The first missing component makes the
/// destination safely missing; every existing component must be a real
/// directory except for a final file destination.
pub(crate) fn inspect_managed_path(
    target: &Path,
    rel_path: &[&str],
    expected: ExpectedPathKind,
) -> Result<ManagedPathState> {
    let mut current = PathBuf::from(target);

    for (index, segment) in rel_path.iter().enumerate() {
        current.push(segment);
        let relative = rel_path[..=index].join("/");
        let is_final = index + 1 == rel_path.len();
        let metadata = match fs::symlink_metadata(&current) {
            Ok(metadata) => metadata,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                return Ok(ManagedPathState::Missing);
            }
            Err(error) => {
                return Err(error)
                    .with_context(|| format!("Failed to inspect managed path {relative}"));
            }
        };

        if metadata.file_type().is_symlink() {
            bail!(
                "Managed path conflict at {}: symbolic links are not allowed",
                relative
            );
        }

        if !is_final {
            if !metadata.is_dir() {
                bail!(
                    "Managed path conflict at {}: expected a directory",
                    relative
                );
            }
            continue;
        }

        match expected {
            ExpectedPathKind::File if !metadata.is_file() => {
                bail!("Managed path conflict at {}: expected a file", relative)
            }
            ExpectedPathKind::Directory if !metadata.is_dir() => bail!(
                "Managed path conflict at {}: expected a directory",
                relative
            ),
            ExpectedPathKind::File | ExpectedPathKind::Directory => {}
        }
    }

    Ok(ManagedPathState::Present)
}

#[cfg(unix)]
pub(crate) fn set_executable_if_shell(path: &Path) -> Result<()> {
    if path.extension().and_then(|extension| extension.to_str()) == Some("sh") {
        use std::os::unix::fs::PermissionsExt;

        let metadata =
            fs::metadata(path).with_context(|| format!("Failed to stat {}", path.display()))?;
        let mut permissions = metadata.permissions();
        permissions.set_mode(permissions.mode() | 0o111);
        fs::set_permissions(path, permissions)
            .with_context(|| format!("Failed to chmod +x {}", path.display()))?;
    }
    Ok(())
}

#[cfg(not(unix))]
pub(crate) fn set_executable_if_shell(_path: &Path) -> Result<()> {
    Ok(())
}

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
