#!/bin/sh
# SPDX-License-Identifier: GPL-3.0-only
# pre-commit entry point for the `radolint` hook (.pre-commit-hooks.yaml).
#
# Runs the radolint release binary that matches the hook's `rev`, downloading it from GitHub
# releases on first use and checking its SHA-256. pre-commit has no way to fetch a prebuilt
# binary by itself, which is why hadolint's own `hadolint` hook needs hadolint on PATH
# (hadolint/hadolint#886). Binaries are cached per version in $XDG_CACHE_HOME/radolint (or
# ~/.cache/radolint; %LOCALAPPDATA%\radolint on Windows).
#
# Set RADOLINT_HOOK_BINARY to use a binary you already have instead of downloading one.
set -eu

if [ -n "${RADOLINT_HOOK_BINARY:-}" ]; then
    exec "$RADOLINT_HOOK_BINARY" "$@"
fi

repo_root=$(cd "$(dirname "$0")/../.." && pwd)
version=$(sed -n 's/^version = "\(.*\)"$/\1/p' "$repo_root/Cargo.toml" | head -n 1)
if [ -z "$version" ]; then
    echo "radolint hook: cannot read the version from $repo_root/Cargo.toml" >&2
    exit 2
fi

ext=tar.gz
exe=radolint
case "$(uname -s)" in
    Linux) os=unknown-linux-musl ;;
    Darwin) os=apple-darwin ;;
    MINGW* | MSYS* | CYGWIN* | Windows_NT) os=pc-windows-msvc ext=zip exe=radolint.exe ;;
    *) os= ;;
esac
case "$(uname -m)" in
    x86_64 | amd64) arch=x86_64 ;;
    aarch64 | arm64) arch=aarch64 ;;
    *) arch= ;;
esac
target="$arch-$os"
case "$target" in
    x86_64-unknown-linux-musl | aarch64-unknown-linux-musl | x86_64-apple-darwin | \
        aarch64-apple-darwin | x86_64-pc-windows-msvc) ;;
    *)
        echo "radolint hook: no prebuilt radolint for $(uname -s) $(uname -m)." >&2
        echo "Install radolint yourself and set RADOLINT_HOOK_BINARY, or use the radolint-system hook." >&2
        exit 2
        ;;
esac

if [ "$os" = pc-windows-msvc ] && [ -n "${LOCALAPPDATA:-}" ]; then
    cache_root="$LOCALAPPDATA/radolint"
else
    cache_root="${XDG_CACHE_HOME:-$HOME/.cache}/radolint"
fi
name="radolint-v$version-$target"
bin="$cache_root/$name/$exe"

if [ ! -x "$bin" ]; then
    mkdir -p "$cache_root"
    tmp=$(mktemp -d "$cache_root/.download.XXXXXX")
    trap 'rm -rf "$tmp"' EXIT
    url="https://github.com/DevanshChoubey/radolint/releases/download/v$version/$name.$ext"
    echo "radolint hook: downloading $name.$ext" >&2
    if command -v curl >/dev/null 2>&1; then
        curl -fsSL --retry 3 -o "$tmp/$name.$ext" "$url"
        curl -fsSL --retry 3 -o "$tmp/$name.$ext.sha256" "$url.sha256"
    elif command -v wget >/dev/null 2>&1; then
        wget -q -O "$tmp/$name.$ext" "$url"
        wget -q -O "$tmp/$name.$ext.sha256" "$url.sha256"
    else
        echo "radolint hook: needs curl or wget to download radolint" >&2
        exit 2
    fi
    expected=$(cut -d ' ' -f 1 "$tmp/$name.$ext.sha256")
    if command -v sha256sum >/dev/null 2>&1; then
        actual=$(sha256sum "$tmp/$name.$ext" | cut -d ' ' -f 1)
    else
        actual=$(shasum -a 256 "$tmp/$name.$ext" | cut -d ' ' -f 1)
    fi
    if [ "$expected" != "$actual" ]; then
        echo "radolint hook: checksum mismatch for $name.$ext" >&2
        exit 2
    fi
    if [ "$ext" = zip ]; then
        if command -v unzip >/dev/null 2>&1; then
            unzip -q "$tmp/$name.$ext" -d "$tmp"
        else
            tar -xf "$tmp/$name.$ext" -C "$tmp"
        fi
    else
        tar -xzf "$tmp/$name.$ext" -C "$tmp"
    fi
    chmod +x "$tmp/$name/$exe"
    # A directory left without the binary (an interrupted install) would make mv nest the new
    # tree inside it, so clear it first.
    [ -x "$bin" ] || rm -rf "${cache_root:?}/$name"
    # Another hook process may have finished first; either copy is the same file.
    mv "$tmp/$name" "$cache_root/$name" 2>/dev/null || true
    # exec below skips the EXIT trap, so clean up here.
    rm -rf "$tmp"
    trap - EXIT
    [ -x "$bin" ] || { echo "radolint hook: could not install $bin" >&2; exit 2; }
fi

exec "$bin" "$@"
