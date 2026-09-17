#!/usr/bin/env sh
# Install script for warp-tui
#
# Downloads the latest release binary for the current OS/architecture from
# GitHub releases and installs it into $INSTALL_DIR (defaults to
# $HOME/.local/bin).
#
# Usage:
#   curl -fsSL https://raw.githubusercontent.com/mertssmnoglu/warp-tui/main/scripts/install.sh | sh
#
# Environment variables:
#   INSTALL_DIR   Directory to install the binary into (default: $HOME/.local/bin)
#   VERSION       Specific release tag to install, e.g. v1.2.3 (default: latest)

set -eu

REPO="mertssmnoglu/warp-tui"
BIN_NAME="warp-tui"
INSTALL_DIR="${INSTALL_DIR:-$HOME/.local/bin}"
VERSION="${VERSION:-latest}"

log() {
    printf '==> %s\n' "$1"
}

err() {
    printf 'error: %s\n' "$1" >&2
    exit 1
}

detect_os() {
    os=$(uname -s)
    case "$os" in
        Linux) echo "linux" ;;
        Darwin) echo "apple-darwin" ;;
        MINGW* | MSYS* | CYGWIN*) echo "pc-windows-msvc" ;;
        *) err "unsupported OS: $os" ;;
    esac
}

detect_arch() {
    arch=$(uname -m)
    case "$arch" in
        x86_64 | amd64) echo "x86_64" ;;
        aarch64 | arm64) echo "aarch64" ;;
        *) err "unsupported architecture: $arch" ;;
    esac
}

resolve_version() {
    if command -v curl >/dev/null 2>&1; then
        curl -fsSL "https://api.github.com/repos/${REPO}/releases/latest" |
            grep '"tag_name":' | sed -E 's/.*"tag_name": *"([^"]+)".*/\1/'
    elif command -v wget >/dev/null 2>&1; then
        wget -qO- "https://api.github.com/repos/${REPO}/releases/latest" |
            grep '"tag_name":' | sed -E 's/.*"tag_name": *"([^"]+)".*/\1/'
    else
        err "curl or wget is required to install ${BIN_NAME}"
    fi
}

main() {
    os_part=$(detect_os)
    arch_part=$(detect_arch)
    target="${arch_part}-${os_part}"

    tag="$VERSION"
    if [ "$tag" = "latest" ]; then
        tag=$(resolve_version)
        [ -n "$tag" ] || err "failed to resolve the latest release tag"
    fi
    version_num="${tag#v}"

    case "$os_part" in
        pc-windows-msvc) asset="${BIN_NAME}-${version_num}-${target}.exe" ;;
        *) asset="${BIN_NAME}-${version_num}-${target}" ;;
    esac

    url="https://github.com/${REPO}/releases/download/${tag}/${asset}"

    tmp_dir=$(mktemp -d)
    trap 'rm -rf "$tmp_dir"' EXIT

    log "Downloading ${asset} (${tag})"
    if command -v curl >/dev/null 2>&1; then
        curl -fsSL "$url" -o "$tmp_dir/$asset" || err "failed to download $url"
    elif command -v wget >/dev/null 2>&1; then
        wget -q "$url" -O "$tmp_dir/$asset" || err "failed to download $url"
    else
        err "curl or wget is required to install ${BIN_NAME}"
    fi

    mkdir -p "$INSTALL_DIR"
    install -m 755 "$tmp_dir/$asset" "$INSTALL_DIR/$BIN_NAME"

    log "Installed ${BIN_NAME} to ${INSTALL_DIR}/${BIN_NAME}"

    case ":$PATH:" in
        *":$INSTALL_DIR:"*) ;;
        *) log "Note: ${INSTALL_DIR} is not in your PATH. Add it with:
    export PATH=\"${INSTALL_DIR}:\$PATH\"" ;;
    esac
}

main "$@"
