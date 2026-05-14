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
| **空仓** | `docs/compliance/sources/` 不存在或为空 | 软退出:"无合规要求,跳过"。**写一份空报告留痕**(避免下次重复探测) |
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

**Severity 优先级:【强制】> 【推荐】> 【参考】**。三档**都参与**校验,但分类报告时:

- 【强制】违反 → 视为必须修复
- 【推荐】违反 → 列出但允许豁免
- 【参考】违反 → 列出仅作提示

向用户通报装载结果:

> **Q1: 装载到 N 条规则,来自 M 份标准。**
>
> **推测**:【强制】X 条 / 【推荐】Y 条 / 【参考】Z 条
> **理由**:从已生成的 `docs/compliance/*.md` 索引得出
> **推荐选择**:`进入 Stage 3 跑校验`
>
> 是否同意?
