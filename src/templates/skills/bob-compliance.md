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
| **空仓** | `sources/` 空 **且** scope 内无 spec 含 `## 9.5 涉及设计模式` | 软退出:"无合规要求,跳过"。**写一份空报告留痕** |
| **空仓但有模式** | `sources/` 空,但 scope 内 ≥1 个 spec 含 `## 9.5 涉及设计模式` | 跳过 sources 规则,**仍进 Stage 3 只跑模式符合度** |
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

固定 schema(以阿里 PDF 为例),包括 YAML frontmatter + 目录 + 按维度分节的规则 ID:

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

### 第二类规则源:spec 声明的设计模式

除 `docs/compliance/*.md` 外,**额外 grep `docs/specs/spec-*.md` 的 `## 9.5 涉及设计模式` 段**,把每行解析成一条模式规则并入同一索引:

```
PAT-N-k → (spec文件, 模式名, Context类, 参与角色, 可观察痕迹, severity)
```

`PAT-*` 与 `ALI-*` 走**同一套**校验 / 分类 / 报告管线。无任何 spec 含 §9.5 时,这一类为空,后续模式符合度静默跳过(零噪音)。

**Severity 优先级:【强制】> 【推荐】> 【参考】**。三档**都参与**校验,但分类报告时:

- 【强制】违反 → 视为必须修复
- 【推荐】违反 → 列出但允许豁免
- 【参考】违反 → 列出仅作提示

向用户通报装载结果:

> **Q1: 装载到 N 条规则,来自 M 份标准。**
>
> **推测**:【强制】X 条 / 【推荐】Y 条 / 【参考】Z 条;**+ J 条模式规则(PAT-*),来自 K 份 spec**
> **理由**:从已生成的 `docs/compliance/*.md` 索引得出
> **推荐选择**:`进入 Stage 3 跑校验`
>
> 是否同意?

---

## Stage 3. Diff 校验 (模式 B 核心)

### 3.1 定位检查范围 (优先级从高到低)

1. **显式参数**:`/bob-compliance --story <story-id>` → 读取该 story 在 `docs/bob/02-stories-*.md` 里记录的 base ref,跑 `git diff <base>..HEAD`
2. **当前活跃 story**:`docs/bob/02-stories-*.md` 存在且能识别出"当前 story" → 用该 story 的 base..HEAD
3. **Fallback**:`git diff master..HEAD` + 未提交的工作目录变更
4. **`--all-branch` flag**:`git diff master..HEAD` 全分支 diff,忽略 story 划分

**收敛相关 spec(模式符合度专用):** 确定 diff 范围后,取 §9.5 里 `Context 类 / 落点包` 路径**与 diff 变更文件有交集**的 spec —— 只校验「diff 动了其 Context / 参与类」的模式,避免拿全仓 spec 误判。

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

**模式规则(PAT-*)校验:** 按 spec §9.5 声明的「可观察痕迹」逐项核对 diff + 相关已存在文件。
例(Strategy):① 接口 `*ShippingFeeStrategy` 存在且落 `usecase/port` ② 实现数 ≥2 ③ Context 依赖接口而非 `new` 具体类 —— 三项全过记 `PASS`,任一不满足记 `FAIL`。

### 3.3 分类

每条命中的规则归入三类:

| 分类 | 定义 |
|---|---|
| **违反** | 明确触碰规则;diff 里有反例代码 |
| **待量化** | 规则模糊或需求未给出基线(例:阿里规约要求"接口幂等",但 spec 未明确该接口是否要求幂等)→ 建议回 spec 补充 |
| **豁免** | spec 的"交给 Superpowers 的开放问题"段已显式注明豁免理由 |

模式规则(PAT-*)套用同一三类:**违反** = 声明了模式但 diff 找不到对应结构(接口缺失 / 只 1 实现 / Context 直接耦合具体类),按档位(采纳默认【强制】);**待量化** = §9.5 没给可观察痕迹或太模糊,建议回 `/bob-spec` 补痕迹;**豁免** = spec §10 开放问题显式注明降级理由。

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
  ```java
  String DaZhePromotion = ...;
  ```
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

## 模式符合度

> 仅当 scope 内 spec 含 §9.5 时出现;无模式规则时写「无模式约束」。

| ID | 模式 | 判定 | 位置 | 期望结构 vs 实际 | 修复建议 |
|---|---|---|---|---|---|
| PAT-1-1 | Strategy | **FAIL** | `usecase/PlaceOrderUseCase.java:88` | 期望:依赖 `ShippingFeeStrategy` 接口 + ≥2 实现;实际:`switch(region)` 内联 | 抽 `ShippingFeeStrategy` 接口落 `usecase/port`,各 region 一个实现落 `adapter/` |

全部 PASS 或无模式规则时,本段写一行:`无模式约束 / 全部符合`。

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

- **目录即配置**:`docs/compliance/sources/` 空 **且** scope 内无 spec 含 §9.5 ⇒ Stage 0 软退出
- **永不内置标准**:run-bob 二进制里**不**打包任何标准 PDF / md
- **upgrade 边界**:用户的 `sources/`、生成的 md、`.compliance.lock`、报告文件 —— run-bob upgrade **永不触碰**
- **PMD/SonarQube 兼容**:空 sources/ ⇒ 与现有静态扫描工具零冲突
- **Claude 唯一执行器**:PDF → md、规则抽取、diff 校验全在 Claude 内部,不引入新二进制 / 不调外部进程
