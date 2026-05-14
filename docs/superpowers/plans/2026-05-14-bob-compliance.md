# bob-compliance Skill (phase 3) Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add `/bob-compliance` — a phase-3 skill that converts team-provided spec documents (PDF / docx / md) into structured rule markdowns, guides Claude to write compliant code via a new CLAUDE.md R13, and verifies the per-story diff against the rules after TDD finishes.

**Architecture:** New skill template `bob-compliance.md` ships alongside the existing 6 skills; new harness asset `docs/compliance/README.md` documents the directory contract (PMD/SonarQube opt-out, source format hints, cache mechanism); new HARNESS_DIR `docs/compliance/sources/` is created by `init`; CLAUDE.md gains R13 to enforce read-before-write semantics; bob-spec.md's three "下一步" reminders are extended to mention `/bob-compliance`.

**Tech Stack:** Rust 1.75+, clap 4.5.4, anyhow 1.0, colored 2.1, dev-dep tempfile 3. **No new crates.**

**Spec:** [`docs/superpowers/specs/2026-05-14-bob-compliance-design.md`](../specs/2026-05-14-bob-compliance-design.md)

---

## File Structure

| File | Action | Responsibility |
|---|---|---|
| `src/templates/skills/bob-compliance.md` | **Create** | New skill, ~400 lines, 5 stages (探测 → 生成 → 装载 → 校验 → 报告) |
| `src/templates/root/compliance-README.md` | **Create** | `docs/compliance/README.md` template — fixed 3-section README explaining the contract |
| `src/templates/root/CLAUDE.md` | **Modify** | Add R13 after R12 (compliance gate rule) |
| `src/templates/skills/bob-spec.md` | **Modify** | Append `/bob-compliance` line to each of Template A/B/C's "下一步" sections (3 spots) |
| `src/assets.rs` | **Modify** | Add 2 new asset entries (skill + compliance-README) and 1 new HARNESS_DIRS entry (`docs/compliance/sources/`) |
| `tests/integration.rs` | **Modify** | Append 7 new tests + update `upgrade_safe_field_matches_category_policy` allowlist |

**Untouched** (the plan must NOT modify these):
- `src/commands/init.rs` / `upgrade.rs` / `status.rs` / `gitignore.rs` — existing loops handle new assets/dirs automatically
- `Cargo.toml`
- `README.md` (top-level) — out of scope for this plan; user can add a "compliance" subsection later if they want

---

## Architectural Notes for the Engineer

Read these before starting — they explain decisions not obvious from individual tasks.

### 1. R-rule numbering: use R13, not R14

The spec says "R14" but inspecting `src/templates/root/CLAUDE.md` shows the current last rule is **R12** (no R13 yet). The spec was written assuming R13 was taken — it isn't. Use **R13** for the compliance rule. Don't introduce a numbering gap.

### 2. HarnessDoc upgrade-safety: extend the allowlist

The existing SSoT drift-guard test `upgrade_safe_field_matches_category_policy` in `tests/integration.rs` asserts that the only `HarnessDoc` asset with `upgrade_safe = true` is `README-RUN-BOB.md`. All other HarnessDoc files (CLAUDE.md, ARCHITECTURE.md) are user-owned and must be `upgrade_safe = false`.

When you add `docs/compliance/README.md` as a HarnessDoc with `upgrade_safe = true`, this drift guard will fail. The right fix is to **extend the allowlist** to include compliance/README.md, not to invent a new Category. The allowlist approach keeps the type system simple and the new file is conceptually the same kind of "machine-managed doc" as README-RUN-BOB.md.

### 3. Skill assets are always `included_in_minimal = true`

All existing bob-* skills have `included_in_minimal = true`. Follow this pattern for bob-compliance — even minimal mode should install the skill, because the skill's Stage 0 gracefully handles missing `docs/compliance/sources/` (soft-exits with "no compliance required").

### 4. `compliance-README.md` is `included_in_minimal = false`

The README and the directory `docs/compliance/sources/` are the "harness scaffold" — they make sense for full projects. Minimal mode = "skills only", so we skip these like we skip CLAUDE.md / ARCHITECTURE.md / docs/bob/ / docs/specs/. Skill still works; user can manually `mkdir docs/compliance/sources` if they want to use it.

### 5. Skill content is markdown prose — don't try to paste it all into this plan

The bob-compliance.md skill template is ~400 lines of structured prose (frontmatter + 5 stages + 三段式 examples + schemas). This plan provides:
- The exact frontmatter (must match)
- Section structure (Stage 0–4 headers)
- The list of **load-bearing tokens** that tests will assert
- A reference to existing skills (bob-nfr.md, bob-survey.md) for prose style

The implementer's job is to write coherent Markdown prose around those tokens, in the same style as the existing bob-* skills. **Do not invent new section headers** or skip stages — the test will catch missing tokens.

### 6. Test that init creates the README + dir, but doesn't pre-populate sources/

After `run-bob init`:
- `docs/compliance/` exists
- `docs/compliance/sources/` exists (empty)
- `docs/compliance/README.md` exists with the canonical content
- `docs/compliance/.compliance.lock` does NOT exist
- `docs/compliance/<any>.md` (generated files) do NOT exist

Generated files are runtime products of `/bob-compliance`, not assets — run-bob never writes them.

### 7. bob-spec.md has THREE 下一步 sections (A/B/C templates)

Confirmed via `grep -n "下一步" src/templates/skills/bob-spec.md`. Each template's "下一步" already mentions `/bob-nfr`. Add `/bob-compliance` to all three. The existing `bob_spec_mentions_nfr_reminder` test asserts `/bob-nfr` appears ≥3 times — your new `/bob-compliance` reminder should also appear ≥3 times.

---

## Task 1: Register assets + extend drift-guard allowlist (TDD)

**Files:**
- Create: `src/templates/skills/bob-compliance.md` (stub — minimal valid content for compilation)
- Create: `src/templates/root/compliance-README.md` (stub — minimal valid content for compilation)
- Modify: `src/assets.rs`
- Modify: `tests/integration.rs`

**Goal:** Lay down the SSoT plumbing — register 2 new assets + 1 new HARNESS_DIR + extend the drift-guard allowlist. Tests must pass at the end of this task, even though template files contain only stubs. Contents land in Tasks 2–6.

- [ ] **Step 1: Create stub `src/templates/skills/bob-compliance.md`**

Write exactly this minimal content (frontmatter + one heading — enough to be a valid skill):

```markdown
---
name: bob-compliance
description: stub — to be replaced in Task 3
---

# Bob Compliance Skill (stub)

Real content lands in Tasks 3-5.
```

- [ ] **Step 2: Create stub `src/templates/root/compliance-README.md`**

Write exactly this minimal stub:

```markdown
# 项目级合规检查 (stub)

Real content lands in Task 2.
```

- [ ] **Step 3: Add 2 new assets + 1 new HARNESS_DIR in `src/assets.rs`**

Append the new skill asset **at the end** of the `// --- Skills` section in `HARNESS_ASSETS` (after `bob-nfr`):

```rust
    Asset {
        rel_path: &[".claude", "skills", "bob-compliance", "SKILL.md"],
        content: include_str!("templates/skills/bob-compliance.md"),
        category: Category::Skill,
        included_in_minimal: true,
        upgrade_safe: true,
    },
```

Append the new HarnessDoc asset **at the end** of the `// --- Harness documents` section in `HARNESS_ASSETS` (after `README-RUN-BOB.md`):

```rust
    Asset {
        rel_path: &["docs", "compliance", "README.md"],
        content: include_str!("templates/root/compliance-README.md"),
        category: Category::HarnessDoc,
        included_in_minimal: false,
        upgrade_safe: true,
    },
```

Append the new HARNESS_DIRS entry **at the end** of the `HARNESS_DIRS` array:

```rust
    HarnessDir {
        rel_path: &["docs", "compliance", "sources"],
        note: "(用户合规规约原始文件 — PDF/docx/md/txt;空目录 = 无合规要求)",
    },
```

- [ ] **Step 4: Extend the drift-guard allowlist in `tests/integration.rs`**

Locate `upgrade_safe_field_matches_category_policy` and find the `Category::HarnessDoc => { ... }` arm. Change from:

```rust
            Category::HarnessDoc => {
                let is_readme = asset.rel_path == ["README-RUN-BOB.md"];
                if is_readme {
                    assert!(
                        asset.upgrade_safe,
                        "README-RUN-BOB.md must be upgrade_safe=true"
                    );
                } else {
                    assert!(
                        !asset.upgrade_safe,
                        "{} is a user-owned HarnessDoc but upgrade_safe=true",
                        display
                    );
                }
            }
```

to:

```rust
            Category::HarnessDoc => {
                let upgrade_safe_docs: &[&[&str]] = &[
                    &["README-RUN-BOB.md"],
                    &["docs", "compliance", "README.md"],
                ];
                let is_upgrade_safe_doc = upgrade_safe_docs
                    .iter()
                    .any(|s| asset.rel_path == *s);
                if is_upgrade_safe_doc {
                    assert!(
                        asset.upgrade_safe,
                        "{} is a machine-managed HarnessDoc, must be upgrade_safe=true",
                        display
                    );
                } else {
                    assert!(
                        !asset.upgrade_safe,
                        "{} is a user-owned HarnessDoc but upgrade_safe=true",
                        display
                    );
                }
            }
```

- [ ] **Step 5: Run the full test suite**

Run: `cargo test`

Expected: every test passes — including pre-existing `status_checks_every_file_init_writes` (which now sees the new README in init output and verifies status lists it), `upgrade_safe_field_matches_category_policy`, and all other tests.

If `status_checks_every_file_init_writes` fails, it means the new asset was created by init but status didn't list it. Status reads `HARNESS_ASSETS` too, so this should "just work" — if not, debug there.

- [ ] **Step 6: Commit**

```bash
git add src/templates/skills/bob-compliance.md src/templates/root/compliance-README.md src/assets.rs tests/integration.rs
git commit -m "$(cat <<'EOF'
feat(compliance): register bob-compliance assets in SSoT

Adds skill stub + docs/compliance/README.md stub + docs/compliance/sources/
dir. Extends drift-guard allowlist for the new machine-managed README.
Real content lands in Tasks 2-6.

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
EOF
)"
```

---

## Task 2: Write the canonical `docs/compliance/README.md` content (TDD)

**Files:**
- Modify: `src/templates/root/compliance-README.md`
- Modify: `tests/integration.rs`

**Goal:** Replace the stub with the canonical 3-section README that init writes into every full project. Lock in content via a token-presence test.

- [ ] **Step 1: Add the failing test**

Append to `tests/integration.rs`:

```rust
#[test]
fn init_creates_compliance_readme_with_pmd_note() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let target = tmp.path();
    std::process::Command::new(run_bob_bin())
        .args(["init", "--dir"])
        .arg(target)
        .status()
        .expect("init");

    let p = target.join("docs").join("compliance").join("README.md");
    assert!(p.is_file(), "compliance/README.md missing at {}", p.display());
    let content = std::fs::read_to_string(&p).expect("read README");

    // Load-bearing tokens — the 3-section contract
    for token in &[
        // Section 1: 用法
        "项目级合规检查",
        "sources/",
        "/bob-compliance",
        "PDF",
        "docx",
        // Section 2: PMD/SonarQube 例外
        "PMD",
        "SonarQube",
        "保持",
        "为空",
        // Section 3: 缓存
        ".compliance.lock",
        "sha256",
        "filename",
    ] {
        assert!(
            content.contains(token),
            "compliance/README.md must mention {}; got:\n{}",
            token,
            content
        );
    }
}
```

- [ ] **Step 2: Run the test, expect failure**

Run: `cargo test --test integration init_creates_compliance_readme_with_pmd_note`
Expected: FAIL — the stub doesn't contain any of the required tokens.

- [ ] **Step 3: Replace stub with canonical content**

Overwrite `src/templates/root/compliance-README.md` with this exact content:

```markdown
# 项目级合规检查

这个目录由 `run-bob` 维护,目的是把团队 / 项目级的**自然语言规约**(命名、异常处理、
安全实践、漏洞防范)注入 Claude Code 工作流,让 AI 写代码时一次合规。

## 用法

1. 把项目要遵守的规约文档放进 `sources/`,任意格式(PDF / docx / md / txt)
2. 写 story 跑 `/bob-spec` 时,bob-spec 会自动提示是否需要先跑 `/bob-compliance`
3. `/bob-compliance` 会:
   - 把 `sources/` 里每份文档结构化成 `docs/compliance/<标准名>.md`(带规则 ID)
   - Superpowers TDD 时 Claude 自动读这些 md(由 CLAUDE.md R13 强制)
   - TDD 完成后对 diff 跑一次校验,产物落到 `docs/bob/05-compliance-<story>.md`

## 例外:已经有 PMD / SonarQube / SpotBugs ?

**保持 `sources/` 为空即可**。run-bob 不会重复跑机械可检项,
也不替代任何 IDE / CI 静态扫描工具。

`/bob-compliance` 只解决你的 CI 工具**不擅长**的那部分 —— 自然语言级的、
需要语义理解的规约(异常处理思路、安全实践、命名约定中的语义部分等)。

## 缓存机制

`.compliance.lock` 记录 `sources/` 里每份文件的 `filename + size + sha256`。
再次运行 `/bob-compliance` 时:

- sha256 完全一致 → 直接复用现有 `docs/compliance/*.md`,跳过生成
- 任何文件新增 / 修改 → 仅重新生成漂移的部分
- `sources/` 为空 → Stage 0 软退出,无任何副作用

## 不会发生的事

- `run-bob upgrade` **永远不会**触碰 `sources/`、`.compliance.lock`、或任何 `*.md` 生成产物
- run-bob 二进制**不内置**任何合规标准(版权 + 团队差异)
- `/bob-compliance` 不调用任何外部进程,纯 Claude 内部处理
```

- [ ] **Step 4: Run the test, expect pass**

Run: `cargo test --test integration init_creates_compliance_readme_with_pmd_note`
Expected: PASS — all tokens present.

- [ ] **Step 5: Run the full test suite to confirm no regression**

Run: `cargo test`
Expected: every pre-existing test still passes.

- [ ] **Step 6: Commit**

```bash
git add src/templates/root/compliance-README.md tests/integration.rs
git commit -m "$(cat <<'EOF'
feat(compliance): write canonical docs/compliance/README.md template

3-section contract: usage (sources/ + auto-flow), PMD/SonarQube
opt-out, cache mechanism. Token-presence test locks the structure.

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
EOF
)"
```

---

## Task 3: Write bob-compliance skill — frontmatter + Stage 0 (TDD)

**Files:**
- Modify: `src/templates/skills/bob-compliance.md`
- Modify: `tests/integration.rs`

**Goal:** Replace the stub with the frontmatter and Stage 0 (status detection). Establish the test scaffold that subsequent tasks (4, 5) will extend with more tokens.

Style reference: read `src/templates/skills/bob-nfr.md` lines 1–100 to see the frontmatter + 三段式 format. Match its tone and structure.

- [ ] **Step 1: Add the failing test (initial token set — Tasks 4-5 will extend it)**

Append to `tests/integration.rs`:

```rust
#[test]
fn init_creates_bob_compliance_skill() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let target = tmp.path();
    std::process::Command::new(run_bob_bin())
        .args(["init", "--dir"])
        .arg(target)
        .status()
        .expect("init");

    let p = target
        .join(".claude")
        .join("skills")
        .join("bob-compliance")
        .join("SKILL.md");
    assert!(p.is_file(), "bob-compliance SKILL.md missing at {}", p.display());
    let content = std::fs::read_to_string(&p).expect("read skill");

    // Frontmatter
    assert!(content.starts_with("---"), "must start with YAML frontmatter");
    assert!(content.contains("name: bob-compliance"), "frontmatter name");
    assert!(content.contains("description:"), "frontmatter description");

    // Stage 0 tokens (will be extended in Tasks 4 and 5 with additional stage tokens)
    for token in &[
        // CLI / invocation
        "/bob-compliance",
        "--story",
        "--refresh",
        "--all-branch",
        // 三段式
        "三段式",
        "推测",
        "推荐选择",
        // Stage 0
        "Stage 0",
        "状态探测",
        "空仓",
        "首次",
        "漂移",
        "冷藏",
        "docs/compliance/sources/",
        ".compliance.lock",
        "sha256",
    ] {
        assert!(content.contains(token), "bob-compliance must mention {}", token);
    }
}
```

- [ ] **Step 2: Run the test, expect failure**

Run: `cargo test --test integration init_creates_bob_compliance_skill`
Expected: FAIL — the stub doesn't contain these tokens.

- [ ] **Step 3: Replace the stub with frontmatter + Stage 0 prose**

Overwrite `src/templates/skills/bob-compliance.md` with this content (and only this — Tasks 4 and 5 will append Stages 1–4):

```markdown
---
name: bob-compliance
description: |
  触发条件:用户输入 /bob-compliance(主入口:对当前 story diff 做合规校验),
  或 /bob-compliance --story <story-path>(指定 story 范围),
  或 /bob-compliance --refresh(强制重新生成 sources/ 下的结构化 md),
  或 /bob-compliance --all-branch(忽略 story 划分,校验整个分支 diff)。

  在 docs/compliance/sources/ 下放规约原始文件(PDF / docx / md / txt)之后,
  本技能(1)动态生成结构化规则 markdown(带规则 ID + 强制档位),(2)在
  Superpowers TDD 完成 + UT 跑绿之后,对当前 story 的 diff 跑合规校验,
  产物落 docs/bob/05-compliance-<story>.md。

  适用于 Bob 4 环 Clean Architecture 工作流的 phase 3:per-story 实施完后
  的代码合规 review。结构对称 phase 2 的 /bob-nfr。

  当用户说"跑合规"、"代码 review 一下"、"过一遍阿里规约"、"检查命名 / 异常 / 安全"
  时也应触发此技能。
---

# Bob Compliance Skill

## 触发

```
/bob-compliance                       # 主入口:校验当前 story 的 diff
/bob-compliance --story <story-path>  # 指定 story 范围
/bob-compliance --refresh             # 强制重新生成 sources/ 下的结构化 md
/bob-compliance --all-branch          # 忽略 story 划分,校验整个分支 diff
```

或自然语言触发:"跑合规"、"代码 review 一下"、"过一遍阿里规约"、"检查命名 / 异常 / 安全"。

## 前置条件

- 项目位于 git 仓库内
- 建议:Superpowers TDD 已完成 + UT 跑绿后再启动合规 review
- `docs/compliance/sources/` 存在(由 `run-bob init` 创建);若不存在,创建空目录即可

## 提问规约(强制三段式)

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

per-story 代码合规校验。**只**回答两个问题:

1. **`docs/compliance/sources/` 下的规约,目前有没有结构化好的 markdown?需不需要(重新)生成?**
2. **当前 story 的 diff 有没有违反任何已加载的规则?**

**不写业务代码、不出新 spec、不画架构**。产出合规校验报告 + 建议修复 story 清单。

## 工作流(5 个 Stage)

```
Stage 0. 状态探测(sources/ 状态 vs .compliance.lock)
Stage 1. 生成 / 刷新结构化 md(仅在首次或漂移时执行)
Stage 2. 装载所有 docs/compliance/*.md,建立规则索引
Stage 3. 对当前 story 的 diff 跑规则校验
Stage 4. 写报告 + 建议新增 story 清单
```

---

## Stage 0. 状态探测

读取 `docs/compliance/sources/` 状态并判定:

| 状态 | 判定条件 | 后续行为 |
|---|---|---|
| **空仓** | `docs/compliance/sources/` 不存在或为空 | 软退出:"无合规要求,跳过"。**写一份空报告留痕**(避免下次重复探测) |
| **首次** | `sources/` 有文件,但无 `.compliance.lock` | → Stage 1 全量生成 |
| **漂移** | 至少一个源文件的 size 或 sha256 与 `.compliance.lock` 记录不匹配,或 sources/ 中有 lock 未记录的新文件 | → Stage 1 增量生成漂移部分 |
| **冷藏** | sources 的全部文件 sha256 与 lock 完全一致 | 跳过 Stage 1,直接 → Stage 2 |

向用户三段式通报探测结果:

> **Q0: 探测到 sources/ 状态为 <状态>。**
>
> **推测**:<状态>。命中 X 个源文件:[列出文件名]。
> **理由**:<根据 lock 比对的具体证据>
> **推荐选择**:`继续(Stage 1 全量 / 增量 / 跳过)`
>
> 是否同意?(回"是"继续;回"刷新"强制走 --refresh 路径;回"取消"退出)

`--refresh` flag 显式触发"漂移"路径,即使 sha256 都匹配,也走 Stage 1。
```

- [ ] **Step 4: Run the test, expect pass**

Run: `cargo test --test integration init_creates_bob_compliance_skill`
Expected: PASS — all Stage 0 tokens present.

- [ ] **Step 5: Run the full test suite to confirm no regression**

Run: `cargo test`
Expected: green.

- [ ] **Step 6: Commit**

```bash
git add src/templates/skills/bob-compliance.md tests/integration.rs
git commit -m "$(cat <<'EOF'
feat(compliance): bob-compliance skill — frontmatter + Stage 0

CLI surface (--story / --refresh / --all-branch), 三段式 prompting
contract, and Stage 0 status detection (空仓 / 首次 / 漂移 / 冷藏).
Stages 1-4 land in Tasks 4-5.

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
EOF
)"
```

---

## Task 4: bob-compliance skill — Stage 1 (生成) + Stage 2 (装载) (TDD)

**Files:**
- Modify: `src/templates/skills/bob-compliance.md`
- Modify: `tests/integration.rs`

**Goal:** Append Stage 1 (convert raw sources/ files into structured md with rule IDs) and Stage 2 (load all docs/compliance/*.md into an in-memory rule index). Extend the existing test to assert these stages' tokens.

- [ ] **Step 1: Extend the test with Stage 1 + Stage 2 tokens**

In `tests/integration.rs`, find `init_creates_bob_compliance_skill` and add these tokens **inside the existing `for token in &[...]` loop** (append to the array, before the closing `]`):

```rust
        // Stage 1 — generation
        "Stage 1",
        "结构化",
        "规则 ID",
        "强制",
        "推荐",
        "参考",
        "frontmatter",
        "alibaba-songshan",
        "ALI-1.1.2",  // sample rule ID from the schema example
        "generated_to",
        // Stage 2 — load
        "Stage 2",
        "装载",
        "规则索引",
        "Severity",
```

- [ ] **Step 2: Run the test, expect failure**

Run: `cargo test --test integration init_creates_bob_compliance_skill`
Expected: FAIL — Stages 1 and 2 don't exist yet.

- [ ] **Step 3: Append Stage 1 + Stage 2 to the skill template**

At the end of `src/templates/skills/bob-compliance.md`, append:

```markdown

---

## Stage 1. 生成 / 刷新结构化 md

仅在 Stage 0 判定为 **首次** / **漂移** / `--refresh` 时执行。对每个新增 / 漂移的源文件做转换:

### 1.1 按格式读取原文

| 后缀 | 读取方式 |
|---|---|
| `.pdf` | Read 工具的 `pages` 参数分批读(每次 5-10 页),逐批抽取规则 |
| `.docx` | 同 PDF,Read 工具直接处理 |
| `.md`、`.txt`、`.markdown` | 一次性 Read 全文 |
| 其他 | 跳过该文件,在最终报告里标注"格式不支持" |

### 1.2 抽取规则

Claude 在原文里识别以下结构(以阿里嵩山版为典型):

- **一级维度**(7 大块):编程规约 / 异常日志 / 单元测试 / 安全规约 / MySQL / 工程结构 / 设计规约
- **二级子节**:每个维度内的子节(例:编程规约 → 命名风格 / 常量定义 / 代码格式 ...)
- **单条规则**:标题 + 强制档位(【强制】/【推荐】/【参考】)+ 反例 + 正例 + 说明

### 1.3 写结构化 md

输出文件名:`docs/compliance/<source-stem>.md`(把源文件名去后缀作为 stem,中文文件名照样保留)。

固定 schema(以阿里 PDF 为例):

```markdown
---
name: alibaba-songshan
version: 1.7.0
authority: 阿里巴巴 / 嵩山版 2020-08-03
language: java
source_filename: 阿里巴巴Java开发规范（嵩山版）.pdf
source_sha256: a1b2c3...
---

# 目录

- §1 编程规约
  - §1.1 命名风格
  - §1.2 常量定义
  ...
- §2 异常日志
- §3 单元测试
- §4 安全规约
- §5 MySQL 数据库
- §6 工程结构
- §7 设计规约

---

# §1 编程规约

## §1.1 命名风格

### [ALI-1.1.1] 【强制】命名不能以下划线或美元符号开始或结束

**反例:** `_name` / `__name` / `$name` / `name_` / `name$` / `name__`

**适用范围:** Java 所有标识符

**检测提示:** 静态扫描可覆盖;diff 级检测 `grep -nE "^[_$]|[_$]$"`

### [ALI-1.1.2] 【强制】禁止拼音英文混合,禁止纯中文命名

**正例:** `ali` / `alibaba` / `taobao` / `hangzhou`(国际通用名)
**反例:** `DaZhePromotion[打折]` / `String fw[福娃]` / `int 某变量`
```

规则 ID 命名规则:`[<STANDARD>-<§>.<§>.<n>]`,STANDARD 用源文件 stem 大写缩写(例:`ALI`、`CCAF`、`VULN`)。

### 1.4 更新 `.compliance.lock`

整文件原子重写(不增量改);单条记录写完 + 整文件落盘后再处理下一个源文件。中途中断重跑时 Stage 0 会自然识别为"漂移"并补全。

```yaml
generated_at: 2026-05-14T03:21:00Z
sources:
  - filename: 阿里巴巴Java开发规范（嵩山版）.pdf
    size: 1908201
    sha256: a1b2c3...
    generated_to: alibaba-songshan.md
```

### 1.5 边界情况

- 源文件不可读(损坏 / 权限)→ 输出错误,跳过该文件继续;lock 不记录该文件;下次重跑时仍按"漂移"处理
- 同一 stem 重复(`foo.pdf` 和 `foo.docx`)→ 后写覆盖前写,**在 lock 里报告冲突**,建议用户改名

---

## Stage 2. 装载 (双模式都用)

读所有 `docs/compliance/*.md`(不含 `README.md`),建立内存规则索引:

```
rule_id → (file, section, severity, title)
```

**Severity 优先级:【强制】> 【推荐】> 【参考】**。三档**都参与**校验,但分类报告时:

- 【强制】违反 → 视为必须修复
- 【推荐】违反 → 列出但允许豁免
- 【参考】违反 → 列出仅作提示

向用户通报装载结果:

> **Q1: 装载到 N 条规则,来自 M 份标准。**
>
> **推测**:【强制】X 条 / 【推荐】Y 条 / 【参考】Z 条
> **理由**:从已生成的 `docs/compliance/*.md` 索引得出
> **推荐选择**:`进入 Stage 3 跑校验`
>
> 是否同意?
```

- [ ] **Step 4: Run the test, expect pass**

Run: `cargo test --test integration init_creates_bob_compliance_skill`
Expected: PASS.

- [ ] **Step 5: Run the full test suite**

Run: `cargo test`
Expected: green.

- [ ] **Step 6: Commit**

```bash
git add src/templates/skills/bob-compliance.md tests/integration.rs
git commit -m "$(cat <<'EOF'
feat(compliance): bob-compliance skill — Stage 1 (gen) + Stage 2 (load)

Stage 1: PDF/docx/md → structured markdown with rule IDs ([ALI-1.1.2]
style), YAML frontmatter, atomic .compliance.lock rewrite.
Stage 2: load all docs/compliance/*.md into rule index with severity
priority (强制 > 推荐 > 参考).

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
EOF
)"
```

---

## Task 5: bob-compliance skill — Stage 3 (校验) + Stage 4 (报告) (TDD)

**Files:**
- Modify: `src/templates/skills/bob-compliance.md`
- Modify: `tests/integration.rs`

**Goal:** Append Stage 3 (diff scanning against rules, classification: 违反 / 待量化 / 豁免) and Stage 4 (write report to `docs/bob/05-compliance-<story>.md` + suggest fix stories). Complete the skill.

- [ ] **Step 1: Extend the test with Stage 3 + Stage 4 tokens**

In `tests/integration.rs`, in `init_creates_bob_compliance_skill`, append these tokens to the existing `for token in &[...]` loop:

```rust
        // Stage 3 — diff check
        "Stage 3",
        "diff",
        "违反",
        "待量化",
        "豁免",
        // Diff scope priority
        "git diff",
        "master..HEAD",
        // Stage 4 — report + handoff
        "Stage 4",
        "docs/bob/05-compliance-",
        "建议新增 story 清单",
        "/bob-stories",
        // Cross-skill handoff
        "/bob-nfr",
```

- [ ] **Step 2: Run the test, expect failure**

Run: `cargo test --test integration init_creates_bob_compliance_skill`
Expected: FAIL — Stages 3 and 4 don't exist yet.

- [ ] **Step 3: Append Stage 3 + Stage 4 to the skill template**

At the end of `src/templates/skills/bob-compliance.md`, append:

```markdown

---

## Stage 3. Diff 校验 (模式 B 核心)

### 3.1 定位检查范围 (优先级从高到低)

1. **显式参数**:`/bob-compliance --story <story-id>` → 读取该 story 在 `docs/bob/02-stories-*.md` 里记录的 base ref,跑 `git diff <base>..HEAD`
2. **当前活跃 story**:`docs/bob/02-stories-*.md` 存在且能识别出"当前 story" → 用该 story 的 base..HEAD
3. **Fallback**:`git diff master..HEAD` + 未提交的工作目录变更
4. **`--all-branch` flag**:`git diff master..HEAD` 全分支 diff,忽略 story 划分

向用户三段式确认范围:

> **Q2: 检查范围:<具体 ref 范围>,涉及 N 个文件 / M 行新增。**
>
> **推测**:<具体路径列表>
> **理由**:<按上述优先级判定>
> **推荐选择**:`确认范围,开始校验`
>
> 是否同意?

### 3.2 逐条规则比对

对每条加载到的规则,在 diff 内匹配:

- **可机械检测**(命名、空格、关键字位置等)→ Claude 用模式匹配 / 正则检视 diff 文本
- **需语义判断**(异常处理思路、并发设计、安全实践)→ Claude 逐文件逐函数检视上下文,判断是否符合规则意图

**优先级**:先跑【强制】,再【推荐】,最后【参考】。中途如发现【强制】违反过多(> 10 条),可三段式询问是否暂停【推荐】 / 【参考】,先修【强制】。

### 3.3 分类

每条命中的规则归入三类:

| 分类 | 定义 |
|---|---|
| **违反** | 明确触碰规则;diff 里有反例代码 |
| **待量化** | 规则模糊或需求未给出基线(例:阿里规约要求"接口幂等",但 spec 未明确该接口是否要求幂等)→ 建议回 spec 补充 |
| **豁免** | spec 的"交给 Superpowers 的开放问题"段已显式注明豁免理由 |

---

## Stage 4. 报告 + 建议新增 story 清单

写报告到 `docs/bob/05-compliance-<story>.md`(或 `<branch>.md` 当走 `--all-branch` 时)。固定结构:

```markdown
# 合规校验报告 · <story-name>

**日期:** 2026-05-14
**范围:** <base-ref>..HEAD,N 个文件 / M 行新增
**加载标准:** alibaba-songshan, ccaf-internal, ...

## 违反清单

### 【强制】违反 (X 条)

#### [ALI-1.1.2] 禁止拼音英文混合
- **位置:** `src/main/java/com/example/order/OrderService.java:42`
- **代码片段:**
  \`\`\`java
  String DaZhePromotion = ...;
  \`\`\`
- **修复建议:** 改为 `String discountPromotion = ...;` 或 `String promotion = ...;`

#### [ALI-2.2.1] 异常不能裸吞
- **位置:** `src/main/java/com/example/order/OrderRepository.java:78-80`
- **代码片段:** ...
- **修复建议:** 至少 `log.error("...", e)`,或重抛业务异常

### 【推荐】违反 (Y 条)
...

### 【参考】违反 (Z 条)
...

## 待量化 (W 条)
- [ALI-7.3] 设计规约要求"接口幂等" — spec 未明确该接口是否要求幂等,
  建议回 spec 补充

## 豁免 (V 条)
...

## 建议新增 story 清单

1. **R-compliance-001 修复 OrderService 拼音命名**
   - story 类型:重构
   - 影响范围:`OrderService` 及其调用方,共 5 处
   - 估时:0.5h

2. **R-compliance-002 OrderRepository 异常补 log**
   - story 类型:重构
   - 影响范围:OrderRepository 全文件
   - 估时:0.5h

## 下一步

- 如有违反,把建议 story 喂给 `/bob-stories --refactor`
- 跑 `/bob-nfr` 做非功能复盘
```

向用户三段式收口:

> **Q3: 校验完成。违反 X【强制】 + Y【推荐】 + Z【参考】,待量化 W 条。**
>
> **推测**:【强制】X 条必须修;建议生成 K 个修复 story
> **理由**:<列出最高优先级的几条>
> **推荐选择**:`生成 story 清单 → 喂给 /bob-stories --refactor`
>
> 是否同意?(回"是"生成;回"否"只留报告;回"细看 [ID]"展开某条具体细节)

---

## 不变量

- **目录即配置**:`docs/compliance/sources/` 空 ⇒ Stage 0 软退出
- **永不内置标准**:run-bob 二进制里**不**打包任何标准 PDF / md
- **upgrade 边界**:用户的 `sources/`、生成的 md、`.compliance.lock`、报告文件 —— run-bob upgrade **永不触碰**
- **PMD/SonarQube 兼容**:空 sources/ ⇒ 与现有静态扫描工具零冲突
- **Claude 唯一执行器**:PDF → md、规则抽取、diff 校验全在 Claude 内部,不引入新二进制 / 不调外部进程
```

- [ ] **Step 4: Run the test, expect pass**

Run: `cargo test --test integration init_creates_bob_compliance_skill`
Expected: PASS.

- [ ] **Step 5: Run the full test suite**

Run: `cargo test`
Expected: green.

- [ ] **Step 6: Commit**

```bash
git add src/templates/skills/bob-compliance.md tests/integration.rs
git commit -m "$(cat <<'EOF'
feat(compliance): bob-compliance skill — Stage 3 (check) + Stage 4 (report)

Stage 3: diff scope priority (--story > active story > master..HEAD >
--all-branch), per-rule mechanical vs semantic matching, classification
into 违反 / 待量化 / 豁免.
Stage 4: structured report at docs/bob/05-compliance-<story>.md with
fix-story suggestions ready to feed /bob-stories --refactor.

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
EOF
)"
```

---

## Task 6: Add R13 to CLAUDE.md (TDD)

**Files:**
- Modify: `src/templates/root/CLAUDE.md`
- Modify: `tests/integration.rs`

**Goal:** Add R13 "compliance gate" rule that mandates Claude reads `docs/compliance/*.md` before writing implementation code. This is the **mode A** enforcement mechanism — load-time injection of rules.

- [ ] **Step 1: Add the failing test**

Append to `tests/integration.rs`:

```rust
#[test]
fn claude_md_has_r13_compliance() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let target = tmp.path();
    std::process::Command::new(run_bob_bin())
        .args(["init", "--dir"])
        .arg(target)
        .status()
        .expect("init");

    let p = target.join("CLAUDE.md");
    assert!(p.is_file(), "CLAUDE.md missing");
    let content = std::fs::read_to_string(&p).expect("read CLAUDE.md");

    for token in &[
        "R13",
        "合规规则前置",
        "docs/compliance/",
        "/bob-compliance",
        "【强制】",
        "规则 ID",
        "豁免",
    ] {
        assert!(
            content.contains(token),
            "CLAUDE.md must contain {} for R13; got:\n{}",
            token,
            content
        );
    }
}
```

- [ ] **Step 2: Run the test, expect failure**

Run: `cargo test --test integration claude_md_has_r13_compliance`
Expected: FAIL — R13 doesn't exist yet.

- [ ] **Step 3: Insert R13 in CLAUDE.md**

In `src/templates/root/CLAUDE.md`, locate the end of the `### R12. ...` section. The section ends just before `## 工作流总览` (around line 270 in the current file). The exact closing line of R12 is:

```
legacy 是另一个"外部世界",和 MySQL / 人大金仓 / 微信支付 SDK 没区别,统一用端口 + ACL 隔离。
```

**Insert** the new R13 section **immediately after** that closing line and **before** the empty line that precedes `## 工作流总览`:

```markdown

### R13. 合规规则前置(项目级)

实现代码之前,**必须**检查 `docs/compliance/*.md`:

- 若不存在或目录为空 → 跳过(无合规要求)
- 若存在 → 读取与当前文件 / 模块相关的章节,**严格遵守所有【强制】条款**
- 在代码注释里引用规则 ID(例:`// 遵守 [ALI-1.1.2] 命名规约`),便于后续 `/bob-compliance` 校验时复核
- 不得擅自违反【强制】条款 —— 如确需违反,必须在 spec 的"交给 Superpowers 的开放问题"段写明**豁免**理由,
  否则 `/bob-compliance` Stage 3 会标记为违反

R13 是 `/bob-compliance` 工作流的**模式 A** 载体(写代码时的"指导"环节)。
Claude 在 TDD 时本来就会读 CLAUDE.md,R13 是最廉价的注入路径:合规知识 = 自然语言规约,
Claude 直接消费 markdown,无需任何额外工具。
```

- [ ] **Step 4: Run the test, expect pass**

Run: `cargo test --test integration claude_md_has_r13_compliance`
Expected: PASS.

- [ ] **Step 5: Run the full test suite**

Run: `cargo test`
Expected: green. The existing `init_creates_claude_md_with_r0` test only checks for R0 / R12 / etc., not R13 absence, so it will still pass.

- [ ] **Step 6: Commit**

```bash
git add src/templates/root/CLAUDE.md tests/integration.rs
git commit -m "$(cat <<'EOF'
feat(compliance): add R13 — compliance gate rule in CLAUDE.md

Mode A enforcement: Claude must read docs/compliance/*.md before
writing implementation code, cite rule IDs in comments, and never
violate 【强制】 without explicit spec-level 豁免.

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
EOF
)"
```

---

## Task 7: Add `/bob-compliance` reminder to bob-spec 下一步 (TDD)

**Files:**
- Modify: `src/templates/skills/bob-spec.md`
- Modify: `tests/integration.rs`

**Goal:** Append a `/bob-compliance` reminder to each of bob-spec's Template A/B/C "下一步" sections (mirroring the existing `/bob-nfr` reminder). After this, the standard TDD-completion handoff is: compliance first (strict gate), then NFR (open questions).

- [ ] **Step 1: Add the failing test**

Append to `tests/integration.rs`:

```rust
#[test]
fn bob_spec_mentions_compliance_in_all_three_templates() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let target = tmp.path();
    std::process::Command::new(run_bob_bin())
        .args(["init", "--dir"])
        .arg(target)
        .status()
        .expect("init");

    let p = target.join(".claude").join("skills").join("bob-spec").join("SKILL.md");
    let content = std::fs::read_to_string(&p).expect("read bob-spec");

    // bob-spec must mention /bob-compliance at least 3 times (one per template's 下一步)
    let count = content.matches("/bob-compliance").count();
    assert!(
        count >= 3,
        "bob-spec must mention /bob-compliance at least 3 times (Template A/B/C reminder); found {}",
        count
    );

    // And the reminder text should hint at "先 /bob-compliance 再 /bob-nfr" ordering
    assert!(
        content.contains("docs/compliance/sources/"),
        "bob-spec compliance reminder must reference the sources/ directory"
    );
}
```

- [ ] **Step 2: Run the test, expect failure**

Run: `cargo test --test integration bob_spec_mentions_compliance_in_all_three_templates`
Expected: FAIL — /bob-compliance is not mentioned yet.

- [ ] **Step 3: Update each of bob-spec.md's three 下一步 sections**

Open `src/templates/skills/bob-spec.md`. There are three `下一步` sections at approximately lines 369, 442, 522 (numbers may shift after prior edits; locate by `grep -n "下一步" src/templates/skills/bob-spec.md`).

Each one currently has a line like:

```markdown
5. (可选)Superpowers TDD 完成 + UT 完备后,跑 `/bob-nfr <本 spec 路径>` 过 13 张 NFR 卡片
```

(or similar — the exact wording varies slightly across templates A/B/C).

**For each of the three 下一步 sections, insert this line IMMEDIATELY BEFORE the existing `/bob-nfr` line** (so compliance runs first as a strict gate, then NFR):

```markdown
4.5. (可选,如 `docs/compliance/sources/` 非空)Superpowers TDD 完成 + UT 完备后,先跑 `/bob-compliance` 做合规校验,产物落 `docs/bob/05-compliance-<story>.md`
```

Note the `4.5.` numbering puts compliance between item 4 and item 5 (the NFR step). If a particular template's `下一步` uses different numbering, adapt to fit while preserving the intent: "compliance reminder BEFORE the NFR reminder". The exact item number is not load-bearing; the presence of `/bob-compliance` and `docs/compliance/sources/` IS.

- [ ] **Step 4: Run the test, expect pass**

Run: `cargo test --test integration bob_spec_mentions_compliance_in_all_three_templates`
Expected: PASS.

- [ ] **Step 5: Run the full test suite — including bob_spec_mentions_nfr_reminder**

Run: `cargo test`
Expected: every pre-existing test still passes, including `bob_spec_mentions_nfr_reminder` (the /bob-nfr count is unchanged).

- [ ] **Step 6: Commit**

```bash
git add src/templates/skills/bob-spec.md tests/integration.rs
git commit -m "$(cat <<'EOF'
feat(spec): add /bob-compliance reminder to Template A/B/C 下一步 sections

Compliance review runs BEFORE NFR review — it's a strict gate (must
not violate 【强制】 conditions), NFR is open-question gathering.
Reminder kicks in only if docs/compliance/sources/ is non-empty.

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
EOF
)"
```

---

## Task 8: init creates compliance dirs + handles --minimal correctly (TDD)

**Files:**
- Modify: `tests/integration.rs`

**Goal:** End-to-end verification that `run-bob init` creates the compliance scaffold correctly in full mode, and correctly skips it in `--minimal` mode (while still installing the skill).

- [ ] **Step 1: Add the failing test**

Append to `tests/integration.rs`:

```rust
#[test]
fn init_creates_compliance_dir_and_sources() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let target = tmp.path();
    std::process::Command::new(run_bob_bin())
        .args(["init", "--dir"])
        .arg(target)
        .status()
        .expect("init");

    assert!(
        target.join("docs").join("compliance").is_dir(),
        "docs/compliance/ must be created"
    );
    assert!(
        target.join("docs").join("compliance").join("sources").is_dir(),
        "docs/compliance/sources/ must be created"
    );
    assert!(
        target.join("docs").join("compliance").join("README.md").is_file(),
        "docs/compliance/README.md must be installed"
    );

    // sources/ must be empty (user populates it)
    let entries: Vec<_> = std::fs::read_dir(target.join("docs").join("compliance").join("sources"))
        .expect("read_dir sources")
        .filter_map(|e| e.ok())
        .collect();
    assert!(
        entries.is_empty(),
        "docs/compliance/sources/ must be empty after init; got {} entries",
        entries.len()
    );

    // No generated files / lock yet (those are runtime products of /bob-compliance)
    assert!(
        !target.join("docs").join("compliance").join(".compliance.lock").exists(),
        ".compliance.lock must NOT exist after init"
    );
}

#[test]
fn init_minimal_skips_compliance_dir_but_installs_skill() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let target = tmp.path();
    std::process::Command::new(run_bob_bin())
        .args(["init", "--minimal", "--dir"])
        .arg(target)
        .status()
        .expect("init --minimal");

    // Skill must still be installed (skills always included in minimal)
    let skill = target
        .join(".claude")
        .join("skills")
        .join("bob-compliance")
        .join("SKILL.md");
    assert!(
        skill.is_file(),
        "minimal must install bob-compliance skill"
    );

    // But the compliance/ dir and README must NOT be created
    assert!(
        !target.join("docs").join("compliance").exists(),
        "minimal must NOT create docs/compliance/"
    );
}
```

- [ ] **Step 2: Run the tests, expect pass on the first**

Run: `cargo test --test integration init_creates_compliance_dir_and_sources init_minimal_skips_compliance_dir_but_installs_skill`

Expected: both PASS — the asset/HARNESS_DIRS plumbing from Task 1 plus the correct `included_in_minimal` flags should already deliver this behavior. If either fails, debug:
- Test 1 fail → check Task 1's HARNESS_DIRS entry and Asset entry
- Test 2 fail → check `included_in_minimal=false` on the README asset and that HARNESS_DIRS is skipped in minimal mode (it is, per init.rs)

- [ ] **Step 3: Run the full test suite**

Run: `cargo test`
Expected: green.

- [ ] **Step 4: Commit**

```bash
git add tests/integration.rs
git commit -m "$(cat <<'EOF'
test(compliance): lock in init / init --minimal compliance scaffold behavior

Full init creates docs/compliance/, sources/, README.md (and sources/
stays empty until user populates it). Minimal init installs the skill
but skips the directory — user can manually mkdir if they want it.

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
EOF
)"
```

---

## Task 9: upgrade preserves user-provided sources (TDD)

**Files:**
- Modify: `tests/integration.rs`

**Goal:** Lock in the contract that `run-bob upgrade` never touches user-provided files in `docs/compliance/sources/` or any runtime products (generated md, .compliance.lock).

- [ ] **Step 1: Add the failing test**

Append to `tests/integration.rs`:

```rust
#[test]
fn upgrade_preserves_user_compliance_sources() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let target = tmp.path();

    // Fresh init
    std::process::Command::new(run_bob_bin())
        .args(["init", "--dir"])
        .arg(target)
        .status()
        .expect("init");

    // User drops a "compliance source file" into sources/
    let user_source = target
        .join("docs")
        .join("compliance")
        .join("sources")
        .join("my-team-rules.md");
    let sentinel = "USER COMPLIANCE SOURCE — must survive upgrade\n";
    std::fs::write(&user_source, sentinel).expect("write user source");

    // User simulates a runtime product: a generated md and a lock file
    let generated = target
        .join("docs")
        .join("compliance")
        .join("my-team-rules.md");
    let generated_content = "# Generated structured md (would normally be produced by /bob-compliance)\n";
    std::fs::write(&generated, generated_content).expect("write generated");
    let lock = target.join("docs").join("compliance").join(".compliance.lock");
    let lock_content = "generated_at: 2026-05-14T00:00:00Z\nsources: []\n";
    std::fs::write(&lock, lock_content).expect("write lock");

    // Run upgrade
    let status = std::process::Command::new(run_bob_bin())
        .args(["upgrade", "--dir"])
        .arg(target)
        .status()
        .expect("upgrade");
    assert!(status.success(), "upgrade failed");

    // All three user/runtime artifacts must be byte-identical after upgrade
    assert_eq!(
        std::fs::read_to_string(&user_source).expect("read"),
        sentinel,
        "sources/my-team-rules.md must NOT be touched by upgrade"
    );
    assert_eq!(
        std::fs::read_to_string(&generated).expect("read"),
        generated_content,
        "generated my-team-rules.md must NOT be touched by upgrade"
    );
    assert_eq!(
        std::fs::read_to_string(&lock).expect("read"),
        lock_content,
        ".compliance.lock must NOT be touched by upgrade"
    );
}
```

- [ ] **Step 2: Run the test, expect pass**

Run: `cargo test --test integration upgrade_preserves_user_compliance_sources`
Expected: PASS — upgrade's existing scope (only `upgrade_safe` assets in `HARNESS_ASSETS`) means it never touches anything in `sources/` or any non-asset file in `docs/compliance/`. The README is in `HARNESS_ASSETS` (with `upgrade_safe=true`) and may be refreshed if the embedded template changed; the test doesn't assert on README, so that's fine.

If this test fails on a fresh run, it indicates a real bug in scope-creep — investigate which file upgrade touched and why.

- [ ] **Step 3: Run the full test suite**

Run: `cargo test`
Expected: green.

- [ ] **Step 4: Commit**

```bash
git add tests/integration.rs
git commit -m "$(cat <<'EOF'
test(compliance): lock in upgrade-preserves-user-sources invariant

upgrade must never touch sources/, generated md, or .compliance.lock —
those are user-owned or runtime artifacts of /bob-compliance, not
harness assets.

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
EOF
)"
```

---

## Task 10: Final verification

**Files:**
- None (verification only)

**Goal:** Confirm the full implementation lands cleanly — all tests pass, release build is clean, and manual smoke tests exercise the end-to-end behavior.

- [ ] **Step 1: Run the full test suite**

Run: `cargo test`
Expected: every test passes. New tests added by this plan:
- `init_creates_compliance_readme_with_pmd_note` (Task 2)
- `init_creates_bob_compliance_skill` (Tasks 3-5, accumulated tokens)
- `claude_md_has_r13_compliance` (Task 6)
- `bob_spec_mentions_compliance_in_all_three_templates` (Task 7)
- `init_creates_compliance_dir_and_sources` (Task 8)
- `init_minimal_skips_compliance_dir_but_installs_skill` (Task 8)
- `upgrade_preserves_user_compliance_sources` (Task 9)

Plus the existing test `upgrade_safe_field_matches_category_policy` continues to pass with the extended allowlist (Task 1).

- [ ] **Step 2: Build release**

Run: `cargo build --release`
Expected: clean release build, no warnings.

- [ ] **Step 3: Manual smoke test — init creates the compliance scaffold**

```bash
rm -rf /tmp/run-bob-compliance-smoke && mkdir /tmp/run-bob-compliance-smoke
./target/release/run-bob init --dir /tmp/run-bob-compliance-smoke
ls -la /tmp/run-bob-compliance-smoke/docs/compliance/
cat /tmp/run-bob-compliance-smoke/docs/compliance/README.md | head -20
ls /tmp/run-bob-compliance-smoke/.claude/skills/bob-compliance/
```

Expected output:
- `docs/compliance/` exists with `README.md` and `sources/` subdirectory
- `sources/` is empty
- `bob-compliance/SKILL.md` exists
- README starts with `# 项目级合规检查` and mentions PMD/SonarQube

- [ ] **Step 4: Manual smoke test — init --minimal skips compliance dir**

```bash
rm -rf /tmp/run-bob-compliance-smoke && mkdir /tmp/run-bob-compliance-smoke
./target/release/run-bob init --minimal --dir /tmp/run-bob-compliance-smoke
ls /tmp/run-bob-compliance-smoke/docs/ 2>&1
ls /tmp/run-bob-compliance-smoke/.claude/skills/
```

Expected:
- `docs/` does not exist (or contains no `compliance/` subdir)
- `.claude/skills/bob-compliance/` IS present

- [ ] **Step 5: Manual smoke test — CLAUDE.md contains R13**

```bash
rm -rf /tmp/run-bob-compliance-smoke && mkdir /tmp/run-bob-compliance-smoke
./target/release/run-bob init --dir /tmp/run-bob-compliance-smoke
grep -A 5 "^### R13" /tmp/run-bob-compliance-smoke/CLAUDE.md
```

Expected: R13 section is present, mentions `docs/compliance/` and `/bob-compliance`.

- [ ] **Step 6: Manual smoke test — bob-spec template mentions compliance**

```bash
grep -c "/bob-compliance" /tmp/run-bob-compliance-smoke/.claude/skills/bob-spec/SKILL.md
```

Expected: count ≥ 3 (one per template).

- [ ] **Step 7: (Optional) Push or open a PR**

This task does NOT auto-push — leave that to the user, same as the gitignore feature.

---

## Summary of new tests

After completion the repo will have **7 new integration tests** in `tests/integration.rs`:

1. `init_creates_compliance_readme_with_pmd_note` (Task 2)
2. `init_creates_bob_compliance_skill` (Tasks 3, 4, 5 — accumulated tokens across all 5 stages)
3. `claude_md_has_r13_compliance` (Task 6)
4. `bob_spec_mentions_compliance_in_all_three_templates` (Task 7)
5. `init_creates_compliance_dir_and_sources` (Task 8)
6. `init_minimal_skips_compliance_dir_but_installs_skill` (Task 8)
7. `upgrade_preserves_user_compliance_sources` (Task 9)

Plus 1 updated existing test:
- `upgrade_safe_field_matches_category_policy` — drift-guard allowlist extended (Task 1)

All pre-existing tests continue to pass unchanged.
