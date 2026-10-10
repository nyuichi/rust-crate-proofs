set -o pipefail
bash verification/probes/backend-sse-prefix/why3-driver/run-overlay-negative-10s.sh expected_invalid_u32_negative_index_wrap 2>&1 | tee verification/probes/backend-sse-prefix/why3-driver/evidence/2026-10-05-bounded-closure-smoke/12-negative-u32-negative.log
