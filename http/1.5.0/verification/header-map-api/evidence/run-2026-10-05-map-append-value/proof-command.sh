#!/usr/bin/env bash
set -u
script_dir=$(cd "$(dirname "$0")" && pwd)
package_dir=$(cd "$script_dir/../.." && pwd)
cd "$package_dir"
RUSTFLAGS='--cfg http_map_api_leaf --cfg http_map_find_api_leaf' ../../scripts/run-proof.sh cargo creusot --simple-triggers=false prove --no-cache \
  verif/http_header_map_api_proof_rlib/header/map/append_value_resume.coma \
  -- --features http_map_api_leaf,http_map_find_api_leaf --locked --offline
exit $?
