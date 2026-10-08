#!/usr/bin/env bash
# New external effect lowering; never stock automatic-Drop completion.
set -euo pipefail
cd "$(dirname "$0")"
source /workspace/bytes-proof-tools/activate.sh
export CARGO_NET_OFFLINE=true
feature="${1:-}"
diagnostic="${2:-}"
if [[ -n "$diagnostic" && "$diagnostic" != --diagnostic ]]; then exit 2; fi
if [[ -n "$feature" && "$diagnostic" != --diagnostic ]]; then
    echo 'Negative features require --diagnostic; never positive evidence.' >&2
    exit 2
fi
exec 9>/tmp/itoa-creusot-proof.lock
flock 9
bash generate-native-mir.sh
cargo_args=()
checker_args=()
if [[ -n "$feature" ]]; then
    python3 elaborate.py --features "$feature"
    selected="generated/controls/$feature"
    cargo_args=(--features "$feature")
    checker_args=(--features "$feature")
else
    selected=generated
fi
cp "$selected/shadow.rs" generated/active.rs
set +e
python3 check_correspondence.py --shadow "$selected/shadow.rs" --mapping "$selected/mapping.json" --report generated/correspondence-result.json "${checker_args[@]}"
correspondence_status=$?
set -e
if [[ -z "$diagnostic" && "$correspondence_status" != 0 ]]; then exit "$correspondence_status"; fi
export DROP_GATE_FEATURE="$feature"
export DROP_GATE_MAPPING="$selected/mapping.json"
export DROP_GATE_SHADOW="$selected/shadow.rs"
export DROP_GATE_CORRESPONDENCE_STATUS="$correspondence_status"
feature_tree=$(cargo tree --locked -e features --edges normal,build "${cargo_args[@]}")
if [[ "$feature_tree" == *'creusot-std feature "sc-drf"'* ]]; then exit 2; fi
cargo clean --package bytes-generic-drop-elaboration
cargo creusot --only=coma -- --locked "${cargo_args[@]}"
cargo creusot clean --force
export DUNE_DIR_LOCATIONS="why3find:lib:$CREUSOT_DATA_HOME/share/why3find"
export WHY3CONFIG="$CREUSOT_DATA_HOME/creusot_why3.conf"
python3 - <<'PY'
import json,os,pathlib,subprocess
root=pathlib.Path('verif/bytes_generic_drop_elaboration_rlib')
paths=sorted(str(p) for directory in ['shadow','loan_frame'] for p in (root/directory).rglob('*.coma'))
expected={
 'loan_frame/caller_can_mutate_after_effect.coma',
 'shadow/set_true.coma','shadow/set_true_drop_effect.coma','shadow/set_true_scope.coma',
 'shadow/toggle_after_write.coma','shadow/toggle_drop_effect.coma','shadow/toggle_scope.coma'}
assert {str(pathlib.Path(p).relative_to(root)) for p in paths}==expected, paths
policy=dict(included=paths,excluded={},features=os.environ['DROP_GATE_FEATURE'],
    correspondence_exit_status=int(os.environ['DROP_GATE_CORRESPONDENCE_STATUS']),
    shadow=os.environ['DROP_GATE_SHADOW'],mapping=os.environ['DROP_GATE_MAPPING'],
    scope='Only supported generic normal-return Drop witnesses, body-checked helpers and a legal post-effect mutation control; no Bytes Drop/closed-history/whole-crate claim.',
    not_claimed=['unwind completion','arbitrary Drop glue','Bytes automatic Drop','complete Clone issuance','whole crate'])
pathlib.Path('generated/proof-targets.json').write_text(json.dumps(policy,indent=2)+'\n')
raise SystemExit(subprocess.call(['why3find','prove','--no-autodetect-provers','-j','1',*paths]))
PY
