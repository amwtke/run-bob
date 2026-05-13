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

## 工作流(5 个 Stage + 1 个 refactor 专属)

```
Stage 0. 输入归并(读 survey + 需求 + --refactor flag)
Stage 1. 自动识别模式(feature / refactor / 混合)
Stage 2. 三段式提拆法(LLM 提 UseCase / 改造单元列表)
Stage 2.5. 测试覆盖体检(全分支级,refactor / 混合模式专属)
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

跑完 `/bob-stories` 后,**不自动调用** `/bob-identify`。输出"下一步"命令,由用户自行决定何时开始下一个 story。

用户可使用 `/bob-identify --story <path>` 把 story 文件作为输入:identify 会读取文件 §1 目标 + §2 用户故事 / 改造范围,等价于把内容 inline 传入。这是 skill 模板层的约定,不是 run-bob CLI 新增 flag。

## TL 风对话

跑完拆分后,**主动提醒**(像真 TL 一样):

- "我把 R2 放到了 R1 之后,因为 decorator 收敛依赖状态机已经上提。如果你想并行做,可以接受 R2 中 @Transactional 注解临时多处共存,等 R1 完了再清。"
- "story 3(查询)和 R1/R2 解耦,可以让另一个人并行做。"

用户回"否"或"我换种顺序" → 尊重决定,在索引底部追加 "用户调整后顺序:..." 一行。
