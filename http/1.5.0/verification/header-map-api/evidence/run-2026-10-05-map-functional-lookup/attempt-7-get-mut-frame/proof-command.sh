#!/usr/bin/env bash
set -euo pipefail
source /workspace/proof-tools/activate.sh
script_dir=$(cd "$(dirname "$0")" && pwd)
package_dir=$(cd "$script_dir/../../.." && pwd)
cd "$package_dir"
RUSTFLAGS='--cfg http_map_api_leaf --cfg http_map_find_api_leaf' ../../scripts/run-proof.sh cargo creusot --simple-triggers=false prove --no-cache \
  $(cat "$script_dir/prepared-targets.txt") \
  -- --features http_map_api_leaf,http_map_find_api_leaf --locked --offline
