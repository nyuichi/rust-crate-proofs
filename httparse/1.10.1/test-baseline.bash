#!/usr/bin/env bash
set -euo pipefail

script_dir=$(cd "$(dirname "$0")" && pwd)
cd "$script_dir"

source /workspace/proof-tools/activate.sh
cargo test --locked
cargo test --locked --no-default-features
