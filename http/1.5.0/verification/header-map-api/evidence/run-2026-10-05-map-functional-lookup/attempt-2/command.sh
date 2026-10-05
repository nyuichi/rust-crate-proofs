#!/usr/bin/env bash
set -u
source /workspace/proof-tools/activate.sh
export CARGO_NET_OFFLINE=true
export CARGO_TARGET_DIR=/workspace/rust-crate-proofs/target/http
cargo clean --manifest-path Cargo.toml -p http-header-map-api-proof
clean_status=$?
if [ "$clean_status" -ne 0 ]; then
  echo "cargo clean failed: $clean_status" >&2
  exit "$clean_status"
fi
RUSTFLAGS='--cfg http_map_api_leaf --cfg http_map_find_api_leaf' cargo creusot --simple-triggers=false -- --features http_map_api_leaf,http_map_find_api_leaf --locked --offline
exit $?
