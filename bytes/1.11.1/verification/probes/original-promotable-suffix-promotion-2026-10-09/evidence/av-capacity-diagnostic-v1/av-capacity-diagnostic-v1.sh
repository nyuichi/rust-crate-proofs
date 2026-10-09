#!/usr/bin/env bash
set -euo pipefail
cd /workspace/work/av-capacity-diagnostic-v1-snapshot/bytes/1.11.1/verification/probes/original-promotable-suffix-promotion-2026-10-09
source /workspace/bytes-proof-tools/activate.sh
export CARGO_TARGET_DIR=/workspace/work/av-capacity-target
export CARGO_NET_OFFLINE=true
exec 9>/tmp/itoa-creusot-proof.lock
flock 9
mkdir -p .proof-config/creusot
sed 's/^memlimit = .*/memlimit = 1024/' "$XDG_CONFIG_HOME/creusot/why3.conf" > .proof-config/creusot/why3.conf
export XDG_CONFIG_HOME="$PWD/.proof-config"
export BYTES_DROP_FEATURE=wrong_capacity
python3 elaborate.py --feature wrong_capacity
cargo creusot --only=coma -- --locked
cargo creusot clean --force
python3 - <<'PY_POLICY'
import json,pathlib
paths=sorted(p.as_posix() for p in pathlib.Path('verif').rglob('*.coma'))
assert len(paths)==167
chosen=[p for p in paths if p.endswith('/promotion/shallow_clone_suffix_checked.coma')]
assert len(chosen)==1
policy=dict(included=chosen,excluded={p:'outside the single whole-function capacity control' for p in paths if p not in chosen},translated=paths,features=[],diagnostic=True,correspondence_exit_status=2,terminal_feature='wrong_capacity',source_control='',scope='Isolated wrong-capacity control: cap=len instead of pointer distance plus len; all contracts and physical pointer premises retained. No positive admission.')
pathlib.Path('generated/proof-targets.json').write_text(json.dumps(policy,indent=2)+'\n')
PY_POLICY
export DUNE_DIR_LOCATIONS="why3find:lib:$CREUSOT_DATA_HOME/share/why3find"
export WHY3CONFIG="$CREUSOT_DATA_HOME/creusot_why3.conf"
cargo creusot --only=prove verif/bytes_original_promotable_suffix_promotion_rlib/promotion/shallow_clone_suffix_checked.coma --why3find-arg=-j --why3find-arg=1
