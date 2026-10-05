#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")"

# Keep the targeted Request shortcut profile identical for cleaning,
# translation, and proof. `http_composition_leaf` omits these entrypoints by
# itself; `http_builder_entrypoints_leaf` restores just that source group.
export RUSTFLAGS='--cfg http_composition_leaf --cfg http_builder_entrypoints_leaf'
source /workspace/proof-tools/activate.sh

crate_root=$(cd ../.. && pwd)
proof_workspace=$(cd "$crate_root/../.." && pwd)
export CARGO_TARGET_DIR="$proof_workspace/target/http"

cargo clean -p http-composition-proof
cargo creusot --simple-triggers=false -- --locked --offline

../../scripts/run-proof.sh cargo creusot --simple-triggers=false prove --no-cache \
  verif/http_composition_proof_rlib/request/impl_Request_unit/get.coma \
  verif/http_composition_proof_rlib/request/impl_Request_unit/put.coma \
  verif/http_composition_proof_rlib/request/impl_Request_unit/post.coma \
  verif/http_composition_proof_rlib/request/impl_Request_unit/delete.coma \
  verif/http_composition_proof_rlib/request/impl_Request_unit/options.coma \
  verif/http_composition_proof_rlib/request/impl_Request_unit/head.coma \
  verif/http_composition_proof_rlib/request/impl_Request_unit/connect.coma \
  verif/http_composition_proof_rlib/request/impl_Request_unit/patch.coma \
  verif/http_composition_proof_rlib/request/impl_Request_unit/trace.coma \
  -- --locked --offline
