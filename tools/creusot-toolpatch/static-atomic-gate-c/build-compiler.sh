#!/usr/bin/env bash
set -euo pipefail

script_dir=$(cd -- "$(dirname "$0")" && pwd)
repo_root=$(git -C "$script_dir" rev-parse --show-toplevel)
patch_dir=$repo_root/tools/creusot-toolpatch/patches
source_repo=${CREUSOT_STATIC_ATOMIC_SOURCE_REPO:-/workspace/proof-tools/creusot-source}
target_dir=${CREUSOT_STATIC_ATOMIC_GATE_C_TARGET_DIR:-/workspace/proof-tools/targets/httparse-static-atomic-gate-c}
generated_package_dir=$target_dir/generated-prelude-package
package_manifest=$target_dir/generated-prelude-package.SHA256
pinned=437d3d8d00b8114d7a3b4f7b8738d594a395f5bc

narrowcast_patch=$patch_dir/creusot-narrowcast-backend.patch
gate_a_patch=$patch_dir/httparse-static-atomic-gate-a.patch
gate_b_patch=$patch_dir/httparse-static-atomic-gate-b.patch
gate_c_patch=$patch_dir/httparse-static-atomic-gate-c.patch

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
check_hash 87657e0e1d2448af0d6de1282623793d0fb24bc66448a7ff823a471b1f772bdf "$gate_c_patch" "Gate C patch"

if [[ ! -d "$source_repo/.git" && ! -f "$source_repo/.git" ]]; then
    printf 'Creusot source repository not found: %s\n' "$source_repo" >&2
    exit 1
fi
actual_pinned=$(git -C "$source_repo" rev-parse --verify "$pinned^{commit}")
if [[ "$actual_pinned" != "$pinned" ]]; then
    printf 'Pinned source commit mismatch: expected %s, found %s\n' "$pinned" "$actual_pinned" >&2
    exit 1
fi

source_dir=$(mktemp -d "${TMPDIR:-/tmp}/creusot-static-atomic-gate-c.XXXXXX")
package_stage=
cleanup() {
    [[ -z "$source_dir" ]] || rm -rf -- "$source_dir"
    [[ -z "$package_stage" ]] || rm -rf -- "$package_stage"
}
trap cleanup EXIT
git -C "$source_repo" archive "$pinned" | tar -x -C "$source_dir"
git -C "$source_dir" init --quiet

for patch in "$narrowcast_patch" "$gate_a_patch" "$gate_b_patch" "$gate_c_patch"; do
    git -C "$source_dir" apply --check "$patch"
    git -C "$source_dir" apply "$patch"
done

source /workspace/proof-tools/activate.sh
export RUSTUP_TOOLCHAIN=nightly-2026-02-27
export CARGO_NET_OFFLINE=true
export CARGO_TARGET_DIR=$target_dir
mkdir -p "$target_dir"
cargo build --offline --manifest-path "$source_dir/Cargo.toml" -p creusot-rustc
cargo run --offline --manifest-path "$source_dir/Cargo.toml" -p prelude-generator

compiler_bin=$target_dir/debug/creusot-rustc
prelude_file=$generated_package_dir/creusot/prelude.coma
generated_package_source=$source_dir/target/creusot/packages/creusot
test -x "$compiler_bin"
test -d "$generated_package_source"
package_parent=$(dirname -- "$generated_package_dir")
mkdir -p "$package_parent"
package_stage=$(mktemp -d "$package_parent/.generated-prelude-package.XXXXXX")
cp -a "$generated_package_source/." "$package_stage/"
rm -rf -- "$generated_package_dir"
mv -- "$package_stage" "$generated_package_dir"
package_stage=
test -f "$prelude_file"
grep -Fq 'function avx2_usable : bool' "$prelude_file"
grep -Fq 'function sse42_usable : bool' "$prelude_file"
mkdir -p "$target_dir"
(cd "$generated_package_dir" && find . -type f -print0 | sort -z | xargs -0 sha256sum) > "$package_manifest"
package_sha=$(sha256sum "$package_manifest" | awk '{print $1}')

printf 'Upstream source commit: %s\n' "$pinned"
printf 'narrowcast patch SHA-256: %s\n' "$(sha256sum "$narrowcast_patch" | awk '{print $1}')"
printf 'Gate A patch SHA-256: %s\n' "$(sha256sum "$gate_a_patch" | awk '{print $1}')"
printf 'Gate B patch SHA-256: %s\n' "$(sha256sum "$gate_b_patch" | awk '{print $1}')"
printf 'Gate C patch SHA-256: %s\n' "$(sha256sum "$gate_c_patch" | awk '{print $1}')"
printf 'compiler binary: %s\n' "$compiler_bin"
sha256sum "$compiler_bin"
printf 'generated package: %s\n' "$generated_package_dir"
printf 'generated package manifest: %s\n' "$package_manifest"
printf 'generated package SHA-256: %s\n' "$package_sha"
printf 'generated prelude: %s\n' "$prelude_file"
sha256sum "$prelude_file"
