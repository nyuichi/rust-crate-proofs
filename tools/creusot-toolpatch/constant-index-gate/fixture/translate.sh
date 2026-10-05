#!/usr/bin/env bash
set -euo pipefail

fixture_dir=$(cd -- "$(dirname "$0")" && pwd)
repo_root=$(git -C "$fixture_dir" rev-parse --show-toplevel)
profile_dir=${HTTPARSE_TOOL_PROFILE:-/workspace/httparse-tool-rebuild/string-model}
constant_index_root=${CREUSOT_CONSTANT_INDEX_TARGET_ROOT:-/workspace/httparse-tool-rebuild/constant-index-gate}
run_root=$(cat "$constant_index_root/latest-build")
compiler_bin=$run_root/target/debug/creusot-rustc
fixture_root=${CREUSOT_CONSTANT_INDEX_FIXTURE_ROOT:-$fixture_dir}

test -x "$compiler_bin"
source /workspace/proof-tools/activate.sh
source "$profile_dir/creusot-env.sh"
libs=${HTTPARSE_CREUSOT_LIBS:-$profile_dir/creusot-libs}
export RUSTUP_TOOLCHAIN=nightly-2026-02-27
export CARGO_NET_OFFLINE=true
export CREUSOT_RUSTC=$compiler_bin

run_id=$(date -u +%Y%m%dT%H%M%SZ)-$$
work_dir="$profile_dir/constant-index-gate-$run_id"
target_dir="$profile_dir/targets/constant-index-gate-$run_id"
evidence="$fixture_dir/evidence/translation-$run_id"
mkdir -p "$work_dir" "$target_dir" "$evidence/coma"
sed -e "s|@FIXTURE_ROOT@|$fixture_root|g" \
  -e "s|@CREUSOT_LIBS@|$libs|g" \
  "$fixture_dir/Cargo.toml.in" > "$work_dir/Cargo.toml"
cp "$fixture_dir/why3find.json" "$work_dir/why3find.json"
cp "$fixture_dir/proof-targets.txt" "$work_dir/proof-targets.txt"
cp "$work_dir/Cargo.toml" "$evidence/Cargo.toml"
cp "$fixture_dir/src/lib.rs" "$evidence/lib.rs"
cp "$run_root/toolchain.tsv" "$evidence/toolchain.tsv"
printf 'run_id\t%s\nwork_dir\t%s\ntarget_dir\t%s\ncompiler_bin\t%s\n' \
  "$run_id" "$work_dir" "$target_dir" "$compiler_bin" > "$evidence/environment.tsv"
sha256sum "$work_dir/Cargo.toml" "$fixture_dir/src/lib.rs" \
  "$compiler_bin" \
  "$CREUSOT_DATA_HOME/share/why3find/packages/creusot/creusot/prelude.coma" \
  > "$evidence/SHA256SUMS"
cd "$work_dir"
export CARGO_TARGET_DIR=$target_dir
set +e
cargo creusot --no-check-version --simple-triggers=false -- \
  --manifest-path "$work_dir/Cargo.toml" \
  > "$evidence/stdout.log" 2> "$evidence/stderr.log"
status=$?
set -e
printf '%s\n' "$status" > "$evidence/exit-code.txt"
if (( status != 0 )); then
  printf '%s\n' "$evidence" > "$fixture_dir/translation.latest"
  exit "$status"
fi
cp "$work_dir/Cargo.lock" "$evidence/Cargo.lock"
mapfile -t targets < <(sed '/^[[:space:]]*#/d; /^[[:space:]]*$/d' "$work_dir/proof-targets.txt")
for target in "${targets[@]}"; do
  test -f "$work_dir/$target"
  cp "$work_dir/$target" "$evidence/coma/$(basename "$target")"
done
find "$evidence/coma" -type f -print0 | sort -z | xargs -0 sha256sum > "$evidence/coma.sha256"
printf '%s\n' "$evidence" > "$fixture_dir/translation.latest"
printf 'translation evidence: %s\n' "$evidence"
