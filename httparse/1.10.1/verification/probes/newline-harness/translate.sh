#!/usr/bin/env bash
set -euo pipefail

script_dir=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
source /workspace/proof-tools/activate.sh
source /workspace/scratch/httparse-string-model/creusot-env.sh
export RUSTUP_TOOLCHAIN=nightly-2026-02-27
export CARGO_NET_OFFLINE=true
export CARGO_TARGET_DIR="$script_dir/target/string-creusot"
cd "$script_dir/string"

case "${1:-translate}" in
  translate)
    cargo creusot -- --manifest-path "$script_dir/string/Cargo.toml"
    ;;
  *)
    echo "usage: $0 [translate]" >&2
    exit 2
    ;;
esac
