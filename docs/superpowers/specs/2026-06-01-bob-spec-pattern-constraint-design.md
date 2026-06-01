# run-bob · 可选「涉及设计模式」约束设计(bob-spec 声明 → bob-compliance 机检)

> 设计日期:2026-06-01 · 目标版本:v0.8.0
> 状态:待实现(交给 `superpowers:writing-plans` 接力)

## 0. 目的与一句话总结

在 **bob-spec 收尾**时增加一个 **opt-in 轻探针(默认关)**:Claude 分析当前用例、**建议**几个可用的 GoF 设计模式、问用户**采纳哪些**;采纳的模式以 `PAT-N-k` ID 写进 spec 的 `## 9.5 涉及设计模式` 段,成为交给 Superpowers 的**硬约束**。Superpowers TDD 实现后,**bob-compliance 增一类「模式符合度」机检**,把 diff 里的类/结构回贴到声明模式,缺失则按档位报不通过。

全程 opt-in:不开探针 = 整个特性休眠,对未采用的用户**零噪音**。

## 1. 背景与动机

### 1.1 现状

run-bob 的 bob-* 链路已覆盖 survey → model → stories → identify → onion → spec → (Superpowers TDD) → compliance / nfr。其中:

- **bob-spec**(`src/templates/skills/bob-spec.md`)是建模阶段→实现阶段的桥梁,产出 `docs/specs/spec-*.md`,含 §9 Guardrails / §10 开放问题 / §11 下一步,走「三段式提问规约」。
- **bob-compliance**(`src/templates/skills/bob-compliance.md`)是 per-story 实施后的合规 review,5-Stage,规则带 `[STANDARD-§-n]` ID + 【强制】/【推荐】/【参考】档位,对 story diff 回贴校验,产出 `docs/bob/05-compliance-*.md`。

### 1.2 痛点 / 需求

实现阶段缺一个**可选**约束:让「实现的代码必须对应到特定的设计模式上」可被声明、可被机检。Bob 富 Entity + UseCase 编排默认覆盖多数场景,但出现「多分支策略 / 状态爆炸 / 复杂构造 / 横切装饰」信号时,希望能显式立 GoF 模式并在实现后验证落地。

### 1.3 范围(本次)

- ✅ bob-spec:opt-in 轻探针 + 信号驱动的模式建议 + spec `## 9.5` 新段(schema + `PAT-N-k` ID)
- ✅ bob-compliance:模式符合度作为第二类规则源接入 5-Stage(装载 / 定位 / 校验 / 分类 / 报告 + 空 sources carve-out)
- ✅ 2 条新增集成测试 + 版本 bump 至 v0.8.0 + README Status 更新
- ❌ **不改** CLAUDE.md(刻意 YAGNI,见 §6.2)
- ❌ **不改** Rust 逻辑 / assets.rs(纯模板增内容)
- ❌ **不引入** AST/字节码硬断言工具(机检由 Claude 锚「可观察痕迹」判读,与 bob-compliance 现有语义判断同档)

## 2. 模式语义与决策来源(已澄清)

| 决策点 | 结论 |
|---|---|
| 「涉及模式」指什么 | **GoF 设计模式**(Strategy / State / Factory Method / Builder / Template Method / Decorator / Observer …) |
| 在哪里声明 | bob-spec 收尾的 opt-in 步骤,写进 spec `## 9.5` 段(spec 是模式约束 SSoT) |
| 触发方式 | **轻探针默认关**:bob-spec 用一句三段式问「要不要立模式?」默认推荐跳过 |
| 闭环到哪 | spec 声明 + **实现后 /bob-compliance 机检**(真闭环) |
| 校验者 | **复用 bob-compliance**,不新增 skill |
| CLAUDE.md | 不动 |
| 版本 | v0.8.0(minor) |

## 3. 数据流与契约

```
/bob-spec ──采纳模式──► docs/specs/spec-N-*.md「## 9.5 涉及设计模式」(PAT-N-k 行)
                                   │
                          (Superpowers TDD 实现)
                                   │
/bob-compliance ──grep §9.5──► PAT-N-k 当规则,回贴 story diff ──► 05-compliance-*.md「## 模式符合度」段
```

**契约 marker**:spec 段固定标题 `## 9.5 涉及设计模式`。默认不开启时**整段省略**。bob-compliance 靠 grep 该标题定位;段内是机器可解析表格。无此段 = 该 spec 不约束模式 = compliance 静默跳过。

## 4. bob-spec 改动(`src/templates/skills/bob-spec.md`)

### 4.1 新增 Step S5:设计模式建议(可选 · 轻探针默认关)

放在现有 `Step S4 回写检测` 之后(spec 收尾),严格走已有「提问规约(强制三段式)」,默认推荐**跳过**:

```
> **S5: 要不要为本用例显式立 GoF 设计模式?**
>
> 推测:默认**跳过**——本用例是 <CRUD/单状态迁移编排>,富 Entity + UseCase 编排已覆盖,套模式反增复杂度。
> 理由:Bob 严派只在出现明确信号时才立模式(多分支策略 / 状态爆炸 / 复杂构造 / 横切装饰);R3 富 Entity 默认不需要。
> 推荐选择:`跳过(本 spec 不约束设计模式)`
>
> 是否同意?(回"是"跳过;回"做建议"进入模式分析;回"否,我要用 X"直接登记 X)
```

### 4.2 开启后:信号驱动的候选建议(2-4 个)→ 问采纳

skill 内置「信号 → 模式」判别表,强制锚到本用例的具体 Entity/UseCase/端口,每个候选必须给「不立的代价」:

| 出现的信号 | 候选 GoF 模式 | 典型落点包 |
|---|---|---|
| 同一动作按类型走多分支(运费 / 计价 / 风控规则) | **Strategy** | `usecase/port` 接口 + `adapter/` 实现 |
| Entity 状态多、迁移复杂、`switch(status)` 散落 | **State** | `entity/` 内 |
| 构造步骤多 / 可选参数爆炸 | **Builder / Factory Method** | `entity/` 或 `usecase` |
| 一组算法骨架相同、步骤可替换 | **Template Method** | `usecase` |
| 横切包裹(已有 `TransactionalUseCaseDecorator` 即此) | **Decorator** | `framework` |

采纳交互仍走三段式(推荐一个子集)。采纳上限**软建议 ≤3 个/spec**(过多 = 用例该拆的信号)。

### 4.3 spec 新段 schema(写进模板 A,标题固定,采纳才出现)

```markdown
## 9.5 涉及设计模式(可选 · 仅本 spec 开启模式建议时存在)

> 采纳的模式是 Superpowers 实现的**硬约束**,/bob-compliance 会回贴校验。

| ID | 模式 | 为什么(锚本用例) | 角色映射 | 可观察痕迹(机检锚点) | 档位 |
|---|---|---|---|---|---|
| PAT-1-1 | Strategy | 运费按 region 多分支(§5 端口 ShippingFeeCalc) | Context=`PlaceOrderUseCase`;Strategy 接口=`ShippingFeeStrategy`;≥2 实现 | 存在接口 `*ShippingFeeStrategy` 落 `usecase/port` + ≥2 实现落 `adapter/`;Context 依赖接口非具体类 | 【强制】 |
```

- **`PAT-N-k` ID**:借 bob-compliance 的 `[STANDARD-§-n]` 习语(N=spec 序号,k=序号),让 compliance 报告能像 `[ALI-1.1.2]` 一样引用。
- **「可观察痕迹」是设计命门**:必须 grep / 结构可判定(接口名约定 + 落点包 + 实现数 + 依赖方向),否则机检空对空。
- **档位**复用【强制】/【推荐】词表,采纳默认【强制】。

### 4.4 §11 下一步

在已有 compliance 提醒后追一句:「若本 spec 声明了 §9.5,`/bob-compliance` 会附带跑模式符合度」。**保持测试要求的 `/bob-compliance` ≥3 次、`/bob-nfr` ≥3 次提及不变**(纯增,不减)。

> 注:§9.5 主体落在**模板 A(命令型)**;模板 B(查询)/ C(重构)以一句指针引用同一节,避免重复维护。

## 5. bob-compliance 改动(`src/templates/skills/bob-compliance.md`)

模式符合度作为**第二类规则源**接入,与 `docs/compliance/sources/` 并列但独立,嵌入现有 5-Stage:

### 5.1 Stage 0 状态探测 —— 加 carve-out

判定表新增一行:

| 状态 | 条件 | 行为 |
|---|---|---|
| **空仓但有模式** | `sources/` 空,但 scope 内 ≥1 个 spec 含 `## 9.5 涉及设计模式` | 跳过 sources 规则,**仍进 Stage 3 只跑模式符合度** |

(原「空仓软退出」仅在**既无 sources 又无 spec 模式**时触发。)

### 5.2 Stage 2 装载 —— 多读一个规则源

现有装载 `docs/compliance/*.md` 之外,额外 grep `docs/specs/spec-*.md` 的 `## 9.5` 段,每行解析成:

```
PAT-N-k → (spec文件, 模式名, Context类, 参与角色, 可观察痕迹, severity)
```

并入同一内存规则索引,`PAT-*` 与 `ALI-*` 走同一套校验/分类/报告管线。Q1 装载通报多报一行「+ J 条模式规则,来自 K 份 spec」。

### 5.3 Stage 3.1 定位 —— story→spec 链接

现有 4 级范围优先级(显式 story / 当前 story / fallback / all-branch)**不变**。确定 diff 范围后**收敛相关 spec**:取 §9.5 里 `Context 类 / 落点包` 路径**与 diff 变更文件有交集**的 spec —— 只校验「diff 动了其 Context/参与类」的模式,避免全仓 spec 误判。

### 5.4 Stage 3.2 校验 —— 「可观察痕迹」当 checklist

对每条 `PAT-N-k`,Claude 按声明的可观察痕迹逐项核对 diff + 相关已存在文件。Strategy 例:① 接口 `*ShippingFeeStrategy` 存在且落 `usecase/port` ② 实现数 ≥2 ③ Context 依赖接口而非 `new` 具体类 —— 三项全过 = PASS。

### 5.5 Stage 3.3 分类 —— 复用现有三类

| 类 | 模式语境含义 |
|---|---|
| **违反** | 声明了模式但 diff 找不到对应结构(接口缺失 / 只 1 实现 / Context 直接耦合具体类)→ 按档位(采纳默认【强制】) |
| **待量化** | spec §9.5 没给「可观察痕迹」或太模糊 → 建议回 `/bob-spec` 补痕迹 |
| **豁免** | spec §10 开放问题显式注明降级理由 |

### 5.6 Stage 4 报告 —— 加一段

`05-compliance-*.md`「违反清单」后插 `## 模式符合度`,逐 `PAT-N-k` 列 PASS/FAIL + 位置 + 期望结构 vs 实际 + 修复建议。**全 PASS 或无模式规则时**该段写「无模式约束 / 全部符合」,不污染报告。

## 6. 护栏与设计取舍

### 6.1 反过度设计护栏(写进 bob-spec,与 Bob 哲学一致)

- 探针**默认关、默认推荐跳过**;建议步骤强制走「信号 → 模式」表,**禁止为纯 CRUD / 单状态迁移套模式**。
- 每个候选必须给「不立的代价」。
- 采纳软上限 ≤3 个/spec(过多 = 用例该拆)。

### 6.2 CLAUDE.md 不动(刻意 YAGNI)

加硬规则 = 全局常开常审,与「可选」矛盾。模式约束的开关权完全在 spec §9.5 的有无;CLAUDE.md 保持纯净。

### 6.3 留痕一致性

`PAT-N-k` ID 贯穿 spec↔report;模式 severity 复用【强制】/【推荐】词表;三段式提问全程复用 —— 新能力在两个 skill 里**零新习语**,认知负担最小。

## 7. 工程影响

### 7.1 分发 / upgrade

仅 2 个 skill 模板纯增内容 → 两者 `upgrade_safe: true`,老项目 `run-bob upgrade` 自动同步。`assets.rs`、Rust 逻辑**零改动**(无新增文件)。

### 7.2 版本

`Cargo.toml` `0.7.7 → 0.8.0`(minor,面向用户新能力)+ 一条 `chore(release)` commit + README「Status」更新。release.yml 靠 tag 触发,机械收尾。

## 8. 测试策略

现有断言全部是纯增不破坏(`/bob-compliance`≥3 次、`/bob-nfr`≥3 次、关键 token、frontmatter name)。**新增 2 条集成测试**:

1. `bob-spec/SKILL.md` 含 `涉及设计模式` + `PAT-` + 「轻探针默认跳过」关键词。
2. `bob-compliance/SKILL.md` 含 `模式符合度` + 「空仓但有模式」carve-out 关键词 + grep `docs/specs` 装载关键词。

并跑全量 `cargo test`(15+ 集成测试)确保绿。

## 9. 风险与对策

| 风险 | 对策 |
|---|---|
| 用户为 CRUD 滥用模式 → 过度抽象 | 探针默认关 + 「信号→模式」表 + 「不立的代价」+ ≤3 软上限 |
| 「可观察痕迹」写得太虚 → 机检空对空 | spec 段强制要求痕迹可 grep/结构判定;太虚的归「待量化」回退补痕迹 |
| story→spec 多对多定位误判 | 仅校验 diff 与 Context/落点包有交集的 spec |
| 空 sources 误软退出漏跑模式 | Stage 0 carve-out「空仓但有模式」 |
| 报告噪音(未采用者) | 无模式规则时段落写空,且 sources 空+无模式仍软退出 |

## 10. 实施计划(供 `superpowers:writing-plans` 接力)

1. 改 `bob-spec.md`:加 Step S5 + 模板 A `## 9.5` + 「信号→模式」表 + §11 指针(保持 `/bob-compliance`/`/bob-nfr` ≥3 次)。
2. 改 `bob-compliance.md`:Stage 0 carve-out / Stage 2 装载 spec 模式 / Stage 3.1 定位 / Stage 3.2-3.3 校验分类 / Stage 4 报告段。
3. 加 2 条集成测试。
4. `cargo test` 全绿。
5. bump `Cargo.toml` 至 0.8.0 + README Status。
6. 提交(feat + chore(release))。
