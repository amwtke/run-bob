# Codex + Claude Code Dual-Skill Support Design

**Date:** 2026-08-16  
**Status:** Approved in conversation; awaiting written-spec review

## 1. Context

`run-bob` currently embeds its skill templates at compile time and installs every
generated skill under `.claude/skills`. This makes the harness directly
discoverable by Claude Code, but not by Codex, whose repository-scoped skill
location is `.agents/skills`.

The migration must make the same workflows available to both hosts without
moving, deleting, renaming, or deprecating the existing Claude Code paths.
Claude Code remains a first-class supported host.

OpenAI's current skill documentation defines `.agents/skills/<name>/SKILL.md`
as the repository location, `$name` as Codex's explicit invocation syntax, and
permits a `SKILL.md` containing `name` and `description` without optional UI
metadata: <https://developers.openai.com/codex/skills>.

## 2. Goals

1. Make `run-bob init` generate real skill files for Claude Code and Codex.
2. Make `run-bob upgrade` migrate existing Claude-only projects safely.
3. Make `run-bob status` verify both hosts and fail when either managed tree is
   incomplete.
4. Preserve all existing Claude Code paths, slash invocations, workflows, and
   user-owned project files.
5. Keep one template source for each generated asset while materializing two
   independent output trees.
6. Make interactive skill scripts run from the invoking skill's own directory.
7. Expose this repository's contributor `install` skill to both hosts.
8. Make the contributor install workflow bootstrap Rust automatically when the
   development toolchain is absent.

## 3. Non-goals

- Do not generate, replace, append to, or otherwise manage `AGENTS.md`.
- Do not remove or rename `CLAUDE.md`.
- Do not convert either skill tree to symlinks, hardlinks, or junctions.
- Do not add an uninstall command.
- Do not redesign the Bob workflow or alter its architecture rules.
- Do not aggressively split the long `bob-model` or `bob-spec` instructions in
  this migration.
- Do not require Rust for users who install or run the released prebuilt
  `run-bob` binary.
- Do not add optional `agents/openai.yaml` metadata in this migration.

## 4. Installed Layout

A normal or minimal initialization installs the same managed skill files to
both roots:

```text
<target>/
├── .claude/skills/
│   ├── bob-survey/
│   ├── bob-model/
│   ├── bob-stories/
│   ├── bob-identify/
│   ├── bob-onion/
│   ├── bob-spec/
│   ├── bob-compliance/
│   ├── bob-nfr/
│   └── visual-md/
└── .agents/skills/
    ├── bob-survey/
    ├── bob-model/
    ├── bob-stories/
    ├── bob-identify/
    ├── bob-onion/
    ├── bob-spec/
    ├── bob-compliance/
    ├── bob-nfr/
    └── visual-md/
```

This is eight Bob workflow skills plus the auxiliary `visual-md` skill. The
`bob-model` and `visual-md` script files are present under both roots.

The repository itself also contains both contributor entry points:

```text
.claude/skills/install/SKILL.md
.agents/skills/install/SKILL.md
```

The generated target trees are byte-identical. The contributor install copies
are also host-neutral and byte-identical.

## 5. Asset Registry

`src/templates/skills` and `src/templates/scripts` remain the only source for
generated skill content. The flat embedded asset registry gains a mirrored
`.agents/skills` destination for every `.claude/skills` asset, and both entries
use the same `include_str!` source.

The implementation may distinguish `ClaudeSkill` and `CodexSkill` categories
for clearer CLI headings. It must not maintain a second template directory.

A registry-parity test enforces the following invariant for every managed
`.claude/skills/<tail>` asset:

- `.agents/skills/<tail>` exists in the registry;
- content is identical;
- minimal-mode inclusion is identical;
- upgrade ownership is identical;
- executable-file treatment is identical.

This test is the guard against a future release adding a skill to one host
only.

## 6. Host-Neutral Skill Contracts

Every generated `SKILL.md` keeps the existing Claude Code syntax and adds the
Codex syntax:

| Action | Claude Code | Codex |
|---|---|---|
| Explicit skill invocation | `/bob-model` | `$bob-model` |
| Superpowers brainstorming | `superpowers:brainstorming` | `$brainstorming` |
| Superpowers planning | `superpowers:writing-plans` | `$writing-plans` |
| Superpowers execution | `superpowers:executing-plans` | `$executing-plans` |
| Branch completion | `superpowers:finishing-a-development-branch` | `$finishing-a-development-branch` |

Descriptions front-load the natural-language triggers and both invocation
forms. They remain valid YAML with only `name` and `description`; in particular,
the `bob-model` description is shortened to satisfy Codex metadata validation
without removing its mandatory-stage semantics.

Host-specific prose is made additive and neutral:

- retain Claude Code instructions and slash examples;
- add Codex equivalents rather than replacing Claude syntax;
- say "current host agent" where the executor is not inherently Claude-only;
- use the host's available document-reading capability for PDF/DOCX inputs;
- preserve the same output files, review protocol, stage gates, and safety
  invariants.

`bob-survey` treats either managed root as evidence that Bob skills are
installed, without double-counting a dual-host installation.

## 7. Skill-Relative Scripts

`bob-model` and `visual-md` currently contain `.claude/skills/.../scripts`
paths. Each skill instead defines `<skill-dir>` as the directory containing the
currently loaded `SKILL.md` and launches scripts from `<skill-dir>/scripts`.

Resolution rules are:

1. use the path of the skill selected by the current host;
2. verify the expected script exists before executing it;
3. report the resolved script path on failure;
4. do not silently cross from `.agents` to `.claude` or vice versa.

This keeps the two output trees independently usable and prevents Codex from
depending on Claude's copy as an implementation detail.

## 8. Init Semantics

`run-bob init` preserves its existing per-file behavior:

- fresh target: write both skill trees;
- `--minimal`: write both skill trees and skip non-skill harness assets;
- without `--force`: skip each existing destination independently;
- with `--force`: overwrite managed destinations under both roots;
- never ignore either skill root in `.gitignore`;
- set the executable bit on every generated `.sh` file on Unix.

An existing customized Claude skill is therefore not overwritten by a normal
`init`, while a missing Codex copy can still be installed from the embedded
template.

## 9. Upgrade and Backup Semantics

An existing Claude-only project migrates through the normal upgrade command:

```text
run-bob upgrade --dry-run
run-bob upgrade
run-bob status
```

`upgrade --dry-run` reports all missing Codex assets but writes nothing.
Applying the upgrade installs missing `.agents/skills` files. Any managed file
whose content is outdated follows the existing backup-before-overwrite rule,
with its complete host-relative path retained under
`.run-bob-backup/<UTC-timestamp>/`.

The migration may make small additive edits to shared `SKILL.md` templates for
dual invocation and host-neutral paths. Those changes therefore also update
the managed Claude copies under the established upgrade contract. Safety is
provided by all of the following:

- slash invocations and Claude branches remain present;
- changed Claude files are backed up before replacement;
- `CLAUDE.md`, `ARCHITECTURE.md`, `CleanArchitectureTest.java`, generated
  documents, and compliance sources remain user-owned and untouched;
- a legacy-migration regression test proves that adding an already-current
  Codex tree does not rewrite the Claude tree;
- a compatibility contract test proves no managed Claude path was removed.

Both updated and newly installed `.sh` files receive executable permissions on
Unix. Permission handling is shared by `init` and `upgrade` so the two commands
cannot drift.

## 10. Status and Errors

`status` presents Claude Code and Codex skills as distinct groups. A complete
harness requires all applicable managed files under both roots.

- complete: print success and return `Ok(())`;
- any missing or wrong-kind path: print each failure and return an error, which
  gives the CLI a non-zero exit status;
- non-Java targets continue to skip the optional Java skeleton;
- the error recommends `run-bob init` for an uninitialized target and
  `run-bob upgrade` for an existing/legacy harness.

File and directory conflicts must include the exact destination path in the
error. This migration does not generate links and does not broaden writes
beyond the explicit target directory.

## 11. Contributor Install Skill and Rust Bootstrap

The contributor skill remains repository-specific. Claude Code invokes
`/install`; Codex invokes `$install`. Both copies describe and execute the same
workflow:

```text
repository check
→ optional fast-forward sync
→ ensure Rust toolchain
→ cargo build --release --locked
→ cargo test --locked
→ cargo install --locked --path .
→ verify run-bob version and help
```

Rust is a build toolchain, not an end-user runtime dependency. Released binary
installation through `install.sh` or `install.ps1` remains unchanged and does
not install Rust.

For source installation, a small repository-local bootstrap helper provides a
deterministic, testable boundary for toolchain preparation. The POSIX and
PowerShell entry points implement equivalent decisions:

1. If `rustc` and `cargo` exist and satisfy the repository's minimum version,
   use them unchanged.
2. If the toolchain is absent, use official `rustup` installation channels,
   select the `minimal` profile, and install stable automatically. Invoking the
   repository `/install` or `$install` skill is the user's authorization for
   this scoped toolchain installation; platform approval prompts still apply.
3. If an older rustup-managed toolchain exists, install a suitable stable
   toolchain for this build without changing the user's global default.
4. If an older non-rustup toolchain exists, stop with a precise explanation
   rather than replacing a package-manager-owned installation.
5. Re-run `rustc --version` and `cargo --version`. A failed or still-inadequate
   installation stops before build, test, or binary replacement.

The POSIX installer uses Rust's official TLS-constrained command from
<https://rust-lang.org/learn/get-started/>. Native Windows uses the official
architecture-appropriate `rustup-init.exe`; Visual C++ Build Tools failures are
reported rather than hidden. The minimal profile contains `rustc`, `rust-std`,
and `cargo`, as documented at
<https://rust-lang.github.io/rustup/concepts/profiles.html>.

Tests place mock executables first in an isolated `PATH` and use temporary
Cargo/Rustup homes so they can simulate the toolchain and downloader without
network access or modification of the developer machine. Official Rust
endpoints are fixed production constants, not environment-configurable URLs.

## 12. Documentation

Update current user-facing sources, not historical plans/specs:

- `README.md`;
- `src/templates/root/README-RUN-BOB.md`;
- CLI about text and next-step output;
- `Cargo.toml` package description and explicit `rust-version = "1.75"`;
- both contributor install skill copies.

Documentation must explain:

- the two host roots;
- `/skill` versus `$skill` invocation;
- eight Bob skills plus `visual-md`;
- binary update versus per-project `run-bob upgrade`;
- the legacy Claude-only migration flow;
- that prebuilt users do not need Rust;
- that source-install skills bootstrap Rust only when needed.

Historical dated design and plan documents remain historical records. This
document supersedes their Claude-only assumptions for skill installation.

## 13. Test Strategy

Implementation follows test-driven development. Add failing tests before each
behavioral change.

### Registry and init

- every Claude skill asset has an identical Codex mirror;
- fresh init creates the complete byte-identical trees;
- all shell scripts are executable on Unix;
- minimal init installs both skill hosts and no non-skill assets;
- normal init preserves an existing Claude sentinel while installing missing
  Codex files;
- `visual-md` and its scripts are included explicitly.

### Upgrade

- dry-run from a Claude-only fixture creates no Codex files;
- applying upgrade adds all Codex files;
- an already-current Claude tree remains byte- and mode-identical while the
  missing Codex tree is installed;
- stale files under both roots are backed up independently and restored;
- newly installed shell scripts are executable.

### Status

- a complete dual-host harness succeeds;
- deleting one Claude asset fails;
- deleting one Codex asset fails;
- failure has a non-zero process exit code and an actionable message.

### Skill contracts

- every generated skill has valid `name` and `description` metadata;
- metadata meets Codex validation limits;
- both explicit invocation forms remain documented;
- no interactive generated skill hard-codes the opposite host root;
- repository install copies are identical and discoverable.

### Rust bootstrap

Use mocked commands and isolated temporary homes to cover:

- suitable Rust already installed;
- toolchain completely absent and installed successfully;
- official installer failure;
- installer reports success but `rustc`/`cargo` remain unavailable;
- old rustup-managed toolchain uses a build-local suitable toolchain;
- old non-rustup toolchain stops without modifying it.

Tests must never download Rust or alter the machine's real toolchain.

## 14. Verification

Before completion, run:

```text
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test --locked
Codex quick validation against every generated .agents skill
Claude/Codex tree parity audit
git diff --check
```

Forward-test at least one generated Bob skill from `.agents/skills` in an
isolated temporary target. Separately inspect the generated `.claude/skills`
tree and confirm the slash invocation, directory names, script resolution, and
Superpowers handoff remain intact.

## 15. Acceptance Criteria

The migration is complete only when:

1. fresh init and minimal init generate both real skill trees;
2. Codex can discover and explicitly invoke every generated skill;
3. Claude Code retains every existing skill path and slash invocation;
4. interactive scripts work from either host's own directory;
5. legacy upgrade is dry-run-safe, backup-safe, and permission-correct;
6. status fails when either host is incomplete;
7. repository `/install` and `$install` share one behavior and automatically
   install an absent Rust build toolchain through official rustup;
8. prebuilt run-bob installation still has no Rust prerequisite;
9. all automated validation and forward tests pass.
