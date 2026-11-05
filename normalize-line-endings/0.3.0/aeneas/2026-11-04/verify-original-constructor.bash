#!/usr/bin/env bash
set -euo pipefail
cd -- "$(dirname -- "${BASH_SOURCE[0]}")"
source /workspace/aeneas-tools/activate.sh
supplement_root="$PWD/evidence/original-constructor"
(cd "$supplement_root"; python3 tools/check-source.py)
(cd "$supplement_root/source"; CARGO_NET_OFFLINE=true charon cargo --preset aeneas --sysroot default --start-from crate::normalized --dest ../evidence > ../evidence/charon.log 2>&1)
mkdir -p "$supplement_root/evidence/regenerated"
aeneas -backend lean -namespace OriginalConstructorGenerated -subdir OriginalConstructorGenerated -abort-on-error -dest "$supplement_root/evidence/regenerated" "$supplement_root/evidence/normalize_line_endings.llbc" > "$supplement_root/evidence/aeneas.log" 2>&1
cmp "$supplement_root/evidence/regenerated/OriginalConstructorGenerated/NormalizeLineEndings.lean" proofs/OriginalConstructorGenerated/NormalizeLineEndings.lean
(cd proofs; lake build; lake env lean OriginalConstructorSupplement.lean) 2>&1 | tee evidence/original-constructor-integrated-audit.log
