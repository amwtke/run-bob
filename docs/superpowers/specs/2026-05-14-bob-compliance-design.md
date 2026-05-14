# run-bob · `/bob-compliance` 技能设计(phase 3)

> 状态:设计已定稿,待用户审阅 → 转交 `superpowers:writing-plans` 落地。
> 日期:2026-05-14
> 适用仓库:`/Users/xiaojin/workshop/run-bob`
> 参照文档:`/Volumes/ExternalSSD/Downloads/阿里巴巴Java开发规范（嵩山版）.pdf`(1.8MB, 59 页, 7 维度 × 强制/推荐/参考)
> 相关 spec:
> - `docs/superpowers/specs/2026-05-14-bob-nfr-design.md` (phase-2 NFR 复盘 — 结构对称)
> - `docs/superpowers/specs/2026-05-08-run-bob-design.md` (v0.1.0 整体设计)

---

## 0. 目的与一句话总结

为 run-bob harness 增加 **phase-3 合规校验** 能力。在团队提供项目级合规文档(阿里 Java 规约 / 中心 CCAF / 信息中心漏洞规范 / 任意自定义)之后,`/bob-compliance` 完成两件事:

1. **指导(模式 A)** —— TDD 开始之前把规则注入 Claude 的可见上下文,让代码"一次写对"
2. **校验(模式 B)** —— TDD 完成之后对 diff 跑一遍,列违规清单 + 修复建议 story

合规文档**不内置**,由团队按约定路径自行提供;**目录留空 = 软退出**,与已有 PMD/SonarQube 等 CI 静态扫描不冲突。

---

## 1. 背景

### 1.1 现状

run-bob 当前的链路 `survey → stories → identify → onion → spec → Superpowers TDD → nfr` 没有任何 **代码合规** 把关。
团队普遍面临:

- 内部规范文档(中心 CCAF、信息中心漏洞分析规范)散落在 wiki/PDF,没人翻
- 阿里 Java 规约 / Google Java Style 大家"听说过"但写代码时记不住
- IDE 插件能扫一部分,但只覆盖**机械可检**项(命名、空格);**自然语言级**条款(异常处理思路、安全实践)无人执行
- 同一份代码不同人 review 标准不一致,review 成本高

### 1.2 用户提的 3 条原始需求(2026-05-14)

| # | 原话 | 落地点 |
|---|---|---|
| 1 | `init` 创建 `docs/compliance/` + 提示:有 PMD 就别管 | run-bob init 改动 |
| 2 | 用户放进原始文件 → 动态生成 md → 指导编码 | `/bob-compliance` Stage 0-1(生成) + R14 规则(指导) |
| 3 | 下次发现已有合规文件且无新增 → 直接复用 | `.compliance.lock` 缓存校验 |

### 1.3 设计哲学

- **目录即配置** —— 文件夹空 / 满 决定行为;无开关,无配置文件
- **不内置标准** —— 版权 + 团队差异,只提供约定路径和 Claude 的转换能力
- **R 规则 + 技能 双层** —— 硬约束(R14)在 CLAUDE.md;技能 `/bob-compliance` 做生成 + 校验
- **对称 `/bob-nfr`** —— 5 stage 三段式("推测 / 推荐选择"),phase-2 / phase-3 走同一形状

---

## 2. 文件 / 目录布局

### 2.1 项目目录(用户侧)

```
your-project/
├── docs/
│   ├── compliance/
│   │   ├── README.md                    # ← run-bob init 写,upgrade-safe
│   │   ├── sources/                     # ← 用户原始文件,任意格式
│   │   │   ├── 阿里巴巴Java开发规范（嵩山版）.pdf
│   │   │   ├── ccaf-internal.docx       # (示例,未来加入)
│   │   │   └── vuln-it-center.md
│   │   ├── .compliance.lock             # ← Claude 维护
│   │   ├── alibaba-songshan.md          # ← /bob-compliance 生成
│   │   ├── ccaf-internal.md             # ← 生成
│   │   └── vuln-it-center.md            # ← 生成(可能 md→md,只是结构化)
│   ├── bob/
│   │   └── 05-compliance-<story>.md     # ← 每个 story 一份复盘报告
│   └── specs/
└── .claude/skills/bob-compliance/SKILL.md  # ← 新技能
```

### 2.2 文件归属表

| 文件 / 目录 | 谁创建 | 谁维护 | upgrade 行为 |
|---|---|---|---|
| `docs/compliance/` | `run-bob init` | run-bob | 目录存在性保证 |
| `docs/compliance/README.md` | `run-bob init` | run-bob 模板 | **upgrade_safe = true**,模板更新会覆盖;**included_in_minimal = false**(同 CLAUDE.md / ARCHITECTURE.md) |
| `docs/compliance/sources/` | `run-bob init` | 用户 | 目录存在性保证,内容不动 |
| `docs/compliance/sources/*` | 用户手动 | 用户 | run-bob 永不触碰 |
| `docs/compliance/*.md`(生成) | `/bob-compliance` | Claude | run-bob 永不触碰 |
| `docs/compliance/.compliance.lock` | `/bob-compliance` | Claude | run-bob 永不触碰 |
| `docs/bob/05-compliance-*.md` | `/bob-compliance` | Claude | run-bob 永不触碰 |
| `.claude/skills/bob-compliance/SKILL.md` | `run-bob init` | run-bob 模板 | **upgrade_safe = true** |

### 2.3 `docs/compliance/README.md` 模板内容

由 run-bob 模板写入,固定 3 段:

```markdown
# 项目级合规检查

这个目录由 `run-bob` 维护,目的是把团队 / 项目级的**自然语言规约**(命名、异常处理、
安全实践、漏洞防范)注入 Claude Code 工作流,让 AI 写代码时一次合规。

## 用法

1. 把项目要遵守的规约文档放进 `sources/`,任意格式(PDF / docx / md / txt)
2. 写 story 跑 `/bob-spec` 时,bob-spec 会自动提示是否需要先跑 `/bob-compliance`
3. `/bob-compliance` 会:
   - 把 `sources/` 里每份文档结构化成 `docs/compliance/<标准名>.md`(带规则 ID)
   - Superpowers TDD 时 Claude 自动读这些 md(由 CLAUDE.md R14 强制)
   - TDD 完成后对 diff 跑一次校验,产物落到 `docs/bob/05-compliance-<story>.md`

## 例外:已经有 PMD / SonarQube / SpotBugs ?

**保持 `sources/` 为空即可**。run-bob 不会重复跑机械可检项,
也不替代任何 IDE / CI 静态扫描工具。

`/bob-compliance` 只解决你的 CI 工具**不擅长**的那部分 —— 自然语言级的、需要语义理解的规约。

## 缓存

`.compliance.lock` 记录 sources/ 里每份文件的 `filename + size + sha256`。
再次运行 `/bob-compliance` 时,sha256 未变 → 直接用现有 md,不重新生成。
```

---

## 3. 新技能 `/bob-compliance`

技能文件:`.claude/skills/bob-compliance/SKILL.md`(英文 frontmatter + 中文正文,同 bob-nfr / bob-survey)。

### 3.1 Frontmatter

```yaml
---
name: bob-compliance
description: 项目级合规校验 phase 3 技能 — 把 docs/compliance/sources/ 下的合规文档结构化成 markdown,并在 TDD 完成后对 diff 做合规校验。
---
```

### 3.2 三段式约定(对齐其他 bob-* 技能)

每个 stage 输出**推测**章节(自动判断 + 解释)和**推荐选择**章节(明确给出建议,用户可推翻)。

### 3.3 五个 Stage

#### Stage 0:状态探测

读取并判定:

| 状态 | 判定 | 后续行为 |
|---|---|---|
| **空仓** | `docs/compliance/sources/` 不存在或为空 | 软退出:"无合规要求,跳过"。返回 |
| **首次** | `sources/` 有文件,但无 `.compliance.lock` | → Stage 1 全量生成 |
| **漂移** | `sources/` 有新增 / 修改文件(sha256 不匹配 lock) | → Stage 1 增量生成漂移部分 |
| **冷藏** | sources sha256 与 lock 完全一致 | 跳过 Stage 1,直接 → Stage 2 |

#### Stage 1:生成结构化 markdown(模式 A 准备工作)

对每个新增 / 漂移的 `sources/<file>` 做转换:

1. **读原文** —— 按格式分派:`.pdf` 用 Read 工具的 `pages` 参数分批读;`.docx` 同样;`.md/.txt` 直接读
2. **抽取规则** —— Claude 识别:
   - 一级目录(7 大维度,阿里 PDF 例:编程规约 / 异常日志 / 单元测试 / 安全规约 / MySQL / 工程结构 / 设计规约)
   - 二级目录(每个维度内的子节)
   - 单条规则(标题 + 强制档位 + 反例 + 正例 + 说明)
3. **写结构化 md** —— 输出 `docs/compliance/<filename-stem>.md`,固定 schema:

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
  - §1.3 代码格式
  - §1.4 OOP 规约
  ...
- §2 异常日志
  - §2.1 错误码
  - §2.2 异常处理
  - §2.3 日志规约
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

**检测提示:** 静态扫描可覆盖;diff 级检测 `grep -nE "^[_$]|[_$]$"` 标识符行

### [ALI-1.1.2] 【强制】禁止拼音英文混合,禁止纯中文命名

**正例:** `ali` / `alibaba` / `taobao` / `hangzhou`(国际通用名)
**反例:** `DaZhePromotion[打折]` / `String fw[福娃]` / `int 某变量`

...
```

4. **更新 `.compliance.lock`** —— YAML,**整文件原子重写**(不增量改);单条记录写完 + 整文件落盘后再处理下一个源文件,中途中断重跑时 Stage 0 会自然识别为"漂移"并补全:

```yaml
generated_at: 2026-05-14T03:21:00Z
sources:
  - filename: 阿里巴巴Java开发规范（嵩山版）.pdf
    size: 1908201
    sha256: a1b2c3...
    generated_to: alibaba-songshan.md
```

**边界情况:**
- 源文件不可读(损坏 / 权限)→ 输出错误,跳过该文件继续后续;lock 不记录该文件;下次重跑时仍按"漂移"处理
- 同一 stem 重复(例如 `foo.pdf` 和 `foo.docx`)→ 生成 `foo.md` 时后写覆盖前写,**并在 lock 里报告冲突**;建议用户改名

#### Stage 2:装载已有 md(双模式都用)

读所有 `docs/compliance/*.md`(不含 `README.md`),建立内存索引:`rule_id → (file, section, severity, title)`。

**Severity 优先级:【强制】> 【推荐】> 【参考】**。三档**都参与**校验,但分类报告时:
- 【强制】违反 → 视为必须修复
- 【推荐】违反 → 列出但允许豁免
- 【参考】违反 → 列出仅作提示

#### Stage 3:Diff 校验(模式 B 核心)

1. **定位检查范围**(优先级从高到低):
   1. 显式参数:`/bob-compliance --story <story-id>` → 该 story 在 `docs/bob/02-stories-*.md` 里记录的 base ref..HEAD
   2. `docs/bob/02-stories-*.md` 存在且当前活跃 story 可识别 → 用该 story 的 base..HEAD
   3. fallback:当前 git 工作目录 vs `master`(`git diff master..HEAD` + uncommitted changes)
   4. `--all-branch` flag:取 `master..HEAD` 全分支 diff,忽略 story 划分
2. **逐条规则比对:**
   - 跑 `git diff <base>..HEAD`,得到新增 / 修改行
   - 对每条规则,在 diff 内匹配:
     - **可机械检测**(命名规则、空格、关键字)→ Claude 用模式匹配/正则
     - **需语义判断**(异常处理思路、并发设计)→ Claude 逐文件逐函数检视
3. **分类输出:**
   - `违反` —— 明确触碰【强制】或【推荐】条款
   - `待量化` —— 规则模糊或需求未给出基线
   - `豁免` —— spec 已在"开放问题"段注明豁免

#### Stage 4:报告 + 下一步

写 `docs/bob/05-compliance-<story>.md`,固定结构:

```markdown
# 合规校验报告 · <story-name>

**日期:** 2026-05-14
**范围:** <base-ref>..HEAD,N 个文件 / M 行新增
**加载标准:** alibaba-songshan, ccaf-internal, ...

## 违反清单

### 【强制】违反 (2 条)

#### [ALI-1.1.2] 禁止拼音英文混合
- **位置:** `src/main/java/com/example/order/OrderService.java:42`
- **代码片段:**
  ```java
  String DaZhePromotion = ...;
  ```
- **修复建议:** 改为 `String discountPromotion = ...;` 或 `String promotion = ...;`

#### [ALI-2.2.1] 异常不能裸吞
- **位置:** `src/main/java/com/example/order/OrderRepository.java:78-80`
- **代码片段:**
  ```java
  try { ... } catch (Exception e) { }
  ```
- **修复建议:** 至少 `log.error("...", e)`,或重抛业务异常

### 【推荐】违反 (1 条)
...

### 【参考】违反 (0 条)

## 待量化 (1 条)
- [ALI-7.3] 设计规约要求"接口幂等" —— spec 未明确该接口是否要求幂等,建议回 spec 补充

## 豁免 (0 条)

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

---

## 4. CLAUDE.md 新增 R14

R14 是**模式 A 的实施载体**。位置:CLAUDE.md "R 规则"段末尾,紧接 R13。

```markdown
**R14:合规规则前置(项目级)** ——
实现代码之前,**必须**检查 `docs/compliance/*.md`:
- 若不存在或目录为空 → 跳过
- 若存在 → 读取与当前文件 / 模块相关的章节,**严格遵守所有【强制】条款**
- 在代码注释里引用规则 ID(例:`// 遵守 [ALI-1.1.2] 命名规约`),便于后续 `/bob-compliance` 校验时复核
- 不得擅自违反【强制】条款 —— 如确需违反,必须在 spec 的"开放问题"段写明豁免理由,
  否则 `/bob-compliance` Stage 3 会标记为违反

为什么 R 规则形式而非工具调用:Claude 在 TDD 时本来就读 CLAUDE.md,R 规则是最廉价的注入路径。
合规知识 = 自然语言规约,Claude 直接消费 markdown,无需任何额外工具。
```

---

## 5. bob-spec 的 `下一步` 段改动

对齐已有 `/bob-nfr` 提醒模式。bob-spec 三个模板(A/B/C)的"下一步"段都加一行:

```diff
+ 4. (可选,如 docs/compliance/sources/ 非空)Superpowers 实现 + UT 完成后,
+    先跑 `/bob-compliance` 做合规校验,产物落 docs/bob/05-compliance-<story>.md,
+    再跑 `/bob-nfr` 做非功能复盘
```

如果 sources/ 为空,提醒文本依然在,只是 Claude 实际跑 `/bob-compliance` 时 Stage 0 软退出 —— 一次性,无副作用。

---

## 6. run-bob 二进制改动总览

| 文件 | 动作 | 内容 |
|---|---|---|
| `src/templates/skills/bob-compliance.md` | **新建** | 新技能,约 400 行(对标 bob-nfr ~350 行) |
| `src/templates/root/compliance-README.md` | **新建** | docs/compliance/README.md 模板 |
| `src/templates/root/CLAUDE.md` | **改** | 末尾加 R14 |
| `src/templates/skills/bob-spec.md` | **改** | A/B/C 三个模板的"下一步"加合规提醒 |
| `src/assets.rs::HARNESS_ASSETS` | **改** | 新增 2 项:`bob-compliance/SKILL.md`(Skill 类) 和 `docs/compliance/README.md`(HarnessDoc 类) —— 都 `upgrade_safe = true` |
| `src/assets.rs::HARNESS_DIRS` | **改** | 新增 `docs/compliance/sources/` |
| `tests/integration.rs` | 加 ~8 测试 | 见 §7 |

**不改动:**
- `src/commands/init.rs` —— 现有循环处理新 asset / dir,无需修改
- `src/commands/upgrade.rs` —— 同上
- `src/commands/status.rs` —— drift guard 自动捕获新 asset
- `src/commands/gitignore.rs` —— 与本特性无关

---

## 7. 测试

### 7.1 init 集成测试

1. **init_creates_compliance_dir** —— `docs/compliance/` 和 `docs/compliance/sources/` 存在
2. **init_creates_compliance_readme** —— `docs/compliance/README.md` 存在,关键 token 出现:`sources/`, `PMD`, `/bob-compliance`, `.compliance.lock`, `SonarQube`
3. **init_minimal_skips_compliance_dirs** —— `init --minimal` 不创建 compliance/ 目录(但 skill 还在)
4. **init_creates_bob_compliance_skill** —— `.claude/skills/bob-compliance/SKILL.md` 存在,关键 token:`/bob-compliance`, `docs/compliance/sources/`, `.compliance.lock`, `Stage 0`-`Stage 4`, `【强制】`, `R14`

### 7.2 CLAUDE.md / bob-spec 改动测试

5. **claude_md_has_r14** —— `CLAUDE.md` 包含 `R14`、`docs/compliance/`、`/bob-compliance`、`【强制】`
6. **bob_spec_mentions_compliance** —— bob-spec.md 三个模板都提到 `/bob-compliance`(出现次数 ≥ 3)

### 7.3 upgrade 行为测试

7. **upgrade_safe_field_matches_for_compliance_assets** —— 现有的 SSoT drift guard 测试自动覆盖新 asset
8. **upgrade_preserves_user_sources** —— 模拟用户在 `docs/compliance/sources/` 放文件,跑 upgrade,文件未被触碰

---

## 8. 不变量

- **目录即配置:** sources/ 空 ⇒ /bob-compliance 软退出;不需要任何 CLI flag
- **永不内置标准:** run-bob 二进制里**不**打包阿里 / 任何标准 PDF 或 md
- **upgrade 边界:** 用户的 sources/、生成的 md、lock 文件、报告文件 —— run-bob upgrade **永不触碰**
- **PMD/SonarQube 兼容:** run-bob 不替代静态扫描工具;空目录策略让团队选择是否启用
- **Claude 是唯一执行器:** PDF→md、规则抽取、diff 校验,全在 Claude 内部完成,不引入新二进制 / 不调外部进程
- **R14 是模式 A 的唯一载体:** 不在技能里硬编码"读 compliance md",靠 R 规则机制注入

---

## 9. 未来扩展(YAGNI,不实现)

- `run-bob compliance import <pdf>` —— 命令行级别的 PDF 转换工具
- 内置标准目录(maven-central 风格,团队订阅)
- 把规则注入 ArchUnit `FORBIDDEN_IN_INNER` 数组 —— 只有可机械检测的子集
- 与 Sonar / SpotBugs 的 XML report 互导
- `/bob-compliance --fix` 自动修复(Claude 直接改代码)

---

## 10. 验收清单

- [ ] `cargo test` 全绿,新增 8 测试通过
- [ ] `run-bob init /tmp/test` 后,`docs/compliance/` 和 `docs/compliance/sources/` 存在,README.md 内容正确
- [ ] 把阿里 PDF 放进 `sources/`,Claude Code 里跑 `/bob-compliance`,Stage 1 生成 `alibaba-songshan.md`,lock 写入正确
- [ ] 第二次跑 `/bob-compliance`,Stage 0 命中"冷藏",不重新生成
- [ ] 触碰 PDF(改名 / 替换)后,Stage 0 命中"漂移",Stage 1 重新生成
- [ ] 写一个故意违规的 Java 文件,跑 `/bob-compliance`(模式 B),Stage 4 报告里能列出违规
- [ ] CLAUDE.md 含 R14;bob-spec 三个模板都提到 /bob-compliance
- [ ] `init --minimal` 不创建 compliance 目录,但 skill 仍安装
