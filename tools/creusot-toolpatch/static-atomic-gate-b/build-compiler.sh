#!/usr/bin/env bash
set -euo pipefail

script_dir=$(cd "$(dirname "$0")" && pwd)
repo_root=$(git -C "$script_dir" rev-parse --show-toplevel)
patch_dir=$repo_root/tools/creusot-toolpatch/patches
source_repo=${CREUSOT_STATIC_ATOMIC_SOURCE_REPO:-/workspace/proof-tools/creusot-source}
target_dir=${CREUSOT_STATIC_ATOMIC_TARGET_DIR:-/workspace/proof-tools/targets/httparse-static-atomic-gate-b}
pinned=437d3d8d00b8114d7a3b4f7b8738d594a395f5bc

narrowcast_patch=$patch_dir/creusot-narrowcast-backend.patch
gate_a_patch=$patch_dir/httparse-static-atomic-gate-a.patch
gate_b_patch=$patch_dir/httparse-static-atomic-gate-b.patch

check_hash() {
    local expected=$1 path=$2 label=$3 actual
    actual=$(sha256sum "$path" | awk '{print $1}')
    if [[ "$actual" != "$expected" ]]; then
        printf '%s SHA-256 mismatch: expected %s, found %s\n' "$label" "$expected" "$actual" >&2
        exit 1
    fi
}

check_hash c3ac3e596b822f483d2f0f782169aacc5c752449cf66771ee7268ee2feb8eb24 "$narrowcast_patch" "narrowcast patch"
check_hash e075f03f479f7efab0967345e2a25e84d063c0a8357ece399f097ece17d2e044 "$gate_a_patch" "Gate A patch"
check_hash 4f70d3d5bf8e72355b80a98829cb927423ee9e665c02083973b43ca0d5f4d2d8 "$gate_b_patch" "Gate B patch"

if [[ ! -d "$source_repo/.git" && ! -f "$source_repo/.git" ]]; then
    printf 'Creusot source repository not found: %s\n' "$source_repo" >&2
    exit 1
fi
actual_pinned=$(git -C "$source_repo" rev-parse --verify "$pinned^{commit}")
if [[ "$actual_pinned" != "$pinned" ]]; then
    printf 'Pinned source commit mismatch: expected %s, found %s\n' "$pinned" "$actual_pinned" >&2
    exit 1
fi

source_dir=$(mktemp -d "${TMPDIR:-/tmp}/creusot-static-atomic-gate-b.XXXXXX")
trap 'rm -rf "$source_dir"' EXIT
git -C "$source_repo" archive "$pinned" | tar -x -C "$source_dir"

for patch in "$narrowcast_patch" "$gate_a_patch" "$gate_b_patch"; do
    git -C "$source_dir" apply --check "$patch"
    git -C "$source_dir" apply "$patch"
done

source /workspace/proof-tools/activate.sh
export RUSTUP_TOOLCHAIN=nightly-2026-02-27
export CARGO_NET_OFFLINE=true
export CARGO_TARGET_DIR=$target_dir
mkdir -p "$target_dir"
cargo build --offline --manifest-path "$source_dir/Cargo.toml" -p creusot-rustc

compiler_bin=$target_dir/debug/creusot-rustc
test -x "$compiler_bin"
printf 'Upstream source commit: %s\n' "$pinned"
printf 'narrowcast patch SHA-256: %s\n' "$(sha256sum "$narrowcast_patch" | awk '{print $1}')"
printf 'Gate A patch SHA-256: %s\n' "$(sha256sum "$gate_a_patch" | awk '{print $1}')"
printf 'Gate B patch SHA-256: %s\n' "$(sha256sum "$gate_b_patch" | awk '{print $1}')"
printf 'compiler binary: %s\n' "$compiler_bin"
sha256sum "$compiler_bin"
