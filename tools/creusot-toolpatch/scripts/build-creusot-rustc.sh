#!/usr/bin/env bash
set -euo pipefail

bundle_dir=$(cd "$(dirname "$0")/.." && pwd)
creusot_repo=${1:-/tmp/creusot-source}
target_dir=${2:-/workspace/proof-tools/targets/creusot-cast-compdiv}
creusot_commit=437d3d8d00b8114d7a3b4f7b8738d594a395f5bc
source_dir=$(mktemp -d /tmp/creusot-narrowcast-build.XXXXXX)
trap 'rm -rf "$source_dir"' EXIT

actual_commit=$(git -C "$creusot_repo" rev-parse HEAD)
if [[ "$actual_commit" != "$creusot_commit" ]]; then
  printf 'Expected Creusot commit %s, found %s\n' "$creusot_commit" "$actual_commit" >&2
  exit 1
fi

git -C "$creusot_repo" archive "$creusot_commit" | tar -x -C "$source_dir"
git -C "$source_dir" apply "$bundle_dir/patches/creusot-narrowcast-backend.patch"

export RUSTUP_HOME=${RUSTUP_HOME:-/tmp/rustup-home}
export CARGO_HOME=${CARGO_HOME:-/tmp/cargo-home}
export RUSTUP_TOOLCHAIN=${RUSTUP_TOOLCHAIN:-nightly-2026-02-27}
export CARGO_TARGET_DIR=$target_dir
export CARGO_NET_OFFLINE=true
export PATH="/tmp/cargo-home/bin:$PATH"
cargo build --offline --manifest-path "$source_dir/Cargo.toml" -p creusot-rustc

binary="$target_dir/debug/creusot-rustc"
test -x "$binary"
printf '%s\n' "$binary"
