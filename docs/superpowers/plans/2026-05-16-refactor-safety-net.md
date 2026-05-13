# Refactor Test Safety Net Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Ship a three-layer test safety net across `bob-stories` (Stage 2.5 coverage audit + R0.x characterize stories), `bob-identify --refactor` (Step B1.0 soft gate), and `bob-spec --refactor` Template C (Step 0 full-branch requirement + R0 interlock). All checks operate at **branch granularity** (every `if/else`, `switch case`, `throw`, early `return` must have a test that hits it).

**Architecture:** Pure template additions/edits — three skill markdown files get targeted insertions and renames; the test file gets three new token-presence tests. No new Rust code, no `Cargo.toml` change, no new crate dependencies. The three skills interlock so there's no path through the workflow that skips all three checks.

**Tech Stack:** Same as run-bob (Rust 1.75+, clap, anyhow, colored, tempfile). All content is Markdown.

**Spec:** [`docs/superpowers/specs/2026-05-16-refactor-safety-net-design.md`](../specs/2026-05-16-refactor-safety-net-design.md)

**Out of scope:** Fixture-based behavior verification (deferred per spec §7). ARCHITECTURE.md unchanged. `/bob-onion` unchanged. `/bob-survey` unchanged.

---

## File Structure

| File | Action | Responsibility |
|---|---|---|
| `src/templates/skills/bob-stories.md` | **Modify** | Add Stage 2.5 to workflow summary + insert full Stage 2.5 section between Stage 2 and Stage 3 + add characterize R0 template to Stage 4 |
| `src/templates/skills/bob-identify.md` | **Modify** | Insert Step B1.0 (test coverage status check) at the start of "### 模式 B1(棕地全量重构)" section + add identity §8 warning slot |
| `src/templates/skills/bob-spec.md` | **Modify** | Template C: rename Step 1 → Step 0, upgrade to branch-coverage + R0 interlock language, renumber Step 2/3/4/5 → 1/2/3/4 |
| `tests/integration.rs` | **Modify** | Append 3 new token-presence tests |
| `README.md` | **Modify** *(optional Task 4)* | Mention the safety net in `/bob-stories` subsection |

**Untouched** (plan must NOT modify):
- `Cargo.toml`, `src/main.rs`, `src/lib.rs`, `src/commands/*`, `src/assets.rs` (no new asset)
- `src/templates/skills/bob-survey.md`, `bob-onion.md`
- `src/templates/root/*` (CLAUDE.md, ARCHITECTURE.md, README-RUN-BOB.md, *.java)

---

## Architectural Notes for the Engineer

Read these before starting.

### 1. Three skills, three insertions, one safety property

The spec's §2 three-layer table is the load-bearing diagram:

| Path | Activation point |
|---|---|
| survey → stories → identify --story → onion → spec | bob-stories **Stage 2.5** (early — R0 stories emerge at split time) |
| survey → identify --refactor (skip stories) | bob-identify **Step B1.0** (soft 三段式 gate) |
| Direct `/bob-spec --refactor <类>` | bob-spec Template C **Step 0** (fallback, was Step 1) |

Each task corresponds to one layer. They don't share code (it's all Markdown), but they share vocabulary — `characterize`, `全分支级`, `R0.x`, `branch`. Use the spec terms verbatim so cross-references inside the skill bodies don't drift.

### 2. Branch enumeration is well-defined

Per the spec, a "branch" = one of: `if`, `else if`, `else`, each `switch case`, `throw`, early `return`, or a key `&&` / `||` short-circuit. The R0.x story template lists branches as `B1, B2, ..., Bn`. Treat this as the canonical naming throughout all three skill bodies — don't invent alternates.

### 3. bob-spec Step renumbering must touch all references

Template C currently has Step 1, 2, 3, 4, 5. After this change:
- Step 1 → Step 0 (renamed + upgraded)
- Step 2 → Step 1
- Step 3 → Step 2
- Step 4 → Step 3
- Step 5 → Step 4

Make sure to find ALL textual references — the running prose, the Step headers, the test scenario references (e.g., "场景 5:回归基线" stays since scenarios are independent of step numbers).

### 4. Test token list is the contract

These tests assert key tokens are present in the installed skill files. They DON'T verify LLM behavior; they verify the prompt body says the right things. When you write a token, that exact substring must appear in the skill source. Spec §7 enumerates the tokens — use them verbatim.

### 5. No new HARNESS_ASSETS

Unlike phases 0 and 1, this work doesn't add a skill. The 3 existing skills (bob-stories, bob-identify, bob-spec) are already in `HARNESS_ASSETS` with `upgrade_safe: true`. `run-bob upgrade` will sync the edits to existing projects for free.

---

## Task 1: bob-stories Stage 2.5 + characterize R0 template

**Files:**
- Modify: `src/templates/skills/bob-stories.md`
- Modify: `tests/integration.rs`

**Goal:** Add Stage 2.5 (test coverage audit) between Stage 2 and Stage 3 in bob-stories; add a `characterize` R0 story template to Stage 4. Token test asserts the new content lands.

- [ ] **Step 1: Read the current Stage 2 / Stage 3 boundary to confirm anchors**

Run: `sed -n '105,135p' /Users/xiaojin/workshop/run-bob/src/templates/skills/bob-stories.md`

Expected:
- Line ~110: `## Stage 2. 三段式提拆法`
- Line ~123: `用户可"合并 X 进 Y"、"拆 X 为 X1+X2"、"丢掉 X"等指令调整。`
- Line ~125: `### 拆分原则`
- Line ~131: `## Stage 3. 三段式拆顺序与依赖`

If line numbers differ or content differs, STOP and report BLOCKED.

- [ ] **Step 2: Write the failing token test**

Append to `/Users/xiaojin/workshop/run-bob/tests/integration.rs`:

```rust
#[test]
fn bob_stories_mentions_test_coverage_stage() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let target = tmp.path();
    std::process::Command::new(run_bob_bin())
        .args(["init", "--dir"])
        .arg(target)
        .status()
        .expect("init");

    let p = target.join(".claude").join("skills").join("bob-stories").join("SKILL.md");
    let content = std::fs::read_to_string(&p).unwrap();

    for token in &[
        "Stage 2.5",
        "测试覆盖体检",
        "R0.",
        "characterize",
        "全分支覆盖",
        "未覆盖分支",
    ] {
        assert!(
            content.contains(token),
            "bob-stories must mention {} for safety net integration",
            token
        );
    }
}
```

- [ ] **Step 3: Run the test (must fail)**

Run: `cd /Users/xiaojin/workshop/run-bob && cargo test --test integration bob_stories_mentions_test_coverage_stage 2>&1 | tail -10`

Expected: failure on `bob-stories must mention Stage 2.5`.

- [ ] **Step 4: Update the workflow summary to include Stage 2.5**

The workflow ASCII block at line ~63-69 currently lists 5 stages. Replace with 6 stages.

`old_string`:

```
## 工作流(5 个 Stage)

```
Stage 0. 输入归并(读 survey + 需求 + --refactor flag)
Stage 1. 自动识别模式(feature / refactor / 混合)
Stage 2. 三段式提拆法(LLM 提 UseCase / 改造单元列表)
Stage 3. 三段式拆顺序与依赖
Stage 4. 写汇总索引 + 每个 story 明细
```
```

`new_string`:

```
## 工作流(5 个 Stage + 1 个 refactor 专属)

```
Stage 0. 输入归并(读 survey + 需求 + --refactor flag)
Stage 1. 自动识别模式(feature / refactor / 混合)
Stage 2. 三段式提拆法(LLM 提 UseCase / 改造单元列表)
Stage 2.5. 测试覆盖体检(全分支级,refactor / 混合模式专属)
Stage 3. 三段式拆顺序与依赖
Stage 4. 写汇总索引 + 每个 story 明细
```
```

- [ ] **Step 5: Insert Stage 2.5 section between Stage 2 and Stage 3**

The boundary is right after the `### 拆分原则` subsection of Stage 2 and right before `## Stage 3. 三段式拆顺序与依赖`. Find this snippet in `src/templates/skills/bob-stories.md`:

`old_string`:

```
### 拆分原则

- **feature stories**:1 个动词 = 1 个 UseCase = 1 story。如果两个动词共享 Entity 状态机但语义独立(approve vs reject),仍拆成两个 story。
- **refactor stories**:1 个原子改造 = 1 story。原子的定义:能在 1 个 PR 里干完,改完后所有现有测试仍绿,ArchUnit 通过。

---

## Stage 3. 三段式拆顺序与依赖
```

`new_string`:

```
### 拆分原则

- **feature stories**:1 个动词 = 1 个 UseCase = 1 story。如果两个动词共享 Entity 状态机但语义独立(approve vs reject),仍拆成两个 story。
- **refactor stories**:1 个原子改造 = 1 story。原子的定义:能在 1 个 PR 里干完,改完后所有现有测试仍绿,ArchUnit 通过。

---

## Stage 2.5. 测试覆盖体检(refactor / 混合模式专属,全分支级)

**触发**:仅 refactor / 混合模式。feature 纯新功能模式跳过本 Stage。

**目的**:在拆顺序之前,先对每个 refactor 单元的"被改方法"做全分支级覆盖审查;无覆盖 / 部分覆盖的方法自动产 R0.x 特征测试 story,排到所有 R_i 之前。

### Step A. 枚举分支

对每个 refactor 单元 R_i 的每个被改方法 m,LLM 读 m 源码,列出所有分支:`if / else / switch case / throw / 早 return / 关键 && 短路`,编号 `B1, B2, ..., Bn`。

### Step B. 映射测试

`grep -rn '<方法名>' src/test/java/ 2>/dev/null` 找出引用 m 的测试方法。读每个测试体,判定它覆盖了哪些分支(`Bk1, Bk2 ...`)。

### Step C. 决定 R0.x(三态)

| 测试状态 | R0.x 产物 |
|---|---|
| 方法 m **无任何测试** | `R0.x · 为 <类>.<m> 写全分支覆盖测试`(全部 B1..Bn) |
| 方法 m 有测试,**部分分支未覆盖** | `R0.x · 为 <类>.<m> 补未覆盖分支测试`(列出未覆盖的 Bk) |
| 方法 m 全分支已覆盖 | ✓ 不出 R0 |

### 示例输出

```
R1 · OrderService 状态机上提:
  - cancel() · 4 分支(B1..B4)
    · testCancelHappyPath:88 覆盖 B1, B3
    · ✗ B2(status=SHIPPED 拒绝)、B4(已 PaidNotShipped 警告)未覆盖
    → R0.1 · 为 OrderService.cancel 补未覆盖分支(B2, B4)
  - confirm() · 3 分支 · 无任何测试
    → R0.2 · 为 OrderService.confirm 写全分支覆盖测试(B1, B2, B3)

R2 · TransactionalDecorator 收敛:
  - apply()   · 5 分支 · 无任何测试 → R0.3 · 写全分支覆盖
  - rollback() · 3 分支 · 全覆盖 ✓

→ 生成 3 个 R0.x stories,排在 R1/R2 之前(R0.1 → R0.2 → R0.3 → R1 → R2)
```

### 三段式收敛

> **Q1.5: 接受这 N 个 R0.x stories?**
>
> **推测**:体检发现 N 个方法、X 个未覆盖分支。建议 N 个 R0.x stories 全部接受,先写测试再重构。
> **理由**:Michael Feathers《Legacy Code》第一条—没特征测试不要碰 legacy。全分支级是因为 happy path 测试容易让人误以为"覆盖了"。
> **推荐选择**:`接受 N 个 R0.x stories`
>
> 是否同意?(回"是"走推荐;回"否,合并 R0.1+R0.2 进一个 story"重判;回"否,只要 R0.2/R0.3 不要 R0.1"切到手动)

用户可调:合并、丢弃、把多个方法的 R0 合到 1 个 story。

---

## Stage 3. 三段式拆顺序与依赖
```

- [ ] **Step 6: Add characterize R0 template to Stage 4**

After Stage 4's refactor template (around line ~218-238 currently), find the closing fence of the refactor template and append the characterize template right before the next section "## 与 /bob-identify 的关系". Run:

`grep -n 'refactor 模板\|## 与 /bob-identify' /Users/xiaojin/workshop/run-bob/src/templates/skills/bob-stories.md`

You should see:
- A line referencing the refactor template (around line 218)
- `## 与 /bob-identify 的关系` (around line 242)

The refactor template's closing fence is the ` ```` ` immediately before `## 与 /bob-identify 的关系`. Apply:

`old_string`:

```
`/bob-identify --refactor src/main/java/.../service/OrderService.java`
```
````

---

## 与 /bob-identify 的关系
```

`new_string`:

```
`/bob-identify --refactor src/main/java/.../service/OrderService.java`
```
````

characterize 模板 `docs/bob/02-stories/R0.<x>-<method-kebab>.md`(由 Stage 2.5 产出):

```markdown
# Story R0.1 · 为 OrderService.cancel 补未覆盖分支测试
类型 · characterize · 优先级 · High · 依赖 · -

## 1. 目标
补齐 OrderService.cancel 未覆盖分支测试,作为后续重构(R1)的基线。
**不改任何生产代码**。

## 2. 当前分支盘点
- B1: status=NEW 正常取消 ✓ 已覆盖(testCancelHappyPath:88)
- B2: status=SHIPPED 拒绝取消 ✗ 未覆盖
- B3: status=PAID 退款分支 ✓ 已覆盖(testCancelHappyPath:88 内部)
- B4: status=PaidNotShipped 警告分支 ✗ 未覆盖

## 3. 改造范围
- src/test/java/.../service/OrderServiceTest.java(新增 2 个 test 方法:testCancelShipped / testCancelPaidNotShipped)
- 无生产代码改动

## 4. 验收
- 新增 2 个测试 → 全绿(测试反映现行为)
- 全分支覆盖:B1-B4 都至少有 1 个 test 命中
- commit message: `test: characterize OrderService.cancel uncovered branches B2/B4`

## 5. 下一步
完成后,R1(OrderService 状态机上提)可以开始
```

"全分支覆盖型" R0(全新无测试)使用同一模板,但 §2 当前分支盘点所有 B_i 都标 ✗,§3 改造范围写 `新增 N 个 test 方法`。

---

## 与 /bob-identify 的关系
```

- [ ] **Step 7: Run the token test (must pass)**

Run: `cd /Users/xiaojin/workshop/run-bob && cargo test --test integration bob_stories_mentions_test_coverage_stage 2>&1 | tail -10`

Expected: pass.

- [ ] **Step 8: Run the full suite**

Run: `cd /Users/xiaojin/workshop/run-bob && cargo test 2>&1 | tail -5`

Expected: **30 passed; 0 failed** (29 prior + 1 new).

The pre-existing `init_creates_bob_stories_skill` test (which checks for tokens like "Stage 0", "Stage 4", etc.) should still pass — we only added content.

- [ ] **Step 9: Commit**

```bash
cd /Users/xiaojin/workshop/run-bob && git add src/templates/skills/bob-stories.md tests/integration.rs && git -c commit.gpgsign=false commit -m "$(cat <<'EOF'
feat(stories): add Stage 2.5 全分支级 test coverage audit + R0.x characterize

In refactor / 混合 mode, after enumerating refactor units, bob-stories
now runs a branch-level coverage audit on every affected method:
- no tests → R0.x story "为 <类>.<m> 写全分支覆盖测试"
- partial → R0.x story "为 <类>.<m> 补未覆盖分支测试" (lists Bk)
- full coverage → ✓ no R0

R0.x stories use the new "characterize" type — same template as
refactor but acceptance is branch-level, with §2 当前分支盘点 listing
B1..Bn and their ✓/✗ status.

Layer 1 of the three-layer refactor safety net per the 2026-05-16
spec. Layer 2 (identify Step B1.0) and layer 3 (spec Step 0) land
in subsequent commits.
EOF
)"
```

---

## Task 2: bob-identify Step B1.0 (soft coverage gate)

**Files:**
- Modify: `src/templates/skills/bob-identify.md`
- Modify: `tests/integration.rs`

**Goal:** Insert Step B1.0 (test coverage status check) at the start of "### 模式 B1(棕地全量重构)" section. Soft 三段式 — user can decline; if declined, leaves a warning in the identity §8 output.

- [ ] **Step 1: Locate the B1 mode section**

Run: `grep -n '^### 模式 B1\|^\*\*Step B1\.\|^## ' /Users/xiaojin/workshop/run-bob/src/templates/skills/bob-identify.md | head -10`

Expected:
- A line like `### 模式 B1(棕地全量重构)` (around line 176)
- `**Step B1.1**:跑代码扫描:`  (around line 178)

If structure differs, STOP and report.

- [ ] **Step 2: Write the failing test**

Append to `/Users/xiaojin/workshop/run-bob/tests/integration.rs`:

```rust
#[test]
fn bob_identify_refactor_mentions_test_coverage_check() {
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
        "Step B1.0",
        "测试覆盖现状",
        "分支",
        "测试覆盖警告",
    ] {
        assert!(
            content.contains(token),
            "bob-identify --refactor must mention {} for B1 safety gate",
            token
        );
    }
}
```

- [ ] **Step 3: Run the test (must fail)**

Run: `cd /Users/xiaojin/workshop/run-bob && cargo test --test integration bob_identify_refactor_mentions_test_coverage_check 2>&1 | tail -10`

Expected: failure on `Step B1.0`.

- [ ] **Step 4: Insert Step B1.0 at the start of B1 mode**

Use `Edit` on `/Users/xiaojin/workshop/run-bob/src/templates/skills/bob-identify.md`. The anchor is the B1 mode header + the existing Step B1.1 start.

`old_string`:

```
### 模式 B1(棕地全量重构)

**Step B1.1**:跑代码扫描:
```

`new_string`:

```
### 模式 B1(棕地全量重构)

**Step B1.0:测试覆盖现状(soft 前置,全分支级)**

`--refactor` 入口先跑覆盖审查:`find src/test -name "*.java"` 找全部测试文件 → 对待重构的类逐一 grep 方法名 → 读测试体 → 枚举分支。

三段式追问:

> **Q0:这些待重构类的测试覆盖情况?**
>
> **推测**:扫了一遍——
>   - OrderService.cancel(4 分支):2 ✓ 2 ✗ → 需补 R0
>   - OrderService.confirm(3 分支):0 ✓ 3 ✗ → 需写全 R0
>   - LegacyPricingService.calc(2 分支):0 测试文件 → 需写全 R0
> **理由**:grep + 读测试体 + 分支枚举
> **推荐选择**:`先 /bob-stories --refactor 拆 R0 写测试,再 identify`
>
> 是否同意?

用户应答处理:
- **"是"** → 提前结束 identify,提示 `/bob-stories --refactor`
- **"否,我先识别再说"** → 继续 identify,但在 identity 文档 §8 段附加 "⚠ 测试覆盖警告:[列出每个无 / 部分覆盖的方法 + 分支编号]"——后续 onion / spec 阶段能看到这个警告

不强制阻断。但警告留痕,后段无法回避。

**Step B1.1**:跑代码扫描:
```

- [ ] **Step 5: Run the test (must pass)**

Run: `cd /Users/xiaojin/workshop/run-bob && cargo test --test integration bob_identify_refactor_mentions_test_coverage_check`

Expected: pass.

- [ ] **Step 6: Run the full suite**

Run: `cd /Users/xiaojin/workshop/run-bob && cargo test 2>&1 | tail -5`

Expected: **31 passed; 0 failed** (30 prior + 1 new). Existing `init_creates_bob_identify_skill` and `bob_identify_mentions_survey_soft_prompt` / `bob_identify_mentions_stories_soft_prompt` still pass.

- [ ] **Step 7: Commit**

```bash
cd /Users/xiaojin/workshop/run-bob && git add src/templates/skills/bob-identify.md tests/integration.rs && git -c commit.gpgsign=false commit -m "$(cat <<'EOF'
feat(identify): add Step B1.0 全分支级 test coverage soft gate in B1 mode

When user runs /bob-identify --refactor (B1 全量重构), the skill now
first runs a branch-level coverage audit on the refactor targets and
asks the user (三段式) whether to detour to /bob-stories --refactor
to write characterize tests first.

Soft gate: user can decline ("否,我先识别再说") and identify continues,
but writes a "⚠ 测试覆盖警告" into the identity §8 output so onion /
spec layers downstream see the gap.

Layer 2 of the 3-layer safety net per the 2026-05-16 spec. Layer 1
(stories Stage 2.5) already landed. Layer 3 (spec Template C Step 0)
lands next.
EOF
)"
```

---

## Task 3: bob-spec Template C Step 0 (rename + branch upgrade + R0 interlock)

**Files:**
- Modify: `src/templates/skills/bob-spec.md`
- Modify: `tests/integration.rs`

**Goal:** Rename Template C's Step 1 → Step 0, upgrade language to require branch-level coverage, add R0 interlock note, renumber Step 2/3/4/5 → 1/2/3/4. Token test asserts the changes land.

- [ ] **Step 1: Read Template C Step 1-5 block to confirm anchors**

Run: `sed -n '470,510p' /Users/xiaojin/workshop/run-bob/src/templates/skills/bob-spec.md`

Expected:
- `### Step 1:加测试覆盖现状(防止改坏)`
- 2-line bullet about writing integration tests + 禁止删
- `### Step 2:抽 \`OrderRepository\` 端口`
- `### Step 3:状态机上提到 Order`
- `### Step 4:框架边界外推`
- `### Step 5:删除 legacy Service`

If line numbers shift but content matches, the Edit will still work; only worry if the content itself differs.

- [ ] **Step 2: Write the failing test**

Append to `/Users/xiaojin/workshop/run-bob/tests/integration.rs`:

```rust
#[test]
fn bob_spec_template_c_mentions_step_0_with_stories_interlock() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let target = tmp.path();
    std::process::Command::new(run_bob_bin())
        .args(["init", "--dir"])
        .arg(target)
        .status()
        .expect("init");

    let p = target.join(".claude").join("skills").join("bob-spec").join("SKILL.md");
    let content = std::fs::read_to_string(&p).unwrap();

    for token in &[
        "Step 0",
        "全分支级",
        "若 docs/bob/02-stories",
        "characterize",
    ] {
        assert!(
            content.contains(token),
            "bob-spec Template C must mention {} for Step 0 stories interlock",
            token
        );
    }
}
```

- [ ] **Step 3: Run the test (must fail)**

Run: `cd /Users/xiaojin/workshop/run-bob && cargo test --test integration bob_spec_template_c_mentions_step_0_with_stories_interlock 2>&1 | tail -10`

Expected: failure on `Step 0`.

- [ ] **Step 4: Rename Step 1 → Step 0 with upgraded language**

Use `Edit` on `/Users/xiaojin/workshop/run-bob/src/templates/skills/bob-spec.md`.

`old_string`:

```
### Step 1:加测试覆盖现状(防止改坏)
- 写集成测试覆盖现 `OrderApplicationService.cancel`,记录现行为基线
- **禁止**这一步删任何代码
```

`new_string`:

```
### Step 0:测试覆盖现状(全分支级)

- 若 `docs/bob/02-stories-*.md` 索引里有 `R0.x · characterize · <本类>` 已完成 → 引用并跳过本步,前提:R0.x 的全分支盘点表(§2)覆盖了本 spec affected 的所有分支
- 否则,本步执行:
  - 枚举 affected method 的所有分支(if/else/switch case/throw/早 return)
  - 写测试覆盖每一个分支(无遗漏)
  - 跑测试 → 全绿(记录为基线)
- **禁止**这一步删任何代码

参见 bob-stories Stage 2.5 的全分支体检规则。
```

- [ ] **Step 5: Renumber Step 2 → Step 1**

`old_string`:

```
### Step 2:抽 `OrderRepository` 端口
```

`new_string`:

```
### Step 1:抽 `OrderRepository` 端口
```

- [ ] **Step 6: Renumber Step 3 → Step 2**

`old_string`:

```
### Step 3:状态机上提到 Order
```

`new_string`:

```
### Step 2:状态机上提到 Order
```

- [ ] **Step 7: Renumber Step 4 → Step 3**

`old_string`:

```
### Step 4:框架边界外推
```

`new_string`:

```
### Step 3:框架边界外推
```

- [ ] **Step 8: Renumber Step 5 → Step 4**

`old_string`:

```
### Step 5:删除 legacy Service
```

`new_string`:

```
### Step 4:删除 legacy Service
```

- [ ] **Step 9: Update the "改造步骤" intro that references "原子化,每步一次提交"**

The Template C section likely has an intro before the Step list saying something about original steps. Verify there's no count reference (e.g., "5 步" or "五步") that would now be stale:

Run: `grep -n '5 步\|5个 Step\|五步\|五个 Step' /Users/xiaojin/workshop/run-bob/src/templates/skills/bob-spec.md`

If no matches → no further edit needed.

If matches → STOP and report (the plan didn't anticipate that wording; need to handle case-by-case).

- [ ] **Step 10: Run the token test (must pass)**

Run: `cd /Users/xiaojin/workshop/run-bob && cargo test --test integration bob_spec_template_c_mentions_step_0_with_stories_interlock`

Expected: pass.

- [ ] **Step 11: Run the full suite**

Run: `cd /Users/xiaojin/workshop/run-bob && cargo test 2>&1 | tail -5`

Expected: **32 passed; 0 failed** (31 prior + 1 new). Pre-existing `init_creates_bob_spec_skill` still passes — it checks tokens like "Given-When-Then", "ARCHITECTURE.md", "TransactionalUseCaseDecorator", "Superpowers", "命令型", "查询型", "重构型", "交给 Superpowers 的开放问题", "技术栈", "5 问决策树", "推测". None of those touch Step numbering, so the renumber doesn't break it.

- [ ] **Step 12: Commit**

```bash
cd /Users/xiaojin/workshop/run-bob && git add src/templates/skills/bob-spec.md tests/integration.rs && git -c commit.gpgsign=false commit -m "$(cat <<'EOF'
feat(spec): Template C Step 0 全分支级 + stories R0 interlock

Renames the test-coverage step from Step 1 to Step 0 (semantically:
before any refactor action), upgrades the requirement from "覆盖现行为
基线" to enumerate-all-branches + cover-every-branch + green, and adds
an explicit interlock: if /bob-stories already emitted an R0.x
characterize story for this class and it's done, skip this Step 0
(don't double up).

Renumbers subsequent steps:
- Step 2 → Step 1 (抽端口)
- Step 3 → Step 2 (状态机上提)
- Step 4 → Step 3 (框架边界外推)
- Step 5 → Step 4 (删除 legacy)

Layer 3 of the 3-layer safety net per the 2026-05-16 spec. Layer 1
(stories Stage 2.5) and layer 2 (identify Step B1.0) already landed.
The full safety net is now complete — no path through the bob workflow
can refactor without a branch-level test gate.
EOF
)"
```

---

## Task 4 (optional): README mention

**Files:**
- Modify: `README.md`

**Goal:** Mention the test safety net in the `/bob-stories` README subsection. Optional — skip if time-constrained.

- [ ] **Step 1: Find the bob-stories subsection in README**

Run: `grep -n '/bob-stories.*phase 1' /Users/xiaojin/workshop/run-bob/README.md`

Expected: a line around 215 referencing `/bob-stories <requirement>` (phase 1...).

- [ ] **Step 2: Edit the description paragraph**

Use `Edit` on `/Users/xiaojin/workshop/run-bob/README.md`.

`old_string`:

```
Output: `docs/bob/02-stories-*.md` index + `docs/bob/02-stories/<n>-<slug>.md` per-story files.
```

`new_string`:

```
Output: `docs/bob/02-stories-*.md` index + `docs/bob/02-stories/<n>-<slug>.md` per-story files. In refactor / mixed mode, also runs a Stage 2.5 全分支级 test coverage audit and auto-emits `R0.x · characterize` stories for any method whose branches aren't fully covered — so every refactor downstream has a green safety net before it starts.
```

- [ ] **Step 3: Verify nothing else regressed**

Run: `cd /Users/xiaojin/workshop/run-bob && cargo test 2>&1 | tail -3`

Expected: 32 passing (README isn't tested).

- [ ] **Step 4: Commit**

```bash
cd /Users/xiaojin/workshop/run-bob && git add README.md && git -c commit.gpgsign=false commit -m "$(cat <<'EOF'
docs(readme): mention Stage 2.5 safety net in /bob-stories subsection

Adds one sentence to the /bob-stories README description noting that
refactor/mixed mode runs a 全分支级 coverage audit and emits R0.x
characterize stories to ensure each refactor has a green test
baseline before it starts.
EOF
)"
```

---

## Final Verification

After all tasks committed:

- [ ] **Run the full suite**

Run: `cd /Users/xiaojin/workshop/run-bob && cargo test 2>&1 | tail -5`

Expected: **32 passed; 0 failed** (Task 4 doesn't add tests).

- [ ] **Smoke-test the binary**

```bash
cd /Users/xiaojin/workshop/run-bob && cargo build --release 2>&1 | tail -1
tmp=$(mktemp -d)
./target/release/run-bob init --dir "$tmp" >/dev/null
ls "$tmp/.claude/skills/" | sort
# Verify each layer's token landed
grep -c "Stage 2.5" "$tmp/.claude/skills/bob-stories/SKILL.md" && echo "✓ stories Stage 2.5"
grep -c "Step B1.0" "$tmp/.claude/skills/bob-identify/SKILL.md" && echo "✓ identify Step B1.0"
grep -c "Step 0" "$tmp/.claude/skills/bob-spec/SKILL.md" && echo "✓ spec Step 0"
grep -c "全分支级" "$tmp/.claude/skills/bob-spec/SKILL.md" && echo "✓ spec mentions 全分支级"
grep -c "characterize" "$tmp/.claude/skills/bob-stories/SKILL.md" && echo "✓ stories has characterize template"
rm -rf "$tmp"
```

Expected: all 5 ✓ markers.

- [ ] **Smoke-test upgrade**

```bash
tmp=$(mktemp -d)
./target/release/run-bob init --dir "$tmp" >/dev/null
./target/release/run-bob upgrade --dir "$tmp" 2>&1 | tail -3
# Modify one skill to be "stale" and verify upgrade refreshes
echo "STALE" > "$tmp/.claude/skills/bob-stories/SKILL.md"
./target/release/run-bob upgrade --dir "$tmp" 2>&1 | tail -5
grep -c "Stage 2.5" "$tmp/.claude/skills/bob-stories/SKILL.md" && echo "✓ upgrade restored Stage 2.5"
rm -rf "$tmp"
```

Expected: first upgrade no-op; second upgrade restores the stale file; the restored content includes "Stage 2.5".

- [ ] **Spec coverage**

| Spec section | Implemented in |
|---|---|
| §2 三层联动表 | Task 1 + 2 + 3 (each implements one layer) |
| §3 bob-stories Stage 2.5(branch enumeration / 3 outcomes / 三段式) | Task 1 |
| §4 R0.x characterize template | Task 1 |
| §5 bob-identify Step B1.0(soft gate + warning) | Task 2 |
| §6 bob-spec Template C Step 0(rename + branch upgrade + interlock + renumber) | Task 3 |
| §7 token tests(3 new) | Task 1 + 2 + 3 |
| §8 ARCHITECTURE.md unchanged | (Explicit non-change) |
| §9 决策记录 | (In spec) |
| §10 实施草图 | Tasks 1-3 (+ optional Task 4) — this plan |

No gap.
