set -o pipefail
bash verification/probes/backend-sse-prefix/why3-driver/run-overlay-proof.sh movemask_narrow_preserves_bits 2>&1 | tee verification/probes/backend-sse-prefix/why3-driver/evidence/2026-10-05-bounded-closure-smoke/04-movemask-narrow.log
