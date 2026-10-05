#!/usr/bin/env bash
set -u
source /workspace/proof-tools/activate.sh
export CARGO_NET_OFFLINE=true
export CARGO_TARGET_DIR=/workspace/rust-crate-proofs/target/http
RUSTFLAGS='--cfg http_map_api_leaf --cfg http_map_find_api_leaf' cargo creusot --simple-triggers=false -- --features http_map_api_leaf,http_map_find_api_leaf --locked --offline
exit $?
