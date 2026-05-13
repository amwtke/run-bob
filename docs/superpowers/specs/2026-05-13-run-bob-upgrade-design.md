# run-bob · `upgrade` 子命令设计

> 状态:设计已定稿,待用户审阅 → 转交 `superpowers:writing-plans` 落地。
> 日期:2026-05-13
> 适用仓库:`/Users/xiaojin/workshop/run-bob`
> 对照参考:[`amwtke/superpowers-to-trae`](https://github.com/amwtke/superpowers-to-trae) 的 `upgrade` 子命令(`--addons` / `--no-backup` 语义)
> 相关 spec:`docs/superpowers/specs/2026-05-08-run-bob-design.md`(v0.1.0 完整设计)

---

## 0. 目的与一句话总结

让用户在任意已 `run-bob init` 过的项目里执行 `run-bob upgrade`,把目标目录下的 **skills + 纯生成型模板** 对齐到当前 `run-bob` 二进制内嵌的版本,**不触碰用户自定义文件**,默认带备份,可 `--dry-run` 预览。

> 「有更新就重新运行这个命令更新目标文件夹下的 skills」—— 用户原话。

---

## 1. 背景与现状

### 1.1 现状

- run-bob 通过 `include_str!` 把所有模板编译进二进制(`src/assets.rs::HARNESS_ASSETS`)。**二进制本身就是模板的版本载体**,无独立版本元数据。
- 子命令仅 `init`(安装,`--force` 覆盖) 与 `status`(校验存在性)。
- 没有 addon 机制 —— run-bob 就是单一 harness。

### 1.2 对照 `superpowers-to-trae`

| 维度 | superpowers-to-trae | run-bob `upgrade` 决策 |
|---|---|---|
| 检测来源 | `Cargo.toml [package.metadata.upstream]` + 上游对比 | **嵌入内容字节级 diff**(更简单,无网络) |
| 覆盖范围 | skills + rules,保留 DOMAIN.md / BOB.md | **upgrade-safe 资产**,保留用户自定义文件 |
| 备份 | 默认开,`--no-backup` 跳过 | **同**:默认开,`--no-backup` 跳过 |
| addons | `--addons ddd,bob` | **不引入**(run-bob 无 addon) |
| AGENTS.md append-only | 是 | **不适用**(run-bob 无 AGENTS.md) |

### 1.3 关键约束

- **零网络依赖**:upgrade 不调 GitHub API、不下载二进制。语义是「让目标项目对齐**当前**二进制」,而非「检查上游有没有更新」。后者由 `install.sh` / `install.ps1` 一键覆盖二进制本身负责。
- **不引入新依赖**:不加 `sha2` / `reqwest` 等 crate。比较用 `String ==`。
- **`init` 与 `status` 行为不变**:upgrade 只新增,不修改它们的语义。
- **SSoT 单一**:`HARNESS_ASSETS` 仍是唯一资产清单;新增字段属于这张表的扩展,不开第二张表。
- **`superpowers-to-trae` 仅作 CLI 表面参考**:不导入、不移植、不镜像该仓库的任何代码 / 模板 / 资产。引用范围仅限 CLI 语义对照(flag 命名、"upgrade vs install-once 文件" 的取舍方向)。run-bob 的模板内容来源始终是本仓库 `src/templates/` + `include_str!`。

---

## 2. CLI 表面

```
run-bob upgrade [--dir <path>] [--dry-run] [--no-backup]
```

| flag | short | 默认 | 行为 |
|---|---|---|---|
| `--dir <path>` | `-d` | `.` | 目标项目目录(与 `init` / `status` 一致) |
| `--dry-run` | `-n` | off | 只 diff、不写文件、不创建备份 |
| `--no-backup` |  | off | 跳过备份,直接覆盖 |

不引入的 flag(明确决定):

- `--force` —— upgrade 本身就是「按 diff 来」,无需另一个开关。要覆盖 user-owned 文件请用 `init --force`。
- `--addons` —— run-bob 无 addon。
- `--backup-dir <path>` —— 备份目录固定为 `<target>/.run-bob-backup/<UTC-timestamp>/`,YAGNI。

`run-bob upgrade --help` 必须列出 `--dir / --dry-run / --no-backup` 三个 flag(测试断言)。

---

## 3. 资产分类(SSoT 扩展)

### 3.1 `Asset` 结构体新增字段

```rust
pub struct Asset {
    pub rel_path: &'static [&'static str],
    pub content: &'static str,
    pub category: Category,
    pub included_in_minimal: bool,
    pub upgrade_safe: bool,   // ← 新增
}
```

语义:**「upgrade 时是否可以无脑覆盖?」**

为什么单开字段而不从 `Category` 反推:`Category::HarnessDoc` 里既有 `README-RUN-BOB.md`(可覆盖)又有 `CLAUDE.md` / `ARCHITECTURE.md`(不可覆盖),单一维度不够用。新字段把判定明确化,避免隐式规则。

### 3.2 各资产取值

| `rel_path` | `category` | `upgrade_safe` | 理由 |
|---|---|---|---|
| `.claude/skills/bob-identify/SKILL.md` | Skill | **true** | 纯 harness,跟二进制版本对齐 |
| `.claude/skills/bob-onion/SKILL.md` | Skill | **true** | 同上 |
| `.claude/skills/bob-spec/SKILL.md` | Skill | **true** | 同上 |
| `README-RUN-BOB.md` | HarnessDoc | **true** | 纯说明文档,用户不该手改 |
| `src/main/java/com/example/shared/usecase/UseCase.java` | SharedJava | **true** | 通用接口,无项目特异性 |
| `src/main/java/com/example/shared/framework/transaction/TransactionalUseCaseDecorator.java` | SharedJava | **true** | 通用 decorator |
| `CLAUDE.md` | HarnessDoc | **false** | 用户有「技术栈约定」自定义段 |
| `ARCHITECTURE.md` | HarnessDoc | **false** | `/bob-onion` 维护的 SSoT |
| `src/test/java/architecture/CleanArchitectureTest.java` | ArchUnit | **false** | `FORBIDDEN_IN_INNER` 由 `/bob-onion` 回写 |

### 3.3 影响范围

| 子命令 | 是否读 `upgrade_safe` |
|---|---|
| `init` | 否(行为不变) |
| `status` | 否(行为不变) |
| `upgrade` | 是(只处理 `upgrade_safe == true` 的资产) |

### 3.4 测试漂移守卫

补一个集成测试:遍历 `HARNESS_ASSETS`,断言所有 `Category::Skill` 与 `Category::SharedJava` 的资产 `upgrade_safe == true`,所有 `Category::ArchUnit` 的资产 `upgrade_safe == false`。`HarnessDoc` 类别允许混合(README 可,CLAUDE/ARCHITECTURE 不可),逐项断言。

> 用意:未来加新资产时,如果忘了设这个字段,测试会失败提醒。

---

## 4. 检测算法

### 4.1 三态分类

对每个 `upgrade_safe == true` 的资产:

| 磁盘状态 | 与嵌入内容比较 | 状态 | 行为 |
|---|---|---|---|
| 文件不存在 | n/a | **MISSING** | 直接安装(像 init) |
| 文件存在 | 内容 == 嵌入 | **UP-TO-DATE** | 跳过 |
| 文件存在 | 内容 != 嵌入 | **OUTDATED** | 备份 + 覆盖 |

比较方式:`fs::read_to_string(path)?` 后做 `&str == &str`。

> 字节级判定。模板都是几 KB 文本;不引入 SHA256 / `sha2` crate。

读文件失败(权限 / 非法 UTF-8)直接 `bail!`,upgrade 不应在不可读的目标上继续。

### 4.2 user-owned 资产(`upgrade_safe == false`)

- upgrade **完全不读** 它们的磁盘内容,**不比对**,**不覆盖**。
- 在末尾的 summary 中列出文件名,提示用户「想覆盖请用 `init --force`」。

### 4.3 零变更短路

如果所有 upgrade-safe 资产都 UP-TO-DATE:

- 不创建备份目录
- 不进入 "Applying changes..." 阶段
- 打印 `✓ All upgrade-safe assets are up to date.`
- exit 0

---

## 5. 备份机制

### 5.1 路径约定

- 备份根:`<target>/.run-bob-backup/`
- 单次 upgrade 子目录:`<UTC-timestamp>/`,格式 `YYYYMMDDTHHMMSSZ`(例如 `20260513T103045Z`)
- 完整路径:`<target>/.run-bob-backup/20260513T103045Z/<原相对路径>`

例如 `.claude/skills/bob-identify/SKILL.md` 被覆盖前,备份至:

```
<target>/.run-bob-backup/20260513T103045Z/.claude/skills/bob-identify/SKILL.md
```

镜像原相对路径,便于 `cp -r` 回滚。

### 5.2 时机

- **只在有 OUTDATED 文件实际被覆盖时** 才创建 timestamp 目录(MISSING 不产生备份,因为没有原文件)
- 创建顺序:**先 backup 所有 OUTDATED 文件 → 再覆盖**,任何一步失败立即 `bail!`,避免「备份了但没覆盖」或「覆盖了但备份失败」的中间态

### 5.3 与 flag 的交互

| 场景 | 是否创建备份 |
|---|---|
| 默认 | 是,仅当有 OUTDATED |
| `--no-backup` | 否 |
| `--dry-run` | 否(无论是否 `--no-backup`) |

### 5.4 不做的事

- 不维护 retention / 清理策略 —— 用户用 `.gitignore` 或定期手动清(README 提示)
- 不写 manifest 文件 —— 文件结构本身就是 manifest

---

## 6. 输出格式

### 6.1 常态(有变更)

```
🛠 run-bob upgrade
  → target: /Users/.../api
  → mode:   default (backup enabled)

Checking upgrade-safe assets...
  ↑ .claude/skills/bob-identify/SKILL.md     (outdated)
  ✓ .claude/skills/bob-onion/SKILL.md        (up to date)
  ↑ .claude/skills/bob-spec/SKILL.md         (outdated)
  + README-RUN-BOB.md                        (missing — will install)
  ✓ src/main/java/com/example/shared/usecase/UseCase.java         (up to date)
  ✓ src/main/java/com/example/shared/framework/transaction/TransactionalUseCaseDecorator.java (up to date)

Applying changes...
  📦 backup: .run-bob-backup/20260513T103045Z/ (2 files)
  ✓ .claude/skills/bob-identify/SKILL.md     (updated)
  ✓ .claude/skills/bob-spec/SKILL.md         (updated)
  ✓ README-RUN-BOB.md                        (installed)

ℹ 3 user-owned files skipped (CLAUDE.md, ARCHITECTURE.md, src/test/java/architecture/CleanArchitectureTest.java).
  Use `run-bob init --force` if you need to overwrite them.

✓ upgrade complete. 2 updated, 1 installed, 3 up to date.
```

符号约定(沿用 `main.rs` 已有的 `success` / `info` / `warn` / `skip`):

| 符号 | 含义 |
|---|---|
| `✓` (绿) | up to date / 操作成功 |
| `↑` (黄) | outdated,将覆盖 |
| `+` (绿) | missing,将安装 |
| `📦` | 备份目录创建提示 |
| `ℹ` (蓝) | user-owned 文件跳过提示 |

### 6.2 零变更

```
🛠 run-bob upgrade
  → target: /Users/.../api

Checking upgrade-safe assets...
  ✓ .claude/skills/bob-identify/SKILL.md     (up to date)
  ... (全部 ✓)

ℹ 3 user-owned files skipped.

✓ All upgrade-safe assets are up to date.
```

### 6.3 `--dry-run`

将 "Applying changes..." 段替换为:

```
→ dry-run: no files would be written. Run without --dry-run to apply.
```

仍输出 user-owned skip 提示和 summary,但 summary 改为 `would update 2, would install 1`。

---

## 7. 实现拆解(代码层)

### 7.1 文件结构

新增:

- `src/commands/upgrade.rs`:命令入口 + 算法
- `tests/integration.rs`:新增 7 个测试用例(见 §8)

修改:

- `src/main.rs`:加 `Commands::Upgrade { dir, dry_run, no_backup }` 分支
- `src/commands/mod.rs`:加 `pub mod upgrade;`
- `src/assets.rs`:`Asset` 加 `upgrade_safe: bool`,逐资产填值

### 7.2 `upgrade.rs` 核心函数

伪代码:

```rust
pub fn run(target_dir: &str, dry_run: bool, no_backup: bool) -> Result<()> {
    let target = canonicalize(target_dir)?;
    print_header(&target, dry_run, no_backup);

    // 1. 分类
    let mut up_to_date = vec![];
    let mut outdated = vec![];     // (asset, current_content)
    let mut missing = vec![];

    for asset in HARNESS_ASSETS.iter().filter(|a| a.upgrade_safe) {
        let path = asset_path(&target, asset);
        if !path.exists() {
            missing.push(asset);
        } else {
            let current = fs::read_to_string(&path)?;
            if current == asset.content {
                up_to_date.push(asset);
            } else {
                outdated.push((asset, current));
            }
        }
        print_check_line(asset, &state);
    }

    // 2. 零变更短路
    if outdated.is_empty() && missing.is_empty() {
        print_user_owned_skip_note();
        print_all_up_to_date();
        return Ok(());
    }

    // 3. dry-run 短路
    if dry_run {
        print_dry_run_note();
        print_user_owned_skip_note();
        print_summary_dry(&outdated, &missing, &up_to_date);
        return Ok(());
    }

    // 4. 备份(可选)
    if !no_backup && !outdated.is_empty() {
        let backup_dir = make_backup_dir(&target)?;
        for (asset, content) in &outdated {
            write_backup(&backup_dir, asset, content)?;
        }
        print_backup_created(&backup_dir, outdated.len());
    }

    // 5. 覆盖 / 安装
    for (asset, _) in &outdated {
        install_asset(&target, asset, /*force=*/true)?;
        print_updated(asset);
    }
    for asset in &missing {
        install_asset(&target, asset, /*force=*/false)?;
        print_installed(asset);
    }

    // 6. user-owned skip 提示 + summary
    print_user_owned_skip_note();
    print_summary(&outdated, &missing, &up_to_date);
    Ok(())
}
```

### 7.3 复用 `init` 的写入逻辑

`init.rs::install_asset` 已经是「按 `force` 写文件」的纯函数。把它的 `install_asset` / `write_file` 提升到 `commands/mod.rs` 或 `assets.rs` 顶层(`pub(crate)`),让 `init` 与 `upgrade` 共用,避免逻辑漂移。

### 7.4 时间戳生成

无新依赖:用 `std::time::SystemTime::now()` + 手算 UTC `YYYYMMDDTHHMMSSZ`。具体细节属于实现层,plan 阶段决定;若过于繁琐再讨论是否引入 `chrono` / `time` crate(YAGNI 倾向:先手算)。

---

## 8. 测试计划

加到 `tests/integration.rs`(7 个用例 + 1 个 SSoT 守卫):

### 8.1 CLI 表面

1. **`upgrade_help_lists_flags`** —— `run-bob upgrade --help` 输出包含 `--dir` / `--dry-run` / `--no-backup`

### 8.2 检测正确性

2. **`upgrade_on_fresh_init_is_noop`** —— `init` 后立刻 `upgrade`,所有都 UP-TO-DATE,**无 `.run-bob-backup/` 目录**,stdout 含 `All upgrade-safe assets are up to date`
3. **`upgrade_replaces_stale_skill`** —— init 后手动把 `bob-identify/SKILL.md` 改成 `"stale"`,upgrade 后文件内容 == 嵌入内容,且 `.run-bob-backup/<ts>/.claude/skills/bob-identify/SKILL.md` 内容 == `"stale"`
4. **`upgrade_installs_missing_skill`** —— init 后删掉 `bob-spec/SKILL.md`,upgrade 后被装回,**不创建备份目录**(MISSING 不需要备份)

### 8.3 保护用户自定义

5. **`upgrade_skips_user_owned`** —— init 后把 `CLAUDE.md` / `ARCHITECTURE.md` / `CleanArchitectureTest.java` 改成已知字符串,upgrade 后内容不变,stdout 含 user-owned skip 提示

### 8.4 flag 行为

6. **`upgrade_dry_run_writes_nothing`** —— 制造 outdated,`upgrade --dry-run` 后磁盘内容仍是旧版,无备份目录,stdout 含 `dry-run`
7. **`upgrade_no_backup_skips_backup`** —— 制造 outdated,`upgrade --no-backup` 后文件被覆盖但 `.run-bob-backup/` 不存在

### 8.5 SSoT 漂移守卫

8. **`upgrade_safe_field_matches_category_policy`** —— 遍历 `HARNESS_ASSETS`,断言:
   - 所有 `Category::Skill` → `upgrade_safe == true`
   - 所有 `Category::SharedJava` → `upgrade_safe == true`
   - 所有 `Category::ArchUnit` → `upgrade_safe == false`
   - `Category::HarnessDoc` 中,`rel_path` 为 `["README-RUN-BOB.md"]` 时 `upgrade_safe == true`,`CLAUDE.md` / `ARCHITECTURE.md` 时 `false`

> 这是「未来加资产时忘了设字段」的守卫。`HarnessDoc` 类别允许混合,所以逐项断言。

### 8.6 测试运行

全部跑通才允许合并:`cargo test` 当前 16 个测试 + 新增 8 个 = **24 个全绿**。

---

## 9. 文档同步

`README.md` 的 "Update" 段当前只描述「重跑 install.sh 升级二进制」。需要补一段说明:

- **升级二进制**:重跑 `install.sh` / `install.ps1`(已有)
- **升级目标项目中的 skills**:在目标项目里跑 `run-bob upgrade`(新增段落)

不另外开一个 README-RUN-BOB.md 的章节 —— 那是给「被 init 过的项目」用户看的,而 upgrade 是开发者侧操作,放主 README 即可。

---

## 10. 风险与权衡

| 风险 | 缓解 |
|---|---|
| 用户改了 README-RUN-BOB.md(虽然不该改)→ upgrade 静默覆盖 | 默认备份兜底;README 显式说明该文件「不应手改」 |
| 备份目录 `.run-bob-backup/` 被 commit 到 git | 在 README "Update" 段建议加进 `.gitignore` |
| 时间戳碰撞(同一秒内两次 upgrade) | 极小概率;若发生,第二次会因目录已存在而 `bail!`,用户重试即可。不引入随机后缀(YAGNI) |
| `Asset` 字段越加越多 | 当前 4 个字段(`rel_path` / `content` / `category` / `included_in_minimal`)+ 1 个新字段(`upgrade_safe`)= 5 个,仍在可控范围。若未来需要再加,届时考虑用 bitflags |
| 用户跑 `run-bob upgrade` 时二进制本身是旧版 | 用户原话「如果有更新就重新运行这个命令」—— 这里的「更新」由 install.sh 重装二进制承担。README 在 "Update" 段把两件事的顺序写清楚(先升级二进制,再 `run-bob upgrade` 应用) |

---

## 11. 决策记录(用户答复留痕)

| 维度 | 决策 |
|---|---|
| 检测机制 | 内容字节级 diff(`String ==`),不联网,不存版本元数据 |
| 覆盖范围 | `upgrade_safe == true` 的资产:3 个 skills + README-RUN-BOB.md + 2 个 shared Java 文件 |
| 备份策略 | 默认开,`--no-backup` 跳过,`--dry-run` 不创建 |
| dry-run | 提供 |
| addons | 不引入 |
| force | 不引入 |

---

## 12. 转交

设计定稿后转交 `superpowers:writing-plans`,产出可执行的、按测试驱动顺序排好的实施计划。
