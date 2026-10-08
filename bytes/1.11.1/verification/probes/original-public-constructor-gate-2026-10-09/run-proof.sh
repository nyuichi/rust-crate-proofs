#!/usr/bin/env bash
# Constructor/read source gate; includes both unrestricted From refinements.
set -euo pipefail
cd "$(dirname "$0")"
source /workspace/bytes-proof-tools/activate.sh
export CARGO_NET_OFFLINE=true
exec 9>/tmp/itoa-creusot-proof.lock
flock 9
feature_tree=$(cargo tree --locked -e features --edges normal,build --features public_constructor "$@")
if [[ "$feature_tree" == *'creusot-std feature "sc-drf"'* ]]; then exit 2; fi
# Cargo does not track emitted verif files when switching back to cached features.
# Force this package (not dependencies) to retranslate before removing dangling tasks.
cargo clean --package bytes-original-public-constructor-gate
cargo creusot --only=coma -- --locked --features public_constructor "$@"
cargo creusot clean --force
export DUNE_DIR_LOCATIONS="why3find:lib:$CREUSOT_DATA_HOME/share/why3find"
export WHY3CONFIG="$CREUSOT_DATA_HOME/creusot_why3.conf"
python3 - <<'PY'
import json,pathlib,subprocess
root=pathlib.Path('verif/bytes_original_public_constructor_gate_rlib/public_constructor')
paths=sorted(str(p) for p in root.rglob('*.coma'))
assert paths and len([p for p in paths if p.endswith('/from__refines.coma')]) == 2
pathlib.Path('generated/proof-targets.json').write_text(json.dumps({'included':paths,'excluded':{},'scope':'Actual constructor/read bodies and both unrestricted From refinements; dependencies are component evidence, not whole-crate integration.','not_claimed':['Promotable Clone/promotion','closed-client final arrival','automatic Drop','whole-crate architecture admission']},indent=2)+'\n')
raise SystemExit(subprocess.call(['why3find','prove','--no-autodetect-provers','-j','1',*paths]))
PY
