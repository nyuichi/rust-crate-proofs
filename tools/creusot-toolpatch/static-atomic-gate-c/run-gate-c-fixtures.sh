#!/usr/bin/env bash
set -euo pipefail

script_dir=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)
repo_root=$(git -C "$script_dir" rev-parse --show-toplevel)
fixture_source="$script_dir/fixture"
patch_file="$repo_root/tools/creusot-toolpatch/patches/httparse-static-atomic-gate-c.patch"
compiler_bin=${CREUSOT_STATIC_ATOMIC_GATE_C_RUSTC:-/workspace/proof-tools/targets/httparse-static-atomic-gate-c/debug/creusot-rustc}
target_dir=${CREUSOT_STATIC_ATOMIC_GATE_C_TARGET_DIR:-/workspace/proof-tools/targets/httparse-static-atomic-gate-c}
prelude_package_dir=$target_dir/generated-prelude-package
package_manifest=$target_dir/generated-prelude-package.SHA256
why3_config=${CREUSOT_STATIC_ATOMIC_GATE_C_WHY3_CONFIG:-/workspace/proof-tools/config/creusot/why3.conf}
why3_bin=${CREUSOT_STATIC_ATOMIC_GATE_C_WHY3_BIN:-/workspace/proof-tools/creusot-data/bin/why3}
run_root=${CREUSOT_STATIC_ATOMIC_GATE_C_RUN_ROOT:-/tmp/httparse-static-atomic-gate-c}
target=x86_64-unknown-linux-gnu
crate_name=static_atomic_gate_c_fixture
profile=x86-runtime-caps-v1

source /workspace/proof-tools/activate.sh
sha_file() { sha256sum "$1" | cut -d' ' -f1; }
[[ -x "$compiler_bin" ]] || { echo "missing Gate C compiler: $compiler_bin" >&2; exit 2; }
[[ -f "$patch_file" ]] || { echo "missing Gate C patch: $patch_file" >&2; exit 2; }
[[ -x "$why3_bin" ]] || { echo "missing Why3 executable: $why3_bin" >&2; exit 2; }
[[ -f "$why3_config" ]] || { echo "missing Why3 configuration: $why3_config" >&2; exit 2; }
[[ -d "$prelude_package_dir" ]] || { echo "missing generated Creusot package: $prelude_package_dir" >&2; exit 2; }
[[ -f "$prelude_package_dir/creusot/prelude.coma" ]] || { echo "missing generated prelude: $prelude_package_dir/creusot/prelude.coma" >&2; exit 2; }
[[ -f "$package_manifest" ]] || { echo "missing generated package manifest: $package_manifest" >&2; exit 2; }
diff -u "$package_manifest" <(cd "$prelude_package_dir" && find . -type f -print0 | sort -z | xargs -0 sha256sum)
mkdir -p "$run_root"
run_id=$(od -An -N32 -tx1 /dev/urandom | tr -d ' \n')
run_dir="$run_root/$run_id"
mkdir -p "$run_dir"

target_cfg=$(rustc --print cfg --target "$target" | sort)
target_cfg_hash=$(printf '%s\n' "$target_cfg" | sha256sum | cut -d' ' -f1)
package_sha=$(sha_file "$package_manifest")
prelude_sha=$(sha_file "$prelude_package_dir/creusot/prelude.coma")
why3_sha=$(sha_file "$why3_bin")
why3_config_sha=$(sha_file "$why3_config")

sha_fixture() {
    (cd "$fixture_source" && sha256sum Cargo.toml Cargo.lock src/lib.rs) | sha256sum | cut -d' ' -f1
}
sha_text() { printf '%s' "$1" | sha256sum | cut -d' ' -f1; }
manifest_value() {
    awk -F '\t' -v key="$2" '$1 == key { print $2; exit }' "$1"
}

run_case() {
    local name=$1 features=$2 expected=$3 profile_value=${4:-$profile} rustflags=${5:-} bad_entry=${6:-} encoded_flags=${7:-}
    local case_dir="$run_dir/$name" package="$run_dir/$name/fixture"
    local manifest="$case_dir/manifest.tsv" normal_report="$case_dir/normal.audit" proof_report="$case_dir/proof.audit"
    local source_sha fixture_sha compiler_sha patch_sha runner_sha normal_input proof_input case_run_id
    local cargo_features=()
    local encoded_rustflags_env=()
    [[ -z "$encoded_flags" ]] || encoded_rustflags_env+=("CARGO_ENCODED_RUSTFLAGS=$encoded_flags")
    mkdir -p "$package/src"
    cp "$fixture_source/src/lib.rs" "$package/src/lib.rs"
    cp "$fixture_source/Cargo.lock" "$package/Cargo.lock"
    sed "s#path = \"../../../../creusot-libs/creusot-std\"#path = \"$repo_root/creusot-libs/creusot-std\"#" \
        "$fixture_source/Cargo.toml" > "$package/Cargo.toml"
    source_sha=$(sha_file "$fixture_source/src/lib.rs")
    fixture_sha=$(sha_fixture)
    compiler_sha=$(sha_file "$compiler_bin")
    patch_sha=$(sha_file "$patch_file")
    runner_sha=$(sha_file "$script_dir/run-gate-c-fixtures.sh")
    case_run_id=$(od -An -N32 -tx1 /dev/urandom | tr -d ' \n')
    normal_input=$(sha_text "mode=normal-audit;crate=$crate_name;target=$target;profile=$profile_value;features=$features;rustflags=$rustflags;encoded_rustflags=$encoded_flags;target_cfg_sha256=$target_cfg_hash;source=$source_sha;fixture=$fixture_sha;compiler=$compiler_sha;patch=$patch_sha;runner=$runner_sha;prelude_package=$package_sha;prelude=$prelude_sha;why3=$why3_sha;why3_config=$why3_config_sha;incoming_cfg=creusot;driver_strips_cfg=true")
    proof_input=$(sha_text "mode=proof-audit;crate=$crate_name;target=$target;profile=$profile_value;features=$features;rustflags=$rustflags;encoded_rustflags=$encoded_flags;target_cfg_sha256=$target_cfg_hash;source=$source_sha;fixture=$fixture_sha;compiler=$compiler_sha;patch=$patch_sha;runner=$runner_sha;prelude_package=$package_sha;prelude=$prelude_sha;why3=$why3_sha;why3_config=$why3_config_sha;cargo_creusot=--no-check-version;cargo_offline=true;proof_cfg=creusot")
    {
        printf 'creusot-static-atomic-gate-c-v1\n'
        printf 'profile\t%s\n' "$profile_value"
        printf 'crate\t%s\nmanifest_dir\t%s\ntarget\t%s\n' "$crate_name" "$package" "$target"
        printf 'source_sha256\t%s\nrun_id\t%s\n' "$source_sha" "$case_run_id"
        printf 'normal_input_sha256\t%s\nproof_input_sha256\t%s\n' "$normal_input" "$proof_input"
        printf 'compiler_sha256\t%s\npatch_sha256\t%s\nfixture_sha256\t%s\nrunner_sha256\t%s\n' \
            "$compiler_sha" "$patch_sha" "$fixture_sha" "$runner_sha"
        [[ -z "$bad_entry" ]] || printf '%s\n' "$bad_entry"
        printf 'static\tCACHE\tvalue_invariant\n'
    } > "$manifest"
    [[ -z "$features" ]] || cargo_features=(--features "$features")

    if [[ "$expected" == reject-profile || "$expected" == reject-cfg || "$expected" == reject-cfg-arg || "$expected" == reject-entry ]]; then
        local result=0
        (cd "$package" && \
            env \
            "${encoded_rustflags_env[@]}" \
            RUSTC="$compiler_bin" \
            RUSTFLAGS="--cfg=creusot $rustflags" \
            CARGO_TARGET_DIR="$case_dir/normal-target" \
            CREUSOT_STATIC_ATOMIC_MODE=normal-audit \
            CREUSOT_STATIC_ATOMIC_MANIFEST="$manifest" \
            CREUSOT_STATIC_ATOMIC_OUTPUT="$normal_report" \
            CREUSOT_STATIC_ATOMIC_SOURCE_SHA256="$source_sha" \
            cargo check --offline "${cargo_features[@]}" >"$case_dir/normal.log" 2>&1) || result=$?
        [[ "$result" -ne 0 ]] || { echo "FAIL: $name unexpectedly passed audit" >&2; exit 1; }
        case "$expected" in
            reject-profile) grep -Fq 'Gate C manifest requires exactly profile x86-runtime-caps-v1' "$case_dir/normal.log" ;;
            reject-cfg) grep -Fq 'Gate C profile rejects explicit -C target-cpu and -C target-feature overrides' "$case_dir/normal.log" ;;
            reject-cfg-arg) grep -Fq 'Gate C profile rejects manually supplied --cfg `target_feature`' "$case_dir/normal.log" ;;
            reject-entry) grep -Fq 'invalid or duplicate static audit manifest entry' "$case_dir/normal.log" ;;
        esac
        printf 'PASS (fail-closed %s): %s\n' "$expected" "$name"
        return
    fi

    (cd "$package" && \
        RUSTC="$compiler_bin" \
        RUSTFLAGS='--cfg=creusot' \
        CARGO_TARGET_DIR="$case_dir/normal-target" \
        CREUSOT_STATIC_ATOMIC_MODE=normal-audit \
        CREUSOT_STATIC_ATOMIC_MANIFEST="$manifest" \
        CREUSOT_STATIC_ATOMIC_OUTPUT="$normal_report" \
        CREUSOT_STATIC_ATOMIC_SOURCE_SHA256="$source_sha" \
        cargo check --offline "${cargo_features[@]}" >"$case_dir/normal.log" 2>&1)
    grep -Fxq 'creusot-static-atomic-audit-gate-c-v1' "$normal_report"
    grep -Fxq $'profile\tx86-runtime-caps-v1' "$normal_report"
    grep -Fxq $'mode\tNormal' "$normal_report"
    grep -Fxq $'target\tx86_64-unknown-linux-gnu' "$normal_report"
    grep -Fxq $'effective_target_feature\tavx2\tsession=false\tcfg=false' "$normal_report"
    grep -Fxq $'effective_target_feature\tsse4.2\tsession=false\tcfg=false' "$normal_report"

    local proof_result=0
    (cd "$package" && \
        CREUSOT_RUSTC="$compiler_bin" \
        CARGO_TARGET_DIR="$case_dir/proof-target" \
        CREUSOT_STATIC_ATOMIC_MODE=proof-audit \
        CREUSOT_STATIC_ATOMIC_MANIFEST="$manifest" \
        CREUSOT_STATIC_ATOMIC_OUTPUT="$proof_report" \
        CREUSOT_STATIC_ATOMIC_SOURCE_SHA256="$source_sha" \
        CREUSOT_STATIC_ATOMIC_GATE_B_NORMAL_AUDIT="$normal_report" \
        CREUSOT_STATIC_ATOMIC_GATE_B=1 \
        cargo creusot --no-check-version -- --offline "${cargo_features[@]}" >"$case_dir/proof.log" 2>&1) || proof_result=$?
    if [[ "$expected" == reject-signature ]]; then
        [[ "$proof_result" -ne 0 ]] || { echo "FAIL: $name unexpectedly passed signature audit" >&2; exit 1; }
        grep -Fq 'must have the exact non-generic signature fn(u8, bool, bool) -> bool' "$case_dir/proof.log"
        printf 'PASS (fail-closed wrong signature): %s\n' "$name"
        return
    fi
    [[ "$proof_result" -eq 0 ]] || { echo "FAIL: $name translation failed" >&2; cat "$case_dir/proof.log" >&2; exit 1; }
    grep -Fxq 'creusot-static-atomic-audit-gate-c-v1' "$proof_report"
    grep -Fxq $'profile\tx86-runtime-caps-v1' "$proof_report"
    grep -Fxq $'mode\tProof' "$proof_report"
    grep -Fxq $'effective_target_feature\tavx2\tsession=false\tcfg=false' "$proof_report"
    grep -Fxq $'effective_target_feature\tsse4.2\tsession=false\tcfg=false' "$proof_report"
    diff -u <(grep '^access[[:space:]]' "$normal_report") <(grep '^access[[:space:]]' "$proof_report")

    local out="$package/verif/static_atomic_gate_c_fixture_rlib"
    local index="$out/static-atomic-coverage.tsv"
    [[ -f "$index" ]] || { echo "missing Gate C coverage index: $index" >&2; exit 1; }
    grep -Fxq $'accessor\tstore_zero\tM_store_zero\tprogram-body-lowered' "$index"
    ! grep -Fq 'program-goals-emitted' "$index"
    for module in CACHE store_zero store_one store_two store_three read_cache read_twice; do
        local module_file="$out/$module.coma"
        [[ -f "$module_file" ]] || { echo "missing Gate C module $module_file" >&2; exit 1; }
        grep -Fq 'avx2_usable' "$module_file"
        grep -Fq 'sse42_usable' "$module_file"
    grep -Fq 'StaticAtomicCaps' "$module_file"
    done
    grep -Fq 'predicate value_invariant' "$out/CACHE.coma"
    grep -Fq 'avx2_usable' "$out/CACHE.coma"
    grep -Fq 'sse42_usable' "$out/CACHE.coma"
    [[ $(grep -Fc 'any_ (static_atomic_observation: UInt8.t)' "$out/read_twice.coma") -eq 2 ]]
    [[ $(grep -Fc 'value_invariant static_atomic_observation StaticAtomicCaps.avx2_usable StaticAtomicCaps.sse42_usable' "$out/read_twice.coma") -eq 2 ]]

    local type_only_log="$case_dir/type-only.log" type_only_commands="$case_dir/type-only-commands.txt"
    : > "$type_only_log"
    : > "$type_only_commands"
    for module_file in "$out"/*.coma; do
        printf '%q prove --type-only -C %q -L %q -L %q -F coma %q\n' \
            "$why3_bin" "$why3_config" "$prelude_package_dir" "$out" "$module_file" >> "$type_only_commands"
        "$why3_bin" prove --type-only -C "$why3_config" \
            -L "$prelude_package_dir" -L "$out" -F coma "$module_file" >> "$type_only_log" 2>&1
    done
    printf 'package_sha256\t%s\nprelude_sha256\t%s\nwhy3_sha256\t%s\nwhy3_config_sha256\t%s\n' \
        "$package_sha" "$prelude_sha" "$why3_sha" "$why3_config_sha" > "$case_dir/type-only-provenance.tsv"
    printf 'PASS (type-only, no solver): %s COMA modules against retained generated package\n' \
        "$(find "$out" -maxdepth 1 -type f -name '*.coma' | wc -l | tr -d ' ')"
    printf 'PASS (translation + fixed capability-symbol identity, no solver): %s\n' "$name"
    printf '  coverage: %s\n' "$index"
}

run_case positive '' translate
run_case wrong-signature wrong-signature reject-signature
run_case wrong-profile '' reject-profile x86-runtime-caps-v2
run_case wrong-target-features '' reject-cfg x86-runtime-caps-v1 '-C target-feature=+avx2'
run_case manual-target-feature-cfg '' reject-cfg-arg x86-runtime-caps-v1 '--cfg=target_feature="avx2" --cap-lints=allow'
run_case encoded-target-feature-cfg '' reject-cfg-arg x86-runtime-caps-v1 '' '' $'--cfg\x1fcreusot\x1f--cfg\x1ftarget_feature="sse4.2"\x1f--cap-lints=allow'
run_case caller-capability-path '' reject-entry x86-runtime-caps-v1 '' $'capability\tcrate::fake_avx2_usable'

printf 'Gate C fixture artifacts: %s\n' "$run_dir"
printf 'rustc --print cfg target features SHA-256: %s\n' "$target_cfg_hash"
printf 'generated package: %s\n' "$prelude_package_dir"
printf 'generated package SHA-256: %s\n' "$package_sha"
printf 'generated prelude SHA-256: %s\n' "$prelude_sha"
