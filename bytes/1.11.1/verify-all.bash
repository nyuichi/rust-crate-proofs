#!/usr/bin/env bash
# Select `verified` for the modified production API (defaults to std/host).
# The default retains the original runtime entry and its recorded blockers.
set -euo pipefail
script_dir=$(cd "$(dirname "$0")" && pwd)
if [[ "${1:-}" == "verified" ]]; then
    shift
    if [[ $# -ne 0 ]]; then
        printf 'verified accepts configuration through BYTES_VERIFIED_FEATURES/BYTES_VERIFIED_TARGET\n' >&2
        exit 2
    fi
    exec "$script_dir/scripts/verify-verified-bytes.sh"
fi
exec "$script_dir/scripts/verify-bytes.sh" runtime "$@"
