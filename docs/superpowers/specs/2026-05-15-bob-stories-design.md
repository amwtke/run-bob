# bob-stories · 故事拆分 skill 设计 + bob-survey 改造

> 状态:设计已定稿,待用户审阅 → 转交 `superpowers:writing-plans` 落地。
> 日期:2026-05-15
> 适用仓库:`/Users/xiaojin/workshop/run-bob`
> 相关:
> - 上游 spec:`docs/superpowers/specs/2026-05-14-bob-survey-design.md`(phase 0)
> - 上游 plan:`docs/superpowers/plans/2026-05-14-bob-survey.md`
> - bob 工作流既有 skills:bob-identify / bob-onion / bob-spec / bob-survey

---

## 0. 目的与一句话总结

新增 `/bob-stories <需求>` skill 作为 bob 工作流 phase 1,在 `/bob-survey` 之后、`/bob-identify` 之前接入。给 Medium / Hard 难度需求**1:1 拆 UseCase 故事**(1 story = 1 UseCase),支持 `--refactor` 模式拆 α→γ 改造单元。**同时回头给 `/bob-survey` 加第 4 因子"前置重构量"**,使 survey 的难度判定包含"为接这个需求需要先动多少现有文件"。

> 让 bob 工作流的中段从"survey 完直接 identify"变成"survey 给完总体判定 → stories 把大需求 1:1 切成 UseCase 故事 → 用户挑一个 story 跑 identify→onion→spec→TDD"。

---

## 1. 背景与现状

### 1.1 现状(phase 0 已交付)

```
需求 → /bob-survey → 推荐 → /bob-identify → /bob-onion → /bob-spec → Superpowers TDD
                              ↑                                       ↓
                              (这一步对 Medium/Hard 需求承载过重)
```

`/bob-survey` 输出 G/β/γ + 总分 + 3 因子难度 + 3 档推荐。Medium/Hard 时推荐 B2 清洁孤岛或先重构,但**没有具体拆 story 这一步**。用户拿到推荐后,直接把整个需求扔给 `/bob-identify`,identify 一锅煮所有概念 / UseCase / Entity / 状态机,**上下文密度过高**。

### 1.2 缺什么

1. **缺中间拆分步**:Medium/Hard 需求要按 UseCase 一份份切,每份单独走 identify→onion→spec
2. **缺重构-需求耦合视角**:survey 现有 3 因子(跨环数 / 状态机增量 / legacy 复用)**不直接量化**"接这个新需求要先动多少现有文件"
3. **缺 identify 的 per-story 入口**:identify 现在只吃整段描述,没有 `--story <path>` 输入

### 1.3 phase 1 范围

| 工作项 | 改什么 | 复杂度 |
|---|---|---|
| **A** `/bob-survey` 改造 | 加第 4 因子 "前置重构量";改推荐矩阵 Medium/Hard 格的下一步命令 | 中 |
| **B** `/bob-stories` 新增 | 全新 skill,feature + refactor 双模式 | 大 |
| **C** `/bob-identify` 微调 | soft 检测 stories 索引;新增 `--story <path>` 入口约定 | 小 |

### 1.4 关键约束

- **不动 `/bob-onion` / `/bob-spec`**:它们消费的是 ARCHITECTURE.md 与 spec,跟 stories 解耦
- **`/bob-stories` 自成体系**:不主动调 archcheck / survey,但读取 survey 的最新报告作为输入
- **三段式 + TL 风**:继承现有 bob skills
- **`superpowers-to-trae` 仅作 CLI 风格参考**:不导入代码 / 模板

---

## 2. `/bob-survey` 改造(第 4 因子)

### 2.1 新增因子:前置重构量

| 等级 | 标准 |
|---|---|
| **Easy** | 需要动 0-2 个现有文件 |
| **Medium** | 3-7 个现有文件 |
| **Hard** | 8+ 个现有文件 或 跨模块 |

判定方式:LLM 三段式追问 + survey 自己的 6 维度评分证据可辅助引证。

Stage 2 新增 Q4 段:

> **Q4: 接这个需求需要先动多少现有文件?**
>
> **推测**:从 6 维度评分里看,有 4 处违规 + 需求又点了 OrderService → 推估 5 个文件。
> **理由**:OrderService、OrderRepository 需要抽端口、`@Transactional` 收敛、状态机上提。
> **推荐选择**:`Medium`(3-7)
>
> 是否同意?(回"是"走推荐;回"否,理由是 ..."重判;回"否,我选 Hard"切到 Hard)

### 2.2 组合规则(仍沿用 3 因子规则,扩到 4)

- **任一因子 Hard → 总评 Hard**
- 否则 **≥ 2 Medium → 总评 Medium**
- 否则 → **Easy**

### 2.3 推荐矩阵下一步命令变化

绿地不变(均 → `/bob-identify`)。

棕地的 **Medium / Hard 格子下一步命令从 `/bob-identify` 改成 `/bob-stories`**。Easy 仍直接 `/bob-identify`。

具体改动表(完整对照):

| 评分 \ 难度 | Easy | Medium | Hard |
|---|---|---|---|
| **80-100(γ 健康)** | 🟢 `/bob-identify` | 🟢 **`/bob-stories <需求>`**(变)| 🟡 **`/bob-stories <需求>`**(变);或 `/bob-onion --refresh` 增端口 |
| **60-79(β 可接受)** | 🟢 `/bob-identify`(B2 模式)| 🟡 **`/bob-stories <需求>`**(变)+ 提前列 ACL 表 | 🔴 先 `/bob-onion --refactor` 出三动作改造计划 |
| **0-59(α 烂底子)** | 🟡 警告:能做但债会变重;建议 B2 + 隔离严格 | 🔴 先重构再接 | 🔴 拒绝接需求;先 B1 全量重构;给"必须先改完哪 5 个东西"的清单 |

🔴(0-59 + Medium)/(0-59 + Hard)/(60-79 + Hard) 不变,因为这些情况下推荐的是先全量重构,不进 stories 阶段。

### 2.4 实施变更点

| 文件 | 改动 |
|---|---|
| `src/templates/skills/bob-survey.md` | Stage 1 增加第 4 因子描述;Stage 2 加 Q4 段;Stage 3 矩阵下一步命令更新 |
| `tests/integration.rs::init_creates_bob_survey_skill` | 新增 token:`Q4` / `前置重构量` / `/bob-stories` |
| `docs/superpowers/specs/2026-05-14-bob-survey-design.md` | 末尾加 §11 "v2 修订" 说明第 4 因子的引入 |

ARCHITECTURE.md §12 表头**暂不动**——表是 free-form,survey 在 §6 评分明细里列因子,§9(skill 输出报告)里展开。

---

## 3. `/bob-stories` CLI 表面

```
/bob-stories <需求>                  # 主入口:从需求拆 UseCase
/bob-stories --refactor [path]       # 纯重构模式:拆 α→γ 改造单元
/bob-stories --from-survey <path>    # 显式指定 survey 报告
/bob-stories --refresh               # 已有 02-stories-*.md 时强制重跑
```

自然语言触发:"拆 story"、"把这个需求拆开"、"按 UseCase 切一切"、"先拆几个故事"。

### 3.1 自动检测

- **survey 读取**:若 `docs/bob/00-survey-*.md` 距今 < 7 天,自动读取作为输入
- **模式提示**:若 `<需求>` 里含"重构"/"refactor"/"改造"/"port"/"adapter"等关键词,三段式追问是否切到 `--refactor` 模式
- **自动双模式**:若 survey 第 4 因子 `前置重构量 ≥ Medium`,**自动**同时输出"前置重构 stories"和"新功能 stories"两组(见 §4.1)

### 3.2 前置条件

- `<需求>` 必须有(无需求 → 拒绝)
- 项目内有 git(用于读 archcheck-report / survey 输出等)

---

## 4. 输出与文件布局

### 4.1 汇总索引 `docs/bob/02-stories-<slug>-<YYYYMMDD>.md`

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
`/bob-identify <story 描述>`(若不传 `--story` 而手动复述)
```

### 4.2 每个故事 `docs/bob/02-stories/<n>-<slug>.md`

**feature story 模板**:

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
(若已上 survey 推荐)
```

**refactor story 模板**(类型 = refactor):

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

## 5. 工作流(5 个 Stage)

```
Stage 0. 输入归并(读 survey + 需求 + --refactor flag)
Stage 1. 自动识别模式(feature / refactor / 混合)
Stage 2. 三段式提拆法(LLM 提 UseCase / 改造单元列表)
Stage 3. 三段式拆顺序与依赖
Stage 4. 写汇总索引 + 每个 story 明细
```

### Stage 2 示例(三段式)

> **Q1: 这个需求要拆成几个 story?**
>
> **推测**:5 个。新功能 3 个(ApproveOrder / RejectOrder / ViewApprovalHistory),前置重构 2 个(OrderService 状态机上提 / Decorator 收敛)。
> **理由**:你说"审批+驳回+查询" 3 个动作;survey 报告 OrderService.cancel/confirm 还在 service 包(扣分维度 4),先得提到 entity;@Transactional 散在 3 个 service 类(扣分维度 5),收一收。
> **推荐选择**:`5 个 story,先重构 2 个再做新功能 3 个`
>
> 是否同意?(回"是"走推荐;回"否,合并 R2 进 1"重判;回"否,我重新画一下"切到手动列表)

### Stage 3 示例(三段式)

> **Q2: 顺序与依赖?**
>
> **推测**:R1 → R2 → 1 → 2 → 3。
> **理由**:R1(状态机上提)是 1/2/3 的前置;R2(decorator 收敛)如果想让 1 的事务边界干净也得先做。3(查询)不依赖 R1/R2,可并行,但放最后保稳。
> **推荐选择**:`R1 → R2 → 1 → 2 → 3`
>
> 是否同意?

### Stage 4 写文件细则

- 汇总索引文件路径:`docs/bob/02-stories-<slug>-<YYYYMMDD>.md`
- 故事文件目录:`docs/bob/02-stories/`(若不存在则创建)
- 故事文件命名:
  - feature:`<n>-<usecase-kebab>.md`(n = 1, 2, 3...)
  - refactor:`R<n>-<unit-kebab>.md`(n = 1, 2, 3...)

---

## 6. `/bob-identify` 集成

`bob-identify` SKILL.md 在已有 "## 先检查 /bob-survey (soft 前置)" 段下方,加一段:

```markdown
## 再检查 /bob-stories (soft 前置)

若 survey 难度 ≥ Medium **且** `docs/bob/02-stories-*.md` 不存在,三段式追问是否先跑 `/bob-stories`。

若有 `02-stories-*.md` 而用户直接跑 `/bob-identify <需求>` 不指定 `--story`,三段式追问:"看起来你已拆过 N 个 story,要不要先指明哪个 story?"

### --story <path> 入口约定

`/bob-identify --story docs/bob/02-stories/01-approve-order.md`

行为:从 story 文件读 §1 目标 + §2 用户故事 / 改造范围,作为 identify 的输入。等价于把 story 内容 inline 传给 `/bob-identify`。

注意:这是 skill 模板约定的调用形式,不是 run-bob CLI 增加新 flag。
```

**不强制**。用户回"否"或不带 `--story` 则照旧。

---

## 7. 测试

继续 phase 0 风格——只测 token + 文件落位:

1. **`init_creates_bob_stories_skill`** — `.claude/skills/bob-stories/SKILL.md` 存在,含 Stage 0-4 / 三段式 / `--refactor` / `--from-survey` / `汇总索引` / "前置重构" / "新功能" / `02-stories-*.md` 等 key token
2. **`bob_survey_mentions_fourth_factor`** — `bob-survey.md` 含 `前置重构量` / `Q4` / `/bob-stories`
3. **`bob_identify_mentions_stories_soft_prompt`** — `bob-identify.md` 含 `02-stories-*.md` / `--story`

更深的 fixture-based 行为验证(LLM 拆得对不对、依赖图准不准)继续延后,与 phase 0 处理方式一致。

---

## 8. 与 ARCHITECTURE.md 的关系

`/bob-stories` 不写 ARCHITECTURE.md。

未来若希望"哪些 story 已完成"被 ARCHITECTURE.md 引用,可在 §12 体检记录的"详报"列追加 stories 索引路径。本设计**不**做这一步——避免 ARCHITECTURE.md 被 phase 1 二次污染,等用户真有这个 use case 再补。

---

## 9. 决策记录

| 维度 | 决策 |
|---|---|
| 故事粒度 | 1 story = 1 UseCase(feature)或 1 原子改造单元(refactor) |
| 触发位置 | survey 后 / identify 前;Medium/Hard 触发(Easy 跳过) |
| 重构需求支持 | `--refactor` 模式 + survey 第 4 因子驱动的自动双模式 |
| 输出布局 | 1 份汇总 + N 份明细(在 docs/bob/02-stories/) |
| 拆分机制 | LLM 三段式提推荐,用户可调 |
| survey 第 4 因子 | "前置重构量",rubric: 0-2 / 3-7 / 8+ 文件 |
| 组合规则 | 沿用"任一 Hard → Hard;≥2 Medium → Medium;else Easy" |
| 集成 identify | soft 检测 + `--story <path>` 约定(skill 模板内,非 CLI flag) |
| ARCHITECTURE.md 改动 | 无 |
| 测试 | 仅 token + 文件落位 |

---

## 10. 实施草图(供 writing-plans 起步)

预计 5 个 task:

1. 改造 `/bob-survey`(加第 4 因子 + 矩阵下一步)+ 修订旧 spec § "v2"
2. 创建 `/bob-stories` skill 模板(全文)+ HARNESS_ASSETS 注册 + token 测试
3. 微调 `/bob-identify`(加 stories soft 段 + `--story` 约定)+ token 测试
4. 旧 plan 的衔接 / README "five skills" 更新(可选)
5. 整体 smoke 测 + push

详细拆解由 `writing-plans` 出。

---

## 11. 转交

设计定稿后转交 `superpowers:writing-plans`,产出可执行实施计划。
