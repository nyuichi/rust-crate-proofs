# Promotable Root suffix advance and first Clone (AV)

AV appends two modules and ten ordinary proof bodies to the complete published
AU positive (commit 60d70b215fd29150adf149a9ac793eb7f52b7a1d). The frozen prefix
is f3627094e2121b4b52d5d2c881adc6cd23d1f80ac8c33cdf0e21a0ed56ecc1e9;
21 other AU source modules remain unchanged, and the exact lib module routes
are declared. The original enum and allocation descriptor are preserved.
D-AV records the changed suffix premise and staged proof/reuse budget before
experiments.

## Promotable Root suffix advance and first Clone (AV) — 2026-10-09

AV proves Root advance before first Clone while preserving the original allocation base, capacity and complete contents. A separate BoundPtr view tracks offset+len==original capacity, including a fully drained owned-zero view. First promotion body-proves the actual offset_from(original base)+suffix len capacity reconstruction; its full Payload retains original capacity/content while the child exposes the exact suffix. Ghost callback selection uses immutable original base/table parity, followed by one actual native vtable call. Advancing by one can change the current pointer parity without changing that vtable.

The selected ten new bodies pass 236 prover leaves before the single complete proof passes all 167 targets/1683 prover leaves/zero null and structural leaves, no exclusions/features. The immutable full origin is a28ac41b5d7d447f0324b37ade94c9cbbd22ff3f3d7aa95effeee5764e82cb47. Admission will explicitly reuse its 334 COMA/proof files after unchanged critical-input identity checks; no second whole solver run is claimed. Fresh actual Cargo four-artifact capture passes. Native capture passes 292 cases and 30 selected MIR bodies, including 29 production bodies; both callback bodies are captured, but no odd allocation parity is claimed observed.

The new generic suffix_pointer distance boundary borrows live positive-capacity same-allocation PhysicalRegion/BoundPtr authority and exports offset_from as the bound view offset. It requires exact stored pointers, namespace/capacity/zero origin, nonnull, no wrap and isize bounds; one-past offsets are included, dangling zero-capacity allocations excluded. It returns no resource and assumes no Bytes capacity/ownership/last-owner law. Caller bodies establish capacity reconstruction and the normal promoted Root Drop, surviving child suffix read and final recovery/free. All existing generic physical, weak-memory, vtable and normal-Drop TCB remains explicit.

One private copied-source cap=len mutation rejects two exact proof leaves in the affected promotion whole-function target: one file, 149 prover leaves, two nulls and zero structural leaves, with all 167 translated and 166 explicit exclusions. Independent reprinting matches both tasks: wellformed_Payload and suffix_clone_result capacity-conservation goals, not native counterexamples. Full correspondence and one refreshed allocation-recovery structural check pass. See evidence/AV_CAPACITY_CONTROL_AUDIT.md and evidence/AV_ORIGIN_AUDIT.md in the AV probe. Only the new capacity statement changes; the inherited equal-zero adapter remains untouched. The primary 334 COMA/proof files and critical inputs stay byte-identical. Old audited controls are reused. Canonical admitted_reuse and independent immutable archive-only audit PASS: SHA256 a03720f4286d0583cb5dbbeef0b25a311647431daea1b73293abbddea4284489, 2,164 members, all 167/1683/0/0 targets and 334 origin/canonical proof files identical. All 550 unique critical-input path/hash rows match. Fresh correspondence exits 0; the original proof policy remains diagnostic with correspondence exit 2 and an archived `Proved (167 files)` marker, without an archived numeric prover exit or second solver run. Exact nested AU/AT/AS/AR/AQ/AP reconstruction and captured-input checker replay pass without Cargo or a prover. The four Cargo artifacts remain location-bound snapshots; all eight external executable hashes match the manifest, while their payloads are not bundled. See verification/probes/original-promotable-suffix-promotion-2026-10-09/evidence/AV_CANONICAL_AUDIT.md. Frontend syntax/mode diagnostics and inode exhaustion are preserved separately; no semantic positive-proof failure occurred. Full original architecture remains NOT ADMITTED. Raw suffix Drop without promotion, concurrent promotion losers, arbitrary escaping/concurrent owners, other representation paths, unwind, BytesMut and remaining APIs/configurations remain open.

## Reconstruct published archives before audit

Both archives are distributed in exact parts listed by
`evidence/PUBLISHED_ARCHIVES.json`: the full proof origin has five parts and
the canonical archive has ten. From this probe directory, reconstruct and
verify every listed archive before extraction or checker hydration:

```python
from pathlib import Path
import hashlib, json

index = json.loads(Path("evidence/PUBLISHED_ARCHIVES.json").read_text())
for item in index["archives"]:
    manifest_path = Path(item["parts_manifest"])
    manifest = json.loads(manifest_path.read_text())
    output = Path(item["archive"])
    assert output.name == manifest["archive"]
    complete = hashlib.sha256()
    total = 0
    with output.open("wb") as out:
        for part in manifest["parts"]:
            data = (manifest_path.parent / part["file"]).read_bytes()
            assert len(data) == part["bytes"]
            assert hashlib.sha256(data).hexdigest() == part["sha256"]
            out.write(data)
            complete.update(data)
            total += len(data)
    assert total == manifest["bytes"] == item["bytes"]
    assert complete.hexdigest() == manifest["sha256"] == item["sha256"]
```

The assembled archives are ignored local files; reconstruction does not change
the audited bytes. Extract the canonical archive into scratch, restore its
`inputs/repository/bytes/1.11.1` tree and place `probe/` at this AV sibling path.
Hydrate AU, AT, AS, AR, AQ and AP from the nested captured canonical archives,
preserving separately captured documentation/evidence. Restore AU's
`inputs/au-proof-origin/au-full-diagnostic-v1.{log,json,tar.gz}` into AU's
`evidence/`; the AV origin is under outer `inputs/av-proof-origin/`. Then run
`python3 check_correspondence.py --audit-compiled-capture-only` from restored AV.
This invokes neither Cargo nor a prover and uses no live target directory.
See `evidence/AV_CANONICAL_AUDIT.md` for exact hydration details and limits.
