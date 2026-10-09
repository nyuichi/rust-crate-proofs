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
if [[ -f elaborate.py ]]; then python3 elaborate.py --feature "${BYTES_DROP_FEATURE:-}"; fi
export BYTES_DROP_FEATURE="${BYTES_DROP_FEATURE:-}"
# Explicit isolated copied-source control, prepared after frozen-source generation.
source_control="${BYTES_SCOPE_SOURCE_CONTROL:-}"
if [[ -n "$source_control" ]]; then
  if [[ "$source_control" != omit_root_recovery_publication || "${BYTES_SCOPE_DIAGNOSTIC:-0}" != 1 || "${BYTES_DROP_CHECKER_SKIP:-0}" != 1 ]]; then
    echo 'copied-source defect requires explicit diagnostic/checker-skip mode' >&2
    exit 2
  fi
  python3 - <<'PY_CONTROL'
from pathlib import Path
p=Path('src/lifecycle.rs')
s=p.read_text()
anchor='if id == Int::new(0).into_inner() { state.recovery = Some(sealed); }'
assert s.count(anchor)==1,'source-control anchor must be unique'
p.write_text(s.replace(anchor,'if id == Int::new(0).into_inner() { let _=sealed; }',1))
PY_CONTROL
fi
cargo clean --package bytes-original-raw-suffix-drop
rm -rf verif
cargo creusot --only=coma -- --locked "$@"
cargo creusot clean --force
checker_status=0
if [[ "${BYTES_TRANSLATE_ONLY:-0}" != 1 ]]; then
  if [[ "${BYTES_DROP_CHECKER_SKIP:-0}" == 1 && "${BYTES_SCOPE_DIAGNOSTIC:-0}" == 1 ]]; then
    printf '{"status":"not_run","reason":"deliberate semantic development control; final structural controls are separate"}\n' > generated/correspondence.json
    checker_status=2
  elif [[ -f check_correspondence.py ]]; then
    python3 check_correspondence.py --capture-compiled-inputs --shadow src/promotion.rs --mapping generated/mapping.json > generated/compiled-capture-summary.json || checker_status=$?
    if [[ "$checker_status" == 0 ]]; then
      python3 check_correspondence.py --shadow src/promotion.rs --mapping generated/mapping.json > generated/correspondence.json || checker_status=$?
    fi
  else
    printf '{"status":"coverage_failure","reason":"independent checker not ready"}\n' > generated/correspondence.json
    checker_status=2
  fi
fi
if [[ "$checker_status" != 0 && "${BYTES_SCOPE_DIAGNOSTIC:-0}" != 1 ]]; then exit "$checker_status"; fi
export BYTES_SCOPE_CHECKER_STATUS="$checker_status"
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
if not features and os.environ.get('BYTES_SCOPE_DIAGNOSTIC')!='1':assert status==0 and not os.environ.get('BYTES_SCOPE_SOURCE_CONTROL','')
selected=paths;excluded={}
if os.environ.get('BYTES_SCOPE_NEW_TARGETS_ONLY')=='1':
    assert os.environ.get('BYTES_SCOPE_DIAGNOSTIC')=='1' and os.environ.get('BYTES_DROP_CHECKER_SKIP')=='1'
    import hashlib
    inherited_bytes=pathlib.Path('inherited-targets.json').read_bytes()
    inherited=json.loads(inherited_bytes)
    assert inherited['schema']=='ax-inherited-targets-v1'
    assert inherited['ancestor']=='original-promotable-suffix-promotion-2026-10-09'
    assert inherited['ancestor_archive_sha256']=='a03720f4286d0583cb5dbbeef0b25a311647431daea1b73293abbddea4284489'
    assert inherited['source_prefix_sha256']=='67b6643e09f88fd4129db990418207aeecf9c81a90d51113b6b8759f513f2e5e'
    old=set(inherited['relative_targets']);assert len(old)==inherited['target_count']==167
    current={'/'.join(pathlib.Path(p).parts[2:]):p for p in paths}
    assert old<=set(current),'an inherited target disappeared during development translation'
    selected=[p for p in paths if '/'.join(pathlib.Path(p).parts[2:]) not in old]
    assert selected,'no new development targets'
    excluded={current[p]:'unchanged inherited AV target; development-only exclusion' for p in sorted(old)}
pathlib.Path('generated/proof-targets.json').write_text(json.dumps(dict(
    included=selected,excluded=excluded,translated=paths,features=features,correspondence_exit_status=status,
    diagnostic=os.environ.get('BYTES_SCOPE_DIAGNOSTIC')=='1',
    terminal_feature=os.environ.get('BYTES_DROP_FEATURE',''),
    source_control=os.environ.get('BYTES_SCOPE_SOURCE_CONTROL',''),
    scope='Nonempty owned Box<[u8]> converted to the raw Bytes representation, advanced to a suffix, copied through chunk/to_vec, then ordinarily dropped and physically freed. Includes amount==len with a positive original allocation. No Clone/promotion occurs on this path. Empty Box, promotion, concurrent CAS loser, general representations, unwind and the full original crate remain unadmitted.'),indent=2)+'\n')
PY_TARGETS
if [[ "${BYTES_TRANSLATE_ONLY:-0}" == 1 ]]; then exit 0; fi
export DUNE_DIR_LOCATIONS="why3find:lib:$CREUSOT_DATA_HOME/share/why3find"
export WHY3CONFIG="$CREUSOT_DATA_HOME/creusot_why3.conf"
if [[ "${BYTES_SCOPE_NEW_TARGETS_ONLY:-0}" == 1 ]]; then
  mapfile -t selected_targets < <(python3 -c 'import json; print("\n".join(json.load(open("generated/proof-targets.json"))["included"]))')
  cargo creusot --only=prove "${selected_targets[@]}" --why3find-arg=-j --why3find-arg=1
else
  cargo creusot --only=prove --why3find-arg=-j --why3find-arg=1
fi
