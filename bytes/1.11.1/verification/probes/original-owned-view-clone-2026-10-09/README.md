# Owned-view Shared Clone closure (AT)

AT appends ordinary proof helpers and a runtime-variable clone/replace client
to the complete AS positive at commit 2e3525dfa642c809c4d8612efd8a2cecb0067500.
The immutable prefix is 9fbe1698c4a1a338c7bc30eeb60af8b69bc4077304b63acab9b3be34a7650744.
All 20 inherited Rust files and native/production/ancestor bodies remain unchanged;
only src/owned_clone_extension.rs is added. The package is renamed, while lib and
build routes are unchanged. The complete positive gate and immutable audit now pass; details follow below.

clone_owned_api accepts a valid owned Shared view at any length, including zero.
It preserves exact pointer, length, content and view bound and the original
allocation/control/lifetime metadata, and issues a fresh real ticket. It retains
acceptance of the borrowed source. The actual stored-vtable call, Relaxed load,
guarded Relaxed increment, AtomicPtr construction and Shared vtable are retained.

The native witness owned_view_clone_scope takes input, a, b, advance_by and rounds.
Its proof preconditions are nonempty input, a<b<=input length and advance_by<=b-a.
After slicing and advancing, it repeatedly clones and replaces value, for any
finite runtime rounds. The loop ledger is precisely the singleton of the current
actual ticket/fraction. The replacement Drop must retire the old owner before
installing next. Complete drain keeps Shared ownership; final Drop recovers and
frees the original allocation even though the byte result is empty.

Fractions are not assumed constant: on_register splits the residual State pool
and leaves its borrowed source unchanged. The replacement ticket and newly
constructed atomic model identity may differ each iteration. Only allocation
metadata and the exact suffix content are fixed. Native live-count bounds must
follow from the loop body, never a quota on rounds.

## Audited outcome

AT proves reusable Shared Clone for valid owned views of any length, including
zero after complete advance. It preserves the borrowed source, exact suffix,
pointer/length and original allocation metadata while issuing a fresh actual
ticket. The native client performs any finite runtime number of Clone and
replacement operations. Each normal assignment Drop retires the previous owner
before installation; the singleton ticket/fraction inventory and final recovery
and free of the original allocation are proved. No positive quota on rounds or
constant fraction/atomic identity is assumed.

Canonical v2 proves all155 targets/1417 prover leaves/zero null and structural
leaves, correspondence0, features[], exclusions{}, diagnosticfalse. Independent
immutable reconstruction checks1544 members, production63, privateStd110, six
configs, eight tool rows and four actual Cargo artifacts, then replays main52/native45 controls
without a live target directory. The one-prover/1024MiB/sc-drf-off budget and
location-bound native Cargo joins remain explicit. Native capture contains145
cases,30 MIR bodies and29 production bodies.

Five semantic controls retain16 independently reprinted failed tasks; the
replacement-Drop control proves only its affected whole-function target and
records154 exclusions. Duplicate reuse is rejected with E0382. Unchanged audited
Acquire/free/final-Drop mechanism controls are hash-linked rather than rerun.
These are proof/type/correspondence sensitivity observations, not native
counterexamples or proofs of generic boundary adequacy.

The two initial singleton-map failures are preserved. A body-proved ordinary
insert/remove algebra lemma uses existing Std ext_eq and adds no trust. The only
new generic trusted function is the ghost-only exact callback registration
instance. Inherited physical, compiler, erased-scope and weak-memory assumptions
remain. Three control-fixture errors and canonical-v1's stale launcher-text
check are preserved tool/packaging diagnostics; repaired metadata leaves all
155 COMA/proof hashes unchanged.

Canonical SHA256958ed73642e1b305b12c6f9269f38de0ee4559e014d83ca58616ff37bc33bc8d.
See verification/probes/original-owned-view-clone-2026-10-09/evidence/AT_CANONICAL_AUDIT.md.
Full original architecture remains NOT ADMITTED. Modular returned owners,
Root/general Static/custom-owner methods, arbitrary escaping/concurrent owners,
unwind, BytesMut and remaining APIs/configurations remain open. Ask Astra
「次何するのがいい？」after audited publication and execute the recommendation.
