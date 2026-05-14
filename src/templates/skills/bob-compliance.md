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
