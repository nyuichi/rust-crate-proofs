#!/usr/bin/env bash
set -euo pipefail

script_dir=$(cd -- "$(dirname "$0")" && pwd)
repo_root=$(git -C "$script_dir" rev-parse --show-toplevel)
patch_dir=$repo_root/tools/creusot-toolpatch/patches
source_repo=${CREUSOT_CONSTANT_INDEX_SOURCE_REPO:-/workspace/proof-tools/creusot-source}
target_root=${CREUSOT_CONSTANT_INDEX_TARGET_ROOT:-/workspace/httparse-tool-rebuild/constant-index-gate}
pinned=437d3d8d00b8114d7a3b4f7b8738d594a395f5bc
run_id=$(date -u +%Y%m%dT%H%M%SZ)-$$
run_root=$target_root/$run_id
source_dir=$run_root/source
target_dir=$run_root/target
patches=(
  "$patch_dir/creusot-narrowcast-backend.patch"
  "$patch_dir/httparse-string-model.patch"
  "$patch_dir/httparse-constant-index.patch"
)

[[ -d "$source_repo/.git" || -f "$source_repo/.git" ]]
actual_pinned=$(git -C "$source_repo" rev-parse --verify "$pinned^{commit}")
[[ "$actual_pinned" == "$pinned" ]]
mkdir -p "$source_dir"
git -C "$source_repo" archive "$pinned" | tar -x -C "$source_dir"
for patch in "${patches[@]}"; do
  git -C "$source_dir" apply --check "$patch"
  git -C "$source_dir" apply "$patch"
done

source /workspace/proof-tools/activate.sh
export RUSTUP_TOOLCHAIN=nightly-2026-02-27
export CARGO_NET_OFFLINE=true
export CARGO_TARGET_DIR=$target_dir
cargo build --offline --manifest-path "$source_dir/Cargo.toml" -p creusot-rustc
compiler_bin=$target_dir/debug/creusot-rustc
test -x "$compiler_bin"
{
  printf 'run_id\t%s\n' "$run_id"
  printf 'upstream_commit\t%s\n' "$pinned"
  printf 'compiler_bin\t%s\n' "$compiler_bin"
  printf 'compiler_sha256\t%s\n' "$(sha256sum "$compiler_bin" | awk '{print $1}')"
  printf 'rustc_vv\t%s\n' "$(rustc -Vv | tr '\n' ' ')"
  printf 'source_tree_base\t%s\n' "$source_repo"
  for patch in "${patches[@]}"; do
    printf 'patch_%s\t%s\n' "$(basename "$patch")" "$(sha256sum "$patch" | awk '{print $1}')"
  done
} > "$run_root/toolchain.tsv"
cp "${patches[2]}" "$run_root/httparse-constant-index.patch"
printf '%s\n' "$run_root" > "$target_root/latest-build"
printf 'compiler: %s\n' "$compiler_bin"
printf 'toolchain manifest: %s\n' "$run_root/toolchain.tsv"
