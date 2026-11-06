#!/usr/bin/env bash
# Install /buddy ol pal (buddy).
#
# Two paths:
#   1. prebuilt binary: downloaded from the GitHub releases page when one
#      matches this machine (linux x86_64 today; more as releases add them).
#   2. from source: cargo build --release, needs rustup or cargo on PATH.
#
# Env: BUDDY_BIN sets the install directory (default ~/.local/bin).
set -euo pipefail

REPO="aloysiusvonfck/buddy-ol-pal"
dest="${BUDDY_BIN:-$HOME/.local/bin}"
machine="$(uname -s)-$(uname -m)"

fetch_prebuilt() {
  # Best-effort: if there is a release binary for this machine, use it.
  command -v curl >/dev/null 2>&1 || return 1
  case "$machine" in
    Linux-x86_64)  asset="buddy-linux-x86_64" ;;
    Linux-aarch64) asset="buddy-linux-aarch64" ;;
    *) return 1 ;;
  esac
  url="https://github.com/$REPO/releases/latest/download/$asset"
  tmp="$(mktemp -d)"
  if ! curl -fsSL -o "$tmp/buddy" "$url" 2>/dev/null; then
    rm -rf "$tmp"; return 1
  fi
  chmod +x "$tmp/buddy"
  mkdir -p "$dest"
  install -m 755 "$tmp/buddy" "$dest/buddy"
  rm -rf "$tmp"
  echo "installed prebuilt binary -> $dest/buddy"
  return 0
}

build_from_source() {
  cd "$(dirname "$0")"
  if ! command -v cargo >/dev/null 2>&1; then
    echo "buddy: no cargo on PATH, and no prebuilt binary for $machine." >&2
    echo "install rust first: https://rustup.rs" >&2
    exit 1
  fi
  cargo build --release
  mkdir -p "$dest"
  install -m 755 target/release/buddy "$dest/buddy"
  echo "built from source -> $dest/buddy"
}

if fetch_prebuilt; then :; else build_from_source; fi

case ":$PATH:" in
  *":$dest:"*) : ;;
  *) echo "note: $dest is not on your PATH - add it to keep 'buddy' findable" ;;
esac
echo "next: eval \"\$(buddy shell bash)\"   # (or: buddy shell zsh) wire up passive capture"
