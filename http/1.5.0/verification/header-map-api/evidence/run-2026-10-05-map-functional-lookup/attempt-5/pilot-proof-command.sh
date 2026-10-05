#!/usr/bin/env bash
set -u
RUSTFLAGS='--cfg http_map_api_leaf --cfg http_map_find_api_leaf' ../../scripts/run-proof.sh cargo creusot --simple-triggers=false prove --no-cache \
  verif/http_header_map_api_proof_rlib/header/map/impl_HeaderMap_T/get2.coma \
  verif/http_header_map_api_proof_rlib/header/map/as_header_name/impl_Sealed_for_HeaderName/find.coma \
  verif/http_header_map_api_proof_rlib/header/map/as_header_name/impl_Sealed_for_HeaderName/find__refines.coma \
  verif/http_header_map_api_proof_rlib/header/map/as_header_name/impl_Sealed_for_ref_HeaderName/find.coma \
  verif/http_header_map_api_proof_rlib/header/map/as_header_name/impl_Sealed_for_ref_HeaderName/find__refines.coma \
  -- --features http_map_api_leaf,http_map_find_api_leaf --locked --offline
