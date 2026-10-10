#!/usr/bin/env bash
set -euo pipefail

script_dir=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)
toolpatch_dir=$(cd -- "$script_dir/.." && pwd)
fixtures="$script_dir/fixtures"
patch_file="$toolpatch_dir/patches/httparse-static-atomic-gate-a.patch"
rustc_bin=${CREUSOT_STATIC_ATOMIC_RUSTC:-/workspace/proof-tools/targets/httparse-static-atomic-gate-a/debug/creusot-rustc}
run_root=${CREUSOT_STATIC_ATOMIC_RUN_DIR:-${TMPDIR:-/tmp}/httparse-static-atomic-gate-a}
target=x86_64-unknown-linux-gnu
crate_name=static_atomic_gate_a_fixture

if [[ ! -x "$rustc_bin" ]]; then
    echo "missing target-local creusot-rustc: $rustc_bin" >&2
    exit 2
fi
source /workspace/proof-tools/activate.sh
export LD_LIBRARY_PATH="/workspace/proof-tools/rustup/toolchains/nightly-2026-02-27-x86_64-unknown-linux-gnu/lib:$LD_LIBRARY_PATH"
mkdir -p "$run_root"

sha256_file() {
    sha256sum "$1" | awk '{ print $1 }'
}

sha256_text() {
    printf '%s' "$1" | sha256sum | awk '{ print $1 }'
}

fresh_run_id() {
    od -An -N32 -tx1 /dev/urandom | tr -d ' \n'
}

manifest_value() {
    awk -F '\t' -v wanted="$2" '$1 == wanted && NF == 2 { print $2 }' "$1"
}

report_value() {
    awk -F '\t' -v wanted="$2" '$1 == wanted && NF == 2 { print $2 }' "$1"
}

validate_report() {
    local report=$1 manifest=$2 mode=$3 override_run_id=${4:-}
    local report_mode=Normal
    [[ "$mode" == proof-audit ]] && report_mode=Proof
    if [[ $(head -n 1 "$report") != creusot-static-atomic-audit-v2 ]]; then
        echo "FAIL: $report has no Gate A v2 report header" >&2
        return 1
    fi
    if [[ $(report_value "$report" mode) != "$report_mode" ]]; then
        echo "FAIL: report mode does not match invocation for $report" >&2
        return 1
    fi
    local key expected actual
    for key in crate target source_sha256 run_id normal_input_sha256 proof_input_sha256 \
        compiler_sha256 patch_sha256 fixture_sha256 runner_sha256; do
        expected=$(manifest_value "$manifest" "$key")
        [[ "$key" == run_id && -n "$override_run_id" ]] && expected=$override_run_id
        actual=$(report_value "$report" "$key")
        if [[ -z "$expected" || "$actual" != "$expected" ]]; then
            echo "FAIL: report provenance mismatch for $key in $report" >&2
            return 1
        fi
    done
}

fixture_for_case_dir() {
    local case_name
    case_name=$(basename -- "$1")
    [[ "$case_name" == positive ]] && case_name=positive.rs
    [[ "$case_name" == cfg-mismatch ]] && case_name=cfg-mismatch.rs
    printf '%s/%s' "$fixtures" "$case_name"
}

verify_context_inputs() {
    local manifest=$1 src=$2 case_dir=$3
    local fixture_file source_sha
    fixture_file=$(fixture_for_case_dir "$case_dir")
    source_sha=$(manifest_value "$manifest" source_sha256)
    [[ $(sha256_file "$src") == "$source_sha" ]] || return 1
    [[ $(sha256_file "$fixture_file") == "$source_sha" ]] || return 1
    [[ $(sha256_file "$rustc_bin") == "$(manifest_value "$manifest" compiler_sha256)" ]] || return 1
    [[ $(sha256_file "$patch_file") == "$(manifest_value "$manifest" patch_sha256)" ]] || return 1
    [[ $(sha256_file "$script_dir/run-gate-a-fixtures.sh") == "$(manifest_value "$manifest" runner_sha256)" ]] || return 1
}

case_manifest_entries() {
    case "$1" in
        positive.rs)
            printf 'static\tCACHE\tcache_value_invariant\n'
            printf 'static\tfirst::read_local::CACHE\tcache_value_invariant\n'
            printf 'static\tsecond::read_local::CACHE\tcache_value_invariant\n'
            ;;
        negative-join.rs)
            printf 'static\tLEFT\tcache_value_invariant\n'
            printf 'static\tRIGHT\tcache_value_invariant\n'
            ;;
        *) printf 'static\tCACHE\tcache_value_invariant\n' ;;
    esac
}

prepare_case() {
    local name=$1 case_dir=$2 src=$3 manifest=$4
    local source_sha256 fixture_sha256 compiler_sha256 patch_sha256 runner_sha256 run_id
    local normal_input_sha256 proof_input_sha256
    source_sha256=$(sha256_file "$src")
    fixture_sha256=$(sha256_file "$fixtures/$name")
    if [[ "$source_sha256" != "$fixture_sha256" ]]; then
        echo "fixture copy hash mismatch: $name" >&2
        exit 1
    fi
    compiler_sha256=$(sha256_file "$rustc_bin")
    patch_sha256=$(sha256_file "$patch_file")
    runner_sha256=$(sha256_file "$script_dir/run-gate-a-fixtures.sh")
    run_id=$(fresh_run_id)
    normal_input_sha256=$(sha256_text "mode=normal-audit;crate=$crate_name;target=$target;source=$source_sha256;compiler=$compiler_sha256;patch=$patch_sha256;fixture=$fixture_sha256;runner=$runner_sha256;incoming_cfg=creusot;normal_driver_strips_cfg=true")
    proof_input_sha256=$(sha256_text "mode=proof-audit;crate=$crate_name;target=$target;source=$source_sha256;compiler=$compiler_sha256;patch=$patch_sha256;fixture=$fixture_sha256;runner=$runner_sha256;proof_cfg=creusot;creusot_rustc_args=nightly-2026-02-27")
    {
        printf 'creusot-static-atomic-v2\n'
        printf 'crate\t%s\n' "$crate_name"
        printf 'manifest_dir\t%s\n' "$case_dir"
        printf 'target\t%s\n' "$target"
        printf 'source_sha256\t%s\n' "$source_sha256"
        printf 'run_id\t%s\n' "$run_id"
        printf 'normal_input_sha256\t%s\n' "$normal_input_sha256"
        printf 'proof_input_sha256\t%s\n' "$proof_input_sha256"
        printf 'compiler_sha256\t%s\n' "$compiler_sha256"
        printf 'patch_sha256\t%s\n' "$patch_sha256"
        printf 'fixture_sha256\t%s\n' "$fixture_sha256"
        printf 'runner_sha256\t%s\n' "$runner_sha256"
        case_manifest_entries "$name"
    } >"$manifest"
    printf '%s' "$source_sha256"
}

run_audit() {
    local mode=$1 case_dir=$2 src=$3 manifest=$4 source_sha256=$5 output_dir=$6 report=$7 log=$8
    local manifest_dir=${9:-$case_dir}
    local -a cfg_args=()
    if [[ "$mode" == normal-audit ]]; then
        # Poison the input intentionally. The normal driver must remove Creusot cfg.
        cfg_args+=(--cfg=creusot)
    fi
    verify_context_inputs "$manifest" "$src" "$case_dir" || return 1
    mkdir -p "$output_dir"
    if ! env -u CREUSOT_ARGS \
        CREUSOT_STATIC_ATOMIC_MODE="$mode" \
        CREUSOT_STATIC_ATOMIC_MANIFEST="$manifest" \
        CREUSOT_STATIC_ATOMIC_OUTPUT="$report" \
        CREUSOT_STATIC_ATOMIC_SOURCE_SHA256="$source_sha256" \
        CARGO_MANIFEST_DIR="$manifest_dir" \
        "$rustc_bin" --crate-name "$crate_name" --crate-type lib --edition=2021 \
        --target "$target" --out-dir "$output_dir" "${cfg_args[@]}" "$src" >"$log" 2>&1; then
        return 1
    fi
    verify_context_inputs "$manifest" "$src" "$case_dir" || return 1
    validate_report "$report" "$manifest" "$mode" || return 1
}

expect_rejection() {
    local name=$1 expected=$2
    local case_dir="$run_root/$name" src="$fixtures/$name" manifest source_sha256 log
    mkdir -p "$case_dir"
    cp -- "$src" "$case_dir/case.rs"
    src="$case_dir/case.rs"
    manifest="$case_dir/manifest.tsv"
    source_sha256=$(prepare_case "$name" "$case_dir" "$src" "$manifest")
    log="$case_dir/proof.log"
    if run_audit proof-audit "$case_dir" "$src" "$manifest" "$source_sha256" "$case_dir/out" "$case_dir/proof.audit" "$log"; then
        echo "FAIL: $name unexpectedly passed the proof-cfg audit" >&2
        exit 1
    fi
    if ! grep -Fq -- "$expected" "$log"; then
        echo "FAIL: $name failed for a different reason; expected diagnostic: $expected" >&2
        cat "$log" >&2
        exit 1
    fi
    printf 'PASS (rejected as expected): %s\n' "$name"
}

positive_dir="$run_root/positive"
mkdir -p "$positive_dir"
cp -- "$fixtures/positive.rs" "$positive_dir/case.rs"
positive_src="$positive_dir/case.rs"
positive_manifest="$positive_dir/manifest.tsv"
positive_hash=$(prepare_case positive.rs "$positive_dir" "$positive_src" "$positive_manifest")
run_audit normal-audit "$positive_dir" "$positive_src" "$positive_manifest" "$positive_hash" \
    "$positive_dir/normal-out" "$positive_dir/normal.audit" "$positive_dir/normal.log"
run_audit proof-audit "$positive_dir" "$positive_src" "$positive_manifest" "$positive_hash" \
    "$positive_dir/proof-out" "$positive_dir/proof.audit" "$positive_dir/proof.log"

extract_executable_mir() {
    awk '/^access\t/ { print } /^begin_mir\t/ { inside=1 } inside { print } /^end_mir\t/ { inside=0 }' "$1"
}
extract_executable_mir "$positive_dir/normal.audit" >"$positive_dir/normal.executable"
extract_executable_mir "$positive_dir/proof.audit" >"$positive_dir/proof.executable"
diff -u "$positive_dir/normal.executable" "$positive_dir/proof.executable"
grep -Fq $'access\t' "$positive_dir/proof.audit"
grep -Fq $'first::read_local::CACHE\tload' "$positive_dir/proof.audit"
grep -Fq $'second::read_local::CACHE\tload' "$positive_dir/proof.audit"
printf 'PASS: positive normal/proof executable MIR and registered static identities match\n'

copied_report="$positive_dir/copied-old-report.audit"
cp -- "$positive_dir/normal.audit" "$copied_report"
stale_manifest="$positive_dir/fresh-run-manifest.tsv"
fresh_id=$(fresh_run_id)
awk -F '\t' -v fresh_id="$fresh_id" 'BEGIN { OFS="\t" } $1 == "run_id" { $2=fresh_id } { print }' \
    "$positive_manifest" >"$stale_manifest"
if validate_report "$copied_report" "$stale_manifest" normal-audit >"$positive_dir/copied-report-check.log" 2>&1; then
    echo 'FAIL: report copied from a previous run passed the fresh run-id check' >&2
    exit 1
fi
grep -Fq 'provenance mismatch for run_id' "$positive_dir/copied-report-check.log"
printf 'PASS: a copied report from an earlier run is rejected by its run id\n'

config_log="$positive_dir/bad-source-hash.log"
if run_audit normal-audit "$positive_dir" "$positive_src" "$positive_manifest" \
    aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa \
    "$positive_dir/bad-source-out" "$positive_dir/bad-source.audit" "$config_log"; then
    echo 'FAIL: source hash mismatch unexpectedly passed manifest validation' >&2
    exit 1
fi
grep -Fq 'source hash in manifest does not match' "$config_log"
config_log="$positive_dir/bad-package-path.log"
wrong_manifest_dir="$positive_dir/wrong-package"
mkdir -p "$wrong_manifest_dir"
if run_audit normal-audit "$positive_dir" "$positive_src" "$positive_manifest" "$positive_hash" \
    "$positive_dir/bad-package-out" "$positive_dir/bad-package.audit" "$config_log" "$wrong_manifest_dir"; then
    echo 'FAIL: wrong package path unexpectedly passed target routing' >&2
    exit 1
fi
grep -Fq 'matched the static audit registration, but CARGO_MANIFEST_DIR is' "$config_log"
wrong_target_manifest="$positive_dir/wrong-target.tsv"
sed 's/^target\tx86_64-unknown-linux-gnu$/target\taarch64-unknown-linux-gnu/' \
    "$positive_manifest" >"$wrong_target_manifest"
config_log="$positive_dir/bad-target.log"
if run_audit normal-audit "$positive_dir" "$positive_src" "$wrong_target_manifest" "$positive_hash" \
    "$positive_dir/bad-target-out" "$positive_dir/bad-target.audit" "$config_log"; then
    echo 'FAIL: mismatched compilation target unexpectedly passed audit' >&2
    exit 1
fi
grep -Fq 'does not match static audit manifest target' "$config_log"
printf 'PASS: source digest, package path, and target mismatches fail closed\n'

mismatch_dir="$run_root/cfg-mismatch"
mkdir -p "$mismatch_dir"
cp -- "$fixtures/cfg-mismatch.rs" "$mismatch_dir/case.rs"
mismatch_src="$mismatch_dir/case.rs"
mismatch_manifest="$mismatch_dir/manifest.tsv"
mismatch_hash=$(prepare_case cfg-mismatch.rs "$mismatch_dir" "$mismatch_src" "$mismatch_manifest")
run_audit normal-audit "$mismatch_dir" "$mismatch_src" "$mismatch_manifest" "$mismatch_hash" \
    "$mismatch_dir/normal-out" "$mismatch_dir/normal.audit" "$mismatch_dir/normal.log"
run_audit proof-audit "$mismatch_dir" "$mismatch_src" "$mismatch_manifest" "$mismatch_hash" \
    "$mismatch_dir/proof-out" "$mismatch_dir/proof.audit" "$mismatch_dir/proof.log"
extract_executable_mir "$mismatch_dir/normal.audit" >"$mismatch_dir/normal.executable"
extract_executable_mir "$mismatch_dir/proof.audit" >"$mismatch_dir/proof.executable"
if diff -u "$mismatch_dir/normal.executable" "$mismatch_dir/proof.executable" >"$mismatch_dir/diff"; then
    echo 'FAIL: cfg-mismatch.rs unexpectedly had matching executable reports' >&2
    exit 1
fi
grep -Fq 'AtomicU8::store' "$mismatch_dir/normal.audit"
grep -Fq 'AtomicU8::load' "$mismatch_dir/proof.audit"
printf 'PASS: cfg-mismatch.rs is detected by the normal/proof executable comparison\n'

expect_rejection negative-rawptr.rs 'registered static reference escapes through an unsupported assignment'
expect_rejection negative-helper.rs 'registered static reference is passed to an unknown or unsupported callee'
expect_rejection negative-ordering.rs 'uses a non-Relaxed or unresolved ordering'
expect_rejection negative-const-alias.rs 'registered static reference'
expect_rejection negative-init-helper.rs 'initializer is not the recognized AtomicU8::new(const_u8) form'
expect_rejection negative-join.rs 'registered static reference escapes through an unsupported assignment'
expect_rejection negative-public.rs 'is externally visible'
expect_rejection negative-type.rs 'expected the resolved sysroot AtomicU8 type'
expect_rejection negative-return.rs 'registered static reference is returned from'
expect_rejection negative-aggregate.rs 'registered static reference escapes through an unsupported assignment'
expect_rejection negative-capture.rs 'registered static reference escapes through an unsupported assignment'
expect_rejection negative-generic.rs 'registered static reference is passed to an unknown or unsupported callee'
expect_rejection negative-indirect.rs 'registered static reference is passed to an unknown or unsupported callee'

echo "Gate A fixture reports: $run_root"
