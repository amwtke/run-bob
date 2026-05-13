# bob-survey · 架构体检 + 需求难度评估 skill 设计

> 状态:设计已定稿,待用户审阅 → 转交 `superpowers:writing-plans` 落地。
> 日期:2026-05-14
> 适用仓库:`/Users/xiaojin/workshop/run-bob`
> 相关:
> - 现有 bob skills:`bob-identify` / `bob-onion` / `bob-spec`
> - 现有本地 skill:`/archcheck`(deeparch-archcheck 的本地版,5 维度架构体检)
> - run-bob v0.1.0 设计:`docs/superpowers/specs/2026-05-08-run-bob-design.md`
> - run-bob upgrade 设计:`docs/superpowers/specs/2026-05-13-run-bob-upgrade-design.md`

---

## 0. 目的与一句话总结

新增 `/bob-survey <需求>` skill,在 `/bob-identify` 之前做一道"TL 接需求"动作:对项目现状做架构体检(0-100 分)、对新需求做难度判定(Easy/Medium/Hard)、给 3 档落地建议(🟢/🟡/🔴)。产出 `docs/bob/00-survey-*.md` + `ARCHITECTURE.md §9 体检记录`。`/bob-identify` 启动时检测并提示是否补跑(soft,不强制)。

> 让 bob 工作流的第一步从"直接做身份测试"变成"TL 先看一眼底子能不能接这个需求,再决定怎么开始"。

---

## 1. 背景与现状

### 1.1 现状

bob 工作流当前是一条直线:

```
需求 → /bob-identify → /bob-onion → /bob-spec → Superpowers TDD
```

缺一个**接需求的判断步**:

- 不知道这个仓库当前底子是 G / β / γ 哪一档
- 不知道这个需求落到这个底子上是 Easy / Medium / Hard
- 没人提醒"这底子接这需求会出事,先重构"

`/archcheck` 这个本地 skill 已经做"架构 5 维度体检",但定位是通用 Clean Architecture 健康检查,不与 bob 的 G/B1/B2 模式、`@Transactional` 唯一性、`FORBIDDEN_IN_INNER` 等 Bob 特定约束绑死。

### 1.2 关键约束

- **不引入新 skill 链依赖**:bob-survey 自成体系,不主动调 `/archcheck`(可选读取 `archcheck-report-*.md`)
- **不修改 bob-onion / bob-spec**:本次只新增 bob-survey + 微调 bob-identify(softly 检测 survey)
- **必须带需求跑**:无需求 → 报错退出。"光给架构打分"是另一个 use case,不在本设计内。
- **三段式提问 + TL 对话风继承现有 bob skills**:推测 / 理由 / 推荐选择,用户可"否"。
- **`superpowers-to-trae` 仅作 CLI 风格参考**:不导入任何代码/模板。

### 1.3 不在本设计内(后续阶段)

用户已锁定**分阶段**:本设计**只覆盖 phase 0**(survey)。后续两个阶段(后续 spec 另开):

- phase 1:`/bob-stories` —— 把大需求拆成 AI 可消化粒度的故事
- phase 2:`/bob-nfr` —— NFR(安全/性能/秒杀/分布式/微服务)沟通与记录

---

## 2. CLI 表面

```
/bob-survey <需求一句话或几段>          # 主入口
/bob-survey --archcheck <path>          # 显式指定要消化的 archcheck 报告
/bob-survey --no-record                 # 跑完不写 ARCHITECTURE.md §9
/bob-survey --refresh                   # 已有 docs/bob/00-survey-*.md 时强制重跑
```

自然语言触发(写进 skill 的 `description` frontmatter):
- "接需求前先体检"
- "现在能不能接这个需求"
- "这个需求要不要先重构"
- "看一下我现在的底子"

**无需求传入 → 拒绝**,提示"必须带需求"(详见 §4 难度判定:难度只能从需求语义出发)。

---

## 3. 三态识别(repo state 判定)

| 状态 | 判定条件 | 走什么子流程 |
|---|---|---|
| **G(绿地)** | 项目里没有 `src/main/java`(或目录为空) | 跳过 6 维度打分,直接评估需求复杂度,推荐 G 模式走 `/bob-identify` |
| **β(棕地未跑过 bob)** | 有 `src/main/java`,但 `ARCHITECTURE.md` 不存在 / 文件中 §4-§7 全为占位符 | 跑全部 6 维度打分(底子是 α/β,大概率低分) |
| **γ(成熟 bob)** | 有 `src/main/java` + `ARCHITECTURE.md` §4-§7 填了 + `.claude/skills/bob-*` 存在 | 跑全部 6 维度打分(预期高分;低分意味着代码漂移了) |

判定方式:checker 看几个 sentinels(目录存在性 + ARCHITECTURE.md 字符串匹配),无 LLM judgment。

---

## 4. 评分:6 维度 × 0-20 = 总分 100

### 4.1 6 维度

| # | 维度 | 评分规则 | 静态/LLM |
|---|---|---|---|
| 1 | **Entity 纯度** | `grep -r 'org.springframework\|jakarta.persistence\|lombok\|org.slf4j' src/main/java/com/example/*/entity/` → 0 违规 = 20;每出现 1 个 file 扣 4(下限 0) | 静态 |
| 2 | **UseCase 纯度** | 同上,作用域换成 `*/usecase/` (排除 `usecase/port/`) | 静态 |
| 3 | **端口位置** | 列出所有 `*Repository` / `*Port` / `*Gateway` 接口,看它们在 `usecase/port/` 还是 `adapter/` → usecase/port 占比 ≥ 80% = 20;线性扣分 | LLM 辅助(找接口) + 静态(算占比) |
| 4 | **状态机位置** | LLM 抽 3-5 个最关键 Entity 方法签名,若 `entity` 包内的状态修改方法 / status guard 多于 `service` 包 = 20;反之扣分 | LLM |
| 5 | **@Transactional 唯一性** | `grep -rn '@Transactional' src/main/java/` → 仅在 `shared/framework/transaction/` 包 = 20;每多 1 个文件 扣 5 | 静态 |
| 6 | **FORBIDDEN_IN_INNER 违规** | 读 `CleanArchitectureTest.java` 的 `FORBIDDEN_IN_INNER` 数组,跑一次 archunit-equivalent 静态扫描 → 0 违规 = 20;每 5 个 file 扣 1 | 静态 |

### 4.2 证据片段(每维度必带)

每项给一个 ≤ 3 行的**具体证据片段**(file:line + 一句简评),让用户立刻看清扣在哪了。**禁止只给分数不给证据**。

> 比喻:这是 TL 拿着 IDE 在你机器上跑 `Cmd+Shift+F` 检查 R7-R10 的过程,不是写论文。

### 4.3 与 archcheck 报告的关系(soft)

若项目里存在 `archcheck-report-*.md`(或用户 `--archcheck` 指定路径),survey 读取它的结论作为**第 7 个参考维度**,但**不计入总分**(只在 §6 报告里展开为附录,标"参考 archcheck 报告: ..."一行)。

---

## 5. 需求难度判定(Easy / Medium / Hard)

不靠 LLM "猜",靠**三因子量表**:

| 因子 | Easy | Medium | Hard |
|---|---|---|---|
| **跨环数** | 单一 UseCase、单 Entity | 多个 UseCase、需要新端口 | 跨 BC、新状态机、需要新 Adapter family |
| **状态机增量** | 0 个新状态 / 0 个新转移 | 1-2 个新状态 / 新转移 | 多状态机相互依赖、需要 saga / 分布式事务 |
| **legacy 复用** | 不依赖 legacy | 依赖 1-2 个 legacy `@Service`(可走 ACL) | 依赖 ≥ 3 个 legacy + legacy 内部还要改 |

### 5.1 组合规则

- 任一因子触发 Hard → 总评 **Hard**
- 否则,若有 ≥ 2 个 Medium → 总评 **Medium**
- 否则 → 总评 **Easy**

### 5.2 提问方式

LLM 三段式追问用户(推测/理由/推荐选择),不像评分那样静态扫描——因为难度是从需求语义出发的,机器算不准。每个因子追一两个问题,得出 Easy/Medium/Hard。

---

## 6. 推荐矩阵(3 × 3 + 绿地)

### 6.1 绿地特殊

跳过架构维度,只看需求复杂度 → "走 G 模式 `/bob-identify`"。难度判定仍走 §5,但推荐固定:

- Easy / Medium / Hard 均 → 🟢 `/bob-identify <需求>`(G 模式)
- 若难度 Hard,附加提示:"绿地下接 Hard 需求建议拆 story(下阶段 phase 1)"

### 6.2 棕地(β/γ)矩阵

| 评分 \ 难度 | Easy | Medium | Hard |
|---|---|---|---|
| **80-100(γ 健康)** | 🟢 直接 `/bob-identify` | 🟢 直接 `/bob-identify`(B2 模式)| 🟡 推荐 B2 清洁孤岛;或先 `/bob-onion --refresh` 增端口 |
| **60-79(β 可接受)** | 🟢 `/bob-identify`(B2 模式)| 🟡 B2 清洁孤岛 + 提前列 ACL 表 | 🔴 先 `/bob-onion --refactor` 出三动作改造计划,选定 3-5 个文件改完再回来 |
| **0-59(α 烂底子)** | 🟡 警告:能做但债会变重;建议 B2 + 隔离严格 | 🔴 先重构再接 | 🔴 拒绝接需求,先 B1 全量重构;给"必须先改完哪 5 个东西"的清单 |

### 6.3 每个格子的展开

每条推荐在 §6 产出报告里展开 3 行:

- **推荐的下一步命令**:具体的 `/bob-...` 调用
- **一句话理由**:为什么是这个推荐
- **风险提示**:如选择无视推荐继续硬接的后果(用 TL 口气)

---

## 7. 产出与 §9 体检记录

### 7.1 主产出 `docs/bob/00-survey-<slug>-<YYYYMMDD>.md`

文件结构(模板):

```markdown
# 架构体检 · <需求一行话>
日期 · 2026-05-13 · 状态 · γ · 总分 76/100 · 需求难度 · Medium · 推荐 · 🟡 B2 + ACL

## 1. 仓库状态
γ(成熟 bob)· src/main/java 存在 · ARCHITECTURE.md §4-§7 已填 · 距上次 onion 18 天

## 2. 评分明细
| 维度 | 分 | 证据 |
| Entity 纯度 | 20 | (无违规) |
| UseCase 纯度 | 12 | OrderApprovalUseCase.java:8 仍 import slf4j |
| 端口位置 | 16 | 12/14 接口在 usecase/port,FxRateGateway.java 在 adapter |
| 状态机位置 | 15 | Order.confirm 在 entity ✓;Order.cancel 还在 OrderService.java:42 |
| @Transactional 唯一 | 20 | 仅 TransactionalUseCaseDecorator.java |
| FORBIDDEN 违规 | -7 → 13 | 35 处 slf4j 在 usecase(可接受底线) |

## 3. 需求难度三因子
跨环数 · Medium(2 UseCase + 1 新 Port)
状态机增量 · Easy(0 新状态)
legacy 复用 · Medium(依赖 LegacyPricingService)
→ 总评 **Medium**

## 4. 推荐
🟡 **B2 清洁孤岛 + 提前列 ACL 表**
理由:γ 底子 + Medium 需求,跑硬可以,但 legacy 复用会让新 usecase 与 LegacyPricingService 耦合。先在 docs/bob/02-acl-*.md 列出 LegacyPricingService 的 ACL 接口契约,再 /bob-identify。
风险:若忽略本建议直接接,3 个月内会出现"新 usecase 改不动 because legacy 改了"。

## 5. 下一步
推荐命令:`/bob-identify <需求> --acl LegacyPricingService`
```

### 7.2 ARCHITECTURE.md §9 体检记录

每次跑 survey 在 `ARCHITECTURE.md` 末尾(或固定 §9 段)追加一行:

```markdown
## 9. 架构体检记录
| 日期 | 状态 | 总分 | 需求 | 难度 | 推荐 | 详报 |
|---|---|---|---|---|---|---|
| 2026-05-13 | γ | 76 | 订单分润 | Medium | 🟡 B2+ACL | docs/bob/00-survey-润分配-20260513.md |
```

- 若 ARCHITECTURE.md 没有 §9 段,survey 在最后追加这个 section
- 若有 §9 段,在表格末尾追加一行
- `--no-record` 时跳过此步

`/bob-onion` 看到此表能感知历史,可在 onion 阶段引用最近一次体检结论(下个迭代再做集成,本设计不改 onion)。

---

## 8. 与现有 skill 的集成(soft)

### 8.1 bob-identify 的微调

`bob-identify` SKILL.md 触发段加一段:

> 若 `docs/bob/00-survey-*.md` 不存在 或最新一份距今 > 7 天,先三段式追问:
> > 是否已跑过 `/bob-survey`?
> > 推测:看起来没有,建议先跑一遍以确认当前底子能接这个需求。
> > 理由:跳过体检直接接需求,γ 底子下问题不大,β/α 底子下大概率会拖垮新代码质量。
> > 推荐选择:`先跑 /bob-survey`
> > 是否同意?(回"是"→跳出 identify,执行 survey;回"否"→继续 identify)

**不强制**。用户回"否"则照旧走。这保持 TL 风格——TL 也会问"你有没有评估过"而不是"必须先评估"。

### 8.2 bob-onion / bob-spec

不修改。后续迭代再决定是否引用 §9 历史。

---

## 9. 测试

**以后再补**。

第一版只做最小验证:确保 `run-bob init` 装 `/bob-survey` skill 后,Claude Code 能识别它,产出文件落在正确目录。具体集成测/fixture 验证在后续 patch 单独设计。

---

## 10. 决策记录(用户答复留痕)

| 维度 | 决策 |
|---|---|
| 阶段范围 | 仅 phase 0(survey)。phase 1(stories)、phase 2(NFR)后续 spec 另开 |
| skill 输入 | 必须带需求 |
| 与 /archcheck 关系 | 自成体系;若项目有 `archcheck-report-*.md` 则读取作参考维度,不计入总分 |
| 评分架构 | 6 个 Bob 独有维度 × 0-20 = 100 |
| 推荐机制 | 量表 × 三档(🟢/🟡/🔴) |
| 产出位置 | docs/bob/00-survey-*.md + ARCHITECTURE.md §9 |
| bob-identify 集成 | soft 提示,不强制 |
| 三段式 + TL 风 | 继承 |
| 测试 | 第一版从简,以后再补 |

---

## 11. 转交

设计定稿后转交 `superpowers:writing-plans`,产出可执行的实施计划。

实施工作大致包含:

1. **新增模板** `src/templates/skills/bob-survey.md`(完整 skill 内容)
2. **`HARNESS_ASSETS` 加一条** + `upgrade_safe: true`(走 R5 既有路径,upgrade 会自动分发)
3. **微调** `src/templates/skills/bob-identify.md` 顶部加 §8.1 描述的 soft 检测段
4. **微调** `src/templates/root/ARCHITECTURE.md` 末尾加 §9 体检记录空表头(供 survey 追加)
5. **README + README-RUN-BOB**(可选)提一笔 phase 0
6. **集成测一条最小骨架**(验证 survey skill 装到 .claude/skills/bob-survey/SKILL.md,内容包含关键 token)
