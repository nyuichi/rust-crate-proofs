set -o pipefail
bash verification/probes/backend-sse-prefix/why3-driver/run-overlay-proof.sh uri_allowed_mask_16_sse 2>&1 | tee verification/probes/backend-sse-prefix/why3-driver/evidence/2026-10-05-bounded-closure-smoke/08-uri-allowed-mask.log
