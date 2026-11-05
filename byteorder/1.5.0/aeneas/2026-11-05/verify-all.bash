#!/usr/bin/env bash
set -euo pipefail
cd -- "$(dirname -- "${BASH_SOURCE[0]}")"
source "${AENEAS_HOME:-/workspace/aeneas-tools}/activate.sh"
export CARGO_NET_OFFLINE=true
python3 tools/check-source.py | tee evidence/source-provenance.log
cargo test 2>&1 | tee evidence/original-tests.log
find proofs/Generated proofs/ByteorderApiRoots proofs/ByteorderFloatRoots proofs/ReadVariableGenerated proofs/ByteorderU24Roots proofs/Byteorder -maxdepth 1 -type f -print0 | sort -z | xargs -0 sha256sum > evidence/generated-before.sha256
python3 tools/extract.py
sha256sum --check evidence/generated-before.sha256
if rg -n '\bsorry\b|\baxiom\b' proofs --glob '*.lean' --glob '!**/.lake/**' --glob '!**/FunsExternal_Template.lean'; then exit 1; fi
(cd proofs; for proof_module in Proofs SignedProofs FloatProofs ReadVariableProofs U24Exact24 NativeIdentityProofs; do lake build +"$proof_module"; done) 2>&1 | tee evidence/verification.log

python3 tools/write-union-audit.py
(cd proofs; lake env lean ../evidence/UnionAudit.lean) 2>&1 | tee evidence/union-audit.log
python3 tools/check-audits.py evidence/union-audit.log
