# run-bob Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build `run-bob` — a Rust CLI that, like `ddd-run`, ships an embedded harness for Claude Code projects, but targets pure Bob 4-ring Clean Architecture (single Bounded Context, sync, no Domain Event by default) instead of DDD tactical level.

**Architecture:** Rust 1.75+ / Cargo / clap. All templates embedded at compile time via `include_str!()`. The CLI exposes two subcommands: `run-bob init` (install harness assets to a target directory) and `run-bob status` (verify install). Templates are language-aware (Java/Spring defaults, ArchUnit guard) but the CLI itself is language-agnostic. Engineering form mirrors `ddd-run` 1:1 — what differs is the *content* of the templates and skill workflow (identify → onion → spec) plus a participatory G/B1/B2 mode dispatch.

**Tech Stack:** Rust 1.75+, `clap = "=4.5.4"`, `anyhow = "1.0"`, `colored = "2.1"`, dev-dep `tempfile = "3"`. Templates target Java 17 / Spring Boot but the CLI delivers them as static text — it does not parse or compile Java.

**Spec:** [`docs/superpowers/specs/2026-05-08-run-bob-design.md`](../specs/2026-05-08-run-bob-design.md)

**Reference implementation:** `/Users/xiaojin/workspace/ddd-run/` — same Cargo.toml dependencies, same `init`/`status` shape, same `include_str!()` mechanism. When a task says "照搬 ddd-run X", you can `cat /Users/xiaojin/workspace/ddd-run/X` and adapt.

---

## File Structure

| File | Action | Responsibility |
|---|---|---|
| `Cargo.toml` | **Create** | Cargo manifest, name = "run-bob", dependencies + dev-dep `tempfile` |
| `src/main.rs` | **Create** | CLI entry point: clap parser + dispatch to `commands::init` / `commands::status` |
| `src/commands/mod.rs` | **Create** | Module declaration for `init` + `status` |
| `src/commands/init.rs` | **Create** | `run-bob init` impl: install templates to target dir |
| `src/commands/status.rs` | **Create** | `run-bob status` impl: verify install |
| `src/templates/skills/bob-identify.md` | **Create** | Skill 1 — 5-question decision tree, G/B1/B2 modes |
| `src/templates/skills/bob-onion.md` | **Create** | Skill 2 — 4-ring architecture design + ARCHITECTURE.md maintenance |
| `src/templates/skills/bob-spec.md` | **Create** | Skill 3 — per-use-case spec → Superpowers handoff |
| `src/templates/root/CLAUDE.md` | **Create** | Project-level rules R0-R12 |
| `src/templates/root/ARCHITECTURE.md` | **Create** | 4-ring SSoT template (replaces ddd-run's DOMAIN.md) |
| `src/templates/root/README-RUN-BOB.md` | **Create** | In-project user guide |
| `src/templates/root/CleanArchitectureTest.java` | **Create** | ArchUnit guard, parameterized `FORBIDDEN_IN_INNER` |
| `src/templates/root/UseCase.java` | **Create** | Shared `UseCase<C, R>` interface skeleton |
| `src/templates/root/TransactionalUseCaseDecorator.java` | **Create** | Shared decorator with the **only** `@Transactional` |
| `tests/integration.rs` | **Create** | Integration tests for init / status (via `tempfile`) |
| `.claude/skills/install/SKILL.md` | **Create** | Local skill for cargo install (mirrors ddd-run's) |

**Already committed (do not touch in this plan):**
- `.gitignore`, `LICENSE`, `README.md`
- `docs/superpowers/specs/2026-05-08-run-bob-design.md` (the design spec)

---

## Task 1: Bootstrap Rust project skeleton

**Files:**
- Create: `Cargo.toml`
- Create: `src/main.rs`
- Create: `src/commands/mod.rs`
- Create: `src/commands/init.rs`
- Create: `src/commands/status.rs`

- [ ] **Step 1: Write `Cargo.toml`**

Write `/Users/xiaojin/workspace/run-bob/Cargo.toml`:

```toml
[package]
name = "run-bob"
version = "0.1.0"
edition = "2021"
authors = ["Jin <amwtke@github>"]
description = "A CLI to bootstrap Bob's 4-ring Clean Architecture + Superpowers harness for Claude Code projects"
license = "MIT"
repository = "https://github.com/amwtke/run-bob"
readme = "README.md"

[dependencies]
# Pinned to versions compatible with Rust 1.75+.
# Newer clap_lex (>=1.0.1) requires edition2024 (Rust 1.85+).
clap = { version = "=4.5.4", features = ["derive"] }
anyhow = "1.0"
colored = "2.1"

[dev-dependencies]
tempfile = "3"

[profile.release]
opt-level = "z"
lto = true
codegen-units = 1
strip = true
```

- [ ] **Step 2: Write `src/main.rs`**

Write `/Users/xiaojin/workspace/run-bob/src/main.rs`:

```rust
//! run-bob: A CLI to bootstrap Bob's 4-ring Clean Architecture + Superpowers harness for Claude Code projects.

use anyhow::Result;
use clap::{Parser, Subcommand};
use colored::*;

mod commands;

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

- [ ] **Step 3: Write `src/commands/mod.rs`**

Write `/Users/xiaojin/workspace/run-bob/src/commands/mod.rs`:

```rust
pub mod init;
pub mod status;
```

- [ ] **Step 4: Write `src/commands/init.rs` (helpers + empty run())**

Write `/Users/xiaojin/workspace/run-bob/src/commands/init.rs`:

```rust
//! `run-bob init` command — installs harness assets into the target directory.

use anyhow::{Context, Result};
use colored::*;
use std::fs;
use std::path::{Path, PathBuf};

// Templates will be embedded here as include_str!() in subsequent tasks.

pub fn run(target_dir: &str, force: bool, minimal: bool) -> Result<()> {
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
    println!();

    // Skill / template installation will be wired in subsequent tasks.

    print_next_steps(minimal);

    Ok(())
}

/// Install a skill as `.claude/skills/<name>/SKILL.md`.
#[allow(dead_code)]
fn install_skill(target: &Path, name: &str, content: &str, force: bool) -> Result<()> {
    let skill_dir = target.join(".claude").join("skills").join(name);
    ensure_dir(&skill_dir)?;
    let path = skill_dir.join("SKILL.md");
    write_file(&path, content, force, &format!(".claude/skills/{}/SKILL.md", name))
}

/// Install a root-level file.
#[allow(dead_code)]
fn install_root_file(target: &Path, name: &str, content: &str, force: bool) -> Result<()> {
    let path = target.join(name);
    write_file(&path, content, force, name)
}

/// Install a Java file at an arbitrary path under target.
#[allow(dead_code)]
fn install_java_file(
    target: &Path,
    rel_path: &[&str],
    content: &str,
    force: bool,
) -> Result<()> {
    let mut path = target.to_path_buf();
    for seg in rel_path {
        path = path.join(seg);
    }
    let display = rel_path.join("/");
    write_file(&path, content, force, &display)
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
    crate::success(display);
    Ok(())
}

/// Make sure a directory exists.
#[allow(dead_code)]
fn ensure_dir(path: &Path) -> Result<()> {
    fs::create_dir_all(path)
        .with_context(|| format!("Failed to create directory {}", path.display()))
}

fn print_next_steps(minimal: bool) {
    println!();
    println!("{}", "━".repeat(60).bright_black());
    println!("{}", "Next steps".bold().green());
    println!("{}", "━".repeat(60).bright_black());

    if minimal {
        println!("  Skills installed. Open Claude Code and try:");
        println!("    {} /bob-identify <your business description>", "•".cyan());
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
    println!("  2. {} open Claude Code in this directory.", "Launch".cyan());
    println!();
    println!("  3. {} start the workflow:", "Run".cyan());
    println!(
        "       {}",
        "/bob-identify <your business description>".bold().green()
    );
    println!("       {}", "/bob-onion".bold().green());
    println!(
        "       {}",
        "/bob-spec <use case>".bold().green()
    );
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
```

- [ ] **Step 5: Write `src/commands/status.rs` (helpers + empty run())**

Write `/Users/xiaojin/workspace/run-bob/src/commands/status.rs`:

```rust
//! `run-bob status` — check whether the harness is properly installed.

use anyhow::{Context, Result};
use colored::*;
use std::path::{Path, PathBuf};

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

    // Skill / harness / shared / archunit checks will be wired in subsequent tasks.
    // For now, just print a stub.
    println!("{}", "(no checks wired yet — bootstrap stage)".bright_black());
    let _ = (target, &mut all_ok);

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

#[allow(dead_code)]
fn check(target: &Path, rel: &str) -> bool {
    let p = target.join(rel);
    if p.is_file() {
        println!("  {} {}", "✓".green(), rel);
        true
    } else {
        println!("  {} {}", "✗".red(), rel.red());
        false
    }
}

#[allow(dead_code)]
fn check_dir(target: &Path, rel: &str) -> bool {
    let p = target.join(rel);
    if p.is_dir() {
        println!("  {} {}/", "✓".green(), rel);
        true
    } else {
        println!("  {} {}/", "✗".red(), rel.red());
        false
    }
}
```

- [ ] **Step 6: Verify build passes**

Run: `cd /Users/xiaojin/workspace/run-bob && cargo build 2>&1 | tail -20`
Expected: `Finished \`dev\` profile [unoptimized + debuginfo] target(s)` with 0 errors. (Warnings about `dead_code` are tolerated for unused helper functions; they will be wired in next tasks.)

- [ ] **Step 7: Smoke test the binary**

Run: `cd /Users/xiaojin/workspace/run-bob && ./target/debug/run-bob --version`
Expected: `run-bob 0.1.0`

Run: `./target/debug/run-bob --help`
Expected: shows `init` and `status` subcommands.

- [ ] **Step 8: Commit**

```bash
cd /Users/xiaojin/workspace/run-bob
git add Cargo.toml src/
git commit -m "feat(cli): bootstrap Rust project skeleton with init/status subcommands"
```

---

## Task 2: Bootstrap integration test infrastructure

**Files:**
- Create: `tests/integration.rs`

- [ ] **Step 1: Write `tests/integration.rs` with smoke test**

Write `/Users/xiaojin/workspace/run-bob/tests/integration.rs`:

```rust
//! Integration tests for run-bob.
//! Each test creates a tempdir, runs init/status, and verifies filesystem state.

use std::process::Command;

/// Path to the cargo-built binary under test.
fn run_bob_bin() -> std::path::PathBuf {
    // CARGO_BIN_EXE_<name> is set by Cargo for integration tests.
    std::path::PathBuf::from(env!("CARGO_BIN_EXE_run-bob"))
}

#[test]
fn binary_prints_version() {
    let output = Command::new(run_bob_bin())
        .arg("--version")
        .output()
        .expect("run run-bob --version");
    assert!(output.status.success(), "run-bob --version failed");
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(
        stdout.contains("run-bob 0.1.0"),
        "expected version in output, got: {}",
        stdout
    );
}

#[test]
fn init_help_lists_flags() {
    let output = Command::new(run_bob_bin())
        .args(["init", "--help"])
        .output()
        .expect("run run-bob init --help");
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    for flag in &["--force", "--minimal", "--dir"] {
        assert!(stdout.contains(flag), "expected {} flag in help: {}", flag, stdout);
    }
}
```

- [ ] **Step 2: Run integration tests**

Run: `cd /Users/xiaojin/workspace/run-bob && cargo test 2>&1 | tail -10`
Expected: 2 tests pass (`binary_prints_version`, `init_help_lists_flags`).

- [ ] **Step 3: Commit**

```bash
cd /Users/xiaojin/workspace/run-bob
git add tests/integration.rs
git commit -m "test: bootstrap integration test infrastructure with tempfile"
```

---

## Task 3: Install /install local skill

**Files:**
- Create: `.claude/skills/install/SKILL.md`

- [ ] **Step 1: Copy ddd-run's install skill verbatim, then string-replace**

Run: `cp /Users/xiaojin/workspace/ddd-run/.claude/skills/install/SKILL.md /tmp/install-source.md`

Then read `/tmp/install-source.md` and write `/Users/xiaojin/workspace/run-bob/.claude/skills/install/SKILL.md` with these substitutions applied throughout:

| Find | Replace with |
|---|---|
| `ddd-run` | `run-bob` |
| `[package].name = "ddd-run"` | `[package].name = "run-bob"` |
| `~/.cargo/bin/ddd-run` | `~/.cargo/bin/run-bob` |
| `ddd-run --version` | `run-bob --version` |
| `ddd-run --help` | `run-bob --help` |
| `ddd-run init` | `run-bob init` |
| `ddd-run status` | `run-bob status` |

Implementation hint: a single `sed -e 's/ddd-run/run-bob/g'` will do all of these (the `[package].name` replacement is the same string), since `ddd-run` is unique in the source file.

```bash
mkdir -p /Users/xiaojin/workspace/run-bob/.claude/skills/install
sed 's/ddd-run/run-bob/g' /Users/xiaojin/workspace/ddd-run/.claude/skills/install/SKILL.md \
  > /Users/xiaojin/workspace/run-bob/.claude/skills/install/SKILL.md
```

- [ ] **Step 2: Verify content**

Run: `grep -c 'ddd-run' /Users/xiaojin/workspace/run-bob/.claude/skills/install/SKILL.md`
Expected: 0 (no leftover references)

Run: `grep -c 'run-bob' /Users/xiaojin/workspace/run-bob/.claude/skills/install/SKILL.md`
Expected: ≥ 8 (multiple references throughout the doc)

- [ ] **Step 3: Commit**

```bash
cd /Users/xiaojin/workspace/run-bob
git add .claude/skills/install/SKILL.md
git commit -m "feat(skills): add local /install skill for cargo install workflow"
```

---

## Task 4: ARCHITECTURE.md template (TDD)

**Files:**
- Create: `src/templates/root/ARCHITECTURE.md`
- Modify: `src/commands/init.rs` (add include_str! + install_root_file call)
- Modify: `src/commands/status.rs` (add check for ARCHITECTURE.md)
- Modify: `tests/integration.rs` (assert ARCHITECTURE.md is created)

- [ ] **Step 1: Write the failing integration test**

Edit `/Users/xiaojin/workspace/run-bob/tests/integration.rs` to add at the bottom:

```rust
#[test]
fn init_creates_architecture_md() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let target = tmp.path();

    let status = Command::new(run_bob_bin())
        .args(["init", "--dir"])
        .arg(target)
        .status()
        .expect("run run-bob init");
    assert!(status.success(), "run-bob init failed");

    let p = target.join("ARCHITECTURE.md");
    assert!(p.is_file(), "missing {}", p.display());
    let content = std::fs::read_to_string(&p).unwrap();
    assert!(
        content.contains("# 架构(Bob 4 环)"),
        "ARCHITECTURE.md must have Bob 4-ring header"
    );
    assert!(
        content.contains("Single Source of Truth"),
        "ARCHITECTURE.md must declare itself as SSoT"
    );
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cd /Users/xiaojin/workspace/run-bob && cargo test init_creates_architecture_md 2>&1 | tail -15`
Expected: FAIL with "missing .../ARCHITECTURE.md".

- [ ] **Step 3: Write the ARCHITECTURE.md template**

Create `/Users/xiaojin/workspace/run-bob/src/templates/root/ARCHITECTURE.md` using **spec §4.4** as the source of truth (verbatim). The file should:

1. Start with `# 架构(Bob 4 环)· <项目/上下文名>`
2. Include the SSoT declaration block: "本文档是本项目 Bob 4 环架构的 Single Source of Truth..."
3. Include all 11 sections from spec §4.4:
   - §📌 状态(模式 G/B1/B2 复选框 + workflow 复选框)
   - §1 上下文(Context)
   - §2 4 环包结构(包含 4-ring ASCII art)
   - §3 核心 Entity 与状态机(模板 §3.x `<Entity 名>`)
   - §4 端口清单(usecase/port/)
   - §5 UseCase 清单
   - §6 配件清单(项目特化)
   - §7 装配点(framework/)
   - §8 α/β/γ 评级与重构计划(仅 B1/B2)
   - §9 ArchUnit 作用域(包含 G/B1 默认 + B2 多包注释)
   - §10 ADR(包含 ADR-1 / ADR-2 模板)
   - §11 下一步

The §9 ArchUnit example **must** include both the default `packages = "com.example"` form and the B2 array form `packages = {"com.example.<feature>", "com.example.shared"}`.

The §6 配件清单 table must include at minimum these rows: Spring, SLF4J, 达梦驱动 (used as illustrative — leave them as `_待填充_` rows initially or as illustrative pre-populated rows; spec §1.3 has the canonical table to draw from).

Reference: `/Users/xiaojin/workspace/run-bob/docs/superpowers/specs/2026-05-08-run-bob-design.md` §4.4 (lines 355-435).

End the file with a trailing line: `*Managed by run-bob + /bob-onion skill.*`

- [ ] **Step 4: Wire it through `init.rs`**

Edit `/Users/xiaojin/workspace/run-bob/src/commands/init.rs`:

After the line `// Templates will be embedded here as include_str!() in subsequent tasks.`, replace with:

```rust
const ROOT_ARCHITECTURE: &str = include_str!("../templates/root/ARCHITECTURE.md");
```

Inside `pub fn run(...)`, after the print statements and before `print_next_steps(minimal)`, add:

```rust
if !minimal {
    println!("{}", "Installing harness documents...".bold());
    install_root_file(&target, "ARCHITECTURE.md", ROOT_ARCHITECTURE, force)?;
}
```

Remove `#[allow(dead_code)]` from `install_root_file` and `write_file` if present.

- [ ] **Step 5: Run integration test, expect pass**

Run: `cd /Users/xiaojin/workspace/run-bob && cargo test init_creates_architecture_md 2>&1 | tail -10`
Expected: PASS.

- [ ] **Step 6: Wire status check**

Edit `/Users/xiaojin/workspace/run-bob/src/commands/status.rs`. Replace the stub block with:

```rust
println!("{}", "Harness documents".bold());
all_ok &= check(&target, "ARCHITECTURE.md");
```

Remove `#[allow(dead_code)]` from `check`.

- [ ] **Step 7: Run all tests**

Run: `cd /Users/xiaojin/workspace/run-bob && cargo test 2>&1 | tail -10`
Expected: 3 tests pass.

- [ ] **Step 8: Commit**

```bash
cd /Users/xiaojin/workspace/run-bob
git add src/templates/root/ARCHITECTURE.md src/commands/init.rs src/commands/status.rs tests/integration.rs
git commit -m "feat(templates): add ARCHITECTURE.md as 4-ring SSoT template"
```

---

## Task 5: CLAUDE.md template (R0-R12) (TDD)

**Files:**
- Create: `src/templates/root/CLAUDE.md`
- Modify: `src/commands/init.rs`
- Modify: `src/commands/status.rs`
- Modify: `tests/integration.rs`

- [ ] **Step 1: Write the failing test**

Append to `/Users/xiaojin/workspace/run-bob/tests/integration.rs`:

```rust
#[test]
fn init_creates_claude_md_with_r0() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let target = tmp.path();

    Command::new(run_bob_bin())
        .args(["init", "--dir"])
        .arg(target)
        .status()
        .expect("init");

    let p = target.join("CLAUDE.md");
    assert!(p.is_file(), "CLAUDE.md missing");
    let content = std::fs::read_to_string(&p).unwrap();

    // Must declare itself a run-bob harness
    assert!(content.contains("run-bob"), "CLAUDE.md must reference run-bob");
    // R0 meta-rule must be present
    assert!(content.contains("R0"), "CLAUDE.md must have R0 meta-rule");
    assert!(
        content.contains("通用判定优先于具体清单") || content.contains("5 问决策树"),
        "R0 must reference the decision tree"
    );
    // R12 must be present (B2 mode)
    assert!(content.contains("R12"), "CLAUDE.md must have R12 (B2 clean island)");
    // 4-ring package names use "entity" (not "domain")
    assert!(content.contains("entity"), "CLAUDE.md must use 'entity' not 'domain' as Ring 1");
    // Must have technology-stack-pending warning
    assert!(
        content.contains("## 技术栈约定"),
        "CLAUDE.md must have ## 技术栈约定 section"
    );
}
```

- [ ] **Step 2: Run, expect fail**

Run: `cd /Users/xiaojin/workspace/run-bob && cargo test init_creates_claude_md_with_r0 2>&1 | tail -10`
Expected: FAIL with "CLAUDE.md missing".

- [ ] **Step 3: Write the CLAUDE.md template**

Create `/Users/xiaojin/workspace/run-bob/src/templates/root/CLAUDE.md` based on spec §6 ("CLAUDE.md 模板:R0-R12"). Source the structure from `/Users/xiaojin/workspace/ddd-run/src/templates/root/CLAUDE.md` and apply these transformations:

**Headers / wording:**
- "DDD + Superpowers harness" → "Bob 4 环 Clean Architecture + Superpowers harness"
- "ddd-run" → "run-bob" (everywhere)

**Mode block (new, immediately after `## 项目定位`):**

```markdown
## 模式
- [ ] G(绿地新项目)
- [ ] B1(棕地全量重构)
- [ ] B2(棕地增量新功能 — 清洁孤岛)
```

**Diagram (replace ddd-run's 分层架构 ASCII):**
Use spec §6 / spec §0.3 / `README.md` §"Why this exists" — the 4-ring with `entity / usecase / adapter / framework`.

**R0 (new, before R1):**

```markdown
### R0. 通用判定优先于具体清单

本文件 R7-R12 列出的"禁止 import"清单(Spring / SLF4J / Jakarta / Lombok)只是
2026 年 Java/Spring 生态的典型样本,不构成穷举。

遇到任何新外部库 / 新注解 / 新框架 / 新信创组件,必须先跑 `ARCHITECTURE.md §配件清单`
扩充,且在 inner 包(entity / usecase)使用前先跑 5 问决策树
(详见 `/bob-identify` skill 与 ARCHITECTURE.md §配件清单 章节)。

判定为配件的所有外部依赖,无论是否在 R7-R12 显式列出,都不允许出现在 entity/** 与 usecase/** 包。
R0 是 R7-R12 的母规则——R7-R12 是 R0 的当前可执行实例。
```

**R1 (modify ddd-run R1):** Change the workflow steps to:

```
/bob-identify → /bob-onion(更新 ARCHITECTURE.md) → /bob-spec
              → superpowers:brainstorming(若 ## 技术栈约定 未填)
              → superpowers:writing-plans
              → superpowers:executing-plans(TDD)
              → superpowers:finishing-a-development-branch
```

**R2:** Anchor changed from `DOMAIN.md` to `ARCHITECTURE.md §3 (Entity)` + `§4 (端口)` + `§5 (UseCase)`.

**R3:** Same富 Entity model + UseCase 编排 / 禁止贫血 wording. The R3 补充 (Clean Architecture) paragraph stays.

**R4 (replace ddd-run R4 wholesale):**

```markdown
### R4-bob. Entity 状态迁移自封

Bob 不强调 DDD 聚合边界(那是 DDD 战术级);相反,本规则要求每个 Entity 自封
所有状态迁移规则。

- ✅ `order.payTo(paymentGateway, inventoryClient)` — Entity 内 `ensureStatus(...)` 守
- ❌ `if (order.getStatus() == CREATED) order.setStatus(PAID)` — 外部修改(贫血)

参考 atlas Stage 5 §4.1 动作 3:状态机上提到 Entity。
```

**R5:** Wording `<EntityName>Repository`(不再叫"聚合根 Repository").

**R6:** unchanged (TDD 节奏).

**R7 (new package layout):** Replace ddd-run's `..domain..` references with `..entity..`. Show the 4-ring directory tree using `entity / usecase / adapter / framework` (no `domain/` folder). Include the shared/ skeleton directory tree:

```
com.example.<bizname>/
├── entity/                                  Ring 1 — POJO 状态机
│   ├── <Entity>.java
│   ├── <ValueObject>.java
│   └── <EntityName>Id.java
├── usecase/                                 Ring 2 — Interactor (POJO)
│   ├── <Command>Command.java                record
│   ├── <Command>UseCase.java                implements UseCase<C, R>
│   ├── <Command>Result.java                 record
│   └── port/
│       ├── <EntityName>Repository.java      端口接口
│       ├── <Gateway>.java                   出站端口
│       ├── ClockPort.java                   时钟端口(若需)
│       └── LoggerPort.java                  日志端口(若需)
├── adapter/                                 Ring 3 — 允许 Spring/JPA
│   ├── web/<Aggregate>Controller.java
│   ├── persistence/...
│   ├── acl/<Gateway>HttpAcl.java
│   ├── time/SystemClockAdapter.java
│   └── logging/Slf4jLoggerAdapter.java
└── framework/                               Ring 4 — 装配
    └── config/<Feature>UseCaseConfig.java   @Bean 装配 + 装饰器包裹

com.example.shared/
├── usecase/UseCase.java                     通用 UseCase<C, R>
└── framework/transaction/
    └── TransactionalUseCaseDecorator.java   全工程唯一 @Transactional
```

**R8:** unchanged from ddd-run (TransactionalUseCaseDecorator 唯一).

**R9 (扩充):** Copy ddd-run R9 list verbatim, then append:
- ❌ Entity 不得 `LocalDateTime.now()` / `UUID.randomUUID()` / `System.currentTimeMillis()`(用 ClockPort / IdGenerator)
- ❌ 决策树判为配件的库一律不得在 inner 包(entity / usecase),即使本文件未显式列出
- ❌ B2 模式:新功能不得 import legacy 的 `@Service` 类(必须通过 usecase/port 端口 + adapter/acl ACL 隔离)

**R10 (replace ddd-run R10):**

```markdown
### R10-bob. 跨上下文 / 异步 = 升级触发器

Bob 默认假设单 Bounded Context + 同步业务。

- 出现"跨聚合协作"、"事件驱动"、"Saga 补偿"、"最终一致性"需求时:
  1. 停下,不要直接引入 `@EventListener` / `Outbox` / `ApplicationEventPublisher`
  2. 在 `ARCHITECTURE.md §10 ADR` 记录"升级到 DDD 战术级"的决定
  3. 修改 `CleanArchitectureTest.java` 的 `no_event_listener_unless_decided` 规则到 messaging 限定
  4. 考虑切换到 ddd-run harness(它支持 Domain Event 一等公民)

参考 atlas Stage 6 §2.5:Bob `order.payTo(pg, ic)` Sync vs DDD `order.pay()` + Event Async 的
差异。Bob 的 Sync 风格在跨 BC 场景下退化,这是触发升级的信号。
```

**R11:** unchanged (ArchUnit 守卫).

**R12 (new):**

```markdown
### R12. 增量新功能(B2)必须 γ,即使周围是 α/β legacy

棕地增量新功能场景下:

- 新功能落在独立包(如 `com.example.<feature>`),与 legacy 同级或子级,**不混在 legacy 包内**
- 新功能不允许跨包"复用" legacy `@Service`;若需调用,通过 usecase/port 端口 + adapter/acl ACL 包装
- 新包内禁止"为兼容 legacy 风格"的妥协(如新 usecase 加 `@Service` "保持一致")
- ArchUnit `@AnalyzeClasses` 改为多包数组,**必须**包含 `com.example.shared`,否则
  `transactional_methods_only_in_decorator` 规则评估不到装饰器
  
  ```java
  @AnalyzeClasses(
      packages = {"com.example.<feature>", "com.example.shared"},
      importOptions = DoNotIncludeTests.class
  )
  ```

legacy 是另一个"外部世界",和 MySQL / 人大金仓 / 微信支付 SDK 没区别,统一用端口 + ACL 隔离。
```

**Other sections:**
- `## 修改 ARCHITECTURE.md 的流程` (replaces ddd-run's 修改 DOMAIN.md 的流程, same wording with `ARCHITECTURE.md` and `/bob-onion` substituted)
- `## 代码质量底线`:照搬 ddd-run, replace "聚合" with "Entity",add a bullet "禁止 `LocalDateTime.now()` / `UUID.randomUUID()` 在 inner 包(用 ClockPort / IdGenerator)"

End the file with `*Generated by run-bob. 本文件可根据项目实际情况调整,但不要删除"强制规则"部分。*`

Reference: `/Users/xiaojin/workspace/ddd-run/src/templates/root/CLAUDE.md` (245 lines) as the structural skeleton; transform per spec §6.

- [ ] **Step 4: Wire through init.rs**

Edit `/Users/xiaojin/workspace/run-bob/src/commands/init.rs`. Add at the top with other constants:

```rust
const ROOT_CLAUDE_MD: &str = include_str!("../templates/root/CLAUDE.md");
```

In `run()` inside the `if !minimal {` block (where ARCHITECTURE.md is installed), add:

```rust
install_root_file(&target, "CLAUDE.md", ROOT_CLAUDE_MD, force)?;
```

Place it **before** the ARCHITECTURE.md install so the user sees CLAUDE.md first in the output.

- [ ] **Step 5: Run integration test**

Run: `cd /Users/xiaojin/workspace/run-bob && cargo test init_creates_claude_md_with_r0 2>&1 | tail -10`
Expected: PASS.

- [ ] **Step 6: Wire status check**

Edit `/Users/xiaojin/workspace/run-bob/src/commands/status.rs`. Add `check(&target, "CLAUDE.md");` line **before** the ARCHITECTURE.md check inside the "Harness documents" section:

```rust
println!("{}", "Harness documents".bold());
all_ok &= check(&target, "CLAUDE.md");
all_ok &= check(&target, "ARCHITECTURE.md");
```

- [ ] **Step 7: Run all tests**

Run: `cd /Users/xiaojin/workspace/run-bob && cargo test 2>&1 | tail -10`
Expected: 4 tests pass.

- [ ] **Step 8: Commit**

```bash
cd /Users/xiaojin/workspace/run-bob
git add src/templates/root/CLAUDE.md src/commands/init.rs src/commands/status.rs tests/integration.rs
git commit -m "feat(templates): add CLAUDE.md with R0-R12 hard rules for Bob 4-ring"
```

---

## Task 6: README-RUN-BOB.md template (TDD)

**Files:**
- Create: `src/templates/root/README-RUN-BOB.md`
- Modify: `src/commands/init.rs`, `src/commands/status.rs`, `tests/integration.rs`

- [ ] **Step 1: Write failing test**

Append to `/Users/xiaojin/workspace/run-bob/tests/integration.rs`:

```rust
#[test]
fn init_creates_readme_run_bob_with_3_modes() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let target = tmp.path();
    Command::new(run_bob_bin()).args(["init", "--dir"]).arg(target).status().expect("init");

    let p = target.join("README-RUN-BOB.md");
    assert!(p.is_file(), "README-RUN-BOB.md missing");
    let content = std::fs::read_to_string(&p).unwrap();
    for token in &[
        "/bob-identify",
        "/bob-onion",
        "/bob-spec",
        "ARCHITECTURE.md",
        "G(",     // G mode
        "B1",     // B1 mode
        "B2",     // B2 mode
    ] {
        assert!(content.contains(token), "README-RUN-BOB.md must contain {}", token);
    }
}
```

- [ ] **Step 2: Run, expect fail**

Run: `cargo test init_creates_readme_run_bob_with_3_modes 2>&1 | tail -10`
Expected: FAIL.

- [ ] **Step 3: Write README-RUN-BOB.md**

Create `/Users/xiaojin/workspace/run-bob/src/templates/root/README-RUN-BOB.md` by adapting `/Users/xiaojin/workspace/ddd-run/src/templates/root/README-DDD-HARNESS.md` per spec §10 (the table of substitutions).

Required sections (must exist verbatim or paraphrased):

1. `# Bob 4 环 Clean Architecture + Superpowers Harness`
2. `## Clean Architecture 速览(Bob 同心圆 4 环)` — 4-ring ASCII diagram
3. `### atlas Stage 5 §4 三个落地动作`:
   - 接口位置反转
   - 框架边界外推
   - 状态机上提到 Entity
4. `### ArchUnit 守卫`
5. `## 一、这套 harness 是什么?`
6. `## 二、目录布局` — show layout including `.claude/skills/{bob-identify,bob-onion,bob-spec}` + `CLAUDE.md` / `ARCHITECTURE.md` / `README-RUN-BOB.md` + `docs/{bob,specs}` + `src/main/java/com/example/shared/...` + `src/test/java/architecture/`
7. `## 三、两份锚点文档`:CLAUDE.md + ARCHITECTURE.md
8. `## 四、三个 Skill 的使用`:
   - `### 4.1 /bob-identify` — 5 问决策树 + G/B1/B2 三模式
   - `### 4.2 /bob-onion` — 4 环设计 + ARCHITECTURE.md SSoT
   - `### 4.3 /bob-spec` — 命令 / 查询 / 重构 三模板
9. `## 五、完整工作流` — workflow ASCII (replace ddd-run skill names with bob-*)
10. `## 六、一个完整示例` — use atlas Stage 5 OrderUseCase example (payOrder / shipOrder / cancelOrder), G mode walkthrough
11. `## 七、常见问题` — at minimum these Q&A:
    - Q: 可以跳过 `/bob-identify` 直接 `/bob-onion` 吗?(简化版可以,但建议至少跑一次 5 问决策树)
    - Q: ARCHITECTURE.md 什么时候会被修改?(只有 `/bob-onion` 修改它)
    - Q: 我已有 Spring 项目要加新功能,要全部重构吗?(不必。用 B2 模式开清洁孤岛,只对新功能严格 4 环。)
    - Q: 我用什么语言?(Superpowers brainstorming 决定;CLAUDE.md `## 技术栈约定` 段写回。)
    - Q: 我只需要做个原型/小脚本,真的要走完这么多步吗?(不必。本 harness 目标是长期迭代的领域系统。)
12. End: `*Generated by run-bob. 如需更新 harness,重新运行 \`run-bob init --force\`。*`

Reference: `/Users/xiaojin/workspace/ddd-run/src/templates/root/README-DDD-HARNESS.md` (255 lines) as the skeleton.

- [ ] **Step 4: Wire init.rs + status.rs**

Add to `init.rs`:
```rust
const ROOT_README: &str = include_str!("../templates/root/README-RUN-BOB.md");
```

In `run()` inside `if !minimal {`:
```rust
install_root_file(&target, "README-RUN-BOB.md", ROOT_README, force)?;
```

Place it **after** ARCHITECTURE.md install.

Add to `status.rs` "Harness documents" section, after ARCHITECTURE.md:
```rust
all_ok &= check(&target, "README-RUN-BOB.md");
```

- [ ] **Step 5: Run all tests**

Run: `cargo test 2>&1 | tail -10`
Expected: 5 tests pass.

- [ ] **Step 6: Commit**

```bash
cd /Users/xiaojin/workspace/run-bob
git add src/templates/root/README-RUN-BOB.md src/commands/init.rs src/commands/status.rs tests/integration.rs
git commit -m "feat(templates): add README-RUN-BOB.md in-project user guide"
```

---

## Task 7: CleanArchitectureTest.java template (TDD)

**Files:**
- Create: `src/templates/root/CleanArchitectureTest.java`
- Modify: `src/commands/init.rs`, `src/commands/status.rs`, `tests/integration.rs`

- [ ] **Step 1: Write failing test**

Append to `/Users/xiaojin/workspace/run-bob/tests/integration.rs`:

```rust
#[test]
fn init_installs_archunit_test_at_correct_path() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let target = tmp.path();

    Command::new(run_bob_bin()).args(["init", "--dir"]).arg(target).status().expect("init");

    let archunit = target
        .join("src").join("test").join("java")
        .join("architecture").join("CleanArchitectureTest.java");
    assert!(archunit.is_file(), "expected ArchUnit template at {}", archunit.display());

    let content = std::fs::read_to_string(&archunit).unwrap();
    assert!(content.contains("@ArchTest"), "must contain @ArchTest");
    assert!(content.contains("layered_dependencies"), "must include layered_dependencies");
    // run-bob uses 'entity' not 'domain'
    assert!(
        content.contains("entity_pure_of_frameworks"),
        "must include entity_pure_of_frameworks rule"
    );
    assert!(
        content.contains("usecase_pure_of_frameworks"),
        "must include usecase_pure_of_frameworks rule"
    );
    assert!(
        content.contains("FORBIDDEN_IN_INNER"),
        "must use parameterized FORBIDDEN_IN_INNER array"
    );
    assert!(
        content.contains("transactional_methods_only_in_decorator"),
        "must include transactional decorator rule"
    );
    assert!(
        content.contains("no_event_listener_unless_decided"),
        "must include R10-bob no_event_listener_unless_decided rule"
    );
    // Default analyze packages = "com.example" (covers shared + bizname)
    assert!(
        content.contains("packages = \"com.example\""),
        "default @AnalyzeClasses must use com.example to cover shared + business"
    );
}
```

- [ ] **Step 2: Run, expect fail**

Run: `cargo test init_installs_archunit_test_at_correct_path 2>&1 | tail -10`
Expected: FAIL.

- [ ] **Step 3: Write the template**

Create `/Users/xiaojin/workspace/run-bob/src/templates/root/CleanArchitectureTest.java` per spec §7. The full file is shown in spec §7 (from the `package architecture;` line through the closing `}` of the class). Copy it verbatim.

Critical rules to verify present:
1. `@AnalyzeClasses(packages = "com.example", importOptions = DoNotIncludeTests.class)` — base package, NOT `com.example.<bizname>` (covers shared + business)
2. `private static final String[] FORBIDDEN_IN_INNER = { "org.springframework..", "jakarta..", "org.slf4j..", "lombok.." };` plus commented信创 entries (达梦 / 人大金仓 / 东方通) and a "项目自加" placeholder
3. `layered_dependencies` rule with 4 layers `entity / usecase / adapter / framework`
4. `entity_pure_of_frameworks` (using `..entity..` package selector + `FORBIDDEN_IN_INNER` array)
5. `usecase_pure_of_frameworks` (same)
6. `transactional_methods_only_in_decorator` and `transactional_classes_only_in_decorator` referencing `com.example.shared.framework.transaction.TransactionalUseCaseDecorator`
7. `web_controller_no_entity` (using `..adapter.web..` and `..entity..`)
8. `repository_impl_location`
9. `no_event_listener_unless_decided` (R10-bob)
10. JavaDoc class header explains:
    - "由 run-bob init 生成"
    - 维护规则(不要删,可以追加)
    - "引入新外部库时,跑 ARCHITECTURE.md §"5 问决策树","若判为配件,把根包加到 FORBIDDEN_IN_INNER"
    - "B2 模式(清洁孤岛)修改 @AnalyzeClasses 的 packages,必须把 shared 加进数组"
    - "调整 base 包:本模板假设 base 是 com.example,含两个直接子包:com.example.shared + com.example.<bizname>"

- [ ] **Step 4: Wire through init.rs**

Add to `init.rs`:
```rust
const ROOT_ARCHUNIT_TEST: &str = include_str!("../templates/root/CleanArchitectureTest.java");
```

Add a helper function for ArchUnit install (mirrors ddd-run pattern):

```rust
fn install_archunit_test(target: &Path, force: bool) -> Result<()> {
    let path = target
        .join("src").join("test").join("java")
        .join("architecture").join("CleanArchitectureTest.java");
    write_file(
        &path,
        ROOT_ARCHUNIT_TEST,
        force,
        "src/test/java/architecture/CleanArchitectureTest.java",
    )
}
```

Inside `run()`'s `if !minimal {` block, add (after README-RUN-BOB.md install):

```rust
install_archunit_test(&target, force)?;
```

- [ ] **Step 5: Run integration test**

Run: `cargo test init_installs_archunit_test_at_correct_path 2>&1 | tail -10`
Expected: PASS.

- [ ] **Step 6: Wire status check**

Edit `status.rs`. Add a new section heading + check after the "Harness documents" section:

```rust
println!();
println!("{}", "ArchUnit guards".bold());
all_ok &= check(&target, "src/test/java/architecture/CleanArchitectureTest.java");
```

- [ ] **Step 7: Run all tests**

Run: `cargo test 2>&1 | tail -10`
Expected: 6 tests pass.

- [ ] **Step 8: Commit**

```bash
cd /Users/xiaojin/workspace/run-bob
git add src/templates/root/CleanArchitectureTest.java src/commands/init.rs src/commands/status.rs tests/integration.rs
git commit -m "feat(templates): add ArchUnit test for Bob 4-ring with parameterized blacklist"
```

---

## Task 8: shared/ Java skeletons (UseCase + Decorator) (TDD)

**Files:**
- Create: `src/templates/root/UseCase.java`
- Create: `src/templates/root/TransactionalUseCaseDecorator.java`
- Modify: `src/commands/init.rs`, `src/commands/status.rs`, `tests/integration.rs`

- [ ] **Step 1: Write failing tests**

Append to `/Users/xiaojin/workspace/run-bob/tests/integration.rs`:

```rust
#[test]
fn init_installs_shared_usecase_interface() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let target = tmp.path();

    Command::new(run_bob_bin()).args(["init", "--dir"]).arg(target).status().expect("init");

    let p = target
        .join("src").join("main").join("java")
        .join("com").join("example").join("shared")
        .join("usecase").join("UseCase.java");
    assert!(p.is_file(), "expected UseCase.java at {}", p.display());

    let content = std::fs::read_to_string(&p).unwrap();
    assert!(
        content.contains("package com.example.shared.usecase;"),
        "must declare correct package"
    );
    assert!(
        content.contains("public interface UseCase<C, R>"),
        "must declare generic UseCase<C, R> interface"
    );
    assert!(content.contains("R execute(C cmd)"), "must declare execute method");
    // Must NOT contain Spring or any framework import
    assert!(!content.contains("org.springframework"), "UseCase.java must not import Spring");
    assert!(!content.contains("@Transactional"), "UseCase.java must not have @Transactional");
}

#[test]
fn init_installs_transactional_decorator() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let target = tmp.path();

    Command::new(run_bob_bin()).args(["init", "--dir"]).arg(target).status().expect("init");

    let p = target
        .join("src").join("main").join("java")
        .join("com").join("example").join("shared")
        .join("framework").join("transaction").join("TransactionalUseCaseDecorator.java");
    assert!(p.is_file(), "expected decorator at {}", p.display());

    let content = std::fs::read_to_string(&p).unwrap();
    assert!(
        content.contains("package com.example.shared.framework.transaction;"),
        "must declare correct package"
    );
    assert!(content.contains("@Transactional"), "decorator must have @Transactional");
    assert!(
        content.contains("implements UseCase<C, R>"),
        "decorator must implement UseCase<C, R>"
    );
    assert!(
        content.contains("import org.springframework.transaction.annotation.Transactional;"),
        "decorator must import Spring's @Transactional"
    );
}
```

- [ ] **Step 2: Run, expect fail**

Run: `cargo test init_installs_shared 2>&1 | tail -15`
Expected: 2 FAIL.

- [ ] **Step 3: Write `UseCase.java`**

Create `/Users/xiaojin/workspace/run-bob/src/templates/root/UseCase.java` per spec §8.1:

```java
package com.example.shared.usecase;

/**
 * 通用 UseCase 接口。所有 usecase 类必须 implements 此接口。
 *
 * 不允许 import 任何框架代码(Spring / Jakarta / SLF4J / Lombok)。
 *
 * @param <C> Command 类型(usecase 包内的 record)
 * @param <R> Result 类型(usecase 包内的 record)
 */
public interface UseCase<C, R> {
    R execute(C cmd);
}
```

- [ ] **Step 4: Write `TransactionalUseCaseDecorator.java`**

Create `/Users/xiaojin/workspace/run-bob/src/templates/root/TransactionalUseCaseDecorator.java` per spec §8.2:

```java
package com.example.shared.framework.transaction;

import com.example.shared.usecase.UseCase;
import org.springframework.transaction.annotation.Transactional;

/**
 * 全工程唯一的 @Transactional 所在地。
 *
 * 用法:在 framework/config/<Feature>UseCaseConfig.java:
 *
 *   @Bean
 *   UseCase<MyCommand, MyResult> myUseCase(MyRepository repo, ...) {
 *       return new TransactionalUseCaseDecorator<>(
 *           new MyUseCase(repo, ...));
 *   }
 *
 * 命令、查询统一走装饰器,无例外。
 */
public class TransactionalUseCaseDecorator<C, R> implements UseCase<C, R> {

    private final UseCase<C, R> inner;

    public TransactionalUseCaseDecorator(UseCase<C, R> inner) {
        this.inner = inner;
    }

    @Override
    @Transactional
    public R execute(C cmd) {
        return inner.execute(cmd);
    }
}
```

- [ ] **Step 5: Wire init.rs**

Add to `init.rs`:
```rust
const SHARED_USECASE: &str = include_str!("../templates/root/UseCase.java");
const SHARED_DECORATOR: &str = include_str!("../templates/root/TransactionalUseCaseDecorator.java");
```

Add helper functions:

```rust
fn install_shared_usecase(target: &Path, force: bool) -> Result<()> {
    install_java_file(
        target,
        &["src", "main", "java", "com", "example", "shared", "usecase", "UseCase.java"],
        SHARED_USECASE,
        force,
    )
}

fn install_shared_decorator(target: &Path, force: bool) -> Result<()> {
    install_java_file(
        target,
        &[
            "src", "main", "java", "com", "example", "shared",
            "framework", "transaction", "TransactionalUseCaseDecorator.java",
        ],
        SHARED_DECORATOR,
        force,
    )
}
```

Remove `#[allow(dead_code)]` from `install_java_file`.

Inside `run()`'s `if !minimal {` block, add (after `install_archunit_test`):

```rust
println!();
println!("{}", "Installing shared Java skeletons...".bold());
install_shared_usecase(&target, force)?;
install_shared_decorator(&target, force)?;
```

- [ ] **Step 6: Run integration tests**

Run: `cargo test init_installs_shared 2>&1 | tail -10`
Expected: 2 PASS.

- [ ] **Step 7: Wire status checks**

Edit `status.rs`. Add a new section heading + 2 checks after the ArchUnit section:

```rust
println!();
println!("{}", "Shared Java skeletons".bold());
all_ok &= check(&target, "src/main/java/com/example/shared/usecase/UseCase.java");
all_ok &= check(&target, "src/main/java/com/example/shared/framework/transaction/TransactionalUseCaseDecorator.java");
```

- [ ] **Step 8: Run all tests**

Run: `cargo test 2>&1 | tail -10`
Expected: 8 tests pass.

- [ ] **Step 9: Commit**

```bash
cd /Users/xiaojin/workspace/run-bob
git add src/templates/root/UseCase.java src/templates/root/TransactionalUseCaseDecorator.java src/commands/init.rs src/commands/status.rs tests/integration.rs
git commit -m "feat(templates): add shared UseCase interface + TransactionalUseCaseDecorator"
```

---

## Task 9: bob-identify skill (TDD)

**Files:**
- Create: `src/templates/skills/bob-identify.md`
- Modify: `src/commands/init.rs`, `src/commands/status.rs`, `tests/integration.rs`

- [ ] **Step 1: Write failing test**

Append to `/Users/xiaojin/workspace/run-bob/tests/integration.rs`:

```rust
#[test]
fn init_creates_bob_identify_skill() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let target = tmp.path();
    Command::new(run_bob_bin()).args(["init", "--dir"]).arg(target).status().expect("init");

    let p = target.join(".claude").join("skills").join("bob-identify").join("SKILL.md");
    assert!(p.is_file(), "bob-identify SKILL.md missing");
    let content = std::fs::read_to_string(&p).unwrap();

    // YAML frontmatter
    assert!(content.starts_with("---"), "must start with YAML frontmatter");
    assert!(content.contains("name: bob-identify"), "frontmatter name");
    assert!(content.contains("description:"), "frontmatter description");

    // Required sections
    for token in &[
        "5 问决策树",   // 5-question decision tree
        "Q1",
        "Q2",
        "Q3",
        "Q4",
        "Q5",
        "G",            // greenfield mode
        "B1",           // brownfield refactor
        "B2",           // brownfield incremental
        "推测",         // Q&A 三段式 — 推测
        "推荐",         // 推荐
        "清洁孤岛",     // B2 clean island
    ] {
        assert!(content.contains(token), "bob-identify must mention {}", token);
    }
}
```

- [ ] **Step 2: Run, expect fail**

Run: `cargo test init_creates_bob_identify_skill 2>&1 | tail -10`
Expected: FAIL.

- [ ] **Step 3: Write the skill**

Create `/Users/xiaojin/workspace/run-bob/src/templates/skills/bob-identify.md` per spec §3.

YAML frontmatter:
```yaml
---
name: bob-identify
description: |
  触发条件:用户输入 /bob-identify <业务描述>(模式 G:绿地新项目),
  或 /bob-identify --refactor [path](模式 B1:对已有 α/β 代码做全量身份测试),
  或 /bob-identify <新功能描述>(模式 B2:已有 src/main/java + 描述新功能 = auto-detect 棕地增量)。

  对给定的业务描述 / 已有代码 / 新功能,跑一遍 5 问决策树
  (Q1 业务意义会变? Q2 有副作用? Q3 翻译者还是编排者?
   Q4 出现在 inner 包? Q5 棕地 legacy 复用?),
  把每一个候选概念 / 类 / import / 注解分类为 CORE / ADAPTER /
  FRAMEWORK / TOOL / 违规,产出一份结构化分析文档作为
  /bob-onion 的输入。

  适用于 Bob 4 环 Clean Architecture 的第一阶段:从模糊业务描述
  / 已有代码 / 新功能描述里提取核心 vs 配件骨架。
  当用户说"做身份测试"、"区分核心和配件"、"这段代码哪些是核心
  哪些是框架"、"这个功能里什么是 Entity"时也应触发此技能。
---
```

Then write the skill body following spec §3 exactly:

1. `# Bob Identity Test Skill`
2. `## 触发` — 3 命令式触发 + 自然语言触发
3. `## 目标` — 跑 5 问决策树,产出分类表;**不画架构、不写代码、不出 spec**
4. `## 提问规约(强制)` — 三段式(推测 + 理由 + 推荐),从 spec §2.1 抄
5. `## 5 问决策树` — 完整决策树文本,从 spec §1.1 抄(包含 Q1-Q5 完整描述)
6. `## 副作用的精确含义` — 4 类(读外部状态 / 改外部状态 / 非确定性 / 依赖容器/框架生命周期)+ 反例(Objects.requireNonNull 等),从 spec §1.2 抄
7. `## 工作流` — 3 个子节(模式 G / 模式 B1 / 模式 B2),每个子节列 Step,从 spec §3.3 抄
8. `## 产出文档模板` — 完整 markdown 模板,从 spec §3.4 抄(包含 7 个 §)
9. `## 反模式` — 5 条 ❌,从 spec §3.5 抄
10. `## 与其他 skill 衔接` — 上游/下游
11. `## 文件落位` — `docs/bob/01-identity-<topic-slug>.md`

Use spec §3 verbatim (lines 226-323) as the canonical source. Ensure all "推测 + 理由 + 推荐" example dialogues are preserved.

- [ ] **Step 4: Wire init.rs**

Add to `init.rs`:
```rust
const SKILL_BOB_IDENTIFY: &str = include_str!("../templates/skills/bob-identify.md");
```

In `run()`, **before** the `if !minimal {` block, add:

```rust
println!("{}", "Installing skills...".bold());
install_skill(&target, "bob-identify", SKILL_BOB_IDENTIFY, force)?;
```

Remove `#[allow(dead_code)]` from `install_skill` and `ensure_dir`.

- [ ] **Step 5: Run integration test**

Run: `cargo test init_creates_bob_identify_skill 2>&1 | tail -10`
Expected: PASS.

- [ ] **Step 6: Wire status check**

Edit `status.rs`. Add a "Skills" section heading **before** the "Harness documents" section:

```rust
println!("{}", "Skills".bold());
all_ok &= check(&target, ".claude/skills/bob-identify/SKILL.md");

println!();
println!("{}", "Harness documents".bold());
// ... existing checks
```

- [ ] **Step 7: Run all tests**

Run: `cargo test 2>&1 | tail -10`
Expected: 9 tests pass.

- [ ] **Step 8: Commit**

```bash
cd /Users/xiaojin/workspace/run-bob
git add src/templates/skills/bob-identify.md src/commands/init.rs src/commands/status.rs tests/integration.rs
git commit -m "feat(skills): add /bob-identify skill (5-question decision tree, G/B1/B2 modes)"
```

---

## Task 10: bob-onion skill (TDD)

**Files:**
- Create: `src/templates/skills/bob-onion.md`
- Modify: `src/commands/init.rs`, `src/commands/status.rs`, `tests/integration.rs`

- [ ] **Step 1: Write failing test**

Append to `/Users/xiaojin/workspace/run-bob/tests/integration.rs`:

```rust
#[test]
fn init_creates_bob_onion_skill() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let target = tmp.path();
    Command::new(run_bob_bin()).args(["init", "--dir"]).arg(target).status().expect("init");

    let p = target.join(".claude").join("skills").join("bob-onion").join("SKILL.md");
    assert!(p.is_file(), "bob-onion SKILL.md missing");
    let content = std::fs::read_to_string(&p).unwrap();

    assert!(content.starts_with("---"));
    assert!(content.contains("name: bob-onion"));

    for token in &[
        "ARCHITECTURE.md",
        "4 环",
        "端口清单",
        "状态机",
        "TransactionalUseCaseDecorator",
        "FORBIDDEN_IN_INNER",   // wired to ArchUnit blacklist回写
        "ADR",
        "推测",                 // 三段式
        "G",
        "B1",
        "B2",
    ] {
        assert!(content.contains(token), "bob-onion must mention {}", token);
    }
}
```

- [ ] **Step 2: Run, expect fail**

Run: `cargo test init_creates_bob_onion_skill 2>&1 | tail -10`
Expected: FAIL.

- [ ] **Step 3: Write the skill**

Create `/Users/xiaojin/workspace/run-bob/src/templates/skills/bob-onion.md` per spec §4.

YAML frontmatter:
```yaml
---
name: bob-onion
description: |
  触发条件:用户输入 /bob-onion(默认读最新 docs/bob/01-identity-*.md),
  或 /bob-onion --identity <path> 指定 identity 文档,
  或 /bob-onion --refresh 跳过 identity 直接基于现有 ARCHITECTURE.md 增补。

  基于 /bob-identify 的输出,产出正式的 Bob 4 环架构设计:划出 4 环包结构、
  列端口清单、提取 Entity 状态机、决定装饰器边界、回写 ArchUnit 黑名单,
  并自动更新项目根目录的 ARCHITECTURE.md(4 环架构 SSoT)。
  棕地模式额外产出 α→γ 重构计划(B1)或清洁孤岛布局 + Legacy ACL(B2)。
  不写实现代码,只做架构设计。产出会被 /bob-spec 引用以生成 Superpowers spec。

  当用户说"画 4 环架构"、"设计端口"、"出重构计划"、"画洋葱图"、
  "决定状态机怎么放"时也应触发此技能。
---
```

Then write the skill body following spec §4 exactly:

1. `# Bob 4-Ring Architecture Design Skill`
2. `## 触发` — 3 命令式 + 自然语言
3. `## 前置条件` — 至少一份 identity 文档(`--refresh` 例外)
4. `## 目标` — 7 件事(spec §4.2 列表)
5. `## 提问规约(强制)` — 三段式
6. `## 工作流` — Step O1 到 O9,每步对话式提问示例(spec §4.3)
7. `## ARCHITECTURE.md 模板` — **完整模板**,从 spec §4.4 抄(11 节)
8. `## 反模式` — 5 条 ❌(spec §4.5)
9. `## 与其他 skill 衔接`
10. `## 文件落位` — `docs/bob/02-onion-<topic>.md` + 更新 `ARCHITECTURE.md` + 追加 `CleanArchitectureTest.java` 黑名单

Use spec §4 verbatim (lines 324-450) as canonical source.

- [ ] **Step 4: Wire init.rs**

Add to `init.rs`:
```rust
const SKILL_BOB_ONION: &str = include_str!("../templates/skills/bob-onion.md");
```

In `run()`, after the `bob-identify` install:
```rust
install_skill(&target, "bob-onion", SKILL_BOB_ONION, force)?;
```

- [ ] **Step 5: Run integration test**

Run: `cargo test init_creates_bob_onion_skill 2>&1 | tail -10`
Expected: PASS.

- [ ] **Step 6: Wire status check**

Edit `status.rs`. Add to the "Skills" section:
```rust
all_ok &= check(&target, ".claude/skills/bob-onion/SKILL.md");
```

- [ ] **Step 7: Run all tests**

Run: `cargo test 2>&1 | tail -10`
Expected: 10 tests pass.

- [ ] **Step 8: Commit**

```bash
cd /Users/xiaojin/workspace/run-bob
git add src/templates/skills/bob-onion.md src/commands/init.rs src/commands/status.rs tests/integration.rs
git commit -m "feat(skills): add /bob-onion skill (4-ring design + ARCHITECTURE.md SSoT)"
```

---

## Task 11: bob-spec skill (TDD)

**Files:**
- Create: `src/templates/skills/bob-spec.md`
- Modify: `src/commands/init.rs`, `src/commands/status.rs`, `tests/integration.rs`

- [ ] **Step 1: Write failing test**

Append to `/Users/xiaojin/workspace/run-bob/tests/integration.rs`:

```rust
#[test]
fn init_creates_bob_spec_skill() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let target = tmp.path();
    Command::new(run_bob_bin()).args(["init", "--dir"]).arg(target).status().expect("init");

    let p = target.join(".claude").join("skills").join("bob-spec").join("SKILL.md");
    assert!(p.is_file(), "bob-spec SKILL.md missing");
    let content = std::fs::read_to_string(&p).unwrap();

    assert!(content.starts_with("---"));
    assert!(content.contains("name: bob-spec"));

    for token in &[
        "Given-When-Then",
        "ARCHITECTURE.md",
        "TransactionalUseCaseDecorator",
        "Superpowers",
        "命令型",        // command template A
        "查询型",        // query template B
        "重构型",        // refactor template C
        "交给 Superpowers 的开放问题",
        "技术栈",
        "5 问决策树",   // R0 reference
        "推测",
    ] {
        assert!(content.contains(token), "bob-spec must mention {}", token);
    }
}
```

- [ ] **Step 2: Run, expect fail**

Run: `cargo test init_creates_bob_spec_skill 2>&1 | tail -10`
Expected: FAIL.

- [ ] **Step 3: Write the skill**

Create `/Users/xiaojin/workspace/run-bob/src/templates/skills/bob-spec.md` per spec §5.

YAML frontmatter:
```yaml
---
name: bob-spec
description: |
  触发条件:用户输入 /bob-spec <用例名>(默认命令型),
  或 /bob-spec --query <查询名>(查询型读模型),
  或 /bob-spec --refactor <类名>(B1 重构型 spec)。

  读取项目根目录的 ARCHITECTURE.md,为指定用例生成一份 Superpowers
  可直接消化的 spec 文档:严格使用 ARCHITECTURE.md §3 §4 §5 中的术语
  (Entity / 端口 / UseCase),包含用例描述、前置/后置条件、业务规则、
  Given-When-Then 测试场景、纯 POJO usecase + framework Config 接口约定、
  Guardrails(给 Superpowers 实现时遵守)、和"交给 Superpowers 的开放问题"
  (技术栈决策)。

  这个 skill 是 Bob 4 环建模阶段与 Superpowers 实现阶段的桥梁。
  当用户说"生成 spec"、"出 TDD 测试场景"、"准备给 Superpowers 的输入"、
  "把这个用例写清楚"时也应触发此技能。
---
```

Then write the body following spec §5:

1. `# Bob 4-Ring → Superpowers Spec Bridge Skill`
2. `## 触发` — 3 个命令式 + 自然语言
3. `## 前置条件` — `ARCHITECTURE.md` §4 §5 已填
4. `## 适用范围:命令 / 查询 / 重构 三类` — 表格
5. `## 提问规约(强制)` — 三段式
6. `## 目标` — 3 个硬约束(术语锚定 / 测试友好 / 4 环纪律)
7. `## 工作流` — Step S1-S4
8. `## 模板 A:命令型 spec` — **完整模板**(spec §5.5,包含 11 节,带 Java 代码示例)
9. `## 模板 B:查询型 spec(简化)` — spec §5.6
10. `## 模板 C:重构型 spec(B1 专用)` — spec §5.7
11. `## 反模式` — 6 条 ❌(spec §5.8)
12. `## 与其他 skill 衔接`
13. `## 文件落位` — `docs/specs/spec-<n>-<slug>.md`

Use spec §5 verbatim (lines 451-704). Especially preserve all the Java code examples in 模板 A §8 (Command record / Result record / UseCase POJO / Entity method / framework Config / Controller).

- [ ] **Step 4: Wire init.rs**

Add to `init.rs`:
```rust
const SKILL_BOB_SPEC: &str = include_str!("../templates/skills/bob-spec.md");
```

In `run()`, after the `bob-onion` install:
```rust
install_skill(&target, "bob-spec", SKILL_BOB_SPEC, force)?;
```

- [ ] **Step 5: Run integration test**

Run: `cargo test init_creates_bob_spec_skill 2>&1 | tail -10`
Expected: PASS.

- [ ] **Step 6: Wire status check**

Edit `status.rs`. Add to the "Skills" section:
```rust
all_ok &= check(&target, ".claude/skills/bob-spec/SKILL.md");
```

- [ ] **Step 7: Run all tests**

Run: `cargo test 2>&1 | tail -10`
Expected: 11 tests pass.

- [ ] **Step 8: Commit**

```bash
cd /Users/xiaojin/workspace/run-bob
git add src/templates/skills/bob-spec.md src/commands/init.rs src/commands/status.rs tests/integration.rs
git commit -m "feat(skills): add /bob-spec skill (command/query/refactor templates → Superpowers)"
```

---

## Task 12: --minimal mode + working directories + minimal-mode tests

**Files:**
- Modify: `src/commands/init.rs`
- Modify: `src/commands/status.rs`
- Modify: `tests/integration.rs`

- [ ] **Step 1: Write failing tests for working directories + --minimal behavior**

Append to `/Users/xiaojin/workspace/run-bob/tests/integration.rs`:

```rust
#[test]
fn init_creates_working_directories() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let target = tmp.path();
    Command::new(run_bob_bin()).args(["init", "--dir"]).arg(target).status().expect("init");

    assert!(target.join("docs").join("bob").is_dir(), "docs/bob/ missing");
    assert!(target.join("docs").join("specs").is_dir(), "docs/specs/ missing");
}

#[test]
fn init_minimal_skips_archunit_and_shared_and_anchors() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let target = tmp.path();

    Command::new(run_bob_bin())
        .args(["init", "--minimal", "--dir"])
        .arg(target)
        .status()
        .expect("init --minimal");

    // Skills must still be installed
    for skill in &["bob-identify", "bob-onion", "bob-spec"] {
        let p = target.join(".claude").join("skills").join(skill).join("SKILL.md");
        assert!(p.is_file(), "minimal must still install skill {}", skill);
    }

    // Anchor docs must NOT be installed
    for f in &["CLAUDE.md", "ARCHITECTURE.md", "README-RUN-BOB.md"] {
        assert!(
            !target.join(f).exists(),
            "minimal must not install {}",
            f
        );
    }

    // ArchUnit must NOT be installed
    assert!(
        !target.join("src").join("test").join("java").join("architecture")
            .join("CleanArchitectureTest.java").exists(),
        "minimal must not install ArchUnit"
    );

    // Shared骨架 must NOT be installed
    assert!(
        !target.join("src").join("main").join("java").join("com").join("example")
            .join("shared").join("usecase").join("UseCase.java").exists(),
        "minimal must not install UseCase.java"
    );
    assert!(
        !target.join("src").join("main").join("java").join("com").join("example")
            .join("shared").join("framework").join("transaction")
            .join("TransactionalUseCaseDecorator.java").exists(),
        "minimal must not install TransactionalUseCaseDecorator.java"
    );

    // Working directories must NOT be created in --minimal
    assert!(!target.join("docs").join("bob").exists(), "minimal must not create docs/bob/");
    assert!(!target.join("docs").join("specs").exists(), "minimal must not create docs/specs/");
}

#[test]
fn status_reports_complete_after_full_init() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let target = tmp.path();
    Command::new(run_bob_bin()).args(["init", "--dir"]).arg(target).status().expect("init");

    let output = Command::new(run_bob_bin())
        .args(["status", "--dir"])
        .arg(target)
        .output()
        .expect("status");
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert!(stdout.contains("harness is complete"), "got:\n{}", stdout);
    // Mention all 9 file checks
    for token in &[
        "bob-identify",
        "bob-onion",
        "bob-spec",
        "CLAUDE.md",
        "ARCHITECTURE.md",
        "README-RUN-BOB.md",
        "CleanArchitectureTest.java",
        "UseCase.java",
        "TransactionalUseCaseDecorator.java",
        "docs/bob",
        "docs/specs",
    ] {
        assert!(stdout.contains(token), "status must list {}; got:\n{}", token, stdout);
    }
}

#[test]
fn status_flags_missing_after_minimal_init() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let target = tmp.path();
    Command::new(run_bob_bin()).args(["init", "--minimal", "--dir"]).arg(target).status().expect("init");

    let output = Command::new(run_bob_bin())
        .args(["status", "--dir"])
        .arg(target)
        .output()
        .expect("status");
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("some assets are missing"), "got:\n{}", stdout);
}
```

- [ ] **Step 2: Run, expect 4 failures**

Run: `cargo test 2>&1 | tail -15`
Expected: existing tests pass; 4 new tests fail because:
- `docs/bob/` and `docs/specs/` are not yet created in init
- `--minimal` may currently still install root files (we want to confirm)
- status checklist is missing the dir checks

- [ ] **Step 3: Add working directories creation to init.rs**

Edit `init.rs`. Inside `run()`'s `if !minimal {` block, **after** the shared install, add:

```rust
println!();
println!("{}", "Creating working directories...".bold());
ensure_dir(&target.join("docs").join("bob"))?;
ensure_dir(&target.join("docs").join("specs"))?;
crate::success("docs/bob/   (identify & onion intermediate notes)");
crate::success("docs/specs/ (bob-spec outputs → Superpowers inputs)");
```

- [ ] **Step 4: Verify --minimal already skips anchor/archunit/shared/dirs**

The `if !minimal { ... }` block guards CLAUDE.md / ARCHITECTURE.md / README-RUN-BOB.md / ArchUnit / shared骨架 / working directories. The skill installs are **outside** that block. Verify by re-reading `init.rs` — the structure should be:

```rust
// Skills always install (no minimal guard)
println!("{}", "Installing skills...".bold());
install_skill(&target, "bob-identify", SKILL_BOB_IDENTIFY, force)?;
install_skill(&target, "bob-onion",    SKILL_BOB_ONION, force)?;
install_skill(&target, "bob-spec",     SKILL_BOB_SPEC, force)?;

// Everything else only in non-minimal mode
if !minimal {
    println!();
    println!("{}", "Installing harness documents...".bold());
    install_root_file(&target, "CLAUDE.md", ROOT_CLAUDE_MD, force)?;
    install_root_file(&target, "ARCHITECTURE.md", ROOT_ARCHITECTURE, force)?;
    install_root_file(&target, "README-RUN-BOB.md", ROOT_README, force)?;
    install_archunit_test(&target, force)?;

    println!();
    println!("{}", "Installing shared Java skeletons...".bold());
    install_shared_usecase(&target, force)?;
    install_shared_decorator(&target, force)?;

    println!();
    println!("{}", "Creating working directories...".bold());
    ensure_dir(&target.join("docs").join("bob"))?;
    ensure_dir(&target.join("docs").join("specs"))?;
    crate::success("docs/bob/   (identify & onion intermediate notes)");
    crate::success("docs/specs/ (bob-spec outputs → Superpowers inputs)");
}
```

- [ ] **Step 5: Add working directory checks to status.rs**

Edit `status.rs`. After the "Shared Java skeletons" section, append:

```rust
println!();
println!("{}", "Working directories".bold());
all_ok &= check_dir(&target, "docs/bob");
all_ok &= check_dir(&target, "docs/specs");
```

Remove `#[allow(dead_code)]` from `check_dir`.

- [ ] **Step 6: Run all tests**

Run: `cd /Users/xiaojin/workspace/run-bob && cargo test 2>&1 | tail -10`
Expected: 15 tests pass.

- [ ] **Step 7: Commit**

```bash
cd /Users/xiaojin/workspace/run-bob
git add src/commands/init.rs src/commands/status.rs tests/integration.rs
git commit -m "feat(init): create docs/bob and docs/specs working directories; verify --minimal scope"
```

---

## Task 13: End-to-end verification + final push

**Files:** None — this task is end-to-end manual + final integration smoke.

- [ ] **Step 1: Build release**

Run: `cd /Users/xiaojin/workspace/run-bob && cargo build --release 2>&1 | tail -5`
Expected: success.

- [ ] **Step 2: Run all integration tests once more**

Run: `cargo test 2>&1 | tail -10`
Expected: 15 tests pass, 0 failures.

- [ ] **Step 3: cargo install end-to-end**

Run: `cargo install --path . 2>&1 | tail -5`
Expected: `Installed package run-bob v0.1.0` to `~/.cargo/bin/run-bob`.

- [ ] **Step 4: Manual smoke test in a real tempdir**

```bash
TEMP_PROJECT=$(mktemp -d)
cd "$TEMP_PROJECT"
~/.cargo/bin/run-bob init
~/.cargo/bin/run-bob status
```

Expected: 
- init prints all 9 install lines + 2 directory creates + next-steps block
- status reports "harness is complete" with all checks green
- All 9 expected files present, all 2 directories present

Verify file count:

```bash
find "$TEMP_PROJECT" -type f -not -path '*/\.*' | wc -l
```
Expected: ≥ 9 files.

- [ ] **Step 5: Verify --minimal mode**

```bash
TEMP_MINIMAL=$(mktemp -d)
cd "$TEMP_MINIMAL"
~/.cargo/bin/run-bob init --minimal
~/.cargo/bin/run-bob status
```

Expected:
- init only installs 3 skills, no anchor/archunit/shared/dirs
- status reports "some assets are missing"

- [ ] **Step 6: Verify --force**

```bash
cd "$TEMP_PROJECT"
echo "tampered" > CLAUDE.md
~/.cargo/bin/run-bob init --force
diff CLAUDE.md <(grep -c 'tampered' CLAUDE.md)
```
Expected: CLAUDE.md is overwritten back to template (no "tampered" string).

- [ ] **Step 7: Cleanup test artifacts**

```bash
rm -rf "$TEMP_PROJECT" "$TEMP_MINIMAL"
```

- [ ] **Step 8: Verify cargo install paths and final state**

Run: `which run-bob`
Expected: `/Users/xiaojin/.cargo/bin/run-bob` (or your `$CARGO_HOME/bin`).

Run: `run-bob --version`
Expected: `run-bob 0.1.0`.

- [ ] **Step 9: Update root README.md status badge**

Edit `/Users/xiaojin/workspace/run-bob/README.md`. Change the line:

```markdown
🚧 **In design phase.** The full design spec is at:
```

to:

```markdown
✅ **Implementation complete (v0.1.0).** Build with `cargo install --path .` (or via `/install` skill in this repo). The full design spec is at:
```

- [ ] **Step 10: Final commit**

```bash
cd /Users/xiaojin/workspace/run-bob
git add README.md
git commit -m "docs(README): mark v0.1.0 implementation complete"
```

- [ ] **Step 11: Push to GitHub**

Run: `git push origin master 2>&1`
Expected: all commits since the initial push are now on `origin/master`.

- [ ] **Step 12: Verify on GitHub**

Run: `git log --oneline | head -16`
Expected: ~13 commits (1 per task) on top of the initial spec commit.

---

## Self-Review Notes

This plan was checked against the spec on 2026-05-08. Coverage map:

| Spec section | Implemented in task |
|---|---|
| §0 Overview / 3 modes | Tasks 5 (CLAUDE.md), 6 (README-RUN-BOB.md), 9-11 (skills declare modes) |
| §1 5-question decision tree | Task 9 (bob-identify embeds it) |
| §1.3 配件清单 | Tasks 4 (ARCHITECTURE.md §6), 7 (FORBIDDEN_IN_INNER) |
| §2 三段式提问规约 | Tasks 9, 10, 11 (each skill embeds it) |
| §3 /bob-identify | Task 9 |
| §4 /bob-onion + ARCHITECTURE.md template | Tasks 10 (skill), 4 (template) |
| §5 /bob-spec + 3 sub-templates | Task 11 |
| §6 CLAUDE.md R0-R12 | Task 5 |
| §7 ArchUnit parameterized | Task 7 |
| §8 shared/ skeletons | Task 8 |
| §9 Rust CLI | Tasks 1, 2 (foundation) + tasks 4-12 (incremental wiring) |
| §10 README-RUN-BOB.md | Task 6 |
| §11 File inventory (19 files) | All 13 tasks combined |
| §12 Acceptance criteria A1-G2 | Task 13 (end-to-end) + Tasks 4-12 (per-template assertions) |

Critical bugs caught during spec self-review (now baked into this plan):
- Task 7 test asserts `packages = "com.example"` (not `com.example.<bizname>`) so ArchUnit reaches the shared decorator package
- Task 5 test asserts `entity` (not `domain`) is the Ring 1 keyword
- Task 12 test asserts `--minimal` skips anchor docs, ArchUnit, shared, AND working directories

No placeholders. All file paths are absolute. All commands have expected output. Type/method names are consistent across tasks (`install_skill`, `install_root_file`, `install_java_file`, `install_archunit_test`, `install_shared_usecase`, `install_shared_decorator`, `check`, `check_dir`).
