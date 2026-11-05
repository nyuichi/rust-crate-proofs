#!/usr/bin/env bash
set -euo pipefail
cd -- "$(dirname -- "${BASH_SOURCE[0]}")"
source "${AENEAS_HOME:-/workspace/aeneas-tools}/activate.sh"
export CARGO_NET_OFFLINE=true
python3 tools/check-source.py | tee evidence/source-provenance.log
python3 tools/extract.py | tee evidence/selector-guard.log
python3 tools/write-union-audit.py
(cd proofs; for proof_module in Proofs SignedProofs FloatProofs VariableProofs ReadVariableProofs U24Exact24 U24Write24 SignedVariableContracts SignedReadProofs NativeIdentityProofs; do lake build +"$proof_module"; done; lake env lean ../evidence/UnionAudit.lean) 2>&1 | tee evidence/verification.log
python3 tools/check-audits.py evidence/verification.log
