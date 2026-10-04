#!/usr/bin/env bash
# Current verification scope is default std, x86_64, native atomics.
# This entry intentionally verifies the actual runtime, and currently reports
# the recorded translation blockers. Probe proofs are separate partial results.
set -euo pipefail
script_dir=$(cd "$(dirname "$0")" && pwd)
exec "$script_dir/scripts/verify-bytes.sh" runtime "$@"
