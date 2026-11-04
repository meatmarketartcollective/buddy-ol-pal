#!/usr/bin/env bash
# Install hatch (tb) from source. Needs a Rust toolchain: rustup or cargo on PATH.
set -euo pipefail
cd "$(dirname "$0")"
cargo build --release
dest="${HATCH_BIN:-$HOME/.local/bin}"
mkdir -p "$dest"
install -m 755 target/release/tb "$dest/tb"
echo "installed $dest/tb - make sure $dest is on your PATH"
