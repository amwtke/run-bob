# run-bob Auto-`.gitignore` on init/upgrade Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make `run-bob init` and `run-bob upgrade` automatically maintain a `# run-bob` block inside the target directory's `.gitignore` so transient artifacts (currently just `.run-bob-backup/`) never get committed. Add a `--no-gitignore` opt-out to both commands.

**Architecture:** New module `src/commands/gitignore.rs` exposes a pure `compute_update` (testable in isolation) and a thin `apply` that does the file I/O. Both `init.rs` and `upgrade.rs` call `gitignore::apply(&target, no_gitignore)` after all asset writes complete and before the closing summary. Idempotency is keyed off the literal marker line `# run-bob` — the function locates the block, diffs its entries against `GITIGNORE_ENTRIES`, and only writes when something is actually missing.

**Tech Stack:** Rust 1.75+, `clap = "=4.5.4"`, `anyhow = "1.0"`, `colored = "2.1"`, dev-dep `tempfile = "3"`. **No new crates.**

**Spec:** [`docs/superpowers/specs/2026-05-14-gitignore-management-design.md`](../specs/2026-05-14-gitignore-management-design.md)

---

## File Structure

| File | Action | Responsibility |
|---|---|---|
| `src/commands/gitignore.rs` | **Create** | Pure `compute_update` + I/O `apply` + `print_report`; owns `GITIGNORE_BLOCK_HEADER` and `GITIGNORE_ENTRIES` constants |
| `src/commands/mod.rs` | **Modify** | Add `pub mod gitignore;` |
| `src/commands/init.rs` | **Modify** | Add `no_gitignore: bool` parameter to `run`; call `gitignore::apply` + `print_report` after asset install, before `print_next_steps` |
| `src/commands/upgrade.rs` | **Modify** | Add `no_gitignore: bool` parameter to `run`; call `gitignore::apply` + `print_report` before final summary line in every branch (no-op short circuit, dry-run short circuit, normal apply path) |
| `src/main.rs` | **Modify** | Add `--no-gitignore` flag to both `Init` and `Upgrade` clap variants; thread it into the `run(...)` calls |
| `tests/integration.rs` | **Modify** | Append integration tests: init creates `.gitignore`, init idempotent, upgrade creates `.gitignore`, `--no-gitignore` skip, respects user extras |

**Untouched** (the plan must NOT modify these):
- `src/assets.rs` — no new asset; gitignore is not an asset, it's user-owned with run-bob-managed insertion
- All template files under `src/templates/`
- `src/commands/status.rs` — `status` doesn't touch `.gitignore`
- `Cargo.toml`
- `README.md` — out of scope for this plan; can be added later

---

## Architectural Notes for the Engineer

Read these before starting — they explain decisions not obvious from individual tasks.

### 1. Why `compute_update` is a pure function

Algorithm correctness across 4 cases (create / append-block / add-missing-entries / no-op) plus edge cases (header variants, user-added entries) is best validated with fast unit tests, not by spinning tempdirs and running the binary every time. Keep `compute_update` taking `(current: Option<&str>, header: &str, entries: &[&str])` so unit tests can drive it directly. `apply` is then a thin wrapper that reads the file, calls `compute_update`, and writes if needed.

### 2. Why `lines.split('\n')` and not `lines()`

`str::lines()` strips trailing newlines, which makes round-tripping lossy: a file ending in `\n` and one not ending in `\n` collapse to the same `Vec<&str>`. Use `str::split('\n')` everywhere — it gives a trailing empty string when the input ends with `\n`, so `split('\n').collect::<Vec<_>>().join("\n")` is byte-identical to the input.

### 3. Header matching is strict

Per spec §2.1: `# run-bob` (trim-equals) only. Variants like `#run-bob`, `## run-bob`, `# RUN-BOB` do **not** match — they get treated as "no block exists" and a fresh block is appended at the end. Don't try to be clever about normalizing variants; users who hand-crafted a near-miss are better served by getting a properly-formatted block than by us silently adopting their typo.

### 4. The block ends at the first blank line OR foreign comment header OR EOF

Inside the block, entries are non-empty, non-`#`-prefixed lines. A line starting with `#` that isn't `# run-bob` itself terminates the block (it's someone else's section). A blank line also terminates. EOF terminates. This is what makes Case C (insert missing entries inside an existing block) unambiguous about where to insert.

### 5. Where to insert missing entries in Case C

Insert at the **end of the block** (just before whatever terminator the block has — blank line, foreign comment, or EOF). This preserves any ordering the user established for the existing entries and any of their own additions inside the block.

### 6. Don't touch user-added entries inside the block

If the user added `my-local-cache/` inside the `# run-bob` block, leave it alone. We only **add** missing run-bob entries; we never **remove** anything from the block. This is critical for trust — users should be able to add their own lines into our section without fear of them being scrubbed on the next `upgrade`.

### 7. Call site ordering in `upgrade.rs`

`upgrade.rs` has three exit paths: (a) zero-change short circuit, (b) dry-run short circuit, (c) full apply. The gitignore step must run in (a) and (c) **with file I/O**, in (b) **without file I/O** (it's a dry-run — we still report what *would* happen, but don't actually write the file). Threading `dry_run` into `gitignore::apply` keeps this clean. Actually, simpler: in dry-run path, just call a `gitignore::dry_run_report(target)` that reads but never writes, or skip entirely and print a note. **Decision: in dry-run path, skip gitignore work entirely and print a one-line note "→ skipped: --dry-run".** This matches the existing semantics that dry-run writes nothing.

---

## Task 1: Create gitignore module skeleton

**Files:**
- Create: `src/commands/gitignore.rs`
- Modify: `src/commands/mod.rs`

**Goal:** Lay down the module, types, and constants. No real logic yet — `apply` returns `Skipped` unconditionally. Verify it compiles and links into the binary.

- [ ] **Step 1: Create `src/commands/gitignore.rs` with types + stub**

Write this exact content:

```rust
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
```

- [ ] **Step 2: Wire into `src/commands/mod.rs`**

Edit `src/commands/mod.rs` from:

```rust
pub mod init;
pub mod status;
pub mod upgrade;
```

to:

```rust
pub mod gitignore;
pub mod init;
pub mod status;
pub mod upgrade;
```

- [ ] **Step 3: Verify it compiles**

Run: `cargo build`
Expected: clean build, no errors.

- [ ] **Step 4: Commit**

```bash
git add src/commands/gitignore.rs src/commands/mod.rs
git commit -m "$(cat <<'EOF'
feat(gitignore): add module skeleton for .gitignore management

Stub apply() returns Skipped; real algorithm lands in subsequent tasks.

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
EOF
)"
```

---

## Task 2: Implement Case A — create `.gitignore` when missing (TDD)

**Files:**
- Modify: `src/commands/gitignore.rs`

**Goal:** When `.gitignore` doesn't exist in the target dir, `apply` creates it containing exactly the run-bob block. Drive the algorithm out via a unit test on the pure `compute_update`.

- [ ] **Step 1: Add unit test in `gitignore.rs` `#[cfg(test)] mod tests` block**

Append at the bottom of `src/commands/gitignore.rs`:

```rust
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
}
```

- [ ] **Step 2: Run the test, expect failure**

Run: `cargo test --lib case_a_creates_when_missing`
Expected: compile error — `compute_update` is not yet defined.

- [ ] **Step 3: Implement `compute_update` for Case A only**

In `src/commands/gitignore.rs`, **add these imports** at the top alongside the existing ones:

```rust
use anyhow::Context;
use std::fs;
```

Then add this function above the `#[cfg(test)]` block:

```rust
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
        Some(_existing) => {
            // Cases B/C/D handled in later tasks.
            unimplemented!("cases B/C/D not yet implemented")
        }
    }
}
```

Also update `apply` to use it:

```rust
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
```

- [ ] **Step 4: Run the test, expect pass**

Run: `cargo test --lib case_a_creates_when_missing`
Expected: 1 passing.

- [ ] **Step 5: Commit**

```bash
git add src/commands/gitignore.rs
git commit -m "$(cat <<'EOF'
feat(gitignore): implement Case A — create .gitignore when missing

Pure compute_update() introduced; apply() wires file I/O around it.

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
EOF
)"
```

---

## Task 3: Implement Case B — append block when no `# run-bob` block exists (TDD)

**Files:**
- Modify: `src/commands/gitignore.rs`

**Goal:** When `.gitignore` exists but contains no `# run-bob` block, append the block at the end. Ensure at least one blank line separates it from prior content.

- [ ] **Step 1: Add unit tests for Case B variants**

In the `tests` module in `src/commands/gitignore.rs`, append:

```rust
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
```

- [ ] **Step 2: Run tests, expect panic on `unimplemented!`**

Run: `cargo test --lib case_b`
Expected: 3 tests panic with `not yet implemented`.

- [ ] **Step 3: Implement Case B in `compute_update`**

In `src/commands/gitignore.rs`, replace the `Some(_existing) => unimplemented!(...)` arm with:

```rust
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
```

- [ ] **Step 4: Run all tests, expect pass**

Run: `cargo test --lib`
Expected: all 4 (1 from Task 2 + 3 from Task 3) passing.

- [ ] **Step 5: Commit**

```bash
git add src/commands/gitignore.rs
git commit -m "$(cat <<'EOF'
feat(gitignore): implement Case B — append block when missing

Ensures exactly one blank line separator between existing content and
the new run-bob block, regardless of how existing content terminates.

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
EOF
)"
```

---

## Task 4: Implement Case C — add missing entries inside existing block (TDD)

**Files:**
- Modify: `src/commands/gitignore.rs`

**Goal:** When the `# run-bob` block exists but is missing some entries, append the missing entries at the end of the block (preserving order and user-added lines).

Note: today `GITIGNORE_ENTRIES` has 1 entry, so Case C is hard to exercise with production entries alone. Drive the test by passing a synthetic 2-entry list directly to `compute_update`. This validates the algorithm for future-proofing.

- [ ] **Step 1: Add unit tests for Case C**

In the `tests` module, append:

```rust
    #[test]
    fn case_c_appends_missing_entry_at_block_end() {
        let existing = "# run-bob\n.run-bob-backup/\n";
        let (new_content, action) = compute_update(
            Some(existing),
            "# run-bob",
            &[".run-bob-backup/", ".run-bob-cache/"], // future 2nd entry
        );
        assert_eq!(
            new_content.as_deref(),
            Some("# run-bob\n.run-bob-backup/\n.run-bob-cache/\n")
        );
        assert_eq!(action, GitignoreReport::Updated { added: 1 });
    }

    #[test]
    fn case_c_block_followed_by_other_content() {
        // Block is followed by a blank line + foreign section. Insertion
        // must happen at the END of the block, before the blank line.
        let existing = "\
# run-bob
.run-bob-backup/

# my own ignores
local-cache/
";
        let (new_content, action) = compute_update(
            Some(existing),
            "# run-bob",
            &[".run-bob-backup/", ".run-bob-cache/"],
        );
        let expected = "\
# run-bob
.run-bob-backup/
.run-bob-cache/

# my own ignores
local-cache/
";
        assert_eq!(new_content.as_deref(), Some(expected));
        assert_eq!(action, GitignoreReport::Updated { added: 1 });
    }

    #[test]
    fn case_c_empty_block_gets_all_entries() {
        // Header is there but no entries underneath.
        let existing = "# run-bob\n";
        let (new_content, action) =
            compute_update(Some(existing), GITIGNORE_BLOCK_HEADER, GITIGNORE_ENTRIES);
        assert_eq!(
            new_content.as_deref(),
            Some("# run-bob\n.run-bob-backup/\n")
        );
        assert_eq!(action, GitignoreReport::Updated { added: 1 });
    }
```

- [ ] **Step 2: Run tests, expect panic on `unimplemented!`**

Run: `cargo test --lib case_c`
Expected: 3 tests panic.

- [ ] **Step 3: Implement Case C (and D as no-op fall-through) in `compute_update`**

Replace the `Some(_start) => unimplemented!(...)` arm with:

```rust
                Some(start) => {
                    // Find block end: first blank line, or foreign comment header,
                    // or EOF (= lines.len()). We scan from start+1.
                    let mut end = lines.len();
                    for (i, line) in lines.iter().enumerate().skip(start + 1) {
                        let trimmed = line.trim();
                        if trimmed.is_empty() {
                            end = i;
                            break;
                        }
                        if trimmed.starts_with('#') && trimmed != header {
                            end = i;
                            break;
                        }
                    }

                    use std::collections::HashSet;
                    let existing_entries: HashSet<&str> = lines[start + 1..end]
                        .iter()
                        .map(|l| l.trim())
                        .filter(|l| !l.is_empty() && !l.starts_with('#'))
                        .collect();

                    let missing: Vec<&&str> = entries
                        .iter()
                        .filter(|e| !existing_entries.contains(*e as &str))
                        .collect();

                    if missing.is_empty() {
                        // Case D: nothing to do.
                        return (None, GitignoreReport::UpToDate);
                    }

                    // Case C: insert missing entries at `end`.
                    let mut new_lines: Vec<String> =
                        lines.iter().map(|s| s.to_string()).collect();
                    for (offset, entry) in missing.iter().enumerate() {
                        new_lines.insert(end + offset, entry.to_string());
                    }
                    let out = new_lines.join("\n");
                    (
                        Some(out),
                        GitignoreReport::Updated { added: missing.len() },
                    )
                }
```

- [ ] **Step 4: Run all tests, expect pass**

Run: `cargo test --lib`
Expected: all 7 tests passing.

- [ ] **Step 5: Commit**

```bash
git add src/commands/gitignore.rs
git commit -m "$(cat <<'EOF'
feat(gitignore): implement Case C — add missing entries inside block

Block ends at first blank line, foreign comment, or EOF. Missing
entries are inserted at the block end, preserving user additions.
Falls through to Case D no-op when all entries are already present.

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
EOF
)"
```

---

## Task 5: Verify Case D idempotency byte-for-byte (TDD)

**Files:**
- Modify: `src/commands/gitignore.rs`

**Goal:** Lock down the no-op invariant: when the block is fully up to date, `compute_update` returns `(None, UpToDate)` and `apply` does not touch the file (timestamp unchanged is a downstream concern; the contract here is "no write").

- [ ] **Step 1: Add unit test for Case D**

In the `tests` module, append:

```rust
    #[test]
    fn case_d_noop_when_block_already_complete() {
        let existing = "# run-bob\n.run-bob-backup/\n";
        let (new_content, action) =
            compute_update(Some(existing), GITIGNORE_BLOCK_HEADER, GITIGNORE_ENTRIES);
        assert_eq!(new_content, None);
        assert_eq!(action, GitignoreReport::UpToDate);
    }

    #[test]
    fn case_d_noop_with_surrounding_content() {
        let existing = "\
target/

# run-bob
.run-bob-backup/

# my section
other/
";
        let (new_content, action) =
            compute_update(Some(existing), GITIGNORE_BLOCK_HEADER, GITIGNORE_ENTRIES);
        assert_eq!(new_content, None);
        assert_eq!(action, GitignoreReport::UpToDate);
    }
```

- [ ] **Step 2: Run tests, expect pass (algorithm should already cover this)**

Run: `cargo test --lib case_d`
Expected: 2 tests passing. (Case D fell out of Case C's "missing is empty" branch in Task 4.)

If they fail, revisit the `existing_entries` collection in Task 4 — make sure the line `.run-bob-backup/` (trimmed) is correctly captured.

- [ ] **Step 3: Commit**

```bash
git add src/commands/gitignore.rs
git commit -m "$(cat <<'EOF'
test(gitignore): lock in Case D no-op idempotency

Two regression tests: bare block, and block surrounded by user sections.

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
EOF
)"
```

---

## Task 6: Header strictness — variants must NOT match (TDD)

**Files:**
- Modify: `src/commands/gitignore.rs`

**Goal:** Per spec §2.1, only the exact line `# run-bob` (trim-equals) counts as the block header. Variants like `#run-bob`, `## run-bob`, `# RUN-BOB` are treated as foreign content, and a fresh block is appended after them.

- [ ] **Step 1: Add unit tests**

In the `tests` module, append:

```rust
    #[test]
    fn header_no_space_variant_does_not_match() {
        let existing = "#run-bob\n.run-bob-backup/\n";
        let (new_content, action) =
            compute_update(Some(existing), GITIGNORE_BLOCK_HEADER, GITIGNORE_ENTRIES);
        // We should append our own block; the user's `#run-bob` is foreign.
        let nc = new_content.expect("must write");
        assert!(nc.contains("#run-bob\n"), "must preserve user line");
        assert!(nc.contains("# run-bob\n"), "must add canonical block header");
        assert_eq!(action, GitignoreReport::Updated { added: 1 });
    }

    #[test]
    fn header_double_hash_variant_does_not_match() {
        let existing = "## run-bob\n.run-bob-backup/\n";
        let (new_content, _) =
            compute_update(Some(existing), GITIGNORE_BLOCK_HEADER, GITIGNORE_ENTRIES);
        let nc = new_content.expect("must write");
        assert!(nc.contains("## run-bob\n"));
        assert!(nc.contains("# run-bob\n"));
    }

    #[test]
    fn header_case_variant_does_not_match() {
        let existing = "# RUN-BOB\n.run-bob-backup/\n";
        let (new_content, _) =
            compute_update(Some(existing), GITIGNORE_BLOCK_HEADER, GITIGNORE_ENTRIES);
        let nc = new_content.expect("must write");
        assert!(nc.contains("# RUN-BOB\n"));
        assert!(nc.contains("# run-bob\n"));
    }
```

- [ ] **Step 2: Run tests, expect pass (the existing `l.trim() == header` check is already strict)**

Run: `cargo test --lib header_`
Expected: 3 tests passing.

If they fail, the matching logic is too loose — revisit and ensure `trim() == header` is the *only* matcher.

- [ ] **Step 3: Commit**

```bash
git add src/commands/gitignore.rs
git commit -m "$(cat <<'EOF'
test(gitignore): lock in strict header matching ("# run-bob" only)

Regression coverage for #run-bob, ## run-bob, # RUN-BOB variants —
none of them match; a canonical block is appended instead.

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
EOF
)"
```

---

## Task 7: Respect user-added entries inside the block (TDD)

**Files:**
- Modify: `src/commands/gitignore.rs`

**Goal:** If the user adds their own entries inside the `# run-bob` block (e.g. `my-local-cache/`), `compute_update` must NOT remove them. When all run-bob entries are present, the result is Case D no-op — even with foreign entries inside the block.

- [ ] **Step 1: Add unit test**

In the `tests` module, append:

```rust
    #[test]
    fn user_added_entries_inside_block_preserved() {
        let existing = "# run-bob\n.run-bob-backup/\nmy-local-cache/\n";
        let (new_content, action) =
            compute_update(Some(existing), GITIGNORE_BLOCK_HEADER, GITIGNORE_ENTRIES);
        assert_eq!(new_content, None, "no write — all run-bob entries already present");
        assert_eq!(action, GitignoreReport::UpToDate);
    }
```

- [ ] **Step 2: Run test, expect pass**

Run: `cargo test --lib user_added_entries`
Expected: 1 passing. The check is implicit: `existing_entries` includes `my-local-cache/`, but since we only diff against `GITIGNORE_ENTRIES` (which doesn't contain it), the user line is left alone.

- [ ] **Step 3: Commit**

```bash
git add src/commands/gitignore.rs
git commit -m "$(cat <<'EOF'
test(gitignore): preserve user-added entries inside the run-bob block

Locks in the trust contract: we only add missing run-bob entries,
never remove or rearrange anything inside the block.

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
EOF
)"
```

---

## Task 8: Wire `gitignore::apply` into `init` + add `--no-gitignore` flag

**Files:**
- Modify: `src/main.rs`
- Modify: `src/commands/init.rs`
- Modify: `tests/integration.rs`

**Goal:** `run-bob init [--no-gitignore]` invokes `gitignore::apply` at the end of installation, printing the report. Default behavior: writes `.gitignore` (creates or updates). With `--no-gitignore`: skips, prints `→ skipped: --no-gitignore`.

- [ ] **Step 1: Write the integration test (will fail — flag doesn't exist yet)**

Append to `tests/integration.rs`:

```rust
#[test]
fn init_creates_gitignore_with_run_bob_block() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let target = tmp.path();

    let status = std::process::Command::new(run_bob_bin())
        .args(["init", "--dir"])
        .arg(target)
        .status()
        .expect("init");
    assert!(status.success(), "init failed");

    let gitignore = target.join(".gitignore");
    assert!(gitignore.is_file(), ".gitignore must be created by init");
    let content = std::fs::read_to_string(&gitignore).expect("read .gitignore");
    assert!(
        content.contains("# run-bob"),
        ".gitignore must contain run-bob block header; got:\n{}",
        content
    );
    assert!(
        content.contains(".run-bob-backup/"),
        ".gitignore must contain .run-bob-backup/; got:\n{}",
        content
    );
}

#[test]
fn init_no_gitignore_flag_skips_gitignore() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let target = tmp.path();

    let status = std::process::Command::new(run_bob_bin())
        .args(["init", "--no-gitignore", "--dir"])
        .arg(target)
        .status()
        .expect("init --no-gitignore");
    assert!(status.success());

    assert!(
        !target.join(".gitignore").exists(),
        "--no-gitignore must skip .gitignore creation"
    );
}

#[test]
fn init_help_lists_no_gitignore_flag() {
    let output = std::process::Command::new(run_bob_bin())
        .args(["init", "--help"])
        .output()
        .expect("init --help");
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(
        stdout.contains("--no-gitignore"),
        "init --help must list --no-gitignore; got:\n{}",
        stdout
    );
}

#[test]
fn init_run_twice_keeps_gitignore_byte_identical() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let target = tmp.path();

    std::process::Command::new(run_bob_bin())
        .args(["init", "--dir"])
        .arg(target)
        .status()
        .expect("first init");

    let first = std::fs::read(target.join(".gitignore")).expect("read first .gitignore");

    std::process::Command::new(run_bob_bin())
        .args(["init", "--force", "--dir"])
        .arg(target)
        .status()
        .expect("second init");

    let second = std::fs::read(target.join(".gitignore")).expect("read second .gitignore");
    assert_eq!(first, second, "second init must leave .gitignore byte-identical");
}
```

- [ ] **Step 2: Run the tests, expect failure (flag doesn't exist)**

Run: `cargo test --test integration init_no_gitignore_flag_skips_gitignore`
Expected: clap reports unrecognized argument `--no-gitignore`, test fails.

- [ ] **Step 3: Add `--no-gitignore` to `Commands::Init` in `src/main.rs`**

In `src/main.rs`, change the `Init` variant from:

```rust
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
```

to:

```rust
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

        /// Skip writing the run-bob block into the target directory's .gitignore
        #[arg(long)]
        no_gitignore: bool,
    },
```

And in the `match cli.command` block, change:

```rust
        Commands::Init {
            force,
            minimal,
            dir,
        } => {
            commands::init::run(&dir, force, minimal)?;
        }
```

to:

```rust
        Commands::Init {
            force,
            minimal,
            dir,
            no_gitignore,
        } => {
            commands::init::run(&dir, force, minimal, no_gitignore)?;
        }
```

- [ ] **Step 4: Thread `no_gitignore` into `init::run` and call `gitignore::apply`**

In `src/commands/init.rs`, change the `run` signature from:

```rust
pub fn run(target_dir: &str, force: bool, minimal: bool) -> Result<()> {
```

to:

```rust
pub fn run(target_dir: &str, force: bool, minimal: bool, no_gitignore: bool) -> Result<()> {
```

Then at the end of the function, **just before** `print_next_steps(minimal);`, insert:

```rust
    println!();
    println!("{}", "Updating .gitignore...".bold());
    let report = crate::commands::gitignore::apply(&target, no_gitignore)?;
    crate::commands::gitignore::print_report(&report);
```

Also add a header-mode hint near the top — change:

```rust
    if force {
        println!("  {} {}", "→ mode:".dimmed(), "--force (will overwrite)".yellow());
    }
    if minimal {
        println!("  {} {}", "→ mode:".dimmed(), "--minimal (skills only)".yellow());
    }
```

to:

```rust
    if force {
        println!("  {} {}", "→ mode:".dimmed(), "--force (will overwrite)".yellow());
    }
    if minimal {
        println!("  {} {}", "→ mode:".dimmed(), "--minimal (skills only)".yellow());
    }
    if no_gitignore {
        println!("  {} {}", "→ mode:".dimmed(), "--no-gitignore (skip .gitignore)".yellow());
    }
```

- [ ] **Step 5: Run the new integration tests, expect pass**

Run:
```
cargo test --test integration init_creates_gitignore_with_run_bob_block init_no_gitignore_flag_skips_gitignore init_help_lists_no_gitignore_flag init_run_twice_keeps_gitignore_byte_identical
```
Expected: 4 passing.

- [ ] **Step 6: Run the full test suite to confirm no regressions**

Run: `cargo test`
Expected: every prior test still passes.

- [ ] **Step 7: Commit**

```bash
git add src/main.rs src/commands/init.rs tests/integration.rs
git commit -m "$(cat <<'EOF'
feat(init): auto-manage .gitignore on init, add --no-gitignore

Default behaviour creates/updates the run-bob block in the target's
.gitignore. --no-gitignore opts out. Idempotent on repeated runs.

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
EOF
)"
```

---

## Task 9: Wire `gitignore::apply` into `upgrade` + add `--no-gitignore` flag

**Files:**
- Modify: `src/main.rs`
- Modify: `src/commands/upgrade.rs`
- Modify: `tests/integration.rs`

**Goal:** `run-bob upgrade [--no-gitignore]` invokes `gitignore::apply` on every exit path **except** dry-run (which prints a one-line skip note instead).

- [ ] **Step 1: Write integration tests**

Append to `tests/integration.rs`:

```rust
#[test]
fn upgrade_creates_gitignore_when_missing() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let target = tmp.path();

    // init first so we have a valid harness layout
    std::process::Command::new(run_bob_bin())
        .args(["init", "--dir"])
        .arg(target)
        .status()
        .expect("init");

    // Then delete .gitignore to simulate an older run-bob install that
    // never wrote one. upgrade must re-create it.
    let gitignore = target.join(".gitignore");
    assert!(gitignore.is_file(), "init should have created .gitignore");
    std::fs::remove_file(&gitignore).expect("remove .gitignore");

    let status = std::process::Command::new(run_bob_bin())
        .args(["upgrade", "--dir"])
        .arg(target)
        .status()
        .expect("upgrade");
    assert!(status.success());

    assert!(gitignore.is_file(), "upgrade must re-create .gitignore");
    let content = std::fs::read_to_string(&gitignore).expect("read");
    assert!(content.contains("# run-bob"));
    assert!(content.contains(".run-bob-backup/"));
}

#[test]
fn upgrade_no_gitignore_flag_leaves_gitignore_alone() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let target = tmp.path();

    std::process::Command::new(run_bob_bin())
        .args(["init", "--dir"])
        .arg(target)
        .status()
        .expect("init");

    let gitignore = target.join(".gitignore");
    std::fs::remove_file(&gitignore).expect("remove for test");
    assert!(!gitignore.exists());

    let status = std::process::Command::new(run_bob_bin())
        .args(["upgrade", "--no-gitignore", "--dir"])
        .arg(target)
        .status()
        .expect("upgrade --no-gitignore");
    assert!(status.success());

    assert!(
        !gitignore.exists(),
        "--no-gitignore must not re-create .gitignore"
    );
}

#[test]
fn upgrade_help_lists_no_gitignore_flag() {
    let output = std::process::Command::new(run_bob_bin())
        .args(["upgrade", "--help"])
        .output()
        .expect("upgrade --help");
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(
        stdout.contains("--no-gitignore"),
        "upgrade --help must list --no-gitignore; got:\n{}",
        stdout
    );
}

#[test]
fn upgrade_dry_run_does_not_create_gitignore() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let target = tmp.path();

    std::process::Command::new(run_bob_bin())
        .args(["init", "--dir"])
        .arg(target)
        .status()
        .expect("init");

    std::fs::remove_file(target.join(".gitignore")).expect("remove");

    let status = std::process::Command::new(run_bob_bin())
        .args(["upgrade", "--dry-run", "--dir"])
        .arg(target)
        .status()
        .expect("upgrade --dry-run");
    assert!(status.success());

    assert!(
        !target.join(".gitignore").exists(),
        "dry-run must not touch .gitignore"
    );
}
```

- [ ] **Step 2: Run the tests, expect failure (flag/wiring missing)**

Run: `cargo test --test integration upgrade_no_gitignore_flag_leaves_gitignore_alone`
Expected: unrecognized argument or test failure.

- [ ] **Step 3: Add `--no-gitignore` to `Commands::Upgrade` in `src/main.rs`**

In `src/main.rs`, change the `Upgrade` variant from:

```rust
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
```

to:

```rust
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

        /// Skip writing the run-bob block into the target directory's .gitignore
        #[arg(long)]
        no_gitignore: bool,
    },
```

And update the dispatch block from:

```rust
        Commands::Upgrade {
            dir,
            dry_run,
            no_backup,
        } => {
            commands::upgrade::run(&dir, dry_run, no_backup)?;
        }
```

to:

```rust
        Commands::Upgrade {
            dir,
            dry_run,
            no_backup,
            no_gitignore,
        } => {
            commands::upgrade::run(&dir, dry_run, no_backup, no_gitignore)?;
        }
```

- [ ] **Step 4: Thread `no_gitignore` into `upgrade::run` and call at every exit path except dry-run**

In `src/commands/upgrade.rs`, change the `run` signature from:

```rust
pub fn run(target_dir: &str, dry_run: bool, no_backup: bool) -> Result<()> {
```

to:

```rust
pub fn run(target_dir: &str, dry_run: bool, no_backup: bool, no_gitignore: bool) -> Result<()> {
```

Update `print_header` to also display `--no-gitignore` mode. Change:

```rust
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
```

to:

```rust
fn print_header(target: &Path, dry_run: bool, no_backup: bool, no_gitignore: bool) {
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
    if no_gitignore {
        println!("  {} {}", "→ mode:".dimmed(), "--no-gitignore (skip .gitignore)".yellow());
    }
    println!();
}
```

And update the one caller — change `print_header(&target, dry_run, no_backup);` to `print_header(&target, dry_run, no_backup, no_gitignore);`.

Now add gitignore handling. At **three points** in `upgrade::run`, insert the gitignore section:

**Point 1 — zero-change short circuit (just before the final success line):**

Locate:

```rust
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
```

Replace with:

```rust
    if outdated.is_empty() && missing.is_empty() {
        println!();
        print_user_owned_skip_note(&user_owned);
        println!();
        println!("{}", "Updating .gitignore...".bold());
        let report = crate::commands::gitignore::apply(&target, no_gitignore)?;
        crate::commands::gitignore::print_report(&report);
        println!();
        println!(
            "{} {}",
            "✓".green().bold(),
            "All upgrade-safe assets are up to date.".green()
        );
        return Ok(());
    }
```

**Point 2 — dry-run short circuit (no I/O, just print a skip note):**

Locate:

```rust
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
```

Replace with:

```rust
    if dry_run {
        println!();
        println!(
            "{} dry-run: no files would be written. Run without --dry-run to apply.",
            "→".cyan().bold()
        );
        println!();
        print_user_owned_skip_note(&user_owned);
        println!();
        println!("{}", "Updating .gitignore...".bold());
        println!(
            "  {} {}",
            "→".bright_black(),
            "skipped: --dry-run".bright_black()
        );
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
```

**Point 3 — full apply path (just before the closing success line):**

Locate:

```rust
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

Replace with:

```rust
    println!();
    print_user_owned_skip_note(&user_owned);
    println!();
    println!("{}", "Updating .gitignore...".bold());
    let report = crate::commands::gitignore::apply(&target, no_gitignore)?;
    crate::commands::gitignore::print_report(&report);
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

- [ ] **Step 5: Run the new integration tests, expect pass**

Run:
```
cargo test --test integration upgrade_creates_gitignore_when_missing upgrade_no_gitignore_flag_leaves_gitignore_alone upgrade_help_lists_no_gitignore_flag upgrade_dry_run_does_not_create_gitignore
```
Expected: 4 passing.

- [ ] **Step 6: Run the full test suite to confirm no regressions**

Run: `cargo test`
Expected: every prior test still passes (including all the existing `upgrade_*` tests).

- [ ] **Step 7: Commit**

```bash
git add src/main.rs src/commands/upgrade.rs tests/integration.rs
git commit -m "$(cat <<'EOF'
feat(upgrade): auto-manage .gitignore on upgrade, add --no-gitignore

Wires gitignore::apply into all three exit paths (no-op, normal apply).
Dry-run prints a skip note but never writes. --no-gitignore opts out.

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
EOF
)"
```

---

## Task 10: End-to-end test — append to existing user `.gitignore`

**Files:**
- Modify: `tests/integration.rs`

**Goal:** Cover the realistic case where the user already has a project `.gitignore` (e.g. `target/\n*.log\n` from a Maven project) and runs `run-bob init`. The existing contents must be preserved, and the run-bob block must be appended.

- [ ] **Step 1: Write the test**

Append to `tests/integration.rs`:

```rust
#[test]
fn init_appends_to_existing_gitignore_preserving_content() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let target = tmp.path();

    // Pre-existing .gitignore (e.g. user's Maven project setup).
    std::fs::write(target.join(".gitignore"), "target/\n*.log\n").expect("write pre-existing");

    let status = std::process::Command::new(run_bob_bin())
        .args(["init", "--dir"])
        .arg(target)
        .status()
        .expect("init");
    assert!(status.success());

    let content = std::fs::read_to_string(target.join(".gitignore")).expect("read");
    // User content preserved.
    assert!(content.contains("target/\n"), "user line target/ must survive");
    assert!(content.contains("*.log\n"), "user line *.log must survive");
    // Run-bob block added.
    assert!(content.contains("# run-bob\n.run-bob-backup/"), "run-bob block must be present");
    // Block is separated from user content by a blank line.
    assert!(
        content.contains("*.log\n\n# run-bob"),
        "must have blank line separator between user content and block; got:\n{}",
        content
    );
}
```

- [ ] **Step 2: Run the test, expect pass**

Run: `cargo test --test integration init_appends_to_existing_gitignore_preserving_content`
Expected: passing (the algorithm from Task 3 handles this).

- [ ] **Step 3: Commit**

```bash
git add tests/integration.rs
git commit -m "$(cat <<'EOF'
test(gitignore): cover init appending to existing user .gitignore

End-to-end test for the Maven/Gradle scenario where the user already
has a project .gitignore. run-bob must preserve it and append the
block with a blank-line separator.

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
EOF
)"
```

---

## Task 11: Final verification

**Files:**
- None (verification only)

**Goal:** Confirm the full test suite is green, the binary builds in release mode, and a manual smoke test exercises the feature end-to-end.

- [ ] **Step 1: Run the full test suite**

Run: `cargo test`
Expected: every test passes. There should now be **13 unit tests** in `gitignore.rs` and **9 new integration tests** added to `tests/integration.rs`, in addition to all pre-existing tests.

- [ ] **Step 2: Build release**

Run: `cargo build --release`
Expected: clean release build.

- [ ] **Step 3: Manual smoke test — Case A (file missing)**

```bash
rm -rf /tmp/run-bob-smoke && mkdir /tmp/run-bob-smoke
./target/release/run-bob init --dir /tmp/run-bob-smoke
cat /tmp/run-bob-smoke/.gitignore
```

Expected output:
```
# run-bob
.run-bob-backup/
```

- [ ] **Step 4: Manual smoke test — Case D (idempotency)**

```bash
md5sum /tmp/run-bob-smoke/.gitignore
./target/release/run-bob init --force --dir /tmp/run-bob-smoke
md5sum /tmp/run-bob-smoke/.gitignore
```

Expected: both md5 hashes identical.

- [ ] **Step 5: Manual smoke test — `--no-gitignore`**

```bash
rm -rf /tmp/run-bob-smoke && mkdir /tmp/run-bob-smoke
./target/release/run-bob init --no-gitignore --dir /tmp/run-bob-smoke
ls /tmp/run-bob-smoke/.gitignore 2>&1
```

Expected: `ls: cannot access ...: No such file or directory`.

- [ ] **Step 6: Manual smoke test — `upgrade` path with backup**

```bash
rm -rf /tmp/run-bob-smoke && mkdir /tmp/run-bob-smoke
./target/release/run-bob init --dir /tmp/run-bob-smoke
# Corrupt a skill to force upgrade to write
echo "STALE" > /tmp/run-bob-smoke/.claude/skills/bob-survey/SKILL.md
./target/release/run-bob upgrade --dir /tmp/run-bob-smoke
ls /tmp/run-bob-smoke/.run-bob-backup/   # should show one timestamp dir
cat /tmp/run-bob-smoke/.gitignore         # block should be present and unchanged
```

Expected: a `.run-bob-backup/<UTC-timestamp>/` directory exists, and `.gitignore` still contains the run-bob block (Case D no-op was hit).

- [ ] **Step 7: (Optional) Push or open a PR**

This task list does NOT auto-push — leave that to the user.

---

## Summary of new tests

After completion the repo will have:

- **13 unit tests** in `src/commands/gitignore.rs` covering the 4 algorithm cases (1+3+3+2) + header variants (3) + user-added entries (1).
- **9 new integration tests** in `tests/integration.rs`:
  - `init_creates_gitignore_with_run_bob_block`
  - `init_no_gitignore_flag_skips_gitignore`
  - `init_help_lists_no_gitignore_flag`
  - `init_run_twice_keeps_gitignore_byte_identical`
  - `init_appends_to_existing_gitignore_preserving_content`
  - `upgrade_creates_gitignore_when_missing`
  - `upgrade_no_gitignore_flag_leaves_gitignore_alone`
  - `upgrade_help_lists_no_gitignore_flag`
  - `upgrade_dry_run_does_not_create_gitignore`

All existing tests continue to pass unchanged.
