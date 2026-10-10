#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")"

# Translate and prove the actual Error formatter bodies with the same source
# profile. The dynamic Error APIs are excluded only from this proof harness.
export RUSTFLAGS='--cfg http_composition_leaf --cfg http_error_fmt_leaf'
source /workspace/proof-tools/activate.sh

crate_root=$(cd ../.. && pwd)
proof_workspace=$(cd "$crate_root/../.." && pwd)
export CARGO_TARGET_DIR="$proof_workspace/target/http"

cargo clean -p http-error-formatting-proof
cargo creusot --simple-triggers=false -- --locked --offline

../../scripts/run-proof.sh cargo creusot --simple-triggers=false prove --no-cache \
  verif/http_error_formatting_proof_rlib/error/impl_Debug_for_Error/fmt.coma \
  verif/http_error_formatting_proof_rlib/error/impl_Display_for_Error/fmt.coma \
  verif/http_error_formatting_proof_rlib/error/impl_Debug_for_Error/fmt__refines.coma \
  verif/http_error_formatting_proof_rlib/error/impl_Display_for_Error/fmt__refines.coma \
  -- --locked --offline
