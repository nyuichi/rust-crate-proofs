#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")"
source /workspace/aeneas-tools/activate.sh
python3 reproduce-frontends.py
python3 check-source-ir.py
# Prerequisite: the pinned sibling nohash-length-prefix family and old public
# proof environment have been verified/built from their unchanged sources.
packet="$PWD"
cd /workspace/full-api/endian/nohash-publication-candidate/nohash-hasher/0.2.0/aeneas/2026-11-05/proofs
lake env python3 "$packet/run-lean.py"
lake env python3 "$packet/run-additional-lean.py"
lake env python3 "$packet/run-coimport.py"
