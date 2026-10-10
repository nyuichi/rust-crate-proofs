#!/usr/bin/env bash
set -euo pipefail

script_dir=$(cd "$(dirname "$0")" && pwd)
bundle_dir=$(cd "$script_dir/.." && pwd)
repo_root=$(cd "$bundle_dir/../.." && pwd)
compiler_repo=/workspace/proof-tools/creusot-source
scratch=/workspace/scratch/httparse-derived-traits-nativeenv
source_dir=$scratch/build-source
libs_dir=$scratch/creusot-libs
libs_seed=/workspace/scratch/httparse-derived-traits/creusot-libs
frozen_libs_seed=$repo_root/httparse/1.10.1/verification/probes/derived-traits-nativeenv/evidence/nativeenv-full-checkpoint/toolchain/stdlib
fallback_libs_seed=$scratch/creusot-libs-frozen-seed
target_dir=/workspace/proof-tools/targets/httparse-derived-traits-nativeenv
isolated_data=$scratch/creusot-data
isolated_config=$scratch/config
isolated_cache=$scratch/cache
active_data=/workspace/proof-tools/creusot-data
why3_conf=$scratch/why3.conf
pinned=437d3d8d00b8114d7a3b4f7b8738d594a395f5bc
narrowcast_patch=$bundle_dir/patches/creusot-narrowcast-backend.patch
string_patch=$bundle_dir/patches/httparse-string-model.patch
derived_patch=$bundle_dir/patches/httparse-derived-trait-discriminant.patch
nativeenv_patch=$bundle_dir/patches/httparse-derived-trait-nativeenv.patch
stdlib_patch=$bundle_dir/patches/httparse-string-std.patch

expected_narrowcast_sha=c3ac3e596b822f483d2f0f782169aacc5c752449cf66771ee7268ee2feb8eb24
expected_string_sha=8b3205493c45b909c8fb980b3e6cb3c4f63d52511060f12ec227c4e105d4f2ea
expected_derived_sha=90bed76ac9626b5738ee55a6dfbdf909c6e972ba40063db5ca6e296e8c8dedc4
expected_nativeenv_sha=b6aca857fd852e64aa52090a356cf7da8cb4015a95670804d760f56691f3420a
expected_std_sha=7b92f2d54dc845a982246c220bc37004ae14d6ee5e51ff0ff4bbc87eb90b5ab9
expected_seed_num_sha=b777d9fb2cc922148df8718b9a175d249de2174fc5d57f22cf24a63af91256d2
expected_seed_string_sha=e51e9dd373e683c63c424aca31bb4850f0437e1c5c56c325c8590e46670957df
expected_seed_convert_sha=5a4346a05298dbf71fe16b35718ac57552105426a4e4cc9161176b4bf3ae5348

check_hash() {
  local expected=$1 path=$2 label=$3 actual
  actual=$(sha256sum "$path" | cut -d ' ' -f 1)
  if [[ "$actual" != "$expected" ]]; then
    printf '%s hash mismatch: expected %s, found %s\n' "$label" "$expected" "$actual" >&2
    exit 1
  fi
}

seed_triplet_matches() {
  local root=$1 actual
  [[ -f "$root/creusot-std/src/std/num.rs" ]] || return 1
  [[ -f "$root/creusot-std/src/std/string.rs" ]] || return 1
  [[ -f "$root/creusot-std/src/std/convert.rs" ]] || return 1
  actual=$(sha256sum "$root/creusot-std/src/std/num.rs" | cut -d ' ' -f 1)
  [[ "$actual" == "$expected_seed_num_sha" ]] || return 1
  actual=$(sha256sum "$root/creusot-std/src/std/string.rs" | cut -d ' ' -f 1)
  [[ "$actual" == "$expected_seed_string_sha" ]] || return 1
  actual=$(sha256sum "$root/creusot-std/src/std/convert.rs" | cut -d ' ' -f 1)
  [[ "$actual" == "$expected_seed_convert_sha" ]] || return 1
}

check_hash "$expected_narrowcast_sha" "$narrowcast_patch" "narrowcast patch"
check_hash "$expected_string_sha" "$string_patch" "string compiler patch"
check_hash "$expected_derived_sha" "$derived_patch" "derived-trait patch"
check_hash "$expected_nativeenv_sha" "$nativeenv_patch" "derived-trait native-env patch"
check_hash "$expected_std_sha" "$stdlib_patch" "string standard-library patch"

actual_pinned=$(git -C "$compiler_repo" rev-parse HEAD)
if [[ "$actual_pinned" != "$pinned" ]]; then
  printf 'Expected pinned Creusot source %s, found %s\n' "$pinned" "$actual_pinned" >&2
  exit 1
fi

if [[ ! -d "$source_dir" ]]; then
  mkdir -p "$source_dir"
  git -C "$compiler_repo" archive "$pinned" | tar -x -C "$source_dir"
  git -C "$source_dir" apply "$narrowcast_patch"
  git -C "$source_dir" apply "$string_patch"
  git -C "$source_dir" apply "$derived_patch"
  git -C "$source_dir" apply "$nativeenv_patch"
else
  git -C "$source_dir" apply --reverse --check "$narrowcast_patch" >/dev/null
  git -C "$source_dir" apply --reverse --check "$string_patch" >/dev/null
  git -C "$source_dir" apply --reverse --check "$derived_patch" >/dev/null
  git -C "$source_dir" apply --reverse --check "$nativeenv_patch" >/dev/null
fi

if seed_triplet_matches "$libs_seed"; then
  selected_libs_seed=$libs_seed
  seed_origin=existing-derived-traits-seed
else
  printf 'Existing isolated library seed is absent or differs from the pinned num/string/convert inputs; rebuilding it from frozen stdlib files.\n'
  check_hash "$expected_seed_num_sha" "$frozen_libs_seed/num.rs" "frozen creusot-std num.rs"
  check_hash "$expected_seed_string_sha" "$frozen_libs_seed/string.rs" "frozen creusot-std string.rs"
  check_hash "$expected_seed_convert_sha" "$frozen_libs_seed/convert.rs" "frozen creusot-std convert.rs"
  if [[ ! -d "$repo_root/creusot-libs" ]]; then
    printf 'Cannot rebuild the isolated library seed: %s is missing.\n' "$repo_root/creusot-libs" >&2
    exit 1
  fi
  rm -rf "$fallback_libs_seed"
  cp -a "$repo_root/creusot-libs" "$fallback_libs_seed"
  cp "$frozen_libs_seed/num.rs" "$fallback_libs_seed/creusot-std/src/std/num.rs"
  cp "$frozen_libs_seed/string.rs" "$fallback_libs_seed/creusot-std/src/std/string.rs"
  cp "$frozen_libs_seed/convert.rs" "$fallback_libs_seed/creusot-std/src/std/convert.rs"
  selected_libs_seed=$fallback_libs_seed
  seed_origin=frozen-nativeenv-toolchain-stdlib
fi

if ! seed_triplet_matches "$libs_dir"; then
  rm -rf "$libs_dir"
  cp -a "$selected_libs_seed" "$libs_dir"
fi
check_hash "$expected_seed_num_sha" "$libs_dir/creusot-std/src/std/num.rs" "isolated creusot-std num.rs"
check_hash "$expected_seed_string_sha" "$libs_dir/creusot-std/src/std/string.rs" "isolated creusot-std string.rs"
check_hash "$expected_seed_convert_sha" "$libs_dir/creusot-std/src/std/convert.rs" "isolated creusot-std convert.rs"

source /workspace/proof-tools/activate.sh
export RUSTUP_TOOLCHAIN=nightly-2026-02-27
export CARGO_NET_OFFLINE=true
export CARGO_TARGET_DIR=$target_dir
mkdir -p "$target_dir"

generated=$source_dir/target/creusot/packages/creusot/creusot
rm -rf "$generated"
cargo run --offline --manifest-path "$source_dir/prelude-generator/Cargo.toml"
test -f "$generated/prelude.coma"

mkdir -p "$isolated_data"
for item in bin _opam toolchains; do
  if [[ ! -e "$isolated_data/$item" ]]; then
    ln -s "$active_data/$item" "$isolated_data/$item"
  fi
done
if [[ ! -d "$isolated_data/share" ]]; then
  cp -a "$active_data/share" "$isolated_data/share"
fi
cp "$active_data/why3find.json" "$isolated_data/why3find.json"
isolated_package=$isolated_data/share/why3find/packages/creusot/creusot
rm -rf "$isolated_package"
mkdir -p "$(dirname "$isolated_package")"
cp -a "$generated" "$isolated_package"
cp /workspace/proof-tools/config/creusot/why3.conf "$why3_conf"
mkdir -p "$isolated_config/creusot" "$isolated_cache"
cp "$why3_conf" "$isolated_config/creusot/why3.conf"

cargo build --offline --manifest-path "$source_dir/Cargo.toml" -p creusot-rustc
compiler_bin=$target_dir/debug/creusot-rustc
test -x "$compiler_bin"

cat > "$scratch/creusot-env.sh" <<EOF
export CREUSOT_DATA_HOME=$isolated_data
export CREUSOT_RUSTC=$compiler_bin
export WHY3CONFIG=$why3_conf
export CARGO_TARGET_DIR=$target_dir
export CARGO_NET_OFFLINE=true
export DUNE_DIR_LOCATIONS=why3find:lib:$isolated_data/share/why3find
export XDG_CONFIG_HOME=$isolated_config
export XDG_CACHE_HOME=$isolated_cache
EOF

printf 'Pinned source: %s\n' "$pinned"
printf 'narrowcast patch: %s\n' "$expected_narrowcast_sha"
printf 'string compiler patch: %s\n' "$expected_string_sha"
printf 'derived-trait patch: %s\n' "$expected_derived_sha"
printf 'native-env patch: %s\n' "$expected_nativeenv_sha"
printf 'string stdlib patch: %s\n' "$expected_std_sha"
printf 'isolated compiler: %s\n' "$compiler_bin"
printf 'isolated libraries: %s\n' "$libs_dir"
printf 'library seed origin: %s (%s)\n' "$seed_origin" "$selected_libs_seed"
printf 'library seed (num/string/convert): %s %s %s\n' \
  "$expected_seed_num_sha" "$expected_seed_string_sha" "$expected_seed_convert_sha"
printf 'generated prelude: %s\n' "$generated/prelude.coma"
printf 'loaded package path: %s\n' "$isolated_package"
printf 'isolated XDG config/cache: %s %s\n' "$isolated_config" "$isolated_cache"
sha256sum "$compiler_bin" "$generated/prelude.coma" "$isolated_package/prelude.coma" \
  "$libs_dir/creusot-std/src/std/num.rs" \
  "$libs_dir/creusot-std/src/std/string.rs" \
  "$libs_dir/creusot-std/src/std/convert.rs"
