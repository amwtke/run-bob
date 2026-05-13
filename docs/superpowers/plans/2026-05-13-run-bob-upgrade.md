# run-bob `upgrade` Subcommand Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add `run-bob upgrade` — a subcommand that brings a previously-`init`'d project's harness assets (skills + pure-generated templates) into sync with the current `run-bob` binary, via byte-level content diff, while leaving user-customized files untouched. Defaults to backup; `--no-backup` and `--dry-run` available.

**Architecture:** Extend `Asset` (SSoT in `src/assets.rs`) with a single new flag `upgrade_safe: bool`. The new `src/commands/upgrade.rs` iterates only `upgrade_safe == true` entries, classifies each as **UP-TO-DATE / OUTDATED / MISSING** by comparing on-disk content with the `include_str!`-embedded content, and (in non-dry-run mode) first backs up OUTDATED files to `<target>/.run-bob-backup/<UTC-timestamp>/<original-relpath>`, then overwrites them; MISSING entries are installed without backup. User-owned files (`CLAUDE.md`, `ARCHITECTURE.md`, `CleanArchitectureTest.java`) are never read or written.

**Tech Stack:** Rust 1.75+, `clap = "=4.5.4"`, `anyhow = "1.0"`, `colored = "2.1"`, dev-dep `tempfile = "3"`. **No new crates** — UTC timestamp computed in-house via the Hinnant `civil_from_days` algorithm.

**Spec:** [`docs/superpowers/specs/2026-05-13-run-bob-upgrade-design.md`](../specs/2026-05-13-run-bob-upgrade-design.md)

**Out of scope (reference only, do NOT import code/templates):** `https://github.com/amwtke/superpowers-to-trae` — referenced for CLI surface semantics only.

---

## File Structure

| File | Action | Responsibility |
|---|---|---|
| `src/assets.rs` | **Modify** | Add `upgrade_safe: bool` to `Asset`; fill values for all 8 existing assets |
| `src/commands/mod.rs` | **Modify** | Add `pub mod upgrade;` |
| `src/commands/upgrade.rs` | **Create** | `run-bob upgrade` impl — detection + apply + backup + dry-run + output |
| `src/main.rs` | **Modify** | Add `Commands::Upgrade { dir, dry_run, no_backup }` variant + dispatch |
| `tests/integration.rs` | **Modify** | 8 new test cases (7 behavior + 1 SSoT drift guard) |
| `README.md` | **Modify** | Add "Upgrade harness assets in a project" subsection under "Update" |

**Untouched** (the plan must NOT modify these):
- All 8 template files under `src/templates/`
- `src/commands/init.rs` (its `install_asset` / `write_file` are not refactored — `upgrade.rs` carries its own small local helpers per YAGNI)
- `src/commands/status.rs`
- `Cargo.toml`

---

## Architectural Notes for the Engineer

Read these before starting — they explain decisions that are not obvious from individual tasks.

### 1. Why a separate `upgrade.rs` instead of extending `init.rs`?

`init` does "install given asset to given path with optional `--force`". `upgrade` does "classify all assets, then conditionally back up + overwrite + install". They share the goal of "write a file" but have very different control flow. Sharing code would require either threading a `mode: enum { Init, Upgrade }` parameter through `init.rs` or extracting a tiny shared write-helper. Both make `init.rs` worse, not better. Keep them separate; the duplicated lines are 5 trivial ones (`mkdir -p` + `fs::write`).

### 2. Why `upgrade_safe: bool` on `Asset`, not a separate registry?

Spec §3.1: `Category::HarnessDoc` contains both `README-RUN-BOB.md` (upgrade-safe) and `CLAUDE.md` / `ARCHITECTURE.md` (not). One existing category-based reverse-lookup won't work. A new bool field keeps the SSoT single-source and the semantics explicit. The cost is one extra column in the literal table; the benefit is `init` / `status` don't need to know `upgrade` exists, and `upgrade` doesn't need to know `init`'s categories.

### 3. Why byte-level `String ==`, not SHA256?

Templates are a few KB each. `&str == &str` is already a byte-level comparison and infinitely faster than rolling SHA256 (which would also require adding `sha2 = "0.10"`). The only theoretical motivation for hashing is "compare without holding both strings in memory" — irrelevant at this scale.

### 4. Why is the timestamp algorithm so weird-looking?

`days_to_ymd` is Howard Hinnant's `civil_from_days` algorithm (the canonical algorithm used in C++'s `<chrono>`). It's branch-light, handles negative inputs, and is correct for any year. Don't "simplify" it — you'll get it wrong. Just paste it verbatim.

### 5. The backup directory MUST be a sibling of the assets, not under `.claude/`

Spec §5.1: `<target>/.run-bob-backup/<ts>/`. NOT `<target>/.claude/.run-bob-backup/`. Putting backups under `.claude/` would confuse Claude Code itself (it scans skills under `.claude/skills/`).

### 6. Order of operations matters for atomicity

When applying OUTDATED files:
1. First create the backup dir
2. Copy ALL outdated files into the backup dir
3. Then overwrite them on disk

If you reverse 2 and 3, a crash mid-stream leaves a partial state with no recovery. The spec calls for this in §5.2.

---

## Task 1: Add `upgrade_safe` field to `Asset` (SSoT drift guard test first)

**Files:**
- Modify: `src/assets.rs`
- Modify: `tests/integration.rs` (append at end)

**Goal:** Extend the SSoT registry with the new field. Lock in the per-asset values with a drift-guard test that will fail next time someone adds a template without setting this field.

- [ ] **Step 1: Write the drift-guard test (will fail to compile because field doesn't exist)**

Append to `/Users/xiaojin/workshop/run-bob/tests/integration.rs`:

```rust
/// SSoT drift guard: every entry in HARNESS_ASSETS must have a deliberate
/// `upgrade_safe` value matching its policy.
///   - Skill + SharedJava → always upgrade-safe (pure generated content)
///   - ArchUnit          → never upgrade-safe (FORBIDDEN_IN_INNER is user-edited)
///   - HarnessDoc        → mixed (README is safe; CLAUDE / ARCHITECTURE are not)
///
/// This test runs *inside* the binary crate so it can read the private constant
/// via a public re-export. If a new asset is added without setting
/// upgrade_safe correctly, this fails before any new behavior test.
#[test]
fn upgrade_safe_field_matches_category_policy() {
    use run_bob::assets::{Category, HARNESS_ASSETS};

    for asset in HARNESS_ASSETS {
        let display = asset.rel_path.join("/");
        match asset.category {
            Category::Skill => assert!(
                asset.upgrade_safe,
                "{} is a Skill but upgrade_safe=false",
                display
            ),
            Category::SharedJava => assert!(
                asset.upgrade_safe,
                "{} is a SharedJava but upgrade_safe=false",
                display
            ),
            Category::ArchUnit => assert!(
                !asset.upgrade_safe,
                "{} is an ArchUnit but upgrade_safe=true",
                display
            ),
            Category::HarnessDoc => {
                let is_readme = asset.rel_path == ["README-RUN-BOB.md"];
                if is_readme {
                    assert!(
                        asset.upgrade_safe,
                        "README-RUN-BOB.md must be upgrade_safe=true"
                    );
                } else {
                    assert!(
                        !asset.upgrade_safe,
                        "{} is a user-owned HarnessDoc but upgrade_safe=true",
                        display
                    );
                }
            }
        }
    }
}
```

- [ ] **Step 2: Run the test (must fail to compile)**

Run: `cd /Users/xiaojin/workshop/run-bob && cargo test --test integration upgrade_safe_field_matches_category_policy 2>&1 | head -40`

Expected: compile error mentioning either `no field 'upgrade_safe' on type Asset` OR `assets is private` / `no module named 'run_bob' in scope`. Both are acceptable failure modes — we're about to fix both.

- [ ] **Step 3: Create `src/lib.rs` so integration tests can read `HARNESS_ASSETS`**

Integration tests live in `tests/integration.rs` — a separate crate. To call into our code, we need a library target. The existing `init.rs` references `crate::success` and `crate::skip` (defined in `main.rs`), so those helpers MUST move into the library too, otherwise the library won't compile.

First, verify the current `Cargo.toml` has no explicit `[[bin]]` / `[lib]` section (so Cargo's auto-detection takes over):

Run: `grep -E '^\[(\[bin|lib)' /Users/xiaojin/workshop/run-bob/Cargo.toml`

Expected: empty output. If non-empty, stop and report — the setup is non-standard and the plan needs adjustment.

Create `/Users/xiaojin/workshop/run-bob/src/lib.rs`:

```rust
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
```

Then rewrite `/Users/xiaojin/workshop/run-bob/src/main.rs` entirely:

```rust
//! run-bob: A CLI to bootstrap Bob's 4-ring Clean Architecture + Superpowers harness for Claude Code projects.

use anyhow::Result;
use clap::{Parser, Subcommand};

use run_bob::commands;

#[derive(Parser, Debug)]
#[command(name = "run-bob")]
#[command(about = "Bootstrap Bob's 4-ring Clean Architecture + Superpowers harness for Claude Code projects", long_about = None)]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Initialize the run-bob harness in the current directory
    Init {
        /// Overwrite existing files
        #[arg(short, long)]
        force: bool,

        /// Only install skills, skip CLAUDE.md / ARCHITECTURE.md / README / shared / ArchUnit
        #[arg(short, long)]
        minimal: bool,

        /// Target directory (default: current directory)
        #[arg(short, long, default_value = ".")]
        dir: String,
    },

    /// Check if the current project has the run-bob harness properly installed
    Status {
        /// Target directory (default: current directory)
        #[arg(short, long, default_value = ".")]
        dir: String,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init {
            force,
            minimal,
            dir,
        } => {
            commands::init::run(&dir, force, minimal)?;
        }
        Commands::Status { dir } => {
            commands::status::run(&dir)?;
        }
    }

    Ok(())
}
```

(`Commands::Upgrade` is intentionally NOT added here — that happens in Task 2. The helpers (`success`/`info`/`warn`/`skip`) are gone from `main.rs` because they're in `lib.rs` now; existing `crate::success` calls in `init.rs` still resolve, since within the library `crate::success` == `run_bob::success`.)

- [ ] **Step 4: Add the `upgrade_safe` field to the `Asset` struct**

In `/Users/xiaojin/workshop/run-bob/src/assets.rs`, find:

```rust
pub struct Asset {
    /// Path segments relative to the target directory.
    pub rel_path: &'static [&'static str],
    /// File content baked at compile time via `include_str!`.
    pub content: &'static str,
    pub category: Category,
    /// True if the asset is installed even in `--minimal` mode.
    pub included_in_minimal: bool,
}
```

Replace with:

```rust
pub struct Asset {
    /// Path segments relative to the target directory.
    pub rel_path: &'static [&'static str],
    /// File content baked at compile time via `include_str!`.
    pub content: &'static str,
    pub category: Category,
    /// True if the asset is installed even in `--minimal` mode.
    pub included_in_minimal: bool,
    /// True if `run-bob upgrade` may overwrite this file when its content
    /// drifts from the embedded version. User-owned files (CLAUDE.md,
    /// ARCHITECTURE.md, CleanArchitectureTest.java) MUST be false.
    pub upgrade_safe: bool,
}
```

- [ ] **Step 5: Set `upgrade_safe` on every entry in `HARNESS_ASSETS`**

In the same file, find the `pub const HARNESS_ASSETS: &[Asset] = &[ ... ];` block. Set the new field on each of the 8 entries according to this table (already vetted in the spec §3.2):

| rel_path | upgrade_safe |
|---|---|
| `.claude/skills/bob-identify/SKILL.md` | `true` |
| `.claude/skills/bob-onion/SKILL.md` | `true` |
| `.claude/skills/bob-spec/SKILL.md` | `true` |
| `CLAUDE.md` | `false` |
| `ARCHITECTURE.md` | `false` |
| `README-RUN-BOB.md` | `true` |
| `src/test/java/architecture/CleanArchitectureTest.java` | `false` |
| `src/main/java/com/example/shared/usecase/UseCase.java` | `true` |
| `src/main/java/com/example/shared/framework/transaction/TransactionalUseCaseDecorator.java` | `true` |

For each entry, add the field on a new line after `included_in_minimal`. Example for the first one:

```rust
Asset {
    rel_path: &[".claude", "skills", "bob-identify", "SKILL.md"],
    content: include_str!("templates/skills/bob-identify.md"),
    category: Category::Skill,
    included_in_minimal: true,
    upgrade_safe: true,
},
```

And for `CLAUDE.md`:

```rust
Asset {
    rel_path: &["CLAUDE.md"],
    content: include_str!("templates/root/CLAUDE.md"),
    category: Category::HarnessDoc,
    included_in_minimal: false,
    upgrade_safe: false,
},
```

Do the same for the remaining 7. Triple-check `upgrade_safe: false` is set for `CLAUDE.md`, `ARCHITECTURE.md`, and `CleanArchitectureTest.java`.

- [ ] **Step 6: Run the drift guard test (must pass)**

Run: `cd /Users/xiaojin/workshop/run-bob && cargo test --test integration upgrade_safe_field_matches_category_policy -- --nocapture`

Expected: `test upgrade_safe_field_matches_category_policy ... ok`. If any per-asset assertion message fires, fix the `upgrade_safe` value for that asset in step 5 and rerun.

- [ ] **Step 7: Run the full existing test suite (must stay green)**

Run: `cd /Users/xiaojin/workshop/run-bob && cargo test`

Expected: all 16 prior tests + 1 new = **17 passed; 0 failed**. If any of the 16 prior tests fail, the most likely cause is the `mod` → `pub mod` / library refactor in step 3; verify `src/lib.rs` exists and `src/main.rs` no longer has `mod assets;` / `mod commands;`.

- [ ] **Step 8: Commit**

```bash
cd /Users/xiaojin/workshop/run-bob && git add src/lib.rs src/main.rs src/assets.rs tests/integration.rs && git -c commit.gpgsign=false commit -m "$(cat <<'EOF'
feat(assets): add Asset.upgrade_safe field + library facade for tests

Marks each HARNESS_ASSETS entry as upgrade-safe (skills, README,
shared Java skeletons) or user-owned (CLAUDE.md, ARCHITECTURE.md,
CleanArchitectureTest.java). Adds a drift-guard test asserting the
policy per Category.

Promotes assets/commands to pub via src/lib.rs so integration tests
can read HARNESS_ASSETS directly. init/status behavior unchanged.

Prepares ground for run-bob upgrade (spec 2026-05-13).
EOF
)"
```

---

## Task 2: Wire up `Commands::Upgrade` skeleton + `--help` test

**Files:**
- Modify: `src/main.rs`
- Modify: `src/commands/mod.rs`
- Create: `src/commands/upgrade.rs`
- Modify: `tests/integration.rs` (append)

**Goal:** End-to-end wiring so `run-bob upgrade --help` lists the right flags. Implementation is a no-op stub that just prints a header — actual logic comes in Task 3.

- [ ] **Step 1: Write the failing `--help` test**

Append to `/Users/xiaojin/workshop/run-bob/tests/integration.rs`:

```rust
#[test]
fn upgrade_help_lists_flags() {
    let output = std::process::Command::new(run_bob_bin())
        .args(["upgrade", "--help"])
        .output()
        .expect("run run-bob upgrade --help");
    assert!(
        output.status.success(),
        "run-bob upgrade --help failed: {:?}",
        output
    );
    let stdout = String::from_utf8(output.stdout).unwrap();
    for flag in &["--dir", "--dry-run", "--no-backup"] {
        assert!(
            stdout.contains(flag),
            "expected {} in `upgrade --help` output, got:\n{}",
            flag,
            stdout
        );
    }
}
```

- [ ] **Step 2: Run the test (must fail)**

Run: `cd /Users/xiaojin/workshop/run-bob && cargo test --test integration upgrade_help_lists_flags 2>&1 | tail -20`

Expected: failure with something like `error: unrecognized subcommand 'upgrade'` (clap rejects the unknown subcommand and exits non-zero).

- [ ] **Step 3: Add `pub mod upgrade;` to the commands module index**

Edit `/Users/xiaojin/workshop/run-bob/src/commands/mod.rs`. Current content:

```rust
pub mod init;
pub mod status;
```

Replace with:

```rust
pub mod init;
pub mod status;
pub mod upgrade;
```

- [ ] **Step 4: Create the `upgrade` module with a stub `run` function**

Create `/Users/xiaojin/workshop/run-bob/src/commands/upgrade.rs`:

```rust
//! `run-bob upgrade` — re-sync upgrade-safe harness assets with the embedded version.
//!
//! See `docs/superpowers/specs/2026-05-13-run-bob-upgrade-design.md` for design.

use anyhow::{Context, Result};
use colored::*;
use std::path::PathBuf;

pub fn run(target_dir: &str, dry_run: bool, no_backup: bool) -> Result<()> {
    let target = PathBuf::from(target_dir)
        .canonicalize()
        .with_context(|| format!("Failed to resolve target directory: {}", target_dir))?;

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

    // Detection + apply land in Task 3+. This stub keeps the wiring honest.
    crate::info("upgrade implementation pending — this is a wiring stub.");
    Ok(())
}
```

- [ ] **Step 5: Add the `Upgrade` variant to the CLI enum and dispatch it**

Edit `/Users/xiaojin/workshop/run-bob/src/main.rs`. Find the `Commands` enum and add a third variant after `Status`. The enum becomes:

```rust
#[derive(Subcommand, Debug)]
enum Commands {
    /// Initialize the run-bob harness in the current directory
    Init {
        /// Overwrite existing files
        #[arg(short, long)]
        force: bool,

        /// Only install skills, skip CLAUDE.md / ARCHITECTURE.md / README / shared / ArchUnit
        #[arg(short, long)]
        minimal: bool,

        /// Target directory (default: current directory)
        #[arg(short, long, default_value = ".")]
        dir: String,
    },

    /// Check if the current project has the run-bob harness properly installed
    Status {
        /// Target directory (default: current directory)
        #[arg(short, long, default_value = ".")]
        dir: String,
    },

    /// Re-sync upgrade-safe harness assets in a target project with the current run-bob binary
    Upgrade {
        /// Target directory (default: current directory)
        #[arg(short, long, default_value = ".")]
        dir: String,

        /// Only report what would change; do not write any files
        #[arg(short = 'n', long)]
        dry_run: bool,

        /// Skip the safety backup before overwriting (dangerous)
        #[arg(long)]
        no_backup: bool,
    },
}
```

And update the `match cli.command { ... }` block to add a third arm:

```rust
fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init {
            force,
            minimal,
            dir,
        } => {
            commands::init::run(&dir, force, minimal)?;
        }
        Commands::Status { dir } => {
            commands::status::run(&dir)?;
        }
        Commands::Upgrade {
            dir,
            dry_run,
            no_backup,
        } => {
            commands::upgrade::run(&dir, dry_run, no_backup)?;
        }
    }

    Ok(())
}
```

- [ ] **Step 6: Run the `--help` test (must pass)**

Run: `cd /Users/xiaojin/workshop/run-bob && cargo test --test integration upgrade_help_lists_flags`

Expected: `test upgrade_help_lists_flags ... ok`.

- [ ] **Step 7: Run the full suite to ensure nothing broke**

Run: `cd /Users/xiaojin/workshop/run-bob && cargo test`

Expected: **18 passed; 0 failed** (16 prior + 2 from Tasks 1–2).

- [ ] **Step 8: Commit**

```bash
cd /Users/xiaojin/workshop/run-bob && git add src/main.rs src/commands/mod.rs src/commands/upgrade.rs tests/integration.rs && git -c commit.gpgsign=false commit -m "$(cat <<'EOF'
feat(upgrade): wire up `run-bob upgrade` subcommand skeleton

Adds Commands::Upgrade variant with --dir / --dry-run / --no-backup
flags. The handler is currently a header-printing stub — detection
and apply land in subsequent commits per the 2026-05-13 spec.
EOF
)"
```

---

## Task 3: Detection + report (no-op, dry-run, user-owned skip)

**Files:**
- Modify: `src/commands/upgrade.rs`
- Modify: `tests/integration.rs` (append)

**Goal:** Implement the read-only half of upgrade — classify every `upgrade_safe == true` asset as UP-TO-DATE / OUTDATED / MISSING, print the check lines, print the user-owned skip summary, and (when nothing differs) print "All upgrade-safe assets are up to date." This makes the dry-run, no-op, and user-owned-skip tests pass for free because we don't write anything yet.

- [ ] **Step 1: Write 3 behavior tests at once (they all share the read-only execution path)**

Append to `/Users/xiaojin/workshop/run-bob/tests/integration.rs`:

```rust
#[test]
fn upgrade_on_fresh_init_is_noop() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let target = tmp.path();

    // Fresh init lays down everything.
    let status = std::process::Command::new(run_bob_bin())
        .args(["init", "--dir"])
        .arg(target)
        .status()
        .expect("init");
    assert!(status.success());

    let output = std::process::Command::new(run_bob_bin())
        .args(["upgrade", "--dir"])
        .arg(target)
        .output()
        .expect("upgrade");
    assert!(output.status.success(), "upgrade failed: {:?}", output);
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert!(
        stdout.contains("All upgrade-safe assets are up to date"),
        "expected no-op message, got:\n{}",
        stdout
    );
    // No backup dir created on no-op.
    assert!(
        !target.join(".run-bob-backup").exists(),
        ".run-bob-backup must not exist after a no-op upgrade"
    );
}

#[test]
fn upgrade_skips_user_owned() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let target = tmp.path();

    std::process::Command::new(run_bob_bin())
        .args(["init", "--dir"])
        .arg(target)
        .status()
        .expect("init");

    // Hand-edit the 3 user-owned files. Upgrade must NOT touch them.
    let sentinel = "USER-EDIT-DO-NOT-OVERWRITE\n";
    let user_owned = [
        target.join("CLAUDE.md"),
        target.join("ARCHITECTURE.md"),
        target.join("src/test/java/architecture/CleanArchitectureTest.java"),
    ];
    for p in &user_owned {
        std::fs::write(p, sentinel).expect("write sentinel");
    }

    let output = std::process::Command::new(run_bob_bin())
        .args(["upgrade", "--dir"])
        .arg(target)
        .output()
        .expect("upgrade");
    assert!(output.status.success(), "upgrade failed: {:?}", output);

    for p in &user_owned {
        let actual = std::fs::read_to_string(p).expect("read user-owned");
        assert_eq!(
            actual, sentinel,
            "upgrade must not touch user-owned file {}",
            p.display()
        );
    }

    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(
        stdout.contains("user-owned files skipped"),
        "expected user-owned skip note, got:\n{}",
        stdout
    );
}

#[test]
fn upgrade_dry_run_writes_nothing() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let target = tmp.path();

    std::process::Command::new(run_bob_bin())
        .args(["init", "--dir"])
        .arg(target)
        .status()
        .expect("init");

    // Corrupt one skill file so detection sees OUTDATED.
    let skill = target.join(".claude/skills/bob-identify/SKILL.md");
    std::fs::write(&skill, "STALE\n").expect("write stale");

    let output = std::process::Command::new(run_bob_bin())
        .args(["upgrade", "--dry-run", "--dir"])
        .arg(target)
        .output()
        .expect("upgrade --dry-run");
    assert!(output.status.success(), "upgrade --dry-run failed: {:?}", output);

    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(
        stdout.contains("dry-run"),
        "expected dry-run note, got:\n{}",
        stdout
    );

    // File on disk is still the stale version.
    let actual = std::fs::read_to_string(&skill).expect("read");
    assert_eq!(actual, "STALE\n", "dry-run must not write files");
    assert!(
        !target.join(".run-bob-backup").exists(),
        ".run-bob-backup must not exist after dry-run"
    );
}
```

- [ ] **Step 2: Run the three tests (they must fail — stub is still in place)**

Run: `cd /Users/xiaojin/workshop/run-bob && cargo test --test integration upgrade_on_fresh_init_is_noop upgrade_skips_user_owned upgrade_dry_run_writes_nothing 2>&1 | tail -30`

Expected: each test fails on its assertion about stdout content. (The stub command exits successfully, so we'll see assertion failures, not crashes.)

- [ ] **Step 3: Implement detection — classify each upgrade-safe asset**

Replace the entire content of `/Users/xiaojin/workshop/run-bob/src/commands/upgrade.rs` with:

```rust
//! `run-bob upgrade` — re-sync upgrade-safe harness assets with the embedded version.
//!
//! See `docs/superpowers/specs/2026-05-13-run-bob-upgrade-design.md` for design.

use anyhow::{Context, Result};
use colored::*;
use std::fs;
use std::path::{Path, PathBuf};

use crate::assets::{Asset, HARNESS_ASSETS};

/// Per-asset classification result.
enum State {
    UpToDate,
    Outdated,
    Missing,
}

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
        if !path.exists() {
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

    // Apply step lands in Task 4. For now, panic if we reach it during testing.
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
```

- [ ] **Step 4: Run the three new tests (must pass)**

Run: `cd /Users/xiaojin/workshop/run-bob && cargo test --test integration upgrade_on_fresh_init_is_noop upgrade_skips_user_owned upgrade_dry_run_writes_nothing`

Expected: all three pass.

Note: `upgrade_skips_user_owned` doesn't have anything OUTDATED among upgrade-safe assets (it only edits user-owned files, which we skip), so it hits the zero-change short circuit. That's correct.

- [ ] **Step 5: Run the full suite**

Run: `cd /Users/xiaojin/workshop/run-bob && cargo test`

Expected: **21 passed; 0 failed** (16 prior + 5 new from Tasks 1–3).

- [ ] **Step 6: Commit**

```bash
cd /Users/xiaojin/workshop/run-bob && git add src/commands/upgrade.rs tests/integration.rs && git -c commit.gpgsign=false commit -m "$(cat <<'EOF'
feat(upgrade): implement detection + dry-run + user-owned skip

Classifies every upgrade_safe asset as UP-TO-DATE / OUTDATED / MISSING
via byte-level content diff against the embedded template. Prints
per-asset status, the user-owned skip note, and exits cleanly on
no-op or --dry-run paths. Apply logic still pending (Task 4).
EOF
)"
```

---

## Task 4: Apply OUTDATED with backup + `--no-backup`

**Files:**
- Modify: `src/commands/upgrade.rs`
- Modify: `tests/integration.rs` (append)

**Goal:** Implement the write path for OUTDATED files: optionally back up the originals to `<target>/.run-bob-backup/<UTC-ts>/<rel-path>`, then overwrite. MISSING is handled in Task 5.

- [ ] **Step 1: Write the two behavior tests**

Append to `/Users/xiaojin/workshop/run-bob/tests/integration.rs`:

```rust
#[test]
fn upgrade_replaces_stale_skill() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let target = tmp.path();

    std::process::Command::new(run_bob_bin())
        .args(["init", "--dir"])
        .arg(target)
        .status()
        .expect("init");

    let skill = target.join(".claude/skills/bob-identify/SKILL.md");
    let original = std::fs::read_to_string(&skill).expect("read original");
    std::fs::write(&skill, "STALE-CONTENT\n").expect("write stale");

    let output = std::process::Command::new(run_bob_bin())
        .args(["upgrade", "--dir"])
        .arg(target)
        .output()
        .expect("upgrade");
    assert!(output.status.success(), "upgrade failed: {:?}", output);

    // File is restored to embedded content.
    let after = std::fs::read_to_string(&skill).expect("read after");
    assert_eq!(
        after, original,
        "skill must be restored to embedded content after upgrade"
    );

    // Backup directory exists; find its single timestamp subdir.
    let backup_root = target.join(".run-bob-backup");
    assert!(backup_root.is_dir(), ".run-bob-backup must exist");
    let entries: Vec<_> = std::fs::read_dir(&backup_root)
        .expect("read_dir backup")
        .filter_map(|e| e.ok())
        .collect();
    assert_eq!(
        entries.len(),
        1,
        "exactly one timestamp dir expected under .run-bob-backup"
    );
    let ts_dir = entries[0].path();
    let backed_up = ts_dir.join(".claude/skills/bob-identify/SKILL.md");
    assert!(
        backed_up.is_file(),
        "backed-up file expected at {}",
        backed_up.display()
    );
    let backup_content = std::fs::read_to_string(&backed_up).expect("read backup");
    assert_eq!(
        backup_content, "STALE-CONTENT\n",
        "backup must preserve the original (stale) content"
    );
}

#[test]
fn upgrade_no_backup_skips_backup() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let target = tmp.path();

    std::process::Command::new(run_bob_bin())
        .args(["init", "--dir"])
        .arg(target)
        .status()
        .expect("init");

    let skill = target.join(".claude/skills/bob-onion/SKILL.md");
    let embedded = std::fs::read_to_string(&skill).expect("read embedded");
    std::fs::write(&skill, "STALE\n").expect("write stale");

    let output = std::process::Command::new(run_bob_bin())
        .args(["upgrade", "--no-backup", "--dir"])
        .arg(target)
        .output()
        .expect("upgrade --no-backup");
    assert!(output.status.success(), "upgrade --no-backup failed: {:?}", output);

    // File was overwritten.
    let after = std::fs::read_to_string(&skill).expect("read after");
    assert_eq!(after, embedded, "skill must be overwritten with --no-backup");

    // But no backup directory was created.
    assert!(
        !target.join(".run-bob-backup").exists(),
        ".run-bob-backup must not exist with --no-backup"
    );
}
```

- [ ] **Step 2: Run the two tests (must fail with bail message)**

Run: `cd /Users/xiaojin/workshop/run-bob && cargo test --test integration upgrade_replaces_stale_skill upgrade_no_backup_skips_backup 2>&1 | tail -20`

Expected: both fail because `upgrade` exits with `anyhow::bail!("upgrade apply step not yet implemented (Task 4)")` — `output.status.success()` is false, so the first assertion in each test fires.

- [ ] **Step 3: Implement the apply path (backup + overwrite) for OUTDATED files**

In `/Users/xiaojin/workshop/run-bob/src/commands/upgrade.rs`, replace the trailing block:

```rust
    // Apply step lands in Task 4. For now, panic if we reach it during testing.
    let _ = no_backup;
    anyhow::bail!("upgrade apply step not yet implemented (Task 4)");
```

with:

```rust
    // Apply: optional backup, then overwrite OUTDATED. MISSING handled in Task 5.
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

    // MISSING handling lands in Task 5.

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
```

(Note: the closing `}` already exists in the file; the snippet replaces the `bail!` block and ends before that closing brace. Be careful not to duplicate or drop braces.)

- [ ] **Step 4: Add the UTC timestamp helper at the bottom of the file**

Append to `/Users/xiaojin/workshop/run-bob/src/commands/upgrade.rs`:

```rust
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
```

- [ ] **Step 5: Run the two new tests (must pass)**

Run: `cd /Users/xiaojin/workshop/run-bob && cargo test --test integration upgrade_replaces_stale_skill upgrade_no_backup_skips_backup`

Expected: both pass. If `upgrade_replaces_stale_skill` fails with "backed-up file expected at ...", check the `asset_path` helper is being passed `backup_root` (which already includes `<target>/.run-bob-backup/<ts>`), not `target`.

- [ ] **Step 6: Run the full suite**

Run: `cd /Users/xiaojin/workshop/run-bob && cargo test`

Expected: **23 passed; 0 failed** (16 prior + 7 new from Tasks 1–4).

- [ ] **Step 7: Commit**

```bash
cd /Users/xiaojin/workshop/run-bob && git add src/commands/upgrade.rs tests/integration.rs && git -c commit.gpgsign=false commit -m "$(cat <<'EOF'
feat(upgrade): apply OUTDATED files with optional backup

Implements the write path for the OUTDATED state: backs up originals
to .run-bob-backup/<UTC-ts>/<rel-path> first, then overwrites with the
embedded content. --no-backup skips the backup step. UTC timestamp
formatted via in-house Hinnant civil_from_days (no chrono dep).
EOF
)"
```

---

## Task 5: Install MISSING files

**Files:**
- Modify: `src/commands/upgrade.rs`
- Modify: `tests/integration.rs` (append)

**Goal:** Handle the MISSING state — install any upgrade-safe asset that doesn't exist on disk. No backup (there's nothing to back up).

- [ ] **Step 1: Write the missing-install test**

Append to `/Users/xiaojin/workshop/run-bob/tests/integration.rs`:

```rust
#[test]
fn upgrade_installs_missing_skill() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let target = tmp.path();

    std::process::Command::new(run_bob_bin())
        .args(["init", "--dir"])
        .arg(target)
        .status()
        .expect("init");

    // Delete a skill so the upgrade sees MISSING.
    let skill = target.join(".claude/skills/bob-spec/SKILL.md");
    std::fs::remove_file(&skill).expect("remove skill");
    assert!(!skill.exists());

    let output = std::process::Command::new(run_bob_bin())
        .args(["upgrade", "--dir"])
        .arg(target)
        .output()
        .expect("upgrade");
    assert!(output.status.success(), "upgrade failed: {:?}", output);

    // Skill was re-installed.
    assert!(skill.is_file(), "bob-spec/SKILL.md must be installed back");
    let content = std::fs::read_to_string(&skill).expect("read installed");
    assert!(
        content.contains("name: bob-spec"),
        "installed content must match the embedded template"
    );

    // MISSING does not create a backup directory.
    assert!(
        !target.join(".run-bob-backup").exists(),
        ".run-bob-backup must NOT exist when only MISSING was applied"
    );

    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(
        stdout.contains("installed"),
        "expected 'installed' in summary, got:\n{}",
        stdout
    );
}
```

- [ ] **Step 2: Run the test (must fail — file isn't re-installed yet)**

Run: `cd /Users/xiaojin/workshop/run-bob && cargo test --test integration upgrade_installs_missing_skill 2>&1 | tail -15`

Expected: failure on `assert!(skill.is_file(), ...)` because Task 4's apply path only handles OUTDATED.

- [ ] **Step 3: Add the MISSING install loop to `upgrade.rs`**

In `/Users/xiaojin/workshop/run-bob/src/commands/upgrade.rs`, find the comment `// MISSING handling lands in Task 5.` and replace that single line with:

```rust
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
```

- [ ] **Step 4: Run the missing-install test (must pass)**

Run: `cd /Users/xiaojin/workshop/run-bob && cargo test --test integration upgrade_installs_missing_skill`

Expected: pass.

- [ ] **Step 5: Run the full suite**

Run: `cd /Users/xiaojin/workshop/run-bob && cargo test`

Expected: **24 passed; 0 failed** (16 prior + 8 new from Tasks 1–5).

- [ ] **Step 6: Commit**

```bash
cd /Users/xiaojin/workshop/run-bob && git add src/commands/upgrade.rs tests/integration.rs && git -c commit.gpgsign=false commit -m "$(cat <<'EOF'
feat(upgrade): install MISSING upgrade-safe assets

Files that should exist per HARNESS_ASSETS but are absent on disk
are installed (parent dirs created as needed) without a backup —
nothing to preserve. Completes the upgrade behavior per spec §4.
EOF
)"
```

---

## Task 6: README "Upgrade harness assets" section

**Files:**
- Modify: `README.md`

**Goal:** Document the new subcommand for end users. No test (doc-only).

- [ ] **Step 1: Locate the existing "Update" section**

Run: `grep -n '^### Update' /Users/xiaojin/workshop/run-bob/README.md`

Expected: a line like `LINE:### Update`. The current "Update" section only talks about re-running the install one-liner to refresh the binary.

- [ ] **Step 2: Read the section to anchor your edit**

Open `/Users/xiaojin/workshop/run-bob/README.md`. Find this block:

```markdown
### Update

Re-run the same one-liner — the installer always pulls the latest release and overwrites in place.
```

- [ ] **Step 3: Append an "Upgrade harness assets in a project" subsection right after that paragraph**

Insert after the existing "Re-run the same one-liner..." sentence (and before the next `###` header), this block:

```markdown
### Upgrade harness assets in a project

After upgrading the `run-bob` binary itself, run this **inside any project that was previously `run-bob init`'d** to sync the harness assets (skills + shared Java skeletons + `README-RUN-BOB.md`) to the new version:

```bash
cd your-project/
run-bob upgrade
```

What `upgrade` does:

- **Compares** the on-disk content of each upgrade-safe asset against the binary's embedded version (byte-for-byte).
- **Backs up** any file that differs to `.run-bob-backup/<UTC-timestamp>/<original-path>` (add this directory to `.gitignore`), then **overwrites** with the embedded version.
- **Installs** any upgrade-safe asset that's missing.
- **Never touches** user-owned files: `CLAUDE.md` (your project rules), `ARCHITECTURE.md` (the 4-ring SSoT), and `src/test/java/architecture/CleanArchitectureTest.java` (your `FORBIDDEN_IN_INNER` list). Use `run-bob init --force` if you really want to reset those.

Flags:

| Flag | Effect |
|---|---|
| `--dir <path>` | Target project directory (default `.`) |
| `--dry-run` / `-n` | Report what would change; write nothing |
| `--no-backup` | Skip the safety backup (use only when you trust git to recover) |

Typical flow after a `run-bob` binary upgrade:

```bash
# 1. Upgrade the binary (existing one-liner; pulls latest release)
curl -fsSL https://raw.githubusercontent.com/amwtke/run-bob/master/install.sh | sh

# 2. Preview what would change in your project
cd your-project/
run-bob upgrade --dry-run

# 3. Apply
run-bob upgrade
```
```

(Note the triple-backtick `bash` blocks are nested inside a markdown insertion; preserve them when pasting.)

- [ ] **Step 4: Verify the section renders**

Run: `grep -A2 '^### Upgrade harness assets' /Users/xiaojin/workshop/run-bob/README.md`

Expected: the section heading + the first two lines you just inserted.

- [ ] **Step 5: Commit**

```bash
cd /Users/xiaojin/workshop/run-bob && git add README.md && git -c commit.gpgsign=false commit -m "$(cat <<'EOF'
docs(readme): document `run-bob upgrade` for refreshing harness assets

Adds an "Upgrade harness assets in a project" subsection covering
what upgrade syncs vs. what it leaves alone, the --dry-run and
--no-backup flags, and the typical post-binary-upgrade flow.
EOF
)"
```

---

## Final Verification

After all six tasks are committed:

- [ ] **Run the full integration suite**

Run: `cd /Users/xiaojin/workshop/run-bob && cargo test`

Expected: **24 passed; 0 failed**.

- [ ] **Smoke-test the binary on a real tempdir**

Run:

```bash
cd /Users/xiaojin/workshop/run-bob \
  && cargo build --release \
  && tmp=$(mktemp -d) \
  && ./target/release/run-bob init --dir "$tmp" >/dev/null \
  && ./target/release/run-bob upgrade --dir "$tmp" \
  && echo "STALE" > "$tmp/.claude/skills/bob-identify/SKILL.md" \
  && ./target/release/run-bob upgrade --dry-run --dir "$tmp" \
  && ./target/release/run-bob upgrade --dir "$tmp" \
  && ls "$tmp/.run-bob-backup"/*/.claude/skills/bob-identify/SKILL.md \
  && rm -rf "$tmp"
```

Expected, in order:
1. First `upgrade` after `init` → `All upgrade-safe assets are up to date.`
2. After corrupting the skill, `upgrade --dry-run` → reports `outdated` and `dry-run: no files would be written.`
3. `upgrade` (non-dry) → creates a backup, applies the update, reports `1 updated`.
4. The `ls` confirms a backed-up `SKILL.md` exists with the corrupted content.

- [ ] **Verify the spec is fully covered**

| Spec section | Implemented in |
|---|---|
| §2 CLI surface (`--dir / --dry-run / --no-backup`) | Task 2 |
| §3 `Asset.upgrade_safe` + per-asset values | Task 1 |
| §4 Detection (MISSING / UP-TO-DATE / OUTDATED) | Task 3 |
| §5 Backup at `.run-bob-backup/<ts>/` | Task 4 |
| §6 Output format (✓ / ↑ / + / ℹ / 📦) | Tasks 3, 4, 5 |
| §7.4 In-house timestamp (no new dep) | Task 4 |
| §8 All 8 tests | Tasks 1–5 |
| §9 README "Update" section | Task 6 |

No spec section is uncovered.
