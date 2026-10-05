set -o pipefail
bash verification/probes/backend-sse-prefix/why3-driver/run-overlay-proof.sh prefix_len_from_mask 2>&1 | tee verification/probes/backend-sse-prefix/why3-driver/evidence/2026-10-05-bounded-closure-smoke/03-prefix-length.log
