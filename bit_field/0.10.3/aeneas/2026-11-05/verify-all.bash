#!/usr/bin/env bash
set -euo pipefail
cd -- "$(dirname -- "${BASH_SOURCE[0]}")"
source "${AENEAS_HOME:-/workspace/aeneas-tools}/activate.sh"
export CARGO_NET_OFFLINE=true
python3 tools/check-source.py | tee evidence/source-provenance.log
cargo test --locked 2>&1 | tee evidence/original-tests.log
sha256sum proofs/BitField/{Types.lean,Funs.lean,FunsExternal_Template.lean,FunsExternal.lean,translation.json} proofs/BitArrayGenerated/{BitField.lean,translation.json} > evidence/generated-before.sha256
python3 tools/extract.py 2>&1 | tee evidence/extraction.log
sha256sum --check evidence/generated-before.sha256
if rg -n '\b(sorry|axiom)\b' proofs/Proofs.lean proofs/ArrayProofs.lean proofs/ArrayBitsProofs.lean proofs/BitArrayGenerated/BitField.lean proofs/BitField/{Types,Funs,FunsExternal}.lean; then exit 1; fi
python3 tools/write-union-audit.py
(cd proofs; lake build +Proofs; lake env lean ../evidence/UnionAudit.lean) 2>&1 | tee evidence/verification.log
python3 tools/check-layout-elaboration.py
python3 tools/check-audits.py evidence/verification.log
