#!/usr/bin/env bash
set -euo pipefail
cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.."
source /workspace/aeneas-tools/activate.sh
CARGO_NET_OFFLINE=true charon cargo --preset aeneas --sysroot default --monomorphize --start-from crate::scalar_first --include normalize_line_endings --dest investigation/scalar-mono > investigation/scalar-mono/charon.log 2>&1
if aeneas -backend lean -abort-on-error -dest investigation/scalar-mono investigation/scalar-mono/normalize_original_contracts.llbc > investigation/scalar-mono/aeneas.log 2>&1; then
 echo 'Unexpected success: reassess the recorded blocker'; exit 1
fi
grep -q 'Unsupported switch scrutinee' investigation/scalar-mono/aeneas.log
