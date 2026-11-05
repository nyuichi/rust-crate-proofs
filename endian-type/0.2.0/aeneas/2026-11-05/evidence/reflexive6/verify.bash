#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")"
source /workspace/aeneas-tools/activate.sh
python3 check-source-ir.py
python3 reproduce-frontends.py
packet="$PWD"
# Any already prepared pinned Aeneas Lean environment is suitable; this environment
# supplies only dependencies, not an accidental import of its own public Proofs.
cd /workspace/full-api/endian/nohash-publication-candidate/nohash-hasher/0.2.0/aeneas/2026-11-05/proofs
lake env python3 "$packet/run-lean.py"
lake env python3 "$packet/run-coimport.py"
