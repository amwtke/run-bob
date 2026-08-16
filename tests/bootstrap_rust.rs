#![cfg(unix)]

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

struct Sandbox {
    _temp: tempfile::TempDir,
    root: PathBuf,
    mock_bin: PathBuf,
    home: PathBuf,
    cargo_home: PathBuf,
    rustup_home: PathBuf,
    log: PathBuf,
}

impl Sandbox {
    fn new() -> Self {
        let temp = tempfile::tempdir().expect("create bootstrap sandbox");
        let root = temp.path().to_path_buf();
        let mock_bin = root.join("mock bin");
        let home = root.join("home with spaces");
        let cargo_home = root.join("cargo home");
        let rustup_home = root.join("rustup home");
        let log = root.join("commands.log");

        for directory in [&mock_bin, &home, &cargo_home, &rustup_home] {
            fs::create_dir_all(directory).expect("create isolated directory");
        }

        Self {
            _temp: temp,
            root,
            mock_bin,
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
        self.run_script(&bootstrap_script(), arguments)
    }

    fn run_script(&self, script: &Path, arguments: &[&str]) -> Output {
        let path = format!("{}:/usr/bin:/bin", self.mock_bin.display());
        Command::new(script)
            .args(arguments)
            .env_clear()
            .env("PATH", path)
            .env("HOME", &self.home)
            .env("CARGO_HOME", &self.cargo_home)
            .env("RUSTUP_HOME", &self.rustup_home)
            .env("RUN_BOB_TEST_LOG", &self.log)
            .env("TMPDIR", &self.root)
            .output()
            .expect("execute POSIX Rust bootstrap")
    }

    fn log(&self) -> String {
        fs::read_to_string(&self.log).unwrap_or_default()
    }
}

fn bootstrap_script() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("scripts")
        .join("bootstrap-rust.sh")
}

fn stdout_stderr(output: &Output) -> String {
    format!(
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )
}

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

#[test]
fn bootstrap_rust_posix_uses_stable_for_old_active_rustup_toolchain() {
    let sandbox = Sandbox::new();
    let old_sysroot = sandbox.root.join("toolchains/old");
    fs::create_dir_all(old_sysroot.join("bin")).expect("create old mock sysroot");
    fs::write(old_sysroot.join("bin/rustc"), "mock").expect("create rustup compiler marker");
    sandbox.rustc("1.74.1", Some(&old_sysroot));
    sandbox.cargo("1.74.1");
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

#[test]
fn bootstrap_rust_posix_downloads_the_official_installer_when_all_tools_are_absent() {
    let sandbox = Sandbox::new();
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
