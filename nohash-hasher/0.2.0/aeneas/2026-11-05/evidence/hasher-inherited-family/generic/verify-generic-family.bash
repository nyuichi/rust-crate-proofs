#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")"
source /workspace/aeneas-tools/activate.sh
python3 regenerate-family.py
family_root="$PWD"
# Existing pinned environment supplies the unchanged old public module for coimport.
cd /workspace/full-api/endian/nohash-publication-candidate/nohash-hasher/0.2.0/aeneas/2026-11-05/proofs
lake env python3 "$family_root/run-lean-family.py"
