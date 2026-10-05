#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")"

# This profile imports the exact production Uri, StatusCode, and Version
# modules. `http_uri_default_leaf` omits only Uri::Builder and the generic
# `from_maybe_shared` optimization; `http_uri_leaf` omits its dyn Any parsing
# helpers. Uri's ordinary Display/Debug implementations remain in the profile.
export RUSTFLAGS='--cfg http_composition_leaf --cfg http_builder_entrypoints_leaf --cfg http_composition_uri_defaults_leaf --cfg http_uri_default_leaf --cfg http_uri_leaf --cfg http_uri_authority_scanner_leaf --cfg http_uri_scheme_leaf --cfg http_uri_host_leaf'
source /workspace/proof-tools/activate.sh

crate_root=$(cd ../.. && pwd)
proof_workspace=$(cd "$crate_root/../.." && pwd)
export CARGO_TARGET_DIR="$proof_workspace/target/http"

cargo clean -p http-composition-proof
cargo creusot --simple-triggers=false -- --locked --offline

../../scripts/run-proof.sh cargo creusot --simple-triggers=false prove --no-cache \
  verif/http_composition_proof_rlib/uri/impl_Default_for_Uri/default.coma \
  verif/http_composition_proof_rlib/status/impl_Default_for_StatusCode/default.coma \
  verif/http_composition_proof_rlib/version/impl_Default_for_Version/default.coma \
  verif/http_composition_proof_rlib/request/impl_Parts/new.coma \
  verif/http_composition_proof_rlib/response/impl_Parts/new.coma \
  verif/http_composition_proof_rlib/request/impl_Default_for_Builder/default.coma \
  verif/http_composition_proof_rlib/response/impl_Default_for_Builder/default.coma \
  verif/http_composition_proof_rlib/request/impl_Builder/new.coma \
  verif/http_composition_proof_rlib/response/impl_Builder/new.coma \
  verif/http_composition_proof_rlib/request/impl_Request_unit/builder.coma \
  verif/http_composition_proof_rlib/response/impl_Response_unit/builder.coma \
  -- --locked --offline
