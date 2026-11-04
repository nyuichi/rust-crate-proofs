#!/usr/bin/env bash
set -euo pipefail
cd -- "$(dirname -- "${BASH_SOURCE[0]}")"
# AENEAS_HOME may point to an alternative installation of the pinned tools.
tool_home="${AENEAS_HOME:-/workspace/aeneas-tools}"
if [[ -f "$tool_home/activate.sh" ]]; then
  source "$tool_home/activate.sh"
fi
python3 - <<'PY'
from pathlib import Path
original = Path('src/lib.rs').read_text()
expected = original.replace("Some('\\n') if self.prev_was_cr", "Some(c) if c == '\\n' && self.prev_was_cr").replace("Some('\\r') =>", "Some(c) if c == '\\r' =>")
assert Path('translation-source/src/lib.rs').read_text() == expected
PY
mkdir -p evidence proofs/Generated
cargo test --manifest-path Cargo.toml 2>&1 | tee evidence/original-tests.log
cargo test --manifest-path translation-source/Cargo.toml 2>&1 | tee evidence/adapted-tests.log
sha256sum proofs/Generated/NormalizeLineEndings.lean proofs/Generated/translation.json \
  > evidence/generated-before.sha256
(
  cd translation-source
  charon cargo --preset=aeneas --sysroot default
) 2>&1 | tee evidence/charon-guard.log
aeneas -backend lean -emit-json -abort-on-error -warnings-as-errors \
  -dest proofs/Generated translation-source/normalize_line_endings.llbc \
  2>&1 | tee evidence/aeneas-guard.log
sha256sum --check evidence/generated-before.sha256
if rg -n '\bsorry\b|\baxiom\b' proofs/Generated --glob '*.lean'; then
  echo 'Unproved generated declaration found' >&2
  exit 1
fi
(
  cd proofs
  lake build
  lake env lean Proofs.lean 2>&1 | tee ../evidence/axioms.log
) 2>&1 | tee evidence/verification.log
python3 - <<'AUDIT'
import json, re
from pathlib import Path
llbc = json.loads(Path('translation-source/normalize_line_endings.llbc').read_text())
assert llbc['has_errors'] is False
log = Path('evidence/axioms.log').read_text()
match = re.search(r"'Proofs.normalized_spec' depends on axioms: \[(.*?)\]", log)
assert match, 'No axiom audit result found'
assert set(match[1].split(', ')) <= {'propext', 'Classical.choice', 'Quot.sound'}
print('Public theorem axiom audit passed')
AUDIT
sha256sum src/lib.rs translation-source/src/lib.rs \
  proofs/Generated/NormalizeLineEndings.lean proofs/Generated/translation.json \
  > evidence/source-generated.sha256
