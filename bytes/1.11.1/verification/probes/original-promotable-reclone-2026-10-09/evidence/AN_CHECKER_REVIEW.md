# AN checker review — closed out, 2026-10-09

## Final closeout

The final full `audit()` baseline passed. The previously accepted contradictory
`pointer_event::bind_read_only` postcondition now rejects with
`directly imported proof support source changed: pointer_event`. The actual AN
routes resolve ten independently pinned imported source modules, covering the
external body-proved helpers as well as generic trusted support boundaries.

A full-audit client macro mutation was also replayed: the client was prefixed
with the `proof_assert!` override, the active source was recomposed, and both
client/active mapping hashes were refreshed. It rejects on the complete client
token surface, rather than a stale receipt. Extension source, exact registration
contracts, immutable core/helper segments and crate-root bindings remain closed.

The reviewed AM checker SHA is checked before its import. The AN native checker
SHA is checked before its import and invocation. The native production input
inventory remains independently anchored, including global bindings and include
routes. No remaining blocker was found within this bounded correspondence gate.

All four archived compiled inputs were independently read and checked against
their receipt hashes and current compiler-output bytes: the public record,
Cargo run-build fingerprint, build-output directives and root-output path.
Their paths/hashes and the build/extractor identities are recorded in
`generated/compiled-inputs/public-records-build-receipt.json`. This closes the
previous live-cache-only evidence omission.

`TCB.md` accurately distinguishes the two new generic registration instances
from body-proved `load_visible_snapshot` and child-cursor acceptance. The
captured private Std Acquire `Committer::shoot_load` contract was inspected:
it requires the matching permission, advances the current view, and relates
the observed value/published view to an actual history entry. The Snapshot
documentation confirms a zero-sized logical value whose construction does not
move ownership or execute its Pearlite expression natively. No Bytes ownership,
last-owner, or destructor law was added to trust by this increment.

Only this review note was edited. The baseline checker refreshed its designated
compiled-input evidence receipts; no Rust source, generated proof source,
build input or prover state was changed. Canonical proof/archive admission
remains the parent's subsequent gate.

## Preserved initial review and resolved findings

This review used read-only in-memory mutations. No Rust, generated artifact,
build input or prover state was changed. Results describe the initial checker
draft; the gaps below were subsequently fixed and retested as recorded above.

## Independently exercised checks

The positive full `audit()` passed. Each of these mutations was rejected by its
actual source-checking component:

- Prepending `macro_rules! proof_assert { ($($tokens:tt)*) => {}; }` to the client.
- Prepending the same macro override to the extension.
- Prepending an import alias to the extension.

The complete client token stream and raw extension SHA close these changes,
including injected items, modified registrations and resource-valued getters.
The frozen AL prefix and AM helpers, exact selected `lib.rs` route and complete
active-file composition prevent appending an unchecked implementation between
the reviewed segments. Both callback registrations' complete pre/postcondition
equivalence contracts are checked separately from the extension digest.

## Confirmed blocker: external proof dependencies

The full initial `audit()` also **accepted** an in-memory change to
`original-shared-lifecycle-2026-10-08/src/pointer_event.rs` adding
`#[ensures(0int == 1int)]` to the existing trusted `bind_read_only` function.
Its baseline full audit passed in the same process. This was not a receipt/hash
mismatch control: the dependent source was simply outside the initial AN gate's
checked source inventory.

The twelve copied-source entries do not cover sibling-probe imports. The
production inventory covers native `src/` files, not proof files under
`verification/probes`. The initial AN checker reused AL's module-route check
but did not rerun its complete reviewed-support-source checks.

Required fix: independently pin and verify the contents of every actual
imported proof source, including `relaxed.rs`, `fraction_map.rs`, `release.rs`,
`pointer_event.rs`, and `promotion_tags.rs`; resolve these from the selected
AN module paths. Include the production-owned proof modules in that accounting
or explicitly connect them to the existing frozen production inventory. The
body-proved external helpers also need protection against newly added trust or
false contracts. Re-run the contradictory external-pointer mutation after the
fix, alongside a hidden-trust mutation in a body-proved external helper.

## Other pending closeout items

The parent independently identified two additional draft omissions: verify the
reviewed AM checker digest before importing it, and archive actual AN compiled
record/fingerprint/build-output/root-output bytes with metadata rather than
retaining only live-cache paths and hashes. These fixes and final mutation
receipts require closeout review.

The three-owner native sequence, returned-ID ledger mapping, saved-return
ordering and full normal terminal place identities were consistent with the
architecture review. Native production imports/includes are now protected by
the independently anchored 63-entry inventory. Generic callback erasure,
compiler/MIR adequacy, physical/synchronization boundaries, and normal-only
terminal-place interpretation remain explicit TCB; this source review does not
prove their adequacy or the full crate.
