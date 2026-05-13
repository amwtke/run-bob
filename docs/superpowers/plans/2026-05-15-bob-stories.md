# bob-stories + survey v2 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Ship phase 1 of the bob workflow: a new `/bob-stories <requirement>` skill that 1:1-splits Medium/Hard requirements into UseCase stories (or refactor units via `--refactor`), AND a v2 revision of `/bob-survey` that adds a 4th difficulty factor "前置重构量" and routes Medium/Hard recommendations to `/bob-stories`. `/bob-identify` gets a soft-prompt for the stories index.

**Architecture:** Pure template additions/edits — one new skill markdown at `src/templates/skills/bob-stories.md`, one new `HARNESS_ASSETS` entry in `src/assets.rs`, targeted edits to `bob-survey.md` (Stage 2 adds Q4 + Stage 3 matrix's Medium/Hard cells redirect to `/bob-stories`) and `bob-identify.md` (new "再检查 /bob-stories" soft block + `--story <path>` convention). No new Rust code, no new crate deps. Verified by token-presence integration tests, same pattern as phase 0.

**Tech Stack:** Same as run-bob (Rust 1.75+, clap, anyhow, colored, tempfile). All content is Markdown.

**Spec:** [`docs/superpowers/specs/2026-05-15-bob-stories-design.md`](../specs/2026-05-15-bob-stories-design.md)

**Out of scope (deferred per spec §7):** Fixture-based behavior tests, ARCHITECTURE.md §12 changes, phase 2 `/bob-nfr`.

---

## File Structure

| File | Action | Responsibility |
|---|---|---|
| `src/templates/skills/bob-stories.md` | **Create** | Full `/bob-stories` skill body (frontmatter + workflow stages + Q1/Q2 三段式 + feature/refactor story templates) |
| `src/assets.rs` | **Modify** | Append one `Asset` entry for bob-stories (`upgrade_safe: true`, `included_in_minimal: true`) |
| `src/templates/skills/bob-survey.md` | **Modify** | Stage 2 add Q4 factor + combination rule wording stays;Stage 3 matrix change Medium/Hard下一步命令 from `/bob-identify` to `/bob-stories`;Stage 4 template reference;header wording "三因子" → "四因子" |
| `src/templates/skills/bob-identify.md` | **Modify** | Append "## 再检查 /bob-stories (soft 前置)" block after existing survey soft-prompt block;document `--story <path>` skill-level convention |
| `tests/integration.rs` | **Modify** | (a) update `init_creates_bob_survey_skill` token list with `Q4`/`前置重构量`/`/bob-stories`;(b) add 3 new tests:`init_creates_bob_stories_skill`/`bob_survey_matrix_routes_to_stories`/`bob_identify_mentions_stories_soft_prompt` |
| `README.md` | **Modify** *(optional Task 4)* | "four skills" → "five skills";add `/bob-stories` subsection |

**Untouched** (plan must NOT modify):
- `Cargo.toml`
- `src/main.rs`, `src/lib.rs`, `src/commands/*`
- `src/templates/skills/bob-onion.md`, `src/templates/skills/bob-spec.md`
- `src/templates/root/*` (CLAUDE.md, ARCHITECTURE.md, README-RUN-BOB.md, *.java)

---

## Architectural Notes for the Engineer

Read these before starting.

### 1. Skill files are Markdown prompts for Claude

run-bob ships templates via `include_str!`; the CLI never parses or runs them. When a user types `/bob-stories` inside a Claude Code session, Claude reads the installed skill body as a system prompt for that conversation. So:

- Quality of writing matters; we're authoring instructions, not code.
- Tests check that key tokens are present in the installed file. We do NOT verify LLM behavior in automation (deferred).

### 2. Sibling skills set the convention

Read these before writing/editing:
- `src/templates/skills/bob-survey.md` — your most recent sibling. Same author, same voice, same 三段式 conventions. The phase 1 work edits this file directly (Task 1).
- `src/templates/skills/bob-identify.md` — already has `## 先检查 /bob-survey (soft 前置)` from Task 3 of phase 0; the new stories soft block sits right after it.
- `src/templates/skills/bob-onion.md` / `bob-spec.md` — for tone reference.

### 3. The HARNESS_ASSETS entry inherits all the pipelines

Registering with `upgrade_safe: true` + `included_in_minimal: true` automatically wires:
- `run-bob init` installs it (incl. `--minimal`)
- `run-bob status` checks it (drift guard `status_checks_every_file_init_writes`)
- `run-bob upgrade` syncs it on version drift
- The policy guard `upgrade_safe_field_matches_category_policy` enforces `Category::Skill` → `upgrade_safe: true`

The `init_minimal_skips_archunit_and_shared_and_anchors` test was already extended in phase 0 to list bob-survey; you'll also need to add bob-stories to that list.

### 4. The "三因子" → "四因子" rename matters

The spec §2 says Stage 2 of bob-survey expands from 3 to 4 factors. Several strings need updating:
- Workflow ASCII summary at top (`Stage 2. 需求难度三因子判定` → 四因子)
- Stage 2 section header
- Stage 4 output template (`## 3. 需求难度三因子` → 四因子)
- Description in YAML frontmatter (mentions "三因子")

The combination rule itself doesn't change (任一 Hard → Hard;≥2 Medium → Medium;else Easy) — it just applies to 4 factors instead of 3.

### 5. Recommendation matrix changes only Medium/Hard cells in non-🔴 rows

Cells that already say "先 `/bob-onion --refactor`" or "先 B1 全量重构" (the 🔴 cells) stay unchanged — those are pre-stories-phase decisions.

Cells to change (`/bob-identify` → `/bob-stories <需求>`):
- 80-100 / Medium: 🟢 → uses `/bob-stories`
- 80-100 / Hard: 🟡 → uses `/bob-stories`
- 60-79 / Easy: stays 🟢 `/bob-identify`(B2 模式)— Easy never gets stories
- 60-79 / Medium: 🟡 → uses `/bob-stories`
- 60-79 / Hard: stays 🔴 (refactor first)
- 0-59 cells: stay as-is (low score routes through refactor or warning)

### 6. `--story <path>` is a skill-level convention, NOT a CLI flag

It's just text in the `bob-identify.md` body that says "if you see `/bob-identify --story <path>`, read the file at <path> and treat its content as the requirement input." The Rust CLI doesn't gain a `--story` argument. This keeps run-bob CLI surface stable.

---

## Task 1: bob-survey v2 — add 4th factor + reroute matrix

**Files:**
- Modify: `src/templates/skills/bob-survey.md`
- Modify: `tests/integration.rs`

**Goal:** Update bob-survey to (a) collect a 4th difficulty factor "前置重构量", (b) route Medium/Hard recommendations to `/bob-stories` instead of `/bob-identify`. Existing test must accept the new tokens.

### Subgoals enumerated

1. Frontmatter description: change "(跨环数 / 状态机增量 / legacy 复用 三因子)" → "(跨环数 / 状态机增量 / legacy 复用 / 前置重构量 四因子)"
2. Workflow ASCII block: `Stage 2. 需求难度三因子判定` → `Stage 2. 需求难度四因子判定`
3. Stage 2 section header: `## Stage 2. 需求难度三因子判定` → `## Stage 2. 需求难度四因子判定`
4. Stage 2 prose: `LLM 三段式追问用户得出三因子等级。` → `LLM 三段式追问用户得出四因子等级。`
5. Add `### 因子 4: 前置重构量` block after factor 3
6. Stage 3 matrix: change Medium/Hard 下一步 commands per §5 of Architectural Notes above
7. Stage 4 output template: `## 3. 需求难度三因子` → `## 3. 需求难度四因子` + add `前置重构量 · ...` line
8. Update existing test `init_creates_bob_survey_skill` with 3 new tokens

- [ ] **Step 1: Read the current Stage 2 block to anchor edits**

Run: `sed -n '171,222p' /Users/xiaojin/workshop/run-bob/src/templates/skills/bob-survey.md`

You should see:
- Line 171: `## Stage 2. 需求难度三因子判定`
- Line 173: `LLM 三段式追问用户得出三因子等级。`
- Lines 175-216: 因子 1/2/3 blocks
- Line 218: `### 组合规则`

If line numbers differ or content differs from the spec, STOP and report BLOCKED.

- [ ] **Step 2: Update the existing test `init_creates_bob_survey_skill` with new required tokens**

Open `/Users/xiaojin/workshop/run-bob/tests/integration.rs` and find the `init_creates_bob_survey_skill` test (search for `fn init_creates_bob_survey_skill`). Locate its token list. Use `Edit` to extend the assertion list with 3 new tokens.

`old_string`:

```rust
        "Hard",
        // Recommendation matrix
```

`new_string`:

```rust
        "Hard",
        // v2 — 4th factor + stories routing
        "前置重构量",
        "Q4",
        "/bob-stories",
        // Recommendation matrix
```

This makes the test failing until Steps 4-8 land.

- [ ] **Step 3: Run the test (must fail)**

Run: `cd /Users/xiaojin/workshop/run-bob && cargo test --test integration init_creates_bob_survey_skill 2>&1 | tail -15`

Expected: failure on `bob-survey must mention 前置重构量` (the file doesn't have it yet).

- [ ] **Step 4: Update frontmatter description**

Edit `/Users/xiaojin/workshop/run-bob/src/templates/skills/bob-survey.md`:

`old_string`:

```
  (跨环数 / 状态机增量 / legacy 复用 三因子),结合两者给 3 档
```

`new_string`:

```
  (跨环数 / 状态机增量 / legacy 复用 / 前置重构量 四因子),结合两者给 3 档
```

- [ ] **Step 5: Update workflow ASCII block**

`old_string`:

```
Stage 2. 需求难度三因子判定
```

`new_string`:

```
Stage 2. 需求难度四因子判定
```

This appears twice in the file (workflow ASCII at top + Stage 2 section header). Use `replace_all: true` to update both occurrences in one call.

- [ ] **Step 6: Update Stage 2 prose**

`old_string`:

```
LLM 三段式追问用户得出三因子等级。
```

`new_string`:

```
LLM 三段式追问用户得出四因子等级。
```

- [ ] **Step 7: Add 因子 4 block**

Edit the file to insert a new `### 因子 4` block right before `### 组合规则`.

`old_string`:

```
### 因子 3: legacy 复用

> **Q3: 这个需求需要复用 legacy 代码吗?**
>
> **推测**:<基于需求描述里出现的 legacy 名词>
> **理由**:<一句话>
> **推荐选择**:`Easy` / `Medium` / `Hard`
>
> 标准:
> - Easy = 不依赖 legacy
> - Medium = 依赖 1-2 个 legacy `@Service`(可走 ACL)
> - Hard = ≥ 3 个 legacy + 还需要改 legacy 内部

### 组合规则
```

`new_string`:

```
### 因子 3: legacy 复用

> **Q3: 这个需求需要复用 legacy 代码吗?**
>
> **推测**:<基于需求描述里出现的 legacy 名词>
> **理由**:<一句话>
> **推荐选择**:`Easy` / `Medium` / `Hard`
>
> 标准:
> - Easy = 不依赖 legacy
> - Medium = 依赖 1-2 个 legacy `@Service`(可走 ACL)
> - Hard = ≥ 3 个 legacy + 还需要改 legacy 内部

### 因子 4: 前置重构量

> **Q4: 接这个需求需要先动多少现有文件?**
>
> **推测**:<结合 6 维度评分扣分点 + 需求碰到的类>
> **理由**:<一句话,引用具体的扣分维度或类名>
> **推荐选择**:`Easy` / `Medium` / `Hard`
>
> 标准:
> - Easy = 0-2 个现有文件需动
> - Medium = 3-7 个现有文件需动
> - Hard = 8+ 个现有文件 或 跨模块

> 说明:这一因子量"为了让新需求干净落地需要先改造多少现有代码"。结合 Stage 1 的 6 维度评分扣分点可以快速推估——通常扣分维度对应的文件就是要改造的候选。

### 组合规则
```

- [ ] **Step 8: Update Stage 3 棕地 matrix's Medium/Hard cells**

Read the current matrix block to confirm exact text:

Run: `sed -n '232,240p' /Users/xiaojin/workshop/run-bob/src/templates/skills/bob-survey.md`

You should see the 3×3 棕地 matrix. Apply this Edit:

`old_string`:

```
| **80-100(γ 健康)** | 🟢 `/bob-identify` | 🟢 `/bob-identify`(B2 模式) | 🟡 B2 清洁孤岛;或先 `/bob-onion --refresh` 增端口 |
| **60-79(β 可接受)** | 🟢 `/bob-identify`(B2 模式) | 🟡 B2 清洁孤岛 + 提前列 ACL 表 | 🔴 先 `/bob-onion --refactor` 出三动作改造计划 |
```

`new_string`:

```
| **80-100(γ 健康)** | 🟢 `/bob-identify` | 🟢 `/bob-stories <需求>` | 🟡 `/bob-stories <需求>`(B2 清洁孤岛);或先 `/bob-onion --refresh` 增端口 |
| **60-79(β 可接受)** | 🟢 `/bob-identify`(B2 模式) | 🟡 `/bob-stories <需求>` + 提前列 ACL 表 | 🔴 先 `/bob-onion --refactor` 出三动作改造计划 |
```

The 0-59 row stays unchanged — those cells route to refactor or warning, not stories.

- [ ] **Step 9: Update Stage 4 output template's 难度因子 section**

Find the embedded markdown template at the end of Stage 4 (it shows the schema for `00-survey-*.md`). The current template has:

```
## 3. 需求难度三因子
跨环数 · <Easy/Medium/Hard>(证据)
状态机增量 · <Easy/Medium/Hard>(证据)
legacy 复用 · <Easy/Medium/Hard>(证据)
→ 总评 **<Easy/Medium/Hard>**
```

Apply this Edit:

`old_string`:

```
## 3. 需求难度三因子
跨环数 · <Easy/Medium/Hard>(证据)
状态机增量 · <Easy/Medium/Hard>(证据)
legacy 复用 · <Easy/Medium/Hard>(证据)
→ 总评 **<Easy/Medium/Hard>**
```

`new_string`:

```
## 3. 需求难度四因子
跨环数 · <Easy/Medium/Hard>(证据)
状态机增量 · <Easy/Medium/Hard>(证据)
legacy 复用 · <Easy/Medium/Hard>(证据)
前置重构量 · <Easy/Medium/Hard>(证据:N 个文件需动,如 OrderService.java / OrderRepository.java)
→ 总评 **<Easy/Medium/Hard>**
```

- [ ] **Step 10: Update Stage 5 next-step pointer**

The very end of Stage 5 (or Stage 4) likely has a "推荐命令:`/bob-identify ...`" line. Per the spec the Medium/Hard path should now point at `/bob-stories`. Locate it via:

Run: `grep -n '推荐命令' /Users/xiaojin/workshop/run-bob/src/templates/skills/bob-survey.md`

If found, apply:

`old_string`:

```
推荐命令:`/bob-identify <需求> [--acl ...]`
```

`new_string`:

```
推荐命令(由 §4 推荐决定):
- Easy / 🟢 → `/bob-identify <需求>`
- Medium/Hard / 🟢🟡 → `/bob-stories <需求>`(用 --refresh 强制重跑)
- 🔴 → 先重构(`/bob-onion --refactor` 或 B1 全量重构)
```

If the line doesn't appear there (skill body may not literally have this string outside the embedded template), skip this step.

- [ ] **Step 11: Run the bob-survey test (must now pass)**

Run: `cd /Users/xiaojin/workshop/run-bob && cargo test --test integration init_creates_bob_survey_skill 2>&1 | tail -10`

Expected: test passes.

- [ ] **Step 12: Run the full suite (must stay green)**

Run: `cd /Users/xiaojin/workshop/run-bob && cargo test 2>&1 | tail -5`

Expected: **27 passed; 0 failed** (no new tests added in this task, just an existing test updated).

- [ ] **Step 13: Commit**

```bash
cd /Users/xiaojin/workshop/run-bob && git add src/templates/skills/bob-survey.md tests/integration.rs && git -c commit.gpgsign=false commit -m "$(cat <<'EOF'
feat(survey): v2 — add 4th factor 前置重构量 + route Medium/Hard to /bob-stories

Adds Q4 difficulty factor (number of pre-touch files: 0-2 / 3-7 / 8+
or cross-module) to bob-survey Stage 2. Combination rule unchanged.

Updates the recommendation matrix Medium/Hard cells in 80-100 and
60-79 score bands to point at /bob-stories <需求> instead of
/bob-identify — Medium/Hard requirements now go through story
splitting before identify. Easy and 0-59 cells unchanged.

Phase 1 prep for the new /bob-stories skill (next commit).
EOF
)"
```

---

## Task 2: Create `/bob-stories` skill + register in HARNESS_ASSETS + token test

**Files:**
- Create: `src/templates/skills/bob-stories.md`
- Modify: `src/assets.rs`
- Modify: `tests/integration.rs`

**Goal:** Ship the new skill template body + wire it in.

- [ ] **Step 1: Write the failing token test**

Append to `/Users/xiaojin/workshop/run-bob/tests/integration.rs`:

```rust
#[test]
fn init_creates_bob_stories_skill() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let target = tmp.path();
    std::process::Command::new(run_bob_bin())
        .args(["init", "--dir"])
        .arg(target)
        .status()
        .expect("init");

    let p = target.join(".claude").join("skills").join("bob-stories").join("SKILL.md");
    assert!(p.is_file(), "bob-stories SKILL.md missing at {}", p.display());
    let content = std::fs::read_to_string(&p).unwrap();

    // Frontmatter
    assert!(content.starts_with("---"));
    assert!(content.contains("name: bob-stories"));
    assert!(content.contains("description:"));

    // Load-bearing tokens
    for token in &[
        // CLI
        "/bob-stories",
        "--refactor",
        "--from-survey",
        "--refresh",
        // 三段式 conventions
        "三段式",
        "推测",
        "推荐选择",
        // Stages
        "Stage 0",
        "Stage 1",
        "Stage 2",
        "Stage 3",
        "Stage 4",
        // Mode detection
        "feature",
        "refactor",
        "混合",
        // Output paths
        "docs/bob/02-stories-",
        "docs/bob/02-stories/",
        // Story types in index
        "前置重构 stories",
        "新功能 stories",
        // Identify handoff
        "/bob-identify",
        "--story",
        // Survey input
        "00-survey-",
    ] {
        assert!(content.contains(token), "bob-stories must mention {}", token);
    }
}
```

Also extend `init_minimal_skips_archunit_and_shared_and_anchors` to include bob-stories (per Architectural Note #3):

`old_string`:

```rust
    for skill in &["bob-identify", "bob-onion", "bob-spec", "bob-survey"] {
```

`new_string`:

```rust
    for skill in &["bob-identify", "bob-onion", "bob-spec", "bob-survey", "bob-stories"] {
```

- [ ] **Step 2: Run the test (must fail)**

Run: `cd /Users/xiaojin/workshop/run-bob && cargo test --test integration init_creates_bob_stories_skill 2>&1 | tail -15`

Expected: failure on `bob-stories SKILL.md missing` (the file doesn't exist).

- [ ] **Step 3: Create the skill template**

Use `Write` to create `/Users/xiaojin/workshop/run-bob/src/templates/skills/bob-stories.md` with this exact content:

````markdown
---
name: bob-stories
description: |
  触发条件:用户输入 /bob-stories <需求>(主入口:从需求拆 UseCase),
  或 /bob-stories --refactor [path](纯重构模式:拆 α→γ 改造单元),
  或 /bob-stories --from-survey <path>(显式指定 survey 报告),
  或 /bob-stories --refresh(已有 02-stories-*.md 时强制重跑)。

  在 /bob-survey 之后、/bob-identify 之前接入。把 Medium/Hard 难度
  的需求 1:1 拆成 UseCase 故事(1 story = 1 UseCase),feature 与
  refactor 双模式支持,自动识别"前置重构量 ≥ Medium"时输出双表。
  产出 docs/bob/02-stories-<slug>-<date>.md 汇总索引 + 每个故事
  一份明细在 docs/bob/02-stories/<n>-<slug>.md。

  适用于 Bob 4 环 Clean Architecture 工作流的 phase 1:把大需求
  按 AI 友好的粒度切片。当用户说"拆 story"、"把这个需求拆开"、
  "按 UseCase 切一切"、"先拆几个故事"时也应触发此技能。
---

# Bob Stories Skill

## 触发

```
/bob-stories <需求>                  # 主入口:从需求拆 UseCase
/bob-stories --refactor [path]       # 纯重构模式:拆 α→γ 改造单元
/bob-stories --from-survey <path>    # 显式指定 survey 报告
/bob-stories --refresh               # 已有 02-stories-*.md 时强制重跑
```

或自然语言触发:"拆 story"、"把这个需求拆开"、"按 UseCase 切一切"、"先拆几个故事"。

## 前置条件

- **必须带需求**(或 `--refactor [path]` 指向重构对象)。无任一输入 → 拒绝。
- 项目在 git 仓库内(用于读取 `docs/bob/00-survey-*.md` 等输入)。
- 建议先跑过 `/bob-survey`——本 skill 在 survey 难度 ≥ Medium 时被 survey 推荐;若未跑过 survey,可以独立用,但拆分质量会下降。

## 提问规约(强制)

任何需要用户选择的问题,**必须**按下面三段式输出。**禁止**抛开放问题。

格式:

> **[问题序号] [问题]**
>
> **推测**:<你的判断>
> **理由**:<一句话>
> **推荐选择**:`<具体一个选项>`
>
> 是否同意?(回"是"走推荐;回"否,理由是 ..."重判;回"否,我选 X"切到 X)

## 目标

把大需求 1:1 拆成 UseCase 级别的故事,**每个故事 = 一个独立交付单元**,可直接喂 `/bob-identify --story <path>` 开始 identify→onion→spec→TDD 链。

- 1 story = 1 UseCase(feature 模式),或 1 个原子改造单元(refactor 模式)
- 自动识别"前置重构 + 新功能"混合需求,输出双表
- 不写代码、不出 spec、不画架构。**只回答**:"这个大需求该拆成哪些 story,顺序与依赖怎么走?"

## 工作流(5 个 Stage)

```
Stage 0. 输入归并(读 survey + 需求 + --refactor flag)
Stage 1. 自动识别模式(feature / refactor / 混合)
Stage 2. 三段式提拆法(LLM 提 UseCase / 改造单元列表)
Stage 3. 三段式拆顺序与依赖
Stage 4. 写汇总索引 + 每个 story 明细
```

---

## Stage 0. 输入归并

读取以下输入,按优先级合并:

1. 命令行 `<需求>` / `--refactor [path]` / `--from-survey <path>` 参数
2. 若 `--from-survey` 未指定,自动找 `docs/bob/00-survey-*.md` 中最新的一份(< 7 天)
3. 若 survey 报告存在,提取:
   - 总评难度(必须是 Medium 或 Hard,否则三段式追问"这个需求 Easy,要拆 stories 吗?")
   - 4 因子取值(尤其是 **前置重构量**,触发自动双模式)
   - 6 维度评分扣分点(用于推估 refactor 单元)

向用户**三段式**通报归并结果。

---

## Stage 1. 自动识别模式

3 种模式之一:

| 模式 | 触发 | 输出 |
|---|---|---|
| **feature** | 纯功能需求,survey 前置重构量 = Easy | 仅"新功能 stories"表 |
| **refactor** | `--refactor` flag 或 需求里含"重构/改造/port/adapter"等关键词 | 仅"前置重构 stories"表 |
| **混合** | survey 前置重构量 ≥ Medium,且需求里有新功能动作 | 双表:"前置重构 stories" + "新功能 stories" |

三段式追问用户确认模式:

> **Q0: 这个需求是 feature / refactor / 混合?**
>
> **推测**:<从 survey 第 4 因子 + 关键词判断>
> **理由**:<一句话>
> **推荐选择**:`feature` / `refactor` / `混合`
>
> 是否同意?

---

## Stage 2. 三段式提拆法

LLM 从需求 + 6 维度扣分点里抽出 UseCase / 改造单元,三段式输出:

> **Q1: 这个需求要拆成几个 story?**
>
> **推测**:5 个。新功能 3 个(ApproveOrder / RejectOrder / ViewApprovalHistory),前置重构 2 个(OrderService 状态机上提 / Decorator 收敛)。
> **理由**:你说"审批+驳回+查询" 3 个动作;survey 报告 OrderService.cancel/confirm 还在 service 包(扣分维度 4),先得提到 entity;@Transactional 散在 3 个 service 类(扣分维度 5),收一收。
> **推荐选择**:`5 个 story,先重构 2 个再做新功能 3 个`
>
> 是否同意?(回"是"走推荐;回"否,合并 R2 进 1"重判;回"否,我重新画一下"切到手动列表)

用户可"合并 X 进 Y"、"拆 X 为 X1+X2"、"丢掉 X"等指令调整。

### 拆分原则

- **feature stories**:1 个动词 = 1 个 UseCase = 1 story。如果两个动词共享 Entity 状态机但语义独立(approve vs reject),仍拆成两个 story。
- **refactor stories**:1 个原子改造 = 1 story。原子的定义:能在 1 个 PR 里干完,改完后所有现有测试仍绿,ArchUnit 通过。

---

## Stage 3. 三段式拆顺序与依赖

> **Q2: 顺序与依赖?**
>
> **推测**:R1 → R2 → 1 → 2 → 3。
> **理由**:R1(状态机上提)是 1/2/3 的前置;R2(decorator 收敛)如果想让 1 的事务边界干净也得先做。3(查询)不依赖 R1/R2,可并行,但放最后保稳。
> **推荐选择**:`R1 → R2 → 1 → 2 → 3`
>
> 是否同意?

### 依赖图记法

每个 story 的依赖列在 `依赖` 字段里,逗号分隔 ID(如 `R1, R2`)。`-` 表示无依赖。

---

## Stage 4. 写产出

### 汇总索引 `docs/bob/02-stories-<slug>-<YYYYMMDD>.md`

`<slug>` 由需求一行话生成(3-5 个汉字 / 英文 kebab),`<YYYYMMDD>` 是 UTC 日期。

模板:

```markdown
# 故事拆分 · <需求一行话>
日期 · <YYYY-MM-DD> · 模式 · <feature | refactor | feature+refactor> · 共 <N> story

## 1. 前置重构 stories(共 M)
> 仅在 feature+refactor 混合模式时出现。survey 第 4 因子 = Medium/Hard 触发。

| # | 改造单元 | 优先级 | 依赖 | 明细 |
|---|---|---|---|---|
| R1 | OrderService 状态机上提 | High | - | docs/bob/02-stories/R1-order-state-lift.md |
| R2 | TransactionalDecorator 收敛 | High | R1 | docs/bob/02-stories/R2-tx-decorator.md |

## 2. 新功能 stories(共 N-M)
| # | UseCase | 优先级 | 依赖 | 明细 |
|---|---|---|---|---|
| 1 | ApproveOrder | High | R1, R2 | docs/bob/02-stories/01-approve-order.md |
| 2 | RejectOrder | Medium | R1 | docs/bob/02-stories/02-reject-order.md |
| 3 | ViewApprovalHistory | Low | - | docs/bob/02-stories/03-view-history.md |

## 3. 推荐执行顺序
R1 → R2 → 1 → 2 → 3

(理由:R1 是 1/2/3 的前置;R2 让 1 的事务边界干净;3 与 R1/R2 解耦,可并行)

## 4. 下一步
对每个 story:
`/bob-identify --story docs/bob/02-stories/01-approve-order.md`
或
`/bob-identify <story 描述>`(若不传 --story 而手动复述)
```

### 每个故事文件

feature 模板 `docs/bob/02-stories/<n>-<usecase-kebab>.md`:

```markdown
# Story 01 · ApproveOrder
类型 · feature · 优先级 · High · 依赖 · R1, R2

## 1. 目标
管理员可以审批订单,通过后订单状态从 Submitted → Approved。

## 2. 用户故事(Given-When-Then 摘要)
- Given: 订单状态为 Submitted
- When: 管理员调用 approve(orderId, comment)
- Then: 订单状态变为 Approved,记录 approver/comment/approvedAt

## 3. 涉及概念
- Entity: Order(状态机 +1 转移 Submitted→Approved)
- Port: OrderRepository, AuditLog
- UseCase: ApproveOrderUseCase

## 4. 验收
- TDD 红→绿→重构通过
- ArchUnit 通过
- Entity.approve() 方法在 entity 包内

## 5. 下一步
`/bob-identify --story docs/bob/02-stories/01-approve-order.md`
```

refactor 模板 `docs/bob/02-stories/R<n>-<unit-kebab>.md`:

```markdown
# Story R1 · OrderService 状态机上提
类型 · refactor · 优先级 · High · 依赖 · -

## 1. 目标
把 OrderService.confirm() / OrderService.cancel() 里的状态判断 + 修改逻辑移到 Order entity。

## 2. 改造范围
- src/main/java/.../service/OrderService.java(删 3 个方法)
- src/main/java/.../entity/Order.java(加 confirm()/cancel() 方法)
- src/test/java/.../service/OrderServiceTest.java(测试随之迁移)

## 3. 验收
- ArchUnit 通过
- 现有 OrderServiceTest 测试**仍绿**(行为不变,只是位置变)
- 6 维度评分中"状态机位置"维度从 5 升到 ≥ 15
- @Transactional 收敛(若上原本散在 service 上,本 story 顺手收掉)

## 4. 下一步
`/bob-identify --refactor src/main/java/.../service/OrderService.java`
```

---

## 与 /bob-identify 的关系

跑完 `/bob-stories` 后,**不自动调用** `/bob-identify`。输出"下一步"命令,由用户自行决定何时开始下一个 story。

用户可使用 `/bob-identify --story <path>` 把 story 文件作为输入:identify 会读取文件 §1 目标 + §2 用户故事 / 改造范围,等价于把内容 inline 传入。这是 skill 模板层的约定,不是 run-bob CLI 新增 flag。

## TL 风对话

跑完拆分后,**主动提醒**(像真 TL 一样):

- "我把 R2 放到了 R1 之后,因为 decorator 收敛依赖状态机已经上提。如果你想并行做,可以接受 R2 中 @Transactional 注解临时多处共存,等 R1 完了再清。"
- "story 3(查询)和 R1/R2 解耦,可以让另一个人并行做。"

用户回"否"或"我换种顺序" → 尊重决定,在索引底部追加 "用户调整后顺序:..." 一行。
````

- [ ] **Step 4: Register the new asset in `HARNESS_ASSETS`**

Edit `/Users/xiaojin/workshop/run-bob/src/assets.rs`. Find this snippet (which is the current last skill entry):

```rust
    Asset {
        rel_path: &[".claude", "skills", "bob-survey", "SKILL.md"],
        content: include_str!("templates/skills/bob-survey.md"),
        category: Category::Skill,
        included_in_minimal: true,
        upgrade_safe: true,
    },
    // --- Harness documents (skipped in --minimal) ---
```

Insert a new entry for bob-stories between bob-survey and the Harness documents comment:

```rust
    Asset {
        rel_path: &[".claude", "skills", "bob-survey", "SKILL.md"],
        content: include_str!("templates/skills/bob-survey.md"),
        category: Category::Skill,
        included_in_minimal: true,
        upgrade_safe: true,
    },
    Asset {
        rel_path: &[".claude", "skills", "bob-stories", "SKILL.md"],
        content: include_str!("templates/skills/bob-stories.md"),
        category: Category::Skill,
        included_in_minimal: true,
        upgrade_safe: true,
    },
    // --- Harness documents (skipped in --minimal) ---
```

- [ ] **Step 5: Run the new test (must pass)**

Run: `cd /Users/xiaojin/workshop/run-bob && cargo test --test integration init_creates_bob_stories_skill 2>&1 | tail -15`

Expected: pass. If any token assertion fires, check the missing token against the actual file content.

- [ ] **Step 6: Run the full suite**

Run: `cd /Users/xiaojin/workshop/run-bob && cargo test 2>&1 | tail -10`

Expected: **28 passed; 0 failed** (27 prior + 1 new). The drift guards (`status_checks_every_file_init_writes`, `upgrade_safe_field_matches_category_policy`) auto-cover the new asset.

- [ ] **Step 7: Commit**

```bash
cd /Users/xiaojin/workshop/run-bob && git add src/templates/skills/bob-stories.md src/assets.rs tests/integration.rs && git -c commit.gpgsign=false commit -m "$(cat <<'EOF'
feat(skills): add /bob-stories skill (phase 1 — story splitting)

New /bob-stories <requirement> skill that 1:1-splits Medium/Hard
requirements into UseCase stories (feature mode) or α→γ refactor
units (--refactor mode), with auto-detection of feature+refactor
mixed mode when survey factor 4 = Medium/Hard.

Produces docs/bob/02-stories-<slug>-<date>.md index +
docs/bob/02-stories/<n>-<slug>.md per-story details. Soft handoff
to /bob-identify via the --story <path> skill-level convention
(not a Rust CLI flag).

Wires into HARNESS_ASSETS as upgrade-safe + included-in-minimal,
auto-flowing through init/status/upgrade pipelines.
EOF
)"
```

---

## Task 3: bob-identify — stories soft prompt + `--story` convention

**Files:**
- Modify: `src/templates/skills/bob-identify.md`
- Modify: `tests/integration.rs`

**Goal:** Insert a "再检查 /bob-stories (soft 前置)" block after the existing "先检查 /bob-survey" block + document the `--story <path>` skill-level input.

- [ ] **Step 1: Locate the existing soft-survey block**

Run: `grep -n '^## ' /Users/xiaojin/workshop/run-bob/src/templates/skills/bob-identify.md | head -8`

You should see:
- `## 触发`
- `## 先检查 /bob-survey (soft 前置)`
- `## 目标`
- ...

If the structure differs, STOP and report.

- [ ] **Step 2: Write the failing test**

Append to `/Users/xiaojin/workshop/run-bob/tests/integration.rs`:

```rust
#[test]
fn bob_identify_mentions_stories_soft_prompt() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let target = tmp.path();
    std::process::Command::new(run_bob_bin())
        .args(["init", "--dir"])
        .arg(target)
        .status()
        .expect("init");

    let p = target.join(".claude").join("skills").join("bob-identify").join("SKILL.md");
    let content = std::fs::read_to_string(&p).unwrap();

    for token in &[
        "/bob-stories",
        "02-stories-",
        "--story",
    ] {
        assert!(
            content.contains(token),
            "bob-identify must mention {} for stories integration",
            token
        );
    }
}
```

- [ ] **Step 3: Run the test (must fail)**

Run: `cd /Users/xiaojin/workshop/run-bob && cargo test --test integration bob_identify_mentions_stories_soft_prompt 2>&1 | tail -10`

Expected: failure on `bob-identify must mention /bob-stories`.

- [ ] **Step 4: Insert the new soft block**

Use `Edit` on `/Users/xiaojin/workshop/run-bob/src/templates/skills/bob-identify.md`. The anchor is the closing line of the existing `## 先检查 /bob-survey (soft 前置)` block — `**不强制**。用户明确说"否"则照旧继续。这一节遵从 TL 风:提问而不阻塞。` followed by blank line + `## 目标`.

`old_string`:

```
**不强制**。用户明确说"否"则照旧继续。这一节遵从 TL 风:提问而不阻塞。

## 目标
```

`new_string`:

```
**不强制**。用户明确说"否"则照旧继续。这一节遵从 TL 风:提问而不阻塞。

## 再检查 /bob-stories (soft 前置)

启动时若检测到下面任一条件,**三段式追问**用户是否先跑 `/bob-stories`:

- survey 报告显示难度 ≥ Medium,但项目内不存在 `docs/bob/02-stories-*.md`
- 项目内有 `02-stories-*.md` 但用户没有传 `--story <path>` flag,直接复述了整段需求

格式:

> **Q0b:看起来你已拆过 N 个 story,要不要先指明哪个 story?**
>
> **推测**:建议先指 story。我可以从需求里推测当前要做哪个 UseCase,但直接接整段需求容易把多个 UseCase 一锅煮,违反 1 story = 1 UseCase 的原则。
> **理由**:`docs/bob/02-stories-*.md` 索引里列了 N 个 story,每个都是独立交付单元。
> **推荐选择**:`/bob-identify --story docs/bob/02-stories/01-<...>.md`
>
> 是否同意?(回"是"→等用户给 story 路径;回"否"→把整段需求当一个 ad-hoc story 继续做身份测试)

### --story <path> 入口约定

`/bob-identify --story docs/bob/02-stories/01-approve-order.md`

行为:从 story 文件读 §1 目标 + §2 用户故事 / 改造范围 ,作为 identify 的输入。等价于把 story 内容 inline 传给 `/bob-identify`。

注意:这是 skill 模板约定的调用形式,不是 run-bob CLI 增加新 flag。

**不强制**。用户回"否"或不带 `--story` 则照旧。

## 目标
```

- [ ] **Step 5: Run the test (must pass)**

Run: `cd /Users/xiaojin/workshop/run-bob && cargo test --test integration bob_identify_mentions_stories_soft_prompt`

Expected: pass.

- [ ] **Step 6: Run the full suite**

Run: `cd /Users/xiaojin/workshop/run-bob && cargo test 2>&1 | tail -5`

Expected: **29 passed; 0 failed** (28 prior + 1 new). The existing `init_creates_bob_identify_skill` test still passes — we only added content, didn't remove anything.

- [ ] **Step 7: Commit**

```bash
cd /Users/xiaojin/workshop/run-bob && git add src/templates/skills/bob-identify.md tests/integration.rs && git -c commit.gpgsign=false commit -m "$(cat <<'EOF'
feat(skills): bob-identify soft-prompts /bob-stories + documents --story

Adds a "再检查 /bob-stories (soft 前置)" block after the existing
survey soft block. If survey says Medium/Hard but no stories index
exists, identify asks whether to split first. If stories index
exists but no --story <path> is given, identify asks which story
to focus on.

Documents the --story <path> skill-level convention — Claude reads
the story file's §1+§2 as identify input. NOT a Rust CLI flag.

Three-段式 prompt; user can decline and continue with ad-hoc identify.
EOF
)"
```

---

## Task 4 (optional): README — "five skills" update

**Files:**
- Modify: `README.md`

**Goal:** Mention `/bob-stories` in the README so users discover phase 1. Optional — skip if time-constrained.

- [ ] **Step 1: Locate the "four skills" section**

Run: `grep -n '^### The four skills\|^#### ' /Users/xiaojin/workshop/run-bob/README.md | head -10`

Expected: `### The four skills` followed by 4 `####` skill subsections.

- [ ] **Step 2: Rename section and add bob-stories**

Rename:

`old_string`:

```
### The four skills
```

`new_string`:

```
### The five skills
```

Then add the new sub-section **immediately after** the `/bob-survey` subsection and **before** `/bob-identify`. First, find the location:

Run: `grep -n '/bob-survey\|/bob-identify' /Users/xiaojin/workshop/run-bob/README.md | head -5`

You'll see the `#### 🩺 \`/bob-survey ...\`` heading and just below it the `/bob-survey` paragraph + a blank line + `#### 🔍 \`/bob-identify ...\``.

Edit pattern — anchor on the last sentence of the `/bob-survey` paragraph + the blank line + `/bob-identify` header:

`old_string`:

```
#### 🔍 `/bob-identify <business description>` (or `--refactor` / new-feature description)
```

`new_string`:

```
#### 🧩 `/bob-stories <requirement>` (phase 1 — split into UseCase stories)
Triggered after survey for Medium/Hard requirements. 1:1-splits the requirement into UseCase-level stories — each one a deliverable unit you can feed to `/bob-identify --story <path>`. Supports `--refactor [path]` for pure refactor work (α→γ improvement units) and auto-detects "feature + refactor" mixed mode when survey's 4th factor (前置重构量) is Medium/Hard. Output: `docs/bob/02-stories-*.md` index + `docs/bob/02-stories/<n>-<slug>.md` per-story files.

#### 🔍 `/bob-identify <business description>` (or `--refactor` / new-feature description)
```

- [ ] **Step 3: Verify**

Run: `grep -n '^#### ' /Users/xiaojin/workshop/run-bob/README.md | head -7`

Expected: five `####` skill headers — `/bob-survey`, `/bob-stories`, `/bob-identify`, `/bob-onion`, `/bob-spec` (in that order).

Run: `cd /Users/xiaojin/workshop/run-bob && cargo test 2>&1 | tail -3` — confirm still 29 passing.

- [ ] **Step 4: Commit**

```bash
cd /Users/xiaojin/workshop/run-bob && git add README.md && git -c commit.gpgsign=false commit -m "$(cat <<'EOF'
docs(readme): document /bob-stories as the new phase-1 skill

Renames "four skills" → "five skills" and adds a /bob-stories
subsection covering when it triggers (Medium/Hard after survey),
what it produces (index + per-story files), and the --refactor
mode for pure refactor work.
EOF
)"
```

---

## Final Verification

After all tasks committed:

- [ ] **Run the full suite**

Run: `cd /Users/xiaojin/workshop/run-bob && cargo test 2>&1 | tail -5`

Expected: **29 passed; 0 failed** (27 prior + 2 new).

- [ ] **Smoke-test the binary**

```bash
cd /Users/xiaojin/workshop/run-bob && cargo build --release 2>&1 | tail -1
tmp=$(mktemp -d)
./target/release/run-bob init --dir "$tmp" >/dev/null
ls "$tmp/.claude/skills/" | sort
test -f "$tmp/.claude/skills/bob-stories/SKILL.md" && echo "✓ bob-stories installed"
grep -c "前置重构量" "$tmp/.claude/skills/bob-survey/SKILL.md" && echo "✓ survey has 前置重构量"
grep -c "/bob-stories" "$tmp/.claude/skills/bob-survey/SKILL.md" && echo "✓ survey points to stories"
grep -c "/bob-stories" "$tmp/.claude/skills/bob-identify/SKILL.md" && echo "✓ identify references stories"
grep -c "\\-\\-story" "$tmp/.claude/skills/bob-identify/SKILL.md" && echo "✓ identify documents --story"
./target/release/run-bob status --dir "$tmp" 2>&1 | tail -3
rm -rf "$tmp"
```

Expected: 5 skills installed, all four ✓ markers print, status reports "harness is complete".

- [ ] **Smoke-test upgrade**

```bash
tmp=$(mktemp -d)
./target/release/run-bob init --dir "$tmp" >/dev/null
./target/release/run-bob upgrade --dir "$tmp" 2>&1 | tail -3
rm "$tmp/.claude/skills/bob-stories/SKILL.md"
./target/release/run-bob upgrade --dir "$tmp" 2>&1 | tail -3
test -f "$tmp/.claude/skills/bob-stories/SKILL.md" && echo "✓ upgrade reinstalled bob-stories"
rm -rf "$tmp"
```

Expected: first upgrade no-op; second upgrade installs 1 file.

- [ ] **`--minimal` smoke test**

```bash
tmp=$(mktemp -d)
./target/release/run-bob init --minimal --dir "$tmp" >/dev/null
ls "$tmp/.claude/skills/" | sort
test -f "$tmp/.claude/skills/bob-stories/SKILL.md" && echo "✓ bob-stories in --minimal"
rm -rf "$tmp"
```

Expected: 5 skills installed, bob-stories included.

- [ ] **Spec coverage**

| Spec section | Implemented in |
|---|---|
| §1.3 (a) survey 改造 | Task 1 |
| §1.3 (b) bob-stories 新增 | Task 2 |
| §1.3 (c) bob-identify 微调 | Task 3 |
| §2 第 4 因子 + 矩阵 | Task 1 |
| §3 CLI 表面 + 自动检测 | Task 2 |
| §4 输出与文件布局(汇总 + 明细) | Task 2 |
| §5 5 个 Stage + Q1/Q2 三段式 | Task 2 |
| §6 bob-identify 集成 + --story | Task 3 |
| §7 测试 token-only | Task 1/2/3 tests |
| §8 不动 ARCHITECTURE.md | (No task — explicit non-change) |
| §9 决策记录 | (In spec only) |
| §10 实施草图 | Tasks 1-3 (this plan) |

No gap.
