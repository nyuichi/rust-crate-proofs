#!/usr/bin/env bash
set -euo pipefail

probe_dir=$(cd "$(dirname "$0")" && pwd)
repo_root=$(cd "$probe_dir/../../../../.." && pwd)
manifest=$probe_dir/Cargo.toml
negative_manifest=$probe_dir/negative/Cargo.toml
mode=${1:-translate}
shift || true

proof_env=(
  CREUSOT_DATA_HOME=/workspace/scratch/httparse-derived-traits/creusot-data
  CREUSOT_RUSTC=/workspace/proof-tools/targets/httparse-derived-traits/debug/creusot-rustc
  CARGO_TARGET_DIR=/workspace/proof-tools/targets/httparse-derived-traits
  CARGO_NET_OFFLINE=true
  WHY3CONFIG=/workspace/scratch/httparse-derived-traits/why3.conf
  DUNE_DIR_LOCATIONS=why3find:lib:/workspace/scratch/httparse-derived-traits/creusot-data/share/why3find
  XDG_CONFIG_HOME=/workspace/scratch/httparse-derived-traits/config
  XDG_CACHE_HOME=/workspace/scratch/httparse-derived-traits/cache
)

source /workspace/proof-tools/activate.sh
source /workspace/scratch/httparse-derived-traits/creusot-env.sh
export RUSTUP_TOOLCHAIN=nightly-2026-02-27
export CARGO_NET_OFFLINE=true
cd "$probe_dir"

case "$mode" in
  native)
    cargo check --offline --manifest-path "$manifest" "$@"
    ;;
  translate)
    cargo creusot -- --manifest-path "$manifest" "$@"
    ;;
  negative-native)
    cd "$probe_dir/negative"
    cargo check --offline --manifest-path "$negative_manifest" "$@"
    ;;
  negative-translate)
    cd "$probe_dir/negative"
    cargo creusot -- --manifest-path "$negative_manifest" "$@"
    ;;
  typecheck-positive|prove-positive)
    cd "$probe_dir"
    mapfile -t targets < <(sed '/^[[:space:]]*#/d; /^[[:space:]]*$/d' "$probe_dir/proof-positive.targets")
    if [[ "$mode" == typecheck-positive ]]; then
      why3 prove --type-only -C "$WHY3CONFIG" \
        -L "$CREUSOT_DATA_HOME/share/why3find/packages/creusot" -L verif \
        "${targets[@]}" "$@"
    else
      mkdir -p "$probe_dir/evidence"
      "$repo_root/httparse/1.10.1/run-proof.bash" \
        env "${proof_env[@]}" \
        /workspace/scratch/httparse-derived-traits/creusot-data/bin/why3find prove -s -j 1 --no-cache \
        --log-prover-results "$probe_dir/evidence/derived-positive-prover-results.jsonl" \
        "${targets[@]}" "$@"
    fi
    ;;
  typecheck-negative|prove-negative)
    cd "$probe_dir/negative"
    mapfile -t targets < <(sed '/^[[:space:]]*#/d; /^[[:space:]]*$/d' "$probe_dir/proof-negative.targets")
    if [[ "$mode" == typecheck-negative ]]; then
      why3 prove --type-only -C "$WHY3CONFIG" \
        -L "$CREUSOT_DATA_HOME/share/why3find/packages/creusot" -L verif \
        "${targets[@]}" "$@"
    else
      mkdir -p "$probe_dir/evidence"
      "$repo_root/httparse/1.10.1/run-proof.bash" \
        env "${proof_env[@]}" \
        /workspace/scratch/httparse-derived-traits/creusot-data/bin/why3find prove -s -j 1 --no-cache \
        --log-prover-results "$probe_dir/evidence/derived-negative-prover-results.jsonl" \
        "${targets[@]}" "$@"
    fi
    ;;
  *)
    printf 'Usage: %s [native|translate|negative-native|negative-translate|typecheck-positive|typecheck-negative|prove-positive|prove-negative] [flags...]\n' "$0" >&2
    exit 2
    ;;
esac
