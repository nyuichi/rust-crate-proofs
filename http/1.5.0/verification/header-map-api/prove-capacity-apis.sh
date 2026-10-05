#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")"

export RUSTFLAGS='--cfg http_map_api_leaf'
source /workspace/proof-tools/activate.sh

crate_root=$(cd ../.. && pwd)
proof_workspace=$(cd "$crate_root/../.." && pwd)
export CARGO_TARGET_DIR="$proof_workspace/target/http"

cargo clean -p http-header-map-api-proof
cargo creusot --simple-triggers=false -- --features http_map_api_leaf --locked --offline

../../scripts/run-proof.sh cargo creusot --simple-triggers=false prove --no-cache \
  verif/http_header_map_api_proof_rlib/header/map/to_raw_capacity.coma \
  verif/http_header_map_api_proof_rlib/header/map/impl_HeaderMap_T/try_with_capacity.coma \
  verif/http_header_map_api_proof_rlib/header/map/impl_HeaderMap_T/with_capacity.coma \
  verif/http_header_map_api_proof_rlib/header/map_capacity/checked_raw_capacity.coma \
  verif/http_header_map_api_proof_rlib/header/map_capacity/usable_capacity.coma \
  -- --features http_map_api_leaf --locked --offline
