#!/usr/bin/env sh
# install.sh — one-stop installer for run-bob.
#
# Usage:
#   curl -fsSL https://raw.githubusercontent.com/amwtke/run-bob/master/install.sh | sh
#
# Environment:
#   RUN_BOB_INSTALL_DIR  destination directory (default: $HOME/.local/bin)
#   RUN_BOB_VERSION      release tag to install  (default: latest)
#
# Detects OS + arch, downloads the matching prebuilt tarball from
# GitHub Releases, and drops the `run-bob` binary into the install dir.

set -eu

REPO="amwtke/run-bob"
BIN_NAME="run-bob"
INSTALL_DIR="${RUN_BOB_INSTALL_DIR:-$HOME/.local/bin}"
VERSION="${RUN_BOB_VERSION:-latest}"

err() { printf '\033[31merror:\033[0m %s\n' "$1" >&2; exit 1; }
ok()  { printf '\033[32m✓\033[0m %s\n' "$1"; }
inf() { printf '\033[34m→\033[0m %s\n' "$1"; }
warn(){ printf '\033[33m⚠\033[0m %s\n' "$1"; }

# --- detect OS ---
case "$(uname -s)" in
    Linux)  os=linux  ;;
    Darwin) os=darwin ;;
    *) err "unsupported OS: $(uname -s). See https://github.com/${REPO}/releases for manual download." ;;
esac

# --- detect arch ---
case "$(uname -m)" in
    x86_64|amd64)   arch=x86_64   ;;
    arm64|aarch64)  arch=aarch64  ;;
    *) err "unsupported architecture: $(uname -m)" ;;
esac

# --- map to Rust target triple ---
if [ "$os" = "linux"  ] && [ "$arch" = "x86_64"  ]; then target="x86_64-unknown-linux-gnu"
elif [ "$os" = "darwin" ] && [ "$arch" = "aarch64" ]; then target="aarch64-apple-darwin"
elif [ "$os" = "darwin" ] && [ "$arch" = "x86_64"  ]; then target="x86_64-apple-darwin"
else
    err "no prebuilt binary for $os/$arch. Try: cargo install --git https://github.com/${REPO}"
fi

# --- resolve latest version if needed ---
if [ "$VERSION" = "latest" ]; then
    inf "resolving latest release..."
    VERSION=$(curl -fsSL "https://api.github.com/repos/${REPO}/releases/latest" \
        | sed -nE 's/.*"tag_name"[[:space:]]*:[[:space:]]*"([^"]+)".*/\1/p' \
        | head -n 1)
    [ -n "$VERSION" ] || err "failed to resolve latest version. Check network or set RUN_BOB_VERSION."
fi

asset="${BIN_NAME}-${VERSION}-${target}.tar.gz"
url="https://github.com/${REPO}/releases/download/${VERSION}/${asset}"

echo
echo "run-bob installer"
echo "  target  : $target"
echo "  version : $VERSION"
echo "  url     : $url"
echo "  destdir : $INSTALL_DIR"
echo

# --- download + extract ---
tmp=$(mktemp -d 2>/dev/null || mktemp -d -t run-bob)
trap 'rm -rf "$tmp"' EXIT INT TERM

inf "downloading..."
curl -fL --progress-bar "$url" -o "${tmp}/${asset}" \
    || err "download failed: $url"

inf "extracting..."
tar -xzf "${tmp}/${asset}" -C "$tmp"

src="${tmp}/${BIN_NAME}-${VERSION}-${target}/${BIN_NAME}"
[ -f "$src" ] || err "binary not found in archive: $src"

# --- install ---
mkdir -p "$INSTALL_DIR"
mv "$src" "${INSTALL_DIR}/${BIN_NAME}"
chmod +x "${INSTALL_DIR}/${BIN_NAME}"

# macOS: strip quarantine attribute so Gatekeeper doesn't block first run
if [ "$os" = "darwin" ]; then
    xattr -d com.apple.quarantine "${INSTALL_DIR}/${BIN_NAME}" 2>/dev/null || true
fi

echo
ok "installed: ${INSTALL_DIR}/${BIN_NAME}"
echo

# --- PATH guidance ---
case ":${PATH}:" in
    *":${INSTALL_DIR}:"*)
        ok "${INSTALL_DIR} is on your PATH."
        echo "  Try: run-bob --version"
        ;;
    *)
        warn "${INSTALL_DIR} is NOT on your PATH."
        echo
        echo "  Add it to your shell rc (e.g. ~/.zshrc, ~/.bashrc):"
        echo
        echo "    export PATH=\"${INSTALL_DIR}:\$PATH\""
        echo
        echo "  Then reload:"
        echo "    source ~/.zshrc   # or ~/.bashrc"
        ;;
esac
