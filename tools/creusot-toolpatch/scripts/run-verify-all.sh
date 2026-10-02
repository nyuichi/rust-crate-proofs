#!/usr/bin/env bash
set -euo pipefail
script_dir=$(cd "$(dirname "$0")" && pwd)
repo_root=${1:-${RUST_CRATE_PROOFS_ROOT:-/workspace/rust-crate-proofs}}
cd "$repo_root/itoa/1.0.18"
exec "$script_dir/run-proof.sh" ./verify-all.bash
