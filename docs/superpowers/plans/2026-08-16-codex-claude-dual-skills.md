# Codex + Claude Code Dual-Skill Support Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make `run-bob` generate, upgrade, validate, and document real skills for both Claude Code and Codex while preserving every existing Claude path and slash invocation, and make the repository install skill bootstrap a missing Rust toolchain automatically.

**Architecture:** Keep `src/templates/skills` and `src/templates/scripts` as the only generated-skill content source, but register two independent destinations for each asset: `.claude/skills` and `.agents/skills`. Reuse the existing per-file init/upgrade semantics, add host-specific reporting categories, share Unix executable-bit handling, and keep contributor source installation behind deterministic POSIX and PowerShell helpers that use official rustup endpoints without changing a pre-existing global default.

**Tech Stack:** Rust 2021 with MSRV 1.75, `clap`, `anyhow`, standard-library integration tests, POSIX `sh`, PowerShell, Markdown/YAML skill files, Cargo locked builds.

---

## Guardrails for every task

- Keep `.claude/skills/**` as real files at the existing paths. Never move, delete, rename, link, or replace that root with `.agents`.
- Create `.agents/skills/**` as independent real files whose bytes match the corresponding Claude files.
- Keep all Claude `/bob-*`, `/visual-md`, `/install`, and `superpowers:*` forms. Codex `$...` forms are additive.
- Do not create or manage `AGENTS.md`. Do not rename or weaken `CLAUDE.md`; it remains user-owned during upgrade.
- Add the failing test before each behavior change, run it to observe the intended failure, then implement the smallest passing change.
- Use `cargo test --locked` for the complete suite. Do not regenerate dependencies without the lockfile.
- Tests for Rust bootstrap must use temporary homes and mocked executables. They must not access the network or modify the machine's real Rust installation.
- Leave the released-binary installers `install.sh` and `install.ps1` independent of Rust. The new Rust bootstrap applies only to source/contributor installation.

## Task 0: Prepare the currently Rust-free development shell

**Files:** None in the repository.

- [ ] **Step 1: Reconfirm whether Rust is installed but absent from `PATH`.**

  Run:

  ```bash
  command -v rustc || true
  command -v cargo || true
  test -x "${CARGO_HOME:-$HOME/.cargo}/bin/rustc" && "${CARGO_HOME:-$HOME/.cargo}/bin/rustc" --version || true
  test -x "${CARGO_HOME:-$HOME/.cargo}/bin/cargo" && "${CARGO_HOME:-$HOME/.cargo}/bin/cargo" --version || true
  ```

  The planning session observed `rustc: command not found`. If both absolute binaries already exist, source `${CARGO_HOME:-$HOME/.cargo}/env` and skip the download.

- [ ] **Step 2: Install official stable Rust with the minimal profile only when still absent.**

  The user explicitly authorized automatic Rust installation for this work. Run Rust's official TLS-constrained installer:

  ```bash
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \
    | sh -s -- -y --profile minimal --default-toolchain stable --no-modify-path
  ```

  This creates the first stable default for a previously Rust-free account; it does not replace an existing toolchain and does not edit shell startup files.

- [ ] **Step 3: Activate and verify Rust for this and subsequent worker shells.**

  ```bash
  . "${CARGO_HOME:-$HOME/.cargo}/env"
  rustc --version
  cargo --version
  cargo test --locked binary_prints_version -- --exact
  ```

  Expected: `rustc` and `cargo` succeed and the baseline test passes. Every later worker starting a fresh shell must source the same `env` file before a Cargo command if `$CARGO_HOME/bin` is not already on `PATH`.

- [ ] **Step 4: Do not commit environment setup.**

  Confirm `git status --short` contains no files produced by rustup inside the repository.

## Task 1: Mirror the embedded skill registry and make init dual-host

**Files:**

- Modify: `src/assets.rs`
- Modify: `src/commands/init.rs`
- Modify: `tests/integration.rs`

- [ ] **Step 1: Add reusable dual-host test helpers.**

  In `tests/integration.rs`, add these logical skill names and roots:

  ```rust
  const GENERATED_SKILLS: &[&str] = &[
      "bob-survey",
      "bob-model",
      "bob-stories",
      "bob-identify",
      "bob-onion",
      "bob-spec",
      "bob-compliance",
      "bob-nfr",
      "visual-md",
  ];

  const SKILL_ROOTS: &[&str] = &[".claude/skills", ".agents/skills"];
  ```

  Add a recursive helper that returns a `BTreeMap<String, (Vec<u8>, Option<u32>)>` for every file below a root. On Unix, store `permissions().mode() & 0o111`; on non-Unix, store `None`. Do not follow symlink directories.

- [ ] **Step 2: Write the failing registry and init tests.**

  Add:

  - `asset_registry_has_identical_claude_and_codex_skill_entries`
  - `init_installs_byte_identical_dual_skill_trees`
  - `minimal_init_installs_all_nine_skills_for_both_hosts_only`
  - `init_without_force_preserves_existing_claude_skill_and_adds_codex`

  The registry test must strip the first two path segments from every skill asset, compare Claude and Codex maps, and assert equal content, `included_in_minimal`, `upgrade_safe`, and shell-file treatment for all 22 asset tails:

  ```text
  bob-identify/SKILL.md
  bob-onion/SKILL.md
  bob-spec/SKILL.md
  bob-survey/SKILL.md
  bob-stories/SKILL.md
  bob-nfr/SKILL.md
  bob-compliance/SKILL.md
  bob-model/SKILL.md
  bob-model/scripts/server.cjs
  bob-model/scripts/helper.js
  bob-model/scripts/start-server.sh
  bob-model/scripts/stop-server.sh
  bob-model/scripts/frame-template.html
  visual-md/SKILL.md
  visual-md/scripts/server.cjs
  visual-md/scripts/client.js
  visual-md/scripts/md2html.cjs
  visual-md/scripts/slugify.cjs
  visual-md/scripts/frame-template.html
  visual-md/scripts/start-server.sh
  visual-md/scripts/stop-server.sh
  visual-md/scripts/package.json
  ```

  The preservation test must pre-create `.claude/skills/bob-identify/SKILL.md` with a sentinel, run normal init without `--force`, assert the sentinel is unchanged, and assert `.agents/skills/bob-identify/SKILL.md` contains the embedded template.

- [ ] **Step 3: Run the new tests and confirm the Codex tree is missing.**

  Run:

  ```bash
  cargo test --locked asset_registry_has_identical_claude_and_codex_skill_entries -- --exact
  cargo test --locked init_installs_byte_identical_dual_skill_trees -- --exact
  ```

  Expected: both fail because no `.agents/skills` destinations are registered or installed.

- [ ] **Step 4: Split the skill reporting categories without changing ownership semantics.**

  In `src/assets.rs`, replace `Category::Skill` with:

  ```rust
  #[derive(Clone, Copy, PartialEq, Eq)]
  pub enum Category {
      ClaudeSkill,
      CodexSkill,
      HarnessDoc,
      ArchUnit,
      SharedJava,
  }
  ```

  Add `Category::is_skill()` using `matches!(self, Self::ClaudeSkill | Self::CodexSkill)`. Return `Installing Claude Code skills...` / `Claude Code skills` and `Installing Codex skills...` / `Codex skills` from the header methods. Keep Java-skeleton behavior unchanged. Update existing policy tests to call `category.is_skill()` rather than matching the removed variant.

- [ ] **Step 5: Register every Codex mirror from the same template source.**

  Keep all existing `.claude/skills` entries, changing only their category to `ClaudeSkill`. Immediately after the complete Claude block, add a `.agents/skills` entry for each of the 22 tails above, categorized as `CodexSkill`, using the exact same `include_str!`, `included_in_minimal: true`, and `upgrade_safe: true` values. Do not create a second template directory.

- [ ] **Step 6: Make init output teach both invocation forms.**

  In `src/commands/init.rs`, keep the existing per-file skip/force logic. Update `print_next_steps` so both minimal and full output show:

  ```text
  Claude Code: /bob-survey ...   /bob-model ...
  Codex:      $bob-survey ...   $bob-model ...
  ```

  Keep the statement that survey is optional and model is mandatory.

- [ ] **Step 7: Run the focused tests and the existing init suite.**

  Run:

  ```bash
  cargo test --locked asset_registry_has_identical_claude_and_codex_skill_entries -- --exact
  cargo test --locked init_installs_byte_identical_dual_skill_trees -- --exact
  cargo test --locked minimal_init_installs_all_nine_skills_for_both_hosts_only -- --exact
  cargo test --locked init_without_force_preserves_existing_claude_skill_and_adds_codex -- --exact
  cargo test --locked init_
  ```

  Expected: all pass; the existing Claude assertions remain unchanged and green.

- [ ] **Step 8: Commit the registry and init slice.**

  ```bash
  git add src/assets.rs src/commands/init.rs tests/integration.rs
  git commit -m "feat: install skills for Codex and Claude Code"
  ```

## Task 2: Make legacy upgrade safe, host-complete, and permission-correct

**Files:**

- Modify: `src/lib.rs`
- Modify: `src/commands/init.rs`
- Modify: `src/commands/upgrade.rs`
- Modify: `tests/integration.rs`

- [ ] **Step 1: Write a legacy Claude-only fixture helper.**

  Initialize a temporary project normally, snapshot the `.claude/skills` file bytes and Unix execute bits, then remove only `.agents/skills`. Use this helper in the migration tests; never remove `.claude` in the fixture setup.

- [ ] **Step 2: Add the failing upgrade tests.**

  Add:

  - `upgrade_dry_run_from_claude_only_writes_nothing`
  - `upgrade_from_claude_only_adds_codex_without_touching_claude`
  - `upgrade_backs_up_stale_files_for_both_skill_hosts`
  - `upgrade_missing_shell_scripts_are_executable_for_both_hosts`
  - `init_and_upgrade_reject_wrong_kind_destinations_before_writing`
  - Unix: `init_force_and_upgrade_refuse_symlinked_destinations_without_following`

  Required assertions:

  - dry-run leaves `.agents/skills` absent and the Claude snapshot unchanged;
  - real upgrade restores all 22 Codex assets, leaves Claude bytes and modes identical, and creates no `.run-bob-backup` when every change was `MISSING`;
  - independently stale `.claude/skills/bob-identify/SKILL.md` and `.agents/skills/bob-identify/SKILL.md` are both backed up under the same timestamp with their distinct sentinel bytes before being restored;
  - removing both hosts' `bob-model/scripts/start-server.sh` and `visual-md/scripts/start-server.sh`, then upgrading, produces executable files on Unix.
  - a directory occupying a managed `SKILL.md`, or a file occupying a required parent directory, causes an exact-path error before any other managed destination is changed;
  - a regular and dangling symlink anywhere below either managed root is treated as a conflict by forced init and upgrade, the external target remains byte-identical, and no missing Codex files are installed after the failed preflight.

- [ ] **Step 3: Run the tests and observe the existing permission bug.**

  ```bash
  cargo test --locked upgrade_dry_run_from_claude_only_writes_nothing -- --exact
  cargo test --locked upgrade_from_claude_only_adds_codex_without_touching_claude -- --exact
  cargo test --locked upgrade_missing_shell_scripts_are_executable_for_both_hosts -- --exact
  ```

  Expected: discovery/install behavior becomes available from Task 1, but executable tests fail because `upgrade` currently writes `.sh` files as mode `0644`; conflict tests fail because writes currently follow or misclassify managed symlink/wrong-kind paths.

- [ ] **Step 4: Add a shared managed-path preflight and executable-bit helper.**

  In `src/lib.rs`, add an `ExpectedPathKind::{File, Directory}` inspector that walks every existing component below the already-canonical target with `symlink_metadata`. It must return `Missing`/`Present` only for safe paths and return an error containing the exact relative conflict path for every symlink, dangling symlink, ancestor file, final directory-instead-of-file, or file-instead-of-directory. It must never canonicalize through and follow a managed symlink.

  Before writing anything, init preflights every applicable asset, working directory, and `.gitignore` destination; upgrade preflights every applicable upgrade-safe asset and `.gitignore`. A conflict aborts the command without modifying any managed file. This is deliberately fail-safe: an existing Claude symlink is left untouched and usable by Claude Code, but run-bob refuses to overwrite through it.

  Also move `set_executable_if_shell` from `src/commands/init.rs` to `src/lib.rs` as `pub(crate) fn set_executable_if_shell(path: &Path) -> Result<()>`, retaining the current Unix `mode | 0o111` behavior and the no-op non-Unix branch. Call `crate::set_executable_if_shell(path)?` from init after every successful write.

- [ ] **Step 5: Apply permissions after both upgrade write paths.**

  In `src/commands/upgrade.rs`, call the shared helper immediately after writing every `OUTDATED` destination and every `MISSING` destination. Do not chmod backups and do not touch `UP_TO_DATE` files.

- [ ] **Step 6: Run the upgrade regression group.**

  ```bash
  cargo test --locked upgrade_
  ```

  Expected: all legacy, dry-run, backup, content-replacement, and executable-bit tests pass.

- [ ] **Step 7: Commit the upgrade slice.**

  ```bash
  git add src/lib.rs src/commands/init.rs src/commands/upgrade.rs tests/integration.rs
  git commit -m "fix: migrate Codex skills safely during upgrade"
  ```

## Task 3: Make status require both hosts and return a failing exit code

**Files:**

- Modify: `src/commands/status.rs`
- Modify: `tests/integration.rs`

- [ ] **Step 1: Add failing status process tests.**

  Add:

  - `status_succeeds_for_complete_dual_host_harness`
  - `status_fails_when_claude_skill_is_missing`
  - `status_fails_when_codex_skill_is_missing`
  - `status_reports_wrong_kind_and_symlink_conflicts_at_exact_paths`

  Initialize a full fixture, delete one host's `bob-spec/SKILL.md`, invoke the binary with `.output()`, and assert:

  - process status is non-zero;
  - stdout includes the exact missing relative path and the correct `Claude Code skills` or `Codex skills` heading;
  - the message recommends `run-bob upgrade` for an existing harness and mentions `run-bob init` for a new target.

  For wrong-kind coverage, replace one `SKILL.md` with a directory and replace one required working directory with a file. On Unix, also replace a managed file with a dangling symlink. Assert status does not follow the link, prints the exact conflicting path and kind, and exits non-zero.

  Update `status_flags_missing_after_minimal_init` to assert non-zero rather than checking stdout alone.

- [ ] **Step 2: Run the failing tests.**

  ```bash
  cargo test --locked status_fails_when_claude_skill_is_missing -- --exact
  cargo test --locked status_fails_when_codex_skill_is_missing -- --exact
  ```

  Expected: output reports the missing files, but both processes currently exit successfully.

- [ ] **Step 3: Return an error for an incomplete harness.**

  In `src/commands/status.rs`, use the shared no-follow inspector from Task 2 for files and working directories, preserving all per-path reporting and distinguishing missing from wrong-kind/symlink conflicts. After printing the incomplete summary, return `anyhow::bail!` with this actionable meaning:

  ```text
  harness is incomplete; use `run-bob upgrade` for an existing harness or `run-bob init` for a new target
  ```

  Return `Ok(())` only from the complete branch.

- [ ] **Step 4: Run all status tests.**

  ```bash
  cargo test --locked status_
  ```

  Expected: complete dual-host fixtures return zero; either missing host returns non-zero; Java-skeleton skipping remains unchanged.

- [ ] **Step 5: Commit the status slice.**

  ```bash
  git add src/commands/status.rs tests/integration.rs
  git commit -m "fix: fail status when either skill host is incomplete"
  ```

## Task 4: Make all generated skill metadata and invocations Codex-compatible

**Files:**

- Modify: `src/templates/skills/bob-identify.md`
- Modify: `src/templates/skills/bob-onion.md`
- Modify: `src/templates/skills/bob-spec.md`
- Modify: `src/templates/skills/bob-survey.md`
- Modify: `src/templates/skills/bob-stories.md`
- Modify: `src/templates/skills/bob-nfr.md`
- Modify: `src/templates/skills/bob-compliance.md`
- Modify: `src/templates/skills/bob-model.md`
- Modify: `src/templates/skills/visual-md.md`
- Modify: `tests/integration.rs`

- [ ] **Step 1: Add a no-dependency frontmatter parser for contract tests.**

  In `tests/integration.rs`, extract the text between the first two `---` delimiters. Treat non-indented `key:` lines as top-level keys and decode the existing `description: |` block by removing its two-space indentation and joining lines with newlines. Do not add a YAML crate.

- [ ] **Step 2: Add the failing metadata and dual-invocation tests.**

  Add:

  - `generated_skill_metadata_is_codex_compatible`
  - `generated_skills_document_both_host_invocations`

  For each of the nine generated Codex `SKILL.md` files, assert:

  - top-level keys are exactly `name` and `description`;
  - name equals the directory name, uses only lowercase ASCII letters, digits, and single hyphens, and is at most 64 characters;
  - decoded description is non-empty, at most 1024 Unicode characters, and contains neither `<` nor `>`;
  - description contains both `/name` and `$name`;
  - body is non-empty and documents the current-host output rule.

  For both generated roots, assert each file still contains `/name`, now also contains `$name`, and remains byte-identical to the opposite host.

- [ ] **Step 3: Run the tests and capture both current incompatibilities.**

  ```bash
  cargo test --locked generated_skill_metadata_is_codex_compatible -- --exact
  cargo test --locked generated_skills_document_both_host_invocations -- --exact
  ```

  Expected: failures report absent `$...` forms, angle-bracket arguments in descriptions, and a `bob-model` description longer than Codex's 1024-character limit.

- [ ] **Step 4: Rewrite only the nine frontmatter descriptions.**

  Use this exact contract in every description:

  - put natural-language trigger terms first;
  - name the Claude form `/skill-name` and Codex form `$skill-name` in the first three lines;
  - refer to arguments as `参数`, `文档路径`, `story 路径`, or square-bracket notation, never angle brackets;
  - retain each skill's purpose, stage, mandatory/optional status, and primary output;
  - keep `bob-model` between roughly 300 and 700 decoded characters so it is clearly below 1024.

  Do not alter the long workflow bodies in this step.

- [ ] **Step 5: Add the standard dual-host body contract to every skill.**

  Immediately after each title/trigger section, add equivalent text with these four statements:

  ```markdown
  ## 双宿主调用约定

  - Claude Code 使用 `/skill-name`；Codex 使用 `$skill-name`，参数语义完全相同。
  - 本文保留 slash 形式以保护 Claude Code 兼容性。
  - 向用户给出下一步命令时，使用当前宿主的调用形式。
  - 不从一个宿主的 skill 根回退到另一个宿主。
  ```

  Keep every existing slash example. Add a paired `$skill-name` command block in each trigger section; do not mechanically duplicate every downstream example in the rest of the file.

- [ ] **Step 6: Run metadata, parity, and existing skill-contract tests.**

  ```bash
  cargo test --locked generated_skill_metadata_is_codex_compatible -- --exact
  cargo test --locked generated_skills_document_both_host_invocations -- --exact
  cargo test --locked bob_
  ```

  Expected: new Codex contracts pass and all pre-existing slash-based Bob contracts remain green.

- [ ] **Step 7: Commit the metadata and invocation slice.**

  ```bash
  git add src/templates/skills tests/integration.rs
  git commit -m "feat(skills): add Codex invocation contracts"
  ```

## Task 5: Remove host-root hard-coding and preserve both Superpowers handoffs

**Files:**

- Modify: `src/templates/skills/bob-model.md`
- Modify: `src/templates/skills/visual-md.md`
- Modify: `src/templates/skills/bob-survey.md`
- Modify: `src/templates/skills/bob-compliance.md`
- Modify: `src/templates/skills/bob-nfr.md`
- Modify: `src/templates/skills/bob-onion.md`
- Modify: `src/templates/skills/bob-spec.md`
- Modify: `tests/integration.rs`

- [ ] **Step 1: Add focused failing compatibility tests.**

  Add:

  - `interactive_skills_resolve_scripts_from_active_skill_dir`
  - `survey_accepts_either_managed_skill_root_without_double_counting`
  - `bob_spec_preserves_claude_and_adds_codex_superpowers_handoffs`
  - `document_skills_define_reading_for_both_hosts`

  Assertions must include:

  - `bob-model` and `visual-md` define `<skill-dir>` as the directory containing the currently loaded `SKILL.md`;
  - both reference their scripts only through `<skill-dir>/scripts`, check existence, report the resolved missing path, and prohibit cross-host fallback;
  - neither contains `.claude/skills/bob-model/scripts`, `.agents/skills/bob-model/scripts`, `.claude/skills/visual-md/scripts`, or `.agents/skills/visual-md/scripts`;
  - survey contains both `.claude/skills/bob-*` and `.agents/skills/bob-*`, plus the meanings “either complete” and “do not double-count”;
  - `bob-spec` contains each pair: `superpowers:brainstorming` / `$brainstorming`, `superpowers:writing-plans` / `$writing-plans`, `superpowers:executing-plans` / `$executing-plans`, and `superpowers:finishing-a-development-branch` / `$finishing-a-development-branch`;
  - `bob-model` and `bob-compliance` preserve Claude Code's `Read`/`pages` branch and add a Codex PDF/document-capability branch.

- [ ] **Step 2: Run the focused tests and observe the hard-coded Claude paths.**

  ```bash
  cargo test --locked interactive_skills_resolve_scripts_from_active_skill_dir -- --exact
  cargo test --locked survey_accepts_either_managed_skill_root_without_double_counting -- --exact
  cargo test --locked bob_spec_preserves_claude_and_adds_codex_superpowers_handoffs -- --exact
  cargo test --locked document_skills_define_reading_for_both_hosts -- --exact
  ```

  Expected: all fail on the current Claude-only wording or paths.

- [ ] **Step 3: Make `bob-model` resolve its own script.**

  Define `<skill-dir>` in the body as the absolute directory containing the active `SKILL.md`. Replace the server launch with:

  ```bash
  SCRIPT="<skill-dir>/scripts/start-server.sh"
  if [ ! -f "$SCRIPT" ]; then
    echo "bob-model start script not found: $SCRIPT" >&2
    exit 1
  fi
  "$SCRIPT" --project-dir "<project-root>"
  ```

  State explicitly that the current host supplies `<skill-dir>`, the resolved path is reported on failure, and fallback to the other host root is forbidden. Update the existing test that currently expects `.claude/skills/bob-model/scripts/start-server.sh` to expect `<skill-dir>/scripts/start-server.sh`.

- [ ] **Step 4: Make every `visual-md` script reference skill-relative.**

  Define the same `<skill-dir>` contract and replace all five hard-coded sites:

  ```text
  node "<skill-dir>/scripts/slugify.cjs"
  SCRIPTS_DIR="<skill-dir>/scripts"
  "<skill-dir>/scripts/start-server.sh"
  node "<skill-dir>/scripts/md2html.cjs"
  "<skill-dir>/scripts/stop-server.sh"
  ```

  Add an existence check before launch and preserve the existing session/output behavior.

- [ ] **Step 5: Make survey and document ingestion host-neutral.**

  In `bob-survey`, define a mature Bob installation as a complete set under either root; a dual installation counts once. In `bob-model` and `bob-compliance`, retain the Claude Code `Read` tool with paged reads, add the current Codex PDF/document reading capability, and stop with a clear error if the selected host cannot read the supplied document. Replace generic executor references to “Claude” with “current host agent” while retaining explicit Claude Code branches.

  In `bob-onion`, keep `CLAUDE.md R8` but explain that both hosts read it explicitly as the Bob project rule document; do not claim Codex auto-loads it. In `bob-nfr`, describe Claude Superpowers TDD and the Codex skill equivalents additively.

- [ ] **Step 6: Add the Codex Superpowers mapping to `bob-spec`.**

  Preserve the existing Claude sequence and add this table near the handoff section:

  | Stage | Claude Code | Codex |
  |---|---|---|
  | Brainstorm | `superpowers:brainstorming` | `$brainstorming` |
  | Plan | `superpowers:writing-plans` | `$writing-plans` |
  | Execute | `superpowers:executing-plans` | `$executing-plans` |
  | Finish branch | `superpowers:finishing-a-development-branch` | `$finishing-a-development-branch` |

  Keep the instruction to write confirmed stack decisions to `CLAUDE.md ## 技术栈约定`, noting that either host maintains this explicit Bob document.

- [ ] **Step 7: Run all focused and existing skill tests.**

  ```bash
  cargo test --locked interactive_skills_resolve_scripts_from_active_skill_dir -- --exact
  cargo test --locked survey_accepts_either_managed_skill_root_without_double_counting -- --exact
  cargo test --locked bob_spec_preserves_claude_and_adds_codex_superpowers_handoffs -- --exact
  cargo test --locked document_skills_define_reading_for_both_hosts -- --exact
  cargo test --locked bob_model
  cargo test --locked bob_spec
  ```

  Expected: both host branches are present, interactive skills are self-contained, and all original Claude contracts remain green.

- [ ] **Step 8: Commit the host-neutral workflow slice.**

  ```bash
  git add src/templates/skills tests/integration.rs
  git commit -m "fix(skills): resolve workflows from the active host"
  ```

## Task 6: Add and test deterministic POSIX Rust bootstrap

**Files:**

- Create: `scripts/bootstrap-rust.sh`
- Create: `tests/bootstrap_rust.rs`
- Modify: `Cargo.toml`
- Modify: `Cargo.lock`

- [ ] **Step 1: Declare and prove the source-build MSRV.**

  Add `rust-version = "1.75"` in `[package]`. The bootstrap helper reads this exact field; do not keep a second hard-coded minimum. Change the existing lockfile header from version 4 to version 3: the official [Cargo changelog](https://doc.rust-lang.org/cargo/CHANGELOG.html) states that lockfile v4 requires Rust/Cargo 1.78+, while v3 is compatible with 1.75. This lockfile contains registry dependencies only, so no v4-only Git URL encoding is needed.

  ```bash
  rustup toolchain install 1.75 --profile minimal
  cargo +1.75 test --locked binary_prints_version -- --exact
  ```

  Expected: Cargo 1.75 accepts the v3 lockfile and the baseline test passes. If a dependency rejects 1.75, pin that dependency compatibly instead of weakening the declared MSRV.

- [ ] **Step 2: Build isolated command-mocking helpers in `tests/bootstrap_rust.rs`.**

  Under `#[cfg(unix)]`, create temporary `mock-bin`, home, Cargo home, Rustup home, and command log paths. Put executable mocks first on `PATH` and retain `/usr/bin:/bin` only for basic shell utilities. Every case installs a mock `curl`; non-download cases use a fail-and-log mock so no accidental branch can reach the network. `RUN_BOB_TEST_LOG` is consumed only by mocks; production code gets no test mode or endpoint override.

- [ ] **Step 3: Add the supported-toolchain failing test.**

  Add `bootstrap_rust_posix_uses_supported_toolchain_and_forwards_cargo_args`. Mock `rustc 1.75.0` and cargo, omit rustup, then invoke:

  ```text
  scripts/bootstrap-rust.sh --run-cargo check --locked --all-targets
  ```

  Assert cargo receives exactly `check --locked --all-targets`, and no curl/rustup call occurs.

- [ ] **Step 4: Run the test and confirm the helper is absent.**

  ```bash
  cargo test --locked --test bootstrap_rust bootstrap_rust_posix_uses_supported_toolchain_and_forwards_cargo_args -- --exact
  ```

  Expected: failure because `scripts/bootstrap-rust.sh` does not exist.

- [ ] **Step 5: Implement the public bootstrap contract and direct path.**

  Create a POSIX `sh` script with `set -eu`, locate the repository from the script directory, verify its package name, parse the complete semantic `rust-version`, and compare numeric major/minor/patch values. Its only public forms are:

  ```text
  ./scripts/bootstrap-rust.sh
  ./scripts/bootstrap-rust.sh --run-cargo <cargo arguments>
  ```

  No arguments ensures and re-verifies `rustc`/`cargo`. `--run-cargo` does the same preparation and then executes the selected direct cargo or `rustup run stable cargo`, forwarding every remaining argument unchanged. Unknown forms and an empty `--run-cargo` fail with usage. The helper does not own build, test, binary installation, or `run-bob` verification.

- [ ] **Step 6: Add missing and ownership-sensitive tests.**

  Add:

  - `bootstrap_rust_posix_installs_a_missing_toolchain`
  - `bootstrap_rust_posix_reports_official_installer_failure`
  - `bootstrap_rust_posix_rejects_incomplete_post_install_toolchain`
  - `bootstrap_rust_posix_uses_stable_for_old_active_rustup_toolchain`
  - `bootstrap_rust_posix_stops_for_old_non_rustup_toolchain`
  - `bootstrap_rust_posix_stops_for_old_system_rust_with_unrelated_rustup`

  The download mock verifies `--proto =https`, `--tlsv1.2`, the fixed `https://sh.rustup.rs` URL, and its `-o` destination, then writes a fake installer there. Fake rustup records `toolchain install stable --profile minimal`, supports `which rustc`, and implements `run stable rustc` / `run stable cargo`. No case may invoke `rustup default`.

- [ ] **Step 7: Run the red decision tests.**

  ```bash
  cargo test --locked --test bootstrap_rust bootstrap_rust_posix_ -- --nocapture
  ```

  Expected: the direct path passes; missing/old/error cases fail until their branches exist.

- [ ] **Step 8: Implement safe rustup selection.**

  Use the fixed production URL and these decisions:

  1. both tools present and sufficient: use them unchanged;
  2. tools completely absent and rustup present: install stable minimal and use `rustup run stable`;
  3. all three absent: download to a `mktemp -d` directory, run that exact file with `-y --profile minimal --default-toolchain stable --no-modify-path`, trap-clean the validated temp directory, then verify;
  4. tools present but old: use rustup only when `rustc --print sysroot` and `rustup which rustc` prove the active compiler is rustup-owned; otherwise stop without replacement, even if unrelated rustup is on `PATH`;
  5. partial non-rustup installation: stop without replacement;
  6. after any bootstrap: re-check complete versions before returning or running cargo.

  Do not write shell startup files. Make the script executable in Git.

- [ ] **Step 9: Run all POSIX bootstrap tests.**

  ```bash
  cargo test --locked --test bootstrap_rust bootstrap_rust_posix_ -- --nocapture
  ```

  Expected: all pass without network or real toolchain changes.

- [ ] **Step 10: Commit the POSIX bootstrap.**

  ```bash
  git add Cargo.toml Cargo.lock scripts/bootstrap-rust.sh tests/bootstrap_rust.rs
  git commit -m "feat(install): bootstrap Rust safely on POSIX"
  ```

## Task 7: Add equivalent PowerShell bootstrap and native CI

**Files:**

- Create: `scripts/bootstrap-rust.ps1`
- Create: `tests/bootstrap-rust.Tests.ps1`
- Create: `.github/workflows/ci.yml`
- Modify: `tests/bootstrap_rust.rs`

- [ ] **Step 1: Add static and native PowerShell tests.**

  Add Rust test `bootstrap_rust_powershell_has_safe_equivalent_contract`. It reads the script on every OS and asserts fixed x64/arm64 official URLs; stable/minimal/no-modify-path flags; `RunCargo` argument forwarding; post-bootstrap version verification; and absence of `rustup default`, `SetEnvironmentVariable`, hard-coded build/test/install stages, or endpoint override variables.

  Create a Pester 5 suite that dot-sources production code and mocks command discovery, external execution, and `Invoke-WebRequest`. Cover supported, absent, installer failure, incomplete post-install, old active-rustup, old system Rust with unrelated rustup, X64/Arm64 mapping, unsupported architecture, exact cargo arguments, and surfaced Visual C++/installer errors. Use `$TestDrive`; never access the network.

- [ ] **Step 2: Run the red tests.**

  ```bash
  cargo test --locked --test bootstrap_rust bootstrap_rust_powershell_has_safe_equivalent_contract -- --exact
  ```

  When `pwsh` is available:

  ```bash
  pwsh -NoProfile -Command "Invoke-Pester ./tests/bootstrap-rust.Tests.ps1 -CI -Output Detailed"
  ```

  Expected: failure because the PowerShell helper and its mockable boundaries are absent.

- [ ] **Step 3: Implement `scripts/bootstrap-rust.ps1`.**

  Match the POSIX no-argument and `-RunCargo <string[]>` contracts. Use `[version]` comparison, fixed download constants, and small mockable boundary functions; guard the main call so dot-sourcing only defines functions. Prove active rustup ownership before replacing an old compiler. Map architectures exactly:

  ```text
  X64   -> https://static.rust-lang.org/rustup/dist/x86_64-pc-windows-msvc/rustup-init.exe
  Arm64 -> https://static.rust-lang.org/rustup/dist/aarch64-pc-windows-msvc/rustup-init.exe
  ```

  Reject other architectures before download. Use a GUID temp file, run `-y --profile minimal --default-toolchain stable --no-modify-path`, clean it in `finally`, and surface rustup/Visual C++ failures. For old active-rustup only, install/use stable without changing the pre-existing default.

- [ ] **Step 4: Run static tests locally and behavioral tests on Windows.**

  ```bash
  cargo test --locked --test bootstrap_rust bootstrap_rust_powershell_ -- --nocapture
  ```

  The task remains incomplete until the Pester suite passes natively on Windows.

- [ ] **Step 5: Add native cross-platform CI.**

  Add `.github/workflows/ci.yml` for pushes and pull requests with a stable `cargo test --locked` matrix on Ubuntu/macOS/Windows, Ubuntu fmt/clippy, an Ubuntu Rust 1.75 locked-test job, and a Windows job that installs Pester 5 for the current user and executes the suite. Use `actions/checkout@v4` and `dtolnay/rust-toolchain@stable`; configure the MSRV job with toolchain `1.75`. Do not modify the tag-only release workflow.

- [ ] **Step 6: Commit PowerShell bootstrap and CI.**

  ```bash
  git add scripts/bootstrap-rust.ps1 tests/bootstrap-rust.Tests.ps1 tests/bootstrap_rust.rs .github/workflows/ci.yml
  git commit -m "feat(install): bootstrap Rust safely on Windows"
  ```

## Task 8: Publish one contributor install skill to both hosts

**Files:**

- Modify: `.claude/skills/install/SKILL.md`
- Create: `.agents/skills/install/SKILL.md`
- Modify: `tests/integration.rs`

- [ ] **Step 1: Add the failing repository-skill parity test.**

  Add `repository_install_skills_are_identical_and_dual_host`. Read both files at runtime using `env!("CARGO_MANIFEST_DIR")` so a missing file is a test failure, not a compile failure. Assert:

  - the Claude file still exists at its original path;
  - the Codex file exists and bytes are identical;
  - frontmatter contains natural-language triggers, `/install`, and `$install`, contains no angle brackets, and is at most 1024 decoded characters;
  - body names both source helper paths;
  - body contains all three locked Cargo stages and does not hard-code a test count;
  - body contains neither the Claude-only `! curl` prefix nor `rustup default`.

- [ ] **Step 2: Run the test and confirm the Codex contributor skill is missing.**

  ```bash
  cargo test --locked repository_install_skills_are_identical_and_dual_host -- --exact
  ```

  Expected: failure because `.agents/skills/install/SKILL.md` does not exist and the Claude copy still asks the user to install Rust manually.

- [ ] **Step 3: Rewrite the install workflow around the tested helpers.**

  Keep repository-root validation and safe Git behavior:

  - clean worktree plus origin: `git pull --ff-only` on the checked-out branch;
  - dirty/diverged worktree: stop and ask; never stash, reset, or checkout automatically;
  - no origin: skip sync and use current source.

  Then run each Cargo gate through the selected bootstrap helper so an old rustup-managed system does not depend on parent-shell PATH changes:

  ```text
  POSIX:
    ./scripts/bootstrap-rust.sh --run-cargo build --release --locked
    ./scripts/bootstrap-rust.sh --run-cargo test --locked
    ./scripts/bootstrap-rust.sh --run-cargo install --locked --path .

  Windows:
    & .\scripts\bootstrap-rust.ps1 -RunCargo @('build','--release','--locked')
    & .\scripts\bootstrap-rust.ps1 -RunCargo @('test','--locked')
    & .\scripts\bootstrap-rust.ps1 -RunCargo @('install','--locked','--path','.')
  ```

  Explain that invoking `/install` or `$install` authorizes automatic official rustup installation only when Rust is absent, while platform prompts remain visible. State that existing sufficient Rust is unchanged, old rustup is used locally without a default switch, and old non-rustup Rust is not replaced.

  After install, verify the absolute binary under `CARGO_INSTALL_ROOT`, else `CARGO_HOME`, else the user's `.cargo/bin`, using `--version` and `--help`; compare the version with `Cargo.toml`. Remove manual rustup commands, the `!` shell prefix, and stale “15 tests” wording.

- [ ] **Step 4: Materialize the byte-identical Codex copy.**

  Add `.agents/skills/install/SKILL.md` with exactly the same bytes as the revised Claude file. Do not symlink it and do not make one file refer to the other.

- [ ] **Step 5: Run the parity test and both Codex quick validations.**

  ```bash
  cargo test --locked repository_install_skills_are_identical_and_dual_host -- --exact
  CODEX_SKILL_VALIDATOR="${CODEX_HOME:-$HOME/.codex}/skills/.system/skill-creator/scripts/quick_validate.py"
  test -f "$CODEX_SKILL_VALIDATOR"
  python3 "$CODEX_SKILL_VALIDATOR" .claude/skills/install
  python3 "$CODEX_SKILL_VALIDATOR" .agents/skills/install
  ```

  Expected: all pass and both validators print `Skill is valid!`.

- [ ] **Step 6: Commit the contributor skill.**

  ```bash
  git add .claude/skills/install/SKILL.md .agents/skills/install/SKILL.md tests/integration.rs
  git commit -m "feat(skills): expose source installation to both hosts"
  ```

## Task 9: Update current documentation, package metadata, and CLI guidance

**Files:**

- Modify: `README.md`
- Modify: `src/templates/root/README-RUN-BOB.md`
- Modify: `src/main.rs`
- Modify: `Cargo.toml`
- Modify: `tests/integration.rs`
- Modify: `docs/superpowers/specs/2026-08-16-codex-claude-dual-skills-design.md`

- [ ] **Step 1: Add user-facing documentation contract tests.**

  Add:

  - `cli_help_names_codex_and_claude_code`
  - `generated_readme_explains_both_skill_hosts`

  Assert CLI help and generated `README-RUN-BOB.md` contain `Claude Code`, `Codex`, `.claude/skills`, `.agents/skills`, `/bob-model`, `$bob-model`, `8 Bob`, and `visual-md`. Preserve existing tests for `CLAUDE.md` and architecture content.

- [ ] **Step 2: Run the tests and observe Claude-only wording.**

  ```bash
  cargo test --locked cli_help_names_codex_and_claude_code -- --exact
  cargo test --locked generated_readme_explains_both_skill_hosts -- --exact
  ```

  Expected: both fail on current Claude-only text and the stale three/eight-skill descriptions.

- [ ] **Step 3: Update CLI and package metadata.**

  Change the binary module comment, Clap `about`, and Cargo package description to “Claude Code and Codex projects”. Keep `rust-version = "1.75"` from Task 6. Update init next steps only if Task 1 did not already cover all full/minimal branches.

- [ ] **Step 4: Rewrite current README installation and migration guidance.**

  In `README.md`:

  - describe eight `bob-*` skills plus the auxiliary `visual-md` skill;
  - show both real directory trees and state they are byte-identical outputs from one embedded template source;
  - show `/skill` for Claude Code and `$skill` for Codex;
  - retain every existing slash workflow and add a compact Codex mapping rather than replacing it;
  - distinguish prebuilt install, which needs no Rust, from source install, whose contributor skill/helpers bootstrap missing Rust;
  - document legacy migration as `run-bob upgrade --dry-run`, `run-bob upgrade`, then `run-bob status`;
  - state that current Claude customizations follow the existing backup-before-overwrite contract and user-owned `CLAUDE.md` / `ARCHITECTURE.md` remain untouched;
  - link both `.claude/skills/install/SKILL.md` and `.agents/skills/install/SKILL.md`.

  Do not add uninstall instructions that recursively remove either shared skills root.

- [ ] **Step 5: Bring the generated project README up to the same host contract.**

  In `src/templates/root/README-RUN-BOB.md`, replace the stale “three Claude Code skills” and Claude-only tree with the complete nine-skill dual tree. Explain that `CLAUDE.md` remains the Bob project rule document and Codex workflows consult it explicitly; do not claim Codex automatically loads it and do not introduce `AGENTS.md`. Add the same Superpowers mapping table used in `bob-spec`.

- [ ] **Step 6: Finalize the approved design status without rewriting history.**

  Change the current design document status to `Approved`. Do not edit older dated specs or plans; the current design already states that it supersedes their Claude-only installation assumptions.

- [ ] **Step 7: Run documentation and CLI tests.**

  ```bash
  cargo test --locked cli_help_names_codex_and_claude_code -- --exact
  cargo test --locked generated_readme_explains_both_skill_hosts -- --exact
  cargo test --locked init_creates_readme_run_bob_with_3_modes -- --exact
  cargo test --locked binary_prints_version -- --exact
  ```

  Expected: all pass, with old Claude document contracts intact.

- [ ] **Step 8: Commit current documentation.**

  ```bash
  git add README.md src/templates/root/README-RUN-BOB.md src/main.rs Cargo.toml tests/integration.rs docs/superpowers/specs/2026-08-16-codex-claude-dual-skills-design.md
  git commit -m "docs: explain Codex and Claude Code workflows"
  ```

## Task 10: Run full compatibility verification and a generated-tree forward test

**Files:**

- Modify only if verification exposes a defect in files already listed above.

- [ ] **Step 1: Format and lint the complete implementation.**

  ```bash
  cargo fmt --check
  cargo clippy --locked --all-targets -- -D warnings
  git diff --check
  ```

  Expected: zero warnings, errors, or whitespace failures.

- [ ] **Step 2: Run the entire locked test suite.**

  ```bash
  cargo test --locked
  cargo +1.75 test --locked
  ```

  Expected: every unit and integration test passes on stable and the declared MSRV. On non-Windows, report the PowerShell process tests as cfg-skipped while the cross-platform PowerShell contract test passes.

- [ ] **Step 3: Generate a fresh minimal dual-host fixture.**

  ```bash
  RUN_BOB_VERIFY_DIR="$(mktemp -d)"
  cargo run --locked -- init --minimal --no-gitignore --dir "$RUN_BOB_VERIFY_DIR"
  diff -ru "$RUN_BOB_VERIFY_DIR/.claude/skills" "$RUN_BOB_VERIFY_DIR/.agents/skills"
  ```

  Expected: init reports separate Claude Code and Codex groups; recursive diff has no output.

- [ ] **Step 4: Run Codex metadata validation against every generated skill and both contributor copies.**

  ```bash
  CODEX_SKILL_VALIDATOR="${CODEX_HOME:-$HOME/.codex}/skills/.system/skill-creator/scripts/quick_validate.py"
  test -f "$CODEX_SKILL_VALIDATOR"
  for skill_dir in "$RUN_BOB_VERIFY_DIR"/.agents/skills/* .claude/skills/install .agents/skills/install; do
    python3 "$CODEX_SKILL_VALIDATOR" "$skill_dir"
  done
  ```

  Expected: eleven `Skill is valid!` results: nine generated Codex skills plus the two contributor paths. Count and report all eleven explicitly.

- [ ] **Step 5: Forward-test a generated Codex skill, then inspect Claude compatibility.**

  Confirm in the fixture:

  ```bash
  test -x "$RUN_BOB_VERIFY_DIR/.agents/skills/bob-model/scripts/start-server.sh"
  test -x "$RUN_BOB_VERIFY_DIR/.agents/skills/visual-md/scripts/start-server.sh"
  test -x "$RUN_BOB_VERIFY_DIR/.claude/skills/bob-model/scripts/start-server.sh"
  test -x "$RUN_BOB_VERIFY_DIR/.claude/skills/visual-md/scripts/start-server.sh"
  rg -n '\$bob-model|<skill-dir>/scripts/start-server.sh' "$RUN_BOB_VERIFY_DIR/.agents/skills/bob-model/SKILL.md"
  rg -n '/bob-model|superpowers:' "$RUN_BOB_VERIFY_DIR/.claude/skills/bob-model/SKILL.md" "$RUN_BOB_VERIFY_DIR/.claude/skills/bob-spec/SKILL.md"
  ```

  Then start an isolated, read-only Codex process from the generated target and explicitly invoke the generated repository skill:

  ```bash
  codex exec --ephemeral --sandbox read-only --skip-git-repo-check \
    --cd "$RUN_BOB_VERIFY_DIR" \
    --output-last-message "$RUN_BOB_VERIFY_DIR/codex-forward-result.txt" \
    'Use $bob-survey. Do not run the survey and do not write files. Read the loaded skill invocation contract, then reply with exactly RUN_BOB_CODEX_SKILL_OK.'
  rg -x 'RUN_BOB_CODEX_SKILL_OK' "$RUN_BOB_VERIFY_DIR/codex-forward-result.txt"
  ```

  Expected: all four scripts are executable; the nested Codex process discovers and loads `$bob-survey`; Codex instructions contain `$bob-model` and a skill-relative script; Claude instructions retain `/bob-model` and Claude Superpowers forms. Authentication/network failure is a genuine blocked acceptance check and must not be relabeled as a passing static forward test.

- [ ] **Step 6: Exercise the legacy migration sequence once end-to-end.**

  Create a second temp fixture, initialize it, move only `.agents` aside as a recoverable Claude-only simulation, and save a Claude checksum manifest:

  ```bash
  RUN_BOB_LEGACY_DIR="$(mktemp -d)"
  cargo run --locked -- init --no-gitignore --dir "$RUN_BOB_LEGACY_DIR"
  find "$RUN_BOB_LEGACY_DIR/.claude/skills" -type f -exec cksum {} \; | sort > "$RUN_BOB_LEGACY_DIR/claude-before.cksum"
  mv "$RUN_BOB_LEGACY_DIR/.agents" "$RUN_BOB_LEGACY_DIR/held-out-agents"
  cargo run --locked -- upgrade --dry-run --dir "$RUN_BOB_LEGACY_DIR"
  test ! -e "$RUN_BOB_LEGACY_DIR/.agents/skills"
  cargo run --locked -- upgrade --dir "$RUN_BOB_LEGACY_DIR"
  cargo run --locked -- status --dir "$RUN_BOB_LEGACY_DIR"
  find "$RUN_BOB_LEGACY_DIR/.claude/skills" -type f -exec cksum {} \; | sort > "$RUN_BOB_LEGACY_DIR/claude-after.cksum"
  diff -u "$RUN_BOB_LEGACY_DIR/claude-before.cksum" "$RUN_BOB_LEGACY_DIR/claude-after.cksum"
  ```

  Expected: dry-run writes nothing; real upgrade adds Codex; status succeeds; the saved Claude checksum manifest remains identical.

- [ ] **Step 7: Review scope and repository state.**

  ```bash
  git status --short
  git log --oneline --decorate -12
  rg -n 'TODO|TBD|\.codex/skills|rustup default' src .claude .agents scripts README.md Cargo.toml
  git diff --exit-code -- install.sh install.ps1
  ```

  Expected: only intentional changes exist; no generated skill uses `.codex/skills`; no install workflow changes a pre-existing global rustup default; no unfinished markers remain; the released-binary installers are byte-unchanged.

- [ ] **Step 8: Commit only verification-driven fixes, if any.**

  If verification required code or documentation corrections, rerun Steps 1–7, inspect `git status --short`, stage only the exact paths corrected, and commit them with message `fix: close dual-host compatibility gaps`.

  If no files changed, do not create an empty commit.
