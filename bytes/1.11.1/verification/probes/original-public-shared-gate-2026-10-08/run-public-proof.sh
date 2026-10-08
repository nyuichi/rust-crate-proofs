#!/usr/bin/env bash
# Elevated execution required for Why3 sockets. This is a selected body gate,
# not the all-domain From trait or the unconditional final-cleanup theorem.
set -euo pipefail
cd "$(dirname "$0")"
source /workspace/bytes-proof-tools/activate.sh
export CARGO_NET_OFFLINE=true
exec 9>/tmp/itoa-creusot-proof.lock
flock 9
feature_tree=$(cargo tree --locked -e features --edges normal,build --features public_shared)
if [[ "$feature_tree" == *'creusot-std feature "sc-drf"'* ]]; then exit 2; fi
cargo creusot --only=coma -- --locked --features public_shared
cargo creusot clean --force
export DUNE_DIR_LOCATIONS="why3find:lib:$CREUSOT_DATA_HOME/share/why3find"
export WHY3CONFIG="$CREUSOT_DATA_HOME/creusot_why3.conf"
python3 - <<'PY'
import json,pathlib,subprocess
r=pathlib.Path('verif/bytes_original_public_shared_gate_rlib/public_shared')
paths=sorted(str(p) for p in r.rglob('*.coma') if p.name!='from__refines.coma')
pathlib.Path('generated/public-proof-targets.json').write_text(json.dumps({'included':paths,'excluded':{'from__refines.coma':'Invalid full-domain len<capacity precondition; Vec::new is a counterexample.'},'not_claimed':'At least one/once final cleanup; automatic Drop; other representations.'},indent=2)+'\n')
raise SystemExit(subprocess.call(['why3find','prove','--no-autodetect-provers','-j','1',*paths]))
PY
