#!/usr/bin/env bash
set -euo pipefail
probe_dir=$(cd "$(dirname "$0")" && pwd)
mode=${1:-translate}
shift || true
source /workspace/proof-tools/activate.sh
export CREUSOT_DATA_HOME=/workspace/scratch/httparse-derived-traits-nativeenv/creusot-data
export CREUSOT_RUSTC=/workspace/proof-tools/targets/httparse-derived-traits-nativeenv/debug/creusot-rustc
export CARGO_TARGET_DIR=/workspace/proof-tools/targets/httparse-derived-traits-nativeenv
export CARGO_NET_OFFLINE=true
export WHY3CONFIG=/workspace/scratch/httparse-derived-traits-nativeenv/why3.conf
export DUNE_DIR_LOCATIONS=why3find:lib:/workspace/scratch/httparse-derived-traits-nativeenv/creusot-data/share/why3find
export XDG_CONFIG_HOME=/workspace/scratch/httparse-derived-traits-nativeenv/config
export XDG_CACHE_HOME=/workspace/scratch/httparse-derived-traits-nativeenv/cache
export RUSTUP_TOOLCHAIN=nightly-2026-02-27
cd "$probe_dir"
case "$mode" in
  translate-positive)
    cargo creusot -- --manifest-path "$probe_dir/Cargo.toml" "$@"
    ;;
  translate-negative)
    cd "$probe_dir/negative"
    cargo creusot -- --manifest-path "$probe_dir/negative/Cargo.toml" "$@"
    ;;
  typecheck-positive|typecheck-negative|typecheck-all-positive|typecheck-all-negative)
    if [[ "$mode" == typecheck-negative || "$mode" == typecheck-all-negative ]]; then
      cd "$probe_dir/negative"
      if [[ "$mode" == typecheck-all-negative ]]; then
        mapfile -t targets < <(find "$probe_dir/negative/verif" -type f -name '*.coma' | sort)
      else
        target_list="$probe_dir/proof-negative.targets"
        mapfile -t targets < <(sed '/^[[:space:]]*#/d; /^[[:space:]]*$/d' "$target_list")
      fi
    elif [[ "$mode" == typecheck-all-positive ]]; then
      mapfile -t targets < <(find "$probe_dir/verif" -type f -name '*.coma' | sort)
    else
      target_list="$probe_dir/proof-positive.targets"
      mapfile -t targets < <(sed '/^[[:space:]]*#/d; /^[[:space:]]*$/d' "$target_list")
    fi
    why3 prove --type-only -C "$WHY3CONFIG" \
      -L "$CREUSOT_DATA_HOME/share/why3find/packages/creusot" -L verif \
      "${targets[@]}" "$@"
    ;;
  failclosed-generic)
    cd "$probe_dir/failclosed-generic"
    log="$probe_dir/evidence/generic-failclosed-translation.log"
    mkdir -p "$probe_dir/evidence"
    if cargo creusot -- --manifest-path "$probe_dir/failclosed-generic/Cargo.toml" "$@" >"$log" 2>&1; then
      cat "$log"
      printf 'Expected the generic derived method to fail closed, but translation succeeded.\n' >&2
      exit 1
    fi
    if ! rg -q 'cannot safely inherit the trait specification for this automatically-derived method: inherited predicates are not proven by its native parameter environment' "$log"; then
      cat "$log"
      printf 'Generic fixture failed for a reason other than the expected native-environment diagnostic.\n' >&2
      exit 1
    fi
    cat "$log"
    ;;
  *)
    printf 'Usage: %s [translate-positive|translate-negative|typecheck-positive|typecheck-negative|typecheck-all-positive|typecheck-all-negative|failclosed-generic] [flags...]\n' "$0" >&2
    exit 2
    ;;
esac
