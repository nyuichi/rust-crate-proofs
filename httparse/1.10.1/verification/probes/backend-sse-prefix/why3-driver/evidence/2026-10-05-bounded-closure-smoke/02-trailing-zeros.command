set -o pipefail
bash verification/probes/backend-sse-prefix/why3-driver/run-overlay-proof.sh trailing_zeros_shift_to_nth 2>&1 | tee verification/probes/backend-sse-prefix/why3-driver/evidence/2026-10-05-bounded-closure-smoke/02-trailing-zeros.log
