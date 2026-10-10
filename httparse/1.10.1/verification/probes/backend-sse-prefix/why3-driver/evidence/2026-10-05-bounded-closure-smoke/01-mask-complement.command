set -o pipefail
bash verification/probes/backend-sse-prefix/why3-driver/run-overlay-proof.sh mask_complement_preserves_low_bits 2>&1 | tee verification/probes/backend-sse-prefix/why3-driver/evidence/2026-10-05-bounded-closure-smoke/01-mask-complement.log
