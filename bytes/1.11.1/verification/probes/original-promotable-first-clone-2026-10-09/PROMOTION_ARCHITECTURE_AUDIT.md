# First-promotion architecture audit

Date: 2026-10-09. Static, read-only review of `src/promotion.rs` and its
immediate state, pointer, field-event, physical-borrow, and free interfaces.
No prover or native test was run for this review. This is an architecture
review, not a source-correspondence admission.

## Resource and state flow

The architecture uses an external `PromotionScope` for root ownership. The
root `Bytes` proof value stores only a `RootDescriptor` (pointer, capacity,
expected content, and atomic-model identity). In `RawPhase`, the scope owns the
actual `Recovery`, `PhysicalRegion`, mutable `Perm<ModelAtomicPtr<()>>`, and
`SyncView`. Its validity relates the permission to the actual `Bytes.data`
field, and relates the physical region to the Box allocation and expected
bytes. I did not find a resource getter on `RootDescriptor` or a standalone
weak `Bytes` invariant that manufactures this authority.

On the selected first-clone path, `shallow_clone_vec_checked` allocates the
actual `Shared { buf, cap, ref_cnt: AtomicUsize::new(2) }`, makes the typed
control and physical allocations available through `FullBorrow`, and calls
`owned_pointer::exchange_singleton` with the scope's existing mutable pointer
permission. On success, the same allocation, physical region, and updated
pointer permission move into `SharedPhase`. `State::initialize_pair` consumes
the actual count-field permission and payload, and returns one state with
distinct root and child tickets. `ScopedFieldInvariant::bind` names the actual
`Shared.ref_cnt` field. The root keeps the updated mutable pointer permission;
the child gets a separate `ReadOnlyPointer` binding to its own atomic pointer
field. I found no guessed ticket IDs, fabricated increment, or re-binding of
the updated root history as read-only.

The CAS error arm is marked unreachable in this proof scope. That follows from
the actual singleton history precondition and the strong compare-exchange
failure inequality in the separately audited generic owned-pointer boundary.
It excludes competing writers from this closed client; it does not verify the
native race-loser behavior as unreachable for arbitrary `Bytes` clients.

The normal client order is child cleanup, read through the still-live root
physical capability, then root cleanup. The child cleanup removes its ticket
and cannot be final while the root ticket remains. Root cleanup requires one
remaining ticket, obtains the latest pointer through `get_mut_finish`, then
uses the release/acquire path to recover and free the payload and typed
`Shared` allocation. Completion checks carry the actual buffer/control
pointers, sizes, alignment, and allocation receipts. The public state
observation is resource-free; the fraction/pool projection does not return a
ticket or permission. `logic::any()` fallbacks in this module occur only in
logic functions whose executable callers require a matching phase.

## TCB boundaries and correspondence blocker

The resource/state flow is coherent for this closed normal-return scope, but
the proof module is a shadow, not an implementation that calls the production
API. `from_box_scoped` reconstructs the constructor state; clone and drop use
checked shims; and `read_root` calls `physical_projection::borrow` instead of
the native `AsRef`/`as_slice` path. Its correctness as a refinement therefore
depends on an independent checker binding these helpers to the exact native
constructor, vtables, clone callbacks, read path, `AtomicMut::with_mut`
destructor closure, and cleanup/free effects.

At review time, `generated/correspondence.json` was `not_run` for the active
diagnostic proof session. The separate `generated/native-correspondence.json`
pass checks a real API witness and selected production source/MIR; by its own
scope it makes no mathematical proof claim, and it does not bind
`src/promotion.rs`. In particular, `*_table_reification`,
`*_clone_registration`/`*_drop_registration`, `erased_call::registered3`, and
`erased_call::invoke3` are trusted boundaries. Their contracts relate an
erased native pointer to a checked shim, but the relation must be independently
validated against the actual source/MIR before this shadow is admitted. The
new correspondence checker was still in progress during this review, so this
is an explicit outstanding gate rather than a finding that the checker failed.

Other explicit generic TCBs include the exposed-provenance low-bit tag/clear
roundtrip, `AtomicPtr` creation and permission interpretation, strong CAS/load
and terminal `get_mut` interpretation, exact-field atomic event adapters,
physical borrow/free projection, and typed deallocation receipts. In
particular, `owned_pointer::get_mut_finish` retains only the latest-value and
maximal-timestamp consequences under exclusive access; it does not return a
`SyncView` or assert an Acquire/fence. The counter, reclamation, and allocation
facts used by the proof are then body-proved in the protocol/cleanup path,
subject to those generic field and deallocation boundaries.

The claimed client remains limited to a nonempty Box, one thread with no
competing pointer writer, the first successful promotion, and normal return.
The source/MIR checker records unwind edges and native loser branches, but
those behaviors are not established by this proof client. It also reports
that nested `AtomicMut::with_mut` closure bodies are source-checked while MIR
captures the outer callback invocation; compiler/MIR adequacy, the Rust memory
model, allocator behavior, and pointer provenance remain outside that
checker.

## Review result

I found no contradiction in the internal root/child permission flow or the
nonfinal-then-final cleanup sequence. The remaining admission blocker is the
independent correspondence check for the actual `promotion.rs` helper bodies
and trusted callback/table erasure, plus its controls and the completed proof
receipt. This review does not establish that correspondence or admit the
client.

## Final canonical-capture close-out — 2026-10-09

The previous review accurately described the state at that review time. The
subsequent canonical capture now closes that specific correspondence blocker
for the bounded first-promotion client: `evidence/promotion-canonical-v1.tar.gz`
has SHA-256
`53a500c22c725a259f20377e4da56c9e82bb4fc695400397c55940bf28e8eed1`, records
112/808/0/0 targets/leaves, and carries a zero-status source/native
correspondence receipt. An independent archive reconstruction passed the main
checker, replayed 25/25 main checker controls and 24/24 native source/MIR
controls, and checked the recorded compiled Cargo inputs against both the
generated record and current live Cargo artifacts. Details and path-sensitive
replay notes are in `ROOT_AUDIT.md` and `ROOT_AUDIT.json`.

This closes the evidence gap for the exact client described above, not for the
whole bytes 1.11.1 crate. Its explicit normal-return, nonempty-input scope,
generic TCB boundaries, and exclusions remain as stated in the canonical
receipt and README. The captured proof used one concurrent prover slot but
successful leaves came from Alt-Ergo, Z3, and CVC5.
