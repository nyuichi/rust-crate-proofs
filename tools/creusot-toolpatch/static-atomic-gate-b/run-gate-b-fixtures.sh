#!/usr/bin/env bash
set -euo pipefail

script_dir=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)
repo_root=$(cd -- "$script_dir/../../.." && pwd)
fixture_source="$script_dir/fixture"
patch_file="$repo_root/tools/creusot-toolpatch/patches/httparse-static-atomic-gate-b.patch"
compiler_bin=${CREUSOT_STATIC_ATOMIC_RUSTC:-/workspace/proof-tools/targets/httparse-static-atomic-gate-b/debug/creusot-rustc}
run_root=${CREUSOT_STATIC_ATOMIC_RUN_ROOT:-/tmp/httparse-static-atomic-gate-b}
target=x86_64-unknown-linux-gnu
crate_name=static_atomic_gate_b_fixture

source /workspace/proof-tools/activate.sh
[[ -x "$compiler_bin" ]] || { echo "missing Gate B compiler: $compiler_bin" >&2; exit 2; }
[[ -f "$patch_file" ]] || { echo "missing Gate B patch: $patch_file" >&2; exit 2; }
mkdir -p "$run_root"
run_id=$(od -An -N32 -tx1 /dev/urandom | tr -d ' \n')
run_dir="$run_root/$run_id"
mkdir -p "$run_dir"

sha_file() { sha256sum "$1" | cut -d' ' -f1; }
sha_fixture() {
    (cd "$fixture_source" && sha256sum Cargo.toml Cargo.lock src/lib.rs) | sha256sum | cut -d' ' -f1
}
sha_text() { printf '%s' "$1" | sha256sum | cut -d' ' -f1; }

verify_inputs() {
    [[ $(sha_file "$fixture_source/src/lib.rs") == "$source_sha" ]]
    [[ $(sha_fixture) == "$fixture_sha" ]]
    [[ $(sha_file "$compiler_bin") == "$compiler_sha" ]]
    [[ $(sha_file "$patch_file") == "$patch_sha" ]]
    [[ $(sha_file "$script_dir/run-gate-b-fixtures.sh") == "$runner_sha" ]]
}

manifest_value() {
    awk -F '\t' -v key="$2" '$1 == key { print $2; exit }' "$1"
}

validate_report() {
    local report=$1 mode=$2 manifest=$3
    grep -Fxq $'creusot-static-atomic-audit-v2' "${report}"
    grep -Fxq $'mode\t'"$mode" "$report"
    for key in run_id normal_input_sha256 proof_input_sha256 compiler_sha256 patch_sha256 fixture_sha256 runner_sha256 source_sha256; do
        local expected
        expected=$(manifest_value "$manifest" "$key")
        grep -Fxq "${key}"$'\t'"${expected}" "$report"
    done
    grep -Fxq $'target\tx86_64-unknown-linux-gnu' "$report"
    grep -Fxq $'crate\tstatic_atomic_gate_b_fixture' "$report"
}

run_case() {
    local name=$1 features=$2 expected=$3
    local case_dir="$run_dir/$name"
    local package="$case_dir/fixture"
    local manifest="$case_dir/manifest.tsv"
    local normal_report="$case_dir/normal.audit"
    local proof_report="$case_dir/proof.audit"
    local source_sha fixture_sha compiler_sha patch_sha runner_sha normal_input proof_input
    local case_run_id
    local cargo_features=()
    mkdir -p "$package/src"
    cp "$fixture_source/src/lib.rs" "$package/src/lib.rs"
    cp "$fixture_source/Cargo.lock" "$package/Cargo.lock"
    sed "s#path = \"../../../../creusot-libs/creusot-std\"#path = \"$repo_root/creusot-libs/creusot-std\"#" \
        "$fixture_source/Cargo.toml" > "$package/Cargo.toml"
    source_sha=$(sha_file "$fixture_source/src/lib.rs")
    fixture_sha=$(sha_fixture)
    compiler_sha=$(sha_file "$compiler_bin")
    patch_sha=$(sha_file "$patch_file")
    runner_sha=$(sha_file "$script_dir/run-gate-b-fixtures.sh")
    case_run_id=$(od -An -N32 -tx1 /dev/urandom | tr -d ' \n')
    normal_input=$(sha_text "mode=normal-audit;crate=$crate_name;target=$target;features=$features;source=$source_sha;fixture=$fixture_sha;compiler=$compiler_sha;patch=$patch_sha;runner=$runner_sha;incoming_cfg=creusot;driver_strips_cfg=true")
    proof_input=$(sha_text "mode=proof-audit;crate=$crate_name;target=$target;features=$features;source=$source_sha;fixture=$fixture_sha;compiler=$compiler_sha;patch=$patch_sha;runner=$runner_sha;cargo_creusot=--no-check-version;cargo_offline=true;proof_cfg=creusot")
    {
        printf 'creusot-static-atomic-v2\n'
        printf 'crate\t%s\nmanifest_dir\t%s\ntarget\t%s\n' "$crate_name" "$package" "$target"
        printf 'source_sha256\t%s\nrun_id\t%s\n' "$source_sha" "$case_run_id"
        printf 'normal_input_sha256\t%s\nproof_input_sha256\t%s\n' "$normal_input" "$proof_input"
        printf 'compiler_sha256\t%s\npatch_sha256\t%s\nfixture_sha256\t%s\nrunner_sha256\t%s\n' \
            "$compiler_sha" "$patch_sha" "$fixture_sha" "$runner_sha"
        printf 'static\tCACHE\tvalue_invariant\n'
    } > "$manifest"
    [[ -z "$features" ]] || cargo_features=(--features "$features")
    verify_inputs
    (cd "$package" && \
        RUSTC="$compiler_bin" \
        RUSTFLAGS='--cfg=creusot' \
        CARGO_TARGET_DIR="$case_dir/normal-target" \
        CREUSOT_STATIC_ATOMIC_MODE=normal-audit \
        CREUSOT_STATIC_ATOMIC_MANIFEST="$manifest" \
        CREUSOT_STATIC_ATOMIC_OUTPUT="$normal_report" \
        CREUSOT_STATIC_ATOMIC_SOURCE_SHA256="$source_sha" \
        cargo check --offline "${cargo_features[@]}" >"$case_dir/normal.log" 2>&1)
    verify_inputs
    validate_report "$normal_report" Normal "$manifest"

    rm -f "$proof_report"
    if (cd "$package" && \
        CREUSOT_RUSTC="$compiler_bin" \
        CARGO_TARGET_DIR="$case_dir/proof-target" \
        CREUSOT_STATIC_ATOMIC_MODE=proof-audit \
        CREUSOT_STATIC_ATOMIC_MANIFEST="$manifest" \
        CREUSOT_STATIC_ATOMIC_OUTPUT="$proof_report" \
        CREUSOT_STATIC_ATOMIC_SOURCE_SHA256="$source_sha" \
        CREUSOT_STATIC_ATOMIC_GATE_B_NORMAL_AUDIT="$normal_report" \
        CREUSOT_STATIC_ATOMIC_GATE_B=1 \
        cargo creusot --no-check-version -- --offline "${cargo_features[@]}" >"$case_dir/proof.log" 2>&1); then
        if [[ "$expected" == reject ]]; then
            echo "FAIL: $name unexpectedly passed Gate B audit" >&2
            cat "$case_dir/proof.log" >&2
            exit 1
        fi
    else
        if [[ "$expected" != reject ]]; then
            echo "FAIL: $name translation failed unexpectedly" >&2
            cat "$case_dir/proof.log" >&2
            exit 1
        fi
        grep -Fq 'Gate B accessor' "$case_dir/proof.log"
        grep -Fq 'excluded, trusted, or has no body' "$case_dir/proof.log"
        [[ ! -e "$package/verif/static_atomic_gate_b_fixture_rlib/static-atomic-coverage.tsv" ]]
        printf 'PASS (fail-closed untranslated writer): %s\n' "$name"
        return
    fi
    verify_inputs
    validate_report "$proof_report" Proof "$manifest"
    diff -u <(grep '^access[[:space:]]' "$normal_report") <(grep '^access[[:space:]]' "$proof_report")

    local out="$package/verif/static_atomic_gate_b_fixture_rlib"
    local index="$out/static-atomic-coverage.tsv"
    [[ -f "$index" ]] || { echo "missing Gate B coverage index: $index" >&2; exit 1; }
    grep -Fxq $'creusot-static-atomic-coverage-v1' "$index"
    for key in run_id normal_input_sha256 proof_input_sha256 compiler_sha256 patch_sha256 fixture_sha256 runner_sha256; do
        local value
        value=$(manifest_value "$manifest" "$key")
        grep -Fxq "${key}"$'\t'"${value}" "$index"
    done
    grep -Fq $'init\tCACHE\tvalue_invariant\t' "$index"
    grep -Fq $'predicate-body\tvalue_invariant\tM_CACHE\tdependency-lowered' "$index"
    while IFS=$'\t' read -r tag body module static predicate method block statement status; do
        [[ "$tag" == access ]] || continue
        local module_file="$out/$module.coma"
        [[ -f "$module_file" ]] || { echo "missing indexed caller module $module_file" >&2; exit 1; }
        grep -Fq 'predicate value_invariant' "$module_file"
    done < "$index"

    case "$name" in
        positive)
            grep -Fq 'goal vc_static_atomic_init: value_invariant 0' "$out/M_CACHE.coma"
            grep -Fq '{[@expl:static AtomicU8 store preserves CACHE] value_invariant' "$out/M_store_one.coma"
            [[ $(grep -Fc 'any_ (static_atomic_observation: UInt8.t)' "$out/M_read_twice.coma") -eq 2 ]]
            grep -Fq 'assume value_invariant static_atomic_observation' "$out/M_read_cache.coma"
            ;;
        wrong-init)
            grep -Fq 'goal vc_static_atomic_init: value_invariant 2' "$out/M_CACHE.coma"
            grep -Fq $'init\tCACHE\tvalue_invariant\t2\t' "$index"
            ;;
        wrong-store)
            grep -Fq '{[@expl:static AtomicU8 store preserves CACHE] value_invariant' "$out/M_store_invalid.coma"
            grep -Fq 'value_invariant ([%#slib] (2: UInt8.t))' "$out/M_store_invalid.coma"
            ;;
        latest-read)
            [[ $(grep -Fc 'any_ (static_atomic_observation: UInt8.t)' "$out/M_latest_read_is_one.coma") -eq 1 ]]
            grep -Fq 'assume value_invariant static_atomic_observation' "$out/M_latest_read_is_one.coma"
            ;;
        equal-loads)
            [[ $(grep -Fc 'any_ (static_atomic_observation: UInt8.t)' "$out/M_loads_are_equal.coma") -eq 2 ]]
            grep -Fq 'assume value_invariant static_atomic_observation' "$out/M_loads_are_equal.coma"
            ;;
    esac
    printf 'PASS (translated obligations/index only, no solver): %s\n' "$name"
    printf '  coverage: %s\n' "$index"
}

run_case positive '' translate
run_case wrong-init wrong-init translate
run_case wrong-store wrong-store translate
run_case latest-read latest-read translate
run_case equal-loads equal-loads translate
run_case skipped-writer skipped-writer reject

printf 'Gate B fixture artifacts: %s\n' "$run_dir"
