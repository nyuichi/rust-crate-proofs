# AV evidence closure bootstrap (AW-PACK)

This packaging increment preserves the published AV evidence byte for byte. It adds no API coverage or native proof authority. Full original bytes verification remains **NOT ADMITTED**. No Rust, Cargo build, translation or solver was run.

The externally held raw manifest SHA-256 is `31d82fc5061a6112ee8031aed021c325f114001c93510ec4777d4c6acdf56288`. Supply this independently; computing a digest from an untrusted replacement manifest does not establish authority. The manifest pins the complete file, object, transport and typed provenance graph. AV canonical `a03720f4286d0583cb5dbbeef0b25a311647431daea1b73293abbddea4284489` and diagnostic origin `a28ac41b5d7d447f0324b37ade94c9cbbd22ff3f3d7aa95effeee5764e82cb47` remain separate immutable provenance objects.

The shared validator/hydrator is ../../tools/evidence_closure.py. Published split parts and exact nested archive members supply old objects without recompression or copied large blobs. Provision the referenced repository files before offline use. Both hydration and scratch checkout destinations must be fresh. The validator rejects unsafe paths, duplicate JSON keys, collisions, undeclared objects and cycles; every hydrated regular file has a declared size and hash. Bootstrap-specific inventory/replay code stays outside the generic validator.

Run the complete offline bootstrap from the repository root, choosing unused scratch destinations:

```sh
python3 bytes/1.11.1/verification/evidence-closures/av-admitted-v1/bootstrap_av.py \
  --manifest bytes/1.11.1/verification/evidence-closures/av-admitted-v1/manifest.json \
  --expected-root-sha256 31d82fc5061a6112ee8031aed021c325f114001c93510ec4777d4c6acdf56288 \
  --repository-root /workspace/bytes-work \
  --cas-root bytes/1.11.1/verification/evidence-closures/av-admitted-v1/objects \
  --cache-root /workspace/work/evidence-closure-cache \
  --work-root /workspace/work \
  --hydrate-to /workspace/work/av-closure-fresh-hydration \
  --checkout-to /workspace/work/av-closure-fresh-checkout
```

The captured preflight validates 12,497 logical files and 23 objects, including exactly 2,164 canonical members, 2,128 origin members, all 334 COMA/proof files and six exact ancestor digests. Archived evidence audit and captured-input correspondence checker both return 0. The proof inventory remains 167 targets, 1,683 prover leaves, zero null and structural leaves. Fresh-destination rejection, corrupted-required-object rejection before checker execution and 22 parser controls pass. Independent audit PASS/GO is recorded in [AW_PACK_AUDIT.md](AW_PACK_AUDIT.md) and [AW_PACK_AUDIT.json](AW_PACK_AUDIT.json). See reports/ for exact copied preflight logs and their hash index.

Logical expanded bytes are 940,024,573; unique object bytes are 793,163,181; referenced published transport bytes are 598,032,311; the reported small new CAS-object bytes are 1,519,092. This CAS metric excludes the roughly 9.15 MB manifest, reports, tools and documentation; it is not the total publication size. Root measured the current cache-excluded bundle at about 11.27 MB before final audit reports. These measures describe different layers. Historical archive bytes are retained; future closures can reference the same objects instead of recursively embedding them. External tool payloads and location-bound Cargo snapshots retain their existing audit limitations.
