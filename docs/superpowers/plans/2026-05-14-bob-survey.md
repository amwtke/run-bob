# bob-survey Skill Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Ship `/bob-survey <需求>` as phase 0 of the bob workflow — a TL-style intake skill that classifies the repo state (G/β/γ), scores the architecture across 6 bob-specific dimensions (× 0-20 = 100), judges requirement difficulty by a 3-factor rubric, and emits a 3-tier recommendation (🟢/🟡/🔴). Produces `docs/bob/00-survey-<slug>-<date>.md` + appends to `ARCHITECTURE.md §12 架构体检记录`. `bob-identify` gets a soft prompt that asks whether survey was run.

**Architecture:** Pure template additions — one new skill markdown at `src/templates/skills/bob-survey.md`, one new `HARNESS_ASSETS` entry in `src/assets.rs`, additive edits to `bob-identify.md` (soft survey prompt) and `ARCHITECTURE.md` (empty §12 header). No new Rust code, no new crate deps, no `Cargo.toml` change. Verified by integration tests that check key tokens in the installed skill files (same pattern as existing `init_creates_bob_identify_skill` etc.).

**Tech Stack:** Same as run-bob (Rust 1.75+, clap, anyhow, colored, tempfile). All content is Markdown.

**Spec:** [`docs/superpowers/specs/2026-05-14-bob-survey-design.md`](../specs/2026-05-14-bob-survey-design.md)

**Out of scope (deferred per spec §1.3):** phase 1 (`/bob-stories`), phase 2 (`/bob-nfr`). Detailed integration/fixture testing per spec §9.

---

## File Structure

| File | Action | Responsibility |
|---|---|---|
| `src/templates/skills/bob-survey.md` | **Create** | The complete `/bob-survey` skill prompt body (YAML frontmatter + workflow stages + scoring rubric + recommendation matrix + output template) |
| `src/assets.rs` | **Modify** | Append one entry to `HARNESS_ASSETS` for the new skill (`upgrade_safe: true`, `included_in_minimal: true`) |
| `src/templates/root/ARCHITECTURE.md` | **Modify** | Append empty `## 9. 架构体检记录` section as a stable insertion point for survey rows |
| `src/templates/skills/bob-identify.md` | **Modify** | Insert a "have you run /bob-survey?" soft-prompt block near the top (after "## 触发", before "## 目标") |
| `tests/integration.rs` | **Modify** | Add 3 new integration tests (1 per modified/created template file) |

**Untouched** (plan must NOT modify these):
- `Cargo.toml`
- `src/main.rs`, `src/lib.rs`, `src/commands/*`
- `src/templates/skills/bob-onion.md`, `src/templates/skills/bob-spec.md`
- `src/templates/root/CLAUDE.md`, `src/templates/root/README-RUN-BOB.md`
- `src/templates/root/UseCase.java`, `src/templates/root/TransactionalUseCaseDecorator.java`
- `src/templates/root/CleanArchitectureTest.java`

---

## Architectural Notes for the Engineer

Read these before starting.

### 1. The skill body is a prompt for Claude, not Rust code

`run-bob` ships templates as `include_str!`-embedded markdown strings. The CLI never parses or executes them — it installs them at `<target>/.claude/skills/<name>/SKILL.md`. When the user invokes `/bob-survey` inside a Claude Code session in that target project, Claude reads that file and uses it as a system prompt for that conversation. So:

- **Quality of writing matters more than code correctness.** The skill body's job is to drive a high-quality conversation between Claude and the user.
- **Test what's testable** — we verify the *installed file* contains key tokens (the conversation contract). We do NOT and CANNOT auto-test the actual LLM behavior. That's deferred per spec §9.

### 2. The HARNESS_ASSETS entry inherits the upgrade pipeline

Adding the entry with `upgrade_safe: true` means:
- `run-bob init` will install it
- `run-bob status` will check it (drift guard tests already enforce this)
- `run-bob upgrade` will treat it as upgrade-safe (sync embedded → on-disk when version changes)
- The drift-guard test from the `upgrade` work (`upgrade_safe_field_matches_category_policy`) will enforce `Category::Skill` → `upgrade_safe == true`

You get all of this for free by adding one struct literal to the array.

### 3. ARCHITECTURE.md §12 is a stable insertion point

The survey skill needs a deterministic place to append rows. By shipping an empty `## 9. 架构体检记录` header with a table header row in the template, we guarantee:
- Every fresh `run-bob init` creates a project with §12 ready
- Existing projects can re-run `run-bob upgrade` — wait, no: ARCHITECTURE.md is `upgrade_safe: false`. So existing projects won't get §12 from upgrade. The survey skill's Stage 5 needs to handle the "no §12 yet" case (append section) AND the "§12 exists" case (append row).

This is already specified in spec §7.2 (the skill's Stage 5 handles "section absent" by appending a new one; "section present" by appending a row).

**Note on section number:** spec §7.2 referred to "§9" but `ARCHITECTURE.md` already has `## 9. ArchUnit 作用域`. We use the next available number, **§12**, after `## 11. 下一步`. This is a documentation-only change vs. spec; the behavior is identical (append to dedicated section).

### 4. Existing skill style is the reference

Read `src/templates/skills/bob-identify.md`, `bob-onion.md`, `bob-spec.md` before writing `bob-survey.md`. The three of them establish:
- YAML frontmatter shape (name + multi-line description with trigger conditions + natural-language triggers)
- 三段式提问规约 block format
- Stage-based workflow structure
- Markdown heading conventions in Chinese

`bob-survey.md` should feel like a fourth sibling — same voice, same conventions.

### 5. Tests check token presence, not LLM behavior

Look at `init_creates_bob_identify_skill` in `tests/integration.rs`:

```rust
for token in &[
    "5 问决策树",
    "Q1", "Q2", "Q3", "Q4", "Q5",
    "G", "B1", "B2",
    "推测", "推荐", "清洁孤岛",
] {
    assert!(content.contains(token), "bob-identify must mention {}", token);
}
```

That's the pattern. We don't run an LLM in tests — we assert the skill text contains the load-bearing words. Use the same approach.

---

## Task 1: Create `bob-survey.md` + register in HARNESS_ASSETS + token test

**Files:**
- Create: `src/templates/skills/bob-survey.md`
- Modify: `src/assets.rs`
- Modify: `tests/integration.rs` (append)

**Goal:** Ship the new skill template + wire it into the asset registry. Test asserts the installed file contains key load-bearing tokens.

- [ ] **Step 1: Write the failing token test**

Append to `/Users/xiaojin/workshop/run-bob/tests/integration.rs`:

```rust
#[test]
fn init_creates_bob_survey_skill() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let target = tmp.path();
    std::process::Command::new(run_bob_bin())
        .args(["init", "--dir"])
        .arg(target)
        .status()
        .expect("init");

    let p = target.join(".claude").join("skills").join("bob-survey").join("SKILL.md");
    assert!(p.is_file(), "bob-survey SKILL.md missing at {}", p.display());
    let content = std::fs::read_to_string(&p).unwrap();

    // Frontmatter contract
    assert!(content.starts_with("---"), "must start with YAML frontmatter");
    assert!(content.contains("name: bob-survey"), "frontmatter name");
    assert!(content.contains("description:"), "frontmatter description");

    // Load-bearing tokens — the conversation contract
    for token in &[
        // Workflow
        "/bob-survey",
        "三段式",
        "推测",
        "推荐选择",
        // Three repo states
        "G(绿地)",
        "β(棕地未跑过 bob)",
        "γ(成熟 bob)",
        // 6 scoring dimensions
        "Entity 纯度",
        "UseCase 纯度",
        "端口位置",
        "状态机位置",
        "@Transactional 唯一",
        "FORBIDDEN_IN_INNER",
        // Difficulty rubric
        "跨环数",
        "状态机增量",
        "legacy 复用",
        "Easy",
        "Medium",
        "Hard",
        // Recommendation matrix
        "🟢",
        "🟡",
        "🔴",
        // Output schema
        "docs/bob/00-survey-",
        "ARCHITECTURE.md",
        "§12",
        // Soft handoff to identify
        "/bob-identify",
    ] {
        assert!(content.contains(token), "bob-survey must mention {}", token);
    }
}
```

- [ ] **Step 2: Run the test (must fail)**

Run: `cd /Users/xiaojin/workshop/run-bob && cargo test --test integration init_creates_bob_survey_skill 2>&1 | tail -20`

Expected: compile error (`run_bob` not available — wait, `tests/integration.rs` already uses the run-bob binary as a subprocess, so this should compile; the assertion will fail because the file doesn't exist yet) OR runtime failure on `bob-survey SKILL.md missing`.

- [ ] **Step 3: Create the skill template file**

Create `/Users/xiaojin/workshop/run-bob/src/templates/skills/bob-survey.md` with the following exact content:

````markdown
---
name: bob-survey
description: |
  触发条件:用户输入 /bob-survey <需求一句话或几段>,
  或 /bob-survey --archcheck <path>(消化已有 archcheck 报告作参考),
  或 /bob-survey --no-record(跑完不写 ARCHITECTURE.md §12),
  或 /bob-survey --refresh(已有 00-survey-*.md 时强制重跑)。

  在跑 /bob-identify 之前做一道 TL 接需求动作:对当前仓库做架构体检
  (6 个 Bob 独有维度 × 0-20 = 100 分),对新需求做难度判定
  (跨环数 / 状态机增量 / legacy 复用 三因子),结合两者给 3 档
  落地建议(🟢 直接接 / 🟡 准备一下再接 / 🔴 先重构再接)。
  产出 docs/bob/00-survey-<slug>-<date>.md 与 ARCHITECTURE.md §12
  体检记录追加一行。不写代码、不出 spec。

  适用于 Bob 4 环 Clean Architecture 工作流的 phase 0:接需求时
  先评估底子能不能接。当用户说"接需求前先体检"、"现在能不能接
  这个需求"、"这个需求要不要先重构"、"看一下我现在的底子"时
  也应触发此技能。
---

# Bob Survey Skill

## 触发

```
/bob-survey <需求一句话或几段>     # 主入口
/bob-survey --archcheck <path>     # 消化已有 archcheck 报告作参考维度
/bob-survey --no-record            # 跑完不写 ARCHITECTURE.md §12
/bob-survey --refresh              # 已有 00-survey-*.md 时强制重跑
```

或自然语言触发:"接需求前先体检"、"现在能不能接这个需求"、"这个需求要不要先重构"、"看一下我现在的底子"。

## 前置条件

- **必须带需求**。无需求传入 → 拒绝运行,提示用户至少给一段需求描述。难度判定从需求语义出发,机器算不准。
- 项目位于 git 仓库内(用于读取 `archcheck-report-*.md` / ARCHITECTURE.md 等文件)。

## 提问规约(强制三段式)

任何需要用户选择的问题,**必须**按下面三段式输出。**禁止**抛开放问题。

格式:

> **[问题序号] [问题]**
>
> **推测**:<你的判断,基于上下文的最优解>
> **理由**:<一句话,为什么这么推测——引用代码事实/Bob 原则/常见模式>
> **推荐选择**:`<具体一个选项>`
>
> 是否同意?(回"是"走推荐;回"否,理由是 ..."重判;回"否,我选 X"切到 X)

## 目标

产出一份**架构体检报告 + 需求难度评估 + 落地建议**,让用户在跑 /bob-identify 前知道:

1. 当前仓库是 G/β/γ 哪一档
2. 这个需求是 Easy/Medium/Hard
3. 推荐 🟢/🟡/🔴 三档之一(直接接 / 先做几个准备 / 先重构再接)

**不写代码、不出 spec**。只回答一个问题:**这个需求现在能不能接,怎么接最稳?**

## 工作流(5 个 Stage)

```
Stage 0. 仓库状态判定(G / β / γ)
Stage 1. 6 维度评分(β/γ);绿地跳过
Stage 2. 需求难度三因子判定
Stage 3. 推荐矩阵 → 🟢/🟡/🔴
Stage 4. 写 docs/bob/00-survey-<slug>-<date>.md
Stage 5. 追加 ARCHITECTURE.md §12 一行(除非 --no-record)
```

---

## Stage 0. 仓库状态判定

| 状态 | 判定 | 后续 |
|---|---|---|
| **G(绿地)** | 无 `src/main/java` 或目录为空 | 跳过 Stage 1,直接做 Stage 2-3 |
| **β(棕地未跑过 bob)** | 有 `src/main/java`,但无 ARCHITECTURE.md 或 §4-§7 是占位符 | 跑 Stage 1(预期低分) |
| **γ(成熟 bob)** | 有 `src/main/java` + ARCHITECTURE.md §4-§7 填好 + `.claude/skills/bob-*` 存在 | 跑 Stage 1(预期高分) |

判定 sentinels(无 LLM judgment):

- `ls src/main/java/` 是否存在且非空 → G 与否
- `grep -c '^## 4\\.' ARCHITECTURE.md` 与 §4 段下是否有非占位符内容 → β vs γ
- `ls .claude/skills/bob-*` 是否齐 → 辅助判 γ

向用户**三段式**通报判定结果。

---

## Stage 1. 6 维度评分(β/γ)

每维度 0-20 分,总分 100。**禁止只给分数不给证据**:每项必带 ≤ 3 行的 file:line + 一句简评。

### 维度 1: Entity 纯度(0-20)

```bash
grep -rn 'org\.springframework\|jakarta\.persistence\|lombok\|org\.slf4j' \
  src/main/java/com/example/*/entity/ 2>/dev/null
```

- 0 违规 → 20 分
- 每 1 个 file 出现违规扣 4 分(下限 0)

### 维度 2: UseCase 纯度(0-20)

```bash
grep -rn 'org\.springframework\|jakarta\.persistence\|lombok\|org\.slf4j' \
  src/main/java/com/example/*/usecase/ \
  --exclude-dir=port 2>/dev/null
```

- 同上规则

### 维度 3: 端口位置(0-20)

列出所有 `*Repository` / `*Port` / `*Gateway` 接口(grep + 头文件扫描),看它们落在 `usecase/port/` 还是 `adapter/`。

- usecase/port 占比 ≥ 80% → 20 分
- 占比每降 5% 扣 1 分(下限 0)

### 维度 4: 状态机位置(0-20)

抽 3-5 个最关键 Entity 的状态修改方法(如 `confirm()` / `cancel()` / `pay()`),看它们落在 entity 包内还是 service 包内。

- entity 内方法数 > service 内方法数 → 20 分
- 1:1 → 10 分
- entity 内 < service 内 → 5 分
- 完全在 service 内 → 0 分

附 3-5 行证据(file:line)。

### 维度 5: @Transactional 唯一性(0-20)

```bash
grep -rn '@Transactional' src/main/java/ 2>/dev/null
```

- 仅在 `shared/framework/transaction/` → 20 分
- 每多 1 个文件扣 5 分(下限 0)

### 维度 6: FORBIDDEN_IN_INNER 违规(0-20)

读 `src/test/java/architecture/CleanArchitectureTest.java` 的 `FORBIDDEN_IN_INNER` 数组,对 `entity/` 和 `usecase/`(排除 `usecase/port/`)做静态扫描。

- 0 违规 → 20 分
- 每 5 个 file 违规扣 1 分(下限 0)

### 与 archcheck 报告的关系(soft 参考)

若项目内存在 `archcheck-report-*.md` 或用户传 `--archcheck <path>`,**读取**并作为第 7 个**参考**维度,**不计入总分**,只在产出报告的附录里展开"参考 archcheck 报告:...一行"。

### 评分汇总

| 维度 | 分 | 证据 |
|---|---|---|
| Entity 纯度 | X | ... |
| UseCase 纯度 | X | ... |
| 端口位置 | X | ... |
| 状态机位置 | X | ... |
| @Transactional 唯一 | X | ... |
| FORBIDDEN 违规 | X | ... |
| **总分** | **X/100** |  |

---

## Stage 2. 需求难度三因子判定

LLM 三段式追问用户得出三因子等级。

### 因子 1: 跨环数

> **Q1: 这个需求需要修改 / 新增几个 UseCase?**
>
> **推测**:<从需求描述里数动词,推断 UseCase 数量>
> **理由**:<一句话>
> **推荐选择**:`Easy` / `Medium` / `Hard`
>
> 标准:
> - Easy = 1 个 UseCase
> - Medium = 2-3 个 UseCase + 可能需要新端口
> - Hard = 跨 BC / 新 Adapter family / 大幅扩端口

### 因子 2: 状态机增量

> **Q2: 这个需求会让任何 Entity 多几个新状态 / 新转移?**
>
> **推测**:<基于需求语义>
> **理由**:<一句话>
> **推荐选择**:`Easy` / `Medium` / `Hard`
>
> 标准:
> - Easy = 0 个新状态 / 0 个新转移
> - Medium = 1-2 个新状态 / 新转移
> - Hard = 多状态机交互 / saga / 分布式事务

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

- 任一因子 Hard → 总评 **Hard**
- 否则 ≥ 2 个 Medium → 总评 **Medium**
- 否则 → **Easy**

---

## Stage 3. 推荐矩阵

### 绿地(G)

| 难度 | 推荐 | 备注 |
|---|---|---|
| Easy | 🟢 直接 `/bob-identify <需求>`(G 模式) |  |
| Medium | 🟢 直接 `/bob-identify <需求>`(G 模式) |  |
| Hard | 🟢 直接 `/bob-identify <需求>`(G 模式) | 附加提示:建议拆 story(phase 1,未实施) |

### 棕地(β / γ)3×3

| 评分 \\ 难度 | Easy | Medium | Hard |
|---|---|---|---|
| **80-100(γ 健康)** | 🟢 `/bob-identify` | 🟢 `/bob-identify`(B2 模式) | 🟡 B2 清洁孤岛;或先 `/bob-onion --refresh` 增端口 |
| **60-79(β 可接受)** | 🟢 `/bob-identify`(B2 模式) | 🟡 B2 清洁孤岛 + 提前列 ACL 表 | 🔴 先 `/bob-onion --refactor` 出三动作改造计划 |
| **0-59(α 烂底子)** | 🟡 警告:能做但债会变重;建议 B2 + 隔离严格 | 🔴 先重构再接 | 🔴 拒绝接需求;先 B1 全量重构;给"必须先改完哪 5 个东西"的清单 |

每个格子在产出报告里展开为 3 行:

- **推荐的下一步命令**:具体的 `/bob-...` 调用
- **一句话理由**:为什么是这个推荐
- **风险提示**:若忽略本建议直接接,会出现 X(用 TL 口气)

---

## Stage 4. 写产出报告

路径:`docs/bob/00-survey-<slug>-<YYYYMMDD>.md`

`<slug>` 由需求一行话生成(3-5 个汉字 / 英文 kebab),`<YYYYMMDD>` 是 UTC 日期。

模板:

```markdown
# 架构体检 · <需求一行话>
日期 · <YYYY-MM-DD> · 状态 · <G/β/γ> · 总分 <X>/100 · 需求难度 · <Easy/Medium/Hard> · 推荐 · <🟢/🟡/🔴 标题>

## 1. 仓库状态
<G/β/γ> · <证据:目录存在性 / ARCHITECTURE.md §4-§7 填充状态 / 距上次 onion 多少天>

## 2. 评分明细
<同 Stage 1 表格;绿地此节写"(绿地,跳过评分)">

## 3. 需求难度三因子
跨环数 · <Easy/Medium/Hard>(证据)
状态机增量 · <Easy/Medium/Hard>(证据)
legacy 复用 · <Easy/Medium/Hard>(证据)
→ 总评 **<Easy/Medium/Hard>**

## 4. 推荐
<🟢/🟡/🔴 标题>
理由:...
风险:若忽略本建议直接接,...

## 5. 下一步
推荐命令:`/bob-identify <需求> [--acl ...]`
```

---

## Stage 5. ARCHITECTURE.md §12 体检记录

打开 `ARCHITECTURE.md`,找到 `## 9. 架构体检记录` 段:

- **若不存在**(老项目尚未跑过 survey),在文件末尾追加:

  ```markdown

  ## 9. 架构体检记录
  | 日期 | 状态 | 总分 | 需求 | 难度 | 推荐 | 详报 |
  |---|---|---|---|---|---|---|
  | <YYYY-MM-DD> | <G/β/γ> | <X> | <需求一行话> | <Easy/Medium/Hard> | <🟢/🟡/🔴 标题> | docs/bob/00-survey-<slug>-<YYYYMMDD>.md |
  ```

- **若已存在 §12 段**(新项目模板已 ship 空表头),在表格末尾追加一行

`--no-record` 时跳过此 Stage。

绿地项目此节追加内容简化:总分填 "N/A"。

---

## TL 风对话

在体检完成后,**主动 raise concern**(像真 TL 一样):

- "你这个需求看起来 Medium,但我注意到 6 维度里 X 还在 Y,会被卡住。这是不是要先单独修?"
- 给推荐时**明确说出代价**:"你也可以直接接,但代价是 [预测后果];好处是 [加速度]。"

用户回"否"或"我先这样" → 尊重决定,但在报告末尾追加一行 "用户选择忽略推荐:..."。

---

## 与 bob-identify 的关系

跑完 `/bob-survey` 后输出"下一步"命令,由用户自行决定是否执行 `/bob-identify`。**不自动调用** `/bob-identify`。

`bob-identify` 在启动时若检测到无 `docs/bob/00-survey-*.md` 或最新一份距今 > 7 天,会三段式追问是否先跑 survey(见 `bob-identify` skill 的描述)。
````

(End of `bob-survey.md` content. Make sure the closing ` ``` ` markers in the bash blocks are present.)

- [ ] **Step 4: Register the new skill in `HARNESS_ASSETS`**

In `/Users/xiaojin/workshop/run-bob/src/assets.rs`, find the `// --- Skills (installed even in --minimal) ---` block. Currently it contains 3 entries (bob-identify, bob-onion, bob-spec). Append a fourth entry **between the last skill (bob-spec) and the next category comment** (`// --- Harness documents (skipped in --minimal) ---`).

Find this line:

```rust
    Asset {
        rel_path: &[".claude", "skills", "bob-spec", "SKILL.md"],
        content: include_str!("templates/skills/bob-spec.md"),
        category: Category::Skill,
        included_in_minimal: true,
        upgrade_safe: true,
    },
    // --- Harness documents (skipped in --minimal) ---
```

Insert the new entry between them so it becomes:

```rust
    Asset {
        rel_path: &[".claude", "skills", "bob-spec", "SKILL.md"],
        content: include_str!("templates/skills/bob-spec.md"),
        category: Category::Skill,
        included_in_minimal: true,
        upgrade_safe: true,
    },
    Asset {
        rel_path: &[".claude", "skills", "bob-survey", "SKILL.md"],
        content: include_str!("templates/skills/bob-survey.md"),
        category: Category::Skill,
        included_in_minimal: true,
        upgrade_safe: true,
    },
    // --- Harness documents (skipped in --minimal) ---
```

- [ ] **Step 5: Run the new test (must pass)**

Run: `cd /Users/xiaojin/workshop/run-bob && cargo test --test integration init_creates_bob_survey_skill 2>&1 | tail -15`

Expected: `test init_creates_bob_survey_skill ... ok`. If any token assertion fires, the most likely cause is a copy-paste tweak in Step 3 — re-check the missing token against the actual file content (the test message tells you which token).

- [ ] **Step 6: Run the full suite**

Run: `cd /Users/xiaojin/workshop/run-bob && cargo test 2>&1 | tail -10`

Expected: **25 passed; 0 failed** (24 prior from `upgrade` work + 1 new).

Pay attention to two pre-existing tests that may now need scrutiny:
- `status_checks_every_file_init_writes` — verifies every file `init` writes is also checked by `status`. Adding a new asset to `HARNESS_ASSETS` automatically wires both, so this should still pass.
- `upgrade_safe_field_matches_category_policy` — the drift guard. The new entry has `category: Category::Skill` and `upgrade_safe: true`, so it satisfies the policy.

If either fails, do not "fix" the drift guard test — adjust the new asset entry instead.

- [ ] **Step 7: Commit**

```bash
cd /Users/xiaojin/workshop/run-bob && git add src/templates/skills/bob-survey.md src/assets.rs tests/integration.rs && git -c commit.gpgsign=false commit -m "$(cat <<'EOF'
feat(skills): add /bob-survey skill (phase 0 architecture intake)

New /bob-survey <requirement> skill that classifies repo state
(G/β/γ), scores the architecture across 6 bob-specific dimensions
(0-20 each, total 100), judges requirement difficulty by a 3-factor
rubric (cross-rings, state-machine delta, legacy reuse), and emits a
3-tier recommendation (🟢 direct / 🟡 prepare / 🔴 refactor first).

Wires into HARNESS_ASSETS as upgrade-safe so it flows through
init/status/upgrade automatically. ARCHITECTURE.md §12 hand-off and
bob-identify soft prompt land in later tasks.
EOF
)"
```

---

## Task 2: Append §12 to `ARCHITECTURE.md` template

**Files:**
- Modify: `src/templates/root/ARCHITECTURE.md`
- Modify: `tests/integration.rs` (append)

**Goal:** Ship a stable §12 insertion point in fresh projects so survey can append rows deterministically. The survey skill itself also handles the "§12 missing" case (existing projects) — this task only covers fresh-init.

- [ ] **Step 1: Read the current template tail to know where to append**

Run: `tail -30 /Users/xiaojin/workshop/run-bob/src/templates/root/ARCHITECTURE.md`

Expected: the file ends with some `## 8` or earlier section. Note the last section number actually present and confirm there is no existing `## 9` section.

If §12 already exists, STOP — the plan needs revision before continuing.

- [ ] **Step 2: Write the failing test**

Append to `/Users/xiaojin/workshop/run-bob/tests/integration.rs`:

```rust
#[test]
fn init_creates_architecture_md_with_section_9() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let target = tmp.path();
    std::process::Command::new(run_bob_bin())
        .args(["init", "--dir"])
        .arg(target)
        .status()
        .expect("init");

    let p = target.join("ARCHITECTURE.md");
    assert!(p.is_file());
    let content = std::fs::read_to_string(&p).unwrap();

    // §12 header + empty table header must be shipped so /bob-survey
    // can append rows deterministically.
    assert!(
        content.contains("## 9. 架构体检记录"),
        "ARCHITECTURE.md must ship empty §12 header"
    );
    for col in &["日期", "状态", "总分", "需求", "难度", "推荐", "详报"] {
        assert!(
            content.contains(col),
            "ARCHITECTURE.md §12 must have column header {}",
            col
        );
    }
}
```

- [ ] **Step 3: Run the test (must fail)**

Run: `cd /Users/xiaojin/workshop/run-bob && cargo test --test integration init_creates_architecture_md_with_section_9 2>&1 | tail -10`

Expected: failure on `must ship empty §12 header` (the template doesn't have §12 yet).

- [ ] **Step 4: Insert §12 before the trailing footer**

The current template ends with `## 11. 下一步` followed by a `---` separator and `*Managed by run-bob + /bob-onion skill.*`. The new §12 must land **between §11's content and the `---` footer**, not after it.

Use `Edit` on `/Users/xiaojin/workshop/run-bob/src/templates/root/ARCHITECTURE.md`:

`old_string`:

```
- [ ] 启动 Superpowers brainstorming 决定栈细节(若 CLAUDE.md `## 技术栈约定` 段未填)

---
*Managed by run-bob + /bob-onion skill.*
```

`new_string`:

```
- [ ] 启动 Superpowers brainstorming 决定栈细节(若 CLAUDE.md `## 技术栈约定` 段未填)

## 12. 架构体检记录

> 由 `/bob-survey` 自动追加。每次 `/bob-survey <需求>` 跑完会在此表追加一行。
> `bob-onion` / `bob-spec` 可参考最近一次结论。

| 日期 | 状态 | 总分 | 需求 | 难度 | 推荐 | 详报 |
|---|---|---|---|---|---|---|

---
*Managed by run-bob + /bob-onion skill.*
```

That trailing pipe row with no content rows is the empty table header — `/bob-survey` appends rows below it. If the Edit fails on uniqueness, the template was modified after this plan was written; STOP and report.

- [ ] **Step 5: Run the test (must pass)**

Run: `cd /Users/xiaojin/workshop/run-bob && cargo test --test integration init_creates_architecture_md_with_section_9`

Expected: pass.

- [ ] **Step 6: Run the full suite**

Run: `cd /Users/xiaojin/workshop/run-bob && cargo test 2>&1 | tail -5`

Expected: **26 passed; 0 failed** (25 prior + 1 new).

- [ ] **Step 7: Commit**

```bash
cd /Users/xiaojin/workshop/run-bob && git add src/templates/root/ARCHITECTURE.md tests/integration.rs && git -c commit.gpgsign=false commit -m "$(cat <<'EOF'
feat(templates): add §12 体检记录 to ARCHITECTURE.md

Ships an empty §12 table header so /bob-survey can append rows
deterministically in fresh projects. The skill itself handles the
"§12 absent" case for existing projects.
EOF
)"
```

---

## Task 3: Soft survey prompt in `bob-identify.md`

**Files:**
- Modify: `src/templates/skills/bob-identify.md`
- Modify: `tests/integration.rs` (append)

**Goal:** Insert a "have you run /bob-survey?" soft prompt near the top of `bob-identify.md`. Not enforcing — Claude asks once, user can decline.

- [ ] **Step 1: Locate the insertion point in `bob-identify.md`**

Run: `grep -n '^## ' /Users/xiaojin/workshop/run-bob/src/templates/skills/bob-identify.md | head -10`

Expected: a listing of sections like:
```
LINE:## 触发
LINE:## 目标
LINE:## 提问规约(强制)
...
```

The insert goes between `## 触发` and `## 目标` (or whatever section follows "触发"). The exact section names may differ slightly — that's fine, just know where the first two `##` headers are.

- [ ] **Step 2: Write the failing test**

Append to `/Users/xiaojin/workshop/run-bob/tests/integration.rs`:

```rust
#[test]
fn bob_identify_mentions_survey_soft_prompt() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let target = tmp.path();
    std::process::Command::new(run_bob_bin())
        .args(["init", "--dir"])
        .arg(target)
        .status()
        .expect("init");

    let p = target.join(".claude").join("skills").join("bob-identify").join("SKILL.md");
    let content = std::fs::read_to_string(&p).unwrap();

    // The soft prompt must mention survey and the 7-day threshold.
    for token in &[
        "/bob-survey",
        "docs/bob/00-survey",
        "7 天",
        "soft",  // marker we'll include in the new section header
    ] {
        assert!(
            content.contains(token),
            "bob-identify must mention {} for survey integration",
            token
        );
    }
}
```

- [ ] **Step 3: Run the test (must fail)**

Run: `cd /Users/xiaojin/workshop/run-bob && cargo test --test integration bob_identify_mentions_survey_soft_prompt 2>&1 | tail -10`

Expected: failure on `bob-identify must mention /bob-survey for survey integration`.

- [ ] **Step 4: Read the current bob-identify content around the insertion point**

Run: `head -40 /Users/xiaojin/workshop/run-bob/src/templates/skills/bob-identify.md`

Note the exact text of the "## 触发" section (and what comes right after it) — you'll need to anchor the Edit on a unique snippet.

- [ ] **Step 5: Insert the soft-prompt block via `Edit`**

Use the `Edit` tool on `/Users/xiaojin/workshop/run-bob/src/templates/skills/bob-identify.md` with this exact anchor (confirmed unique on lines 31-33 of the current file):

`old_string`:

```
或自然语言触发:"做身份测试"、"区分核心和配件"、"这段代码哪些是核心哪些是框架"、"这个功能里什么是 Entity"。

## 目标
```

`new_string`:

```
或自然语言触发:"做身份测试"、"区分核心和配件"、"这段代码哪些是核心哪些是框架"、"这个功能里什么是 Entity"。

## 先检查 /bob-survey (soft 前置)

启动时若检测到下面任一条件,**三段式追问**用户是否先跑 `/bob-survey`:

- 项目内不存在 `docs/bob/00-survey-*.md`
- 最新一份 `00-survey-*.md` 距今 > 7 天

格式:

> **Q0:看起来你还没跑过 `/bob-survey`(或最近一次已超过 7 天),是否先体检一次?**
>
> **推测**:建议先跑。我可以直接做身份测试,但跳过体检直接接需求,γ 底子下问题不大,β/α 底子下大概率会拖垮新代码质量。
> **理由**:`/bob-survey` 会给出 6 维度评分 + 需求难度 + 三档建议,15 秒内告诉你"现在能不能接 / 要不要先做几个修补"。
> **推荐选择**:`先跑 /bob-survey <需求>`
>
> 是否同意?(回"是"→跳出 identify 让用户执行 survey;回"否"→继续做身份测试)

**不强制**。用户明确说"否"则照旧继续。这一节遵从 TL 风:提问而不阻塞。

## 目标
```

The match relies on the natural-language trigger line (unique to bob-identify) plus the blank line + `## 目标` heading. If the Edit fails on uniqueness, the file was modified after this plan was written — STOP and report rather than guessing a new anchor.

- [ ] **Step 6: Run the test (must pass)**

Run: `cd /Users/xiaojin/workshop/run-bob && cargo test --test integration bob_identify_mentions_survey_soft_prompt`

Expected: pass.

- [ ] **Step 7: Run the full suite**

Run: `cd /Users/xiaojin/workshop/run-bob && cargo test 2>&1 | tail -5`

Expected: **27 passed; 0 failed** (26 prior + 1 new).

Note: the existing `init_creates_bob_identify_skill` test checks for specific tokens like "5 问决策树", "Q1"-"Q5", "推测", "推荐". Your insertion must NOT remove any of those — the new soft-prompt section adds content, doesn't replace anything.

- [ ] **Step 8: Commit**

```bash
cd /Users/xiaojin/workshop/run-bob && git add src/templates/skills/bob-identify.md tests/integration.rs && git -c commit.gpgsign=false commit -m "$(cat <<'EOF'
feat(skills): bob-identify soft-prompts /bob-survey

Inserts a "have you run /bob-survey in the last 7 days?" check near
the top of bob-identify.md. Three-段式 prompt; user can decline and
continue with identify. Honors the TL style — ask, don't block.
EOF
)"
```

---

## Task 4: README mention (optional)

**Files:**
- Modify: `README.md`

**Goal:** Add one-paragraph mention of `/bob-survey` in the README so users discover it. The README already documents `/bob-identify`, `/bob-onion`, `/bob-spec`.

This task is **optional** — per spec §11 it's marked "可选". Skip if time-constrained.

- [ ] **Step 1: Locate the "The three skills" section**

Run: `grep -n '^### The three skills\|^#### ' /Users/xiaojin/workshop/run-bob/README.md | head -10`

Expected: a `### The three skills` heading followed by `#### 🔍 /bob-identify`, `#### 🧅 /bob-onion`, `#### 📝 /bob-spec` sub-sections.

- [ ] **Step 2: Rename the section and add /bob-survey**

Use Edit on `/Users/xiaojin/workshop/run-bob/README.md`.

First rename `### The three skills` → `### The four skills`:

```
old_string: ### The three skills
new_string: ### The four skills
```

Then add the new sub-section **immediately above** `#### 🔍 /bob-identify`:

```
old_string: #### 🔍 `/bob-identify <business description>` (or `--refactor` / new-feature description)
new_string: #### 🩺 `/bob-survey <requirement>` (phase 0 — TL intake)
Architectural health check + requirement difficulty + recommendation, before you even start identifying. Classifies the repo as G (greenfield) / β (brownfield no bob) / γ (mature bob), scores the architecture across 6 bob-specific dimensions (0-20 each, total 100), judges the requirement on 3 factors (cross-rings, state-machine delta, legacy reuse), and emits a 3-tier recommendation: 🟢 go ahead, 🟡 prepare some things first, 🔴 refactor before accepting. Output: `docs/bob/00-survey-*.md` + a row appended to `ARCHITECTURE.md §12 体检记录`. Run before `/bob-identify` (it'll soft-prompt you anyway).

#### 🔍 `/bob-identify <business description>` (or `--refactor` / new-feature description)
```

(The trailing blank line + next `####` header is preserved by the `new_string`.)

- [ ] **Step 3: Verify**

Run: `grep -n '^#### ' /Users/xiaojin/workshop/run-bob/README.md | head -6`

Expected: four `####` skill headers now — `/bob-survey`, `/bob-identify`, `/bob-onion`, `/bob-spec` (in that order).

Run: `cd /Users/xiaojin/workshop/run-bob && cargo test 2>&1 | tail -3` — confirm still 27 passing (no test touches README).

- [ ] **Step 4: Commit**

```bash
cd /Users/xiaojin/workshop/run-bob && git add README.md && git -c commit.gpgsign=false commit -m "$(cat <<'EOF'
docs(readme): document /bob-survey as the new phase-0 skill

Renames "three skills" → "four skills" and adds a /bob-survey
subsection covering repo-state classification, 6-dimension scoring,
3-factor difficulty rubric, and 3-tier recommendation.
EOF
)"
```

---

## Final Verification

After all tasks committed:

- [ ] **Run the full suite**

Run: `cd /Users/xiaojin/workshop/run-bob && cargo test 2>&1 | tail -5`

Expected (if Task 4 done): **27 passed; 0 failed**.
Expected (if Task 4 skipped): **27 passed; 0 failed** (Task 4 doesn't add tests).

- [ ] **Smoke-test the binary**

```bash
cd /Users/xiaojin/workshop/run-bob && cargo build --release 2>&1 | tail -1
tmp=$(mktemp -d)
./target/release/run-bob init --dir "$tmp" >/dev/null
ls "$tmp/.claude/skills/" | sort
test -f "$tmp/.claude/skills/bob-survey/SKILL.md" && echo "✓ bob-survey installed"
grep -c "^## 9\. 架构体检记录" "$tmp/ARCHITECTURE.md" && echo "✓ §12 present"
grep -c "/bob-survey" "$tmp/.claude/skills/bob-identify/SKILL.md" && echo "✓ identify references survey"
rm -rf "$tmp"
```

Expected:
```
bob-identify
bob-onion
bob-spec
bob-survey
✓ bob-survey installed
1
✓ §12 present
N  (some count > 0)
✓ identify references survey
```

- [ ] **Smoke-test upgrade**

```bash
tmp=$(mktemp -d)
./target/release/run-bob init --dir "$tmp" >/dev/null
# Sanity: upgrade should be no-op because everything is fresh
./target/release/run-bob upgrade --dir "$tmp" 2>&1 | tail -5
# Delete the new skill to simulate an upgrade scenario
rm "$tmp/.claude/skills/bob-survey/SKILL.md"
# Upgrade should reinstall it
./target/release/run-bob upgrade --dir "$tmp" 2>&1 | tail -5
test -f "$tmp/.claude/skills/bob-survey/SKILL.md" && echo "✓ upgrade reinstalled bob-survey"
rm -rf "$tmp"
```

Expected:
- First upgrade: "All upgrade-safe assets are up to date."
- Second upgrade (after deletion): summary shows "1 installed"
- Final check: `✓ upgrade reinstalled bob-survey`

- [ ] **Spec coverage verification**

| Spec section | Implemented in |
|---|---|
| §2 CLI (4 trigger forms) | Task 1 (frontmatter + 触发 section in skill) |
| §3 三态识别 (G/β/γ) | Task 1 (Stage 0 in skill) |
| §4 6-dimension scoring | Task 1 (Stage 1 + 评分汇总) |
| §5 3-factor difficulty | Task 1 (Stage 2) |
| §6 推荐矩阵 (3×3 + 绿地) | Task 1 (Stage 3) |
| §7.1 产出 docs/bob/00-survey-*.md | Task 1 (Stage 4 template) |
| §7.2 ARCHITECTURE.md §12 | Task 1 (Stage 5) + Task 2 (template seeds §12) |
| §8.1 bob-identify soft prompt | Task 3 |
| §8.2 bob-onion / bob-spec untouched | (No task — explicit non-change) |
| §12 Testing deferred | (Minimal token tests in Tasks 1-3 only) |

No spec section is uncovered.
