#!/usr/bin/env bash
# Install hatch (tb).
#
# Two paths:
#   1. prebuilt binary: downloaded from the GitHub releases page when one
#      matches this machine (linux x86_64 today; more as releases add them).
#   2. from source: cargo build --release, needs rustup or cargo on PATH.
#
# Env: HATCH_BIN sets the install directory (default ~/.local/bin).
set -euo pipefail

REPO="aloysiusvonfck/hatch"
dest="${HATCH_BIN:-$HOME/.local/bin}"
machine="$(uname -s)-$(uname -m)"

fetch_prebuilt() {
  # Best-effort: if there is a release binary for this machine, use it.
  command -v curl >/dev/null 2>&1 || return 1
  case "$machine" in
    Linux-x86_64) asset="tb-linux-x86_64" ;;
    *) return 1 ;;
  esac
  url="https://github.com/$REPO/releases/latest/download/$asset"
  tmp="$(mktemp -d)"
  if ! curl -fsSL -o "$tmp/tb" "$url" 2>/dev/null; then
    rm -rf "$tmp"; return 1
  fi
  chmod +x "$tmp/tb"
  mkdir -p "$dest"
  install -m 755 "$tmp/tb" "$dest/tb"
  rm -rf "$tmp"
  echo "installed prebuilt binary -> $dest/tb"
  return 0
}

build_from_source() {
  cd "$(dirname "$0")"
  if ! command -v cargo >/dev/null 2>&1; then
    echo "hatch: no cargo on PATH, and no prebuilt binary for $machine." >&2
    echo "install rust first: https://rustup.rs" >&2
    exit 1
  fi
  cargo build --release
  mkdir -p "$dest"
  install -m 755 target/release/tb "$dest/tb"
  echo "built from source -> $dest/tb"
}

if fetch_prebuilt; then :; else build_from_source; fi

case ":$PATH:" in
  *":$dest:"*) : ;;
  *) echo "note: $dest is not on your PATH - add it to keep 'tb' findable" ;;
esac
echo "next: eval \"\$(tb shell bash)\"   # (or: tb shell zsh) wire up passive capture"
