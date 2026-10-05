#!/usr/bin/env bash
set -euo pipefail
source /workspace/proof-tools/activate.sh
export DUNE_DIR_LOCATIONS='why3find:lib:/workspace/proof-tools/creusot-data/share/why3find'
script_dir=$(cd "$(dirname "$0")" && pwd)
package_dir=$(cd "$script_dir/../../../../.." && pwd)
cd "$package_dir"
exec ../../scripts/run-proof.sh /workspace/proof-tools/creusot-data/bin/why3find prove --root . --no-cache --show-progress always evidence/run-2026-10-05-map-try-insert-entry-prefix/attempt-1-frontend/emitted-fresh/verif/http_header_map_api_proof_rlib/header/map/impl_HeaderMap_T/try_insert_entry.coma
