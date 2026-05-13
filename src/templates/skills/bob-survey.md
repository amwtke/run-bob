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
- `grep -c '^## 4\.' ARCHITECTURE.md` 与 §4 段下是否有非占位符内容 → β vs γ
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

| 评分 \ 难度 | Easy | Medium | Hard |
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

打开 `ARCHITECTURE.md`,找到 `## 12. 架构体检记录` 段:

- **若不存在**(老项目尚未跑过 survey),在文件末尾追加:

  ```markdown

  ## 12. 架构体检记录
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
