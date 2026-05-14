# run-bob · `/bob-model` 技能设计(领域建模阶段)

> 状态:设计已定稿,待用户审阅 → 转交 `superpowers:writing-plans` 落地。
> 日期:2026-05-14
> 适用仓库:`/Users/xiaojin/workshop/run-bob`
> 参照需求文档(brainstorming 时拿来锚定的例子):`/Users/xiaojin/workshop/ycb需求.md`
> 相关 spec:
> - `docs/superpowers/specs/2026-05-14-bob-survey-design.md`(上游)
> - `docs/superpowers/specs/2026-05-15-bob-stories-design.md`(下游;model 之后才切 story)
> - `docs/superpowers/specs/2026-05-14-bob-nfr-design.md` / `2026-05-14-bob-compliance-design.md`(对称的"复盘"phase,本 spec 沿用同一形状)

---

## 0. 目的与一句话总结

把 PM 风格的散文需求文档(术语零散、规则混在 AC 里、隐含假设、跨 story 的共享业务规则没单独列出)**翻译**成一份下游 4 个技能都能直接消费的「领域模型快照」:

- 统一术语(下游 stories/identify/onion/spec 用同一套词)
- 沉淀业务规则(数学公式 / 约束 / 不变量,带规则 ID,可被 spec 引用、被 TDD 测试引用)
- 给出 Entity 草图(属性 + 状态机种子 + 不变量)
- 给出 UseCase 初步清单(指导 /bob-stories 的切片)
- **显式列出"需求文档没说清楚的开放问题"**,避免 Claude 在下游悄悄假设

**插入点:** `/bob-survey` 之后,`/bob-stories` 之前 —— 见 §3.1 工作流位置。

---

## 1. 背景

### 1.1 为什么需要这个阶段

run-bob 当前链路:`/bob-survey → /bob-stories → /bob-identify → /bob-onion → /bob-spec → TDD → /bob-compliance → /bob-nfr`。

这条链路的隐藏假设是「需求已经被表达成接近 UseCase 粒度的形式」。但实际拿到的 PM 需求文档通常是:
- 术语在散文里飘 —— "订单/单子/单据"可能指同一件事,也可能不是
- 业务规则混在 AC 里 —— 价格公式藏在 YCB-001 的 AC#4,后面 YCB-002/YCB-003 还会引用,但没有显式编号
- 隐含假设没说 —— "订单超时未支付怎么处理"、"折扣可以叠加吗"
- Entity / 状态机没识别 —— "PENDING_PAYMENT" 出现了,但完整状态机要看完所有 story 才能拼出来

下游技能在没有统一模型的情况下,每个 spec 自己重新发现一遍这些东西,术语漂移、规则重复、盲区分散 —— 这是个真问题。

### 1.2 设计 brainstorm 关键回合(为后人留参考)

我最初提议「stories 先于 model」(实施层 YAGNI 直觉),用户直觉反对,论据:
- 切 story 时还没有共同词汇 → 不同 story 用不同术语
- 切片边界应该沿 Entity 边界,没建模就不知道边界在哪
- 业务规则跨 story 共享,先建模一次比每个 story 各自发现一次便宜
- 领域建模(设计层)和实施(实施层)YAGNI 方向相反 —— 设计层是越早稳定下游摩擦越小

最终敲定 **`model → stories`**。

### 1.3 为什么不合并到现有技能

| 现有技能 | 它已经做的 | 为什么不能再扩 |
|---|---|---|
| `/bob-survey` | 架构体检 + 难度评分 + 3 档推荐 | TL **triage**,产出"go/yellow/red";扩成"建模"会模糊职责 |
| `/bob-stories` | 把大需求 1:1 切成 UseCase 级 story | 它是**切片器**;现在改成「先建模再切」恰好是把建模拎出来 |
| `/bob-identify` | 每个概念跑 5Q,分类 CORE/ADAPTER/FRAMEWORK/TOOL/违规 | 它是**分类器**,不是**词典构建器**;扩成建模会臃肿、且无法跨 story 沉淀共享词汇 |
| `/bob-onion` | 4 环架构 + 端口清单 + 状态机详设 | 它在**架构层**;model 在它之前的**领域层** |
| `/bob-spec` | per-UseCase GWT + Java stubs | 它是**实施层**;每个 spec 只看自己 |

### 1.4 设计哲学

- **md 是 SSoT** —— html 是 md 的视图,每次重写
- **html 入 git** —— 团队 PR 时可以浏览器直接看
- **Mermaid CDN 渲染** —— 单文件、自包含、无外部资产管线;断网时降级为代码块,不影响下游 Claude 离线消费
- **目录即配置** —— 没有 flag 开关,看输入文档体量自动决定要不要建模
- **不内置任何业务术语** —— 完全由 Claude 从用户提供的需求文档现场抽取

---

## 2. 输入与输出

### 2.1 输入

| 来源 | 优先级 | 说明 |
|---|---|---|
| `/bob-model <doc-path>` 参数 | 主入口 | 任意路径,任意格式(.md / .pdf / .docx / .txt) |
| `/bob-model --story <story-path>` | 退路 | 已有 stories 时,以 story 为锚反向建模(用于补建模) |
| 自动扫描 | fallback | 无参数时,扫描 `docs/bob/00-survey-*.md` 提到的源文档路径 |

### 2.2 输出

**总是同时生成两份**,同源:

| 文件 | 角色 | git 入库 |
|---|---|---|
| `docs/bob/03-model-<slug>-<date>.md` | **SSoT** — 下游技能解析的真相源 | ✅ |
| `docs/bob/03-model-<slug>-<date>.html` | **视图** — 团队 PR review 时浏览器打开 | ✅ 同入,每次重写 |

`<slug>` 取自需求文档名(`ycb需求.md` → `ycb`),`<date>` 是 YYYYMMDD。

### 2.3 编号槽位说明

`docs/bob/` 当前已用 `00-/01-/02-/04-/05-` —— `03-` 是空的,直接占用。**编号≠时间序**(stories `02-` 在 identify `01-` 之后跑是早有先例),只表示 phase 槽位。

---

## 3. 工作流

### 3.1 流程图与位置

```
要求文档 (ycb需求.md)
   ↓
/bob-survey       →  00-survey-*.md      (TL 体检 + 难度推荐)
   ↓
/bob-model        →  03-model-*.md(+.html)   ← 本 spec
   ↓
/bob-stories      →  02-stories-*.md     (基于 model 的词汇切片)
   ↓
/bob-identify     →  01-identity-*.md    (5Q,每个 story 一份)
   ↓
/bob-onion        →  ARCHITECTURE.md     (4 环,Entity 状态机细化)
   ↓
/bob-spec         →  spec-*.md           (per-UseCase GWT;业务规则引用 BR-NNN)
   ↓
Superpowers TDD   →  /bob-compliance     →  /bob-nfr
```

### 3.2 五个 Stage(对齐 /bob-nfr、/bob-compliance 的形状)

```
Stage 0. 入口体检 + 短路判定(可跳过建模)
Stage 1. 读源文档 + 抽取术语 / 实体 / 规则 / UseCase / 开放问题(三段式追问填空)
Stage 2. 沉淀到结构化 md(SSoT)
Stage 3. 生成 html 视图(Mermaid CDN + 内联 CSS)
Stage 4. 三段式收口 + 通报下一步(`/bob-stories` 或直接 `/bob-identify`)
```

### 3.3 Stage 0:入口体检 + 短路判定

读取并通报:

| 状态 | 判定条件 | 后续行为 |
|---|---|---|
| **缺输入** | 无 `<doc-path>` 参数 + 无 `--story` + 找不到 survey 的源文档引用 | 拒绝运行,提示用户提供文档路径 |
| **极小需求** | 文档体量 < 50 行 + 概念 ≤ 3 + 无跨 story 共享规则 | 三段式询问"建议跳过建模,直接 `/bob-identify`";用户同意则写**一份占位 md** 留痕(空段 + 短路理由) |
| **常规** | 其他 | → Stage 1 |

> **Q0: 这份需求需要单独建模吗?**
>
> **推测**:常规 / 极小 / 缺输入
> **理由**:`<具体证据:行数 / 概念数 / 跨 story 共享规则数>`
> **推荐选择**:`继续建模` / `跳过` / `补充输入`
>
> 是否同意?

### 3.4 Stage 1:抽取(领域核心工作)

Claude 按 5 段顺序识别,**每段都三段式追问填空**(不抛开放问题):

#### 1.1 术语表(Glossary)

| 字段 | 说明 |
|---|---|
| 术语(中文 / 英文) | 例:订单 / Order |
| 定义 | 一句话 |
| 来源 | story / AC 编号 |
| 同义词 | 散文里出现的其他叫法,统一指向此条 |

#### 1.2 Entity 草图

每个 Entity 一段:

- **属性清单** —— 名字 + 类型 + 是否必填(从 AC 推断)
- **状态机种子** —— 已出现的状态名 + 已知的迁移箭头;**未确认的状态用虚线**,标注"待 /bob-spec 确认"
- **不变量** —— 用 AC 措辞反推的 invariant,例:"一个 Order 只能包含一个商家的餐品"(YCB-001 AC#5)
- **生命周期事件**(可选) —— "创建时""提交时""完结时"等关键 hook

#### 1.3 业务规则清单(Business Rules)

每条规则一行:

| 规则 ID | 类型 | 公式/约束 | 来源 |
|---|---|---|---|
| BR-001 | 计算 | `Σ(item.price × item.qty) + 1 + 3` | YCB-001 价格计算 |
| BR-002 | 格式 | orderNumber = `yyyyMMddHHmmss + 6 位随机数` | YCB-001 AC#2 |
| BR-003 | 约束 | 一个 Order 只能一个商家 | YCB-001 AC#5 |

规则 ID 命名:`BR-<NNN>`,顺序递增,跨 story 共享。下游 spec 在 GWT 里直接 `// 验证 BR-001`,TDD 测试方法名可以是 `shouldCalculatePriceAccordingToBR001`。

#### 1.4 UseCase 初步清单

| UseCase | 涉及 Entity | 涉及规则 | 来源 story | 备注 |
|---|---|---|---|---|
| CreateOrder | Order | BR-001 / BR-002 / BR-003 | YCB-001 + 1.1 + 1.3 | 命令型 |
| CalculateOrderPrice | Order | BR-001 | YCB-001 + 1.2 | 命令型(组合在 CreateOrder 内) |
| ApplyDiscount | Order, DiscountScheme | BR-002(暂占位)| YCB-003 | 命令型 |

这只是**初步切**,/bob-stories 拿到后会按 4 因子复评再 1:1 切到 UseCase 粒度。

#### 1.5 开放问题清单

**显式列出**散文需求没说清楚的:

| 编号 | 问题 | 影响哪个下游 | 暂定假设(允许带着假设走下游)|
|---|---|---|---|
| Q1 | YCB-003 多种折扣是否可以叠加?顺序? | spec / TDD | 暂定不可叠加,以单一最高优惠为准 |
| Q2 | 除 PENDING_PAYMENT 外完整 Order 状态机? | onion / spec | 暂只建模 PENDING_PAYMENT;其他状态用虚线 |
| Q3 | 订单超时未支付是否自动取消? | onion / spec | 暂不实现 |
| Q4 | 配送地址需要校验吗?(手机号格式) | spec | 暂不校验 |

下游 `/bob-spec` 的"交给 Superpowers 的开放问题"段会进一步消化这些 Q。

### 3.5 Stage 2:写 md(SSoT)

`docs/bob/03-model-<slug>-<date>.md` 固定 schema:

```markdown
---
name: bob-model
source_doc: /Users/.../ycb需求.md
source_doc_sha256: a1b2c3...
generated_at: 2026-05-14T03:21:00Z
target_phase: pre-stories
---

# 领域模型 · YCB 订餐 · 2026-05-14

## 0. 元信息
- 源文档:`ycb需求.md`(sha256 `a1b2c3...`)
- 生成时间:`2026-05-14T03:21:00Z`
- 后续步骤:`/bob-stories ycb需求.md`(基于本模型切片)

## 1. 术语表
| 中文 | 英文 | 定义 | 来源 | 同义词 |
|---|---|---|---|---|
| 订单 | Order | 用户提交的餐品购买订单 | YCB-001 | 单子/单据 |
...

## 2. Entity 草图
### 2.1 Order
**属性**
- orderId: String (必填,系统生成)
- orderNumber: String (必填,系统生成,见 BR-002)
- userId: String (必填,登录态注入)
- ...

**状态机种子**(Mermaid `stateDiagram-v2`)
- 已出现:PENDING_PAYMENT
- 待确认:PAID / DELIVERED / CANCELLED / TIMEOUT_CANCELLED(虚线)

**不变量**
- INV-Order-1: 单个 Order 仅含一个商家的餐品(YCB-001 AC#5)
- INV-Order-2: orderNumber 格式 `yyyyMMddHHmmss + 6 位随机数`(BR-002)

...

## 3. 业务规则
### BR-001 价格计算
**公式**: `Σ(item.price × item.qty) + 打包费(1) + 配送费(3)`
**精度**: 金额字段保留 2 位小数(YCB-001 1.3 AC#5)
**来源**: YCB-001 1.2 全部 AC

### BR-002 orderNumber 格式
**格式**: `yyyyMMddHHmmss + 6 位随机数`(20 个字符)
**来源**: YCB-001 AC#2

...

## 4. UseCase 初步清单
| UseCase | 涉及 Entity | 涉及规则 | 来源 story |
...

## 5. 开放问题
| 编号 | 问题 | 影响哪个下游 | 暂定假设 |
...
```

### 3.6 Stage 3:写 html(视图)

`docs/bob/03-model-<slug>-<date>.html`:**单文件,自包含**(除 Mermaid CDN 外无任何外部依赖),由 Claude 直接产出整段 html。

#### 3.6.1 页面结构

```
┌────────────────────────────────────────────┐
│  顶部 metadata 卡片                          │
│  (源文档 / 日期 / Claude 版本)              │
├──────────┬─────────────────────────────────┤
│ 粘性 TOC │ § 1. 术语表       (HTML 表)      │
│          │ § 2. Entity 草图  (Mermaid)      │
│  - 术语  │   2.1 Order       classDiagram   │
│  - Entity│        + stateDiagram-v2         │
│  - 规则  │ § 3. 业务规则     (卡片 + 表)    │
│  - UC    │ § 4. UseCase      (Mermaid)      │
│  - Q     │ § 5. 开放问题     (checklist)    │
└──────────┴─────────────────────────────────┘
```

#### 3.6.2 Mermaid 图清单

| 段 | Mermaid 类型 | 内容 |
|---|---|---|
| 2.x Entity | `classDiagram` | 一个 class 块,列属性,加 注释行 `<<entity>>` |
| 2.x 状态机 | `stateDiagram-v2` | 已出现状态实线,待确认虚线(`-->` vs `..>`) |
| 4. UseCase 关系 | `flowchart LR` | UseCase 节点 + 依赖箭头 |

#### 3.6.3 资源策略

- **CDN 一行**:`<script src="https://cdn.jsdelivr.net/npm/mermaid@10/dist/mermaid.min.js"></script>`
- **离线降级**:Mermaid 没加载到,代码块原样显示(浏览器看见的是 `<pre><code class="language-mermaid">...</code></pre>`),工程师仍能读 DSL
- **CSS 内联**:`<style>` 标签,~200 行,实现:
  - 左侧粘性 TOC(`position: sticky; top: 0`)
  - 主区最大宽度 `960px` 居中
  - 代码 / 表 / 标题字号、间距、配色(参考 Stripe / Tailwind 文档站的简洁风)
  - 暗黑模式:**不做**(YAGNI)

#### 3.6.4 html 体积控制

- 不内联 Mermaid 库(节省 ~1MB,代价是断网时降级)
- 不内联其他字体 / 图标(使用系统 sans-serif)
- 预期单文件 30-80 KB,git diff 友好

### 3.7 Stage 4:收口 + 下一步

三段式通报建模结果:

> **Q3: 建模完成。N 条术语 / M 个 Entity / K 条业务规则 / W 个开放问题。**
>
> **推测**:难度 `<Medium/Hard>` → 建议下一步 `/bob-stories`(基于本模型切片);如难度 `<Easy>`,可直接 `/bob-identify`
> **理由**:从 /bob-survey 推荐 + 本模型规模综合判断
> **推荐选择**:`继续 /bob-stories` / `直接 /bob-identify` / `打开 html 团队 review 后再决定`
>
> 是否同意?(回"是"按推荐;回"打开 html"提示文件路径 + 暂停)

---

## 4. CLAUDE.md 改动

无需新 R 规则。Model 是**翻译**,不是**强制约束**;下游消费它的约束已经分布在 R7-R13 里(命名 / 包结构 / 合规等)。

但需要在工作流总览图(`## 工作流总览`)里加一行:

```
+ /bob-model       → docs/bob/03-model-*.md (+ .html)
```

---

## 5. bob-survey 的"下一步"段改动

对齐 `/bob-stories`、`/bob-nfr`、`/bob-compliance` 已有的 soft-prompt 模式。`/bob-survey` 的"下一步"段加一句:

> 4. (强烈推荐)有源需求文档 + 难度 Medium/Hard → 先跑 `/bob-model <doc-path>` 把领域模型抽出来,再走 `/bob-stories`

(`/bob-survey` 已有 4 步,这条插在末尾;或按现有编号微调,保持「先 model 再 stories」次序。)

---

## 6. run-bob 二进制改动

| 文件 | 动作 | 内容 |
|---|---|---|
| `src/templates/skills/bob-model.md` | **新建** | 新技能,~450 行(对标 bob-nfr / bob-compliance) |
| `src/templates/root/CLAUDE.md` | **改** | 工作流总览图加一行 `/bob-model` |
| `src/templates/skills/bob-survey.md` | **改** | "下一步"加 model 提示 |
| `src/assets.rs::HARNESS_ASSETS` | **改** | 新增 1 项 `bob-model/SKILL.md`,Skill / `upgrade_safe=true` / `included_in_minimal=true` |
| `tests/integration.rs` | 加 ~4 测试 | 详见 §7 |

**不改动:**
- 现有任何 commands/ 代码
- 现有任何 docs/compliance/ / .gitignore 逻辑
- `src/templates/root/compliance-README.md`、其他模板

---

## 7. 测试

### 7.1 模板存在 + 内容测试

1. **init_creates_bob_model_skill** —— `.claude/skills/bob-model/SKILL.md` 存在,关键 token:`/bob-model`、`Stage 0`-`Stage 4`、`docs/bob/03-model-`、`Mermaid`、`classDiagram`、`stateDiagram-v2`、`术语表`、`Entity 草图`、`业务规则`、`BR-`、`UseCase 初步清单`、`开放问题`
2. **bob_model_skill_mentions_dual_output** —— skill 同时提到 `.md` 和 `.html` 两个输出文件
3. **bob_model_skill_explains_cdn_strategy** —— skill 提到"CDN"和"降级"(确保未来不会被改成内联)

### 7.2 上游 hook

4. **bob_survey_mentions_model_soft_prompt** —— bob-survey 的"下一步"段提到 `/bob-model`

### 7.3 工作流图

5. **claude_md_workflow_lists_bob_model** —— CLAUDE.md 的工作流总览图含 `/bob-model`

### 7.4 minimal 模式

minimal 模式自动覆盖(asset `included_in_minimal=true`,与其他 skill 一致);不需要专门测试。

### 7.5 不需要的测试

- 不测 html 内容(html 是 Claude runtime 产出,不在 init/upgrade 资产范畴)
- 不测 md 内容(同上)
- 不测 Mermaid 是否能渲染(浏览器层,超出 run-bob 边界)

---

## 8. 不变量

- **md 是 SSoT** —— html 仅是视图,每次 `/bob-model` 运行重写
- **html 入 git** —— 团队 PR review 用,差异可见
- **Mermaid via CDN** —— 单 `<script>` 标签;离线时优雅降级为代码块
- **`docs/bob/03-` 槽位独占** —— model 用这个槽,其他 skill 不占
- **不内置业务术语** —— 完全 runtime 抽取,无 schema fixture
- **不引入新 R 规则** —— Model 是翻译层,不构造硬约束

---

## 9. 未来扩展(YAGNI,不实现)

- 增量建模:`/bob-model --diff <old>` 只比对差异
- 多需求文档合并:`/bob-model <doc1> <doc2>`
- 暗黑模式 html
- 输出 PNG/SVG(从 Mermaid 静态化,需要 puppeteer 等工具链 → 太重)
- 输出 PlantUML / draw.io 格式

---

## 10. 验收清单

- [ ] `cargo test` 全绿,新增 5 测试通过
- [ ] `run-bob init /tmp/test` 后,`.claude/skills/bob-model/SKILL.md` 存在
- [ ] 在 Claude Code 里跑 `/bob-model /Users/xiaojin/workshop/ycb需求.md`
- [ ] 输出 `docs/bob/03-model-ycb-<日期>.md` —— 包含 5 段固定结构、术语 ≥ 4 条、Entity ≥ 2 个、BR ≥ 3 条、开放问题 ≥ 3 个
- [ ] 输出 `docs/bob/03-model-ycb-<日期>.html` —— 浏览器打开:
  - 顶部 metadata 卡可见
  - 左侧粘性 TOC 滚动跟随
  - Entity classDiagram 渲染成方框
  - Order 状态机渲染 PENDING_PAYMENT + 虚线箭头
  - 业务规则表 / 开放问题列表样式正常
- [ ] 浏览器断网:html 仍可打开,Mermaid 区域降级为代码块
- [ ] bob-survey 的"下一步"段含 `/bob-model` 提示
- [ ] CLAUDE.md 工作流总览图含 `/bob-model`
- [ ] 再跑一次 `/bob-model` 同源文档 → md / html 都被原文件名覆盖(不堆积新 timestamp 文件,因为日期未变;若跨天则新文件)
