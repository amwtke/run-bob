# 可选「涉及设计模式」约束 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 给 run-bob 的 bob-spec / bob-compliance 两个 skill 模板加一条 opt-in 的「涉及设计模式」约束:bob-spec 收尾建议 GoF 模式并记入 spec §9.5,bob-compliance 实现后机检模式符合度。

**Architecture:** 纯模板内容增量(2 个 `.md` 文件),由 `tests/integration.rs` 的「init 到 tempdir → 读已装 SKILL.md → 断言 token」做守卫。无 Rust 逻辑 / assets.rs 改动。两个 skill 均 `upgrade_safe: true`,老项目 `run-bob upgrade` 自动同步。

**Tech Stack:** Rust 2021 / cargo test(集成测试)/ Markdown 模板(`include_str!` 编译期内联)。

**设计依据:** `docs/superpowers/specs/2026-06-01-bob-spec-pattern-constraint-design.md`

---

## 文件结构

| 文件 | 职责 | 改动 |
|---|---|---|
| `src/templates/skills/bob-spec.md` | 模式建议交互 + spec §9.5 声明段 | 加 Step S5 + 模板 A §9.5 + §11 指针 |
| `src/templates/skills/bob-compliance.md` | 模式符合度机检 | Stage 0 carve-out / Stage 2 装载 / Stage 3 校验 / Stage 4 报告段 |
| `tests/integration.rs` | token 守卫 | 加 3 个 `#[test]` |
| `Cargo.toml` | 版本 | `0.7.7 → 0.8.0` |
| `README.md` | Status | 更新版本与能力描述 |

**TDD 节奏(本仓库特例):** 这些是模板内容编辑,「失败的测试」= 断言新 token 尚不存在(红)→ 编辑模板加入内容 → 重跑(绿)→ commit。`cargo test` 会重新编译,把新模板 `include_str!` 进二进制,所以测试能看到新内容。

---

## Task 1: bob-spec — Step S5 模式建议探针(默认关)

**Files:**
- Test: `tests/integration.rs`(新增 `bob_spec_has_pattern_probe`)
- Modify: `src/templates/skills/bob-spec.md`(在 `Step S4` 之后、`---` / `## 模板 A` 之前插入 Step S5)

- [ ] **Step 1: 写失败测试**

在 `tests/integration.rs` 末尾追加:

```rust
#[test]
fn bob_spec_has_pattern_probe() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let target = tmp.path();
    Command::new(run_bob_bin()).args(["init", "--dir"]).arg(target).status().expect("init");

    let p = target.join(".claude").join("skills").join("bob-spec").join("SKILL.md");
    let content = std::fs::read_to_string(&p).expect("read bob-spec");

    for token in &[
        "Step S5",
        "要不要为本用例显式立 GoF 设计模式",
        "跳过(本 spec 不约束设计模式)",
        "信号 → 模式",
        "不立的代价",
        "Strategy",
        "State",
        "Template Method",
        "Decorator",
    ] {
        assert!(content.contains(token), "bob-spec must mention {} (Step S5)", token);
    }
}
```

- [ ] **Step 2: 跑测试确认失败**

Run: `cargo test --test integration bob_spec_has_pattern_probe`
Expected: FAIL —— `bob-spec must mention Step S5 (Step S5)`(内容尚未加)

- [ ] **Step 3: 编辑模板加入 Step S5**

在 `src/templates/skills/bob-spec.md` 中,定位到 Step S4 的结尾这一行:

```
→ 这一步是关键纪律,防止 spec 引入 ARCHITECTURE.md 没登记的术语。
```

在该行**之后、紧接着的 `---` 之前**,插入:

````markdown

### Step S5:设计模式建议(可选 · 轻探针默认关)

spec 主体渲染后(S3)、回写检测(S4)通过后,收尾时**问一句**是否要为本用例显式立 GoF 设计模式。
默认推荐**跳过**——Bob 富 Entity(R3)+ UseCase 编排已覆盖多数场景,为 CRUD / 单状态迁移硬套模式只会增复杂度。

> **S5: 要不要为本用例显式立 GoF 设计模式?**
>
> **推测**:默认**跳过**——本用例是 <CRUD / 单状态迁移编排>,富 Entity + UseCase 编排已覆盖。
> **理由**:Bob 严派只在出现明确信号时才立模式(多分支策略 / 状态爆炸 / 复杂构造 / 横切装饰);R3 富 Entity 默认不需要。
> **推荐选择**:`跳过(本 spec 不约束设计模式)`
>
> 是否同意?(回"是"跳过,不写 §9.5;回"做建议"进入模式分析;回"否,我要用 X"直接登记 X 到 §9.5)

**若用户开启** → 出「信号驱动」的候选建议(2-4 个),每个候选必须锚到本用例的具体 Entity / UseCase / 端口,并给出**不立的代价**,杜绝为 CRUD 硬套:

| 出现的信号 | 候选 GoF 模式 | 典型落点包 |
|---|---|---|
| 同一动作按类型走多分支(运费 / 计价 / 风控规则) | **Strategy** | `usecase/port` 接口 + `adapter/` 实现 |
| Entity 状态多、迁移复杂、`switch(status)` 散落 | **State** | `entity/` 内 |
| 构造步骤多 / 可选参数爆炸 | **Builder / Factory Method** | `entity/` 或 `usecase` |
| 一组算法骨架相同、步骤可替换 | **Template Method** | `usecase` |
| 横切包裹(已有 `TransactionalUseCaseDecorator` 即此) | **Decorator** | `framework` |

> 以上是「**信号 → 模式**」判别表:**先有信号,再立模式**。无信号不建议。采纳上限软建议 ≤3 个/spec(超过 = 用例该拆的信号)。

采纳交互仍走「提问规约(强制三段式)」,推荐一个子集。采纳的模式写进**模板 A §9.5 涉及设计模式**(见下),成为交给 Superpowers 的硬约束。
````

- [ ] **Step 4: 跑测试确认通过**

Run: `cargo test --test integration bob_spec_has_pattern_probe`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add tests/integration.rs src/templates/skills/bob-spec.md
git commit -m "feat(bob-spec): opt-in Step S5 pattern-suggestion probe (default off)

Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>"
```

---

## Task 2: bob-spec — 模板 A §9.5 声明段 + §11 指针

**Files:**
- Test: `tests/integration.rs`(新增 `bob_spec_has_pattern_section`)
- Modify: `src/templates/skills/bob-spec.md`(模板 A:§9 Guardrails 之后插 §9.5;§11 下一步加指针)

- [ ] **Step 1: 写失败测试**

在 `tests/integration.rs` 末尾追加:

```rust
#[test]
fn bob_spec_has_pattern_section() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let target = tmp.path();
    Command::new(run_bob_bin()).args(["init", "--dir"]).arg(target).status().expect("init");

    let p = target.join(".claude").join("skills").join("bob-spec").join("SKILL.md");
    let content = std::fs::read_to_string(&p).expect("read bob-spec");

    for token in &[
        "## 9.5 涉及设计模式",
        "PAT-1-1",
        "可观察痕迹",
        "机检锚点",
        "角色映射",
    ] {
        assert!(content.contains(token), "bob-spec must mention {} (§9.5)", token);
    }

    // 不破坏既有契约:/bob-compliance 仍 ≥3 次提及
    assert!(
        content.matches("/bob-compliance").count() >= 3,
        "bob-spec must still mention /bob-compliance >=3 times"
    );
}
```

- [ ] **Step 2: 跑测试确认失败**

Run: `cargo test --test integration bob_spec_has_pattern_section`
Expected: FAIL —— `bob-spec must mention ## 9.5 涉及设计模式 (§9.5)`

- [ ] **Step 3a: 在模板 A 插入 §9.5**

在 `src/templates/skills/bob-spec.md` 模板 A 内,定位到 §9 Guardrails 的最后一行(以 `mvn test -Dtest=CleanArchitectureTest` 结尾的那条 `✅`),它正好在 `## 10. 交给 Superpowers 的开放问题` 之前。在 `## 10.` **之前**插入:

````markdown
## 9.5 涉及设计模式(可选 · 仅本 spec 在 Step S5 开启模式建议时存在)

> 本段由 `/bob-spec` Step S5 写入。采纳的模式是 Superpowers 实现的**硬约束**,
> `/bob-compliance` 会按「可观察痕迹」回贴校验。未开启 Step S5 时整段省略。

| ID | 模式 | 为什么(锚本用例) | 角色映射 | 可观察痕迹(机检锚点) | 档位 |
|---|---|---|---|---|---|
| PAT-1-1 | Strategy | 运费按 region 多分支(§5 端口 `ShippingFeeCalc`) | Context=`PlaceOrderUseCase`;Strategy 接口=`ShippingFeeStrategy`;≥2 实现 | 存在接口 `*ShippingFeeStrategy` 落 `usecase/port` + ≥2 实现落 `adapter/`;Context 依赖接口非具体类 | 【强制】 |

**字段约定:**
- **ID** = `PAT-<本 spec 序号 N>-<序 k>`,贯穿 spec ↔ `/bob-compliance` 报告(同 `[ALI-1.1.2]` 习语)。
- **可观察痕迹**是**强制字段**,必须可 grep / 结构判定(接口名约定 + 落点包 + 实现数 + 依赖方向);写不出可观察痕迹的模式不要采纳(否则机检空对空)。
- **档位**复用 `/bob-compliance` 的【强制】/【推荐】词表,采纳默认【强制】。

````

- [ ] **Step 3b: 在模板 A §11 下一步加指针**

在模板 A 的 `## 11. 下一步` 代码块里,找到这一行:

```
4.5. (可选,如 `docs/compliance/sources/` 非空)Superpowers TDD 完成 + UT 完备后,先跑 `/bob-compliance` 做合规校验,产物落 `docs/bob/05-compliance-<story>.md`
```

在其**正下方**新增一行(放进同一 ``` 代码块内):

```
4.6. (可选,如本 spec 含 §9.5 涉及设计模式)上一步的 `/bob-compliance` 会**附带跑模式符合度**,把 §9.5 的 PAT-N-k 回贴 diff,缺失按档位报不通过
```

> 注:模板 B(查询)/ C(重构)不复制 §9.5 主体,各加一句指针「设计模式声明见模板 A §9.5,查询/重构型如需立模式同样走 Step S5」。这步在模板 B、C 的「下一步」段各加一句即可,无需测试守卫。

- [ ] **Step 4: 跑测试确认通过**

Run: `cargo test --test integration bob_spec_has_pattern_section bob_spec_mentions_compliance_in_all_three_templates`
Expected: PASS（两者皆绿;后者确认 `/bob-compliance` ≥3 未被破坏)

- [ ] **Step 5: Commit**

```bash
git add tests/integration.rs src/templates/skills/bob-spec.md
git commit -m "feat(bob-spec): add §9.5 涉及设计模式 section + §11 compliance pointer

Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>"
```

---

## Task 3: bob-compliance — Stage 0 carve-out + Stage 2 装载 spec 模式

**Files:**
- Test: `tests/integration.rs`(新增 `bob_compliance_loads_spec_patterns`)
- Modify: `src/templates/skills/bob-compliance.md`(Stage 0 判定表加行;Stage 2 加装载段)

- [ ] **Step 1: 写失败测试**

在 `tests/integration.rs` 末尾追加:

```rust
#[test]
fn bob_compliance_loads_spec_patterns() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let target = tmp.path();
    Command::new(run_bob_bin()).args(["init", "--dir"]).arg(target).status().expect("init");

    let p = target.join(".claude").join("skills").join("bob-compliance").join("SKILL.md");
    let content = std::fs::read_to_string(&p).expect("read bob-compliance");

    for token in &[
        "空仓但有模式",      // Stage 0 carve-out
        "docs/specs/spec-",  // Stage 2 第二类规则源
        "## 9.5 涉及设计模式",
        "PAT-",
    ] {
        assert!(content.contains(token), "bob-compliance must mention {} (load patterns)", token);
    }
}
```

- [ ] **Step 2: 跑测试确认失败**

Run: `cargo test --test integration bob_compliance_loads_spec_patterns`
Expected: FAIL —— `bob-compliance must mention 空仓但有模式 (load patterns)`

- [ ] **Step 3a: Stage 0 判定表加 carve-out 行**

在 `src/templates/skills/bob-compliance.md` 的 `## Stage 0. 状态探测` 判定表里,找到 `**空仓**` 那一行:

```
| **空仓** | `docs/compliance/sources/` 不存在或为空 | 软退出:"无合规要求,跳过"。**写一份空报告留痕**(避免下次重复探测) |
```

把它**替换为**下面两行(空仓收紧 + 新增 carve-out):

```
| **空仓** | `sources/` 空 **且** scope 内无 spec 含 `## 9.5 涉及设计模式` | 软退出:"无合规要求,跳过"。**写一份空报告留痕** |
| **空仓但有模式** | `sources/` 空,但 scope 内 ≥1 个 spec 含 `## 9.5 涉及设计模式` | 跳过 sources 规则,**仍进 Stage 3 只跑模式符合度** |
```

- [ ] **Step 3b: Stage 2 加「第二类规则源:spec 模式」装载段**

在 `## Stage 2. 装载 (双模式都用)` 段,找到这段(读 compliance md、建索引)之后、`**Severity 优先级**` 之前,插入:

````markdown

### 第二类规则源:spec 声明的设计模式

除 `docs/compliance/*.md` 外,**额外 grep `docs/specs/spec-*.md` 的 `## 9.5 涉及设计模式` 段**,把每行解析成一条模式规则并入同一索引:

```
PAT-N-k → (spec文件, 模式名, Context类, 参与角色, 可观察痕迹, severity)
```

`PAT-*` 与 `ALI-*` 走**同一套**校验 / 分类 / 报告管线。无任何 spec 含 §9.5 时,这一类为空,后续模式符合度静默跳过(零噪音)。
````

- [ ] **Step 3c: Stage 2 的 Q1 通报加一行**

在 `## Stage 2` 末尾的 `> **Q1: 装载到 N 条规则,来自 M 份标准。**` 三段式块里,`**推测**` 那一行后补一句:

```
> **推测**:【强制】X 条 / 【推荐】Y 条 / 【参考】Z 条;**+ J 条模式规则(PAT-*),来自 K 份 spec**
```

(替换原 `> **推测**:【强制】X 条 / 【推荐】Y 条 / 【参考】Z 条` 这一行)

- [ ] **Step 4: 跑测试确认通过**

Run: `cargo test --test integration bob_compliance_loads_spec_patterns`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add tests/integration.rs src/templates/skills/bob-compliance.md
git commit -m "feat(bob-compliance): load spec §9.5 patterns as 2nd rule source + empty-sources carve-out

Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>"
```

---

## Task 4: bob-compliance — Stage 3 校验/分类 + Stage 4 报告段

**Files:**
- Test: `tests/integration.rs`(新增 `bob_compliance_checks_pattern_conformance`)
- Modify: `src/templates/skills/bob-compliance.md`(Stage 3.1 定位、3.2 校验、3.3 分类;Stage 4 报告加段)

- [ ] **Step 1: 写失败测试**

在 `tests/integration.rs` 末尾追加:

```rust
#[test]
fn bob_compliance_checks_pattern_conformance() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let target = tmp.path();
    Command::new(run_bob_bin()).args(["init", "--dir"]).arg(target).status().expect("init");

    let p = target.join(".claude").join("skills").join("bob-compliance").join("SKILL.md");
    let content = std::fs::read_to_string(&p).expect("read bob-compliance");

    for token in &[
        "## 模式符合度",       // Stage 4 报告段
        "可观察痕迹",          // Stage 3.2 checklist
        "Context",            // story→spec 定位锚点
        "PASS",               // 逐 PAT 判定
    ] {
        assert!(content.contains(token), "bob-compliance must mention {} (conformance)", token);
    }
}
```

- [ ] **Step 2: 跑测试确认失败**

Run: `cargo test --test integration bob_compliance_checks_pattern_conformance`
Expected: FAIL —— `bob-compliance must mention ## 模式符合度 (conformance)`

- [ ] **Step 3a: Stage 3.1 加 story→spec 收敛**

在 `### 3.1 定位检查范围 (优先级从高到低)` 的 4 级优先级列表**之后**(`向用户三段式确认范围` 之前),插入:

````markdown

**收敛相关 spec(模式符合度专用):** 确定 diff 范围后,取 §9.5 里 `Context 类 / 落点包` 路径**与 diff 变更文件有交集**的 spec —— 只校验「diff 动了其 Context / 参与类」的模式,避免拿全仓 spec 误判。
````

- [ ] **Step 3b: Stage 3.2 加「可观察痕迹当 checklist」**

在 `### 3.2 逐条规则比对` 段尾(`**优先级**:先跑【强制】...` 之后),插入:

````markdown

**模式规则(PAT-*)校验:** 按 spec §9.5 声明的「可观察痕迹」逐项核对 diff + 相关已存在文件。
例(Strategy):① 接口 `*ShippingFeeStrategy` 存在且落 `usecase/port` ② 实现数 ≥2 ③ Context 依赖接口而非 `new` 具体类 —— 三项全过记 `PASS`,任一不满足记 `FAIL`。
````

- [ ] **Step 3c: Stage 3.3 分类表补模式语境**

在 `### 3.3 分类` 的三类表格**下方**补一句:

````markdown

模式规则(PAT-*)套用同一三类:**违反** = 声明了模式但 diff 找不到对应结构(接口缺失 / 只 1 实现 / Context 直接耦合具体类),按档位(采纳默认【强制】);**待量化** = §9.5 没给可观察痕迹或太模糊,建议回 `/bob-spec` 补痕迹;**豁免** = spec §10 开放问题显式注明降级理由。
````

- [ ] **Step 3d: Stage 4 报告加「## 模式符合度」段**

在 `## Stage 4. 报告` 的报告模板里,找到 `## 豁免 (V 条)` 段之后、`## 建议新增 story 清单` 之前,插入(在 ```markdown 报告模板代码块内):

````markdown

## 模式符合度

> 仅当 scope 内 spec 含 §9.5 时出现;无模式规则时写「无模式约束」。

| ID | 模式 | 判定 | 位置 | 期望结构 vs 实际 | 修复建议 |
|---|---|---|---|---|---|
| PAT-1-1 | Strategy | **FAIL** | `usecase/PlaceOrderUseCase.java:88` | 期望:依赖 `ShippingFeeStrategy` 接口 + ≥2 实现;实际:`switch(region)` 内联 | 抽 `ShippingFeeStrategy` 接口落 `usecase/port`,各 region 一个实现落 `adapter/` |

全部 PASS 或无模式规则时,本段写一行:`无模式约束 / 全部符合`。
````

- [ ] **Step 4: 跑测试确认通过**

Run: `cargo test --test integration bob_compliance_checks_pattern_conformance`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add tests/integration.rs src/templates/skills/bob-compliance.md
git commit -m "feat(bob-compliance): pattern-conformance check (locate/verify/classify/report)

Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>"
```

---

## Task 5: 版本 bump v0.8.0 + README + 全量回归

**Files:**
- Modify: `Cargo.toml`(version)
- Modify: `Cargo.lock`(随 build 自动更新 run-bob 条目)
- Modify: `README.md`(Status 行)

- [ ] **Step 1: 改 Cargo.toml 版本**

把 `Cargo.toml` 的:

```toml
version = "0.7.7"
```

改为:

```toml
version = "0.8.0"
```

- [ ] **Step 2: 更新 README Status**

`README.md` 当前 Status 段:

```
## Status

✅ **v0.3.0** — 7 phases live. Spec list under [`docs/superpowers/specs/`](docs/superpowers/specs/).
```

改为:

```
## Status

✅ **v0.8.0** — bob-* pipeline + optional 涉及设计模式 constraint (bob-spec §9.5 → bob-compliance 模式符合度). Spec list under [`docs/superpowers/specs/`](docs/superpowers/specs/).
```

- [ ] **Step 3: 全量回归(构建 + 所有集成测试)**

Run: `cargo test`
Expected: 全绿。包含既有 60+ 测试 + 本次新增 4 个(`bob_spec_has_pattern_probe` / `bob_spec_has_pattern_section` / `bob_compliance_loads_spec_patterns` / `bob_compliance_checks_pattern_conformance`)。
特别确认未回归:`bob_spec_mentions_compliance_in_all_three_templates`、`bob_spec_mentions_nfr_*`、`binary_prints_version`(后者自动读 `0.8.0`)。

- [ ] **Step 4: Commit**

```bash
git add Cargo.toml Cargo.lock README.md
git commit -m "chore(release): bump version to v0.8.0

Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>"
```

---

## 自检(写计划后回看 spec)

- **Spec 覆盖**:§4 bob-spec → Task 1+2;§5 bob-compliance → Task 3+4;§7.2 版本 → Task 5;§8 测试 → 每 Task 内 TDD + Task 5 回归。✅ 无遗漏。
- **占位符**:每个编辑步骤给了完整可插入的 markdown 正文 + 精确锚点 + 精确命令。✅
- **类型/命名一致**:`PAT-N-k`、`## 9.5 涉及设计模式`、`## 模式符合度`、`空仓但有模式`、`可观察痕迹` 在 spec / 计划 / 测试三处拼写一致。✅
- **CLAUDE.md 不动 / assets.rs 不动**:本计划无相关 Task,与设计 §1.3 / §6.2 一致。✅
