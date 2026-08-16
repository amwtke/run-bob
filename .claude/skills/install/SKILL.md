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
- 用户显式调用 `/install`、`$install`，或明确要求安装、更新或重新安装 run-bob，都授权完成整个源码安装；Rust 缺失时无需再次确认 Rust 自动引导。
- “可以安装吗”“会下载什么”“能否更新”等探索性或疑问式表达不构成授权。先回答问题，并在任何安装或网络变更之前取得明确确认。
- 获得授权后，仓库 helper 可以在 Rust 缺失时自动运行 Rust 官方 rustup 安装程序。操作系统、网络代理、安全软件或 Visual C++ 安装提示仍须展示并由用户处理。

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

按当前平台只选择一个 helper。在同一个 shell 或 PowerShell 进程中先确定唯一安装根，再依次执行三个阶段和验证。每一阶段成功后才能进入下一阶段，任何失败都停止，不安装未经测试的二进制。

POSIX 系统：

以下整段在子 shell 中执行；`set -eu` 让失败立即终止后续阶段，导出的安装根在子 shell 结束后不会泄漏到父 shell。

```bash
(
    set -eu

    if [ -n "${CARGO_INSTALL_ROOT:-}" ]; then
        install_root=$CARGO_INSTALL_ROOT
    elif [ -n "${CARGO_HOME:-}" ]; then
        install_root=$CARGO_HOME
    elif [ -n "${HOME:-}" ]; then
        install_root=$HOME/.cargo
    else
        printf '%s\n' 'error: HOME is required when Cargo install roots are unset' >&2
        exit 1
    fi
    mkdir -p "$install_root"
    install_root=$(CDPATH= cd -P "$install_root" && pwd)
    export CARGO_INSTALL_ROOT="$install_root"

    ./scripts/bootstrap-rust.sh --run-cargo build --release --locked
    ./scripts/bootstrap-rust.sh --run-cargo test --locked
    ./scripts/bootstrap-rust.sh --run-cargo install --locked --path .

    manifest_version=$(awk '
        BEGIN { in_package = 0; found = 0 }
        /^[[:space:]]*\[package\][[:space:]]*$/ {
            in_package = 1
            next
        }
        /^[[:space:]]*\[/ {
            if (in_package) {
                in_package = 0
            }
            next
        }
        in_package && /^[[:space:]]*version[[:space:]]*=/ {
            if ($0 !~ /^[[:space:]]*version[[:space:]]*=[[:space:]]*"[0-9][0-9]*\.[0-9][0-9]*\.[0-9][0-9]*"[[:space:]]*$/) {
                exit 2
            }
            value = $0
            sub(/^[[:space:]]*version[[:space:]]*=[[:space:]]*"/, "", value)
            sub(/"[[:space:]]*$/, "", value)
            print value
            found++
        }
        END {
            if (found != 1) {
                exit 3
            }
        }
    ' Cargo.toml) || {
        printf '%s\n' 'error: Cargo.toml [package].version must appear exactly once as X.Y.Z' >&2
        exit 1
    }
    expected_version="run-bob $manifest_version"
    run_bob_bin="$install_root/bin/run-bob"
    if [ ! -f "$run_bob_bin" ]; then
        printf 'error: installed binary is missing\npath: %s\nexpected: %s\nactual: %s\n' \
            "$run_bob_bin" "$expected_version" 'missing' >&2
        exit 1
    fi
    installed_version=
    if ! installed_version=$("$run_bob_bin" --version); then
        printf 'error: installed binary --version failed\npath: %s\nexpected: %s\nactual: %s\n' \
            "$run_bob_bin" "$expected_version" "$installed_version" >&2
        exit 1
    fi
    if [ "$installed_version" != "$expected_version" ]; then
        printf 'error: installed binary version mismatch\npath: %s\nexpected: %s\nactual: %s\n' \
            "$run_bob_bin" "$expected_version" "$installed_version" >&2
        exit 1
    fi
    if ! "$run_bob_bin" --help >/dev/null; then
        printf 'error: installed binary --help failed\npath: %s\nexpected: %s\nactual: %s\n' \
            "$run_bob_bin" "$expected_version" "$installed_version" >&2
        exit 1
    fi
    printf 'RUN_BOB_BIN=%s\nRUN_BOB_VERSION=%s\n' "$run_bob_bin" "$manifest_version"
)
```

Windows PowerShell：

```powershell
$hadCargoInstallRoot = Test-Path Env:CARGO_INSTALL_ROOT
$previousCargoInstallRoot = $env:CARGO_INSTALL_ROOT
try {
    if (-not [string]::IsNullOrWhiteSpace($env:CARGO_INSTALL_ROOT)) {
        $installRoot = $env:CARGO_INSTALL_ROOT
    }
    elseif (-not [string]::IsNullOrWhiteSpace($env:CARGO_HOME)) {
        $installRoot = $env:CARGO_HOME
    }
    elseif (-not [string]::IsNullOrWhiteSpace($env:USERPROFILE)) {
        $installRoot = Join-Path $env:USERPROFILE '.cargo'
    }
    else {
        throw 'USERPROFILE is required when Cargo install roots are unset'
    }
    $installRoot = [System.IO.Path]::GetFullPath($installRoot)
    $env:CARGO_INSTALL_ROOT = $installRoot

    & .\scripts\bootstrap-rust.ps1 -RunCargo @('build','--release','--locked')
    if ($LASTEXITCODE -ne 0) { throw 'run-bob build stage failed' }
    & .\scripts\bootstrap-rust.ps1 -RunCargo @('test','--locked')
    if ($LASTEXITCODE -ne 0) { throw 'run-bob test stage failed' }
    & .\scripts\bootstrap-rust.ps1 -RunCargo @('install','--locked','--path','.')
    if ($LASTEXITCODE -ne 0) { throw 'run-bob install stage failed' }

    $inPackage = $false
    $manifestVersion = $null
    $manifestVersionCount = 0
    foreach ($manifestLine in Get-Content -LiteralPath .\Cargo.toml -ErrorAction Stop) {
        if ($manifestLine -match '^\s*\[package\]\s*$') {
            $inPackage = $true
            continue
        }
        if ($inPackage -and $manifestLine -match '^\s*\[') {
            $inPackage = $false
            continue
        }
        if ($inPackage -and $manifestLine -match '^\s*version\s*=') {
            if ($manifestLine -notmatch '^\s*version\s*=\s*"([0-9]+\.[0-9]+\.[0-9]+)"\s*$') {
                throw 'Cargo.toml [package].version must be exactly X.Y.Z'
            }
            $manifestVersion = $Matches[1]
            $manifestVersionCount++
        }
    }
    if ($manifestVersionCount -ne 1) {
        throw 'Cargo.toml [package].version must appear exactly once as X.Y.Z'
    }
    $expectedVersion = "run-bob $manifestVersion"
    $runBobBinary = Join-Path $installRoot 'bin\run-bob.exe'
    if (-not (Test-Path -LiteralPath $runBobBinary -PathType Leaf)) {
        throw "installed binary is missing: path=$runBobBinary expected=$expectedVersion actual=missing"
    }
    $installedVersionLines = @(& $runBobBinary --version)
    if ($LASTEXITCODE -ne 0) {
        $installedVersion = $installedVersionLines -join [Environment]::NewLine
        throw "installed binary --version failed: path=$runBobBinary expected=$expectedVersion actual=$installedVersion"
    }
    $installedVersion = $installedVersionLines -join [Environment]::NewLine
    if ($installedVersion -cne $expectedVersion) {
        throw "installed binary version mismatch: path=$runBobBinary expected=$expectedVersion actual=$installedVersion"
    }
    $null = & $runBobBinary --help
    if ($LASTEXITCODE -ne 0) {
        throw "installed binary --help failed: path=$runBobBinary expected=$expectedVersion actual=$installedVersion"
    }
    Write-Output "RUN_BOB_BIN=$runBobBinary"
    Write-Output "RUN_BOB_VERSION=$manifestVersion"
}
finally {
    if ($hadCargoInstallRoot) {
        $env:CARGO_INSTALL_ROOT = $previousCargoInstallRoot
    }
    else {
        Remove-Item Env:CARGO_INSTALL_ROOT -ErrorAction SilentlyContinue
    }
}
```

优先使用非空的 `CARGO_INSTALL_ROOT`，其次是非空的 `CARGO_HOME`，最后才是 POSIX 的 `$HOME/.cargo` 或 Windows 的 `$env:USERPROFILE\.cargo`；缺少所需用户目录变量时停止。先把选择结果规范化为绝对路径，再用导出的 `CARGO_INSTALL_ROOT` 固定当前进程及其 helper 子进程。这个环境变量有意覆盖 Cargo 配置中的 `install.root`，保证实际安装位置与后续验证位置不会分叉。

只通过所选 helper 调用 Cargo，不在外层 shell 直接运行 Cargo，也不提供或运行独立的 Rust 安装命令。helper 的工具链策略是：

- 已有 Rust 满足 `Cargo.toml` 的 `rust-version` 时保持不变。
- Rust 完全缺失时才自动使用官方 rustup 安装程序。
- 旧版本若由 rustup 管理，只为本次构建安装并选择合适的 stable 工具链，不切换用户已有的全局默认工具链。
- 旧版本若不由 rustup 管理，停止并给出诊断，不替换它。
- 不修改 shell 启动文件、PATH 持久配置或任何工具链默认设置。

## 4. 用绝对路径验证

验证已在代码块作用域内完成：限定读取 Cargo.toml 的 [package].version，以绝对二进制路径执行 --version 和 --help，并严格要求输出等于 run-bob X.Y.Z。不要在作用域结束后重新读取环境变量或重新计算优先级，也不要复用局部变量或依赖 PATH 中可能存在的旧版本。

只有文件存在、两条命令成功且版本严格相等时，代码块才输出 RUN_BOB_BIN 和 RUN_BOB_VERSION。失败时停止并报告：

- 解析出的绝对安装路径；
- manifest 版本与实际版本；
- 失败阶段及关键 stderr；
- 建议用户检查 Cargo 安装根、文件权限或前一阶段输出；Claude Code 中重试 `/install`，Codex 中重试 `$install`。

不要自动修改用户的 shell 启动文件。若安装目录不在 PATH，只说明绝对路径已经可用，并让用户自行决定是否配置 PATH。

## 5. 汇报

宿主依据已输出的 RUN_BOB_BIN 和 RUN_BOB_VERSION 记录汇报。成功时简要报告 Git 同步状态、Rust 工具链验证通过、三个 locked 阶段均成功、绝对安装路径及已验证版本。不要硬编码测试数量，也不要粘贴完整构建日志。

## 边界

- 不修改 `Cargo.toml`、`src/` 或用户项目。
- 不使用 `sudo`、`cargo install --force`，不跳过测试。
- 不自动 stash、reset、checkout、merge 或 rebase。
- 不下载或执行 helper 之外的安装脚本。
- 不修改 PATH、shell 启动文件或 rustup 全局默认工具链。
