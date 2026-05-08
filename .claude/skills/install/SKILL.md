---
name: install
description: |
  触发条件:用户输入 /install,或说"安装 run-bob"、"编译并安装"、"装一下这个 CLI"、
  "把 run-bob 装到本地"、"cargo install 一下"。
  该 skill 只服务于当前仓库 (run-bob),负责端到端地:
    1. 探测并准备 Rust 工具链 (rustup / rustc / cargo)
    2. 在 release 模式下编译 run-bob
    3. 通过 `cargo install --path .` 将二进制安装到 ~/.cargo/bin/
    4. 验证 `run-bob --version` / `run-bob status --help` 可用
  不适用于其它 Rust 项目,也不负责部署到远端机器。
---

# Install Skill — 本地构建并安装 run-bob

## 触发
用户使用 `/install` 命令,或明确要求"编译并安装 run-bob"。

## 目标
把当前仓库根目录下的 Rust CLI (`Cargo.toml` 中 `name = "run-bob"`) 编译成 release 二进制并
安装到 `~/.cargo/bin/run-bob`,让用户在任意目录下都能调用 `run-bob init` / `run-bob status`。

## 前置检查

在任何动作前,先用一次并行调用完成环境体检:

```bash
rustc --version        # 工具链存在?
cargo --version        # cargo 可用?
which run-bob          # 是否已装过旧版?
pwd && ls Cargo.toml   # 确认在 run-bob 仓库根目录
```

判定规则:
- `rustc` / `cargo` 都有 → 跳到「编译」。
- 只有 `rustc` 缺失或版本 < 1.75 → 进入「准备工具链」。
- `which run-bob` 找到旧版 → 继续,`cargo install` 会原子替换。
- 不在仓库根目录(没有 `Cargo.toml` 且 `[package].name != "run-bob"`)→ 停下,让用户确认 `cd` 到正确目录。

## 步骤 1:准备 Rust 工具链 (仅在缺失时)

**不要**擅自下载脚本跑 `rustup-init.sh`——这是一次性写入用户 shell 配置的动作,blast radius 大。
遇到缺失时按如下顺序处理:

1. 先告诉用户"检测到没有 Rust 工具链",并**明确询问**是否允许安装。
2. 用户同意后,给出官方一行命令,让用户**自己**在终端执行(建议用 `! ` 前缀):

   ```
   ! curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain stable
   ```

3. 提醒用户安装完成后 `source "$HOME/.cargo/env"` 或重开终端,再让 skill 重新触发。

如果仅仅是 `rustc` 版本过低 (< 1.75):

```bash
rustup update stable
rustup default stable
```

本项目 `Cargo.toml` 固定了 `clap = "=4.5.4"`,目的就是兼容 Rust 1.75+;不要随意升级 clap。

## 步骤 2:编译 (release)

```bash
cargo build --release
```

- 必须在仓库根目录执行。
- 首次编译会拉依赖、构建,慢一些;失败时先看编译错误,不要盲目重试。
- 成功后产物位于 `./target/release/run-bob`。

## 步骤 3:安装到 ~/.cargo/bin/

```bash
cargo install --path .
```

- 这会再跑一次 release build (cargo 目前不能直接复用 step 2 的产物,属正常行为)。
- 安装路径默认为 `$CARGO_HOME/bin`,即 `~/.cargo/bin/run-bob`。
- 已存在的旧版会被覆盖,但不会影响用 `run-bob init` 安装过 harness 的其它项目目录。
- 若用户 PATH 不包含 `~/.cargo/bin`,安装完后要提醒他们把这一行加到 shell rc:
  ```
  export PATH="$HOME/.cargo/bin:$PATH"
  ```

## 步骤 4:验证

```bash
which run-bob
run-bob --version
run-bob --help
```

三条命令都要成功,且 `--version` 输出与 `Cargo.toml` 中 `[package].version` 一致。
任一失败,回头查 PATH 或 `cargo install` 的输出。

## 产出格式

完成后给用户一段简短总结,结构固定:

```
✓ rust: <rustc version>
✓ build: release OK (<duration>s)
✓ install: ~/.cargo/bin/run-bob (v<X.Y.Z>)

下一步:
  cd <某个新项目>
  run-bob init
```

不要贴完整编译日志。如果有 warning 且不影响使用,一句话提过即可。

## 边界

- **不要**修改 `Cargo.toml` 或 `src/`,这是 install skill,不是 build skill。
- **不要**运行 `cargo install --force` 除非用户明确要求——默认的覆盖已足够。
- **不要**往用户 shell rc 里写任何东西,PATH 配置交给用户自己做。
- **不要**跨目录安装(`--dir /somewhere/else`),坚持在当前仓库根目录操作。
- 不要试着用 `sudo`,`cargo install` 是用户级别安装,不需要 root。
