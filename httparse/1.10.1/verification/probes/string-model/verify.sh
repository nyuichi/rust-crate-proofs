#!/usr/bin/env bash
set -euo pipefail

probe_dir=$(cd "$(dirname "$0")" && pwd)
repo_root=$(cd "$probe_dir/../../../../.." && pwd)
build_script=$repo_root/tools/creusot-toolpatch/scripts/build-httparse-string-model.sh
manifest=$probe_dir/Cargo.toml
negative_manifest=$probe_dir/negative/Cargo.toml
error_manifest=$probe_dir/error/Cargo.toml
mode=${1:-translate}
shift || true

if [[ ! -x /workspace/proof-tools/targets/httparse-string-model/debug/creusot-rustc || \
      ! -f /workspace/scratch/httparse-string-model/creusot-env.sh ]]; then
  "$build_script"
fi

source /workspace/proof-tools/activate.sh
source /workspace/scratch/httparse-string-model/creusot-env.sh
export RUSTUP_TOOLCHAIN=nightly-2026-02-27
export CARGO_NET_OFFLINE=true
isolated_env=(
  CREUSOT_DATA_HOME=/workspace/scratch/httparse-string-model/creusot-data
  CREUSOT_RUSTC=/workspace/proof-tools/targets/httparse-string-model/debug/creusot-rustc
  CARGO_TARGET_DIR=/workspace/proof-tools/targets/httparse-string-model
  CARGO_NET_OFFLINE=true
  WHY3CONFIG=/workspace/scratch/httparse-string-model/why3.conf
  DUNE_DIR_LOCATIONS=why3find:lib:/workspace/scratch/httparse-string-model/creusot-data/share/why3find
  XDG_CONFIG_HOME=/workspace/scratch/httparse-string-model/config
  XDG_CACHE_HOME=/workspace/scratch/httparse-string-model/cache
)
cd "$probe_dir"

case "$mode" in
  native)
    cargo check --offline --manifest-path "$manifest" "$@"
    ;;
  native-negative)
    cd "$probe_dir/negative"
    cargo check --offline --manifest-path "$negative_manifest" "$@"
    ;;
  native-error)
    cd "$probe_dir/error"
    cargo check --offline --manifest-path "$error_manifest" "$@"
    ;;
  translate)
    cargo creusot -- --manifest-path "$manifest" "$@"
    ;;
  translate-negative)
    cd "$probe_dir/negative"
    cargo creusot -- --manifest-path "$negative_manifest" "$@"
    ;;
  translate-error)
    cd "$probe_dir/error"
    cargo creusot -- --manifest-path "$error_manifest" "$@"
    ;;
  prove)
    cd "$probe_dir"
    mkdir -p "$probe_dir/evidence"
    "$repo_root/httparse/1.10.1/run-proof.bash" \
      env "${isolated_env[@]}" \
      /workspace/scratch/httparse-string-model/creusot-data/bin/why3find prove -s -j 1 --no-cache \
      verif/httparse_string_model_harness_rlib "$@"
    ;;
  prove-negative)
    cd "$probe_dir/negative"
    mkdir -p "$probe_dir/evidence"
    "$repo_root/httparse/1.10.1/run-proof.bash" \
      env "${isolated_env[@]}" \
      /workspace/scratch/httparse-string-model/creusot-data/bin/why3find prove -s -j 1 --no-cache \
      --log-prover-results "$probe_dir/evidence/negative-prover-results.jsonl" \
      verif/httparse_string_model_negative_harness_rlib/deliberately_false_string_claim.coma \
      verif/httparse_string_model_negative_harness_rlib/deliberately_false_utf8_claim.coma "$@"
    ;;
  prove-error)
    cd "$probe_dir/error"
    mkdir -p "$probe_dir/evidence"
    "$repo_root/httparse/1.10.1/run-proof.bash" \
      env "${isolated_env[@]}" \
      /workspace/scratch/httparse-string-model/creusot-data/bin/why3find prove -s -j 1 --no-cache \
      --log-prover-results "$probe_dir/evidence/error-prover-results.jsonl" \
      verif/httparse_string_model_error_harness_rlib/actual_error_description.coma \
      verif/httparse_string_model_error_harness_rlib/impl_Error/description_str.coma \
      verif/httparse_string_model_error_harness_rlib/impl_Error_for_Error/description.coma "$@"
    ;;
  *)
    printf 'Usage: %s [native|native-negative|native-error|translate|translate-negative|translate-error|prove|prove-negative|prove-error] [cargo flags...]\n' "$0" >&2
    exit 2
    ;;
esac
