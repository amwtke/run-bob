---
name: bob-model
description: |
  触发条件:用户输入 /bob-model <doc-path>(主入口:把一份需求文档建模成结构化领域快照),
  或 /bob-model --story <story-path>(退路:已有 stories 时反向补建模),
  或 /bob-model --refresh(强制重写已有模型,即使源文档未变化)。

  在 /bob-survey 之后、/bob-stories 之前运行。读 PM 风格的散文需求文档
  (.md / .pdf / .docx / .txt),抽取出:1) 术语表,2) Entity 草图(属性 + 状态机种子 + 不变量),
  3) 业务规则清单(BR-NNN,跨 story 共享),4) UseCase 初步清单,5) 开放问题。
  同时产出 docs/bob/03-model-<slug>-<date>.md(下游 SSoT)和 .html(团队 PR review 用,Mermaid CDN)。

  适用于 Bob 4 环 Clean Architecture 工作流的领域建模 phase。结构上对称
  phase 2 (/bob-nfr) 和 phase 3 (/bob-compliance),都用 5 stage + 三段式。

  当用户说"建个模"、"做下领域建模"、"统一下术语"、"抽取业务规则"时也应触发此技能。
---

# Bob Model Skill

## 触发

```
/bob-model <doc-path>          # 主入口:对源需求文档建模
/bob-model --story <path>      # 退路:已有 stories 时反向建模
/bob-model --refresh           # 强制重写,即使源文档未变化
```

或自然语言触发:"建个模"、"做下领域建模"、"统一下术语"、"抽取业务规则"。

## 前置条件

- 项目位于 git 仓库内
- 建议:`/bob-survey` 已完成 + 难度评估 Medium/Hard
- 源需求文档可读(PDF/docx 需要 Read 工具的 `pages` 参数;md/txt 直读)

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

**翻译**散文需求文档为下游可消费的领域模型快照。只回答两个问题:

1. **这份需求里的术语 / Entity / 业务规则 / UseCase 各是什么?**
2. **PM 没说清楚、需要交给 `/bob-spec` 进一步消化的开放问题有哪些?**

**不写代码、不切 story、不画架构**。产出一份 md(SSoT)+ 一份 html(团队视图)。

## 工作流(5 个 Stage)

```
Stage 0. 入口体检 + 短路判定(可跳过建模)
Stage 1. 抽取(术语 / Entity / 规则 / UseCase / 开放问题)— 三段式追问填空
Stage 2. 写 md(SSoT,下游消费)
Stage 3. 写 html(视图,团队 PR review)
Stage 4. 三段式收口 + 通报下一步(/bob-stories 或 /bob-identify)
```

---

## Stage 0. 入口体检 + 短路判定

读取并通报:

| 状态 | 判定条件 | 后续行为 |
|---|---|---|
| **缺输入** | 无 `<doc-path>` 参数 + 无 `--story` + 找不到 survey 的源文档引用 | 拒绝运行,提示用户提供文档路径 |
| **极小需求** | 文档体量 < 50 行 + 概念 ≤ 3 + 无跨 story 共享规则 | 三段式询问"建议跳过建模,直接 `/bob-identify`";用户同意则写**一份占位 md** 留痕(空段 + 短路理由)|
| **常规** | 其他 | → Stage 1 |

向用户三段式通报:

> **Q0: 这份需求需要单独建模吗?**
>
> **推测**:常规 / 极小需求 / 缺输入
> **理由**:`<具体证据:行数 / 概念数 / 跨 story 共享规则数>`
> **推荐选择**:`继续建模` / `跳过(直接 /bob-identify)` / `补充输入`
>
> 是否同意?

输出文件名计算:
- `<slug>` = 源文档名去后缀(`ycb需求.md` → `ycb`,`阿里规约.pdf` → `阿里规约`)
- `<date>` = `YYYYMMDD`(UTC)
- 输出:`docs/bob/03-model-<slug>-<date>.md` + `docs/bob/03-model-<slug>-<date>.html`
- **同一天再跑** → 覆盖同名文件;**跨天** → 新文件,旧文件保留(团队可手动清理)

`--refresh` flag 显式触发 Stage 1,即使 Stage 0 判定为"极小需求"。

---

## Stage 1. 抽取(领域核心工作)

仅在 Stage 0 判定为 **常规** 或用户主动选择 `--refresh` 时执行。

### 1.1 按格式读源文档

| 后缀 | 读取方式 |
|---|---|
| `.pdf` | Read 工具的 `pages` 参数分批读(每次 5-10 页),逐批抽取 |
| `.docx` | 同 PDF,Read 工具直接处理(注:抽取质量取决于文档结构) |
| `.md`、`.txt`、`.markdown` | 一次性 Read 全文 |
| 其他 | 跳过,在 md/html 报告里标注"格式不支持" |

### 1.2 抽取 5 段内容

Claude 按下列顺序识别,**每段三段式追问填空**(不抛开放问题)。

#### A. 术语表(Glossary)

| 字段 | 说明 |
|---|---|
| 中文 | 例:订单 |
| 英文 | 例:Order |
| 定义 | 一句话 |
| 来源 | story / AC 编号 |
| 同义词 | 散文里出现的其他叫法,统一指向此条 |

#### B. Entity 草图

每个 Entity 一段:

- **属性清单** —— 名字 + 类型 + 必填(从 AC 推断)
- **状态机种子** —— 已出现的状态名 + 已知的迁移箭头;**未确认的状态用虚线**,标注"待 /bob-spec 确认"
- **不变量** —— 用 AC 措辞反推的 invariant,例:"一个 Order 只能包含一个商家的餐品"(YCB-001 AC#5)
- **生命周期事件**(可选) —— "创建时""提交时""完结时"等关键 hook

#### C. 业务规则清单(Business Rules)

| 规则 ID | 类型 | 公式/约束 | 来源 |
|---|---|---|---|
| BR-001 | 计算 | `Σ(item.price × item.qty) + 1 + 3` | YCB-001 价格计算 |
| BR-002 | 格式 | orderNumber = `yyyyMMddHHmmss + 6 位随机数` | YCB-001 AC#2 |
| BR-003 | 约束 | 一个 Order 只能一个商家 | YCB-001 AC#5 |

规则 ID 命名:`BR-<NNN>`,顺序递增,跨 story 共享。下游 spec 在 GWT 里直接 `// 验证 BR-001`,TDD 测试方法名可以是 `shouldCalculatePriceAccordingToBR001`。

#### D. UseCase 初步清单

| UseCase | 涉及 Entity | 涉及规则 | 来源 story | 备注 |
|---|---|---|---|---|
| CreateOrder | Order | BR-001 / BR-002 / BR-003 | YCB-001 + 1.1 + 1.3 | 命令型 |
| CalculateOrderPrice | Order | BR-001 | YCB-001 + 1.2 | 命令型 |
| ApplyDiscount | Order, DiscountScheme | (待定) | YCB-003 | 命令型 |

这只是**初步切**。`/bob-stories` 拿到后会按 4 因子复评再 1:1 切到 UseCase 粒度。

#### E. 开放问题清单

**显式列出**散文需求没说清楚的:

| 编号 | 问题 | 影响哪个下游 | 暂定假设 |
|---|---|---|---|
| Q1 | YCB-003 多种折扣是否可以叠加?顺序? | spec / TDD | 暂定不可叠加,以最高优惠为准 |
| Q2 | 除 PENDING_PAYMENT 外完整 Order 状态机? | onion / spec | 暂只建模 PENDING_PAYMENT;其他用虚线 |
| Q3 | 订单超时未支付是否自动取消? | onion / spec | 暂不实现 |
| Q4 | 配送地址校验?(手机号格式) | spec | 暂不校验 |

下游 `/bob-spec` 的"交给 Superpowers 的开放问题"段会进一步消化这些 Q。

### 1.3 三段式确认抽取结果

> **Q1: 抽取完成。N 条术语 / M 个 Entity / K 条业务规则 / U 个 UseCase / W 个开放问题。**
>
> **推测**:看起来覆盖完整 / 还有遗漏(列出我看到但未确认的概念)
> **理由**:基于源文档的 grep + AC 反推
> **推荐选择**:`确认无误,进入 Stage 2 写 md` / `补充遗漏后再写`
>
> 是否同意?
