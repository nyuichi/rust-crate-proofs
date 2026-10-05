#!/usr/bin/env bash
set +e
source /workspace/proof-tools/activate.sh
export CARGO_NET_OFFLINE=true
export CARGO_TARGET_DIR=/tmp/http-map-phase-two-emit-confirm-target
export RUSTFLAGS='--cfg http_map_api_leaf --cfg http_map_find_api_leaf'
cargo creusot --simple-triggers=false --output-dir /workspace/rust-crate-proofs/http/1.5.0/verification/header-map-api/evidence/run-2026-10-05-map-do-insert-phase-two/attempt-1-bounded-loop/emitted-confirm -- --features http_map_api_leaf,http_map_find_api_leaf --locked --offline > /workspace/rust-crate-proofs/http/1.5.0/verification/header-map-api/evidence/run-2026-10-05-map-do-insert-phase-two/attempt-1-bounded-loop/fresh-emission.log 2>&1
emit_status=$?
printf '%s\n' "$emit_status" > /workspace/rust-crate-proofs/http/1.5.0/verification/header-map-api/evidence/run-2026-10-05-map-do-insert-phase-two/attempt-1-bounded-loop/fresh-emission.exit-status
cat /workspace/rust-crate-proofs/http/1.5.0/verification/header-map-api/evidence/run-2026-10-05-map-do-insert-phase-two/attempt-1-bounded-loop/fresh-emission.log
exit "$emit_status"
