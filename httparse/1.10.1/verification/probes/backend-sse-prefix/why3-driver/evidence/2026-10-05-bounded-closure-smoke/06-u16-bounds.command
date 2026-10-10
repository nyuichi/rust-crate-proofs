set -o pipefail
bash verification/probes/backend-sse-prefix/why3-driver/run-overlay-proof.sh u16_nth_out_of_bounds_smoke 2>&1 | tee verification/probes/backend-sse-prefix/why3-driver/evidence/2026-10-05-bounded-closure-smoke/06-u16-bounds.log
