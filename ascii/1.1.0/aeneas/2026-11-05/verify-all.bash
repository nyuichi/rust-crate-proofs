#!/usr/bin/env bash
set -euo pipefail
cd -- "$(dirname -- "${BASH_SOURCE[0]}")"
source "${AENEAS_HOME:-/workspace/aeneas-tools}/activate.sh"
export CARGO_NET_OFFLINE=true
export RUSTFLAGS="-A bindings_with_variant_name"
export RUSTDOCFLAGS="-Z unstable-options --doctest-build-arg=-A --doctest-build-arg=bindings_with_variant_name"
python3 tools/check-source.py | tee evidence/source-provenance.log
set +e
cargo test 2>&1 | tee evidence/original-tests-full.log
original_status=${PIPESTATUS[0]}
set -e
if [[ "$original_status" -ne 0 ]]; then
 python3 - <<'TESTCHECK'
from pathlib import Path
s=Path('evidence/original-tests-full.log').read_text()
assert 'test result: FAILED. 36 passed; 1 failed;' in s
assert 'ascii_char::tests::is_digit_strange_radixes' in s
assert 'to_digit: invalid radix' in s
print('Observed upstream radix-0/1 test incompatibility with pinned Rust; in the test comparison with standard char::is_digit, not the generated crate body')
TESTCHECK
fi
cargo test -- --skip ascii_char::tests::is_digit_strange_radixes 2>&1 | tee evidence/original-tests.log
sha256sum proofs/Generated/{Ascii.lean,translation.json} > evidence/generated-before.sha256
python3 tools/extract.py
sha256sum --check evidence/generated-before.sha256
if rg -n '\bsorry\b|\baxiom\b' proofs --glob '*.lean' --glob '!**/.lake/**'; then exit 1; fi
(cd proofs; lake build; lake env lean Proofs.lean) 2>&1 | tee evidence/verification.log
python3 - <<'INNER'
from pathlib import Path
import re
rows=re.findall(r"'Proofs\.(\w+)' depends on axioms: \[(.*?)\]",Path('evidence/verification.log').read_text())
assert len(rows)>=25,rows
for n,a in rows:assert set(a.split(', '))<={'propext','Classical.choice','Quot.sound'},(n,a)
print('All twenty-four API theorem and casefold helper transitive axiom audits passed')
INNER
