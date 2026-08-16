#!/bin/sh
set -eu

usage() {
    printf '%s\n' "Usage: $0 [--run-cargo <cargo arguments>]" >&2
}

fail() {
    printf 'error: %s\n' "$1" >&2
    exit 1
}

normalize_version() {
    version=$1
    case "$version" in
        ''|*[!0-9.]*|.*|*.|*..*) return 1 ;;
    esac

    major=${version%%.*}
    remainder=${version#*.}
    if [ "$remainder" = "$version" ]; then
        return 1
    fi
    case "$remainder" in
        *.*)
            minor=${remainder%%.*}
            patch=${remainder#*.}
            case "$patch" in *.*) return 1 ;; esac
            ;;
        *)
            minor=$remainder
            patch=0
            ;;
    esac
    [ -n "$major" ] && [ -n "$minor" ] && [ -n "$patch" ] || return 1
    printf '%s.%s.%s\n' "$major" "$minor" "$patch"
}

tool_version() {
    tool=$1
    version_line=$("$tool" --version 2>/dev/null) || return 1
    set -- $version_line
    [ "$#" -ge 2 ] || return 1
    normalize_version "$2"
}

rustup_tool_version() {
    rustup_tool=$1
    rustup_version_line=$("$rustup_path" run stable "$rustup_tool" --version 2>/dev/null) || return 1
    set -- $rustup_version_line
    [ "$#" -ge 2 ] || return 1
    normalize_version "$2"
}

version_at_least() {
    actual=$1
    required=$2
    actual_major=${actual%%.*}
    actual_rest=${actual#*.}
    actual_minor=${actual_rest%%.*}
    actual_patch=${actual_rest#*.}
    required_major=${required%%.*}
    required_rest=${required#*.}
    required_minor=${required_rest%%.*}
    required_patch=${required_rest#*.}

    [ "$actual_major" -gt "$required_major" ] && return 0
    [ "$actual_major" -lt "$required_major" ] && return 1
    [ "$actual_minor" -gt "$required_minor" ] && return 0
    [ "$actual_minor" -lt "$required_minor" ] && return 1
    [ "$actual_patch" -ge "$required_patch" ]
}

canonical_directory() {
    [ -d "$1" ] || return 1
    CDPATH= cd -P "$1" 2>/dev/null && pwd
}

active_compiler_is_rustup_owned() {
    active_sysroot=$("$rustc_path" --print sysroot 2>/dev/null) || return 1
    rustup_compiler=$("$rustup_path" which rustc 2>/dev/null) || return 1
    [ -f "$rustup_compiler" ] || return 1

    active_sysroot=$(canonical_directory "$active_sysroot") || return 1
    rustup_bin=$(canonical_directory "$(dirname "$rustup_compiler")") || return 1
    rustup_sysroot=$(canonical_directory "$rustup_bin/..") || return 1
    [ "$active_sysroot" = "$rustup_sysroot" ]
}

select_rustup_stable() {
    "$rustup_path" toolchain install stable --profile minimal ||
        fail "rustup could not install the stable toolchain with the minimal profile"
    selected_mode=rustup
}

verify_rustup_stable() {
    selected_rustc_version=$(rustup_tool_version rustc) ||
        fail "bootstrap did not provide a complete Rust toolchain (rustc unavailable or invalid)"
    selected_cargo_version=$(rustup_tool_version cargo) ||
        fail "bootstrap did not provide a complete Rust toolchain (cargo unavailable or invalid)"
    version_at_least "$selected_rustc_version" "$required_version" ||
        fail "bootstrapped rustc $selected_rustc_version is older than required Rust $required_version"
    version_at_least "$selected_cargo_version" "$required_version" ||
        fail "bootstrapped cargo $selected_cargo_version is older than required Rust $required_version"
}

bootstrap_tmp=
bootstrap_tmp_valid=false

cleanup_bootstrap_tmp() {
    if [ "$bootstrap_tmp_valid" = true ] && [ -n "$bootstrap_tmp" ]; then
        rm -rf "$bootstrap_tmp"
        bootstrap_tmp=
        bootstrap_tmp_valid=false
    fi
}

download_rustup() {
    temp_base=${TMPDIR:-/tmp}
    temp_base=$(canonical_directory "$temp_base") || fail "temporary directory is unavailable"
    bootstrap_tmp=$(mktemp -d "$temp_base/run-bob-rust.XXXXXX") ||
        fail "could not create a temporary directory for the Rust installer"
    case "$bootstrap_tmp" in
        "$temp_base"/run-bob-rust.*) bootstrap_tmp_valid=true ;;
        *) fail "temporary Rust installer path was outside the requested directory" ;;
    esac
    trap cleanup_bootstrap_tmp 0 HUP INT TERM

    installer_file=$bootstrap_tmp/rustup-init.sh
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs -o "$installer_file" ||
        fail "could not download the official Rust installer"
    sh "$installer_file" -y --profile minimal --default-toolchain stable --no-modify-path ||
        fail "official Rust installer failed"

    cargo_home=${CARGO_HOME:-${HOME:-}/.cargo}
    [ -n "$cargo_home" ] || fail "CARGO_HOME and HOME are unavailable"
    if [ -x "$cargo_home/bin/rustup" ]; then
        rustup_path=$cargo_home/bin/rustup
    else
        rustup_path=$(command -v rustup 2>/dev/null || true)
    fi
    [ -n "$rustup_path" ] || fail "official installer completed but rustup is unavailable"

    selected_mode=rustup
    cleanup_bootstrap_tmp
    trap - 0 HUP INT TERM
}

case $# in
    0) run_cargo=false ;;
    *)
        if [ "$1" != "--run-cargo" ]; then
            usage
            exit 64
        fi
        shift
        if [ "$#" -eq 0 ]; then
            usage
            exit 64
        fi
        run_cargo=true
        ;;
esac

script_dir=$(CDPATH= cd -P "$(dirname "$0")" && pwd)
repo_root=$(CDPATH= cd -P "$script_dir/.." && pwd)
manifest=$repo_root/Cargo.toml
[ -f "$manifest" ] || fail "Cargo.toml not found at $manifest"

package_name=$(awk '
    /^\[package\][[:space:]]*$/ { package = 1; next }
    /^\[/ { if (package) exit }
    package && /^[[:space:]]*name[[:space:]]*=/ {
        line = $0
        sub(/^[[:space:]]*name[[:space:]]*=[[:space:]]*"/, "", line)
        sub(/"[[:space:]]*$/, "", line)
        print line
        exit
    }
' "$manifest")
[ "$package_name" = "run-bob" ] || fail "expected package name run-bob in $manifest"

required_version=$(awk '
    /^\[package\][[:space:]]*$/ { package = 1; next }
    /^\[/ { if (package) exit }
    package && /^[[:space:]]*rust-version[[:space:]]*=/ {
        line = $0
        sub(/^[[:space:]]*rust-version[[:space:]]*=[[:space:]]*"/, "", line)
        sub(/"[[:space:]]*$/, "", line)
        print line
        exit
    }
' "$manifest")
required_version=$(normalize_version "$required_version") ||
    fail "Cargo.toml rust-version must be a complete numeric major.minor or major.minor.patch version"

rustc_path=$(command -v rustc 2>/dev/null || true)
cargo_path=$(command -v cargo 2>/dev/null || true)
rustup_path=$(command -v rustup 2>/dev/null || true)
selected_mode=

if [ -n "$rustc_path" ] && [ -n "$cargo_path" ]; then
    rustc_version=$(tool_version "$rustc_path") || fail "could not read a complete rustc semantic version"
    cargo_version=$(tool_version "$cargo_path") || fail "could not read a complete cargo semantic version"
    if version_at_least "$rustc_version" "$required_version" &&
        version_at_least "$cargo_version" "$required_version"; then
        selected_mode=direct
    else
        [ -n "$rustup_path" ] ||
            fail "installed Rust $rustc_version is too old and is not rustup-owned; refusing to replace it"
        active_compiler_is_rustup_owned ||
            fail "installed Rust $rustc_version is too old and is not rustup-owned; refusing to replace it"
        select_rustup_stable
    fi
elif [ -z "$rustc_path" ] && [ -z "$cargo_path" ]; then
    if [ -n "$rustup_path" ]; then
        select_rustup_stable
    else
        download_rustup
    fi
else
    if [ -n "$rustc_path" ] && [ -n "$rustup_path" ] && active_compiler_is_rustup_owned; then
        select_rustup_stable
    else
        fail "a partial non-rustup Rust toolchain is installed; refusing to replace it automatically"
    fi
fi

if [ "$selected_mode" = direct ]; then
    # Re-read both commands after selection so no unchecked tool is executed.
    rustc_version=$(tool_version "$rustc_path") || fail "could not re-verify rustc"
    cargo_version=$(tool_version "$cargo_path") || fail "could not re-verify cargo"
    version_at_least "$rustc_version" "$required_version" || fail "rustc changed during verification"
    version_at_least "$cargo_version" "$required_version" || fail "cargo changed during verification"
    if [ "$run_cargo" = true ]; then
        exec "$cargo_path" "$@"
    fi
else
    verify_rustup_stable
    if [ "$run_cargo" = true ]; then
        exec "$rustup_path" run stable cargo "$@"
    fi
fi
