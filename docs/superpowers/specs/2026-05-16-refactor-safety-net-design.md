# 重构测试安全网 · 三层联动设计

> 状态:设计已定稿,待用户审阅 → 转交 `superpowers:writing-plans` 落地。
> 日期:2026-05-16
> 适用仓库:`/Users/xiaojin/workshop/run-bob`
> 相关:
> - phase 0 spec:`docs/superpowers/specs/2026-05-14-bob-survey-design.md`
> - phase 1 spec:`docs/superpowers/specs/2026-05-15-bob-stories-design.md`
> - bob 工作流既有 skills:bob-survey / bob-stories / bob-identify / bob-onion / bob-spec

---

## 0. 目的与一句话总结

**让 bob 链上的每次重构都有测试安全网兜底**:在 `bob-stories --refactor` / `bob-identify --refactor` / `bob-spec --refactor` 三层都加入测试覆盖检查,粒度到**全分支级**(每个 if/else/switch case/throw/早 return 都得有测试命中)。无路径漏闸——无论用户从 stories、identify 还是 spec 进入重构,都至少有一道检查。

---

## 1. 背景与现状

### 1.1 现状

`bob-spec` Template C **已经**有 Step 1 "加测试覆盖现状(防止改坏)",但只在 spec 阶段触发,且只要求"覆盖现行为基线",**没规定到分支粒度**。

更早的阶段:
- `bob-stories --refactor` 出的 refactor story 验收只说"现有测试**仍绿**"——隐含"现有测试已经够",未做覆盖审查
- `bob-identify --refactor` 入口完全不问测试覆盖

结果:
- 如果用户走 stories → identify → spec 完整链,spec Step 1 兜底,但用户**直到 spec 阶段**才知道要补测试
- 如果用户跳过 stories(B1 直接 identify→spec),仍有兜底但更晚
- 如果用户直接调 Superpowers 实施 refactor story 而绕过 spec,**完全无闸**

**且即使到 spec 阶段,"覆盖现行为"也不等于"全分支覆盖"**——一个 happy path 测试可以让人误以为"有覆盖",但实际错误分支、异常分支可能完全没测,重构后悄然失效。

### 1.2 改进目标

1. **三层联动**:stories / identify / spec 都加测试覆盖检查,任一入口都被覆盖
2. **全分支粒度**:不止"有没有测试",而是"每个分支是否有测试命中"
3. **soft 不阻断**:三段式提醒,用户可"否"继续,但警告留痕

### 1.3 关键约束

- **不改 `/bob-onion`**:架构设计阶段不参与测试检查
- **不引入新 skill**:仅扩展三个现有 skill 的 refactor 入口
- **不引入新 Rust 代码**:全部是 skill 模板内容修改 + token 测试
- **三段式 + TL 风**:继承现有规约
- **测试 token-only**:与 phase 0 / phase 1 一致

---

## 2. 三层联动表

| 用户路径 | 在哪激活安全网 | 时机 |
|---|---|---|
| survey → stories → identify --story → onion → spec | **stories Stage 2.5** | 早(拆分时就出 R0) |
| survey → identify --refactor(跳 stories,B1 直接) | **identify Step B1.0** | 中(soft 问) |
| 直接 `/bob-spec --refactor <类>` | **spec Template C Step 0** | 兜底(改名自现 Step 1,要求升级到全分支) |

无论哪条路径,在动代码前都至少有一道闸。

---

## 3. `bob-stories` Stage 2.5 · 测试覆盖体检

### 3.1 触发与位置

refactor 模式 / 混合模式(含 refactor 子表)专属。位置在 Stage 2(出 refactor 单元清单 R1/R2/R3)**之后**、Stage 3(排顺序)**之前**:

```
Stage 0. 输入归并
Stage 1. 自动识别模式
Stage 2. 三段式提拆法(出 refactor 单元清单)
Stage 2.5. 测试覆盖体检(全分支级)        ← 新增
Stage 3. 三段式拆顺序与依赖
Stage 4. 写汇总索引 + 每个 story 明细
```

feature 模式不触发本 Stage(没有重构对象就没有"现行为"要锁定)。

### 3.2 体检流程

对每个 refactor 单元 R_i 的每个被改方法 m:

**Step A. 枚举分支**

LLM 读 m 源码,列出所有分支:`if / else / switch case / throw / 早 return / 关键 && 短路`,编号 `B1, B2, ..., Bn`。

**Step B. 映射测试**

`grep -rn '<方法名>' src/test/java/ 2>/dev/null` 找出引用 m 的测试方法。读每个测试体,判定它覆盖了哪些分支(Bk1, Bk2 ...)。

**Step C. 列未覆盖分支 → 决定 R0.x**

| 测试状态 | R0.x 产物 |
|---|---|
| 方法 m **无任何测试** | `R0.x · 为 <类>.<m> 写全分支覆盖测试`(全部 B1..Bn) |
| 方法 m 有测试,**部分分支未覆盖** | `R0.x · 为 <类>.<m> 补未覆盖分支测试`(列出未覆盖 Bk) |
| 方法 m 全分支已覆盖 | ✓ 不出 R0 |

### 3.3 输出示例

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

### 3.4 三段式收敛

> **Q1.5: 接受这 N 个 R0.x stories?**
>
> 推测:体检发现 N 个方法、X 个未覆盖分支。建议 N 个 R0.x stories 全部接受,先写测试再重构。
> 理由:Michael Feathers《Legacy Code》第一条—没特征测试不要碰 legacy。全分支级是因为 happy path 测试容易让人误以为"覆盖了"。
> 推荐选择:`接受 N 个 R0.x stories`
>
> 是否同意?(回"是"走推荐;回"否,合并 R0.1+R0.2 进一个 story"重判;回"否,只要 R0.2/R0.3 不要 R0.1"切到手动)

用户可调:合并、丢弃、把多个方法的 R0 合到 1 个 story。

---

## 4. R0.x story 模板(characterize 类型)

复用 refactor 模板,**类型字段标 `characterize`**,验收换成分支级:

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

## 5. `bob-identify --refactor` · Step B1.0 测试覆盖现状

### 5.1 触发与位置

在现有 B1 工作流的 Step B1.1(代码扫描)**之前**插入 Step B1.0。

仅 `--refactor` 模式触发;G / B2 模式不触发。

### 5.2 三段式

LLM 跑 `find src/test -name "*.java"` 找出全部测试文件 → 对待重构的类逐一 grep + 读测试体 + 枚举分支 → 出推测:

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

### 5.3 用户应答处理

- "是" → 提前结束 identify,提示 `/bob-stories --refactor`
- "否,我先识别再说" → 继续 identify;**在 identity 文档 §8 段附加 "⚠ 测试覆盖警告:[列每个无 / 部分覆盖的方法 + 分支编号]"**——后续 onion / spec 能看到这个警告

不强制阻断。但警告留痕,后段无法回避。

---

## 6. `bob-spec --refactor` · Template C Step 0

### 6.1 改名与升级

把 Template C 现有 Step 1 改名为 Step 0,内容升级到全分支级 + 与 stories R0 互锁:

```markdown
### Step 0:测试覆盖现状(全分支级)

- 若 `docs/bob/02-stories-*.md` 索引里有 `R0.x · characterize · <本类>` 已完成 → 引用并跳过本步,前提:R0.x 的全分支盘点表(§2)覆盖了本 spec affected 的所有分支
- 否则,本步执行:
  - 枚举 affected method 的所有分支(if/else/switch case/throw/早 return)
  - 写测试覆盖每一个分支(无遗漏)
  - 跑测试 → 全绿(记录为基线)
- **禁止**这一步删任何代码
```

### 6.2 Step 编号顺延

旧 Template C 的 5 个 Step 全部 -1,顺延一位:

| 旧 | 新 |
|---|---|
| Step 1 加测试覆盖 | Step 0 测试覆盖现状(全分支级) |
| Step 2 抽端口 | Step 1 抽端口 |
| Step 3 状态机上提 | Step 2 状态机上提 |
| Step 4 框架边界外推 | Step 3 框架边界外推 |
| Step 5 删除 legacy | Step 4 删除 legacy |

测试场景里"场景 5:回归基线"沿用,不改编号(场景与 step 是独立编号系)。

### 6.3 实施要点

- spec 模板里 Step 0 段落明确写"全分支级"3 个字,避免被摘抄到别处时丢失语义
- spec 模板里加一行 explicit reference:"参见 bob-stories Stage 2.5 的全分支体检规则"

---

## 7. 测试

继续 phase 0 / phase 1 风——只测 token + 文件落位:

| 测试名 | 断言 token |
|---|---|
| `bob_stories_mentions_test_coverage_stage` | `Stage 2.5`、`测试覆盖体检`、`R0.`、`characterize`、`全分支覆盖`、`未覆盖分支` |
| `bob_identify_refactor_mentions_test_coverage_check` | `Step B1.0`、`测试覆盖现状`、`分支` |
| `bob_spec_template_c_mentions_step_0_with_stories_interlock` | `Step 0`、`全分支级`、`若 docs/bob/02-stories`、`characterize` |

更深的 fixture-based 行为验证(分支枚举准不准、R0 拆得对不对)继续延后,与 phase 0 / phase 1 一致。

---

## 8. 与 ARCHITECTURE.md 的关系

不动。本设计在 skill 模板层,不需要在 ARCHITECTURE.md 加新段。

---

## 9. 决策记录(用户答复留痕)

| 维度 | 决策 |
|---|---|
| 覆盖判定粒度 | **方法 + 全分支级**(每个 if/else/switch case/throw/早 return 各 1 个分支) |
| 无测试 → | 写全分支覆盖型 R0 |
| 有测试 → | 列已覆盖 + 未覆盖,补未覆盖型 R0 |
| 全覆盖 → | ✓ 不出 R0 |
| identify 入口门 | soft(三段式提醒 + 警告留痕)|
| R0.x story 类型 | characterize(复用 refactor 模板,改验收为全分支)|
| spec Template C Step 重命名 | Step 1→0,后续顺延 |
| 三层联动 | stories(早)→ identify(中)→ spec(兜底)|
| 测试 | token + 文件落位 |
| ARCHITECTURE.md 改动 | 无 |
| onion 改动 | 无 |

---

## 10. 实施草图(供 writing-plans 起步)

预计 3 个 task + 1 个可选:

1. **bob-stories**:加 Stage 2.5 测试覆盖体检 + R0.x characterize 类型 + 模板;更新 token 测试
2. **bob-identify**:加 Step B1.0 测试覆盖现状(三段式 + 警告留痕);更新 token 测试
3. **bob-spec**:Template C 改名 Step 1→Step 0、升级全分支级、加 stories R0 互锁;Step 编号顺延;更新 token 测试
4. (可选)README 在 `/bob-stories` 描述里提一句 "refactor 模式自动出 R0 特征测试 stories"

详细拆解由 `writing-plans` 出。

---

## 11. 转交

设计定稿后转交 `superpowers:writing-plans`,产出可执行实施计划。
