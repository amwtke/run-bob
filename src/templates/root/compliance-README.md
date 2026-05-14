# 项目级合规检查

这个目录由 `run-bob` 维护,目的是把团队 / 项目级的**自然语言规约**(命名、异常处理、
安全实践、漏洞防范)注入 Claude Code 工作流,让 AI 写代码时一次合规。

## 前置条件

`/bob-compliance` 只在 `sources/` **非空**时才真正跑:

- `sources/` 至少有一份规约文档(PDF / docx / md / txt) → 正常走 5 stage 流程
- `sources/` 空 → Stage 0 软退出,留一份空报告留痕,**无任何副作用**

换句话说:**不放规约文档 = 不启用合规检查**。run-bob 不会硬塞任何规则给你。

## 用法

1. 把项目要遵守的规约文档放进 `sources/`,任意格式(PDF / docx / md / txt)
2. 写 story 跑 `/bob-spec` 时,bob-spec 会自动提示是否需要先跑 `/bob-compliance`
3. `/bob-compliance` 会:
   - 把 `sources/` 里每份文档结构化成 `docs/compliance/<标准名>.md`(带规则 ID)
   - Superpowers TDD 时 Claude 自动读这些 md(由 CLAUDE.md R13 强制)
   - TDD 完成后对 diff 跑一次校验,产物落到 `docs/bob/05-compliance-<story>.md`

## 例外:已经有 PMD / SonarQube / SpotBugs ?

**保持 `sources/` 为空即可**。run-bob 不会重复跑机械可检项,
也不替代任何 IDE / CI 静态扫描工具。

`/bob-compliance` 只解决你的 CI 工具**不擅长**的那部分 —— 自然语言级的、
需要语义理解的规约(异常处理思路、安全实践、命名约定中的语义部分等)。

## 缓存机制

`.compliance.lock` 记录 `sources/` 里每份文件的 `filename + size + sha256`。
再次运行 `/bob-compliance` 时:

- sha256 完全一致 → 直接复用现有 `docs/compliance/*.md`,跳过生成
- 任何文件新增 / 修改 → 仅重新生成漂移的部分
- `sources/` 为空 → Stage 0 软退出,无任何副作用

## 不会发生的事

- `run-bob upgrade` **永远不会**触碰 `sources/`、`.compliance.lock`、或任何 `*.md` 生成产物
- run-bob 二进制**不内置**任何合规标准(版权 + 团队差异)
- `/bob-compliance` 不调用任何外部进程,纯 Claude 内部处理
