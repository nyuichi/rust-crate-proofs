set -o pipefail
bash verification/probes/backend-sse-prefix/why3-driver/run-overlay-proof.sh match_uri_char_16_sse_pure 2>&1 | tee verification/probes/backend-sse-prefix/why3-driver/evidence/2026-10-05-bounded-closure-smoke/09-match-uri.log
