# AO architecture and correspondence review

Scope: `original-promotable-surviving-child-2026-10-09`. Read-only Rust/checker
review; no solver runs by this reviewer. Full original bytes 1.11.1 architecture
is **NOT ADMITTED**. The parent reports the first diagnostic gate at 125 targets,
983 actual goals, zero nulls, archived as SHA256
`6ac281984cc2b3392eac2bde26cc813abfef8c395559b46b2186072b2f2fcede`.
This review does not independently certify that archive or the final gate.

## Ownership and native correspondence

Reviewed active source SHA256:
`e607bc1c5587b3d5e0a8260a3891807db3ade7ab331c264a3e49d1de443ca984`.
The inherited prefix is AN's entire positive source, SHA256
`35c37cca2f01ddc1e12e57de6a2ee5959ea42744ddd7bb6ee9f716c85c65e94c`.
The new extension is SHA256
`768dcce2a6d539921360668622c1de8a4d11c3259c7a92365c4b44fb581b3ec0`.

No ownership/interface defect was found in that extension. The root terminal
adapter consumes PromotionScope by value. Its callback consumes the actual
updated AtomicPtr permission through get_mut_finish and the actual root core
through the existing release_core body. It outputs only the actual affine
cursor in `DetachedScope { cursor: Cursor }` and a nonfinal Completion. It
retains no root ticket, root core, physical recovery or pointer permission.
There is no logical getter returning State, Perm, ticket or another resource.

The callback's precondition is general ledger length greater than one. Only
the selected client derives the concrete two-owner state, using actual returned
ticket identities. The existing State::on_release body publishes the original
recovery; the surviving child's final release, Acquire, Pending::recover and
existing free bodies establish recovery and the payload/control receipts.
Neither root handoff nor final recovery is a new Bytes-specific trusted law.
The surviving read borrows physical authority with the child's own ticket.
The final empty ledger follows actual map removal from the prior singleton,
not a definition that declares a finished scope empty.

The native normal-edge correspondence is original `_3`, bb2 to bb3, followed
by survivor read, return evaluation in bb5, and survivor `_2`, bb6 to bb7.
The native original APIs and ABI remain unchanged. New even/odd registration
instances equate the complete callback pre/postconditions through the existing
generic erased-call boundary. Terminal-place consumption, address
nonobservation, absence of independent field drop glue, ghost erasure, native
compiler/MIR correspondence and the inherited generic physical/atomic contracts
remain explicit TCB. Their adequacy is not proved by the caller gate. The
survivor escapes the inner lexical scope only; arbitrary outer escapes,
concurrency, unwind completion, other API/configuration coverage and whole-crate
admission remain open.

## Adversarial checker review

Tests below used in-memory source overrides only. No Rust, native source,
generated proof input or solver configuration was changed by this reviewer.
The parent's temporary missing-publication control was not treated as a
positive baseline.

1. External pointer_event source with a trusted `ensures(0int == 1int)` addition
   was rejected by the independently pinned imported-source digest.
2. A client-prefixed proof_assert macro override, with recomposed active source
   and refreshed mapping client/active hashes, passed composition/hash agreement
   and was rejected by the complete-client pin. Receipt agreement alone was
   therefore not mistaken for source correspondence.
3. A concrete inherited-source anchoring gap was found. A read_bytes fixture
   appended the same trusted false-postcondition function to both AN and AO
   field_event.rs. `audit_inherited_an_sources` accepted the matching copies.
   Its equality check did not anchor AN's local support bytes to the published
   AL/AM manifest. Later AL support audits read AL paths, not these copies.
   Requested fix: invoke pinned AN.assert_frozen_prefix or validate AO support
   overrides directly against the anchored manifest, and anchor inherited lib
   through the reviewed AN module-route check rather than mutable-copy equality.
4. Cargo route review found that checking only package/dependencies/workspace
   permitted additional tables such as a creusot-std path patch. Requested fix:
   exact parsed manifest matching the reviewed small AO manifest, including only
   the three existing empty negative features.
5. Transitive checker loading review found that AN pins AM before import, but
   AM imports AL before a deferred AL digest check, and AL immediately imports
   AI and its native checker. Requested fix: AO checks actual transitive Python
   digests before loading AN. Published ancestor checkers remain unchanged.

The three hardening requests were sent to the checker owner and parent.
Recheck status at this revision: **pending**. These are checker-input closure
issues; no Rust contract weakening or new trusted ownership premise was
requested. Final publication still requires the restored all-target proof,
full correspondence and independent archive audit.

## Independent hardening recheck — closed

Rechecked checker SHA256
`b6d0a002d15c3407891678127659b604e66ece475afe022aa040193e5be21207`.
The three requests above are resolved for the reviewed AO routes. The original
findings remain recorded rather than replaced by an unexplained pass.

The checker now invokes AN.assert_frozen_prefix and anchors the paired local
sources through the AM frozen AL manifest; AN.assert_module_route rechecks the
inherited crate root. Exact parsed Cargo matching rejects unreviewed tables.
Five transitive ancestor checker inputs are digest-checked before importing AN,
in addition to the AN checker pin itself.

Independent in-memory recheck passed the inherited positive-source/module-route
baseline using saved positive active SHA256 `e607bc1c...`. It rejected ten
concrete cases: the original matching AN/AO field_event trusted-falsepost
fixture; a matching inherited lib extra-module route; Cargo path patch; Cargo
target dependency; mutation of each of the five transitive checker files; and
a fresh AO checker import with altered AL checker bytes. For the last case,
instrumented spec_from_file_location recorded no ancestor import before the
rejection, establishing that the check runs before executing changed ancestor
code. The exact new extension still passes the cursor-only/complete-source pin.
No files under any ancestor or proof source were modified for these checks.

The previous external falsepost and refreshed-hash/recomposed macro rejection
results remain applicable: those gates were retained. This is a component
recheck, not the final full CLI/control-suite or archive audit. The live AO
client was intentionally an omitted-survivor semantic control during review,
so it was not passed off as a canonical positive input. Parent must restore and
run the complete gate and independent evidence audit before publication.
No further blocker was found in the bounded Rust architecture or the corrected
checker input routes. Full original architecture remains **NOT ADMITTED**.
