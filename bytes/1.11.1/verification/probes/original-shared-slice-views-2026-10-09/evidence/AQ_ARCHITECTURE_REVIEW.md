# AQ architecture and checker review

Status: bounded source/interface and adversarial component review. The parent
reports positive diagnostic 139 targets / 1202 goals / 0 null / 0 structural,
archive d936353824969733639d56f2b1ad75643e8198e07d27a19eaeea3e1a5ecdc6f8.
This note is not a canonical archive audit or full original-crate admission.

## Ownership and native interpretation

The transformed prefix is the complete published AP positive source, with only
the explicit OriginalSharedProof enum extension. Root/Child payloads and all
other prefix bytes remain unchanged; every inherited target was retained.
View carries a real ChildProof and resource-free bounded pointer metadata.
SharedCore retains original allocation base/capacity/content, physical borrow,
control borrow, scoped invariant and actual lifetime ticket. View offsets do not
replace allocation metadata. Clone uses the actual Relaxed load/increment and
body-proved registration. View Drop calls the existing actual shared release
path with its original recovery and both deallocation receipts.

Empty has unbound pointer metadata and a readonly null atomic-field binding;
it has no ticket, PhysicalRegion, Recovery or allocation receipt. Native
new_empty_with_ptr removes provenance using null.wrapping_add(address). Its
zero-length read needs no allocation liveness, and actual static_drop has no
owner effect. The client proves the original allocation is recovered exactly
once either by owner Drop (selected Empty) or selected View Drop (nonempty).

Generic pointer add/wrapping/without-provenance metadata, exact null-word
reification, callback erasure, scoped event completeness and normal terminal
MIR interpretation remain explicit assumed TCB. Their adequacy is not derived
from the Bytes caller proof. No Bytes-specific ownership, content, lastness,
refcount or destructor law was made trusted. No resource-returning logic getter,
Ghost-to-runtime pointer, readonly root reseal or fixed-ID assumption was added.
The concrete Range<usize> normal valid-range theorem includes empty endpoints;
arbitrary RangeBounds, panic/unwind, concurrency and the rest of the API remain
open. Full original architecture remains NOT ADMITTED.

## Exact failed interface and repair

First semantic diagnostic 139/1261/3 had only slice_view nulls. Printed tasks
showed shifted as an uninterpreted cross-module predicate, preventing offset
and namespace projection; current_address/raw-pointer relations were similarly
opaque. The generic interface now exposes the pure shifted definition and an
exact result-address/current-address post. A separate Resolve task was on an
impossible Child result branch: clone_shared_view only promised owned Child-or-
View despite its body always constructing View. Its stronger body-proved View
post eliminates that branch. No Resolve axiom or weakened contract was used.
Earlier frontend and reserved-keyword backend failures were separately archived.
Native local begin is explicitly alpha-renamed view_begin in the shadow.

## Independent adversarial review

Reviewed the main checker's single enum transformation, exact full extension /
client / pointer hashes, inherited imported-module closure, pre-import ancestor
pins, exact Cargo/lock/build/extractor routing, four-artifact capture, and native
checker source / 25 MIR / full production inventory joins. Main proof source
pins are independent of refreshed generator mapping hashes. Native guards pin
real Range branches, add versus wrapping_add, empty provenance removal, static
callback, and original / first / owner / selected normal Drop places/order.

Independent component tests rejected:
- proof_assert macro shadowing in the whole client;
- false postcondition in the trusted generic pointer boundary;
- qualified trusted annotation on body-proved slice_view;
- appended resource-returning logical getter.

A concrete archive-only capture gap was reproduced: after a baseline using the
actual four Cargo artifacts passed, changing actual_out_dir, compiled input,
root-output path and captured root-output bytes to /tmp/unrelated-package/out,
with fresh corresponding hashes and an otherwise unchanged real fingerprint /
build-output route, was accepted. Live assert_compiled_records constructed the
correct route, but assert_compiled_capture lacked the cross-artifact join.

The checker owner added actual_out_dir == build_output.parent/out and the exact
public_records.rs filename requirement, plus a fresh-hash control. The same
independent fixture now rejects while its real-artifact baseline still passes.
Reproducer: /workspace/work/aq-review-probe.py; all artifact mutations were under
/workspace/work/aq-review-capture. No positive source, generated proof input,
Cargo invocation or prover was touched by these adversarial tests.

Final canonical positive replay and immutable archive audit are still the
parent's publication gates. Component checks here do not replace those gates.

Additional native-only independent replay passed the actual full native bundle
baseline. Two in-memory MIR mutations with refreshed selected-capture hashes
were rejected at semantic structure checks: reversing slice bb22's empty/nonempty
edges (`Bytes::slice bb22 control edge changed`), and replacing first Drop's
place _11 with owner _7 at client bb5 (`nested_slice_scope bb5 operations/edge
changed`). Reproducer: /workspace/work/aq-native-review.py. This native-only
baseline does not claim the concurrently selected negative proof source passes
the main correspondence gate.

## Final checker replay before canonical proof

The positive main CLI passes and the structural suite rejects 76/76 mutations.
Two AQ-local integration issues were corrected: compiled-record reconstruction
initially invoked AL's stale extractor pin rather than the reviewed AQ extractor;
two control fixtures initially targeted the source-mapping predicate instead of
the predicate that owns native edge/readiness validation. Ancestors were not
changed. The full suite now runs a full positive audit first, captures the four
actual artifacts, and exercises the same owning predicates used by main audit.
Neither issue is presented as a failed Bytes semantic VC.
