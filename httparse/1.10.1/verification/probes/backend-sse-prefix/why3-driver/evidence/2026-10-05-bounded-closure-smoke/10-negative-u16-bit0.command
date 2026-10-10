set -o pipefail
bash verification/probes/backend-sse-prefix/why3-driver/run-overlay-negative-10s.sh expected_invalid_u16_wrong_high_bit 2>&1 | tee verification/probes/backend-sse-prefix/why3-driver/evidence/2026-10-05-bounded-closure-smoke/10-negative-u16-bit0.log
