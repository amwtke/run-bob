# run-bob

> A tiny Rust CLI that bootstraps a **Bob's 4-ring Clean Architecture + Superpowers harness** for Claude Code projects in one command.

`run-bob init` installs five Claude Code skills (`bob-survey`, `bob-stories`, `bob-identify`, `bob-onion`, `bob-spec`) plus anchor documents (`CLAUDE.md`, `ARCHITECTURE.md`, `README-RUN-BOB.md`) plus shared Java skeletons (`UseCase`, `TransactionalUseCaseDecorator`) plus an ArchUnit guard, giving you a structured workflow from **fuzzy business description / existing α legacy → architecture survey → story split → identity test → 4-ring design → spec → TDD implementation**.

```
   business need / α legacy code / new feature
              ↓
  /bob-identify   ──→ docs/bob/01-identity-*.md     (5-question decision tree)
              ↓
  /bob-onion      ──→ ARCHITECTURE.md                (4-ring SSoT)
              ↓
  /bob-spec       ──→ docs/specs/spec-*.md           (+ open questions for Superpowers)
              ↓
  Superpowers     ──→ brainstorming → writing-plans → executing-plans (TDD) → finishing-a-development-branch
```

## Why this exists

Bob Martin's *Clean Architecture* (2017) prescribes a 4-ring Onion model with strict inward-only dependencies — `entity` ← `usecase` ← `adapter` ← `framework`. The promise: business core stays independent from Spring / database / UI, so you can swap any "detail" without touching the core.

In practice, 80% of "Clean Architecture" projects in the wild are stuck at **β state** — folders are named correctly but the use case layer still imports `org.springframework.*` / `org.slf4j.*`. Single-database migration (信创: Oracle → 达梦, Spring Boot → 东方通) is enough to expose the deception.

`run-bob` is a constraint-first harness for AI coding tools (Claude Code, Cursor, Codex). It encodes Bob's three landing actions from atlas Stage 5 §4:

1. **Interface position inversion** — business interfaces in `usecase/port`, adapters implement
2. **Framework boundary push-out** — UseCase classes are pure POJO (zero Spring, zero SLF4J)
3. **State machine lift** — business rules live on the Entity, not in Service

The single `@Transactional` in the entire project lives in **one decorator class** in the framework layer (`shared.framework.transaction.TransactionalUseCaseDecorator`). An ArchUnit test enforces the 4-ring rules mechanically in CI.

## Status

✅ **Implementation complete (v0.1.0).** See [Install](#install) below. The full design spec is at:

- [`docs/superpowers/specs/2026-05-08-run-bob-design.md`](docs/superpowers/specs/2026-05-08-run-bob-design.md)

## Install

### One-liner (recommended)

**Linux + macOS:**

```bash
curl -fsSL https://raw.githubusercontent.com/amwtke/run-bob/master/install.sh | sh
```

**Windows (PowerShell):**

```powershell
iwr -useb https://raw.githubusercontent.com/amwtke/run-bob/master/install.ps1 | iex
```

The installer detects your OS/arch, fetches the matching prebuilt binary from the latest [GitHub Release](https://github.com/amwtke/run-bob/releases), and drops `run-bob` into your install dir.

Defaults:

|  | POSIX (Linux + macOS) | Windows |
|---|---|---|
| Install dir | `~/.local/bin` | `%USERPROFILE%\bin` |
| Version | latest release tag | latest release tag |

Override via env vars:

| Variable | Effect |
|---|---|
| `RUN_BOB_INSTALL_DIR` | Custom install directory |
| `RUN_BOB_VERSION` | Pin to a specific tag (e.g. `v0.1.0`) |

For POSIX `curl ... | sh`, put the env vars on the **`sh` side** of the pipe — variable bindings in front of `curl` go to `curl`, not the downstream shell:

```bash
curl -fsSL https://raw.githubusercontent.com/amwtke/run-bob/master/install.sh \
  | RUN_BOB_VERSION=v0.1.0 RUN_BOB_INSTALL_DIR=/usr/local/bin sh
```

For PowerShell, set `$env:*` before the `iex` line — they're visible to the downstream invocation:

```powershell
$env:RUN_BOB_VERSION = 'v0.1.0'
$env:RUN_BOB_INSTALL_DIR = "$env:USERPROFILE\bin"
iwr -useb https://raw.githubusercontent.com/amwtke/run-bob/master/install.ps1 | iex
```

macOS first-run note: the installer auto-strips the quarantine attribute, so Gatekeeper shouldn't get in the way. If it does, run `xattr -d com.apple.quarantine ~/.local/bin/run-bob`.

### Update

Re-run the same one-liner — the installer always pulls the latest release and overwrites in place.

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

### Manual download

Grab a tarball/zip directly from the [Releases page](https://github.com/amwtke/run-bob/releases) and unpack the `run-bob` binary into any directory on your PATH.

### From source (requires Rust ≥ 1.75)

```bash
cargo install --git https://github.com/amwtke/run-bob
```

Or clone + local install (preferred during development):

```bash
git clone https://github.com/amwtke/run-bob
cd run-bob
cargo install --path .
```

After install, `run-bob` is at `~/.cargo/bin/run-bob`.

### Via Claude Code `/install` skill (contributors only)

When you open **this repo** in Claude Code, a local skill is available:

```
/install
```

End-to-end: `git pull` → toolchain check → `cargo build --release` → `cargo test` (16 integration tests must pass) → `cargo install --path .` → `run-bob --version` verify.

**Why use the skill instead of bare `cargo install`?** It enforces "test before install" — refuses to install a binary whose test suite is failing, so a broken `git pull` won't quietly land in your `$PATH`. It also handles uncommitted-local-changes safely (asks before stashing). See [`.claude/skills/install/SKILL.md`](.claude/skills/install/SKILL.md) for the exact procedure.

## Usage

### Bootstrap a project

```bash
cd your-new-project/    # or an empty directory
run-bob init
```

This creates:

```
your-new-project/
├── .claude/
│   └── skills/
│       ├── bob-identify/SKILL.md     # 🔍 identity test (5-question decision tree)
│       ├── bob-onion/SKILL.md        # 🧅 4-ring design + ARCHITECTURE.md SSoT
│       └── bob-spec/SKILL.md         # 📝 spec generation → Superpowers handoff
├── CLAUDE.md                          # 🛡 project-level hard rules (R0-R12)
├── ARCHITECTURE.md                    # 📘 4-ring architecture SSoT
├── README-RUN-BOB.md                  # 📖 in-project user guide
├── docs/
│   ├── bob/                           # identify/onion intermediate notes
│   └── specs/                         # bob-spec outputs
└── src/
    ├── main/java/com/example/shared/
    │   ├── usecase/UseCase.java                            # generic UseCase<C, R>
    │   └── framework/transaction/
    │       └── TransactionalUseCaseDecorator.java          # the only @Transactional
    └── test/java/architecture/
        └── CleanArchitectureTest.java                      # ArchUnit guard
```

### Flags

```bash
run-bob init --force      # overwrite existing files
run-bob init --minimal    # only install the 3 skills (skip anchor docs / ArchUnit / shared)
run-bob init --dir ./api  # initialize a subdirectory
```

### Check harness status

```bash
run-bob status
```

Prints a green/red checklist of every required asset (3 skills + 3 anchor docs + ArchUnit + 2 shared Java files + 2 working dirs).

### The five skills

After `run-bob init`, open Claude Code in that directory and use these in order:

#### 🩺 `/bob-survey <requirement>` (phase 0 — TL intake)
Architectural health check + requirement difficulty + recommendation, before you even start identifying. Classifies the repo as G (greenfield) / β (brownfield no bob) / γ (mature bob), scores the architecture across 6 bob-specific dimensions (0-20 each, total 100), judges the requirement on 4 factors (cross-rings, state-machine delta, legacy reuse, pre-touch refactor count), and emits a 3-tier recommendation: 🟢 go ahead, 🟡 prepare some things first, 🔴 refactor before accepting. Output: `docs/bob/00-survey-*.md` + a row appended to `ARCHITECTURE.md §12 体检记录`. Run before `/bob-identify` (it'll soft-prompt you anyway).

#### 🧩 `/bob-stories <requirement>` (phase 1 — split into UseCase stories)
Triggered after survey for Medium/Hard requirements. 1:1-splits the requirement into UseCase-level stories — each one a deliverable unit you can feed to `/bob-identify --story <path>`. Supports `--refactor [path]` for pure refactor work (α→γ improvement units) and auto-detects "feature + refactor" mixed mode when survey's 4th factor (前置重构量) is Medium/Hard. Output: `docs/bob/02-stories-*.md` index + `docs/bob/02-stories/<n>-<slug>.md` per-story files. In refactor / mixed mode, also runs a Stage 2.5 全分支级 test coverage audit and auto-emits `R0.x · characterize` stories for any method whose branches aren't fully covered — so every refactor downstream has a green safety net before it starts.

#### 🔍 `/bob-identify <business description>` (or `--refactor` / new-feature description)
Identity test. Runs the **5-question decision tree** on every concept / import / annotation in your business description (or existing code, or new feature) and classifies each as **CORE / ADAPTER / FRAMEWORK / TOOL / 违规**. Auto-detects mode G (greenfield) / B1 (full refactor) / B2 (clean island for incremental new features). Output: `docs/bob/01-identity-*.md`.

#### 🧅 `/bob-onion`
4-ring architecture design. Reads the identity table, designs the 4 rings (entity / usecase / adapter / framework), lists ports + Entity state machines + decorator wiring, and **updates `ARCHITECTURE.md` (the SSoT)**. Also writes back project-specific entries to `CleanArchitectureTest.java`'s `FORBIDDEN_IN_INNER` array.

#### 📝 `/bob-spec <use case>` (or `--query` / `--refactor`)
Per-use-case spec. Reads `ARCHITECTURE.md`, produces a spec with **Given-When-Then scenarios**, full Java code stubs (Command / Result / UseCase POJO / Entity method / framework Config / Controller), Guardrails for Superpowers, and "open questions for Superpowers" (tech-stack decisions). Output: `docs/specs/spec-*.md`. Three templates: command / query / refactor.

### The two anchor documents

#### 🛡 `CLAUDE.md` — project-level Claude Code rules
Hard rules **R0-R12** that constrain every code generation in this project:
- R0: 5-question decision tree is the meta-rule (R7-R12 are its instances)
- R7: 4-ring package structure (entity/usecase/adapter/framework)
- R8: only one `@Transactional` (in `TransactionalUseCaseDecorator`)
- R9: anti-pattern hard list (no Spring/SLF4J/Jakarta/Lombok in inner rings; no `LocalDateTime.now()` in Entity)
- R10-bob: cross-context / async = upgrade trigger (refuses Domain Event by default)
- R12: B2 clean-island rule (new feature must be γ even if surrounded by legacy)

#### 📘 `ARCHITECTURE.md` — 4-ring architecture SSoT
The Single Source of Truth: bounded context, **port inventory**, **Entity state machines**, UseCase list, project-specific gadget catalog, ADRs. **Every class name, method name, and test description in the codebase must reference this document.** Managed exclusively by `/bob-onion`.

### A complete walkthrough (G mode)

```bash
# 1. Bootstrap once
mkdir my-order-system && cd my-order-system
run-bob init

# 2. Open Claude Code in this directory, then run the skills:
> /bob-identify 订单系统:用户支付订单,扣库存,发短信通知。已支付订单不能再支付。

# Claude runs the 5-question decision tree, classifies Order/PaymentGateway/SLF4J/...
# → docs/bob/01-identity-order.md

> /bob-onion

# Claude designs 4 rings + port inventory + Order state machine
# → updates ARCHITECTURE.md (SSoT)

> /bob-spec PayOrder

# Claude generates Given-When-Then spec + Java code stubs
# → docs/specs/spec-1-pay-order.md

# 3. First-time tech-stack decision (Superpowers):
> Run superpowers:brainstorming based on the open questions in spec-1.
> Confirm the stack and write back to CLAUDE.md ## 技术栈约定.

# 4. Implementation flow (Superpowers):
> superpowers:writing-plans → executing-plans (TDD) → finishing-a-development-branch

# 5. For the next use case, jump straight back to /bob-spec — the stack is decided.
```

See [`README-RUN-BOB.md`](src/templates/root/README-RUN-BOB.md) (installed by `run-bob init`) for the full in-project guide with FAQ.

## Three modes

| Mode | Scenario |
|---|---|
| **G** Greenfield | Brand new project. From business description → 4-ring → spec. |
| **B1** Brownfield full refactor | Existing α/β code → identity test → α→γ refactor plan |
| **B2** Brownfield incremental | Existing project + new feature. New code must be γ even if surrounded by legacy. "Clean island" via ACL ports. |

## Relation to ddd-run

[`ddd-run`](https://github.com/amwtke/ddd-run) is the DDD-flavored sibling of this project. It targets DDD tactical level (Aggregate / Bounded Context / Ubiquitous Language). `run-bob` targets pure Bob 4-ring (single Bounded Context, sync, no Domain Event by default).

Both share the same engineering form: Rust CLI + embedded skill templates + anchor documents + ArchUnit. The contents differ to reflect the methodological difference (atlas Stage 6 §2: "Clean Arch ⊂ DDD tactical").

Pick `ddd-run` if you need strategic design (multiple BCs, async events, business-expert collaboration); pick `run-bob` if you need pure 4-ring discipline for a mid-complexity single-context system or for incremental new features inside an α/β legacy codebase.

## License

MIT
