# run-bob

> A tiny Rust CLI that bootstraps a **Bob's 4-ring Clean Architecture + Superpowers harness** for Claude Code projects in one command.

`run-bob init` installs three Claude Code skills (`bob-identify`, `bob-onion`, `bob-spec`) plus anchor documents (`CLAUDE.md`, `ARCHITECTURE.md`, `README-RUN-BOB.md`) plus shared Java skeletons (`UseCase`, `TransactionalUseCaseDecorator`) plus an ArchUnit guard, giving you a structured workflow from **fuzzy business description / existing α legacy → identity test → 4-ring design → spec → TDD implementation**.

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

🚧 **In design phase.** The full design spec is at:

- [`docs/superpowers/specs/2026-05-08-run-bob-design.md`](docs/superpowers/specs/2026-05-08-run-bob-design.md)

Implementation will follow via `superpowers:writing-plans` → `superpowers:executing-plans` (TDD).

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
