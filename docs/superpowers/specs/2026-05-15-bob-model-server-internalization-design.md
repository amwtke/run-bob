# run-bob · bob-model 评审服务内化设计(移除 superpowers 依赖)

> 状态:设计已定稿,待用户审阅 → 转交 `superpowers:writing-plans` 落地。
> 日期:2026-05-15
> 适用仓库:`/Users/xiaojin/workshop/run-bob`
> 实施目标:把 bob-model 交互式 review 用到的 WebSocket server / 客户端 helper / 启停脚本从 `superpowers/brainstorming` 复制到 run-bob 项目内,改 namespace 为 `bob-review`,解除 superpowers 插件版本耦合。
> 上游依据:
> - `docs/superpowers/specs/2026-05-15-bob-model-interactive-review-design.md`(交互式 review canvas 设计,本 spec 在其基础上抽掉 superpowers 依赖)
> - 4 个 commit(`045a858 / 575bde4 / ad130b0 / e2f3484 / bc8d231 / 420a6cb`)交互式 review V1 已上线 v0.4.0

---

## 0. 目的与一句话总结

V0.4.0 上线的交互式 review canvas 依赖 superpowers 插件的 `scripts/start-server.sh`(硬编码路径 `5.1.0`)。本次实施把那 4 个脚本**完整复制到 run-bob 内**,改命名为 `bob-review`,让 `/bob-model` 不再需要目标机器装 superpowers 即可工作。同时 bob-model skill 从 flat `.md` 转为标准 Claude Code dir skill(`bob-model/SKILL.md` + `bob-model/scripts/`)。

---

## 1. 背景与动机

### 1.1 当前 V0.4.0 状态(已 ship)

- bob-model skill 的 Stage 3 启服调用:`~/.claude/plugins/cache/.../superpowers/5.1.0/skills/brainstorming/scripts/start-server.sh`
- 路径硬编码 `5.1.0`(已记录为 minor m3 风险)
- 客户端依赖 `window.brainstorm.send`(superpowers 注入)
- session 目录在 `<project>/.superpowers/brainstorm/`

### 1.2 痛点

1. **版本耦合**:superpowers 升级到 `5.2.x+` 时,start-server.sh 路径失效 → Stage 3 启服失败 → 降级为只读 html
2. **受众限制**:没装 superpowers 插件的用户用不了 `/bob-model` 交互式 review
3. **语义错位**:bob-model 的"评审"场景被记到 superpowers 的"brainstorming"目录里,命名空间不贴合

### 1.3 范围(本次)

**做**:bob-model 的 review server 内化 + 改 namespace
**不做**(未来再说):
- 把内化的 server 抽成通用工具(供其它 bob-* / 非 bob skill 复用)
- 用 Rust 重写 server(去掉 Node.js 依赖)
- 多人协作 / 评论 thread / diff view 等 V2 功能

---

## 2. 文件布局

### 2.1 run-bob 源(`src/templates/`)

```
src/templates/
├─ skills/
│  ├─ bob-model/                       (新增 dir,替代旧 bob-model.md)
│  │  ├─ SKILL.md                       (原 bob-model.md 内容,改 Stage 3 启服路径)
│  │  └─ scripts/                       (新增,4 个脚本)
│  │     ├─ server.cjs                  (移植自 superpowers,改 namespace)
│  │     ├─ helper.js                   (移植自 superpowers,改 namespace)
│  │     ├─ start-server.sh             (移植 + 加 Node 探测 + 改 namespace)
│  │     └─ stop-server.sh              (移植,改 namespace)
│  ├─ bob-survey.md                     (不变,flat 单文件)
│  ├─ bob-stories.md                    (不变)
│  ├─ bob-identify.md                   (不变)
│  ├─ bob-onion.md                      (不变)
│  ├─ bob-spec.md                       (不变)
│  ├─ bob-nfr.md                        (不变)
│  └─ bob-compliance.md                 (不变)
```

**注意**:其它 bob-* skill **保留 flat .md** —— 风格不一致(bob-model 是 dir / 其它是 flat)是合理的,bob-model 比其它更复杂(带 scripts)。

### 2.2 目标项目装好后(`.claude/skills/`)

```
.claude/skills/
├─ bob-model/                          (整 dir,递归 cp 自 src/templates/skills/bob-model/)
│  ├─ SKILL.md
│  └─ scripts/
│     ├─ server.cjs                    (chmod 644)
│     ├─ helper.js                     (chmod 644)
│     ├─ start-server.sh               (chmod 755 可执行)
│     └─ stop-server.sh                (chmod 755 可执行)
└─ bob-*.md                            (其它 flat 不变)
```

### 2.3 运行时(目标项目根)

```
<project-root>/.bob/                   (新增,.gitignore)
└─ model-review/
   └─ <session-id>-<ts>/               (e.g. 57340-1778813594)
      ├─ content/
      │  ├─ 03-model-<slug>-<date>.html
      │  ├─ 03-model-<slug>-<date>-v2.html
      │  └─ ...
      └─ state/
         ├─ server-info                 (启动后写,含 port/url/screen_dir/state_dir)
         ├─ server-stopped              (关闭后写,含 reason/timestamp)
         ├─ server.pid
         ├─ server.log                  (JSONL,server 行为日志)
         └─ events                      (JSONL,bob-model-feedback 事件)
```

### 2.4 `.gitignore` 改动

```
# bob-model interactive review session state (runtime, ephemeral)
.bob/
```

(替代 V0.4.0 加的 `.superpowers/` 条目;`.superpowers/` 仍然保留以兼容 brainstorming 自己的 session)

---

## 3. Namespace 变更清单

| 原(superpowers) | 新(bob-review) |
|---|---|
| `BRAINSTORM_PORT` | `BOB_REVIEW_PORT` |
| `BRAINSTORM_HOST` | `BOB_REVIEW_HOST` |
| `BRAINSTORM_URL_HOST` | `BOB_REVIEW_URL_HOST` |
| `BRAINSTORM_DIR` | `BOB_REVIEW_DIR` |
| `BRAINSTORM_OWNER_PID` | `BOB_REVIEW_OWNER_PID` |
| `window.brainstorm` (JS 全局) | `window.bobReview` |
| `window.brainstorm.send / .choice` | `window.bobReview.send / .choice` |
| session dir `<root>/.superpowers/brainstorm/<id>-<ts>/` | `<root>/.bob/model-review/<id>-<ts>/` |
| script log prefix `source: 'user-event'` | 不变(server 内部,不影响接口) |

skill 模板里的 JS(page-helper)已经引用 `window.brainstorm.send` —— 本次实施需要同步更新为 `window.bobReview.send`(在 `src/templates/skills/bob-model/SKILL.md` 的 § JS 块里 sed-like 替换)。

---

## 4. start-server.sh 启服路径

### 4.1 改前(V0.4.0,bob-model.md Stage 3.1)

```bash
SCRIPT=~/.claude/plugins/cache/claude-plugins-official/superpowers/5.1.0/skills/brainstorming/scripts/start-server.sh
"$SCRIPT" --project-dir <repo-root>
```

### 4.2 改后

```bash
SCRIPT="<project-root>/.claude/skills/bob-model/scripts/start-server.sh"
"$SCRIPT" --project-dir <project-root>
```

`<project-root>` 由 Claude 在 Stage 3 跑前用 `git rev-parse --show-toplevel` 或 `pwd` 取得;不依赖具体用户名。

### 4.3 fallback(`start-server.sh` 不可执行 / 不存在)

- 旧降级路径**保留**:若启服失败,Stage 3 把 Stage 2 compose 的 html 直接写到 `<root>/docs/bob/03-model-<slug>-<date>.html`(只读 fallback),通知用户"interactive review 不可用"
- 新增情况:目标项目是用旧版 run-bob(无 dir skill)init 出来的 → 脚本路径不存在 → 同上降级,并提示用户跑 `run-bob upgrade-safe-assets`

---

## 5. Node.js 探测

### 5.1 `start-server.sh` 开头新增

```bash
if ! command -v node >/dev/null 2>&1; then
  echo '{"error":"node not found","fix":"Install Node.js >=14 to use interactive review. Skill will fall back to read-only html if unavailable."}' >&2
  exit 2
fi

NODE_MAJOR=$(node -e 'process.stdout.write(String(process.versions.node.split(".")[0]))')
if [ "$NODE_MAJOR" -lt 14 ]; then
  echo "{\"error\":\"node $NODE_MAJOR too old\",\"fix\":\"Upgrade Node.js to >=14\"}" >&2
  exit 3
fi
```

退出码:`0 ok / 2 node missing / 3 node too old / 其它 server.cjs 自身错误`。

### 5.2 `run-bob install` skill 加 Node 探测(可选)

`install` skill 跑完 `cargo install` 后,加一步检测:
- `command -v node && node --version` 报告
- 若没装,打印 warning:"Node.js (>=14) is needed for /bob-model interactive review. Without it, you can still use bob-model with read-only fallback."

不强制装 node;只是给用户知情权。

### 5.3 README 改动

`README.md` "依赖" 段加一行:
```
- (可选)Node.js >=14 —— 仅 /bob-model 交互式 review 需要;不装则降级为只读 html
```

---

## 6. 资产分发(run-bob CLI 怎么把 dir skill 拷到目标项目)

### 6.1 现有 `run-bob init` 行为

跑 `cargo run -- init` 时,run-bob 从 `src/templates/` 拷贝到目标项目根:
- `src/templates/root/*` → `<project>/`
- `src/templates/skills/*.md` → `<project>/.claude/skills/`(目前只处理 flat .md)

### 6.2 改动:递归 cp 支持 dir skill

`src/init.rs`(或等价位置)的 skill 拷贝逻辑:

```rust
// Before:
for entry in fs::read_dir("src/templates/skills")? {
    if entry.path().extension() == Some("md") {
        copy_skill_md(entry.path(), target);
    }
}

// After:
for entry in fs::read_dir("src/templates/skills")? {
    let path = entry.path();
    if path.is_dir() {
        copy_skill_dir(&path, target)?;  // recursively copy dir
    } else if path.extension() == Some("md") {
        copy_skill_md(&path, target)?;   // unchanged for flat skills
    }
}
```

`copy_skill_dir`: 递归 cp 整个目录,保持文件权限(scripts/`*.sh` 需 chmod +x);跳过 user-owned 文件覆盖。

### 6.3 改动:`upgrade-safe-assets` 同步

`run-bob upgrade-safe-assets`(用户已有的升级命令)逻辑同样改 — 对 dir skill 整 dir 比对,新文件加,旧文件升,user-owned 保留(per 现有规则)。

`upgrade-safe-assets` 必须能处理"flat .md → dir skill"的旧→新过渡(用户原 bob-model.md 单文件 → 升级后变 bob-model/SKILL.md + scripts/)。具体策略:
- 检测到目标项目有旧 `bob-model.md` flat → 删旧 + 装新 dir(不属 user-owned 因为是 run-bob 拥有的)
- 检测到目标项目已有新 `bob-model/` dir → 内部递归 upgrade

### 6.4 二进制嵌入

run-bob 现在用 `include_str!` 嵌入模板?或者运行时 `fs::read`?需检查现有实现。若 `include_str!`,需要为 dir skill 的每个文件单独 `include_str!` 一份,在 init 时按相对路径写出。简单的实现:用 `include_dir!` crate(社区维护)或者手写一个 build.rs 生成清单。

**实施 plan 里**应明确这一点:check 当前嵌入方式,如果用 `include_str!` 则加 4 个新的 const + 写盘逻辑;如果有更动态的方式则改一处。

---

## 7. LICENSE / 归属

`server.cjs` / `helper.js` / `start-server.sh` / `stop-server.sh` 顶部加注释 header:

```
// Adapted from superpowers brainstorming visual companion
// Source: https://github.com/anthropics/claude-plugins-official superpowers@5.1.0
// License: MIT (Anthropic)
// Migrated to run-bob with namespace bob-review for /bob-model interactive review.
```

对于 bash 脚本用 `#` 注释,JS 用 `//` 注释。

run-bob `README.md` 的 "Acknowledgements" / 类似段落加一行(或新加):
```
Interactive review server adapted from superpowers brainstorming visual companion (MIT, Anthropic).
```

---

## 8. SKILL.md 与 page-helper JS 的改动

### 8.1 `src/templates/skills/bob-model/SKILL.md`

由 `src/templates/skills/bob-model.md` 改名而来;内容上的具体改动:

| 位置 | 改动 |
|---|---|
| Stage 3.1 启服路径 | 改为 `<project-root>/.claude/skills/bob-model/scripts/start-server.sh` |
| Stage 3.5 启服重试 | 同上路径 |
| §html widget 规范 → §JS → `window.brainstorm.send` | 替换为 `window.bobReview.send`(2-3 处) |
| §html widget 规范 → §JS → "WebSocket helper 未加载" toast | 文案保留,但 helper 检测从 `!window.brainstorm` 改为 `!window.bobReview` |

### 8.2 不动的部分

- §命名规约 / §产物报告规约 / §评论协议与 schema / §html widget 规范的非 JS 部分 / Stage 0/1/2/4 / 不变量列表 / 工作流概述 —— 全部不动

---

## 9. 错误处理矩阵

| 错 | 检测 | 恢复 |
|---|---|---|
| Node.js 缺失 | start-server.sh 退出码 2 | Claude 终端报"node missing";降级到只读 html |
| Node.js < 14 | start-server.sh 退出码 3 | Claude 报"node too old";降级到只读 html |
| 脚本路径不存在(旧版 init / 未 upgrade) | start-server.sh 启动前 stat 失败 | Claude 报"upgrade run-bob assets";建议跑 `run-bob upgrade-safe-assets` |
| start-server.sh 启服失败(端口冲突 / OS 限制) | 退出码非 0 | Claude 报具体 stderr;降级到只读 html |
| `.bob/model-review/` 创建失败(权限 / 只读 FS) | mkdir 失败 | Claude 报路径 + 建议 |
| `window.bobReview` 未注入(browser bug) | page JS 加载时 `if (!window.bobReview)` | toast 提示用户;submit 按钮 disabled |
| server 中途死 | Claude 读 events 前 stat server-stopped | 自动重启(复用 session-dir) |
| 旧 dir 残留(stale session) | session-dir 已存在 | 复用(server.cjs 已支持);多 session 共存 |

---

## 10. 测试策略

### 10.1 集成测试(新增 / 改动)

| 测试名 | 验证 |
|---|---|
| `init_creates_bob_model_skill_dir`(改) | `<project>/.claude/skills/bob-model/SKILL.md` 存在;`scripts/{server.cjs,helper.js,start-server.sh,stop-server.sh}` 存在;`*.sh` 有可执行权限 |
| `init_creates_bob_model_skill`(改) | SKILL.md 保留原 token 列表(`classDiagram` / `stateDiagram-v2` / `flowchart` / `粘性 TOC` / `BR-` / 不变量 / ...);加 `window.bobReview` |
| `upgrade_migrates_flat_to_dir`(新) | 模拟旧版项目(有 `bob-model.md` flat)→ 跑 upgrade → 验证转成 dir 结构 |
| `upgrade_preserves_user_compliance_sources`(现有) | 仍然 pass,不受影响 |
| `start_server_script_has_node_check`(新) | `grep "command -v node" start-server.sh` 命中 |
| `bob_model_skill_uses_bobreview_namespace`(新) | SKILL.md 不含 `window.brainstorm`;含 `window.bobReview` |

### 10.2 手动端到端

跑一遍完整 `/bob-model` 在干净项目(不装 superpowers):
1. ✓ `run-bob init` 创建 bob-model/ dir + scripts
2. ✓ 跑 `/bob-model <doc>` 时 start-server.sh 启动 + server.cjs 监听 + 给 URL
3. ✓ 浏览器开 URL,看见 widgets
4. ✓ 提交评论,events 写到 `.bob/model-review/<session>/state/events`
5. ✓ Claude 终端"继续",读 events,重写 html,推 screen_dir
6. ✓ Stage 4 推进,dump md,SIGTERM server

### 10.3 回归

`/bob-model` 在装了 superpowers 的机器上**也能**跑 —— 因为本次实施是把脚本从 superpowers 路径搬到项目本地,**两者共存不冲突**:
- bob-model 用 `.bob/model-review/`(新)
- brainstorming 用 `.superpowers/brainstorm/`(旧,不动)
- 两者 namespace 不重(BOB_REVIEW_* vs BRAINSTORM_*)
- 浏览器 global 不重(`window.bobReview` vs `window.brainstorm`)

---

## 11. V1 范围 / V2 留底

### V1(本设计)

- Node.js port + namespace rename
- bob-model 从 flat → dir skill
- Node.js 探测(优雅降级)
- 集成测试改造

### V2 留底(不在本次)

- ❌ Rust 重写 server(去掉 Node 依赖)
- ❌ 抽通用 web-review primitive(供 bob-survey / bob-stories / ... 复用)
- ❌ 多人 review 协作(per 上一个 spec §8)
- ❌ Windows native shell 启服(目前用 bash,需要 Git Bash;Windows native 留 V2)

---

## 12. 实施计划(供 `superpowers:writing-plans` 接力)

预计 12-15 个 task,大致分组:

1. **复制 4 个脚本到 src/templates/scripts/bob-model/** + namespace rename
2. **flat skill → dir skill**(bob-model.md → bob-model/SKILL.md + scripts/)
3. **更新 SKILL.md**(启服路径 + window.bobReview)
4. **改 Rust init/upgrade 逻辑**(支持 dir skill)
5. **加 Node 探测**(start-server.sh)
6. **集成测试改造 / 新增**(6 项)
7. **README + ARCHITECTURE.md** 更新
8. **手动 e2e 测试**
9. **commit + push + bump v0.5.0 + release tag**

writing-plans 应把每组拆为 bite-sized step(2-5 min each)。

---

## 13. 风险与对策

| 风险 | 概率 | 影响 | 对策 |
|---|---|---|---|
| `include_str!` 在 dir skill 下要写一堆 const(笨重) | 高 | 低-中 | 用 `include_dir!` crate(社区)或 build.rs 生成清单;两者择一 |
| upgrade 路径处理 "flat → dir" 过渡有 bug | 中 | 高 | 单独写测试覆盖;给用户 dry-run 选项 |
| 复制的脚本 bash shebang 在 Windows 不工作 | 中 | 中 | Windows 用户用 Git Bash(已说明);native PowerShell 留 V2 |
| LICENSE 不一致 / 上游有 patent claim | 低 | 高 | 加 header 注释明示 + README acknowledgement;Anthropic MIT 应该 ok |
| run-bob 镜像源 / cargo install 网络问题让 `node` 探测误判 | 低 | 低 | install 不强求 node;仅 warning |

---

## 14. 决策日志(brainstorming 记录)

- **范围**:只做 bob-model,不抽通用 primitive(用户:"先做 bob-model 这块的重构,后面的事情以后说")
- **skill 包装**:转 dir(`bob-model/SKILL.md` + `scripts/`),与 Claude Code 标准 dir skill 一致;其它 bob-* 保持 flat
- **namespace**:`bob-review`(取语义,区别于 brainstorming)
- **状态目录**:`<root>/.bob/model-review/<session>/`(项目本地,.gitignore)
- **迁移策略**:Node port + rename(不 Rust 重写,3-4 小时 vs 2-3 天)
- **Node 必需**:是;装 Node ≥14;不装则降级到只读 html

---

*Generated by `superpowers:brainstorming` · 2026-05-15 · 待 `superpowers:writing-plans` 接力*
