# AP: runtime-variable finite owners in actual Vec loops

Development after audited AO commit 48cf2a0e365dd00b20bf88b93bec8d75a7dde5ca.
The original bytes 1.11.1 full architecture remains **NOT ADMITTED**.

The caller supplies a runtime count. After the original Box-backed owner
retires, the surviving child uses actual Shared-vtable Clone in a Vec push
loop. A pop loop lexically retires each peer; the survivor reads, saves its
return and finally retires. The Vec destructor executes with zero elements;
its own container storage remains distinct from Bytes payload/control receipts.

The new proof premise is an inductive exact inventory over the actual Vec and
affine cursor, not fixed-count unrolling or a quota. The real guarded refcount
abort remains native. Only normal completion is claimed; success for every
count, allocation failure, unwind, concurrency and arbitrary escaping owners
remain open. Source/MIR loop and destructor correspondence remains an explicit
generic compiler/erasure boundary pending replacement by verified lowering.

The restored feature-free gate passes 130 targets / 1040 prover leaves /
zero null or structural leaves, with correspondence status 0 and no features,
exclusions, diagnostic flags or source controls. The final main suite performs
a full positive correspondence audit, records all four actual compiled Cargo
inputs, then rejects 35/35 mutations; native controls reject 76/76.
Independent immutable archive reconstruction passed; see evidence/AP_CANONICAL_AUDIT.md.

The first run left one creation-loop inventory goal unproved (129/1075/1).
A nontrusted, Snapshot-only push lemma preserves the identical complete
inventory and source sequence. Its body returns unit and extracts no owner
resource. The next run proves all 130 targets without a quota or unrolling.

Nine semantic controls leave actual nulls 2/1/1/1/2/2/2/2/2: omitted
registration, peer effect, whole drain, forgotten peer, nonempty Vec cleanup,
root recovery publication, final Acquire and two frees. Paired retained
Shared/promotion paths are one defect per feature, not extra coverage. Free
controls use conjured receipts; their exact failure tasks are proof sensitivity
evidence, not direct native event-absence or deallocation theorems.
Four frontend controls reject duplicate peer/survivor with E0382, retirement
across a live read with E0502/E0505, and Snapshot-to-Ghost owner extraction
with E0277 because the owner type does not satisfy Plain. Captures exclude
proof outputs and all solver caches. See evidence for exact archived tasks.

The initial structural-control receipt label positive_baseline represented
pinned input readiness only. Immutable diagnostic captures are retained;
the final control driver performs the full audit before recording that field.
None of the earlier diagnostic correspondence-not_run receipts is admission. Earlier source/proof evidence is unchanged.
After an audited published increment, ask Astra 「次何するのがいい？」 and
execute the next recommendation until full original verification is complete.

Canonical archive SHA256 a62cfcc22afa2756c56fea1230c3d6845ec3bdf66eaa20bf1779d26da12761d1, 1065 members.
Independent replay preserves all four actual compiled inputs. Cargo root-output
contains the original external OUT_DIR; replay is location-bound, not a claim
of portable self-contained tooling. No archived artifact was rewritten.
