#!/usr/bin/env bash
set -euo pipefail

script_dir=$(cd "$(dirname "$0")" && pwd)
repo_root=$(git -C "$script_dir" rev-parse --show-toplevel)
bundle_dir=$repo_root/tools/creusot-toolpatch
proof_tools=/workspace/proof-tools
compiler_repo=$proof_tools/creusot-source
opam_sources=$proof_tools/creusot-data/_opam/.opam-switch/sources
rebuild_root=${HTTPARSE_REBUILD_ROOT:-/workspace/httparse-tool-rebuild}
profile_dir=$rebuild_root/string-model
source_dir=$profile_dir/compiler-source
libs_dir=$profile_dir/creusot-libs
data_dir=$profile_dir/creusot-data
config_dir=$profile_dir/config
cache_dir=$profile_dir/cache
target_dir=$rebuild_root/targets/creusot-cast-compdiv
why3_conf=$profile_dir/why3.conf
baseline_builder=$bundle_dir/scripts/build-creusot-rustc.sh
evidence_dir=$repo_root/httparse/1.10.1/verification/probes/method-utf8/evidence/tool-rebuild-20261005

creusot_commit=437d3d8d00b8114d7a3b4f7b8738d594a395f5bc
stdlib_seed_commit=6263082
stdlib_seed_tree=daf48d3435a26fa967e3e5727c0f386515a44002
narrowcast_patch=$bundle_dir/patches/creusot-narrowcast-backend.patch
string_patch=$bundle_dir/patches/httparse-string-model.patch
stdlib_patch=$bundle_dir/patches/httparse-string-std.patch
newline_manifest=$repo_root/httparse/1.10.1/verification/probes/newline-harness/evidence/translation-20261005/INPUTS-SHA256SUMS
expected_narrowcast_sha=c3ac3e596b822f483d2f0f782169aacc5c752449cf66771ee7268ee2feb8eb24
expected_string_sha=8b3205493c45b909c8fb980b3e6cb3c4f63d52511060f12ec227c4e105d4f2ea
expected_std_sha=7b92f2d54dc845a982246c220bc37004ae14d6ee5e51ff0ff4bbc87eb90b5ab9
why3_commit=2c0f2992af85f82f3eda0f158dcf10e62e0db875
why3find_commit=3a98fc320b9cbf2e71860da1c8dc188a966eee96
why3_source=$opam_sources/why3
why3find_source=$opam_sources/why3find

check_hash() {
  local expected=$1 path=$2 label=$3 actual
  actual=$(sha256sum "$path" | cut -d ' ' -f 1)
  if [[ "$actual" != "$expected" ]]; then
    printf '%s SHA-256 mismatch: expected %s, found %s\n' "$label" "$expected" "$actual" >&2
    exit 1
  fi
}

check_hash "$expected_narrowcast_sha" "$narrowcast_patch" "narrowcast compiler patch"
check_hash "$expected_string_sha" "$string_patch" "string compiler patch"
check_hash "$expected_std_sha" "$stdlib_patch" "string standard-library patch"

actual_creusot_commit=$(git -C "$compiler_repo" rev-parse HEAD)
if [[ "$actual_creusot_commit" != "$creusot_commit" ]]; then
  printf 'Expected clean Creusot checkout at %s, found %s\n' "$creusot_commit" "$actual_creusot_commit" >&2
  exit 1
fi
if [[ -n $(git -C "$compiler_repo" status --porcelain) ]]; then
  printf 'Pinned Creusot checkout has uncommitted changes: %s\n' "$compiler_repo" >&2
  exit 1
fi
actual_seed_tree=$(git -C "$repo_root" rev-parse "$stdlib_seed_commit:creusot-libs")
if [[ "$actual_seed_tree" != "$stdlib_seed_tree" ]]; then
  printf 'Expected frozen creusot-libs tree %s, found %s\n' "$stdlib_seed_tree" "$actual_seed_tree" >&2
  exit 1
fi
actual_why3_commit=$(git -C "$why3_source" rev-parse HEAD)
actual_why3find_commit=$(git -C "$why3find_source" rev-parse HEAD)
if [[ "$actual_why3_commit" != "$why3_commit" || -n $(git -C "$why3_source" status --porcelain) ]]; then
  printf 'Why3 source does not match clean pinned commit %s\n' "$why3_commit" >&2
  exit 1
fi
if [[ "$actual_why3find_commit" != "$why3find_commit" || -n $(git -C "$why3find_source" status --porcelain) ]]; then
  printf 'why3find source does not match clean pinned commit %s\n' "$why3find_commit" >&2
  exit 1
fi

mkdir -p "$profile_dir" "$evidence_dir"

if [[ ! -d "$source_dir" ]]; then
  mkdir -p "$source_dir"
  git -C "$compiler_repo" archive "$creusot_commit" | tar -x -C "$source_dir"
  git -C "$source_dir" apply "$narrowcast_patch"
  git -C "$source_dir" apply "$string_patch"
else
  git -C "$source_dir" apply --reverse --check "$narrowcast_patch" >/dev/null
  git -C "$source_dir" apply --reverse --check "$string_patch" >/dev/null
fi

if [[ ! -d "$libs_dir" ]]; then
  mkdir -p "$libs_dir"
  git -C "$repo_root" archive "$stdlib_seed_commit" creusot-libs | \
    tar -x --strip-components=1 -C "$libs_dir"
  patch -p2 -d "$libs_dir" < "$stdlib_patch"
else
  patch -R --dry-run -p2 -d "$libs_dir" < "$stdlib_patch" >/dev/null
fi

match_manifest=$evidence_dir/newline-stdlib-match.tsv
printf 'path\texpected_sha256\tactual_sha256\tresult\n' > "$match_manifest"
matched_library_files=0
while read -r expected archived_path; do
  case "$archived_path" in
    scratch/httparse-string-model/creusot-libs/*)
      rel_path=${archived_path#scratch/httparse-string-model/creusot-libs/}
      actual=$(sha256sum "$libs_dir/$rel_path" | cut -d ' ' -f 1)
      if [[ "$actual" != "$expected" ]]; then
        printf 'New stdlib source mismatch at %s: expected %s, found %s\n' \
          "$rel_path" "$expected" "$actual" >&2
        exit 1
      fi
      printf '%s\t%s\t%s\tmatch\n' "$rel_path" "$expected" "$actual" >> "$match_manifest"
      matched_library_files=$((matched_library_files + 1))
      ;;
  esac
done < "$newline_manifest"
if [[ "$matched_library_files" -ne 22 ]]; then
  printf 'Expected 22 frozen newline library inputs, matched %s\n' "$matched_library_files" >&2
  exit 1
fi

source "$proof_tools/activate.sh"
export OPAMSWITCH=$proof_tools/creusot-data
export RUSTUP_TOOLCHAIN=nightly-2026-02-27
export CARGO_NET_OFFLINE=true
export CARGO_TARGET_DIR=$target_dir
mkdir -p "$target_dir"

# The generator's output is timestamp-copied into a package under source_dir.
# Clear only this isolated generated package to prevent a stale prelude.
generated=$source_dir/target/creusot/packages/creusot/creusot
rm -rf "$generated"
cargo run --offline --manifest-path "$source_dir/prelude-generator/Cargo.toml"
test -s "$generated/prelude.coma"

active_data=$proof_tools/creusot-data
mkdir -p "$data_dir"
for item in bin _opam toolchains; do
  if [[ ! -e "$data_dir/$item" ]]; then
    ln -s "$active_data/$item" "$data_dir/$item"
  fi
done
if [[ ! -d "$data_dir/share" ]]; then
  cp -a "$active_data/share" "$data_dir/share"
fi
cp "$active_data/why3find.json" "$data_dir/why3find.json"
isolated_package=$data_dir/share/why3find/packages/creusot/creusot
rm -rf "$isolated_package"
mkdir -p "$(dirname "$isolated_package")"
cp -a "$generated" "$isolated_package"
cp "$proof_tools/config/creusot/why3.conf" "$why3_conf"
mkdir -p "$config_dir/creusot" "$cache_dir"
cp "$why3_conf" "$config_dir/creusot/why3.conf"

cargo build --offline --manifest-path "$source_dir/Cargo.toml" -p creusot-rustc
compiler_bin=$target_dir/debug/creusot-rustc
test -x "$compiler_bin"

cat > "$profile_dir/creusot-env.sh" <<EOF
source $proof_tools/activate.sh
export OPAMSWITCH=$proof_tools/creusot-data
export RUSTUP_TOOLCHAIN=nightly-2026-02-27
export CREUSOT_DATA_HOME=$data_dir
export CREUSOT_RUSTC=$compiler_bin
export WHY3CONFIG=$why3_conf
export CARGO_TARGET_DIR=$target_dir
export CARGO_NET_OFFLINE=true
export DUNE_DIR_LOCATIONS=why3find:lib:$data_dir/share/why3find
export XDG_CONFIG_HOME=$config_dir
export XDG_CACHE_HOME=$cache_dir
export HTTPARSE_CREUSOT_LIBS=$libs_dir
export HTTPARSE_TOOL_PROFILE=$profile_dir
EOF

find "$libs_dir" -type f -print0 | sort -z | xargs -0 sha256sum > "$evidence_dir/creusot-libs.sha256"
find "$source_dir" -type f -not -path "$source_dir/target/*" -print0 | \
  sort -z | xargs -0 sha256sum > "$evidence_dir/compiler-source.sha256"
find "$isolated_package" -type f -print0 | sort -z | xargs -0 sha256sum > \
  "$evidence_dir/prelude-package.sha256"

printf 'key\tvalue\n' > "$evidence_dir/PROFILE.tsv"
printf 'profile_root\t%s\n' "$profile_dir" >> "$evidence_dir/PROFILE.tsv"
printf 'compiler_source_commit\t%s\n' "$creusot_commit" >> "$evidence_dir/PROFILE.tsv"
printf 'compiler_upstream_source_tree\t%s\n' "$(git -C "$compiler_repo" rev-parse "$creusot_commit^{tree}")" >> "$evidence_dir/PROFILE.tsv"
printf 'compiler_patched_source_manifest_sha256\t%s\n' "$(sha256sum "$evidence_dir/compiler-source.sha256" | cut -d ' ' -f 1)" >> "$evidence_dir/PROFILE.tsv"
printf 'compiler_binary\t%s\n' "$compiler_bin" >> "$evidence_dir/PROFILE.tsv"
printf 'compiler_binary_sha256\t%s\n' "$(sha256sum "$compiler_bin" | cut -d ' ' -f 1)" >> "$evidence_dir/PROFILE.tsv"
printf 'rust_toolchain\t%s\n' "$(rustc --version --verbose | tr '\n' ';')" >> "$evidence_dir/PROFILE.tsv"
printf 'rustc_sysroot_source_commit\t%s\n' 6a979b3e32522049d0acb4a47f7ae44b7c8abfd5 >> "$evidence_dir/PROFILE.tsv"
printf 'target_directory\t%s\n' "$target_dir" >> "$evidence_dir/PROFILE.tsv"
printf 'target_directory_reused\t%s\n' yes >> "$evidence_dir/PROFILE.tsv"
printf 'target_directory_previous_profile\t%s\n' "${HTTPARSE_PREVIOUS_TARGET_PROFILE:-unrecorded}" >> "$evidence_dir/PROFILE.tsv"
printf 'target_directory_previous_binary_sha256\t%s\n' "${HTTPARSE_PREVIOUS_BINARY_SHA256:-unrecorded}" >> "$evidence_dir/PROFILE.tsv"
printf 'target_directory_previous_builder_sha256\t%s\n' "$(sha256sum "$baseline_builder" | cut -d ' ' -f 1)" >> "$evidence_dir/PROFILE.tsv"
printf 'narrowcast_patch_sha256\t%s\n' "$expected_narrowcast_sha" >> "$evidence_dir/PROFILE.tsv"
printf 'string_compiler_patch_sha256\t%s\n' "$expected_string_sha" >> "$evidence_dir/PROFILE.tsv"
printf 'stdlib_patch_sha256\t%s\n' "$expected_std_sha" >> "$evidence_dir/PROFILE.tsv"
printf 'stdlib_seed_commit\t%s\n' "$stdlib_seed_commit" >> "$evidence_dir/PROFILE.tsv"
printf 'stdlib_seed_tree\t%s\n' "$stdlib_seed_tree" >> "$evidence_dir/PROFILE.tsv"
printf 'stdlib_seed_num_sha256\t%s\n' "$(git -C "$repo_root" show "$stdlib_seed_commit:creusot-libs/creusot-std/src/std/num.rs" | sha256sum | cut -d ' ' -f 1)" >> "$evidence_dir/PROFILE.tsv"
printf 'stdlib_result_manifest_sha256\t%s\n' "$(sha256sum "$evidence_dir/creusot-libs.sha256" | cut -d ' ' -f 1)" >> "$evidence_dir/PROFILE.tsv"
printf 'newline_manifest_library_files_matched\t%s\n' "$matched_library_files" >> "$evidence_dir/PROFILE.tsv"
printf 'newline_manifest_library_match_sha256\t%s\n' "$(sha256sum "$match_manifest" | cut -d ' ' -f 1)" >> "$evidence_dir/PROFILE.tsv"
printf 'stdlib_result_string_sha256\t%s\n' "$(sha256sum "$libs_dir/creusot-std/src/std/string.rs" | cut -d ' ' -f 1)" >> "$evidence_dir/PROFILE.tsv"
printf 'stdlib_result_num_sha256\t%s\n' "$(sha256sum "$libs_dir/creusot-std/src/std/num.rs" | cut -d ' ' -f 1)" >> "$evidence_dir/PROFILE.tsv"
printf 'prelude_package_manifest_sha256\t%s\n' "$(sha256sum "$evidence_dir/prelude-package.sha256" | cut -d ' ' -f 1)" >> "$evidence_dir/PROFILE.tsv"
printf 'prelude_sha256\t%s\n' "$(sha256sum "$generated/prelude.coma" | cut -d ' ' -f 1)" >> "$evidence_dir/PROFILE.tsv"
printf 'installed_prelude_sha256\t%s\n' "$(sha256sum "$isolated_package/prelude.coma" | cut -d ' ' -f 1)" >> "$evidence_dir/PROFILE.tsv"
printf 'why3_config_sha256\t%s\n' "$(sha256sum "$why3_conf" | cut -d ' ' -f 1)" >> "$evidence_dir/PROFILE.tsv"
printf 'why3_source_commit\t%s\n' "$actual_why3_commit" >> "$evidence_dir/PROFILE.tsv"
printf 'why3_source_tree\t%s\n' "$(git -C "$why3_source" rev-parse "$why3_commit^{tree}")" >> "$evidence_dir/PROFILE.tsv"
printf 'why3_opam_package_sha256\t%s\n' "$(sha256sum "$proof_tools/creusot-data/_opam/.opam-switch/packages/why3.git-2c0f2992/opam" | cut -d ' ' -f 1)" >> "$evidence_dir/PROFILE.tsv"
printf 'why3find_source_commit\t%s\n' "$actual_why3find_commit" >> "$evidence_dir/PROFILE.tsv"
printf 'why3find_source_tree\t%s\n' "$(git -C "$why3find_source" rev-parse "$why3find_commit^{tree}")" >> "$evidence_dir/PROFILE.tsv"
printf 'why3find_opam_package_sha256\t%s\n' "$(sha256sum "$proof_tools/creusot-data/_opam/.opam-switch/packages/why3find.git-3a98fc32/opam" | cut -d ' ' -f 1)" >> "$evidence_dir/PROFILE.tsv"
printf 'opam_why3_installed_package\t%s\n' "$(opam list --installed --columns=name,version | awk '$1 == "why3" {print $2}')" >> "$evidence_dir/PROFILE.tsv"
printf 'opam_why3find_installed_package\t%s\n' "$(opam list --installed --columns=name,version | awk '$1 == "why3find" {print $2}')" >> "$evidence_dir/PROFILE.tsv"
printf 'why3_binary\t%s\n' "$(command -v why3)" >> "$evidence_dir/PROFILE.tsv"
printf 'why3_version\t%s\n' "$(why3 --version)" >> "$evidence_dir/PROFILE.tsv"
printf 'why3_binary_sha256\t%s\n' "$(sha256sum "$(command -v why3)" | cut -d ' ' -f 1)" >> "$evidence_dir/PROFILE.tsv"
printf 'why3find_binary\t%s\n' "$(command -v why3find)" >> "$evidence_dir/PROFILE.tsv"
printf 'why3find_version\t%s\n' "$(why3find --version)" >> "$evidence_dir/PROFILE.tsv"
printf 'why3find_binary_sha256\t%s\n' "$(sha256sum "$(command -v why3find)" | cut -d ' ' -f 1)" >> "$evidence_dir/PROFILE.tsv"
printf 'z3_binary\t%s\n' "$(command -v z3)" >> "$evidence_dir/PROFILE.tsv"
printf 'z3_version\t%s\n' "$(z3 --version)" >> "$evidence_dir/PROFILE.tsv"
printf 'z3_binary_sha256\t%s\n' "$(sha256sum "$(command -v z3)" | cut -d ' ' -f 1)" >> "$evidence_dir/PROFILE.tsv"

cat > "$profile_dir/creusot-env.path" <<EOF
$profile_dir/creusot-env.sh
EOF

printf 'string profile ready\ncompiler=%s\nlibs=%s\nprelude=%s\nenv=%s\n' \
  "$compiler_bin" "$libs_dir" "$isolated_package/prelude.coma" "$profile_dir/creusot-env.sh"
cat "$evidence_dir/PROFILE.tsv"
