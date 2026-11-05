#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")"
python3 regenerate.py
cd proofs
lake build Proofs OriginalDelegationProofs
lake env lean WholeAudit.lean
