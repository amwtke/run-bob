# bob-model Server Internalization Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Internalize the 4 brainstorming server scripts (server.cjs / helper.js / start-server.sh / stop-server.sh) into run-bob, rename namespace to `bob-review`, and remove the dependency on `superpowers/brainstorming/scripts/`.

**Architecture:** Add 4 new `Asset` entries to `src/assets.rs` registry (each `include_str!`'d into the binary); install them to `.claude/skills/bob-model/scripts/`; chmod +x for `.sh` files; update `src/templates/skills/bob-model.md` to call the local path and use `window.bobReview` namespace; add `.bob/` to gitignore.

**Tech Stack:**
- Rust (run-bob CLI): `src/assets.rs` (asset registry) / `src/commands/init.rs` (write logic) / `src/commands/upgrade.rs` (upgrade logic) / `src/commands/gitignore.rs` (entry list) / `tests/integration.rs` (tests)
- Node.js (script runtime): standard library only, ≥14
- Bash (launcher scripts): GNU/Mac compatible
- Source spec: `docs/superpowers/specs/2026-05-15-bob-model-server-internalization-design.md` (commit `661b030`)

---

## File Structure

### Files to create

```
src/templates/scripts/bob-model/                  (new dir)
├─ server.cjs                                     copied from superpowers/brainstorming + namespace rename
├─ helper.js                                      copied + namespace rename
├─ start-server.sh                                copied + namespace + Node detection
└─ stop-server.sh                                 copied + namespace
```

### Files to modify

| File | Change |
|---|---|
| `src/assets.rs` | Add 4 `Asset` entries for the scripts (each pointing to `.claude/skills/bob-model/scripts/<name>` via `rel_path`) |
| `src/commands/init.rs` | Extend `write_file` to chmod +x for `.sh` files (Unix only; Windows skips) |
| `src/commands/gitignore.rs` | Extend `GITIGNORE_ENTRIES` with `.bob/` |
| `src/templates/skills/bob-model.md` | Update Stage 3.1 start-server.sh path (hardcoded → project-relative); replace `window.brainstorm` → `window.bobReview` (~3 places); update state-dir references |
| `tests/integration.rs` | Add 4 new tests + extend `init_creates_bob_model_skill` token list with `window.bobReview` |
| `Cargo.toml` | Version bump 0.4.0 → 0.5.0 |
| `Cargo.lock` | Auto-sync via `cargo build` |
| `README.md` | Add Node.js to optional dependencies |

### Files NOT to touch

- Other `bob-*.md` skill templates (only bob-model is affected)
- `src/templates/root/CLAUDE.md` / `ARCHITECTURE.md` (user-owned)
- `.gitignore` of run-bob repo itself (existing `.bob/` already absent from run-bob's own gitignore is fine — repo doesn't run model on itself)

---

## Task 1: Copy 4 scripts from superpowers + namespace rename

**Files:**
- Create: `src/templates/scripts/bob-model/server.cjs`
- Create: `src/templates/scripts/bob-model/helper.js`
- Create: `src/templates/scripts/bob-model/start-server.sh`
- Create: `src/templates/scripts/bob-model/stop-server.sh`

**Source:**
- `/Users/xiaojin/.claude/plugins/cache/claude-plugins-official/superpowers/5.1.0/skills/brainstorming/scripts/*`

- [ ] **Step 1.1: Create source directory**

```bash
mkdir -p /Users/xiaojin/workshop/run-bob/src/templates/scripts/bob-model
```

- [ ] **Step 1.2: Copy server.cjs and apply namespace rename**

```bash
SRC=/Users/xiaojin/.claude/plugins/cache/claude-plugins-official/superpowers/5.1.0/skills/brainstorming/scripts
DST=/Users/xiaojin/workshop/run-bob/src/templates/scripts/bob-model
cp "$SRC/server.cjs" "$DST/server.cjs"
sed -i.bak \
  -e 's/BRAINSTORM_PORT/BOB_REVIEW_PORT/g' \
  -e 's/BRAINSTORM_HOST/BOB_REVIEW_HOST/g' \
  -e 's/BRAINSTORM_URL_HOST/BOB_REVIEW_URL_HOST/g' \
  -e 's/BRAINSTORM_DIR/BOB_REVIEW_DIR/g' \
  -e 's|/tmp/brainstorm|/tmp/bob-review|g' \
  -e 's/BRAINSTORM_OWNER_PID/BOB_REVIEW_OWNER_PID/g' \
  "$DST/server.cjs"
rm "$DST/server.cjs.bak"
```

- [ ] **Step 1.3: Prepend LICENSE header to server.cjs**

Use the Edit tool to prepend (find the existing first line via `head -1 server.cjs`, then Edit to insert above it):

```javascript
// Adapted from superpowers brainstorming visual companion
// Source: superpowers@5.1.0 (Anthropic, MIT)
// Migrated to run-bob with namespace bob-review for /bob-model interactive review.
// See docs/superpowers/specs/2026-05-15-bob-model-server-internalization-design.md

```

- [ ] **Step 1.4: Copy + rename helper.js**

```bash
cp "$SRC/helper.js" "$DST/helper.js"
sed -i.bak \
  -e 's/window\.brainstorm/window.bobReview/g' \
  "$DST/helper.js"
rm "$DST/helper.js.bak"
```

Add same LICENSE header (4 lines) at top of helper.js via Edit.

- [ ] **Step 1.5: Copy + rename start-server.sh**

```bash
cp "$SRC/start-server.sh" "$DST/start-server.sh"
chmod +x "$DST/start-server.sh"
sed -i.bak \
  -e 's/BRAINSTORM_PORT/BOB_REVIEW_PORT/g' \
  -e 's/BRAINSTORM_HOST/BOB_REVIEW_HOST/g' \
  -e 's/BRAINSTORM_URL_HOST/BOB_REVIEW_URL_HOST/g' \
  -e 's/BRAINSTORM_DIR/BOB_REVIEW_DIR/g' \
  -e 's/BRAINSTORM_OWNER_PID/BOB_REVIEW_OWNER_PID/g' \
  -e 's|\.superpowers/brainstorm|.bob/model-review|g' \
  "$DST/start-server.sh"
rm "$DST/start-server.sh.bak"
```

Add LICENSE header (use `#` comment instead of `//`).

- [ ] **Step 1.6: Copy + rename stop-server.sh**

```bash
cp "$SRC/stop-server.sh" "$DST/stop-server.sh"
chmod +x "$DST/stop-server.sh"
sed -i.bak \
  -e 's/BRAINSTORM_PORT/BOB_REVIEW_PORT/g' \
  -e 's/BRAINSTORM_DIR/BOB_REVIEW_DIR/g' \
  -e 's|\.superpowers/brainstorm|.bob/model-review|g' \
  "$DST/stop-server.sh"
rm "$DST/stop-server.sh.bak"
```

Add LICENSE header.

- [ ] **Step 1.7: Verify namespace rename complete**

```bash
cd /Users/xiaojin/workshop/run-bob
grep -rn "BRAINSTORM\|window\.brainstorm" src/templates/scripts/bob-model/
```

Expected: 0 matches (everything was renamed).

```bash
grep -rln "BOB_REVIEW\|window.bobReview\|.bob/model-review" src/templates/scripts/bob-model/
```

Expected: 4 files (all 4 scripts).

- [ ] **Step 1.8: Commit**

```bash
cd /Users/xiaojin/workshop/run-bob
git add src/templates/scripts/bob-model/
git commit -m "feat(bob-model): copy 4 brainstorming server scripts with bob-review namespace rename"
```

---

## Task 2: Add Node.js detection to start-server.sh

**Files:**
- Modify: `src/templates/scripts/bob-model/start-server.sh`

- [ ] **Step 2.1: Find insertion point**

```bash
head -25 src/templates/scripts/bob-model/start-server.sh
```

Locate the line right after the LICENSE header + shebang where `set -e` or similar pre-flight setup ends (before the first `--project-dir` arg parsing).

- [ ] **Step 2.2: Insert Node detection block via Edit**

Find a unique anchor (e.g., the first `case` statement for arg parsing or a comment) and insert this block just above it:

```bash
# --- Node.js detection (added by run-bob bob-review internalization) ---
if ! command -v node >/dev/null 2>&1; then
  echo '{"error":"node not found","fix":"Install Node.js >=14 to use /bob-model interactive review. Without node, the skill falls back to read-only html."}' >&2
  exit 2
fi
NODE_MAJOR=$(node -e 'process.stdout.write(String(process.versions.node.split(".")[0]))')
if [ "$NODE_MAJOR" -lt 14 ]; then
  echo "{\"error\":\"node $NODE_MAJOR too old\",\"fix\":\"Upgrade Node.js to >=14\"}" >&2
  exit 3
fi
```

- [ ] **Step 2.3: Verify**

```bash
grep -n "command -v node" src/templates/scripts/bob-model/start-server.sh
# Expect: 1 line
grep -n "NODE_MAJOR" src/templates/scripts/bob-model/start-server.sh
# Expect: 2 lines
```

- [ ] **Step 2.4: Smoke test the script syntactically**

```bash
bash -n src/templates/scripts/bob-model/start-server.sh
echo "exit=$?"
```

Expected: exit 0 (syntax OK).

- [ ] **Step 2.5: Commit**

```bash
cd /Users/xiaojin/workshop/run-bob
git add src/templates/scripts/bob-model/start-server.sh
git commit -m "feat(bob-model): add Node.js >=14 detection to start-server.sh"
```

---

## Task 3: Register 4 scripts in assets.rs

**Files:**
- Modify: `src/assets.rs`

- [ ] **Step 3.1: Locate bob-model Skill asset entry**

```bash
grep -n "bob-model.*SKILL.md" src/assets.rs
```

Expected: line ~118 (the bob-model.md skill entry). Insert the 4 new entries immediately after this entry.

- [ ] **Step 3.2: Add 4 Asset entries via Edit**

Find:

```rust
    Asset {
        rel_path: &[".claude", "skills", "bob-model", "SKILL.md"],
        content: include_str!("templates/skills/bob-model.md"),
        category: Category::Skill,
        included_in_minimal: true,
        upgrade_safe: true,
    },
```

Insert immediately after (preserve the closing `},` of bob-model entry):

```rust
    Asset {
        rel_path: &[".claude", "skills", "bob-model", "scripts", "server.cjs"],
        content: include_str!("templates/scripts/bob-model/server.cjs"),
        category: Category::Skill,
        included_in_minimal: true,
        upgrade_safe: true,
    },
    Asset {
        rel_path: &[".claude", "skills", "bob-model", "scripts", "helper.js"],
        content: include_str!("templates/scripts/bob-model/helper.js"),
        category: Category::Skill,
        included_in_minimal: true,
        upgrade_safe: true,
    },
    Asset {
        rel_path: &[".claude", "skills", "bob-model", "scripts", "start-server.sh"],
        content: include_str!("templates/scripts/bob-model/start-server.sh"),
        category: Category::Skill,
        included_in_minimal: true,
        upgrade_safe: true,
    },
    Asset {
        rel_path: &[".claude", "skills", "bob-model", "scripts", "stop-server.sh"],
        content: include_str!("templates/scripts/bob-model/stop-server.sh"),
        category: Category::Skill,
        included_in_minimal: true,
        upgrade_safe: true,
    },
```

- [ ] **Step 3.3: Compile check**

```bash
cd /Users/xiaojin/workshop/run-bob
cargo build --release 2>&1 | tail -5
```

Expected: compiles successfully. (If `include_str!` fails on a script path, the path is wrong.)

- [ ] **Step 3.4: Commit**

```bash
cd /Users/xiaojin/workshop/run-bob
git add src/assets.rs
git commit -m "feat(bob-model): register 4 review server scripts in asset registry"
```

---

## Task 4: Add executable permission handling for .sh files in init.rs

**Files:**
- Modify: `src/commands/init.rs` (write_file function around line 76)

- [ ] **Step 4.1: Write the failing test first (TDD)**

Open `tests/integration.rs` and add a new test:

```rust
#[test]
fn init_makes_sh_scripts_executable() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let target = tmp.path();
    run_bob::commands::init::run(target.to_str().unwrap(), false, false, true)
        .expect("init failed");

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let script_path = target.join(".claude/skills/bob-model/scripts/start-server.sh");
        let meta = std::fs::metadata(&script_path).expect("script must exist");
        let mode = meta.permissions().mode();
        assert!(
            mode & 0o111 != 0,
            "start-server.sh should be executable; got mode {:o}",
            mode
        );
    }
}
```

- [ ] **Step 4.2: Run test to verify it fails**

```bash
cargo test --release --test integration init_makes_sh_scripts_executable 2>&1 | tail -10
```

Expected: FAIL with "should be executable" assertion (current code uses fs::write without chmod).

- [ ] **Step 4.3: Add chmod logic to write_file in init.rs**

Find in `src/commands/init.rs`:

```rust
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
```

Replace with:

```rust
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
    set_executable_if_shell(path)?;
    crate::success(display);
    Ok(())
}

#[cfg(unix)]
fn set_executable_if_shell(path: &Path) -> Result<()> {
    if path.extension().and_then(|e| e.to_str()) == Some("sh") {
        use std::os::unix::fs::PermissionsExt;
        let metadata = fs::metadata(path)
            .with_context(|| format!("Failed to stat {}", path.display()))?;
        let mut perms = metadata.permissions();
        perms.set_mode(perms.mode() | 0o111);
        fs::set_permissions(path, perms)
            .with_context(|| format!("Failed to chmod +x {}", path.display()))?;
    }
    Ok(())
}

#[cfg(not(unix))]
fn set_executable_if_shell(_path: &Path) -> Result<()> {
    Ok(())  // Windows: shell scripts run via bash; permission bit unused
}
```

- [ ] **Step 4.4: Run test to verify it passes**

```bash
cargo test --release --test integration init_makes_sh_scripts_executable 2>&1 | tail -10
```

Expected: PASS.

- [ ] **Step 4.5: Run full test suite to check for regressions**

```bash
cargo test --release 2>&1 | tail -5
```

Expected: all tests pass (54 + 1 new = 55).

- [ ] **Step 4.6: Commit**

```bash
cd /Users/xiaojin/workshop/run-bob
git add src/commands/init.rs tests/integration.rs
git commit -m "feat(init): chmod +x for .sh scripts on Unix; add integration test"
```

---

## Task 5: Update bob-model SKILL.md template (path + namespace)

**Files:**
- Modify: `src/templates/skills/bob-model.md`

- [ ] **Step 5.1: Update Stage 3.1 start-server.sh path**

Find in `src/templates/skills/bob-model.md`:

```bash
SCRIPT=/Users/xiaojin/.claude/plugins/cache/claude-plugins-official/superpowers/5.1.0/skills/brainstorming/scripts/start-server.sh
"$SCRIPT" --project-dir <repo-root>
```

Replace with:

```bash
SCRIPT="<project-root>/.claude/skills/bob-model/scripts/start-server.sh"
"$SCRIPT" --project-dir <project-root>
```

- [ ] **Step 5.2: Replace window.brainstorm with window.bobReview throughout JS**

```bash
cd /Users/xiaojin/workshop/run-bob
grep -n "window.brainstorm" src/templates/skills/bob-model.md
```

Expected: 3-5 hits in the embedded JS code block. Apply Edit tool for each (or use the Edit tool with replace_all if all are unambiguous).

Since the JS block lives inside a 4-space-indented code block, the strings are: `window.brainstorm.send`, `!window.brainstorm`, etc.

Use Edit with `replace_all: true`:
- old: `window.brainstorm`
- new: `window.bobReview`

- [ ] **Step 5.3: Update state-dir references in the skill body**

Search for `.superpowers/brainstorm` references:

```bash
grep -n "\.superpowers/brainstorm" src/templates/skills/bob-model.md
```

For each hit (in description paragraphs, examples, etc.), Edit to `.bob/model-review`.

- [ ] **Step 5.4: Verify all renames complete**

```bash
cd /Users/xiaojin/workshop/run-bob
grep -nE "window\.brainstorm|\.superpowers/brainstorm|5\.1\.0/skills/brainstorming" src/templates/skills/bob-model.md
```

Expected: 0 matches.

```bash
grep -nE "window\.bobReview|\.bob/model-review|\.claude/skills/bob-model/scripts" src/templates/skills/bob-model.md
```

Expected: ≥ 3 matches.

- [ ] **Step 5.5: Commit**

```bash
cd /Users/xiaojin/workshop/run-bob
git add src/templates/skills/bob-model.md
git commit -m "feat(bob-model): update SKILL.md to use local scripts path + bobReview namespace"
```

---

## Task 6: Add .bob/ to gitignore management

**Files:**
- Modify: `src/commands/gitignore.rs`

- [ ] **Step 6.1: Write the failing test first**

Add to `tests/integration.rs`:

```rust
#[test]
fn init_adds_bob_dir_to_gitignore() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let target = tmp.path();
    run_bob::commands::init::run(target.to_str().unwrap(), false, false, false)
        .expect("init failed");

    let gitignore_content = std::fs::read_to_string(target.join(".gitignore"))
        .expect("gitignore should exist after init");
    assert!(
        gitignore_content.contains(".bob/"),
        "gitignore should contain .bob/ entry; got: {}",
        gitignore_content
    );
}
```

- [ ] **Step 6.2: Run test to verify it fails**

```bash
cargo test --release --test integration init_adds_bob_dir_to_gitignore 2>&1 | tail -5
```

Expected: FAIL ("gitignore should contain .bob/").

- [ ] **Step 6.3: Extend GITIGNORE_ENTRIES**

Find in `src/commands/gitignore.rs` line 11:

```rust
pub const GITIGNORE_ENTRIES: &[&str] = &[".run-bob-backup/"];
```

Replace with:

```rust
pub const GITIGNORE_ENTRIES: &[&str] = &[".run-bob-backup/", ".bob/"];
```

- [ ] **Step 6.4: Run test to verify pass**

```bash
cargo test --release --test integration init_adds_bob_dir_to_gitignore 2>&1 | tail -5
```

Expected: PASS.

- [ ] **Step 6.5: Check existing gitignore tests still pass**

```bash
cargo test --release --test integration gitignore 2>&1 | tail -5
```

Expected: all gitignore-related tests pass (the "UpToDate" semantics adjust for the new entry).

- [ ] **Step 6.6: Commit**

```bash
cd /Users/xiaojin/workshop/run-bob
git add src/commands/gitignore.rs tests/integration.rs
git commit -m "feat(gitignore): include .bob/ for bob-model interactive review runtime state"
```

---

## Task 7: Update existing init_creates_bob_model_skill test for new tokens

**Files:**
- Modify: `tests/integration.rs` (in the existing `init_creates_bob_model_skill` test)

- [ ] **Step 7.1: Locate existing test**

```bash
grep -n "fn init_creates_bob_model_skill\b" tests/integration.rs
```

Find the existing token list (the array of strings passed to `assert!(content.contains(token))`).

- [ ] **Step 7.2: Add new required tokens via Edit**

Find the existing tokens list (around line 1740-1782, search for `"md 是 SSoT"`). Add these new tokens at the end (before the closing `]`):

```rust
        // bob-review (post-internalization)
        "window.bobReview",
        ".claude/skills/bob-model/scripts/start-server.sh",
```

And REMOVE these obsolete tokens if present (search and remove):

```rust
        // OLD — these refer to the pre-internalization superpowers paths
        "window.brainstorm",  // if present
```

Note: if the existing test list does NOT have these old tokens, skip the removal.

- [ ] **Step 7.3: Run test**

```bash
cargo test --release --test integration init_creates_bob_model_skill 2>&1 | tail -8
```

Expected: PASS (asserting new tokens present and old absent).

- [ ] **Step 7.4: Commit**

```bash
cd /Users/xiaojin/workshop/run-bob
git add tests/integration.rs
git commit -m "test(bob-model): require bobReview namespace + local scripts path"
```

---

## Task 8: Add new integration test: scripts get installed

**Files:**
- Modify: `tests/integration.rs`

- [ ] **Step 8.1: Add test for all 4 scripts**

Append to `tests/integration.rs`:

```rust
#[test]
fn init_installs_bob_model_scripts() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let target = tmp.path();
    run_bob::commands::init::run(target.to_str().unwrap(), false, false, true)
        .expect("init failed");

    let scripts_dir = target.join(".claude/skills/bob-model/scripts");
    for fname in ["server.cjs", "helper.js", "start-server.sh", "stop-server.sh"] {
        let path = scripts_dir.join(fname);
        assert!(path.is_file(), "{} should be installed; not found", path.display());
    }

    // Verify content namespace
    let server_content = std::fs::read_to_string(scripts_dir.join("server.cjs"))
        .expect("server.cjs readable");
    assert!(
        server_content.contains("BOB_REVIEW_PORT"),
        "server.cjs should contain BOB_REVIEW_PORT (got namespace-renamed); content head: {}",
        &server_content[..server_content.len().min(200)]
    );
    assert!(
        !server_content.contains("BRAINSTORM_PORT"),
        "server.cjs must NOT contain old BRAINSTORM_PORT after namespace rename"
    );

    let helper_content = std::fs::read_to_string(scripts_dir.join("helper.js"))
        .expect("helper.js readable");
    assert!(
        helper_content.contains("window.bobReview"),
        "helper.js should expose window.bobReview"
    );
    assert!(
        !helper_content.contains("window.brainstorm"),
        "helper.js must NOT contain old window.brainstorm"
    );
}
```

- [ ] **Step 8.2: Run test**

```bash
cargo test --release --test integration init_installs_bob_model_scripts 2>&1 | tail -8
```

Expected: PASS.

- [ ] **Step 8.3: Commit**

```bash
cd /Users/xiaojin/workshop/run-bob
git add tests/integration.rs
git commit -m "test(bob-model): verify 4 scripts installed with bob-review namespace"
```

---

## Task 9: Add integration test: start-server.sh has Node detection

**Files:**
- Modify: `tests/integration.rs`

- [ ] **Step 9.1: Add test**

Append:

```rust
#[test]
fn start_server_script_has_node_detection() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let target = tmp.path();
    run_bob::commands::init::run(target.to_str().unwrap(), false, false, true)
        .expect("init failed");

    let content = std::fs::read_to_string(
        target.join(".claude/skills/bob-model/scripts/start-server.sh")
    ).expect("start-server.sh readable");

    assert!(
        content.contains("command -v node"),
        "start-server.sh should detect missing node"
    );
    assert!(
        content.contains("NODE_MAJOR") && content.contains("< 14"),
        "start-server.sh should reject Node < 14"
    );
}
```

- [ ] **Step 9.2: Run test**

```bash
cargo test --release --test integration start_server_script_has_node_detection 2>&1 | tail -5
```

Expected: PASS.

- [ ] **Step 9.3: Commit**

```bash
cd /Users/xiaojin/workshop/run-bob
git add tests/integration.rs
git commit -m "test(bob-model): verify start-server.sh detects Node.js >=14"
```

---

## Task 10: Add integration test: upgrade replaces stale scripts

**Files:**
- Modify: `tests/integration.rs`

- [ ] **Step 10.1: Add test verifying upgrade-safe behavior for scripts**

Append:

```rust
#[test]
fn upgrade_replaces_stale_bob_model_scripts() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let target = tmp.path();

    // First init
    run_bob::commands::init::run(target.to_str().unwrap(), false, false, true)
        .expect("init failed");

    // Tamper with server.cjs to simulate a stale version
    let server_path = target.join(".claude/skills/bob-model/scripts/server.cjs");
    std::fs::write(&server_path, "// stale version, should be replaced\n").unwrap();
    let after_tamper = std::fs::read_to_string(&server_path).unwrap();
    assert_eq!(after_tamper, "// stale version, should be replaced\n");

    // Run upgrade
    run_bob::commands::upgrade::run(target.to_str().unwrap(), false, true)
        .expect("upgrade failed");

    // Verify content restored
    let after_upgrade = std::fs::read_to_string(&server_path).unwrap();
    assert!(
        after_upgrade.contains("BOB_REVIEW_PORT"),
        "upgrade should restore stale server.cjs with bob-review namespace"
    );
    assert!(
        after_upgrade.len() > 100,
        "upgrade should restore full content (got {} bytes)",
        after_upgrade.len()
    );
}
```

Note: check the actual `upgrade::run` signature in `src/commands/upgrade.rs`. The signature may be `(target_dir, no_backup, no_gitignore)`. Adjust test as needed.

- [ ] **Step 10.2: Verify run signature**

```bash
grep -n "pub fn run" src/commands/upgrade.rs
```

Adjust test invocation to match signature.

- [ ] **Step 10.3: Run test**

```bash
cargo test --release --test integration upgrade_replaces_stale_bob_model_scripts 2>&1 | tail -8
```

Expected: PASS.

- [ ] **Step 10.4: Commit**

```bash
cd /Users/xiaojin/workshop/run-bob
git add tests/integration.rs
git commit -m "test(bob-model): verify upgrade restores stale scripts to bundled version"
```

---

## Task 11: Update README.md with Node.js optional dependency note

**Files:**
- Modify: `README.md`

- [ ] **Step 11.1: Find dependencies / requirements section**

```bash
grep -n "依赖\|Dependencies\|Requirements\|要求" /Users/xiaojin/workshop/run-bob/README.md
```

If no such section, add one before the install instructions.

- [ ] **Step 11.2: Add Node.js note**

Add a line/bullet:

```markdown
- (可选)Node.js ≥14 —— `/bob-model` 交互式 review 需要。不装时降级为只读 html。
```

- [ ] **Step 11.3: Commit**

```bash
cd /Users/xiaojin/workshop/run-bob
git add README.md
git commit -m "docs(readme): note Node.js >=14 as optional dependency for /bob-model"
```

---

## Task 12: Bump version + release

**Files:**
- Modify: `Cargo.toml`
- Modify: `Cargo.lock` (auto)

- [ ] **Step 12.1: Run full test suite to confirm green**

```bash
cd /Users/xiaojin/workshop/run-bob
cargo test --release 2>&1 | tail -5
```

Expected: all tests pass (≥ 58 = 54 baseline + 4 new from Tasks 4/6/8/9/10).

- [ ] **Step 12.2: Bump version**

Edit `Cargo.toml`:

```toml
version = "0.5.0"
```

(was `0.4.0`)

- [ ] **Step 12.3: Rebuild to sync Cargo.lock**

```bash
cargo build --release 2>&1 | tail -3
```

Expected: `Compiling run-bob v0.5.0`.

- [ ] **Step 12.4: Commit version bump**

```bash
cd /Users/xiaojin/workshop/run-bob
git add Cargo.toml Cargo.lock
git commit -m "chore(release): bump version to v0.5.0"
```

- [ ] **Step 12.5: Tag with release notes**

```bash
git tag -a v0.5.0 -m "$(cat <<'EOF'
v0.5.0 — bob-model server internalization (remove superpowers dependency)

把 bob-model 交互式 review 用到的 4 个脚本(server.cjs / helper.js /
start-server.sh / stop-server.sh)从 superpowers 内化到 run-bob,
改 namespace 为 bob-review,目标机器不再需要装 superpowers。

主要变化:
- 4 个脚本作为 run-bob 资产注册,init 时安装到 .claude/skills/bob-model/scripts/
- .sh 脚本在 Unix 系统自动获得可执行权限
- start-server.sh 增加 Node.js >=14 检测,缺失时优雅降级
- SKILL.md 改用项目本地路径 + window.bobReview namespace
- .bob/ 自动加入 .gitignore
- 7 个新增 / 改造的集成测试

V2 留底: Rust 重写 server / 通用化为多 skill 共享 primitive / Windows
native PowerShell / 多人协作。

参见: docs/superpowers/specs/2026-05-15-bob-model-server-internalization-design.md

Full Changelog: https://github.com/amwtke/run-bob/compare/v0.4.0...v0.5.0
EOF
)"
```

- [ ] **Step 12.6: Push commits and tag**

```bash
git push origin master
git push origin v0.5.0
```

- [ ] **Step 12.7: Verify CI started**

```bash
gh run list --workflow release.yml --limit 2 2>&1 | head -3
```

Expected: top row shows `in_progress` for `v0.5.0`.

- [ ] **Step 12.8: Wait for release publish (optional)**

```bash
# After ~2 minutes, verify
gh release view v0.5.0 2>&1 | head -10
```

Expected: release `v0.5.0` exists with multi-platform binaries.

---

## Manual End-to-End Validation (after Task 12)

These are not subagent tasks — the user / operator validates manually in a clean environment.

- [ ] **MVP 1: Fresh project init**

```bash
mkdir -p /tmp/test-bob-review
cd /tmp/test-bob-review
git init
~/.cargo/bin/run-bob init . --force
```

Verify:
- `.claude/skills/bob-model/SKILL.md` exists
- `.claude/skills/bob-model/scripts/{server.cjs,helper.js,start-server.sh,stop-server.sh}` all exist
- `.sh` files have executable bit (`ls -l .claude/skills/bob-model/scripts/start-server.sh` → `-rwx...`)
- `.gitignore` contains `.bob/`

- [ ] **MVP 2: Start server manually**

```bash
.claude/skills/bob-model/scripts/start-server.sh --project-dir .
```

Verify:
- JSON output with `port`, `url`, `screen_dir`, `state_dir`
- `screen_dir` is under `<project>/.bob/model-review/<session>/content/`
- Server accessible: `curl -sS -o /dev/null -w "%{http_code}\n" $URL` → `200` or `404` (no html yet)

- [ ] **MVP 3: Stop server**

```bash
.claude/skills/bob-model/scripts/stop-server.sh <state_dir>
```

Or read the session dir from state and use stop-server.sh per its arg conventions.

Verify: server stopped, `server-stopped` file exists.

- [ ] **MVP 4: Cleanup**

```bash
rm -rf /tmp/test-bob-review
```

---

## Risks & Rollback

| Risk | Likelihood | Impact | Mitigation |
|---|---|---|---|
| `include_str!` fails compile (path wrong) | Medium | High | Task 3.3 compile check catches early |
| sed namespace rename misses occurrences | Medium | Medium | Task 1.7 grep verification catches |
| chmod logic breaks on non-Unix CI | Low | Medium | Task 4 uses `#[cfg(unix)]` gate |
| Existing tests break due to gitignore entry change | Medium | Low | Task 6.5 verifies gitignore tests still pass |
| Upstream superpowers brainstorming server has breaking change | Low | None now (we forked) | Cherry-pick manually if needed |
| Node detection regex breaks on unusual node versions | Low | Low | Manual MVP 1 validates real env |

**Rollback**: `git revert <commit-range>` for any phase; binary release v0.4.0 still works for users on master.

---

## V2 留底 (not in this plan)

- Rust rewrite of server.cjs (remove Node.js dep)
- General `web-review` primitive (other bob-* / non-bob skills reuse)
- Windows native PowerShell launcher (no Git Bash needed)
- Multi-user collaborative review

---

*Plan generated by superpowers:writing-plans · 2026-05-15 · 待 superpowers:subagent-driven-development 或 superpowers:executing-plans 接力*
