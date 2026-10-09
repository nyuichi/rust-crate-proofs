# AX origin closure audit

**Result: pass for byte-exact origin closure and promotion. This is not outer admission.**

The independently held raw manifest pin is `1f9a614d5c49802dc8d5145f05dbb09f0ef2c72f483f66cae19ba6c037ec9db7` for `bytes-original-raw-suffix-drop-ax-proof-origin`. The promoted [manifest](../manifest.json) has that exact SHA-256. Its capture report SHA-256 is `cde07ce070fd06e28a95203b5afd4e7d0b554fc441d84cd4c014cca778748c32`; the separate [promotion receipt](../PROMOTION_RECEIPT.json) SHA-256 is `dbd375a64a83d4d57d10577591821cae35e02bb04d19d09bd2b5b08173908876`.

I validated the promoted manifest with the captured `evidence_closure.py` (SHA-256 `ef7e19053b8c0db0495b83328a5901ec45578fac92f8eea7f576155bc8356e1c`), supplying the external root SHA, repository root `/workspace/bytes-work`, the promoted `objects` directory, and a fresh scratch validation cache. Validation passed: 12,929 logical files; 422 objects; 455 mounts; 963,108,398 logical expanded bytes. The object transport breakdown is 398 local CAS objects, 18 published-part references, and 6 archive-member references. The 398 promoted CAS files total 12,696,249 bytes and match the preflight CAS tree by filename, size, and SHA-256.

The promoted `manifest.json` and `capture-report.json` are byte-for-byte equal to the preflight files at `/workspace/work/ax-compose-preflight-out-v2`. The promotion receipt records `exact_validated_preflight_bytes_promoted`, says no composer or prover was invoked for promotion, and preserves the preflight proof provenance. The origin proof receipt is `ax/probe/evidence/AX_FULL_PROOF_RUN.json` (SHA-256 `9499ae885e6339eaca9897bce69ea46c3fd133c9ef28357ffc77ca63cf9a54fd`): 177 files, 1,798 prover leaves, 0 null, and 0 structural. It also records `policy_diagnostic: true` and correspondence exit 2.

This result validates the promoted evidence bytes and their closed manifest. It does not promote the proof to admitted status: the recorded correspondence exit is 2, and a separate outer admission manifest/correspondence gate remains necessary. No proof or full correspondence run was repeated for this audit.

The machine-readable audit is [AX_ORIGIN_CLOSURE_AUDIT.json](AX_ORIGIN_CLOSURE_AUDIT.json). The validator summary used here is recorded at `/workspace/work/ax-origin-validation-audit.json` (SHA-256 `624cac9ef0fc0594ce8fc1b5c013b5c38cde0362748aae08060998c17033d60a`).
