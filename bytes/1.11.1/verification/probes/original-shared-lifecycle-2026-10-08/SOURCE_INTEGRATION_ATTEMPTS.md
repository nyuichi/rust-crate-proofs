# Source integration authoring record — 2026-10-08

These are failed or interrupted private source-correspondent experiments, not
verification coverage for the original public API. Preserve their snapshots.
Only a later complete matching-source proof can supersede their result.

| Archive | Status | Observed proof output | Meaning |
|---|---|---|---|
| consuming-clone-authoring-2026-10-08.tar.gz | interrupted/rejected draft | 40 Coma,43 JSON,17 nulls; no complete dependency manifest | Consuming `(self)->(Self,Self)` is not original borrowed-source Clone; never adopt as a replacement API. |
| integrated-source-leaf-2026-10-08.tar.gz | completed failed proof | 40 Coma,43 JSON,18 nulls across5 functions | Borrowed-source clone, real field events and source abort guard are present; strong source correspondence/cleanup contracts still fail. |

The second snapshot retains3 dangling older JSON files explicitly. Its complete
failed-run counts include those files; final acceptance must regenerate a clean
matching Coma/JSON tree. Independent archive checks are in
`evidence/root-authoring-audit.json`. Its36 recorded input hashes match; the
archive SHA also freezes the exact outputs and logs. Native2-test success and
the E0505 live-slice rejection are separate evidence, not semantic proof success.

Concrete interface repairs identified from this run:

- The trait field projection needed an explicit implementation postcondition;
  after adding it, its refinement body passes. No trusted bytes projection law.
- Borrowed byte contents retain exact sequence equality. The range contract now
  states `len <= capacity`; the remaining extensional equality must be proved.
- Recovery metadata containing only a numeric control address cannot identify
  the exact pointer/provenance of its typed permission or native buffer. Carry
  exact control/base pointer observations; do not add address-to-pointer identity.
- A release receipt authenticates its result, but does not expose that another
  actual ticket is still registered. The stronger peer observation must use
  live fragments and the authoritative map inside the same Release callback,
  delegating the existing transition without an additional native event.
- Cross-crate callers must receive the initializer atomic/public-field relation
  explicitly in its body-proved contract rather than rely on an opaque definition.

The native/model event, scoped actual-token control access, B1/B3/B4 physical
access and typed deallocation boundaries remain explicit generic TCB. Bytes
registration, count-to-ticket, lastness and cleanup laws must remain body proved.
No public vtable dispatch, arbitrary Clone/overflow, real-thread transport or
automatic Drop conclusion follows from these bounded sequential leaf experiments.
