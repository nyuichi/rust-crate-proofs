#!/usr/bin/env bash
# Experimental IR adaptation only; not used by the production report.
set -euo pipefail
cd -- "$(dirname -- "${BASH_SOURCE[0]}")/../.."
source "${AENEAS_HOME:-/workspace/aeneas-tools}/activate.sh"
experiment=evidence/original-mir-options
[[ "$(git -C "${AENEAS_HOME:-/workspace/aeneas-tools}/aeneas" rev-parse HEAD)" == be86ab0f1989d04944292b345b24c435be3ad10a ]]
[[ "$(git -C "${AENEAS_HOME:-/workspace/aeneas-tools}/aeneas/charon" rev-parse HEAD)" == 08d7c9d50144bd84848212c24701253585e90d1f ]]
printf '%s\n' '1ae29ba0393192db102a26bf8db05c15ed05adc7c4474ad1bde05ef2bc937225  src/lib.rs' | sha256sum --check
charon cargo --preset aeneas --sysroot default --no-dedup-serialized-ast --dest "$experiment" > "$experiment/charon-plain-json.log" 2>&1
python3 "$experiment/normalize_char_switch.py"
aeneas -backend lean -emit-json -abort-on-error -warnings-as-errors -dest "$experiment/generated" "$experiment/normalized.llbc" > "$experiment/aeneas-normalized-ir.log" 2>&1
mkdir -p "$experiment/proofs/Generated" "$experiment/proofs/.lake"
cp proofs/{Proofs.lean,NormalizationSpec.lean,lakefile.lean,lake-manifest.json,lean-toolchain} "$experiment/proofs/"
cp "$experiment/generated/Normalized.lean" "$experiment/proofs/Generated/NormalizeLineEndings.lean"
if [[ -d proofs/.lake/packages && ! -e "$experiment/proofs/.lake/packages" ]]; then
  ln -s "$(pwd)/proofs/.lake/packages" "$experiment/proofs/.lake/packages"
fi
(
  cd "$experiment/proofs"
  lake build > ../normalized-ir-proof-build.log 2>&1
  lake env lean Proofs.lean > ../normalized-ir-axioms.log 2>&1
)
