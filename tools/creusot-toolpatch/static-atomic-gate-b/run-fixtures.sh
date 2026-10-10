#!/usr/bin/env bash
set -euo pipefail

repo_root=$(git rev-parse --show-toplevel)
script_dir="$repo_root/tools/creusot-toolpatch/static-atomic-gate-b"
fixture_dir="$script_dir/fixture"
required_targets="$script_dir/required-targets.tsv"
patch_file="$repo_root/tools/creusot-toolpatch/patches/httparse-static-atomic-gate-b.patch"
compiler_bin=${CREUSOT_STATIC_ATOMIC_RUSTC:-/workspace/proof-tools/targets/httparse-static-atomic-gate-b/debug/creusot-rustc}
run_root=${CREUSOT_STATIC_ATOMIC_GATE_B_RUN_ROOT:-/workspace/proof-tools/targets/httparse-static-atomic-gate-b/fixture-runs}
crate_name=static_atomic_gate_b_fixture
package_name=static-atomic-gate-b-fixture
target=x86_64-unknown-linux-gnu

source /workspace/proof-tools/activate.sh
[[ -x "$compiler_bin" ]] || { echo "missing target-local compiler: $compiler_bin" >&2; exit 1; }
[[ -f "$patch_file" ]] || { echo "missing Gate B patch: $patch_file" >&2; exit 1; }
mkdir -p "$run_root"

sha256_file() {
    sha256sum "$1" | awk '{print $1}'
}

sha256_text() {
    printf '%s' "$1" | sha256sum | awk '{print $1}'
}

fixture_sha256() {
    (cd "$fixture_dir" && sha256sum Cargo.toml Cargo.lock src/lib.rs) | sha256sum | awk '{print $1}'
}

required_targets_sha256() {
    sha256_file "$required_targets"
}

source_sha256() {
    sha256_file "$fixture_dir/src/lib.rs"
}

check_context_unchanged() {
    [[ $(source_sha256) == "$initial_source_sha" ]] || { echo "fixture Rust source changed during run" >&2; exit 1; }
    [[ $(fixture_sha256) == "$initial_fixture_sha" ]] || { echo "fixture inputs changed during run" >&2; exit 1; }
    [[ $(required_targets_sha256) == "$initial_targets_sha" ]] || { echo "required target manifest changed during run" >&2; exit 1; }
    [[ $(sha256_file "$compiler_bin") == "$initial_compiler_sha" ]] || { echo "compiler binary changed during run" >&2; exit 1; }
    [[ $(sha256_file "$patch_file") == "$initial_patch_sha" ]] || { echo "Gate B patch changed during run" >&2; exit 1; }
    [[ $(sha256_file "$script_dir/run-fixtures.sh") == "$initial_runner_sha" ]] || { echo "runner script changed during run" >&2; exit 1; }
}

manifest_value() {
    awk -F '\t' -v key="$2" '$1 == key { print $2; exit }' "$1"
}

validate_report_context() {
    local report=$1 manifest=$2 key expected actual
    for key in crate target source_sha256 run_id normal_input_sha256 proof_input_sha256 compiler_sha256 patch_sha256 fixture_sha256 runner_sha256; do
        expected=$(manifest_value "$manifest" "$key")
        actual=$(manifest_value "$report" "$key")
    [[ -n "$expected" && "$actual" == "$expected" ]] || {
            echo "audit context mismatch for $key in $report" >&2
            return 1
        }
    done
}

fresh_run_id() {
    od -An -N32 -tx1 /dev/urandom | tr -d ' \n'
}

declare -A expected_result
expected_result[positive]=translate
expected_result[wrong-init]=translate
expected_result[wrong-store]=translate
expected_result[latest-read]=translate
expected_result[equal-loads]=translate
expected_result[skipped-writer]=reject
expected_result[partial-predicate]=reject
expected_result[prophetic-predicate]=reject
expected_result[spoof-predicate]=reject
expected_result[span-literal-mismatch]=reject

initial_source_sha=$(source_sha256)
initial_fixture_sha=$(fixture_sha256)
initial_targets_sha=$(required_targets_sha256)
initial_compiler_sha=$(sha256_file "$compiler_bin")
initial_patch_sha=$(sha256_file "$patch_file")
initial_runner_sha=$(sha256_file "$script_dir/run-fixtures.sh")
run_id=$(fresh_run_id)
out="$run_root/$run_id"
mkdir -p "$out"
printf 'scenario\tfeatures\ttranslation\tcoverage_or_error\n' > "$out/results.tsv"
printf 'scenario\tartifact\tsha256\n' > "$out/artifact-hashes.tsv"

run_case() {
    local scenario=$1 features=$2 expected_error=${3:-}
    local case_dir="$out/$scenario"
    local src_hash fixture_hash normal_hash proof_hash manifest normal_report proof_report
    local targets_hash
    local compiler_hash patch_hash runner_hash result output_dir
    local -a feature_args=()

    [[ -z "$features" ]] || feature_args=(--features "$features")
    mkdir -p "$case_dir"
    src_hash=$(source_sha256)
    fixture_hash=$(fixture_sha256)
    targets_hash=$(required_targets_sha256)
    compiler_hash=$(sha256_file "$compiler_bin")
    patch_hash=$(sha256_file "$patch_file")
    runner_hash=$(sha256_file "$script_dir/run-fixtures.sh")
    awk -F '\t' -v scenario="$scenario" 'NR > 1 && $1 == scenario { found = 1 } END { exit !found }' "$required_targets" || {
        echo "required target manifest has no entries for $scenario" >&2
        return 1
    }
    normal_hash=$(sha256_text "mode=normal-audit;crate=$crate_name;target=$target;features=$features;source=$src_hash;fixture=$fixture_hash;targets=$targets_hash;compiler=$compiler_hash;patch=$patch_hash;runner=$runner_hash;incoming_cfg=creusot;normal_driver_strips_cfg=true")
    proof_hash=$(sha256_text "mode=proof-audit;crate=$crate_name;target=$target;features=$features;source=$src_hash;fixture=$fixture_hash;targets=$targets_hash;compiler=$compiler_hash;patch=$patch_hash;runner=$runner_hash;proof_cfg=creusot;creusot_rustc_args=nightly-2026-02-27")
    manifest="$case_dir/manifest.tsv"
    normal_report="$case_dir/normal.audit"
    proof_report="$case_dir/proof.audit"
    output_dir="$fixture_dir/verif/static_atomic_gate_b_fixture_rlib"
    {
        printf 'creusot-static-atomic-v2\n'
        printf 'crate\t%s\n' "$crate_name"
        printf 'manifest_dir\t%s\n' "$fixture_dir"
        printf 'target\t%s\n' "$target"
        printf 'source_sha256\t%s\n' "$src_hash"
        printf 'run_id\t%s\n' "$run_id"
        printf 'normal_input_sha256\t%s\n' "$normal_hash"
        printf 'proof_input_sha256\t%s\n' "$proof_hash"
        printf 'compiler_sha256\t%s\n' "$compiler_hash"
        printf 'patch_sha256\t%s\n' "$patch_hash"
        printf 'fixture_sha256\t%s\n' "$fixture_hash"
        printf 'runner_sha256\t%s\n' "$runner_hash"
        printf '# required_targets_sha256\t%s\n' "$targets_hash"
        printf 'static\tCACHE\tvalue_invariant\n'
    } > "$manifest"

    check_context_unchanged
    (
        cd "$fixture_dir"
        CARGO_TARGET_DIR="$run_root/cargo-target" cargo clean -p "$package_name" >/dev/null
        RUSTC="$compiler_bin" RUSTFLAGS='--cfg=creusot' \
            CARGO_TARGET_DIR="$run_root/cargo-target" \
            CREUSOT_STATIC_ATOMIC_MODE=normal-audit \
            CREUSOT_STATIC_ATOMIC_MANIFEST="$manifest" \
            CREUSOT_STATIC_ATOMIC_OUTPUT="$normal_report" \
            CREUSOT_STATIC_ATOMIC_SOURCE_SHA256="$src_hash" \
            cargo check --offline "${feature_args[@]}"
    ) > "$case_dir/normal.log" 2>&1 || {
        cat "$case_dir/normal.log" >&2
        echo "normal cfg audit failed for $scenario" >&2
        return 1
    }
    check_context_unchanged
    validate_report_context "$normal_report" "$manifest"
    if [[ $scenario == skipped-writer ]]; then
        grep -Fq $'access\tskipped_writer\tCACHE\tstore\tOrdering::Relaxed' "$normal_report"
    fi

    local proof_status=0
    (
        cd "$fixture_dir"
        CARGO_TARGET_DIR="$run_root/cargo-target" cargo clean -p "$package_name" >/dev/null
        rm -rf "$output_dir"
        CREUSOT_RUSTC="$compiler_bin" \
            CARGO_TARGET_DIR="$run_root/cargo-target" \
            CREUSOT_STATIC_ATOMIC_MODE=proof-audit \
            CREUSOT_STATIC_ATOMIC_MANIFEST="$manifest" \
            CREUSOT_STATIC_ATOMIC_OUTPUT="$proof_report" \
            CREUSOT_STATIC_ATOMIC_SOURCE_SHA256="$src_hash" \
            CREUSOT_STATIC_ATOMIC_GATE_B_NORMAL_AUDIT="$normal_report" \
            CREUSOT_STATIC_ATOMIC_GATE_B=1 \
            cargo creusot --no-check-version -- --offline "${feature_args[@]}"
    ) > "$case_dir/proof.log" 2>&1 || proof_status=$?
    check_context_unchanged

    if [[ ${expected_result[$scenario]} == reject ]]; then
        [[ $proof_status -ne 0 ]] || { echo "expected Gate B rejection for $scenario" >&2; return 1; }
        grep -Fq -- "$expected_error" "$case_dir/proof.log" || {
            echo "wrong rejection for $scenario; expected: $expected_error" >&2
            cat "$case_dir/proof.log" >&2
            return 1
        }
        if [[ -f "$proof_report" ]]; then
            validate_report_context "$proof_report" "$manifest"
        fi
        [[ ! -f "$output_dir/static-atomic-coverage.tsv" ]] || {
            echo "rejected scenario unexpectedly emitted a Gate B coverage index: $scenario" >&2
            return 1
        }
        if [[ -d "$output_dir" ]] && find "$output_dir" -type f -name '*.coma' -print -quit | grep -q .; then
            echo "rejected scenario unexpectedly emitted COMA modules: $scenario" >&2
            return 1
        fi
        if [[ $scenario == skipped-writer ]]; then
            [[ ! -f "$proof_report" ]] || {
                echo "skipped writer was rejected only after a complete proof audit report" >&2
                return 1
            }
            printf '%s\t%s\trejected-as-designed\tproof MIR audit rejected skipped writer; normal audit recorded the store\n' \
                "$scenario" "$features" >> "$out/results.tsv"
            printf 'PASS: proof MIR audit rejected skipped writer while normal MIR records its store\n'
            return 0
        fi
        printf '%s\t%s\trejected-as-designed\t%s\n' "$scenario" "$features" "$expected_error" >> "$out/results.tsv"
        printf 'PASS: expected fail-closed rejection: %s\n' "$scenario"
        return 0
    fi

    [[ -f "$proof_report" ]] || { echo "missing proof audit report for $scenario" >&2; cat "$case_dir/proof.log" >&2; return 1; }
    validate_report_context "$proof_report" "$manifest"
    [[ $proof_status -eq 0 ]] || { cat "$case_dir/proof.log" >&2; echo "Gate B translation failed for $scenario" >&2; return 1; }
    local coverage="$output_dir/static-atomic-coverage.tsv"
    [[ -f "$coverage" ]] || { echo "missing coverage index for $scenario" >&2; return 1; }
    grep -Fq $'run_id\t'"$run_id" "$coverage"
    if [[ $scenario == positive ]]; then
        grep -Fq $'predicate\tvalue_invariant\tstructurally-total' "$coverage"
        grep -Fq $'init\tCACHE\tvalue_invariant\t0\tM_CACHE\tvc-emitted' "$coverage"
        grep -Fq 'goal vc_static_atomic_init: value_invariant (0: UInt8.t)' "$output_dir/CACHE.coma"
        grep -Fq $'accessor\tread_cache\tM_read_cache\tprogram-goals-emitted' "$coverage"
        grep -Fq $'accessor\tread_twice\tM_read_twice\tprogram-goals-emitted' "$coverage"
        grep -Fq $'accessor\tstore_one\tM_store_one\tprogram-goals-emitted' "$coverage"
        [[ $(grep -c $'^access\tread_twice\t' "$coverage") -eq 2 ]]
        [[ $(grep -Fc 'any_ (static_atomic_observation: UInt8.t)' "$output_dir/read_twice.coma") -eq 2 ]]
        grep -Fq '! -{value_invariant static_atomic_observation}-' "$output_dir/read_cache.coma"
        grep -Fq 'value_invariant ([%#slib] (1: UInt8.t))' "$output_dir/store_one.coma"
        grep -Fq 'false} any' "$output_dir/read_cache.coma"
        grep -Fq $'predicate-body\tvalue_invariant\tM_CACHE\tdependency-lowered' "$coverage"
        result="translation-and-indexed"
    elif [[ $scenario == wrong-init ]]; then
        grep -Fq 'goal vc_static_atomic_init: value_invariant (2: UInt8.t)' "$output_dir/CACHE.coma"
        result="translation; init-VC-for-2-emitted-unproved"
    elif [[ $scenario == wrong-store ]]; then
        grep -Fq 'store preserves CACHE' "$output_dir/store_invalid.coma"
        grep -Fq 'value_invariant ([%#slib] (2: UInt8.t))' "$output_dir/store_invalid.coma"
        result="translation; store-VC-for-2-emitted-unproved"
    elif [[ $scenario == latest-read ]]; then
        grep -Fq 'latest_read_is_one' "$coverage"
        grep -Fq 'false} any' "$output_dir/latest_read_is_one.coma"
        result="translation; assertion-VC-emitted-unproved-by-fresh-I"
    elif [[ $scenario == equal-loads ]]; then
        grep -Fq 'loads_are_equal' "$coverage"
        [[ $(grep -Fc 'any_ (static_atomic_observation: UInt8.t)' "$output_dir/loads_are_equal.coma") -eq 2 ]]
        grep -Fq 'false} any' "$output_dir/loads_are_equal.coma"
        result="translation; equality-VC-emitted-unproved-by-independent-I"
    fi
    mkdir -p "$case_dir/modules"
    cp -a "$output_dir/." "$case_dir/modules/"
    while IFS= read -r -d '' artifact; do
        relative=${artifact#"$case_dir/modules/"}
        printf '%s\t%s\t%s\n' "$scenario" "$relative" "$(sha256_file "$artifact")" \
            >> "$out/artifact-hashes.tsv"
    done < <(find "$case_dir/modules" -type f -print0 | sort -z)
    printf '%s\t%s\t%s\t%s\n' "$scenario" "$features" "translated-no-solver" "$result" >> "$out/results.tsv"
    printf 'PASS: translated and inspected: %s\n' "$scenario"
}

run_case positive ""
run_case wrong-init wrong-init
run_case wrong-store wrong-store
run_case latest-read latest-read
run_case equal-loads equal-loads
run_case skipped-writer skipped-writer 'registered static reference has no recognized direct load/store use in skipped_writer'
run_case partial-predicate partial-predicate 'outside the closed total-expression subset'
run_case prophetic-predicate prophetic-predicate 'nonprophetic'
run_case spoof-predicate spoof-predicate 'outside the closed total-expression subset'
run_case span-literal-mismatch span-literal-mismatch 'normal/proof executable MIR and access inventories differ'

check_context_unchanged
printf 'Gate B translation fixtures complete; no Why3 solver was invoked.\n'
printf 'Evidence directory: %s\n' "$out"
cat "$out/results.tsv"
printf 'Generated module hashes: %s\n' "$out/artifact-hashes.tsv"
printf 'Required targets manifest SHA-256: %s\n' "$initial_targets_sha"
