---
name: install
description: |
  在当前 run-bob 仓库中执行安全的源码安装。用户输入 /install 或 $install，或者提出“安装 run-bob”“编译并安装”“更新 run-bob”“重新安装这个 CLI”时使用。检查仓库和 Git 状态，通过仓库自带的 Rust 引导脚本完成 locked 构建、测试和安装，并验证已安装版本。仅适用于 run-bob 源码仓库，不用于预编译二进制或其他 Rust 项目。
---

# 安装 run-bob

从当前仓库源码安全地同步、构建、测试、安装并验证 `run-bob`。

## 宿主约定

- Claude Code 中的显式调用是 `/install`。
- Codex 中的显式调用是 `$install`。
- 自然语言请求也可触发本 skill。后续提示使用当前宿主的调用形式，不建议用户切换到另一宿主的形式。
- 用户显式调用 `/install` 或 `$install`，即授权在 Rust 缺失时由仓库 helper 自动运行 Rust 官方 rustup 安装程序。操作系统、网络代理、安全软件或 Visual C++ 安装提示仍须展示并由用户处理。

## 1. 验证仓库

先确认当前目录就是仓库根目录：

1. `Cargo.toml` 必须存在。
2. 其 `[package]` 下的 `name` 必须是 `run-bob`。
3. `scripts/bootstrap-rust.sh` 与 `scripts/bootstrap-rust.ps1` 必须都是当前源码树中的常规文件。

任一条件不满足就停止，报告实际目录，并让用户切换到正确的 run-bob 仓库根目录。不要从其他目录或另一 skill 根目录寻找 helper。

## 2. 安全同步 Git

若当前目录不是 Git 工作树，或没有名为 `origin` 的远端，跳过同步并明确说明将使用当前源码。

若有 `origin`：

1. 运行 `git status --short`。有任何输出即停止，列出改动并请用户先处理；不要自动 stash、reset 或 checkout。
2. 用 `git branch --show-current` 取得当前分支。结果为空表示 detached HEAD，应停止并请用户选择分支。
3. 工作树干净时，仅运行：

```bash
branch=$(git branch --show-current)
git pull --ff-only origin "$branch"
```

`--ff-only` 失败表示本地与远端无法快进或同步失败。停止并展示错误，让用户决定如何处理；不要改用 merge、rebase、reset、stash 或 checkout。

同步后再次验证仓库根目录和两个 helper 仍存在。

## 3. 通过源码 helper 构建、测试、安装

按当前平台只选择一个 helper，并依次执行三个阶段。每一阶段成功后才能进入下一阶段，任何失败都停止，不安装未经测试的二进制。

POSIX 系统：

```bash
./scripts/bootstrap-rust.sh --run-cargo build --release --locked
./scripts/bootstrap-rust.sh --run-cargo test --locked
./scripts/bootstrap-rust.sh --run-cargo install --locked --path .
```

Windows PowerShell：

```powershell
& .\scripts\bootstrap-rust.ps1 -RunCargo @('build','--release','--locked')
& .\scripts\bootstrap-rust.ps1 -RunCargo @('test','--locked')
& .\scripts\bootstrap-rust.ps1 -RunCargo @('install','--locked','--path','.')
```

只通过所选 helper 调用 Cargo，不在外层 shell 直接运行 Cargo，也不提供或运行独立的 Rust 安装命令。helper 的工具链策略是：

- 已有 Rust 满足 `Cargo.toml` 的 `rust-version` 时保持不变。
- Rust 完全缺失时才自动使用官方 rustup 安装程序。
- 旧版本若由 rustup 管理，只为本次构建安装并选择合适的 stable 工具链，不切换用户已有的全局默认工具链。
- 旧版本若不由 rustup 管理，停止并给出诊断，不替换它。
- 不修改 shell 启动文件、PATH 持久配置或任何工具链默认设置。

## 4. 用绝对路径验证

安装成功后按以下优先级确定安装根：

1. 非空的 `CARGO_INSTALL_ROOT`
2. 非空的 `CARGO_HOME`
3. POSIX 下用户主目录的 `.cargo`，Windows 下用户配置目录的 `.cargo`

将根目录规范化为绝对路径。POSIX 二进制是 `bin/run-bob`，Windows 二进制是 `bin/run-bob.exe`。先确认它是常规文件，再以这个绝对路径运行 `--version` 和 `--help`；不要依赖 PATH 中可能存在的旧版本。

同时读取 `Cargo.toml` 的 `[package].version`，要求绝对路径输出的 `run-bob X.Y.Z` 与其完全一致。若找不到文件、任一命令失败或版本不一致，停止并报告：

- 解析出的绝对安装路径；
- manifest 版本与实际版本；
- 失败阶段及关键 stderr；
- 建议用户检查 Cargo 安装根、文件权限或前一阶段输出后重试当前宿主的 `/install` 或 `$install`。

不要自动修改用户的 shell 启动文件。若安装目录不在 PATH，只说明绝对路径已经可用，并让用户自行决定是否配置 PATH。

## 5. 汇报

成功时简要报告 Git 同步状态、实际 Rust 版本、三个 locked 阶段均成功、绝对安装路径及已验证版本。不要硬编码测试数量，也不要粘贴完整构建日志。

## 边界

- 不修改 `Cargo.toml`、`src/` 或用户项目。
- 不使用 `sudo`、`cargo install --force`，不跳过测试。
- 不自动 stash、reset、checkout、merge 或 rebase。
- 不下载或执行 helper 之外的安装脚本。
- 不修改 PATH、shell 启动文件或 rustup 全局默认工具链。
