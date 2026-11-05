#!/usr/bin/env bash
set -euo pipefail
cd -- "$(dirname -- "${BASH_SOURCE[0]}")"
source /workspace/aeneas-tools/activate.sh
python3 tools/check-source.py
(cd source; CARGO_NET_OFFLINE=true charon cargo --preset aeneas --sysroot default --start-from crate::normalized --dest ../evidence > ../evidence/charon.log 2>&1)
mkdir -p evidence/regenerated
 aeneas -backend lean -namespace OriginalConstructorGenerated -subdir OriginalConstructorGenerated -abort-on-error -dest evidence/regenerated evidence/normalize_line_endings.llbc > evidence/aeneas.log 2>&1
cmp evidence/regenerated/OriginalConstructorGenerated/NormalizeLineEndings.lean proofs/OriginalConstructorGenerated/NormalizeLineEndings.lean
(cd proofs; lake build > ../evidence/lean-build.log 2>&1; lake env lean OriginalConstructorSupplement.lean > ../evidence/audit.log 2>&1)
python3 tools/check-source.py
