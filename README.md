# run-bob

> A tiny Rust CLI that bootstraps a **Bob's 4-ring Clean Architecture + Superpowers harness** for Claude Code projects in one command.

`run-bob init` installs 7 Claude Code skills (`bob-survey`, `bob-stories`, `bob-identify`, `bob-onion`, `bob-spec`, `bob-nfr`, `bob-compliance`) plus anchor documents (`CLAUDE.md` with hard rules R0–R13, `ARCHITECTURE.md`, `README-RUN-BOB.md`) plus shared Java skeletons + an ArchUnit guard + a managed `.gitignore` block. Together they give Claude Code a structured pipeline from **fuzzy business need / α legacy → architecture survey → story split → identity test → 4-ring design → spec → TDD implementation → NFR & compliance review**.

```
   business need / α legacy code / new feature
              ↓
  /bob-survey     ──→ docs/bob/00-survey-*.md       (architecture + difficulty health check)
              ↓                                       (Medium/Hard → split first)
  /bob-stories    ──→ docs/bob/02-stories-*.md       (UseCase-level stories, +safety net)
              ↓
  /bob-identify   ──→ docs/bob/01-identity-*.md      (5-question decision tree)
              ↓
  /bob-onion      ──→ ARCHITECTURE.md                 (4-ring SSoT)
              ↓
  /bob-spec       ──→ docs/specs/spec-*.md            (+ open questions for Superpowers)
              ↓
  Superpowers     ──→ brainstorming → writing-plans → executing-plans (TDD)
              ↓
  /bob-compliance ──→ docs/bob/05-compliance-*.md     (rule-based diff check)
              ↓
  /bob-nfr        ──→ docs/bob/04-nfr-*.md            (13-card NFR review)
              ↓
              superpowers:finishing-a-development-branch
```

## Status

✅ **v0.2.0** — 6 phases live. Spec list under [`docs/superpowers/specs/`](docs/superpowers/specs/).

---

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
| `RUN_BOB_VERSION` | Pin to a specific tag (e.g. `v0.2.0`) |

For POSIX `curl ... | sh`, put env vars on the **`sh` side** of the pipe:

```bash
curl -fsSL https://raw.githubusercontent.com/amwtke/run-bob/master/install.sh \
  | RUN_BOB_VERSION=v0.2.0 RUN_BOB_INSTALL_DIR=/usr/local/bin sh
```

For PowerShell, set `$env:*` before the `iex` line:

```powershell
$env:RUN_BOB_VERSION = 'v0.2.0'
$env:RUN_BOB_INSTALL_DIR = "$env:USERPROFILE\bin"
iwr -useb https://raw.githubusercontent.com/amwtke/run-bob/master/install.ps1 | iex
```

macOS first-run note: the installer auto-strips the quarantine attribute. If Gatekeeper still complains, run `xattr -d com.apple.quarantine ~/.local/bin/run-bob`.

### Manual / source install

- **Tarball:** grab from the [Releases page](https://github.com/amwtke/run-bob/releases), unpack into any directory on your PATH.
- **From source (Rust ≥ 1.75):** `cargo install --git https://github.com/amwtke/run-bob` or clone + `cargo install --path .`.

---

## Update

There are **two independent things to update**, and they are kept in lockstep on purpose:

| What | When | How |
|---|---|---|
| The **`run-bob` binary** on your machine | A new release is published | re-run the one-liner — it overwrites in place |
| The **harness assets inside a project** (skills, README-RUN-BOB.md, shared Java skeletons, `docs/compliance/README.md`) | After upgrading the binary | run `run-bob upgrade` inside the project |

### Step 1 — Update the binary

```bash
# Linux / macOS — same one-liner as install, just re-run it
curl -fsSL https://raw.githubusercontent.com/amwtke/run-bob/master/install.sh | sh

# Or pin to a specific tag
curl -fsSL https://raw.githubusercontent.com/amwtke/run-bob/master/install.sh \
  | RUN_BOB_VERSION=v0.2.0 sh
```

Windows PowerShell uses the `install.ps1` variant — see the [Install](#install) section above.

Verify:

```bash
run-bob --version
```

### Step 2 — Update the harness inside each project

A project initialized by an older `run-bob` has older skill content baked in. After upgrading the binary, run **inside each project that was previously `run-bob init`'d**:

```bash
cd your-project/
run-bob upgrade --dry-run   # preview what would change
run-bob upgrade             # apply
```

What `upgrade` does:

- **Compares** the on-disk content of every upgrade-safe asset (skills, `README-RUN-BOB.md`, `UseCase.java`, `TransactionalUseCaseDecorator.java`, `docs/compliance/README.md`) against the binary's embedded version (byte-for-byte).
- **Backs up** any file that differs to `.run-bob-backup/<UTC-timestamp>/<original-path>`, then **overwrites** with the embedded version. The backup directory is auto-added to `.gitignore`.
- **Installs** any upgrade-safe asset that's missing (e.g. you upgraded across a release that introduced a new skill — `upgrade` will drop it in).
- **Never touches** user-owned files: `CLAUDE.md` (your project rules + tech-stack decisions), `ARCHITECTURE.md` (the 4-ring SSoT), `CleanArchitectureTest.java` (your `FORBIDDEN_IN_INNER` list), and anything under `docs/compliance/sources/`, `docs/compliance/*.md` (generated), `docs/compliance/.compliance.lock`, `docs/bob/`, `docs/specs/`. Use `run-bob init --force` if you really want to reset the user-owned anchors.

### Step 3 — Verify

```bash
run-bob status
```

Prints a green/red checklist of every required asset.

---

## Command reference

`run-bob` has **3 subcommands**: `init` (one-time bootstrap), `upgrade` (refresh harness), `status` (audit). All three take a `--dir` flag to target a directory other than the cwd.

### `run-bob init` — bootstrap a project

```bash
run-bob init [--dir <path>] [--force] [--minimal] [--no-gitignore]
```

| Flag | Effect | When to use it |
|---|---|---|
| `-d`, `--dir <path>` | Target directory (default `.`) | Bootstrapping a sibling subdirectory (e.g. `--dir ./api`) without `cd` |
| `-f`, `--force` | Overwrite existing files **including user-owned anchors** (`CLAUDE.md`, `ARCHITECTURE.md`, `CleanArchitectureTest.java`) | You explicitly want to reset the harness — destroys local edits to those 3 files |
| `-m`, `--minimal` | **Only** install the 7 skills under `.claude/skills/`. Skip anchor docs, ArchUnit guard, shared Java skeletons, `docs/bob/`, `docs/specs/`, `docs/compliance/` | Adding bob skills to an existing project that already has its own architecture conventions / Java layout |
| `--no-gitignore` | Skip writing the `# run-bob` block (containing `.run-bob-backup/`) into the target directory's `.gitignore` | You manage `.gitignore` by hand or have your own ignore strategy |

Default behavior creates this layout:

```
your-project/
├── .claude/skills/
│   ├── bob-survey/SKILL.md       # 🩺 phase 0 — TL intake (health check + recommendation)
│   ├── bob-stories/SKILL.md      # 🧩 phase 1 — split into UseCase stories
│   ├── bob-identify/SKILL.md     # 🔍 5-question identity test (G/B1/B2 mode)
│   ├── bob-onion/SKILL.md        # 🧅 4-ring design → ARCHITECTURE.md SSoT
│   ├── bob-spec/SKILL.md         # 📝 spec gen → Superpowers handoff
│   ├── bob-nfr/SKILL.md          # ⚖️ phase 2 — 13-card NFR review after TDD
│   └── bob-compliance/SKILL.md   # 🛡 phase 3 — rule-based diff compliance check
├── CLAUDE.md                     # 🛡 project-level hard rules R0–R13 (user-owned)
├── ARCHITECTURE.md               # 📘 4-ring architecture SSoT (user-owned)
├── README-RUN-BOB.md             # 📖 in-project user guide (upgrade-safe)
├── .gitignore                    # auto-managed `# run-bob` block (+ existing user content)
├── docs/
│   ├── bob/                      # bob skills' intermediate outputs
│   ├── specs/                    # bob-spec outputs
│   └── compliance/               # phase-3 compliance scaffold
│       ├── README.md             # how to use this folder (auto-managed)
│       └── sources/              # ← drop your team standards here (PDF/docx/md)
└── src/
    ├── main/java/com/example/shared/
    │   ├── usecase/UseCase.java
    │   └── framework/transaction/TransactionalUseCaseDecorator.java
    └── test/java/architecture/CleanArchitectureTest.java
```

### `run-bob upgrade` — refresh harness in a project

```bash
run-bob upgrade [--dir <path>] [--dry-run] [--no-backup] [--no-gitignore]
```

| Flag | Effect | When to use it |
|---|---|---|
| `-d`, `--dir <path>` | Target directory (default `.`) | Upgrading a sibling subdirectory without `cd` |
| `-n`, `--dry-run` | Report what **would** change; write **nothing** | Always — use this first to preview before applying |
| `--no-backup` | Skip the safety backup before overwriting | You trust git fully, want minimal noise on disk. **Dangerous if anyone hand-edited the upgrade-safe files** |
| `--no-gitignore` | Skip the `.gitignore` maintenance step | You manage `.gitignore` by hand |

Typical flow after a binary update:

```bash
run-bob upgrade --dry-run    # see what would change
run-bob upgrade              # apply (creates .run-bob-backup/<UTC>/ first)
run-bob status               # confirm everything's in place
```

### `run-bob status` — audit harness completeness

```bash
run-bob status [--dir <path>]
```

| Flag | Effect | When to use it |
|---|---|---|
| `-d`, `--dir <path>` | Target directory (default `.`) | Auditing a sibling subdirectory |

Prints `✓` / `✗` for every asset run-bob expects. Exit code is non-zero when assets are missing — handy in CI to refuse merges from a corrupted harness.

### Globals

| Flag | Effect |
|---|---|
| `-h`, `--help` | Print help (works on every subcommand too) |
| `-V`, `--version` | Print version and exit |

---

## The 7 skills (workflow order)

After `run-bob init`, open Claude Code in that directory and use the skills in this order. Each skill writes its output into `docs/bob/` or `docs/specs/` so the next skill can pick up where the previous one stopped.

| # | Skill | Phase | Output |
|---|---|---|---|
| 0 | 🩺 `/bob-survey <requirement>` | TL intake | `docs/bob/00-survey-*.md` + row in `ARCHITECTURE.md §12` |
| 1 | 🧩 `/bob-stories <requirement>` | Story split (for Medium/Hard) | `docs/bob/02-stories-*.md` + per-story files |
| 2 | 🔍 `/bob-identify <description>` | 5-question identity test | `docs/bob/01-identity-*.md` |
| 3 | 🧅 `/bob-onion` | 4-ring design | updates `ARCHITECTURE.md` (SSoT) |
| 4 | 📝 `/bob-spec <use case>` | Per-use-case spec | `docs/specs/spec-*.md` |
| — | _Superpowers TDD_ | brainstorm → plan → execute | branch with passing tests |
| 5 | 🛡 `/bob-compliance` | Compliance gate (phase 3) | `docs/bob/05-compliance-*.md` |
| 6 | ⚖️ `/bob-nfr <spec-path>` | NFR review (phase 2) | `docs/bob/04-nfr-*.md` |

Compliance runs **before** NFR by convention: compliance is the strict-rule gate, NFR is open-question gathering.

See [`README-RUN-BOB.md`](src/templates/root/README-RUN-BOB.md) (installed by `run-bob init`) for the full per-skill detail + FAQ.

---

## Three modes

| Mode | Scenario |
|---|---|
| **G** Greenfield | Brand new project. From business description → 4-ring → spec. |
| **B1** Brownfield full refactor | Existing α/β code → identity test → α→γ refactor plan |
| **B2** Brownfield incremental | Existing project + new feature. New code must be γ even if surrounded by legacy. "Clean island" via ACL ports. |

`/bob-identify` auto-detects which mode applies from the 5-question decision tree.

---

## Why this exists

Bob Martin's *Clean Architecture* (2017) prescribes a 4-ring Onion model with strict inward-only dependencies — `entity` ← `usecase` ← `adapter` ← `framework`. The promise: business core stays independent from Spring / database / UI.

In practice, 80% of "Clean Architecture" projects in the wild are stuck at **β state** — folder names are correct, but the use case layer still imports `org.springframework.*` / `org.slf4j.*`. A single migration (信创: Oracle → 达梦, Spring Boot → 东方通) exposes the deception.

`run-bob` is a constraint-first harness for AI coding tools. It encodes Bob's three landing actions:

1. **Interface position inversion** — business interfaces in `usecase/port`, adapters implement
2. **Framework boundary push-out** — UseCase classes are pure POJO (zero Spring, zero SLF4J)
3. **State machine lift** — business rules live on the Entity, not in Service

The single `@Transactional` in the entire project lives in **one decorator class** (`shared.framework.transaction.TransactionalUseCaseDecorator`). An ArchUnit test enforces the 4-ring rules mechanically in CI.

---

## Relation to ddd-run

[`ddd-run`](https://github.com/amwtke/ddd-run) is the DDD-flavored sibling. It targets DDD tactical level (Aggregate / Bounded Context / Ubiquitous Language); `run-bob` targets pure Bob 4-ring (single Bounded Context, sync, no Domain Event by default).

Both share the same engineering form: Rust CLI + embedded skill templates + anchor documents + ArchUnit. Pick `ddd-run` if you need strategic design (multiple BCs, async events); pick `run-bob` if you need pure 4-ring discipline for a single-context system or for incremental new features inside an α/β codebase.

---

## Contributor install via `/install` skill

Open **this repo** in Claude Code and run `/install` — it does `git pull` → toolchain check → `cargo build --release` → `cargo test` → `cargo install --path .` → `run-bob --version` verify, refusing to install a binary whose tests fail.

See [`.claude/skills/install/SKILL.md`](.claude/skills/install/SKILL.md) for the exact procedure.

---

## License

MIT
