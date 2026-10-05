#!/usr/bin/env bash
set -euo pipefail
source /workspace/proof-tools/activate.sh
export DUNE_DIR_LOCATIONS='why3find:lib:/workspace/proof-tools/creusot-data/share/why3find'
export CARGO_NET_OFFLINE=true
export CARGO_TARGET_DIR=/tmp/http-map-do-insert-phase-two-proof-target
printf '%s\n' "$(date -u +%Y-%m-%dT%H:%M:%SZ)" > evidence/run-2026-10-05-map-do-insert-phase-two/attempt-1-bounded-loop/direct-proof-started-at.txt
set +e
../../scripts/run-proof.sh /workspace/proof-tools/creusot-data/bin/why3find prove --root . --no-cache --show-progress always evidence/run-2026-10-05-map-do-insert-phase-two/attempt-1-bounded-loop/comas/header/map/do_insert_phase_two.coma > evidence/run-2026-10-05-map-do-insert-phase-two/attempt-1-bounded-loop/direct-proof.log 2>&1
proof_status=$?
printf '%s\n' "$proof_status" > evidence/run-2026-10-05-map-do-insert-phase-two/attempt-1-bounded-loop/direct-proof.exit-status
printf '%s\n' "$(date -u +%Y-%m-%dT%H:%M:%SZ)" > evidence/run-2026-10-05-map-do-insert-phase-two/attempt-1-bounded-loop/direct-proof-completed-at.txt
cat evidence/run-2026-10-05-map-do-insert-phase-two/attempt-1-bounded-loop/direct-proof.log
exit "$proof_status"
