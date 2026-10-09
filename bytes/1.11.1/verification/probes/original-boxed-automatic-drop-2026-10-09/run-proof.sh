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
feature_tree=$(cargo tree --locked -e features --edges normal,build --features public_constructor "$@")
if [[ "$feature_tree" == *'creusot-std feature "sc-drf"'* ]]; then
  printf 'sc-drf must be disabled\n' >&2
  exit 2
fi
mkdir -p generated
if [[ -f elaborate.py ]]; then python3 elaborate.py --feature "${BYTES_DROP_FEATURE:-}"; fi
checker_status=0
if [[ "${BYTES_TRANSLATE_ONLY:-0}" != 1 ]]; then
  if [[ "${BYTES_DROP_CHECKER_SKIP:-0}" == 1 && "${BYTES_SCOPE_DIAGNOSTIC:-0}" == 1 ]]; then
    printf '{"status":"not_run","reason":"deliberate semantic development control; final structural controls are separate"}\n' > generated/correspondence.json
    checker_status=2
  elif [[ -f check_correspondence.py ]]; then
    python3 check_correspondence.py --shadow generated/boxed_client.rs --mapping generated/mapping.json > generated/correspondence.json || checker_status=$?
  else
    printf '{"status":"coverage_failure","reason":"independent checker not ready"}\n' > generated/correspondence.json
    checker_status=2
  fi
fi
if [[ "$checker_status" != 0 && "${BYTES_SCOPE_DIAGNOSTIC:-0}" != 1 ]]; then exit "$checker_status"; fi
export BYTES_DROP_FEATURE="${BYTES_DROP_FEATURE:-}"
export BYTES_SCOPE_CHECKER_STATUS="$checker_status"
cargo clean --package bytes-original-boxed-automatic-drop
rm -rf verif
cargo creusot --only=coma -- --locked --features public_constructor "$@"
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
if not features and os.environ.get('BYTES_SCOPE_DIAGNOSTIC')!='1':assert status==0
pathlib.Path('generated/proof-targets.json').write_text(json.dumps(dict(
    included=paths,excluded={},features=features,correspondence_exit_status=status,
    diagnostic=os.environ.get('BYTES_SCOPE_DIAGNOSTIC')=='1',
    terminal_feature=os.environ.get('BYTES_DROP_FEATURE',''),
    scope='Unrestricted boxed read and unpromoted normal automatic Drop under explicit generic pointer/free/terminal/native/erasure TCB; no promotion/unwind/fullcrate admission.'),indent=2)+'\n')
PY_TARGETS
if [[ "${BYTES_TRANSLATE_ONLY:-0}" == 1 ]]; then exit 0; fi
export DUNE_DIR_LOCATIONS="why3find:lib:$CREUSOT_DATA_HOME/share/why3find"
export WHY3CONFIG="$CREUSOT_DATA_HOME/creusot_why3.conf"
cargo creusot --only=prove --why3find-arg=-j --why3find-arg=1
