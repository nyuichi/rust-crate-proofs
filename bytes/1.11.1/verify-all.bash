#!/usr/bin/env bash
# Verify the original bytes 1.11.1 runtime; retired alternative APIs are not targets.
set -euo pipefail
script_dir=$(cd "$(dirname "$0")" && pwd)
if [[ "${1:-}" == "verified" ]]; then
    printf 'The alternative verified API was removed; see verification/REMOVAL_2026-10-07_JA.md\n' >&2
    exit 2
fi
exec "$script_dir/scripts/verify-bytes.sh" runtime "$@"
