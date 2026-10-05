#!/usr/bin/env bash
set -euo pipefail

probe_dir=$(cd "$(dirname "$0")" && pwd)
repo_root=$(git -C "$probe_dir" rev-parse --show-toplevel)
profile_dir=${HTTPARSE_TOOL_PROFILE:-/workspace/httparse-tool-rebuild/string-model}
mode=${1:-translate}
shift || true

source /workspace/proof-tools/activate.sh
source "$profile_dir/creusot-env.sh"
export RUSTUP_TOOLCHAIN=nightly-2026-02-27
export CARGO_NET_OFFLINE=true

work_dir="$profile_dir/method-utf8-harness"
mkdir -p "$work_dir"
sed -e "s|@METHOD_UTF8_PROBE@|$probe_dir|g" \
    -e "s|@HTTPARSE_CREUSOT_LIBS@|$HTTPARSE_CREUSOT_LIBS|g" \
    "$probe_dir/Cargo.toml.in" > "$work_dir/Cargo.toml"
cp "$probe_dir/why3find.json" "$work_dir/why3find.json"
cp "$probe_dir/proof-targets.txt" "$work_dir/proof-targets.txt"

case "$mode" in
  native)
    export CARGO_TARGET_DIR="$profile_dir/targets/method-utf8-native"
    cargo check --offline --manifest-path "$work_dir/Cargo.toml" "$@"
    ;;
  translate)
    run_id="$(date -u +%Y%m%dT%H%M%SZ)-$$"
    target_dir="$profile_dir/targets/method-utf8-translate-$run_id"
    export CARGO_TARGET_DIR=$target_dir
    rm -rf "$target_dir"
    rm -rf "$work_dir/verif"
    evidence="$probe_dir/evidence/translation-$run_id"
    mkdir -p "$evidence/coma"
    cp "$work_dir/Cargo.toml" "$evidence/"
    cp "$work_dir/proof-targets.txt" "$evidence/"
    cp "$probe_dir/src/lib.rs" "$evidence/harness-lib.rs"
    cp "$repo_root/httparse/1.10.1/src/parse_method_utf8.rs" "$evidence/"
    cp "$HTTPARSE_CREUSOT_LIBS/creusot-std/src/std/string.rs" "$evidence/stdlib-string.rs"
    cp "$CREUSOT_DATA_HOME/share/why3find/packages/creusot/creusot/prelude.coma" "$evidence/prelude.coma"
    cp "$WHY3CONFIG" "$evidence/why3.conf"
    cp "$probe_dir/evidence/tool-rebuild-20261005/PROFILE.tsv" \
      "$probe_dir/evidence/tool-rebuild-20261005/creusot-libs.sha256" \
      "$evidence/"
    cp "$probe_dir/evidence/tool-rebuild-20261005/compiler-source.sha256" \
      "$probe_dir/evidence/tool-rebuild-20261005/prelude-package.sha256" \
      "$repo_root/tools/creusot-toolpatch/patches/creusot-narrowcast-backend.patch" \
      "$repo_root/tools/creusot-toolpatch/patches/httparse-string-model.patch" \
      "$repo_root/tools/creusot-toolpatch/patches/httparse-string-std.patch" \
      "$evidence/"
    printf 'run_id\t%s\ntarget_dir\t%s\n' "$run_id" "$target_dir" > "$evidence/environment.tsv"
    sha256sum "$work_dir/Cargo.toml" "$probe_dir/src/lib.rs" \
      "$repo_root/httparse/1.10.1/src/parse_method_utf8.rs" \
      "$HTTPARSE_CREUSOT_LIBS/creusot-std/src/std/string.rs" \
      "$CREUSOT_DATA_HOME/share/why3find/packages/creusot/creusot/prelude.coma" \
      > "$evidence/SHA256SUMS"
    printf 'cargo creusot -- --manifest-path %s/Cargo.toml\n' "$work_dir" > "$evidence/command.txt"
    cd "$work_dir"
    set +e
    cargo creusot -- --manifest-path "$work_dir/Cargo.toml" "$@" \
      > "$evidence/stdout.log" 2> "$evidence/stderr.log"
    status=$?
    set -e
    printf '%s\n' "$status" > "$evidence/exit-code.txt"
    if (( status != 0 )); then exit "$status"; fi
    cp "$work_dir/Cargo.lock" "$evidence/Cargo.lock"
    mapfile -t actual_targets < <(find "$work_dir/verif" -type f -name '*.coma' -printf 'verif/%P\n' | sort)
    mapfile -t expected_targets < <(sed '/^[[:space:]]*#/d; /^[[:space:]]*$/d' "$work_dir/proof-targets.txt" | sort)
    if [[ "${actual_targets[*]}" != "${expected_targets[*]}" ]]; then
      printf 'Translated COMA inventory differs from proof-targets.txt\n' > "$evidence/inventory-error.txt"
      printf 'actual: %s\nexpected: %s\n' "${actual_targets[*]}" "${expected_targets[*]}" >> "$evidence/inventory-error.txt"
      exit 2
    fi
    find "$work_dir/verif/httparse_method_utf8_harness_rlib" -type f -name '*.coma' -print0 | \
      sort -z | while IFS= read -r -d '' file; do cp "$file" "$evidence/coma/"; done
    find "$evidence/coma" -type f -print0 | sort -z | xargs -0 sha256sum > "$evidence/coma.sha256"
    printf '%s\n' "$evidence" > "$work_dir/translation.latest"
    printf 'translation evidence: %s\n' "$evidence"
    ;;
  typecheck)
    evidence=$(cat "$work_dir/translation.latest")
    mapfile -t targets < <(sed '/^[[:space:]]*#/d; /^[[:space:]]*$/d' "$evidence/proof-targets.txt")
    files=()
    for target in "${targets[@]}"; do
      test -f "$work_dir/$target"
      files+=("$work_dir/$target")
    done
    set +e
    "$CREUSOT_DATA_HOME/bin/why3" prove --type-only -C "$WHY3CONFIG" \
      -L "$CREUSOT_DATA_HOME/share/why3find/packages/creusot" \
      -L "$work_dir/verif" "${files[@]}" "$@" \
      > "$evidence/typecheck.stdout.log" 2> "$evidence/typecheck.stderr.log"
    status=$?
    set -e
    printf '%s\n' "$status" > "$evidence/typecheck.exit-code.txt"
    exit "$status"
    ;;
  prove-helper|prove-caller)
    target=method_from_bytes
    if [[ "$mode" == prove-caller ]]; then target=map_method_outcome; fi
    target_file="$work_dir/verif/httparse_method_utf8_harness_rlib/$target.coma"
    test -f "$target_file"
    evidence="$probe_dir/evidence/proof-$(date -u +%Y%m%dT%H%M%SZ)-$target"
    mkdir -p "$evidence"
    cp "$target_file" "$evidence/"
    cp "$work_dir/Cargo.toml" "$probe_dir/src/lib.rs" \
      "$repo_root/httparse/1.10.1/src/parse_method_utf8.rs" \
      "$HTTPARSE_CREUSOT_LIBS/creusot-std/src/std/string.rs" \
      "$CREUSOT_DATA_HOME/share/why3find/packages/creusot/creusot/prelude.coma" \
      "$evidence/"
    cp "$work_dir/Cargo.lock" "$WHY3CONFIG" \
      "$probe_dir/evidence/tool-rebuild-20261005/PROFILE.tsv" \
      "$probe_dir/evidence/tool-rebuild-20261005/creusot-libs.sha256" \
      "$probe_dir/evidence/tool-rebuild-20261005/compiler-source.sha256" \
      "$probe_dir/evidence/tool-rebuild-20261005/prelude-package.sha256" \
      "$repo_root/tools/creusot-toolpatch/patches/creusot-narrowcast-backend.patch" \
      "$repo_root/tools/creusot-toolpatch/patches/httparse-string-model.patch" \
      "$repo_root/tools/creusot-toolpatch/patches/httparse-string-std.patch" \
      "$evidence/"
    why3cmd=("$CREUSOT_DATA_HOME/bin/why3" prove -C "$WHY3CONFIG"
      -L "$CREUSOT_DATA_HOME/share/why3find/packages/creusot"
      -L "$work_dir/verif" "$target_file" -a split_vc -P Z3,4.15.3 -t 30 -m 1000 --json)
    printf '%q ' "${why3cmd[@]}" > "$evidence/command.txt"
    printf '\n' >> "$evidence/command.txt"
    set +e
    "$repo_root/httparse/1.10.1/run-proof.bash" \
      env CREUSOT_DATA_HOME="$CREUSOT_DATA_HOME" CREUSOT_RUSTC="$CREUSOT_RUSTC" \
      WHY3CONFIG="$WHY3CONFIG" DUNE_DIR_LOCATIONS="$DUNE_DIR_LOCATIONS" \
      XDG_CONFIG_HOME="$XDG_CONFIG_HOME" XDG_CACHE_HOME="$XDG_CACHE_HOME" \
      "${why3cmd[@]}" > "$evidence/stdout.log" 2> "$evidence/stderr.log"
    status=$?
    set -e
    printf '%s\n' "$status" > "$evidence/exit-code.txt"
    exit "$status"
    ;;
  *)
    printf 'Usage: %s [native|translate|typecheck|prove-helper|prove-caller] [flags...]\n' "$0" >&2
    exit 2
    ;;
esac
