#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")"
source /workspace/bytes-proof-tools/activate.sh
export CARGO_NET_OFFLINE=true
exec 9>/tmp/itoa-creusot-proof.lock
flock 9
mkdir -p .proof-config/creusot generated
sed 's/^memlimit = .*/memlimit = 1024/' "$XDG_CONFIG_HOME/creusot/why3.conf" > .proof-config/creusot/why3.conf
export XDG_CONFIG_HOME="$PWD/.proof-config"
printf 'Generic pointer diagnostic control; one prover; 1024 MiB; sc-drf disabled\n'
features=$(cargo tree --locked -e features --edges normal,build "$@")
if [[ "$features" == *'creusot-std feature "sc-drf"'* ]]; then exit 2; fi
cargo clean --package bytes-owned-pointer-prerequisite
rm -rf verif .why3find
cargo creusot --only=coma -- --locked "$@"
cargo creusot clean --force
python3 - "$@" <<'PY'
import pathlib,json,sys
paths=sorted(p.as_posix() for p in pathlib.Path('verif').rglob('*.coma'))
assert paths
pathlib.Path('generated/proof-targets.json').write_text(json.dumps(dict(included=paths,excluded={},args=sys.argv[1:],scope='Deliberate generic pointer diagnostic; no actual Bytes lifecycle admission',diagnostic=True),indent=2)+'\n')
PY
export DUNE_DIR_LOCATIONS="why3find:lib:$CREUSOT_DATA_HOME/share/why3find"
export WHY3CONFIG="$CREUSOT_DATA_HOME/creusot_why3.conf"
cargo creusot --only=prove --why3find-arg=-j --why3find-arg=1
