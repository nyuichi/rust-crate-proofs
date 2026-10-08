#!/usr/bin/env bash
# Invoke with elevated execution: Why3 requires Unix-domain sockets.
set -euo pipefail
cd "$(dirname "$0")"
source /workspace/bytes-proof-tools/activate.sh
export CARGO_NET_OFFLINE=true
exec 9>"${BYTES_PROOF_LOCK:-/tmp/itoa-creusot-proof.lock}"
flock 9
# Keep the shared tool configuration unchanged while enforcing this gate's budget.
task_config="$PWD/.proof-config"
mkdir -p "$task_config/creusot"
sed 's/^memlimit = .*/memlimit = 1024/' "$XDG_CONFIG_HOME/creusot/why3.conf" > "$task_config/creusot/why3.conf"
export XDG_CONFIG_HOME="$task_config"
printf 'Atomic event: one prover; 1024 MiB; native weak orderings; sc-drf disabled\n'
feature_tree=$(cargo tree --locked -e features --edges normal,build "$@")
if [[ "$feature_tree" == *'creusot-std feature "sc-drf"'* ]]; then
  printf 'sc-drf must be disabled\n' >&2
  exit 2
fi
mkdir -p generated
checker_status=0
python3 check_closed_scope.py > generated/correspondence.json || checker_status=$?
if [[ "$checker_status" != 0 && "${BYTES_SCOPE_DIAGNOSTIC:-0}" != 1 ]]; then exit "$checker_status"; fi
export BYTES_SCOPE_CHECKER_STATUS="$checker_status"
cargo clean --package bytes-scoped-issuance-cursor
rm -rf verif
cargo creusot --only=coma -- --locked "$@"
cargo creusot clean --force
python3 - "$@" <<'PY_TARGETS'
import json,pathlib,os,sys
root=pathlib.Path('.')
paths=sorted(p.as_posix() for p in pathlib.Path('verif').rglob('*.coma'))
assert paths, 'no proof targets'
args=sys.argv[1:];features=[]
for i,a in enumerate(args):
    if a=='--features':features.extend(args[i+1].replace(',', ' ').split())
    elif a.startswith('--features='):features.extend(a.split('=',1)[1].replace(',', ' ').split())
status=int(os.environ['BYTES_SCOPE_CHECKER_STATUS'])
if not features and os.environ.get('BYTES_SCOPE_DIAGNOSTIC')!='1':assert len(paths)==38 and status==0
pathlib.Path('generated/proof-targets.json').write_text(json.dumps(dict(
    included=paths,excluded={},features=features,correspondence_exit_status=status,
    diagnostic=os.environ.get('BYTES_SCOPE_DIAGNOSTIC')=='1',
    scope='Complete fresh protocol client only; generic closed-scope/event TCB, no actual Bytes or unwind admission.'),indent=2)+'\n')
PY_TARGETS
if [[ "${BYTES_TRANSLATE_ONLY:-0}" == 1 ]]; then exit 0; fi
export DUNE_DIR_LOCATIONS="why3find:lib:$CREUSOT_DATA_HOME/share/why3find"
export WHY3CONFIG="$CREUSOT_DATA_HOME/creusot_why3.conf"
cargo creusot --only=prove --why3find-arg=-j --why3find-arg=1
