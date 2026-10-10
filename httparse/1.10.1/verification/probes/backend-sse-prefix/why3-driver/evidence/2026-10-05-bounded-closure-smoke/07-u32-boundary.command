set -o pipefail
bash verification/probes/backend-sse-prefix/why3-driver/run-overlay-proof.sh u32_nth_boundary_smoke 2>&1 | tee verification/probes/backend-sse-prefix/why3-driver/evidence/2026-10-05-bounded-closure-smoke/07-u32-boundary.log
