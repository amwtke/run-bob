#[test]
fn bootstrap_rust_powershell_has_safe_equivalent_contract() {
    let script_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("scripts")
        .join("bootstrap-rust.ps1");
    let script = std::fs::read_to_string(&script_path)
        .unwrap_or_else(|error| panic!("read {}: {error}", script_path.display()));
    let script = script.replace("\r\n", "\n");

    for required in [
        "[CmdletBinding(PositionalBinding = $false)]",
        "https://static.rust-lang.org/rustup/dist/x86_64-pc-windows-msvc/rustup-init.exe",
        "https://static.rust-lang.org/rustup/dist/aarch64-pc-windows-msvc/rustup-init.exe",
        "--profile",
        "minimal",
        "--default-toolchain",
        "stable",
        "--no-modify-path",
        "[string[]] $RunCargo",
        "Invoke-RunBobExternalProcess",
        "rustc",
        "cargo",
        "--version",
        "[version]",
        "Get-RunBobCommandPath",
        "Get-RunBobArchitecture",
        "Invoke-WebRequest",
        "[guid]::NewGuid()",
        "finally",
    ] {
        assert!(
            script.contains(required),
            "PowerShell bootstrap is missing required contract marker {required:?}"
        );
    }

    let lowered = script.to_ascii_lowercase();
    for forbidden in [
        "rustup default",
        "setenvironmentvariable",
        "read-host",
        "$profile",
        "add-content",
        "$env:path =",
        "cargo build",
        "cargo test",
        "cargo install",
        "run-bob --version",
        "rustup_dist_server",
        "rustup_update_root",
    ] {
        assert!(
            !lowered.contains(forbidden),
            "PowerShell bootstrap contains forbidden behavior {forbidden:?}"
        );
    }

    assert!(
        lowered
            .contains("invoke-runbobexternalprocess -filepath $cargopath -argumentlist $runcargo"),
        "PowerShell bootstrap must forward RunCargo unchanged to direct cargo"
    );
    assert!(
        script.contains("@('run', 'stable', 'cargo') + $RunCargo"),
        "PowerShell bootstrap must append exact RunCargo elements after rustup's fixed prefix"
    );
    assert!(
        script.contains(
            "'-y', '--profile', 'minimal', '--default-toolchain', 'stable', '--no-modify-path'",
        ),
        "the official installer arguments must be one exact contiguous safe sequence"
    );

    let postcheck_call = concat!(
        "Assert-RunBobSelectedToolchain -Mode $mode -RequiredVersion $requiredVersion `\n",
        "        -RustcPath $rustcPath -CargoPath $cargoPath -RustupPath $rustupPath",
    );
    let postcheck_position = script
        .find(postcheck_call)
        .expect("the selected toolchain must be post-checked with all selected paths");
    assert_eq!(
        script.matches(postcheck_call).count(),
        1,
        "the exact selected-toolchain post-check invocation must occur once"
    );
    let selection_position = script[..postcheck_position]
        .rfind("$mode = '")
        .expect("a toolchain mode must be selected before its post-check");
    let cargo_gate_position = script[postcheck_position..]
        .find("if ($RunCargoSpecified)")
        .map(|offset| postcheck_position + offset)
        .expect("cargo execution must remain behind the RunCargo gate");
    assert!(
        selection_position < postcheck_position && postcheck_position < cargo_gate_position,
        "post-check must occur after selection and before any requested cargo execution"
    );

    for required_postcheck in [
        "$rustcVersion = Get-RunBobToolVersion -ToolPath $RustcPath -ToolName rustc",
        "$cargoVersion = Get-RunBobToolVersion -ToolPath $CargoPath -ToolName cargo",
        "$rustcVersion = Get-RunBobToolVersion -ToolPath $RustupPath -ToolName rustc -RustupPath $RustupPath",
        "$cargoVersion = Get-RunBobToolVersion -ToolPath $RustupPath -ToolName cargo -RustupPath $RustupPath",
        "Test-RunBobVersionAtLeast -Actual $rustcVersion -Required $RequiredVersion",
        "Test-RunBobVersionAtLeast -Actual $cargoVersion -Required $RequiredVersion",
    ] {
        assert!(
            script.contains(required_postcheck),
            "PowerShell bootstrap is missing post-check behavior {required_postcheck:?}"
        );
    }
    assert_eq!(
        script
            .matches(
                "https://static.rust-lang.org/rustup/dist/x86_64-pc-windows-msvc/rustup-init.exe",
            )
            .count(),
        1,
        "the X64 production endpoint must be one fixed constant"
    );
    assert_eq!(
        script
            .matches(
                "https://static.rust-lang.org/rustup/dist/aarch64-pc-windows-msvc/rustup-init.exe",
            )
            .count(),
        1,
        "the Arm64 production endpoint must be one fixed constant"
    );
}

#[test]
fn ci_uses_stable_rust_action_with_explicit_msrv_toolchain() {
    let workflow_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join(".github")
        .join("workflows")
        .join("ci.yml");
    let workflow = std::fs::read_to_string(&workflow_path)
        .unwrap_or_else(|error| panic!("read {}: {error}", workflow_path.display()));
    let workflow = workflow.replace("\r\n", "\n");

    assert!(
        workflow.contains(concat!(
            "      - name: Install Rust 1.75\n",
            "        uses: dtolnay/rust-toolchain@stable\n",
            "        with:\n",
            "          toolchain: 1.75\n",
        )),
        "MSRV CI must configure Rust 1.75 through the stable rust-toolchain action"
    );
    assert!(
        !workflow.contains("uses: dtolnay/rust-toolchain@1.75"),
        "MSRV must not use a dynamic action ref for the Rust version"
    );
}

#[cfg(unix)]
use std::fs;
#[cfg(unix)]
use std::os::unix::fs::{symlink, PermissionsExt};
#[cfg(unix)]
use std::path::{Path, PathBuf};
#[cfg(unix)]
use std::process::{Command, Output};

#[cfg(unix)]
struct Sandbox {
    _temp: tempfile::TempDir,
    root: PathBuf,
    mock_bin: PathBuf,
    utility_bin: PathBuf,
    home: PathBuf,
    cargo_home: PathBuf,
    rustup_home: PathBuf,
    log: PathBuf,
}

#[cfg(unix)]
impl Sandbox {
    fn new() -> Self {
        let temp = tempfile::tempdir().expect("create bootstrap sandbox");
        let root = temp.path().to_path_buf();
        let mock_bin = root.join("mock bin");
        let utility_bin = root.join("basic utilities");
        let home = root.join("home with spaces");
        let cargo_home = root.join("cargo home");
        let rustup_home = root.join("rustup home");
        let log = root.join("commands.log");

        for directory in [&mock_bin, &utility_bin, &home, &cargo_home, &rustup_home] {
            fs::create_dir_all(directory).expect("create isolated directory");
        }
        for (name, candidates) in [
            ("sh", &["/bin/sh", "/usr/bin/sh"][..]),
            ("awk", &["/usr/bin/awk", "/bin/awk"][..]),
            ("dirname", &["/usr/bin/dirname", "/bin/dirname"][..]),
            ("mktemp", &["/usr/bin/mktemp", "/bin/mktemp"][..]),
            ("rm", &["/bin/rm", "/usr/bin/rm"][..]),
            ("cat", &["/bin/cat", "/usr/bin/cat"][..]),
            ("mkdir", &["/bin/mkdir", "/usr/bin/mkdir"][..]),
            ("chmod", &["/bin/chmod", "/usr/bin/chmod"][..]),
            ("pwd", &["/bin/pwd", "/usr/bin/pwd"][..]),
        ] {
            let source = candidates
                .iter()
                .map(Path::new)
                .find(|candidate| candidate.is_file())
                .unwrap_or_else(|| panic!("required basic utility {name} is unavailable"));
            symlink(source, utility_bin.join(name)).expect("link isolated basic utility");
        }

        Self {
            _temp: temp,
            root,
            mock_bin,
            utility_bin,
            home,
            cargo_home,
            rustup_home,
            log,
        }
    }

    fn mock(&self, name: &str, body: &str) {
        let path = self.mock_bin.join(name);
        let script = format!("#!/bin/sh\nset -eu\n{body}\n");
        fs::write(&path, script).expect("write command mock");
        let mut permissions = fs::metadata(&path).expect("mock metadata").permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(path, permissions).expect("make command mock executable");
    }

    fn fail_curl(&self) {
        self.mock(
            "curl",
            r#"printf 'curl' >> "$RUN_BOB_TEST_LOG"
for arg in "$@"; do printf ' <%s>' "$arg" >> "$RUN_BOB_TEST_LOG"; done
printf '\n' >> "$RUN_BOB_TEST_LOG"
exit 97"#,
        );
    }

    fn rustc(&self, version: &str, sysroot: Option<&Path>) {
        let sysroot_branch = match sysroot {
            Some(path) => format!(
                "if [ \"$#\" -eq 2 ] && [ \"$1\" = \"--print\" ] && [ \"$2\" = \"sysroot\" ]; then\n  printf '%s\\n' '{}'\n  exit 0\nfi\n",
                path.display()
            ),
            None => String::new(),
        };
        self.mock(
            "rustc",
            &format!(
                r#"if [ "$#" -eq 1 ] && [ "$1" = "--version" ]; then
  printf '%s\n' 'rustc {version} (mock)'
  exit 0
fi
{sysroot_branch}exit 96"#
            ),
        );
    }

    fn cargo(&self, version: &str) {
        self.mock(
            "cargo",
            &format!(
                r#"if [ "$#" -eq 1 ] && [ "$1" = "--version" ]; then
  printf '%s\n' 'cargo {version} (mock)'
  exit 0
fi
printf 'cargo' >> "$RUN_BOB_TEST_LOG"
for arg in "$@"; do printf ' <%s>' "$arg" >> "$RUN_BOB_TEST_LOG"; done
printf '\n' >> "$RUN_BOB_TEST_LOG""#
            ),
        );
    }

    fn rustup(&self, rustc_version: &str, cargo_version: Option<&str>, which_rustc: &Path) {
        self.mock(
            "rustup",
            &rustup_mock_body(rustc_version, cargo_version, which_rustc),
        );
    }

    fn cargo_home_mock(&self, name: &str, body: &str) {
        let path = self.cargo_home.join("bin").join(name);
        fs::create_dir_all(path.parent().expect("cargo-home mock parent"))
            .expect("create cargo-home bin");
        let script = format!("#!/bin/sh\nset -eu\n{body}\n");
        fs::write(&path, script).expect("write cargo-home command mock");
        let mut permissions = fs::metadata(&path)
            .expect("cargo-home mock metadata")
            .permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(path, permissions).expect("make cargo-home command mock executable");
    }

    fn cargo_home_rustc(&self, version: &str) {
        self.cargo_home_mock(
            "rustc",
            &format!(
                r#"if [ "$#" -eq 1 ] && [ "$1" = "--version" ]; then
  printf '%s\n' 'rustc {version} (mock)'
  exit 0
fi
exit 96"#
            ),
        );
    }

    fn cargo_home_cargo(&self, version: &str) {
        self.cargo_home_mock(
            "cargo",
            &format!(
                r#"if [ "$#" -eq 1 ] && [ "$1" = "--version" ]; then
  printf '%s\n' 'cargo {version} (mock)'
  exit 0
fi
printf 'cargo' >> "$RUN_BOB_TEST_LOG"
for arg in "$@"; do printf ' <%s>' "$arg" >> "$RUN_BOB_TEST_LOG"; done
printf '\n' >> "$RUN_BOB_TEST_LOG""#
            ),
        );
    }

    fn cargo_home_rustup(
        &self,
        rustc_version: &str,
        cargo_version: Option<&str>,
        which_rustc: &Path,
    ) {
        self.cargo_home_mock(
            "rustup",
            &rustup_mock_body(rustc_version, cargo_version, which_rustc),
        );
    }

    fn download_curl(&self, installer_body: &str) {
        let template = r#"printf 'curl' >> "$RUN_BOB_TEST_LOG"
for arg in "$@"; do printf ' <%s>' "$arg" >> "$RUN_BOB_TEST_LOG"; done
printf '\n' >> "$RUN_BOB_TEST_LOG"
[ "$#" -eq 7 ] || exit 98
[ "$1" = "--proto" ] || exit 98
[ "$2" = "=https" ] || exit 98
[ "$3" = "--tlsv1.2" ] || exit 98
[ "$4" = "-sSf" ] || exit 98
[ "$5" = "https://sh.rustup.rs" ] || exit 98
[ "$6" = "-o" ] || exit 98
case "$7" in */run-bob-rust.*/rustup-init.sh) ;; *) exit 98 ;; esac
cat > "$7" <<'RUN_BOB_INSTALLER'
#!/bin/sh
set -eu
__INSTALLER_BODY__
RUN_BOB_INSTALLER"#;
        self.mock(
            "curl",
            &template.replace("__INSTALLER_BODY__", installer_body),
        );
    }

    fn run(&self, arguments: &[&str]) -> Output {
        self.run_script_with_homes(
            &bootstrap_script(),
            arguments,
            Some(&self.home),
            Some(&self.cargo_home),
        )
    }

    fn run_script(&self, script: &Path, arguments: &[&str]) -> Output {
        self.run_script_with_homes(script, arguments, Some(&self.home), Some(&self.cargo_home))
    }

    fn run_script_with_homes(
        &self,
        script: &Path,
        arguments: &[&str],
        home: Option<&Path>,
        cargo_home: Option<&Path>,
    ) -> Output {
        let path = format!("{}:{}", self.mock_bin.display(), self.utility_bin.display());
        let mut command = Command::new(script);
        command
            .args(arguments)
            .env_clear()
            .env("PATH", path)
            .env("RUSTUP_HOME", &self.rustup_home)
            .env("RUN_BOB_TEST_LOG", &self.log)
            .env("TMPDIR", &self.root);
        if let Some(home) = home {
            command.env("HOME", home);
        }
        if let Some(cargo_home) = cargo_home {
            command.env("CARGO_HOME", cargo_home);
        }
        command.output().expect("execute POSIX Rust bootstrap")
    }

    fn assert_rust_tools_are_omitted_from_isolated_path(&self) {
        let path = format!("{}:{}", self.mock_bin.display(), self.utility_bin.display());
        let output = Command::new(self.utility_bin.join("sh"))
            .arg("-c")
            .arg(
                "for tool in rustc cargo rustup; do if command -v \"$tool\" >/dev/null 2>&1; then exit 91; fi; done; exit 0",
            )
            .env_clear()
            .env("PATH", path)
            .output()
            .expect("probe isolated command path");
        assert!(output.status.success(), "{}", stdout_stderr(&output));
    }

    fn log(&self) -> String {
        fs::read_to_string(&self.log).unwrap_or_default()
    }
}

#[cfg(unix)]
fn bootstrap_script() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("scripts")
        .join("bootstrap-rust.sh")
}

#[cfg(unix)]
fn stdout_stderr(output: &Output) -> String {
    format!(
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )
}

#[cfg(unix)]
fn rustup_mock_body(
    rustc_version: &str,
    cargo_version: Option<&str>,
    which_rustc: &Path,
) -> String {
    let cargo_branch = match cargo_version {
        Some(version) => format!(
            r#"if [ "$#" -eq 4 ] && [ "$1" = "run" ] && [ "$2" = "stable" ] && [ "$3" = "cargo" ] && [ "$4" = "--version" ]; then
  printf '%s\n' 'cargo {version} (mock)'
  exit 0
fi
if [ "$#" -ge 3 ] && [ "$1" = "run" ] && [ "$2" = "stable" ] && [ "$3" = "cargo" ]; then
  exit 0
fi
"#
        ),
        None => String::new(),
    };
    format!(
        r#"printf 'rustup' >> "$RUN_BOB_TEST_LOG"
for arg in "$@"; do printf ' <%s>' "$arg" >> "$RUN_BOB_TEST_LOG"; done
printf '\n' >> "$RUN_BOB_TEST_LOG"
if [ "$#" -eq 4 ] && [ "$1" = "toolchain" ] && [ "$2" = "install" ] && [ "$3" = "stable" ] && [ "$4" = "--profile" ]; then
  exit 95
fi
if [ "$#" -eq 5 ] && [ "$1" = "toolchain" ] && [ "$2" = "install" ] && [ "$3" = "stable" ] && [ "$4" = "--profile" ] && [ "$5" = "minimal" ]; then
  exit 0
fi
if [ "$#" -eq 2 ] && [ "$1" = "which" ] && [ "$2" = "rustc" ]; then
  printf '%s\n' '{}'
  exit 0
fi
if [ "$#" -eq 4 ] && [ "$1" = "run" ] && [ "$2" = "stable" ] && [ "$3" = "rustc" ] && [ "$4" = "--version" ]; then
  printf '%s\n' 'rustc {rustc_version} (mock)'
  exit 0
fi
{cargo_branch}exit 94"#,
        which_rustc.display()
    )
}

#[cfg(unix)]
fn installer_that_creates_rustup(
    rustc_version: &str,
    cargo_version: Option<&str>,
    which_rustc: &Path,
) -> String {
    let rustup_body = rustup_mock_body(rustc_version, cargo_version, which_rustc);
    format!(
        r#"printf 'installer' >> "$RUN_BOB_TEST_LOG"
for arg in "$@"; do printf ' <%s>' "$arg" >> "$RUN_BOB_TEST_LOG"; done
printf '\n' >> "$RUN_BOB_TEST_LOG"
mkdir -p "$CARGO_HOME/bin"
cat > "$CARGO_HOME/bin/rustup" <<'RUN_BOB_RUSTUP'
#!/bin/sh
set -eu
{rustup_body}
RUN_BOB_RUSTUP
chmod 755 "$CARGO_HOME/bin/rustup""#
    )
}

#[cfg(unix)]
fn installer_that_creates_tool_proxies(
    rustc_version: &str,
    cargo_version: &str,
    which_rustc: &Path,
) -> String {
    let rustup_body = rustup_mock_body(rustc_version, Some(cargo_version), which_rustc);
    format!(
        r#"printf 'installer' >> "$RUN_BOB_TEST_LOG"
for arg in "$@"; do printf ' <%s>' "$arg" >> "$RUN_BOB_TEST_LOG"; done
printf '\n' >> "$RUN_BOB_TEST_LOG"
mkdir -p "$CARGO_HOME/bin"
cat > "$CARGO_HOME/bin/rustup" <<'RUN_BOB_RUSTUP'
#!/bin/sh
set -eu
{rustup_body}
RUN_BOB_RUSTUP
cat > "$CARGO_HOME/bin/rustc" <<'RUN_BOB_RUSTC'
#!/bin/sh
set -eu
if [ "$#" -eq 1 ] && [ "$1" = "--version" ]; then
  printf '%s\n' 'rustc {rustc_version} (mock)'
  exit 0
fi
exit 96
RUN_BOB_RUSTC
cat > "$CARGO_HOME/bin/cargo" <<'RUN_BOB_CARGO'
#!/bin/sh
set -eu
if [ "$#" -eq 1 ] && [ "$1" = "--version" ]; then
  printf '%s\n' 'cargo {cargo_version} (mock)'
  exit 0
fi
printf 'cargo' >> "$RUN_BOB_TEST_LOG"
for arg in "$@"; do printf ' <%s>' "$arg" >> "$RUN_BOB_TEST_LOG"; done
printf '\n' >> "$RUN_BOB_TEST_LOG"
RUN_BOB_CARGO
chmod 755 "$CARGO_HOME/bin/rustup" "$CARGO_HOME/bin/rustc" "$CARGO_HOME/bin/cargo""#
    )
}

#[cfg(unix)]
#[test]
fn bootstrap_rust_posix_uses_supported_toolchain_and_forwards_cargo_args() {
    let sandbox = Sandbox::new();
    sandbox.rustc("1.75.0", None);
    sandbox.cargo("1.75.0");
    sandbox.fail_curl();

    let output = sandbox.run(&["--run-cargo", "check", "--locked", "--all-targets"]);

    assert!(output.status.success(), "{}", stdout_stderr(&output));
    assert_eq!(sandbox.log(), "cargo <check> <--locked> <--all-targets>\n");
}

#[cfg(unix)]
#[test]
fn bootstrap_rust_posix_uses_sufficient_cargo_home_tools_without_changing_path() {
    let sandbox = Sandbox::new();
    let stable_sysroot = sandbox.root.join("toolchains/stable");
    fs::create_dir_all(stable_sysroot.join("bin")).expect("create stable mock sysroot");
    sandbox.cargo_home_rustc("1.80.0");
    sandbox.cargo_home_cargo("1.80.0");
    sandbox.cargo_home_rustup("1.80.0", Some("1.80.0"), &stable_sysroot.join("bin/rustc"));
    sandbox.fail_curl();

    let output = sandbox.run(&["--run-cargo", "check", "--locked", "--all-targets"]);

    assert!(output.status.success(), "{}", stdout_stderr(&output));
    assert_eq!(sandbox.log(), "cargo <check> <--locked> <--all-targets>\n");
}

#[cfg(unix)]
#[test]
fn bootstrap_rust_posix_falls_back_to_home_dot_cargo_tools() {
    let sandbox = Sandbox::new();
    sandbox.cargo_home_rustc("1.80.0");
    sandbox.cargo_home_cargo("1.80.0");
    let default_cargo_home = sandbox.home.join(".cargo");
    fs::create_dir_all(&default_cargo_home).expect("create default Cargo home");
    fs::rename(
        sandbox.cargo_home.join("bin"),
        default_cargo_home.join("bin"),
    )
    .expect("move mocks to HOME/.cargo/bin");
    sandbox.fail_curl();

    let output = sandbox.run_script_with_homes(
        &bootstrap_script(),
        &["--run-cargo", "check", "--locked"],
        Some(&sandbox.home),
        None,
    );

    assert!(output.status.success(), "{}", stdout_stderr(&output));
    assert_eq!(sandbox.log(), "cargo <check> <--locked>\n");
}

#[cfg(unix)]
#[test]
fn bootstrap_rust_posix_second_call_reuses_officially_installed_tool_proxies() {
    let sandbox = Sandbox::new();
    let stable_sysroot = sandbox.root.join("toolchains/stable");
    fs::create_dir_all(stable_sysroot.join("bin")).expect("create stable mock sysroot");
    sandbox.download_curl(&installer_that_creates_tool_proxies(
        "1.80.0",
        "1.80.0",
        &stable_sysroot.join("bin/rustc"),
    ));

    let first_output = sandbox.run(&[]);
    assert!(
        first_output.status.success(),
        "{}",
        stdout_stderr(&first_output)
    );
    let first_log = sandbox.log();
    assert!(first_log.contains("curl <--proto>"), "{first_log}");
    assert!(first_log.contains("installer <-y>"), "{first_log}");

    let second_output = sandbox.run(&["--run-cargo", "metadata", "--locked"]);
    assert!(
        second_output.status.success(),
        "{}",
        stdout_stderr(&second_output)
    );
    let combined_log = sandbox.log();
    let second_log = combined_log
        .strip_prefix(&first_log)
        .expect("second invocation appends to the command log");
    assert_eq!(second_log, "cargo <metadata> <--locked>\n");
    assert!(!second_log.contains("curl"));
    assert!(!second_log.contains("installer"));
    assert!(!second_log.contains("rustup"));
}

#[cfg(unix)]
#[test]
fn bootstrap_rust_posix_uses_cargo_home_rustup_without_official_download() {
    let sandbox = Sandbox::new();
    let stable_sysroot = sandbox.root.join("toolchains/stable");
    fs::create_dir_all(stable_sysroot.join("bin")).expect("create stable mock sysroot");
    sandbox.cargo_home_rustup("1.80.0", Some("1.80.0"), &stable_sysroot.join("bin/rustc"));
    sandbox.fail_curl();

    let output = sandbox.run(&[]);

    assert!(output.status.success(), "{}", stdout_stderr(&output));
    assert_eq!(
        sandbox.log(),
        concat!(
            "rustup <toolchain> <install> <stable> <--profile> <minimal>\n",
            "rustup <run> <stable> <rustc> <--version>\n",
            "rustup <run> <stable> <cargo> <--version>\n"
        )
    );
}

#[cfg(unix)]
#[test]
fn bootstrap_rust_posix_accepts_sufficient_direct_prerelease_versions() {
    let sandbox = Sandbox::new();
    sandbox.rustc("1.90.0-nightly-2026-01-01", None);
    sandbox.cargo("1.90.0-beta.1");
    sandbox.fail_curl();

    let output = sandbox.run(&["--run-cargo", "check", "--locked"]);

    assert!(output.status.success(), "{}", stdout_stderr(&output));
    assert_eq!(sandbox.log(), "cargo <check> <--locked>\n");
}

#[cfg(unix)]
#[test]
fn bootstrap_rust_posix_treats_equal_core_prerelease_as_older_than_required_stable() {
    let sandbox = Sandbox::new();
    sandbox.rustc("1.75.0-nightly", None);
    sandbox.cargo("1.75.0-beta.2");
    sandbox.fail_curl();

    let output = sandbox.run(&[]);
    let diagnostics = stdout_stderr(&output);

    assert!(!output.status.success(), "{diagnostics}");
    assert!(
        diagnostics.contains("rustc 1.75.0-nightly"),
        "{diagnostics}"
    );
    assert!(diagnostics.contains("cargo 1.75.0-beta.2"), "{diagnostics}");
    assert!(
        diagnostics.contains("requires rustc and cargo >= 1.75.0"),
        "{diagnostics}"
    );
}

#[cfg(unix)]
#[test]
fn bootstrap_rust_posix_installs_a_missing_toolchain() {
    let sandbox = Sandbox::new();
    let stable_sysroot = sandbox.root.join("toolchains/stable");
    fs::create_dir_all(stable_sysroot.join("bin")).expect("create stable mock sysroot");
    sandbox.rustup("1.76.0", Some("1.76.0"), &stable_sysroot.join("bin/rustc"));
    sandbox.fail_curl();

    let output = sandbox.run(&[]);

    assert!(output.status.success(), "{}", stdout_stderr(&output));
    assert_eq!(
        sandbox.log(),
        concat!(
            "rustup <toolchain> <install> <stable> <--profile> <minimal>\n",
            "rustup <run> <stable> <rustc> <--version>\n",
            "rustup <run> <stable> <cargo> <--version>\n"
        )
    );
    assert!(!sandbox.log().contains("default"));
}

#[cfg(unix)]
#[test]
fn bootstrap_rust_posix_reports_official_installer_failure() {
    let sandbox = Sandbox::new();
    sandbox.download_curl(
        r#"printf 'installer' >> "$RUN_BOB_TEST_LOG"
for arg in "$@"; do printf ' <%s>' "$arg" >> "$RUN_BOB_TEST_LOG"; done
printf '\n' >> "$RUN_BOB_TEST_LOG"
exit 42"#,
    );

    let output = sandbox.run(&[]);
    let diagnostics = stdout_stderr(&output);

    assert!(!output.status.success(), "{diagnostics}");
    assert!(
        diagnostics.contains("official Rust installer failed"),
        "{diagnostics}"
    );
    assert!(sandbox
        .log()
        .contains("curl <--proto> <=https> <--tlsv1.2> <-sSf> <https://sh.rustup.rs> <-o> <"));
    assert!(sandbox.log().contains(
        "installer <-y> <--profile> <minimal> <--default-toolchain> <stable> <--no-modify-path>\n"
    ));
    assert!(!sandbox.log().contains("rustup <default>"));
}

#[cfg(unix)]
#[test]
fn bootstrap_rust_posix_rejects_incomplete_post_install_toolchain() {
    let sandbox = Sandbox::new();
    let stable_sysroot = sandbox.root.join("toolchains/stable");
    fs::create_dir_all(stable_sysroot.join("bin")).expect("create stable mock sysroot");
    sandbox.download_curl(&installer_that_creates_rustup(
        "1.76.0",
        None,
        &stable_sysroot.join("bin/rustc"),
    ));

    let output = sandbox.run(&[]);
    let diagnostics = stdout_stderr(&output);

    assert!(!output.status.success(), "{diagnostics}");
    assert!(
        diagnostics.contains("complete Rust toolchain"),
        "{diagnostics}"
    );
    assert!(sandbox
        .log()
        .contains("rustup <run> <stable> <cargo> <--version>\n"));
    assert!(!sandbox.log().contains("rustup <default>"));
}

#[cfg(unix)]
#[test]
fn bootstrap_rust_posix_rejects_missing_home_before_official_installer() {
    let sandbox = Sandbox::new();
    sandbox.download_curl("exit 0");

    let output = sandbox.run_script_with_homes(&bootstrap_script(), &[], None, None);
    let diagnostics = stdout_stderr(&output);

    assert!(!output.status.success(), "{diagnostics}");
    assert!(
        diagnostics.contains("CARGO_HOME and HOME are unavailable"),
        "{diagnostics}"
    );
    assert_eq!(sandbox.log(), "");
}

#[cfg(unix)]
#[test]
fn bootstrap_rust_posix_supported_path_tools_do_not_require_home() {
    let sandbox = Sandbox::new();
    sandbox.rustc("1.80.0", None);
    sandbox.cargo("1.80.0");
    sandbox.fail_curl();

    let output = sandbox.run_script_with_homes(
        &bootstrap_script(),
        &["--run-cargo", "check", "--locked"],
        None,
        None,
    );

    assert!(output.status.success(), "{}", stdout_stderr(&output));
    assert_eq!(sandbox.log(), "cargo <check> <--locked>\n");
}

#[cfg(unix)]
#[test]
fn bootstrap_rust_posix_uses_stable_for_old_active_rustup_toolchain() {
    let sandbox = Sandbox::new();
    let old_sysroot = sandbox.root.join("toolchains/old");
    fs::create_dir_all(old_sysroot.join("bin")).expect("create old mock sysroot");
    fs::write(old_sysroot.join("bin/rustc"), "mock").expect("create rustup compiler marker");
    sandbox.rustc("1.70.0-nightly", Some(&old_sysroot));
    sandbox.cargo("1.70.0-nightly");
    sandbox.rustup("1.76.0", Some("1.76.0"), &old_sysroot.join("bin/rustc"));
    sandbox.fail_curl();

    let output = sandbox.run(&["--run-cargo", "test", "--locked"]);

    assert!(output.status.success(), "{}", stdout_stderr(&output));
    assert_eq!(
        sandbox.log(),
        concat!(
            "rustup <which> <rustc>\n",
            "rustup <toolchain> <install> <stable> <--profile> <minimal>\n",
            "rustup <run> <stable> <rustc> <--version>\n",
            "rustup <run> <stable> <cargo> <--version>\n",
            "rustup <run> <stable> <cargo> <test> <--locked>\n"
        )
    );
    assert!(!sandbox.log().contains("default"));
}

#[cfg(unix)]
#[test]
fn bootstrap_rust_posix_stops_for_old_non_rustup_toolchain() {
    let sandbox = Sandbox::new();
    let system_sysroot = sandbox.root.join("system-rust");
    fs::create_dir_all(&system_sysroot).expect("create system sysroot");
    sandbox.rustc("1.74.1", Some(&system_sysroot));
    sandbox.cargo("1.74.1");
    sandbox.fail_curl();

    let output = sandbox.run(&[]);
    let diagnostics = stdout_stderr(&output);

    assert!(!output.status.success(), "{diagnostics}");
    assert!(diagnostics.contains("not rustup-owned"), "{diagnostics}");
    assert_eq!(sandbox.log(), "");
}

#[cfg(unix)]
#[test]
fn bootstrap_rust_posix_reports_both_detected_versions_when_cargo_is_old() {
    let sandbox = Sandbox::new();
    sandbox.rustc("1.80.0", None);
    sandbox.cargo("1.74.9");
    sandbox.fail_curl();

    let output = sandbox.run(&[]);
    let diagnostics = stdout_stderr(&output);

    assert!(!output.status.success(), "{diagnostics}");
    assert!(diagnostics.contains("rustc 1.80.0"), "{diagnostics}");
    assert!(diagnostics.contains("cargo 1.74.9"), "{diagnostics}");
    assert!(
        diagnostics.contains("requires rustc and cargo >= 1.75.0"),
        "{diagnostics}"
    );
}

#[cfg(unix)]
#[test]
fn bootstrap_rust_posix_stops_for_old_system_rust_with_unrelated_rustup() {
    let sandbox = Sandbox::new();
    let system_sysroot = sandbox.root.join("system-rust");
    let unrelated_sysroot = sandbox.root.join("unrelated-rustup");
    fs::create_dir_all(&system_sysroot).expect("create system sysroot");
    fs::create_dir_all(unrelated_sysroot.join("bin")).expect("create unrelated sysroot");
    fs::write(unrelated_sysroot.join("bin/rustc"), "mock").expect("create compiler marker");
    sandbox.rustc("1.74.1", Some(&system_sysroot));
    sandbox.cargo("1.74.1");
    sandbox.rustup(
        "1.76.0",
        Some("1.76.0"),
        &unrelated_sysroot.join("bin/rustc"),
    );
    sandbox.fail_curl();

    let output = sandbox.run(&[]);
    let diagnostics = stdout_stderr(&output);

    assert!(!output.status.success(), "{diagnostics}");
    assert!(diagnostics.contains("not rustup-owned"), "{diagnostics}");
    assert_eq!(sandbox.log(), "rustup <which> <rustc>\n");
    assert!(!sandbox.log().contains("toolchain"));
}

#[cfg(unix)]
#[test]
fn bootstrap_rust_posix_downloads_the_official_installer_when_all_tools_are_absent() {
    let sandbox = Sandbox::new();
    sandbox.assert_rust_tools_are_omitted_from_isolated_path();
    let stable_sysroot = sandbox.root.join("toolchains/stable");
    fs::create_dir_all(stable_sysroot.join("bin")).expect("create stable mock sysroot");
    sandbox.download_curl(&installer_that_creates_rustup(
        "1.75.0",
        Some("1.75.0"),
        &stable_sysroot.join("bin/rustc"),
    ));

    let output = sandbox.run(&[]);

    assert!(output.status.success(), "{}", stdout_stderr(&output));
    let log = sandbox.log();
    assert!(
        log.contains("curl <--proto> <=https> <--tlsv1.2> <-sSf> <https://sh.rustup.rs> <-o> <"),
        "{log}"
    );
    assert!(log.contains(
        "installer <-y> <--profile> <minimal> <--default-toolchain> <stable> <--no-modify-path>\n"
    ));
    assert!(log.contains("rustup <run> <stable> <rustc> <--version>\n"));
    assert!(log.contains("rustup <run> <stable> <cargo> <--version>\n"));
    assert!(!log.contains("rustup <default>"));
    assert!(
        log.lines().all(|line| line.starts_with("curl ")
            || line.starts_with("installer ")
            || line.starts_with("rustup ")),
        "unexpected command escaped the mock boundary: {log}"
    );
    let leaked_temporary_directory = fs::read_dir(&sandbox.root)
        .expect("read sandbox root")
        .filter_map(Result::ok)
        .any(|entry| {
            entry
                .file_name()
                .to_string_lossy()
                .starts_with("run-bob-rust.")
        });
    assert!(
        !leaked_temporary_directory,
        "installer temp directory leaked"
    );
}

#[cfg(unix)]
#[test]
fn bootstrap_rust_posix_isolated_path_omits_host_rust_tools() {
    let sandbox = Sandbox::new();
    sandbox.fail_curl();

    let host_rustc = Command::new("rustc")
        .arg("--version")
        .output()
        .expect("the cargo test host provides rustc");
    let host_cargo = Command::new("cargo")
        .arg("--version")
        .output()
        .expect("the cargo test host provides cargo");
    assert!(host_rustc.status.success());
    assert!(host_cargo.status.success());

    sandbox.assert_rust_tools_are_omitted_from_isolated_path();
    assert_eq!(sandbox.log(), "");
}

#[cfg(unix)]
#[test]
fn bootstrap_rust_posix_stops_for_a_partial_toolchain_without_invoking_rustup() {
    let sandbox = Sandbox::new();
    let sysroot = sandbox.root.join("partial-rust");
    fs::create_dir_all(&sysroot).expect("create partial sysroot");
    sandbox.rustc("1.75.0", Some(&sysroot));
    sandbox.fail_curl();

    let output = sandbox.run(&[]);
    let diagnostics = stdout_stderr(&output);

    assert!(!output.status.success(), "{diagnostics}");
    assert!(
        diagnostics.contains("partial non-rustup Rust toolchain"),
        "{diagnostics}"
    );
    assert_eq!(sandbox.log(), "");
}

#[cfg(unix)]
#[test]
fn bootstrap_rust_posix_repairs_a_partial_active_rustup_toolchain() {
    let sandbox = Sandbox::new();
    let active_sysroot = sandbox.root.join("toolchains/active");
    fs::create_dir_all(active_sysroot.join("bin")).expect("create active mock sysroot");
    fs::write(active_sysroot.join("bin/rustc"), "mock").expect("create compiler marker");
    sandbox.rustc("1.75.0", Some(&active_sysroot));
    sandbox.rustup("1.76.0", Some("1.76.0"), &active_sysroot.join("bin/rustc"));
    sandbox.fail_curl();

    let output = sandbox.run(&[]);

    assert!(output.status.success(), "{}", stdout_stderr(&output));
    assert_eq!(
        sandbox.log(),
        concat!(
            "rustup <which> <rustc>\n",
            "rustup <toolchain> <install> <stable> <--profile> <minimal>\n",
            "rustup <run> <stable> <rustc> <--version>\n",
            "rustup <run> <stable> <cargo> <--version>\n"
        )
    );
}

#[cfg(unix)]
#[test]
fn bootstrap_rust_posix_rejects_unknown_and_empty_public_forms() {
    let sandbox = Sandbox::new();
    sandbox.fail_curl();

    for arguments in [&["--run-cargo"][..], &["--unknown"][..]] {
        let output = sandbox.run(arguments);
        let diagnostics = stdout_stderr(&output);
        assert_eq!(output.status.code(), Some(64), "{diagnostics}");
        assert!(diagnostics.contains("Usage:"), "{diagnostics}");
    }
    assert_eq!(sandbox.log(), "");
}

#[cfg(unix)]
#[test]
fn bootstrap_rust_posix_handles_repository_and_tool_paths_with_spaces() {
    let sandbox = Sandbox::new();
    sandbox.rustc("1.75.0", None);
    sandbox.cargo("1.75.0");
    sandbox.fail_curl();

    let repository = sandbox.root.join("repository with spaces");
    let scripts = repository.join("scripts");
    fs::create_dir_all(&scripts).expect("create spaced repository");
    fs::copy(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml"),
        repository.join("Cargo.toml"),
    )
    .expect("copy manifest");
    let copied_script = scripts.join("bootstrap-rust.sh");
    fs::copy(bootstrap_script(), &copied_script).expect("copy bootstrap helper");
    let mut permissions = fs::metadata(&copied_script)
        .expect("copied helper metadata")
        .permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(&copied_script, permissions).expect("make copied helper executable");

    let output = sandbox.run_script(&copied_script, &["--run-cargo", "metadata", "--locked"]);

    assert!(output.status.success(), "{}", stdout_stderr(&output));
    assert_eq!(sandbox.log(), "cargo <metadata> <--locked>\n");
}
