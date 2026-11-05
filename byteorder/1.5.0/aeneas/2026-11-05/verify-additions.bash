#!/usr/bin/env bash
set -euo pipefail
cd -- "$(dirname -- "${BASH_SOURCE[0]}")"
source "${AENEAS_HOME:-/workspace/aeneas-tools}/activate.sh"
export CARGO_NET_OFFLINE=true
python3 tools/check-source.py | tee evidence/source-provenance.log
python3 tools/extract.py | tee evidence/selector-guard.log
(cd proofs; lake build; lake env lean Proofs.lean; lake env lean SignedProofs.lean; lake env lean FloatProofs.lean) 2>&1 | tee evidence/additions-verification.log
python3 tools/check-audits.py evidence/additions-verification.log
