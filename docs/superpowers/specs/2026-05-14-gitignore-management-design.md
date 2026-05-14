# run-bob · `init` / `upgrade` 自动维护 `.gitignore` 设计

> 状态:设计已定稿,待用户审阅 → 转交 `superpowers:writing-plans` 落地。
> 日期:2026-05-14
> 适用仓库:`/Users/xiaojin/workshop/run-bob`
> 相关 spec:
> - `docs/superpowers/specs/2026-05-08-run-bob-design.md` (v0.1.0 整体设计)
> - `docs/superpowers/specs/2026-05-13-run-bob-upgrade-design.md` (`upgrade` 子命令,首次引入 `.run-bob-backup/`)

---

## 0. 目的

`run-bob upgrade` 会在目标目录下创建 `.run-bob-backup/<timestamp>/` 用于回滚保护;该目录是**本地/瞬态产物**,不应进入 git 历史。本特性让 `init` / `upgrade` 在运行时主动维护目标目录的 `.gitignore`,把 run-bob 产生的瞬态产物自动加入 ignore 列表,避免:

1. 用户每次手工编辑 `.gitignore`;
2. `.run-bob-backup/` 误提交到团队仓库,污染历史。

> 注:harness 自身的核心产物(`CLAUDE.md` / `ARCHITECTURE.md` / `.claude/skills/**` / `src/test/java/architecture/CleanArchitectureTest.java` / `src/main/java/com/example/shared/**`) **仍然提交到 git**,这是 run-bob 的本意 —— 团队通过这些共享文件对齐架构约束。本特性**只 ignore 瞬态产物**。

---

## 1. 适用范围

### 1.1 触发命令

- `run-bob init <target>` 在 target 目录写完所有 asset 之后执行
- `run-bob upgrade <target>` 在 target 目录写完所有 asset 之后执行

`status` 不涉及。

### 1.2 操作位置

- 只操作 **target 目录下** 的 `.gitignore`,不沿父目录向上查找 git 仓库根。
- 这避免 monorepo 中误改根 `.gitignore`,语义与用户预期一致(「在某个文件夹运行命令」)。

### 1.3 ignore 内容(当前版本)

```
# run-bob
.run-bob-backup/
```

设计上预留扩展位:在 `src/commands/gitignore.rs` 用一个常量数组承载所有条目,未来新增瞬态产物只需追加一行。

```rust
const GITIGNORE_BLOCK_HEADER: &str = "# run-bob";
const GITIGNORE_ENTRIES: &[&str] = &[".run-bob-backup/"];
```

---

## 2. 行为(四种 case)

| Case | 文件状态 | 动作 | 输出标记 |
|---|---|---|---|
| A | `.gitignore` 不存在 | 创建文件,写入完整 block(header + entries,末尾换行) | `+ .gitignore (created, 1 entry)` |
| B | `.gitignore` 存在,无 `# run-bob` 区块 | **追加** block。若原文件最后一行非空,先补一个空行再写 block,使新 block 与上方内容之间至少有一个空行 | `↑ .gitignore (added 1 entry)` |
| C | `.gitignore` 存在,有 `# run-bob` 区块,**缺少**部分 entries | 在区块内部追加缺失 entries(放在区块末尾) | `↑ .gitignore (added N entries)` |
| D | `.gitignore` 存在,有 `# run-bob` 区块,所有 entries 齐全 | 不写文件,no-op | `✓ .gitignore (up to date)` |

### 2.1 「区块」的定义

- header 行的匹配采用**精确比较**(trim 两端空白后):字符串等于 `# run-bob` 才算 run-bob 区块开始。`#run-bob` / `## run-bob` / `# RUN-BOB` 等变体都**不识别**,会触发 Case B 在文件末尾追加规范区块。
- 区块从 `# run-bob` header 行开始,直到遇到下一个**空行** 或**另一个 comment header**(以 `#` 开头但不等于 `# run-bob` 的行)或**文件结尾**为止 —— 中间所有非空非注释行都视为 run-bob 的 entries。

### 2.2 幂等性

同样的 `(target, run-bob 二进制版本)` 组合,无论运行几次 `init`/`upgrade`,只要:
- `.gitignore` 上次执行后没被手工反向改动
- run-bob 二进制内嵌的 entries 列表没变

→ 第 2 次起永远命中 Case D,文件**字节级不变**。

### 2.3 不做的事

- **不删除** `# run-bob` 区块里用户手工额外加入的条目(尊重用户编辑);
- **不重排** 已存在的 entries;
- **不去重** —— 如果用户在 `.gitignore` 其他位置也写了 `.run-bob-backup/`,run-bob 不感知,也不清理,只确保自己的区块包含它;
- **不调用** `git` 命令(纯文件 I/O,gitignore 不需要 git 进程参与)。

---

## 3. CLI 接口

### 3.1 新增 flag

| 命令 | 新增 flag | 默认 | 语义 |
|---|---|---|---|
| `run-bob init` | `--no-gitignore` | **不**设置 → 启用 gitignore 维护 | 设置后跳过整个 gitignore 流程 |
| `run-bob upgrade` | `--no-gitignore` | **不**设置 → 启用 gitignore 维护 | 同上 |

flag 风格与现有 `--no-backup` 对齐(`upgrade` 已有此 pattern)。

### 3.2 输出位置

在所有 asset 写入完成 / summary 行打印**之前**,插入一个独立小节:

```
Updating .gitignore...
  ✓ .gitignore (up to date)
```

或:

```
Updating .gitignore...
  + .gitignore (created, 1 entry)
```

跳过时:

```
Updating .gitignore...
  → skipped: --no-gitignore
```

### 3.3 错误处理

- 写文件失败 → 报错并退出非零(沿用 `anyhow::Context`)。已写入的 asset 不回滚 —— 与现有 `init`/`upgrade` 失败语义一致。
- target 目录不存在 → 不会走到 gitignore 步骤(`init`/`upgrade` 入口的 `canonicalize` 会先失败)。

---

## 4. 模块设计

### 4.1 新模块 `src/commands/gitignore.rs`

```rust
use std::path::Path;
use anyhow::Result;

const GITIGNORE_BLOCK_HEADER: &str = "# run-bob";
const GITIGNORE_ENTRIES: &[&str] = &[".run-bob-backup/"];

pub enum GitignoreReport {
    Skipped,                  // --no-gitignore
    Created { entries: usize },
    Updated { added: usize }, // Case B + C 合并
    UpToDate,
}

pub fn apply(target: &Path, skip: bool) -> Result<GitignoreReport> { ... }

// 打印 reporter,供 init/upgrade 复用
pub fn print_report(report: &GitignoreReport) { ... }
```

`apply` 内部:
1. `skip == true` → 直接返回 `Skipped`;
2. 读 `target/.gitignore`(不存在 → Case A);
3. 解析定位 `# run-bob` 区块的起止行号(不存在 → Case B);
4. 在区块内枚举现存 entries,与 `GITIGNORE_ENTRIES` 求差集 —— 空集 → Case D;
5. 否则 → Case C(原文件区块内追加) 或 Case B(整体追加新区块)。

### 4.2 集成点

```rust
// init.rs 末尾,print_next_steps 之前:
let report = gitignore::apply(&target, no_gitignore)?;
println!();
println!("{}", "Updating .gitignore...".bold());
gitignore::print_report(&report);

// upgrade.rs 末尾,最后一行 summary 之前同样:
let report = gitignore::apply(&target, no_gitignore)?;
println!();
println!("{}", "Updating .gitignore...".bold());
gitignore::print_report(&report);
```

### 4.3 CLI 解析(`src/main.rs` 或 clap derive 位置)

为 `init` / `upgrade` 子命令分别添加 `--no-gitignore` 布尔 flag,默认 `false`,透传到 `run(...)`。

---

## 5. 测试

新增 `tests/gitignore_integration.rs`(或追加到现有 `tests/init_*.rs` / `tests/upgrade_*.rs`):

1. **A_creates_when_missing**:fresh dir 跑 `init` → `.gitignore` 被创建,包含 `# run-bob\n.run-bob-backup/\n`。
2. **B_appends_when_block_missing**:预置 `.gitignore` 内容为 `target/\n*.log\n`,跑 `init` → 原内容保留 + 末尾追加空行 + run-bob 区块。
3. **C_inserts_missing_entries**:预置 `.gitignore` 仅含 `# run-bob\n`(header 在但无 entries) → 跑 `init` → entries 被加在区块内。
4. **D_noop_when_up_to_date**:预置已完整 `.gitignore` → 跑 `init` 两次,文件字节级不变(用 `assert_eq!` 比对 before/after bytes)。
5. **flag_no_gitignore**:存在的 `.gitignore` 跑 `init --no-gitignore` → 文件未被修改;不存在 `.gitignore` 跑 → 仍不创建。
6. **upgrade_path**:对 `upgrade` 子命令重复 case A + D,确认两个命令都走同一逻辑。
7. **respects_user_extra_entries**:预置 `.gitignore` 含 `# run-bob\n.run-bob-backup/\nmy-local-cache/\n`(用户在 run-bob 区块里加了自己的条目),跑 `init` → 文件不变(`my-local-cache/` 不被删,`.run-bob-backup/` 也不重复)。

---

## 6. 未来扩展(YAGNI 不实现)

- **可配置 entry 列表**:目前 `GITIGNORE_ENTRIES` 是编译期常量。若未来出现更多瞬态产物(例:`.run-bob-cache/`),直接追加常量即可,无需 CLI 配置。
- **递归向上查找 git 根**:不做。语义和性能上得不偿失。
- **跨平台路径分隔符**:`.gitignore` 始终用 `/`(git 规范),无需特殊处理。

---

## 7. 验收

- [ ] `cargo test` 全绿,新增 7 个测试通过
- [ ] 手工跑 `cargo run -- init /tmp/run-bob-test` 验证 Case A
- [ ] 手工再跑一次 `init` 验证 Case D(`.gitignore` 字节不变)
- [ ] 手工预置不同 `.gitignore` 验证 Case B / C
- [ ] `--no-gitignore` 跳过且文案正确
- [ ] `run-bob upgrade` 走同一逻辑,输出对称
- [ ] README 增补一行说明(可选)

---

## 8. 不变量回顾

- run-bob 是**约束优先**的 harness:它管的是「让团队对齐架构 + 让本地工具不污染历史」。本特性属于后者。
- harness 核心产物(CLAUDE.md / ARCHITECTURE.md / skills / Java 骨架)永远进 git,这条线**不变**。
